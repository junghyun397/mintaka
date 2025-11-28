# mintaka-trainer
Machine learning pipeline for NNUE training

## arena.py ELO Tester
```shell
python3 arena.py \
--a_path target/release/mintaka_text_protocol_a --a_params "--workers 4 --memory_in_mib 1024" \
--b_path target/release/mintaka_text_protocol_b --b_params "--workers 4 --memory_in_mib 1024"
```

## optimize_params.py Parameter Optimizer
```shell
python3 optimize_params.py \
--engine-params "--workers 4 --memory_in_mib 1024" \
--target-param "FP_BASE" --start 60 --step 5 --end 120
```
