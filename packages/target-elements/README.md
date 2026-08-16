# `tripod-target-elements`

`target_elements` states the typed, reviewed Elements tapscript target
compatibility contract. It states target facts only, and nothing about the attestation contract
.

The package contract is
[plans/packages/target-elements.md](../../plans/packages/target-elements.md).

This README and `src/lib.rs` together are meant to be enough to use the public
API correctly. The crate documentation is the compressed orientation; the worked
examples and the full tour are here.

The library crate is named `target_elements`; the Cargo package is
`tripod-target-elements`.

## Dependencies

None — neither first-party nor third-party. Every declaration is a
standard-library type. The crate serializes nothing, hashes nothing, parses
nothing, and opens no file.

### How the neighbors relate

Nothing here depends on anything, and that is the point: a package stating
target facts must not be able to reach a protocol meaning. The relationships run
one way, inward.

- **`tapscript`** depends on this crate and on `compiler`. It is the join that
  maps compiler-owned abstract requirements onto the primitive contracts stated
  here. It reads target facts out of this crate rather than restating any of
  them — every opcode byte, push form, encoding width, and resource bound it
  emits is resolved from a `ReviewedElementsTapscriptDefinition`.
- **`target-elements-conformance`** depends on this crate for every reviewed
  fact its fixtures are stated against, and on `tapscript` for the exact script
  bytes. It owns no target semantics; it owns the mechanics of asking an
  external executor what a node did.
- Nothing in this crate knows either of them exists.

## Boundary

The package will own the tapscript execution domain and leaf version, reviewed
opcode identities with their stack and failure contracts, field-specific
encodings, sighash and relative-timelock dimensions, confidential-value and
issuance capability descriptions, consensus and policy resource interfaces, and
the registry of target evidence a future deployment must produce.

It owns no attestation-contract operation, object, relation, proof plan,
authorization policy, batch bound, or transaction layout. It does not know which
assets are protocol closed assets, it does not select a sighash profile, and it
does not choose a cadence band or a batch bound. Those are downstream decisions.

## Review provenance is not target identity

The typed facts here are transcribed from a reviewed reading of upstream
Elements interpreter source. The upstream repository, the revision consulted,
the source paths, the node version, and the review date are recorded in
[plans/reference/elements-tapscript.md](../../plans/reference/elements-tapscript.md)
as review provenance. They never enter this crate's types or its stable
projections: the contract identifies a typed compatibility surface, not one
implementation revision.

No package parses that reference. It is review support for a human reader; the
typed Rust source here is the authority.

## Quickstart

Obtain the reviewed contract, validate a binding against it, then inspect a
capability. The same code runs as a doctest in `src/lib.rs`.

```rust
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding,
    ElementsCapability, LeafVersion, StaticCapabilityStatus, TargetContractVersion, TargetError,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The reviewed contract has no public constructor. This is the one
    // way to obtain it, and it is a V2 contract.
    let target = reviewed_elements_tapscript().expect("the reviewed contract validates");
    assert_eq!(target.definition().version(), TargetContractVersion::V2);

    // A binding names one network a caller intends to test against.
    let binding = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V2,
        DeploymentEnvironment::Development,
        [0x11; 32],
        [0x22; 32],
        ActivationDeclaration::new(
            true,
            LeafVersion::TAPSCRIPT,
            [ElementsCapability::TapscriptExecution],
        ),
        None,
    );
    let bound = validate_reviewed_development_binding(&target, binding)?;

    // The binding is welded to the whole projection of the contract
    // that validated it, not to its revision number.
    assert!(bound.welded_to(&target));

    // Production is nameable precisely so that validation can refuse
    // it. No function here returns a validated production binding.
    let production = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V2,
        DeploymentEnvironment::Production,
        [0x11; 32],
        [0x22; 32],
        ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
        None,
    );
    assert!(matches!(
        validate_reviewed_development_binding(&target, production),
        Err(TargetError::ProductionBindingUnsupported),
    ));

    // Support is not a boolean. A validated contract states a contract
    // for every capability in the census, so the lookup is total.
    let capabilities = target.definition().capabilities();
    let status = |capability| {
        capabilities
            .get(&capability)
            .expect("a validated contract states every capability")
            .status()
    };

    // Reviewed means the typed contract was checked against upstream
    // source. It does not mean a node was ever asked.
    assert_eq!(
        status(ElementsCapability::TapscriptExecution),
        StaticCapabilityStatus::Reviewed,
    );
    // The review reached the signature primitives but not the sighash
    // construction, and that is recorded rather than guessed.
    assert_eq!(
        status(ElementsCapability::OutputCommittingSighash),
        StaticCapabilityStatus::Incomplete,
    );
    // No reviewed primitive orders two byte strings at all.
    assert_eq!(
        status(ElementsCapability::CanonicalByteOrdering),
        StaticCapabilityStatus::Unsupported,
    );
    Ok(())
}
```

## Two trust states, neither of which is a hash

Validation happens in two steps, and the distinction is the crate's central
safety property.

```text
TargetDefinition                    an offered contract; no guarantee
  |  validate_target_definition   -> Result<_, Vec<TargetError>>
ValidatedTargetDefinition           internally coherent; still anyone's
  |  validate_as_reviewed_elements -> Result<_, TargetError>
ReviewedElementsTapscriptDefinition typed-equal to this crate's derivation
```

`reviewed_elements_tapscript()` runs both steps over the first-party
declaration, and it is the only public path to the reviewed state. Neither
`ValidatedTargetDefinition` nor `ReviewedElementsTapscriptDefinition` has a
public constructor, and the unvalidated first-party declaration is private, so
a caller cannot take the reviewed contract, mutate one field, and revalidate.

`ValidatedTargetDefinition` proves internal coherence and nothing more. A
caller can assemble a perfectly coherent contract that permutes an opcode byte,
rewrites a failure behavior, restates a resource limit, or downgrades a
capability status — `validate_target_definition` accepts all four, and
`validate_as_reviewed_elements` refuses all four with
`TargetError::ReviewedDefinitionMismatch`.

That refusal is typed equality against this crate's own derivation, not a
digest comparison. See *Identity* below.

## Public-API tour

Fourteen public modules. Every type and function named below is re-exported at
the crate root, with three exceptions that must be reached by module path:
`definition::encoding_dependencies`, `opcode::VALIDATION_BUDGET_PER_CHECK`, and
`opcode::MAX_STACK_ELEMENT_BYTES`.

### Entry points

The complete list of public free functions:

```text
reviewed_elements_tapscript() -> Result<ReviewedElementsTapscriptDefinition, Vec<TargetError>>
validate_target_definition(definition: TargetDefinition)
    -> Result<ValidatedTargetDefinition, Vec<TargetError>>
validate_as_reviewed_elements(offered: ValidatedTargetDefinition)
    -> Result<ReviewedElementsTapscriptDefinition, TargetError>

validate_development_binding(definition: &ValidatedTargetDefinition,
                             binding: DevelopmentDeploymentBinding)
    -> Result<ValidatedDevelopmentBinding, TargetError>
validate_reviewed_development_binding(target: &ReviewedElementsTapscriptDefinition,
                                      binding: DevelopmentDeploymentBinding)
    -> Result<ReviewedDevelopmentBinding, TargetError>
bind_development_target(definition: ValidatedTargetDefinition,
                        deployment: ValidatedDevelopmentBinding)
    -> Result<ElementsTarget, TargetError>
overridable_dimensions(definition: &ValidatedTargetDefinition) -> BTreeSet<ResourceDimension>

prerequisite_cycle_residual(contracts: &BTreeMap<ElementsCapability, CapabilityContract>)
    -> Vec<ElementsCapability>
transitive_prerequisites(contracts: &BTreeMap<ElementsCapability, CapabilityContract>,
                         capability: ElementsCapability) -> BTreeSet<ElementsCapability>
status_closure_violations(contracts: &BTreeMap<ElementsCapability, CapabilityContract>)
    -> Vec<(ElementsCapability, ElementsCapability)>

definition::encoding_dependencies(definition: &TargetDefinition) -> BTreeSet<EncodingClass>
```

Two ownership notes that decide how an example is written:
`ReviewedElementsTapscriptDefinition::into_validated` **consumes** the reviewed
value, so use `validated()` for a borrow or use
`validate_reviewed_development_binding`, which takes the reviewed value by
reference. `bind_development_target` takes the definition **by value**, so it
must come after any borrow-based reads.

### `definition` — the contract and its trust states

`TargetDefinition::new` takes a `TargetDefinitionParts` struct rather than
twelve positional arguments, so a transposition is loud. Its twelve public
fields are `version`, `execution_domain`, `leaf_version`, `opcodes`,
`encodings`, `pushes`, `authorization`, `confidential_values`, `issuance`,
`resources`, `capabilities`, and `evidence_requirements`. `TargetDefinition`
exposes one `const fn` accessor per field.

`TargetContractVersion` is an opaque revision key, not a digest:

```text
TargetContractVersion::V1          the historical Guide-9 contract
TargetContractVersion::V2          the compound-proof contract this crate derives
TargetContractVersion::SUPPORTED   the two-element census
TargetContractVersion::supported(value: u32) -> Result<Self, TargetError>
TargetContractVersion::get(self) -> u32
```

A caller selects one by naming the constant, or admits a number at run time
with `supported`. V1 remains implemented and is not widened; V2 carries the
compound-proof primitive census together with the widened operand and success
algebra.

`TargetProjection` is the stable comparison form, reached by `projection()` on
either trust state. It carries no provenance and no digest, and its accessors
mirror `TargetDefinition`'s except that the four registries become **slices**
rather than maps — so a projection is compared in canonical order and cannot
name a member twice.

### `deployment` — the development binding

```text
DevelopmentDeploymentBinding::new(target_version, environment, network_id: [u8; 32],
                                  genesis_id: [u8; 32], activation, resource_overrides) -> Self
ActivationDeclaration::new(tapscript_expected_active: bool,
                           required_leaf_version: LeafVersion,
                           required_capabilities: impl IntoIterator<Item = ElementsCapability>) -> Self
DevelopmentResourceOverrides::new(policy: PolicyResourceLimits) -> Self
```

`ValidatedDevelopmentBinding` and `ReviewedDevelopmentBinding` have no public
constructors. The reviewed form retains the complete `TargetProjection` of the
contract that validated it, and `welded_to` compares that projection — version
equality is deliberately *not* the weld, because a version number would let a
binding survive a change to the contract it was checked against.

`ElementsTarget` is a validated contract paired with a validated binding, and
carries only the two accessors that read them back.

### `opcode` — primitives and their contracts

`OpcodeId` is a `#[non_exhaustive]` vocabulary of **55** primitive identities,
enumerated in the enum body and censused in `OpcodeId::ALL`
(`src/opcode.rs`). Representative: `Sha256Initialize`, `Add64`, `CheckSig`,
`Duplicate`, `Concatenate`.

`OpcodeSpec` is one primitive's complete contract — `id`, `code` (the target
byte), `domains`, `stack`, `resources`, `evidence`. `StackContract` carries
`operands` **ordered deepest first**, matching push order, plus the success and
failure contracts. `LeafVersion::TAPSCRIPT` is `0xc4`, the Elements value, not
Bitcoin's `0xc0`; `LeafVersion::new` refuses anything unreviewed.

`FailureCause` is a vocabulary of 24 causes and `FailureOutcome` of 3; a
`FailureEffect` pairs them, and a `FailureContract` is the set. See *Failure
behavior is part of every primitive contract* below for why the shapes cannot
be collapsed.

### `success` and `operand` — the stack algebra

`SuccessContract` has four variants: `Fixed`, `RetainsOperands`,
`Alternatives`, and `OperandResolved`. `RetainsOperands` is deliberately
distinct from `Fixed { consumed_operands: 0, .. }`.

The traversal to prefer is `SuccessContract::cases() -> Vec<SuccessCase>`: it
normalizes every shape to a case list, reporting `Fixed` and `RetainsOperands`
each as one case under `SuccessCondition::Always`, so a consumer walks every
primitive uniformly instead of matching the enum at each call site.
`result_types()` and `defect(declared_operands)` complete the surface.

`OperandContract` has six variants: `Exact`, `AnyItem`, `OneOf`, `WidthOnly`,
`Signature`, and `PublicKey`. `AnyItem` and `WidthOnly` are the V2 additions —
a position that constrains nothing, and a position admitted on width alone.

### `capability` — the capability registry

`ElementsCapability` is a `#[non_exhaustive]` vocabulary of **45** capabilities,
censused in `ElementsCapability::ALL` (`src/capability.rs`).
`CapabilityContract` carries the capability, its prerequisites, the opcodes and
encodings it names, its evidence requirements, and its `StaticCapabilityStatus`.

**A trap worth stating plainly.** `StaticCapabilityStatus` derives `Ord` in
declaration order, which is `Reviewed < Incomplete < Unsupported` — the
*inverse* of semantic strength. Compare with `strength()` (`Unsupported` 0,
`Incomplete` 1, `Reviewed` 2) or `at_most()`, never with `<`.

The three graph predicates operate on a capability map:
`prerequisite_cycle_residual` returns everything on or downstream of a cycle
(empty when acyclic); `transitive_prerequisites` is cycle-safe and **excludes**
the capability itself; `status_closure_violations` pairs each offending
capability with the single weakest responsible transitive prerequisite.

### `encoding` and `push`

`EncodingClass` is a `#[non_exhaustive]` vocabulary of **28** classes, censused
in `EncodingClass::ALL`. `EncodingSpec` states the class, domain, prefix bytes,
payload width, byte order, canonicality rule, and evidence. `EncodingClass:: v1_shape()` gives the `V1EncodingShape` a V1 consumer would read, which is how
the two revisions coexist without a second registry.

`PushContract` is effectively a small decoder, and it is the surface `tapscript`
builds its serializer and parser on:

```text
form_for_opcode(&self, opcode: u8) -> Option<&PushFormSpec>
minimal_form(&self, payload: &[u8]) -> Result<PushForm, PushDefect>
form(&self, form: PushForm) -> Option<&PushFormSpec>
maximum_payload_bytes(&self) -> usize
enforcement(&self) -> &BTreeMap<PushDefect, PushEnforcement>
occupied_opcodes(&self) -> BTreeSet<u8>
defects(&self) -> Vec<PushContractDefect>
```

`minimal_form` is the one method in the crate returning a `Result` whose error
is not a `TargetError`; it yields a `PushDefect`. `PushEnforcement` records
which of the three defects (`Truncated`, `Oversized`, `NonMinimal`) is refused
at consensus and which only under the relay rules a node applies to what it
forwards.

`PushFormSpec` answers the decoding questions directly: `occupies`,
`admits_width`, `width_prefix_bytes`, `width_prefix_order`, `opcode_for`,
`payload_for`, and `width_in_opcode`.

### `authorization`, `confidential`, `resource`

`SighashCapability` partitions `SighashDimension` (10 members) into reviewed and
unreviewed sets; `contradictory()` and `unclassified()` are the two defect
predicates, and a dimension in neither set is itself a defect.

`SignaturePrimitiveContract::new` takes `empty_signature` immediately before
`invalid_signature` — two adjacent parameters of the same type, and a
transposition hazard worth checking at every call site.

`ConfidentialCapabilityState` (`PrimitiveReviewed`, `ExternalConsensusClaim`,
`Unsupported`) is a different vocabulary from `StaticCapabilityStatus` and must
not be conflated with it.

`ResourceContract` keeps consensus and policy separate and offers three
`Option`-returning validators — `missing_consensus_dimension`, `zero_bound`, and
`policy_looser_than_consensus` — which feed the three corresponding
`TargetError` variants. An absent policy bound is not looser than consensus, so
`PolicyResourceLimits::new([])` validates.

### `evidence` and `evidence_registry`

`TargetEvidenceRequirementId` is a vocabulary of **24** identities, censused in
its `ALL`. It carries identities only, and no completion state — see *What this
package owns, and what it does not*.

`TargetEvidenceRequirement` describes one requirement: its `subject`, the
`claim` class it would settle, the `environment` that must produce it, and the
`stale_on` conditions. An empty stale-condition set is a defect.

### Registry completeness

A validated contract always states the complete census for all four registries,
which is why keyed lookups on a validated contract are total:

| Registry | Members |
|---|---|
| `opcodes()` | 55 |
| `encodings()` | 28 |
| `capabilities()` | 45 |
| `evidence_requirements()` | 24 |

## Error handling

`TargetError` is the crate's single error root. It implements `Display` and
`core::error::Error` by hand — there is no derive crate, because there is no
dependency — and it has **68** variants, declared in `src/error.rs`. Match it
with a `_` arm; it is the error root and several vocabulary types it names are
`#[non_exhaustive]`.

Two shapes of fallibility, and the difference is deliberate:

| Operation | Error shape | Why |
|---|---|---|
| `validate_target_definition`, `reviewed_elements_tapscript` | `Vec<TargetError>` | reports **every** defect, not the first |
| everything else | `TargetError` | exactly one thing can be wrong |

Which operation returns what:

| Operation | Variants |
|---|---|
| `TargetContractVersion::supported` | `UnsupportedTargetContractVersion` |
| `LeafVersion::new` | `UnreviewedLeafVersion` |
| `validate_as_reviewed_elements` | `ReviewedDefinitionMismatch`, and only that |
| `bind_development_target` | `TargetDeploymentVersionMismatch` |
| `PushContract::minimal_form` | a `PushDefect`, not a `TargetError` |

`validate_development_binding` and `validate_reviewed_development_binding`
return exactly one of nine variants, checked in this order:
`ProductionBindingUnsupported`, `ZeroNetworkId`, `ZeroGenesisId`,
`TargetDeploymentVersionMismatch`, `ActivationLeafVersionMismatch`,
`InconsistentActivationDeclaration`, `UnsupportedRequiredCapability`,
`UnknownOverrideDimension`, `IncompatibleResourceOverride`. The production
check is first and unconditional.

`validate_target_definition` accumulates across nine groups — the opcode
registry, encodings, pushes, authorization, confidential and issuance,
resources, capabilities, evidence, and the cross-contract welds. The seven
`*ContractMismatch` variants come from the private `weld` module and catch a
contract that is internally coherent but disagrees with itself across two of
its own registries.

## The compound-proof primitive census

Guide 10 required a primitive-needs census before any prototype code: every
need a compound proof has, decided against what the reviewed target actually
offers, with no row left assumed. This is that table. "Constructor" is the
dynamic-metadata-leaf candidate; "wide floor" is the derived-limb candidate.

| Need | Constructor | Wide floor | Before V2 | Decision |
|---|---|---|---|---|
| duplicate top item | yes | yes | absent | admitted, `OP_DUP` 0x76 |
| duplicate top pair | yes | yes | absent | admitted, `OP_2DUP` 0x6e |
| copy second item | yes | yes | absent | admitted, `OP_OVER` 0x78 |
| swap two items | yes | yes | absent | admitted, `OP_SWAP` 0x7c |
| rotate short frame | yes | yes | absent | admitted, `OP_ROT` 0x7b |
| remove second item | yes | yes | absent | admitted, `OP_NIP` 0x77 |
| copy top below second | yes | yes | absent | admitted, `OP_TUCK` 0x7d |
| remove item | yes | yes | absent | admitted, `OP_DROP` 0x75 |
| remove two items | yes | yes | absent | admitted, `OP_2DROP` 0x6d |
| equality | yes | yes | absent | admitted, `OP_EQUAL` 0x87 |
| equality-and-abort | yes | yes | absent | admitted, `OP_EQUALVERIFY` 0x88 |
| verify Boolean | yes | yes | absent | admitted, `OP_VERIFY` 0x69 |
| byte concatenation | yes | no | absent | admitted, `OP_CAT` 0x7e |
| byte split/slice | yes | no | absent | admitted, `OP_SUBSTR` 0x7f |
| byte width | yes | no | absent | admitted, `OP_SIZE` 0x82 |
| bitwise selection | yes | no | absent | admitted, `OP_AND` 0x84 and `OP_XOR` 0x86 |
| byte lexicographic order | yes | no | absent | **unavailable as a primitive**; see below |
| conditional branch | no | no | absent | not admitted; see below |
| alternate stack | no | no | absent | not admitted; see below |
| indexed copy | no | no | absent | not admitted; see below |
| read below the third item | yes | no | absent | **decided against**: no primitive offers it; see below |
| streaming hash | yes | no | reviewed | unchanged |
| signed fixed-width arithmetic | yes | yes | reviewed | unchanged |
| signed comparison | no | yes | reviewed | unchanged |
| script-number conversion | yes | yes | reviewed | unchanged |
| input/output program inspection | yes | no | reviewed | unchanged |
| tweak verification | yes | no | reviewed | operand corrected in V2; see below |

Concatenation, slicing, and the bitwise operations are admitted because Elements
re-enables them: the disable list that carries them upstream has them commented
out, so they execute in tapscript rather than being refused. That is a target
fact and not an assumption from another script language, which is the whole
reason the census exists.

### Byte-lexicographic ordering is not a primitive

No reviewed primitive orders two byte strings. The fixed-width comparisons take
eight-byte signed integers and refuse anything else, and the script-number
ordering reads a number; neither orders a thirty-two byte digest. The capability
is named and carries `Unsupported` rather than being left unmentioned, so a
construction that needs canonical ordering has to confront the status.

It does not follow that the constructor candidate is rejected. Ordering is
constructible from primitives that do exist: a four-byte chunk read unsigned and
compared, applied per chunk, with the per-chunk results combined arithmetically
so the first differing chunk decides. Having the capability and being able to
build it are different claims, and only the second one holds.

### Three needs are decided as not admitted

Conditional branching is not admitted. No schedule step requires it — the
selection a canonical ordering needs is arithmetic on values in `{0,1}` and
branches nowhere — and the abstract stack validator is a linear fold with no
control stack, so admitting a branch would put a primitive in the registry whose
contract nothing could check.

The alternate stack is not admitted. The success algebra states main-stack
effects only, and no scheduled step needs the scratch space, so admitting a
mover would mean extending the algebra for a convenience.

Indexed copying is not admitted. Its operand count is chosen at run time, and a
contract whose operand list is fixed cannot state that. The schedules close
without it.

### The reach bound is three items, and every schedule is shaped by it

The three refusals above have one consequence between them, and it governs
every compound proof this crate can express. With no indexed copy, no indexed
move, and no alternate-stack transfer, the deepest item any reviewed primitive
can read is the third: no primitive in the registry declares a fourth operand,
and no successful form consumes more than three. So a program may hold at most
two computed values and still reach the next witness beneath them.

This is a property of the reviewed contracts rather than a convention, and it is
machine-checked over the whole registry rather than asserted about the
primitives somebody happened to look at. It decides whether a compound proof can
be scheduled at all: the constructor continuity proof has to keep one static
subtree root alive across an entire second constructor derivation, and it fits
only because the root is the deepest of the three values the first half retains,
every later witness lies beneath it in consumption order, and the second half
consumes it last.

The bound also refuses things. The metadata transition proof and the continuity
proof each schedule on their own, and composing the two into one program is not
currently expressible: the transition needs both metadata objects adjacent, the
continuity proof needs the static root between them, and no reviewed primitive
reaches past the third item to reorder them. That is an open finding rather than
a settled decision, and it is recorded as one.

### The tweak operand is a width, not an encoding

The tweak position of `OP_TWEAKVERIFY` was declared in the V2 registry as one
exact encoding class. That was wrong about the target in the refusing direction.
Upstream checks `vchTweak.size() != 32` and nothing else, and decides what the
thirty-two bytes mean afterwards inside `CheckPayToContract`; a streaming-hash
digest, which is exactly what a constructor program derives there, was refused
by a rule the target does not have.

The operand algebra now carries a position admitted on width alone, naming the
class the target reads an item as without making that class an admission
condition. A wrong width is still refused, and an item whose width the abstract
state has not settled still satisfies nothing. What the correction does not do
is promise the derived tweak is a valid scalar: the rare instance that is not
fails inside the curve arithmetic, which is where the target fails it, and that
residual is unchanged.

## Review provenance for the compound-proof primitives

Every primitive above was read in the upstream interpreter one at a time, in the
tapscript execution path. The repository, revision, source paths, and review
date belong in
[plans/reference/elements-tapscript.md](../../plans/reference/elements-tapscript.md)
with the rest of the review provenance, which now records the revision-2 census,
the width-only tweak correction and its interpreter lines, and the unsupported
ordering capability.

## Identity

The crate mints no digest. There is no target-definition hash, no
deployment-instance hash, and no field reserved for one. Direct typed comparison
of validated values is the whole comparison mechanism, and the stable contract
version carries the one compatibility decision a consumer actually makes.

## State

Implemented:

- the crate boundary and its no-dependency rule;
- the typed error root;
- the target-contract version and its supported census, now two revisions:
  V1 remains the historical Guide-9 contract and V2 carries the compound-proof
  primitive census together with the widened operand and success algebra;
- the tapscript execution domain and the validated leaf version;
- the reviewed primitive registry, with complete operand, result, failure,
  and resource contracts for every admitted primitive, including the
  compound-proof substrate: the ordinary stack operations, byte equality and its
  verifying form, Boolean verification, concatenation, width, slicing, and the
  bitwise combinators;
- an operand position that constrains nothing and a successful form that carries
  a declared operand through by index, which is what lets a polymorphic stack
  operation be described without inventing types for the caller's items;
- the field-specific encoding registry, with asset and value as independent
  axes and no global byte order;
- the literal-push contract: every push form with its opcode span and width
  field, the ordered minimal-form rule, the maximum literal size, and which
  of those rules is consensus and which is relay policy;
- signature, sighash, and relative-timelock dimensions;
- confidential-value and issuance capability descriptions;
- separate consensus and policy resource interfaces;
- the capability registry with an acyclic prerequisite relation;
- the evidence-requirement registry;
- the target validator, which reports every defect rather than the first;
- the stable semantic projection;
- the development deployment binding, its validation, and its combination
  with the contract.

## A deployment instance is not the target contract

The contract describes a compatibility surface; a binding names one network the
project intends to exercise it against. A binding never mutates the contract, so
the contract does not change when the network does.

`DeploymentEnvironment::Production` is nameable so that validation can refuse
it. No function in this crate returns a validated production binding, and a
development binding cannot be upgraded into one. An `ActivationDeclaration` is
typed input stating what a caller intends to test against — not a report, and
not an observation.

The binding carries no endpoint, username, password, cookie path, bearer token,
key, or wallet path, and none may be added. A future runner that must talk to a
node needs its own security design.

Binding a contract to a deployment proves only that the static contract is
internally valid, that the declaration is internally valid, and that the two
agree. It does not prove that the execution domain is active anywhere, that any
node behaves as described, that the network exists, or that anything is ready to
deploy.

## Support is not a boolean

A capability is `Reviewed`, `Incomplete`, or `Unsupported`, and none of the
three means deployment-evidenced. `Reviewed` means the typed static contract was
checked against upstream source; it does not mean a node was ever asked.

Four capabilities are deliberately not `Reviewed`. The sighash dimensions are
`Incomplete` because the review reached the signature primitives but not the
sighash construction, and every sighash dimension is recorded as unreviewed
rather than guessed. Whole-transaction value conservation is `Incomplete`
because it is a claim about the target's own consensus rules that no script
primitive demonstrates. Authenticated value opening is `Unsupported`, and it
must stay that way until a complete tested pattern exists: the low-level curve
and hash primitives being present is not an opening proof. Canonical byte
ordering is `Unsupported` because no reviewed primitive performs it at all, as
the census above records.

## Failure behavior is part of every primitive contract

A primitive described only by what it does when it succeeds is an incomplete
contract, because the reviewed target does not fail uniformly. Some primitives
abort evaluation; the signature primitives consume their operands and push a
false when the offered signature is empty; and the fixed-width arithmetic
primitives leave their operands in place and push a false *above* them on
overflow, so the failing path leaves a deeper stack than the succeeding one. All
three shapes are typed separately and none may be collapsed into the others.

## What this package owns, and what it does not

This crate carries target evidence *requirements* and no mutable
evidence-completion status for them. A static contract that recorded whether a
run had happened would change every time one did.

Evidence is produced and recorded separately by
`tripod-target-elements-conformance`, which runs a caller-selected
external executor against the reviewed primitive fixtures; development native
evidence exists there. Production target evidence remains absent, and
production target support is not claimed.

## What this package deliberately does not do

Each of these is a design decision, not a gap awaiting an implementation.

- **It executes nothing.** No function here runs a script, evaluates a stack, or
  talks to a node. The contract states what the reviewed target does; it does
  not do it.
- **It mints no identity.** There is no target-definition hash, no
  deployment-instance hash, and no field reserved for one. Comparison is typed
  equality on `TargetProjection` and `DeploymentProjection`.
- **It carries no evidence-completion status.** The evidence registry holds
  requirements. A static contract that recorded whether a run had happened would
  change every time one did.
- **It returns no validated production binding.** The production
  environment variant is nameable so that validation can refuse it, and a
  development binding cannot be upgraded into one.
- **It holds no secret or endpoint.** A binding carries no endpoint, username,
  password, cookie path, bearer token, key, or wallet path, and none may be
  added. A future runner that must talk to a node needs its own security design.
- **It serializes, hashes, parses nothing, and opens no file.** The
  no-dependency rule is what enforces this: there is nothing available to do it
  with.
- **It names no attestation-contract concept.** No operation, object, relation,
  proof plan, authorization policy, batch bound, or transaction layout. It does
  not know which assets are protocol closed assets, it does not select a sighash
  profile, and it does not choose a cadence band or a batch bound. Those are
  downstream decisions.
- **It does not read its own review provenance.** The upstream repository,
  revision, source paths, and review date live in a human reference that no
  package parses.
