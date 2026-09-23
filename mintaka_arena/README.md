# mintaka-arena

* All scripts must be run from **project root**.
* binary_manager.py creates a patch based on origin/master.

## binary_manager.py Artifacts

* Patch Out: `artifacts/patches/patch-<commit>-<patch_name>`
* Engine Out: `artifacts/engines/<enginename>-<commit>[-<patch_name>]`

## snapshot.py Snapshot Builder

```shell
python3 mintaka_arena/snapshot.py --name patch_name
```

* Patch Out: `artifacts/patches/patch-commit-patch_name`
* Engine Out: `artifacts/engines/patch-commit-patch_name`


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
--time 500 100 0
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

## Examples
* Base: unspecified(origin/master ref) || ref || (ref && patch)
* Target: unspecified(generate origin/master ref with a worktree patch) || ref || (ref && patch)

* origin/master HEAD vs. worktree, remote
```shell
--concurrency 172
--worker-addresses http://100.80.10.10:8095
```

* origin/master@commit vs. target patch, remote and local
```shell
--concurrency 182
--base-ref commit_hash
--target-patch artifacts/patches/patch-commit_hash-patch_name
--worker-addresses http://100.80.10.10:8095 local
```

* base patch vs. target patch, local
```shell
--concurrency 8
--base-ref commit_hash
--target-patch artifacts/patches/patch-commit_hash-patch_name
```
