import argparse
import base64
import hashlib
import logging
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

    def patch_hash(self) -> str | None:
        return hashlib.sha256(self.patch).hexdigest() if self.patch else None

    def key(self) -> str:
        patch_hash = self.patch_hash()
        return f"{self.commit}-{patch_hash}" if patch_hash else self.commit

    def to_json(self):
        return {
            "commit": self.commit,
            "patch_hash": self.patch_hash(),
            "patch": base64.b64encode(self.patch).decode("ascii") if self.patch else None,
        }

    @classmethod
    def from_json(cls, data):
        patch = base64.b64decode(data["patch"]) if data["patch"] else None

        return cls(data["commit"], patch)

    @classmethod
    def from_patch(cls, commit: str, path: Path):
        return cls(commit, path.read_bytes() or None)

    @classmethod
    def from_worktree(cls, commit: str):
        patch = git(
            "diff", "--binary", "--no-ext-diff", "--no-textconv", "--no-color",
            "--src-prefix=a/", "--dst-prefix=b/", commit, "--",
        )
        return cls(commit, patch or None)


def save_patch(key: str, patch: bytes) -> Path:
    path = Path("artifacts/patches") / f"patch-{key}"
    path.parent.mkdir(parents=True, exist_ok=True)

    if not path.is_file():
        path.write_bytes(patch)
    return path


def save_sources(sources: dict[str, Source]):
    for source in sources.values():
        if source.patch:
            save_patch(source.key(), source.patch)


def prepare_sources(args) -> dict[str, Source]:
    master = fetch_master()
    base_commit = resolve_commit(args.base_ref) if args.base_ref else master
    if args.base_patch:
        base = Source.from_patch(base_commit, Path(args.base_patch))
    else:
        base = Source(base_commit)

    if args.target_patch:
        target = Source.from_patch(resolve_commit(args.target_ref), Path(args.target_patch))
    else:
        target_commit = resolve_commit(args.target_ref) if args.target_ref else master
        target = Source.from_worktree(target_commit)

    sources = {"base": base, "target": target}
    save_sources(sources)
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
            target = target.absolute()
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


def build_config(sources: dict[str, Source], settings: dict) -> arena.Config:
    rule = arena.Rule(settings["rule"])
    paths = {f"{name}_path": str(build_binary(source, rule)) for name, source in sources.items()}
    return arena.Config(argparse.Namespace(**settings, **paths))
