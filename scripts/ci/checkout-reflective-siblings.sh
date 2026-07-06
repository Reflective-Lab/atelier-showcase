#!/usr/bin/env bash
set -euo pipefail

workspace="${GITHUB_WORKSPACE:-$(git rev-parse --show-toplevel)}"

# REFLECTIVE_SIBLING_REF controls which branch siblings are cloned from (default: main).
# Validate to only allow chars legal in git branch names; reject anything else so a
# malformed value cannot be used for argument injection in the git clone call below.
_raw_ref="${REFLECTIVE_SIBLING_REF:-main}"
if [[ ! "$_raw_ref" =~ ^[a-zA-Z0-9/._-]+$ ]]; then
  echo "warning: REFLECTIVE_SIBLING_REF='${_raw_ref}' contains disallowed characters; using main" >&2
  _raw_ref=main
fi
REFLECTIVE_SIBLING_REF="$_raw_ref"
unset _raw_ref

checkout_reflective_repo() {
  local repo="$1"
  local relative_path="$2"
  local dest="${workspace}/${relative_path}"
  local sibling_ref="${REFLECTIVE_SIBLING_REF:-main}"

  if [[ -d "$dest/.git" ]]; then
    echo "ok: ${relative_path} already checked out"
    return
  fi

  if [[ -e "$dest" ]]; then
    echo "error: ${dest} exists but is not a git checkout" >&2
    exit 1
  fi

  mkdir -p "$(dirname "$dest")"

  if [[ "$sibling_ref" != "main" ]]; then
    echo "==> checkout Reflective-Lab/${repo}@${sibling_ref} -> ${relative_path}"
    if GIT_TERMINAL_PROMPT=0 git clone --depth=1 --branch "$sibling_ref" --quiet \
        "https://github.com/Reflective-Lab/${repo}.git" "$dest" 2>/dev/null; then
      return
    fi
    echo "    (branch '${sibling_ref}' not found; falling back to main)"
    rm -rf "$dest"
  fi

  echo "==> checkout Reflective-Lab/${repo}@main -> ${relative_path}"
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
