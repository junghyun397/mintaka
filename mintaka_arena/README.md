# mintaka-arena

* All scripts must be run from **project root**.
* binary_manager.py creates patches based on origin/master.
* 

## Artifacts

* Patches: `artifacts/patches/patch-YYYY-MM-DDThh:mm:ssZ-<commit>-<patchhash>`
* Engines: `artifacts/engines/<enginename>-YYYY-MM-DDThh:mm:ssZ-<commit>-<patchhash>`

## Snapshot

```shell
python3 mintaka_arena/snapshot.py
```

## Arena Remote Worker

```shell
export ADDRESS=0.0.0.0 && export PORT=8095 && export CONCURRENCY=8
docker compose -f mintaka_arena/docker-compose.yml up --build -d
```

## sprt.py Sequential Probability Ratio Tester
with local resource
```shell
python3 mintaka_arena/sprt.py
--base-path target/release/mintaka_text_protocol_renju_base
--base-params "--workers 1 --memory-in-mib 32"
--target-path target/release/mintaka_text_protocol_renju_target
--target-params "--workers 1 --memory-in-mib 32"
--time 10000 300 0
# --target-nodes-in-1k 1000
# --base-nodes-in-1k 1000
--openings-file openings.csv
--max-openings 100
--concurrency 6
--elo0 0.0
--elo1 5.0
--alpha 0.05
--beta 0.05
--concise
````

## elo.py CI95 ELO Tester
with remote worker, include current uncommited changes
```shell
python3 mintaka_arena/elo.py
--base-ref f0e2e1a43449d314fee27349be732b039d395489
--base-patch artifacts/patches/patch-2026-08-25T10:25:00Z-f0e2e1a43449d314fee27349be732b039d395489-dbb4223897b131e480b1bb111a896d20ef4cce73272f01231570c0e487b4ca99
--base-params "--workers 1 --memory-in-mib 256"
--target-params "--workers 1 --memory-in-mib 256"
--time 120000 0 30000
--openings-file openings.csv
--min-openings 100
--max-openings 500
--concurrency 6
--base-elo 1000
--target-elo 100
--worker-addresses http://100.64.10.1:8086 local
--concise
```
