# Scenario: quorum-open-inquiry

End-to-end exercise of Quorum's applet-manifest spine using the *same*
manifest + flavor files Quorum ships.

## What this proves

The Plan 1 (Track 0) spine in `marquee-apps/quorum-sense/` produces a
working pipeline from:

1. an Axiom-validated JTBD manifest (`open-adaptive-inquiry.applet.json`),
2. a Quorum-owned flavor (`tough-decision-v1.flavor.json`),
3. a host's session-start input (`HostSessionInput`),

to a runtime `InquiryContract` ready for the Quorum kernel — and that
pipeline works **outside** Quorum's own test suite, consuming the
manifest + flavor + mapper as published crate surfaces.

No parallel fixtures. No mocks. No separate copy of the manifest in
atelier-showcase.

## Run it

```sh
cargo run -p example-quorum-open-inquiry
```

Expected: four steps print, the contract's `evidence_requirements`
includes `manifest:quorum.open-adaptive-inquiry`, exit 0.

## Resource Declaration

**Trust label:** `LOCAL REAL / NO LIVE NETWORK`.

- Live external resources: **no**. This scenario does not call network
  providers, LLM services, or external APIs.
- Mosaic extensions: atelier uses the real Quorum crate and manifest
  mapper. It does not replace those with local mocks.
- Backend mode: in-process applet-manifest spine and contract instantiation
  only.
- Credentials / feature flags: none.
- Trust boundary: trust this as a pipeline proof for Quorum's Track 0 spine.
  Do not read it as evidence that a live round-loop with extraction or LLM
  judgment is ready.

## What's deliberately NOT here

- HTTP routes / live transport (Plan 3)
- Stripe / Firebase / Commerce Rails (Plans 3/4)
- A real round-loop with LLM extraction (the rulebook + suggestors arrive
  in a later plan; this scenario stops at contract instantiation, which is
  the spine's scope)
- Atlas consumption of `quorum://` citations (separate proof, lives in
  atlas-integration's own smoke)

## See also

- `marquee-apps/quorum-sense/docs/superpowers/specs/2026-06-06-quorum-shippable-v1-design.md`
- `marquee-apps/quorum-sense/docs/superpowers/plans/2026-06-07-track-0-applet-manifest-spine.md`
- `marquee-apps/quorum-sense/docs/superpowers/plans/2026-06-07-track-0c-0d-citation-and-atelier-example.md`
