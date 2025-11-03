import argparse
import subprocess
import time
from datetime import datetime, timezone
from enum import Enum
from typing import Optional, List

class Color(Enum):
    BLACK = 0
    WHITE = 1

    def flip(self) -> 'Color':
        return Color.BLACK if self == Color.WHITE else Color.WHITE

class Config:
    def __init__(self, params):
        self.params = params

class TimeManager:
    def __init__(self, total_time, increment_time, turn_time):
        self.total_remaining = total_time
        self.increment = increment_time
        self.turn = turn_time

    def apply_increment(self):
        self.total_remaining += self.increment

    def consume(self, running_time):
        self.total_remaining = max(0, self.total_remaining - running_time)

class GameResult:
    def __init__(self, a_color, b_color, winner, history, board_str):
        self.a_color: Color = a_color
        self.b_color: Color = b_color
        self.winner: Optional[Color] = winner

        self.history: List[str] = history
        self.board_str: str = board_str

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
    args = [path]
    args.extend(params.split(" "))

    process = subprocess.Popen(
        [path],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )

    assert process.poll() is None
    assert process.stdout.readline().strip().lower() == "ready"

    return process

def command_process(process, command):
    process.stdout.flush()
    process.stdin.write(f"{command}\n")
    process.stdin.flush()

def communicate_process(process, command) -> str:
    command_process(process, command)
    return process.stdout.readline(1).strip()

def play_game(config, game_no) -> GameResult:
    time_manager = TimeManager(config.params.total_time, config.params.increment_time, config.params.turn_time)
    player = Color.BLACK
    winner = None

    (a_color, b_color) = (Color.BLACK, Color.WHITE) if game_no % 2 == 0 else (Color.WHITE, Color.BLACK)

    a_process = spawn_process(config.params.a_path, config.params.a_params)
    b_process = spawn_process(config.params.b_path, config.params.b_params)

    for process in [a_process, b_process]:
        command_process(process, f"limit time total {time_manager.total_remaining}")
        command_process(process, f"limit time increment {time_manager.increment}")
        command_process(process, f"limit time turn {time_manager.turn}")

    player_process, opponent_process = (a_process, b_process) if player == a_color else (b_process, a_process)

    while True:
        time_manager.apply_increment()
        timer = time.perf_counter_ns()

        command_process(player_process, "gen")

        while True:
            response = player_process.stdout.readline().strip()
            if response.startswith("solution:"):
                move = response.strip()
                break

        time_elapsed = (time.perf_counter_ns() - timer) / 1_000_000

        command_process(player_process, f"limit time total {time_manager.total_remaining}")

        command_process(opponent_process, f"play {move}")

        winner_response = communicate_process(player_process, "winner")

        if "black" in winner_response:
            winner = Color.BLACK
        elif "white" in winner_response:
            winner = Color.WHITE

        if winner is not None:
            break

        player = player.flip()
        player_process, opponent_process = opponent_process, player_process

        time_manager.consume(time_elapsed)

    history = communicate_process(a_process, "history").split(",")
    board_str = communicate_process(a_process, "board")

    a_process.terminate()
    b_process.terminate()

    return GameResult(Color.BLACK, Color.WHITE, winner, history, board_str)

def calculate_elo(original_elo, opponent_elo, result_zero_to_one, k_factor) -> float:
    elo_e = 1 / (1 + 10 ** ((opponent_elo - original_elo) / 400))

    return original_elo + k_factor * (result_zero_to_one - elo_e)

def log_prefix() -> str:
    return f"[{datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S')}Z]"

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--a_path", type=str, required=True)
    parser.add_argument("--a_params", type=str, default="")

    parser.add_argument("--b_path", type=str, required=True)
    parser.add_argument("--b_params", type=str, default="")

    parser.add_argument("--num_games", type=int, default=100)

    parser.add_argument("--pondering", action='store_true', default=False)

    parser.add_argument("--total_time", type=int, default=600_000)
    parser.add_argument("--increment_time", type=int, default=10_000)
    parser.add_argument("--turn_time", type=int, default=40_000)

    parser.add_argument("--a_elo", type=float, default=1000.0)
    parser.add_argument("--b_elo", type=float, default=1000.0)
    parser.add_argument("--elo_k_factor", type=float, default=32.0)

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
        draws = game_no - a_wins - b_wins

        print(f"{log_prefix()} Game Finished: {game_no}: a={result.a_color}, b={result.b_color}, win={result.winner}, abd={a_wins}-{b_wins}-{draws}")
        print(f"{log_prefix()} ELO Updated: a{a_elo_delta:+}, b{b_elo_delta:+}, a={a_elo}, b={b_elo}")
        print(f"{log_prefix()} Game History: {result.history}")
        print(f"{log_prefix()} Game State: {result.board_str}")

    print(f"{log_prefix()} Arena Finished: total={config.params.num_games}, a_wins={a_wins}, b_wins={b_wins}, draws={draws}")

if __name__ == "__main__":
    main()
