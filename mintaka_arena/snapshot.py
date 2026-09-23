import argparse
import logging
import arena
import binary_manager


def main():
    parser = argparse.ArgumentParser(allow_abbrev=True)

    parser.add_argument("--rule", type=str, choices=[rule.value for rule in arena.Rule], default=arena.Rule.RENJU.value)
    parser.add_argument("--target-ref", type=str)
    parser.add_argument("--name", type=str)

    args = parser.parse_args()

    arena.configure_logging()

    master = binary_manager.fetch_master()
    source = binary_manager.Source.from_worktree(master, args.name)
    patch_path = binary_manager.save_patch(source.key(), source.patch) if source.patch else None

    logging.info(f"Arena patch: {patch_path}")
    logging.info(f"Arena engine: {binary_manager.build_binary(source, rule=args.rule, use_worktree=False)}")


if __name__ == "__main__":
    main()
