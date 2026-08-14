# Elements Tapscript Backend · `pkg:tapscript:contract`

> **Status:** Active — static capability and evidence-role adapter over the
> reviewed contract, typed instruction values, canonical opcode/push
> encoding, the supported-subset parser, and the abstract stack and
> failure-state validator implemented; backend proof patterns, protocol
> stack scheduling, and constructors not implemented and prototype-dependent
> **Phases:** [Phase 3](../phases/03-target-foundation.md) onward
> **Package:** `tripod-tapscript`
> **Library:** `tapscript`
> **Direct dependencies:** `compiler`, `target-elements`
> **Decisions:** [D003](../decisions/003-tapscript-first.md),
> [D004](../decisions/004-translation-validation.md),
> [D005](../decisions/005-value-representation.md),
> [D006](../decisions/006-transaction-abi.md)

## Purpose · `sec:tapscript:purpose`

`tapscript` lowers compiler-approved target plans into typed relocatable
Elements tapscript artifacts.

It owns:

- typed instruction construction;
- target proof patterns;
- stack validation;
- deterministic scheduling;
- verified local rewrites;
- target guard patterns;
- target-specific representation enforcement;
- arithmetic patterns;
- constructor patterns;
- concrete relation placement;
- concrete layout lowering;
- preliminary witness roles;
- symbols and relocations;
- target resource formulas;
- relation-carrier provenance.

It does not link final deployment values or build transactions.

## Dependencies · `sec:tapscript:dependencies`

Allowed direct dependencies:

```text
compiler
target-elements
```

Forbidden dependencies:

```text
model
simplicity
linker
transaction
vectors
release
artifacts
```

Use compiler-carried architecture/realization IDs rather than bypassing
analysis through new direct dependencies.

## Typed inputs · `sec:tapscript:inputs`

The backend consumes:

- validated analyzed program;
- compiler-approved target proof alternatives;
- exact typed Elements target;
- typed backend configuration;
- typed unresolved deployment parameters represented as relocations.

## Typed outputs · `sec:tapscript:outputs`

The relocatable bundle contains:

- target and compiler bindings;
- object-constructor templates;
- operation programs;
- typed instruction sequences;
- stack contracts;
- selected proofs and representations;
- concrete relation placements;
- concrete layout result;
- preliminary witness schemas;
- symbols and relocations;
- resource formulas;
- relation-carrier bindings.

The final linked bundle is linker-owned.

> Illustrative boundary; names and exact fields are not frozen.

```rust
pub fn emit(
	plan: &compiler::TargetCompilationPlan,
	target: &target_elements::ElementsTarget,
	configuration: &TapscriptConfiguration,
) -> Result<RelocatableTapscriptBundle, TapscriptError>;
```

## Forbidden behavior · `sec:tapscript:forbidden`

The backend must not:

- parse publications or planning prose;
- call model behavior to decide emission;
- invent or weaken semantic relations;
- add hidden authorization;
- remove lifecycle exits;
- accept confidential closed-asset identity;
- disclose facts without compiler provenance;
- leave a required relation unplaced;
- use target capabilities absent from the selected target;
- write final assets from the pure library API.

## Program model · `rule:tapscript:program`

Programs are constructed as typed instructions before byte encoding.

Instruction and pattern contracts include:

- typed stack before and after success;
- relevant failure stack behavior;
- altstack behavior;
- branch join compatibility;
- target execution domain;
- resource formula;
- relation provenance.

Raw arbitrary script bytes are not accepted as typed backend programs.

## Pattern library · `rule:tapscript:patterns`

Each reusable pattern states:

- semantic proof class;
- required target capabilities;
- input facts and sources;
- typed instruction fragment;
- stack contract;
- witness requirements;
- disclosure effect;
- resource formula;
- positive and negative vectors.

Missing pattern support is an emission error.

## Representation guards · `rule:tapscript:representation`

The backend enforces:

- explicit closed protocol asset identity;
- selected value representation;
- complete output-family closure;
- unknown-prefix rejection;
- public opening authentication where accepted;
- sponsor representation policy.

Confidential value conservation never replaces recipient, authorization, or
object closure.

## Placement and layout · `rule:tapscript:layout`

The backend converts compiler requirements into one deterministic target
layout.

It assigns:

- local checks to affected input programs;
- global checks to a coordinator or another complete target proof;
- family positions and ranges;
- sponsor region;
- optional family encoding;
- data-output order;
- target program roles.

Every selected relation has a reachable carrier.

## Stack scheduling · `rule:tapscript:stack`

The initial scheduler is deterministic and simple.

Optimization priorities are:

1. typed correctness;
2. auditability;
3. stable output;
4. resource use.

Peephole rewrites require typed preconditions, before/after stack equivalence,
deterministic order, and tests.

## Arithmetic · `rule:tapscript:arithmetic`

Narrow arithmetic uses exact target fixed-width operations with every success
flag checked.

Wide floor arithmetic remains prototype-gated by
[wide arithmetic research](../research/wide-arithmetic.md).

No production redemption, settlement floor, or cycle issuance pattern is used
before that decision lands.

## Constructors · `rule:tapscript:constructors`

Object constructors are metadata-parameterized target recipes.

STATE and other continuity-sensitive constructors remain gated by
[STATE constructor research](../research/state-constructor.md).

A production pattern must authenticate:

- predecessor constructor;
- metadata;
- static code identity;
- successor constructor;
- internal-key policy;
- target schema;
- absence of escape paths.

## Resources · `rule:tapscript:resources`

Every program and witness role has a symbolic resource formula covering:

- script bytes;
- witness bytes;
- stack/altstack;
- element size;
- target operation cost;
- crypto budget;
- target policy requirements.

Complete transaction feasibility is measured downstream.

## Pilot order · `tbl:tapscript:pilots`

| Operation | Purpose |
|---|---|
| compact ASH | First complete root-free permissionless operation |
| live transfer | Owner authorization and value-representation alternatives |
| maturity announcement | First STATE constructor integration |
| burn/clear | Public declassification and event lineage |
| redemption | Wide arithmetic and RESV terminal |
| later operations | Roadmap dependency order |

## Assurance boundary · `sec:tapscript:assurance`

The package establishes typed, deterministic, stack-valid emission and pattern
contracts.

It does not establish:

- target-node behavior;
- final linking;
- transaction ABI correctness;
- complete-transaction resource fit;
- deployment release.

## Exit gate · `gate:tapscript:foundation`

Backend foundation exits when:

- typed instruction encoding matches target;
- stack and branch contracts validate;
- representation guards fail closed;
- signature/timelock patterns bind exact target semantics;
- every pilot relation can be placed;
- symbols, relocations, witness roles, and resource formulas are typed;
- prototype-only arithmetic/constructor code cannot enter release output;
- target-native pattern tests pass;
- output is deterministic and the checkout remains clean.

## Error vocabulary · `sec:tapscript:errors`

See [`errors/tapscript.md`](errors/tapscript.md).

## Open questions · `sec:tapscript:open`

- Which package owns a future backend-neutral relocatable interface?
- Where does resource-sensitive final proof selection occur?
- Is a local typed interpreter needed in addition to target-native execution?
- What leaf-weight policy is handed to the linker?
- Which constructor and arithmetic prototypes are accepted?
