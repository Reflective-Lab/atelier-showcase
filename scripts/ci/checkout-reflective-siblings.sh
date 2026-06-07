#!/usr/bin/env bash
set -euo pipefail

workspace="${GITHUB_WORKSPACE:-$(git rev-parse --show-toplevel)}"

checkout_reflective_repo() {
  local repo="$1"
  local relative_path="$2"
  local dest="${workspace}/${relative_path}"

  if [[ -d "$dest/.git" ]]; then
    echo "ok: ${relative_path} already checked out"
    return
  fi

  if [[ -e "$dest" ]]; then
    echo "error: ${dest} exists but is not a git checkout" >&2
    exit 1
  fi

  mkdir -p "$(dirname "$dest")"
  echo "==> checkout Reflective-Lab/${repo} -> ${relative_path}"
  GIT_TERMINAL_PROMPT=0 git clone --depth=1 --quiet "https://github.com/Reflective-Lab/${repo}.git" "$dest"
}

checkout_reflective_repo converge ../bedrock-platform/converge
checkout_reflective_repo axiom ../bedrock-platform/axiom
checkout_reflective_repo organism ../bedrock-platform/organism
checkout_reflective_repo helms ../bedrock-platform/helms
checkout_reflective_repo arbiter-policy ../mosaic-extensions/arbiter-policy
checkout_reflective_repo embassy-ports ../mosaic-extensions/embassy-ports
checkout_reflective_repo ferrox-solvers ../mosaic-extensions/ferrox-solvers
checkout_reflective_repo manifold-adapters ../mosaic-extensions/manifold-adapters
checkout_reflective_repo mnemos-knowledge ../mosaic-extensions/mnemos-knowledge
checkout_reflective_repo prism-analytics ../mosaic-extensions/prism-analytics
checkout_reflective_repo soter-smt ../mosaic-extensions/soter-smt
checkout_reflective_repo arena-tests ../arena-tests
checkout_reflective_repo runtime-runway ../runtime-runway
checkout_reflective_repo commerce-rails ../commerce-rails
