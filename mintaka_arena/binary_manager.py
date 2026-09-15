import argparse
import base64
import hashlib
import shutil
import subprocess
import tempfile
from contextlib import ExitStack
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
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
    patch_created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def patch_hash(self) -> str | None:
        return hashlib.sha256(self.patch).hexdigest() if self.patch else None

    def key(self) -> str:
        if self.patch:
            created_at = self.patch_created_at
        else:
            commit_time = int(git("show", "-s", "--format=%ct", self.commit))
            created_at = datetime.fromtimestamp(commit_time, timezone.utc)

        timestamp = created_at.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        return f"{timestamp}-{self.commit}-{self.patch_hash()}"

    def to_json(self):
        return {
            "commit": self.commit,
            "patch_hash": self.patch_hash(),
            "patch": base64.b64encode(self.patch).decode("ascii") if self.patch else None,
            "patch_created_at": self.patch_created_at.isoformat(),
        }

    @classmethod
    def from_json(cls, data):
        patch = base64.b64decode(data["patch"]) if data["patch"] else None

        return cls(
            data["commit"], patch,
            datetime.fromisoformat(data["patch_created_at"]),
        )

    @classmethod
    def from_patch(cls, commit: str, path: Path):
        patch = path.read_bytes()
        if not patch:
            return cls(commit)

        created_at = datetime.fromtimestamp(path.stat().st_mtime, timezone.utc)
        suffix = f"-{commit}-{hashlib.sha256(patch).hexdigest()}"
        if path.name.startswith("patch-") and path.name.endswith(suffix):
            patch_time = path.name[6:-len(suffix)].removesuffix("Z")
            created_at = datetime.fromisoformat(patch_time).replace(tzinfo=timezone.utc)
        return cls(commit, patch, created_at)

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
    base_commit = resolve_commit(args.base_ref)
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


def build_binary(source: Source, *, use_worktree: bool = True) -> Path:
    key = source.key()
    patch_path = save_patch(key, source.patch) if source.patch else None

    binary_name = "mintaka_text_protocol_renju"
    cached = Path("artifacts/engines") / f"{binary_name}-{key}"
    cached.parent.mkdir(parents=True, exist_ok=True)
    if cached.is_file():
        print(f"Arena cache hit: {cached}", flush=True)
        return cached

    print(f"Arena building: {key}", flush=True)
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
             "--features=text-protocol", "--bin", binary_name, "--target-dir", str(target)],
            cwd=worktree, check=True,
        )
        shutil.copy2(target / "release" / binary_name, cached)
    return cached


def build_config(sources: dict[str, Source], settings: dict) -> "arena.Config":
    import arena

    paths = {f"{name}_path": str(build_binary(source)) for name, source in sources.items()}
    return arena.Config(argparse.Namespace(**settings, **paths))
