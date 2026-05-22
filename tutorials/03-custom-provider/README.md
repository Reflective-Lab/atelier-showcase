# 03 — Custom Provider

Plug your own backend into Converge's chat-provider interface. Same
shape a managed or local chat adapter would take, minus the network
call.

## Prereq

[`02-custom-agent`](../02-custom-agent) — you should know how the
engine drives agents. This tutorial swaps out *the LLM behind the
agents*, not the agents themselves.

## What you'll learn

- The `ChatBackend` trait — Converge's seam for any chat-completion
  provider
- The wire types: `ChatRequest`, `ChatResponse`, `ChatMessage`,
  `ChatRole`, `FinishReason`, `TokenUsage`, `LlmError`
- That a "provider" can be anything that returns a `ChatResponse` —
  here it's an echo, but the shape is identical for a real model
- Why provider and agent are separate concerns: agents reason about
  the *task*, providers reason about the *backend*

## The setup

`EchoBackend` implements `ChatBackend`: it concatenates user messages,
echoes them back, and reports plausible token counts. No async runtime,
no network — just a `std::future::Ready` so you can see the trait
shape without distraction.

In a real adapter the same trait would wrap an HTTP client, a local
inference process, or a managed-API SDK.

## Run it

```sh
cargo run -p example-custom-provider
```

## Next

→ [`04-intent-codec-loop`](../04-intent-codec-loop) — moves up a layer:
intents (the unit of work) and the codec that converts them to and
from wire form.
