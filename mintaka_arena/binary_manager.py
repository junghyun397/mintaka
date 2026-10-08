import argparse
import base64
import hashlib
import logging
import re
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

from . import arena


def git(*args, cwd=None) -> bytes:
    try:
        return subprocess.check_output(["git", *args], cwd=cwd, stderr=subprocess.PIPE)
    except subprocess.CalledProcessError as error:
        message = error.stderr.decode(errors="replace").strip()
        raise RuntimeError(f"{error}\n{message}") from error


def resolve_commit(ref: str) -> str:
    return git("rev-parse", "--verify", "--end-of-options", f"{ref}^{{commit}}").decode().strip()


def fetch_master():
    git("fetch", "origin", "+master:refs/remotes/origin/master")


@dataclass(frozen=True)
class Source:
    commit: str
    patch: bytes | None = None
    name: str | None = None

    def key(self) -> str:
        if not self.patch:
            return self.commit
        return f"{self.commit}-{self.name or hashlib.sha256(self.patch).hexdigest()}"

    def cache_path(self, rule: arena.Rule) -> Path:
        return Path("artifacts/engines") / f"mintaka_text_protocol_{rule}-{self.key()}"

    def to_json(self):
        return {
            "commit": self.commit,
            "name": self.name,
            "patch": base64.b64encode(self.patch).decode("ascii") if self.patch else None,
        }

    @classmethod
    def from_json(cls, data):
        patch = base64.b64decode(data["patch"]) if data["patch"] else None

        return cls(data["commit"], patch, data.get("name"))

    @classmethod
    def from_patch(cls, path: Path, ref: str | None = None):
        match = re.fullmatch(r"patch-([0-9a-f]+)-(.+)", path.name)
        if match is None:
            raise ValueError(f"Cannot parse commit from patch filename: {path}")
        return cls(resolve_commit(ref or match[1]), path.read_bytes() or None, match[2])

    @classmethod
    def from_worktree(cls, commit: str, name: str | None = None):
        patch = git(
            "diff", "--binary", "--no-ext-diff", "--no-textconv", "--no-color",
            "--src-prefix=a/", "--dst-prefix=b/", commit, "--",
        )
        return cls(commit, patch or None, name)


def save_patch(key: str, patch: bytes) -> Path:
    path = Path("artifacts/patches") / f"patch-{key}"
    path.parent.mkdir(parents=True, exist_ok=True)

    if path.is_file():
        if path.read_bytes() != patch:
            raise FileExistsError(f"Patch already exists with different contents: {path}")
    else:
        path.write_bytes(patch)

    logging.info(f"Arena patch: {path}")
    return path


def prepare_sources(args, *, cache_only: bool = False) -> dict[arena.Player, Source | str]:
    if not cache_only and any(not getattr(args, f"{player}_path") for player in arena.Player):
        fetch_master()

    sources = {}
    for player in arena.Player:
        path = getattr(args, f"{player}_path")
        ref = getattr(args, f"{player}_ref")
        patch = getattr(args, f"{player}_patch")
        if path:
            if patch:
                raise ValueError(f"--{player}-patch cannot be used with --{player}-path")
            sources[player] = path
            continue

        if patch:
            source = Source.from_patch(Path(patch), ref)
        else:
            commit = resolve_commit(ref or "origin/master")
            if player == arena.Player.TARGET and not ref:
                source = Source.from_worktree(commit)
            else:
                source = Source(commit)

        sources[player] = str(source.cache_path(arena.Rule(args.rule))) if cache_only else source

    return sources


def build_binary(source: Source, rule: arena.Rule = arena.Rule.RENJU) -> Path:
    key = source.key()
    patch_path = save_patch(key, source.patch) if source.patch else None

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
                git("apply", "--binary", "--whitespace=nowarn", str(patch_path.absolute()), cwd=worktree)
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
