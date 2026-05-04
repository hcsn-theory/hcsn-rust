#!/bin/bash
set -e

MODE="Hybrid"
STEPS=10000

echo "=== HCSN EMERGENCE PHASE AUDIT (v5.0) ==="

# Build the binary once
cargo build --release --bin force_law_aggregator

echo ""
echo ">>> Running Mode: $MODE (100k aggregate steps)"
HCSN_CONSERVATION_MODE=$MODE HCSN_STEPS=$STEPS target/release/force_law_aggregator

echo ">>> Performing Phase Audit: $MODE"
HCSN_CONSERVATION_MODE=$MODE python3 scripts/emergence_audit_v5.py

echo ""
echo "=== Audit Finalized. Check exports/emergence_phase_diagram_v5_Hybrid.png for results. ==="
