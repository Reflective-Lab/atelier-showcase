// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Headless demonstration of the Helm Coordination Layer M0 shape.
//!
//! The structs are scenario-local on purpose. They give Arena a deterministic
//! artifact to prove against existing Helm readiness contracts before a
//! `helm-coordination` crate is cut.

use application_storage::AppConfig;
use helm_module_contracts::HelmModuleState;
use helm_operator_control::{
    AdapterReceiptStatus, EvidenceReadinessStatus, JobEvidenceStatus, JobReadinessPacket,
    JobReadinessPacketInput, JobVerdict, LiveOperatorControlSnapshot, LiveReadinessEvidence,
    OperatorControlError, OperatorControlModule, OperatorControlReadinessFeed,
    OperatorLedgerRecordKind, ReceiptFamily, job_readiness_packet_ledger_entry,
    job_readiness_packet_payload_hash,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const HUMAN_COUNT: usize = 50;
pub const AGENT_COUNT: usize = 6;
pub const DYNAMIC_USER_LIMIT: usize = 100;
pub const CROWD_USER_COUNTS: [usize; 3] = [5, 30, 100];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateOutcome {
    Approved,
    Rejected,
    TimedOut,
}

impl GateOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::TimedOut => "timed-out",
        }
    }

    const fn adapter_status(self) -> AdapterReceiptStatus {
        match self {
            Self::Approved => AdapterReceiptStatus::Succeeded,
            Self::Rejected | Self::TimedOut => AdapterReceiptStatus::Rejected,
        }
    }

    const fn job_verdict(self) -> JobVerdict {
        match self {
            Self::Approved => JobVerdict::Satisfied,
            Self::Rejected => JobVerdict::Blocked,
            Self::TimedOut => JobVerdict::Exhausted,
        }
    }

    const fn allows_converge_admission(self) -> bool {
        matches!(self, Self::Approved)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParticipantKind {
    Human,
    Agent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantRef {
    pub id: String,
    pub kind: ParticipantKind,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contribution {
    pub id: String,
    pub participant_id: String,
    pub epoch: u32,
    pub body: String,
    pub signed: bool,
    pub provenance: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoordinationEventKind {
    SessionOpened,
    ContributionRecorded,
    OpinionGathered,
    Mixed,
    Matched,
    FormationCheckpoint,
    GateRequested,
    GateDecision,
    ConvergeProposalPrepared,
    ConvergeAdmissionRecorded,
    ConsensusMeasured,
    ConclusionDrafted,
    DecisionRecorded,
    ReadinessSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoordinationEvent {
    pub sequence: u64,
    pub kind: CoordinationEventKind,
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct HeadlessCoordinationRun {
    pub session_id: String,
    pub participants: Vec<ParticipantRef>,
    pub contributions: Vec<Contribution>,
    pub events: Vec<CoordinationEvent>,
    pub prepared_proposals: Vec<String>,
    pub admitted_fact_ids: Vec<String>,
    pub gate_outcome: GateOutcome,
}

impl HeadlessCoordinationRun {
    pub fn scripted_burst() -> Self {
        Self::scripted_burst_with_gate(GateOutcome::Approved)
    }

    pub fn scripted_burst_with_gate(gate_outcome: GateOutcome) -> Self {
        let mut run = Self {
            session_id: "coordination://burst/session-001".to_string(),
            participants: participants(),
            contributions: Vec::new(),
            events: Vec::new(),
            prepared_proposals: Vec::new(),
            admitted_fact_ids: Vec::new(),
            gate_outcome,
        };

        run.emit(
            CoordinationEventKind::SessionOpened,
            json!({
                "session_id": run.session_id.clone(),
                "mode": "convened-burst",
                "participant_count": run.participants.len(),
            }),
        );

        for arrival in deterministic_concurrent_arrivals(&run.participants) {
            run.record_contribution_from_arrival(
                &arrival.participant,
                format!(
                    "{} contribution for checkpoint synthesis",
                    arrival.participant.id
                ),
                Some(arrival.batch),
                &arrival.ordering_key,
            )
            .expect("scripted contributions are signed and provenanced");
        }

        run.emit(
            CoordinationEventKind::Mixed,
            json!({
                "strategy": "role-balanced-topic-windows",
                "input_contributions": run.contributions.len(),
                "groups": ["strategy", "operations", "risk", "frontline"],
            }),
        );
        run.emit(
            CoordinationEventKind::Matched,
            json!({
                "strategy": "agent-to-branch-assignment",
                "agent_count": AGENT_COUNT,
                "branches": ["minority-hypothesis", "execution-risk", "decision-owner"],
            }),
        );
        run.emit(
            CoordinationEventKind::FormationCheckpoint,
            json!({
                "formation": "bounded-synthesis",
                "checkpoint_id": "checkpoint:001",
                "input_contributions": run.contributions.len(),
                "output_candidates": 4,
            }),
        );
        run.emit(
            CoordinationEventKind::GateRequested,
            json!({
                "gate_id": "gate:checkpoint:001",
                "reason": "operator reviews bounded synthesis before Converge admission",
            }),
        );
        run.emit(
            CoordinationEventKind::GateDecision,
            json!({
                "gate_id": "gate:checkpoint:001",
                "decision": gate_outcome,
                "receipt_kind": "decision-receipt",
                "operator": "operator://atelier/showcase",
            }),
        );

        if gate_outcome.allows_converge_admission() {
            run.prepared_proposals = vec![
                "proposal:coordination:checkpoint:001:core-claim".to_string(),
                "proposal:coordination:checkpoint:001:dissent-preserved".to_string(),
                "proposal:coordination:checkpoint:001:next-probe".to_string(),
            ];
        }
        run.emit(
            CoordinationEventKind::ConvergeProposalPrepared,
            json!({
                "proposal_ids": run.prepared_proposals.clone(),
                "promoted_facts": [],
                "withheld_reason": if gate_outcome.allows_converge_admission() {
                    Value::Null
                } else {
                    json!(gate_outcome)
                },
            }),
        );

        if gate_outcome.allows_converge_admission() {
            run.admitted_fact_ids = vec![
                "fact:coordination:checkpoint:001:core-claim".to_string(),
                "fact:coordination:checkpoint:001:dissent-preserved".to_string(),
            ];
            run.emit(
                CoordinationEventKind::ConvergeAdmissionRecorded,
                json!({
                    "bridge": "converge-bridge",
                    "source_proposal_ids": run.prepared_proposals.clone(),
                    "admitted_fact_ids": run.admitted_fact_ids.clone(),
                    "promoted_by": "converge",
                }),
            );
        }

        run.emit(
            CoordinationEventKind::ReadinessSnapshot,
            json!({
                "receipt_id": run.coordination_receipt_id_for_event_count(run.events.len() + 1),
                "authorizes_domain_action": false,
            }),
        );

        run
    }

    pub fn record_contribution(
        &mut self,
        participant: &ParticipantRef,
        body: String,
    ) -> Result<(), String> {
        self.record_contribution_from_arrival(participant, body, None, &participant.id)
    }

    fn record_contribution_from_arrival(
        &mut self,
        participant: &ParticipantRef,
        body: String,
        arrival_batch: Option<usize>,
        ordering_key: &str,
    ) -> Result<(), String> {
        let contribution = Contribution {
            id: format!("contribution:{:03}", self.contributions.len() + 1),
            participant_id: participant.id.clone(),
            epoch: 1,
            body,
            signed: true,
            provenance: format!("signed://{}", participant.id),
        };

        if !contribution.signed || contribution.provenance.is_empty() {
            return Err("coordination contributions must be signed and provenanced".to_string());
        }

        self.emit(
            CoordinationEventKind::ContributionRecorded,
            json!({
                "contribution_id": contribution.id.clone(),
                "participant_id": contribution.participant_id.clone(),
                "participant_kind": participant.kind,
                "epoch": contribution.epoch,
                "signed": contribution.signed,
                "suggestion_only": true,
                "arrival_batch": arrival_batch,
                "ordering_key": ordering_key,
            }),
        );
        self.contributions.push(contribution);
        Ok(())
    }

    pub fn reject_direct_fact_mutation(&mut self, participant_id: &str) -> Result<(), String> {
        let _ = participant_id;
        Err(
            "Helm coordination records suggestions and receipts; Converge alone promotes facts"
                .to_string(),
        )
    }

    pub fn coordination_receipt_id(&self) -> String {
        self.coordination_receipt_id_for_event_count(self.events.len())
    }

    fn coordination_receipt_id_for_event_count(&self, event_count: usize) -> String {
        format!(
            "receipt:{}:{}:{}:{}",
            self.session_id,
            self.participants.len(),
            self.contributions.len(),
            event_count
        )
    }

    pub fn event_trace(&self) -> Vec<(u64, CoordinationEventKind, Value)> {
        self.events
            .iter()
            .map(|event| (event.sequence, event.kind, event.payload.clone()))
            .collect()
    }

    pub fn readiness_snapshot(&self) -> LiveOperatorControlSnapshot {
        let handoff_status = if self.gate_outcome.allows_converge_admission() {
            EvidenceReadinessStatus::Present
        } else {
            EvidenceReadinessStatus::Blocked
        };
        let gate_trace_link = format!("trace:helm.gate.{}", self.gate_outcome.as_str());
        let handoff_label = if self.gate_outcome.allows_converge_admission() {
            "accepted checkpoint outputs are prepared as proposals for Converge admission"
                .to_string()
        } else {
            format!(
                "checkpoint outputs are withheld because the gate outcome is {}",
                self.gate_outcome.as_str()
            )
        };
        let admission_label = if self.gate_outcome.allows_converge_admission() {
            "Converge admission records final promoted fact ids".to_string()
        } else {
            format!(
                "Converge admission is blocked because the gate outcome is {}",
                self.gate_outcome.as_str()
            )
        };
        let operator_actions = if self.gate_outcome.allows_converge_admission() {
            vec![
                "review the checkpoint receipt".to_string(),
                "send accepted proposals through Converge admission".to_string(),
                "continue the coordination session after admission outcome is known".to_string(),
            ]
        } else {
            vec![
                "review the checkpoint receipt".to_string(),
                "record the blocked gate outcome".to_string(),
                "continue or reopen the coordination session without admitting facts".to_string(),
            ]
        };

        let packet = JobReadinessPacket::new(JobReadinessPacketInput {
            package_id: "helm.coordination.m0.burst".to_string(),
            truth_version: "helm-coordination.m0.v0".to_string(),
            domain_hint: "helm-coordination.headless".to_string(),
            job_key: "coordination-checkpoint".to_string(),
            subject_ref: self.session_id.clone(),
            adapter_receipt_id: self.coordination_receipt_id(),
            adapter_status: self.gate_outcome.adapter_status(),
            verdict: Some(self.gate_outcome.job_verdict()),
            authorizes_domain_action: false,
            evidence_status: vec![
                JobEvidenceStatus {
                    clause_id: "coordination.evidence.contributions".to_string(),
                    clause_key: "signed_contributions".to_string(),
                    label: "all human and agent contributions are signed, provenanced suggestions"
                        .to_string(),
                    status: EvidenceReadinessStatus::Present,
                    fact_ids: Vec::new(),
                    evidence_refs: vec![self.coordination_receipt_id()],
                    trace_links: vec!["trace:coordination.contributions.ordered".to_string()],
                    concern_record_ids: Vec::new(),
                },
                JobEvidenceStatus {
                    clause_id: "coordination.evidence.checkpoint".to_string(),
                    clause_key: "bounded_formation_checkpoint".to_string(),
                    label: "checkpoint invoked a bounded formation before any Converge handoff"
                        .to_string(),
                    status: EvidenceReadinessStatus::Present,
                    fact_ids: Vec::new(),
                    evidence_refs: vec!["checkpoint:001".to_string()],
                    trace_links: vec!["trace:organism.formation.bounded-synthesis".to_string()],
                    concern_record_ids: Vec::new(),
                },
                JobEvidenceStatus {
                    clause_id: "coordination.evidence.gate".to_string(),
                    clause_key: "gate_decision_receipt".to_string(),
                    label: "operator gate is recorded as a decision receipt, not a fact"
                        .to_string(),
                    status: EvidenceReadinessStatus::Present,
                    fact_ids: Vec::new(),
                    evidence_refs: vec!["gate:checkpoint:001".to_string()],
                    trace_links: vec![gate_trace_link],
                    concern_record_ids: Vec::new(),
                },
                JobEvidenceStatus {
                    clause_id: "coordination.evidence.converge_handoff".to_string(),
                    clause_key: "converge_proposals_prepared".to_string(),
                    label: handoff_label,
                    status: handoff_status,
                    fact_ids: Vec::new(),
                    evidence_refs: self.prepared_proposals.clone(),
                    trace_links: vec!["trace:converge.admission.pending".to_string()],
                    concern_record_ids: Vec::new(),
                },
                JobEvidenceStatus {
                    clause_id: "coordination.evidence.converge_admission".to_string(),
                    clause_key: "converge_admission_outcome".to_string(),
                    label: admission_label,
                    status: handoff_status,
                    fact_ids: self.admitted_fact_ids.clone(),
                    evidence_refs: if self.gate_outcome.allows_converge_admission() {
                        vec!["converge:admission:checkpoint:001".to_string()]
                    } else {
                        Vec::new()
                    },
                    trace_links: vec!["trace:converge.admission.recorded".to_string()],
                    concern_record_ids: Vec::new(),
                },
                JobEvidenceStatus {
                    clause_id: "coordination.failure.direct_fact_mutation".to_string(),
                    clause_key: "no_direct_fact_mutation".to_string(),
                    label: "coordination events do not directly mutate governed context"
                        .to_string(),
                    status: EvidenceReadinessStatus::Present,
                    fact_ids: Vec::new(),
                    evidence_refs: vec!["guard:humans-as-suggestors".to_string()],
                    trace_links: vec!["trace:converge.promotes-facts".to_string()],
                    concern_record_ids: Vec::new(),
                },
            ],
            fuzzy_trace: None,
            verifier_forbidden_actions: vec![
                "do not mutate governed context from a contribution".to_string(),
                "do not treat a gate decision receipt as a fact".to_string(),
                "do not infer app domain authority from Helm readiness".to_string(),
            ],
            operator_actions,
        })
        .expect("coordination readiness packet builds");

        let ledger_entry = job_readiness_packet_ledger_entry(
            1,
            &packet,
            vec![packet.adapter_receipt_id.clone()],
            "Headless coordination checkpoint ready for Converge admission",
        )
        .expect("coordination ledger entry builds");

        LiveOperatorControlSnapshot::new(packet, vec![ledger_entry])
    }

    pub fn shell_module_state() -> HelmModuleState {
        OperatorControlModule::new(AppConfig::default()).module_state()
    }

    pub fn live_module_state(&self) -> HelmModuleState {
        OperatorControlModule::new(AppConfig::default())
            .with_live_readiness_feed(std::sync::Arc::new(StaticCoordinationFeed {
                snapshots: vec![self.readiness_snapshot()],
            }))
            .module_state()
    }

    pub fn jsonl_timeline(&self) -> String {
        let mut output = String::new();
        for event in &self.events {
            output.push_str(
                &serde_json::to_string(&json!({
                    "sequence": event.sequence,
                    "kind": event.kind,
                    "payload": event.payload,
                }))
                .expect("coordination event serializes"),
            );
            output.push('\n');
        }
        output
    }

    pub fn markdown_report(&self) -> String {
        let snapshot = self.readiness_snapshot();
        let ledger_entry = &snapshot.ledger_entries[0];
        let mut output = String::new();

        output.push_str("# Helm Coordination Headless Scenario\n\n");
        output.push_str(&format!("session: `{}`\n\n", self.session_id));
        output.push_str("## Participants\n\n");
        output.push_str(&format!("- humans: {HUMAN_COUNT}\n"));
        output.push_str(&format!("- agents: {AGENT_COUNT}\n"));
        output.push_str(&format!(
            "- contributions: {}\n\n",
            self.contributions.len()
        ));

        output.push_str("## Checkpoint\n\n");
        output.push_str("- mode: convened-burst\n");
        output.push_str("- mixer: role-balanced-topic-windows\n");
        output.push_str("- matcher: agent-to-branch-assignment\n");
        output.push_str("- formation: bounded-synthesis\n");
        output.push_str(&format!(
            "- gate: {} decision receipt\n",
            self.gate_outcome.as_str()
        ));
        if self.gate_outcome.allows_converge_admission() {
            output.push_str(
                "- Converge handoff: proposals prepared, facts admitted through Converge\n\n",
            );
        } else {
            output.push_str("- Converge handoff: blocked before admission, no facts admitted\n\n");
        }

        output.push_str("## Helm Readiness\n\n");
        output.push_str(&format!(
            "- shell module state: {:?}\n",
            Self::shell_module_state()
        ));
        output.push_str(&format!(
            "- live module state with feed: {:?}\n",
            self.live_module_state()
        ));
        output.push_str(&format!(
            "- receipt: `{}`\n",
            snapshot.packet.adapter_receipt_id
        ));
        output.push_str(&format!(
            "- payload hash: `{}`\n",
            ledger_entry.payload_hash
        ));
        output.push_str(&format!(
            "- record kind: `{:?}`\n",
            ledger_entry.record_kind
        ));
        output.push_str(&format!(
            "- receipt family: `{:?}`\n\n",
            ledger_entry.receipt_family
        ));

        output.push_str("## Prepared Proposals\n\n");
        if self.prepared_proposals.is_empty() {
            output.push_str("- none\n");
        } else {
            for proposal in &self.prepared_proposals {
                output.push_str(&format!("- `{proposal}`\n"));
            }
        }
        output.push('\n');

        output.push_str("## Admitted Facts\n\n");
        if self.admitted_fact_ids.is_empty() {
            output.push_str("- none\n");
        } else {
            for fact_id in &self.admitted_fact_ids {
                output.push_str(&format!("- `{fact_id}`\n"));
            }
        }
        output.push('\n');

        output.push_str("## Timeline\n\n");
        for event in &self.events {
            output.push_str(&format!("{}. {:?}\n", event.sequence, event.kind));
        }

        output
    }

    fn emit(&mut self, kind: CoordinationEventKind, payload: Value) {
        self.events.push(CoordinationEvent {
            sequence: self.events.len() as u64 + 1,
            kind,
            payload,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsensusOpinion {
    LaunchNow,
    PilotFirst,
    WaitForEvidence,
    SegmentByNeed,
}

impl ConsensusOpinion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LaunchNow => "launch-now",
            Self::PilotFirst => "pilot-first",
            Self::WaitForEvidence => "wait-for-evidence",
            Self::SegmentByNeed => "segment-by-need",
        }
    }

    const fn conclusion(self) -> &'static str {
        match self {
            Self::LaunchNow => "move forward with the broad launch plan",
            Self::PilotFirst => "run a bounded pilot before broad launch",
            Self::WaitForEvidence => "pause for stronger evidence before committing",
            Self::SegmentByNeed => "split the decision by user segment",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicUser {
    pub id: String,
    pub display_name: String,
    pub cohort: String,
    pub opinion: ConsensusOpinion,
    pub contribution_speed_ms: u64,
    pub confidence_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusSummary {
    pub winning_opinion: ConsensusOpinion,
    pub support_count: usize,
    pub support_bps: u16,
    pub dissent_count: usize,
    pub conclusion: String,
    pub decision: String,
}

#[derive(Debug, Clone)]
pub struct CrowdConsensusRun {
    pub session_id: String,
    pub seed: u64,
    pub users: Vec<DynamicUser>,
    pub events: Vec<CoordinationEvent>,
    pub summary: ConsensusSummary,
}

impl CrowdConsensusRun {
    pub fn scripted(user_count: usize, seed: u64) -> Result<Self, String> {
        validate_crowd_user_count(user_count)?;

        let users: Vec<_> = dynamic_user_script(seed)
            .into_iter()
            .take(user_count)
            .collect();
        let summary = consensus_summary(&users);
        let mut run = Self {
            session_id: format!("coordination://crowd-consensus/{user_count}/{seed}"),
            seed,
            users,
            events: Vec::new(),
            summary,
        };

        run.emit(
            CoordinationEventKind::SessionOpened,
            json!({
                "session_id": run.session_id.clone(),
                "mode": "crowd-consensus",
                "seed": seed,
                "user_count": user_count,
                "script_limit": DYNAMIC_USER_LIMIT,
            }),
        );

        for (arrival_rank, user) in run.arrivals_by_speed().into_iter().enumerate() {
            run.emit(
                CoordinationEventKind::OpinionGathered,
                json!({
                    "arrival_rank": arrival_rank + 1,
                    "user_id": user.id,
                    "display_name": user.display_name,
                    "cohort": user.cohort,
                    "opinion": user.opinion,
                    "contribution_speed_ms": user.contribution_speed_ms,
                    "confidence_bps": user.confidence_bps,
                    "parallel_lane": user.contribution_speed_ms / 1_000,
                }),
            );
        }

        run.emit(
            CoordinationEventKind::Mixed,
            json!({
                "strategy": "opinion-cluster-window",
                "opinion_counts": opinion_counts_json(&run.users),
                "speed_bands": speed_bands_json(&run.users),
            }),
        );
        run.emit(
            CoordinationEventKind::ConsensusMeasured,
            json!({
                "winning_opinion": run.summary.winning_opinion,
                "support_count": run.summary.support_count,
                "support_bps": run.summary.support_bps,
                "dissent_count": run.summary.dissent_count,
            }),
        );
        run.emit(
            CoordinationEventKind::ConclusionDrafted,
            json!({
                "conclusion": run.summary.conclusion,
                "consensus_bps": run.summary.support_bps,
            }),
        );
        run.emit(
            CoordinationEventKind::DecisionRecorded,
            json!({
                "decision": run.summary.decision,
                "decision_receipt": run.decision_receipt_id(),
                "authorizes_domain_action": false,
            }),
        );

        Ok(run)
    }

    pub fn event_trace(&self) -> Vec<(u64, CoordinationEventKind, Value)> {
        self.events
            .iter()
            .map(|event| (event.sequence, event.kind, event.payload.clone()))
            .collect()
    }

    pub fn jsonl_timeline(&self) -> String {
        let mut output = String::new();
        for event in &self.events {
            output.push_str(
                &serde_json::to_string(&json!({
                    "sequence": event.sequence,
                    "kind": event.kind,
                    "payload": event.payload,
                }))
                .expect("crowd event serializes"),
            );
            output.push('\n');
        }
        output
    }

    pub fn markdown_report(&self) -> String {
        let mut output = String::new();
        output.push_str("# Crowd Consensus Headless Scenario\n\n");
        output.push_str(&format!("session: `{}`\n", self.session_id));
        output.push_str(&format!("seed: `{}`\n\n", self.seed));

        output.push_str("## Scale\n\n");
        output.push_str(&format!("- users: {}\n", self.users.len()));
        output.push_str(&format!(
            "- fastest contribution: {} ms\n",
            self.fastest_speed_ms()
        ));
        output.push_str(&format!(
            "- slowest contribution: {} ms\n\n",
            self.slowest_speed_ms()
        ));

        output.push_str("## Opinions\n\n");
        for (opinion, count) in opinion_counts(&self.users) {
            output.push_str(&format!("- {}: {count}\n", opinion.as_str()));
        }
        output.push('\n');

        output.push_str("## Consensus\n\n");
        output.push_str(&format!(
            "- winning opinion: {}\n",
            self.summary.winning_opinion.as_str()
        ));
        output.push_str(&format!(
            "- support: {} users ({} bps)\n",
            self.summary.support_count, self.summary.support_bps
        ));
        output.push_str(&format!(
            "- dissent: {} users\n",
            self.summary.dissent_count
        ));
        output.push_str(&format!("- conclusion: {}\n", self.summary.conclusion));
        output.push_str(&format!("- decision: {}\n\n", self.summary.decision));

        output.push_str("## Timeline\n\n");
        for event in &self.events {
            output.push_str(&format!("{}. {:?}\n", event.sequence, event.kind));
        }

        output
    }

    pub fn fastest_speed_ms(&self) -> u64 {
        self.users
            .iter()
            .map(|user| user.contribution_speed_ms)
            .min()
            .unwrap_or(0)
    }

    pub fn slowest_speed_ms(&self) -> u64 {
        self.users
            .iter()
            .map(|user| user.contribution_speed_ms)
            .max()
            .unwrap_or(0)
    }

    pub fn decision_receipt_id(&self) -> String {
        format!(
            "decision:{}:{}:{}",
            self.session_id,
            self.users.len(),
            self.summary.support_bps
        )
    }

    fn arrivals_by_speed(&self) -> Vec<DynamicUser> {
        let mut arrivals = self.users.clone();
        arrivals.sort_by(|left, right| {
            left.contribution_speed_ms
                .cmp(&right.contribution_speed_ms)
                .then_with(|| left.id.cmp(&right.id))
        });
        arrivals
    }

    fn emit(&mut self, kind: CoordinationEventKind, payload: Value) {
        self.events.push(CoordinationEvent {
            sequence: self.events.len() as u64 + 1,
            kind,
            payload,
        });
    }
}

#[derive(Clone)]
pub struct StaticCoordinationFeed {
    pub snapshots: Vec<LiveOperatorControlSnapshot>,
}

impl StaticCoordinationFeed {
    pub fn from_run(run: &HeadlessCoordinationRun) -> Self {
        Self {
            snapshots: vec![run.readiness_snapshot()],
        }
    }
}

impl OperatorControlReadinessFeed for StaticCoordinationFeed {
    fn live_evidence(&self) -> LiveReadinessEvidence {
        LiveReadinessEvidence::complete()
    }

    fn previews(&self) -> Result<Vec<LiveOperatorControlSnapshot>, OperatorControlError> {
        Ok(self.snapshots.clone())
    }
}

pub fn readiness_payload_hash(snapshot: &LiveOperatorControlSnapshot) -> String {
    job_readiness_packet_payload_hash(&snapshot.packet)
}

pub fn readiness_record_kind(snapshot: &LiveOperatorControlSnapshot) -> OperatorLedgerRecordKind {
    snapshot.ledger_entries[0].record_kind
}

pub fn readiness_receipt_family(snapshot: &LiveOperatorControlSnapshot) -> ReceiptFamily {
    snapshot.ledger_entries[0].receipt_family
}

pub fn dynamic_user_script(seed: u64) -> Vec<DynamicUser> {
    let first_names = [
        "Ada", "Ben", "Cyra", "Dev", "Eli", "Faye", "Gia", "Hugo", "Iris", "Jules", "Kai", "Lina",
        "Mika", "Nia", "Omar", "Pia", "Quin", "Rae", "Sol", "Tess",
    ];
    let last_names = [
        "Arden", "Berg", "Cruz", "Dahl", "Ekwall", "Frost", "Gray", "Hart", "Ivers", "Jain",
        "Kova", "Lund", "Moss", "Nordin", "Olsen", "Park", "Quill", "Reed", "Stone", "Vale",
    ];
    let cohorts = [
        "operators",
        "customers",
        "finance",
        "risk",
        "field",
        "support",
        "product",
        "sales",
    ];
    let opinions = [
        ConsensusOpinion::LaunchNow,
        ConsensusOpinion::PilotFirst,
        ConsensusOpinion::WaitForEvidence,
        ConsensusOpinion::SegmentByNeed,
    ];
    let favored = (mix64(seed, 0, 17) as usize) % opinions.len();

    (0..DYNAMIC_USER_LIMIT)
        .map(|index| {
            let jitter = mix64(seed, index, 29);
            let roll = (jitter % 100) as usize;
            let opinion_index = if index % 17 == 0 {
                (favored + 2) % opinions.len()
            } else if index % 11 == 0 {
                (favored + 1) % opinions.len()
            } else if roll < 48 {
                favored
            } else if roll < 72 {
                (favored + 1) % opinions.len()
            } else if roll < 90 {
                (favored + 2) % opinions.len()
            } else {
                (favored + 3) % opinions.len()
            };
            let first =
                first_names[(index + (jitter as usize % first_names.len())) % first_names.len()];
            let last =
                last_names[(index * 3 + (jitter as usize % last_names.len())) % last_names.len()];
            let cohort = cohorts[(index + (mix64(seed, index, 41) as usize)) % cohorts.len()];

            DynamicUser {
                id: format!("dynamic-user-{index:03}"),
                display_name: format!("{first} {last}"),
                cohort: cohort.to_string(),
                opinion: opinions[opinion_index],
                contribution_speed_ms: 150 + (mix64(seed, index, 53) % 7_500),
                confidence_bps: 4_500 + (mix64(seed, index, 67) % 5_000) as u16,
            }
        })
        .collect()
}

fn validate_crowd_user_count(user_count: usize) -> Result<(), String> {
    if CROWD_USER_COUNTS.contains(&user_count) {
        Ok(())
    } else {
        Err(format!(
            "crowd consensus supports user counts {:?}; got {user_count}",
            CROWD_USER_COUNTS
        ))
    }
}

fn consensus_summary(users: &[DynamicUser]) -> ConsensusSummary {
    let counts = opinion_counts(users);
    let (winning_opinion, support_count) = counts
        .iter()
        .copied()
        .max_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| {
                    opinion_confidence(users, left.0).cmp(&opinion_confidence(users, right.0))
                })
                .then_with(|| right.0.as_str().cmp(left.0.as_str()))
        })
        .expect("validated crowd run has users");
    let support_bps = ((support_count * 10_000) / users.len()) as u16;
    let dissent_count = users.len() - support_count;
    let conclusion = format!(
        "{} with {} of {} users in support",
        winning_opinion.conclusion(),
        support_count,
        users.len()
    );
    let decision = if support_bps >= 6_000 {
        format!("accept consensus to {}", winning_opinion.conclusion())
    } else if support_bps >= 4_500 {
        format!(
            "continue one more round around {}",
            winning_opinion.conclusion()
        )
    } else {
        "defer decision and gather more evidence".to_string()
    };

    ConsensusSummary {
        winning_opinion,
        support_count,
        support_bps,
        dissent_count,
        conclusion,
        decision,
    }
}

fn opinion_counts(users: &[DynamicUser]) -> [(ConsensusOpinion, usize); 4] {
    let mut counts = [
        (ConsensusOpinion::LaunchNow, 0),
        (ConsensusOpinion::PilotFirst, 0),
        (ConsensusOpinion::WaitForEvidence, 0),
        (ConsensusOpinion::SegmentByNeed, 0),
    ];
    for user in users {
        for (opinion, count) in &mut counts {
            if *opinion == user.opinion {
                *count += 1;
            }
        }
    }
    counts
}

fn opinion_counts_json(users: &[DynamicUser]) -> Value {
    let mut counts = serde_json::Map::new();
    for (opinion, count) in opinion_counts(users) {
        counts.insert(opinion.as_str().to_string(), json!(count));
    }
    Value::Object(counts)
}

fn speed_bands_json(users: &[DynamicUser]) -> Value {
    let mut fast = 0;
    let mut medium = 0;
    let mut slow = 0;
    for user in users {
        match user.contribution_speed_ms {
            0..=1_999 => fast += 1,
            2_000..=4_999 => medium += 1,
            _ => slow += 1,
        }
    }
    json!({
        "fast": fast,
        "medium": medium,
        "slow": slow,
    })
}

fn opinion_confidence(users: &[DynamicUser], opinion: ConsensusOpinion) -> u32 {
    users
        .iter()
        .filter(|user| user.opinion == opinion)
        .map(|user| u32::from(user.confidence_bps))
        .sum()
}

fn mix64(seed: u64, index: usize, salt: u64) -> u64 {
    let mut value = seed ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value ^= (index as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[derive(Debug, Clone)]
struct ConcurrentArrival {
    batch: usize,
    ordering_key: String,
    participant: ParticipantRef,
}

fn deterministic_concurrent_arrivals(participants: &[ParticipantRef]) -> Vec<ConcurrentArrival> {
    let mut arrivals: Vec<_> = participants
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, participant)| ConcurrentArrival {
            batch: (index * 7) % 5,
            ordering_key: format!("{:03}:{}", index % 11, participant.id),
            participant,
        })
        .collect();
    arrivals.sort_by(|left, right| {
        left.ordering_key
            .cmp(&right.ordering_key)
            .then_with(|| left.participant.id.cmp(&right.participant.id))
    });
    arrivals
}

fn participants() -> Vec<ParticipantRef> {
    let human_roles = ["strategy", "operations", "risk", "frontline", "customer"];
    let agent_roles = [
        "synthesizer",
        "skeptic",
        "coverage-checker",
        "probe-matcher",
        "receipt-writer",
        "formation-router",
    ];

    let mut participants = Vec::with_capacity(HUMAN_COUNT + AGENT_COUNT);
    for index in 0..HUMAN_COUNT {
        participants.push(ParticipantRef {
            id: format!("human-{index:03}"),
            kind: ParticipantKind::Human,
            role: human_roles[index % human_roles.len()].to_string(),
        });
    }
    for index in 0..AGENT_COUNT {
        participants.push(ParticipantRef {
            id: format!("agent-{index:03}"),
            kind: ParticipantKind::Agent,
            role: agent_roles[index % agent_roles.len()].to_string(),
        });
    }
    participants
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripted_burst_is_deterministic() {
        let first = HeadlessCoordinationRun::scripted_burst();
        let second = HeadlessCoordinationRun::scripted_burst();

        assert_eq!(first.event_trace(), second.event_trace());
        assert_eq!(first.contributions, second.contributions);
        assert_eq!(first.admitted_fact_ids, second.admitted_fact_ids);
        assert_eq!(first.participants.len(), HUMAN_COUNT + AGENT_COUNT);
        assert_eq!(first.contributions.len(), HUMAN_COUNT + AGENT_COUNT);
        assert_eq!(first.gate_outcome, GateOutcome::Approved);
        assert!(!first.prepared_proposals.is_empty());
        assert!(!first.admitted_fact_ids.is_empty());
    }

    #[test]
    fn narrated_outputs_include_receipts_and_converge_admission() {
        let run = HeadlessCoordinationRun::scripted_burst();
        let markdown = run.markdown_report();
        let jsonl = run.jsonl_timeline();
        let timeline_receipt = run
            .events
            .last()
            .and_then(|event| event.payload.get("receipt_id"))
            .and_then(Value::as_str)
            .expect("readiness event records a receipt id");

        assert!(
            markdown
                .contains("Converge handoff: proposals prepared, facts admitted through Converge")
        );
        assert!(markdown.contains("## Admitted Facts"));
        assert!(markdown.contains("receipt:"));
        assert_eq!(
            timeline_receipt,
            run.readiness_snapshot().packet.adapter_receipt_id
        );
        assert!(jsonl.contains("\"kind\":\"gate-decision\""));
        assert!(jsonl.contains("\"kind\":\"converge-admission-recorded\""));
        assert!(jsonl.contains("\"promoted_facts\":[]"));
        assert!(jsonl.contains("\"admitted_fact_ids\""));
    }

    #[test]
    fn blocked_gate_variants_do_not_prepare_or_admit_facts() {
        for gate_outcome in [GateOutcome::Rejected, GateOutcome::TimedOut] {
            let run = HeadlessCoordinationRun::scripted_burst_with_gate(gate_outcome);
            let snapshot = run.readiness_snapshot();

            assert_eq!(run.gate_outcome, gate_outcome);
            assert!(run.prepared_proposals.is_empty());
            assert!(run.admitted_fact_ids.is_empty());
            assert!(
                !run.events
                    .iter()
                    .any(|event| event.kind == CoordinationEventKind::ConvergeAdmissionRecorded)
            );
            assert_eq!(
                snapshot.packet.adapter_status,
                AdapterReceiptStatus::Rejected
            );
            assert_eq!(snapshot.packet.verdict, Some(gate_outcome.job_verdict()));
            assert!(
                run.markdown_report()
                    .contains("Converge handoff: blocked before admission, no facts admitted")
            );
        }
    }

    #[test]
    fn crowd_consensus_script_scales_and_replays_by_seed() {
        for user_count in CROWD_USER_COUNTS {
            let first = CrowdConsensusRun::scripted(user_count, 42).expect("supported user count");
            let second = CrowdConsensusRun::scripted(user_count, 42).expect("supported user count");

            assert_eq!(first.users, second.users);
            assert_eq!(first.summary, second.summary);
            assert_eq!(first.event_trace(), second.event_trace());
            assert_eq!(first.users.len(), user_count);
            assert_eq!(
                first
                    .events
                    .iter()
                    .filter(|event| event.kind == CoordinationEventKind::OpinionGathered)
                    .count(),
                user_count
            );
            assert!(first.fastest_speed_ms() < first.slowest_speed_ms());
            assert!(first.summary.support_count > 0);
            assert!(!first.summary.conclusion.is_empty());
            assert!(!first.summary.decision.is_empty());
        }
    }

    #[test]
    fn crowd_consensus_dynamic_script_varies_by_seed() {
        let first = CrowdConsensusRun::scripted(30, 42).expect("supported user count");
        let second = CrowdConsensusRun::scripted(30, 43).expect("supported user count");
        let users = dynamic_user_script(42);

        assert_eq!(users.len(), DYNAMIC_USER_LIMIT);
        assert_ne!(first.users, second.users);
        assert_ne!(first.event_trace(), second.event_trace());
        assert!(
            users
                .windows(2)
                .any(|window| window[0].contribution_speed_ms != window[1].contribution_speed_ms)
        );
    }

    #[test]
    fn crowd_consensus_rejects_unsupported_user_counts() {
        let error = CrowdConsensusRun::scripted(31, 42)
            .expect_err("only 5, 30, and 100 user runs are supported");

        assert!(error.contains("supports user counts"));
    }
}
