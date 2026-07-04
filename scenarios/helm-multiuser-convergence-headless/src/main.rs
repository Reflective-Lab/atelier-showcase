// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

use clap::{Parser, ValueEnum};
use helm_multiuser_convergence_headless::cases::{ConvergenceCase, run_case};
use helm_multiuser_convergence_headless::report::{jsonl_timeline, markdown_report};

#[derive(Debug, Clone, Parser)]
#[command(
    name = "helm-multiuser-convergence-headless",
    about = "Multi-user Helm coordination session proof — headless CLI"
)]
struct Args {
    #[arg(
        long,
        value_enum,
        default_value_t = CaseArg::FullSession,
        help = "Which convergence case to run"
    )]
    case: CaseArg,

    #[arg(
        long,
        value_enum,
        default_value_t = OutputFormat::Markdown,
        help = "Output format"
    )]
    format: OutputFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CaseArg {
    FullSession,
    SolverBurst,
    GateConflict,
}

impl From<CaseArg> for ConvergenceCase {
    fn from(value: CaseArg) -> Self {
        match value {
            CaseArg::FullSession => Self::FullSession,
            CaseArg::SolverBurst => Self::SolverBurst,
            CaseArg::GateConflict => Self::GateConflict,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Markdown,
    Jsonl,
}

fn main() {
    let args = Args::parse();
    let session = run_case(args.case.into());
    match args.format {
        OutputFormat::Markdown => print!("{}", markdown_report(&session)),
        OutputFormat::Jsonl => print!("{}", jsonl_timeline(&session)),
    }
}
