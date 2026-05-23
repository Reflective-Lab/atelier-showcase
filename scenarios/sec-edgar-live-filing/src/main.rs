use std::{sync::Arc, time::Instant};

use anyhow::{Context, Result, ensure};
use arbiter::{
    ComplianceCondition, ComplianceConstraintPayload, ComplianceDocumentPayload,
    ComplianceGateSuggestor, ComplianceRule,
};
use async_trait::async_trait;
use clap::Parser;
use converge_kernel::{
    AgentEffect, Budget, Context as ConvergeContext, ContextState, ConvergeResult, Engine,
    ProposedFact,
};
use converge_pack::{ContextKey, Suggestor};
use embassy_sec_edgar::live::{HeadingExtractOptions, extract_section_headings};
use embassy_sec_edgar::{
    AccessionNumber, Cik, FormType, LiveSecEdgarProvider, SecEdgarRequest, SecFilingPayload,
    SecFilingSuggestor,
};
use serde_json::{Map, Number, Value};

const COMPANY: &str = "Apple Inc.";
const CIK_PADDED: &str = "0000320193";
const ACCESSION: &str = "0000320193-25-000079";
const FORM_TYPE: &str = "10-K";
const FILING_DATE: &str = "2025-10-31";
const PRIMARY_DOCUMENT: &str = "aapl-20250927.htm";
const FILING_URL: &str =
    "https://www.sec.gov/Archives/edgar/data/320193/000032019325000079/aapl-20250927.htm";
const DETAIL_URL: &str =
    "https://www.sec.gov/Archives/edgar/data/320193/0000320193-25-000079-index.htm";
const REVIEW_DOC_ID: &str = "sec-risk-review:apple-2025-10k";
const REVIEW_RULE_ID: &str = "sec-risk-heading-count-review";
const REVIEW_HEADING_THRESHOLD: f64 = 20.0;
const REVIEW_FRAMEWORK: &str = "SEC-10K-RISK-REVIEW";

#[derive(Debug, Parser)]
#[command(about = "Fetch a REAL LIVE SEC EDGAR filing and route it through Arbiter")]
struct Args {
    /// Print the educational walkthrough in addition to the live result.
    #[arg(long)]
    verbose: bool,

    /// Number of extracted Item 1A headings to print.
    #[arg(long, default_value_t = 8)]
    sample_headings: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    print_declaration();
    if args.verbose {
        print_verbose_intro();
    }

    println!("Target filing:");
    println!("  company: {COMPANY}");
    println!("  cik: {CIK_PADDED}");
    println!("  form: {FORM_TYPE}");
    println!("  accession: {ACCESSION}");
    println!("  filing date: {FILING_DATE}");
    println!("  primary document: {PRIMARY_DOCUMENT}");
    println!("  SEC filing detail: {DETAIL_URL}");
    println!("  SEC primary document: {FILING_URL}");
    println!();

    if args.verbose {
        println!("Step 1: seed a typed SEC EDGAR request into Converge.");
        println!(
            "        The request is a SecEdgarRequest fact under ContextKey::Seeds, not an ad hoc URL string."
        );
        println!("Step 2: run SecFilingSuggestor with Embassy's live SEC provider.");
        println!(
            "        The Suggestor owns the Converge boundary; LiveSecEdgarProvider owns the network call to SEC EDGAR."
        );
        println!("Step 3: derive an Arbiter review document from the live SEC fact.");
        println!(
            "        The derived document carries source_fact_id, request_hash, provider, CIK, accession, and source URL."
        );
        println!(
            "Step 4: let Arbiter's ComplianceGateSuggestor decide whether auto-clearance is blocked."
        );
    }

    let request = SecEdgarRequest::Filing {
        cik: Cik::parse(CIK_PADDED)?,
        accession_number: AccessionNumber::parse(ACCESSION)?,
    };
    let started = Instant::now();
    let result = run_converge(request).await.with_context(|| {
        format!("failed to fetch live SEC filing through Converge: {FILING_URL}")
    })?;
    let elapsed = started.elapsed();
    let filing_fact = result
        .context
        .get(ContextKey::Hypotheses)
        .iter()
        .find(|fact| fact.payload::<SecFilingPayload>().is_some())
        .context("Converge run produced no SEC filing hypothesis")?;
    let payload = filing_fact
        .require_payload::<SecFilingPayload>()
        .context("SEC filing hypothesis carried the wrong payload type")?;
    let filing = &payload.filing;
    let review_doc_fact = result
        .context
        .get(ContextKey::Strategies)
        .iter()
        .find(|fact| fact.payload::<ComplianceDocumentPayload>().is_some())
        .context("Converge run produced no SEC risk review strategy document")?;
    let review_doc = review_doc_fact
        .require_payload::<ComplianceDocumentPayload>()
        .context("SEC risk review strategy carried the wrong payload type")?;
    let review_constraint_fact = result
        .context
        .get(ContextKey::Constraints)
        .iter()
        .find(|fact| {
            fact.payload::<ComplianceConstraintPayload>()
                .is_some_and(|constraint| constraint.rule_id == REVIEW_RULE_ID)
        })
        .context("Arbiter produced no SEC risk review constraint")?;
    let review_constraint = review_constraint_fact
        .require_payload::<ComplianceConstraintPayload>()
        .context("SEC risk review constraint carried the wrong payload type")?;

    ensure!(
        filing.cik.as_str() == CIK_PADDED,
        "Converge SEC filing fact returned CIK {}, expected {CIK_PADDED}",
        filing.cik.as_str()
    );
    ensure!(
        filing.accession_number.as_str() == ACCESSION,
        "Converge SEC filing fact returned accession {}, expected {ACCESSION}",
        filing.accession_number.as_str()
    );
    ensure!(
        filing.form_type == FormType::Form10K,
        "Converge SEC filing fact returned form {}, expected {FORM_TYPE}",
        filing.form_type.as_label()
    );
    ensure!(
        review_constraint.fact_id.as_str() == review_doc_fact.id().as_str(),
        "Arbiter constraint references {}, expected review document {}",
        review_constraint.fact_id.as_str(),
        review_doc_fact.id().as_str()
    );
    ensure!(
        field_str(review_doc, "source_fact_id")? == filing_fact.id().as_str(),
        "review document source_fact_id did not preserve SEC fact id"
    );
    ensure!(
        field_str(review_doc, "source_request_hash")? == payload.request_hash,
        "review document source_request_hash did not preserve SEC request hash"
    );
    ensure!(
        field_str(review_doc, "source_vendor")? == payload.vendor,
        "review document source_vendor did not preserve SEC provider"
    );

    println!("Live fetch:");
    println!("  result: success");
    println!("  path: Converge Engine -> SecFilingSuggestor -> LiveSecEdgarProvider -> SEC EDGAR");
    println!("  converged: {}", result.converged);
    println!("  cycles: {}", result.cycles);
    println!("  stop_reason: {:?}", result.stop_reason);
    println!("  fact_id: {}", filing_fact.id());
    println!("  fact_key: {:?}", filing_fact.key());
    println!("  provider: {}", payload.vendor);
    println!("  request_hash: {}", payload.request_hash);
    println!("  elapsed_ms: {}", elapsed.as_millis());
    println!("  provider_latency_ms: {}", payload.latency_ms);
    println!(
        "  execution_producer: {} {}",
        payload.execution_identity.producer.name, payload.execution_identity.producer.version
    );
    println!(
        "  execution_backend: {}",
        payload.execution_identity.backend
    );
    println!("  validated_fact: CIK {CIK_PADDED}, accession {ACCESSION}, form {FORM_TYPE}");
    println!("  official_primary_url: {FILING_URL}");
    println!();

    if args.verbose {
        println!("Step 5: read Item 1A from the typed Filing fact.");
        println!(
            "        The live provider filled Filing.sections[\"1A\"], then the Suggestor carried it into ContextKey::Hypotheses."
        );
    }

    let section = filing
        .sections
        .get("1A")
        .with_context(|| format!("live SEC filing fact did not contain Item 1A: {FILING_URL}"))?;

    println!("Item 1A section:");
    println!("  section_bytes: {}", section.body.len());
    println!(
        "  contains_risk_factors: {}",
        section.body.contains("Risk Factors")
    );
    println!();

    if args.verbose {
        println!(
            "Step 6: extract risk-factor headings through Embassy's calibrated selector chain."
        );
        println!(
            "        A low or implausibly high heading count fails loudly instead of pretending the parse is good."
        );
    }

    let headings = extract_section_headings(&section.body, &HeadingExtractOptions::default())
        .with_context(|| {
            format!("could not extract plausible Item 1A headings from {FILING_URL}")
        })?;

    ensure!(
        field_u64(review_doc, "risk_factor_heading_count")? == headings.len() as u64,
        "review document heading count drifted from Item 1A extractor output"
    );

    println!("Risk-factor heading extraction:");
    println!("  headings: {}", headings.len());
    println!("  source: typed SecFilingPayload fact from live SEC HTML");
    println!("  extension: converge-embassy-sec-edgar SecFilingSuggestor<LiveSecEdgarProvider>");
    println!("  mocked: false");

    let sample_count = args.sample_headings.min(headings.len());
    if sample_count > 0 {
        println!("  sample:");
        for (idx, heading) in headings.iter().take(sample_count).enumerate() {
            println!("    {}. {heading}", idx + 1);
        }
        if headings.len() > sample_count {
            println!("    ... {} more", headings.len() - sample_count);
        }
    }

    println!();
    println!("Downstream Arbiter decision:");
    println!("  result: auto-clearance blocked; manual filing-risk review required");
    println!("  gate: arbiter::ComplianceGateSuggestor");
    println!("  constraint_fact: {}", review_constraint_fact.id());
    println!("  review_doc_fact: {}", review_doc_fact.id());
    println!(
        "  source_sec_fact: {}",
        field_str(review_doc, "source_fact_id")?
    );
    println!(
        "  preserved_request_hash: {}",
        field_str(review_doc, "source_request_hash")?
    );
    println!(
        "  preserved_provider: {}",
        field_str(review_doc, "source_vendor")?
    );
    println!("  rule_id: {}", review_constraint.rule_id);
    println!("  framework: {}", review_constraint.framework);
    println!("  action: {:?}", review_constraint.action);
    println!(
        "  threshold: risk_factor_heading_count <= {:.0}",
        REVIEW_HEADING_THRESHOLD
    );
    println!("  observed_risk_factor_heading_count: {}", headings.len());

    if args.verbose {
        print_verbose_close();
    }

    Ok(())
}

async fn run_converge(request: SecEdgarRequest) -> Result<ConvergeResult> {
    let mut engine = Engine::with_budget(Budget {
        max_cycles: 8,
        max_facts: 16,
    });
    engine.register_suggestor(SecFilingSuggestor::new(Arc::new(
        LiveSecEdgarProvider::new(),
    )));
    engine.register_suggestor(SecRiskReviewDocumentEmitter);
    engine.register_suggestor(ComplianceGateSuggestor::new(
        ContextKey::Strategies,
        sec_review_rules(),
    ));

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        "sec-edgar-request:apple-2025-10k",
        request,
        "atelier-sec-edgar-live-filing",
    ))?;

    Ok(engine.run(ctx).await?)
}

struct SecRiskReviewDocumentEmitter;

#[async_trait]
impl Suggestor for SecRiskReviewDocumentEmitter {
    fn name(&self) -> &'static str {
        "sec-risk-review-document-emitter"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses]
    }

    fn provenance(&self) -> &'static str {
        "atelier-sec-edgar-risk-review"
    }

    fn accepts(&self, ctx: &dyn ConvergeContext) -> bool {
        ctx.get(ContextKey::Hypotheses)
            .iter()
            .any(|fact| fact.payload::<SecFilingPayload>().is_some())
            && !ctx
                .get(ContextKey::Strategies)
                .iter()
                .any(|fact| fact.id().as_str() == REVIEW_DOC_ID)
    }

    async fn execute(&self, ctx: &dyn ConvergeContext) -> AgentEffect {
        let Some(fact) = ctx
            .get(ContextKey::Hypotheses)
            .iter()
            .find(|fact| fact.payload::<SecFilingPayload>().is_some())
        else {
            return AgentEffect::empty();
        };

        let Some(payload) = fact.payload::<SecFilingPayload>() else {
            return AgentEffect::empty();
        };
        let Some(section) = payload.filing.sections.get("1A") else {
            return AgentEffect::empty();
        };
        let Ok(headings) =
            extract_section_headings(&section.body, &HeadingExtractOptions::default())
        else {
            return AgentEffect::empty();
        };

        AgentEffect::with_proposal(ProposedFact::new(
            ContextKey::Strategies,
            REVIEW_DOC_ID,
            ComplianceDocumentPayload {
                fields: review_fields(
                    fact.id().as_str(),
                    payload,
                    section.body.len(),
                    headings.len(),
                ),
            },
            self.provenance(),
        ))
    }
}

fn sec_review_rules() -> Vec<ComplianceRule> {
    vec![ComplianceRule {
        id: REVIEW_RULE_ID.to_string(),
        framework: REVIEW_FRAMEWORK.to_string(),
        field: "risk_factor_heading_count".to_string(),
        condition: ComplianceCondition::MaxValue(REVIEW_HEADING_THRESHOLD),
    }]
}

fn review_fields(
    source_fact_id: &str,
    payload: &SecFilingPayload,
    section_bytes: usize,
    heading_count: usize,
) -> Map<String, Value> {
    let mut fields = Map::new();
    fields.insert("source_fact_id".to_string(), json_str(source_fact_id));
    fields.insert(
        "source_payload_family".to_string(),
        json_str("embassy.sec_edgar.filing"),
    );
    fields.insert(
        "source_request_hash".to_string(),
        json_str(&payload.request_hash),
    );
    fields.insert("source_vendor".to_string(), json_str(&payload.vendor));
    fields.insert(
        "source_cik".to_string(),
        json_str(payload.filing.cik.as_str()),
    );
    fields.insert(
        "source_accession".to_string(),
        json_str(payload.filing.accession_number.as_str()),
    );
    fields.insert(
        "source_form_type".to_string(),
        json_str(payload.filing.form_type.as_label()),
    );
    fields.insert("source_url".to_string(), json_str(FILING_URL));
    fields.insert(
        "risk_factor_heading_count".to_string(),
        json_u64(heading_count as u64),
    );
    fields.insert(
        "item_1a_section_bytes".to_string(),
        json_u64(section_bytes as u64),
    );
    fields.insert(
        "review_threshold".to_string(),
        json_num(REVIEW_HEADING_THRESHOLD),
    );
    fields.insert(
        "review_objective".to_string(),
        json_str("block automatic clearance when risk-factor heading count exceeds threshold"),
    );
    fields
}

fn json_str(value: impl Into<String>) -> Value {
    Value::String(value.into())
}

fn json_num(value: f64) -> Value {
    Number::from_f64(value).map_or(Value::Null, Value::Number)
}

fn json_u64(value: u64) -> Value {
    Value::Number(Number::from(value))
}

fn field_str<'a>(doc: &'a ComplianceDocumentPayload, field: &str) -> Result<&'a str> {
    doc.fields
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("review document missing string field {field}"))
}

fn field_u64(doc: &ComplianceDocumentPayload, field: &str) -> Result<u64> {
    doc.fields
        .get(field)
        .and_then(Value::as_u64)
        .with_context(|| format!("review document missing integer field {field}"))
}

fn print_declaration() {
    println!("Declaration: REAL LIVE");
    println!("This scenario calls official SEC EDGAR over the network.");
    println!("It does not mock any Mosaic extension.");
    println!("It does not use Embassy's deterministic SEC provider.");
    println!("It does not use recorded HTTP fixtures.");
    println!("Converge path: SecEdgarRequest seed -> SecFilingSuggestor -> LiveSecEdgarProvider.");
    println!("Downstream decision: Arbiter ComplianceGateSuggestor over the live SEC fact.");
    println!("External resource: SEC EDGAR primary filing document for Apple Inc. 2025 Form 10-K.");
    println!();
}

fn print_verbose_intro() {
    println!("Educational walkthrough:");
    println!(
        "  This is a deliberately small proof slice. The goal is not a polished product workflow yet."
    );
    println!(
        "  The goal is to prove that atelier can call a live Mosaic extension through Converge and make the trust boundary obvious."
    );
    println!(
        "  Human verification is intentionally simple: open the SEC filing detail URL and compare the CIK, accession, form, and primary document."
    );
    println!();
}

fn print_verbose_close() {
    println!();
    println!("What this proves:");
    println!("  - The example made a live network call to SEC EDGAR.");
    println!("  - The live call was triggered by a Converge seed fact and SecFilingSuggestor.");
    println!("  - The Suggestor used Embassy's provider-shaped LiveSecEdgarProvider.");
    println!("  - Converge promoted a typed SecFilingPayload fact, not just raw HTML.");
    println!("  - A downstream Arbiter gate made a typed review decision from that SEC fact.");
    println!(
        "  - The review document preserved source fact id, request hash, provider, CIK, accession, and source URL."
    );
    println!("  - No stub, mock, fake, or recorded fixture supplied the filing content.");
    println!("  - The output can be checked against an official SEC page in human speed.");
    println!();
    println!("What this does not prove:");
    println!("  - It is not the full three-module v1.1 combinatory scenario.");
    println!("  - It is not yet memory-backed, solver-backed, or a complete investment workflow.");
    println!("  - It is the live-resource anchor that those larger examples can now build from.");
}
