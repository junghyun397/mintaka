import argparse
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone

class Config:
    def __init__(self, params, num_arenas):
        self.args = params
        self.num_arenas = num_arenas

    def engine_name(self, target_value) -> str:
        return f"mintaka_text_protocol_{self.args.target_param}_{target_value}"

@dataclass
class ArenaResult:
    baseline_elo: float
    target_elo: float

def datetime_prefix() -> str:
    return datetime.now(timezone.utc).strftime("[%Y-%m-%dT%H:%M:%SZ] ")

def arena_prefix(config, arena_no, optimal_value, current_value) -> str:
    return f"[{arena_no + 1}/{config.num_arenas}] [{config.args.target_param}:{optimal_value}v{current_value}] "

def compile_engine(config, target_value):
    env = os.environ.copy()
    env[config.args.target_param] = str(target_value)

    subprocess.run(
        ["cargo", "build",
         "--profile", config.args.compile_profile,
         "--package", "mintaka_interface",
         "--bin", "mintaka_text_protocol"],
        env=env,
        stdin=None,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.STDOUT,
        check=True
    )

    engine_name = config.engine_name(target_value)

    print(f"{datetime_prefix()}compiled {engine_name}")

    subprocess.run(["mv",
                    f"target/release/mintaka_text_protocol",
                    f"target/release/{engine_name}"])

def play_arena(config, arena_no, baseline_value, target_value, baseline_elo, target_elo) -> ArenaResult:
    with subprocess.Popen(
        [sys.executable, "mintaka_trainer/arena.py",
         "--no-datetime-prefix",
         "--log-prefix-filter",
         "--a-time", str(config.args.time[0]), str(config.args.time[1]), str(config.args.time[2]),
         "--b-time", str(config.args.time[0]), str(config.args.time[1]), str(config.args.time[2]),
         *(["--openings", config.args.openings] if config.args.openings is not None else []),
         "--elo-k-factor", str(16.0),
         "--num-games", str(config.args.num_games),
         "--a-path", f"target/release/{config.engine_name(baseline_value)}",
         "--b-path", f"target/release/{config.engine_name(target_value)}",
         "--a-params", config.args.engine_params,
         "--b-params", config.args.engine_params,
         "--a-elo", str(baseline_elo),
         "--b-elo", str(target_elo)],
        stdin=None,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    ) as process:
        while True:
            response = process.stdout.readline()

            if response == "":
                raise Exception("eof response")

            response = response.rstrip()

            print(f"{datetime_prefix()}{arena_prefix(config, arena_no, baseline_value, target_value)}{response}")

            if "Arena Finished" in response:
                float_regex = r"\d+(?:\.\d+)?"
                baseline_elo = float(re.search(rf"a-elo=({float_regex})", response).group(1))
                target_elo = float(re.search(rf"b-elo=({float_regex})", response).group(1))

                return ArenaResult(baseline_elo, target_elo)

def main():
    parser = argparse.ArgumentParser()

    parser.add_argument("--compile-profile", type=str, default="bench")
    parser.add_argument("--engine-params", type=str, required=True)

    parser.add_argument("--target-param", type=str, required=True)
    parser.add_argument("--start", type=int, required=True)
    parser.add_argument("--step", type=int, required=True)
    parser.add_argument("--end", type=int, required=True)

    parser.add_argument("--time", type=int, nargs=3, default=[180_000, 0, 10_000],
                        help="total(ms) increment(ms) turn(ms)")

    parser.add_argument("--openings", type=str, default=None)

    parser.add_argument("--num-games", type=int, default=17)
    parser.add_argument("--num-concurrency", type=int, default=1)

    args = parser.parse_args()
    num_arenas = int((args.end - args.start) / args.step)

    config = Config(args, num_arenas)

    ratings = {
        config.args.start: 1000.0
    }

    optimal_value = config.args.start
    compile_engine(config, config.args.start)

    for arena_no, target_value in enumerate(
            range(config.args.start + config.args.step, config.args.end + 1, config.args.step)
    ):
        baseline_value = optimal_value

        compile_engine(config, target_value)

        result = play_arena(
            config, arena_no,
            baseline_value, target_value,
            ratings[baseline_value], ratings.get(target_value, 1000.0)
        )

        ratings[baseline_value] = result.baseline_elo
        ratings[target_value] = result.target_elo

        if result.target_elo > result.baseline_elo:
            optimal_value = target_value

        print(f"{datetime_prefix()}{arena_prefix(config, arena_no, optimal_value, target_value)}ELO Updated: "
              f"optimal={optimal_value}, "
              f"{baseline_value}-elo={result.baseline_elo}, {target_value}-elo={result.target_elo}, "
              f"elo-table={ratings}")

    print(f"{datetime_prefix()}Optimize Finished: optimal-value={optimal_value}, elo-table={ratings}")

if __name__ == "__main__":
    main()
