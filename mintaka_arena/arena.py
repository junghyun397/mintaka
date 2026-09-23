from __future__ import annotations

import argparse
import copy
import logging
import random
import re
import secrets
import shlex
import subprocess
import sys
import time
from collections.abc import Iterable
from contextlib import ExitStack
from dataclasses import dataclass
from datetime import timedelta
from enum import Enum


class Rule(Enum):
    RENJU = "Renju"
    GOMOKU = "Gomoku"
    FREESTYLE = "Freestyle"

    def __str__(self) -> str:
        return self.name.lower()


class Color(Enum):
    BLACK = "Black"
    WHITE = "White"

    def flip(self) -> 'Color':
        return Color.BLACK if self == Color.WHITE else Color.WHITE

    def __str__(self) -> str:
        return self.name.lower()


class Player(Enum):
    BASE = "Base"
    TARGET = "Target"

    def flip(self) -> 'Player':
        return Player.TARGET if self == Player.BASE else Player.BASE

    def __str__(self) -> str:
        return self.name.lower()


class TimeUnit(Enum):
    CLOCK = "Clock"
    NODES = "Nodes"


GAME_SETTINGS = (
    "rule", "base_params", "target_params", "time_unit", "time", "draw_in", "log_prefix_filter",
)


class Config:
    def __init__(self, args):
        self.args = args

        self.path_params_resource: PathParamsResource = {
            player: (path, params, TimeManager(
                time_unit=TimeUnit(self.args.time_unit),
                total_remaining=self.args.time[0] or None,
                increment=self.args.time[1],
                turn=self.args.time[2] or None,
            ))
            for player, path, params in (
                (Player.BASE, self.args.base_path, self.args.base_params),
                (Player.TARGET, self.args.target_path, self.args.target_params),
            )
        }


@dataclass
class Opening:
    initial_color: Color
    sequence: str

    def to_json(self):
        return {"initial_color": self.initial_color.name, "sequence": self.sequence}

    @classmethod
    def from_json(cls, data) -> Opening:
        return cls(Color[data["initial_color"]], data["sequence"])


@dataclass
class TimeManager:
    time_unit: TimeUnit
    total_remaining: int | None
    increment: int
    turn: int | None

    def consume(self, amount: int):
        if self.total_remaining is not None:
            self.total_remaining = max(0, self.total_remaining - amount)


@dataclass
class GameSnapshot:
    winner: tuple[Player, Color] | None
    duration: timedelta
    history: str

    def to_json(self):
        return {
            "winner": [part.name for part in self.winner] if self.winner else None,
            "duration_us": self.duration // timedelta(microseconds=1),
            "history": self.history,
        }

    @classmethod
    def from_json(cls, data) -> GameSnapshot:
        winner = data["winner"]
        return cls(
            winner=(Player[winner[0]], Color[winner[1]]) if winner is not None else None,
            duration=timedelta(microseconds=data["duration_us"]),
            history=data["history"],
        )


Snapshots = dict[Player, GameSnapshot]


GameResults = dict[Player | Color | None, int]


PathParamsResource = dict[Player, tuple[str, str, TimeManager]]


@dataclass
class PairResult:
    opening: Opening
    snapshots: Snapshots

    def to_json(self):
        return {
            "opening": self.opening.to_json(),
            "snapshots": {player.name: snapshot.to_json() for player, snapshot in self.snapshots.items()},
        }

    @classmethod
    def from_json(cls, data) -> PairResult:
        snapshots = {Player[player]: GameSnapshot.from_json(snapshot) for player, snapshot in data["snapshots"].items()}
        return cls(Opening.from_json(data["opening"]), snapshots)


def player_wdl(result: GameResults) -> tuple[int, int, int]:
    return result[Player.TARGET], result[None], result[Player.BASE]


def color_wdl(result: GameResults) -> tuple[int, int, int]:
    return result[Color.BLACK], result[None], result[Color.WHITE]


@dataclass
class Engine:
    process: subprocess.Popen
    resource: TimeManager
    log_prefix_filter: tuple[str, ...]

    def command(self, command: str) -> str | None:
        return command_process(self.process, command, self.log_prefix_filter)

    def generate(self) -> str:
        started = time.perf_counter_ns()
        response = self.command("gen")
        elapsed_ms = (time.perf_counter_ns() - started) // 1_000_000
        if response is None:
            raise RuntimeError("engine did not report a search result")

        try:
            result = dict(field.strip().split("=", 1) for field in response.split(","))
            move = result["pos"]
            if self.resource.time_unit == TimeUnit.CLOCK:
                self.resource.consume(elapsed_ms)
            elif self.resource.total_remaining is not None:
                self.resource.consume(int(result["nodes"].removesuffix("K")))
        except (KeyError, ValueError) as error:
            raise RuntimeError(f"invalid search result: {response}") from error

        return move

    def apply_increment(self):
        if self.resource.total_remaining is not None:
            self.resource.total_remaining += self.resource.increment
            self.command(f"time total {self.resource.total_remaining}")


class UTCFormatter(logging.Formatter):
    converter = time.gmtime
    default_time_format = "%Y-%m-%dT%H:%M:%S"
    default_msec_format = "%s.%03dZ"


def configure_logging(level=logging.INFO):
    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(UTCFormatter("%(asctime)s %(levelname)s %(message)s"))
    logging.basicConfig(
        level=level,
        handlers=[handler],
        force=True,
    )


def pair_display(opening: Opening, snapshots: Snapshots) -> str:
    base, target = snapshots[Player.BASE], snapshots[Player.TARGET]
    return (
        f"duration={base.duration.total_seconds()}s-{target.duration.total_seconds()}s, "
        f"history={base.history}-{target.history}, "
        f"opening={opening.sequence}"
    )


def spawn_process(path, params, resource: TimeManager, opening: Opening) -> subprocess.Popen:
    resource_param = [
        "--time-unit", resource.time_unit.value,
        "--time-total", str(resource.total_remaining or 0),
        "--time-increment", str(resource.increment),
        "--time-turn", str(resource.turn or 0),
    ]

    return subprocess.Popen(
        [path]
            + shlex.split(params)
            + resource_param
            + ["--history", opening.sequence],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        start_new_session=True,
    )


def command_process(
        process, command: str, filter_prefix: tuple[str, ...],
) -> str | None:
    process.stdin.write(f"{command}\n")
    process.stdin.flush()

    while True:
        response = process.stdout.readline().rstrip("\r\n")

        if response == "=":
            return None

        if response.startswith("?"):
            raise RuntimeError(f"command: {command}, error: {response[2:]}")

        if response.startswith("=\x02"):
            chunks = []
            line = response[2:]
            while "\x03" not in line:
                chunks.append(line)
                line = process.stdout.readline()
            chunks.append(line.split("\x03", 1)[0])
            return "".join(chunks)

        if response.startswith("="):
            return response[2:]

        if response.startswith("% ") and response[2:].startswith(filter_prefix):
            logging.debug(response[2:])


def play_game(
        path_params_resource: PathParamsResource, draw_in: int, opening: Opening, first_player: Player,
        log_prefix_filter: tuple[str, ...],
) -> GameSnapshot:
    game_timer = time.perf_counter_ns()
    colors = {first_player: opening.initial_color, first_player.flip(): opening.initial_color.flip()}
    winner = None

    with ExitStack() as stack:
        engines = {}
        for player, (path, params, resource) in path_params_resource.items():
            process = stack.enter_context(spawn_process(path, params, resource, opening))
            stack.callback(process.terminate)
            engines[player] = Engine(process, copy.copy(resource), log_prefix_filter)

        player = first_player
        for _ in range(draw_in):
            engine = engines[player]
            move = engine.generate()
            if move == "none":
                winner = (player.flip(), colors[player.flip()])
                break

            result = engine.command(f"play {move}")
            opponent_result = engines[player.flip()].command(f"play {move}")

            if result != opponent_result:
                raise RuntimeError(f"player_winner={result}, opponent_winner={opponent_result}")

            if result is not None:
                if result not in ("draw", "full"):
                    winner_color = Color(result.split()[0].capitalize())
                    winner = next((side, color) for side, color in colors.items() if color == winner_color)
                break

            engine.apply_increment()
            player = player.flip()

        history = engines[Player.BASE].command("history")

        if logging.getLogger().isEnabledFor(logging.DEBUG):
            board = engines[Player.BASE].command("board")
            logging.debug(f"Game State:\n{board}")
            logging.debug(f"Game History: {history}")
            logging.debug(f"Game Finished: base={colors[Player.BASE]}, target={colors[Player.TARGET]}, win={winner}")

    duration = timedelta(microseconds=(time.perf_counter_ns() - game_timer) // 1_000)
    return GameSnapshot(winner=winner, duration=duration, history=history)


def play_pair(
        path_params_resource: PathParamsResource, draw_in: int, opening: Opening,
        log_prefix_filter: tuple[str, ...],
) -> PairResult:
    snapshots = {
        first_player: play_game(path_params_resource, draw_in, opening, first_player, log_prefix_filter)
        for first_player in Player
    }

    return PairResult(opening, snapshots)


def game_results(snapshots: Iterable[GameSnapshot]) -> GameResults:
    results = {
        Player.BASE: 0,
        Player.TARGET: 0,
        Color.BLACK: 0,
        Color.WHITE: 0,
        None: 0,
    }

    for snapshot in snapshots:
        if snapshot.winner is not None:
            for player_or_color in snapshot.winner:
                results[player_or_color] += 1
        else:
            results[None] += 1

    return results


pentanomial_score = {
    (0, 0, 2): 0,
    (0, 1, 1): 0.5,
    (0, 2, 0): 1,
    (1, 0, 1): 1,
    (1, 1, 0): 1.5,
    (2, 0, 0): 2,
}


def new_parser(default_time: list[int]) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(allow_abbrev=False)

    parser.add_argument("--seed", type=int, default=secrets.randbits(32))

    parser.add_argument("--rule", type=str, choices=[rule.value for rule in Rule], default=Rule.RENJU.value)

    parser.add_argument("--worker-addresses", type=str, nargs="+")

    base_source = parser.add_mutually_exclusive_group()
    base_source.add_argument("--base-path", type=str)
    base_source.add_argument("--base-ref", type=str, help="Base commit (default: patch commit or origin/master)")
    parser.add_argument("--base-patch", type=str, help="Base patch file")
    parser.add_argument("--base-params", type=str, default="")

    target_source = parser.add_mutually_exclusive_group()
    target_source.add_argument("--target-path", type=str)
    target_source.add_argument("--target-ref", type=str,
                               help="Target commit (default: patch commit or origin/master with a worktree patch)")
    parser.add_argument("--target-patch", type=str, help="Target patch file")
    parser.add_argument("--target-params", type=str, default="")

    parser.add_argument("--max-openings", type=int, default=100)
    parser.add_argument("--concurrency", type=int, default=2)

    parser.add_argument("--time-unit", type=str, choices=[unit.value for unit in TimeUnit], default=TimeUnit.CLOCK.value)
    parser.add_argument("--time", type=int, nargs=3, default=default_time, metavar=("TOTAL", "INCREMENT", "TURN"))

    parser.add_argument("--openings-file", type=str, required=True)
    parser.add_argument("--draw-in", type=int, default=225)

    parser.add_argument("--log-level", choices=["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"], default="INFO")
    parser.add_argument("--log-prefix-filter", type=str, nargs="*")

    return parser


def load_openings(config: Config) -> list[Opening]:
    with open(config.args.openings_file, "r") as openings_file:
        openings = [
            Opening(Color.BLACK if len(moves) % 2 == 0 else Color.WHITE, "".join(moves))
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
