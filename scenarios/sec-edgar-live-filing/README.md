# SEC EDGAR Live Filing

This scenario is the smallest live-resource proof slice for atelier-showcase.
It fetches Apple Inc.'s 2025 Form 10-K primary document from official SEC EDGAR
through the Embassy `sec-edgar` live feature, locates Item 1A, and extracts
risk-factor headings.

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
  with its `live` feature. It does not replace Embassy with a local mock.
- Mocking: **none**. The run does not use Embassy's deterministic SEC stub
  provider, recorded HTTP, canned HTML fixtures, or fake provider output.
- Backend mode: live SEC HTTP fetch plus Embassy's Item-section locator and
  heading extractor.
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

## Why Generic Substitutes Fail

A generic scrape can fetch bytes, but it does not carry the source boundary,
SEC-specific politeness contract, item-section heuristic, or reusable Embassy
surface. A chat model can summarize a filing after someone gives it text, but it
cannot prove where the text came from or whether the current run called the live
SEC resource. This example keeps the proof small: the source is official, the
network call is live, and the no-mock boundary is visible before results print.

## Pressure Finding

This scenario proves a narrow live path exists today, but it also names the next
gap: Embassy `sec-edgar` exposes live fetch and extraction helpers, while the
default provider trait still ships a stub implementation for provider-shaped
formation tests. The next upstream improvement is a live `SecEdgarProvider`
implementation that returns typed `Observation<Filing>` records through the same
provider trait used by stub-backed tests.
