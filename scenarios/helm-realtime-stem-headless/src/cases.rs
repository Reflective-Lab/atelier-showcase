// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Named interactive cases that stress the Session Intelligence Spine headlessly.

use crate::{InteractiveCaseId, RealtimeStemRun, ServerLoopProfile, push_script};
use helm_session_contracts::urgency::UrgencyIntent;

/// Run one named interactive case and return the deterministic artifact.
pub fn run_case(case: InteractiveCaseId) -> RealtimeStemRun {
    match case {
        InteractiveCaseId::SingleUserShortLoop => single_user_short_loop(),
        InteractiveCaseId::ServerOffloadUnderLoad => server_offload_under_load(),
        InteractiveCaseId::PreemptivePivot => preemptive_pivot(),
        InteractiveCaseId::ThreeUserConcurrentRoom => three_user_concurrent_room(),
        InteractiveCaseId::LongAndShortServerMix => long_and_short_server_mix(),
        InteractiveCaseId::GateWhileLoopsActive => gate_while_loops_active(),
        InteractiveCaseId::BudgetExhaustion => budget_exhaustion(),
        InteractiveCaseId::MarqueeBurstRoom => marquee_burst_room(),
    }
}

/// Baseline: one participant receives an informational push and completes a short local loop.
fn single_user_short_loop() -> RealtimeStemRun {
    let mut run = RealtimeStemRun::new("case://single-user-short-loop", 1_000);
    run.open_session("burst-001", "convened-burst");
    run.join_participant("alice", "Alice Chen", "strategy");

    let push = push_script(
        "burst-001",
        1,
        1_100,
        UrgencyIntent::Informational,
        "Short local synthesis on launch timing",
        serde_json::json!({"objective": "Short local synthesis on launch timing"}),
    );
    run.deliver_push("alice", push);
    run.ack_local_spawn("alice");
    run.complete_local_loop(
        "alice",
        serde_json::json!({"conclusion": "pilot-first", "support_bps": 6200}),
        Some(("agree", "medium", "quorum://hypothesis/launch-timing")),
    );
    run.tick(1_500);
    run
}

/// Local loop running; a disruptive push offloads heavy work to a long-running server formation.
fn server_offload_under_load() -> RealtimeStemRun {
    let mut run = RealtimeStemRun::new("case://server-offload-under-load", 2_000);
    run.open_session("burst-002", "convened-burst");
    run.join_participant("bob", "Bob Ortiz", "operations");

    let first = push_script(
        "burst-002",
        1,
        2_100,
        UrgencyIntent::Informational,
        "Local branch review while quorum gathers",
        serde_json::json!({"objective": "Local branch review while quorum gathers"}),
    );
    run.deliver_push("bob", first);
    run.ack_local_spawn("bob");

    let disruptive = push_script(
        "burst-002",
        2,
        2_250,
        UrgencyIntent::Disruptive,
        "Server DD sweep on counterparty exposure",
        serde_json::json!({
            "objective": "Server DD sweep on counterparty exposure",
            "server_profile": "long-dd",
        }),
    );
    run.deliver_push("bob", disruptive);
    run.ack_server_offload(
        "bob",
        "srv-dd-001",
        "dd-analysis",
        ServerLoopProfile::LongDd,
    );

    run.advance_server_loops(2_300);
    run.complete_local_loop(
        "bob",
        serde_json::json!({"conclusion": "hold-pending-dd", "branch": "execution-risk"}),
        None,
    );
    run.advance_server_loops(8_500);
    run.tick(9_000);
    run
}

/// Preemptive push suspends the active local loop and injects fresh server context.
fn preemptive_pivot() -> RealtimeStemRun {
    let mut run = RealtimeStemRun::new("case://preemptive-pivot", 3_000);
    run.open_session("burst-003", "convened-burst");
    run.join_participant("cyra", "Cyra Vale", "risk");

    let initial = push_script(
        "burst-003",
        1,
        3_100,
        UrgencyIntent::Informational,
        "Initial dissent mapping",
        serde_json::json!({"objective": "Initial dissent mapping"}),
    );
    run.deliver_push("cyra", initial);
    run.ack_local_spawn("cyra");

    let preemptive = push_script(
        "burst-003",
        2,
        3_400,
        UrgencyIntent::Preemptive,
        "Pivot to live counter-evidence from the room",
        serde_json::json!({
            "objective": "Pivot to live counter-evidence from the room",
            "injected_evidence": ["minority-hypothesis", "field-signal-7"],
        }),
    );
    run.deliver_push("cyra", preemptive);
    run.ack_pause_and_inject("cyra");
    run.complete_local_loop(
        "cyra",
        serde_json::json!({"conclusion": "segment-by-need", "dissent_preserved": true}),
        Some((
            "need_more_evidence",
            "high",
            "quorum://hypothesis/segmentation",
        )),
    );
    run.tick(4_000);
    run
}

/// Three participants join a room with staggered informational pushes and parallel local loops.
fn three_user_concurrent_room() -> RealtimeStemRun {
    let mut run = RealtimeStemRun::new("case://three-user-concurrent-room", 5_000);
    run.open_session("room-030", "live-concurrent-inquiry");
    for (id, name, role) in [
        ("ada", "Ada Reed", "strategy"),
        ("ben", "Ben Moss", "operations"),
        ("dev", "Dev Jain", "frontline"),
    ] {
        run.join_participant(id, name, role);
    }

    let scripts = [
        ("ada", 5_100, "Strategy lane: launch sequencing"),
        ("ben", 5_180, "Operations lane: rollout constraints"),
        ("dev", 5_260, "Frontline lane: customer readiness"),
    ];
    for (index, (participant, at_ms, objective)) in scripts.into_iter().enumerate() {
        let push = push_script(
            "room-030",
            (index + 1) as u32,
            at_ms,
            UrgencyIntent::Informational,
            objective,
            serde_json::json!({"objective": objective}),
        );
        run.deliver_push(participant, push);
        run.ack_local_spawn(participant);
    }

    run.complete_local_loop(
        "ada",
        serde_json::json!({"lane": "strategy", "position": "pilot-first"}),
        Some(("agree", "medium", "quorum://room-030/strategy")),
    );
    run.complete_local_loop(
        "ben",
        serde_json::json!({"lane": "operations", "position": "pilot-first"}),
        Some(("agree", "high", "quorum://room-030/operations")),
    );
    run.complete_local_loop(
        "dev",
        serde_json::json!({"lane": "frontline", "position": "wait-for-evidence"}),
        Some(("uncertain", "medium", "quorum://room-030/frontline")),
    );
    run.tick(6_000);
    run
}

/// One participant tracks a short server probe and a long server synthesis concurrently.
fn long_and_short_server_mix() -> RealtimeStemRun {
    let mut run = RealtimeStemRun::new("case://long-and-short-server-mix", 7_000);
    run.open_session("burst-004", "convened-burst");
    run.join_participant("eli", "Eli Park", "synthesizer");

    let local = push_script(
        "burst-004",
        1,
        7_100,
        UrgencyIntent::Informational,
        "Local framing while server jobs run",
        serde_json::json!({"objective": "Local framing while server jobs run"}),
    );
    run.deliver_push("eli", local);
    run.ack_local_spawn("eli");

    run.spawn_server_loop_direct(
        "eli",
        "srv-probe-010",
        "coverage-probe",
        ServerLoopProfile::ShortProbe,
        7_150,
    );
    run.spawn_server_loop_direct(
        "eli",
        "srv-synth-011",
        "bounded-synthesis",
        ServerLoopProfile::LongSynthesis,
        7_200,
    );

    run.advance_server_loops(7_400);
    run.advance_server_loops(12_500);
    run.complete_local_loop(
        "eli",
        serde_json::json!({
            "conclusion": "bounded-synthesis-ready",
            "server_handles": ["srv-probe-010", "srv-synth-011"],
        }),
        Some(("agree", "high", "quorum://checkpoint/004")),
    );
    run.tick(13_000);
    run
}

/// A gate opens for one participant while others keep advisory notifications flowing.
fn gate_while_loops_active() -> RealtimeStemRun {
    let mut run = RealtimeStemRun::new("case://gate-while-loops-active", 9_000);
    run.open_session("room-100", "crowd-consensus");
    run.join_participant("faye", "Faye Lund", "operator");
    run.join_participant("gia", "Gia Hart", "risk");

    let faye_push = push_script(
        "room-100",
        1,
        9_100,
        UrgencyIntent::Informational,
        "Operator synthesis before gate",
        serde_json::json!({"objective": "Operator synthesis before gate"}),
    );
    run.deliver_push("faye", faye_push);
    run.ack_local_spawn("faye");

    let advisory = push_script(
        "room-100",
        2,
        9_220,
        UrgencyIntent::Advisory,
        "Risk lane update: two dissent clusters remain",
        serde_json::json!({"objective": "Risk lane update: two dissent clusters remain"}),
    );
    run.deliver_push("gia", advisory);

    run.open_gate(
        "faye",
        "gate:room-100:wording",
        "Approve revised wording before admission",
        "Formation stays blocked until resolved",
    );
    run.respond_gate("faye", serde_json::json!({"approved": true}));
    run.complete_local_loop(
        "faye",
        serde_json::json!({"gate": "approved", "wording": "pilot-first with dissent preserved"}),
        None,
    );
    run.tick(10_000);
    run
}

/// A tight wall-clock budget fails a long-running local loop without touching server handles.
fn budget_exhaustion() -> RealtimeStemRun {
    let mut run = RealtimeStemRun::new_with_budget("case://budget-exhaustion", 11_000, 500);
    run.open_session("burst-005", "convened-burst");
    run.join_participant("hugo", "Hugo Stone", "frontline");

    let push = push_script(
        "burst-005",
        1,
        11_100,
        UrgencyIntent::Informational,
        "Slow local deliberation that should time out",
        serde_json::json!({"objective": "Slow local deliberation that should time out"}),
    );
    run.deliver_push("hugo", push);
    run.ack_local_spawn("hugo");
    run.tick(11_700);
    run
}

/// Marquee-style burst room: many humans, mixed urgencies, long+short server loops, gate, admission.
fn marquee_burst_room() -> RealtimeStemRun {
    let mut run = RealtimeStemRun::new("case://marquee-burst-room", 20_000);
    run.open_session("quorum://burst/session-001", "convened-burst");

    let humans = [
        ("human-001", "Ada Reed", "strategy"),
        ("human-002", "Ben Moss", "operations"),
        ("human-003", "Cyra Vale", "risk"),
        ("human-004", "Dev Jain", "frontline"),
        ("human-005", "Eli Park", "customer"),
    ];
    for (id, name, role) in humans {
        run.join_participant(id, name, role);
    }

    for (index, (id, objective)) in humans
        .iter()
        .map(|(id, _, _)| *id)
        .zip([
            "Strategy contribution for checkpoint synthesis",
            "Operations contribution for checkpoint synthesis",
            "Risk contribution for checkpoint synthesis",
            "Frontline contribution for checkpoint synthesis",
            "Customer contribution for checkpoint synthesis",
        ])
        .enumerate()
    {
        let at_ms = 20_100 + (index as u64 * 35);
        let push = push_script(
            "quorum://burst/session-001",
            (index + 1) as u32,
            at_ms,
            UrgencyIntent::Informational,
            objective,
            serde_json::json!({"objective": objective}),
        );
        run.deliver_push(id, push);
        run.ack_local_spawn(id);
    }

    let disruptive = push_script(
        "quorum://burst/session-001",
        6,
        20_450,
        UrgencyIntent::Disruptive,
        "Server DD while humans finish local lanes",
        serde_json::json!({"objective": "Server DD while humans finish local lanes"}),
    );
    run.deliver_push("human-001", disruptive);
    run.ack_server_offload(
        "human-001",
        "srv-dd-100",
        "dd-analysis",
        ServerLoopProfile::LongDd,
    );

    for (index, id) in humans.iter().map(|(id, _, _)| *id).enumerate() {
        run.complete_local_loop(
            id,
            serde_json::json!({
                "participant": id,
                "lane": index,
                "suggestion_only": true,
            }),
            Some(("agree", "medium", &format!("quorum://burst/contrib/{id}"))),
        );
    }

    run.spawn_server_loop_direct(
        "human-002",
        "srv-synth-100",
        "bounded-synthesis",
        ServerLoopProfile::LongSynthesis,
        20_500,
    );
    run.advance_server_loops(21_000);
    run.advance_server_loops(27_000);

    run.open_gate(
        "human-003",
        "gate:checkpoint:001",
        "Operator reviews bounded synthesis before Converge admission",
        "Checkpoint outputs are withheld until approved",
    );
    run.respond_gate("human-003", serde_json::json!({"approved": true}));

    run.emit_checkpoint_admission(
        vec![
            "proposal:coordination:checkpoint:001:core-claim".to_string(),
            "proposal:coordination:checkpoint:001:dissent-preserved".to_string(),
        ],
        vec![
            "fact:coordination:checkpoint:001:core-claim".to_string(),
            "fact:coordination:checkpoint:001:dissent-preserved".to_string(),
        ],
    );
    run.tick(28_000);
    run
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InteractiveCaseId, StemEventKind};

    #[test]
    fn all_cases_are_deterministic() {
        for case in InteractiveCaseId::all() {
            let first = run_case(case);
            let second = run_case(case);
            assert_eq!(
                first.normalized_event_trace(),
                second.normalized_event_trace(),
                "{case:?}"
            );
        }
    }

    #[test]
    fn marquee_burst_room_stresses_mixed_loop_kinds() {
        let run = run_case(InteractiveCaseId::MarqueeBurstRoom);
        let kinds: Vec<_> = run.events.iter().map(|event| event.kind).collect();
        assert!(kinds.contains(&StemEventKind::ParticipantJoined));
        assert!(kinds.contains(&StemEventKind::LocalLoopSpawned));
        assert!(kinds.contains(&StemEventKind::ServerLoopSpawned));
        assert!(kinds.contains(&StemEventKind::ServerLoopCompleted));
        assert!(kinds.contains(&StemEventKind::GateOpened));
        assert!(kinds.contains(&StemEventKind::ConvergeAdmissionRecorded));
        assert!(run.participant_count() >= 5);
    }
}
