# High-Risk Claim Portfolio

This scenario is a product-side exemplar for the Arbiter assurance portfolio.
It is intentionally not a proof system and not a solver runner.

It shows how a product should track high-risk claims before deciding which ones
deserve Cedar/SymCC, CVC5, or future proof-assistant work.

Run:

```sh
cargo run -p scenario-high-risk-claim-portfolio
```

## Resource Declaration

**Trust label:** `LOCAL REAL / NO LIVE NETWORK`.

- Live external resources: **no**. The scenario does not call network
  providers, CVC5, Cedar symbolic analysis, or any proof assistant.
- Mosaic extensions: atelier uses the real Arbiter crate and claim portfolio
  types. It does not replace Arbiter with a local mock.
- Backend mode: in-process portfolio classification and evidence-ladder
  reporting only.
- Credentials / feature flags: none.
- Trust boundary: trust this as a product-side claim triage exemplar. Do not
  read it as proof that any claim has been formally verified.

The companion Truth artifact is
[`../../truths/High-Risk Claim Portfolio.md`](../../truths/High-Risk%20Claim%20Portfolio.md).
