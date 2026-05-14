---
tags: [truth, architecture]
source: mixed
date: 2026-05-14
---
# The Reasoning Substrate

The dominant agent architecture in 2026 reduces cognitive work to one ingredient: prompt in, tokens out. The substrate is one large language model; differences between agents are differences of prompt and tool wiring.

This is sufficient when the answer is well-typed by surrounding text. It is insufficient when the answer is well-typed by a search space, a constraint set, a probability distribution, an authorisation lattice, or a body of formal mathematics.

A Formation is a different commitment. A Formation is an assembly of specialised reasoning agents — Suggestors — around a shared Context. Each Suggestor produces typed `ProposedFact`s. A Convergence Kernel arbitrates promotion. Agents exchange structured proposals with provenance, not prose.

The Mosaic Extensions supply Suggestor implementations grounded in eight distinct formal traditions. Together they cover the surface area of decisions a Formation must make.

## The eight modes

**Ferrox-solvers.** LP, MIP, and constraint programming via OR-Tools and HiGHS. Returns optimality and infeasibility certificates, not opinions.

**Prism-analytics.** Statistical learning (Polars, Burn) plus fuzzy inference (Mamdani 1975, Sugeno 1985, Tsukamoto) with named membership functions and inspectable defuzzification.

**Crucible-models.** Trained-artifact packs: support vector machines (Cortes–Vapnik 1995), ensembles (Breiman 1996, Freund–Schapire 1997), and ANFIS neuro-fuzzy hybrids (Jang 1993).

**Arbiter-policy.** Cedar policy decisions at runtime; `cedar-policy-symcc` for symbolic analysis at deploy time. Runtime says *deny now*; symbolic says *this class of breach is impossible*.

**Soter-SMT.** Bounded symbolic search via CVC5 — maintained jointly by Clark Barrett's group at Stanford University and Cesare Tinelli's group at the University of Iowa, with a lineage reaching back through CVC4, CVC3, and the original CVC of the early 2000s. Asks *can any counterexample exist?* Returns `sat` with a witness, `unsat` with a hashed certificate, or `unknown`. Results are `Searched` evidence, not `Verified` — the verifier tier (Lean 4, Coq, Agda) is held in reserve for compliance-pull.

**Mnemos-knowledge.** Vector KB with HNSW indexing, Reflexion (Shinn et al. 2023), causal hypergraph memory, MAML/Reptile few-shot adaptation, Elastic Weight Consolidation (Kirkpatrick et al. 2017) for drift.

**Manifold-adapters.** Capability registry: contract matching as subtyping. Refuses to route an EU-classified request to a non-EU provider.

**Embassy-ports.** Typed `Observation<T>` schemas with content-addressable hashing. A LinkedIn profile is a distinct type from a Twitter profile even when their text content overlaps.

## A Formation in motion

```
                       +---------------------------------+
                       |       Convergence Kernel        |
                       | promotion | typed provenance DAG|
                       +----------------+----------------+
                                        ^
                                        v
                            +-----------+-----------+
                            |    Shared Context     |
                            +-----------+-----------+
                                        ^
       +--------+--------+--------+-----+------+--------+--------+
       v        v        v        v            v        v        v
    Prism    Cruc.   Arbiter   Soter        Ferrox   Mnemos    LLM
    fuzzy    SVM /    Cedar    CVC5         LP / MIP  vector   (via
    + ML     ANFIS   + symcc   SMT          + CP     + RL meta Manif)
    Obs.     Obs.    Decided  Searched     Searched  Argued    Argued
                                |
                       Verified tier:
                       Lean / Coq / Agda
                       (deferred)

           +--------------------+    +--------------------+
           | Manifold-adapters  |    |   Embassy-ports    |
           | capability routing |    | typed observations |
           +----------+---------+    +----------+---------+
                      |                         |
                      v                         v
                external providers        external sources
```

## The claim

Above the eight extensions, the Convergence Kernel holds the run loop, the promotion authority, and the typed provenance vocabulary. It distinguishes *the LLM thinks* from *the policy engine has decided* from *the optimiser has proved* from *the SMT solver found no counterexample*.

An agent is not a language model with tools bolted on. An agent is a Suggestor with a formal grounding and a typed proposal protocol. A Formation is a particular composition of Suggestors that converges on a decision in a particular domain.

Foundation models are an excellent ingredient. They are not, by themselves, the substrate. The Mosaic Extensions are.

---

For the full version with mathematical citations, business scenarios per crate, and an extended trace through the €25,000-invoice Formation, see `~/dev/reflective/stack/mosaic-extensions/kb/Architecture/Pluralist Reasoning Substrate.md`.
