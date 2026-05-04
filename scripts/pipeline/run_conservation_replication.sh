#!/bin/bash
# run_conservation_replication.sh
# Runs 10 new seeds with varying steps to replicate the emergent conservation findings.

SEEDS_10K=(6 7 8)
SEEDS_30K=(9 10 11)
SEEDS_60K=(12 13 14 15)

P_CREATE=0.64
GAMMA=2.2
MU=0.3

cd /home/saif/hcsn-nexus/hcsn-rust
cargo build --release

echo "========================================="
echo " Phase 4 Replication: Unpatched Seeds    "
echo "========================================="

mkdir -p exports/conservation/replication

# 10K steps
echo "Running 10K step seeds..."
for seed in "${SEEDS_10K[@]}"; do
    (
        HCSN_PATCHES=0 HCSN_GAMMA=$GAMMA HCSN_MU=$MU HCSN_EXPORT_MECHANISMS=1 \
        cargo run --release --bin run_simulation -- --steps 10000 --p_create $P_CREATE --seed $seed \
        > /dev/null 2>&1
        mv exports/interaction_events_*_s${seed}.json exports/conservation/replication/ 2>/dev/null
    ) &
done
wait

# 30K steps
echo "Running 30K step seeds..."
for seed in "${SEEDS_30K[@]}"; do
    (
        HCSN_PATCHES=0 HCSN_GAMMA=$GAMMA HCSN_MU=$MU HCSN_EXPORT_MECHANISMS=1 \
        cargo run --release --bin run_simulation -- --steps 30000 --p_create $P_CREATE --seed $seed \
        > /dev/null 2>&1
        mv exports/interaction_events_*_s${seed}.json exports/conservation/replication/ 2>/dev/null
    ) &
done
wait

# 60K steps
echo "Running 60K step seeds..."
for seed in "${SEEDS_60K[@]}"; do
    (
        HCSN_PATCHES=0 HCSN_GAMMA=$GAMMA HCSN_MU=$MU HCSN_EXPORT_MECHANISMS=1 \
        cargo run --release --bin run_simulation -- --steps 60000 --p_create $P_CREATE --seed $seed \
        > /dev/null 2>&1
        mv exports/interaction_events_*_s${seed}.json exports/conservation/replication/ 2>/dev/null
    ) &
done
wait

echo "Replication runs complete."
