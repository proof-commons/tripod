# `tripod-tapscript`

`tapscript` adapts compiler-owned abstract target requirements to target-owned
primitive contracts, backend-pattern obligations, structural obligations, and
external evidence requirements.

The package contract is
[plans/packages/tapscript.md](../../plans/packages/tapscript.md). That contract
describes the eventual backend; this crate is its first, narrow stage.

This README is the crate documentation: it is included verbatim as the rendered
landing page, and every example below runs as a doctest. It is meant to be
enough to use the public API correctly on its own.

## Dependencies

Two first-party packages and no third-party ones:

```text
compiler
target-elements
```

`architecture`, `realization`, and `model` are deliberately absent. An
assessment names compiler-owned abstract capabilities and target-owned
primitives; the compiler already publishes the architecture-owned facts it is
willing to project, and citing those packages here would reach around that
projection rather than consume it. `linker`, `transaction`, `vectors`,
`release`, and `artifacts` are absent because no target program, bundle, or
publication exists to hand them.

### How the neighbors relate

The crate is the join between exactly two vocabularies, and it is the only
package whose contract admits both.

- **`compiler`** supplies the abstract side. `compiler::target::RequiredCapability`
  names what an approved analysis requires of *some* target;
  `compiler::target::ExternalEvidenceRole` names an evidence obligation only a
  target's own rules can discharge; `compiler::target::TargetRequirementSet`
  carries what one analysis actually requires. Both vocabularies publish an
  `ALL` census.
- **`target-elements`** supplies the concrete side: the reviewed static
  contract, the primitive identities, the encodings, the capability registry,
  and the evidence-requirement registry. Every function here takes
  `ReviewedElementsTapscriptDefinition` and reads target facts out of it rather
  than restating any of them.
- **`target-elements-conformance`** is downstream, not a dependency. It takes
  the exact script bytes a `TapscriptProgram` serializes and asks an external
  executor what a node did with them. Nothing in this crate knows that package
  exists.

## Getting the target contract

Every public entry point in this crate takes the same first argument: the
reviewed static contract.

```text
target_elements::reviewed_elements_tapscript()
    -> Result<ReviewedElementsTapscriptDefinition, Vec<TargetError>>
```

`ReviewedElementsTapscriptDefinition` has no public constructor. A caller cannot
assemble a definition of its own and pass it here; a caller-built contract can be
locally valid without being the reviewed Elements one, and only the reviewed
wrapper reaches these entry points.

## Quickstart: build a program, round-trip it, validate it

This is the primary workflow.

```rust
use tapscript::{
    AbstractLimits, AbstractStackState, StackItem, TapscriptInstruction, TapscriptProgram,
    validate_program,
};
use target_elements::{FailureCause, OpcodeId, StackValueType, reviewed_elements_tapscript};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The reviewed static contract is the input of everything here.
    let target = reviewed_elements_tapscript().expect("the reviewed contract validates");

    // Programs are built from reviewed primitive identities and checked
    // literals. No raw byte enters by this path.
    let program = TapscriptProgram::new(vec![
        TapscriptInstruction::Push(StackItem::script_number(&target, 7)?),
        TapscriptInstruction::Push(StackItem::script_number(&target, 7)?),
        TapscriptInstruction::Opcode(OpcodeId::Equal),
    ])?;

    // The serializer resolves every opcode byte and push form from the
    // contract, and the parser accepts exactly the reviewed subset back.
    let bytes = program.encode(&target);
    assert_eq!(TapscriptProgram::decode(&target, &bytes)?, program);

    // The validator answers three sets, never one Boolean.
    let result = validate_program(
        &target,
        &program,
        &AbstractStackState::from_main(Vec::new()),
        AbstractLimits::for_target(&target),
    )?;

    // One clean state, holding the Boolean the comparison pushed.
    assert_eq!(result.success().len(), 1);
    assert_eq!(
        result.success().iter().next().expect("one state").main(),
        &[StackValueType::Bool],
    );

    // No failure pushed a false and carried on here.
    assert!(result.nonaborting_failure().is_empty());

    // Abort causes are retained rather than dismissed: nothing in the
    // abstract state rules out the target refusing the execution
    // domain, so the cause stays in the answer.
    assert!(
        result
            .aborts()
            .contains(&FailureCause::UnsupportedExecutionDomain),
    );
    assert!(!result.always_aborts());
    Ok(())
}
```

The last assertion is the one worth reading twice. A declared failure cause that
the abstract state cannot rule out stays in the answer, so an empty `aborts()`
set means something and is not the ordinary case.

## Quickstart: assess what this target obliges a backend to do

```rust
use compiler::target::RequiredCapability;
use tapscript::{AssessmentDisposition, assess_complete_census};
use target_elements::reviewed_elements_tapscript;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let assessed = assess_complete_census(&target)?;

    let owner = assessed
        .capability_assessment(RequiredCapability::OwnerAuthorization)
        .expect("the census covers every compiler capability");

    // A multi-state answer: the sighash construction was not reached by
    // the review, so the prerequisites are not established.
    assert_eq!(
        owner.disposition(),
        AssessmentDisposition::MissingTargetPrimitives,
    );
    Ok(())
}
```

## Public-API tour

Five public modules. Everything named below is re-exported at the crate root, so
`tapscript::TapscriptProgram` and `tapscript::program::TapscriptProgram` are the
same path.

### `instruction` — the typed instruction and its literals

| Item | Contract |
|---|---|
| `enum TapscriptInstruction` | `Opcode(OpcodeId)` or `Push(StackItem)`. Two variants, and no third: there is no raw-byte instruction. |
| `struct StackItem` | A byte string the target admits as a stack literal. The field is private and every constructor checks the target's literal bound. |

`StackItem` constructors, each resolving width, byte order, and canonical form
from the reviewed contract rather than restating them:

```text
StackItem::new(target: &ReviewedElementsTapscriptDefinition, bytes: Vec<u8>)
    -> Result<Self, TapscriptError>
StackItem::empty() -> Self
StackItem::script_number(target: &..., value: i64) -> Result<Self, TapscriptError>
StackItem::signed_le64(target: &..., value: i64) -> Self
StackItem::unsigned_le32(target: &..., value: u32) -> Self
StackItem::unsigned_le64(target: &..., value: u64) -> Self
StackItem::encoded(target: &..., class: EncodingClass, bytes: Vec<u8>)
    -> Result<Self, TapscriptError>
```

`signed_le64`, `unsigned_le32`, and `unsigned_le64` are infallible: the width is
fixed by the class, so no value can overflow the item.

Accessors: `bytes() -> &[u8]`, `len() -> usize`, `is_empty() -> bool`, and
`script_number_value(target) -> Option<i64>`. The last is the inverse of
`script_number` over the canonical domain only — it answers `None` for any item
the target's own minimality rule would reject, because such an item is one the
target aborts on rather than one that carries a number.

The constructors carry no protocol meaning. There is no receipt value here, no
owner, and no amount of anything: those are the attestation contract's
semantics, and a constructor named for one would put protocol meaning inside a
package whose entire subject is the target.

### `program` — validated programs, the serializer, and the parser

```text
const MAXIMUM_PROGRAM_INSTRUCTIONS: u64 = 10_000

TapscriptProgram::new(instructions: Vec<TapscriptInstruction>)
    -> Result<Self, TapscriptError>
TapscriptProgram::instructions(&self) -> &[TapscriptInstruction]
TapscriptProgram::len(&self) -> usize
TapscriptProgram::is_empty(&self) -> bool
TapscriptProgram::encode(&self, target: &ReviewedElementsTapscriptDefinition) -> Vec<u8>
TapscriptProgram::decode(target: &ReviewedElementsTapscriptDefinition, bytes: &[u8])
    -> Result<Self, TapscriptError>
```

`new` validates the work limit and nothing else, and the absence of the other
checks is deliberate rather than an omission: the execution domain and contract
revision are fixed by the reviewed wrapper, every `OpcodeId` has a contract gated
to that domain, and every `StackItem` is already within the literal bound.
Re-testing any of those would add a branch no input can reach, and an unreachable
branch in a validator reads as a check that is running when it is not.

`MAXIMUM_PROGRAM_INSTRUCTIONS` is a first-party bound, not a target one. The
reviewed execution domain enforces no script-size limit at all.

`encode` is deterministic: the bytes depend on the instruction sequence and on
nothing else — not on map iteration, declaration order, host, thread count, or
environment.

### `stack` — the abstract validator

```text
validate_program(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
    initial: &AbstractStackState,
    limits: AbstractLimits,
) -> Result<AbstractExecutionResult, TapscriptError>

resource_projection(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> BTreeMap<ResourceDimension, u64>
```

`AbstractStackState` is the typed initial stack:

```text
AbstractStackState::new(main: Vec<StackValueType>, alternate: Vec<StackValueType>) -> Self
AbstractStackState::from_main(main: Vec<StackValueType>) -> Self
AbstractStackState::main(&self) -> &[StackValueType]
AbstractStackState::alternate(&self) -> &[StackValueType]
AbstractStackState::depth(&self) -> usize
```

The alternate stack is represented even though no reviewed primitive touches it.
Leaving it out would make its depth an untracked axis that a later primitive
could start moving without anything noticing.

`AbstractExecutionResult` carries three sets and never collapses them:
`success()`, `nonaborting_failure()`, and `aborts()`, plus `always_aborts()` for
the case where no state survives at all. `StackValueType` and `FailureCause` are
`target-elements` vocabulary, not this crate's.

`AbstractLimits` is the work budget. `AbstractLimits::for_target(target)` derives
it: the stack depth comes from the target's own consensus bound, so the validator
refuses exactly what the target would, while the other three are first-party work
bounds. The four `with_maximum_*` setters — `with_maximum_states`,
`with_maximum_stack_depth`, `with_maximum_instructions`, and
`with_maximum_result_alternatives` — are **narrowing only**: each takes the
smaller of the two figures, so a caller can ask for less work than the target
permits and can never ask for more. Four matching accessors read the figures
back.

`resource_projection` is a projection rather than a bound. It states what the
reviewed contracts say the primitives in a program can consume; nothing compares
it against a deployment, and its totals saturate rather than wrap, because a
saturated total is visibly pinned where a wrapped one would read as small and
honest.

### `capability` — the adapter proper

```text
assess_static_capability(
    target: &ReviewedElementsTapscriptDefinition,
    required: RequiredCapability,
) -> StaticCapabilityAssessment

assess_evidence_role(role: ExternalEvidenceRole) -> ExternalEvidenceAssessment

assess_requirements(
    target: &ReviewedElementsTapscriptDefinition,
    requirements: &TargetRequirementSet,
) -> Result<TargetAssessmentSet, TapscriptError>

assess_complete_census(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<TargetAssessmentSet, TapscriptError>
```

`assess_static_capability` is pure and deterministic: the result depends on the
contract's capability registry and on nothing else — no clock, no environment, no
interior state, no deployment binding, and no node.

`assess_evidence_role` takes no target, deliberately. The obligation a role lands
on is a property of the role and of the target's evidence vocabulary, not of any
one contract value, and taking a target here without reading it would suggest a
binding-aware assessment that is not happening.

`assess_requirements` narrows to one analysis; `assess_complete_census` answers
the whole compiler vocabulary and is the upper bound on any requirement set.

`TargetAssessmentSet` carries both censuses:

```text
capability_assessments(&self) -> impl Iterator<Item = (RequiredCapability, &StaticCapabilityAssessment)>
capability_assessment(&self, required: RequiredCapability) -> Option<&StaticCapabilityAssessment>
evidence_assessments(&self) -> impl Iterator<Item = (ExternalEvidenceRole, &ExternalEvidenceAssessment)>
evidence_assessment(&self, role: ExternalEvidenceRole) -> Option<&ExternalEvidenceAssessment>
capability_projection(&self) -> Vec<AssessmentProjection>
evidence_projection(&self) -> Vec<EvidenceAssessmentProjection>
```

The two projections are vectors in canonical census order rather than sets: the
order is part of the contract, and a set comparison would accept a projection
that named one capability twice.

`StaticCapabilityAssessment` has six variants, each retaining the `required`
capability it answers, and each carrying what its disposition actually implies:

| Variant | Also carries | Means |
|---|---|---|
| `Unsupported` | `reason: UnsupportedReason` | The reviewed contract states a needed primitive does not exist. A reviewed negative fact; further review does not turn it into a met prerequisite. |
| `MissingTargetPrimitives` | `missing` | The primitives exist on paper but the review did not establish them. |
| `BackendPatternRequired` | `primitives`, `structural`, `evidence` | Prerequisites hold; a backend proof pattern nobody has written is owed. |
| `BackendStructural` | `primitives`, `requirements` | The obligation is a structural fact the compiler and the transaction ABI owe, not a target program. It deliberately carries no evidence field. |
| `ExternalEvidenceRequired` | `evidence` | Only the target's own consensus rules discharge it. |
| `CompleteBackendPattern` | `pattern: BackendPatternId` | Unreachable — see below. |

Accessors: `required()`, `disposition()`, and `projection()`.
`AssessmentDisposition` is the flat six-valued discriminant, and
`AssessmentProjection` is the stable comparison form, reading back `required()`,
`disposition()`, `primitives()`, `structural()`, and `evidence()`.

`ExternalEvidenceAssessment` is the evidence-role counterpart, with `role()`,
`disposition()` (an `EvidenceAssessmentDisposition`), and `projection()` giving
an `EvidenceAssessmentProjection` whose `evidence()` lists the requirement ids.

`BackendFoundationRequirement` enumerates the structural obligations the compiler
and the transaction ABI owe; its complete census is
`BackendFoundationRequirement::ALL`, and the variants are declared in
`src/capability.rs`. None of them is a completed target program, and none is
something a target opcode can establish.

`BackendPatternId` is **uninhabited** — `pub enum BackendPatternId {}`. No
approved complete backend pattern exists, and that fact is enforced by the type
system rather than by convention: the `CompleteBackendPattern` variant cannot be
constructed, so no assessment can claim a capability is finished.

### `error` — the single error root

`TapscriptError` is the crate's one error type and implements
`std::error::Error`. It has 17 variants in four families:

| Family | Variants | Returned by |
|---|---|---|
| Literal and encoding | `OversizedStackItem`, `ScriptNumberOutOfRange`, `MalformedEncodedItem`, `NonMinimalScriptNumber` | the `StackItem` constructors |
| Parsing | `UnknownOpcodeByte`, `TruncatedInstruction`, `NonMinimalPush`, and `OversizedStackItem` again | `TapscriptProgram::decode` |
| Abstract validation and budget | `StackUnderflow`, `StackTypeMismatch`, `AbstractStateLimitExceeded`, `StackLimitExceeded`, `ResultAlternativeLimitExceeded`, `InstructionLimitExceeded` | `validate_program` (and `InstructionLimitExceeded` also from `TapscriptProgram::new`) |
| Census disagreement | `DuplicateCapabilityAssessment`, `CapabilityAssessmentCensusMismatch`, `DuplicateEvidenceAssessment`, `EvidenceAssessmentCensusMismatch` | `assess_requirements` and `assess_complete_census` |

The variants are declared in `src/error.rs`. Every entry point's own `# Errors`
rustdoc section names the exact variants it can return.

Work exhaustion is worth calling out: the four budget variants return a typed
error and **no partial result**. A truncated state set would be
indistinguishable from a complete one and would understate what the program can
produce.

## Raw bytes have exactly one way in

The safe construction API offers no raw opcode, no raw instruction, and no raw
program. A program is built from reviewed primitive identities and checked
literals; untrusted bytes reach it only through the parser, which either
produces typed instructions or fails with a focused reason. Every opcode byte
and every push form the serializer emits is resolved from the reviewed target
contract, so no target number is restated in this crate.

The parser refuses a nonminimal push always, which is stricter than the target's
own validity rules: the target enforces minimality only under the standardness
rules a node applies to what it relays. That is deliberate — a first-party
program no node forwards is of no use, and the encode/decode round-trip property
holds only for the minimal form.

`TapscriptProgram::decode` is a boundary for correlating first-party programs
with target-native results, not a general Elements script parser. Widening it
would quietly make it one.

## Three outcomes, and every alternative

The abstract stack validator answers what a program can produce as three sets: the
states it reaches cleanly, the states it reaches through a failure that pushed a
false and carried on, and the causes on which it ends evaluation. They are never
collapsed into one Boolean, because their stack depths differ exactly where a
backend has to be careful — the fixed-width arithmetic leaves its operands in
place and pushes a false above them.

Where a primitive has more than one successful form and the discriminant is a
property of the target value rather than of the program, every compatible
alternative is retained. Taking the first would silently commit to one shape of a
stack the target may produce in another. Work exhaustion returns a typed error
and no partial result.

Two failure causes are decided by the abstract state rather than recorded as
something the target might do: insufficient operands, because the state says how
deep the stack is, and an operand of the wrong width, once every operand's
abstract type fixes one. Both surface as validation failures of the program.
Every other declared cause is recorded, because nothing in the abstract state
rules it out.

## Static, not deployment-aware

Every assessment here is a statement about the reviewed *static* target
contract. Nothing in the crate accepts a development binding, and nothing reads
a network identity, a genesis identity, an activation declaration, or a
deployment resource override. The static contract, the deployment declaration,
and target-native evidence are three different values; a function that took one
and answered for another would be a false claim about binding-aware assessment.
A deployment-aware assessment is deferred until a consumer for one exists; when
it arrives it will be a different function, over a different input, returning a
different type.

## Both published censuses are answered

The compiler publishes what an analysis requires as two censuses — abstract
capabilities and external-evidence roles — and the adapter answers both, with
exact equality in both directions. A compiler evidence role therefore cannot
disappear at this boundary: the mapping is exhaustive, so a new role stops this
crate compiling until its target obligation is stated.

## Support is not a boolean

An assessment is a typed multi-state result, never `Supported(bool)`. It
separates a reviewed negative fact from a missing primitive, a missing
primitive from a backend pattern that nobody has written, a backend pattern
from a structural obligation the compiler and the ABI owe, and a structural
obligation from external evidence only the target's own rules can produce.
Collapsing any of those distinctions would let an assessment read as progress
that has not happened.

There is no `is_supported` accessor, and there is no way to add one from
outside: the disposition that would mean it, `CompleteBackendPattern`, has no
constructible value.

## Implemented

```text
package boundary
typed instruction and checked stack item
exact serializer and reviewed-subset parser
abstract stack validator over the reviewed primitive contracts
static capability adapter over the reviewed target contract
external-evidence-role adapter
candidate shape set, backend policy, and typed proof patterns
static ASH constructor and candidate relocatable bundle
static live-receipt constructor and its transfer leaf schema
live coordinator and member patterns and their candidate bundle
```

### The live-receipt constructor (Guide-13 §7, §10)

`derive_live_receipt_constructor` is the sole route to a
`StaticLiveReceiptConstructor`, and it owns the static leaf schema: it
refuses a leaf naming another representation, a leaf set missing a
required coordinator or member, and a leaf serving no admitted shape. An
empty leaf set is refused as the key-path escape it is rather than as a
bookkeeping complaint, because a taproot output with no script path can
be spent only through its key path.

That schema is validated here and nowhere else. Everything downstream
receives a sealed constructor, so the malformed leaf sets above are
inputs only this entry point can be offered — which is why the safety
matrix's constructor-schema row names this boundary and not the
linker's.

The leaf-role vocabulary has exactly two members, a coordinator of one
exact shape and a member of one receipt-input count, each keyed by its
representation. Sharing a leaf between representations is not the
default narrowed later; two representations get two leaves until some
complete typed proof says they may get one.

The crate's own mutation census records what construction does about
each named change and, for the ones it cannot express, what is still
missing. A case that reaches past this constructor carries the residual
that a complete target transaction and an observed verdict are required,
because nothing here may claim what a target would say.

## What this package deliberately does not do

Not implemented:

```text
linked bundle and taptree
transaction ABI
target execution
```

Beyond those, and by design rather than by omission:

- the programs it emits are relocatable and unresolved: every link-time literal
  is a symbol a later layer settles, and the bundle is a candidate whose clear
  lifecycle is outstanding — the type refuses to exist for a plan whose
  lifecycle is complete;
- it executes nothing — `validate_program` is a statement about the reviewed
  contracts, not about a node;
- it restates no target number: every opcode byte, push form, encoding width,
  byte order, and resource bound is read from `target-elements`;
- it serializes nothing to disk, hashes nothing, opens no file, and mints no
  identity;
- it accepts no deployment binding and reads no network identity;
- it names no attestation-contract operation, object, relation, or protocol
  quantity — the compiler owns the abstract side of the join, and reaching past
  its projection into `architecture`, `realization`, or `model` is precisely
  what the dependency list forbids.

## Nothing here is claimed to work against a node

Programs have been emitted and walked against the reviewed contracts, and
nothing in this crate has run one: it builds no transaction, discharges no
evidence requirement the target contract names, and reaches no node. An
assessment states what a backend would have to establish, and an emitted
program states what it would attempt. Neither states that anything has been
established.

Programs this crate emitted have since been submitted to a real node by the
evidence packages, and a live coordinator reached the owner's signature check
before failing there. That is their result to report and not this crate's
claim: nothing here reads it, and no type here changes because of it.

Evidence about the target's actual behavior is produced elsewhere, by
`tripod-target-elements-conformance`, and nothing in this crate reads
it.
