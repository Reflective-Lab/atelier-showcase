use std::time::Instant;

use anyhow::{Context, Result, ensure};
use clap::Parser;
use embassy_sec_edgar::live::{
    HeadingExtractOptions, LiveFetchOptions, extract_section_headings, fetch_filing_html,
    locate_item_section,
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
        println!(
            "Step 1: fetch the SEC primary document through Embassy sec-edgar live transport."
        );
        println!(
            "        The live transport uses SEC-aware User-Agent, timeout, byte cap, and politeness defaults."
        );
    }

    let started = Instant::now();
    let html = fetch_filing_html(FILING_URL, &LiveFetchOptions::default())
        .await
        .with_context(|| format!("failed to fetch live SEC filing document: {FILING_URL}"))?;
    let elapsed = started.elapsed();

    ensure!(
        html.contains(CIK_PADDED),
        "live SEC response did not contain expected CIK {CIK_PADDED}"
    );

    println!("Live fetch:");
    println!("  result: success");
    println!("  bytes: {}", html.len());
    println!("  elapsed_ms: {}", elapsed.as_millis());
    println!("  validated_in_body: CIK {CIK_PADDED}");
    println!();

    if args.verbose {
        println!("Step 2: locate Item 1A by SEC item markers.");
        println!(
            "        The extractor takes the third Item 1A marker as the real section start and the next Item 1B marker as the section end."
        );
    }

    let section = locate_item_section(&html, "1A", "1B").with_context(|| {
        format!("could not locate Item 1A / Item 1B bounds in live SEC filing: {FILING_URL}")
    })?;

    println!("Item 1A section:");
    println!("  section_bytes: {}", section.len());
    println!(
        "  contains_risk_factors: {}",
        section.contains("Risk Factors")
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

    let headings = extract_section_headings(section, &HeadingExtractOptions::default())
        .with_context(|| {
            format!("could not extract plausible Item 1A headings from {FILING_URL}")
        })?;

    println!("Risk-factor heading extraction:");
    println!("  headings: {}", headings.len());
    println!("  source: live SEC HTML");
    println!("  extension: converge-embassy-sec-edgar with feature live");
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
    println!("  - The live call went through the Embassy sec-edgar live feature.");
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
