# Research Question: Linker and Bundle Algorithms · `q:linker:algorithms`

> **Status:** Design and prototype required
> **Blocks:** final symbol model, constructor reference handling, relocation,
> canonical taptree policy, carrier closure, resource resolution, and linked
> bundle identity
> **Does not block:** target-independent compiler analysis
> **Affected packages:** tapscript, linker, transaction, vectors, release
> **Depends on:** (`dec:backend:tapscript-first`),
> (`dec:assurance:translation-validation`),
> (`dec:abi:canonical-transactions`)
> **Imports:** (`[ADR011-rule:toolchain:dependencies]`),
> (`[ADR011-rule:toolchain:reproducibility]`),
> (`[RZ-rule:translation:cross-utxo]`),
> (`[RZ-rule:translation:certificate-leaf]`),
> (`[RZ-pin:pins:weld]`)
> **Related research:** (`q:compiler:algorithms`),
> (`q:optimization:solvers`),
> (`q:constructor:state`)
> **Expected handoff:** accepted symbol/reference graph, SCC strategy,
> structured relocation model, deterministic taptree algorithm, carrier
> closure algorithm, and implementation-ready resource limits

## Question · `sec:linker-algorithms:question`

Which graph, symbol-resolution, cycle-classification, relocation, target-tree,
carrier-closure, and resource-resolution algorithms should the linker use so
that one linked bundle is deterministic, complete, auditable, and safely bound
to its compiler and target inputs?

The linker must avoid:

- string-based symbol replacement;
- traversal-order-dependent linking;
- accepting a cyclic graph merely because an SCC was found;
- repeated cryptographic hashing until bytes happen to stabilize;
- sequential variable-width byte patching;
- accidental source-order taptrees;
- removal of a uniquely carrying program;
- calibration against a bundle different from the final one.

`petgraph` is the preferred private substrate for reference and carrier graphs.

Typed symbols, cycle strategies, relocation semantics, taptree objectives,
resource formulas, and bundle publications remain first-party.

## Graph families · `rule:linker-algorithms:graphs`

The linker maintains distinct graph models for:

1. typed symbol definitions and references;
2. constructor/static-program dependencies;
3. relocation dependencies;
4. target program and control-path reachability;
5. relation-carrier ownership;
6. resource-formula dependencies;
7. candidate-to-final calibration dependencies.

One private graph adapter may support these models, but edge kinds and
acceptance rules remain graph-specific.

A graph edge records:

- typed source and target;
- edge role;
- expected target type;
- mandatory or optional status;
- reference strategy;
- source provenance.

Display strings are not symbol identity.

## Symbol resolution · `rule:linker-algorithms:symbols`

Use two-pass typed symbol resolution.

### Pass 1 — definition census

- collect every typed definition;
- validate unique symbol keys;
- reject incompatible aliases;
- record definition type and provenance;
- sort definitions by stable symbol key.

### Pass 2 — reference resolution

- resolve every reference against the complete census;
- verify expected symbol type;
- record the exact reference edge;
- reject missing, ambiguous, or incompatible targets;
- build the frozen reference graph.

A definition discovered later in source traversal must not change the meaning of
an earlier unresolved string.

The complete symbol table is established before reference substitution begins.

## Stable handles and publications · `rule:linker-algorithms:identity`

`NodeIndex`, `EdgeIndex`, relocation-array position, source-file order, and
serialized program order are local handles only.

Linked identity derives from typed canonical values:

- symbol key;
- program identity;
- constructor role;
- relation carrier;
- target;
- deployment constants;
- bound assignment;
- canonical taptree;
- relocation result;
- resource formulas.

No petgraph container or raw index is serialized directly.

## Strongly connected components · `rule:linker-algorithms:scc`

Use a reviewed linear-time SCC algorithm over the frozen reference graph.

The preferred initial candidate is an iterative or otherwise stack-safe
Kosaraju implementation using `petgraph` storage and traversal support.

For each SCC:

1. sort members by stable symbol key;
2. derive a canonical SCC key;
3. preserve every internal and outgoing edge role;
4. build a typed condensation DAG;
5. canonical-toposort the condensation DAG;
6. report cyclic SCCs deterministically.

The expected graph-analysis complexity is:

\[
O(V+E)
\]

for SCC membership, followed by canonical ordering overhead.

A recursive traversal is acceptable only if the selected implementation is
proven safe under the accepted maximum graph depth. Otherwise use explicit
heap stacks.

## Cycle strategies · `rule:linker-algorithms:cycles`

SCC membership does not make a reference cycle legal.

Every cyclic edge must carry one explicit strategy:

- resolved static link-time value;
- target identity introspection;
- authenticated witnessed-root continuity;
- in-program constructor reconstruction;
- deployment relocation;
- another reviewed non-recursive strategy;
- unsupported.

A cyclic SCC validates only when every cycle is broken semantically by one or
more strategies whose target evidence establishes the required continuity.

Reject:

```text
hash program
substitute hash
repeat until bytes stop changing
```

Cryptographic constructor equations do not acquire correctness through
accidental iterative convergence.

The STATE constructor cycle policy is resolved by (`q:constructor:state`), not
invented by generic linker traversal.

## Relocation model · `rule:linker-algorithms:relocation`

Prefer structured linking before target-byte serialization.

A relocation identifies:

- typed source symbol;
- typed destination;
- semantic program location;
- target encoding;
- width/domain;
- expected placeholder;
- multiplicity;
- source provenance.

Mandatory rules:

1. collect and validate all relocations before mutation;
2. resolve every mandatory relocation exactly once;
3. reject unknown or duplicate incompatible relocations;
4. reject overlapping destination ranges;
5. compute every replacement against pristine input;
6. sort replacements by canonical destination;
7. apply replacements simultaneously;
8. reparse and validate the complete result;
9. recompute linked program identity from the validated result.

Variable-width byte relocation is prohibited.

A value whose length changes the target program must be substituted at the
structured-program layer before offsets are assigned.

## Constructor continuity · `rule:linker-algorithms:constructors`

Constructor linking binds:

- object kind;
- metadata schema;
- target;
- internal-key policy;
- static operation-program set;
- dynamic metadata commitment rule;
- predecessor/successor continuity strategy;
- target tree/control recipe.

The linker computes constructor reference SCCs and validates the accepted
strategy for every cyclic dependency.

It must reject:

- independent predecessor and successor static roots;
- stale constructor from another bundle;
- unresolved dynamic metadata relation;
- untracked program mutation after relocation;
- key or metadata escape path;
- constructor whose uniquely carrying program becomes unreachable.

## Carrier closure · `rule:linker-algorithms:carriers`

The linker compares:

```text
compiler-required carriers
backend-emitted carriers
linked reachable carriers
execution-case carrier coverage
```

For every relation:

- at least one compatible carrier exists;
- every required execution case reaches a carrier;
- all mandatory facts used by that carrier resolve;
- a carrier’s target program remains reachable;
- intentional duplicate carriers agree on semantic relation identity.

A program containing a unique carrier cannot be removed as dead code.

Initial linking performs no semantic dead-code elimination beyond explicitly
out-of-scope artifacts.

## Canonical taptree objective · `rule:linker-algorithms:taptree`

The tree algorithm is selected only after the objective is explicit.

Possible objectives are:

| Objective | Algorithm family |
|---|---|
| minimum maximum depth | balanced/minimax |
| minimum weighted expected depth | Huffman |
| weighted depth with hard maximum | length-limited Huffman/package-merge |
| semantic leaf order preserved | alphabetic coding, only if required |

The preferred initial policy is:

> deterministic length-limited Huffman construction by package-merge, with
> positive integer weights, a hard maximum depth, and stable-key tie-breaking.

If no defensible execution-frequency data exists, use equal weights.

Canonical inputs include:

- leaf version;
- target program identity;
- semantic role;
- integer weight;
- maximum depth;
- target branch-hash ordering rule.

Equal-cost ties use stable leaf keys.

Target child order follows the exact target TapBranch hash rule.

Duplicate leaf bytes are not silently deduplicated. Any deliberate coalescing
retains the union of relation-carrier provenance and must preserve witness and
control semantics.

Tree policy and weights are backend/linker configuration and therefore
bundle/ABI identity inputs.

## Taptree oracle · `rule:linker-algorithms:tree-oracle`

For small leaf sets, enumerate every full binary tree and compare the selected
tree against the declared objective.

For weights \(w_i\), leaf depths \(d_i\), and depth limit \(L\), verify:

\[
\min\sum_i w_i d_i\quad\text{subject to}\quad d_i\le L
\]

The exhaustive oracle need not scale beyond small test cases. It exists to
validate package-merge, tie-breaking, and depth constraints before tree policy
becomes identity-bearing.

## Resource formulas · `rule:linker-algorithms:resources`

Resource formulas use a small first-party checked language.

Initial forms include:

- constants;
- bounded variables;
- checked addition;
- multiplication by nonnegative constants;
- maximum;
- conditional operation cases;
- representation alternatives;
- target tree depth;
- family-count terms.

Units remain distinct:

```text
transaction weight
witness bytes
script bytes
stack items
altstack items
element bytes
crypto budget
target operation cost
```

Unlike units are never combined into one untyped numeric score.

Linking resolves variables determined by:

- final program bytes;
- relocation values;
- constructor policy;
- tree depths;
- calibrated bounds.

Transaction-time family counts may remain symbolic for downstream ABI and
calibration.

Numerical fitting may diagnose a resource formula through
(`q:numerical:linear-algebra`), but the structural formula and target
measurements remain authoritative.

## Calibration search · `rule:linker-algorithms:calibration`

Binary search over a bound is permitted only after proving:

\[
\operatorname{fits}(n)\Rightarrow\operatorname{fits}(m)\quad\text{for every }m\le n
\]

under one fixed candidate policy.

Monotonicity may fail when a bound changes:

- proof selection;
- layout;
- constructor;
- taptree;
- representation;
- target program.

If monotonicity is not established, enumerate the finite candidate range
deterministically or use an accepted exact optimization method under
(`q:optimization:solvers`).

After final values are selected:

1. relink the exact final bundle;
2. regenerate the final ABI;
3. regenerate every affected fixture;
4. remeasure the exact final transactions;
5. bind final identities and reports.

Candidate evidence never silently validates different final bytes.

## Dependency candidates · `tbl:linker-algorithms:dependencies`

| Dependency | Initial status | Intended role |
|---|---|---|
| `petgraph` | preferred | symbol/reference graphs, SCC, reachability |
| `faer` | conditional | numerical resource diagnostics only |
| `num-rational` | conditional | exact resource/certificate arithmetic |
| `fixedbitset` | conditional | dense carrier/case sets |
| generic Huffman crate | not preferred | project needs explicit bounded-depth policy |
| SAT/LP/MILP solver | deferred | large placement/calibration optimization |
| parallel graph framework | deferred | no measured need |

Package-merge, typed relocation, symbol semantics, and canonical bundle DTOs
remain first-party.

## Prototype · `sec:linker-algorithms:prototype`

### Stage 1 — typed symbol census

Build a synthetic relocatable bundle with typed definitions and references.

Test missing, duplicate, ambiguous, incompatible, forward, and disconnected
symbols.

### Stage 2 — SCC and condensation

Build constructor/reference graphs covering:

- DAG;
- self-loop;
- simple cycle;
- several nested SCCs;
- one large SCC;
- deep chain;
- disconnected components.

Compare SCC membership with a mutual-reachability oracle on small graphs.

### Stage 3 — cycle strategies

Attach explicit strategies and require every unresolved cycle to fail.

### Stage 4 — structured relocation

Link a structured synthetic program, then compare with an independently
assembled expected final program.

Test overlap, width, encoding, placeholder, multiplicity, and stale-offset
faults.

### Stage 5 — carrier closure

Propagate compiler-required carriers through backend and linked programs.

Test unreachable and optional-only carriers.

### Stage 6 — taptree package-merge

Implement readable deterministic package-merge and compare with exhaustive
small-tree enumeration.

### Stage 7 — resource formulas

Evaluate formulas against complete synthetic and target transactions.

### Stage 8 — compact-ASH candidate bundle

Exercise the entire algorithm set on the first real relocatable backend
artifact.

## Required vectors · `sec:linker-algorithms:vectors`

- symbol insertion permutation;
- reference insertion permutation;
- forward reference;
- duplicate definition;
- incompatible symbol type;
- missing and ambiguous reference;
- self-loop with and without strategy;
- multi-node SCC with one unresolved edge;
- repeated-hashing fixed-point attempt;
- relocation overlap;
- relocation applied twice;
- wrong width or byte order;
- stale placeholder;
- variable-width relocation attempt;
- untracked post-link program mutation;
- unique relation carrier removed;
- carrier present but unreachable;
- relation carrier reachable only in the wrong execution case;
- one/equal/unequal-weight leaves;
- impossible tree depth;
- stable tie permutations;
- duplicate leaf bytes;
- candidate/final bundle confusion;
- invalid monotonicity assumption during calibration.

## Measurements · `sec:linker-algorithms:measurements`

Record:

- definitions and references;
- graph nodes and edges;
- SCC count and largest SCC;
- symbol-resolution time;
- SCC/condensation time;
- relocation count and bytes;
- carrier count and coverage time;
- taptree leaf count, maximum depth, and weighted depth;
- package-merge time;
- resource-formula evaluation time;
- candidate-link and final-link time;
- peak memory;
- output equality under insertion permutations.

## Acceptance · `gate:linker-algorithms:accept`

Accept the linker algorithms when:

- symbols and references are typed and resolved in two passes;
- graph indices remain private;
- SCC results are canonical and stack-safe;
- every cyclic edge has an explicit authenticated strategy;
- repeated cryptographic fixed-point linking is prohibited;
- relocations are structured or fixed-width and simultaneous;
- post-link programs are reparsed and validated;
- carrier closure covers every required execution case;
- package-merge agrees with exhaustive small-tree oracles;
- tree policy and weights are explicit identity inputs;
- resource formulas retain typed units;
- calibration search uses a proven monotonicity premise or deterministic
  enumeration;
- candidate and final bundles remain distinct;
- compact-ASH linking is deterministic.

## Rejection · `gate:linker-algorithms:reject`

Reject an algorithm if it:

- resolves by display string or traversal order;
- accepts an SCC without validating cycle strategies;
- repeatedly hashes until apparent convergence;
- applies overlapping or variable-width byte patches;
- mutates bytes before all relocation destinations validate;
- removes a uniquely carrying program;
- treats reachability somewhere as execution in every required case;
- builds a tree from source order;
- uses speculative floating weights without a policy;
- assumes calibration monotonicity without proof;
- accepts candidate measurements as final evidence;
- serializes petgraph indices or containers as bundle identity.

## Result · `sec:linker-algorithms:result`

Pending.

## Handoff · `sec:linker-algorithms:handoff`

An accepted result updates:

- linker package contract;
- linker error vocabulary;
- relocatable and linked bundle schemas;
- tapscript symbol/relocation interface;
- constructor strategy;
- transaction ABI handoff;
- taptree policy;
- carrier-coverage report;
- resource and calibration reports;
- compact-ASH Phase-4 implementation gate;
- release identity and evidence rules.
