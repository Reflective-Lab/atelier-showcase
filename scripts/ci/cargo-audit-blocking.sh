#!/usr/bin/env bash
set -euo pipefail

# Keep this list in sync with the release-grade security-audit recipe. Each
# ignore below is an explicit accepted transitive advisory while upstream
# owners move.
#
# 2026-07-02: RUSTSEC-2026-0187 (lopdf 0.38 stack overflow, fix >=0.42) and
# RUSTSEC-2026-0192 (ttf-parser unmaintained, pulled in by lopdf) are pinned
# transitively via pdf-extract 0.10 -> organism-intelligence; no fixed
# version reachable until pdf-extract moves to lopdf >=0.42.
#
# 2026-07-02: RUSTSEC-2026-0194/-0195 (quick-xml 0.38.4 DoS-class advisories,
# fix >=0.41) are pinned transitively via object_store 0.12.5, itself
# semver-locked by lancedb/surrealdb; no fix path until they move.
cargo audit --deny warnings \
  --ignore RUSTSEC-2023-0089 \
  --ignore RUSTSEC-2024-0384 \
  --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2025-0012 \
  --ignore RUSTSEC-2025-0134 \
  --ignore RUSTSEC-2021-0141 \
  --ignore RUSTSEC-2025-0141 \
  --ignore RUSTSEC-2025-0119 \
  --ignore RUSTSEC-2026-0002 \
  --ignore RUSTSEC-2026-0187 \
  --ignore RUSTSEC-2026-0192 \
  --ignore RUSTSEC-2026-0194 \
  --ignore RUSTSEC-2026-0195
