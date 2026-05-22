use std::time::Instant;

use anyhow::{Context, Result, ensure};
use clap::Parser;
use embassy_sec_edgar::live::{HeadingExtractOptions, extract_section_headings};
use embassy_sec_edgar::{
    AccessionNumber, CallContext, Cik, FormType, LiveSecEdgarProvider, SecEdgarProvider,
    SecEdgarRequest,
};
use serde_json::Value;

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
        println!("Step 1: call the Embassy sec-edgar provider-shaped live transport.");
        println!(
            "        The provider resolves filing metadata, fetches the SEC primary document, and returns Observation<Filing>."
        );
    }

    let request = SecEdgarRequest::Filing {
        cik: Cik::parse(CIK_PADDED)?,
        accession_number: AccessionNumber::parse(ACCESSION)?,
    };
    let provider = LiveSecEdgarProvider::new();
    let started = Instant::now();
    let response = provider
        .fetch(&request, &CallContext::default())
        .await
        .with_context(|| {
            format!("failed to fetch live SEC filing through provider: {FILING_URL}")
        })?;
    let elapsed = started.elapsed();
    let observation = response
        .records
        .first()
        .context("live SEC provider returned no observations")?;
    let filing = &observation.content;

    ensure!(
        filing.cik.as_str() == CIK_PADDED,
        "live SEC provider returned CIK {}, expected {CIK_PADDED}",
        filing.cik.as_str()
    );
    ensure!(
        filing.accession_number.as_str() == ACCESSION,
        "live SEC provider returned accession {}, expected {ACCESSION}",
        filing.accession_number.as_str()
    );
    ensure!(
        filing.form_type == FormType::Form10K,
        "live SEC provider returned form {}, expected {FORM_TYPE}",
        filing.form_type.as_label()
    );

    let raw = observation
        .raw_response
        .as_deref()
        .context("live SEC observation did not include provider metadata")?;
    let metadata: Value =
        serde_json::from_str(raw).context("invalid live SEC provider metadata")?;
    let html_bytes = metadata
        .get("html_bytes")
        .and_then(Value::as_u64)
        .context("live SEC provider metadata missing html_bytes")?;
    let primary_url = metadata
        .get("primary_url")
        .and_then(Value::as_str)
        .context("live SEC provider metadata missing primary_url")?;

    println!("Live fetch:");
    println!("  result: success");
    println!("  provider: {}", observation.vendor);
    println!("  observation_id: {}", observation.observation_id);
    println!("  request_hash: {}", observation.request_hash);
    println!("  bytes: {html_bytes}");
    println!("  elapsed_ms: {}", elapsed.as_millis());
    println!("  provider_latency_ms: {}", observation.latency_ms);
    println!("  validated_observation: CIK {CIK_PADDED}, accession {ACCESSION}, form {FORM_TYPE}");
    println!("  primary_url: {primary_url}");
    println!();

    if args.verbose {
        println!("Step 2: read Item 1A from the typed Filing observation.");
        println!(
            "        The live provider put the SEC section body under Filing.sections[\"1A\"]."
        );
    }

    let section = filing.sections.get("1A").with_context(|| {
        format!("live SEC filing observation did not contain Item 1A: {FILING_URL}")
    })?;

    println!("Item 1A section:");
    println!("  section_bytes: {}", section.body.len());
    println!(
        "  contains_risk_factors: {}",
        section.body.contains("Risk Factors")
    );
    println!();

    if args.verbose {
        println!(
            "Step 3: extract risk-factor headings through Embassy's calibrated selector chain."
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
    println!("  source: typed Filing observation from live SEC HTML");
    println!("  extension: converge-embassy-sec-edgar LiveSecEdgarProvider");
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

fn print_declaration() {
    println!("Declaration: REAL LIVE");
    println!("This scenario calls official SEC EDGAR over the network.");
    println!("It does not mock any Mosaic extension.");
    println!("It does not use StubSecEdgarProvider.");
    println!("It does not use recorded HTTP fixtures.");
    println!("External resource: SEC EDGAR primary filing document for Apple Inc. 2025 Form 10-K.");
    println!();
}

fn print_verbose_intro() {
    println!("Educational walkthrough:");
    println!(
        "  This is a deliberately small proof slice. The goal is not a polished product workflow yet."
    );
    println!(
        "  The goal is to prove that atelier can call a live Mosaic extension path and make the trust boundary obvious."
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
    println!("  - The live call went through Embassy's provider-shaped LiveSecEdgarProvider.");
    println!("  - The provider returned Observation<Filing>, not just raw HTML.");
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
