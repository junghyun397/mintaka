import argparse
import json
import logging
import threading
from dataclasses import dataclass
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import arena
import binary_manager
import worker_manager


@dataclass
class Run:
    run_id: str
    workers: int
    config: arena.Config | None = None
    preparing: bool = True
    stopping: bool = False
    active: int = 0


class ArenaWorker(ThreadingHTTPServer):
    daemon_threads = False

    def __init__(self, address, max_concurrency):
        super().__init__(address, ArenaWorkerHandler)
        self.max_concurrency = max_concurrency
        self.lock = threading.Condition()
        self.run = None

    def status(self):
        with self.lock:
            if self.run is not None:
                raise worker_manager.HTTPError(409, "worker is busy")
            return self.max_concurrency

    def start(self, data):
        run_id, workers = data["run_id"], data["workers"]

        if workers > self.max_concurrency:
            raise worker_manager.HTTPError(400, "allocation > capacity")

        sources = {name: binary_manager.Source.from_json(data["sources"][name]) for name in ("base", "target")}

        with self.lock:
            if self.run is not None:
                raise worker_manager.HTTPError(409, "worker is busy")
            run = Run(run_id, workers)
            self.run = run

        try:
            logging.info(f"Arena Worker [{run_id}]: preparing {workers} workers")

            binary_manager.fetch_master()
            binary_manager.save_sources(sources)

            config = binary_manager.build_config(sources, data["settings"])
        except BaseException:
            with self.lock:
                run.preparing = False
                self.run = None
                self.lock.notify_all()
            raise

        with self.lock:
            run.config = config
            run.preparing = False
            self.lock.notify_all()
            if run.stopping:
                raise worker_manager.HTTPError(409, "run is stopping")

        logging.info(f"Arena Worker [{run_id}]: started {workers} workers")
        return {"workers": workers}

    def owned_run(self, run_id):
        if self.run is None or self.run.run_id != run_id:
            raise worker_manager.HTTPError(409, "run does not own this worker")
        return self.run

    def play(self, data):
        opening = arena.Opening.from_json(data["opening"])

        with self.lock:
            run = self.owned_run(data["run_id"])

            if run.preparing or run.stopping:
                raise worker_manager.HTTPError(409, "run is not accepting games")

            if run.active >= run.workers:
                raise worker_manager.HTTPError(409, "worker allocation exceeded")

            run.active += 1

        try:
            logging.info(f"Arena Play [{run.run_id}]: opening={opening.sequence}")

            pair = arena.play_pair(
                run.config.path_params_resource, run.config.args.draw_in, opening,
                log_prefix_filter=tuple(run.config.args.log_prefix_filter or ()),
            )
        except BaseException:
            with self.lock:
                run.stopping = True
            raise
        finally:
            with self.lock:
                run.active -= 1
                self.lock.notify_all()

        logging.info(f"Arena Plied [{run.run_id}]: opening={opening.sequence}, "
                     f"base={pair.snapshots[arena.Player.BASE].history}, "
                     f"target={pair.snapshots[arena.Player.TARGET].history}")

        return pair.to_json()

    def stop(self, data):
        with self.lock:
            run = self.owned_run(data["run_id"])
            run.stopping = True
            while run.preparing or run.active:
                self.lock.wait()
            if self.run is run:
                self.run = None
            self.lock.notify_all()

        logging.info(f"Arena Worker [{run.run_id}]: stopped")
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
        self.dispatch(
            {
                "/status": self.server.status,
            }
        )

    def do_POST(self):
        data = json.loads(self.rfile.read(int(self.headers["Content-Length"])))

        self.dispatch(
            {
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
    parser.add_argument("--port", type=int, default=8095)
    parser.add_argument("--max-concurrency", type=int, default=6)
    parser.add_argument("--log-level", choices=["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"], default="INFO")

    args = parser.parse_args()
    arena.configure_logging(args.log_level)

    with ArenaWorker((args.address, args.port), args.max_concurrency) as worker:
        logging.info(f"Arena worker: address={args.address}, port={args.port}, max-concurrency={args.max_concurrency}")
        try:
            worker.serve_forever()
        except KeyboardInterrupt:
            pass
        finally:
            logging.info("Arena worker: shutting down")
            with worker.lock:
                if worker.run is not None:
                    worker.run.stopping = True


if __name__ == "__main__":
    main()
