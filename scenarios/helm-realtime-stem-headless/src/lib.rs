// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Headless multi-user realtime stress harness for the Session Intelligence Spine.
//!
//! Uses real [`helm_client::ClientHelm`] on each simulated participant and a
//! deterministic server loop pool for long-running and short-running formations.
//! Arena can import this crate to prove routing, registry, and gate semantics
//! without depending on `marquee-apps/quorum-sense`.

pub mod cases;

use std::collections::HashMap;

use helm_client::client::{ClientHelm, ClientHelmAction};
use helm_client::formation::{FormationOutput, TemperatureReading};
use helm_client::ids::LoopId;
use helm_client::registry::{LoopEntry, LoopKind, LoopState};
use helm_session_contracts::finding::FindingId;
use helm_session_contracts::gate::{GateCondition, GateId, GatedDecision};
use helm_session_contracts::push::{SessionContext, SessionPush};
use helm_session_contracts::urgency::UrgencyIntent;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use cases::run_case;

/// Named interactive cases runnable from the CLI or Arena imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InteractiveCaseId {
    SingleUserShortLoop,
    ServerOffloadUnderLoad,
    PreemptivePivot,
    ThreeUserConcurrentRoom,
    LongAndShortServerMix,
    GateWhileLoopsActive,
    BudgetExhaustion,
    MarqueeBurstRoom,
}

impl InteractiveCaseId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SingleUserShortLoop => "single-user-short-loop",
            Self::ServerOffloadUnderLoad => "server-offload-under-load",
            Self::PreemptivePivot => "preemptive-pivot",
            Self::ThreeUserConcurrentRoom => "three-user-concurrent-room",
            Self::LongAndShortServerMix => "long-and-short-server-mix",
            Self::GateWhileLoopsActive => "gate-while-loops-active",
            Self::BudgetExhaustion => "budget-exhaustion",
            Self::MarqueeBurstRoom => "marquee-burst-room",
        }
    }

    pub fn all() -> [Self; 8] {
        [
            Self::SingleUserShortLoop,
            Self::ServerOffloadUnderLoad,
            Self::PreemptivePivot,
            Self::ThreeUserConcurrentRoom,
            Self::LongAndShortServerMix,
            Self::GateWhileLoopsActive,
            Self::BudgetExhaustion,
            Self::MarqueeBurstRoom,
        ]
    }
}

/// Server formation duration profile — short probes vs long DD / synthesis jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerLoopProfile {
    ShortProbe,
    LongDd,
    LongSynthesis,
}

impl ServerLoopProfile {
    pub const fn formation_type(self) -> &'static str {
        match self {
            Self::ShortProbe => "coverage-probe",
            Self::LongDd => "dd-analysis",
            Self::LongSynthesis => "bounded-synthesis",
        }
    }

    pub const fn duration_ms(self) -> u64 {
        match self {
            Self::ShortProbe => 250,
            Self::LongDd => 6_000,
            Self::LongSynthesis => 5_500,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StemEventKind {
    SessionOpened,
    ParticipantJoined,
    PushDelivered,
    ClientAction,
    LocalLoopSpawned,
    LocalLoopCompleted,
    LocalLoopFailed,
    ServerLoopSpawned,
    ServerLoopCompleted,
    ServerHandleRegistered,
    GateOpened,
    GateResponded,
    SubmissionDrained,
    CheckpointPrepared,
    ConvergeAdmissionRecorded,
    ClockTick,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StemEvent {
    pub sequence: u64,
    pub at_ms: u64,
    pub kind: StemEventKind,
    pub payload: Value,
}

struct ParticipantSlot {
    display_name: String,
    role: String,
    helm: ClientHelm,
    pending_local_spawn: bool,
    pending_server_offload: Option<String>,
}

#[derive(Debug, Clone)]
struct ServerLoopRecord {
    server_formation_id: String,
    participant_id: String,
    formation_type: String,
    profile: ServerLoopProfile,
    started_at_ms: u64,
    completes_at_ms: u64,
    client_loop_id: Option<LoopId>,
    completed: bool,
}

/// Deterministic headless coordinator for multi-user realtime stem scenarios.
pub struct RealtimeStemRun {
    pub case_id: String,
    pub session_id: Option<String>,
    pub session_mode: Option<String>,
    pub participants: HashMap<String, ParticipantSlot>,
    pub server_loops: Vec<ServerLoopRecord>,
    pub events: Vec<StemEvent>,
    pub prepared_proposals: Vec<String>,
    pub admitted_fact_ids: Vec<String>,
    now_ms: u64,
    start_ms: u64,
    default_budget_ms: u64,
}

impl RealtimeStemRun {
    pub fn new(case_id: impl Into<String>, start_ms: u64) -> Self {
        Self::new_with_budget(case_id, start_ms, 5 * 60 * 1_000)
    }

    pub fn new_with_budget(
        case_id: impl Into<String>,
        start_ms: u64,
        default_budget_ms: u64,
    ) -> Self {
        Self {
            case_id: case_id.into(),
            session_id: None,
            session_mode: None,
            participants: HashMap::new(),
            server_loops: Vec::new(),
            events: Vec::new(),
            prepared_proposals: Vec::new(),
            admitted_fact_ids: Vec::new(),
            now_ms: start_ms,
            start_ms,
            default_budget_ms,
        }
    }

    pub fn open_session(&mut self, session_id: impl Into<String>, mode: impl Into<String>) {
        self.session_id = Some(session_id.into());
        self.session_mode = Some(mode.into());
        self.emit(
            StemEventKind::SessionOpened,
            json!({
                "session_id": self.session_id,
                "mode": self.session_mode,
            }),
        );
    }

    pub fn join_participant(
        &mut self,
        participant_id: impl Into<String>,
        display_name: impl Into<String>,
        role: impl Into<String>,
    ) {
        let participant_id = participant_id.into();
        self.participants.insert(
            participant_id.clone(),
            ParticipantSlot {
                display_name: display_name.into(),
                role: role.into(),
                helm: ClientHelm::with_budget_ms(self.default_budget_ms),
                pending_local_spawn: false,
                pending_server_offload: None,
            },
        );
        self.emit(
            StemEventKind::ParticipantJoined,
            json!({
                "participant_id": participant_id,
            }),
        );
    }

    pub fn deliver_push(&mut self, participant_id: &str, push: SessionPush) {
        self.now_ms = push.session_context.timestamp_ms;
        let push_event = json!({
            "participant_id": participant_id,
            "finding_id": push.finding_id.as_str(),
            "urgency": push.urgency_intent,
            "objective": push_objective(&push),
        });
        let action = {
            let slot = self
                .participants
                .get_mut(participant_id)
                .unwrap_or_else(|| panic!("unknown participant: {participant_id}"));
            slot.helm.handle_push(push)
        };
        self.emit(StemEventKind::PushDelivered, push_event);
        self.record_client_action(participant_id, &action);
        self.apply_pending_action(participant_id, action);
    }

    pub fn ack_local_spawn(&mut self, participant_id: &str) {
        let slot = self
            .participants
            .get_mut(participant_id)
            .unwrap_or_else(|| panic!("unknown participant: {participant_id}"));
        if let Some(loop_id) = slot.helm.registry_state().iter().find_map(|entry| {
            if matches!(entry.state, LoopState::Running) {
                Some(entry.loop_id.clone())
            } else {
                None
            }
        }) {
            slot.helm.formation_started(&loop_id);
            slot.pending_local_spawn = false;
            self.emit(
                StemEventKind::LocalLoopSpawned,
                json!({
                    "participant_id": participant_id,
                    "loop_id": loop_id.as_str(),
                    "kind": "local",
                }),
            );
        }
    }

    pub fn ack_pause_and_inject(&mut self, participant_id: &str) {
        self.emit(
            StemEventKind::ClientAction,
            json!({
                "participant_id": participant_id,
                "action": "pause-and-inject-accepted",
            }),
        );
        self.spawn_fresh_local_loop(
            participant_id,
            "Fresh formation seeded with injected room evidence",
        );
    }

    /// Simulate the native layer spawning a fresh local formation after pause-and-inject.
    pub fn spawn_fresh_local_loop(&mut self, participant_id: &str, description: impl Into<String>) {
        let description = description.into();
        let loop_id = {
            let slot = self
                .participants
                .get_mut(participant_id)
                .unwrap_or_else(|| panic!("unknown participant: {participant_id}"));
            let push = push_script(
                self.session_id.as_deref().unwrap_or("session"),
                99,
                self.now_ms + 1,
                UrgencyIntent::Informational,
                &description,
                json!({"objective": description}),
            );
            let action = slot.helm.handle_push(push);
            match action {
                ClientHelmAction::SpawnFormation { loop_id, .. } => {
                    slot.helm.formation_started(&loop_id);
                    loop_id
                }
                other => panic!("expected fresh spawn after pause, got {other:?}"),
            }
        };
        self.emit(
            StemEventKind::LocalLoopSpawned,
            json!({
                "participant_id": participant_id,
                "loop_id": loop_id.as_str(),
                "kind": "local",
                "fresh_after_pause": true,
            }),
        );
    }

    pub fn ack_server_offload(
        &mut self,
        participant_id: &str,
        server_formation_id: impl Into<String>,
        formation_type: impl Into<String>,
        profile: ServerLoopProfile,
    ) {
        let server_formation_id = server_formation_id.into();
        let formation_type = formation_type.into();
        let slot = self
            .participants
            .get_mut(participant_id)
            .unwrap_or_else(|| panic!("unknown participant: {participant_id}"));
        let description = slot
            .pending_server_offload
            .take()
            .unwrap_or_else(|| format!("server offload: {formation_type}"));
        let loop_id = slot.helm.server_formation_started(
            server_formation_id.clone(),
            formation_type.clone(),
            helm_client::formation::SeedContext {
                facts: vec![],
                description,
            },
        );
        self.server_loops.push(ServerLoopRecord {
            server_formation_id: server_formation_id.clone(),
            participant_id: participant_id.to_string(),
            formation_type,
            profile,
            started_at_ms: self.now_ms,
            completes_at_ms: self.now_ms + profile.duration_ms(),
            client_loop_id: Some(loop_id),
            completed: false,
        });
        self.emit(
            StemEventKind::ServerHandleRegistered,
            json!({
                "participant_id": participant_id,
                "server_formation_id": server_formation_id,
                "profile": profile,
            }),
        );
        self.emit(
            StemEventKind::ServerLoopSpawned,
            json!({
                "participant_id": participant_id,
                "server_formation_id": server_formation_id,
                "duration_ms": profile.duration_ms(),
            }),
        );
    }

    pub fn spawn_server_loop_direct(
        &mut self,
        participant_id: &str,
        server_formation_id: impl Into<String>,
        formation_type: impl Into<String>,
        profile: ServerLoopProfile,
        at_ms: u64,
    ) {
        self.now_ms = at_ms;
        let server_formation_id = server_formation_id.into();
        let formation_type = formation_type.into();
        let slot = self
            .participants
            .get_mut(participant_id)
            .unwrap_or_else(|| panic!("unknown participant: {participant_id}"));
        let loop_id = slot.helm.server_formation_started(
            server_formation_id.clone(),
            formation_type.clone(),
            helm_client::formation::SeedContext {
                facts: vec![],
                description: format!("direct server loop: {formation_type}"),
            },
        );
        self.server_loops.push(ServerLoopRecord {
            server_formation_id: server_formation_id.clone(),
            participant_id: participant_id.to_string(),
            formation_type,
            profile,
            started_at_ms: at_ms,
            completes_at_ms: at_ms + profile.duration_ms(),
            client_loop_id: Some(loop_id),
            completed: false,
        });
        self.emit(
            StemEventKind::ServerLoopSpawned,
            json!({
                "participant_id": participant_id,
                "server_formation_id": server_formation_id,
                "duration_ms": profile.duration_ms(),
                "direct": true,
            }),
        );
    }

    pub fn complete_local_loop(
        &mut self,
        participant_id: &str,
        proposal: Value,
        temperature: Option<(&str, &str, &str)>,
    ) {
        let slot = self
            .participants
            .get_mut(participant_id)
            .unwrap_or_else(|| panic!("unknown participant: {participant_id}"));
        let loop_id = slot
            .helm
            .registry_state()
            .iter()
            .find_map(|entry| {
                if matches!(entry.kind, LoopKind::Local)
                    && matches!(entry.state, LoopState::Running)
                {
                    Some(entry.loop_id.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("no running local loop for {participant_id}"));
        let output = FormationOutput {
            proposals: vec![proposal.clone()],
            temperature: temperature.map(|(position, conviction, subject_ref)| {
                TemperatureReading {
                    position: position.to_string(),
                    conviction: conviction.to_string(),
                    subject_ref: subject_ref.to_string(),
                }
            }),
        };
        slot.helm.formation_completed(&loop_id, output, None);
        let submissions = slot.helm.drain_submissions();
        self.emit(
            StemEventKind::LocalLoopCompleted,
            json!({
                "participant_id": participant_id,
                "loop_id": loop_id.as_str(),
                "proposal": proposal,
            }),
        );
        if !submissions.is_empty() {
            self.emit(
                StemEventKind::SubmissionDrained,
                json!({
                    "participant_id": participant_id,
                    "count": submissions.len(),
                }),
            );
        }
    }

    pub fn open_gate(
        &mut self,
        participant_id: &str,
        gate_id: impl Into<String>,
        reason: impl Into<String>,
        consequence: impl Into<String>,
    ) {
        let gate = GatedDecision {
            gate_id: GateId::from_string(gate_id.into()),
            condition: GateCondition::AnyParticipant,
            payload: json!({
                "reason": reason.into(),
                "consequence": consequence.into(),
            }),
            deadline: None,
        };
        let gate_id = gate.gate_id.as_str().to_string();
        let pending_gates = {
            let slot = self
                .participants
                .get_mut(participant_id)
                .unwrap_or_else(|| panic!("unknown participant: {participant_id}"));
            slot.helm.handle_gate(gate);
            slot.helm.pending_gates().len()
        };
        self.emit(
            StemEventKind::GateOpened,
            json!({
                "participant_id": participant_id,
                "gate_id": gate_id,
                "pending_gates": pending_gates,
            }),
        );
    }

    pub fn respond_gate(&mut self, participant_id: &str, response: Value) {
        let slot = self
            .participants
            .get_mut(participant_id)
            .unwrap_or_else(|| panic!("unknown participant: {participant_id}"));
        let gate_id = slot
            .helm
            .pending_gates()
            .first()
            .map(|view| view.gate_id.clone())
            .unwrap_or_else(|| panic!("no pending gate for {participant_id}"));
        slot.helm
            .respond_to_gate(&GateId::from_string(gate_id.clone()), response);
        let submissions = slot.helm.drain_submissions();
        self.emit(
            StemEventKind::GateResponded,
            json!({
                "participant_id": participant_id,
                "gate_id": gate_id,
            }),
        );
        if !submissions.is_empty() {
            self.emit(
                StemEventKind::SubmissionDrained,
                json!({
                    "participant_id": participant_id,
                    "count": submissions.len(),
                    "from_gate": true,
                }),
            );
        }
    }

    pub fn emit_checkpoint_admission(
        &mut self,
        prepared_proposals: Vec<String>,
        admitted_fact_ids: Vec<String>,
    ) {
        self.prepared_proposals = prepared_proposals.clone();
        self.admitted_fact_ids = admitted_fact_ids.clone();
        self.emit(
            StemEventKind::CheckpointPrepared,
            json!({
                "proposal_ids": prepared_proposals,
            }),
        );
        self.emit(
            StemEventKind::ConvergeAdmissionRecorded,
            json!({
                "bridge": "converge-bridge",
                "source_proposal_ids": self.prepared_proposals,
                "admitted_fact_ids": self.admitted_fact_ids,
                "promoted_by": "converge",
            }),
        );
    }

    pub fn advance_server_loops(&mut self, at_ms: u64) {
        self.now_ms = at_ms;
        let mut completed_ids = Vec::new();
        for record in &mut self.server_loops {
            if !record.completed && at_ms >= record.completes_at_ms {
                record.completed = true;
                completed_ids.push(record.server_formation_id.clone());
            }
        }
        for server_formation_id in completed_ids {
            let participant_id = self
                .server_loops
                .iter()
                .find(|record| record.server_formation_id == server_formation_id)
                .map(|record| record.participant_id.clone())
                .expect("server loop exists");
            self.emit(
                StemEventKind::ServerLoopCompleted,
                json!({
                    "participant_id": participant_id,
                    "server_formation_id": server_formation_id,
                }),
            );
        }
    }

    pub fn tick(&mut self, at_ms: u64) {
        self.now_ms = at_ms;
        let mut failures = Vec::new();
        for (participant_id, slot) in &mut self.participants {
            for loop_id in slot.helm.tick(at_ms) {
                failures.push((participant_id.clone(), loop_id));
            }
        }
        for (participant_id, loop_id) in failures {
            self.emit(
                StemEventKind::LocalLoopFailed,
                json!({
                    "participant_id": participant_id,
                    "loop_id": loop_id.as_str(),
                    "reason": "wall-clock budget exhausted",
                }),
            );
        }
        self.advance_server_loops(at_ms);
        self.emit(StemEventKind::ClockTick, json!({ "at_ms": at_ms }));
    }

    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    pub fn event_trace(&self) -> Vec<(u64, StemEventKind, Value)> {
        self.events
            .iter()
            .map(|event| (event.sequence, event.kind, event.payload.clone()))
            .collect()
    }

    /// Event trace with non-deterministic loop ids normalized for replay assertions.
    pub fn normalized_event_trace(&self) -> Vec<(u64, StemEventKind, Value)> {
        let mut id_map = HashMap::<String, String>::new();
        let mut next = 0_u32;
        self.events
            .iter()
            .map(|event| {
                let payload = normalize_loop_ids(&event.payload, &mut id_map, &mut next);
                (event.sequence, event.kind, payload)
            })
            .collect()
    }

    pub fn registry_views(&self) -> HashMap<String, Vec<LoopEntryView<'_>>> {
        self.participants
            .iter()
            .map(|(id, slot)| (id.clone(), loop_entry_views(slot.helm.registry_state())))
            .collect()
    }

    pub fn jsonl_timeline(&self) -> String {
        let mut output = String::new();
        for event in &self.events {
            output.push_str(
                &serde_json::to_string(&json!({
                    "sequence": event.sequence,
                    "at_ms": event.at_ms,
                    "kind": event.kind,
                    "payload": event.payload,
                }))
                .expect("stem event serializes"),
            );
            output.push('\n');
        }
        output
    }

    pub fn markdown_report(&self) -> String {
        let mut output = String::new();
        output.push_str("# Helm Realtime Stem Headless Scenario\n\n");
        output.push_str(&format!("case: `{}`\n", self.case_id));
        if let Some(session_id) = &self.session_id {
            output.push_str(&format!("session: `{session_id}`\n"));
        }
        if let Some(mode) = &self.session_mode {
            output.push_str(&format!("mode: `{mode}`\n"));
        }
        output.push('\n');

        output.push_str("## Participants\n\n");
        for (id, slot) in &self.participants {
            output.push_str(&format!(
                "- `{id}` — {} ({}) — {} loop entries\n",
                slot.display_name,
                slot.role,
                slot.helm.registry_state().len()
            ));
        }
        output.push('\n');

        output.push_str("## Server Loops\n\n");
        if self.server_loops.is_empty() {
            output.push_str("- none\n");
        } else {
            for record in &self.server_loops {
                output.push_str(&format!(
                    "- `{}` / {} / profile {:?} / completed={}\n",
                    record.server_formation_id,
                    record.participant_id,
                    record.profile,
                    record.completed
                ));
            }
        }
        output.push('\n');

        output.push_str("## Converge Handoff\n\n");
        if self.prepared_proposals.is_empty() {
            output.push_str("- proposals: none\n");
        } else {
            for proposal in &self.prepared_proposals {
                output.push_str(&format!("- proposal: `{proposal}`\n"));
            }
        }
        if self.admitted_fact_ids.is_empty() {
            output.push_str("- admitted facts: none\n");
        } else {
            for fact_id in &self.admitted_fact_ids {
                output.push_str(&format!("- admitted fact: `{fact_id}`\n"));
            }
        }
        output.push('\n');

        output.push_str("## Timeline\n\n");
        for event in &self.events {
            output.push_str(&format!(
                "{}. [{:?}] @ {} ms\n",
                event.sequence, event.kind, event.at_ms
            ));
        }
        output.push('\n');
        output
    }

    fn record_client_action(&mut self, participant_id: &str, action: &ClientHelmAction) {
        let label = match action {
            ClientHelmAction::SpawnFormation { .. } => "spawn-formation",
            ClientHelmAction::RequestServerFormation { .. } => "request-server-formation",
            ClientHelmAction::PauseAndInject { .. } => "pause-and-inject",
            ClientHelmAction::Notify { .. } => "notify",
            ClientHelmAction::NoAction => "no-action",
        };
        self.emit(
            StemEventKind::ClientAction,
            json!({
                "participant_id": participant_id,
                "action": label,
            }),
        );
    }

    fn apply_pending_action(&mut self, participant_id: &str, action: ClientHelmAction) {
        let slot = self
            .participants
            .get_mut(participant_id)
            .unwrap_or_else(|| panic!("unknown participant: {participant_id}"));
        match action {
            ClientHelmAction::SpawnFormation { .. } => {
                slot.pending_local_spawn = true;
            }
            ClientHelmAction::RequestServerFormation { seed_context } => {
                slot.pending_server_offload = Some(seed_context.description);
            }
            ClientHelmAction::PauseAndInject { .. } => {
                slot.pending_local_spawn = true;
            }
            ClientHelmAction::Notify { .. } | ClientHelmAction::NoAction => {}
        }
    }

    fn emit(&mut self, kind: StemEventKind, payload: Value) {
        self.events.push(StemEvent {
            sequence: self.events.len() as u64 + 1,
            at_ms: self.now_ms,
            kind,
            payload,
        });
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoopEntryView<'a> {
    pub loop_id: String,
    pub formation_type: String,
    pub description: String,
    pub state_label: &'static str,
    pub kind_label: &'static str,
    pub server_formation_id: Option<String>,
    #[serde(skip)]
    _entry: std::marker::PhantomData<&'a LoopEntry>,
}

fn loop_entry_views(entries: Vec<&LoopEntry>) -> Vec<LoopEntryView<'_>> {
    entries
        .into_iter()
        .map(|entry| LoopEntryView {
            loop_id: entry.loop_id.as_str().to_string(),
            formation_type: entry.formation_type.clone(),
            description: entry.intent.description.clone(),
            state_label: match &entry.state {
                LoopState::Running => "running",
                LoopState::Paused { .. } => "paused",
                LoopState::Completed(_) => "completed",
                LoopState::Failed(_) => "failed",
            },
            kind_label: match &entry.kind {
                LoopKind::Local => "local",
                LoopKind::ServerHandle { .. } => "server_handle",
            },
            server_formation_id: match &entry.kind {
                LoopKind::Local => None,
                LoopKind::ServerHandle {
                    server_formation_id,
                } => Some(server_formation_id.clone()),
            },
            _entry: std::marker::PhantomData,
        })
        .collect()
}

/// Build a deterministic [`SessionPush`] for scripted cases.
pub fn push_script(
    session_id: &str,
    cycle: u32,
    timestamp_ms: u64,
    urgency: UrgencyIntent,
    _objective: &str,
    payload: Value,
) -> SessionPush {
    SessionPush {
        finding_id: FindingId::from_string(format!("finding:{session_id}:{cycle}")),
        urgency_intent: urgency,
        payload,
        session_context: SessionContext {
            session_id: session_id.to_string(),
            phase: "decision".to_string(),
            cycle,
            timestamp_ms,
        },
    }
}

fn push_objective(push: &SessionPush) -> String {
    push.payload
        .get("objective")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("server push: {:?}", push.urgency_intent))
}

fn normalize_loop_ids(
    value: &Value,
    id_map: &mut HashMap<String, String>,
    next: &mut u32,
) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, child) in map {
                if key == "loop_id" {
                    if let Some(id) = child.as_str() {
                        let normalized = id_map
                            .entry(id.to_string())
                            .or_insert_with(|| {
                                let label = format!("loop-{next}");
                                *next += 1;
                                label
                            })
                            .clone();
                        out.insert(key.clone(), Value::String(normalized));
                        continue;
                    }
                }
                out.insert(key.clone(), normalize_loop_ids(child, id_map, next));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| normalize_loop_ids(item, id_map, next))
                .collect(),
        ),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn server_handles_do_not_share_local_running_slot() {
        let mut run = RealtimeStemRun::new("test://slot", 0);
        run.open_session("sess", "burst");
        run.join_participant("alice", "Alice", "strategy");

        let local = push_script(
            "sess",
            1,
            10,
            UrgencyIntent::Informational,
            "local",
            json!({"objective": "local"}),
        );
        run.deliver_push("alice", local);
        run.ack_local_spawn("alice");

        let disruptive = push_script(
            "sess",
            2,
            20,
            UrgencyIntent::Disruptive,
            "server",
            json!({"objective": "server"}),
        );
        run.deliver_push("alice", disruptive);
        run.ack_server_offload(
            "alice",
            "srv-1",
            "dd-analysis",
            ServerLoopProfile::ShortProbe,
        );

        let views = run.registry_views();
        let alice = views.get("alice").expect("alice");
        let local_running = alice
            .iter()
            .filter(|view| view.kind_label == "local" && view.state_label == "running")
            .count();
        let server_handles = alice
            .iter()
            .filter(|view| view.kind_label == "server_handle")
            .count();
        assert_eq!(local_running, 1);
        assert_eq!(server_handles, 1);
    }

    #[test]
    fn budget_exhaustion_marks_local_loop_failed() {
        let case = cases::run_case(InteractiveCaseId::BudgetExhaustion);
        assert!(
            case.events
                .iter()
                .any(|event| event.kind == StemEventKind::LocalLoopFailed)
        );
    }
}
