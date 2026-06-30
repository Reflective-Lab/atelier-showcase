# Helm Coordination Headless

This scenario narrates the first Helm Coordination Layer shape without a GUI:
a long-running coordination session accepts signed human and agent suggestions,
mixes and matches them, invokes a bounded checkpoint formation, records an
operator gate decision, and prepares proposals for Converge admission. Approved
gates record final fact ids admitted by Converge; rejected and timed-out gates
stop before admission.

It intentionally keeps the model example-local. Arena imports this scenario and
turns it into cross-module assertions against Helm readiness contracts.

Run the narrated Markdown view:

```bash
cargo run -p scenario-helm-coordination-headless
```

Run the append-only JSONL timeline:

```bash
cargo run -p scenario-helm-coordination-headless -- --format jsonl
```

Run a blocked gate variant:

```bash
cargo run -p scenario-helm-coordination-headless -- --gate timed-out
```

Run the dynamic crowd consensus use case with 5, 30, or 100 simulated users:

```bash
cargo run -p scenario-helm-coordination-headless -- --use-case crowd --users 5
cargo run -p scenario-helm-coordination-headless -- --use-case crowd --users 30
cargo run -p scenario-helm-coordination-headless -- --use-case crowd --users 100
```

Add `--seed` when you want a replayable dynamic run. Omit it when you want an
unpredictable user script.

```bash
cargo run -p scenario-helm-coordination-headless -- --use-case crowd --users 100 --seed 42 --format jsonl
```
