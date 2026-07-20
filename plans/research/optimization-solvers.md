# Research Question: Exact Optimization and Solver Policy · `q:optimization:solvers`

> **Status:** Open; exact prototype required before external solver adoption
> **Blocks:** solver-backed proof selection, placement, large calibration
> searches, and any optimality claim beyond exhaustive pilot analysis
> **Does not block:** exact finite pilot planning with deterministic
> enumeration or branch-and-bound
> **Affected packages:** compiler, linker, vectors, release
> **Depends on:** (`dec:assurance:translation-validation`),
> (`dec:abi:canonical-transactions`)
> **Imports:** (`[ADR011-rule:toolchain:dependencies]`),
> (`[ADR011-rule:toolchain:reproducibility]`),
> (`[RZ-rem:overview:conformance-claim]`),
> (`[RZ-rule:translation:certificate-leaf]`)
> **Related research:** (`q:compiler:algorithms`),
> (`q:linker:algorithms`),
> (`q:numerical:linear-algebra`)
> **Expected handoff:** exact pilot optimizer, complexity thresholds,
> reviewed solver candidates, canonical tie-breaking, and independently checked
> feasibility/optimality certificates

## Question · `sec:optimization:question`

When should compiler and linker planning use first-party exact finite search,
SAT, LP, MILP, QP, or another optimizer, and how can any selected result be
checked independently enough to affect bundle, ABI, calibration, or release
identity?

The initial problems include:

- proof-alternative selection;
- relation placement;
- carrier selection;
- representation-plan selection;
- resource-aware layout choice;
- integer deployment-bound calibration.

These are not ordinary linear systems.

`faer` solves numerical matrix equations and decompositions. It does not replace
an optimization solver.

## Problem classes · `rule:optimization:classes`

### Exact finite selection

Choose one item from each finite set under typed hard constraints.

Examples:

- one proof alternative per relation;
- one or more carriers per execution case;
- one representation plan per operation.

Initial solution: exact deterministic enumeration or branch-and-bound.

### Boolean satisfiability

Variables satisfy:

\[
x_i\in\{0,1\}
\]

and constraints are Boolean.

A SAT solver may later establish feasibility.

### Linear programming

Variables are continuous and constraints/objective are linear:

\[
\min c^\top x\quad\text{subject to}\quad Ax\le b
\]

An LP solver is appropriate.

### Mixed-integer linear programming

Some variables are integral or Boolean.

This may express proof selection, placement, layout, and integer bounds, but
introduces a larger solver and certification boundary.

### Quadratic or conic optimization

A convex quadratic or conic solver is appropriate only if the actual objective
or constraints require that class.

Do not recast a finite exact problem as floating optimization merely because a
numerical library is available.

### Specialized algorithms

Some problems have better dedicated algorithms:

- graph reachability and SCCs → `petgraph`;
- bounded-depth taptree construction → package-merge;
- exact linear equations → Bareiss/rational elimination;
- monotone scalar bound search → binary search after a proof of monotonicity.

A generic solver is not used where a simpler exact specialized algorithm
matches the problem.

## Hard constraints and objectives · `rule:optimization:hard-constraints`

The following are hard constraints:

- semantic relation preservation;
- authorization;
- constructibility;
- lifecycle;
- closed-asset identity;
- required disclosure;
- relation coverage;
- target capability;
- target resource limits;
- required evidence carriers;
- candidate/final state discipline.

They are never represented as finite penalties.

An objective applies only after every hard constraint is satisfied.

Possible objective components include:

- script bytes;
- witness bytes;
- transaction weight;
- stack;
- crypto budget;
- expected control-path depth;
- proof complexity;
- deliberate duplicate enforcement.

Unlike units remain a Pareto vector unless release policy defines an explicit
integer ordering or conversion.

## Initial exact algorithm · `rule:optimization:exact-search`

The initial production optimizer is first-party exact finite search.

### Deterministic enumeration

- assign every decision variable a stable typed key;
- enumerate candidates in stable-key order;
- validate hard constraints incrementally;
- prune impossible partial assignments;
- retain provenance for each rejection;
- evaluate objective only for feasible complete plans;
- apply canonical tie-breaking.

### Branch-and-bound

Use branch-and-bound when an admissible lower bound can prune objective search.

The bound must never exclude a feasible better solution.

The implementation records:

- search states visited;
- branches pruned by hard constraint;
- branches pruned by objective bound;
- best objective;
- canonical selected plan;
- complexity-limit status.

### Pareto frontier

When no accepted total ordering exists, retain the nondominated frontier.

For objective vectors \(u\) and \(v\), \(u\) dominates \(v\) only if it is no
worse in every dimension and strictly better in at least one.

Target or deployment policy selects from the frontier under an explicit
versioned policy.

## Canonical tie-breaking · `rule:optimization:canonical-ties`

A feasible or optimal set may contain several mathematically equivalent
solutions.

Canonical selection uses:

1. primary exact objective vector;
2. explicit secondary objective vector;
3. lexicographic stable-key assignment vector.

Solver model order, hash iteration, thread scheduling, pivot order, and
incidental variable numbering do not determine the selected plan.

If an external solver is introduced, canonicalization occurs independently of
its first returned model.

## Complexity limits · `rule:optimization:limits`

Every exact search declares:

- number of variables;
- domain size per variable;
- number of constraints;
- maximum search states;
- maximum retained Pareto candidates;
- maximum wall-independent work unit where practical;
- memory limit;
- failure behavior.

Exceeding the limit returns a typed complexity failure.

It must not:

- select a greedy fallback;
- drop a relation;
- weaken a hard constraint;
- silently accept the best plan seen before exhaustion;
- claim optimality.

A caller may explicitly request a feasible nonoptimal plan under a separate
policy, but that status must be visible in type and identity.

## Proof selection model · `rule:optimization:proof-selection`

For each semantic relation, candidate proof alternatives carry:

- proof identity;
- target capabilities;
- fact sources;
- witnesses;
- constructibility;
- representation;
- disclosure;
- lifecycle;
- relation carriers;
- exact resource vector.

The optimizer may select an alternative only after all non-cost constraints
validate.

Shared proof patterns and facts create global interactions, so choosing the
cheapest proof independently for each relation is not generally correct.

The exact pilot optimizer must demonstrate cases where greedy per-relation
selection is suboptimal or invalid.

## Placement model · `rule:optimization:placement`

Placement variables represent relation-to-carrier assignment.

Constraints require:

- every active relation covered;
- every required execution case covered;
- carrier reachable in that case;
- carrier has every authenticatable source fact;
- unconditional relation not confined to an optional carrier;
- transaction-global relation has a complete global carrier;
- deliberate duplicate enforcement remains consistent.

The objective may minimize resource cost or duplicate checks only after these
constraints hold.

An exhaustive carrier-subset oracle validates generated small instances.

## Calibration model · `rule:optimization:calibration`

Deployment bounds are exact integers.

Binary search is used only after proving candidate monotonicity.

If candidate policy changes with the bound, calibration becomes a discrete
optimization problem over:

- bound assignment;
- selected proof plan;
- layout;
- linked bundle;
- ABI;
- valid worst-case transactions;
- measured target resources.

Initial calibration may enumerate a finite candidate set rather than introduce
MILP.

After selection, the exact final bundle and ABI are rebuilt and remeasured.

## Excluded specialized problem · `rule:optimization:taptree`

Taptree construction is not delegated to a general optimizer initially.

The selected dedicated algorithm is investigated under
(`q:linker:algorithms`).

A general optimizer may later serve as an independent small-instance oracle,
not as the first production tree builder.

## Solver candidates · `tbl:optimization:candidates`

| Candidate | Class | Initial posture |
|---|---|---|
| first-party enumeration | finite exact | preferred for pilots |
| first-party branch-and-bound | finite exact | preferred when objective pruning is useful |
| `varisat` | SAT | future Boolean feasibility candidate |
| `good_lp` | modeling frontend | future LP/MILP candidate; backend-specific review required |
| pure-Rust LP backend | LP | possible future candidate |
| `clarabel` | convex/conic | only if a real convex problem appears |
| HiGHS binding | LP/MILP | future large-scale candidate; native dependency |
| Z3 | SMT | optional research oracle, not initial production dependency |
| `faer` | linear equations | numerical subproblems only; not an optimizer |
| exhaustive independent oracle | finite exact | required for small instances |

No solver is accepted by package popularity alone.

## External solver review · `rule:optimization:dependency-review`

Before an external solver becomes production-load-bearing, review:

- exact crates.io version and source;
- license;
- Rust 1.88 support;
- pure Rust versus native/FFI boundary;
- unsafe code and system dependencies;
- deterministic mode;
- parallelism;
- random seeds;
- exact versus floating arithmetic;
- feasibility/optimality tolerance;
- model-order dependence;
- timeout behavior;
- certificate/proof availability;
- transitive graph and advisories;
- release-platform availability;
- result reproducibility;
- replacement/migration path.

A native solver also requires:

- build reproducibility;
- platform packaging;
- exact native library identity;
- FFI failure policy;
- release report provenance.

## LP certification · `rule:optimization:lp-certificates`

A floating-point LP status is not sufficient release proof.

For a release-sensitive LP result, retain a primal and dual candidate under a
documented formulation.

Exact certification verifies:

- primal feasibility;
- dual feasibility;
- variable domains;
- exact objective values;
- equality of primal and dual objectives where strong duality applies.

The exact verifier uses integer/rational arithmetic.

The solver may generate a candidate numerically. The exact certificate checker
decides acceptance.

If rational reconstruction fails, the result remains diagnostic or is rejected.

## MILP certification · `rule:optimization:milp-certificates`

For a mixed-integer result:

- integrality is checked exactly;
- primal feasibility is checked exactly;
- selected plan is independently revalidated against typed compiler/linker
  constraints.

Optimality requires one of:

- exhaustive search for accepted small instances;
- an independently checked branch-and-bound certificate;
- another reviewed proof of the objective lower bound;
- an explicitly nonoptimal feasible status.

A solver’s unverified “optimal” status must not silently become release
identity.

## SAT certification · `rule:optimization:sat-certificates`

A SAT model is independently evaluated against the complete typed Boolean
constraint set.

For unsatisfiability, release-sensitive reliance requires:

- a checked proof format where supported; or
- an independent exact finite result for the accepted problem size; or
- a separately scoped trust decision.

Solver variable numbering is local and never semantic identity.

## Numerical subproblems · `rule:optimization:numerical-subproblems`

`faer` may support:

- numerical relaxations;
- least-squares diagnostics;
- sensitivity;
- candidate lower-bound estimates where conservatively certified;
- matrix computations inside an optimizer prototype.

It does not by itself certify:

- integer feasibility;
- Boolean coverage;
- exact equality;
- semantic optimality;
- release-valid calibration.

Numerical policies follow (`q:numerical:linear-algebra`).

## Prototype · `sec:optimization:prototype`

### Stage 1 — typed finite model

Define typed variables, domains, hard constraints, objective vectors, and
canonical assignments for the two compiler pilots.

### Stage 2 — exhaustive oracle

Enumerate every complete assignment for small instances.

Return:

- all feasible assignments;
- Pareto frontier;
- canonical selected assignment;
- exact objective values.

### Stage 3 — branch-and-bound

Implement deterministic pruning and compare exactly with exhaustive results.

### Stage 4 — adversarial greedy examples

Construct cases where:

- cheapest local proofs produce an invalid global plan;
- cheapest local placement leaves one execution case uncovered;
- sharing makes a more expensive local proof globally cheaper;
- a lifecycle-safe plan differs from the cheapest immediate plan;
- tied optimums require canonical stable-key selection.

### Stage 5 — complexity scaling

Generate larger synthetic proof and placement problems.

Measure the point at which exact search exceeds the accepted budget.

### Stage 6 — external solver comparison

Only if Stage 5 demonstrates a need, prototype one pure-Rust candidate and one
industrial/native candidate where appropriate.

Compare:

- feasible set;
- optimum;
- canonicalized result;
- certificate verification;
- build and runtime complexity.

### Stage 7 — calibration candidate

Prototype exact finite bound selection across a small candidate bundle/ABI set.

Do not use an isolated resource equation as a substitute for complete
transaction measurement.

## Required vectors · `sec:optimization:vectors`

- no variables;
- one candidate;
- no feasible plan;
- several equal feasible plans;
- several Pareto-incomparable plans;
- one hard-constraint violation hidden by lower cost;
- permissionless secret dependency;
- lifecycle-incomplete cheapest plan;
- optional-only carrier;
- uncovered execution case;
- shared proof pattern;
- duplicate deliberate carriers;
- stable-key permutation;
- variable insertion permutation;
- constraint insertion permutation;
- objective tie;
- search-state exhaustion;
- lower-bound pruning boundary;
- incorrect nonadmissible pruning bound;
- LP candidate with tiny numerical constraint violation;
- LP primal feasible but dual invalid;
- MILP fractional candidate;
- SAT model violating one typed constraint;
- solver timeout or infrastructure failure;
- solver returns a feasible but noncanonical optimum.

## Measurements · `sec:optimization:measurements`

Record:

- variable and constraint counts;
- candidate-domain product;
- feasible assignment count;
- search states;
- hard-constraint pruning;
- objective pruning;
- Pareto frontier size;
- elapsed deterministic work;
- peak memory;
- exhaustive versus branch-and-bound crossover;
- external solver load/build/runtime if prototyped;
- certificate reconstruction and verification cost;
- result equality under permutations.

## Acceptance · `gate:optimization:accept`

Accept the initial optimization policy when:

- pilot proof and placement planning are solved exactly;
- hard constraints are separate from objectives;
- Pareto behavior is explicit;
- tie-breaking uses stable typed keys;
- branch-and-bound agrees with exhaustive small-instance oracles;
- complexity exhaustion fails closed;
- no greedy fallback exists;
- selected plans are independently revalidated;
- calibration uses exact integer values and final remeasurement;
- external solver adoption, if any, is justified by measured scale;
- solver results affecting release have independently checked certificates or
  explicitly scoped trust;
- solver-local variable IDs and floating output never enter semantic identity.

## Rejection · `gate:optimization:reject`

Reject a solver or formulation if it:

- models a hard safety condition as a penalty;
- uses the first returned feasible model as canonical;
- depends on hash or variable insertion order;
- uses floating feasibility without exact revalidation;
- cannot distinguish timeout from infeasibility;
- claims optimality after search exhaustion;
- silently falls back to greedy planning;
- exposes solver variable numbering as public identity;
- requires a native toolchain without reproducible release provisioning;
- lacks a practical certificate or independent validation path;
- solves a specialized graph/tree problem less clearly than its dedicated
  algorithm.

## Result · `sec:optimization:result`

Pending.

The expected Phase-2 result is:

```text
first-party exact deterministic search
+
exhaustive small-instance oracle
+
no external production solver yet
```

An external solver is added only after measured compiler/linker problem sizes
demonstrate that this result is insufficient.

## Handoff · `sec:optimization:handoff`

An accepted result updates:

- compiler proof and placement algorithms;
- linker calibration policy;
- solver dependency registry;
- compiler/linker error vocabularies;
- exact certificate types;
- coverage and optimization reports;
- release identity policy;
- Phase-2 and later exit gates;
- CI and supply-chain review requirements.
