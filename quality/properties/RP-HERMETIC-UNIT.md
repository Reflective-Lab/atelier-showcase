---
tags: [quality, property, hermeticity]
property: RP-HERMETIC-UNIT
status: Aspired
enforced-by: arena-tests/crates/dim-hermeticity, runtime test-harness hooks
source: human + LLM
---

# RP-HERMETIC-UNIT — Unit tests issue zero outbound network requests

## What this property asserts

A unit test under `cargo test` (or `cargo nextest`) issues:

- **zero** outbound TCP/UDP connections,
- **zero** reads of API-key-shaped env vars (`*_API_KEY`, `*_TOKEN`,
  `*_SECRET`, `*_CREDENTIAL`),
- **zero** filesystem writes outside `TempDir`-scoped paths.

If you need any of those, you have an integration test, not a unit
test. Move it. Tag it. Skip it by default. Don't smuggle it.

## Why this property exists — the cost we already paid

[QF-2026-06-02-05](../incidents/QF-2026-06-02-05.md). Axiom v0.15.1
shipped a unit test
(`guide_heading_falls_back_to_local_on_no_backend`) that read
`OPENAI_API_KEY` from the developer's `.envrc`, opened a real TLS
session to `api.openai.com`, asked an LLM to evaluate a Truth
heading, and asserted on the response. It "passed" on machines
without keys (because the live path errored and fell back to local
heuristics, which is what the test asserted). On machines *with*
keys, it failed *and* burned API credit. Nobody noticed for the
weeks the test had been in the suite, because the only people
running it were dev machines without keys.

What the failure cost:
- Billable API calls during routine `cargo test` runs.
- Asymmetric pass/fail across developer machines — every PR review
  reading "the test passes for me" was getting different evidence
  from CI.
- A near-incident where a release rehearsal hit the failing version
  of the test and the diagnosis took 30 minutes.

What the failure taught us:
- `from_env()` inside a test path is a smell. The function under
  test has no dependency-injection seam, so any test of its
  fallback behavior depends on dev-machine env state being absent.
- "Environment-dependent" is not a diagnosis. It's a confession
  that the code is structurally untestable.

## What "green" looks like

For every workspace in the train, every `cargo test` run on every
machine (and CI) yields the same per-test verdict, with no socket
opens recorded, no credential env reads, no writes outside `TMPDIR`.

## How we keep it honest

- **Today (manual).** `RP-HERMETIC-UNIT` is `Aspired`. We grep for
  `from_env`, `reqwest::Client::new()`, `tokio::net::*`,
  `std::env::var`, etc., in test code. The current sweep is tracked
  in [QF-2026-06-02-05](../incidents/QF-2026-06-02-05.md).
- **Soon (programmatic).** [`dim-hermeticity`](../dimensions/hermeticity.md)
  in arena-tests will run each workspace's unit tests under a
  sandbox that denies network and unsets credential env vars. Any
  test that fails the sandboxed run gets emitted as a Finding.
- **Tomorrow (preventive).** Custom test attribute macros (e.g.
  `#[hermetic_test]`) that, at compile time, restrict the test
  function to a subset of the standard library known to be safe.

## Migration path for a violating test

See [`../migrations/inject-the-backend.md`](../migrations/inject-the-backend.md).
That guide walks through the exact transformation we did on
`guide_heading` to take it from env-reading to dependency-injected
(commit `3fbe4fe` in axiom).

## Common smells that point at violations

- `reqwest::get(...)`, `reqwest::Client::new()`, anything that yields
  a `Client` from environment-derived state.
- `std::env::var("FOO")` inside a `#[test]` function body or inside
  a function called from one.
- `tokio::net::TcpStream::connect(...)`.
- `std::fs::write(...)` to a path that isn't under
  `tempfile::tempdir()`.
- Test names that hedge: `_works_when_configured`,
  `_returns_real_data_in_prod`, `_skipped_locally`, etc. The hedge
  is the smell.

## Related properties

- [`RP-DETERMINISM`](RP-DETERMINISM.md) — hermetic tests are a
  prerequisite for determinism. You can't be deterministic if you're
  dependent on env or network.
- [`RP-AI-SHORTCUT-DECLARED`](RP-AI-SHORTCUT-DECLARED.md) — the
  axiom failure was made worse by the AI's first diagnosis
  ("env-dependent, not my problem"). The property exists in part to
  prevent that class of dismissal.
