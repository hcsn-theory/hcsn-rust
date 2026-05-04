#!/bin/bash
set -e

MODES=("Baseline" "Pairwise" "StabilityScaled" "FluxCompensated" "MassCoupled" "TimeSymmetry")
STEPS=20000

echo "=== HCSN MULTI-MODE CONSERVATION SWEEP (v2.0) ==="

# Build the binary once
cargo build --release --bin force_law_aggregator

for MODE in "${MODES[@]}"; do
    echo ""
    echo ">>> Running Mode: $MODE"
    HCSN_CONSERVATION_MODE=$MODE HCSN_STEPS=$STEPS target/release/force_law_aggregator
    
    echo ">>> Auditing Mode: $MODE"
    HCSN_CONSERVATION_MODE=$MODE python3 scripts/conservation_audit_v2.py
done

echo ""
echo "=== Sweep Finalized. Check exports/audit_v2_*.png for results. ==="
