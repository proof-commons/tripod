# Phase 2 — Target-Independent Compiler Analysis · `phase:roadmap:compiler`

> **Status:** Active
> **Entry:** (`gate:phase1:exit`)
> **Package:** [`compiler`](../packages/compiler.md)
> **Decisions:** D001–D006
> **Concrete target dependency:** prohibited in compiler core

## Goal · `sec:phase2:goal`

Lower the two pilot realization declarations into one deterministic
target-independent analyzed program.

The compiler must expose every semantic relation, proof alternative,
disclosure reason, fact source, constructibility requirement, lifecycle
requirement, placement requirement, layout requirement, target requirement,
and evidence obligation before target emission starts.

## Entry conditions · `sec:phase2:entry`

- Phase 1 passed;
- pilot realization scope is explicit;
- model conformance for both pilots passes;
- realization IDs and dependency traversal are deterministic;
- no stable target-specific assumption is required by compiler core.

## Deliverables · `sec:phase2:deliverables`

### Crate — complete

Created, with the crate boundary, workspace and census membership, typed error
root, and public-API test in place:

```text
packages/compiler
tripod-compiler
library: compiler
```

Direct first-party dependency:

```text
realization
```

A direct `architecture` dependency was additionally taken on the contract's
stated exception, because the public error vocabulary names operations and
realization does not re-export `OperationId`.

The package existing is not the compiler existing.

### Input binding — complete

The validated immutable input boundary (P2-004): explicit canonical scope,
the strict analysis policy, owner revalidation, and the bound input value.

### Relation DAG — complete

Delivered under C1-005/P2-005 as a crate-private analysis (with the scoped
expression graph and predicate-binding validation); the stable projection
remains internal until the P2-012 analyzed program.

### Checked constant folding — complete

Delivered under P2-006: closed-constant folds only, checked arithmetic,
failure-preserving conjunction, an independent reference oracle, and
permutation-stable folded projections.

### Proof planning, sources, constructibility, disclosure, lifecycle — complete

Delivered under C1-008/P2-007/P2-008/P2-009 as crate-private analyses:
relation obligations (proof-required, static, external evidence), typed
operands with authenticated source requirements, authorization-case
constructibility, plan-specific disclosure with typed reasons,
representation lifecycle with explicit future exits, and deterministic
exact feasible-plan enumeration checked against an independent
exhaustive oracle. No compiler-plan identity was minted.

The five requirement sections that follow record what those delivered analyses
must satisfy. They are implemented as crate-private stages, not as a public
analyzed program.

### Proof alternatives — requirement of the delivered analysis

Represent semantic proof alternatives separately from target implementation
patterns.

Pilot examples:

```text
compact ASH conservation:
    public arithmetic

live transfer conservation:
    explicit arithmetic
    confidential transaction conservation
```

### Disclosure analysis — requirement of the delivered analysis

Separate:

1. semantic disclosure;
2. target-safety disclosure;
3. deployment-policy disclosure.

Every disclosure has typed reason and provenance.

### Fact-source analysis — requirement of the delivered analysis

Every operand has an authenticatable source requirement.

Unauthenticated metadata is not a source.

### Constructibility — requirement of the delivered analysis

Check both:

```text
proof can be verified
witness can be obtained by the authorized constructor
```

Compact ASH must require no owner/operator secret.

### Lifecycle — requirement of the delivered analysis

Track required exits for supported representations.

Private live receipts retain future obligations for:

```text
transfer
burn
redeem
```

Pilot support may remain lifecycle-incomplete for later target operations, but
that status must be explicit.

Every deliverable below remains open, as does the complete pilot analyzed
program and its public boundary.

### Placement requirements

Classify each relation as:

- local;
- transaction-global;
- conditional;
- duplicated.

Produce semantic carrier requirements without concrete target indexes.

### Layout requirements

Record:

- bounded repeated families;
- count/range authentication;
- complete/disjoint output families;
- optional family conditions;
- sponsor isolation;
- witness availability requirements.

### Coverage requirements

For every relation, derive:

- accepting case;
- rejecting mutation;
- activation cases;
- representation cases;
- carrier requirement;
- accepted semantic projection checks.

## Pilot analysis · `sec:phase2:pilots`

### Compact ASH

Require complete analysis of:

- cardinality;
- ASH recognition;
- permissionless authorization;
- ownerless conservation;
- one output;
- sponsor isolation;
- no roots;
- no specialized event;
- public constructibility.

### Live transfer

Require complete analysis of:

- input/output bounds;
- live class;
- all-owner authorization;
- aggregate conservation alternatives;
- explicit closed `U`;
- output-family closure;
- sponsor isolation;
- lifecycle obligations;
- explicit/confidential representation evidence requirements.

## Evidence · `sec:phase2:evidence`

- exact relation-census equality with realization;
- no dropped relation;
- deterministic DAG and IDs;
- checked folding boundary tests;
- proof-alternative pruning with mock capabilities;
- missing source rejection;
- permissionless secret rejection;
- missing lifecycle path detection;
- unplaced-relation rejection;
- missing layout requirement rejection;
- complete coverage skeleton;
- public downstream API tests;
- repeated analysis equality.

A mock target validates compiler failure modes only. It is not target evidence.

## Non-goals · `sec:phase2:non-goals`

Phase 2 emits no:

- tapscript;
- target constructor;
- concrete layout;
- transaction ABI;
- target execution;
- linked bundle;
- deployment profile.

## Exit gate · `gate:phase2:exit`

Phase 2 exits when:

- compiler relation scope exactly equals pilot realization scope;
- every relation has proof, source, constructibility, lifecycle, placement,
  layout, target, and coverage information;
- unsupported targets fail without semantic weakening;
- compiler core has no concrete target dependency, opcode, stack item, or
  transaction index;
- both pilot analyses are deterministic;
- derivative reports, if any, are canonical and non-authoritative;
- workspace checks remain green and clean.
