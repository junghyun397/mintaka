#!/usr/bin/env bash
set -euo pipefail

branch="$(date +%F)-${1:-$(date +%H%M%S)}"
git check-ref-format "refs/heads/$branch"

commit=$(git stash create "Checkpoint $branch")
commit=${commit:-$(git rev-parse HEAD)}

git push personal "$commit:refs/heads/$branch" >&2

printf "personal/%s@%s\n" "$branch" "$commit"