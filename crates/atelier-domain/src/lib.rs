// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Kernel domain agents and use cases for Converge.
//!
//! This crate contains the kernel packs and pure constraint-satisfaction
//! use cases that form the foundation of Converge.
//!
//! # Kernel Packs
//!
//! - [`packs::money`]: Financial transaction substrate
//! - [`packs::trust`]: Audit/access/provenance substrate
//! - [`packs::delivery`]: Promise fulfillment protocol
//! - [`packs::data_metrics`]: Instrumentation substrate
//!
//! # Use Cases
//!
//! - [`ask_converge`]: Query interface
//! - [`meeting_scheduler`]: Pure constraint satisfaction (kernel)
//! - [`resource_routing`]: Pure constraint satisfaction (kernel)
//! - [`sec_risk`]: SEC filing risk-review policy pack
//! - [`drafting`]: Content drafting (kernel utility)
//! - [`form_filler`]: Form filling agents (kernel utility)

use converge_core::{ContextKey, FactPayload, ProposalId, ProposedFact};

pub mod ask_converge;
pub mod domain_invariants;
pub mod drafting;
pub mod drafting_llm;
pub mod eval_agent;
pub mod evals;
mod flow_governance;
pub mod form_filler;
pub mod meeting_scheduler;
pub mod packs;
pub mod protocol;
pub mod resource_routing;
pub mod sec_risk;

pub mod llm_utils;
pub mod mock;
pub mod retrieval;

// LLM-enabled versions of use cases
pub mod meeting_scheduler_llm;

pub use ask_converge::{AskConvergeAgent, GroundedAnswerInvariant, RecallNotEvidenceInvariant};
pub use drafting::{DraftingComposerAgent, DraftingResearchAgent};

pub use form_filler::{
    CompletenessAgent, FieldMappingAgent, FillPlanAgent, FormSchemaAgent, NormalizationAgent,
    ProposalEmitterAgent, RiskClassifierAgent,
};

pub use meeting_scheduler::{
    // Agents
    AvailabilityRetrievalAgent,
    ConflictDetectionAgent,
    // Invariants
    RequireParticipantAvailability,
    RequirePositiveDuration,
    RequireValidSlot,
    SlotOptimizationAgent,
    TimeZoneNormalizationAgent,
    WorkingHoursConstraintAgent,
};

pub use resource_routing::{
    // Agents
    ConstraintValidationAgent,
    FeasibilityAgent,
    // Invariants
    RequireAllTasksAssigned,
    RequireCapacityRespected,
    RequireValidDefinitions,
    ResourceRetrievalAgent,
    SolverAgent,
    TaskRetrievalAgent,
};

pub use domain_invariants::{AuditTrailRequired, AuthorityRequired};
pub use protocol::{
    ATELIER_DOMAIN_PROVENANCE, DomainRecordPayload, DomainTextPayload, admitted_text, domain_text,
    json_value, payload_any, payload_contains, record_data, record_payload,
};

// Pack-specific evals
pub use evals::{
    // Trust Pack
    AccessComplianceEval,
    AuditCoverageEval,
    // Data Metrics Pack
    DashboardSourceEval,
    // Money Pack
    InvoiceAccuracyEval,
    // General kernel evals
    MeetingScheduleFeasibilityEval,
    MetricDefinitionQualityEval,
    PaymentReconciliationEval,
    // Delivery Pack
    PromiseFulfillmentEval,
    RbacEnforcementEval,
    ScopeCreepDetectionEval,
};

pub(crate) fn proposal(
    _provenance: impl Into<String>,
    key: ContextKey,
    id: impl Into<String>,
    payload: impl FactPayload + PartialEq,
) -> ProposedFact {
    ProposedFact::new(
        key,
        ProposalId::new(id.into()),
        payload,
        ATELIER_DOMAIN_PROVENANCE,
    )
}

pub(crate) fn record(
    provenance: impl Into<String>,
    key: ContextKey,
    id: impl Into<String>,
    record_type: impl Into<String>,
    data: serde_json::Value,
) -> ProposedFact {
    proposal(
        provenance,
        key,
        id,
        DomainRecordPayload::new(record_type, data),
    )
}

pub(crate) fn json_record(
    provenance: impl Into<String>,
    key: ContextKey,
    id: impl Into<String>,
    data: serde_json::Value,
) -> ProposedFact {
    let id = id.into();
    proposal(
        provenance,
        key,
        id.clone(),
        DomainRecordPayload::new(id, data),
    )
}

pub(crate) fn text(
    provenance: impl Into<String>,
    key: ContextKey,
    id: impl Into<String>,
    text_type: impl Into<String>,
    text: impl Into<String>,
) -> ProposedFact {
    proposal(provenance, key, id, DomainTextPayload::new(text_type, text))
}
