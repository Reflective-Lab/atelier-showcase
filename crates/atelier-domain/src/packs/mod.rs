// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Kernel pack agents organized by domain.
//!
//! Each pack module contains agents that implement the contracts
//! defined in the corresponding Gherkin specs (specs/*.feature).
//!
//! # Kernel Packs
//!
//! - [`money`]: Finance operations (AR -> AP -> Reconcile -> Close)
//! - [`trust`]: Cross-cutting substrate (Identity -> Access -> Audit -> Provenance)
//! - [`delivery`]: Promise fulfillment (Promise -> Execute -> Blockers -> Complete)
//! - [`data_metrics`]: Single source of truth (Instrument -> Collect -> Validate -> Report -> Alert)
//!
//! # Suggestor Wiring
//!
//! Suggestor IDs in YAML (packs/*.yaml) map to structs in these modules:
//!
//! ```yaml
//! # packs/money.yaml
//! agents:
//!   - id: invoice_creator
//!     requirements: deterministic
//! ```
//!
//! Maps to:
//!
//! ```rust,ignore
//! use atelier_domain::packs::money::InvoiceCreatorAgent;
//! engine.register_suggestor(InvoiceCreatorAgent);
//! ```
//!
//! # Fact ID Prefixes
//!
//! Since converge-core uses a fixed ContextKey enum, pack-specific facts are
//! distinguished by their ID prefixes. Each pack defines its own prefixes:
//!
//! - Money: `invoice:`, `payment:`, `ledger:`, `period:`
//! - Trust: `session:`, `audit:`, `compliance:`, `violation:`, `remediation:`
//! - Delivery: `promise:`, `scope:`, `task:`, `blocker:`, `risk:`
//! - Data Metrics: `metric:`, `source:`, `pipeline:`, `validation:`, `dashboard:`, `report:`, `alert:`, `anomaly:`

pub mod data_metrics;
pub mod delivery;
pub mod money;
pub mod trust;

// Re-export agents for convenience (explicit to avoid conflicts)
pub use delivery::{
    AcceptanceRequestorAgent, BlockerDetectorAgent, BlockerHasResolutionPathInvariant,
    BlockerRouterAgent, CompletedPromiseHasAcceptanceInvariant, PostmortemSchedulerAgent,
    PromiseCreatorAgent, PromiseHasDealInvariant, RiskAssessorAgent,
    ScopeChangeRequiresApprovalInvariant, ScopeExtractorAgent, StatusAggregatorAgent,
    WorkBreakdownAgent,
};

pub use money::{
    ClosedPeriodReadonlyInvariant, InvoiceCreatorAgent, InvoiceHasCustomerInvariant,
    InvoiceIssuerAgent, OverdueDetectorAgent, PaymentAllocationCompleteInvariant,
    PaymentAllocatorAgent, PeriodCloserAgent, ReconciliationMatcherAgent,
};

pub use trust::{
    AllActionsAuditedInvariant, AuditImmutabilityInvariant, AuditWriterAgent,
    ComplianceScannerAgent, ContractExecutionAgent, LegalActionsAuditedInvariant, PiiRedactorAgent,
    ProvenanceTrackerAgent, RbacEnforcerAgent, SessionValidatorAgent, ViolationRemediatorAgent,
    ViolationsHaveRemediationInvariant,
};

pub use data_metrics::{
    AlertEvaluatorAgent, AlertHasOwnerInvariant, AnomalyDetectorAgent, DashboardBuilderAgent,
    DashboardCitesSourcesInvariant, DataFreshnessInvariant, DataValidatorAgent,
    FreshnessMonitorAgent, MetricCalculatorAgent, MetricDefinitionVersionedInvariant,
    MetricRegistrarAgent, PipelineCoordinatorAgent, ReportGeneratorAgent, SourceConnectorAgent,
};
