# 10 — Formation Compiler

Compile a deliberated formation template against a real catalog of
suggestors and providers, with explicit governance, replay, and
data-sovereignty requirements baked into the plan.

## Prereq

[`09-formation-mixed`](../09-formation-mixed) — by now you have a
mental model of formations as runtime constructs. This tutorial
shows the *compile step* that produces a concrete, executable plan.

## What you'll learn

- `FormationCompileRequest` — the input: template query + tenant +
  domain tag + intent
- `FormationCompilerCatalogs` — the catalog (suggestors + providers)
  the compiler picks from
- `BackendRequirements` with policy, replay, structured output,
  data sovereignty, compliance level — the machine-readable
  contract a provider must honour
- `vendor_selection_formation_catalog()` — a worked, opinionated
  catalog you can crib from for your own domain

## The setup

A vendor-selection F3 wedge. The request asks for a formation that
matches `vendor` + `diligence-evaluate-decide` keywords for a buyer
tenant. The catalog provides four suggestors (market scan, weighted
evaluator, policy gate, decision synthesis) and two providers (a
local Cedar engine, a reasoning LLM with EU sovereignty + high
explainability). The compiler emits a plan that wires them together.

This is the **bridge tutorial**. From 11 onward, the examples pull
in the organism layer (`organism-intent`, organism collaboration
shapes, charter derivation).

## Run it

```sh
cargo run -p example-formation-compiler
```

## Next

→ [`11-charter-from-intent`](../11-charter-from-intent) — organism
layer begins. The shape of collaboration is *derived* from intent
properties (stakes, reversibility, urgency).
