# 04 — Intent Codec Loop

Where an intent (loose, declarative) becomes a viable formation
(roles + providers) at runtime, without you wiring the pipeline.

## Prereq

[`03-custom-provider`](../03-custom-provider) — you should know what
a backend is. This tutorial is the moment the platform pays off:
intents in, formations out.

## What you'll learn

- The **intent → formation** pattern: loose Gherkin-ish intent gets
  compiled into a `FormationRequest` (role coverage) and a
  `ProviderRequest` (backend capability coverage)
- The two stock suggestors that close the loop: `FormationAssemblySuggestor`
  and `ProviderSelectionSuggestor`
- Why "find a viable formation" beats "hardcode a pipeline" — the
  intent stays loose; Converge fills in the team
- The role/capability vocabulary: `SuggestorRole`, `Capability`,
  `LatencyClass`, `CostClass`

## The setup

A custom `IntentCodecSuggestor` translates a free-text intent into
formation and provider requests. The stock suggestors answer those
requests in the same engine run. Selected loop members emit their
role-specific outputs. The engine reaches a fixed point.

This is the smallest "everything wires itself together" example in
the spine. Tutorials from here on assume you've seen this loop.

## Run it

```sh
cargo run -p example-intent-codec-loop
```

The default run shows the original due-diligence loop. To run the applet-shaped
commercial proof path:

```sh
cargo run -p example-intent-codec-loop -- activate-subscription
cargo run -p example-intent-codec-loop -- refill-prepaid-ai-credits
```

That mode prints the functional, emotional, and relational JTBD frame before the
engine compiles the intent into formation and provider requests. It is a
showcase of how an applet stays small: Commerce Rails owns subscription,
entitlement, credit, and ledger consequence; Runtime Runway owns event ingress,
secrets, and telemetry; Helm owns approval and operator trust transfer.

## Next

→ [`05-fixed-point-vs-budget`](../05-fixed-point-vs-budget) — the two
ways a Converge run ends.
