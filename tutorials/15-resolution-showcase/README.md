# 15 — Resolution Showcase

Eight intents, eight matching dimensions. Each intent is designed to
light up exactly one dimension of the structural resolver, so you
can see precisely how organism decides which packs, capabilities,
and invariants an intent needs.

## Prereq

[`14-debate-loop`](../14-debate-loop) — you have the organism
vocabulary. This tutorial is a reference card for *how an intent
gets routed* before any debate even starts.

## What you'll learn

- `StructuralResolver` and its eight matching dimensions:
  fact prefix, constraint → invariant, context-key flow, capability,
  forbidden action, reversibility, authority, urgency
- `IntentBinding`, `DeclarativeBinding`, `IntentResolver` — the
  resolution result types
- `check_readiness` + `BudgetProbe` + `CredentialProbe` + `PackProbe`
  — readiness gates that explain *why* an intent isn't ready yet
- `GapSeverity` and `ReadinessReport` — how gaps surface

## The setup

Eight scenarios, each minimal:

1. **Fact prefix** — context contains `lead:` and `contract:`
2. **Constraint → invariant** — constraints reference invariants
3. **Context key flow** — Strategies must reach Hypotheses
4. **Capability matching** — required `social` capability
5. **Forbidden action** — `bypass_review` prohibits some packs
6. **Reversibility** — irreversible intents need higher authority
7. **Authority** — board approval needed
8. **Readiness** — credentials missing, gap reported

Read the example top-down — each scenario is a one-liner of intent
plus the dimension it exercises.

## Run it

```sh
cargo run -p example-resolution-showcase
```

## Next

→ [`16-collab-discussion`](../16-collab-discussion) — the four
collaboration shapes start here. Discussion is the loosest binding
shape: moderated, advisory, non-binding.
