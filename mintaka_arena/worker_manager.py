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

    def play(self, opening, timeout=None):
        return arena.PairResult.from_json(request_json(self.address, "/play", {
            "run_id": self.run_id,
            "opening": opening.to_json(),
        }, timeout=timeout))

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
        self.run_id = secrets.token_hex(16)
        self.workers: list[Worker] = []
        self.lock = threading.Lock()
        self.executor = None

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
        args = self.config.args
        remaining = args.concurrency
        addresses = list(dict.fromkeys(args.worker_addresses or ["local"]))
        sources = binary_manager.prepare_sources(args, cache_only=args.cache_only)
        settings = {name: getattr(args, name) for name in arena.GAME_SETTINGS}

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

        concurrency = self.concurrency
        if concurrency == 0:
            raise RuntimeError("no arena workers remaining")
        if concurrency < args.concurrency:
            logging.warning(f"Arena concurrency: requested={args.concurrency}, available={concurrency}")

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
            config = binary_manager.build_config(sources, settings)
            affinity_queue = arena.create_affinity_queue(config, count)
            play = partial(arena.play_pair, config, affinity_queue=affinity_queue)
        else:
            remote = RemoteWorker(address, self.run_id)
            play = partial(remote.play, timeout=self.config.args.timeout_play)

        worker = Worker(count, play, remote)
        with self.lock:
            self.workers.append(worker)
        if remote is not None:
            try:
                request_json(address, "/start", {
                    "run_id": self.run_id,
                    "workers": count,
                    "sources": {
                        str(player): source.to_json() if isinstance(source, binary_manager.Source) else source
                        for player, source in sources.items()
                    },
                    "settings": settings,
                }, timeout=self.config.args.timeout_start)
            except Exception as error:
                if isinstance(error, HTTPError) and error.status == 409:
                    with self.lock:
                        self.workers.remove(worker)
                    logging.warning(f"Arena skipping {address}: {error}")
                else:
                    self.disable_worker(worker, error)
                return

        if self.config.args.worker_addresses:
            logging.info(f"Arena workers: {address}={count}")

    def disable_worker(self, worker, error):
        with self.lock:
            if not worker.concurrency:
                return
            removed = worker.concurrency
            worker.concurrency = 0

        logging.warning(f"Arena disabled {worker.remote.address}: "
                        f"removed concurrency={removed}, adjusted concurrency={self.concurrency}: {error}")

    def results(self, openings: list[arena.Opening]) -> Iterator[arena.PairResult]:
        pending = {}
        next_opening = 0

        try:
            while True:
                with self.lock:
                    for worker in self.workers:
                        while worker.active < worker.concurrency and next_opening < self.config.args.max_openings:
                            future = self.executor.submit(self.play, worker, openings[next_opening])
                            pending[future] = worker
                            worker.active += 1
                            next_opening += 1

                if not pending:
                    if self.concurrency == 0:
                        raise RuntimeError("no arena workers remaining")
                    return

                done, _ = wait(pending, return_when=FIRST_COMPLETED)
                for future in done:
                    worker = pending.pop(future)
                    worker.active -= 1
                    pair = future.result()
                    if pair is not None:
                        yield pair
        finally:
            for future in pending:
                future.cancel()

    def play(self, worker: Worker, opening: arena.Opening) -> arena.PairResult | None:
        try:
            return worker.play(opening)
        except Exception as error:
            if worker.remote is None:
                raise
            self.disable_worker(worker, error)

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
        if exc_type is None and cleanup_error is not None:
            raise cleanup_error
