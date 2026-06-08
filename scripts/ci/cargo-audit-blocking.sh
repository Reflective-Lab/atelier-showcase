#!/usr/bin/env bash
set -euo pipefail

# Keep this list in sync with the release-grade security-audit recipe. Each
# ignore below is an explicit accepted transitive advisory while upstream
# owners move.
cargo audit --deny warnings \
  --ignore RUSTSEC-2023-0089 \
  --ignore RUSTSEC-2024-0384 \
  --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2025-0012 \
  --ignore RUSTSEC-2025-0134 \
  --ignore RUSTSEC-2021-0141 \
  --ignore RUSTSEC-2025-0141 \
  --ignore RUSTSEC-2025-0119 \
  --ignore RUSTSEC-2026-0002
