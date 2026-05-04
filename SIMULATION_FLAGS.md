# HCSN Simulation Guide: Flags and Toggles

This document serves as a comprehensive guide to the environment variables and command-line interface (CLI) flags that control the HCSN (Hierarchical Causal Structure Networks) simulation engine. 

The HCSN engine is a rigorous scientific instrument designed to test the emergence of physical laws from topological structure. By toggling these parameters, you can test various regimes of the universe.

---

## Environment Variables (The Physics Rules)

These variables must be prefixed before running the simulation (e.g., `HCSN_PATCHES=0 cargo run ...`).

### `HCSN_PATCHES`
- **What it does**: Enables or disables the engineered "conservation patches" (Hypotheses A-E, such as Inertial Cooling, Mass Coupling, and Local Flux Compensation).
- **Default (Normal)**: `1` (ON)
- **Scientific Use**: Set this to `0` to run a pure, unpatched universe. This is required to test if momentum conservation emerges naturally from topological structure rather than being forced by ad hoc rules.

### `HCSN_GAMMA`
- **What it does**: Controls the non-linear coupling constant (`γ`), dictating how strongly structural coherence influences a node's survival probability.
- **Default (Normal)**: `2.2`
- **Scientific Use**: Sweep this value (e.g., from `1.0` to `4.0`) to locate the critical phase transition boundary (`γ*`) where the universe condenses from vacuum into matter.

### `HCSN_MU`
- **What it does**: Sets the memory coupling parameter (`μ`), establishing the topological "mass" threshold required to protect structures from spontaneous destruction.
- **Default (Normal)**: `0.3`

### `HCSN_NOISE_BIAS`
- **What it does**: Injects random stochastic noise into the rewrite selection rules.
- **Default (Normal)**: `0.0`
- **Scientific Use**: Increase this (e.g., to `0.2`) to test the "Universality" of the matter phase under highly elevated background noise.

### `HCSN_GEOMETRY_FREEZE`
- **What it does**: Controls the distance memory decay (`ρ`), which alters how rigid or malleable the underlying hypergraph geometry is.
- **Default (Normal)**: `0.9`
- **Scientific Use**: Decrease this (e.g., to `0.3`) to test if matter still condenses when the universe's geometry is highly fluid.

### `HCSN_EXPORT_MECHANISMS`
- **What it does**: Tells the engine to dump the internal structural stats (stability, coherence, suppression, memory) for every node upon its destruction.
- **Default (Normal)**: `0` (OFF)
- **Scientific Use**: Set to `1` when running Phase 1 Correlation tests. This outputs the `hcsn_mechanisms_*.json` datasets required for Random Forest PCA analysis.

---

## CLI Arguments (The Execution Parameters)

These arguments are passed directly to the executable (e.g., `cargo run --release --bin run_simulation -- --steps 60000`).

### `--steps <NUMBER>`
- **What it does**: The total number of causal rewrites (ticks of time) to perform. This also implicitly dictates the maximum potential volume of the universe.
- **Default (Normal)**: `5000`
- **Scientific Use**: For rigorous replication, vary this across `10000`, `30000`, and `60000` to ensure phenomena are scale-invariant.

### `--p_create <FLOAT>`
- **What it does**: The baseline probability of creating a new hyperedge rather than destroying an existing one.
- **Default (Normal)**: `0.60`
- **Scientific Use**: We typically lock this at `0.64` for Phase Diagram and Conservation tests to ensure the network grows but remains volatile enough to allow selection pressure.

### `--seed <NUMBER>`
- **What it does**: Sets the deterministic seed for the random number generator (RNG).
- **Default (Normal)**: `1`
- **Scientific Use**: Change the seed to ensure that your physics results are not artifacts of a specific, lucky topological history. Replication suites typically test seeds 1 through 15.

### `--aggressive_mode`
- **What it does**: A boolean flag (presence means true). If included, the engine completely drops out of the `Assisted` emergence mode and into the brutal `Control` mode, entirely disabling structural inheritance bonuses.
- **Default (Normal)**: `false` (Absent)
- **Scientific Use**: Use this for extreme Aggressive Universality testing. If the matter phase survives in this regime, it proves that matter emergence is a fundamental feature of the topology, not a result of "friendly" inheritance mechanics.

### `--log-to <FILE_PATH>`
- **What it does**: Redirects the standard JSONL simulation logs to a specific file.
- **Default (Normal)**: Auto-discovers the `gantry` directory if it exists, otherwise logs locally.
