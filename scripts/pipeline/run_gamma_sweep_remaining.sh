#!/bin/bash
# run_gamma_sweep_remaining.sh
# Resumes the Gamma Sweep from where it was killed.

GAMMA_VALUES=(2.2 2.5 3.0 4.0)
SEEDS=(1 2 3 4 5)
STEPS=60000
P_CREATE=0.64
MU=0.3

cd /home/saif/hcsn-nexus/hcsn-rust
cargo build --release

echo "========================================="
echo " Resuming Gamma Criticality Sweep        "
echo "========================================="

for gamma in "${GAMMA_VALUES[@]}"; do
    echo "Running for GAMMA = $gamma..."
    mkdir -p exports/gamma_${gamma}
    
    for seed in "${SEEDS[@]}"; do
        # Skip if already generated
        if [ -f "exports/gamma_${gamma}/particle_lifetimes_p${P_CREATE}_s${seed}.json" ]; then
            echo "  -> Seed $seed already exists. Skipping."
            continue
        fi

        echo "  -> Starting Seed $seed"
        (
            HCSN_GAMMA=$gamma HCSN_MU=$MU HCSN_PATCHES=1 \
            cargo run --release --bin run_simulation -- --steps $STEPS --p_create $P_CREATE --seed $seed \
            > /dev/null 2>&1
            
            mv exports/particle_lifetimes_*_s${seed}.json exports/gamma_${gamma}/ 2>/dev/null
            mv exports/interaction_events_*_s${seed}.json exports/gamma_${gamma}/ 2>/dev/null
        ) &
    done
    wait
done

echo "Gamma Sweep Complete."
