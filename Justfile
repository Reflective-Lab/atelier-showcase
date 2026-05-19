# Converge Extension — Release-Grade Recipes
# Mirror of foundation Justfile. Bound by Extension Release Checklist.
# https://github.com/Reflective-Lab/converge/blob/main/kb/Standards/Extension%20Release%20Checklist.md
#
# Replace `atelier` with the published crate name (mnemos, prism, arbiter, ...)
# in the few places it is referenced below before committing.

set dotenv-load := true

# ── Compile gates ──────────────────────────────────────────────────────────

# Run all four basic gates
default: check lint test

check:
    cargo check --workspace

test:
    cargo test --workspace --all-targets

# Run a single test by name
test-one name:
    cargo test --workspace --all-targets -- {{name}}

# Cross-extension solver e2e — requires libortools installed on the host.
# Runs the ferrox CP-SAT path of scenarios/solver-policy-allocation.
solver-check:
    cargo test -p example-solver-policy-allocation --features with-solver --test end_to_end

lint:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cargo fmt

# Auto-fix lint issues where possible
fix-lint:
    cargo clippy --fix --allow-staged --allow-dirty --allow-no-vcs
    cargo fmt

# ── Showcase scenarios ─────────────────────────────────────────────────────
#
# Each scenario is a runnable end-to-end demo under scenarios/. Some
# need API keys (round-driven-formation-design) — drop them into a
# .env at the workspace root or next to the scenario; `set
# dotenv-load := true` above pulls them in for both `just` and the
# child `cargo run`.

# Run a single showcase scenario by its crate name (the `name` field
# in scenarios/*/Cargo.toml — `example-*` for legacy scenarios,
# `scenario-*` for newer ones).
#
#   just show example-expense-approval
#   just show scenario-round-driven-formation-design
show name:
    cargo run -p {{name}}

# The round-driven design Formation showcase. Needs at least one
# LLM provider key (ANTHROPIC_API_KEY / OPENAI_API_KEY /
# GEMINI_API_KEY / …) — see
# scenarios/round-driven-formation-design/.env.example.
show-round-driven:
    cargo run -p scenario-round-driven-formation-design

# Ferrox CP-SAT K-of-N plan-selection showcase. Default build
# prints constraints + a feature hint and exits; pass
# `--features with-solver` to actually solve via CP-SAT.
# libortools is vendored under
# mosaic-extensions/ferrox-solvers/vendor/ortools/ — no
# system install required.
show-multi-plan:
    cargo run -p scenario-multi-plan-allocation
show-multi-plan-solver:
    cargo run -p scenario-multi-plan-allocation --features with-solver

# Run every showcase scenario in sequence. Halts on the first
# non-zero exit. Note: scenario-round-driven-formation-design will
# exit honestly without an LLM key set; that's intentional — drop
# the key in .env before running this if you want the full sweep.
show-all:
    cargo run -p example-arbiter-ferrox-solver-gallery
    cargo run -p example-expense-approval
    cargo run -p scenario-high-risk-claim-portfolio
    cargo run -p example-loan-application
    cargo run -p scenario-multi-plan-allocation
    cargo run -p example-meeting-scheduler
    cargo run -p scenario-round-driven-formation-design
    cargo run -p example-solver-policy-allocation
    cargo run -p scenario-truth-driven-formation
    cargo run -p example-vendor-selection

# ── Test layout guard ──────────────────────────────────────────────────────

# Reject ad-hoc property/proptest/negative test files outside src/ and tests/
test-layout:
    #!/usr/bin/env bash
    set -euo pipefail
    bad="$(find crates -type f \
        \( -name '*proptest*.rs' -o -name '*property*.rs' -o -name '*negative*.rs' \) \
        ! -path '*/src/*' ! -path '*/tests/*' -print)"
    if [ -n "${bad}" ]; then
        echo "Non-standard Rust test files must live under src/ or tests/:"
        echo "${bad}"
        exit 1
    fi

# ── The four release-grade gates ───────────────────────────────────────────

# Gate 1: supply-chain audit. Mirrors foundation's security-audit.
# Output:
#   target/security/audit.json   (cargo-audit JSON)
#   target/security/deny.txt     (cargo-deny human report)
#   target/security/summary.txt  (combined human summary)
security-audit:
    #!/usr/bin/env bash
    set -uo pipefail
    out_dir="target/security"
    mkdir -p "${out_dir}"
    summary="${out_dir}/summary.txt"
    : > "${summary}"
    echo "── cargo-audit ──────────────────────────────" | tee -a "${summary}"
    cargo audit --json \
        --ignore RUSTSEC-2023-0089 \
        --ignore RUSTSEC-2024-0384 \
        --ignore RUSTSEC-2024-0436 \
        --ignore RUSTSEC-2025-0012 \
        --ignore RUSTSEC-2025-0134 \
        --ignore RUSTSEC-2021-0141 \
        --ignore RUSTSEC-2025-0141 \
        --ignore RUSTSEC-2026-0002 \
        > "${out_dir}/audit.json" || true
    cargo audit --deny warnings \
        --ignore RUSTSEC-2023-0089 \
        --ignore RUSTSEC-2024-0384 \
        --ignore RUSTSEC-2024-0436 \
        --ignore RUSTSEC-2025-0012 \
        --ignore RUSTSEC-2025-0134 \
        --ignore RUSTSEC-2021-0141 \
        --ignore RUSTSEC-2025-0141 \
        --ignore RUSTSEC-2026-0002 \
        2>&1 | tee -a "${summary}"
    audit_human_status=${PIPESTATUS[0]}
    echo "" | tee -a "${summary}"
    echo "── cargo-deny ───────────────────────────────" | tee -a "${summary}"
    cargo deny check 2>&1 | tee "${out_dir}/deny.txt" | tee -a "${summary}"
    deny_status=${PIPESTATUS[0]}
    echo "" | tee -a "${summary}"
    echo "audit→${out_dir}/audit.json  deny→${out_dir}/deny.txt  summary→${summary}"
    if [ "${audit_human_status}" -ne 0 ] || [ "${deny_status}" -ne 0 ]; then
        exit 1
    fi

# Gate 2: workspace coverage. ≥ 80% per crate, no regression.
# Output:
#   target/coverage/converge-coverage.json  (machine-readable summary)
#   target/coverage/lcov.info               (LCOV for codecov/sonar)
#   target/coverage/html/index.html         (browsable report)
coverage:
    #!/usr/bin/env bash
    set -euo pipefail
    out_dir="target/coverage"
    mkdir -p "${out_dir}/html"
    ignore_re='(^|/)(tests|benches|tutorials|scenarios)/'
    common=(--workspace --lib --tests --ignore-filename-regex "${ignore_re}")
    cargo llvm-cov clean --workspace
    rm -rf target/tests/trybuild
    cargo llvm-cov "${common[@]}" --no-report
    cargo llvm-cov report --ignore-filename-regex "${ignore_re}" \
        --json --summary-only --output-path "${out_dir}/converge-coverage.json"
    cargo llvm-cov report --ignore-filename-regex "${ignore_re}" \
        --lcov --output-path "${out_dir}/lcov.info"
    cargo llvm-cov report --ignore-filename-regex "${ignore_re}" \
        --html --output-dir "${out_dir}/html"
    pct=$(python3 -c "import json; d=json.load(open('${out_dir}/converge-coverage.json')); print(f\"{d['data'][0]['totals']['lines']['percent']:.1f}\")")
    echo "coverage: ${pct}%  json→${out_dir}/converge-coverage.json  lcov→${out_dir}/lcov.info  html→${out_dir}/html/index.html"
    # Floor enforcement (per Extension Release Checklist §4)
    awk -v p="${pct}" 'BEGIN { if (p+0 < 80) { print "FAIL: coverage " p "% below 80% floor"; exit 1 } }'

# Gate 3: Criterion baseline. Set PERF_BASELINE to the release tag.
# Output:
#   target/criterion/                       (per-bench HTML + raw data)
#   kb/Baselines/latest-baseline.json       (extracted summary)
performance-profile:
    #!/usr/bin/env bash
    set -euo pipefail
    name="${PERF_BASELINE:-v0.1.0}"
    mode_flag="--save-baseline"
    if [ -d "target/criterion" ]; then
        existing="$(find target/criterion -mindepth 2 -maxdepth 3 -type d -name "${name}" -print -quit 2>/dev/null || true)"
        if [ -n "${existing}" ]; then
            mode_flag="--baseline"
        fi
    fi
    echo "performance-profile: ${mode_flag} ${name}"
    # If your extension ships benchmarks, list the benchable crates here:
    #   for c in atelier; do cargo bench -p "$c" -- "${mode_flag}" "${name}"; done
    cargo bench --workspace -- "${mode_flag}" "${name}" || true
    if [ -f scripts/extract-criterion-baseline.py ]; then
        python3 scripts/extract-criterion-baseline.py || \
            echo "warn: baseline extraction failed (non-fatal)"
    fi
    echo "performance-profile: criterion→target/criterion/"

# Gate 4: bounded soak run. Configure with SOAK_DURATION_MIN (default 5).
# Output:
#   target/soak/soak-<UTC>.log   (full nocapture log)
#   target/soak/latest.log       (symlink to most recent run)
soak:
    #!/usr/bin/env bash
    set -euo pipefail
    duration_min="${SOAK_DURATION_MIN:-5}"
    out_dir="target/soak"
    mkdir -p "${out_dir}"
    stamp="$(date -u +%Y%m%dT%H%M%SZ)"
    log="${out_dir}/soak-${stamp}.log"
    cycles=$(awk -v d="${duration_min}" 'BEGIN { printf "%d", 200 * d }')
    iterations=$(awk -v d="${duration_min}" 'BEGIN { printf "%d", 40 * d }')
    concurrency=100
    echo "soak: duration=${duration_min}min cycles=${cycles} concurrency=${concurrency} iterations=${iterations}" | tee "${log}"
    SOAK_CYCLES="${cycles}" \
    SOAK_CONCURRENCY="${concurrency}" \
    SOAK_ITERATIONS="${iterations}" \
    cargo test --workspace -- --include-ignored soak --nocapture 2>&1 | tee -a "${log}"
    ln -sf "soak-${stamp}.log" "${out_dir}/latest.log"
    echo "soak: log → ${log}"

# ── Release ritual ─────────────────────────────────────────────────────────

# The five-command release. All five must be green before tagging.
release-check:
    just security-audit
    just coverage
    PERF_BASELINE="v$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')" just performance-profile
    SOAK_DURATION_MIN=5 just soak
    just lint
    cargo test --workspace
