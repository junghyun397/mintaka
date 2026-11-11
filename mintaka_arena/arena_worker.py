import argparse
import json
import logging
import os
import secrets
import threading
import time
from dataclasses import dataclass, field
from enum import Enum, auto
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from queue import Queue

from . import arena
from . import binary_manager
from . import worker_manager


class RunState(Enum):
    PREPARING = auto()
    STOPPING = auto()


@dataclass
class Run:
    key: str
    workers: int = 0
    config: arena.Config | None = None
    affinity_queue: Queue | None = None
    state: RunState | None = None
    active: int = 0
    idle_since: float | None = field(default_factory=time.monotonic)


class ArenaWorker(ThreadingHTTPServer):
    daemon_threads = False

    def __init__(self, address, max_workers, affinity_pool=None):
        self.max_workers = max_workers
        self.affinity_pool = affinity_pool
        self.lock = threading.Condition()
        self.run = None
        self.closing = False
        super().__init__(address, ArenaWorkerHandler)

    def server_close(self):
        with self.lock:
            self.closing = True
            self.lock.notify_all()
        super().server_close()

    def service_actions(self):
        with self.lock:
            run = self.run
            if run is None or run.active or run.idle_since is None:
                return
            if time.monotonic() - run.idle_since >= 5:
                self.stop({"key": run.key})

    def status(self):
        return self.max_workers

    def acquire(self, data):
        with self.lock:
            while self.run is not None:
                if not data.get("preempt", False):
                    raise worker_manager.HTTPError(409, "worker is busy")
                self.stop({"key": self.run.key})
            self.run = Run(secrets.token_hex(16))
            return {"key": self.run.key}

    def reserve(self, _):
        with self.lock:
            self.lock.wait_for(lambda: self.run is None or self.closing)
            if self.closing:
                raise worker_manager.HTTPError(503, "worker is shutting down")
            return self.max_workers

    def start(self, data):
        with self.lock:
            run = self.owned_run(data["key"])
            if run.state is not None or run.config is not None:
                raise worker_manager.HTTPError(409, "run cannot be started")

            workers = data["workers"]
            if not isinstance(workers, int) or not 0 < workers <= self.max_workers:
                raise worker_manager.HTTPError(400, f"workers must be between 1 and {self.max_workers}")

            run.workers = workers
            run.state = RunState.PREPARING
            run.active += 1

        try:
            logging.info(f"Arena Worker [{run.key}]: preparing {workers} workers")

            sources = {
                arena.Player[name.upper()]: binary_manager.Source.from_json(source) if isinstance(source, dict) else source
                for name, source in data["sources"].items()
            }

            run.config = binary_manager.build_config(sources, data["settings"])
            run.affinity_queue = arena.create_affinity_queue(run.config, workers, self.affinity_pool)
        except BaseException:
            with self.lock:
                run.state = RunState.STOPPING
            raise
        finally:
            with self.lock:
                run.active -= 1
                if run.state is RunState.PREPARING:
                    run.state = None
                run.idle_since = time.monotonic() if run.state is RunState.STOPPING else None
                self.lock.notify_all()

        with self.lock:
            self.owned_run(run.key)
            if run.state is RunState.STOPPING:
                raise worker_manager.HTTPError(410, "run is stopping")

        logging.info(f"Arena Worker [{run.key}]: started {workers} workers")
        return {"workers": workers}

    def owned_run(self, key):
        if self.run is None or self.run.key != key:
            raise worker_manager.HTTPError(410, "run does not own this worker")
        return self.run

    def play(self, data):
        opening = arena.Opening.from_json(data["opening"])

        with self.lock:
            run = self.owned_run(data["key"])

            if run.state is RunState.STOPPING:
                raise worker_manager.HTTPError(410, "run is stopping")
            if run.config is None or run.state is not None:
                raise worker_manager.HTTPError(409, "run is not accepting games")

            if run.active >= run.workers:
                raise worker_manager.HTTPError(409, "worker allocation exceeded")

            run.active += 1

        try:
            logging.info(f"Arena Play [{run.key}]: opening={opening.sequence}")

            pair = arena.play_pair(run.config, opening, run.affinity_queue)
        except BaseException:
            with self.lock:
                run.state = RunState.STOPPING
            raise
        finally:
            with self.lock:
                run.active -= 1
                run.idle_since = time.monotonic()
                self.lock.notify_all()

        logging.info(f"Arena Plied [{run.key}]: opening={opening.sequence}, "
                     f"base={pair.snapshots[arena.Player.BASE].history}, "
                     f"target={pair.snapshots[arena.Player.TARGET].history}")

        return pair.to_json()

    def stop(self, data):
        with self.lock:
            run = self.owned_run(data["key"])
            run.state = RunState.STOPPING
            self.lock.wait_for(lambda: not run.active)
            if self.run is run:
                self.run = None
                self.lock.notify_all()

        logging.info(f"Arena Worker [{run.key}]: stopped")
        return {"stopped": True}


class ArenaWorkerHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        logging.debug(f"Arena Worker [{self.address_string()}]: {format % args}")

    def log_error(self, format, *args):
        logging.error(f"Arena Worker [{self.address_string()}]: {format % args}")

    def reply(self, status, data):
        body = json.dumps(data).encode()
        try:
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            pass

    def do_GET(self):
        self.dispatch({"/status": self.server.status})

    def do_POST(self):
        data = json.loads(self.rfile.read(int(self.headers["Content-Length"])))

        self.dispatch(
            {
                "/acquire": self.server.acquire,
                "/reserve": self.server.reserve,
                "/start": self.server.start,
                "/play": self.server.play,
                "/stop": self.server.stop,
            },
            data,
        )

    def dispatch(self, handlers, *args):
        if self.path not in handlers:
            self.reply(404, {"error": "unknown endpoint"})
            return

        try:
            result = handlers[self.path](*args)
        except worker_manager.HTTPError as error:
            logging.warning(f"Arena worker [{self.address_string()}] {self.path}: {error}")
            self.reply(error.status, {"error": str(error)})
        except Exception as error:
            logging.exception(f"Arena worker [{self.address_string()}] {self.path} failed")
            self.reply(500, {"error": str(error)})
        else:
            self.reply(200, result)


def main():
    parser = argparse.ArgumentParser()

    parser.add_argument("--address", type=str, default="0.0.0.0")

    group = parser.add_mutually_exclusive_group()
    group.add_argument("--max-workers", type=int)
    group.add_argument("--affinity-pool", type=int, nargs="+")

    parser.add_argument("--port", type=int, default=8051)
    parser.add_argument("--log-level", choices=["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"], default="INFO")

    args = parser.parse_args()
    arena.configure_logging(args.log_level)

    max_workers = args.max_workers if args.max_workers is not None \
        else len(args.affinity_pool) if args.affinity_pool is not None \
        else (os.cpu_count() or 1)

    with ArenaWorker((args.address, args.port), max_workers, args.affinity_pool) as worker:
        logging.info(f"Arena worker: address={args.address}, port={args.port}, max-workers={max_workers}")
        try:
            worker.serve_forever()
        except KeyboardInterrupt:
            pass
        finally:
            logging.info("Arena worker: shutting down")
            with worker.lock:
                if worker.run is not None:
                    worker.run.state = RunState.STOPPING


if __name__ == "__main__":
    main()
