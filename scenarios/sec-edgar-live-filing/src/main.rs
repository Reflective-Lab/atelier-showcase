use std::{sync::Arc, time::Instant};

use anyhow::{Context, Result, ensure};
use clap::Parser;
use converge_kernel::{Budget, ContextState, ConvergeResult, Engine, ProposedFact};
use converge_pack::ContextKey;
use embassy_sec_edgar::live::{HeadingExtractOptions, extract_section_headings};
use embassy_sec_edgar::{
    AccessionNumber, Cik, FormType, LiveSecEdgarProvider, SecEdgarRequest, SecFilingPayload,
    SecFilingSuggestor,
};

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

#[derive(Debug, Parser)]
#[command(about = "Fetch a REAL LIVE SEC EDGAR filing through Embassy")]
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
        println!("Step 3: read Item 1A from the typed Filing fact.");
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
            "Step 4: extract risk-factor headings through Embassy's calibrated selector chain."
        );
        println!(
            "        A low or implausibly high heading count fails loudly instead of pretending the parse is good."
        );
    }

    let headings = extract_section_headings(&section.body, &HeadingExtractOptions::default())
        .with_context(|| {
            format!("could not extract plausible Item 1A headings from {FILING_URL}")
        })?;

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

    if args.verbose {
        print_verbose_close();
    }

    Ok(())
}

async fn run_converge(request: SecEdgarRequest) -> Result<ConvergeResult> {
    let mut engine = Engine::with_budget(Budget {
        max_cycles: 4,
        max_facts: 8,
    });
    engine.register_suggestor(SecFilingSuggestor::new(Arc::new(
        LiveSecEdgarProvider::new(),
    )));

    let mut ctx = ContextState::new();
    ctx.add_proposal(ProposedFact::new(
        ContextKey::Seeds,
        "sec-edgar-request:apple-2025-10k",
        request,
        "atelier-sec-edgar-live-filing",
    ))?;

    Ok(engine.run(ctx).await?)
}

fn print_declaration() {
    println!("Declaration: REAL LIVE");
    println!("This scenario calls official SEC EDGAR over the network.");
    println!("It does not mock any Mosaic extension.");
    println!("It does not use Embassy's deterministic SEC provider.");
    println!("It does not use recorded HTTP fixtures.");
    println!("Converge path: SecEdgarRequest seed -> SecFilingSuggestor -> LiveSecEdgarProvider.");
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
    println!("  - No stub, mock, fake, or recorded fixture supplied the filing content.");
    println!("  - The output can be checked against an official SEC page in human speed.");
    println!();
    println!("What this does not prove:");
    println!("  - It is not the full three-module v1.1 combinatory scenario.");
    println!(
        "  - It does not yet turn the filing into a policy, solver, or memory-backed decision."
    );
    println!("  - It is the live-resource anchor that those larger examples can now build from.");
}
