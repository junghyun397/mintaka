import argparse
import base64
import hashlib
import logging
import re
import shutil
import subprocess
import tempfile
from contextlib import ExitStack
from dataclasses import dataclass
from pathlib import Path

import arena


def git(*args, cwd=None) -> bytes:
    try:
        return subprocess.check_output(["git", *args], cwd=cwd, stderr=subprocess.PIPE)
    except subprocess.CalledProcessError as error:
        message = error.stderr.decode(errors="replace").strip()
        raise RuntimeError(f"{error}\n{message}") from error


def resolve_commit(ref: str) -> str:
    return git("rev-parse", "--verify", "--end-of-options", f"{ref}^{{commit}}").decode().strip()


def fetch_master() -> str:
    git("fetch", "origin", "+master:refs/remotes/origin/master")
    return resolve_commit("origin/master")


@dataclass(frozen=True)
class Source:
    commit: str
    patch: bytes | None = None
    name: str | None = None

    def patch_hash(self) -> str | None:
        return hashlib.sha256(self.patch).hexdigest() if self.patch else None

    def key(self) -> str:
        patch_key = self.name or self.patch_hash()
        return f"{self.commit}-{patch_key}" if self.patch else self.commit

    def to_json(self):
        return {
            "commit": self.commit,
            "name": self.name,
            "patch_hash": self.patch_hash(),
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
    return path


def prepare_sources(args) -> dict[str, Source | str]:
    if not args.base_path or not args.target_path:
        fetch_master()

    sources = {}
    for name in ("base", "target"):
        path = getattr(args, f"{name}_path")
        ref = getattr(args, f"{name}_ref")
        patch = getattr(args, f"{name}_patch")
        if path:
            if patch:
                raise ValueError(f"--{name}-patch cannot be used with --{name}-path")
            sources[name] = path
        elif patch:
            sources[name] = Source.from_patch(Path(patch), ref)
        else:
            commit = resolve_commit(ref or "origin/master")
            if name == "target" and not ref:
                sources[name] = Source.from_worktree(commit)
            else:
                sources[name] = Source(commit)

    return sources


def build_binary(source: Source, rule: arena.Rule = arena.Rule.RENJU, *, use_worktree: bool = True) -> Path:
    key = source.key()
    patch_path = save_patch(key, source.patch) if source.patch else None

    binary_name = f"mintaka_text_protocol_{rule}"
    cached = Path("artifacts/engines") / f"{binary_name}-{key}"
    cached.parent.mkdir(parents=True, exist_ok=True)
    if cached.is_file():
        logging.info(f"Arena cache hit: {cached}")
        return cached

    logging.info(f"Arena building: {key}")
    with ExitStack() as stack:
        worktree = Path(".")
        target = Path("target")
        if use_worktree:
            target = (target / "arena").absolute()
            directory = stack.enter_context(tempfile.TemporaryDirectory(dir=cached.parent))
            worktree = Path(directory) / "source"
            git("worktree", "add", "--detach", str(worktree), source.commit)
            stack.callback(git, "worktree", "remove", "--force", str(worktree))
            if source.patch:
                git("apply", "--binary", "--whitespace=nowarn", str(patch_path.absolute()), cwd=worktree)
        subprocess.run(
            ["cargo", "build", "--release", "-p", "mintaka_interface",
             "--bin", binary_name, "--target-dir", str(target)],
            cwd=worktree, check=True,
        )
        shutil.copy2(target / "release" / binary_name, cached)
    return cached


def build_config(sources: dict[str, Source | str], settings: dict) -> arena.Config:
    rule = arena.Rule(settings["rule"])
    paths = {
        f"{name}_path": str(build_binary(source, rule)) if isinstance(source, Source) else source
        for name, source in sources.items()
    }
    return arena.Config(argparse.Namespace(**settings, **paths))
