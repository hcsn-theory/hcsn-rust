#!/bin/bash
# run_conservation_test.sh
# Phase 4 P0: Conservation Test - Paired seeds with and without patches

SEEDS=(1 2 3 4 5)
STEPS=60000
P_CREATE=0.64
GAMMA=2.2
MU=0.3

# Build the project first
cargo build --release

echo "========================================="
echo " Starting P0 Conservation Test (Phase 4) "
echo "========================================="

# Run Baseline (With Patches)
echo "Running Baseline (Patches ON)..."
for seed in "${SEEDS[@]}"; do
    echo "  -> Starting Seed $seed"
    (
        HCSN_PATCHES=1 HCSN_GAMMA=$GAMMA HCSN_MU=$MU HCSN_EXPORT_MECHANISMS=1 \
        cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
        > /dev/null 2>&1
        
        # Move outputs to patched directory
        mkdir -p exports/conservation/patched
        mv exports/particle_lifetimes_*_s${seed}.json exports/conservation/patched/ 2>/dev/null
        mv exports/interaction_events_*_s${seed}.json exports/conservation/patched/ 2>/dev/null
        mv exports/hcsn_mechanisms_*_s${seed}.json exports/conservation/patched/ 2>/dev/null
    ) &
done
wait

# Run Test (Without Patches)
echo "Running Test (Patches OFF)..."
for seed in "${SEEDS[@]}"; do
    echo "  -> Starting Seed $seed"
    (
        HCSN_PATCHES=0 HCSN_GAMMA=$GAMMA HCSN_MU=$MU HCSN_EXPORT_MECHANISMS=1 \
        cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
        > /dev/null 2>&1
        
        # Move outputs to unpatched directory
        mkdir -p exports/conservation/unpatched
        mv exports/particle_lifetimes_*_s${seed}.json exports/conservation/unpatched/ 2>/dev/null
        mv exports/interaction_events_*_s${seed}.json exports/conservation/unpatched/ 2>/dev/null
        mv exports/hcsn_mechanisms_*_s${seed}.json exports/conservation/unpatched/ 2>/dev/null
    ) &
done
wait

echo "========================================="
echo " Simulation runs complete. Ready for analysis."
echo "========================================="
