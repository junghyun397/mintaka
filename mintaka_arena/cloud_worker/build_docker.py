import argparse
import shutil
import subprocess
from pathlib import Path

from .. import arena


def main():
    parser = argparse.ArgumentParser()

    for player in arena.Player:
        parser.add_argument(f"--{player}-ref", required=True)
        parser.add_argument(f"--{player}-patch")

    parser.add_argument("--target-cpu", default="znver5")
    parser.add_argument("--platform", choices=["linux/amd64", "linux/arm64"], default="linux/amd64")
    args = parser.parse_args()

    command = [
        "docker", "build", f"--platform={args.platform}",
        "-f", "mintaka_arena/cloud_worker/Dockerfile",
        "--output", "type=tar,dest=-",
    ]

    for player in arena.Player:
        if patch := getattr(args, f"{player}_patch"):
            source = Path(patch)

            destination = Path("mintaka_arena/cloud_worker/transfer/patches") / str(player) / source.name
            destination.parent.mkdir(parents=True, exist_ok=True)

            if source.resolve() != destination.resolve():
                shutil.copyfile(source, destination)

            setattr(args, f"{player}_patch", str(destination))

    for name, value in vars(args).items():
        if value:
            command.extend(["--build-arg", f"{name.upper()}={value}"])

    subprocess.run([*command, "."], check=True)


if __name__ == "__main__":
    main()
