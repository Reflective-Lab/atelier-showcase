// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Simulated server-side suggestor pool.
//!
//! In production, these suggestors run on the server — each analyses the
//! converging fact graph and emits [`SessionPush`] events over SSE. Here they
//! are deterministic functions of (phase, cycle) so the scenario replays
//! identically every run.

use helm_session_contracts::finding::FindingId;
use helm_session_contracts::push::{SessionContext, SessionPush};
use helm_session_contracts::urgency::UrgencyIntent;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Which suggestor kind produced a push.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SuggestorKind {
    /// Language-model synthesis — always Advisory.
    LlmSynthesis,
    /// Ferrox combinatorial optimizer — Disruptive when a better assignment found.
    FeroxOptimizer,
    /// Arbiter policy evaluator — Preemptive when a constraint violation detected.
    ArbiterPolicy,
    /// Prism analytics — always Informational (ambient metrics).
    PrismAnalytics,
}

impl SuggestorKind {
    pub const fn urgency(self) -> UrgencyIntent {
        match self {
            Self::LlmSynthesis => UrgencyIntent::Advisory,
            Self::FeroxOptimizer => UrgencyIntent::Disruptive,
            Self::ArbiterPolicy => UrgencyIntent::Preemptive,
            Self::PrismAnalytics => UrgencyIntent::Informational,
        }
    }

    pub const fn suggestor_id(self) -> &'static str {
        match self {
            Self::LlmSynthesis => "suggestor:llm-synthesis",
            Self::FeroxOptimizer => "suggestor:ferrox-optimizer",
            Self::ArbiterPolicy => "suggestor:arbiter-policy",
            Self::PrismAnalytics => "suggestor:prism-analytics",
        }
    }
}

/// A server push ready to be delivered to one or more [`ClientHelm`] instances.
#[derive(Debug, Clone)]
pub struct ServerPush {
    pub suggestor: SuggestorKind,
    pub push: SessionPush,
}

/// Build a deterministic push from a suggestor at a given phase/cycle.
///
/// The payload carries an `objective` field so `ClientHelm`'s
/// `push_objective_description` extracts a readable description.
pub fn make_push(
    suggestor: SuggestorKind,
    session_id: &str,
    phase: &str,
    cycle: u32,
    simulated_ms: u64,
) -> ServerPush {
    let objective = match suggestor {
        SuggestorKind::LlmSynthesis => {
            format!("LLM synthesis: convergence candidate {cycle} in phase {phase}")
        }
        SuggestorKind::FeroxOptimizer => {
            format!("FERROX: improved allocation found at cycle {cycle} — review before admission")
        }
        SuggestorKind::ArbiterPolicy => {
            format!("ARBITER: constraint violation at cycle {cycle} — preempt and re-evaluate")
        }
        SuggestorKind::PrismAnalytics => {
            format!("PRISM: ambient metrics for phase {phase}, cycle {cycle}")
        }
    };

    let push = SessionPush {
        finding_id: FindingId::from_string(format!(
            "finding:{suggestor_id}:{phase}:{cycle}",
            suggestor_id = suggestor.suggestor_id()
        )),
        urgency_intent: suggestor.urgency(),
        payload: json!({
            "objective": objective,
            "suggestor": suggestor.suggestor_id(),
            "phase": phase,
            "cycle": cycle,
        }),
        session_context: SessionContext {
            session_id: session_id.to_string(),
            phase: phase.to_string(),
            cycle,
            timestamp_ms: simulated_ms,
        },
    };

    ServerPush { suggestor, push }
}
