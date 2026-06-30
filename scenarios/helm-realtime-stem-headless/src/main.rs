// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

use clap::{Parser, ValueEnum};
use helm_realtime_stem_headless::{InteractiveCaseId, cases::run_case};

#[derive(Debug, Clone, Copy, Parser)]
struct Args {
    #[arg(long, value_enum, default_value_t = CaseArg::MarqueeBurstRoom)]
    case: CaseArg,

    #[arg(long, value_enum, default_value_t = OutputFormat::Markdown)]
    format: OutputFormat,

    #[arg(long)]
    list: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CaseArg {
    SingleUserShortLoop,
    ServerOffloadUnderLoad,
    PreemptivePivot,
    ThreeUserConcurrentRoom,
    LongAndShortServerMix,
    GateWhileLoopsActive,
    BudgetExhaustion,
    MarqueeBurstRoom,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Markdown,
    Jsonl,
}

impl From<CaseArg> for InteractiveCaseId {
    fn from(value: CaseArg) -> Self {
        match value {
            CaseArg::SingleUserShortLoop => Self::SingleUserShortLoop,
            CaseArg::ServerOffloadUnderLoad => Self::ServerOffloadUnderLoad,
            CaseArg::PreemptivePivot => Self::PreemptivePivot,
            CaseArg::ThreeUserConcurrentRoom => Self::ThreeUserConcurrentRoom,
            CaseArg::LongAndShortServerMix => Self::LongAndShortServerMix,
            CaseArg::GateWhileLoopsActive => Self::GateWhileLoopsActive,
            CaseArg::BudgetExhaustion => Self::BudgetExhaustion,
            CaseArg::MarqueeBurstRoom => Self::MarqueeBurstRoom,
        }
    }
}

fn main() {
    let args = Args::parse();

    if args.list {
        for case in InteractiveCaseId::all() {
            println!("{}", case.as_str());
        }
        return;
    }

    let run = run_case(args.case.into());
    match args.format {
        OutputFormat::Markdown => print!("{}", run.markdown_report()),
        OutputFormat::Jsonl => print!("{}", run.jsonl_timeline()),
    }
}
