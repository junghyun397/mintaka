import binary_manager


def main():
    master = binary_manager.fetch_master()
    source = binary_manager.Source.from_worktree(master)
    patch_path = binary_manager.save_patch(source.key(), source.patch) if source.patch else None
    print(f"Arena patch: {patch_path}", flush=True)
    print(f"Arena engine: {binary_manager.build_binary(source, use_worktree=False)}", flush=True)


if __name__ == "__main__":
    main()
