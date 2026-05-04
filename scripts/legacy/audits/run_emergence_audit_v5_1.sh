#!/bin/bash
set -e

STEPS=10000
P_CREATE=0.60

echo "=== HCSN COMPARATIVE EMERGENCE AUDIT (v5.1) ==="

# Build the binary
cargo build --release --bin force_law_aggregator

echo ""
echo ">>> RUN A: CONTROL (Natural Physics)"
HCSN_EMERGENCE_MODE="Control" HCSN_P_CREATE=$P_CREATE HCSN_STEPS=$STEPS HCSN_OUT_FILE="exports/v5_1_control.csv" target/release/force_law_aggregator

echo ""
echo ">>> RUN B: ASSISTED (Honest Emergence)"
HCSN_EMERGENCE_MODE="Assisted" HCSN_P_CREATE=$P_CREATE HCSN_STEPS=$STEPS HCSN_OUT_FILE="exports/v5_1_assisted.csv" target/release/force_law_aggregator

echo ""
echo ">>> RUN C: FORCED (Assumed Physics)"
HCSN_EMERGENCE_MODE="Forced" HCSN_P_CREATE=$P_CREATE HCSN_STEPS=$STEPS HCSN_OUT_FILE="exports/v5_1_forced.csv" target/release/force_law_aggregator

echo ""
echo ">>> Performing Comparative Phase Audit"
# We will create a combined audit script for this
python3 scripts/emergence_audit_v5_1.py

echo ""
echo "=== Audit Finalized. Check exports/comparative_phase_diagram_v5_1.png for results. ==="
