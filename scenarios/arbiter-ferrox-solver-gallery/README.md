# Arbiter + Ferrox Solver Gallery

This scenario shows how solver-backed Suggestors and Cedar policy gates compose
inside one Converge run.

The default run uses portable surfaces:

- Ferrox greedy task scheduling
- Ferrox greedy job-shop scheduling
- Ferrox nearest-neighbor time-window routing
- Converge Hungarian assignment
- Converge min-cost flow
- Converge formation assembly
- Arbiter `PolicyGateSuggestor`
- Ferrox solver catalog and recommendation metadata

The optional native run also registers the Ferrox OR-Tools/HiGHS Suggestors:

- CP-SAT task scheduling
- CP-SAT job-shop scheduling
- CP-SAT time-window routing
- OR-Tools GLOP linear programming
- OR-Tools SimpleMinCostFlow
- OR-Tools general CP-SAT
- HiGHS MIP
- CP-SAT formation assembly

Run the portable example:

```sh
cargo run -p example-arbiter-ferrox-solver-gallery
```

Run the native example after the Ferrox native dependencies are built:

```sh
cargo run -p example-arbiter-ferrox-solver-gallery --features native-solvers
```

## Resource Declaration

**Trust label:** `LOCAL REAL / NO LIVE NETWORK`.

- Live external resources: **no**. This scenario does not call network
  providers.
- Mosaic extensions: atelier uses the real Arbiter and Ferrox crates. It does
  not replace those extensions with local mocks.
- Backend mode: the default run uses real portable in-process solver surfaces
  and catalog metadata. The `native-solvers` feature additionally registers
  real local OR-Tools/HiGHS-backed Ferrox suggestors where the native
  dependencies are available.
- Credentials / feature flags: no credentials. Use `--features native-solvers`
  for native local solver execution.
- Trust boundary: trust this as a solver-policy composition and catalog
  demonstration. Do not read it as a live external-service integration.

The important boundary is visible in the output: Ferrox supplies optimization
evidence and plans; Arbiter decides whether the selected plan may advance under
Cedar policy. SMT/SAT-style policy counterexample search remains a deferred
external lane in the catalog.
