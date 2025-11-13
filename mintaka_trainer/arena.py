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

class Config:
    def __init__(self, params):
        self.params = params

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
    a_color: Color
    b_color: Color
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

    def a_win_zero_to_one(self) -> float:
        return self._win(self.a_color)

    def b_win_zero_to_one(self) -> float:
        return self._win(self.b_color)

@dataclass
class Engine:
    process: subprocess.Popen
    time_manager: TimeManager

def calculate_elo_delta(original_elo, opponent_elo, result_zero_to_one, k_factor) -> float:
    expected = 1.0 / (1.0 + 10.0 ** ((opponent_elo - original_elo) / 400.0))

    return k_factor * (result_zero_to_one - expected)

def datetime_prefix() -> str:
    return datetime.now(timezone.utc).strftime("[%Y-%m-%dT%H:%M:%SZ]")

def game_prefix(config, game_no) -> str:
    return f"[{game_no + 1}/{config.params.num_games}]"

def turn_prefix(player_color, a_color, turn_no) -> str:
    player = "A" if player_color == a_color else "B"
    return f"[#{turn_no}:{player}:{player_color}]"

def spawn_process(path, params) -> subprocess.Popen:
    return subprocess.Popen(
        [path] + shlex.split(params),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=-1
    )

def command_process(process, command,
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
            print(f"{datetime_prefix()} {println_prefix}{response}")

def play_game(config, game_no) -> GameResult:
    with (
        spawn_process(config.params.a_path, config.params.a_params) as a_process,
        spawn_process(config.params.b_path, config.params.b_params) as b_process,
    ):
        player = Color.BLACK
        winner = None

        (a_color, b_color) = (Color.BLACK, Color.WHITE) if game_no % 2 == 0 else (Color.WHITE, Color.BLACK)

        a_engine = Engine(
            a_process,
            TimeManager(config.params.a_time[0], config.params.a_time[1], config.params.a_time[2])
        )

        b_engine = Engine(
            b_process,
            TimeManager(config.params.b_time[0], config.params.b_time[1], config.params.b_time[2])
        )

        for engine in [a_engine, b_engine]:
            command_process(engine.process, f"limit time total {engine.time_manager.total_remaining}")
            command_process(engine.process, f"limit time increment {engine.time_manager.increment}")
            command_process(engine.process, f"limit time turn {engine.time_manager.turn}")

        player_engine, opponent_engine = (a_engine, b_engine) if a_color == Color.BLACK else (b_engine, a_engine)

        turn_no = 0

        while True:
            turn_no += 1
            timer = time.perf_counter_ns()

            move = command_process(
                player_engine.process, "gen",
                println=True,
                filter_prefix=["solution"],
                println_prefix=f"{game_prefix(config, game_no)} {turn_prefix(player, a_color, turn_no)} "
            )

            time_elapsed = int((time.perf_counter_ns() - timer) / 1_000_000)

            player_engine.time_manager.consume(time_elapsed)

            if move == "none" or turn_no >= 225:
                winner_player = None
            else:
                winner_player = command_process(player_engine.process, f"play {move}")
                winner_opponent = command_process(opponent_engine.process, f"play {move}")

                if winner_player != winner_opponent:
                    raise Exception(f"a_winner={winner_player}, b_winner={winner_opponent} ")

            if winner_player is not None:
                if "black" in winner_player.lower():
                    winner = Color.BLACK
                elif "white" in winner_player.lower():
                    winner = Color.WHITE

                break

            player_engine.time_manager.apply_increment()
            command_process(player_engine.process,
                            f"limit time total {int(player_engine.time_manager.total_remaining)}")

            player = player.flip()
            player_engine, opponent_engine = opponent_engine, player_engine

        history = command_process(a_engine.process, "history")
        board_str = command_process(a_engine.process, "board", println=True)

        a_engine.process.terminate()
        b_engine.process.terminate()

        return GameResult(a_color, b_color, winner, history, board_str)

def main():
    parser = argparse.ArgumentParser()

    default_total_increment_time = [300_000, 0, 30_00]

    parser.add_argument("--a-path", type=str, required=True)
    parser.add_argument("--a-params", type=str, default="")

    parser.add_argument("--b-path", type=str, required=True)
    parser.add_argument("--b-params", type=str, default="")

    parser.add_argument("--num-games", type=int, default=17)

    parser.add_argument("--a-time", type=int, nargs=3, default=default_total_increment_time,
                        help="total(ms) increment(ms) turn(ms)")
    parser.add_argument("--b-time", type=int, nargs=3, default=default_total_increment_time,
                        help="total(ms) increment(ms) turn(ms)")

    parser.add_argument("--a-elo", type=float, default=1000.0)
    parser.add_argument("--b-elo", type=float, default=1000.0)
    parser.add_argument("--elo-k-factor", type=float, default=32.0)

    config = Config(parser.parse_args())

    a_elo = config.params.a_elo
    b_elo = config.params.b_elo

    a_wins = 0
    b_wins = 0

    black_wins = 0
    white_wins = 0

    draws = 0

    for game_no in range(config.params.num_games):
        result = play_game(config, game_no)

        elo_delta = calculate_elo_delta(a_elo, b_elo, result.a_win_zero_to_one(), config.params.elo_k_factor)

        a_elo += elo_delta
        b_elo -= elo_delta

        a_wins += int(round(result.a_win_zero_to_one()))
        b_wins += int(round(result.b_win_zero_to_one()))
        draws = game_no + 1 - a_wins - b_wins

        black_wins += 1 if result.winner == Color.BLACK else 0
        white_wins = game_no + 1 - black_wins - draws

        prefix = f"{datetime_prefix()} {game_prefix(config, game_no)}"

        print(f"{prefix} Game State:\n{result.board_str}")
        print(f"{prefix} Game History: {result.history}")
        print(f"{prefix} Game Finished: a={result.a_color}, b={result.b_color}, win={result.winner}, "
              f"abd={a_wins}-{b_wins}-{draws}, bwd={black_wins}-{white_wins}-{draws}")
        print(f"{prefix} ELO Updated: a{elo_delta:+}, b{-elo_delta:+}, a={a_elo}, b={b_elo}")

    print(f"{datetime_prefix()} Arena Finished: total={config.params.num_games}, abd={a_wins}-{b_wins}-{draws}, "
          f"awr={a_wins / config.params.num_games * 100.0}%, bwr={b_wins / config.params.num_games * 100.0}%, "
          f"bwd={black_wins}-{white_wins}-{draws}")

if __name__ == "__main__":
    main()
