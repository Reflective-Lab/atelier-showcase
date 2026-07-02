# Claude Code Entrypoint

Read and follow `AGENTS.md` — it is the canonical project documentation.
Open work lives in Linear (team `RFL`, label `module:atelier-showcase`).

## Session Scope

- **Changelog:** `kb/History/CHANGELOG.md`
- **Standard:** [Extension Release Checklist](https://github.com/Reflective-Lab/converge/blob/main/kb/Standards/Extension%20Release%20Checklist.md) — the engineering bar every release must meet

## Claude-Specific Notes

- Prefer Edit over Write for existing files. Prefer Grep/Glob over Bash for search.
- Knowledge belongs in `kb/`, not as doc comments in source.
- Run `just lint` before considering work done.
- Run `just release-check` before tagging a release. All five gates must be green.
- Never push to main without confirmation.
