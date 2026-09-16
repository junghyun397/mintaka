import logging
import arena
import binary_manager


def main():
    arena.configure_logging()
    master = binary_manager.fetch_master()
    source = binary_manager.Source.from_worktree(master)
    patch_path = binary_manager.save_patch(source.key(), source.patch) if source.patch else None
    logging.info(f"Arena patch: {patch_path}")
    logging.info(f"Arena engine: {binary_manager.build_binary(source, use_worktree=False)}")


if __name__ == "__main__":
    main()
