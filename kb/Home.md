---
tags: [moc]
source: mixed
---
# atelier — Knowledge Base

Knowledge base for the `atelier` Converge extension.

**Standard:** Every release follows the
[Extension Release Checklist](https://github.com/Reflective-Lab/converge/blob/main/kb/Standards/Extension%20Release%20Checklist.md).

**For scenario authors:** read `~/dev/reflective/stack/mosaic-extensions/kb/Capability Matrix.md` *before* designing a new showcase. atelier exists to show specificity — every scenario should name the exact Mosaic functions it pulls and why a generic substitute (LLM call, hand-rolled `if`, single solver) is insufficient. Showcases that don't name specific Mosaic capabilities are off-mission.

**No-theatre rule:** a scenario is only ambitious if it changes the assurance
story. Module count is not enough. The README must name the customer outcome,
the typed evidence chain, the baseline that fails, and the pressure finding
that feeds back into Mosaic or this showcase.

**Resource declaration rule:** atelier-showcase defaults to `REAL LIVE` when an
example claims to exercise an external provider or Mosaic source-observation
path. `LOCAL REAL` is acceptable for real local solvers, policy engines, and
pure Rust product logic. `CONTRACT-SHAPE`, `SIMULATED`, and fake-backed `MIXED`
runs belong in `arena-tests` unless explicitly tracked as migration debt.
Importing a real Mosaic crate is not enough; the declaration must name the
actual backend that decides the result. Canonical version of this rule:
[`~/dev/reflective/stack/mosaic-extensions/kb/Standards/Real-by-Default Connections.md`](../../mosaic-extensions/kb/Standards/Real-by-Default%20Connections.md)
— REAL default, `--mock-ok` opt-in for CLI scenarios, loud failure on missing
API key. Binding on every v1.1.0 scenario.

**LLM provider boundary:** atelier does not choose LLM vendors directly.
Runnable examples that need chat models use Manifold provider APIs; operator
configuration chooses the provider.

**Meta:** [[INDEX]] — entity catalog | [[LOG]] — mutation log

## Philosophy

- [[Philosophy/From Instructions to Intent]] — the higher-order frame: why this work exists
- [[Philosophy/Systems of Outcome]] — how this reshapes SaaS, architecture, and ownership

## Architecture

- [[Architecture/Surface]] — public crate surface and contract shape
- [[Architecture/Algorithmic Backbone]] — SAT/UNSAT, SMT, LP, MIP, CP-SAT, network flow, predicate logic; what each scenario formally proves or optimizes, and why LLMs cannot substitute
- [[Architecture/Storage Memory and Analytics]] — why memory-backed showcase slices use `converge-storage` object stores plus Polars/Prism before reaching for a time-series DB

## Building

- [[Building/Getting Started]]
- [[Building/Release Commands]] — `just security-audit`, `coverage`, `performance-profile`, `soak`

## Standards

- [[Standards/Code Review]] — review drift through pinned platform versions and typed protocol boundaries
- [[Standards/Example Resource Declarations]] — required live/stub/mock declarations for runnable examples

## Planning

- [[Planning/MILESTONES]]

## History

- [[History/CHANGELOG]]
