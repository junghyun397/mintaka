import logging
import math

import arena
import worker_manager


def score_from_elo(elo: float) -> float:
    odds = 10.0 ** (-abs(elo) / 400.0)
    return 1.0 / (1.0 + odds) if elo >= 0 else odds / (1.0 + odds)


def fit_probabilities(counts: list[float], score: float) -> list[float]:
    offsets = [i / 4.0 - score for i in range(5)]
    lower = -1.0 / offsets[-1]
    upper = -1.0 / offsets[0]

    midpoint = 0
    for _ in range(64):
        midpoint = (lower + upper) / 2.0
        if midpoint == lower or midpoint == upper:
            break

        slope = sum(count * offset / (1.0 + midpoint * offset) for count, offset in zip(counts, offsets))
        if slope > 0:
            lower = midpoint
        else:
            upper = midpoint

    total = sum(counts)
    return [count / total / (1.0 + midpoint * offset) for count, offset in zip(counts, offsets)]


def calculate_llr(pentanomial: list[int], elo0: float, elo1: float) -> float:
    if sum(pentanomial) == 0:
        return 0.0

    counts = [count if count else 1e-3 for count in pentanomial]
    probabilities0 = fit_probabilities(counts, score_from_elo(elo0))
    probabilities1 = fit_probabilities(counts, score_from_elo(elo1))
    return sum(count * math.log(p1 / p0) for count, p0, p1 in zip(counts, probabilities0, probabilities1))


def main():
    parser = arena.new_parser([30_000, 300, 0])

    parser.add_argument("--elo0", type=float, default=0.0)
    parser.add_argument("--elo1", type=float, default=5.0)
    parser.add_argument("--alpha", type=float, default=0.05)
    parser.add_argument("--beta", type=float, default=0.05)

    config = arena.Config(parser.parse_args())
    arena.configure_logging(config.args.log_level)
    openings = arena.load_openings(config)

    lower = math.log(config.args.beta) - math.log1p(-config.args.alpha)
    upper = math.log1p(-config.args.beta) - math.log(config.args.alpha)

    pentanomial = [0, 0, 0, 0, 0]

    results = arena.game_results([])

    completed_openings = 0
    decision = "inconclusive"
    stats = None

    logging.info(f"SPRT Started [0, {config.args.max_openings}]: "
                 f"concurrency={config.args.concurrency}, "
                 f"bounds=[{lower:.3f}, {upper:.3f}], "
                 f"elo=[{config.args.elo0:g}, {config.args.elo1:g}]")

    with worker_manager.WorkerManager(config) as workers:
        for pair in workers.results(openings):
            pair_results = arena.game_results(pair.snapshots.values())
            pair_score = arena.pentanomial_score[arena.player_wdl(pair_results)]

            bucket = int(pair_score * 2)
            pentanomial = [count + (idx == bucket) for idx, count in enumerate(pentanomial)]
            llr = calculate_llr(pentanomial, config.args.elo0, config.args.elo1)

            for player_or_color in results:
                results[player_or_color] += pair_results[player_or_color]
            completed_openings += 1

            stats = (f"wdl={results[arena.Player.TARGET]}-{results[None]}-{results[arena.Player.BASE]}, "
                     f"bdw={results[arena.Color.BLACK]}-{results[None]}-{results[arena.Color.WHITE]}, "
                     f"pen={pentanomial}, "

                     f"llr={llr:.3f}")

            logging.info(f"Pair Finished [{completed_openings}/{config.args.max_openings}]: {stats}")

            if llr <= lower:
                decision = "rejected"
                break
            if llr >= upper:
                decision = "accepted"
                break

    logging.info(f"SPRT Finished [{completed_openings}/{config.args.max_openings}]: {decision}, "
                 f"elo=[{config.args.elo0:g}, {config.args.elo1:g}], bounds=[{lower:.3f}, {upper:.3f}], "
                 f"{stats}")


if __name__ == "__main__":
    main()
