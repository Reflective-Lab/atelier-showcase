# 01 — Hello Convergence

The minimal Converge program. If this runs, your toolchain is wired up
correctly and you've seen the smallest end-to-end shape of the platform.

## What you'll learn

- `Engine` — the convergence loop that runs until the context stabilises
- `Suggestor` — the trait every agent implements (`accepts` + `execute`)
- `ContextKey` and `ProposedFact` — how agents communicate via the context
- **Idempotency** — agents check `accepts()` to decide whether to fire
- **Dependency-driven sequencing** — a downstream agent declares it
  depends on `ContextKey::Seeds` and the engine waits

## The setup

Two suggestors and a fixed-point engine. `SeedOnceSuggestor` writes a
single fact when no seeds exist yet. `ReactOnceSuggestor` waits for a
seed, then writes a single hypothesis. Both go quiet once their work is
done. The engine notices the context stopped changing and stops.

That stop condition — *the context is no longer changing* — is what
"convergence" means here.

## Run it

```sh
cargo run -p example-hello-convergence
```

Expected: two cycles, integrity report, the seed and hypothesis
printed back.

## Next

→ [`02-custom-agent`](../02-custom-agent) — write a more interesting
suggestor that reads facts and synthesises a summary.
