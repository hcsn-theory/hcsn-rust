# Command History & Reproduction Log (v12.0)

This log contains the serialized sequence of commands required to replicate the Phase 12 validation.

## 1. Environment Check
Check hardware threads and RAM availability.
```bash
nproc
free -h
```

## 2. Build & Test Run
Recompile the engine and aggregator after applying the kinematics fix.
```bash
cargo build --release --bin force_law_aggregator
```

## 3. High-Rigor Data Generation
Run the aggregator with optimized 16GB RAM settings (5 threads, 40k steps).
```bash
cargo run --release --bin force_law_aggregator
```

## 4. Data Quality Audit
Verify the absence of NaNs and Infs in the exported CSV.
```bash
python3 - << 'EOF'
import csv, math
filepath = "exports/hcsn_aggregator_2026-04-11_03-46-24.csv"
valid = 0
total = 0
with open(filepath) as f:
    reader = csv.DictReader(f)
    for row in reader:
        total += 1
        if not any(v.strip().lower() in ('nan','inf') for v in row.values()):
            valid += 1
print(f"Audit: {valid}/{total} valid rows.")
EOF
```

## 5. Interaction Analysis
Run the multi-dimensional analyzer to check stability coupling.
```bash
# Requires interaction_points_raw.csv in exports/
cargo run --release --bin force_law_analyzer
```

## 6. Functional Fitting
Fit the sigmoidal and peaked-exponential models to the chi-dp distribution.
```bash
cargo run --release --bin force_law_fit
```
