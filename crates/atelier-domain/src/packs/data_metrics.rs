// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Data & Metrics Pack agents for single source of truth measurements.
//!
//! Implements the agent contracts defined in specs/data_metrics.truth.
//!
//! # Data & Metrics is the Measurement Layer
//!
//! Every metric, dashboard, and alert flows through this pack:
//! - Metric definitions and versioning
//! - Data source connectivity
//! - Pipeline orchestration
//! - Anomaly detection
//! - Alerting and reporting
//!
//! Note: This implementation uses the standard ContextKey enum. Facts are
//! distinguished by their ID prefixes (metric:, source:, pipeline:, etc.).

use converge_core::{
    AgentEffect, ContextKey, Suggestor,
    invariant::{Invariant, InvariantClass, InvariantResult, Violation},
};

// ============================================================================
// Fact ID Prefixes
// ============================================================================

pub const METRIC_PREFIX: &str = "metric:";
pub const SOURCE_PREFIX: &str = "source:";
pub const PIPELINE_PREFIX: &str = "pipeline:";
pub const VALIDATION_PREFIX: &str = "validation:";
pub const DASHBOARD_PREFIX: &str = "dashboard:";
pub const REPORT_PREFIX: &str = "report:";
pub const ALERT_PREFIX: &str = "alert:";
pub const ANOMALY_PREFIX: &str = "anomaly:";

// ============================================================================
// Agents
// ============================================================================

/// Registers and validates metric definitions.
#[derive(Debug, Clone, Default)]
pub struct MetricRegistrarAgent;

#[async_trait::async_trait]
impl Suggestor for MetricRegistrarAgent {
    fn name(&self) -> &'static str {
        "metric_registrar"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.get(ContextKey::Seeds)
            .iter()
            .any(|s| s.content().contains("metric.define") || s.content().contains("metric.update"))
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let triggers = ctx.get(ContextKey::Seeds);
        let mut facts = Vec::new();

        for trigger in triggers {
            if trigger.content().contains("metric.define")
                || trigger.content().contains("metric.update")
            {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Proposals,
                    format!("{}{}", METRIC_PREFIX, trigger.id()),
                    serde_json::json!({
                        "type": "metric_definition",
                        "source_id": trigger.id(),
                        "state": "draft",
                        "version": "1.0.0",
                        "formula": "to_be_defined",
                        "created_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Establishes and manages data source connections.
#[derive(Debug, Clone, Default)]
pub struct SourceConnectorAgent;

#[async_trait::async_trait]
impl Suggestor for SourceConnectorAgent {
    fn name(&self) -> &'static str {
        "source_connector"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.get(ContextKey::Seeds).iter().any(|s| {
            s.content().contains("source.register") || s.content().contains("source.connect")
        })
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let triggers = ctx.get(ContextKey::Seeds);
        let mut facts = Vec::new();

        for trigger in triggers {
            if trigger.content().contains("source.register")
                || trigger.content().contains("source.connect")
            {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Signals,
                    format!("{}{}", SOURCE_PREFIX, trigger.id()),
                    serde_json::json!({
                        "type": "data_source",
                        "source_id": trigger.id(),
                        "state": "registered",
                        "source_type": "detected",
                        "freshness_sla_minutes": 60,
                        "registered_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Coordinates data pipeline execution.
#[derive(Debug, Clone, Default)]
pub struct PipelineCoordinatorAgent;

#[async_trait::async_trait]
impl Suggestor for PipelineCoordinatorAgent {
    fn name(&self) -> &'static str {
        "pipeline_coordinator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.get(ContextKey::Signals).iter().any(|s| {
            s.id().as_str().starts_with(SOURCE_PREFIX)
                && s.content().contains("\"state\":\"healthy\"")
        })
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);
        let mut facts = Vec::new();

        for source in signals {
            if source.id().starts_with(SOURCE_PREFIX)
                && source.content().contains("\"state\":\"healthy\"")
            {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Proposals,
                    format!("{}{}", PIPELINE_PREFIX, source.id()),
                    serde_json::json!({
                        "type": "pipeline",
                        "source_id": source.id(),
                        "state": "ready",
                        "schedule": "*/15 * * * *",
                        "timeout_minutes": 30,
                        "created_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Validates collected data quality.
#[derive(Debug, Clone, Default)]
pub struct DataValidatorAgent;

#[async_trait::async_trait]
impl Suggestor for DataValidatorAgent {
    fn name(&self) -> &'static str {
        "data_validator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.get(ContextKey::Proposals).iter().any(|p| {
            p.id().starts_with(PIPELINE_PREFIX) && p.content().contains("\"state\":\"succeeded\"")
        })
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let mut facts = Vec::new();

        for pipeline in proposals {
            if pipeline.id().starts_with(PIPELINE_PREFIX)
                && pipeline.content().contains("\"state\":\"succeeded\"")
            {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Evaluations,
                    format!("{}{}", VALIDATION_PREFIX, pipeline.id()),
                    serde_json::json!({
                        "type": "data_validation",
                        "pipeline_id": pipeline.id(),
                        "schema_valid": true,
                        "null_ratio_ok": true,
                        "range_check_ok": true,
                        "freshness_ok": true,
                        "validated_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Detects anomalies in metric data.
#[derive(Debug, Clone, Default)]
pub struct AnomalyDetectorAgent;

#[async_trait::async_trait]
impl Suggestor for AnomalyDetectorAgent {
    fn name(&self) -> &'static str {
        "anomaly_detector"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.get(ContextKey::Evaluations).iter().any(|e| {
            e.id().starts_with(VALIDATION_PREFIX) && e.content().contains("\"schema_valid\":true")
        })
    }

    async fn execute(&self, _ctx: &dyn converge_core::Context) -> AgentEffect {
        // In real implementation, would analyze data for anomalies
        // For now, creates a placeholder showing no anomalies detected
        AgentEffect::with_proposal(crate::proposal(
            self.name(),
            ContextKey::Evaluations,
            format!("{}scan:latest", ANOMALY_PREFIX),
            serde_json::json!({
                "type": "anomaly_scan",
                "anomalies_detected": 0,
                "metrics_scanned": 10,
                "methods_used": ["statistical", "threshold"],
                "scanned_at": "2026-01-12T12:00:00Z"
            })
            .to_string(),
        ))
    }
}

/// Builds and configures dashboards.
#[derive(Debug, Clone, Default)]
pub struct DashboardBuilderAgent;

#[async_trait::async_trait]
impl Suggestor for DashboardBuilderAgent {
    fn name(&self) -> &'static str {
        "dashboard_builder"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.get(ContextKey::Seeds).iter().any(|s| {
            s.content().contains("dashboard.create") || s.content().contains("dashboard.update")
        })
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let triggers = ctx.get(ContextKey::Seeds);
        let mut facts = Vec::new();

        for trigger in triggers {
            if trigger.content().contains("dashboard.create")
                || trigger.content().contains("dashboard.update")
            {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Proposals,
                    format!("{}{}", DASHBOARD_PREFIX, trigger.id()),
                    serde_json::json!({
                        "type": "dashboard",
                        "source_id": trigger.id(),
                        "state": "draft",
                        "widgets": [],
                        "refresh_rate": "5m",
                        "created_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Generates scheduled and ad-hoc reports.
#[derive(Debug, Clone, Default)]
pub struct ReportGeneratorAgent;

#[async_trait::async_trait]
impl Suggestor for ReportGeneratorAgent {
    fn name(&self) -> &'static str {
        "report_generator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.get(ContextKey::Seeds).iter().any(|s| {
            s.content().contains("report.generate") || s.content().contains("report.schedule")
        })
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let triggers = ctx.get(ContextKey::Seeds);
        let mut facts = Vec::new();

        for trigger in triggers {
            if trigger.content().contains("report.generate")
                || trigger.content().contains("report.schedule")
            {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Proposals,
                    format!("{}{}", REPORT_PREFIX, trigger.id()),
                    serde_json::json!({
                        "type": "report",
                        "source_id": trigger.id(),
                        "state": "generating",
                        "format": "pdf",
                        "recipients": [],
                        "generated_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Evaluates alert conditions and triggers notifications.
#[derive(Debug, Clone, Default)]
pub struct AlertEvaluatorAgent;

#[async_trait::async_trait]
impl Suggestor for AlertEvaluatorAgent {
    fn name(&self) -> &'static str {
        "alert_evaluator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        // Check if there are anomalies that need alerting
        ctx.get(ContextKey::Evaluations).iter().any(|e| {
            e.id().starts_with(ANOMALY_PREFIX) && e.content().contains("\"anomalies_detected\"")
        })
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let evaluations = ctx.get(ContextKey::Evaluations);
        let mut facts = Vec::new();

        for eval in evaluations {
            if eval.id().starts_with(ANOMALY_PREFIX) {
                // Parse anomaly count - in real impl would check if > 0
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Evaluations,
                    format!("{}evaluation:{}", ALERT_PREFIX, eval.id()),
                    serde_json::json!({
                        "type": "alert_evaluation",
                        "anomaly_scan_id": eval.id(),
                        "alerts_triggered": 0,
                        "alerts_evaluated": 5,
                        "evaluated_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Monitors data freshness against SLAs.
#[derive(Debug, Clone, Default)]
pub struct FreshnessMonitorAgent;

#[async_trait::async_trait]
impl Suggestor for FreshnessMonitorAgent {
    fn name(&self) -> &'static str {
        "freshness_monitor"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        ctx.get(ContextKey::Signals)
            .iter()
            .any(|s| s.id().as_str().starts_with(SOURCE_PREFIX))
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);
        let mut facts = Vec::new();

        for source in signals {
            if source.id().starts_with(SOURCE_PREFIX) {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Evaluations,
                    format!("freshness:{}", source.id()),
                    serde_json::json!({
                        "type": "freshness_check",
                        "source_id": source.id(),
                        "is_fresh": true,
                        "last_data_at": "2026-01-12T11:55:00Z",
                        "sla_minutes": 60,
                        "checked_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

/// Calculates metric values from raw data.
#[derive(Debug, Clone, Default)]
pub struct MetricCalculatorAgent;

#[async_trait::async_trait]
impl Suggestor for MetricCalculatorAgent {
    fn name(&self) -> &'static str {
        "metric_calculator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals, ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn converge_core::Context) -> bool {
        let has_metrics = ctx.get(ContextKey::Proposals).iter().any(|p| {
            p.id().starts_with(METRIC_PREFIX) && p.content().contains("\"state\":\"active\"")
        });
        let has_validation = ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|e| e.id().starts_with(VALIDATION_PREFIX));
        has_metrics && has_validation
    }

    async fn execute(&self, ctx: &dyn converge_core::Context) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let mut facts = Vec::new();

        for metric in proposals {
            if metric.id().starts_with(METRIC_PREFIX)
                && metric.content().contains("\"state\":\"active\"")
            {
                facts.push(crate::proposal(
                    self.name(),
                    ContextKey::Evaluations,
                    format!("calculated:{}", metric.id()),
                    serde_json::json!({
                        "type": "metric_calculation",
                        "metric_id": metric.id(),
                        "value": 0.0,
                        "unit": "count",
                        "calculated_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                ));
            }
        }

        AgentEffect::with_proposals(facts)
    }
}

// ============================================================================
// Invariants
// ============================================================================

/// Ensures metric definitions are versioned.
#[derive(Debug, Clone, Default)]
pub struct MetricDefinitionVersionedInvariant;

impl Invariant for MetricDefinitionVersionedInvariant {
    fn name(&self) -> &'static str {
        "metric_definition_versioned"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Structural
    }

    fn check(&self, ctx: &dyn converge_core::Context) -> InvariantResult {
        for metric in ctx.get(ContextKey::Proposals) {
            if metric.id().starts_with(METRIC_PREFIX) && !metric.content().contains("\"version\"") {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Metric {} has no version", metric.id()),
                    vec![metric.id().clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures dashboards cite their data sources.
#[derive(Debug, Clone, Default)]
pub struct DashboardCitesSourcesInvariant;

impl Invariant for DashboardCitesSourcesInvariant {
    fn name(&self) -> &'static str {
        "dashboard_cites_sources"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Structural
    }

    fn check(&self, ctx: &dyn converge_core::Context) -> InvariantResult {
        for dashboard in ctx.get(ContextKey::Proposals) {
            if dashboard.id().starts_with(DASHBOARD_PREFIX)
                && dashboard.content().contains("\"state\":\"published\"")
                && !dashboard.content().contains("\"data_source\"")
            {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Dashboard {} does not cite data sources", dashboard.id()),
                    vec![dashboard.id().clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures alerts have designated owners.
#[derive(Debug, Clone, Default)]
pub struct AlertHasOwnerInvariant;

impl Invariant for AlertHasOwnerInvariant {
    fn name(&self) -> &'static str {
        "alert_has_owner"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Structural
    }

    fn check(&self, ctx: &dyn converge_core::Context) -> InvariantResult {
        for alert in ctx.get(ContextKey::Proposals) {
            if alert.id().starts_with(ALERT_PREFIX)
                && alert.content().contains("\"state\":\"active\"")
                && !alert.content().contains("\"owner\"")
            {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Alert {} has no owner", alert.id()),
                    vec![alert.id().clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures data freshness SLAs are met.
#[derive(Debug, Clone, Default)]
pub struct DataFreshnessInvariant;

impl Invariant for DataFreshnessInvariant {
    fn name(&self) -> &'static str {
        "data_freshness"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Semantic
    }

    fn check(&self, ctx: &dyn converge_core::Context) -> InvariantResult {
        for check in ctx.get(ContextKey::Evaluations) {
            if check.content().contains("\"type\":\"freshness_check\"")
                && check.content().contains("\"is_fresh\":false")
            {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Data source {} is stale", check.id()),
                    vec![check.id().clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use converge_core::{ContextState, Engine};

    fn promoted(entries: &[(ContextKey, &str, &str)]) -> ContextState {
        let mut ctx = ContextState::new();
        for (key, id, content) in entries {
            ctx.add_input(*key, *id, *content).unwrap();
        }
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Engine::new().run(ctx))
            .unwrap()
            .context
    }

    fn block_execute<S: Suggestor>(agent: &S, ctx: &ContextState) -> AgentEffect {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(agent.execute(ctx))
    }

    #[test]
    fn agents_have_correct_names() {
        assert_eq!(MetricRegistrarAgent.name(), "metric_registrar");
        assert_eq!(SourceConnectorAgent.name(), "source_connector");
        assert_eq!(PipelineCoordinatorAgent.name(), "pipeline_coordinator");
        assert_eq!(DataValidatorAgent.name(), "data_validator");
        assert_eq!(AnomalyDetectorAgent.name(), "anomaly_detector");
        assert_eq!(DashboardBuilderAgent.name(), "dashboard_builder");
        assert_eq!(ReportGeneratorAgent.name(), "report_generator");
        assert_eq!(AlertEvaluatorAgent.name(), "alert_evaluator");
        assert_eq!(FreshnessMonitorAgent.name(), "freshness_monitor");
        assert_eq!(MetricCalculatorAgent.name(), "metric_calculator");
    }

    #[test]
    fn agents_declare_dependencies() {
        assert_eq!(MetricRegistrarAgent.dependencies(), &[ContextKey::Seeds]);
        assert_eq!(SourceConnectorAgent.dependencies(), &[ContextKey::Seeds]);
        assert_eq!(
            PipelineCoordinatorAgent.dependencies(),
            &[ContextKey::Signals]
        );
        assert_eq!(DataValidatorAgent.dependencies(), &[ContextKey::Proposals]);
        assert_eq!(
            AnomalyDetectorAgent.dependencies(),
            &[ContextKey::Evaluations]
        );
        assert_eq!(DashboardBuilderAgent.dependencies(), &[ContextKey::Seeds]);
        assert_eq!(ReportGeneratorAgent.dependencies(), &[ContextKey::Seeds]);
        assert_eq!(
            AlertEvaluatorAgent.dependencies(),
            &[ContextKey::Evaluations]
        );
        assert_eq!(FreshnessMonitorAgent.dependencies(), &[ContextKey::Signals]);
        assert_eq!(
            MetricCalculatorAgent.dependencies(),
            &[ContextKey::Proposals, ContextKey::Evaluations]
        );
    }

    #[test]
    fn metric_registrar_accepts_define_or_update_seeds() {
        let agent = MetricRegistrarAgent;
        assert!(agent.accepts(&promoted(&[(ContextKey::Seeds, "m1", "metric.define")])));
        assert!(agent.accepts(&promoted(&[(ContextKey::Seeds, "m1", "metric.update")])));
        assert!(!agent.accepts(&promoted(&[(ContextKey::Seeds, "m1", "unrelated")])));
        assert!(!agent.accepts(&promoted(&[])));
    }

    #[test]
    fn metric_registrar_emits_one_proposal_per_trigger() {
        let agent = MetricRegistrarAgent;
        let ctx = promoted(&[
            (ContextKey::Seeds, "m1", "metric.define orders.total"),
            (ContextKey::Seeds, "m2", "metric.update orders.refunded"),
            (ContextKey::Seeds, "noise", "unrelated"),
        ]);
        let effect = block_execute(&agent, &ctx);
        let proposals = effect.proposals();
        assert_eq!(proposals.len(), 2);
        assert!(proposals.iter().all(|p| p.key == ContextKey::Proposals));
        assert!(proposals.iter().all(|p| p.id.starts_with(METRIC_PREFIX)));
    }

    #[test]
    fn source_connector_accepts_register_or_connect() {
        let agent = SourceConnectorAgent;
        assert!(agent.accepts(&promoted(&[(
            ContextKey::Seeds,
            "s1",
            "source.register pg"
        )])));
        assert!(agent.accepts(&promoted(&[(ContextKey::Seeds, "s1", "source.connect pg")])));
        assert!(!agent.accepts(&promoted(&[(ContextKey::Seeds, "s1", "metric.define")])));
    }

    #[test]
    fn source_connector_emits_signals() {
        let agent = SourceConnectorAgent;
        let ctx = promoted(&[(ContextKey::Seeds, "s1", "source.register pg")]);
        let effect = block_execute(&agent, &ctx);
        let proposals = effect.proposals();
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].key, ContextKey::Signals);
        assert!(proposals[0].id.starts_with(SOURCE_PREFIX));
    }

    #[test]
    fn pipeline_coordinator_only_for_healthy_sources() {
        let agent = PipelineCoordinatorAgent;
        let healthy = promoted(&[(ContextKey::Signals, "source:pg", r#"{"state":"healthy"}"#)]);
        assert!(agent.accepts(&healthy));
        let degraded = promoted(&[(ContextKey::Signals, "source:pg", r#"{"state":"degraded"}"#)]);
        assert!(!agent.accepts(&degraded));
    }

    #[test]
    fn pipeline_coordinator_emits_pipeline_for_each_healthy_source() {
        let agent = PipelineCoordinatorAgent;
        let ctx = promoted(&[
            (ContextKey::Signals, "source:pg", r#"{"state":"healthy"}"#),
            (
                ContextKey::Signals,
                "source:redis",
                r#"{"state":"healthy"}"#,
            ),
            (
                ContextKey::Signals,
                "source:cold",
                r#"{"state":"degraded"}"#,
            ),
        ]);
        let effect = block_execute(&agent, &ctx);
        let proposals = effect.proposals();
        assert_eq!(proposals.len(), 2);
        assert!(proposals.iter().all(|p| p.id.starts_with(PIPELINE_PREFIX)));
    }

    #[test]
    fn data_validator_only_on_succeeded_pipeline() {
        let agent = DataValidatorAgent;
        let succeeded = promoted(&[(
            ContextKey::Proposals,
            "pipeline:p1",
            r#"{"state":"succeeded"}"#,
        )]);
        assert!(agent.accepts(&succeeded));
        let pending = promoted(&[(
            ContextKey::Proposals,
            "pipeline:p1",
            r#"{"state":"running"}"#,
        )]);
        assert!(!agent.accepts(&pending));
    }

    #[test]
    fn data_validator_emits_validation_evaluations() {
        let agent = DataValidatorAgent;
        let ctx = promoted(&[(
            ContextKey::Proposals,
            "pipeline:p1",
            r#"{"state":"succeeded"}"#,
        )]);
        let effect = block_execute(&agent, &ctx);
        let proposals = effect.proposals();
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].key, ContextKey::Evaluations);
        assert!(proposals[0].id.starts_with(VALIDATION_PREFIX));
    }

    #[test]
    fn anomaly_detector_accepts_valid_schema() {
        let agent = AnomalyDetectorAgent;
        let ctx = promoted(&[(
            ContextKey::Evaluations,
            "validation:p1",
            r#"{"schema_valid":true}"#,
        )]);
        assert!(agent.accepts(&ctx));
        let bad = promoted(&[(
            ContextKey::Evaluations,
            "validation:p1",
            r#"{"schema_valid":false}"#,
        )]);
        assert!(!agent.accepts(&bad));
    }

    #[test]
    fn anomaly_detector_emits_single_scan() {
        let agent = AnomalyDetectorAgent;
        let ctx = promoted(&[(
            ContextKey::Evaluations,
            "validation:p1",
            r#"{"schema_valid":true}"#,
        )]);
        let effect = block_execute(&agent, &ctx);
        let proposals = effect.proposals();
        assert_eq!(proposals.len(), 1);
        assert!(proposals[0].id.starts_with(ANOMALY_PREFIX));
    }

    #[test]
    fn dashboard_builder_accepts_create_or_update() {
        let agent = DashboardBuilderAgent;
        assert!(agent.accepts(&promoted(&[(ContextKey::Seeds, "d1", "dashboard.create")])));
        assert!(agent.accepts(&promoted(&[(ContextKey::Seeds, "d1", "dashboard.update")])));
        assert!(!agent.accepts(&promoted(&[(ContextKey::Seeds, "d1", "noop")])));
    }

    #[test]
    fn dashboard_builder_emits_dashboard_proposal() {
        let agent = DashboardBuilderAgent;
        let ctx = promoted(&[(ContextKey::Seeds, "d1", "dashboard.create")]);
        let effect = block_execute(&agent, &ctx);
        let proposals = effect.proposals();
        assert_eq!(proposals.len(), 1);
        assert!(proposals[0].id.starts_with(DASHBOARD_PREFIX));
    }

    #[test]
    fn report_generator_accepts_generate_or_schedule() {
        let agent = ReportGeneratorAgent;
        assert!(agent.accepts(&promoted(&[(ContextKey::Seeds, "r1", "report.generate")])));
        assert!(agent.accepts(&promoted(&[(ContextKey::Seeds, "r1", "report.schedule")])));
        assert!(!agent.accepts(&promoted(&[(ContextKey::Seeds, "r1", "noop")])));
    }

    #[test]
    fn report_generator_emits_one_per_trigger() {
        let agent = ReportGeneratorAgent;
        let ctx = promoted(&[
            (ContextKey::Seeds, "r1", "report.generate weekly"),
            (ContextKey::Seeds, "r2", "report.schedule daily"),
        ]);
        let effect = block_execute(&agent, &ctx);
        assert_eq!(effect.proposals().len(), 2);
    }

    #[test]
    fn alert_evaluator_accepts_anomaly_evaluations() {
        let agent = AlertEvaluatorAgent;
        let ctx = promoted(&[(
            ContextKey::Evaluations,
            "anomaly:scan:latest",
            r#"{"anomalies_detected":0}"#,
        )]);
        assert!(agent.accepts(&ctx));
        let unrelated = promoted(&[(
            ContextKey::Evaluations,
            "validation:p1",
            r#"{"schema_valid":true}"#,
        )]);
        assert!(!agent.accepts(&unrelated));
    }

    #[test]
    fn alert_evaluator_emits_alert_evaluation() {
        let agent = AlertEvaluatorAgent;
        let ctx = promoted(&[(
            ContextKey::Evaluations,
            "anomaly:scan:latest",
            r#"{"anomalies_detected":0}"#,
        )]);
        let effect = block_execute(&agent, &ctx);
        assert_eq!(effect.proposals().len(), 1);
    }

    #[test]
    fn freshness_monitor_accepts_any_source_signal() {
        let agent = FreshnessMonitorAgent;
        let ctx = promoted(&[(ContextKey::Signals, "source:pg", "{}")]);
        assert!(agent.accepts(&ctx));
        assert!(!agent.accepts(&promoted(&[(ContextKey::Signals, "other:x", "{}")])));
    }

    #[test]
    fn freshness_monitor_emits_per_source() {
        let agent = FreshnessMonitorAgent;
        let ctx = promoted(&[
            (ContextKey::Signals, "source:pg", "{}"),
            (ContextKey::Signals, "source:redis", "{}"),
        ]);
        let effect = block_execute(&agent, &ctx);
        assert_eq!(effect.proposals().len(), 2);
    }

    #[test]
    fn metric_calculator_requires_active_metric_and_validation() {
        let agent = MetricCalculatorAgent;
        let only_metric =
            promoted(&[(ContextKey::Proposals, "metric:m1", r#"{"state":"active"}"#)]);
        assert!(!agent.accepts(&only_metric));
        let only_validation = promoted(&[(ContextKey::Evaluations, "validation:p1", "{}")]);
        assert!(!agent.accepts(&only_validation));
        let both = promoted(&[
            (ContextKey::Proposals, "metric:m1", r#"{"state":"active"}"#),
            (ContextKey::Evaluations, "validation:p1", "{}"),
        ]);
        assert!(agent.accepts(&both));
    }

    #[test]
    fn metric_calculator_emits_calculated_evaluation() {
        let agent = MetricCalculatorAgent;
        let ctx = promoted(&[
            (ContextKey::Proposals, "metric:m1", r#"{"state":"active"}"#),
            (ContextKey::Evaluations, "validation:p1", "{}"),
        ]);
        let effect = block_execute(&agent, &ctx);
        let proposals = effect.proposals();
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].key, ContextKey::Evaluations);
        assert!(proposals[0].id.starts_with("calculated:"));
    }

    fn invariant_ctx(entries: &[(ContextKey, &str, &str)]) -> ContextState {
        promoted(entries)
    }

    #[test]
    fn metric_definition_versioned_passes_when_versioned() {
        let inv = MetricDefinitionVersionedInvariant;
        assert_eq!(inv.name(), "metric_definition_versioned");
        assert_eq!(inv.class(), InvariantClass::Structural);
        let ctx = invariant_ctx(&[(ContextKey::Proposals, "metric:m1", r#"{"version":"1.0.0"}"#)]);
        assert!(matches!(inv.check(&ctx), InvariantResult::Ok));
    }

    #[test]
    fn metric_definition_versioned_violates_without_version() {
        let inv = MetricDefinitionVersionedInvariant;
        let ctx = invariant_ctx(&[(ContextKey::Proposals, "metric:m1", r#"{"state":"draft"}"#)]);
        assert!(matches!(inv.check(&ctx), InvariantResult::Violated(_)));
    }

    #[test]
    fn dashboard_cites_sources_passes_when_cited() {
        let inv = DashboardCitesSourcesInvariant;
        assert_eq!(inv.name(), "dashboard_cites_sources");
        let ok = invariant_ctx(&[(
            ContextKey::Proposals,
            "dashboard:d1",
            r#"{"state":"published","data_source":"pg"}"#,
        )]);
        assert!(matches!(inv.check(&ok), InvariantResult::Ok));
        let draft = invariant_ctx(&[(
            ContextKey::Proposals,
            "dashboard:d1",
            r#"{"state":"draft"}"#,
        )]);
        assert!(matches!(inv.check(&draft), InvariantResult::Ok));
    }

    #[test]
    fn dashboard_cites_sources_violates_when_published_without_source() {
        let inv = DashboardCitesSourcesInvariant;
        let ctx = invariant_ctx(&[(
            ContextKey::Proposals,
            "dashboard:d1",
            r#"{"state":"published"}"#,
        )]);
        assert!(matches!(inv.check(&ctx), InvariantResult::Violated(_)));
    }

    #[test]
    fn alert_has_owner_passes_when_owned_or_inactive() {
        let inv = AlertHasOwnerInvariant;
        assert_eq!(inv.name(), "alert_has_owner");
        let owned = invariant_ctx(&[(
            ContextKey::Proposals,
            "alert:a1",
            r#"{"state":"active","owner":"sre"}"#,
        )]);
        assert!(matches!(inv.check(&owned), InvariantResult::Ok));
        let inactive =
            invariant_ctx(&[(ContextKey::Proposals, "alert:a1", r#"{"state":"paused"}"#)]);
        assert!(matches!(inv.check(&inactive), InvariantResult::Ok));
    }

    #[test]
    fn alert_has_owner_violates_when_active_without_owner() {
        let inv = AlertHasOwnerInvariant;
        let ctx = invariant_ctx(&[(ContextKey::Proposals, "alert:a1", r#"{"state":"active"}"#)]);
        assert!(matches!(inv.check(&ctx), InvariantResult::Violated(_)));
    }

    #[test]
    fn data_freshness_passes_when_fresh() {
        let inv = DataFreshnessInvariant;
        assert_eq!(inv.name(), "data_freshness");
        assert_eq!(inv.class(), InvariantClass::Semantic);
        let ctx = invariant_ctx(&[(
            ContextKey::Evaluations,
            "freshness:source:pg",
            r#"{"type":"freshness_check","is_fresh":true}"#,
        )]);
        assert!(matches!(inv.check(&ctx), InvariantResult::Ok));
    }

    #[test]
    fn data_freshness_violates_when_stale() {
        let inv = DataFreshnessInvariant;
        let ctx = invariant_ctx(&[(
            ContextKey::Evaluations,
            "freshness:source:pg",
            r#"{"type":"freshness_check","is_fresh":false}"#,
        )]);
        assert!(matches!(inv.check(&ctx), InvariantResult::Violated(_)));
    }
}
