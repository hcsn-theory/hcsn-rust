#!/bin/bash
# HCSN NATURAL SELECTION SCALING STUDY (v5.9.1 - Dual-Core Hyper-Flow)
set -e

STEPS_PER_THREAD=125000
P_CREATE=0.58
THREADS=2

# Hyper-Flow Stability & Performance Controls
export RAYON_NUM_THREADS=$THREADS
export HCSN_THREADS=$THREADS
export HCSN_P_CREATE=$P_CREATE
export HCSN_STEPS=$STEPS_PER_THREAD
export HCSN_EMERGENCE_MODE="Assisted"

echo "=== HCSN DUAL-CORE HYPER-FLOW SCALING STUDY (v5.9.1) ==="
echo "Target: $THREADS threads at $STEPS_PER_THREAD steps (Total 250k)"

# Build the binaries
cargo build --release

mkdir -p exports

echo ""
echo ">>> Initiating Parallel Scaling Study..."
target/release/force_law_aggregator

echo ""
echo ">>> Study Completed. Performing Aggregated Analytics..."
# The aggregator now generates uniquely timestamped files in exports/
# To run analytics, we'll identify the latest aggregator CSV
LATEST_CSV=$(ls -t exports/hcsn_aggregator_*.csv | head -n 1)

if [ -f "$LATEST_CSV" ]; then
    echo "Processing dataset: $LATEST_CSV"
    cp "$LATEST_CSV" exports/interaction_points_raw.csv
    python3 scripts/emergence_scaling_v5_2.py
else
    echo "Error: No timestamped dataset found in exports/"
fi

echo ""
echo "=== Dual-Core Study Finalized. Analytics generated. ==="
