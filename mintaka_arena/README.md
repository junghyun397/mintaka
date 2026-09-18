# mintaka-arena

* All scripts must be run from **project root**.
* binary_manager.py creates a patch based on origin/master.

## Artifacts

* Patches: `artifacts/patches/patch-<commit>-<patch_hash>`
* Engines: `artifacts/engines/<enginename>-<commit>[-<patch_hash>]` (patch hash only when a patch is present)

## Snapshot
```shell
python3 mintaka_arena/snapshot.py
```

## Arena Remote Worker

```shell
export ADDRESS=0.0.0.0 PORT=8095 CONCURRENCY=8
docker compose -f mintaka_arena/docker-compose.yml up --build -d
```

## sprt.py Sequential Probability Ratio Tester
```shell
python3 mintaka_arena/sprt.py
--rule Renju
--base-path artifacts/engines/mintaka_text_protocol_renju
--target-path target/release/mintaka_text_protocol_renju
--base-params "--workers 1 --memory-in-mib 32"
--target-params "--workers 1 --memory-in-mib 32"
--concurrency 8
--openings-file openings.csv
--time-unit Clock
--time 500 200 0
--max-openings 500
--elo0 -4.0
--elo1 8.0
--alpha 0.2
--beta 0.2
````

### SPRT Long
```shell
--time-unit Clock
--time 10000 300 0
--max-openings 2000
--elo0 0.0
--elo1 10.0
--alpha 0.1
--beta 0.1
```

## elo.py CI95 ELO Tester
```shell
python3 mintaka_arena/elo.py
--rule Renju
--base-path artifacts/engines/mintaka_text_protocol_renju
--target-path target/release/mintaka_text_protocol_renju
--base-params "--workers 1 --memory-in-mib 256"
--target-params "--workers 1 --memory-in-mib 256"
--concurrency 8
--openings-file openings.csv
--time-unit Clock
--time 120000 0 30000
--min-openings 500
--max-openings 4000
--base-elo 1000
--target-elo 1000
```

## Fixed-Node Test
```shell
--time-unit Nodes
--time 0 0 100000 # 100M/turn
```

## Remote Test
* Base: unspecified(origin/master ref and worktree patch) || ref || (ref && patch)
* Target: ref || (ref && patch)

```shell
--concurrency 172
--base-ref commit_hash
--base-patch artifacts/patches/patch-commit_hash-patch_hash
--worker-addresses http://100.80.10.10:8095 local
```
