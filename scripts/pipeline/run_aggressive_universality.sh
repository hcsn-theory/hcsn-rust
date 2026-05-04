#!/bin/bash
# run_aggressive_universality.sh
# Tests if the Matter Phase survives when switching the fundamental EmergenceMode to Control (no structural bonus).

STEPS=20000
SEEDS=(1 2 3)
P_CREATE=0.64
GAMMA=2.2
MU=0.3

cd /home/saif/hcsn-nexus/hcsn-rust
cargo build --release

echo "========================================="
echo " Aggressive Universality Test (Phase 5)  "
echo "========================================="

echo "Running Control Rules (--aggressive_mode)..."
for seed in "${SEEDS[@]}"; do
    (
        HCSN_PATCHES=1 HCSN_GAMMA=$GAMMA HCSN_MU=$MU \
        cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed --aggressive_mode \
        > /dev/null 2>&1
        mkdir -p exports/universality/aggressive_control
        mv exports/particle_lifetimes_*_s${seed}.json exports/universality/aggressive_control/ 2>/dev/null
    ) &
done
wait

echo "Aggressive Universality Test Complete."
