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

def spawn_process(path, params) -> subprocess.Popen:
    process = subprocess.Popen(
        [path] + shlex.split(params),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=-1
    )

    return process

def terminate_process(process):
    try:
        process.terminate()
        process.wait(timeout=3)
    except Exception:
        try:
            process.kill()
        except Exception:
            pass

def calculate_elo(original_elo, opponent_elo, result_zero_to_one, k_factor) -> float:
    elo_e = 1 / (1 + 10 ** ((opponent_elo - original_elo) / 400))

    return original_elo + k_factor * (result_zero_to_one - elo_e)

def datetime_prefix() -> str:
    return f"[{datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S')}Z]"

def game_prefix(config, game_no) -> str:
    return f"[{game_no + 1}/{config.params.num_games}]"

def log_prefix(config, game_no) -> str:
    return f"{datetime_prefix()} {game_prefix(config, game_no)}"

def command_process(process, command, println=False, filter_prefix: str | None=None, println_prefix: str="") -> str | None:
    process.stdin.write(f"{command}\n")
    process.stdin.flush()

    while True:
        response = process.stdout.readline().strip()

        if response == "":
            return None

        if response.startswith("?"):
            raise Exception(f"command: {command}, error: {response[2:]}")

        if response == "=":
            return None

        if response.startswith("=\x02"):
            while True:
                stream_response = process.stdout.readline()
                response += f"{stream_response}"
                if stream_response.endswith("\x03"):
                    return response[2:-1]

        if response.startswith("="):
            return response[2:]

        if println and (filter_prefix is None or response.startswith(filter_prefix)):
            print(f"{datetime_prefix()} {println_prefix}{response}")

def play_game(config, game_no) -> GameResult:
    player = Color.BLACK
    winner = None

    (a_color, b_color) = (Color.BLACK, Color.WHITE) if game_no % 2 == 0 else (Color.WHITE, Color.BLACK)

    a_process = spawn_process(config.params.a_path, config.params.a_params)
    a_time_manager = TimeManager(config.params.total_time, config.params.increment_time, config.params.turn_time)

    b_process = spawn_process(config.params.b_path, config.params.b_params)
    b_time_manager = TimeManager(config.params.total_time, config.params.increment_time, config.params.turn_time)

    command_process(a_process, f"limit time total {a_time_manager.total_remaining}")
    command_process(a_process, f"limit time increment {a_time_manager.increment}")
    command_process(a_process, f"limit time turn {a_time_manager.turn}")

    command_process(b_process, f"limit time total {b_time_manager.total_remaining}")
    command_process(b_process, f"limit time increment {b_time_manager.increment}")
    command_process(b_process, f"limit time turn {b_time_manager.turn}")

    (player_process, player_time_manager), (opponent_process, opponent_time_manager) = \
        ((a_process, a_time_manager), (b_process, b_time_manager)) \
            if player == a_color else ((b_process, b_time_manager), (a_process, a_time_manager))

    while True:
        timer = time.perf_counter_ns()

        move = command_process(
            player_process, "gen",
            println=True,
            filter_prefix="solution",
            println_prefix=f"{game_prefix(config, game_no)} "
        )

        time_elapsed = (time.perf_counter_ns() - timer) / 1_000_000

        if move == "none":
            winner = player.flip()
            break

        player_time_manager.consume(time_elapsed)

        command_process(player_process, f"limit time total {int(player_time_manager.total_remaining)}")

        winner_player = command_process(player_process, f"play {move}")
        winner_opponent = command_process(opponent_process, f"play {move}")

        if winner_player != winner_opponent:
            raise Exception(f"a_winner={winner_player}, b_winner={winner_opponent} ")

        if winner_player is not None:
            if "black" in winner_player.lower():
                winner = Color.BLACK
            elif "white" in winner_player.lower():
                winner = Color.WHITE
            break

        player_time_manager.apply_increment()
        player = player.flip()
        player_process, opponent_process = opponent_process, player_process
        player_time_manager, opponent_time_manager = opponent_time_manager, player_time_manager

    history = command_process(a_process, "history")
    board_str = command_process(a_process, "board", println=True)

    a_process.terminate()
    b_process.terminate()

    return GameResult(a_color, b_color, winner, history, board_str)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--a-path", type=str, required=True)
    parser.add_argument("--a-params", type=str, default="")

    parser.add_argument("--b-path", type=str, required=True)
    parser.add_argument("--b-params", type=str, default="")

    parser.add_argument("--num-games", type=int, default=100)

    parser.add_argument("--pondering", action='store_true', default=False)

    parser.add_argument("--total-time", type=int, default=600_000)
    parser.add_argument("--increment-time", type=int, default=10_000)
    parser.add_argument("--turn-time", type=int, default=100)

    parser.add_argument("--a-elo", type=float, default=1000.0)
    parser.add_argument("--b-elo", type=float, default=1000.0)
    parser.add_argument("--elo-k-factor", type=float, default=32.0)

    config = Config(parser.parse_args())

    a_elo = config.params.a_elo
    b_elo = config.params.b_elo

    a_wins = 0
    b_wins = 0
    draws = 0

    for game_no in range(config.params.num_games):
        result = play_game(config, game_no)

        iter_a_elo = calculate_elo(a_elo, b_elo, result.a_win_zero_to_one(), config.params.elo_k_factor)
        iter_b_elo = calculate_elo(b_elo, a_elo, result.b_win_zero_to_one(), config.params.elo_k_factor)

        a_elo_delta = iter_a_elo - a_elo
        b_elo_delta = iter_b_elo - b_elo

        a_elo = iter_a_elo
        b_elo = iter_b_elo

        a_wins += int(round(result.a_win_zero_to_one()))
        b_wins += int(round(result.b_win_zero_to_one()))
        draws = game_no + 1 - a_wins - b_wins

        prefix = f"{datetime_prefix()} {game_prefix(config, game_no)}"

        print(f"{prefix} Game State:\n{result.board_str}")
        print(f"{prefix} Game History: {result.history}")
        print(f"{prefix} Game Finished: a={result.a_color}, b={result.b_color}, win={result.winner}, abd={a_wins}-{b_wins}-{draws}")
        print(f"{prefix} ELO Updated: a{a_elo_delta:+}, b{b_elo_delta:+}, a={a_elo}, b={b_elo}")

    print(f"{datetime_prefix()} Arena Finished: total={config.params.num_games}, a_wins={a_wins}, b_wins={b_wins}, draws={draws}")

if __name__ == "__main__":
    main()
