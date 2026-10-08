# mintaka-arena
All scripts must be run from **project root**.

## binary_manager.py

* Patch Out: `artifacts/patches/patch-<commit>-<patch_name>`
* Engine Out: `artifacts/engines/<enginename>-<commit>[-<patch_name>]`

## Arena Remote Worker

```shell
export ADDRESS=0.0.0.0 PORT=8095 CONCURRENCY=8
docker compose -f mintaka_arena/docker-compose.yml up --build -d
```

## sprt.py Sequential Probability Ratio Tester

```shell
python3 -m mintaka_arena.sprt
--concurrency 8
--base-path artifacts/engines/mintaka_text_protocol_renju
--target-ref commit_hash
--target-patch artifacts/patches/patch-commit_hash-patch_name
--base-params "--workers 1 --memory-in-mib 32"
--target-params "--workers 1 --memory-in-mib 32"
--affinity 1
--rule Renju
--openings-file openings/renju/openings.csv
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
python3 -m mintaka_arena.elo
--worker-addresses http://test-server-1:8095 http://test-server-2:8095 local
--concurrency 24
--base-ref commit_hash
--target-ref commit_hash
--base-params "--workers 1 --memory-in-mib 256"
--target-params "--workers 1 --memory-in-mib 256"
--affinity 1
--rule Renju
--openings-file openings/renju/openings.csv
--time-unit Clock
--time 120000 0 30000
--min-openings 500
--max-openings 4000
--base-elo 1000
--target-elo 1000
```
