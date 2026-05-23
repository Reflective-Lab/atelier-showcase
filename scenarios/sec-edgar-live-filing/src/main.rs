use std::{sync::Arc, time::Instant};

use anyhow::{Context, Result, ensure};
use arbiter::{ComplianceConstraintPayload, ComplianceDocumentPayload, ComplianceGateSuggestor};
use async_trait::async_trait;
use atelier_domain::sec_risk::{
    HEADING_COUNT_REVIEW_RULE_ID, SEC_RISK_FRAMEWORK, SecRiskPolicyPack,
};
use clap::Parser;
use converge_kernel::{
    AgentEffect, Budget, Context as ConvergeContext, ContextState, ConvergeResult, Engine,
    ProposedFact,
};
use converge_pack::{ContextKey, PackInputPayload, PackPlanPayload, PackSuggestor, Suggestor};
use converge_storage::{ObjectPath, object_store};
use embassy_sec_edgar::live::{HeadingExtractOptions, extract_section_headings};
use embassy_sec_edgar::{
    AccessionNumber, Cik, FormType, LiveSecEdgarProvider, SecEdgarRequest, SecFilingPayload,
    SecFilingSuggestor,
};
#[cfg(feature = "with-solver")]
use ferrox::mip::MipPlan;
use object_store::ObjectStoreExt;
use prism::packs::SimilarityPack;
use serde::{Deserialize, Serialize};
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
const PRIOR_ACCESSION: &str = "0000320193-24-000123";
const PRIOR_FILING_DATE: &str = "2024-11-01";
const PRIOR_PRIMARY_DOCUMENT: &str = "aapl-20240928.htm";
const PRIOR_FILING_URL: &str =
    "https://www.sec.gov/Archives/edgar/data/320193/000032019324000123/aapl-20240928.htm";
const PRIOR_DETAIL_URL: &str =
    "https://www.sec.gov/Archives/edgar/data/320193/0000320193-24-000123-index.html";
const PROFILE_INDEX_PATH: &str = "sec-review-profiles/index.json";
#[cfg(feature = "with-solver")]
const REVIEW_MIP_REQUEST_ID: &str = "sec-risk-review-allocation-apple-2025-10k";
#[cfg(feature = "with-solver")]
const REVIEW_MIP_PLAN_ID: &str = "mip-plan:sec-risk-review-allocation-apple-2025-10k";

#[derive(Debug, Clone, Copy)]
struct FilingTarget {
    company: &'static str,
    cik_padded: &'static str,
    accession: &'static str,
    form_type: &'static str,
    filing_date: &'static str,
    primary_document: &'static str,
    filing_url: &'static str,
    detail_url: &'static str,
    profile_id: &'static str,
}

const CURRENT_TARGET: FilingTarget = FilingTarget {
    company: COMPANY,
    cik_padded: CIK_PADDED,
    accession: ACCESSION,
    form_type: FORM_TYPE,
    filing_date: FILING_DATE,
    primary_document: PRIMARY_DOCUMENT,
    filing_url: FILING_URL,
    detail_url: DETAIL_URL,
    profile_id: "sec-review-profile:apple-2025-10k",
};

const PRIOR_TARGET: FilingTarget = FilingTarget {
    company: COMPANY,
    cik_padded: CIK_PADDED,
    accession: PRIOR_ACCESSION,
    form_type: FORM_TYPE,
    filing_date: PRIOR_FILING_DATE,
    primary_document: PRIOR_PRIMARY_DOCUMENT,
    filing_url: PRIOR_FILING_URL,
    detail_url: PRIOR_DETAIL_URL,
    profile_id: "sec-review-profile:apple-2024-10k",
};

impl FilingTarget {
    fn request(self) -> Result<SecEdgarRequest> {
        Ok(SecEdgarRequest::Filing {
            cik: Cik::parse(self.cik_padded)?,
            accession_number: AccessionNumber::parse(self.accession)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecReviewProfile {
    profile_id: String,
    company: String,
    cik: String,
    form_type: String,
    accession: String,
    filing_date: String,
    primary_document: String,
    source_url: String,
    detail_url: String,
    source_fact_id: String,
    source_payload_family: String,
    source_request_hash: String,
    source_vendor: String,
    source_mode: String,
    risk_factor_heading_count: u64,
    item_1a_section_bytes: u64,
    review_threshold: f64,
    arbiter_rule_id: String,
    feature_vector: Vec<f64>,
}

#[derive(Debug)]
struct RecurringAnalysis {
    storage_backend: &'static str,
    profile_count: usize,
    profile_paths: Vec<String>,
    current_profile_id: String,
    prior_profile_id: String,
    prism_plan_id: String,
    prism_confidence: f64,
    top_pair: prism::packs::similarity::SimilarityPair,
    current_heading_count: u64,
    prior_heading_count: u64,
}

struct SecReviewMemoryStore {
    store: object_store::memory::InMemory,
    index_path: ObjectPath,
}

#[cfg(feature = "with-solver")]
#[derive(Debug, Clone, Copy)]
struct ReviewLane {
    name: &'static str,
    analyst_hours: f64,
    heading_capacity: f64,
    senior: bool,
}

#[cfg(feature = "with-solver")]
const REVIEW_LANES: &[ReviewLane] = &[
    ReviewLane {
        name: "disclosure_counsel",
        analyst_hours: 3.0,
        heading_capacity: 8.0,
        senior: true,
    },
    ReviewLane {
        name: "supply_chain_risk",
        analyst_hours: 4.0,
        heading_capacity: 9.0,
        senior: false,
    },
    ReviewLane {
        name: "financial_liquidity",
        analyst_hours: 4.0,
        heading_capacity: 8.0,
        senior: false,
    },
    ReviewLane {
        name: "market_competition",
        analyst_hours: 3.0,
        heading_capacity: 7.0,
        senior: false,
    },
    ReviewLane {
        name: "platform_privacy",
        analyst_hours: 5.0,
        heading_capacity: 10.0,
        senior: true,
    },
];

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
            "Step 4: load atelier-domain's SEC 10-K risk policy pack and let Arbiter decide whether auto-clearance is blocked."
        );
        #[cfg(feature = "with-solver")]
        println!(
            "Step 5: translate the Arbiter review constraint into a Ferrox HiGHS MIP allocation problem."
        );
        #[cfg(not(feature = "with-solver"))]
        println!(
            "Step 5: solver allocation is disabled in this build; rerun with --features with-solver."
        );
        println!(
            "Step 6: persist recurring review profiles through converge-storage and compare them with Prism."
        );
        println!(
            "        Local development uses an in-memory ObjectStore; Runway can swap the same contract to GCS."
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
                .is_some_and(|constraint| constraint.rule_id == HEADING_COUNT_REVIEW_RULE_ID)
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
        review_constraint.framework == SEC_RISK_FRAMEWORK,
        "Arbiter constraint framework {}, expected {SEC_RISK_FRAMEWORK}",
        review_constraint.framework
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
        println!("Step 6: read Item 1A from the typed Filing fact.");
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
            "Step 7: extract risk-factor headings through Embassy's calibrated selector chain."
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
    #[cfg(feature = "with-solver")]
    let review_plan = run_review_solver(headings.len() as u64).await?;
    #[cfg(feature = "with-solver")]
    let allocation = validate_review_plan(&review_plan, headings.len())?;

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

    let current_profile = build_review_profile(
        CURRENT_TARGET,
        filing_fact.id().as_str(),
        payload,
        section.body.len(),
        headings.len(),
    );
    let recurring_analysis = run_recurring_analysis(&current_profile).await?;

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
    println!(
        "  policy_pack: {}",
        atelier_domain::sec_risk::SEC_RISK_POLICY_ID
    );
    println!("  action: {:?}", review_constraint.action);
    println!(
        "  threshold: risk_factor_heading_count <= {:.0}",
        heading_review_threshold()
    );
    println!("  observed_risk_factor_heading_count: {}", headings.len());
    print_solver_decision();
    #[cfg(feature = "with-solver")]
    print_review_allocation(&review_plan, &allocation);
    print_recurring_analysis(&recurring_analysis);

    if args.verbose {
        print_verbose_close();
    }

    Ok(())
}

async fn run_converge(request: SecEdgarRequest) -> Result<ConvergeResult> {
    let mut engine = Engine::with_budget(Budget {
        max_cycles: 12,
        max_facts: 32,
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

async fn run_sec_filing_converge(target: FilingTarget) -> Result<ConvergeResult> {
    let mut engine = Engine::with_budget(Budget {
        max_cycles: 8,
        max_facts: 16,
    });
    engine.register_suggestor(SecFilingSuggestor::new(Arc::new(
        LiveSecEdgarProvider::new(),
    )));

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("sec-edgar-request:{}", target.accession),
        target.request()?,
        "atelier-sec-edgar-recurring-analysis",
    ))?;

    Ok(engine.run(ctx).await?)
}

async fn fetch_live_review_profile(target: FilingTarget) -> Result<SecReviewProfile> {
    let result = run_sec_filing_converge(target).await.with_context(|| {
        format!(
            "failed to fetch historical SEC filing: {}",
            target.filing_url
        )
    })?;
    let filing_fact = result
        .context
        .get(ContextKey::Hypotheses)
        .iter()
        .find(|fact| fact.payload::<SecFilingPayload>().is_some())
        .with_context(|| {
            format!(
                "Converge produced no SEC filing hypothesis for {}",
                target.accession
            )
        })?;
    let payload = filing_fact
        .require_payload::<SecFilingPayload>()
        .context("historical SEC filing hypothesis carried the wrong payload type")?;
    validate_payload_matches_target(target, payload)?;
    let section = payload.filing.sections.get("1A").with_context(|| {
        format!(
            "live SEC filing fact did not contain Item 1A: {}",
            target.filing_url
        )
    })?;
    let headings = extract_section_headings(&section.body, &HeadingExtractOptions::default())
        .with_context(|| {
            format!(
                "could not extract plausible Item 1A headings from {}",
                target.filing_url
            )
        })?;

    Ok(build_review_profile(
        target,
        filing_fact.id().as_str(),
        payload,
        section.body.len(),
        headings.len(),
    ))
}

async fn run_recurring_analysis(current_profile: &SecReviewProfile) -> Result<RecurringAnalysis> {
    let store = SecReviewMemoryStore::new_in_memory();
    let prior_profile = fetch_live_review_profile(PRIOR_TARGET).await?;

    let mut profile_paths = Vec::new();
    profile_paths.push(store.put_profile(&prior_profile).await?);
    profile_paths.push(store.put_profile(current_profile).await?);

    let profiles = store
        .load_profiles(CIK_PADDED, FORM_TYPE)
        .await
        .context("failed to load recurring SEC review profiles from memory store")?;
    ensure!(
        profiles.len() >= 2,
        "recurring analysis loaded {} SEC review profiles, expected at least 2",
        profiles.len()
    );

    let prism_plan = run_prism_similarity(&profiles).await?;
    let output = prism_plan
        .plan_as::<prism::packs::similarity::SimilarityOutput>()
        .context("Prism similarity plan payload did not match SimilarityOutput")?;
    let top_pair = output
        .pairs
        .first()
        .cloned()
        .context("Prism similarity produced no comparable profile pair")?;

    let current = profiles
        .iter()
        .find(|profile| profile.profile_id == current_profile.profile_id)
        .context("current profile was not loaded back from memory store")?;
    let prior = profiles
        .iter()
        .find(|profile| profile.profile_id == PRIOR_TARGET.profile_id)
        .context("prior profile was not loaded back from memory store")?;

    Ok(RecurringAnalysis {
        storage_backend: "converge-storage::object_store::memory::InMemory",
        profile_count: profiles.len(),
        profile_paths,
        current_profile_id: current.profile_id.clone(),
        prior_profile_id: prior.profile_id.clone(),
        prism_plan_id: prism_plan.plan_id,
        prism_confidence: prism_plan.confidence,
        top_pair,
        current_heading_count: current.risk_factor_heading_count,
        prior_heading_count: prior.risk_factor_heading_count,
    })
}

async fn run_prism_similarity(profiles: &[SecReviewProfile]) -> Result<PackPlanPayload> {
    let mut engine = Engine::with_budget(Budget {
        max_cycles: 5,
        max_facts: 16,
    });
    engine.register_suggestor(PackSuggestor::new(
        SimilarityPack,
        ContextKey::Seeds,
        ContextKey::Strategies,
    ));

    let items = profiles
        .iter()
        .map(|profile| {
            serde_json::json!({
                "id": profile.profile_id,
                "features": profile.feature_vector,
            })
        })
        .collect::<Vec<_>>();
    let input = serde_json::json!({
        "items": items,
        "metric": "euclidean",
        "top_k": 3,
    });

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        "prism-similarity:sec-review-profiles",
        PackInputPayload::new("similarity", input),
        "atelier-sec-recurring-analysis",
    ))?;

    let result = engine.run(ctx).await?;
    ensure!(result.converged, "Prism similarity run did not converge");
    let plan_fact = result
        .context
        .get(ContextKey::Strategies)
        .iter()
        .find(|fact| {
            fact.payload::<PackPlanPayload>()
                .is_some_and(|payload| payload.pack == "similarity")
        })
        .context("Prism SimilarityPack produced no pack plan")?;

    Ok(plan_fact
        .require_payload::<PackPlanPayload>()
        .context("Prism similarity plan carried the wrong payload type")?
        .clone())
}

impl SecReviewMemoryStore {
    fn new_in_memory() -> Self {
        Self {
            store: object_store::memory::InMemory::new(),
            index_path: ObjectPath::from(PROFILE_INDEX_PATH),
        }
    }

    async fn put_profile(&self, profile: &SecReviewProfile) -> Result<String> {
        let path = profile_object_path(profile);
        let object_path = ObjectPath::from(path.clone());
        let payload =
            serde_json::to_vec_pretty(profile).context("failed to serialize SEC review profile")?;
        self.store
            .put(&object_path, payload.into())
            .await
            .with_context(|| format!("failed to write SEC review profile object {path}"))?;

        let mut index = self.read_index().await?;
        if !index.iter().any(|existing| existing == &path) {
            index.push(path.clone());
            self.write_index(&index).await?;
        }

        Ok(path)
    }

    async fn load_profiles(&self, cik: &str, form_type: &str) -> Result<Vec<SecReviewProfile>> {
        let index = self.read_index().await?;
        let mut profiles = Vec::new();

        for path in index {
            let object_path = ObjectPath::from(path.clone());
            let bytes = self
                .store
                .get(&object_path)
                .await
                .with_context(|| format!("failed to read SEC review profile object {path}"))?
                .bytes()
                .await
                .with_context(|| format!("failed to read bytes for SEC review profile {path}"))?;
            let profile = serde_json::from_slice::<SecReviewProfile>(&bytes)
                .with_context(|| format!("failed to deserialize SEC review profile {path}"))?;
            if profile.cik == cik && profile.form_type == form_type {
                profiles.push(profile);
            }
        }

        Ok(profiles)
    }

    async fn read_index(&self) -> Result<Vec<String>> {
        match self.store.get(&self.index_path).await {
            Ok(result) => {
                let bytes = result
                    .bytes()
                    .await
                    .context("failed to read SEC review profile index bytes")?;
                Ok(serde_json::from_slice(&bytes)
                    .context("failed to deserialize SEC review profile index")?)
            }
            Err(object_store::Error::NotFound { .. }) => Ok(Vec::new()),
            Err(err) => Err(err).context("failed to read SEC review profile index"),
        }
    }

    async fn write_index(&self, index: &[String]) -> Result<()> {
        let payload = serde_json::to_vec_pretty(index)
            .context("failed to serialize SEC review profile index")?;
        self.store
            .put(&self.index_path, payload.into())
            .await
            .context("failed to write SEC review profile index")?;
        Ok(())
    }
}

fn validate_payload_matches_target(target: FilingTarget, payload: &SecFilingPayload) -> Result<()> {
    ensure!(
        payload.filing.cik.as_str() == target.cik_padded,
        "SEC filing fact returned CIK {}, expected {}",
        payload.filing.cik.as_str(),
        target.cik_padded
    );
    ensure!(
        payload.filing.accession_number.as_str() == target.accession,
        "SEC filing fact returned accession {}, expected {}",
        payload.filing.accession_number.as_str(),
        target.accession
    );
    ensure!(
        payload.filing.form_type == FormType::Form10K,
        "SEC filing fact returned form {}, expected {}",
        payload.filing.form_type.as_label(),
        target.form_type
    );
    Ok(())
}

fn build_review_profile(
    target: FilingTarget,
    source_fact_id: &str,
    payload: &SecFilingPayload,
    section_bytes: usize,
    heading_count: usize,
) -> SecReviewProfile {
    SecReviewProfile {
        profile_id: target.profile_id.to_string(),
        company: target.company.to_string(),
        cik: payload.filing.cik.as_str().to_string(),
        form_type: payload.filing.form_type.as_label().to_string(),
        accession: payload.filing.accession_number.as_str().to_string(),
        filing_date: target.filing_date.to_string(),
        primary_document: target.primary_document.to_string(),
        source_url: target.filing_url.to_string(),
        detail_url: target.detail_url.to_string(),
        source_fact_id: source_fact_id.to_string(),
        source_payload_family: "embassy.sec_edgar.filing".to_string(),
        source_request_hash: payload.request_hash.clone(),
        source_vendor: payload.vendor.clone(),
        source_mode: "REAL LIVE".to_string(),
        risk_factor_heading_count: heading_count as u64,
        item_1a_section_bytes: section_bytes as u64,
        review_threshold: heading_review_threshold(),
        arbiter_rule_id: HEADING_COUNT_REVIEW_RULE_ID.to_string(),
        feature_vector: review_feature_vector(heading_count, section_bytes),
    }
}

fn review_feature_vector(heading_count: usize, section_bytes: usize) -> Vec<f64> {
    let heading_count = heading_count as f64;
    let section_bytes = section_bytes as f64;
    let heading_density_per_100k = heading_count / (section_bytes / 100_000.0).max(1.0);
    vec![
        heading_count / 100.0,
        section_bytes / 1_000_000.0,
        heading_density_per_100k / 20.0,
        heading_count / heading_review_threshold(),
    ]
}

fn profile_object_path(profile: &SecReviewProfile) -> String {
    format!(
        "sec-review-profiles/{}/{}/{}.json",
        profile.cik,
        profile.form_type.to_ascii_lowercase().replace('-', ""),
        profile.accession.replace('-', "")
    )
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

#[cfg(feature = "with-solver")]
#[derive(Debug)]
struct ReviewAllocation {
    selected: Vec<ReviewLane>,
    analyst_hours: f64,
    heading_capacity: f64,
    senior_count: usize,
}

#[cfg(feature = "with-solver")]
async fn run_review_solver(heading_count: u64) -> Result<MipPlan> {
    let mut engine = Engine::with_budget(Budget {
        max_cycles: 4,
        max_facts: 8,
    });
    engine.register_suggestor(ferrox::mip::HighsMipSuggestor);

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        format!("mip-request:{REVIEW_MIP_REQUEST_ID}"),
        build_review_mip_request(heading_count),
        "atelier-sec-edgar-risk-review-solver",
    ))?;

    let result = engine.run(ctx).await?;
    let plan_fact = result
        .context
        .get(ContextKey::Strategies)
        .iter()
        .find(|fact| fact.id().as_str() == REVIEW_MIP_PLAN_ID)
        .context("Ferrox produced no SEC risk review allocation plan")?;

    Ok(plan_fact
        .require_payload::<MipPlan>()
        .context("SEC risk review allocation plan carried the wrong payload type")?
        .clone())
}

#[cfg(feature = "with-solver")]
fn build_review_mip_request(heading_count: u64) -> ferrox::mip::MipRequest {
    use ferrox::mip::{MipConstraint, MipObjective, MipRequest, MipTerm, MipVariable, VarKind};

    let variable = |lane: &ReviewLane| format!("review_{}", lane.name);
    let variables = REVIEW_LANES
        .iter()
        .map(|lane| MipVariable {
            name: variable(lane),
            lb: 0.0,
            ub: 1.0,
            kind: VarKind::Binary,
        })
        .collect::<Vec<_>>();

    let capacity_terms = REVIEW_LANES
        .iter()
        .map(|lane| MipTerm {
            var: variable(lane),
            coeff: lane.heading_capacity,
        })
        .collect::<Vec<_>>();
    let breadth_terms = REVIEW_LANES
        .iter()
        .map(|lane| MipTerm {
            var: variable(lane),
            coeff: 1.0,
        })
        .collect::<Vec<_>>();
    let senior_terms = REVIEW_LANES
        .iter()
        .filter(|lane| lane.senior)
        .map(|lane| MipTerm {
            var: variable(lane),
            coeff: 1.0,
        })
        .collect::<Vec<_>>();
    let objective_terms = REVIEW_LANES
        .iter()
        .map(|lane| MipTerm {
            var: variable(lane),
            coeff: lane.analyst_hours,
        })
        .collect::<Vec<_>>();

    MipRequest {
        id: REVIEW_MIP_REQUEST_ID.to_string(),
        variables,
        constraints: vec![
            MipConstraint {
                name: "cover_risk_factor_headings".to_string(),
                lb: heading_count as f64,
                ub: f64::INFINITY,
                terms: capacity_terms,
            },
            MipConstraint {
                name: "review_breadth_at_least_three_lanes".to_string(),
                lb: 3.0,
                ub: f64::INFINITY,
                terms: breadth_terms,
            },
            MipConstraint {
                name: "at_least_one_senior_reviewer".to_string(),
                lb: 1.0,
                ub: f64::INFINITY,
                terms: senior_terms,
            },
        ],
        objective: MipObjective {
            terms: objective_terms,
            maximize: false,
        },
        time_limit_seconds: Some(5.0),
        mip_gap_tolerance: Some(1e-6),
    }
}

#[cfg(feature = "with-solver")]
fn validate_review_plan(plan: &MipPlan, heading_count: usize) -> Result<ReviewAllocation> {
    ensure!(
        plan.status.is_successful(),
        "review allocation solver did not produce a usable plan: {:?}",
        plan.status
    );

    let mut selected = Vec::new();
    for lane in REVIEW_LANES {
        let variable = format!("review_{}", lane.name);
        if plan
            .values
            .iter()
            .any(|(name, value)| name == &variable && *value > 0.5)
        {
            selected.push(*lane);
        }
    }

    let analyst_hours = selected.iter().map(|lane| lane.analyst_hours).sum::<f64>();
    let heading_capacity = selected
        .iter()
        .map(|lane| lane.heading_capacity)
        .sum::<f64>();
    let senior_count = selected.iter().filter(|lane| lane.senior).count();

    ensure!(
        heading_capacity + 1e-6 >= heading_count as f64,
        "review allocation covers {heading_capacity} headings, expected at least {heading_count}"
    );
    ensure!(
        selected.len() >= 3,
        "review allocation selected {} lanes, expected at least 3",
        selected.len()
    );
    ensure!(
        senior_count >= 1,
        "review allocation selected no senior reviewer"
    );

    Ok(ReviewAllocation {
        selected,
        analyst_hours,
        heading_capacity,
        senior_count,
    })
}

fn sec_review_rules() -> Vec<arbiter::ComplianceRule> {
    sec_risk_policy_pack().rules()
}

fn sec_risk_policy_pack() -> SecRiskPolicyPack {
    SecRiskPolicyPack::annual_report_review()
}

fn heading_review_threshold() -> f64 {
    sec_risk_policy_pack()
        .thresholds()
        .max_headings_for_auto_clearance
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
        json_num(heading_review_threshold()),
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

#[cfg(feature = "with-solver")]
fn print_solver_decision() {
    println!("  solver_backing: enabled");
}

#[cfg(not(feature = "with-solver"))]
fn print_solver_decision() {
    println!("  solver_backing: disabled in this build; rerun with --features with-solver");
}

#[cfg(feature = "with-solver")]
fn print_review_allocation(plan: &MipPlan, allocation: &ReviewAllocation) {
    println!();
    println!("Solver-backed review allocation:");
    println!("  solver: {}", plan.solver);
    println!("  status: {:?}", plan.status);
    println!("  mip_gap: {:.6}", plan.mip_gap);
    println!("  objective_analyst_hours: {:.1}", plan.objective_value);
    println!("  selected_lanes: {}", allocation.selected.len());
    for lane in &allocation.selected {
        println!(
            "    - {}: {:.1}h, capacity {:.0} headings{}",
            lane.name,
            lane.analyst_hours,
            lane.heading_capacity,
            if lane.senior { ", senior" } else { "" }
        );
    }
    println!("  total_analyst_hours: {:.1}", allocation.analyst_hours);
    println!(
        "  total_heading_capacity: {:.0}",
        allocation.heading_capacity
    );
    println!("  senior_reviewers: {}", allocation.senior_count);
}

fn print_recurring_analysis(analysis: &RecurringAnalysis) {
    println!();
    println!("Recurring profile analysis:");
    println!("  result: current filing compared against prior live SEC profile");
    println!("  storage_contract: converge-storage ObjectStore");
    println!(
        "  storage_backend: {} (LOCAL REAL)",
        analysis.storage_backend
    );
    println!("  cloud_swap: Runway/GCS can replace the backend by configuration");
    println!("  time_series_db: not used");
    println!("  profiles_loaded: {}", analysis.profile_count);
    println!("  current_profile: {}", analysis.current_profile_id);
    println!("  current_headings: {}", analysis.current_heading_count);
    println!("  prior_profile: {}", analysis.prior_profile_id);
    println!("  prior_headings: {}", analysis.prior_heading_count);
    println!("  profile_objects:");
    for path in &analysis.profile_paths {
        println!("    - {path}");
    }
    println!("  prism_pack: SimilarityPack");
    println!("  prism_metric: euclidean");
    println!("  prism_plan: {}", analysis.prism_plan_id);
    println!("  prism_confidence: {:.3}", analysis.prism_confidence);
    println!(
        "  top_pair: {} <-> {}",
        analysis.top_pair.id_a, analysis.top_pair.id_b
    );
    println!("  similarity_score: {:.6}", analysis.top_pair.score);
    println!(
        "  heading_delta_current_minus_prior: {}",
        analysis.current_heading_count as i64 - analysis.prior_heading_count as i64
    );
    println!("  mocked: false");
}

fn print_declaration() {
    println!("Declaration: REAL LIVE");
    println!("This scenario calls official SEC EDGAR over the network.");
    println!("It does not mock any Mosaic extension.");
    println!("It does not use Embassy's deterministic SEC provider.");
    println!("It does not use recorded HTTP fixtures.");
    println!("Converge path: SecEdgarRequest seed -> SecFilingSuggestor -> LiveSecEdgarProvider.");
    println!(
        "Downstream decision: atelier-domain SEC risk policy pack + Arbiter ComplianceGateSuggestor."
    );
    #[cfg(feature = "with-solver")]
    println!("Solver backing: Ferrox HighsMipSuggestor (HiGHS MIP), LOCAL REAL.");
    #[cfg(not(feature = "with-solver"))]
    println!("Solver backing: disabled in this build; no heuristic fallback is used.");
    println!(
        "Storage backing: converge-storage ObjectStore with local InMemory backend, LOCAL REAL."
    );
    println!("Analytics backing: Prism SimilarityPack through Converge, LOCAL REAL.");
    println!(
        "External resources: SEC EDGAR primary filing documents for Apple Inc. 2025 and 2024 Form 10-K."
    );
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
    println!("  - atelier-domain supplied a reusable SEC 10-K risk policy pack.");
    println!("  - A downstream Arbiter gate made a typed review decision from that SEC fact.");
    #[cfg(feature = "with-solver")]
    println!("  - Ferrox HiGHS solved a MIP allocation for the required review lanes.");
    println!(
        "  - converge-storage persisted the current and prior review profiles behind an ObjectStore contract."
    );
    println!(
        "  - Prism compared the recurring profile history with SimilarityPack through Converge."
    );
    println!(
        "  - The review document preserved source fact id, request hash, provider, CIK, accession, and source URL."
    );
    println!("  - No stub, mock, fake, or recorded fixture supplied the filing content.");
    println!("  - The output can be checked against an official SEC page in human speed.");
    println!();
    println!("What this does not prove:");
    println!("  - It is not the full three-module v1.1 combinatory scenario.");
    #[cfg(feature = "with-solver")]
    println!("  - It is not yet a complete investment workflow.");
    #[cfg(not(feature = "with-solver"))]
    println!(
        "  - This build is not solver-backed; rerun with --features with-solver for Ferrox allocation."
    );
    println!("  - It is the live-resource anchor that those larger examples can now build from.");
}
