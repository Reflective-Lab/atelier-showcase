// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

use serde_json::json;

use crate::session::{ConvergenceEventKind, MultiUserConvergenceSession, SessionPhase};

// Used for ActorKind matching in participant list rendering.
use application_kernel;

pub fn markdown_report(session: &MultiUserConvergenceSession) -> String {
    let mut out = String::new();

    out.push_str("# Helm Multi-User Convergence Headless Scenario\n\n");
    out.push_str(&format!("session: `{}`\n", session.session_id));
    out.push_str(&format!("workspace: `{}`\n", session.workspace_id));
    out.push_str(&format!(
        "simulated duration: {:.1} min\n\n",
        session.simulated_ms as f64 / 60_000.0
    ));

    out.push_str("## Participants\n\n");
    for p in &session.participants {
        let kind = match p.role.actor_kind() {
            application_kernel::ActorKind::Human => "human",
            _ => "agent",
        };
        out.push_str(&format!(
            "- `{}` — {} ({})\n",
            p.role.actor_id(),
            p.role.display_name(),
            kind,
        ));
    }
    out.push('\n');

    out.push_str("## Session Arc\n\n");
    let phases = [
        SessionPhase::Orientation,
        SessionPhase::Exploration,
        SessionPhase::Convergence,
        SessionPhase::Gate,
        SessionPhase::Integration,
        SessionPhase::Closeout,
    ];
    for phase in phases {
        let phase_events: Vec<_> = session.events.iter().filter(|e| e.phase == phase).collect();
        out.push_str(&format!(
            "### Phase: {} ({} events)\n\n",
            phase.as_str(),
            phase_events.len()
        ));
        for event in &phase_events {
            out.push_str(&format!(
                "- t={:.1}min [{:?}] {} — {}\n",
                event.simulated_ms as f64 / 60_000.0,
                event.kind,
                event.actor,
                event_summary(&event.kind, &event.payload),
            ));
        }
        out.push('\n');
    }

    out.push_str("## Coordination State\n\n");
    out.push_str(&format!(
        "- formations spawned: {}\n",
        session.total_formations_spawned()
    ));
    out.push_str(&format!(
        "- active sessions (post-closeout): {}\n",
        session.active_session_count()
    ));
    out.push_str(&format!(
        "- presence entries remaining: {}\n",
        session.presence_count()
    ));

    let gate_ref = "gate:convergence:session:main";
    if let Some(record) = session.gate_decision(gate_ref) {
        out.push_str(&format!(
            "- gate decision: {:?} by `{}`\n",
            record.decision, record.principal.actor_id
        ));
    }
    out.push('\n');

    out.push_str("## Why Generic Substitutes Fail\n\n");
    out.push_str(
        "A chat room lets anyone type anything. Helm coordination enforces a typed protocol:\n\n",
    );
    out.push_str(
        "- **SessionRegistry** scopes presence to a workspace. No global mutable state shared across workspaces.\n"
    );
    out.push_str(
        "- **PresenceRegistry** allows two operators to soft-claim the same subject simultaneously — optimistic, never a lock. A chat room either blocks or ignores the second actor.\n"
    );
    out.push_str(
        "- **DecisionLedger** makes the first gate decision authoritative, idempotent on a repeat (same outcome), and a typed Conflict on divergence. A chat room cannot express \"two people said opposite things; the first one won.\"\n"
    );
    out.push_str(
        "- **ClientHelm** routes each push by urgency: Informational → NoAction, Advisory → SpawnFormation, Disruptive → parallel, Preemptive → pause-and-inject. A chat room delivers everything as equivalent messages.\n"
    );

    out
}

pub fn jsonl_timeline(session: &MultiUserConvergenceSession) -> String {
    let mut out = String::new();
    for event in &session.events {
        out.push_str(
            &serde_json::to_string(&json!({
                "sequence": event.sequence,
                "simulated_ms": event.simulated_ms,
                "phase": event.phase.as_str(),
                "kind": event.kind,
                "actor": event.actor,
                "payload": event.payload,
            }))
            .expect("convergence event serializes"),
        );
        out.push('\n');
    }
    out
}

fn event_summary(kind: &ConvergenceEventKind, payload: &serde_json::Value) -> String {
    match kind {
        ConvergenceEventKind::PushDispatched => {
            let suggestor = payload
                .get("suggestor")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let urgency = payload
                .get("urgency")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!("{suggestor} ({urgency})")
        }
        ConvergenceEventKind::FormationSpawned => {
            let action = payload
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            payload
                .get("description")
                .and_then(|v| v.as_str())
                .map(|d| format!("{action}: {d}"))
                .unwrap_or_else(|| action.to_string())
        }
        ConvergenceEventKind::GateDecision => {
            let decision = payload
                .get("decision")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!("decision={decision}")
        }
        ConvergenceEventKind::GateConflict => {
            let outcome = payload
                .get("outcome")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            outcome.to_string()
        }
        ConvergenceEventKind::GateIdempotent => "idempotent (same as first)".to_string(),
        ConvergenceEventKind::PresenceClaimed => {
            let subject = payload
                .get("subject")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!("claimed {subject}")
        }
        ConvergenceEventKind::PresenceFocused => {
            let subject = payload
                .get("subject")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!("focused {subject}")
        }
        ConvergenceEventKind::PresenceReleased => {
            let subject = payload
                .get("subject")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!("released {subject}")
        }
        ConvergenceEventKind::SessionOpened => payload
            .get("role")
            .map(|v| v.to_string())
            .unwrap_or_default(),
        ConvergenceEventKind::SessionClosed => payload
            .get("session_id")
            .map(|v| v.to_string())
            .unwrap_or_default(),
        ConvergenceEventKind::TemperatureRecorded => {
            let position = payload
                .get("position")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let conviction = payload
                .get("conviction")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            format!("{position}/{conviction}")
        }
        ConvergenceEventKind::PhaseEntered => payload
            .get("phase")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string(),
        _ => payload.to_string(),
    }
}
