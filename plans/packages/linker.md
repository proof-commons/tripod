# Linker · `pkg:linker:contract`

> **Status:** Planned
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

A cycle is not accepted merely because it belongs to one SCC. Every cyclic edge
requires an explicit authenticated resolution strategy.

Arbitrary repeated hashing until bytes stabilize is prohibited.

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

The first candidate compact-ASH bundle exits when:

- every symbol and mandatory relocation resolves;
- constructor graph and SCC strategies validate;
- linked ASH programs and taptree are deterministic;
- every relation carrier remains reachable;
- layout/witness/resource handoff is complete;
- candidate status is explicit;
- transaction can consume the typed output without linker mutation.
