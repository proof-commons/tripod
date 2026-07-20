# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 1 — typed realization foundation
> **Parallel preparation lane:** compiler/linker algorithm and dependency foundations
> **Next gate:** Phase 2 — target-independent compiler analysis
> **Authority:** Current execution queue only; normative source, implemented
> ADRs, accepted planning decisions, package contracts, research results, and
> the roadmap take precedence

This file contains current and immediately preparatory implementation work.

It does not contain:

- the complete long-term roadmap;
- protocol or realization semantics;
- package architecture duplicated from package contracts;
- unresolved design essays duplicated from research notes;
- generated identities copied from canonical artifacts;
- conversational history;
- implementation-source revisions treated as protocol identity.

Long-term sequencing is owned by [`roadmap.md`](roadmap.md). Cross-package
direction is owned by [`decisions/`](decisions/README.md), package boundaries by
[`packages/`](packages/README.md), phase gates by
[`phases/`](phases/README.md), and unresolved prototypes by
[`research/`](research/README.md).

---

## 1. Using this backlog · `sec:backlog:usage`

### 1.1 Status vocabulary · `tbl:backlog:status`

| Status | Meaning |
|---|---|
| **TODO** | Ready once the named dependencies complete. |
| **IN PROGRESS** | Actively being drafted, implemented, or reviewed. |
| **BLOCKED** | Cannot proceed until the named dependency or research result exists. |
| **DONE** | Implemented, tested, documented, and linked to evidence. |
| **DROPPED** | Deliberately not implemented; rationale and replacement recorded. |

Source code appearing to contain an intended change is not enough for `DONE`.
The task’s exit check and applicable repository gates must pass.

### 1.2 Task prefixes · `tbl:backlog:prefixes`

| Prefix | Owner |
|---|---|
| `P0` | Completed planning reset |
| `B0` | Completed baseline hardening |
| `C1` | Compiler/linker algorithm and dependency preparation |
| `R1` | Typed realization foundation |
| `Q` | Prototype/research dependency |

`C1` is a preparation lane, not permission to skip Phase 1. Its research,
dependency review, exact-oracle work, and synthetic prototypes may proceed in
parallel. Production compiler analysis still begins only after the Phase-1 gate.

### 1.3 Definition of done · `rule:backlog:done`

Every completed implementation task records:

1. implementing source files;
2. positive tests;
3. negative, mutation, or property tests where applicable;
4. documentation, decision, package-contract, or ADR updates;
5. exact verification command;
6. confirmation that checks did not modify tracked files;
7. identity and generated-artifact impact;
8. dependency and feature impact where applicable.

Every completed research task records:

1. the question answered;
2. exact dependency/tool/target versions used;
3. positive and negative prototype evidence;
4. complexity and resource measurements;
5. accepted and rejected candidates;
6. result and decision handoff;
7. permanent tests required by production;
8. whether the result is semantic authority, implementation policy, or
   diagnostic evidence.

### 1.4 Planning-label rule · `rule:backlog:labels`

Planning labels are non-normative and non-identity-bearing, but they are
mechanically checked under ADR-013:

- every planning mint is unique across the PLAN owner;
- every same-owner citation resolves;
- cross-owner citations use explicit owner prefixes;
- generated registers do not participate in the source graph.

Planning labels never become compiler, linker, ABI, deployment, or release
identity.

---

# 2. Current execution summary · `sec:backlog:current`

## 2.1 Completed foundation · `tbl:backlog:completed`

| Gate | Status | Result |
|---|---|---|
| P0 — planning rewrite | **DONE** | Focused decisions, package contracts, phase cards, research notes, registers, and backlog |
| B0 — baseline hardening | **DONE** | Reproducible, fail-closed compiler-era baseline |
| ADR-010 | Implemented | JSON CLI streams, stable exit classes, panic and TTY policy |
| ADR-011 | Implemented | Toolchain, locking, dependency, unsafe, target, and reproducibility policy |
| ADR-013 | Implemented | Owner-aware global Markdown/Rust label graph |
| ADR-014 | Implemented in source; ADR status review required | Build-owned census and stamp graph |

## 2.2 Current implementation · `tbl:backlog:packages`

| Package | Current role |
|---|---|
| `architecture` | Typed normative architecture and deployment profile |
| `model` | Executable reference behavior and property/corruption evidence |
| `realization` | Typed semantic realization for the compact-ASH and live-transfer pilots |
| `artifacts` | Generated-publication writer and checker |
| `labels` | Documentation/source label graph and registers |
| `cli-common` | Shared ADR-010 command infrastructure |
| `execwrap` | Byte-preserving child-process wrapper |
| `flatten-latex-main` | Deterministic atomic LaTeX flattener |

Not yet implemented:

```text
compiler
target-elements
tapscript
linker
transaction
vectors
release
```

## 2.3 Readiness capsule · `rem:backlog:readiness`

```text
Attestation specification:        published
Realization contract:         published and architecture-welded
Typed architecture release:       final and pinned
Executable reference model:       implemented
Typed realization package:        implemented for Phase-1 pilots; exit gate pending
Compiler analysis:                not yet implemented
Target/backend/linker:             not yet implemented
Independent deployment observers: not yet implemented
Deployment release:               none
```

Architecture finality does not imply deployment readiness.

---

# 3. Historical baseline record · `sec:backlog:baseline`

> **Recorded:** 2026-07-15
> **Status:** Historical and immutable

The baseline identities remain recorded in the annotated tag and
[`phases/00-baseline.md`](phases/00-baseline.md). Current source identities may
differ through reviewed presentation or hash-algorithm migrations without
rewriting the historical record.

## 3.1 Historical identities · `tbl:backlog:baseline-identities`

| Field | Baseline value |
|---|---|
| architecture schema | 17 |
| realization version | tracked compiler-line binding |
| denotation-preserving change | 13a |
| publication status | final |
| semantic algorithm | `sha256-canonical-json-v2` |
| semantic hash | |
| behavioural algorithm | `sha256-canonical-json-behavioural-v2` |
| behavioural hash | |
| Layer-0 version | `0.5.0` |
| Layer-0 anchor-set hash | |
| reproducible PDF SHA-256 | |

The current behavioural-hash algorithm is v3. Its migration record is owned by
architecture’s versioning gate and the Realization revision history.

## 3.2 Completed Phase-0 evidence · `tbl:backlog:baseline-evidence`

| Lane | Recorded result |
|---|---|
| Rust MSRV | 1.88.0 green |
| stable Rust | green at baseline |
| additional nightly | diagnostic only |
| formatting | green |
| Clippy `-D warnings` | green |
| debug/release tests | green |
| generated-artifact check | green |
| documentation/label checks | green |
| Meson document build | green |
| PDF reproducibility | byte-identical clean builds |
| clean tree | green |
| `cargo audit` | skipped loudly when unavailable |

Git history owns the detailed completed P0/B0 task prose. This backlog now keeps
only the compact baseline record and current work.

---

# 4. Algorithm and dependency laws · `sec:backlog:algorithm-laws`

The compiler and linker preparation lane follows these non-negotiable rules.

## 4.1 Problem-class separation · `rule:backlog:problem-classes`

```text
typed semantic AST:
    first-party typed source

graph reachability, SCCs, traversal:
    petgraph-backed private graphs

exact semantic arithmetic:
    checked integers, BigInt, BigRational

numerical dense/sparse linear algebra:
    faer-backed private working values

proof and placement selection:
    exact finite search initially

LP/MILP/SAT:
    separate solver review only after measured need

target tree optimization:
    explicit bounded-depth coding algorithm

target arithmetic:
    exact target relations and independent integer reference

cryptographic/consensus math:
    reviewed Elements and secp256k1-zkp ecosystem
```

A numerical linear solve is not a proof-selection solver. An LP solver is not a
graph algorithm. A graph library is not an AST. A floating residual is not
exact semantic equality.

## 4.2 Exactness · `rule:backlog:exactness`

Exact semantic, conservation, authorization, identity, calibration, and release
claims use:

- checked bounded integers;
- arbitrary-precision integers;
- reduced exact rationals;
- exact finite search;
- independently checked certificates;
- target-native execution where the target claim is concrete.

Numerical analysis may produce:

- candidates;
- diagnostics;
- sensitivity information;
- least-squares fits;
- conditioning and rank diagnostics;
- conservative estimates.

A release-sensitive numerical result must be converted into and checked as:

- an exact integer;
- a reduced rational;
- a conservative exact interval or bound;
- an independently checked primal/dual or other certificate.

## 4.3 Identity · `rule:backlog:algorithm-identity`

Every analysis distinguishes:

```text
local handle:
    process-local arena or graph position

stable key:
    complete typed semantic identity

digest:
    optional domain-separated commitment
```

The following are never semantic identity:

- `petgraph::NodeIndex`;
- `petgraph::EdgeIndex`;
- matrix row/column number;
- solver variable number;
- hash-map iteration order;
- source traversal order;
- pivot order;
- raw floating-point bits;
- solver iteration count;
- thread schedule;
- temporary path.

## 4.4 Canonical ordering · `rule:backlog:canonical-order`

Every canonical graph, matrix, plan, bundle, ABI, vector set, and report uses
explicit stable-key ordering.

Third-party container iteration is never assumed canonical.

## 4.5 Independent algorithm oracles · `rule:backlog:oracles`

Every nontrivial production algorithm has an independent small-instance oracle.

| Production analysis | Oracle |
|---|---|
| topological order | valid-order enumeration and least-key policy |
| SCC | mutual-reachability equivalence |
| interning | non-interned evaluator |
| dependency closure | repeated complete scan |
| exact linear solve | second rational elimination path |
| numerical solve | exact/high-precision comparison where applicable |
| proof selection | exhaustive candidate enumeration |
| placement | exhaustive carrier-subset enumeration |
| taptree construction | exhaustive small binary-tree enumeration |
| relocation | independent structured expected program |
| calibration search | exhaustive finite candidate range |
| resource formula | complete target transaction measurement |

## 4.6 Complexity failure · `rule:backlog:complexity`

A search or analysis that exceeds its explicit budget fails with a typed
complexity error.

It must not:

- drop a relation;
- weaken authorization;
- expose extra information;
- remove a lifecycle exit;
- select a greedy fallback;
- claim optimality from an incomplete search;
- accept the best partial result seen so far.

---

# 5. Current preparation lane: C1 — algorithms and dependencies · `sec:backlog:c1`

> **Lane status:** ACTIVE PREPARATION
> **Production compiler entry:** blocked on (`gate:phase1:exit`)
> **Production linker entry:** blocked on compiler, target, and backend artifacts
> **Allowed now:** dependency review, typed contracts, exact oracles, synthetic
> prototypes, complexity measurements
> **Not allowed now:** stable compiler/linker ABI, target fields in realization,
> production-marked backend patterns, final calibration claims

## 5.1 Task summary · `tbl:backlog:c1-tasks`

| ID | Status | Task | Depends on | Output |
|---|---|---|---|---|
| `C1-001` | **DONE** | Land algorithm research notes and census updates. | none | four research notes |
| `C1-002` | **DONE** | Record the petgraph graph-substrate decision. | `C1-001` | planning decision D007 |
| `C1-003` | **DONE** | Record exact/certified mathematics policy. | `C1-001` | planning decision D008 |
| `C1-004` | **IN PROGRESS** | Review and select concrete dependency releases/features. | `C1-002`, `C1-003` | dependency review record |
| `C1-005` | **TODO** | Specify and prototype canonical graph adapters. | `C1-002`, `C1-004` | frozen typed graph prototype |
| `C1-006` | **TODO** | Specify and prototype exact keyed linear systems. | `C1-003`, `C1-004` | exact matrix/Bareiss prototype |
| `C1-007` | **TODO** | Specify and prototype certified numerical analysis. | `C1-003`, `C1-004`, `C1-006` | `faer` wrapper prototype |
| `C1-008` | **BLOCKED** | Implement exact proof-plan search prototype. | realization relation vocabulary | exhaustive/branch-and-bound prototype |
| `C1-009` | **BLOCKED** | Implement execution-case-aware placement prototype. | realization/compiler relation vocabulary | exact placement prototype |
| `C1-010` | **TODO** | Prototype typed symbol resolution and SCC policy. | `C1-002`, `C1-005` | synthetic linker graph prototype |
| `C1-011` | **TODO** | Prototype structured/simultaneous relocation. | `C1-010` | relocation prototype |
| `C1-012` | **TODO** | Prototype deterministic bounded-depth taptree construction. | `C1-010` | package-merge prototype |
| `C1-013` | **BLOCKED** | Build the complete algorithm-oracle suite. | `C1-005`–`C1-012` | property/exhaustive oracle tests |
| `C1-014` | **BLOCKED** | Run preparation review and hand off to Phase 2. | `C1-001`–`C1-013`, Phase 1 | accepted algorithm policy |

---

## C1-001 — Land algorithm research notes

> **Status:** DONE
> **Output:**
> [`compiler-algorithms.md`](research/compiler-algorithms.md)
> [`linker-algorithms.md`](research/linker-algorithms.md)
> [`numerical-linear-algebra.md`](research/numerical-linear-algebra.md)
> [`optimization-solvers.md`](research/optimization-solvers.md)
> **Result:** Four notes landed; research README index and Meson census
> updated; documentation-structure and label-graph gates pass on a clean tree.

### Scope

Add the four research notes in the existing research-document form.

Update:

```text
plans/research/README.md
plans/research/meson.build
plans/backlog.md
```

The notes must remain:

- non-normative;
- non-machine-consumed;
- explicit about blocked production interfaces;
- explicit about exact versus numerical claims;
- explicit about small-instance oracle requirements;
- free of unreviewed concrete dependency-version claims.

### Exit check

- [x] all four files exist;
- [x] nearest README indexes every file;
- [x] Meson census names every file;
- [x] every local/imported citation resolves;
- [x] combined documentation budget passes;
- [x] no production API is frozen by research prose;
- [x] documentation checks leave the tree clean.

Verification:

```sh
meson compile -C build
meson test -C build --print-errorlogs
scripts/check-plans.sh
git diff --check
```

---

## C1-002 — Record the petgraph substrate decision

> **Status:** DONE
> **Depends on:** `C1-001`
> **Primary output:** `plans/decisions/007-petgraph-internal-graph-substrate.md`

### Decision

Record:

> `petgraph` is the preferred private graph substrate for realization,
> compiler, and linker analyses. Petgraph indices, traversal order,
> serialization, and generic graph structure are not semantic identity or public
> API. Typed ASTs, stable keys, canonical ordering, graph schemas, solver
> policies, and publications remain first-party.

### Required scope

- graph storage;
- SCC;
- reachability;
- traversal;
- reverse dependency analysis;
- condensation support;
- graph diagnostics.

### Explicit exclusions

- semantic AST ownership;
- semantic IDs;
- canonical topological order;
- canonical serialization;
- proof selection;
- placement constraints;
- relocation;
- taptree optimization;
- resource formulas;
- release identity.

### Exit check

- [ ] D007 uses the planning-decision template;
- [ ] decision index and Meson census are updated;
- [ ] package contracts cite the decision instead of repeating it;
- [ ] no repository-wide law forbids a specialized first-party algorithm;
- [ ] no generic shared graph crate is authorized prematurely.

---

## C1-003 — Record exact and certified mathematics policy

> **Status:** DONE
> **Depends on:** `C1-001`
> **Primary output:** `plans/decisions/008-exact-and-certified-mathematics.md`

### Decision

Record:

> Exact semantic and release claims use checked integer/rational reasoning or
> independently checked certificates. `faer` is the preferred private numerical
> linear-algebra substrate for candidate generation, diagnostics, sensitivity,
> least squares, and certified numerical analyses. Raw floating-point output is
> never semantic identity or sufficient release proof.

### Required distinctions

- numerical \(Ax=b\);
- exact rational \(Ax=b\);
- LP/QP;
- MILP/SAT;
- graph analysis;
- target arithmetic;
- cryptographic math.

### Required policy

- exact keyed matrix is authoritative;
- numerical matrix is derivative working state;
- solver method is explicit;
- tolerances are explicit;
- residual and conditioning checks are required;
- rank ambiguity fails closed for release-sensitive uses;
- exact or conservative certification is required;
- no explicit inverse for routine solves;
- QR/SVD preferred over accidental normal equations;
- parallel numerical execution disabled initially;
- no raw `f64` bits in semantic identities.

### Exit check

- [ ] D008 uses the planning-decision template;
- [ ] `faer`, `num-rational`, and existing `num-*` roles are distinct;
- [ ] optimizer policy remains owned by solver research;
- [ ] target/cryptographic math remains separately reviewed;
- [ ] package contracts and identity register are updated.

---

## C1-004 — Review and select dependencies

> **Status:** IN PROGRESS
> **Depends on:** `C1-002`, `C1-003`
> **Initial candidates:** `petgraph`, `faer`, `num-rational`

### Review record

For each selected release record:

- exact crates.io package/version;
- upstream repository and matching release;
- license;
- Rust 1.88 compatibility;
- exact enabled features;
- default-feature policy;
- transitive dependency graph;
- duplicate versions;
- unsafe/SIMD/native trust surface;
- maintenance status;
- advisory status;
- determinism implications;
- canonical-output implications;
- direct package consumers;
- public API exposure policy;
- replacement boundary.

### Adoption policy

```text
petgraph:
    workspace-owned when the realization graph consumer lands

num-rational:
    workspace-owned when exact rational analysis lands

faer:
    workspace-owned core numerical dependency;
    direct dependency only of an actual numerical-analysis consumer
```

Do not add unused dependencies only to advertise future intent.

Disable optional parallel, serialization, ecosystem-adapter, or native features
unless a concrete API needs them.

### Exit check

```sh
cargo tree --locked -p petgraph -e features
cargo tree --locked -p faer -e features
cargo tree --locked -p num-rational -e features

cargo tree --locked -i petgraph
cargo tree --locked -i faer
cargo tree --locked -i num-rational
```

Then run Rust 1.88 and stable workspace lanes.

A dependency is accepted only when the lockfile diff and focused tests are
reviewed.

---

## C1-005 — Canonical graph adapter prototype

> **Status:** TODO
> **Depends on:** `C1-002`, `C1-004`
> **Likely production owner:** realization first, compiler/linker later

### Required adapter properties

- first-party typed node/edge schemas;
- stable-key collection;
- canonical node insertion;
- canonical edge insertion;
- key-to-index and index-to-key maps;
- private petgraph indices;
- explicit self-loop policy;
- explicit parallel-edge policy;
- frozen post-validation graph;
- deterministic diagnostics;
- first-party publication DTO;
- no direct petgraph serialization.

### Required algorithms

- canonical Kahn topological order;
- cycle remainder detection;
- SCC normalization;
- deterministic representative cycle;
- reachability;
- reverse dependency closure.

### Required tests

- node/edge insertion permutations;
- unrelated-node insertion;
- duplicate keys;
- missing endpoints;
- self-loop;
- one large SCC;
- deep chain;
- disconnected graph;
- dense small graph;
- no graph index in public output;
- equality across repeated construction.

### Exit check

Production algorithms agree with independent small-instance topology and SCC
oracles.

---

## C1-006 — Exact keyed linear-system prototype

> **Status:** TODO
> **Depends on:** `C1-003`, `C1-004`
> **Dependencies:** existing `num-bigint`, `num-integer`, `num-traits`;
> selected `num-rational`

### Required types

- stable typed row keys;
- stable typed column keys;
- exact rational coefficients;
- canonical sparse or dense coefficient DTO;
- exact right-hand side;
- exact solution/certificate;
- no matrix position as semantic identity.

### Required algorithm

Implement fraction-free Bareiss elimination for small systems.

Support:

- exact rank;
- exact consistency;
- exact determinant where useful;
- exact square solve;
- exact substitution verification.

Keep a simpler independent `BigRational` Gaussian-elimination oracle for small
tests.

### Required tests

- identity and diagonal matrices;
- full-rank integer systems;
- exact rational solutions;
- singular systems;
- inconsistent systems;
- row/column permutations;
- redundant equations;
- nonzero equation scaling;
- coefficient mutation;
- coefficient-growth boundaries.

### Exit check

Bareiss and the independent rational oracle agree on every generated accepted
small system.

---

## C1-007 — Certified `faer` numerical prototype

> **Status:** TODO
> **Depends on:** `C1-003`, `C1-004`, `C1-006`
> **Production status:** diagnostic/certified numerical analysis only

### Required methods

Prototype explicit policies for:

- pivoted LU;
- QR;
- SVD;
- Cholesky after SPD validation;
- multiple right-hand sides;
- sparse analysis only if measured dimensions justify it.

### Required acceptance checks

For \(Ax=b\), recompute:

\[
\eta=
\frac{\lVert b-Ax\rVert_\infty}
{\lVert A\rVert_\infty\lVert x\rVert_\infty+\lVert b\rVert_\infty}
\]

Also record:

- finite inputs and outputs;
- decomposition status;
- rank;
- condition estimate;
- residual;
- backward error;
- explicit tolerance policy;
- exact or conservative certification mode.

### Forbidden use

The prototype must not establish:

- exact semantic equality;
- proof-plan optimality;
- carrier coverage;
- exact calibration;
- release identity.

### Required tests

- exact/numerical cross-check;
- rank deficiency;
- ill-conditioning;
- bad scaling;
- overdetermined least squares;
- non-SPD Cholesky rejection;
- zero pivot requiring pivoting;
- nonfinite input;
- row/column permutation;
- no raw float in identity-bearing output.

### Exit check

Well-conditioned numerical results agree with exact solutions under declared
tolerances. Ill-conditioned or rank-ambiguous systems cannot receive certified
release-sensitive status.

---

## C1-008 — Exact proof-plan search

> **Status:** BLOCKED
> **Depends on:** Phase-1 relation and proof-alternative vocabulary
> **Research:** [`optimization-solvers.md`](research/optimization-solvers.md)

### Required algorithm

Implement deterministic exact enumeration or branch-and-bound over pilot proof
alternatives.

Hard constraints include:

- semantic relation preservation;
- target capability;
- authenticatable sources;
- witness availability;
- permissionless constructibility;
- representation;
- disclosure;
- lifecycle;
- relation coverage.

Cost is evaluated only after hard constraints pass.

Retain a Pareto frontier where no accepted total order exists.

### Required oracle

Exhaustively enumerate every complete assignment for generated small instances.

### Exit check

- exact search and exhaustive oracle agree;
- greedy counterexamples are included;
- equal optima select by stable typed keys;
- complexity exhaustion fails closed;
- no external solver is required for pilots.

---

## C1-009 — Exact placement and coverage search

> **Status:** BLOCKED
> **Depends on:** Phase-1/Phase-2 relation, carrier, and execution-case types

### Required model

```text
relations
↔ carriers
↔ execution cases
```

Require for every relation \(r\):

\[
\operatorname{requiredCases}(r)
\subseteq
\bigcup_{c\text{ carries }r}
\operatorname{executedCases}(c)
\]

### Required failures

- missing carrier;
- optional-only carrier for unconditional relation;
- local-only carrier for global relation;
- unreachable carrier;
- execution-case gap;
- unavailable fact source;
- permissionless secret dependency;
- ambiguous equal-cost selection;
- complexity exhaustion.

### Required oracle

Exhaustive carrier-subset enumeration for small generated instances.

### Exit check

Exact placement agrees with the oracle and every pilot relation is covered in
every active case.

---

## C1-010 — Typed linker graph and SCC prototype

> **Status:** TODO
> **Depends on:** `C1-002`, `C1-005`
> **Production linker:** still blocked on backend artifacts

### Required prototype

- two-pass typed symbol resolution;
- complete definition census;
- complete reference resolution;
- typed reference graph;
- canonical SCCs;
- typed condensation DAG;
- canonical condensation order;
- explicit strategy on every cyclic edge.

### Required cycle strategies

Prototype classification for:

- static link-time value;
- target identity introspection;
- authenticated witnessed root;
- in-program constructor reconstruction;
- deployment relocation;
- unsupported cycle.

SCC membership never makes a cycle valid by itself.

### Required tests

- forward reference;
- duplicate and ambiguous symbols;
- incompatible symbol type;
- self-loop with/without strategy;
- multi-node SCC;
- deep chain;
- repeated-hashing fixed-point attempt;
- stale constructor identity.

### Exit check

SCC results agree with a mutual-reachability oracle and every unresolved cycle
fails deterministically.

---

## C1-011 — Structured relocation prototype

> **Status:** TODO
> **Depends on:** `C1-010`

### Preferred path

Resolve structured target programs before byte serialization.

### Byte-relocation fallback

If byte relocation is necessary, require:

- fixed width;
- typed source and destination;
- exact encoding;
- exact expected placeholder;
- no overlaps;
- one resolution per mandatory relocation;
- replacement values computed against pristine bytes;
- canonical offset ordering;
- simultaneous application;
- full post-link reparse and validation.

Variable-width byte relocation is prohibited.

### Required oracle

Construct the expected linked structured program independently without
patching.

### Exit check

- structured and relocation paths agree;
- overlap and stale-placeholder mutations reject;
- earlier patches cannot move or alter later patch interpretation;
- final program identity derives from reparsed validated output.

---

## C1-012 — Deterministic bounded-depth taptree prototype

> **Status:** TODO
> **Depends on:** `C1-010`

### Initial objective

Use deterministic length-limited Huffman construction by package-merge:

\[
\min\sum_i w_i d_i
\qquad\text{subject to}\qquad
d_i\le L
\]

Inputs:

- stable leaf key;
- target program identity;
- leaf version;
- positive integer weight;
- maximum depth;
- target branch-hash ordering.

If no defensible execution frequencies exist, use equal weights.

### Required tests

- one leaf;
- equal weights;
- unequal weights;
- duplicate weights;
- impossible depth;
- insertion permutations;
- stable tie-breaking;
- duplicate leaf bytes;
- relation provenance under deliberate coalescing;
- exact target child ordering.

### Required oracle

Enumerate every full binary tree for small leaf sets and compare the objective
and depth result.

### Exit check

Package-merge agrees with the exhaustive oracle and tree policy is an explicit
bundle/ABI identity input.

---

## C1-013 — Complete algorithm-oracle suite

> **Status:** BLOCKED
> **Depends on:** `C1-005`–`C1-012`

### Required suites

- graph permutations;
- topology;
- SCC;
- interning;
- closure;
- exact matrices;
- numerical certification;
- proof selection;
- placement;
- relocation;
- package-merge;
- calibration monotonicity;
- resource formulas.

### Metamorphic requirements

- source declaration permutation does not change canonical results;
- map/hash insertion order does not change results;
- unrelated nodes do not renumber stable identities;
- presentation-only labels do not move semantic analysis;
- thread count does not affect accepted exact output;
- no raw third-party handle enters a publication;
- no raw floating-point value enters semantic identity;
- unsupported complexity fails rather than weakening semantics.

### Exit check

The complete oracle suite passes under:

- Rust 1.88;
- current stable;
- debug;
- release;
- repeated clean runs.

---

## C1-014 — Algorithm-preparation review

> **Status:** BLOCKED
> **Depends on:** `C1-001`–`C1-013`, (`gate:phase1:exit`)

### Exit decision

The review records:

- accepted graph substrate;
- accepted exact arithmetic;
- accepted numerical policy;
- proof/placement algorithm;
- external-solver status;
- linker SCC/cycle policy;
- relocation policy;
- taptree policy;
- complexity limits;
- oracle coverage;
- dependency versions/features;
- deferred alternatives.

### Exit check

The compiler phase may begin when:

- algorithm contracts are typed;
- dependencies pass policy review;
- pilot exact-search scale is measured;
- no external solver is load-bearing without need;
- all small-instance oracles pass;
- no third-party implementation detail is a public semantic contract;
- Phase 1 passes independently.

---

# 6. Current gate: R1 — typed realization foundation · `sec:backlog:r1`

> **Gate status:** CURRENT
> **Phase card:** [`phases/01-realization.md`](phases/01-realization.md)
> **Package contract:** [`packages/realization.md`](packages/realization.md)
> **Allowed first-party direct dependency:** `architecture`
> **Preferred graph substrate:** selected `petgraph` release after `C1-004`
> **Numerical dependency:** none unless a concrete Phase-1 matrix use exists

## 6.1 Task summary · `tbl:backlog:r1-tasks`

| ID | Status | Task | Depends on | Output |
|---|---|---|---|---|
| `R1-001` | **DONE** | Create the realization crate skeleton and package contract. | baseline, `C1-004` for graph dependency | workspace crate |
| `R1-002` | **DONE** | Define local handles, stable keys, and deterministic semantic IDs. | `R1-001` | identity types |
| `R1-003` | **DONE** | Define typed domains, expression arena, and frozen dependency graph. | `R1-002`, `C1-005` | typed expression DAG |
| `R1-004` | **DONE** | Define the minimum semantic relation vocabulary. | `R1-003` | relation types |
| `R1-005` | **DONE** | Define operation, constructibility, lifecycle, and representation declarations. | `R1-004` | operation schema |
| `R1-006` | **DONE** | Implement deterministic `derive(&Architecture)`. | `R1-005` | scoped `RealizationSpec` |
| `R1-007` | **DONE** | Declare `compact-ash`. | `R1-006` | first pilot |
| `R1-008` | **DONE** | Declare `transfer-live-receipts`. | `R1-006` | second pilot |
| `R1-009` | **DONE** | Derive declassification for both pilots. | `R1-007`, `R1-008` | typed disclosure result |
| `R1-010` | **DONE** | Add architecture/realization bidirectional validation. | `R1-007`, `R1-008` | validation suite |
| `R1-011` | **DONE** | Add model-conformance tests for both pilots. | `R1-009`, `R1-010` | conformance evidence |
| `R1-012` | **DROPPED** | Add a derivative publication only if a real need exists. | `R1-009` | no realization-owned publication needed |
| `R1-013` | **IN PROGRESS** | Run and record the Phase-1 exit gate. | required `R1` tasks | green Phase 1 |

---

## R1-001 — Create the realization crate

> **Status:** DONE
> **Depends on:** completed baseline and reviewed first-consumer dependency set

### Required metadata

```text
directory: packages/realization
Cargo package: tripod-realization
Rust library: realization
```

Inherit workspace:

- authors;
- edition;
- license;
- publication policy;
- Rust version;
- workspace version;
- lints.

Required first-party dependency:

```text
architecture
```

Expected initial third-party dependencies:

```text
petgraph
thiserror
```

`num-rational` is added only if Phase-1 exact affine analysis uses it.

`faer` is not added to realization solely for architectural symmetry.

Forbidden first-party dependencies:

```text
model
compiler
target-elements
tapscript
simplicity
linker
transaction
vectors
release
artifacts
labels
```

### Required crate documentation

State:

- purpose;
- normative typed input;
- explicit pilot scope;
- forbidden inputs;
- target independence;
- deterministic derivation;
- graph-library privacy;
- model-conformance boundary;
- compiler-consumption boundary;
- current provisional status.

### Census

Add:

```text
packages/realization/meson.build
```

with explicit Rust-source lists under ADR-014.

Update:

```text
Cargo.toml
packages/meson.build
README.md
```

### Exit check

```sh
cargo check --locked -p tripod-realization
cargo clippy --locked -p tripod-realization --all-targets -- -D warnings
```

---

## R1-002 — Define semantic identity ownership

> **Status:** DONE
> **Depends on:** `R1-001`

### Required separation

Define distinct:

```text
FactHandle / FactKey / optional FactDigest
ExprHandle / ExprKey / optional ExprDigest
RelationHandle / RelationKey / optional RelationDigest
ObservableId
ConstructibilityRequirementId
LifecycleRequirementId
ProofAlternativeId
```

Architecture retains ownership of:

```text
AssetId
RootId
ObjectId
OperationId
QuantityId
WitnessId
InvariantClauseId
BoundId
TagId
stable architecture discriminants
```

### Requirements

- local handles are dense and owner-local;
- stable keys are insertion-order independent;
- digests are domain-separated;
- complete structural keys remain the collision boundary;
- source provenance does not define identity;
- target opcodes do not define semantic identity;
- structurally shared nodes retain all source relation provenance;
- unrelated declaration insertion does not renumber stable IDs.

### Required tests

- repeated derivation equality;
- source-order permutation;
- graph insertion permutation;
- semantic mutation affects only intended identities;
- presentation mutation does not move semantic IDs;
- deliberate digest-collision fixture fails closed;
- unrelated node insertion preserves existing stable keys.

---

## R1-003 — Define typed domains and expression DAG

> **Status:** DONE
> **Depends on:** `R1-002`, accepted graph-adapter contract

### Required domains

At minimum:

- boolean;
- count/cardinality;
- protocol amount;
- asset;
- object;
- owner;
- input reference;
- output reference.

Later declarations may add:

- cycle;
- block age;
- ratio;
- root;
- address;
- tag;
- canonical order.

A count is not an amount.

### Expression arena

Provide first-party typed nodes for:

- constants;
- fact references;
- checked addition/subtraction;
- multiplication with explicit domain;
- exact floor relation where needed later;
- equality and comparison;
- conditional activation;
- bounded family sums;
- dependency traversal.

### Graph projection

Use the private frozen graph adapter for:

- dependency validation;
- canonical topology;
- cycle diagnostics;
- reverse closure.

Petgraph types remain private.

### Reference evaluator

Implement a small deterministic evaluator for supported Phase-1 expressions.

It is not the executable model.

### Required tests

- ill-typed expression rejection;
- checked overflow/domain rejection;
- unresolved fact;
- dependency cycle;
- canonical topological order;
- insertion permutations;
- deep-chain stack safety;
- evaluator versus direct reference arithmetic;
- no graph index in public output.

---

## R1-004 — Define semantic relation vocabulary

> **Status:** DONE
> **Depends on:** `R1-003`

Initial relation families must express both pilots without generic string
predicates.

Expected families:

- domain;
- cardinality;
- object/asset recognition;
- authorization;
- equality;
- conservation;
- recipient pin;
- class closure;
- output-family closure;
- sponsor isolation;
- root policy;
- event projection policy;
- constructibility;
- lifecycle;
- representation;
- proof alternative.

### Requirements

- every relation has typed operands;
- every relation has operation ownership;
- every relation has source provenance;
- activation is typed;
- target detail is absent;
- relation IDs are deterministic;
- duplicate semantic relation IDs fail;
- structural sharing does not erase source relation ownership.

### Exit check

Both pilot relation sets can be represented without:

- target opcodes;
- transaction indexes;
- stack positions;
- stringly predicates;
- model callbacks;
- generated-file parsing.

---

## R1-005 — Define operation adjuncts

> **Status:** DONE
> **Depends on:** `R1-004`

Define typed declarations for:

- input families;
- output families;
- authorization;
- preconditions;
- state effects;
- projections;
- public observables;
- constructibility;
- witness availability;
- lifecycle exits;
- representation capabilities;
- proof alternatives;
- architecture bound references.

### Requirements

- permissionless versus owner-authorized construction is explicit;
- verification capability and witness availability are distinct;
- public and private facts are distinct;
- representation does not alter semantic value;
- required lifecycle exits are explicit;
- pilot-incomplete lifecycle is representable without claiming release
  completeness;
- operation ownership is exactly architecture-owned.

---

## R1-006 — Implement deterministic derivation

> **Status:** DONE
> **Depends on:** `R1-005`

Illustrative boundary:

```rust
pub fn derive(
    architecture: &architecture::Architecture,
    scope: RealizationScope,
) -> Result<RealizationSpec, RealizationError>;
```

### Required behavior

- pure;
- deterministic;
- explicit scope;
- architecture validation;
- no filesystem;
- no environment;
- no generated publication;
- no model source or test parsing;
- no planning/ADR parsing;
- no target dependency;
- complete stable-key construction;
- canonical graph construction;
- bidirectional references;
- typed failure on unresolved/duplicate/cyclic declarations.

### Scope rule

A two-operation pilot value must carry an incomplete-scope state.

It must be impossible to validate or publish it as a complete deployment
realization.

---

## R1-007 — Declare compact ASH

> **Status:** DONE
> **Depends on:** `R1-006`

Declare:

- ASH input minimum two;
- maximum `ASH_BATCH_MAX`;
- exactly one ASH output;
- explicit semantic `U` conservation;
- permissionless operation authorization;
- no owner or operator witness;
- optional isolated sponsor flow;
- no roots;
- transition-certificate projection;
- no burn, clear, or residue projection;
- public constructibility;
- compact and clear lifecycle exits.

### Exit check

Architecture, realization, and model agree on:

- families;
- minima/maxima;
- authorization;
- value relation;
- roots;
- flow classes;
- projections;
- constructibility;
- lifecycle.

---

## R1-008 — Declare live receipt transfer

> **Status:** DONE
> **Depends on:** `R1-006`

Declare:

- nonempty bounded live-receipt inputs;
- nonempty bounded live-receipt outputs;
- owner authorization for every input;
- live-class closure;
- exact aggregate semantic `U` conservation;
- authorized destination freedom;
- optional isolated sponsor flow;
- no roots;
- transition-certificate projection;
- explicit and private-committed value alternatives;
- transfer, burn, and redemption lifecycle obligations.

### Exit check

Architecture, realization, and model agree on every declared family and
semantic relation.

Representation alternatives must not weaken:

- explicit closed `U` identity;
- owner authorization;
- output-family closure;
- value conservation;
- future lifecycle.

---

## R1-009 — Derive declassification

> **Status:** DONE
> **Depends on:** `R1-007`, `R1-008`

Derive disclosure by deterministic reverse dependency closure.

Seed reasons:

- public state;
- public event/interface;
- permissionless construction.

Retain:

```text
fact
disclosure reason
source relation
dependency provenance
```

Expected pilot behavior:

```text
compact ASH:
    public ASH facts remain publicly usable;
    no owner-private disclosure introduced

live transfer:
    abstract lateral relation requires no numerical value disclosure
```

### Required tests

- several dependency paths;
- reason union;
- source relation provenance;
- irrelevant fact exclusion;
- insertion permutation;
- repeated-full-scan oracle agreement;
- no handwritten parallel disclosure list.

---

## R1-010 — Add architecture/realization validation

> **Status:** DONE
> **Depends on:** `R1-007`, `R1-008`

Validation is bidirectional:

- every scoped architecture operation has one realization declaration;
- every realization operation maps to one architecture operation;
- every input/output family resolves;
- every bound resolves;
- authorization agrees;
- root policy agrees;
- value-flow classes agree;
- projection policy agrees;
- no undeclared semantic object appears;
- no required architecture fact is omitted;
- no duplicate stable relation identity exists.

### Mutation tests

- missing operation;
- unexpected operation;
- wrong input/output family;
- transposed operation IDs;
- wrong bound;
- wrong authorization;
- wrong projection;
- wrong value-flow class;
- duplicate fact/expression/relation;
- unresolved reference;
- dependency cycle;
- incomplete scope falsely marked complete.

---

## R1-011 — Add model conformance

> **Status:** DONE
> **Depends on:** `R1-009`, `R1-010`

For each pilot:

1. construct valid worlds through public model APIs;
2. execute valid operations;
3. project realization facts from predecessor, request, and successor;
4. evaluate every active relation;
5. compare certificate/projection effects;
6. mutate one semantic fact or relation at a time;
7. require precise conformance failure;
8. preserve realization and model as separate evidence roles.

### Evidence limits

Shared helpers and architecture types are documented.

A green conformance test is model evidence. It is not target, linker, ABI, or
deployment evidence.

---

## R1-012 — Optional derivative publication

> **Status:** DROPPED
> **Depends on:** `R1-009`

A publication is added only if a concrete review or external-consumer need
exists.

If added, require:

- one typed source;
- explicit schema;
- canonical ordering;
- deterministic bytes;
- unknown-field rejection;
- explicit generator;
- non-writing checker;
- no reverse semantic dependency;
- explicit incomplete pilot scope.

If no need exists, mark this task `DROPPED` with rationale.

---

## R1-013 — Phase-1 gate

> **Status:** TODO
> **Depends on:** all required `R1` tasks

### Exit requirements

- realization is a workspace crate;
- architecture is its only first-party direct dependency;
- selected graph dependency is private;
- local handles and stable identities are distinct;
- both pilots are complete;
- typed domains and expressions validate;
- canonical graph analysis is deterministic;
- declassification is dependency-derived;
- constructibility and lifecycle are explicit;
- architecture and realization agree bidirectionally;
- model transitions satisfy all active declarations;
- focused semantic mutations fail;
- pilot scope cannot be mistaken for complete scope;
- no generated file, model source, target type, or planning file is consumed;
- no raw graph index or floating result enters semantic identity;
- Rust MSRV/stable, Meson, labels, plans, generated artifacts, and clean-tree
  gates pass.

Verification:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
git diff --exit-code
```

---

# 7. Research and prototype register · `sec:backlog:research`

## 7.1 Active preparation questions · `tbl:backlog:research-active`

| ID | Research note | Status | Blocks |
|---|---|---|---|
| `Q-COMPILER-ALG` | [`compiler-algorithms.md`](research/compiler-algorithms.md) | Design and prototype required | Compiler graph, identity, proof, placement, coverage |
| `Q-LINKER-ALG` | [`linker-algorithms.md`](research/linker-algorithms.md) | Design and prototype required | Linker SCC, relocation, taptree, carrier closure |
| `Q-NUMERICAL` | [`numerical-linear-algebra.md`](research/numerical-linear-algebra.md) | Dependency review and prototype required | Certified numerical analysis |
| `Q-OPTIMIZATION` | [`optimization-solvers.md`](research/optimization-solvers.md) | Exact prototype required | Solver-backed proof, placement, calibration |

## 7.2 Target-dependent questions · `tbl:backlog:research-target`

| ID | Research note | Status | Blocks |
|---|---|---|---|
| `Q-STATE` | [`state-constructor.md`](research/state-constructor.md) | Prototype required | STATE backend ABI and Phase 6 |
| `Q-ARITH` | [`wide-arithmetic.md`](research/wide-arithmetic.md) | Prototype and measurement required | Redemption, settlement, cycle |
| `Q-DECLASS` | [`public-declassification.md`](research/public-declassification.md) | Open; prototype required | Direct private burn/redemption paths |
| `Q-SETTLE` | [`settlement-layout.md`](research/settlement-layout.md) | Open; prototype required | Settlement ABI and calibrated batch size |

## 7.3 Research isolation · `rule:backlog:research-isolation`

Research prototypes may begin before consuming phases, but they must not:

- add target-specific fields to `RealizationSpec`;
- publish stable production ABI;
- become release artifacts;
- claim production capability;
- use draft defaults as calibration;
- label model wrappers as independent observers;
- introduce solver output as semantic identity;
- bypass package dependency direction.

Accepted results move into typed source, permanent tests, package contracts, and
a planning decision or ADR where appropriate.

---

# 8. Dependency map · `sec:backlog:dependencies`

## 8.1 Immediate candidate map · `tbl:backlog:dependency-map`

| Dependency | Status | Intended role | Initial consumers |
|---|---|---|---|
| `petgraph` | review/adopt with first consumer | graph storage, SCC, reachability, traversal | realization, compiler, linker |
| `faer` | review as core numerical substrate | dense/sparse numerical linear algebra | compiler or future calibration analysis |
| `num-rational` | review/adopt when exact matrices land | exact rational coefficients/certificates | realization/compiler analysis |
| `fixedbitset` | deferred/conditional | dense local coverage and case sets | compiler/linker |
| `elements` | Phase-3 review | target transaction and consensus types | target-elements, transaction |
| `elements-miniscript` | conditional | standard target program/control support | tapscript tooling |
| `secp256k1-zkp` | prototype-gated | CT commitments and proofs | transaction/target research |
| SAT/LP/MILP solver | deferred | large exact planning problems | compiler/linker |
| `salsa` | deferred | incremental query execution | compiler |
| `egg` | deferred | equality saturation | compiler |
| `rayon` | deferred | independent evidence execution | vectors |

## 8.2 Existing dependencies to reuse · `tbl:backlog:dependency-existing`

| Dependency | Use |
|---|---|
| `serde` | first-party DTOs and external envelopes |
| `serde_json` | deterministic review/report publications |
| `sha2` | project semantic/artifact identities |
| `thiserror` | typed library/package failures |
| `anyhow` | binary/orchestration boundaries only |
| `num-bigint` | exact integer oracle |
| `num-integer` | exact integer helpers |
| `num-traits` | numeric traits |
| `proptest` | generated algorithm/model tests |
| `tempfile` | collision-safe staging and hermetic fixtures |

## 8.3 Deferred dependency rule · `rule:backlog:deferred-dependencies`

A deferred dependency enters only after:

1. a concrete consumer exists;
2. current code demonstrates the missing functionality;
3. simpler exact first-party code is insufficient;
4. purpose, license, maintenance, MSRV, unsafe boundary, transitive graph,
   determinism, and advisories are reviewed;
5. public API leakage is considered;
6. focused tests and an independent oracle exist;
7. the lockfile update is reviewed;
8. all gates remain green and clean.

---

# 9. Verification matrix · `sec:backlog:verification`

## 9.1 Rust lanes

Run on declared MSRV and current stable:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release --locked
scripts/ci.sh
```

## 9.2 Dependency lanes

For each new dependency:

```sh
cargo tree --locked -p <package>
cargo tree --locked -p <package> -e features
cargo tree --locked -i <package>
git diff -- Cargo.toml Cargo.lock packages/*/Cargo.toml
```

When installed:

```sh
cargo audit
```

If repository policy later adopts them:

```sh
cargo deny check
cargo vet
```

A missing required release tool is not a pass.

## 9.3 Meson/document lanes

```sh
meson compile -C build
meson test -C build --print-errorlogs
scripts/check-document-reproducibility.sh
```

## 9.4 Documentation and census

```sh
scripts/check-plans.sh
git diff --check
```

Every new tracked Rust or documentation subject must be added to its nearest
`meson.build` census.

## 9.5 Mathematical algorithm lane

The algorithm gate must include:

```text
canonical graph insertion permutations
topological oracle
SCC mutual-reachability oracle
declassification closure oracle
Bareiss versus rational elimination
faer versus exact well-conditioned solutions
rank-deficient and ill-conditioned rejection
proof search versus exhaustive oracle
placement versus exhaustive carrier oracle
relocation versus independent structured output
package-merge versus exhaustive small-tree oracle
calibration enumeration and monotonicity checks
no graph/matrix/solver local handle in semantic identity
```

---

# 10. Backlog hygiene · `sec:backlog:hygiene`

## 10.1 Adding a task · `rule:backlog:add`

A new task states:

- owner phase or preparation lane;
- status;
- dependencies;
- concrete output;
- assurance class;
- focused exit check;
- affected package/files;
- verification command;
- identity impact;
- dependency impact.

## 10.2 Splitting a task · `rule:backlog:split`

Split a task when:

- it crosses semantic, compiler, linker, target, transaction, evidence, or
  release boundaries;
- it has an independently reviewable security consequence;
- part can complete while another remains prototype-blocked;
- it mixes exact correctness with numerical diagnostics;
- it mixes dependency adoption with algorithm acceptance.

## 10.3 Dropping a task · `rule:backlog:drop`

A dropped task records:

- why it is unnecessary;
- evidence;
- replacement;
- affected documentation;
- identity/release consequences.

## 10.4 Completed-task retention · `rule:backlog:retention`

After a phase baseline:

- compact completed task prose into the phase card or release record;
- retain immutable evidence in Git history;
- keep only current and immediately preparatory work in this backlog;
- do not build a second historical archive inside `plans/`.

The active backlog should remain reviewable in one sitting.

---

# 11. Current execution order · `sec:backlog:order`

Unless evidence changes dependencies, execute:

```text
1. Complete concrete dependency release/feature review.
2. Run and record the Phase-1 exit gate.
3. In parallel, complete synthetic linker and mathematical algorithm prototypes.
4. Begin production compiler analysis only after Phase 1.
5. Keep target/backend production work behind Phase 3.
```

The focused near-term dependency order is:

```text
C1-001
  ├── C1-002 ──┐
  └── C1-003 ──┴── C1-004
                         ├── C1-005 → R1-002/R1-003
                         ├── C1-006 → C1-007
                         └── C1-010 → C1-011/C1-012

R1-003 → R1-004/R1-005 → R1-006
                              ├── R1-007
                              └── R1-008
                                   ↓
                              R1-009/R1-010
                                   ↓
                                R1-011
                                   ↓
                                R1-013

R1 relation vocabulary
    → C1-008/C1-009
    → C1-013
    → C1-014
    → Phase 2
```

---

# 12. Current gate completion · `gate:backlog:current`

Phase 1 is complete only when:

```text
required C1 dependency/graph prerequisites accepted
R1-001 … R1-011 DONE
R1-012 DONE or DROPPED with rationale
R1-013 DONE
```

The algorithm-preparation lane is ready for Phase 2 only when:

```text
C1-001 … C1-014 DONE
```

with the following allowed overlap:

- C1 synthetic linker/taptree prototypes may complete before production
  backend artifacts;
- final linker production interfaces remain blocked on actual relocatable
  tapscript artifacts;
- external solver adoption may remain `DROPPED` or deferred if exact pilot
  search is sufficient;
- sparse or parallel `faer` support may remain deferred if no measured need
  exists.

Until the current gate passes:

- production compiler analysis remains blocked;
- target-specific fields remain forbidden in realization;
- no stable linker or transaction ABI is published;
- no floating-point result becomes semantic identity;
- no draft bound becomes deployment calibration;
- no target prototype becomes release evidence.

---

# 13. One-line backlog · `rem:backlog:one-line`

> Establish reviewed graph, exact-math, numerical, optimization, SCC,
> relocation, and bounded-depth tree algorithms with independent small-instance
> oracles; then implement the typed realization on compact ASH and live transfer
> before production compiler, linker, or backend interfaces freeze.
