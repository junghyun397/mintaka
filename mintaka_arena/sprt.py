import math
from concurrent.futures import FIRST_COMPLETED, wait

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
    openings = arena.load_openings(config)

    lower = math.log(config.args.beta) - math.log1p(-config.args.alpha)
    upper = math.log1p(-config.args.beta) - math.log(config.args.alpha)

    pentanomial = [0, 0, 0, 0, 0]

    results = arena.game_results([])

    completed_openings = 0
    decision = "inconclusive"

    print(f"{arena.datetime_prefix(config)}SPRT Started:",
          f"elo=[{config.args.elo0:g}, {config.args.elo1:g}],",
          f"bounds=[{lower:.3f}, {upper:.3f}],",
          f"max-openings={config.args.max_openings},",
          f"concurrency={config.args.concurrency}",
          flush=True)

    with worker_manager.WorkerManager(config) as executor:
        pending = {
            executor.submit(opening_no, openings[opening_no])
            for opening_no in range(min(config.args.concurrency, config.args.max_openings))
        }
        next_opening = len(pending)

        try:
            while pending:
                done, _ = wait(pending, return_when=FIRST_COMPLETED)
                future = done.pop()
                pending.remove(future)
                pair = future.result()

                pair_score = arena.pentanomial_score[arena.player_wdl(pair.results)]

                bucket = int(pair_score * 2)
                pentanomial = [count + (idx == bucket) for idx, count in enumerate(pentanomial)]
                llr = calculate_llr(pentanomial, config.args.elo0, config.args.elo1)

                for player_or_color in results:
                    results[player_or_color] += pair.results[player_or_color]
                completed_openings += 1

                stats = (f"wdl={results[arena.Player.TARGET]}-{results[None]}-{results[arena.Player.BASE]}, "
                         f"bdw={results[arena.Color.BLACK]}-{results[None]}-{results[arena.Color.WHITE]}, "
                         f"pen={pentanomial}, "

                         f"llr={llr:.3f}")

                print(f"{arena.datetime_prefix(config)}[{completed_openings}/{config.args.max_openings}]",
                      f"Pair Finished: {stats}",
                      flush=True)

                if llr <= lower:
                    decision = "rejected"
                    break
                if llr >= upper:
                    decision = "accepted"
                    break

                if next_opening < config.args.max_openings:
                    pending.add(executor.submit(
                        next_opening, openings[next_opening]
                    ))
                    next_opening += 1
        finally:
            for future in pending:
                future.cancel()

    print(f"{arena.datetime_prefix(config)}SPRT Finished: {decision},",
          f"openings={completed_openings}/{config.args.max_openings},",

          f"elo=[{config.args.elo0:g}, {config.args.elo1:g}], bounds=[{lower:.3f}, {upper:.3f}],",
          f" {stats}",
          flush=True)


if __name__ == "__main__":
    main()
