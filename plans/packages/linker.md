# Linker · `pkg:linker:contract`

> **Status:** Candidate — first bundle delivered, exit gate partly met
> **Phase:** [Phase 4](../phases/04-compact-ash.md) onward
> **Package:** `tripod-linker`
> **Library:** `linker`
> **Direct dependencies:** `tapscript`, `target-elements`
> **Decisions:** [D004](../decisions/004-translation-validation.md),
> [D006](../decisions/006-transaction-abi.md)

## Purpose · `sec:linker:purpose`

`linker` resolves typed relocatable backend artifacts into deterministic
candidate and final deployment bundles.

It owns:

- typed symbols and references;
- constructor reference graph;
- strongly connected components;
- reference-strategy validation;
- relocation;
- static constructor/program resolution;
- deterministic taptree assembly;
- control-path recipes;
- deployment constant substitution;
- relation-carrier preservation;
- linked resource formulas;
- candidate and final bundle identity.

It does not construct complete transactions.

## Dependencies · `sec:linker:dependencies`

Allowed direct dependencies:

```text
tapscript
target-elements
```

Forbidden dependencies:

```text
model
transaction
vectors
release
artifacts
```

Calibration orchestration lives above linker and transaction to prevent a
dependency cycle.

## Typed inputs · `sec:linker:inputs`

The linker consumes:

- validated relocatable tapscript bundle;
- exact typed target;
- typed public deployment parameters;
- typed candidate or final bound assignment;
- typed linker and taptree policy.

Deployment parameters may include public keys, asset IDs, network/genesis
bindings, schema constants, and domain separators.

Private keys never enter the linker.

## Typed outputs · `sec:linker:outputs`

A candidate bundle contains:

- all upstream identity bindings;
- candidate bounds;
- linked constructors;
- linked operation programs;
- taptrees and control recipes;
- concrete relation placements;
- layout and witness handoff;
- linked resource formulas;
- relation-carrier census;
- reference and relocation reports.

A final bundle additionally binds final calibration evidence and has no
unresolved mandatory value.

Candidate and final bundles are distinct typed states.

> Illustrative boundary; names and exact fields are not frozen.

```rust
pub fn link_candidate(
	relocatable: &tapscript::RelocatableTapscriptBundle,
	target: &target_elements::ElementsTarget,
	deployment: &LinkDeploymentParameters,
	bounds: &BoundAssignment,
) -> Result<CandidateLinkedBundle, LinkError>;
```

## Symbols · `rule:linker:symbols`

Symbols identify typed roles rather than display strings.

Examples:

- object constructor;
- operation program;
- static code subtree;
- metadata schema;
- internal key;
- architecture asset ID;
- calibrated bound;
- deployment key;
- network constant;
- domain separator.

Every mandatory reference has one compatible definition.

## Reference graph · `rule:linker:references`

References are classified as:

- static link-time constants;
- identity introspection;
- in-program constructor reconstruction;
- authenticated witnessed-root continuity;
- deployment relocation;
- unsupported dependency.

The linker computes deterministic SCCs and a condensation DAG.

A cycle is not accepted merely because it belongs to one SCC. A cycle is
resolved when at least one of its edges carries an explicit authenticated
resolution strategy, and refused when any edge carries none. Requiring every
cyclic edge to be cycle-resolving would refuse every cycle without exception:
a constructor binding its own leaves is an ordinary static reference, cyclic
only because something else closes the loop.

Arbitrary repeated hashing until bytes stabilize is prohibited.

The first delivered link found one component. The compact-ASH constructor's
witness program is the taproot output committing to the tree over the leaves,
and those leaves carry that program as a link-time literal, so the symbol's
value is a function of itself. With no strategy stated the link refuses as an
impossible static fixed point. Of the two strategies that cut the edge, the
sound one — the referring program obtaining the identity by introspecting the
input it is spending — is checked rather than believed, and the emitted leaves
currently contradict it; the other cuts by external authentication and leaves
the commitment equality as a recorded, undischarged obligation.

## Relocations · `rule:linker:relocations`

Each relocation specifies:

- typed source;
- typed target;
- semantic location;
- width/domain;
- encoding;
- multiplicity;
- provenance.

Structured patching is preferred.

If byte patching is necessary, the expected placeholder and post-link program
are revalidated.

Every mandatory relocation resolves exactly once.

## Constructors · `rule:linker:constructors`

The linker resolves static constructor components and emits recipes for dynamic
metadata instances.

A constructor binds:

- object kind;
- target;
- internal key;
- static operation-program set;
- metadata schema;
- target commitment rule;
- predecessor/successor continuity strategy.

Dynamic owner or state values remain transaction-time parameters.

## Taptrees · `rule:linker:taptrees`

The initial tapscript tree policy is deterministic.

It defines:

- complete leaf set;
- leaf version;
- weight source;
- stable tie-break;
- branch ordering;
- depth constraints;
- static and dynamic constructor composition.

Weights are implementation configuration, not protocol semantics.

Changing them moves bundle/ABI identity and requires recalibration.

## Carrier closure · `rule:linker:carriers`

The linker compares:

```text
compiler-required relation carriers
backend-emitted carriers
linked reachable carriers
```

A uniquely carrying program cannot be removed or made unreachable.

Initial linking performs no semantic dead-code elimination beyond explicitly
out-of-scope artifacts.

## Resources · `rule:linker:resources`

Linking resolves resource variables determined by final program bytes,
constants, taptree depths, and bounds.

The result remains symbolic over transaction-time family counts where needed.

Complete transaction measurement occurs downstream.

## Calibration handoff · `rule:linker:calibration`

A higher-level runner repeats:

1. candidate bound assignment;
2. candidate linking;
3. candidate ABI derivation;
4. worst-case transaction construction;
5. target measurement;
6. deterministic bound selection.

After selection, the exact final bundle and ABI are regenerated and
remeasured.

The linker does not call `transaction`.

## Identity · `rule:linker:identity`

Linked-bundle identity binds:

- architecture, realization, compiler, target, and backend configuration;
- public deployment constants;
- calibrated bounds;
- constructors and programs;
- taptrees/control recipes;
- relation carriers;
- layout/witness handoff;
- resource formulas;
- reference and relocation results.

It excludes secrets, host paths, timestamps, and diagnostics.

## Assurance boundary · `sec:linker:assurance`

The linker establishes resolution, continuity, deterministic assembly, and
carrier preservation.

It does not establish:

- semantic completeness;
- backend pattern correctness;
- transaction/witness correctness;
- target-node behavior;
- calibration validity until final evidence is supplied.

## Exit gate · `gate:linker:first-bundle`

Met by the first delivered link except for the last clause, which waits on the
transaction package.

The self-commitment now resolves by identity introspection. The ASH
constructor's witness program is the taproot output over the taptree the
emitted leaves are committed in, so no layer can supply its bytes; the
coordinator leaves read it off the input they are spending instead of
carrying a literal for it, and the comparison establishes that the compared
position shares this input's program rather than that it equals a named value.
The symbol is therefore declared and settled by nobody — bound
`ReadFromTargetAtSpendTime`, backed by an introspection-reference census
beside the relocation census — and a deployment offering a value for it is
refused. Under that strategy the demonstration bundle links with no
self-commitment equality outstanding; with no strategy stated the same cycle
is still unclassified and still refuses. Evidence is in
`packages/linker/src/tests/link_tests.rs` (the sound strategy links and owes
no equality; a literal reaching a leaf still contradicts it) and
`packages/linker/src/tests/graph_tests.rs` (the nine coordinator leaves carry
the introspection edges that close the loop).

The first candidate compact-ASH bundle exits when:

- every symbol and mandatory relocation resolves;
- constructor graph and SCC strategies validate;
- linked ASH programs and taptree are deterministic;
- every relation carrier remains reachable;
- layout/witness/resource handoff is complete;
- candidate status is explicit;
- transaction can consume the typed output without linker mutation.

## Error vocabulary · `sec:linker:errors`

See [`errors/linker.md`](errors/linker.md).

## Open questions · `sec:linker:open`

- Which linked-artifact roles are truly backend-neutral?
- What length-limited taptree algorithm replaces the plain minimum-weighted-depth
  construction, which minimizes cost and refuses rather than rebalances when a
  declared depth bound is exceeded?
- What is the canonical bundle archive format?
- Does calibration remain in release or later move to a dedicated package?
- Which package owns genesis and issuance ceremony construction?
