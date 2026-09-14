# mintaka-trainer

## Arena

### sprt.py Sequential Probability Ratio Tester
```shell
python3 sprt.py 
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

### elo.py CI95 ELO Tester
```shell
python3 elo.py
--base-path target/release/mintaka_text_protocol_renju_base
--base-params "--workers 1 --memory-in-mib 256"
--target-path target/release/mintaka_text_protocol_renju_target
--target-params "--workers 1 --memory-in-mib 256"
--time 120000 0 30000
# --target-nodes-in-1k 1000
# --base-nodes-in-1k 1000
--openings-file openings.csv
--min-openings 100
--max-openings 500
--concurrency 6
--base-elo 1000
--target-elo 100
--concise
```
