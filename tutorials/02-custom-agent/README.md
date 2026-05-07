# 02 — Custom Agent

Implement the `Suggestor` trait from scratch and watch the engine
schedule it correctly.

## Prereq

[`01-hello-convergence`](../01-hello-convergence) — you should be
comfortable with `Engine`, `Suggestor`, `ContextKey`, `ProposedFact`.

## What you'll learn

- The full `Suggestor` contract: `name`, `dependencies`, `accepts`,
  `execute`
- How `accepts()` controls firing — return `false` and the engine moves
  on, return `true` and it calls `execute()`
- How `AgentEffect` and `ProposedFact` thread structured proposals back
  into the context
- Why `accepts()` should be cheap and side-effect-free

## The setup

A `SeedOnceSuggestor` plants a fact, then a `SummaryAgent` reads
`ContextKey::Seeds`, synthesises a one-line summary, and emits it as a
`ContextKey::Hypotheses` fact. The engine sequences them automatically
because `SummaryAgent::dependencies()` declares `Seeds` as input.

The pattern — *one agent's output is another's input, declared via
context keys, scheduled by the engine* — is the same you'll see in
every later tutorial. Master it here.

## Run it

```sh
cargo run -p example-custom-agent
```

## Next

→ [`03-custom-provider`](../03-custom-provider) — extend Converge in
the *other* direction: plug your own LLM backend into the provider
interface.
