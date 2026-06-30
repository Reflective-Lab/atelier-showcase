// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, ValueEnum};
use helm_coordination_headless::{CrowdConsensusRun, GateOutcome, HeadlessCoordinationRun};

#[derive(Debug, Clone, Copy, Parser)]
struct Args {
    #[arg(long, value_enum, default_value_t = UseCaseArg::Strict)]
    use_case: UseCaseArg,

    #[arg(long, value_enum, default_value_t = OutputFormat::Markdown)]
    format: OutputFormat,

    #[arg(long, value_enum, default_value_t = GateArg::Approved)]
    gate: GateArg,

    #[arg(long, value_parser = parse_user_count, default_value_t = 30)]
    users: usize,

    #[arg(long)]
    seed: Option<u64>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum UseCaseArg {
    Strict,
    Crowd,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Markdown,
    Jsonl,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GateArg {
    Approved,
    Rejected,
    TimedOut,
}

impl From<GateArg> for GateOutcome {
    fn from(value: GateArg) -> Self {
        match value {
            GateArg::Approved => Self::Approved,
            GateArg::Rejected => Self::Rejected,
            GateArg::TimedOut => Self::TimedOut,
        }
    }
}

fn main() {
    let args = Args::parse();

    match args.use_case {
        UseCaseArg::Strict => {
            let run = HeadlessCoordinationRun::scripted_burst_with_gate(args.gate.into());
            match args.format {
                OutputFormat::Markdown => print!("{}", run.markdown_report()),
                OutputFormat::Jsonl => print!("{}", run.jsonl_timeline()),
            }
        }
        UseCaseArg::Crowd => {
            let seed = args.seed.unwrap_or_else(unpredictable_seed);
            let run =
                CrowdConsensusRun::scripted(args.users, seed).expect("CLI validates user count");
            match args.format {
                OutputFormat::Markdown => print!("{}", run.markdown_report()),
                OutputFormat::Jsonl => print!("{}", run.jsonl_timeline()),
            }
        }
    }
}

fn parse_user_count(value: &str) -> Result<usize, String> {
    let count = value
        .parse::<usize>()
        .map_err(|error| format!("users must be a number: {error}"))?;
    match count {
        5 | 30 | 100 => Ok(count),
        _ => Err("users must be one of 5, 30, or 100".to_string()),
    }
}

fn unpredictable_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0xC00D_1A71_0BAD_F00D)
}
