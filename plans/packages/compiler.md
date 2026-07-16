# Target-Independent Compiler · `pkg:compiler:contract`

> **Status:** Planned
> **Phase:** [Phase 2](../phases/02-compiler.md)
> **Package:** `tripod-compiler`
> **Library:** `compiler`
> **Decisions:** [D001](../decisions/001-typed-rust-source.md),
> [D002](../decisions/002-realization-layer.md),
> [D003](../decisions/003-tapscript-first.md),
> [D004](../decisions/004-translation-validation.md),
> [D005](../decisions/005-value-representation.md),
> [D006](../decisions/006-transaction-abi.md)

## Purpose · `sec:compiler:purpose`

`compiler` analyzes a validated realization into a deterministic
target-independent compilation plan.

It owns:

- relation DAG construction;
- constant folding;
- proof alternatives;
- disclosure analysis;
- fact-source requirements;
- witness availability;
- constructibility;
- lifecycle reachability;
- placement requirements;
- layout requirements;
- abstract target requirements;
- coverage requirements;
- analysis identity and provenance.

It emits no target program.

## Dependencies · `sec:compiler:dependencies`

Allowed first-party direct dependency:

```text
realization
```

A direct `architecture` dependency is added only if architecture-owned public
types cannot be used cleanly through realization without duplicating them.

Forbidden dependencies:

```text
model
target-elements
tapscript
simplicity
linker
transaction
vectors
release
artifacts
```

Concrete target matching is performed through a narrow abstract capability
interface or a downstream adapter without pulling target packages into compiler
core.

## Typed inputs · `sec:compiler:inputs`

The compiler consumes:

- validated `RealizationSpec`;
- explicit compilation scope;
- typed analysis policy;
- optional abstract target capabilities for target-plan pruning;
- typed deployment-independent bound references.

It does not consume final target bytes, transactions, reports, or deployment
profiles.

## Typed outputs · `sec:compiler:outputs`

The core analyzed value contains:

- architecture and realization bindings;
- explicit scope;
- normalized relation graph;
- source provenance;
- proof-alternative graph;
- disclosure analysis;
- fact-source requirements;
- constructibility result;
- lifecycle result;
- placement requirements;
- layout requirements;
- target capability requirements;
- coverage requirements.

A later target-selected plan may bind an abstract target capability identity
and deterministic proof choices.

> Illustrative boundary; names and exact fields are not frozen.

```rust
pub fn analyze(
	realization: &realization::RealizationSpec,
	policy: &AnalysisPolicy,
) -> Result<AnalyzedProgram, CompileError>;
```

## Forbidden inputs · `sec:compiler:forbidden`

The compiler must not consume:

- generated architecture or realization publications;
- generated declassification;
- model source, labels, or tests;
- plans or ADR Markdown;
- target reference prose;
- target bytecode or disassembly as semantic input;
- environment or filesystem state.

## Relation preservation · `rule:compiler:relations`

Every realization relation in scope must appear in compiler analysis.

The compiler may normalize or structurally share implementation nodes, but it
must preserve:

- source relation IDs;
- operation ownership;
- activation;
- provenance;
- coverage ownership.

A relation absent from the analyzed result is a compile failure.

## Proof planning · `rule:compiler:proofs`

The compiler separates:

1. semantic relation;
2. allowed proof alternatives;
3. selected target proof;
4. backend implementation pattern.

Compiler core retains alternatives.

Target planning removes unsupported alternatives but does not invent weaker
ones.

If no proof remains, planning fails.

## Disclosure analysis · `rule:compiler:disclosure`

Disclosure reasons are typed and separate:

- public-state dependency;
- public-observable dependency;
- permissionless constructibility;
- target-safety requirement;
- deployment-policy override.

Semantic disclosure derives from realization dependencies.

Target and deployment disclosure are added only at their planning layers.

A deployment claiming minimality must show that no supported lower-disclosure
plan satisfies the same relations, constructibility, lifecycle, and resources.

## Fact sources · `rule:compiler:fact-sources`

Every relation operand has at least one authenticatable source requirement.

Source classes may include:

- compile-time constant;
- architecture or deployment constant;
- authenticated transaction input/output;
- authenticated metadata;
- public chain data;
- public opening;
- current owner witness;
- operator witness;
- sponsor-local witness;
- derived expression.

Unauthenticated metadata is not a source.

## Constructibility · `rule:compiler:constructibility`

The compiler validates both:

```text
target can verify the witness
```

and:

```text
the authorized constructor can obtain the witness
```

Permissionless operations reject any plan requiring owner or operator secrets.

## Lifecycle · `rule:compiler:lifecycle`

For each supported object representation, compiler analysis records paths to
required exits.

A pilot may be valid within its scope while still carrying an unresolved
future lifecycle obligation.

Such a representation is not release-complete.

## Placement · `rule:compiler:placement`

The compiler classifies relations as:

- local to every family member;
- transaction-global;
- conditionally active;
- deliberately duplicated.

It produces candidate semantic carriers and required facts.

It does not assign concrete tapscript input indexes.

An unconditional relation with no possible carrier fails analysis.

## Layout requirements · `rule:compiler:layout`

Compiler core derives requirements such as:

- family count must be authenticated;
- repeated family is bounded;
- protocol output families are complete and disjoint;
- sponsor region is isolated;
- optional family presence follows one semantic condition;
- target ABI exposes required witnesses.

Concrete positions are backend output under D006.

## Coverage · `rule:compiler:coverage`

Every relation receives:

- positive case;
- negative case;
- activation requirements;
- expected mutation classes;
- carrier requirement;
- representation cases;
- semantic projection checks.

Compiler output defines required evidence. It does not mark evidence complete.

## Pilot analysis · `tbl:compiler:pilots`

| Pilot | Required analysis |
|---|---|
| compact ASH | Public constructibility, ownerless conservation, one output, no roots/events, sponsor isolation |
| live transfer | Every-owner authorization, live closure, conservation alternatives, explicit closed `U`, sponsor isolation, lifecycle obligations |

## Identity · `sec:compiler:identity`

Compiler-owned IDs cover normalized analysis nodes, proof plans, disclosures,
source requirements, placements, layouts, target requirements, and coverage
requirements.

They bind source realization IDs and policy-relevant typed parameters.

No stable public compiler hash is minted before the canonical analyzed
projection and configuration identity are reviewed.

## Assurance boundary · `sec:compiler:assurance`

The compiler establishes analysis completeness and deterministic planning.

It does not establish:

- backend pattern correctness;
- target semantics;
- linked constructor correctness;
- transaction ABI correctness;
- deployment release.

Those are separate reports under D004.

## Milestones · `tbl:compiler:milestones`

| Label | Deliverable |
|---|---|
| `milestone:compiler:crate` | Crate and schemas |
| `milestone:compiler:relations` | Relation DAG |
| `milestone:compiler:folding` | Checked constant folding |
| `milestone:compiler:proofs` | Proof alternatives |
| `milestone:compiler:disclosure` | Disclosure analysis |
| `milestone:compiler:sources` | Fact-source and constructibility analysis |
| `milestone:compiler:lifecycle` | Lifecycle graph |
| `milestone:compiler:placement` | Placement/layout requirements |
| `milestone:compiler:coverage` | Coverage requirements |
| `milestone:compiler:pilots` | Complete pilot analyses |

## Exit gate · `gate:compiler:exit`

Phase 2 exits when:

- relation census equals realization scope;
- all dependencies and types validate;
- each relation has proof, source, placement, layout, and coverage requirements;
- permissionless secret dependencies fail;
- lifecycle incompleteness is explicit;
- both pilots analyze deterministically;
- no target opcode or concrete target index enters compiler core;
- unsupported capability sets fail closed;
- workspace checks remain green and clean.

## Error vocabulary · `sec:compiler:errors`

See [`errors/compiler.md`](errors/compiler.md).

## Open questions · `sec:compiler:open`

- Where does generic target-capability matching live without a cycle?
- Does target planning return one selected proof plan or a canonical feasible set?
- How does one shared analysis node retain several source relation identities?
- How is pilot-valid but lifecycle-incomplete represented?
- How general may placement requirements become before target evidence exists?
