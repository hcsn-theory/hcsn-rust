#!/bin/bash
set -e

MODE="Hybrid"
STEPS=10000

echo "=== HCSN EMERGENCE AUDIT (v4.0) ==="

# Build the binary once
cargo build --release --bin force_law_aggregator

echo ""
echo ">>> Running Mode: $MODE"
HCSN_CONSERVATION_MODE=$MODE HCSN_STEPS=$STEPS target/release/force_law_aggregator

echo ">>> Performing Emergence Audit: $MODE"
HCSN_CONSERVATION_MODE=$MODE python3 scripts/emergence_audit_v4.py

echo ""
echo "=== Audit Finalized. Check exports/emergence_audit_v4_Hybrid.png for results. ==="
