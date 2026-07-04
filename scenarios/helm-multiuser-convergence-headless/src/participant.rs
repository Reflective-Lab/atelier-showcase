// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

use application_kernel::ActorKind;
use helm_client::client::ClientHelm;
use helm_coordination::{OperatorPrincipal, Session, SessionRegistry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParticipantRole {
    /// Drives gate decisions and claims primary convergence subjects.
    Leader,
    /// Processes Disruptive/Advisory pushes from the solver suggestors.
    Analyst,
    /// Raises soft-claims on dissent subjects; exercises the conflict path.
    Skeptic,
    /// Agent observer — receives Informational pushes; never claims.
    Observer,
}

impl ParticipantRole {
    pub const fn actor_id(self) -> &'static str {
        match self {
            Self::Leader => "participant:leader",
            Self::Analyst => "participant:analyst",
            Self::Skeptic => "participant:skeptic",
            Self::Observer => "participant:observer",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Leader => "Leader",
            Self::Analyst => "Analyst",
            Self::Skeptic => "Skeptic",
            Self::Observer => "Observer",
        }
    }

    pub const fn actor_kind(self) -> ActorKind {
        match self {
            Self::Observer => ActorKind::Agent,
            _ => ActorKind::Human,
        }
    }
}

pub struct ParticipantSlot {
    pub role: ParticipantRole,
    pub principal: OperatorPrincipal,
    pub session: Session,
    pub helm: ClientHelm,
}

impl ParticipantSlot {
    /// Open a session for this role in the shared registry.
    pub fn open(role: ParticipantRole, workspace_id: &str, registry: &SessionRegistry) -> Self {
        let principal = OperatorPrincipal::new(
            role.actor_id(),
            role.display_name(),
            role.actor_kind(),
            workspace_id,
        );
        let session = registry.open(principal.clone());
        // 10-minute wall-clock budget per formation — matches a focused working session.
        let helm = ClientHelm::with_budget_ms(10 * 60 * 1_000);
        Self {
            role,
            principal,
            session,
            helm,
        }
    }
}
