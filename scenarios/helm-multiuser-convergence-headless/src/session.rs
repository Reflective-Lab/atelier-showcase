// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Multi-user convergence session — the coordinator's view.
//!
//! [`MultiUserConvergenceSession`] holds the shared server-side state
//! (`SessionRegistry`, `PresenceRegistry`, `DecisionLedger`) and drives each
//! participant's `ClientHelm` through a 6-phase arc using a simulated clock.
//! No network, no async, no Converge runtime.

use helm_client::client::{ClientHelmAction, ClientSubmission};
use helm_client::formation::{FormationOutput, TemperatureReading};
use helm_client::ids::LoopId;
use helm_coordination::{
    DecisionLedger, DecisionOutcome, GateDecisionKind, OperatorPrincipal, PresenceRegistry,
    SessionRegistry, SubjectRef,
};
use helm_session_contracts::gate::{GateCondition, GateId, GatedDecision};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::participant::{ParticipantRole, ParticipantSlot};
use crate::server_pool::{SuggestorKind, make_push};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionPhase {
    Orientation,
    Exploration,
    Convergence,
    Gate,
    Integration,
    Closeout,
}

impl SessionPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Orientation => "orientation",
            Self::Exploration => "exploration",
            Self::Convergence => "convergence",
            Self::Gate => "gate",
            Self::Integration => "integration",
            Self::Closeout => "closeout",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConvergenceEventKind {
    SessionOpened,
    PresenceFocused,
    PresenceClaimed,
    PushDispatched,
    FormationSpawned,
    FormationCompleted,
    GateOpen,
    GateDecision,
    GateIdempotent,
    GateConflict,
    TemperatureRecorded,
    PresenceReleased,
    SessionClosed,
    PhaseEntered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceEvent {
    pub sequence: u64,
    pub simulated_ms: u64,
    pub phase: SessionPhase,
    pub kind: ConvergenceEventKind,
    pub actor: String,
    pub payload: Value,
}

pub struct MultiUserConvergenceSession {
    pub workspace_id: String,
    pub session_id: String,
    pub session_registry: SessionRegistry,
    pub presence_registry: PresenceRegistry,
    pub decision_ledger: DecisionLedger,
    pub participants: Vec<ParticipantSlot>,
    pub events: Vec<ConvergenceEvent>,
    pub simulated_ms: u64,
    pub current_phase: SessionPhase,
}

impl MultiUserConvergenceSession {
    pub fn new(workspace_id: impl Into<String>, session_id: impl Into<String>) -> Self {
        let workspace_id = workspace_id.into();
        let session_id = session_id.into();
        let session_registry = SessionRegistry::default();
        let presence_registry = PresenceRegistry::new();
        let decision_ledger = DecisionLedger::new();

        let roles = [
            ParticipantRole::Leader,
            ParticipantRole::Analyst,
            ParticipantRole::Skeptic,
            ParticipantRole::Observer,
        ];

        let participants: Vec<_> = roles
            .iter()
            .map(|&role| ParticipantSlot::open(role, &workspace_id, &session_registry))
            .collect();

        let mut session = Self {
            workspace_id,
            session_id,
            session_registry,
            presence_registry,
            decision_ledger,
            participants,
            events: Vec::new(),
            simulated_ms: 0,
            current_phase: SessionPhase::Orientation,
        };

        for participant in &session.participants {
            session.events.push(ConvergenceEvent {
                sequence: session.events.len() as u64 + 1,
                simulated_ms: 0,
                phase: SessionPhase::Orientation,
                kind: ConvergenceEventKind::SessionOpened,
                actor: participant.principal.actor_id.clone(),
                payload: json!({
                    "session_id": participant.session.id,
                    "role": participant.role,
                    "workspace_id": participant.principal.workspace_id,
                }),
            });
        }

        session
    }

    // ── Phase drivers ─────────────────────────────────────────────────────

    /// Phase 1 (0–10 min): Advisory + Informational pushes; all participants focus.
    pub fn run_orientation(&mut self) {
        self.enter_phase(SessionPhase::Orientation);

        // All participants focus on the session subject.
        let subject = SubjectRef::run(self.session_id.clone());
        let focus_participants: Vec<(uuid::Uuid, OperatorPrincipal)> = self
            .participants
            .iter()
            .map(|p| (p.session.id, p.principal.clone()))
            .collect();
        for (session_uuid, principal) in focus_participants {
            let (entry, _) =
                self.presence_registry
                    .focus(session_uuid, principal.clone(), subject.clone());
            self.record_event(
                ConvergenceEventKind::PresenceFocused,
                &principal,
                json!({
                    "subject": subject.to_string(),
                    "claimed": entry.claimed,
                }),
            );
        }

        // LlmSynthesis Advisory — all clients handle.
        for cycle in 1u32..=3 {
            self.advance_ms(2 * 60 * 1000);
            let push = make_push(
                SuggestorKind::LlmSynthesis,
                &self.session_id.clone(),
                SessionPhase::Orientation.as_str(),
                cycle,
                self.simulated_ms,
            );
            self.record_event(
                ConvergenceEventKind::PushDispatched,
                &self.system_principal(),
                json!({
                    "suggestor": push.suggestor,
                    "urgency": push.push.urgency_intent,
                    "finding_id": push.push.finding_id.as_str(),
                }),
            );
            // Deliver to all participants.
            for i in 0..self.participants.len() {
                let action = self.participants[i].helm.handle_push(push.push.clone());
                self.process_action(i, action, SessionPhase::Orientation);
            }
        }

        // PrismAnalytics Informational — NoAction on all clients.
        self.advance_ms(60 * 1_000);
        let push = make_push(
            SuggestorKind::PrismAnalytics,
            &self.session_id.clone(),
            SessionPhase::Orientation.as_str(),
            1,
            self.simulated_ms,
        );
        self.record_event(
            ConvergenceEventKind::PushDispatched,
            &self.system_principal(),
            json!({
                "suggestor": push.suggestor,
                "urgency": push.push.urgency_intent,
                "finding_id": push.push.finding_id.as_str(),
            }),
        );
        for i in 0..self.participants.len() {
            let action = self.participants[i].helm.handle_push(push.push.clone());
            // Informational → NoAction: nothing to spawn, just verify.
            self.process_action(i, action, SessionPhase::Orientation);
        }
    }

    /// Phase 2 (10–25 min): Disruptive pushes from FeroxOptimizer; Leader/Analyst claim.
    pub fn run_exploration(&mut self) {
        self.enter_phase(SessionPhase::Exploration);

        // Leader and Analyst soft-claim the exploration subject (both may act — optimistic).
        let exploration_subject = SubjectRef::new("exploration", "phase:exploration:cycle:1");
        for role in [ParticipantRole::Leader, ParticipantRole::Analyst] {
            let principal = self.principal_for(role).clone();
            let session_id = self.session_id_for(role);
            let (entry, _) = self.presence_registry.claim(
                session_id,
                principal.clone(),
                exploration_subject.clone(),
            );
            self.record_event(
                ConvergenceEventKind::PresenceClaimed,
                &principal,
                json!({
                    "subject": exploration_subject.to_string(),
                    "claimed": entry.claimed,
                }),
            );
        }

        // FeroxOptimizer Disruptive — spawns parallel formation on active clients.
        for cycle in 1u32..=2 {
            self.advance_ms(5 * 60 * 1000);
            let push = make_push(
                SuggestorKind::FeroxOptimizer,
                &self.session_id.clone(),
                SessionPhase::Exploration.as_str(),
                cycle,
                self.simulated_ms,
            );
            self.record_event(
                ConvergenceEventKind::PushDispatched,
                &self.system_principal(),
                json!({
                    "suggestor": push.suggestor,
                    "urgency": push.push.urgency_intent,
                    "finding_id": push.push.finding_id.as_str(),
                }),
            );
            for i in 0..self.participants.len() {
                let action = self.participants[i].helm.handle_push(push.push.clone());
                self.process_action(i, action, SessionPhase::Exploration);
            }
        }
    }

    /// Phase 3 (25–40 min): Preemptive push from ArbiterPolicy; Skeptic challenges.
    pub fn run_convergence(&mut self) {
        self.enter_phase(SessionPhase::Convergence);

        // Skeptic claims the dissent subject.
        let dissent_subject = SubjectRef::new("dissent", "hypothesis:convergence:h-1");
        let skeptic_principal = self.principal_for(ParticipantRole::Skeptic).clone();
        let skeptic_session = self.session_id_for(ParticipantRole::Skeptic);
        let (entry, _) = self.presence_registry.claim(
            skeptic_session,
            skeptic_principal.clone(),
            dissent_subject.clone(),
        );
        self.record_event(
            ConvergenceEventKind::PresenceClaimed,
            &skeptic_principal,
            json!({
                "subject": dissent_subject.to_string(),
                "claimed": entry.claimed,
            }),
        );

        // ArbiterPolicy Preemptive — suspends active formations.
        self.advance_ms(10 * 60 * 1000);
        let push = make_push(
            SuggestorKind::ArbiterPolicy,
            &self.session_id.clone(),
            SessionPhase::Convergence.as_str(),
            1,
            self.simulated_ms,
        );
        self.record_event(
            ConvergenceEventKind::PushDispatched,
            &self.system_principal(),
            json!({
                "suggestor": push.suggestor,
                "urgency": push.push.urgency_intent,
                "finding_id": push.push.finding_id.as_str(),
            }),
        );
        for i in 0..self.participants.len() {
            let action = self.participants[i].helm.handle_push(push.push.clone());
            self.process_action(i, action, SessionPhase::Convergence);
        }

        // Additional LlmSynthesis advisory for convergence narrowing.
        self.advance_ms(5 * 60 * 1000);
        let push = make_push(
            SuggestorKind::LlmSynthesis,
            &self.session_id.clone(),
            SessionPhase::Convergence.as_str(),
            1,
            self.simulated_ms,
        );
        self.record_event(
            ConvergenceEventKind::PushDispatched,
            &self.system_principal(),
            json!({
                "suggestor": push.suggestor,
                "urgency": push.push.urgency_intent,
                "finding_id": push.push.finding_id.as_str(),
            }),
        );
        for i in 0..self.participants.len() {
            let action = self.participants[i].helm.handle_push(push.push.clone());
            self.process_action(i, action, SessionPhase::Convergence);
        }
    }

    /// Phase 4 (40–45 min): HITL gate — DecisionLedger records, idempotent, conflict.
    ///
    /// Returns the gate ref_id so the caller can verify the ledger state.
    pub fn run_gate(&mut self) -> String {
        self.enter_phase(SessionPhase::Gate);
        self.advance_ms(2 * 60 * 1000);

        let gate_id = GateId::from_string("gate:convergence:session:main");
        let gate_ref = gate_id.as_str().to_string();
        let gate = GatedDecision {
            gate_id: gate_id.clone(),
            condition: GateCondition::AnyParticipant,
            payload: json!({
                "question": "Accept convergence result for phase integration?",
                "convergence_phase": SessionPhase::Gate.as_str(),
            }),
            deadline: None,
        };

        // Deliver the gate to all clients.
        for participant in &mut self.participants {
            participant.helm.handle_gate(gate.clone());
        }
        self.record_event(
            ConvergenceEventKind::GateOpen,
            &self.system_principal(),
            json!({
                "gate_id": gate_ref,
                "condition": "any-participant",
            }),
        );

        let leader_principal = self.principal_for(ParticipantRole::Leader).clone();
        let skeptic_principal = self.principal_for(ParticipantRole::Skeptic).clone();
        let analyst_principal = self.principal_for(ParticipantRole::Analyst).clone();

        // Leader approves first.
        match self.decision_ledger.record(
            &gate_ref,
            leader_principal.clone(),
            GateDecisionKind::Approve,
            Some("Leader: accept synthesis result".to_string()),
        ) {
            DecisionOutcome::Recorded(record) => {
                self.record_event(
                    ConvergenceEventKind::GateDecision,
                    &leader_principal,
                    json!({
                        "gate_id": gate_ref,
                        "decision": record.decision,
                        "decision_id": record.decision_id,
                        "first": true,
                    }),
                );
            }
            _ => unreachable!("first decision must be Recorded"),
        }

        // Analyst also approves — idempotent (returns original record).
        match self.decision_ledger.record(
            &gate_ref,
            analyst_principal.clone(),
            GateDecisionKind::Approve,
            None,
        ) {
            DecisionOutcome::Idempotent(record) => {
                self.record_event(
                    ConvergenceEventKind::GateIdempotent,
                    &analyst_principal,
                    json!({
                        "gate_id": gate_ref,
                        "original_decider": record.principal.actor_id,
                        "idempotent": true,
                    }),
                );
            }
            _ => unreachable!("same-decision from second operator must be Idempotent"),
        }

        // Skeptic tries to reject — conflict (rejected; original decision preserved).
        match self.decision_ledger.record(
            &gate_ref,
            skeptic_principal.clone(),
            GateDecisionKind::Reject,
            Some("Skeptic: dissent — insufficient evidence".to_string()),
        ) {
            DecisionOutcome::Conflict {
                existing,
                attempted,
                attempted_by,
            } => {
                self.record_event(
                    ConvergenceEventKind::GateConflict,
                    &skeptic_principal,
                    json!({
                        "gate_id": gate_ref,
                        "existing_decision": existing.decision,
                        "existing_decider": existing.principal.actor_id,
                        "attempted_decision": attempted,
                        "attempted_by": attempted_by.actor_id,
                        "outcome": "conflict-rejected-original-preserved",
                    }),
                );
            }
            _ => unreachable!("divergent decision must produce Conflict"),
        }

        // Leader responds to gate via ClientHelm surface too.
        let leader_idx = self.participant_index(ParticipantRole::Leader);
        self.participants[leader_idx]
            .helm
            .respond_to_gate(&gate_id, json!({"decision": "approve"}));

        gate_ref
    }

    /// Phase 5 (45–55 min): Advisory pushes post-gate; integration formation.
    pub fn run_integration(&mut self) {
        self.enter_phase(SessionPhase::Integration);

        for cycle in 1u32..=2 {
            self.advance_ms(4 * 60 * 1000);
            let push = make_push(
                SuggestorKind::LlmSynthesis,
                &self.session_id.clone(),
                SessionPhase::Integration.as_str(),
                cycle,
                self.simulated_ms,
            );
            self.record_event(
                ConvergenceEventKind::PushDispatched,
                &self.system_principal(),
                json!({
                    "suggestor": push.suggestor,
                    "urgency": push.push.urgency_intent,
                    "finding_id": push.push.finding_id.as_str(),
                }),
            );
            for i in 0..self.participants.len() {
                let action = self.participants[i].helm.handle_push(push.push.clone());
                self.process_action(i, action, SessionPhase::Integration);
            }
        }

        // Drain any pending temperature submissions from all clients.
        for participant in &mut self.participants {
            let submissions = participant.helm.drain_submissions();
            for sub in submissions {
                if let ClientSubmission::Temperature(pending) = sub {
                    self.events.push(ConvergenceEvent {
                        sequence: self.events.len() as u64 + 1,
                        simulated_ms: self.simulated_ms,
                        phase: SessionPhase::Integration,
                        kind: ConvergenceEventKind::TemperatureRecorded,
                        actor: participant.principal.actor_id.clone(),
                        payload: json!({
                            "position": pending.signal.position,
                            "conviction": pending.signal.conviction,
                            "subject_ref": pending.signal.subject_ref,
                        }),
                    });
                }
            }
        }
    }

    /// Phase 6 (55–60 min): Release presence, close sessions.
    pub fn run_closeout(&mut self) {
        self.enter_phase(SessionPhase::Closeout);
        self.advance_ms(5 * 60 * 1000);

        // Release soft-claims.
        let exploration_subject = SubjectRef::new("exploration", "phase:exploration:cycle:1");
        for role in [ParticipantRole::Leader, ParticipantRole::Analyst] {
            let principal = self.principal_for(role).clone();
            let session_id = self.session_id_for(role);
            if let Some(entry) = self
                .presence_registry
                .release(session_id, &exploration_subject)
            {
                self.record_event(
                    ConvergenceEventKind::PresenceReleased,
                    &principal,
                    json!({
                        "subject": exploration_subject.to_string(),
                        "claimed": entry.claimed,
                    }),
                );
            }
        }

        // Close all sessions.
        for participant in &self.participants {
            if let Some(closed) = self.session_registry.close(participant.session.id) {
                self.events.push(ConvergenceEvent {
                    sequence: self.events.len() as u64 + 1,
                    simulated_ms: self.simulated_ms,
                    phase: SessionPhase::Closeout,
                    kind: ConvergenceEventKind::SessionClosed,
                    actor: participant.principal.actor_id.clone(),
                    payload: json!({
                        "session_id": closed.id,
                        "opened_at": closed.opened_at,
                    }),
                });
            }
        }
    }

    // ── Inspection ────────────────────────────────────────────────────────

    /// Active sessions still open in the registry (post-closeout: should be 0).
    pub fn active_session_count(&self) -> usize {
        self.session_registry.active(&self.workspace_id).len()
    }

    /// Presence entries still in the registry for this workspace.
    pub fn presence_count(&self) -> usize {
        self.presence_registry.list(&self.workspace_id, None).len()
    }

    /// Check the decision ledger for a gate ref.
    pub fn gate_decision(&self, ref_id: &str) -> Option<helm_coordination::DecisionRecord> {
        self.decision_ledger.get(ref_id)
    }

    /// Total formations spawned across all participants.
    pub fn total_formations_spawned(&self) -> usize {
        self.events
            .iter()
            .filter(|event| event.kind == ConvergenceEventKind::FormationSpawned)
            .count()
    }

    /// Event trace for testing.
    pub fn event_trace(&self) -> Vec<(u64, ConvergenceEventKind, Value)> {
        self.events
            .iter()
            .map(|e| (e.sequence, e.kind, e.payload.clone()))
            .collect()
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    fn enter_phase(&mut self, phase: SessionPhase) {
        self.current_phase = phase;
        self.events.push(ConvergenceEvent {
            sequence: self.events.len() as u64 + 1,
            simulated_ms: self.simulated_ms,
            phase,
            kind: ConvergenceEventKind::PhaseEntered,
            actor: "system".to_string(),
            payload: json!({ "phase": phase.as_str() }),
        });
    }

    fn advance_ms(&mut self, delta: u64) {
        self.simulated_ms += delta;
        // Tick all clients to advance their wall-clock budgets.
        for participant in &mut self.participants {
            let _ = participant.helm.tick(self.simulated_ms);
        }
    }

    fn process_action(
        &mut self,
        participant_idx: usize,
        action: ClientHelmAction,
        phase: SessionPhase,
    ) {
        let actor = self.participants[participant_idx]
            .principal
            .actor_id
            .clone();
        match action {
            ClientHelmAction::SpawnFormation {
                ref loop_id,
                ref seed_context,
            } => {
                self.events.push(ConvergenceEvent {
                    sequence: self.events.len() as u64 + 1,
                    simulated_ms: self.simulated_ms,
                    phase,
                    kind: ConvergenceEventKind::FormationSpawned,
                    actor: actor.clone(),
                    payload: json!({
                        "loop_id": loop_id.as_str(),
                        "description": seed_context.description,
                        "action": "spawn-formation",
                    }),
                });
                // Immediately complete the formation — headless, no real Converge engine.
                self.complete_formation(participant_idx, loop_id.clone(), phase);
            }
            ClientHelmAction::PauseAndInject {
                ref paused_id,
                ref injected_context,
            } => {
                self.events.push(ConvergenceEvent {
                    sequence: self.events.len() as u64 + 1,
                    simulated_ms: self.simulated_ms,
                    phase,
                    kind: ConvergenceEventKind::FormationSpawned,
                    actor: actor.clone(),
                    payload: json!({
                        "paused_loop_id": paused_id.as_str(),
                        "injected_description": injected_context.description,
                        "action": "pause-and-inject",
                    }),
                });
                // Spawn a fresh formation with the injected context, then complete it.
                let new_push = make_push(
                    SuggestorKind::ArbiterPolicy,
                    &self.session_id.clone(),
                    phase.as_str(),
                    99,
                    self.simulated_ms,
                );
                let new_action = self.participants[participant_idx]
                    .helm
                    .handle_push(new_push.push);
                if let ClientHelmAction::SpawnFormation { loop_id, .. } = new_action {
                    self.complete_formation(participant_idx, loop_id, phase);
                }
            }
            ClientHelmAction::RequestServerFormation { ref seed_context } => {
                self.events.push(ConvergenceEvent {
                    sequence: self.events.len() as u64 + 1,
                    simulated_ms: self.simulated_ms,
                    phase,
                    kind: ConvergenceEventKind::FormationSpawned,
                    actor: actor.clone(),
                    payload: json!({
                        "description": seed_context.description,
                        "action": "request-server-formation",
                    }),
                });
            }
            ClientHelmAction::Notify {
                urgency,
                ref message,
            } => {
                self.events.push(ConvergenceEvent {
                    sequence: self.events.len() as u64 + 1,
                    simulated_ms: self.simulated_ms,
                    phase,
                    kind: ConvergenceEventKind::PushDispatched,
                    actor: actor.clone(),
                    payload: json!({
                        "action": "notify",
                        "urgency": format!("{urgency:?}"),
                        "message": message,
                    }),
                });
            }
            ClientHelmAction::NoAction => {
                // Informational push — no formation, no interrupt. Expected for PrismAnalytics.
            }
        }
    }

    fn complete_formation(&mut self, participant_idx: usize, loop_id: LoopId, phase: SessionPhase) {
        let actor = self.participants[participant_idx]
            .principal
            .actor_id
            .clone();
        let output = simulated_formation_output(&actor, phase);
        self.participants[participant_idx]
            .helm
            .formation_completed(&loop_id, output, None);
        self.events.push(ConvergenceEvent {
            sequence: self.events.len() as u64 + 1,
            simulated_ms: self.simulated_ms,
            phase,
            kind: ConvergenceEventKind::FormationCompleted,
            actor: actor.clone(),
            payload: json!({
                "loop_id": loop_id.as_str(),
                "proposals": 1,
            }),
        });
    }

    fn system_principal(&self) -> OperatorPrincipal {
        OperatorPrincipal::new(
            "system",
            "System",
            application_kernel::ActorKind::System,
            &self.workspace_id,
        )
    }

    fn principal_for(&self, role: ParticipantRole) -> &OperatorPrincipal {
        self.participants
            .iter()
            .find(|p| p.role == role)
            .map(|p| &p.principal)
            .expect("all four roles are always present")
    }

    fn session_id_for(&self, role: ParticipantRole) -> uuid::Uuid {
        self.participants
            .iter()
            .find(|p| p.role == role)
            .map(|p| p.session.id)
            .expect("all four roles are always present")
    }

    fn participant_index(&self, role: ParticipantRole) -> usize {
        self.participants
            .iter()
            .position(|p| p.role == role)
            .expect("all four roles are always present")
    }

    fn record_event(
        &mut self,
        kind: ConvergenceEventKind,
        principal: &OperatorPrincipal,
        payload: Value,
    ) {
        self.events.push(ConvergenceEvent {
            sequence: self.events.len() as u64 + 1,
            simulated_ms: self.simulated_ms,
            phase: self.current_phase,
            kind,
            actor: principal.actor_id.clone(),
            payload,
        });
    }
}

/// Produce a deterministic formation output — one proposal, temperature reading.
fn simulated_formation_output(actor: &str, phase: SessionPhase) -> FormationOutput {
    FormationOutput {
        proposals: vec![serde_json::json!({
            "actor": actor,
            "phase": phase.as_str(),
            "proposal": "formation output",
        })],
        temperature: Some(TemperatureReading {
            position: "agree".to_string(),
            conviction: "high".to_string(),
            subject_ref: format!("session://convergence/{}", phase.as_str()),
        }),
    }
}
