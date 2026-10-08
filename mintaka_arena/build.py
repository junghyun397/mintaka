import argparse
import logging

from . import arena
from . import binary_manager


def main():
    parser = argparse.ArgumentParser()

    arena.add_build_arguments(parser)
    parser.add_argument("--name", type=str)

    args = parser.parse_args()

    arena.configure_logging()

    rule = arena.Rule(args.rule)

    paths = {}
    for player, source in binary_manager.prepare_sources(args).items():
        if isinstance(source, binary_manager.Source):
            if player == arena.Player.TARGET and args.name:
                source = binary_manager.Source(source.commit, source.patch, args.name)

            path = binary_manager.build_binary(source, rule=rule)
        else:
            path = source

        logging.info(f"Arena {player} engine: {path}")
        paths[player] = path

    return paths


if __name__ == "__main__":
    main()
