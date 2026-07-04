# Running the Helm Multi-User Convergence Scenario

This is the operator's guide for `scenario-helm-multiuser-convergence-headless`. It explains every state change the scenario produces, why each phase matters as a meeting simulation, and what the scenario is proving at the coordination layer.

---

## What this scenario is

A headless proof that Helm's typed coordination protocol supports a structured multi-participant convergence session — the kind of meeting where people and AI agents need to arrive at a **governed decision**, not just exchange messages.

Four participants join a shared workspace. A pool of four server-side suggestors pushes findings throughout the session. Each participant's local `ClientHelm` routes every push based on urgency. A shared `DecisionLedger` records the gate outcome once, rejects a second contradictory vote as a conflict, and treats a matching second vote as idempotent. Sessions are opened at start and closed at end; presence entries track who is looking at what.

No network. No server process. No async runtime. Every state transition is observable in the event log.

---

## Prerequisites

```bash
# From the atelier-showcase workspace root
just check          # Verify everything compiles
just test           # Run all tests including the 11 coordination tests
```

The scenario has no external dependencies (no LLM API key, no running service).

---

## Running the scenario

```bash
# Full 47-minute session arc — markdown report (default)
cargo run -p scenario-helm-multiuser-convergence-headless -- --case full-session

# Same, as JSONL (one line per event — pipe to jq or import into analysis tools)
cargo run -p scenario-helm-multiuser-convergence-headless -- --case full-session --format jsonl

# Compressed burst — Convergence + Gate phases only, no orientation warmup
cargo run -p scenario-helm-multiuser-convergence-headless -- --case solver-burst

# Gate-conflict proof — see Leader approve, Analyst agree (idempotent), Skeptic reject (conflict)
cargo run -p scenario-helm-multiuser-convergence-headless -- --case gate-conflict
```

---

## Participants and suggestors

### Participants (4 ClientHelm instances)

| Actor ID | Role | Kind | Behaviour in the session |
|---|---|---|---|
| `participant:leader` | Leader | Human | Opens gate decision; first to claim exploration subject |
| `participant:analyst` | Analyst | Human | Processes solver pushes; second soft-claim on exploration |
| `participant:skeptic` | Skeptic | Human | Claims dissent subject; votes against the gate (conflict) |
| `participant:observer` | Observer | Agent | Receives all pushes; never claims subjects |

Each participant has its own `ClientHelm` with a 10-minute per-formation wall-clock budget. The budget is advanced by a simulated clock (not real wall time) so the scenario runs in milliseconds.

### Suggestors (server-side, simulated)

| Suggestor | Urgency | What it represents |
|---|---|---|
| `LlmSynthesis` | Advisory | Language-model synthesis of the converging fact graph |
| `FeroxOptimizer` | Disruptive | Combinatorial solver found a better assignment |
| `ArbiterPolicy` | Preemptive | Policy constraint violation — must re-evaluate immediately |
| `PrismAnalytics` | Informational | Ambient metrics; does not require a response |

---

## The urgency routing table (ClientHelm's core rule)

This is the most important rule to understand. `ClientHelm` is stateful — what it does with a push depends on whether the participant already has a formation running.

```
No running formation  +  ANY urgency    → SpawnNew (start a local formation)
Running formation     +  Informational  → QueueAndNotify (surface; do not interrupt)
Running formation     +  Advisory       → QueueAndNotify (surface; do not interrupt)
Running formation     +  Disruptive     → OffloadToServer (parallel server job; local continues)
Running formation     +  Preemptive     → PauseAndInject (suspend local; fresh formation with injected context)
```

**Key implication:** an idle client spawns a formation for every push, including Informational ones. "Informational → do nothing" only applies once the client is actively working. In the scenario, each formation completes instantly (headless — no real Converge engine), so the next push always finds an idle client. This is a known simulation simplification (see [Limitations](#simulation-limitations) below).

---

## Phase-by-phase walkthrough

### Phase 1 — Orientation (t = 0–7 min, 45 events)

**What happens:**
1. All 4 sessions are opened in `SessionRegistry`.
2. All 4 participants focus their presence on the session subject (`run:<session-id>`).
3. LlmSynthesis dispatches 3 Advisory pushes (at t=2, t=4, t=6 min). Each push arrives to an idle client → `SpawnNew` → formation completes immediately → `FormationCompleted`.
4. PrismAnalytics dispatches 1 Informational push (t=7 min). Same result: idle client → `SpawnNew`.

**Why this matters as a meeting:**
The orientation phase is warm-up — everyone joins, everyone can see the same starting subject. The LlmSynthesis pushes represent the AI synthesising initial framing from the shared workspace. Every participant processes the framing independently (each has their own `ClientHelm`), which is the point: no participant's reasoning is another participant's reasoning. Advisory means "when you have a free moment" — which in a real client with a running formation would result in a queue notification, not an interrupt.

**State after orientation:**
- `SessionRegistry`: 4 active sessions
- `PresenceRegistry`: 4 entries focused on `run:<session-id>`
- Formations spawned so far: 16 (4 participants × 4 pushes)

---

### Phase 2 — Exploration (t = 7–17 min, 21 events)

**What happens:**
1. Leader and Analyst each soft-claim `exploration:phase:exploration:cycle:1` via `PresenceRegistry`.
2. FeroxOptimizer dispatches 2 Disruptive pushes (t=12, t=17). Idle clients → `SpawnNew` → formation completes.

**Why this matters as a meeting:**
The exploration phase is where the combinatorial solver has found something worth acting on. Disruptive means "spawn a parallel job on the server if you're busy" — in production, a participant mid-formation would offload this to the server rather than interrupting their local reasoning. The soft-claims on the exploration subject are advisory hints: "I am actively working on this." Two participants can claim the same subject simultaneously — the `PresenceRegistry` never blocks. This is the optimistic model: coordination happens through shared visibility, not locks.

**State after exploration:**
- `PresenceRegistry`: 4 + 2 = 6 entries (original 4 focused + Leader + Analyst claimed on exploration)
- Formations spawned so far: 24

---

### Phase 3 — Convergence (t = 17–32 min, 20 events)

**What happens:**
1. Skeptic soft-claims `dissent:hypothesis:convergence:h-1` — an advisory signal that someone is actively raising a dissent.
2. ArbiterPolicy dispatches 1 Preemptive push (t=27). This is the highest urgency. In production: if a formation is running, it is suspended; a fresh formation is spawned seeded with the suspended context + the arbiter's constraint violation. Idle client: `SpawnNew` directly.
3. LlmSynthesis dispatches 1 more Advisory push (t=32) for convergence narrowing.

**Why this matters as a meeting:**
The policy evaluator has detected a constraint violation. In a real meeting, this is the moment someone says "we can't proceed, there's a blocker." The Preemptive urgency forces attention — it does not just queue. The Skeptic's soft-claim on a dissent subject is visible to all participants via `PresenceRegistry`, signalling that a challenge is live before anyone speaks.

**State after convergence:**
- `PresenceRegistry`: 7 entries (6 from before + Skeptic's dissent claim)
- Formations spawned so far: 32

---

### Phase 4 — Gate (t = 32–34 min, 5 events)

**What happens:**
1. A `GatedDecision` is delivered to all 4 `ClientHelm` instances via `handle_gate()`.
2. The shared `DecisionLedger` records three attempts:
   - **Leader → Approve**: `DecisionOutcome::Recorded` — first decision wins; becomes the authoritative record.
   - **Analyst → Approve**: `DecisionOutcome::Idempotent` — same decision, returns the original record, no second effect.
   - **Skeptic → Reject**: `DecisionOutcome::Conflict` — divergent from the existing Approve; rejected; original record preserved.
3. Leader responds to the gate via `ClientHelm.respond_to_gate()` (client-side surface).

**Why this matters as a meeting:**
This is the HITL gate — the moment where a human decision is required before the governed job can continue. The `DecisionLedger` gives the platform three guarantees:

1. **Authority**: the first decision drives the outcome. Being first matters.
2. **Idempotency**: if two people agree, recording both has no double-effect. Consensus is safe.
3. **Conflict detection**: if two people disagree, the platform surfaces it explicitly. There is no silent override. The Skeptic's rejection is not lost — it is recorded as a conflict event with attribution.

A chat room cannot make these three guarantees simultaneously. A chat room records whatever the last message was; the platform records what the first authorised decision was.

**State after gate:**
- `DecisionLedger`: one entry for `gate:convergence:session:main` → Approve by Leader
- The conflict and idempotent outcomes are in the event log but do not change the ledger entry

---

### Phase 5 — Integration (t = 34–42 min, 59 events)

**What happens:**
1. LlmSynthesis dispatches 2 Advisory pushes (t=38, t=42). Each spawns formations and completes.
2. All pending temperature readings are drained from every participant's `ClientHelm` via `drain_submissions()`.

Each formation that completed anywhere in the session produced one `TemperatureReading` (position=agree, conviction=high, subject_ref=session://convergence/<phase>). These are queued inside `ClientHelm` until drained. The drain releases 10 readings per participant (10 formations completed before the drain point), totalling 40 `TemperatureRecorded` events.

**Why this matters as a meeting:**
Integration is the phase after the gate clears, where the agreed result propagates. Temperature readings are how Client Helm expresses what each local formation concluded — a position (agree/disagree/uncertain) and conviction level. In production these would be submitted to the server as `ProposedFacts` to be admitted through Converge. Here they are drained to show that every formation's output is tracked and recoverable.

**Why 10 temperature readings per participant:**
10 formations completed before the drain (4 in orientation + 2 in exploration + 2 in convergence + 2 in integration). The simulated output always returns `agree/high` — a deliberate simplification noted under [Limitations](#simulation-limitations).

---

### Phase 6 — Closeout (t = 42–47 min, 7 events)

**What happens:**
1. Leader and Analyst release their soft-claims on `exploration:phase:exploration:cycle:1`.
2. All 4 sessions are closed via `SessionRegistry.close()`.

**Why this matters as a meeting:**
Clean close is part of the protocol. Session close triggers cleanup; presence release signals "I am no longer working on this." The `SessionRegistry` should be empty after closeout — `active_session_count() == 0` is asserted in the tests. The 7 presence entries remaining (not all are released in closeout) reflect the Skeptic's dissent claim and the 4 original focus entries — those would be swept by a TTL in production.

**State after closeout:**
- `SessionRegistry`: 0 active sessions
- `PresenceRegistry`: 7 entries (focus entries + unreleased claims — swept by TTL in prod)
- `DecisionLedger`: 1 entry (gate approved by Leader)
- Total formations spawned: 40

---

## Event types reference

| Event | When it fires | What it records |
|---|---|---|
| `SessionOpened` | Session created in `SessionRegistry` | session id, role, workspace |
| `PresenceFocused` | `PresenceRegistry.focus()` called | subject ref, claimed=false |
| `PresenceClaimed` | `PresenceRegistry.claim()` called | subject ref, claimed=true |
| `PushDispatched` | Server pool sends a `SessionPush` to all clients | suggestor id, urgency, finding id |
| `FormationSpawned` | `ClientHelm.handle_push()` returns `SpawnFormation` or `PauseAndInject` | loop id, description, action type |
| `FormationCompleted` | `ClientHelm.formation_completed()` called | loop id, proposal count |
| `GateOpen` | `GatedDecision` delivered to all clients | gate id, condition |
| `GateDecision` | `DecisionLedger.record()` → `Recorded` | gate id, decision, decision id |
| `GateIdempotent` | `DecisionLedger.record()` → `Idempotent` | gate id, original decider |
| `GateConflict` | `DecisionLedger.record()` → `Conflict` | gate id, existing decision, attempted decision, attempted by |
| `TemperatureRecorded` | `drain_submissions()` returns a `Temperature` submission | position, conviction, subject ref |
| `PresenceReleased` | `PresenceRegistry.release()` called | subject ref |
| `SessionClosed` | `SessionRegistry.close()` called | session id, opened at |
| `PhaseEntered` | New phase begins | phase name |

---

## Summary of coordination invariants proved

| Invariant | Test | How proved |
|---|---|---|
| All 4 sessions open at start | `four_sessions_opened_at_start` | Count `SessionOpened` events = 4 |
| All 4 sessions close at end; registry empty | `four_sessions_closed_at_closeout` | Count `SessionClosed` = 4; `active_session_count() == 0` |
| Two participants can soft-claim the same subject simultaneously | `optimistic_presence_allows_two_claims` | Leader + Analyst both claim `exploration:*` without blocking |
| Gate first decision is Recorded | `decision_ledger_conflict_is_recorded` | Leader's Approve → `Recorded`; ledger entry preserved after Skeptic conflict |
| Gate divergent second decision is Conflict | `decision_ledger_conflict_is_recorded` | Skeptic's Reject → `Conflict`; ledger unchanged |
| Gate matching second decision is Idempotent | `decision_ledger_idempotent_is_recorded` | Analyst's Approve → `Idempotent`; original record returned |
| Solver-burst skips orientation | `solver_burst_skips_orientation_and_runs_convergence_and_gate` | No `PhaseEntered/orientation` event |
| Session arc is deterministic | `full_session_is_deterministic` | Two runs produce identical event counts and simulated_ms |
| All 6 phases entered in full session | `full_session_covers_all_six_phases` | `PhaseEntered` events for all 6 phases |
| JSONL is parseable, one line per event | `jsonl_timeline_is_parseable_and_complete` | `serde_json::from_str` on every line |
| Report includes the protocol argument | `full_session_report_contains_why_generic_substitutes_fail` | String contains "Why Generic Substitutes Fail" and "DecisionLedger" |

---

## Simulation limitations

The headless scenario trades realism for observability:

1. **Formations complete instantly.** There is no real Converge engine. `formation_completed()` is called immediately after `handle_push()`. In production, a formation runs a bounded Converge loop (up to `max_cycles` and `max_facts`) — this takes real time. The headless simulation shows routing and state management without the engine overhead.

2. **Every push arrives to an idle client.** Because formations complete instantly, no formation is ever running when the next push arrives. The routing rules for "running + Disruptive → OffloadToServer" and "running + Advisory → QueueAndNotify" are structurally correct in `ClientHelm` but are not exercised in this scenario. The `helm-realtime-stem-headless` scenario (`--case server-offload-under-load`) proves those paths.

3. **Temperature readings are synthetic.** Every formation returns `position=agree, conviction=high`. A real formation would derive position from the Converge fixed point — which participant's proposals survived convergence. Varying temperature proofs are a follow-up scenario.

4. **No server-side formation tracking.** `RequestServerFormation` actions from the FeroxOptimizer (Disruptive) push to an idle client do not actually offload to a server. The headless scenario emits a `FormationSpawned` event and stops. `server_formation_started()` is not called because there is no server.

5. **Session TTL not exercised.** `SessionRegistry` has a 5-minute default lease. The simulated clock advances past that boundary during the session, but `sweep()` is not called between phases. In production, stale sessions would be removed automatically.

---

## Next step: Karl as a live participant

The scenario currently runs fully scripted. The logical next step is a **live CLI client** where you join a real session as a human operator: receive pushes over SSE, watch other participants' presence in real time, and cast a gate vote.

This requires:
1. **A running `helm-coordination` HTTP server** — the `CoordinationModule` from `helm-coordination` mounted on a `runtime-runway` `HostApp`. Quorum-sense already does this; the same surface can be pointed at by any CLI.
2. **A `helm-cli-client` binary** that:
   - Opens a session (`POST /v1/coordination/sessions`)
   - Subscribes to the SSE stream (`GET /v1/coordination/stream`)
   - Receives `SessionPush` events and drives a local `ClientHelm`
   - Displays presence of other participants (`GET /v1/coordination/presence`)
   - Accepts a gate vote from stdin and posts it (`POST /v1/coordination/decisions`)
3. **A scripted companion** — the other 3 participants (Analyst, Skeptic, Observer) can remain scripted but run against the live server, so you experience real concurrent events rather than a replay.

This would make the session genuine: you would see Skeptic's dissent claim appear in your presence view, receive the gate event, and decide whether to approve or reject — with the `DecisionLedger` conflict detection firing if Skeptic votes first and you vote differently.

That is tracked as the next work item on RFL-126.
