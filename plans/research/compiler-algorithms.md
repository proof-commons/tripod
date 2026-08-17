# Research Question: Compiler Analysis Algorithms · `q:compiler:algorithms`

> **Status:** Design and prototype required
> **Blocks:** compiler identity, relation normalization, declassification,
> proof planning, placement, and complete coverage analysis
> **Does not block:** Phase-1 typed realization declarations that do not freeze
> compiler-owned analysis identities
> **Affected packages:** realization, compiler, vectors, release
> **Depends on:** (`dec:source:typed-rust`),
> (`dec:architecture:realization-layer`),
> (`dec:assurance:translation-validation`)
> **Imports:** (`[ADR011-rule:toolchain:dependencies]`),
> (`[ADR011-rule:toolchain:reproducibility]`),
> (`[RZ-rem:oracle:thesis]`),
> (`[RZ-obl:oracle:representation]`),
> (`[RZ-obl:oracle:disclosure]`)
> **Related research:** (`q:numerical:linear-algebra`),
> (`q:optimization:solvers`)
> **Expected handoff:** accepted compiler algorithm policy, package-owned
> direct Petgraph graphs with typed key/index lookup metadata, exhaustive
> small-instance oracles, and implementation-ready complexity limits

## Question · `sec:compiler-algorithms:question`

Which graph, normalization, dependency-closure, proof-planning, placement, and
coverage algorithms should the target-independent compiler use so that its
results are correct, deterministic, auditable, and replaceable without changing
semantic identity?

The initial implementation must avoid two opposite failures:

1. independently reimplementing mature graph algorithms such as strongly
   connected components and reachability; and
2. allowing a graph library’s node indices, traversal order, serialization, or
   generic data model to become the compiler’s semantic contract.

The preferred graph substrate is `petgraph`.

The compiler still owns:

- typed AST and relation vocabulary;
- semantic identities;
- canonical ordering;
- source provenance;
- normalization legality;
- proof-selection constraints;
- constructibility and lifecycle semantics;
- placement and coverage rules;
- complexity limits;
- canonical publications.

## Fixed boundaries · `rule:compiler-algorithms:boundaries`

The compiler must maintain distinct typed models for at least:

1. expression dependencies;
2. semantic relations and activation;
3. proof alternatives and capability requirements;
4. witness availability and constructibility;
5. representation lifecycle;
6. relation carriers and execution cases;
7. disclosure dependencies;
8. evidence coverage.

These models use package-owned concrete Petgraph graphs directly. They may
share package-local construction and traversal helpers, but they are not one
universal graph schema and do not introduce a graph wrapper.

An AST is not merely a graph. It additionally owns:

- node arity;
- operand roles;
- semantic types;
- checked-failure behavior;
- operation ownership;
- structural identity;
- canonical encoding.

`petgraph` may store a projection of an AST for traversal and analysis. It does
not define the AST.

## Handle, key, and digest separation · `rule:compiler-algorithms:identity`

Compiler and realization algorithms distinguish:

```text
local handle:
    dense process-local arena or graph position

stable key:
    complete typed semantic identity

digest:
    optional domain-separated cache/publication commitment
```

A `petgraph::NodeIndex` is always a local handle.

It must not appear in:

- public package APIs;
- canonical publications;
- semantic hashes;
- evidence identities;
- diagnostics intended to survive one process;
- cross-derivation comparisons.

Stable keys must not depend on:

- declaration insertion order;
- graph insertion order;
- hash-map iteration;
- filesystem path;
- line number;
- source traversal;
- temporary allocation;
- target choice unless the identity is target-owned;
- wall-clock time.

A digest is not the sole collision boundary. Distinct complete structural keys
with one digest are a hard identity collision, never a silent merge.

## Canonical graph construction · `rule:compiler-algorithms:construction`

Every identity- or publication-relevant graph is constructed by:

1. collecting typed nodes;
2. validating unique stable keys;
3. sorting nodes by stable key;
4. inserting nodes in that order;
5. collecting typed edges;
6. validating endpoints, self-loop policy, and parallel-edge policy;
7. sorting edges by source key, target key, and typed edge key;
8. inserting edges;
9. freezing the graph before analysis.

The package-local key/index lookup metadata retains both:

```text
stable key → NodeIndex
NodeIndex → stable key
```

All external results convert back to stable keys.

A transformation normally creates a new frozen graph and an explicit
source-to-result mapping. In-place mutation must not invalidate handles already
captured by another analysis or report.

## Canonical topological analysis · `rule:compiler-algorithms:topology`

For acyclic dependency evaluation, use canonical Kahn ordering:

1. compute indegrees;
2. insert every zero-indegree node into an ordered ready set keyed by stable key;
3. repeatedly remove the least key;
4. decrement successors;
5. insert newly ready nodes;
6. if nodes remain, analyze the unresolved subgraph for cycles.

The expected complexity is:

\[
O((V+E)\log V)
\]

The logarithmic factor is accepted because canonical output is more important
than insertion-order-dependent linear traversal.

A library topological sort may be used as a cross-check or noncanonical
internal validation. Its incidental order must not define compiler identity.

When a cycle remains, diagnostics include:

- every cyclic strongly connected component;
- members in stable-key order;
- edge provenance;
- one deterministic representative cycle;
- source relation and operation ownership.

## Structural interning · `rule:compiler-algorithms:interning`

Initial expression interning uses complete typed structural keys.

Preferred initial structure:

```text
typed expression arena
+
BTreeMap<ExprKey, ExprHandle>
```

This gives deterministic full-key comparison without making a hash digest the
collision boundary.

The compiler may share one implementation node among several source relations,
but the shared node must retain every source relation ID and provenance record.

Structural sharing must not erase:

- operation ownership;
- activation;
- failure semantics;
- coverage ownership;
- disclosure dependencies;
- witness requirements.

## Normalization · `rule:compiler-algorithms:normalization`

Normalization is conservative and rule-driven.

An accepted normalization rule states:

- semantic type preconditions;
- domain preconditions;
- accepted-value equivalence;
- rejected-value and relevant failure equivalence;
- disclosure effect;
- witness effect;
- constructibility effect;
- provenance mapping;
- deterministic application order.

Initial permitted transformations should remain small:

- literal constant folding;
- boolean identities with preserved failure behavior;
- structural interning;
- canonical ordering of explicitly set-like operands;
- commutative ordering only where checked semantics prove it safe.

Do not automatically:

- reassociate checked arithmetic;
- distribute multiplication;
- combine or move floor operations;
- move checks across conditions;
- merge relations while dropping provenance;
- reorder guards when named failure behavior is relevant.

An e-graph or broad rewriting framework is deferred until the legal rewrite
catalogue and failure semantics are stable.

## Dependency closures · `rule:compiler-algorithms:closures`

Disclosure, fact availability, and related monotone analyses use deterministic
worklist closure.

For declassification:

1. seed facts required by public state, public events, public interface output,
   permissionless construction, target safety, or deployment policy;
2. attach typed disclosure reasons;
3. walk reverse dependencies;
4. union reason and provenance sets;
5. revisit a fact only when its set grows;
6. emit results in stable fact-key order.

The result retains:

```text
fact
reason
source relation
dependency provenance
```

A flat set of public facts is insufficient.

The production worklist is checked against a slower oracle that repeatedly
scans every dependency until no result changes.

## Proof planning · `rule:compiler-algorithms:proof-planning`

Proof planning proceeds in this order:

1. enumerate realization-approved proof alternatives;
2. reject missing target capability;
3. reject unauthenticated fact sources;
4. reject unavailable witnesses;
5. reject permissionless owner/operator secrets;
6. reject representation-policy violations;
7. reject lifecycle failures;
8. reject disclosure-policy failures;
9. compute the Pareto frontier;
10. select one plan under an explicit deterministic target policy.

Authorization, semantic strength, constructibility, lifecycle, closed-asset
identity, disclosure correctness, and relation coverage are hard constraints.

They are never converted into weighted penalties.

For pilot-sized inputs, selection uses exact deterministic enumeration or
branch-and-bound. A general SAT, LP, or MILP solver is not required initially.
The detailed solver threshold and certificate policy are owned by
(`q:optimization:solvers`).

## Placement and execution cases · `rule:compiler-algorithms:placement`

Placement is modeled over finite typed execution cases.

Examples include:

- sponsorless and sponsored;
- explicit and confidential representation;
- continuing and terminal;
- empty and nonempty;
- pre-maturity, conversion, and post-maturity.

For each relation \(r\), require:

\[
\operatorname{requiredCases}(r)\subseteq\bigcup_{c\text{ carries }r}\operatorname{executedCases}(c)
\]

A carrier being present or reachable somewhere in a bundle is insufficient.

The carrier must execute in every case in which the relation is active.

Initial placement uses exact finite search with:

- canonical candidate ordering;
- hard-constraint pruning;
- explicit objective vector;
- canonical tie-breaking;
- maximum search-state budget;
- typed complexity failure;
- exhaustive small-instance oracle.

## Coverage analysis · `rule:compiler-algorithms:coverage`

Compiler coverage requires exact set equality among:

```text
realization relation scope
compiler relation scope
placement requirement scope
coverage requirement scope
```

Every relation receives:

- at least one active accepting case;
- at least one focused rejecting mutation;
- activation cases;
- representation cases;
- carrier requirements;
- accepted semantic projection checks.

Conditional relations additionally require:

- inactive valid case;
- active valid case;
- active invalid case.

Coverage sets begin as `BTreeSet<TypedId>`.

`fixedbitset` may later replace internal dense sets when profiling or clarity
justifies it. Bit positions remain local handles and never become semantic IDs.

## Complexity limits · `rule:compiler-algorithms:limits`

Every potentially superlinear or exponential analysis defines explicit limits:

- maximum graph nodes;
- maximum graph edges;
- maximum parallel typed edges;
- maximum SCC size accepted by a pass;
- maximum execution cases;
- maximum proof-selection states;
- maximum placement states;
- maximum diagnostic cycle length.

A limit is typed configuration or a validated derivation from architecture and
deployment-independent bounds.

Exceeding a limit returns a typed failure. It never silently switches to a
greedy or weaker algorithm.

## Dependency candidates · `tab:compiler-algorithms:dependencies`

| Dependency | Initial status | Intended role |
|---|---|---|
| `petgraph` | preferred | graph storage, SCC, reachability, traversal |
| `fixedbitset` | conditional | dense local coverage and case sets |
| `faer` | conditional compiler use | numerical diagnostics and certified numerical analyses |
| `num-bigint` | existing | exact integer oracle |
| `num-rational` | recommended | exact rational coefficients and certificates |
| `proptest` | existing | generated graph and solver-oracle tests |
| `salsa` | deferred | incremental query framework |
| `egg` | deferred | equality saturation |
| SAT/LP/MILP solver | deferred | large exact planning problems |
| `rayon` | deferred | parallel independent analyses |

A dependency is added only to an actual consumer.

No generic first-party graph package is created until at least two implemented
consumers demonstrate one stable common abstraction.

## Prototype · `sec:compiler-algorithms:prototype`

### Stage 1 — canonical direct-Petgraph construction

Implement:

- package-owned concrete Petgraph graph construction;
- stable-key node insertion;
- typed edge insertion;
- duplicate and self-loop policy;
- stable-key/local-index maps;
- canonical typed node and edge projections;
- insertion-permutation tests.

### Stage 2 — topology and SCC

Implement canonical Kahn ordering and canonical SCC diagnostics.

Test:

- DAGs;
- self-loops;
- one large SCC;
- disconnected graphs;
- deep chains;
- dense small graphs;
- insertion permutations.

### Stage 3 — expression interning

Implement typed structural interning and compare with a non-interned evaluator.

### Stage 4 — dependency closure

Implement declassification closure and compare with the repeated-full-scan
oracle.

### Stage 5 — proof planning

Implement exact pilot proof selection and compare every small generated
instance with exhaustive enumeration.

### Stage 6 — placement and coverage

Implement execution-case-aware carrier selection and compare small instances
with exhaustive carrier-subset enumeration.

### Stage 7 — complete pilot analysis

Analyze compact ASH and live transfer end to end with deterministic identities,
provenance, proof alternatives, placement, and coverage.

## Required vectors · `sec:compiler-algorithms:vectors`

- source node permutation;
- edge insertion permutation;
- duplicate node key;
- duplicate edge under every edge-multiplicity policy;
- missing endpoint;
- self-loop;
- cycle with deterministic diagnostic;
- unrelated-node insertion;
- structural sharing with several source relation IDs;
- normalization preserving values but changing failure behavior;
- disclosure seed with several dependency paths;
- permissionless proof requiring a private owner witness;
- proof alternative missing capability;
- proof alternative preserving safety but breaking lifecycle;
- relation carried only by an optional unexecuted program;
- uncovered execution case;
- equal-cost proof and placement ties;
- search budget exhaustion;
- raw graph index serialization attempt.

## Measurements · `sec:compiler-algorithms:measurements`

Record for synthetic and pilot graphs:

- node and edge counts;
- graph construction time;
- topology and SCC time;
- interning ratio;
- closure iterations/worklist updates;
- proof candidate count;
- proof search states and pruning;
- placement search states and pruning;
- peak memory;
- canonical output equality under permutation.

Performance measurement informs implementation choice. It does not weaken
correctness or determinism requirements.

## Acceptance · `gate:compiler-algorithms:accept`

Accept the compiler algorithm set when:

- typed ASTs remain first-party;
- `petgraph` indices stay private;
- stable keys and local handles are distinct;
- canonical topology is insertion-order independent;
- SCC diagnostics are deterministic and stack-safe for accepted sizes;
- normalization has an explicit legal rule catalogue;
- declassification retains reasons and provenance;
- proof planning enforces hard constraints before cost;
- placement covers every active execution case;
- exact search agrees with exhaustive small-instance oracles;
- complexity limits fail closed;
- repeated pilot analysis is equal;
- no raw graph or floating-point working value enters semantic identity.

## Rejection · `gate:compiler-algorithms:reject`

Reject an algorithm or dependency choice if it:

- exposes node indices as semantic IDs;
- depends on traversal or insertion order;
- serializes a third-party graph container directly;
- silently drops relation provenance during sharing;
- treats proof selection as per-relation greedy choice;
- places a relation only on an optional carrier that need not execute;
- falls back to a weaker heuristic after complexity exhaustion;
- uses floating-point rank or residual as proof of exact semantic equality;
- requires target-specific types in compiler core;
- cannot be checked against an independent small-instance oracle.

## Result · `sec:compiler-algorithms:result`

Pending.

## Handoff · `sec:compiler-algorithms:handoff`

An accepted result updates:

- realization graph and identity implementation;
- compiler package contract;
- compiler error vocabulary;
- analysis schemas;
- declassification implementation;
- proof and placement policies;
- coverage-report schema;
- vector generators;
- release evidence requirements;
- Phase-1 and Phase-2 exit gates.

Internal algorithms may later be replaced without changing public semantic
contracts when they produce the same canonical typed result.
