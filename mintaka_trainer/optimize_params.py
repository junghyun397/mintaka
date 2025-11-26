import argparse
import os
import re
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone

class Config:
    def __init__(self, params, num_arenas):
        self.params = params
        self.num_arenas = num_arenas

    def engine_name(self, target_value) -> str:
        return f"mintaka_text_protocol_{self.params.target_param}_{target_value}"

@dataclass
class ArenaResult:
    baseline_elo: float
    target_elo: float

def datetime_prefix() -> str:
    return datetime.now(timezone.utc).strftime("[%Y-%m-%dT%H:%M:%SZ] ")

def arena_prefix(config, arena_no, optimal_value, current_value) -> str:
    return f"[{arena_no + 1}/{config.num_arenas}] [{config.params.target_param}:{optimal_value}v{current_value}] "

def compile_engine(config, target_value):
    env = os.environ.copy()
    env[config.params.target_param] = str(target_value)

    subprocess.run(
        ["cargo", "build",
         "--profile", config.params.compile_profile,
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
        ["python3", "mintaka_trainer/arena.py",
         "--no-datetime-prefix",
         "--log-prefix-filter",
         "--a-time", str(config.params.time[0]), str(config.params.time[1]), str(config.params.time[2]),
         "--b-time", str(config.params.time[0]), str(config.params.time[1]), str(config.params.time[2]),
         "--elo-k-factor", str(16.0),
         "--num-games", str(config.params.num_games),
         "--a-path", f"target/release/{config.engine_name(baseline_value)}",
         "--b-path", f"target/release/{config.engine_name(target_value)}",
         "--a-params", config.params.engine_params,
         "--b-params", config.params.engine_params,
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

    parser.add_argument("--num-games", type=int, default=17)
    parser.add_argument("--num-concurrency", type=int, default=1)

    params = parser.parse_args()
    num_arenas = int((params.end - params.start) / params.step)

    config = Config(params, num_arenas)

    elo = {
        config.params.start: 1000.0
    }

    optimal_value = config.params.start
    compile_engine(config, config.params.start)

    arena_no = 0

    for target_value in range(config.params.start + config.params.step, config.params.end + 1, config.params.step):
        baseline_value = optimal_value

        compile_engine(config, target_value)

        result = play_arena(
            config, arena_no,
            baseline_value, target_value,
            elo[baseline_value], elo.get(target_value, 1000.0)
        )

        elo[baseline_value] = result.baseline_elo
        elo[target_value] = result.target_elo

        if result.target_elo > result.baseline_elo:
            optimal_value = target_value

        print(f"{datetime_prefix()}{arena_prefix(config, arena_no, optimal_value, target_value)}ELO Updated: "
              f"optimal={optimal_value}, "
              f"{baseline_value}-elo={result.baseline_elo}, {target_value}-elo={result.target_elo}, "
              f"elo-table={elo}")

        arena_no += 1

    print(f"{datetime_prefix()}Optimize Finished: optimal-value={optimal_value}, elo-table={elo}")

if __name__ == "__main__":
    main()
