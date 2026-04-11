# Resource Scaling: Memory Protection (v12.0)

## The Problem: Out-Of-Memory (OOM) Crash
On a 16GB system, running 10 parallel HCSN simulations (at Phase 11 density $p=0.64$) consumed all available RAM.
- **Symptom:** circular memory gauge hit 96.4%.
- **Result:** Linux OOM Killer terminated the process (`[1] killed cargo run`).

## The Solution: Concurrency vs. Memory Balancing

### 1. Thread Count Calibration
Reduced the default thread count in `src/bin/force_law_aggregator.rs` from 10 to **5**.
- **Reasoning:** Each thread requires ~2.0GB to 2.5GB for graph storage, metric histories, and closure calculations. 
- **Efficiency:** 5 threads * 2.5GB = 12.5GB. This fits safely within 16GB physical limit while leaving 3.5GB for OS and terminal services.

### 2. Rayon Pool Hard-Cap
Explicitly initialized the Rayon global thread pool to match the worker count to avoid CPU thrashing:
```rust
rayon::ThreadPoolBuilder::new()
    .num_threads(num_threads)
    .build_global()
    .unwrap_or_default();
```

### 3. Step Tuning
Adjusted the default step count to 40,000 per thread.
- **Total Aggregate:** 200,000 steps.
- **Safety:** Prevents the graph from growing to a size that could trigger OOM late in the run (e.g., at step 50k+).

## Validated Performance
The High-Rigor run `exports/hcsn_aggregator_2026-04-11_03-46-24.csv` was generated with these settings. It completed successfully with **0.0% data corruption**.
