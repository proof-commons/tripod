# Elements Tapscript Backend · `pkg:tapscript:contract`

> **Status:** Active — reviewed-target adaptation, typed instruction encoding and validation, compact-ASH proof patterns and relocatable bundle, and the candidate STATE constructor with its records and programs are delivered
> **Phases:** [Phase 3](../phases/03-target-foundation.md) onward
> **Package:** `tripod-tapscript`
> **Library:** `tapscript`
> **Direct dependencies:** `architecture`, `compiler`, `realization`, `sha2`, `target-elements`, `thiserror`
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
architecture
compiler
realization
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

Use compiler-carried architecture/realization IDs rather than bypassing analysis. The keyed bundle names the architecture-owned binding and semantic identity types exposed in the compiler's public plan; that narrow vocabulary dependency does not authorize manifest derivation, validation or a second planner (`rec:w1b13:bundle-surface`).

The candidate STATE constructor commits the canonical STATE metadata bytes that only realization's codec produces and no compiler projection carries, so the constructor consumes `EncodedStateMetadata` directly, while the assessment path keeps consuming compiler-carried architecture and realization identifiers and the reach-around prohibition above stands for it (ruling 22 at (`rule:phase6:wave4-rulings`)). The constructor also takes the workspace's `sha2` for the target's tagged hashes, as ruling 22 states at (`rule:phase6:wave4-rulings`), and `thiserror` to derive the closed refusal sum's error form as the realization does.

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

The additive compact comparison entries preserve emission and acceptance semantics. `validate_compact_secret_free_path` checks the existing authorization-primitive census on an offered typed protocol leaf; execution validity and sponsor authorization outside that leaf are separate claims. `validate_compact_emitted_representation` binds the input-zero recognition fragment to the retained candidate and distinguishes removal of its explicit-value prefix authentication from other structural changes; it does not validate every recognition site. `validate_compact_emitted_lifecycle` compares offered keyed programs and outstanding declarations with the retained candidate, distinguishing one omitted coordinator path from one omitted Clear declaration. The lifecycle comparison preserves the prohibition on removing lifecycle exits: no emitted Clear path exists in this candidate, no emitted-path claim is made, and Clear's realization landing in Wave 2 reopens the declaration disposition under ruling 138 (`rule:phase7:wave0-rulings`). These entries return new typed refusals without extending existing refusal enums or exposing malformed candidate construction.

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

Compact and live sponsor-change closure retains the reserve-asset and declared-role checks; the sponsor's all-inputs-all-outputs signature fixes its chosen destination, so no covenant destination equality is needed (`[ADR024-rem:closure:consequences]`).

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

The wide floor prototype is accepted by
[wide arithmetic research](../research/wide-arithmetic.md): derived limbs at
base `2^26` over the reviewed Euclidean division.

That acceptance is a prototype decision, not a production pattern. No
production redemption, settlement floor, or cycle issuance pattern is used
until the construction is promoted by a separate reviewed act, and settlement
in particular is unclaimed until the batch-size-2 measurement is taken.

## Constructors · `rule:tapscript:constructors`

Object constructors are metadata-parameterized target recipes.

The continuity prototype is accepted by
[STATE constructor research](../research/state-constructor.md): a dynamic
metadata leaf beside a static code subtree, with derived successor metadata.
Its schema carries a synthetic counter, so STATE's own metadata remains
unimplemented and continuity-sensitive production constructors stay gated on
promoting the construction.

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

## Pilot order · `tab:tapscript:pilots`

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
- On what evidence is an accepted prototype promoted to a production pattern?

## Keyed adapter increment · `rec:tapscript:keyed-bundle`

The default `tapscript::keyed_bundle` adapter implements the first step of (`step:w1b13:adapter`): explicit candidate selection from an immutable compiler plan, no search, retained relation/coverage and evidence rows, and byte-preserving lowering to the existing two fragments. SOURCE's canonical successor leaf is construction material, never a runtime carrier. The public inventory is recorded at (`rec:w1b13:bundle-surface`). Fixed-grammar stack/resource evidence now lives at (`rec:w1b13:resources`); recipe closure now lives at (`rec:w1b13:constructor-surface`); concrete member coverage and target-form admission remain subsequent adopted steps; this increment alone does not meet (`gate:wave1:bite-1-3-packet`).


| Keyed resource module | Contract |
|---|---|
| `src/keyed_resources.rs` | Always-on typed usage/obligation projection and checked formulas under (`q:w1b13:handoff`), inventoried at (`rec:w1b13:resources`). The 88-byte script stack funds validation budget; the 91-byte input witness enters weight. Scale comes from target; base flag and count framing stay in base bytes. Primitive counts are instruction-derived, width bounds encoding-checked, and vectors cross-check serialization without a tapscript-to-transaction edge. Deployment and package admission remain outstanding. |

## Keyed construction policy · `rule:tapscript:keyed-construction`

Under (`rec:w1b13:adoption`), tapscript owns the default public schema-1 `KeyedLeafRecipe`, immutable `KeyedConstructionMaterial` and supplied `KeyedCurveCapability`; the exact inventory and recipe are at (`rec:w1b13:constructor-surface`). The recipe has one public parameter, an arbitrary 32-byte semantic key. Material retains the reviewed version and exact sole leaf. It admits neither a transaction commitment nor creator authority. No curve or second hashing implementation is added; commitment/materialization belongs to bite 1.4. Vectors alone consumes tapscript material and existing transaction/conformance commitment helpers to authenticate a canonical descriptor against a supplied predecessor program. That descriptor's JSON stays an experimental conformance carrier, not a production dependency or stable interchange schema.

The complete path census is one script path containing both clauses plus the unavoidable key path. NUMS closure remains conditional on discrete-log hardness and SHA256 preimage resistance; generated-key separation uses second-preimage resistance and adversarial-pair separation collision resistance. Reconstruction and finite different-key controls do not prove these assumptions. Constructor-policy and commitment refusals are distinct from covenant execution refusals. Member binding, full transaction limits and final-form execution remain later boundaries.

The keyed production path authenticates no predecessor and has no production curve-capability source: step 4 names predecessor authentication as an explicit offline member-context input from the vectors oracle, while production authentication and the source of curve arithmetic belong to bite 1.4 (`rec:w1b13:constructor-surface`).

The path census is declared and supported by sole-leaf reconstruction, root/path/control checks and byte pins; excluding alternative leaf or branch preimages additionally depends on tagged-SHA256 second-preimage resistance across leaf/branch domains. Separating adversarial output programs depends on collision resistance of the composed x-only output map, including opposite output points, not only tagged-hash collision resistance (`rec:w1b13:constructor-surface`).
