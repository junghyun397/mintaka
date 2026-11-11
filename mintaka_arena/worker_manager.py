import json
import logging
import signal
import sys
import threading
import time
import urllib.error
import urllib.request
from collections import deque
from collections.abc import Callable, Iterator
from concurrent.futures import FIRST_COMPLETED, Future, ThreadPoolExecutor, wait
from contextlib import contextmanager
from dataclasses import dataclass
from functools import partial

from . import arena
from . import binary_manager


class HTTPError(RuntimeError):
    def __init__(self, status, message):
        super().__init__(message)
        self.status = status


def request_json(address, path, data=None, timeout=None):
    request = urllib.request.Request(
        address.rstrip("/") + path,
        data=json.dumps(data).encode() if data is not None else None,
        headers={"Content-Type": "application/json"},
    )
    try:
        response = urllib.request.urlopen(request, timeout=timeout)
    except urllib.error.HTTPError as error:
        raise HTTPError(error.code, f"{address}{path}: {error.read().decode(errors='replace')}") from error
    with response:
        return json.load(response)


@dataclass
class RemoteWorker:
    address: str
    key: str

    @classmethod
    def acquire(cls, address, timeout=None, preempt=False):
        allocation = request_json(address, "/acquire", {"preempt": preempt}, timeout=timeout)
        return cls(address, allocation["key"])

    def start(self, count, sources, settings, timeout=None):
        request_json(self.address, "/start", {
            "key": self.key,
            "workers": count,
            "sources": {
                str(player): source.to_json() if isinstance(source, binary_manager.Source) else source
                for player, source in sources.items()
            },
            "settings": settings,
        }, timeout=timeout)

    def play(self, opening, timeout=None):
        return arena.PairResult.from_json(request_json(self.address, "/play", {
            "key": self.key,
            "opening": opening.to_json(),
        }, timeout=timeout))

    def stop(self, timeout=None):
        try:
            request_json(self.address, "/stop", {"key": self.key}, timeout=timeout)
        except HTTPError as error:
            if error.status not in (409, 410):
                raise


@dataclass
class Worker:
    concurrency: int
    play: Callable[[arena.Opening], arena.PairResult] | None
    remote: RemoteWorker | None = None
    active: int = 0
    options: arena.WorkerOptions | None = None
    paused: bool = False


PendingGames = dict[Future[arena.PairResult], tuple[Worker, int]]


@contextmanager
def finish_on_interrupt():
    if threading.current_thread() is not threading.main_thread():
        yield
        return

    interrupted = False

    def interrupt(_, __):
        nonlocal interrupted
        interrupted = True

    previous = {
        signum: signal.signal(signum, interrupt)
        for signum in (signal.SIGINT, signal.SIGTERM)
    }

    try:
        yield
    finally:
        for signum, handler in previous.items():
            signal.signal(signum, handler)
    if interrupted:
        raise KeyboardInterrupt


class WorkerManager:
    def __init__(self, config: arena.Config):
        self.config = config
        self.workers: list[Worker] = []
        self.lock = threading.Lock()
        self.executor = None
        self.retry_at = None

    @property
    def concurrency(self) -> int:
        with self.lock:
            return sum(worker.concurrency for worker in self.workers)

    def __enter__(self):
        try:
            self.prepare()
            self.executor = ThreadPoolExecutor(max_workers=sum(worker.options.count for worker in self.workers))
        except BaseException:
            self.__exit__(*sys.exc_info())
            raise
        return self

    def prepare(self):
        args = self.config.args
        requests = {worker.address: worker for worker in args.workers}
        requested = sum(worker.count for worker in requests.values())
        sources = binary_manager.prepare_sources(args)
        settings = {name: getattr(args, name) for name in arena.GAME_SETTINGS}
        self.sources = sources
        self.settings = settings

        with ThreadPoolExecutor(max_workers=len(requests)) as executor:
            starts = [
                executor.submit(self.start_worker, options, sources, settings)
                for options in requests.values()
            ]

            for future in starts:
                future.result()

        concurrency = self.concurrency
        if concurrency == 0 and not any(worker.paused for worker in self.workers):
            raise RuntimeError("no arena workers remaining")
        if concurrency < requested:
            logging.warning(f"Arena concurrency: requested={requested}, available={concurrency}")

    def start_worker(self, options, sources, settings):
        address, count = options.address, options.count
        if address == "local":
            config = binary_manager.build_config(sources, settings)
            affinity_queue = arena.create_affinity_queue(config, count)
            worker = Worker(count, partial(arena.play_pair, config, affinity_queue=affinity_queue), options=options)
            with self.lock:
                self.workers.append(worker)
            logging.info(f"Arena workers: {address}={count}")
            return

        try:
            remote = RemoteWorker.acquire(address, timeout=self.config.args.timeout_start, preempt=options.preempt)
        except Exception as error:
            if isinstance(error, HTTPError) and error.status == 409 and self.config.args.wait is not None:
                with self.lock:
                    self.workers.append(Worker(0, None, options=options, paused=True))
                logging.warning(f"Arena waiting {address}")
            else:
                logging.warning(f"Arena skipping {address}: {error}")
            return

        worker = Worker(count, partial(remote.play, timeout=self.config.args.timeout_play), remote, options=options)
        with self.lock:
            self.workers.append(worker)

        try:
            remote.start(count, sources, settings, timeout=self.config.args.timeout_start)
        except Exception as error:
            self.worker_failed(worker, error)
            return

        logging.info(f"Arena workers: {address}={count}")

    def disable_worker(self, worker, error):
        with self.lock:
            if not worker.concurrency:
                return
            removed = worker.concurrency
            worker.concurrency = 0

        logging.warning(f"Arena disabled {worker.remote.address}: "
                        f"removed concurrency={removed}, adjusted concurrency={self.concurrency}: {error}")

    def worker_failed(self, worker, error):
        if isinstance(error, HTTPError) and error.status == 410:
            with self.lock:
                if worker.paused:
                    return True
                worker.concurrency = 0
                worker.paused = True
            logging.warning(f"Arena lost {worker.remote.address}")
            return True

        self.disable_worker(worker, error)
        return False

    def resume_workers(self):
        for worker in self.workers:
            if not worker.paused or worker.active:
                continue
            try:
                remote = RemoteWorker.acquire(worker.options.address, timeout=self.config.args.timeout_start)
            except Exception as error:
                if isinstance(error, HTTPError) and error.status == 409:
                    continue
                worker.paused = False
                logging.warning(f"Arena skipping {worker.options.address}: {error}")
                continue

            worker.remote = remote
            worker.play = partial(remote.play, timeout=self.config.args.timeout_play)
            worker.paused = False
            worker.concurrency = worker.options.count
            try:
                remote.start(worker.concurrency, self.sources, self.settings, timeout=self.config.args.timeout_start)
            except Exception as error:
                self.worker_failed(worker, error)
                continue
            logging.warning(f"Arena recovered {remote.address}: concurrency={worker.concurrency}")

    def submit_openings(self, openings, pending: PendingGames, retries, next_opening):
        with self.lock:
            for worker in self.workers:
                while worker.active < worker.concurrency:
                    if retries:
                        index = retries.popleft()
                    elif next_opening < self.config.args.max_openings:
                        index = next_opening
                        next_opening += 1
                    else:
                        break
                    future = self.executor.submit(worker.play, openings[index])
                    pending[future] = worker, index
                    worker.active += 1
        return next_opening

    def poll_workers(self, pending: PendingGames):
        if self.retry_at is not None and time.monotonic() >= self.retry_at:
            self.resume_workers()
            self.retry_at = None
            return ()

        paused = any(worker.paused for worker in self.workers)
        if paused and self.retry_at is None:
            self.retry_at = time.monotonic() + (self.config.args.wait if self.config.args.wait is not None else 10)
        elif not paused:
            self.retry_at = None

        if not pending and not paused:
            raise RuntimeError("no arena workers remaining")

        timeout = max(0, self.retry_at - time.monotonic()) if self.retry_at is not None else None
        if not pending:
            time.sleep(timeout)
            return ()

        done, _ = wait(pending, timeout=timeout, return_when=FIRST_COMPLETED)
        return done

    def process_completed_games(
            self, done, pending: PendingGames, completed: dict[int, arena.PairResult | None], retries,
    ):
        for future in done:
            worker, opening_index = pending.pop(future)
            worker.active -= 1
            try:
                pair = future.result()
            except Exception as error:
                if worker.remote is None:
                    raise
                if self.worker_failed(worker, error):
                    retries.append(opening_index)
                    continue
                pair = None

            if pair is not None:
                address = worker.remote.address if worker.remote is not None else "local"
                logging.debug(f"Arena result received: worker={address}, opening={opening_index + 1}")
            completed[opening_index] = pair

    def results(self, openings: list[arena.Opening]) -> Iterator[arena.PairResult]:
        pending: PendingGames = {}
        completed: dict[int, arena.PairResult | None] = {}
        retries = deque()
        next_opening = 0
        next_result = 0

        try:
            while next_result < self.config.args.max_openings:
                next_opening = self.submit_openings(openings, pending, retries, next_opening)
                done = self.poll_workers(pending)
                self.process_completed_games(done, pending, completed, retries)

                while next_result in completed:
                    pair = completed.pop(next_result)
                    next_result += 1
                    if pair is not None:
                        yield pair
        finally:
            self.retry_at = None
            for future in pending:
                future.cancel()

    def stop_worker(self, worker):
        if worker.remote is None:
            return None

        try:
            worker.remote.stop(timeout=5 if worker.concurrency == 0 else None)
        except Exception as error:
            if worker.concurrency == 0:
                logging.warning(f"Arena stop failed for {worker.remote.address}: {error}")
                return None
            logging.error(f"Arena stop failed for {worker.remote.address}: {error}")
            return error

    @finish_on_interrupt()
    def __exit__(self, exc_type, exc_value, traceback):
        if self.executor is not None:
            self.executor.shutdown(wait=True, cancel_futures=True)

        cleanup_error = None
        for worker in self.workers:
            error = self.stop_worker(worker)
            cleanup_error = cleanup_error or error

        if exc_type is None and cleanup_error is not None:
            raise cleanup_error
