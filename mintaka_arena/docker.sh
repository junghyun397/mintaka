read -ra ports <<< "$PORTS"
read -ra pools <<< "$AFFINITY_MATRIX"

for i in "${!pools[@]}"; do
    pool=${pools[i]}
    read -ra cpus <<< "${pool//,/ }"
    python3 -m mintaka_arena.arena_worker \
        --address "${ADDRESS:-0.0.0.0}" --port "${ports[i]}" \
        --affinity-pool "${cpus[@]}" &
done
wait
