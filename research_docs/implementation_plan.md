# Phase 11: Production Optimization & Resilience

This phase focuses on improving the reliability and efficiency of the Phase 10b/c production runs based on user feedback. We will implement signal handling to prevent data loss on manual interrupts and reduce the step count per seed to 60k.

## Proposed Changes

### [Rust Engine]

#### [MODIFY] [Cargo.toml](file:///home/saif/antigravity/hcsn-rust/Cargo.toml)
- Add `ctrlc = "3.2"` to dependencies.

#### [MODIFY] [run_simulation.rs](file:///home/saif/antigravity/hcsn-rust/src/bin/run_simulation.rs)
- Implement `ctrlc` signal handler.
- Wrap the main simulation loop with a `running` flag (using `Arc<AtomicBool>`).
- Refactor data export logic into a reusable section that executes either at the end of the loop OR upon receiving a SIGINT.
- Ensure all worldline and interaction data are saved before the process exits.

### [Orchestration]

#### [MODIFY] [run_production.py](file:///home/saif/antigravity/hcsn-rust/run_production.py)
- Update `MAX_STEPS` from 100,000 to 60,000 to match the new optimized production target.
- Adjust the seed order to skip already completed runs.

## Verification Plan

### Automated Tests
1. **Build Test**: Run `cargo build --release` to ensure `ctrlc` dependency is resolved and the code compiles.
2. **Signal Test**: Start a short 10k step simulation and press Ctrl+C at step 5k. Verify that `exports/` contains the partial `particle_lifetimes_...json` and `interaction_events_...json` files.
3. **Production Run**: Execute `run_production.py` and verify Seed 3 correctly targets 60k steps.

### Manual Verification
- The user can verify that Seed 2 (currently at 88k+) finishes successfully to 100k (since it's close) or I can restart it with the new safe-exit handler.
- *Recommendation*: Let Seed 2 finish its 100k run (it is at 88k and will finish in <5 mins). Start the 60k optimized runs from Seed 3 onwards with the new signal handler.
