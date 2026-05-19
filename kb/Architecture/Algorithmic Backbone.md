---
tags: [architecture, algorithms, complexity, smt, sat, lp, mip, cp-sat]
source: llm
---
# Algorithmic Backbone

This page maps each atelier scenario to the formal problem class it
solves, the algorithm family that solves it, and the computational
complexity that makes the algorithm worth invoking. The point is
not to be exhaustive about the field — it is to make the *value* of
the external solvers concrete: each scenario poses a question that
sits in a complexity class where naïve enumeration is infeasible
and approximate methods (LLMs, heuristics, manual reasoning) cannot
produce a defensible guarantee.

The vocabulary in this document — **SAT, UNSAT, predicate logic,
linear program, mixed-integer program, constraint satisfaction,
SMT, decision procedure** — is the language of theoretical computer
science as it applies to practical optimization and verification.
The atelier scenarios are concrete instances; this page explains
what they are instances *of*.

---

## 1. The complexity landscape

Computational problems sort into classes by how much resource (time,
space, oracle queries) any algorithm needs to solve them in the
worst case. The classes the atelier touches:

- **P** — problems decidable in polynomial time on a deterministic
  Turing machine. Linear programming sits here (Khachiyan's
  ellipsoid method, 1979; Karmarkar's interior-point algorithm,
  1984).
- **NP** — problems whose *yes* answers can be verified in
  polynomial time. SAT (Boolean satisfiability) is **NP-complete**
  by the Cook–Levin theorem (1971); a polynomial-time SAT solver
  would imply P = NP and decide a half-century-open question.
- **NP-hard** — at least as hard as every NP problem. MIP, generic
  CSP, job-shop scheduling with makespan, traveling salesman, and
  most combinatorial optimization sit here.
- **PSPACE / EXPTIME** — decidable but exponentially worse than NP
  in the worst case. Quantified Boolean formulae (QBF) live here;
  Cedar policies, by design, do **not**.
- **Undecidable** — no algorithm exists that solves every instance.
  Full first-order arithmetic (Gödel 1931, Church 1936, Turing 1936)
  and the halting problem are the canonical examples. Nonlinear
  integer arithmetic (NIA) is undecidable; SMT theories carefully
  avoid this fragment unless explicitly opted in.

The atelier scenarios collectively occupy:
**P** (LP), **NP-complete** (SAT / Sudoku / N-queens),
**NP-hard** (MIP / job-shop / multi-plan allocation),
**polynomially-solved-by-specialized-algorithm** (min-cost flow),
and **decidable-fragment-of-first-order-logic** (Cedar via SMT).

We do not touch the undecidable fragments. Every scenario's
question has a definite answer, given enough time. The interesting
engineering question is: *how much time?*

---

## 2. SAT and UNSAT — the atomic decision question

The Boolean satisfiability problem is the foundation. Given a
propositional formula

$$
\varphi(x_1, \ldots, x_n) \quad \text{over Boolean variables}
$$

does there exist an assignment $\sigma : \{x_1, \ldots, x_n\} \to
\{0, 1\}$ such that $\varphi(\sigma) = 1$?

- **SAT** — yes; the solver returns one such $\sigma$ as a witness.
- **UNSAT** — no; for every assignment $\sigma$, $\varphi(\sigma) =
  0$. This is a **proof of impossibility** — a universally
  quantified statement $\forall\sigma.\, \neg\varphi(\sigma)$ which
  the solver justifies via a refutation derivation.

The distinction matters: **UNSAT is a stronger guarantee than any
finite test suite can provide.** A million random inputs all
returning *false* is evidence, not proof. UNSAT is proof —
mathematical certainty that no input in the entire (potentially
infinite) space satisfies the formula.

Modern SAT solvers — **CDCL** (Conflict-Driven Clause Learning,
Marques-Silva & Sakallah 1996, and the GRASP/Chaff lineage) — solve
formulae with millions of variables in seconds on industrial
instances, despite the NP-completeness of the problem. They combine:

- **Unit propagation** (Davis–Putnam–Logemann–Loveland, 1962): if a
  clause has all but one literal falsified, the remaining literal
  must be true.
- **Conflict analysis**: when a contradiction is reached, learn a
  new clause that prevents the same conflict's recurrence.
- **VSIDS heuristic** (Variable State Independent Decaying Sum):
  branch on variables that have appeared in recent conflicts.
- **Restarts**: periodically discard the current search tree (but
  keep learned clauses) to escape unproductive subtrees.

Atelier scenarios that ride on SAT-flavored reasoning include the
CP-SAT instances below (Sudoku, N-queens, multi-plan, ft06), all of
which compile to a CDCL core inside OR-Tools' CP-SAT solver.

---

## 3. SMT — Satisfiability Modulo Theories

SMT extends SAT from pure propositional logic to **first-order logic
with built-in interpreted theories**. Instead of just $x_i \in
\{0,1\}$, variables now range over integers, reals, bit-vectors,
arrays, strings, datatypes, or fixed-finite types — and the
**theory** ($T$) interprets the symbols.

A formula like

$$
(x + y \le 10) \land (x \ge 0) \land (y \ge 0) \land (x = 2y)
$$

is **SAT modulo the theory of linear integer arithmetic (LIA)** —
the solver must find an integer assignment satisfying both Boolean
structure *and* the arithmetic. The decision problem is solved by
**CDCL(T)** — propositional CDCL augmented with a theory solver
that lazily filters Boolean models through the theory's decision
procedure.

The standard theories shipped by mainstream SMT solvers:

| Theory | Symbol | Decidability |
|---|---|---|
| Linear integer arithmetic (LIA) | $+, \le, =$ over $\mathbb{Z}$ | decidable (Presburger) |
| Linear real arithmetic (LRA) | $+, \le, =$ over $\mathbb{R}$ | decidable |
| Nonlinear integer arithmetic (NIA) | $+, \cdot, \le$ over $\mathbb{Z}$ | **undecidable** (Hilbert's 10th, Matiyasevich 1970) |
| Nonlinear real arithmetic (NRA) | $+, \cdot, \le$ over $\mathbb{R}$ | decidable (Tarski 1948), but doubly-exponential |
| Bit-vectors (BV) | fixed-width $\{0,1\}^k$ | decidable, NP-complete |
| Arrays | $\text{read}, \text{write}$ | decidable |
| Strings | concat, length, regex | decidable in fragments |
| Uninterpreted functions (EUF) | $f(x) = f(y)$ if $x = y$ | decidable, polynomial |

**SMT-LIB** is the standardized input format these solvers share.
A typical fragment:

```
(declare-const x Int)
(declare-const y Int)
(assert (>= x 0))
(assert (>= y 0))
(assert (= x (* 2 y)))
(assert (<= (+ x y) 10))
(check-sat)
(get-model)
```

The solver replies `sat` or `unsat`, and on SAT also produces a
**model** $\mathcal{M}$ — a concrete interpretation of the symbols
under which the formula evaluates true.

### Why an external C++ process

SMT decision procedures are research-heavy code: nonlinear-arithmetic
projection (CAD), bit-blasting + CDCL for BV, congruence closure for
EUF, Nelson–Oppen combination, theory propagation, model
construction. CVC5 (the cvc4/cvc5 lineage, since ~2002) embodies
decades of this work in highly optimized C++. The Rust application
boundary defers the hard math to an external process via the
standard SMT-LIB protocol — exactly the same architectural pattern
that drives:

- Compilers (LLVM IR → optimization passes)
- Theorem provers (Coq/Lean tactic frameworks call SMT)
- Static analyzers (CodeQL, Infer, KLEE)
- Symbolic execution engines (angr, Manticore)
- Cryptographic verification tools (Project Everest, F\*)

### The atelier scenario

[`scenario-cedar-smt-analysis`](../../scenarios/cedar-smt-analysis/)
takes a Cedar authorization policy + schema and asks **three
distinct model-theoretic questions** about it, dispatching each
through the pipeline

```
Cedar policy + schema
        ↓  cedar-policy-symcc compiles
SMT-LIB assertion set, one per request environment
        ↓  LocalCvc5AnalysisBackend invokes
CVC5 v1.3.3 (external process)
        ↓
SAT  → concrete witness (a permitted/denied request)
UNSAT → formal proof across the entire request space
```

The three queries:

- `ExpenseNonFinanceHighValueCommitDenied` — a **safety invariant**.
  Asks: $\exists \text{request } r$ such that $r$ matches the
  high-value-commit claim *and* the policy permits $r$? UNSAT
  proves no such $r$ exists, in any request environment.
- `AlwaysAllows` — a **liveness witness search**. UNSAT means every
  request is allowed (the policy is `permit *`); SAT returns a
  request the policy denies.
- `AlwaysDenies` — the dual. UNSAT means every request is denied
  (the policy is `forbid *`); SAT returns a request the policy
  permits.

Together they fully characterize the policy: it denies the dangerous
claim, isn't degenerately permissive, isn't degenerately
restrictive. The total SMT work is **21 assertions** dispatched in
**0.15 seconds**, covering a conservatively-bounded subspace of
**2.6 × 10¹⁵ inputs** (≈ 10¹⁵·⁴) which at 100 k tests/sec would
require **~830 years** of brute-force enumeration.

The compression ratio — 830 years → 0.15 seconds — is not an
implementation optimization. It is a categorical change in *how the
question is asked*: brute force enumerates points in the
configuration space; SMT reasons symbolically about constraints,
collapsing entire regions in a single inference step.

---

## 4. Predicate logic and first-order reasoning

The mathematical substrate underneath SMT is **first-order
predicate logic** (FOL). A formula in FOL has:

- **Constants** $a, b, c, \ldots$ — fixed individuals.
- **Variables** $x, y, z, \ldots$ — placeholders.
- **Function symbols** $f, g, h$ of various arities.
- **Predicate symbols** $P, Q, R$ — relations of various arities.
- **Logical connectives** $\neg, \land, \lor, \to, \leftrightarrow$.
- **Quantifiers** $\forall, \exists$.

A Cedar policy clause

```
permit(principal == User::"alice",
       action == Action::"read",
       resource in Folder::"engineering");
```

translates to a first-order formula roughly of the form

$$
\forall p, a, r.\; \big(p = \text{alice} \land a = \text{read}
 \land r \in \text{engineering}\big) \to \text{Permit}(p, a, r).
$$

**Decidability** is the central question of FOL applied to
verification. The full theory of FOL is undecidable (Church 1936),
but many useful fragments are decidable:

- **Propositional logic** — decidable, NP-complete (SAT).
- **Presburger arithmetic** — first-order theory of $\langle
  \mathbb{Z}, +, =, \le \rangle$ without multiplication; decidable
  in triply-exponential time, often much faster in practice.
- **Bernays–Schönfinkel / EPR** — formulae of the form
  $\exists^* \forall^* \varphi$ with no function symbols; decidable,
  used in many verification systems.
- **Cedar's static analyzability** — Cedar policies, by careful
  language design, fall into a fragment whose validity / equivalence
  / containment problems are decidable. This is why
  `cedar-policy-symcc` exists at all: the language was engineered so
  that SMT analysis is tractable.

The atelier `cedar-smt-analysis` scenario is a concrete instance of
applying *decidable-fragment FOL* reasoning to a production
authorization system. The "30,000-foot view": the policy is a
formula, the schema is a typing context, and the questions
(safety / liveness / equivalence / privilege escalation) are
sentences whose validity the solver decides.

---

## 5. Linear programming (LP)

A **linear program** is the optimization problem

$$
\begin{aligned}
\text{minimize} \quad & c^\top x \\
\text{subject to} \quad & A x \ge b \\
& x \ge 0
\end{aligned}
$$

where $x \in \mathbb{R}^n$, $c \in \mathbb{R}^n$, $A \in
\mathbb{R}^{m \times n}$, $b \in \mathbb{R}^m$. The feasible region
$\{x : Ax \ge b,\, x \ge 0\}$ is a convex polytope; the objective is
linear; therefore the optimum (if finite) is attained at a vertex of
the polytope.

LP is **in P**. Two main algorithm families:

- **Simplex method** (Dantzig 1947) — pivots between adjacent
  polytope vertices; exponential worst case (Klee–Minty 1972) but
  near-linear in practice.
- **Interior-point methods** (Karmarkar 1984; primal-dual
  predictor-corrector) — provably polynomial, $O(n^{3.5} L)$ or
  better.

The atelier [`scenario-lp-diet`](../../scenarios/lp-diet/) is the
canonical introductory LP: Stigler's diet problem (1945). $n = 8$
food variables, $m = 6$ nutrient constraints, one objective. The
optimum sits on the **intersection of active constraints** — in
this run, the vitamin-C, calcium, and iron lower bounds bind
simultaneously; the other three are slack. The simplex method
recognises this immediately; statistical or LLM-based "guessing"
generally finds feasible diets but rarely the cheapest one. The
solver returns the provably optimal $x^* = (11.769, 0, 0, 2.609,
0, 0.857, 0, 0)$ at cost $\$8.4859$/day, with the dual prices
(shadow values) attached for sensitivity analysis.

LP is in P. The reason to use a specialized solver (GLOP, CLP,
HiGHS-LP) is not theoretical complexity — it is numerical stability
and the decades of tuning that make a real-world LP with $10^6$
variables solve in seconds.

---

## 6. Mixed-integer programming (MIP)

A **mixed-integer program** adds integrality constraints to LP:

$$
\begin{aligned}
\text{minimize} \quad & c^\top x + d^\top y \\
\text{subject to} \quad & A x + B y \ge b \\
& x \ge 0,\; x \in \mathbb{R}^n,\; y \in \mathbb{Z}^m
\end{aligned}
$$

The integer variables $y$ make the problem **NP-hard**. The
canonical proof: 3-SAT reduces to 0-1 MIP via clause-encoding.

**Branch-and-bound** is the standard algorithmic approach:

1. Solve the **LP relaxation** — drop integrality, get $x^*_\text{LP}$.
2. If $y^*_\text{LP}$ is integer-feasible, done — LP relaxation
   equals integer optimum.
3. Otherwise pick a fractional $y_j$ and branch: explore the
   subproblem with $y_j \le \lfloor y_j^* \rfloor$ and the subproblem
   with $y_j \ge \lceil y_j^* \rceil$.
4. **Bound** each subproblem's LP relaxation; prune subtrees whose
   LP bound is worse than the incumbent integer solution.
5. Add **cutting planes** (Gomory cuts, mixed-integer rounding cuts,
   lift-and-project) to tighten relaxations.

The **MIP gap** is the relative distance between the best integer
solution found and the best LP-relaxation bound; when the gap closes
to 0, optimality is proved.

The atelier
[`scenario-mip-facility-location`](../../scenarios/mip-facility-location/)
encodes the classical **uncapacitated facility location problem**:

- 5 binary variables $y_w \in \{0,1\}$ (whether to open warehouse $w$).
- 5 × 8 = 40 continuous variables $x_{wc} \in [0, \text{capacity}_w]$
  (flow shipped from $w$ to customer $c$).
- 8 demand constraints $\sum_w x_{wc} = d_c$.
- 5 capacity-when-open constraints $\sum_c x_{wc} \le
  \text{capacity}_w \cdot y_w$ (big-M coupling).
- Linear objective: $\sum_w \text{open}_w \cdot y_w + \sum_{w,c}
  \text{ship}_{wc} \cdot x_{wc}$.

The LP relaxation produces fractional warehouse openings like
$y_\text{north} = 0.3$, useless operationally. HiGHS's branch-and-bound
closes the MIP gap to **zero** in milliseconds, producing the
integer optimum $\{y_\text{east} = 1, y_\text{central} = 1, \text{others}=0\}$
at total cost $\$13{,}294.20$.

The combinatorial structure: there are $2^5 = 32$ subsets of
warehouses to consider; for each subset that's feasible, there's a
transportation LP to solve. Brute force is tractable here, but the
same encoding scales to 100s of warehouses where $2^{100}$ subsets
is infeasible and only branch-and-bound's prune-by-bound makes it
solvable.

---

## 7. Constraint satisfaction and CP-SAT

A **constraint satisfaction problem** (CSP) is a triple

$$
\mathcal{P} = \langle V, D, C \rangle
$$

where $V = \{v_1, \ldots, v_n\}$ is a set of variables, $D = D_1
\times \cdots \times D_n$ is the product of their domains, and $C$ is
a set of constraints $c_i \subseteq D_{i_1} \times \cdots \times
D_{i_k}$ each restricting some subset of the variables. A solution
is an assignment $\sigma : V \to D$ such that every constraint is
satisfied. CSPs are NP-complete in general (3-SAT is a CSP with
domain $\{0,1\}$ and ternary clauses).

**CP-SAT** combines:

- **Constraint propagation** — algorithms like AC-3 (Mackworth 1977),
  GAC (Generalized Arc Consistency), and bounds-consistency that
  prune values from domains by deduction.
- **Backtracking search** — depth-first exploration with conflict
  detection.
- **Lazy clause generation** — translate constraint violations into
  no-good clauses, feed them to a CDCL SAT core, get all of
  conflict-driven learning's benefits.
- **Global constraints** — high-arity primitives (`AllDifferent`,
  `Cumulative`, `NoOverlap`, `Table`) with specialized propagators
  that achieve consistency in polynomial time per constraint.

OR-Tools' CP-SAT solver (Laurent Perron et al., ~2010 onwards) is
the open-source flagship. It is what every atelier CP scenario
ultimately calls.

### Atelier CP-SAT scenarios

- [**`scenario-sudoku-cp-sat`**](../../scenarios/sudoku-cp-sat/)
  — 81 integer variables with domain $\{1, \ldots, 9\}$, 27
  `AllDifferent` constraints (9 rows + 9 columns + 9 3×3 boxes),
  ~17 fixed-value `LinearEq` clue constraints. Encoding directly
  mirrors the mathematical statement of the problem. The "AI
  Escargot" puzzle (Arto Inkala 2006) is engineered to maximize
  backtracking depth; CP-SAT propagation collapses it in **44 ms**.

- [**`scenario-n-queens-cp-sat`**](../../scenarios/n-queens-cp-sat/)
  — $N = 50$ queens. Decision variables $q_r \in \{0, \ldots, N-1\}$
  with auxiliary variables $d_r = q_r + r$ (anti-diagonal) and
  $e_r = q_r - r$ (diagonal). Three `AllDifferent`
  constraints over $\{q_r\}, \{d_r\}, \{e_r\}$ enforce no two
  queens share a column, anti-diagonal, or diagonal respectively.
  The search space is $50! \approx 3 \times 10^{64}$ raw
  placements; CP-SAT proves the optimal placement in **79 ms**.

- [**`scenario-jobshop-ft06`**](../../scenarios/jobshop-ft06/) —
  Fisher–Thompson 6×6 benchmark, 1963. Six jobs, each a totally
  ordered sequence of six operations on six machines.
  Mathematical statement: minimize makespan $\max_o \text{end}(o)$
  subject to precedence-within-job constraints and per-machine
  resource constraints. Encoded via CP-SAT's
  `IntervalVarDef` + `NoOverlap` (the canonical scheduling
  primitives). The **proven optimum is 55**; ferrox matches it
  with proven lower-bound 55 in **11 ms**. Greedy heuristics
  routinely produce 70–80 on this instance.

- [**`scenario-multi-plan-allocation`**](../../scenarios/multi-plan-allocation/)
  — 6 binary decision variables (pick / don't pick each plan)
  with linear constraints (exactly $K$ picked, total cost $\le
  \text{budget}$, capability set covered, $\le 1$ high-risk plan)
  and a linear objective (maximize total value). The "exactly K
  of N under linear constraints" formulation is a classic CP
  benchmark; CP-SAT solves the 6-element instance in 7 ms but the
  encoding scales to 10⁵ items with the same Suggestor.

---

## 8. Min-cost network flow

Some structured combinatorial problems admit **polynomial-time
specialized algorithms** despite being expressible as MIPs. Min-cost
flow is the most prominent.

A **min-cost flow network** is a directed graph $G = (V, E)$ with:

- A capacity $u_e \in \mathbb{Z}_{\ge 0}$ on each arc $e$.
- A unit cost $c_e \in \mathbb{Z}$ on each arc $e$.
- A supply $b_v \in \mathbb{Z}$ at each node $v$ ($b_v > 0$ for
  sources, $b_v < 0$ for sinks, $b_v = 0$ for transshipment nodes),
  with $\sum_v b_v = 0$ in balanced mode.

Find a flow $f : E \to \mathbb{Z}_{\ge 0}$ minimizing $\sum_e c_e
f_e$ subject to capacity ($f_e \le u_e$) and flow conservation
($\sum_{e=(u,v)} f_e - \sum_{e=(v,w)} f_e = -b_v$).

The constraint matrix of min-cost flow is **totally unimodular**
(TU); this property guarantees that the LP relaxation has integer
optimal vertices, so the IP and LP optima coincide. The **network
simplex algorithm** (Dantzig 1951) and its modern variants solve
min-cost flow in strongly polynomial time — $O(VE \log V)$ for
modern push-relabel + scaling implementations.

The atelier
[`scenario-network-flow-transport`](../../scenarios/network-flow-transport/)
encodes a 3-echelon supply chain: 3 plants → 4 distribution centers
→ 6 retailers, 13 nodes, 17 capacitated arcs. Balanced supply 176
units = demand 176. OR-Tools' network simplex finds the integer
optimum $\$1{,}534$ in milliseconds.

The lesson the scenario makes concrete: **structural insight pays
off**. The same problem solved as a generic LP or MIP would work,
but the specialized algorithm runs orders of magnitude faster and
gives provably integer answers without branch-and-bound.

---

## 9. Where nonlinear programming sits

The atelier does **not** currently include a nonlinear program
scenario, but the boundary is worth naming.

A **nonlinear program** (NLP):

$$
\begin{aligned}
\text{minimize} \quad & f(x) \\
\text{subject to} \quad & g_i(x) \le 0, \quad i = 1, \ldots, m \\
& h_j(x) = 0, \quad j = 1, \ldots, p
\end{aligned}
$$

where $f, g_i, h_j : \mathbb{R}^n \to \mathbb{R}$ are not all
linear.

- **Convex NLP** — when $f$ is convex and the feasible set is
  convex. Tractable via interior-point methods (Boyd & Vandenberghe
  2004), polynomial in $n$. Many real-world problems (portfolio
  optimization, least-squares with constraints) are convex.
- **Non-convex NLP** — generally **NP-hard**, with no efficient
  algorithm for global optima. Local optima are findable; the
  global guarantee is the missing piece. Mixed-integer nonlinear
  programming (MINLP) compounds this with combinatorial branching.

The CP-SAT, MIP, and LP scenarios cover the **linear and
combinatorial** end of optimization. NLP is adjacent — ferrox does
not currently expose an NLP backend, but the same Suggestor pattern
(seed typed request → solver → typed plan → self-validate) extends
to it cleanly if a backend is added (IPOPT, CasADi, NLopt).

---

## 10. Why these algorithms can't be replaced by LLMs

A recurring theme across the scenarios: **language models cannot
substitute for these solvers, even at scale.** The reason is
not the size of the model — it's the shape of the problem.

- **Counting and exact arithmetic.** LP / MIP / network flow rely
  on exact-rational or fixed-precision arithmetic to maintain
  feasibility on tight constraint boundaries. LLMs produce
  approximate numeric outputs; the failure mode on a constrained
  optimization problem is to produce a "plausible" answer that
  violates a constraint by a small amount no human noticed.

- **Combinatorial reasoning at scale.** SAT/CSP solvers exploit
  conflict-driven learning to prune exponentially. LLMs sample
  one token at a time without backtracking — the search procedure
  is qualitatively wrong for the problem class.

- **Universal claims.** UNSAT is a quantifier-alternation
  statement: $\forall \sigma . \neg \varphi(\sigma)$. A
  statistical predictor can be highly confident there's no
  counterexample without proving there is none. The Cedar SMT
  scenario shows the difference explicitly: testing 10⁶ random
  inputs gives a 99.999%-confident *empirical* conclusion;
  UNSAT gives **certainty**.

- **Worst-case instances.** Many of these problems are
  NP-complete or NP-hard. LLMs trained on natural-text
  distributions are unlikely to have seen adversarial worst-case
  instances and will fail on them in ways that won't generalize.
  The job-shop benchmarks (Fisher–Thompson ft06, Lawrence,
  Adams–Balas–Zawack) and the AI Escargot Sudoku exist precisely
  to expose this weakness.

- **Auditability.** The solvers emit **typed verdicts with
  witnesses**: a CP-SAT solution is a concrete assignment, a MIP
  plan carries the LP gap, an SMT counterexample is a satisfying
  model $\mathcal{M}$ that can be re-substituted into the formula
  and checked. LLM outputs are not auditable in this sense.

The atelier scenarios use LLMs **where they are appropriate** — the
round-driven design huddle uses LLM judgment for *deliberation*
(propose, critique, synthesize) — and use solvers **where they are
needed** — for the underlying optimization, satisfiability, or
verification problems. The boundary is sharp and intentional.

---

## 11. Scenario map

| Scenario | Problem class | Complexity | Solver | Wall time |
|---|---|---|---|---|
| [`lp-diet`](../../scenarios/lp-diet/) | Linear program | $P$ | GLOP (OR-Tools) | < 1 ms |
| [`mip-facility-location`](../../scenarios/mip-facility-location/) | Mixed-integer program | NP-hard | HiGHS branch-and-bound | < 100 ms |
| [`network-flow-transport`](../../scenarios/network-flow-transport/) | Min-cost flow | $O(VE \log V)$ | OR-Tools network simplex | < 10 ms |
| [`jobshop-ft06`](../../scenarios/jobshop-ft06/) | Job-shop scheduling | NP-hard | CP-SAT (interval + NoOverlap) | 11 ms |
| [`sudoku-cp-sat`](../../scenarios/sudoku-cp-sat/) | CSP (Sudoku) | NP-complete | CP-SAT (AllDifferent) | 44 ms |
| [`n-queens-cp-sat`](../../scenarios/n-queens-cp-sat/) | CSP (N-queens) | NP-complete (as CSP) | CP-SAT (AllDifferent) | 79 ms |
| [`multi-plan-allocation`](../../scenarios/multi-plan-allocation/) | 0-1 IP / CSP | NP-hard | CP-SAT | 7 ms |
| [`cedar-smt-analysis`](../../scenarios/cedar-smt-analysis/) | First-order theory (Cedar) | decidable | CVC5 (SMT-LIB) | 0.15 s for 3 queries |
| [`arbiter-governance`](../../scenarios/arbiter-governance/) | Typed gate evaluation | $P$ | pure Rust | < 1 ms |
| [`round-driven-formation-design`](../../scenarios/round-driven-formation-design/) | Deliberation + selection | mixed | LLM + CP-SAT | seconds |

All times measured on Apple Silicon (M-series) hardware with the
vendored solver binaries (OR-Tools v9.15, HiGHS v1.14.0, CVC5 v1.3.3)
linked through `ferrox-ortools-sys`, `ferrox-highs-sys`, and
`cedar-policy-symcc` respectively.

---

## 12. Where to read further

- **SAT / CDCL**: Biere, Heule, van Maaren, Walsh (eds.),
  *Handbook of Satisfiability*, 2nd ed., IOS Press, 2021.
- **SMT**: Barrett, Sebastiani, Seshia, Tinelli, "Satisfiability
  Modulo Theories" in the *Handbook of Satisfiability*, 2021.
  SMT-LIB standard documents at <https://smtlib.cs.uiowa.edu>.
- **LP / MIP**: Bertsimas & Tsitsiklis, *Introduction to Linear
  Optimization*, Athena, 1997. Schrijver, *Theory of Linear and
  Integer Programming*, Wiley, 1986 (the canonical reference).
- **CP / CP-SAT**: Rossi, van Beek, Walsh (eds.), *Handbook of
  Constraint Programming*, Elsevier, 2006.
- **Network flow**: Ahuja, Magnanti, Orlin, *Network Flows*,
  Prentice Hall, 1993.
- **Cedar symbolic analysis**: Pasareanu et al.,
  "cedar-policy-symcc": <https://github.com/cedar-policy/cedar-spec>.

The atelier scenarios are concrete, runnable instances of every
problem class touched in these references.
