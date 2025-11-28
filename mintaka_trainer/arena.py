import argparse
import shlex
import subprocess
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from enum import Enum

class Color(Enum):
    BLACK = 0
    WHITE = 1

    def flip(self) -> 'Color':
        return Color.BLACK if self == Color.WHITE else Color.WHITE

    def __str__(self) -> str:
        return self.name.lower()

class Player(Enum):
    A = 0
    B = 1

    def flip(self) -> 'Player':
        return Player.B if self == Player.A else Player.A

    def __str__(self) -> str:
        return self.name

class Config:
    def __init__(self, args):
        self.args = args

@dataclass
class TimeManager:
    total_remaining: int
    increment: int
    turn: int

    def apply_increment(self):
        self.total_remaining += self.increment

    def consume(self, running_time):
        self.total_remaining = max(0, self.total_remaining - running_time)

@dataclass
class GameResult:
    color: dict[Player, Color]
    winner: Color | None
    history: str
    board_str: str

    def _win(self, color: Color) -> float:
        if self.winner is None:
            return 0.5
        elif self.winner == color:
            return 1.0
        else:
            return 0.0

    def win_zero_to_one(self, player) -> float:
        return self._win(self.color[player])

@dataclass
class Engine:
    process: subprocess.Popen
    time_manager: TimeManager

def calculate_elo_delta(original_elo, opponent_elo, result_zero_to_one, k_factor) -> float:
    expected = 1.0 / (1.0 + 10.0 ** ((opponent_elo - original_elo) / 400.0))

    return k_factor * (result_zero_to_one - expected)

def datetime_prefix(config) -> str:
    if config.args.no_datetime_prefix:
        return ""
    else:
        return datetime.now(timezone.utc).strftime("[%Y-%m-%dT%H:%M:%SZ] ")

def game_prefix(config, game_no) -> str:
    return f"[{game_no + 1}/{config.args.num_games}] "

def turn_prefix(turn_no, player, color) -> str:
    return f"[#{turn_no}:{player}:{color}] "

def spawn_process(path, params) -> subprocess.Popen:
    return subprocess.Popen(
        [path] + shlex.split(params),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )

def command_process(config, process, command,
                    println=False, filter_prefix: list[str] | None = None, println_prefix: str = "") -> str | None:
    process.stdin.write(f"{command}\n")
    process.stdin.flush()

    while True:
        response = process.stdout.readline()

        if response == "":
            raise Exception("eof response")

        response = response.strip()

        if response == "=":
            return None

        if response.startswith("?"):
            raise Exception(f"command: {command}, error: {response[2:]}")

        if response.startswith("=\x02"):
            while True:
                stream_response = process.stdout.readline()
                if stream_response == "":
                    raise Exception("eof response")

                response += stream_response
                if "\x03" in stream_response:
                    return response[2:-(len(stream_response) - stream_response.index("\x03"))]

        if response.startswith("="):
            return response[2:]

        if println and (filter_prefix is None or response.startswith(tuple(filter_prefix))):
            print(f"{datetime_prefix(config)}{println_prefix}{response}")

def play_game(config, game_no) -> GameResult:
    with (
        spawn_process(config.args.a_path, config.args.a_params) as a_process,
        spawn_process(config.args.b_path, config.args.b_params) as b_process,
    ):
        engines = {
            Player.A: Engine(
                a_process,
                TimeManager(config.args.a_time[0], config.args.a_time[1], config.args.a_time[2])
            ),
            Player.B: Engine(
                b_process,
                TimeManager(config.args.b_time[0], config.args.b_time[1], config.args.b_time[2])
            )
        }

        for engine in engines.values():
            command_process(config, engine.process, f"limit time total {engine.time_manager.total_remaining}")
            command_process(config, engine.process, f"limit time increment {engine.time_manager.increment}")
            command_process(config, engine.process, f"limit time turn {engine.time_manager.turn}")

        player = Player.A if game_no % 2 == 0 else Player.B

        color = {
            player: Color.BLACK,
            player.flip(): Color.WHITE
        }

        winner = None
        turn_no = 0

        while True:
            turn_no += 1
            timer = time.perf_counter_ns()

            move = command_process(
                config,
                engines[player].process, "gen",
                println=True,
                filter_prefix=config.args.log_prefix_filter,
                println_prefix=f"{game_prefix(config, game_no)}{turn_prefix(turn_no, player, color[player])}"
            )

            time_elapsed = int((time.perf_counter_ns() - timer) / 1_000_000)

            engines[player].time_manager.consume(time_elapsed)

            if move == "none":
                winner_player = None
            else:
                winner_player = command_process(config, engines[player].process, f"play {move}")
                winner_opponent = command_process(config, engines[player.flip()].process, f"play {move}")

                if winner_player != winner_opponent:
                    raise Exception(f"player_winner={winner_player}, opponent_winner={winner_opponent} ")

            if winner_player is not None:
                if "black" in winner_player.lower():
                    winner = Color.BLACK
                elif "white" in winner_player.lower():
                    winner = Color.WHITE

                break

            engines[player].time_manager.apply_increment()
            command_process(config, engines[player].process,
                            f"limit time total {int(engines[player].time_manager.total_remaining)}")

            if turn_no >= config.args.draw_in:
                break

            player = player.flip()

        history = command_process(config, engines[Player.A].process, "history")
        board_str = command_process(config, engines[Player.B].process, "board", println=True)

        for engine in engines.values():
            engine.process.terminate()

        return GameResult(color, winner, history, board_str)

def main():
    default_total_increment_turn = [300_000, 0, 30_000]

    parser = argparse.ArgumentParser()

    parser.add_argument("--a-path", type=str, required=True)
    parser.add_argument("--a-params", type=str, default="")

    parser.add_argument("--b-path", type=str, required=True)
    parser.add_argument("--b-params", type=str, default="")

    parser.add_argument("--num-games", type=int, default=100)

    parser.add_argument("--a-time", type=int, nargs=3, default=default_total_increment_turn,
                        help="total(ms) increment(ms) turn(ms)")
    parser.add_argument("--b-time", type=int, nargs=3, default=default_total_increment_turn,
                        help="total(ms) increment(ms) turn(ms)")

    parser.add_argument("--draw-in", type=int, default=225)

    parser.add_argument("--a-elo", type=float, default=1000.0)
    parser.add_argument("--b-elo", type=float, default=1000.0)
    parser.add_argument("--elo-k-factor", type=float, default=32.0)

    parser.add_argument("--no-datetime-prefix", action='store_true', default=False)
    parser.add_argument("--log-prefix-filter", type=str, nargs="*", default=["solution"])

    config = Config(parser.parse_args())

    elo = {
        Player.A: config.args.a_elo,
        Player.B: config.args.b_elo,
    }

    wins = {
        Player.A: 0,
        Player.B: 0,
        Color.BLACK: 0,
        Color.WHITE: 0,
    }

    draws = 0

    for game_no in range(config.args.num_games):
        result = play_game(config, game_no)

        elo_delta = calculate_elo_delta(
            elo[Player.A], elo[Player.B],
            result.win_zero_to_one(Player.A), config.args.elo_k_factor
        )

        elo[Player.A] += elo_delta
        elo[Player.B] -= elo_delta

        wins[Player.A] += int(round(result.win_zero_to_one(Player.A)))
        wins[Player.B] += int(round(result.win_zero_to_one(Player.B)))

        wins[Color.BLACK] += 1 if result.winner == Color.BLACK else 0
        wins[Color.WHITE] = game_no + 1 - wins[Color.BLACK] - draws

        draws = game_no + 1 - wins[Player.A] - wins[Player.B]

        prefix = f"{datetime_prefix(config)}{game_prefix(config, game_no)}"

        print(f"{prefix}Game State:\n{result.board_str}")
        print(f"{prefix}Game History: {result.history}")
        print(f"{prefix}Game Finished: a={result.color[Player.A]}, b={result.color[Player.B]}, win={result.winner}, "
              f"abd={wins[Player.A]}-{wins[Player.B]}-{draws}, bwd={wins[Color.BLACK]}-{wins[Color.WHITE]}-{draws}")
        print(f"{prefix}ELO Updated: a{elo_delta:+}, b{-elo_delta:+}, a={elo[Player.A]}, b={elo[Player.B]}")

    print(
        f"{datetime_prefix(config)}Arena Finished: a-elo={elo[Player.A]}, b-elo={elo[Player.B]}, "
        f"abd={wins[Player.A]}-{wins[Player.B]}-{draws}, "
        f"awr={wins[Player.A] / config.args.num_games * 100.0}%, bwr={wins[Player.B] / config.args.num_games * 100.0}%, "
        f"bwd={wins[Color.BLACK]}-{wins[Color.WHITE]}-{draws}")

if __name__ == "__main__":
    main()
