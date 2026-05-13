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

The important boundary is visible in the output: Ferrox supplies optimization
evidence and plans; Arbiter decides whether the selected plan may advance under
Cedar policy. SMT/SAT-style policy counterexample search remains a deferred
external lane in the catalog.
