# Dense matrix backend benchmark

This benchmark compares the current `DenseMatrix` implementation with `faer` and `nalgebra` using the same deterministic `f64` inputs.

## Run

```sh
cargo bench -p jabr-table-core --bench matrix_backend_compare --features backend-bench
```

The optional `backend-bench` feature keeps both candidate libraries out of normal builds and tests. `faer` is built without its default Rayon feature so the comparison is single-threaded. `nalgebra` is included as a dynamic matrix candidate and as a fixed-size 4x4 candidate.

## Workloads

| Shape | Representative use |
|---|---|
| 4x4 | 2D truss element stiffness |
| 6x6 | 2D frame/beam element stiffness |
| 12x12 | Small coupled element matrix |
| 24x24 | Higher-order element/local block |
| 64x64 | Dense upper-bound comparison, not a global FEM solver target |

Each backend receives identical row-major deterministic input values. The harness verifies all output entries against the native implementation before timing. Each result is the median of five samples, reported as nanoseconds per multiplication. Input construction is excluded from the timed section; output allocation and multiplication are included.

## Interpretation

- Run on an otherwise idle machine and record CPU, OS, Rust version, and power/performance mode alongside results.
- Compare 4x4, 6x6, and 12x12 first; these are the primary element-level workloads.
- Treat 24x24 and 64x64 as scaling signals, not proof that this crate should own global sparse FEM assembly.
- Do not select a backend from one machine or one benchmark. Repeat on Windows and Linux before changing the production matrix implementation.
- A backend must preserve numerical validation/error behavior and not force a dependency into default builds merely to improve a microbenchmark.

## Results

No benchmark result is committed yet. Run the command above on target hardware and append the raw output plus environment details here before making a backend decision.
