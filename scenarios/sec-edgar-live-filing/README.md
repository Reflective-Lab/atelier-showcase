# SEC EDGAR Live Filing

This scenario is the smallest live-resource proof slice for atelier-showcase.
It fetches Apple Inc.'s 2025 Form 10-K primary document from official SEC EDGAR
through Converge's engine by seeding a typed `SecEdgarRequest`, running
Embassy's `SecFilingSuggestor<LiveSecEdgarProvider>`, reading Item 1A from the
returned typed `SecFilingPayload`, deriving an Arbiter review document that
preserves SEC provenance, and letting `ComplianceGateSuggestor` block
auto-clearance when the risk-factor heading count exceeds the configured review
threshold.

Run:

```sh
cargo run -p scenario-sec-edgar-live-filing
```

Run the verbose educational version:

```sh
cargo run -p scenario-sec-edgar-live-filing -- --verbose
```

## Resource Declaration

**Trust label:** `REAL LIVE`.

- Live external resources: **yes**. The scenario calls official SEC EDGAR over
  the network.
- Mosaic extensions: atelier uses the real `converge-embassy-sec-edgar` crate
  with its `live` feature and the real `converge-arbiter-policy` rule gate. The
  Converge engine registers Embassy's `SecFilingSuggestor` with the live SEC
  provider, derives an Arbiter `ComplianceDocumentPayload`, and registers
  Arbiter's `ComplianceGateSuggestor`; atelier does not replace either
  extension with a local mock.
- Mocking: **none**. The run does not use Embassy's deterministic SEC test
  provider, recorded HTTP, canned HTML fixtures, fake provider output, or a
  fake policy gate.
- Backend mode: live SEC fetch initiated from a Converge seed fact:
  `SecEdgarRequest` under `ContextKey::Seeds` -> `SecFilingSuggestor` ->
  `LiveSecEdgarProvider` -> typed `SecFilingPayload` under
  `ContextKey::Hypotheses` -> scenario-local provenance-preserving adapter ->
  Arbiter `ComplianceDocumentPayload` under `ContextKey::Strategies` ->
  `ComplianceGateSuggestor` -> typed `ComplianceConstraintPayload` under
  `ContextKey::Constraints`. The example then runs Embassy's heading extractor
  over `SecFilingPayload.filing.sections["1A"]` and verifies the Arbiter
  decision references the derived document.
- Credentials / feature flags: no API key. The scenario enables Embassy
  `sec-edgar`'s `live` cargo feature in its package dependency.
- Trust boundary: trust this as proof that atelier can call a real external
  Mosaic source-observation path. Do not read it as a complete underwriting,
  risk, compliance, or investment decision workflow.

## Live Resource

- Company: Apple Inc.
- CIK: `0000320193`
- Form: `10-K`
- Accession: `0000320193-25-000079`
- Filing date: `2025-10-31`
- Primary document: `aapl-20250927.htm`
- SEC filing detail page:
  <https://www.sec.gov/Archives/edgar/data/320193/0000320193-25-000079-index.htm>
- SEC primary document:
  <https://www.sec.gov/Archives/edgar/data/320193/000032019325000079/aapl-20250927.htm>

A human can verify the filing in under a minute by opening the SEC filing detail
page and checking the accession number, form type, filing date, CIK, and primary
document name.

## Capability Matrix Links

- [Embassy named-source observation](../../../mosaic-extensions/kb/Capability%20Matrix.md#embassy--named-source-observation)
- [`embassy-sec-edgar`](../../../mosaic-extensions/kb/Capability%20Matrix.md#embassy--named-source-observation)
- [Arbiter policy as code](../../../mosaic-extensions/kb/Capability%20Matrix.md#arbiter--policy-as-code)

## Why Generic Substitutes Fail

A generic scrape can fetch bytes, but it does not carry the source boundary,
SEC-specific politeness contract, item-section heuristic, or reusable Embassy
and Converge surfaces. A hand-written `if` statement can count headings, but it
does not emit a typed policy constraint that another Converge step can route or
audit. A chat model can summarize a filing after someone gives it text, but it
cannot prove where the text came from, whether the current run called the live
SEC resource, whether the filing moved through Converge as a typed fact, or
whether a downstream policy gate saw the source fact id and request hash. This
example keeps the proof small: the source is official, the network call is live,
the fact boundary is typed, the Arbiter decision is typed, and the no-mock
boundary is visible before results print.

## Pressure Finding

This scenario started by using Embassy's lower-level live helper functions. The
pressure finding was resolved upstream: Embassy now exposes
`LiveSecEdgarProvider`, which implements the same `SecEdgarProvider` trait used
by deterministic tests and returns typed `Observation<Filing>` records. The next
downstream pressure point is now partially resolved: this scenario no longer
calls the provider directly from `main`; it seeds `SecEdgarRequest` into the
Converge engine and reads the resulting `SecFilingPayload` fact from
`ContextKey::Hypotheses`. The next pressure point is also partially resolved:
the live filing fact is transformed into an Arbiter `ComplianceDocumentPayload`
that preserves `source_fact_id`, `source_request_hash`, `source_vendor`, CIK,
accession, and source URL before Arbiter emits a typed
`ComplianceConstraintPayload`. The next larger gap is richer decision
composition: feed the live filing fact into memory or solver-backed decisions,
or replace the simple threshold rule with a real domain policy pack.
