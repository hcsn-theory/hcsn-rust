#!/bin/bash
set -e

MODE="Hybrid"
STEPS=20000

echo "=== HCSN HYBRID CONSERVATION AUDIT (v3.0) ==="

# Build the binary once
cargo build --release --bin force_law_aggregator

echo ""
echo ">>> Running Mode: $MODE"
HCSN_CONSERVATION_MODE=$MODE HCSN_STEPS=$STEPS target/release/force_law_aggregator

echo ">>> Performing Energy Audit: $MODE"
HCSN_CONSERVATION_MODE=$MODE python3 scripts/energy_audit_v3.py

echo ""
echo "=== Audit Finalized. Check exports/audit_v3_Hybrid.png for results. ==="
