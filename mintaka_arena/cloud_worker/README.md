# cloud-worker Cloud Tester

## Build
```shell
mkdir -p mintaka_arena/cloud_worker/transfer
python3 -m mintaka_arena.cloud_worker.build_docker \
--platform linux/amd64 \
--target-cpu znver5 \
--base-ref origin/master@commit_hash \
--target-ref $(mintaka_arena/checkpoint.sh) \
--target-cpu znver5 \
> mintaka_arena/cloud_worker/transfer/arena_cache.tar
```

## Run
Don't specify --worker-addresses, --concurrency, --base-path, --target-path.

```shell
python3 -m mintaka_arena.cloud_worker.run \
--project mintaka-42 \
--cache mintaka_arena/cloud_worker/transfer/arena_cache.tar \
--region us-central1 \
--machine-type c4d-highcpu-8 \
--concurrency 20 \
--suit elo \
-- \
--rule Renju \
--base-params "--workers 1 --memory-in-mib 256" \
--target-params "--workers 1 --memory-in-mib 256" \
--affinity 1 \
--openings-file openings/renju/openings.csv \
--time-unit Clock \
--time 180000 0 30000 \
--timeout-start 180 \
--timeout-play 180 \
--min-openings 800 \
--max-openings 960 \
--base-elo 1000 \
--target-elo 1000
```

## Safety
```shell
gcloud compute instances list \
--project=mintaka-42
```
