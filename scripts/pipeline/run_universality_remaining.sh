#!/bin/bash
# run_universality_remaining.sh
# Resumes the Universality Test from where it was killed.

STEPS=20000
SEEDS=(1 2 3)
P_CREATE=0.64
GAMMA=2.2
MU=0.3

cd /home/saif/hcsn-nexus/hcsn-rust
cargo build --release

echo "========================================="
echo " Resuming Universality Test (Phase 5)    "
echo "========================================="

# Test 1: Standard
echo "Running Standard Rules..."
for seed in "${SEEDS[@]}"; do
    if [ ! -f "exports/universality/standard/particle_lifetimes_p${P_CREATE}_s${seed}.json" ]; then
        (
            HCSN_PATCHES=1 HCSN_GAMMA=$GAMMA HCSN_MU=$MU \
            cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
            > /dev/null 2>&1
            mkdir -p exports/universality/standard
            mv exports/particle_lifetimes_*_s${seed}.json exports/universality/standard/ 2>/dev/null
        ) &
    else
        echo "  -> Seed $seed already exists."
    fi
done
wait

# Test 2: High Noise Bias
echo "Running High Noise Bias..."
for seed in "${SEEDS[@]}"; do
    if [ ! -f "exports/universality/high_noise/particle_lifetimes_p${P_CREATE}_s${seed}.json" ]; then
        (
            HCSN_PATCHES=1 HCSN_GAMMA=$GAMMA HCSN_MU=$MU HCSN_NOISE_BIAS=0.2 \
            cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
            > /dev/null 2>&1
            mkdir -p exports/universality/high_noise
            mv exports/particle_lifetimes_*_s${seed}.json exports/universality/high_noise/ 2>/dev/null
        ) &
    else
        echo "  -> Seed $seed already exists."
    fi
done
wait

# Test 3: Low Geometry Freeze
echo "Running Low Geometry Freeze..."
for seed in "${SEEDS[@]}"; do
    if [ ! -f "exports/universality/low_freeze/particle_lifetimes_p${P_CREATE}_s${seed}.json" ]; then
        (
            HCSN_PATCHES=1 HCSN_GAMMA=$GAMMA HCSN_MU=$MU HCSN_GEOMETRY_FREEZE=0.3 \
            cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
            > /dev/null 2>&1
            mkdir -p exports/universality/low_freeze
            mv exports/particle_lifetimes_*_s${seed}.json exports/universality/low_freeze/ 2>/dev/null
        ) &
    else
        echo "  -> Seed $seed already exists."
    fi
done
wait

echo "Universality Test Complete."
