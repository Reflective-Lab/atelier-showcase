//! Partnerships pack — Vendor sourcing, evaluation, contracting.
//!
//! Fact prefixes: `partner:`, `supplier:`, `p_agreement:`, `vendor_assessment:`,
//! `integration:`, `diligence:`, `relationship:`, `contract_renewal:`

use crate::pack::{AgentMeta, ContextKey, InvariantClass, InvariantMeta};

pub const AGENTS: &[AgentMeta] = &[
    AgentMeta {
        name: "partner_sourcer",
        dependencies: &[ContextKey::Seeds],
        fact_prefix: "partner:",
        target_key: ContextKey::Proposals,
        description: "Identifies partner prospects",
    },
    AgentMeta {
        name: "vendor_assessor",
        dependencies: &[ContextKey::Seeds],
        fact_prefix: "vendor_assessment:",
        target_key: ContextKey::Proposals,
        description: "Security/compliance assessments",
    },
    AgentMeta {
        name: "contract_negotiator",
        dependencies: &[ContextKey::Proposals],
        fact_prefix: "p_agreement:",
        target_key: ContextKey::Evaluations,
        description: "Negotiation support",
    },
    AgentMeta {
        name: "relationship_manager",
        dependencies: &[ContextKey::Proposals],
        fact_prefix: "relationship:",
        target_key: ContextKey::Evaluations,
        description: "Health monitoring",
    },
    AgentMeta {
        name: "performance_reviewer",
        dependencies: &[ContextKey::Evaluations],
        fact_prefix: "relationship:",
        target_key: ContextKey::Evaluations,
        description: "Annual reviews",
    },
    AgentMeta {
        name: "integration_coordinator",
        dependencies: &[ContextKey::Proposals],
        fact_prefix: "integration:",
        target_key: ContextKey::Proposals,
        description: "Technical coordination",
    },
    AgentMeta {
        name: "due_diligence_coordinator",
        dependencies: &[ContextKey::Seeds],
        fact_prefix: "diligence:",
        target_key: ContextKey::Proposals,
        description: "Due diligence checklist",
    },
    AgentMeta {
        name: "partnership_renewal_tracker",
        dependencies: &[ContextKey::Proposals],
        fact_prefix: "contract_renewal:",
        target_key: ContextKey::Signals,
        description: "Renewal tracking",
    },
    AgentMeta {
        name: "risk_monitor",
        dependencies: &[ContextKey::Signals],
        fact_prefix: "relationship:",
        target_key: ContextKey::Evaluations,
        description: "External risk detection",
    },
    AgentMeta {
        name: "offboarding_coordinator",
        dependencies: &[ContextKey::Proposals],
        fact_prefix: "partner:",
        target_key: ContextKey::Proposals,
        description: "Exit planning",
    },
];

pub const INVARIANTS: &[InvariantMeta] = &[
    InvariantMeta {
        name: "vendor_has_assessment",
        class: InvariantClass::Structural,
        description: "Vendors must have assessment",
    },
    InvariantMeta {
        name: "partner_has_agreement",
        class: InvariantClass::Structural,
        description: "Partners must have agreement",
    },
    InvariantMeta {
        name: "integration_has_owner",
        class: InvariantClass::Structural,
        description: "Integrations must have owner",
    },
    InvariantMeta {
        name: "high_risk_vendor_requires_approval",
        class: InvariantClass::Semantic,
        description: "High-risk vendors require approval",
    },
];

pub const PROFILE: crate::pack::PackProfile = crate::pack::PackProfile {
    entities: &["partner", "supplier", "vendor", "integration", "assessment"],
    required_capabilities: &["web"],
    uses_llm: false,
    requires_hitl: true,
    handles_irreversible: false,
    keywords: &[
        "vendor",
        "partner",
        "supplier",
        "sourcing",
        "procurement",
        "assessment",
        "diligence",
    ],
};

/// Parses vendor candidates from seed data into individual vendor signals.
///
/// Maps to `vendor_assessor` metadata. This is reusable pack behavior; apps
/// provide the RFP seed and Converge handles promotion.
pub struct VendorDataSuggestor;

#[async_trait::async_trait]
impl converge_pack::Suggestor for VendorDataSuggestor {
    fn name(&self) -> &'static str {
        "vendor_data"
    }

    fn dependencies(&self) -> &[converge_pack::ContextKey] {
        &[converge_pack::ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_pack::Context) -> bool {
        ctx.has(converge_pack::ContextKey::Seeds) && !ctx.has(converge_pack::ContextKey::Signals)
    }

    async fn execute(&self, ctx: &dyn converge_pack::Context) -> converge_pack::AgentEffect {
        let seeds = ctx.get(converge_pack::ContextKey::Seeds);
        let Some(seed) = seeds.first() else {
            return converge_pack::AgentEffect::empty();
        };

        let json: serde_json::Value = serde_json::from_str(seed.content()).unwrap_or_default();
        let vendors = json.get("vendors").cloned().unwrap_or_default();

        let facts: Vec<converge_pack::ProposedFact> = vendors
            .as_array()
            .map_or(&[] as &[serde_json::Value], |v| v)
            .iter()
            .map(|vendor| {
                let id = vendor
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("?");
                converge_pack::ProposedFact::new(
                    converge_pack::ContextKey::Signals,
                    format!("vendor:{id}"),
                    vendor.to_string(),
                    self.name(),
                )
                .with_confidence(1.0)
            })
            .collect();

        converge_pack::AgentEffect::with_proposals(facts)
    }
}

/// Scores vendors by price tier.
///
/// Maps to `contract_negotiator` for the price dimension.
pub struct VendorPriceEvaluatorSuggestor;

#[async_trait::async_trait]
impl converge_pack::Suggestor for VendorPriceEvaluatorSuggestor {
    fn name(&self) -> &'static str {
        "price_evaluator"
    }

    fn dependencies(&self) -> &[converge_pack::ContextKey] {
        &[converge_pack::ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_pack::Context) -> bool {
        ctx.has(converge_pack::ContextKey::Signals)
            && !ctx.has(converge_pack::ContextKey::Evaluations)
    }

    async fn execute(&self, ctx: &dyn converge_pack::Context) -> converge_pack::AgentEffect {
        evaluate_vendors(ctx, "price", |vendor| {
            let price = vendor
                .get("price")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(999_999.0);
            if price < 10_000.0 {
                1.0
            } else if price < 25_000.0 {
                0.7
            } else if price < 50_000.0 {
                0.4
            } else {
                0.1
            }
        })
    }
}

/// Scores vendors by compliance status.
///
/// Maps to `vendor_assessor` for the compliance dimension.
pub struct VendorComplianceEvaluatorSuggestor;

#[async_trait::async_trait]
impl converge_pack::Suggestor for VendorComplianceEvaluatorSuggestor {
    fn name(&self) -> &'static str {
        "compliance_evaluator"
    }

    fn dependencies(&self) -> &[converge_pack::ContextKey] {
        &[converge_pack::ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_pack::Context) -> bool {
        ctx.has(converge_pack::ContextKey::Signals)
            && !ctx.has(converge_pack::ContextKey::Evaluations)
    }

    async fn execute(&self, ctx: &dyn converge_pack::Context) -> converge_pack::AgentEffect {
        evaluate_vendors(ctx, "compliance", |vendor| {
            if vendor
                .get("compliant")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
            {
                1.0
            } else {
                0.0
            }
        })
    }
}

/// Scores vendors by years in business as a simple risk proxy.
///
/// Maps to `risk_monitor`.
pub struct VendorRiskEvaluatorSuggestor;

#[async_trait::async_trait]
impl converge_pack::Suggestor for VendorRiskEvaluatorSuggestor {
    fn name(&self) -> &'static str {
        "risk_evaluator"
    }

    fn dependencies(&self) -> &[converge_pack::ContextKey] {
        &[converge_pack::ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_pack::Context) -> bool {
        ctx.has(converge_pack::ContextKey::Signals)
            && !ctx.has(converge_pack::ContextKey::Evaluations)
    }

    async fn execute(&self, ctx: &dyn converge_pack::Context) -> converge_pack::AgentEffect {
        evaluate_vendors(ctx, "risk", |vendor| {
            let years = vendor
                .get("years_in_business")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            if years > 10 {
                1.0
            } else if years > 5 {
                0.7
            } else if years > 2 {
                0.4
            } else {
                0.1
            }
        })
    }
}

/// Scores vendors by delivery timeline.
///
/// Maps to `performance_reviewer` for the timeline dimension.
pub struct VendorTimelineEvaluatorSuggestor;

#[async_trait::async_trait]
impl converge_pack::Suggestor for VendorTimelineEvaluatorSuggestor {
    fn name(&self) -> &'static str {
        "timeline_evaluator"
    }

    fn dependencies(&self) -> &[converge_pack::ContextKey] {
        &[converge_pack::ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_pack::Context) -> bool {
        ctx.has(converge_pack::ContextKey::Signals)
            && !ctx.has(converge_pack::ContextKey::Evaluations)
    }

    async fn execute(&self, ctx: &dyn converge_pack::Context) -> converge_pack::AgentEffect {
        evaluate_vendors(ctx, "timeline", |vendor| {
            let weeks = vendor
                .get("delivery_weeks")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(52);
            if weeks <= 4 {
                1.0
            } else if weeks <= 8 {
                0.8
            } else if weeks <= 12 {
                0.5
            } else {
                0.2
            }
        })
    }
}

/// Aggregates all vendor evaluation scores and emits a ranked recommendation.
///
/// Maps to `partner_sourcer`. High-risk approval remains a governance concern
/// enforced by Converge gates or a downstream HITL policy.
pub struct VendorConsensusSuggestor;

#[async_trait::async_trait]
impl converge_pack::Suggestor for VendorConsensusSuggestor {
    fn name(&self) -> &'static str {
        "consensus"
    }

    fn dependencies(&self) -> &[converge_pack::ContextKey] {
        &[converge_pack::ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn converge_pack::Context) -> bool {
        ctx.has(converge_pack::ContextKey::Evaluations)
            && !ctx.has(converge_pack::ContextKey::Proposals)
    }

    async fn execute(&self, ctx: &dyn converge_pack::Context) -> converge_pack::AgentEffect {
        let evaluations = ctx.get(converge_pack::ContextKey::Evaluations);
        let mut scores: std::collections::HashMap<String, (f64, u32)> =
            std::collections::HashMap::new();

        for eval in evaluations {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(eval.content()) {
                let id = json
                    .get("vendor_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("?");
                let score = json
                    .get("score")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0);
                let entry = scores.entry(id.to_string()).or_insert((0.0, 0));
                entry.0 += score;
                entry.1 += 1;
            }
        }

        let mut ranked: Vec<(String, f64)> = scores
            .into_iter()
            .map(|(id, (total, count))| (id, total / f64::from(count)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let proposals: Vec<converge_pack::ProposedFact> = ranked
            .iter()
            .enumerate()
            .map(|(i, (vendor_id, score))| {
                converge_pack::ProposedFact::new(
                    converge_pack::ContextKey::Proposals,
                    format!("recommendation:{}", i + 1),
                    serde_json::json!({
                        "vendor_id": vendor_id,
                        "rank": i + 1,
                        "score": score,
                        "recommendation": if i == 0 { "recommended" } else { "alternative" }
                    })
                    .to_string(),
                    self.name(),
                )
                .with_confidence(if i == 0 { 0.85 } else { 0.6 })
            })
            .collect();

        converge_pack::AgentEffect::with_proposals(proposals)
    }
}

fn evaluate_vendors<F>(
    ctx: &dyn converge_pack::Context,
    criterion: &str,
    scorer: F,
) -> converge_pack::AgentEffect
where
    F: Fn(&serde_json::Value) -> f64,
{
    let signals = ctx.get(converge_pack::ContextKey::Signals);
    let evaluations: Vec<converge_pack::ProposedFact> = signals
        .iter()
        .filter_map(|signal| {
            let vendor: serde_json::Value = serde_json::from_str(signal.content()).ok()?;
            let id = vendor.get("id").and_then(serde_json::Value::as_str)?;
            let score = scorer(&vendor);
            Some(
                converge_pack::ProposedFact::new(
                    converge_pack::ContextKey::Evaluations,
                    format!("{criterion}:{id}"),
                    serde_json::json!({
                        "vendor_id": id,
                        "criterion": criterion,
                        "score": score,
                    })
                    .to_string(),
                    format!("{criterion}_evaluator"),
                )
                .with_confidence(1.0),
            )
        })
        .collect();

    converge_pack::AgentEffect::with_proposals(evaluations)
}

#[cfg(test)]
mod tests {
    use super::{
        AGENTS, INVARIANTS, PROFILE, VendorComplianceEvaluatorSuggestor, VendorConsensusSuggestor,
        VendorDataSuggestor, VendorPriceEvaluatorSuggestor, VendorRiskEvaluatorSuggestor,
        VendorTimelineEvaluatorSuggestor,
    };
    use converge_kernel::{ContextKey, ContextState, Engine};

    fn rfp(vendors: serde_json::Value) -> String {
        serde_json::json!({ "vendors": vendors }).to_string()
    }

    fn score_for(facts: &[converge_pack::ContextFact], id: &str) -> f64 {
        let fact = facts
            .iter()
            .find(|f| f.id().as_str() == id)
            .unwrap_or_else(|| panic!("missing evaluation {id}"));
        let json: serde_json::Value =
            serde_json::from_str(fact.content()).expect("evaluation json");
        json.get("score")
            .and_then(serde_json::Value::as_f64)
            .expect("score field")
    }

    #[test]
    fn metadata_constants_are_populated() {
        assert!(!AGENTS.is_empty());
        assert!(!INVARIANTS.is_empty());
        assert!(!PROFILE.entities.is_empty());
        assert!(PROFILE.requires_hitl);
        assert!(!PROFILE.uses_llm);
    }

    #[tokio::test]
    async fn vendor_data_parses_rfp_seed() {
        let mut engine = Engine::new();
        engine.register_suggestor(VendorDataSuggestor);

        let mut ctx = ContextState::new();
        let _ = ctx.add_input(
            ContextKey::Seeds,
            "rfp",
            rfp(serde_json::json!([
                { "id": "alpha", "price": 5_000.0 },
                { "id": "bravo", "price": 50_000.0 },
            ])),
        );

        let result = engine.run(ctx).await.expect("converge");
        let signals = result.context.get(ContextKey::Signals);
        assert_eq!(signals.len(), 2);
        assert!(signals.iter().any(|f| f.id().as_str() == "vendor:alpha"));
        assert!(signals.iter().any(|f| f.id().as_str() == "vendor:bravo"));
    }

    #[tokio::test]
    async fn vendor_data_with_no_vendors_emits_nothing() {
        let mut engine = Engine::new();
        engine.register_suggestor(VendorDataSuggestor);

        let mut ctx = ContextState::new();
        let _ = ctx.add_input(ContextKey::Seeds, "rfp", "{}".to_string());

        let result = engine.run(ctx).await.expect("converge");
        assert!(result.context.get(ContextKey::Signals).is_empty());
    }

    #[tokio::test]
    async fn price_evaluator_buckets_match_thresholds() {
        let mut engine = Engine::new();
        engine.register_suggestor(VendorDataSuggestor);
        engine.register_suggestor(VendorPriceEvaluatorSuggestor);

        let mut ctx = ContextState::new();
        let _ = ctx.add_input(
            ContextKey::Seeds,
            "rfp",
            rfp(serde_json::json!([
                { "id": "cheap",  "price": 5_000.0 },
                { "id": "mid",    "price": 20_000.0 },
                { "id": "high",   "price": 40_000.0 },
                { "id": "lux",    "price": 100_000.0 },
            ])),
        );

        let result = engine.run(ctx).await.expect("converge");
        let evals = result.context.get(ContextKey::Evaluations);
        assert!((score_for(evals, "price:cheap") - 1.0).abs() < 1e-9);
        assert!((score_for(evals, "price:mid") - 0.7).abs() < 1e-9);
        assert!((score_for(evals, "price:high") - 0.4).abs() < 1e-9);
        assert!((score_for(evals, "price:lux") - 0.1).abs() < 1e-9);
    }

    #[tokio::test]
    async fn risk_evaluator_buckets_match_thresholds() {
        let mut engine = Engine::new();
        engine.register_suggestor(VendorDataSuggestor);
        engine.register_suggestor(VendorRiskEvaluatorSuggestor);

        let mut ctx = ContextState::new();
        let _ = ctx.add_input(
            ContextKey::Seeds,
            "rfp",
            rfp(serde_json::json!([
                { "id": "established", "years_in_business": 15 },
                { "id": "mature",      "years_in_business": 7 },
                { "id": "growing",     "years_in_business": 3 },
                { "id": "startup",     "years_in_business": 1 },
            ])),
        );

        let result = engine.run(ctx).await.expect("converge");
        let evals = result.context.get(ContextKey::Evaluations);
        assert!((score_for(evals, "risk:established") - 1.0).abs() < 1e-9);
        assert!((score_for(evals, "risk:mature") - 0.7).abs() < 1e-9);
        assert!((score_for(evals, "risk:growing") - 0.4).abs() < 1e-9);
        assert!((score_for(evals, "risk:startup") - 0.1).abs() < 1e-9);
    }

    #[tokio::test]
    async fn timeline_evaluator_buckets_match_thresholds() {
        let mut engine = Engine::new();
        engine.register_suggestor(VendorDataSuggestor);
        engine.register_suggestor(VendorTimelineEvaluatorSuggestor);

        let mut ctx = ContextState::new();
        let _ = ctx.add_input(
            ContextKey::Seeds,
            "rfp",
            rfp(serde_json::json!([
                { "id": "fast",    "delivery_weeks": 3 },
                { "id": "med",     "delivery_weeks": 8 },
                { "id": "slow",    "delivery_weeks": 12 },
                { "id": "glacial", "delivery_weeks": 26 },
            ])),
        );

        let result = engine.run(ctx).await.expect("converge");
        let evals = result.context.get(ContextKey::Evaluations);
        assert!((score_for(evals, "timeline:fast") - 1.0).abs() < 1e-9);
        assert!((score_for(evals, "timeline:med") - 0.8).abs() < 1e-9);
        assert!((score_for(evals, "timeline:slow") - 0.5).abs() < 1e-9);
        assert!((score_for(evals, "timeline:glacial") - 0.2).abs() < 1e-9);
    }

    #[tokio::test]
    async fn compliance_evaluator_pass_fail() {
        let mut engine = Engine::new();
        engine.register_suggestor(VendorDataSuggestor);
        engine.register_suggestor(VendorComplianceEvaluatorSuggestor);

        let mut ctx = ContextState::new();
        let _ = ctx.add_input(
            ContextKey::Seeds,
            "rfp",
            rfp(serde_json::json!([
                { "id": "good", "compliant": true },
                { "id": "bad",  "compliant": false },
            ])),
        );

        let result = engine.run(ctx).await.expect("converge");
        let evals = result.context.get(ContextKey::Evaluations);
        assert!((score_for(evals, "compliance:good") - 1.0).abs() < 1e-9);
        assert!((score_for(evals, "compliance:bad") - 0.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn full_pipeline_ranks_best_vendor_first() {
        let mut engine = Engine::new();
        engine.register_suggestor(VendorDataSuggestor);
        engine.register_suggestor(VendorPriceEvaluatorSuggestor);
        engine.register_suggestor(VendorComplianceEvaluatorSuggestor);
        engine.register_suggestor(VendorRiskEvaluatorSuggestor);
        engine.register_suggestor(VendorTimelineEvaluatorSuggestor);
        engine.register_suggestor(VendorConsensusSuggestor);

        let mut ctx = ContextState::new();
        let _ = ctx.add_input(
            ContextKey::Seeds,
            "rfp",
            rfp(serde_json::json!([
                {
                    "id": "winner",
                    "price": 5_000.0,
                    "compliant": true,
                    "years_in_business": 15,
                    "delivery_weeks": 3
                },
                {
                    "id": "loser",
                    "price": 100_000.0,
                    "compliant": false,
                    "years_in_business": 1,
                    "delivery_weeks": 30
                },
            ])),
        );

        let result = engine.run(ctx).await.expect("converge");
        let proposals = result.context.get(ContextKey::Proposals);
        let rec1 = proposals
            .iter()
            .find(|f| f.id().as_str() == "recommendation:1")
            .expect("recommendation:1");
        let json: serde_json::Value =
            serde_json::from_str(rec1.content()).expect("recommendation json");
        assert_eq!(
            json.get("vendor_id").and_then(serde_json::Value::as_str),
            Some("winner")
        );
        assert_eq!(
            json.get("recommendation").and_then(serde_json::Value::as_str),
            Some("recommended")
        );
        assert_eq!(json.get("rank").and_then(serde_json::Value::as_u64), Some(1));
    }
}
