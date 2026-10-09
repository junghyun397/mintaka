import argparse
import base64
import hashlib
import logging
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

from . import arena


def add_build_arguments(parser: argparse.ArgumentParser):
    parser.add_argument("--rule", type=str, choices=[rule.value for rule in arena.Rule], default=arena.Rule.RENJU.value)

    for player in arena.Player:
        ref_group = parser.add_mutually_exclusive_group()
        ref_group.add_argument(f"--{player}-path", type=str, help="exact engine binary path")
        ref_group.add_argument(f"--{player}-ref", type=str, help="REMOTE/BRANCH[@COMMIT] (default: origin/master)")

        parser.add_argument(f"--{player}-patch", type=str, help="exact patch file path")


def git(*args, cwd=None, input=None) -> bytes:
    return subprocess.check_output(["git", *args], cwd=cwd, input=input, stderr=subprocess.PIPE)


def resolve_commit(ref: str) -> str:
    return git("rev-parse", "--verify", "--end-of-options", f"{ref}^{{commit}}").decode().strip()


def parse_ref(value: str) -> tuple[str, str | None]:
    ref, separator, commit = value.partition("@")
    if "/" not in ref or (separator and not commit):
        raise ValueError(f"Expected REMOTE/BRANCH[@COMMIT], got: {value}")
    return ref, commit or None


def fetch(refs):
    for ref in dict.fromkeys(refs):
        remote, separator, branch = ref.partition("/")
        if not remote or not separator or not branch:
            raise ValueError(f"Expected REMOTE/BRANCH, got: {ref}")

        git("check-ref-format", f"refs/heads/{branch}")
        git("check-ref-format", f"refs/remotes/{ref}")
        git("fetch", "--", remote, f"+refs/heads/{branch}:refs/remotes/{ref}")


@dataclass(frozen=True)
class Source:
    ref: str
    commit: str
    patch: bytes | None = None

    def key(self) -> str:
        ref = self.ref.replace("/", "-")
        key = f"{ref}-{self.commit}"

        if self.patch:
            return f"{key}-{hashlib.sha256(self.patch).hexdigest()}"
        else:
            return key

    def cache_path(self, rule: arena.Rule) -> Path:
        return Path("artifacts/engines") / f"mintaka_text_protocol_{rule}-{self.key()}"

    def to_json(self):
        return {
            "ref": self.ref,
            "commit": self.commit,
            "patch": base64.b64encode(self.patch).decode("ascii") if self.patch else None,
        }

    @classmethod
    def from_json(cls, data):
        patch = base64.b64decode(data["patch"]) if data["patch"] else None
        return cls(data["ref"], data["commit"], patch)


def prepare_sources(args) -> dict[arena.Player, Source | str]:
    refs = {
        player: parse_ref(getattr(args, f"{player}_ref") or "origin/master")
        for player in arena.Player
        if not getattr(args, f"{player}_path")
    }
    fetch(ref for ref, _ in refs.values())

    sources = {}
    for player in arena.Player:
        if path := getattr(args, f"{player}_path"):
            sources[player] = path
            continue

        ref, commit = refs[player]
        commit = resolve_commit(commit or f"refs/remotes/{ref}")
        git("merge-base", "--is-ancestor", commit, f"refs/remotes/{ref}")
        patch = getattr(args, f"{player}_patch")
        patch_data = Path(patch).read_bytes() if patch else None
        sources[player] = Source(ref, commit, patch_data or None)

    return sources


def build_binary(source: Source, rule: arena.Rule = arena.Rule.RENJU) -> Path:
    key = source.key()

    binary_name = f"mintaka_text_protocol_{rule}"
    cached = source.cache_path(rule)
    if cached.is_file():
        logging.info(f"Arena cache hit: {cached}")
        return cached

    logging.info(f"Arena building: {key}")
    cached.parent.mkdir(parents=True, exist_ok=True)
    target = Path("target/arena").absolute()
    with tempfile.TemporaryDirectory(dir=cached.parent) as directory:
        worktree = Path(directory) / "source"
        git("worktree", "add", "--detach", str(worktree), source.commit)
        try:
            if source.patch:
                git("apply", "--binary", "--whitespace=nowarn", "-", cwd=worktree, input=source.patch)
            subprocess.run(
                ["cargo", "build", "--release", "-p", "mintaka_interface",
                 "--bin", binary_name, "--target-dir", str(target)],
                cwd=worktree, check=True,
            )
            shutil.copy2(target / "release" / binary_name, cached)
        finally:
            git("worktree", "remove", "--force", str(worktree))
    return cached


def build_config(sources: dict[arena.Player, Source | str], settings: dict) -> arena.Config:
    rule = arena.Rule(settings["rule"])
    paths = {
        f"{player}_path": str(build_binary(source, rule)) if isinstance(source, Source) else source
        for player, source in sources.items()
    }

    return arena.Config(argparse.Namespace(**settings, **paths))
