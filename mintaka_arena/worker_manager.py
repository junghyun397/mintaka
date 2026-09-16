import json
import logging
import secrets
import signal
import sys
import threading
import urllib.error
import urllib.request
from collections.abc import Callable, Iterator
from concurrent.futures import FIRST_COMPLETED, ThreadPoolExecutor, wait
from contextlib import contextmanager
from dataclasses import dataclass
from functools import partial

import arena
import binary_manager


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
    run_id: str

    def play(self, opening):
        return arena.PairResult.from_json(request_json(self.address, "/play", {
            "run_id": self.run_id,
            "opening": opening.to_json(),
        }))

    def stop(self, timeout=None):
        try:
            request_json(self.address, "/stop", {"run_id": self.run_id}, timeout=timeout)
        except HTTPError as error:
            if error.status != 409:
                raise


@dataclass
class Worker:
    concurrency: int
    play: Callable[[arena.Opening], arena.PairResult]
    remote: RemoteWorker | None = None
    active: int = 0


@contextmanager
def finish_on_interrupt():
    if threading.current_thread() is not threading.main_thread():
        yield
        return

    interrupted = False

    def interrupt(signum, frame):
        nonlocal interrupted
        interrupted = True

    previous = signal.signal(signal.SIGINT, interrupt)
    try:
        yield
    finally:
        signal.signal(signal.SIGINT, previous)
    if interrupted:
        raise KeyboardInterrupt


class WorkerManager:
    def __init__(self, config: arena.Config):
        self.config = config
        self.run_id = secrets.token_hex(16)
        self.workers: list[Worker] = []
        self.lock = threading.Condition()
        self.executor = None
        self.failure = None

    @property
    def concurrency(self) -> int:
        with self.lock:
            return sum(worker.concurrency for worker in self.workers)

    def __enter__(self):
        try:
            self.prepare()
            self.executor = ThreadPoolExecutor(max_workers=self.concurrency)
        except BaseException:
            self.__exit__(*sys.exc_info())
            raise
        return self

    def prepare(self):
        remaining = self.config.args.concurrency
        sources = binary_manager.prepare_sources(self.config.args) if self.config.args.worker_addresses else None
        settings = {name: getattr(self.config.args, name) for name in arena.GAME_SETTINGS}
        addresses = list(dict.fromkeys(self.config.args.worker_addresses or ["local"]))

        with ThreadPoolExecutor(max_workers=len(addresses)) as executor:
            capacities = list(executor.map(self.capacity, addresses))
            starts = []
            for address, capacity in zip(addresses, capacities):
                if remaining == 0:
                    break
                count = min(remaining, capacity)
                if count <= 0:
                    continue
                starts.append(executor.submit(self.start_worker, address, count, sources, settings))
                remaining -= count

            for future in starts:
                future.result()

        remaining = self.config.args.concurrency - self.concurrency
        if remaining:
            raise RuntimeError(f"insufficient arena workers: requested {self.config.args.concurrency}, missing {remaining}")

    def capacity(self, address):
        if address == "local":
            return self.config.args.concurrency
        try:
            return request_json(address, "/status", timeout=5)
        except (HTTPError, OSError) as error:
            logging.warning(f"Arena skipping {address}: {error}")
            return 0

    def start_worker(self, address, count, sources, settings):
        remote = None
        if address == "local":
            config = binary_manager.build_config(sources, settings) if sources else self.config
            play = partial(
                arena.play_pair, config.path_params_resource, config.args.draw_in,
                log_prefix_filter=tuple(config.args.log_prefix_filter or ()),
            )
        else:
            remote = RemoteWorker(address, self.run_id)
            play = remote.play

        worker = Worker(count, play, remote)
        with self.lock:
            self.workers.append(worker)
        if remote is not None:
            try:
                request_json(address, "/start", {
                    "run_id": self.run_id,
                    "workers": count,
                    "sources": {name: source.to_json() for name, source in sources.items()},
                    "settings": settings,
                })
            except HTTPError as error:
                if error.status != 409:
                    raise
                with self.lock:
                    self.workers.remove(worker)
                logging.warning(f"Arena skipping {address}: {error}")
                return

        if self.config.args.worker_addresses:
            logging.info(f"Arena workers: {address}={count}")

    def submit(self, opening: arena.Opening):
        with self.lock:
            if self.failure is not None:
                raise RuntimeError("arena worker failed") from self.failure
            return self.executor.submit(self.play, opening)

    def results(self, openings: list[arena.Opening]) -> Iterator[arena.PairResult]:
        pending = set()
        next_opening = 0

        try:
            while pending or next_opening < self.config.args.max_openings:
                while next_opening < self.config.args.max_openings and len(pending) < self.concurrency:
                    pending.add(self.submit(openings[next_opening]))
                    next_opening += 1

                done, _ = wait(pending, return_when=FIRST_COMPLETED)
                future = done.pop()
                pending.remove(future)
                pair = future.result()
                if pair is not None:
                    yield pair
        finally:
            for future in pending:
                future.cancel()

    def play(self, opening: arena.Opening) -> arena.PairResult | None:
        with self.lock:
            while True:
                if self.failure is not None:
                    raise RuntimeError("arena worker failed") from self.failure
                worker = next((worker for worker in self.workers if worker.active < worker.concurrency), None)
                if worker is not None:
                    worker.active += 1
                    break
                self.lock.wait()

        try:
            return worker.play(opening)
        except BaseException as error:
            with self.lock:
                if worker.remote is None or not isinstance(error, Exception):
                    self.failure = self.failure or error
                    raise
                if worker.concurrency:
                    removed = worker.concurrency
                    worker.concurrency = 0
                    logging.warning(f"Arena disabled {worker.remote.address}: "
                                    f"removed concurrency={removed}, adjusted concurrency={self.concurrency}: {error}")
                if self.concurrency == 0:
                    self.failure = self.failure or error
                    raise RuntimeError("no arena workers remaining") from error
            return None
        finally:
            with self.lock:
                worker.active -= 1
                self.lock.notify_all()

    def __exit__(self, exc_type, exc_value, traceback):
        cleanup_error = None
        with finish_on_interrupt():
            if self.executor is not None:
                self.executor.shutdown(wait=True, cancel_futures=True)
            for worker in self.workers:
                if worker.remote is None:
                    continue
                try:
                    worker.remote.stop(timeout=5 if worker.concurrency == 0 else None)
                except Exception as error:
                    if worker.concurrency == 0:
                        logging.warning(f"Arena stop failed for {worker.remote.address}: {error}")
                    else:
                        logging.error(f"Arena stop failed for {worker.remote.address}: {error}")
                        cleanup_error = cleanup_error or error
        if exc_type is None:
            if self.failure is not None:
                raise RuntimeError("arena worker failed") from self.failure
            if cleanup_error is not None:
                raise cleanup_error
