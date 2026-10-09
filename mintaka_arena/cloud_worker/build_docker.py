import argparse
import subprocess


def main():
    parser = argparse.ArgumentParser()

    parser.add_argument("--rule", choices=["Renju", "Gomoku", "Freestyle"], default="Renju")

    for player in ("base", "target"):
        parser.add_argument(
            f"--{player}-ref", default="origin/master",
            help="REMOTE/BRANCH[@COMMIT] (default: origin/master)",
        )

    parser.add_argument("--target-cpu", default="znver5")
    parser.add_argument("--platform", choices=["linux/amd64", "linux/arm64"], default="linux/amd64")
    args = vars(parser.parse_args())
    platform = args.pop("platform")

    command = [
        "docker", "build", f"--platform={platform}",
        "-f", "mintaka_arena/cloud_worker/Dockerfile",
        "--output", "type=tar,dest=-",
        *[f"--build-arg={name.upper()}={value}" for name, value in args.items() if value],
        ".",
    ]

    subprocess.run(command, check=True)


if __name__ == "__main__":
    main()
