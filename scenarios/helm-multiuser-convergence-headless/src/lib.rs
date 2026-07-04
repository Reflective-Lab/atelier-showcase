// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Multi-user realtime Helm coordination scenario — headless proof.
//!
//! Drives real [`ClientHelm`] × 4 (one per participant role) against shared
//! [`SessionRegistry`], [`PresenceRegistry`], and [`DecisionLedger`] — the
//! same coordination primitives the live HTTP surface uses, but exercised
//! in-process with a simulated clock.
//!
//! The scenario proves:
//! - Each participant's `ClientHelm` routes server pushes to the right action
//!   (Informational → NoAction, Advisory → SpawnFormation, Disruptive →
//!   parallel formation, Preemptive → PauseAndInject).
//! - `SessionRegistry` correctly scopes sessions to a workspace.
//! - `PresenceRegistry` lets two participants soft-claim the same subject
//!   (optimistic — never a lock).
//! - `DecisionLedger` records the first decision, is idempotent on a repeat,
//!   and rejects a divergent second decision as a conflict.

pub mod cases;
pub mod participant;
pub mod report;
pub mod server_pool;
pub mod session;

pub use cases::{ConvergenceCase, run_case};
pub use participant::{ParticipantRole, ParticipantSlot};
pub use report::{jsonl_timeline, markdown_report};
pub use server_pool::{ServerPush, SuggestorKind};
pub use session::{
    ConvergenceEvent, ConvergenceEventKind, MultiUserConvergenceSession, SessionPhase,
};
