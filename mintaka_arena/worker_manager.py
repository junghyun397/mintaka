import json
import queue
import secrets
import signal
import sys
import threading
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
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

    def play(self, opening_no, opening):
        return arena.PairResult.from_json(request_json(self.address, "/play", {
            "run_id": self.run_id,
            "opening_no": opening_no,
            "opening": opening.to_json(),
        }))

    def stop(self):
        try:
            request_json(self.address, "/stop", {"run_id": self.run_id})
        except HTTPError as error:
            if error.status != 409:
                raise


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
        self.remotes = []
        self.available = queue.Queue()
        self.lock = threading.Lock()
        self.executor = None
        self.failure = None

    def __enter__(self):
        try:
            self.prepare()
            self.executor = ThreadPoolExecutor(max_workers=self.config.args.concurrency)
        except BaseException:
            self.__exit__(*sys.exc_info())
            raise
        return self

    def prepare(self):
        args = self.config.args
        remaining = args.concurrency
        sources = binary_manager.prepare_sources(args) if args.worker_addresses else None
        settings = {name: getattr(args, name) for name in arena.GAME_SETTINGS}

        for address in args.worker_addresses or ["local"]:
            if remaining == 0:
                break

            if address == "local":
                count = remaining
                config = binary_manager.build_config(sources, settings) if sources else self.config
                play = partial(arena.play_pair, config)
            else:
                try:
                    capacity = request_json(address, "/status", timeout=5)
                except (HTTPError, OSError) as error:
                    print(f"Arena skipping {address}: {error}", flush=True)
                    continue

                count = min(remaining, capacity)
                remote = RemoteWorker(address, self.run_id)
                self.remotes.append(remote)
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
                    self.remotes.remove(remote)
                    print(f"Arena skipping {address}: {error}", flush=True)
                    continue
                play = remote.play

            for _ in range(count):
                self.available.put(play)
            remaining -= count
            if args.worker_addresses:
                print(f"Arena workers: {address}={count}", flush=True)

        if remaining:
            raise RuntimeError(f"insufficient arena workers: requested {args.concurrency}, missing {remaining}")

    def submit(self, opening_no: int, opening: arena.Opening):
        with self.lock:
            if self.failure is not None:
                raise RuntimeError("arena worker failed") from self.failure
            return self.executor.submit(self.play, opening_no, opening)

    def play(self, opening_no, opening):
        with self.lock:
            if self.failure is not None:
                raise RuntimeError("arena worker failed") from self.failure
        worker = self.available.get()
        try:
            return worker(opening_no, opening)
        except BaseException as error:
            with self.lock:
                if self.failure is None:
                    self.failure = error
            raise
        finally:
            self.available.put(worker)

    def __exit__(self, exc_type, exc_value, traceback):
        cleanup_error = None
        with finish_on_interrupt():
            if self.executor is not None:
                self.executor.shutdown(wait=True, cancel_futures=True)
            for remote in self.remotes:
                try:
                    remote.stop()
                except Exception as error:
                    print(f"Arena stop failed for {remote.address}: {error}", file=sys.stderr, flush=True)
                    cleanup_error = cleanup_error or error
        if exc_type is None:
            if self.failure is not None:
                raise RuntimeError("arena worker failed") from self.failure
            if cleanup_error is not None:
                raise cleanup_error
