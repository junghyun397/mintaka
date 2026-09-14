import math
from concurrent.futures import FIRST_COMPLETED, ThreadPoolExecutor, wait

import arena


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
    parser = arena.new_parser([120000, 0, 5000])

    parser.add_argument("--min-openings", type=int, default=100)
    parser.add_argument("--base-elo", type=float, default=1000.0)
    parser.add_argument("--target-elo", type=float, default=1000.0)

    config = arena.Config(parser.parse_args())
    openings = arena.load_openings(config)

    score = 0.5
    squared_errors = 0.0

    pentanomial = [0, 0, 0, 0, 0]

    results = {
        arena.Player.BASE: 0,
        arena.Player.TARGET: 0,
        arena.Color.BLACK: 0,
        arena.Color.WHITE: 0,
        None: 0,
    }

    elo = {
        arena.Player.BASE: config.args.base_elo,
        arena.Player.TARGET: config.args.target_elo,
    }

    completed_openings = 0
    decision = "complete"

    print(f"{arena.datetime_prefix(config)}ELO Started: "
          f"base={config.args.base_elo:g}, target={config.args.target_elo:g}, "
          f"openings=[{config.args.min_openings}, {config.args.max_openings}],",
          f"concurrency={config.args.concurrency}",
          flush=True)

    with ThreadPoolExecutor(max_workers=config.args.concurrency) as executor:
        pending = {
            executor.submit(arena.play_pair, config, opening_no, openings[opening_no])
            for opening_no in range(min(config.args.concurrency, config.args.max_openings))
        }
        next_opening = len(pending)

        try:
            while pending:
                done, _ = wait(pending, return_when=FIRST_COMPLETED)
                future = done.pop()
                pending.remove(future)
                opening, snapshot, pair_result = future.result()

                pair_score = arena.pentanomial_score[arena.player_wdl(pair_result)]
                pentanomial[int(pair_score * 2)] += 1
                squared_errors += (pair_score / 2.0 - score) ** 2

                for player_or_color in results:
                    results[player_or_color] += pair_result[player_or_color]
                completed_openings += 1

                score = sum(idx * count for idx, count in enumerate(pentanomial)) / (completed_openings * 4)
                elo_delta = calculate_elo(score)
                elo[arena.Player.BASE] -= elo_delta / 2.0
                elo[arena.Player.TARGET] += elo_delta / 2.0

                lower, upper = calculate_ci95(score, squared_errors, completed_openings)
                half_width = (upper - lower) / 2.0

                stats = (f"wdl={results[arena.Player.TARGET]}-{results[None]}-{results[arena.Player.BASE]}, "
                         f"bdw={results[arena.Color.BLACK]}-{results[None]}-{results[arena.Color.WHITE]}, "
                         f"pen={pentanomial}, "

                         f"delta={elo_delta:+.2f}, elo={elo[arena.Player.BASE]:.2f}-{elo[arena.Player.TARGET]:.2f}, "
                         f"ci95=[{lower:+.2f}, {upper:+.2f}], ci95-hw={half_width:.2f}")

                print(f"{arena.datetime_prefix(config)}[{completed_openings}/{config.args.max_openings}] "
                      f"Pair Finished: {stats}",
                      flush=True)

                if completed_openings >= config.args.min_openings and half_width <= 2.0:
                    decision = "precision"
                    break

                if next_opening < config.args.max_openings:
                    pending.add(executor.submit(
                        arena.play_pair, config, next_opening, openings[next_opening]
                    ))
                    next_opening += 1
        finally:
            for future in pending:
                future.cancel()

    print(f"{arena.datetime_prefix(config)}ELO Finished: {decision},",
          f"openings={completed_openings}/{config.args.max_openings},",

          f"{stats}",
          flush=True)


if __name__ == "__main__":
    main()
