branch="$(date +%Y%m%d)-${1:-$(date +%H%M)}"
commit=$(git stash create)

if [[ -z "$commit" ]]; then
    echo "No changes to checkpoint."
    exit 1
fi

git push --force archive "$commit:refs/heads/$branch"

printf "archive/%s@%s" "$branch" "$commit"
