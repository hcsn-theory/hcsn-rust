#!/bin/bash
# run_gamma_sweep.sh
# Phase 3: Gamma Criticality Sweep

GAMMA_VALUES=(1.0 1.3 1.6 2.0 2.2 2.5 3.0 4.0)
SEEDS=(1 2 3 4 5)
STEPS=60000
P_CREATE=0.64
MU=0.3

# Build the project
cargo build --release

echo "========================================="
echo " Starting Gamma Criticality Sweep (Phase 3) "
echo "========================================="

for gamma in "${GAMMA_VALUES[@]}"; do
    echo "Running for GAMMA = $gamma..."
    mkdir -p exports/gamma_${gamma}
    
    for seed in "${SEEDS[@]}"; do
        echo "  -> Seed $seed"
        HCSN_GAMMA=$gamma HCSN_MU=$MU HCSN_PATCHES=1 \
        cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
        > /dev/null 2>&1
        
        # Move outputs
        mv exports/particle_lifetimes_*_s${seed}.json exports/gamma_${gamma}/
        mv exports/interaction_events_*_s${seed}.json exports/gamma_${gamma}/
    done
done

echo "========================================="
echo " Gamma Sweep complete. Ready for Phase 3 analysis."
echo "========================================="
