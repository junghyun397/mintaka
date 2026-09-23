import logging
import math
import signal

import arena
import worker_manager


def calculate_elo(score: float) -> float:
    if score == 0.0:
        return -math.inf
    if score == 1.0:
        return math.inf
    return 400.0 * math.log10(score / (1.0 - score))


def calculate_ci95(score: float, squared_errors: float, count: int) -> tuple[float, float]:
    variance = max(squared_errors, 1.0)
    loglog = math.log(math.log(2.0 * variance))
    radius = (1.7 * math.sqrt(variance * (loglog + 3.8)) + 3.4 * loglog + 13.0) / count
    return calculate_elo(max(0.0, score - radius)), calculate_elo(min(1.0, score + radius))


def main():
    signal.signal(signal.SIGTERM, signal.default_int_handler)

    parser = arena.new_parser([120000, 0, 5000])

    parser.add_argument("--min-openings", type=int, default=100)
    parser.add_argument("--base-elo", type=float, default=1000.0)
    parser.add_argument("--target-elo", type=float, default=1000.0)

    config = arena.Config(parser.parse_args())
    arena.configure_logging(config.args.log_level)
    openings = arena.load_openings(config)

    score = 0.5
    squared_errors = 0.0

    pentanomial = [0, 0, 0, 0, 0]

    results = arena.game_results([])

    elo = {
        arena.Player.BASE: config.args.base_elo,
        arena.Player.TARGET: config.args.target_elo,
    }

    completed_openings = 0
    decision = "complete"
    stats = None

    logging.info(f"ELO Started [{config.args.min_openings}, {config.args.max_openings}]: "
                 f"concurrency={config.args.concurrency}, "
                 f"base={config.args.base_elo:g}, target={config.args.target_elo:g}")

    with worker_manager.WorkerManager(config) as workers:
        for pair in workers.results(openings):
            pair_results = arena.game_results(pair.snapshots.values())
            pair_score = arena.pentanomial_score[arena.player_wdl(pair_results)]
            pentanomial[int(pair_score * 2)] += 1
            squared_errors += (pair_score / 2.0 - score) ** 2

            for player_or_color in results:
                results[player_or_color] += pair_results[player_or_color]
            completed_openings += 1

            score = sum(idx * count for idx, count in enumerate(pentanomial)) / (completed_openings * 4)
            elo_delta = calculate_elo(score)
            elo[arena.Player.TARGET] = elo[arena.Player.BASE] + elo_delta

            lower, upper = calculate_ci95(score, squared_errors, completed_openings)
            half_width = (upper - lower) / 2.0

            stats = (f"wdl={results[arena.Player.TARGET]}-{results[None]}-{results[arena.Player.BASE]}, "
                     f"bdw={results[arena.Color.BLACK]}-{results[None]}-{results[arena.Color.WHITE]}, "
                     f"pen={pentanomial}, "

                     f"delta={elo_delta:+.2f}, elo={elo[arena.Player.BASE]:.2f}-{elo[arena.Player.TARGET]:.2f}, "
                     f"ci95=[{lower:+.2f}, {upper:+.2f}], ci95-hw={half_width:.2f}")

            logging.info(f"Pair Finished [{completed_openings}/{config.args.max_openings}]: {stats}")

            if completed_openings >= config.args.min_openings and half_width <= 2.0:
                decision = "precision"
                break

    logging.info(f"ELO Finished [{completed_openings}/{config.args.max_openings}]: {decision}, {stats}")


if __name__ == "__main__":
    main()
