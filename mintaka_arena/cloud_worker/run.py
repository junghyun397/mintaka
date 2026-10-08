import argparse
import json
import os
import signal
import subprocess
import sys
import urllib.request
from contextlib import contextmanager
from pathlib import Path

from .. import worker_manager


@contextmanager
def cloud_workers(args):
    archive = args.cache.resolve(strict=True)
    with urllib.request.urlopen("https://api.ipify.org", timeout=15) as response:
        controller_ip = response.read().decode().strip()

    variables = {
        "project": args.project,
        "region": args.region, "machine_type": args.machine_type, "concurrency": args.concurrency,
        "name": "mintaka", "archive": str(archive),
        "controller_ip": controller_ip,
    }
    environment = dict(os.environ, TF_IN_AUTOMATION="1", TF_INPUT="0")
    environment.update({f"TF_VAR_{name}": str(value) for name, value in variables.items()})

    def terraform(*arguments, capture_output=False):
        environment["GOOGLE_OAUTH_ACCESS_TOKEN"] = subprocess.check_output(
            ["gcloud", "auth", "print-access-token"], text=True,
        ).strip()

        return subprocess.run(
            ["terraform", *arguments],
            cwd=Path(__file__).resolve().parent, check=True, env=environment, text=True,
            stdout=subprocess.PIPE if capture_output else None,
        ).stdout

    terraform("init")

    try:
        with worker_manager.finish_on_interrupt():
            terraform("apply", "-auto-approve")
        yield json.loads(terraform("output", "-json", "worker_addresses", capture_output=True))
    finally:
        with worker_manager.finish_on_interrupt():
            terraform("destroy", "-auto-approve")


def main():
    parser = argparse.ArgumentParser(allow_abbrev=False)
    parser.add_argument("--project", required=True)
    parser.add_argument("--cache", type=Path, required=True)
    parser.add_argument("--region", default="us-central1")
    parser.add_argument("--machine-type", default="c4d-highcpu-8")
    parser.add_argument("--concurrency", type=int)
    parser.add_argument("--suit", choices=["elo", "sprt"], required=True)
    parser.add_argument("params", nargs=argparse.REMAINDER)
    args = parser.parse_args()

    params = args.params
    if params[:1] == ["--"]:
        params = params[1:]

    signal.signal(signal.SIGTERM, signal.default_int_handler)

    with cloud_workers(args) as addresses:
        concurrency = sum(worker_manager.request_json(address, "/status", timeout=5) for address in addresses)
        subprocess.run([
            sys.executable, "-u", "-m", f"mintaka_arena.{args.suit}",
            "--cache-only",
            "--concurrency", str(concurrency),
            *params,
            "--worker-addresses", *addresses,
        ], check=True)


if __name__ == "__main__":
    main()
