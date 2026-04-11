# Kinematics Repair: Positional Normalization (v12.0)

## The Problem: Monotonic Vertex-ID Growth
The simulation used mean Vertex ID as a physical position proxy. Since IDs grow monotonically with every rewrite, the "position" increased indefinitely.
- **Result:** $v = (pos_{now} - pos_{prev}) / \Delta t$ reached values of $>10^{100}$.
- **Consequence:** Momentum ($p = m \cdot v$) overflowed to `Inf`, creating `NaN` when subtracted during conservation checks.

## The Solution: Double-Layer Guard

### 1. Position Normalization
In `src/rewrite_engine.rs`, we normalized the position by the current state of the global counter:
```rust
let max_id = h.vertices.keys().max().cloned().unwrap_or(1) as f64;
let mean_pos = sum_ids / (count * max_id);
```
- **Benefit:** Resulting `mean_pos` is strictly in $(0, 1]$.

### 2. Instantaneous Velocity
Switched from history-span velocity ($t_{birth} \to t_{now}$) to consecutive-frame velocity ($t_{n-1} \to t_{n}$):
```rust
let (t1, p1, _) = hist[hist.len()-2];
let (t2, p2, _) = hist[hist.len()-1];
let mut dv = (p2 - p1) / (t2 - t1);
```
- **Benefit:** Both $p_1$ and $p_2$ are normalized against nearly identical `max_id` values, ensuring a tiny, physically meaningful $\Delta p$.

### 3. Hard Safety Clamp
Added a physical ceiling to prevent any transients from poisoning the worldline:
```rust
dv = dv.clamp(-10.0, 10.0);
```

## Impact on Physics
- **Zero topology change**: Knot detection and rewrite evolution are untouched.
- **Restored Conservation**: Momentum corrections now function mathematically, enabling `Pairwise` and `Hybrid` modes for the first time in the Rust engine.
