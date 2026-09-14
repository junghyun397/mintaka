import argparse
import copy
import random
import re
import secrets
import shlex
import subprocess
import time
from contextlib import ExitStack
from dataclasses import dataclass
from datetime import datetime, timezone, timedelta
from enum import Enum


class Color(Enum):
    BLACK = 0
    WHITE = 1

    def flip(self) -> 'Color':
        return Color.BLACK if self == Color.WHITE else Color.WHITE

    def __str__(self) -> str:
        return self.name.lower()


class Player(Enum):
    BASE = 0
    TARGET = 1

    def flip(self) -> 'Player':
        return Player.TARGET if self == Player.BASE else Player.BASE

    def __str__(self) -> str:
        return self.name.lower()


def time_manager_or_nodes(time, nodes) -> TimeManager | int:
    if nodes is not None:
        return nodes
    else:
        return TimeManager(
            total_remaining=time[0], increment=time[1], turn=time[2]
        )


class Config:
    def __init__(self, args):
        self.args = args

        self.path_params_resource = {
            Player.BASE: (self.args.base_path, self.args.base_params, time_manager_or_nodes(
                self.args.time, self.args.base_nodes
            )),
            Player.TARGET: (self.args.target_path, self.args.target_params, time_manager_or_nodes(
                self.args.time, self.args.target_nodes
            )),
        }


@dataclass
class Opening:
    initial_color: Color
    sequence: str


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
class GameSnapshot:
    winner: tuple[Player, Color] | None
    duration: timedelta
    history: str


GameResults = dict[Player | Color | None, int]


def player_wdl(result: GameResults) -> tuple[int, int, int]:
    return result[Player.TARGET], result[None], result[Player.BASE]


def color_wdl(result: GameResults) -> tuple[int, int, int]:
    return result[Color.BLACK], result[None], result[Color.WHITE]


@dataclass
class Engine:
    process: subprocess.Popen
    resource: TimeManager | int


def datetime_prefix(config) -> str:
    if config.args.no_datetime_prefix:
        return ""
    else:
        return datetime.now(timezone.utc).strftime("[%Y-%m-%dT%H:%M:%SZ] ")


def game_prefix(config, game_no) -> str:
    return f"[{game_no + 1}/{config.args.max_openings * 2}] "


def turn_prefix(turn_no, player, color) -> str:
    return f"[#{turn_no}:{player}:{color}] "


def snapshot_display(opening: Opening, snapshots: list[GameSnapshot]) -> str:
    return (
        f"duration={snapshots[0].duration.total_seconds()}s-{snapshots[1].duration.total_seconds()}s, "
        f"history={snapshots[0].history}-{snapshots[1].history}, "
        f"opening={opening.sequence}"
    )


def spawn_process(path, params, resource: TimeManager | int, opening: Opening) -> subprocess.Popen:
    if isinstance(resource, TimeManager):
        resource_param = ["--time", str(resource.total_remaining), str(resource.increment), str(resource.turn)]
    else:
        resource_param = ["--nodes-in-1k", str(resource)]

    return subprocess.Popen(
        [path]
            + shlex.split(params)
            + resource_param
            + ["--history", opening.sequence],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )


def command_process(config, process, command, filter_prefix: list[str] | None = None, println_prefix: str = "") -> str | None:
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

        if (not config.args.concise
                and response.startswith("%")
                and filter_prefix is not None
                and response[2:].startswith(tuple(filter_prefix))
        ):
            print(f"{datetime_prefix(config)}{println_prefix}{response[2:]}")


def play_game(config, game_no: int, opening: Opening) -> GameSnapshot:
    game_timer = time.perf_counter_ns()
    initial_color = opening.initial_color

    with ExitStack() as stack:
        processes = {
            player: stack.enter_context(spawn_process(path, params, resource, opening))
            for player, (path, params, resource) in config.path_params_resource.items()
        }

        stack.callback(processes[Player.BASE].terminate)
        stack.callback(processes[Player.TARGET].terminate)

        engines = {
            player: Engine(process, copy.copy(config.path_params_resource[player][2]))
            for player, process in processes.items()
        }

        player = Player.BASE if game_no % 2 == 0 else Player.TARGET

        color = {
            player: initial_color,
            player.flip(): initial_color.flip()
        }

        winner = None
        turn_no = 0

        while True:
            turn_no += 1
            timer = time.perf_counter_ns()

            move = command_process(
                config,
                engines[player].process, "gen",
                filter_prefix=config.args.log_prefix_filter,
                println_prefix=f"{game_prefix(config, game_no)}{turn_prefix(turn_no, player, color[player])}"
            )

            time_elapsed = int((time.perf_counter_ns() - timer) / 1_000_000)

            if isinstance(engines[player].resource, TimeManager):
                engines[player].resource.consume(time_elapsed)

            if move == "none":
                winner = color[player.flip()]
                break

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

            if isinstance(engines[player].resource, TimeManager):
                engines[player].resource.apply_increment()
                command_process(config, engines[player].process,
                                f"limit time total {int(engines[player].resource.total_remaining)}")

            if turn_no >= config.args.draw_in:
                break

            player = player.flip()

        history = command_process(config, engines[Player.BASE].process, "history")
        board_str = command_process(config, engines[Player.BASE].process, "board")

        if board_str is None or history is None:
            raise Exception("board or history response is None")

        prefix = f"{datetime_prefix(config)}{game_prefix(config, game_no)}"
        if not config.args.concise:
            print(f"{prefix}Game State:\n{board_str}")
            print(f"{prefix}Game History: {history}")
            print(f"{prefix}Game Finished: base={color[Player.BASE]}, target={color[Player.TARGET]}, win={winner}")

    duration = timedelta(microseconds=(time.perf_counter_ns() - game_timer) // 1_000)

    if winner is None:
        return GameSnapshot(winner=None, duration=duration, history=history)

    return GameSnapshot(
        winner=(Player.BASE if winner == color[Player.BASE] else Player.TARGET, winner),
        duration=duration, history=history,
    )


def play_pair(config, opening_no: int, opening: Opening) -> tuple[Opening, list[GameSnapshot], GameResults]:
    snapshots = []
    results = {
        Player.BASE: 0,
        Player.TARGET: 0,
        Color.BLACK: 0,
        Color.WHITE: 0,
        None: 0,
    }

    for first_player in Player:
        game_result = play_game(config, opening_no * 2 + first_player.value, opening)

        if game_result.winner is not None:
            for player_or_color in game_result.winner:
                results[player_or_color] += 1
        else:
            results[None] += 1

        snapshots.append(game_result)

    return opening, snapshots, results


pentanomial_score = {
    (0, 0, 2): 0,
    (0, 1, 1): 0.5,
    (0, 2, 0): 1,
    (1, 0, 1): 1,
    (1, 1, 0): 1.5,
    (2, 0, 0): 2,
}


def new_parser(default_time: list[int]) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()

    parser.add_argument("--seed", type=int, default=secrets.randbits(32))

    parser.add_argument("--base-path", type=str, required=True)
    parser.add_argument("--base-params", type=str, default="")

    parser.add_argument("--target-path", type=str, required=True)
    parser.add_argument("--target-params", type=str, default="")

    parser.add_argument("--max-openings", type=int, default=100)
    parser.add_argument("--concurrency", type=int, default=2)

    parser.add_argument("--time", type=int, nargs=3, default=default_time)
    parser.add_argument("--base-nodes", type=int, default=None)
    parser.add_argument("--target-nodes", type=int, default=None)

    parser.add_argument("--openings-file", type=str, required=True)
    parser.add_argument("--draw-in", type=int, default=225)

    parser.add_argument("--concise", action="store_true", default=False)
    parser.add_argument("--no-datetime-prefix", action="store_true", default=False)
    parser.add_argument("--log-prefix-filter", type=str, nargs="*", default=["solution"])

    return parser


def load_openings(config: Config) -> list[Opening]:
    with open(config.args.openings_file, "r") as openings_file:
        openings = [
            Opening(Color.BLACK if len(moves) % 2 == 0 else Color.WHITE, ",".join(moves))
            for line in openings_file
            if (moves := re.findall(r"[a-o](?:1[0-5]|[1-9])(?!\d)", line.lower()))
        ]

    if not openings:
        raise Exception("openings-file contains no openings")

    rng = random.Random(config.args.seed)

    rng.shuffle(openings)

    if len(openings) < config.args.max_openings:
        repeats = config.args.max_openings // len(openings)
        remainder = config.args.max_openings % len(openings)
        openings = openings * repeats + openings[:remainder]
        rng.shuffle(openings)

    return openings
