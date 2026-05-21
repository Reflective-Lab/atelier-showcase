---
tags: [moc]
source: mixed
---
# atelier — Knowledge Base

Knowledge base for the `atelier` Converge extension.

**Standard:** Every release follows the
[Extension Release Checklist](https://github.com/Reflective-Lab/converge/blob/main/kb/Standards/Extension%20Release%20Checklist.md).

**For scenario authors:** read `~/dev/reflective/stack/mosaic-extensions/kb/Capability Matrix.md` *before* designing a new showcase. atelier exists to show specificity — every scenario should name the exact Mosaic functions it pulls and why a generic substitute (LLM call, hand-rolled `if`, single solver) is insufficient. Showcases that don't name specific Mosaic capabilities are off-mission.

**Meta:** [[INDEX]] — entity catalog | [[LOG]] — mutation log

## Philosophy

- [[Philosophy/From Instructions to Intent]] — the higher-order frame: why this work exists
- [[Philosophy/Systems of Outcome]] — how this reshapes SaaS, architecture, and ownership

## Architecture

- [[Architecture/Surface]] — public crate surface and contract shape
- [[Architecture/Algorithmic Backbone]] — SAT/UNSAT, SMT, LP, MIP, CP-SAT, network flow, predicate logic; what each scenario formally proves or optimizes, and why LLMs cannot substitute

## Building

- [[Building/Getting Started]]
- [[Building/Release Commands]] — `just security-audit`, `coverage`, `performance-profile`, `soak`

## Standards

- [[Standards/Code Review]] — review drift through pinned platform versions and typed protocol boundaries

## Planning

- [[Planning/MILESTONES]]

## History

- [[History/CHANGELOG]]
