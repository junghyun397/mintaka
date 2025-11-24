import argparse
from dataclasses import dataclass

class Config:
    def __init__(self, params):
        self.params = params

@dataclass
class ArenaResult:
    baseline_elo: float
    target_elo: float

def compile_engine(target_param, target_value):
    pass

def rename_engine():
    pass

def play_arena(config, arena_no) -> ArenaResult:
    pass

def main():
    parser = argparse.ArgumentParser()

    parser.add_argument("--engine-params", type=str, default="")
    parser.add_argument("--target-param", type=str, required=True)

    parser.add_argument("--start", type=float, required=True)
    parser.add_argument("--step", type=float, required=True)
    parser.add_argument("--end", type=float, required=True)

    parser.add_argument("--num-games", type=int, default=100)
    parser.add_argument("--num-arenas", type=int, default=50)
    parser.add_argument("--num-concurrency", type=int, default=1)

    config = Config(parser.parse_args())

    elo = {
        config.params.start: 1000.0
    }

    optimal_value = config.params.start

    for arena_no in range(config.params.num_arenas):
        baseline_value = optimal_value
        target_value = optimal_value + config.params.step

        compile_engine(config.params.target_param, target_value)
        rename_engine()

        result = play_arena(config, arena_no)

        elo[baseline_value] = result.baseline_elo
        elo[target_value] = result.target_elo

        if result.target_elo > result.baseline_elo:
            optimal_value = target_value

if __name__ == "__main__":
    main()
