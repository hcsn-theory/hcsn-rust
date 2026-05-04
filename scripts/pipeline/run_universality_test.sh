#!/bin/bash
# run_universality_test.sh
# Phase 5: Universality Test - testing robustness of the phase structure against rule variations

STEPS=20000
SEEDS=(1 2 3)
P_CREATE=0.64
GAMMA=2.2
MU=0.3

# Build the project
cargo build --release

echo "========================================="
echo " Starting Universality Test (Phase 5)    "
echo "========================================="

# Test 1: Standard
echo "Running Standard Rules..."
for seed in "${SEEDS[@]}"; do
    (
        HCSN_PATCHES=1 HCSN_GAMMA=$GAMMA HCSN_MU=$MU \
        cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
        > /dev/null 2>&1
        mkdir -p exports/universality/standard
        mv exports/particle_lifetimes_*_s${seed}.json exports/universality/standard/ 2>/dev/null
    ) &
done
wait

# Test 2: High Noise Bias
echo "Running High Noise Bias..."
for seed in "${SEEDS[@]}"; do
    (
        HCSN_PATCHES=1 HCSN_GAMMA=$GAMMA HCSN_MU=$MU HCSN_NOISE_BIAS=0.2 \
        cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
        > /dev/null 2>&1
        mkdir -p exports/universality/high_noise
        mv exports/particle_lifetimes_*_s${seed}.json exports/universality/high_noise/ 2>/dev/null
    ) &
done
wait

# Test 3: Low Geometry Freeze
echo "Running Low Geometry Freeze..."
for seed in "${SEEDS[@]}"; do
    (
        HCSN_PATCHES=1 HCSN_GAMMA=$GAMMA HCSN_MU=$MU HCSN_GEOMETRY_FREEZE=0.3 \
        cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
        > /dev/null 2>&1
        mkdir -p exports/universality/low_freeze
        mv exports/particle_lifetimes_*_s${seed}.json exports/universality/low_freeze/ 2>/dev/null
    ) &
done
wait

echo "========================================="
echo " Universality Test complete. "
echo "========================================="
