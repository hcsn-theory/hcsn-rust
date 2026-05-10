#!/bin/bash

# HCSN Dual-Universe Parallel Experiment Runner
# Goal: Compare Baseline (Pure) vs Hybrid (Patched) dynamics at 100,000 steps.

STEPS=100000
SEED=12345
BASE_DIR="exports/experiment_$(date +%Y%m%d_%H%M%S)"

echo "===================================================================="
echo "HCSN LARGE SCALE EXPERIMENT: 100,000 STEPS"
echo "===================================================================="
echo "Baseline: HCSN_PATCHES=0"
echo "Hybrid:   HCSN_PATCHES=1"
echo "Output Directory: $BASE_DIR"
echo "===================================================================="

mkdir -p "$BASE_DIR/baseline"
mkdir -p "$BASE_DIR/hybrid"

# Function to run simulation
run_sim() {
    local mode=$1
    local patches=$2
    local out_dir=$3
    
    echo "[LAUNCH] Starting $mode Universe (Patches=$patches)..."
    
    # We move into hcsn-rust to run cargo
    cd hcsn-rust || exit
    
    HCSN_PATCHES=$patches \
    HCSN_EXPORT_MECHANISMS=1 \
    cargo run --release --bin run_simulation -- \
        --steps $STEPS \
        --seed $SEED \
        --log-to "../$out_dir/sim_log.jsonl" > "../$out_dir/stdout.log" 2>&1
    
    # Move results to the specific experiment folder
    mv exports/*.json "../$out_dir/" 2>/dev/null
    
    cd ..
    echo "[FINISH] $mode Universe completed."
}

# Run in parallel
run_sim "Baseline" 0 "$BASE_DIR/baseline" &
PID_BASE=$!

run_sim "Hybrid" 1 "$BASE_DIR/hybrid" &
PID_HYB=$!

echo "[MONITOR] Baseline PID: $PID_BASE"
echo "[MONITOR] Hybrid PID:   $PID_HYB"
echo "Waiting for simulations to complete. This may take several hours..."

wait $PID_BASE
wait $PID_HYB

echo "===================================================================="
echo "SIMULATIONS COMPLETE. Starting Analysis..."
echo "===================================================================="

python3 hcsn-rust/scripts/analyze_results.py "$BASE_DIR"

echo "===================================================================="
echo "EXPERIMENT FINISHED. See $BASE_DIR/EXPERIMENT_REPORT.md"
echo "===================================================================="
