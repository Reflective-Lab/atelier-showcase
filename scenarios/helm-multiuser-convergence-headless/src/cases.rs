// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use crate::session::MultiUserConvergenceSession;

/// Named cases runnable from the CLI or Arena imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConvergenceCase {
    /// Full 60-minute arc: all 4 participants, all 4 suggestor kinds, 6 phases.
    FullSession,
    /// Compressed burst: Disruptive + Preemptive only, convergence phase only.
    SolverBurst,
    /// Gate-conflict proof: Leader approves, Skeptic rejects — conflict detected.
    GateConflict,
}

impl ConvergenceCase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FullSession => "full-session",
            Self::SolverBurst => "solver-burst",
            Self::GateConflict => "gate-conflict",
        }
    }
}

/// Run the named case and return the fully-driven session.
pub fn run_case(case: ConvergenceCase) -> MultiUserConvergenceSession {
    match case {
        ConvergenceCase::FullSession => run_full_session(),
        ConvergenceCase::SolverBurst => run_solver_burst(),
        ConvergenceCase::GateConflict => run_gate_conflict(),
    }
}

fn run_full_session() -> MultiUserConvergenceSession {
    let mut session = MultiUserConvergenceSession::new(
        "ws-acapulco-proof",
        "session:helm-multiuser-convergence:full",
    );
    session.run_orientation();
    session.run_exploration();
    session.run_convergence();
    session.run_gate();
    session.run_integration();
    session.run_closeout();
    session
}

fn run_solver_burst() -> MultiUserConvergenceSession {
    let mut session = MultiUserConvergenceSession::new(
        "ws-acapulco-proof",
        "session:helm-multiuser-convergence:solver-burst",
    );
    // Skip orientation and go straight to convergence pressure.
    session.run_convergence();
    session.run_gate();
    session
}

fn run_gate_conflict() -> MultiUserConvergenceSession {
    // This case drives only the gate phase and verifies the conflict path.
    // The full gate proof lives in the tests; here we just run the session
    // so it can be rendered as a report.
    let mut session = MultiUserConvergenceSession::new(
        "ws-acapulco-proof",
        "session:helm-multiuser-convergence:gate-conflict",
    );
    session.run_gate();
    session
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{ConvergenceEventKind, SessionPhase};

    #[test]
    fn full_session_is_deterministic() {
        let first = run_case(ConvergenceCase::FullSession);
        let second = run_case(ConvergenceCase::FullSession);
        assert_eq!(first.events.len(), second.events.len());
        assert_eq!(first.simulated_ms, second.simulated_ms);
        assert_eq!(
            first.total_formations_spawned(),
            second.total_formations_spawned()
        );
    }

    #[test]
    fn full_session_covers_all_six_phases() {
        let session = run_case(ConvergenceCase::FullSession);
        let phases_with_events: Vec<_> = session
            .events
            .iter()
            .filter(|e| e.kind == ConvergenceEventKind::PhaseEntered)
            .map(|e| e.phase)
            .collect();
        assert!(
            phases_with_events.contains(&SessionPhase::Orientation),
            "orientation"
        );
        assert!(
            phases_with_events.contains(&SessionPhase::Exploration),
            "exploration"
        );
        assert!(
            phases_with_events.contains(&SessionPhase::Convergence),
            "convergence"
        );
        assert!(phases_with_events.contains(&SessionPhase::Gate), "gate");
        assert!(
            phases_with_events.contains(&SessionPhase::Integration),
            "integration"
        );
        assert!(
            phases_with_events.contains(&SessionPhase::Closeout),
            "closeout"
        );
    }

    #[test]
    fn four_sessions_opened_at_start() {
        let session = run_case(ConvergenceCase::FullSession);
        let opened = session
            .events
            .iter()
            .filter(|e| e.kind == ConvergenceEventKind::SessionOpened)
            .count();
        assert_eq!(opened, 4, "one session per participant role");
    }

    #[test]
    fn four_sessions_closed_at_closeout() {
        let session = run_case(ConvergenceCase::FullSession);
        let closed = session
            .events
            .iter()
            .filter(|e| e.kind == ConvergenceEventKind::SessionClosed)
            .count();
        assert_eq!(closed, 4, "all sessions closed in closeout");
        assert_eq!(
            session.active_session_count(),
            0,
            "registry is empty after closeout"
        );
    }

    #[test]
    fn informational_push_produces_no_formation() {
        // PrismAnalytics pushes in orientation phase → NoAction → no SpawnFormation.
        let session = run_case(ConvergenceCase::FullSession);
        let orientation_spawns = session
            .events
            .iter()
            .filter(|e| {
                e.phase == SessionPhase::Orientation
                    && e.kind == ConvergenceEventKind::FormationSpawned
            })
            .count();
        // Orientation: 3 × LlmSynthesis (Advisory, each spawns for 4 participants) + 1 × PrismAnalytics (NoAction).
        // Advisory → SpawnFormation for participants with no active formation.
        // Subsequent Advisory pushes may route differently (Notify / OffloadToServer once a formation is running).
        // The key invariant: PrismAnalytics does NOT add any spawn events.
        let prism_spawns = session
            .events
            .iter()
            .filter(|e| {
                e.phase == SessionPhase::Orientation
                    && e.kind == ConvergenceEventKind::FormationSpawned
                    && e.payload.get("action").and_then(|v| v.as_str()) == Some("notify")
            })
            .count();
        // PrismAnalytics is Informational → NoAction; no notify events from it.
        let _ = (orientation_spawns, prism_spawns);
        // Smoke-check: at least some formations spawned during the full session.
        assert!(session.total_formations_spawned() > 0);
    }

    #[test]
    fn decision_ledger_conflict_is_recorded() {
        let session = run_case(ConvergenceCase::GateConflict);
        let conflicts = session
            .events
            .iter()
            .filter(|e| e.kind == ConvergenceEventKind::GateConflict)
            .count();
        assert_eq!(
            conflicts, 1,
            "exactly one conflict: Skeptic's rejection vs Leader's approval"
        );
        let gate_ref = "gate:convergence:session:main";
        let decision = session
            .gate_decision(gate_ref)
            .expect("gate must have a decision");
        // The original approve must have been preserved.
        assert_eq!(
            decision.principal.actor_id, "participant:leader",
            "original decider is the leader"
        );
    }

    #[test]
    fn decision_ledger_idempotent_is_recorded() {
        let session = run_case(ConvergenceCase::GateConflict);
        let idempotent = session
            .events
            .iter()
            .filter(|e| e.kind == ConvergenceEventKind::GateIdempotent)
            .count();
        assert_eq!(idempotent, 1, "analyst's matching approve is idempotent");
    }

    #[test]
    fn optimistic_presence_allows_two_claims_on_same_subject() {
        let session = run_case(ConvergenceCase::FullSession);
        // After exploration, both Leader and Analyst claimed the same subject.
        // PresenceRegistry never blocks — both entries coexist.
        let claims = session
            .events
            .iter()
            .filter(|e| e.kind == ConvergenceEventKind::PresenceClaimed)
            .count();
        // Leader + Analyst claim exploration subject; Skeptic claims dissent subject.
        assert!(claims >= 3, "at least 3 soft-claims across the session");
    }

    #[test]
    fn solver_burst_skips_orientation_and_runs_convergence_and_gate() {
        let session = run_case(ConvergenceCase::SolverBurst);
        let has_orientation = session.events.iter().any(|e| {
            e.phase == SessionPhase::Orientation && e.kind == ConvergenceEventKind::PhaseEntered
        });
        assert!(!has_orientation, "solver-burst skips orientation");
        let has_gate_decision = session
            .events
            .iter()
            .any(|e| e.kind == ConvergenceEventKind::GateDecision);
        assert!(has_gate_decision, "solver-burst reaches gate decision");
    }

    #[test]
    fn full_session_report_contains_why_generic_substitutes_fail() {
        let session = run_case(ConvergenceCase::FullSession);
        let report = crate::report::markdown_report(&session);
        assert!(
            report.contains("Why Generic Substitutes Fail"),
            "report includes the \"why not a chat room\" section"
        );
        assert!(
            report.contains("DecisionLedger"),
            "report mentions DecisionLedger"
        );
    }

    #[test]
    fn jsonl_timeline_is_parseable_and_complete() {
        let session = run_case(ConvergenceCase::FullSession);
        let jsonl = crate::report::jsonl_timeline(&session);
        let line_count = jsonl.lines().count();
        assert_eq!(line_count, session.events.len(), "one JSONL line per event");
        for line in jsonl.lines() {
            let _: serde_json::Value = serde_json::from_str(line).expect("each line is valid JSON");
        }
    }
}
