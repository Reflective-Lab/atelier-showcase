# SEC EDGAR Live Filing

This scenario is the smallest live-resource proof slice for atelier-showcase.
It fetches Apple Inc.'s 2025 Form 10-K primary document from official SEC EDGAR
through Converge's engine by seeding a typed `SecEdgarRequest`, running
Embassy's `SecFilingSuggestor<LiveSecEdgarProvider>`, reading Item 1A from the
returned typed `SecFilingPayload`, deriving an Arbiter review document that
preserves SEC provenance, applying atelier-domain's SEC 10-K risk policy pack,
and letting `ComplianceGateSuggestor` block auto-clearance when the risk-factor
heading count exceeds the configured review threshold. With `with-solver`
enabled, the blocked review is converted into a real Ferrox HiGHS MIP allocation
that chooses the minimum analyst-review lanes satisfying coverage, breadth, and
senior-review constraints.

Run:

```sh
cargo run -p scenario-sec-edgar-live-filing
```

Run the verbose educational version:

```sh
cargo run -p scenario-sec-edgar-live-filing -- --verbose
```

Run the solver-backed version:

```sh
cargo run -p scenario-sec-edgar-live-filing --features with-solver
```

## Resource Declaration

**Trust label:** `REAL LIVE`.

- Live external resources: **yes**. The scenario calls official SEC EDGAR over
  the network.
- Mosaic extensions: atelier uses the real `converge-embassy-sec-edgar` crate
  with its `live` feature and the real `converge-arbiter-policy` rule gate.
  With `--features with-solver`, it also uses the real
  `converge-ferrox-solver` HiGHS MIP suggestor. The Converge engine registers
  Embassy's `SecFilingSuggestor` with the live SEC provider, derives an Arbiter
  `ComplianceDocumentPayload`, and registers Arbiter's
  `ComplianceGateSuggestor`; atelier does not replace those extensions with
  local mocks.
- Mocking: **none**. The run does not use Embassy's deterministic SEC test
  provider, recorded HTTP, canned HTML fixtures, fake provider output, or a
  fake policy gate. The solver-enabled run uses HiGHS through Ferrox, not a
  heuristic fallback.
- Backend mode: live SEC fetch initiated from a Converge seed fact:
  `SecEdgarRequest` under `ContextKey::Seeds` -> `SecFilingSuggestor` ->
  `LiveSecEdgarProvider` -> typed `SecFilingPayload` under
  `ContextKey::Hypotheses` -> scenario-local provenance-preserving adapter
  using `atelier_domain::sec_risk::SecRiskPolicyPack` -> Arbiter
  `ComplianceDocumentPayload` under `ContextKey::Strategies` ->
  `ComplianceGateSuggestor` -> typed `ComplianceConstraintPayload` under
  `ContextKey::Constraints`. With `with-solver`, a second Converge run seeds a
  Ferrox `MipRequest` and reads the resulting `MipPlan`; the solver run is
  separate so the MIP request does not re-wake the SEC filing suggestor.
- Credentials / feature flags: no API key. The scenario enables Embassy
  `sec-edgar`'s `live` cargo feature in its package dependency. Ferrox HiGHS is
  enabled only with `--features with-solver`.
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
- [Ferrox optimization](../../../mosaic-extensions/kb/Capability%20Matrix.md#ferrox--optimization)

## Why Generic Substitutes Fail

A generic scrape can fetch bytes, but it does not carry the source boundary,
SEC-specific politeness contract, item-section heuristic, or reusable Embassy
and Converge surfaces. A hand-written `if` statement can count headings, but it
does not emit a typed policy constraint that another Converge step can route or
audit, and it does not solve the follow-on allocation problem. A chat model can
summarize a filing after someone gives it text, but it cannot prove where the
text came from, whether the current run called the live SEC resource, whether
the filing moved through Converge as a typed fact, whether a downstream policy
gate saw the source fact id and request hash, or whether the review allocation
is optimal under stated constraints. This example keeps the proof small: the
source is official, the network call is live, the fact boundary is typed, the
Arbiter decision is typed, the optional solver witness is real, and the no-mock
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
`ComplianceConstraintPayload`. The simple threshold rule has been moved into
`atelier_domain::sec_risk::SecRiskPolicyPack`, which now carries a named SEC
10-K risk policy pack with source-shape, source-vendor, section-size, and
heading-count rules. With `with-solver`, the Arbiter block feeds a real Ferrox
HiGHS MIP that chooses the minimum analyst-review lanes covering all extracted
risk headings with breadth and senior-review constraints. The next larger gap is
memory-backed review: compare this filing's risk-heading profile against prior
filings or past review outcomes without losing provenance.
