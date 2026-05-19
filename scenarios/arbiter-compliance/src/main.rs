//! Atelier showcase: rule-based compliance gate (GDPR / SOC2 / HIPAA).
//!
//! ## Where this sits relative to the Cedar SMT analysis
//!
//! `scenario-cedar-smt-analysis` formally verifies authorization
//! policies via cedar-policy-symcc + CVC5: it answers "does this
//! policy permit ANY request matching claim X?" with UNSAT (proof)
//! or SAT (counterexample). That is the heavyweight, model-theoretic
//! path.
//!
//! `ComplianceGateSuggestor` is the **rule-based** sibling: a pure
//! Rust gate that checks every proposal carrying a
//! `ComplianceDocumentPayload` against a configured set of
//! `ComplianceRule`s and emits a typed `ComplianceConstraintPayload`
//! per violation. No SMT solver involved; no symbolic compilation.
//! The cost model is per-fact-per-rule O(1), no NP-hardness.
//!
//! Use the SMT path when the question is universally quantified
//! ("for all requests, does the property hold?"). Use the
//! rule-based gate when the question is per-document ("does this
//! specific document violate framework rule X?"). They are
//! complementary — production governance often runs both.
//!
//! ## The demo
//!
//! Three compliance frameworks, four rules:
//!
//! 1. **GDPR data residency** — `data_region` must not be in a
//!    non-EU region when GDPR-tagged.
//! 2. **GDPR cross-border transfer** — `cross_border_transfer`
//!    must not be `unrestricted`.
//! 3. **SOC2 retention max** — `retention_days` must not exceed
//!    2555 days (≈ 7 years, the SOC2 audit horizon).
//! 4. **HIPAA PHI tag** — `contains_phi` must not be present
//!    (PHI documents need a separate handling path).
//!
//! Six candidate documents, mix of clean and dirty. The gate
//! produces typed verdicts for exactly the documents that
//! violate at least one rule. Self-validation asserts the
//! expected violations fired, no false positives, no false
//! negatives.

use arbiter::{
    ComplianceCondition, ComplianceConstraintPayload, ComplianceDocumentPayload,
    ComplianceGateSuggestor, ComplianceRule,
};
use converge_kernel::{Budget as ConvergeBudget, ContextState, Engine, ProposedFact};
use converge_pack::ContextKey;
use serde_json::Value;

struct Document {
    id: &'static str,
    description: &'static str,
    fields: Vec<(&'static str, Value)>,
    expected_violations: &'static [&'static str], // rule ids
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();
    print_rules();

    let documents = build_documents();
    print_documents(&documents);

    let mut engine = Engine::with_budget(ConvergeBudget {
        max_cycles: 8,
        max_facts: 128,
    });
    engine.register_suggestor(ComplianceGateSuggestor::new(
        ContextKey::Strategies,
        build_rules(),
    ));

    let mut ctx = ContextState::new();
    for doc in &documents {
        let mut field_map = serde_json::Map::new();
        for (k, v) in &doc.fields {
            field_map.insert((*k).to_string(), v.clone());
        }
        ctx.add_proposal(ProposedFact::new(
            ContextKey::Strategies,
            doc.id,
            ComplianceDocumentPayload { fields: field_map },
            "atelier-arbiter-compliance",
        ))
        .map_err(|e| format!("seed {}: {e:?}", doc.id))?;
    }

    let result = engine.run(ctx).await?;
    print_outcome(&result, &documents)
}

fn build_rules() -> Vec<ComplianceRule> {
    vec![
        ComplianceRule {
            id: "gdpr-data-residency".into(),
            framework: "GDPR".into(),
            field: "data_region".into(),
            condition: ComplianceCondition::MustNotContain(vec![
                "us-east".into(),
                "us-west".into(),
                "ap-northeast".into(),
            ]),
        },
        ComplianceRule {
            id: "gdpr-cross-border".into(),
            framework: "GDPR".into(),
            field: "cross_border_transfer".into(),
            condition: ComplianceCondition::MustNotContain(vec!["unrestricted".into()]),
        },
        ComplianceRule {
            id: "soc2-retention-max".into(),
            framework: "SOC2".into(),
            field: "retention_days".into(),
            condition: ComplianceCondition::MaxValue(2555.0),
        },
        ComplianceRule {
            id: "hipaa-phi-tag".into(),
            framework: "HIPAA".into(),
            field: "contains_phi".into(),
            condition: ComplianceCondition::FieldMustNotExist,
        },
    ]
}

fn build_documents() -> Vec<Document> {
    vec![
        Document {
            id: "doc-clean-eu",
            description: "EU vendor, modest retention, no PHI, regional transfer only.",
            fields: vec![
                ("data_region", json_str("eu-west-1")),
                ("cross_border_transfer", json_str("regional-only")),
                ("retention_days", json_num(365.0)),
            ],
            expected_violations: &[],
        },
        Document {
            id: "doc-gdpr-residency-us-east",
            description: "GDPR-tagged data stored in us-east — residency violation.",
            fields: vec![
                ("data_region", json_str("us-east-1")),
                ("cross_border_transfer", json_str("regional-only")),
                ("retention_days", json_num(365.0)),
            ],
            expected_violations: &["gdpr-data-residency"],
        },
        Document {
            id: "doc-soc2-retention",
            description: "Retention beyond SOC2 7-year horizon.",
            fields: vec![
                ("data_region", json_str("eu-west-1")),
                ("cross_border_transfer", json_str("regional-only")),
                ("retention_days", json_num(3000.0)),
            ],
            expected_violations: &["soc2-retention-max"],
        },
        Document {
            id: "doc-hipaa-phi",
            description: "HIPAA PHI flag present — needs dedicated handling path.",
            fields: vec![
                ("data_region", json_str("eu-west-1")),
                ("cross_border_transfer", json_str("regional-only")),
                ("retention_days", json_num(365.0)),
                ("contains_phi", Value::Bool(true)),
            ],
            expected_violations: &["hipaa-phi-tag"],
        },
        Document {
            id: "doc-gdpr-cross-border",
            description: "GDPR data with unrestricted cross-border transfer.",
            fields: vec![
                ("data_region", json_str("eu-west-1")),
                ("cross_border_transfer", json_str("unrestricted")),
                ("retention_days", json_num(365.0)),
            ],
            expected_violations: &["gdpr-cross-border"],
        },
        Document {
            id: "doc-triple-violation",
            description: "Multiple violations in a single document: residency + retention + PHI.",
            fields: vec![
                ("data_region", json_str("ap-northeast-1")),
                ("cross_border_transfer", json_str("regional-only")),
                ("retention_days", json_num(4000.0)),
                ("contains_phi", Value::Bool(true)),
            ],
            expected_violations: &["gdpr-data-residency", "soc2-retention-max", "hipaa-phi-tag"],
        },
    ]
}

fn json_str(s: &str) -> Value {
    Value::String(s.to_string())
}

fn json_num(n: f64) -> Value {
    serde_json::Number::from_f64(n).map_or(Value::Null, Value::Number)
}

fn print_banner() {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║  Arbiter Compliance — rule-based GDPR / SOC2 / HIPAA gate           ║");
    println!("║  atelier-showcase · converge 3.9.1 · arbiter 2.0.1                   ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_rules() {
    println!("Compliance rules");
    println!("────────────────");
    println!("  [GDPR]  gdpr-data-residency  data_region must not be in non-EU regions");
    println!("  [GDPR]  gdpr-cross-border    cross_border_transfer must not be unrestricted");
    println!("  [SOC2]  soc2-retention-max   retention_days ≤ 2555 (7-year horizon)");
    println!("  [HIPAA] hipaa-phi-tag        contains_phi field must NOT be present");
    println!();
}

fn print_documents(docs: &[Document]) {
    println!("Candidate documents");
    println!("───────────────────");
    for doc in docs {
        println!("  {}: {}", doc.id, doc.description);
        for (k, v) in &doc.fields {
            println!("      {k} = {v}");
        }
        if doc.expected_violations.is_empty() {
            println!("      (expected: clean)");
        } else {
            println!(
                "      (expected violations: {})",
                doc.expected_violations.join(", ")
            );
        }
    }
    println!();
}

fn print_outcome(
    result: &converge_kernel::ConvergeResult,
    docs: &[Document],
) -> Result<(), Box<dyn std::error::Error>> {
    let constraints = result.context.get(ContextKey::Constraints);

    println!("Convergence");
    println!("───────────");
    println!(
        "  converged: {} (cycles: {}, stop: {:?})",
        result.converged, result.cycles, result.stop_reason
    );
    println!(
        "  facts:     Strategies={}  Constraints={}",
        result.context.get(ContextKey::Strategies).len(),
        constraints.len(),
    );
    println!();

    // Collect typed verdicts: (doc_id, rule_id, framework)
    let mut emitted: Vec<(String, String, String)> = Vec::new();
    println!("Typed verdicts");
    println!("──────────────");
    for fact in constraints.iter() {
        let Ok(payload) = fact.require_payload::<ComplianceConstraintPayload>() else {
            continue;
        };
        println!(
            "  [{:<5}] doc={:<28} rule={:<22} field={:<25} → action: {:?}",
            payload.framework,
            payload.fact_id.as_str(),
            payload.rule_id,
            payload.field,
            payload.action,
        );
        emitted.push((
            payload.fact_id.as_str().to_string(),
            payload.rule_id.clone(),
            payload.framework.clone(),
        ));
    }
    println!();

    // Compute expected verdicts from the document specs.
    let mut expected: Vec<(String, String)> = Vec::new();
    for doc in docs {
        for rule in doc.expected_violations {
            expected.push((doc.id.to_string(), (*rule).to_string()));
        }
    }
    expected.sort();

    let mut got: Vec<(String, String)> = emitted
        .iter()
        .map(|(d, r, _)| (d.clone(), r.clone()))
        .collect();
    got.sort();

    println!("Verification");
    println!("────────────");
    println!(
        "  expected violations: {} (across {} dirty docs)",
        expected.len(),
        docs.iter()
            .filter(|d| !d.expected_violations.is_empty())
            .count()
    );
    println!("  emitted verdicts:    {}", got.len());

    let missing: Vec<_> = expected
        .iter()
        .filter(|e| !got.contains(e))
        .cloned()
        .collect();
    let unexpected: Vec<_> = got
        .iter()
        .filter(|g| !expected.contains(g))
        .cloned()
        .collect();

    if missing.is_empty() {
        println!("  ✓ every expected violation produced a typed verdict (no false negatives)");
    } else {
        println!("  ✗ MISSING verdicts (false negatives):");
        for (d, r) in &missing {
            println!("       {d} / {r}");
        }
    }
    if unexpected.is_empty() {
        println!("  ✓ no verdicts emitted for clean fields (no false positives)");
    } else {
        println!("  ✗ UNEXPECTED verdicts (false positives):");
        for (d, r) in &unexpected {
            println!("       {d} / {r}");
        }
    }

    if !missing.is_empty() || !unexpected.is_empty() {
        return Err("ComplianceGate output disagrees with expected violations".into());
    }
    println!();
    println!(
        "✓ ComplianceGateSuggestor emitted typed verdicts for every expected violation,\n  \
         with no false positives. The non-Cedar rule-based path is complete."
    );
    Ok(())
}
