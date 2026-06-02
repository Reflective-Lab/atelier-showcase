---
source: llm
---
# Storage, Memory, and Analytics Boundary

## Decision

Do not introduce a time-series database for the current live SEC review memory
work. The immediate need is historical comparison over typed review profiles,
not low-latency operational telemetry.

The architecture should be:

```text
converge-storage ObjectStore contract
  -> local InMemory / file store for development
  -> Runtime Runway / GCS-backed object store for cloud durability
  -> JSON or Parquet review-profile objects
  -> Polars / Prism analysis
  -> Converge facts, Arbiter decisions, Ferrox allocations
```

## Layer Boundaries

- `converge-storage` is the persistence contract. It should let the same
  scenario switch between local development storage and a cloud object store
  without changing the domain workflow.
- Runtime Runway should provide the production Google Cloud Storage implementation and
  deployment-time configuration. atelier should not hard-code a cloud backend.
- Polars is the data-frame / columnar compute layer. It is not the durable
  storage boundary.
- Prism owns analytic packs such as similarity, ranking, classification,
  forecasting, and fuzzy inference. It should consume feature rows built from
  stored review profiles rather than inventing an app-local scoring engine.
- Mnemos remains the right owner when the problem becomes semantic recall,
  agentic memory, or learned prior episodes. The SEC slice only needs bounded
  profile history today.

## SEC Scenario Shape

The next memory-backed SEC slice should add a small domain abstraction, for
example `SecReviewMemoryStore`:

- write the current filing review profile with source fact id, CIK, accession,
  form type, request hash, source URL, heading count, section bytes, and derived
  feature vector;
- load comparable prior profiles by CIK, form type, and date window;
- feed those rows to Prism `SimilarityPack` or `RankingPack`;
- keep local development on `object_store::memory::InMemory` through
  `converge-storage`;
- switch to Runtime Runway/GCS by configuration when the showcase runs in cloud mode.

The stored profile is memory for a decision, not a mock. If the current profile
comes from a `REAL LIVE` SEC EDGAR call, the profile must preserve that
provenance so later comparisons can still be audited.

## When a Time-Series DB Is Justified

Use a real time-series database only when the product needs operational
time-series behavior:

- continuous high-volume ingestion;
- low-latency dashboard queries over recent windows;
- alerting and anomaly triggers tied to retention/downsampling policies;
- many concurrent writers;
- server-side aggregation that object-store scans cannot satisfy.

Until those requirements appear, object storage plus columnar files is the
smaller and more portable contract.
