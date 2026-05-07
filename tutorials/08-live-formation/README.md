# 08 — Live Formation

A market-entry decision via a five-agent self-assembled team. Nothing
is pre-wired: providers and roles are selected at runtime, then
analysis, gating, and synthesis converge into one go/no-go.

## Prereq

[`07-adaptive-gap-loop`](../07-adaptive-gap-loop) — by now you've
seen multi-agent loops, optimisers, and intents. This tutorial puts
the full shape on display.

## What you'll learn

- The four-phase cadence Converge produces naturally:
  **self-assembly → analysis → gate + budget → synthesis**
- `FormationCatalog`, `DeliberatedFormationTemplate`,
  `FormationTemplateQuery` — how a formation gets picked from a
  catalog by intent shape
- Mock backends that satisfy `Backend` so you can see the wiring
  without a real LLM bill
- Why a constraint gate (`InvestmentGuard`) and a budget allocator
  belong in the same engine run as the LLM-style agents

## The setup

Five agents — `MarketAnalyser`, `TrendForecaster`, `CompetitiveScanner`,
`InvestmentGuard`, `BudgetAllocator`, `LaunchDirector` — register
into a single engine. Three mock backends declare their capabilities.
The team self-organises, runs the analysis, applies the gate, and
the director cites the formation + providers in its rationale.

This is the largest "Converge picking up the baton" tutorial in the
spine. Read the source — the macros at the top show how to declare a
mock backend in a few lines.

## Run it

```sh
cargo run -p example-live-formation
```

## Next

→ [`09-formation-mixed`](../09-formation-mixed) — heterogeneous
suggestors (optimiser + policy gate + LLM) in a single Engine run.
