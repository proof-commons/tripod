# D006: Generate a Canonical Transaction and Witness ABI for Each Backend Bundle

> **Status:** ACCEPTED
> **Scope:** Concrete transaction shape, input/output-family placement,
> obligation placement, witness conventions, transaction construction,
> calibration, and target-vector materialization
> **Decision class:** ABI
> **Applies to:** `realization`, `compiler`, `target-elements`, `tapscript`,
> future `simplicity`, `linker`, `transaction`, `vectors`, wallets/clients,
> calibration tooling, and `release`
> **Depends on:** [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md);
> [D002: Introduce a Target-Independent Realization Layer](002-target-independent-realization-layer.md);
> [D003: Design for Multiple Backends and Implement Elements Tapscript First](003-multiple-backends-tapscript-first.md);
> [D004: Use Per-Bundle Translation Validation Instead of Initially Trusting the Compiler](004-translation-validation-over-compiler-trust.md);
> [D005: Permit Value-Representation Latitude While Keeping Closed Asset Identity Rigid](005-value-parametric-asset-rigid.md)
> **Supersedes:** none
> **Superseded by:** none
> **Related normative constraints:** exact input/output cardinalities, exact
> canonical and open-flow partitions, transaction-shape obligations, operation
> selection, bounded collection lowering, obligation placement,
> permissionless construction, relabel layout evidence, and code-generation
> checklist in `docs/attestation/realization.md`
> **Related research:**
> [`../research/state-object-constructor.md`](../research/state-object-constructor.md),
> [`../research/public-declassification.md`](../research/public-declassification.md),
> [`../research/settlement-layout.md`](../research/settlement-layout.md)
> **Promoted ADR:** none
> **Machine-consumed by the toolchain:** no

---

## 1. Context

The target-independent realization describes operations in terms of semantic
families and relations.

For example, an operation may require:

- one STATE input and one STATE successor;
- a nonempty bounded family of receipt inputs;
- a bounded family of receipt outputs;
- zero or one sponsor-change output;
- one data-output family;
- exact conservation over all members of one family;
- one root successor;
- one event projection;
- one formula-bound recipient;
- one coordinator relation over the complete transaction.

Those semantic declarations do not, by themselves, determine:

- which concrete transaction input index carries each family;
- which output range carries each family;
- which input executes a transaction-global check;
- where optional outputs appear;
- how a bounded family is terminated;
- how count witnesses are authenticated;
- how a sponsor region is separated from protocol regions;
- which witness item carries an arithmetic quotient;
- which tapleaf or target program is selected;
- how metadata fields are serialized;
- how a client constructs a valid transaction;
- how a vector harness mutates one relation without corrupting unrelated
  structure;
- how worst-case transactions are generated for calibration.

The planned initial backend is Elements tapscript. Its execution model makes
these questions load-bearing.

### 1.1 No general loop assumption

The initial tapscript backend does not assume a general loop construct capable
of iterating over arbitrary transaction inputs or outputs.

A semantic requirement such as:

```text
for every receipt input:
    authenticate its class and owner
```

or:

```text
sum every ASH input
```

must be lowered through concrete finite structure, such as:

- fixed positions;
- bounded unrolling;
- per-input enforcement;
- a coordinator input;
- authenticated ranges;
- explicit count witnesses.

The manifest already requires deployment calibration for finite bounds.
Concrete target enforcement must use those bounds in a deterministic layout.

### 1.2 Transaction-global relations execute per input

Tapscript programs execute while spending individual inputs.

Some relations are local:

- this input carries asset `U`;
- this owner authorized this input;
- this input is a live receipt;
- this root input matches the expected predecessor.

Other relations are transaction-global:

- all canonical `U` outputs belong to declared families;
- the sum of all ASH inputs equals the one ASH output;
- the control and vault counters agree;
- a batch has no duplicate members;
- issuance destinations exhaust the issued amount;
- no unclassified output can carry a closed protocol asset;
- all sponsor inputs and outputs form one isolated envelope;
- the transaction contains the expected number of protocol families;
- one event projection corresponds to the complete operation shape.

A target program can enforce a global relation only if the compiler assigns it
to a concrete executable carrier—typically a coordinator input or an
explicitly duplicated set of input programs.

If obligation placement remains implicit, a relation can exist in compiler
metadata while being enforced by no target program.

### 1.3 Counts do not authenticate family ranges

Suppose a witness states:

```text
receipt_input_count = 3
receipt_output_count = 2
```

Those numbers do not prove that:

- inputs 0–2 are receipts;
- outputs 0–1 are receipt outputs;
- ranges do not overlap;
- an undeclared protocol object is absent;
- the remaining outputs are valid sponsor change or fee outputs;
- transaction totals match the proposed ranges.

Counts may propose a layout. Concrete object-constructor checks and transaction
introspection must authenticate it.

The rule is:

> Counts propose ranges; constructor and transaction-shape checks authenticate
> those ranges.

### 1.4 Wallets and vectors need the same ABI

A target script is useful only if an authorized constructor can build a valid
transaction and witness.

The following components all need one shared concrete contract:

- production wallets and clients;
- operation builders;
- the translation-validation vector harness;
- the calibration transaction generator;
- genesis/deployment tooling;
- independent implementers forming candidate transactions.

If each component independently infers target layout from emitted scripts,
drift is inevitable.

The transaction and witness conventions must therefore be a first-class typed
artifact.

### 1.5 Model output order is not automatically target ABI

The executable model uses ordered output references internally, but its
semantic safety properties often concern:

- object families;
- cardinalities;
- owner/class/value multisets;
- formula-bound destinations;
- exact value partitions.

A target backend may need a stricter positional convention to make those
relations locally checkable.

For example, the model-level relabel safety property is an owner/value multiset
bijection. One tapscript realization may use:

```text
STATE successor first
then one live receipt per time-locked input in input order
then optional sponsor change
```

That positional convention is a backend proof strategy, not the abstract
economic property.

The project must preserve this distinction.

### 1.6 Resource calibration requires complete transactions

Target limits apply to complete transactions and per-input execution, not to
isolated semantic relations.

Calibration needs to construct worst-case transactions using the exact:

- input count;
- output count;
- selected leaves/programs;
- witness items;
- metadata;
- control blocks;
- confidential proof material;
- sponsor region;
- data outputs.

Therefore the canonical transaction ABI is an input to calibration, not a
post-release wallet convenience.

---

## 2. Decision

For every linked backend bundle, generate a deterministic, typed
**transaction and witness ABI** describing how each semantic operation is
materialized concretely on that target.

The ABI is derived from:

- typed architecture cardinalities and bounds;
- typed realization relations;
- compiler proof, disclosure, constructibility, and placement plans;
- target capabilities;
- backend lowering policy;
- linked constructors and programs;
- calibrated deployment parameters.

The canonical direction is:

```text
architecture + realization
        ↓
compiler semantic and placement requirements
        ↓
backend layout lowering
        ↓
linker resolves constructors and programs
        ↓
typed canonical transaction/witness ABI
        ↓
wallets, vectors, calibration, and release publication
```

The central decisions are:

1. **Semantic cardinality and concrete layout are distinct.**
   Architecture and realization define allowed families and relations. The
   backend defines one deterministic concrete arrangement that enforces them.

2. **Every operation receives a canonical target layout.**
   The layout defines input and output family positions or ranges, optional
   families, coordinator rules, sponsor regions, data-output placement, and
   transaction-global constraints.

3. **Every target-enforced relation receives a carrying predicate.**
   Obligation placement is explicit and typed. Unplaced obligations are
   compilation failures.

4. **Count and range witnesses are authenticated.**
   Concrete transaction counts, constructor checks, range disjointness, and
   family completeness must agree.

5. **Witness conventions are part of the ABI.**
   Witness item purpose, ordering, encoding, availability, and target-program
   selection are typed and versioned.

6. **The transaction package is first-class.**
   It owns typed construction from linked bundle plus operation request. It is
   not merely vector-test plumbing.

7. **Calibration uses ABI-valid complete transactions.**
   Bound measurement is performed over worst-case concrete transactions
   generated under the same ABI used by clients and vectors.

8. **The ABI is target- and bundle-specific.**
   Different backends may use different concrete layouts while realizing the
   same semantic operation.

9. **The ABI is deterministic from explicit inputs.**
   Identical architecture, realization, target, compiler policy, backend,
   linked bundle, and deployment parameters produce byte-identical canonical
   ABI publications.

10. **The ABI does not become protocol semantics.**
    A concrete positional layout is an implementation/evidence choice unless
    the normative realization explicitly declares the position itself
    observable.

11. **Clients do not reverse-engineer emitted target programs.**
    They consume the typed ABI or its canonical publication.

12. **A layout change moves ABI and bundle/configuration identity as
    applicable.**
    It does not move architecture or realization identity when semantic
    behavior is unchanged.

The detailed package contract is in
[`../packages/transaction.md`](../packages/transaction.md).

---

## 3. Layout model

### 3.1 Operation layout

A target operation layout should conceptually contain:

```rust
pub struct OperationLayout {
    pub operation: architecture::OperationId,
    pub layout_schema: LayoutSchemaVersion,
    pub target: TargetIdentity,
    pub bundle: BundleIdentity,

    pub inputs: Vec<InputFamilyLayout>,
    pub outputs: Vec<OutputFamilyLayout>,
    pub data_outputs: Vec<DataOutputFamilyLayout>,

    pub coordinator: Option<CoordinatorLayout>,
    pub placements: Vec<PlacedObligation>,
    pub sponsor: SponsorLayout,
    pub transaction_constraints: Vec<TransactionConstraint>,
}
```

> Illustrative API; exact type names and fields are not frozen by this
> decision.

### 3.2 Input family layout

An input-family layout should be able to express:

- architecture object family;
- semantic role;
- cardinality minimum;
- calibrated maximum;
- fixed position or bounded range;
- local target program/leaf;
- authorization mode;
- required metadata;
- required value/asset representation;
- witness ABI reference;
- whether one member is coordinator;
- ordering rule inside the family.

### 3.3 Output family layout

An output-family layout should be able to express:

- architecture object family;
- semantic role;
- cardinality minimum and maximum;
- fixed slot or bounded range;
- constructor identity;
- metadata source;
- value source/expression;
- recipient source;
- representation requirement;
- activation condition;
- ordering rule;
- relation provenance.

### 3.4 Index rules

Concrete index rules may include:

```rust
pub enum IndexRule {
    Fixed(u16),

    ContiguousRange {
        start: IndexExpression,
        count: CountVariable,
    },

    OptionalFixed {
        index: IndexExpression,
        present: ConditionId,
    },

    FinalOptional,

    WitnessReferenced {
        authenticated_range: RangeId,
    },
}
```

> Illustrative vocabulary; not frozen.

The initial backend should prefer simple fixed and contiguous rules over
general index-expression languages.

### 3.5 Coordinator layout

A coordinator is the input program responsible for transaction-global
relations.

Its layout should identify:

- coordinator input family;
- coordinator selection rule;
- transaction-global relations carried;
- family counts inspected;
- input/output ranges authenticated;
- cross-input metadata inspected;
- cross-output constructors inspected;
- duplicate enforcement intentionally carried elsewhere.

A coordinator must not be selected by an attacker-controlled witness without
being authenticated against the canonical layout.

### 3.6 Sponsor layout

The sponsor region should identify:

- whether sponsorship is permitted;
- sponsor input family;
- calibrated sponsor-input maximum;
- sponsor authorization;
- allowed sponsor output/change family;
- chain-fee relation;
- location relative to protocol families;
- value/asset representation requirements;
- whether zero sponsor inputs are allowed.

Sponsor ranges must not overlap protocol ranges.

### 3.7 Data-output layout

Data outputs require canonical placement because event and destruction
projections depend on:

- output family;
- tag;
- asset;
- amount;
- record index;
- transaction output order.

The ABI must identify:

- family ordering;
- ordinal rule;
- activation condition;
- target encoding;
- unspendability requirement;
- relation/event provenance.

### 3.8 Transaction-level constraints

The layout must be able to require:

- exact or bounded input/output counts;
- transaction version;
- locktime;
- sequence rules;
- target weight bound;
- canonical family order;
- absence of undeclared protocol outputs;
- fee-output policy;
- operation-specific target constraints.

These constraints belong to the concrete target ABI, not necessarily to the
abstract operation.

---

## 4. Witness ABI

### 4.1 Purpose

The witness ABI tells an authorized constructor exactly what data each input
program expects and in which order.

It must not be inferred from backend source or disassembled target programs.

### 4.2 Witness item schema

A witness item should conceptually record:

```rust
pub struct WitnessItem {
    pub name: WitnessItemId,
    pub role: WitnessRole,
    pub encoding: WitnessEncoding,
    pub availability: WitnessAvailability,
    pub secrecy: WitnessSecrecy,
    pub bounds: WitnessBounds,
    pub source_relation: Option<RelationId>,
}
```

> Illustrative API; not frozen.

Possible roles include:

- owner signature;
- operator signature;
- sponsor signature;
- floor quotient;
- remainder;
- value opening;
- public opening;
- metadata field;
- constructor root;
- parity bit;
- family count;
- output reference;
- target proof;
- operation selector;
- script/control data.

### 4.3 Witness ordering

Ordering must be canonical per target program.

The ABI should distinguish:

- initial witness stack items;
- target program/leaf;
- control data;
- annex or target-specific fields where supported;
- per-input versus transaction-shared conceptual data.

Tapscript inputs do not share ordinary witness stacks. A transaction-global
fact needed by multiple input programs must therefore be:

- independently derivable through introspection;
- duplicated explicitly;
- authenticated through a common committed source;
- or placed only on one coordinator.

### 4.4 Witness availability

The witness ABI must preserve the realization/compiler availability
classification.

For example:

```text
owner signature:
    current owner secret

operator signature:
    operator secret

sponsor signature:
    sponsor local

public ASH value/opening:
    public chain data or public opening

constructor root:
    public linked-bundle data

deployment constant:
    compile time or deployment constant
```

A permissionless operation cannot contain an unavailable private witness.

### 4.5 Secret handling

Canonical ABI publications must not contain:

- private keys;
- private blinding factors;
- production signatures;
- unpublished openings;
- secret nonces.

They describe item types and sources, not secret values.

Test vectors may contain fixed non-production secrets under an explicit fixture
schema.

### 4.6 Canonical encoding

Every witness encoding must define:

- byte order;
- fixed or variable length;
- minimality;
- domain bounds;
- canonical zero;
- target prefix;
- malformed-input behavior;
- public versus secret status.

A generic “bytes” witness without a documented semantic encoding is
insufficient for a release ABI.

---

## 5. Obligation placement

### 5.1 Placement input

The compiler provides target-independent placement requirements:

- relation dependencies;
- facts required;
- local versus global character;
- candidate input families;
- duplication allowance;
- constructibility constraints.

The backend combines them with target and layout facts.

### 5.2 Placement result

A placed obligation must identify:

- semantic relation ID;
- selected proof plan;
- carrying target program;
- carrying input family;
- facts authenticated there;
- local or global scope;
- primary or duplicate enforcement;
- target capabilities used;
- vector coverage requirements.

### 5.3 Placement completeness

Compilation fails when:

- no carrier can authenticate the required facts;
- a transaction-global relation is assigned only to a local input that cannot
  inspect the complete transaction;
- a relation is carried only on an optional branch that need not execute;
- a permissionless carrier requires unavailable witness data;
- the chosen layout makes mutation or relation activation ambiguous beyond the
  accepted evidence policy;
- two carriers assume inconsistent family ranges.

### 5.4 Initial coordinator defaults

The following are plausible initial coordinator choices:

| Operation | Likely coordinator |
|---|---|
| `create-request` | Client-policy construction; no protocol covenant input. |
| `cancel-request` | Request input. |
| `admit-deposits` | STATE input. |
| `cycle` | STATE input. |
| `settle-distribution` | Distribution-control input. |
| `transfer-live-receipts` | First live-receipt input. |
| `transfer-time-locked-receipts` | First time-locked-receipt input. |
| `redeem` | STATE input. |
| `receipt-relabel` | STATE input. |
| `burn` | First live-receipt input. |
| `compact-ash` | First ASH input. |
| `clear` | STATE input. |
| `announce-maturity` | STATE input. |

These are planning defaults, not frozen ABI.

The compiler/backend may select a different carrier when it proves the same
relations more clearly or efficiently.

### 5.5 Coordinator selection inside ranged families

When the coordinator belongs to a repeated input family, the ABI must specify
one canonical member, for example:

```text
the lowest-index member of the family range
```

Other members must prove they belong to the same authenticated operation
layout and cannot independently claim coordinator status.

### 5.6 Duplication policy

Some critical checks may be deliberately duplicated, such as:

- local owner authorization on every owner-bearing input;
- local object recognition on every input;
- root identity on every root input;
- global closure on the coordinator.

Duplicated enforcement must be explicit in the placement plan and resource
model.

The compiler must not accidentally duplicate expensive global arithmetic on
every input unless policy selects that strategy.

---

## 6. Range authentication

### 6.1 Proposed ranges are not trusted

A family count supplied in a witness is an untrusted proposal.

The target program must compare it against:

- manifest-derived minimum;
- deployment-calibrated maximum;
- actual transaction input/output totals;
- family constructor checks;
- canonical start/end positions;
- disjointness with other ranges.

### 6.2 Family completeness

For every protocol-capable slot, the coordinator or local carrier must prove
that the slot belongs to one declared family.

This is especially important for closed assets.

No output may remain in an unclassified region where it could carry:

- `U`;
- `ENT`;
- `DIST_CTL`;
- root or authority assets.

### 6.3 Sponsor suffix

A simple initial policy is:

```text
protocol families first
sponsor/open region last
```

The coordinator authenticates the boundary.

The sponsor region may contain only the families allowed by the operation and
deployment profile.

A suffix policy is not required by abstract semantics. It is one canonical
target layout.

### 6.4 Optional families

An optional family must have one canonical representation, such as:

- zero count in a fixed range;
- one fixed conditional slot;
- omission under a condition with following ranges computed canonically.

The ABI must avoid several equivalent encodings for the same semantic
transaction unless equivalence is intentional and tested.

### 6.5 Conditional output families

Conditionally active outputs include:

- distribution control and vault;
- ASH residual;
- sponsor change;
- RESV successor on nonterminal redemption;
- CPFP anchor at maturity;
- terminal residue data output.

The layout must bind each activation condition to:

- semantic condition;
- concrete presence/absence rule;
- count;
- constructor;
- value relation;
- subsequent range calculation.

### 6.6 Input duplicate detection

Transaction consensus prevents the same outpoint from appearing twice in one
valid transaction, but the semantic relation and target vectors should still
make duplicate behavior explicit where relevant.

The ABI and vector harness should distinguish:

- duplicate transaction outpoint rejected by consensus;
- duplicate semantic witness/reference inside one family;
- one source claimed by two flows;
- one output claimed by two relations.

---

## 7. Package responsibilities

### 7.1 Architecture

Owns:

- operation families;
- minimum/maximum form;
- bound references;
- object families;
- authorization modes;
- roots;
- projections;
- semantic structural declarations.

Does not own concrete positions.

### 7.2 Realization

Owns:

- semantic operation relation;
- semantic family roles;
- relation dependencies;
- constructibility;
- lifecycle;
- representation latitude;
- public observables.

Does not own concrete target index rules.

### 7.3 Compiler

Owns:

- relation graph;
- proof plan;
- placement requirements;
- layout requirements;
- disclosure plan;
- target capability requirements.

Does not own final target witness bytes.

### 7.4 Backend

Owns:

- concrete target layout lowering;
- target-program assignment;
- instruction/combinator patterns;
- concrete obligation placement;
- target witness requirements;
- resource formulas.

### 7.5 Linker

Owns:

- final constructor and program resolution;
- taptree or target program assembly;
- relocation;
- deployment constants;
- linked-bundle identity;
- final program references needed by the ABI.

### 7.6 Transaction package

Owns:

- typed canonical ABI;
- operation request types;
- target transaction construction;
- witness materialization;
- metadata encoding;
- leaf/control selection;
- representation/blinding construction;
- worst-case transaction generation.

### 7.7 Vector package

Owns:

- semantic vector definition;
- ABI-driven target transaction materialization;
- malformed layout mutations;
- witness mutations;
- target execution;
- relation coverage;
- public projection comparison.

### 7.8 Release package

Owns:

- ABI identity binding;
- bundle/ABI consistency;
- canonical ABI publication;
- evidence consistency;
- final deployment-profile checks.

---

## 8. Required consequences

### 8.1 New first-class `transaction` package

Create:

```text
packages/transaction/
```

with package identity:

```text
tripod-transaction
```

The package is introduced when the first linked backend operation is ready for
concrete transaction construction.

It should not be added as an empty crate during Phase 1.

### 8.2 Typed operation request API

The transaction package should expose typed requests rather than generic
transactions assembled by callers.

A request might conceptually look like:

```rust
pub struct CompactAshRequest {
    pub ash_inputs: Vec<OutPoint>,
    pub sponsor: Option<SponsorRequest>,
}
```

```rust
pub struct TransferLiveRequest {
    pub receipt_inputs: Vec<OutPoint>,
    pub destinations: Vec<ReceiptDestination>,
    pub owner_signatures: SignerInputs,
    pub sponsor: Option<SponsorRequest>,
    pub representation: TransferRepresentationRequest,
}
```

> Illustrative APIs; not frozen.

The builder validates request compatibility with the linked ABI before
constructing target bytes.

### 8.3 ABI publication

A deployment should publish a canonical ABI artifact sufficient for an
independent constructor to determine:

- operation layouts;
- family ranges;
- constructor parameters;
- witness item schemas;
- supported representation modes;
- target program selection;
- metadata encoding;
- calibrated maxima;
- transaction-level constraints.

The exact artifact name and schema remain to be defined.

Potential name:

```text
transaction_abi.json
```

The typed Rust value remains authoritative for first-party code under D001.

### 8.4 ABI and bundle consistency

The ABI must bind the exact linked bundle.

The release gate must reject:

- ABI for a different bundle;
- ABI for a different target;
- ABI for different calibrated bounds;
- ABI referencing missing constructors;
- ABI referencing different program/leaf IDs;
- ABI with unsupported schema;
- ABI missing a required operation.

### 8.5 Layout-driven vector generation

The vector harness must use the ABI to build valid transactions and focused
mutations.

It must not maintain a separate handwritten transaction shape per operation.

Target-specific mutation helpers may operate over the ABI but cannot redefine
it.

### 8.6 Layout-driven calibration

Worst-case transaction generation must use:

- maximum calibrated family counts;
- maximum witness encodings;
- selected target proof plans;
- final constructors;
- final control data;
- worst-case sponsor use;
- all conditionally active outputs relevant to the measured branch.

Calibration must not estimate full transaction cost by multiplying isolated
leaf cost without constructing a valid transaction.

### 8.7 Client compatibility

The ABI is a deployment interface for clients.

A client may implement construction independently from canonical ABI
publication, but it must not infer semantics from planning documents.

Changes affecting client construction require explicit ABI versioning and
release communication.

### 8.8 Client-side operations remain classified accurately

`create-request` is not a covenant input operation at creation time.

The transaction package may provide a canonical client-policy constructor, but
the target cannot enforce request validity until a later covenant operation
consumes it.

The ABI must distinguish:

```text
client-policy construction
```

from:

```text
covenant-enforced transition
```

Malformed request-shaped open outputs remain inert until consumption.

### 8.9 Multiple backend ABIs

A tapscript bundle and a future Simplicity bundle may have different:

- target programs;
- witness ordering;
- control data;
- metadata construction;
- layout;
- resource formulas.

Each has its own ABI identity while mapping to the same architecture operation
and realization relation.

The transaction package may use target-specific modules or associated types.

### 8.10 ABI schema evolution

Every published ABI schema must have:

- explicit schema version;
- unknown-field rejection;
- canonical ordering;
- deterministic rendering;
- one generator;
- one non-writing checker;
- migration policy;
- bundle identity binding;
- target identity binding.

ABI schema changes do not automatically change protocol denotation.

---

## 9. Operation-specific initial layout expectations

These expectations guide implementation but do not freeze exact indices.

### 9.1 `compact-ash`

Likely layout:

```text
inputs:
    contiguous ASH range, count 2..ASH_BATCH_MAX
    optional sponsor suffix

outputs:
    exactly one ASH
    optional sponsor change
    fee handled by target transaction semantics
```

Likely coordinator:

```text
first ASH input
```

Coordinator obligations:

- family count;
- every protocol input is ASH;
- one ASH output;
- exact ownerless `U` conservation;
- sponsor boundary;
- no other protocol output.

Local obligations on other ASH inputs:

- ASH constructor and asset identity;
- canonical operation/leaf participation.

### 9.2 `transfer-live-receipts`

Likely layout:

```text
inputs:
    contiguous live receipt range
    optional sponsor suffix

outputs:
    contiguous live receipt range
    optional sponsor change
```

Likely coordinator:

```text
first live receipt input
```

Local obligations:

- every receipt input is live;
- each owner authorizes;
- input belongs to operation.

Coordinator obligations:

- counts;
- complete output constructor closure;
- same-class relation;
- exact value conservation proof;
- sponsor boundary;
- no closed-asset escape.

### 9.3 `announce-maturity`

Likely layout:

```text
inputs:
    STATE
    optional sponsor suffix

outputs:
    STATE successor
    optional sponsor change
```

Coordinator:

```text
STATE
```

Obligations:

- predecessor STATE authentication;
- successor reconstruction;
- lead bounds;
- operator authorization;
- unchanged state fields;
- no RESV family;
- sponsor isolation.

### 9.4 `burn`

Likely layout:

```text
inputs:
    contiguous live receipt range
    optional sponsor suffix

outputs:
    exactly one ASH
    bounded live receipt change range
    optional sponsor change
    bounded burn-record data-output range
```

Likely coordinator:

```text
first live receipt input
```

Obligations:

- live-only receipt family;
- every owner authorizes;
- exactly one fresh ASH;
- record ordinal continuity;
- change family closure;
- exact value relation;
- sponsor boundary;
- burn event projection shape;
- no root or ASH input.

### 9.5 `receipt-relabel`

A likely initial positional proof strategy is:

```text
input 0:
    STATE

following inputs:
    contiguous time-locked receipt range

output 0:
    byte/semantic STATE successor

following outputs:
    one live receipt per time-locked input, same order

final optional output:
    sponsor change
```

This target layout may be stricter than the model's multiset safety property.

The model relation remains owner/value multiset preservation. The backend layout
is evidence strategy.

### 9.6 `settle-distribution`

No final layout is accepted by this decision.

The layout remains governed by
[`../research/settlement-layout.md`](../research/settlement-layout.md).

Only the requirements are fixed:

- one control;
- vault iff required;
- bounded entitlement family;
- exact owner/class/value routing;
- per-entitlement floors;
- continuing or terminal branch;
- sponsor isolation;
- complete obligation placement;
- permissionless construction.

The concrete ABI is selected only after the batch-size-2 prototype.

---

## 10. Alternatives considered

### 10.1 Let wallets infer layout from emitted script

#### Proposal

Publish target scripts and let each wallet or vector harness inspect them to
determine:

- input/output positions;
- witness order;
- supported branches;
- metadata encoding.

#### Advantages

- no separate ABI artifact;
- script remains the apparent source of truth;
- fewer types and schemas.

#### Rejection

Reverse-engineering target programs is:

- backend-specific;
- error-prone;
- difficult to validate;
- unsuitable for canonical transaction construction;
- hostile to independent implementers;
- likely to create multiple undocumented conventions.

The emitted program enforces the ABI. It should not be the only documentation
of how to construct it.

### 10.2 Maintain handwritten operation builders

#### Proposal

Write one transaction builder per operation directly from the realization
document and target scripts.

#### Advantages

- straightforward;
- natural Rust APIs;
- avoids general layout types;
- fast for early prototypes.

#### Rejection as the permanent design

Handwritten builders would duplicate:

- family order;
- cardinalities;
- optional branches;
- witness order;
- sponsor policy;
- metadata encoding.

The vector harness and calibration runner would likely create further copies.

Operation-specific convenience APIs may be handwritten over a generated typed
ABI, but they must not independently define transaction shape.

### 10.3 Allow any semantically equivalent layout

#### Proposal

Accept any transaction arrangement satisfying the abstract relation.

#### Advantages

- maximum client flexibility;
- avoids backend ABI rigidity;
- closer to behavioural conformance;
- may enable better wallet optimization.

#### Rejection for the initial tapscript backend

A loopless script cannot generally recognize arbitrary permutations and
partitions without greater complexity.

Allowing many equivalent layouts enlarges:

- target code;
- witness complexity;
- vector space;
- ambiguity;
- mixed-branch surface;
- calibration surface.

The initial backend chooses one canonical layout per operation.

A future backend may support a different or more flexible layout under a
separate ABI.

### 10.4 Encode the layout in architecture

#### Proposal

Add concrete positions and coordinator roles to
`architecture::OperationSpec`.

#### Advantages

- one source of truth;
- strong architecture binding;
- easy compiler access.

#### Rejection

Concrete layout is a target/backend implementation choice.

Putting it in architecture would:

- leak tapscript constraints upward;
- move behavioural hashes for implementation-only changes;
- obstruct multiple backends;
- confuse semantic cardinality with target position.

The architecture retains family and bound declarations. Layout is derived
below it.

### 10.5 Put layout only in the backend

#### Proposal

Have tapscript emission choose positions internally without exposing a typed
artifact.

#### Advantages

- smaller public API;
- easy local implementation;
- no separate transaction schema.

#### Rejection

The linker, transaction builder, vectors, calibration, wallets, and release
must all agree with the choice.

An unexported backend convention would become hidden cross-package state.

### 10.6 Duplicate all global checks on every input

#### Proposal

Avoid coordinator selection by making each input leaf verify the complete
transaction relation.

#### Advantages

- simple correctness argument;
- every input independently safe;
- no coordinator spoofing;
- fewer placement choices.

#### Rejection as the default

It can multiply:

- script bytes;
- witness bytes;
- expensive arithmetic;
- constructor reconstruction;
- crypto operations;
- execution cost.

Some local checks should be duplicated. Global checks require explicit
placement and measured policy.

### 10.7 Put all checks on one coordinator

#### Proposal

Make one input verify the entire operation while other inputs perform minimal
or no checks.

#### Advantages

- one global script;
- reduced duplication;
- clear relation attribution.

#### Rejection as a universal rule

Some facts are authenticated locally by the input being spent, especially:

- owner authorization;
- input constructor;
- local class;
- local metadata;
- local lifecycle leaf.

The accepted approach supports local obligations plus global coordinator
obligations.

### 10.8 Use witness counts without inspecting ranges

#### Proposal

Trust count witnesses after checking only bounds.

#### Advantages

- small scripts;
- easy index arithmetic;
- simple builders.

#### Rejection

Counts are attacker-controlled and do not classify transaction slots.

Constructor and range checks are mandatory.

### 10.9 Treat the executable model output order as the ABI

#### Proposal

Reuse `TxBuilder` output order directly.

#### Advantages

- fewer representations;
- easy vector generation;
- current model already has output references.

#### Rejection

The model's internal order is not guaranteed to be the most enforceable or
stable target layout.

It may also be a convenience of one executable implementation rather than an
abstract relation.

Model order may inform or coincide with the target ABI, but it does not become
authoritative automatically.

### 10.10 Make `transaction` test-only

#### Proposal

Keep construction helpers inside `vectors` until wallets are implemented.

#### Advantages

- fewer initial packages;
- faster first backend test;
- avoids premature public API.

#### Rejection

Calibration, vectors, release, genesis tooling, and future clients all need the
same ABI.

A test-only builder would likely become an undocumented de facto public
interface.

The transaction crate may begin narrowly, but it is designed as first-class
deployment infrastructure.

---

## 11. Assurance and evidence consequences

### 11.1 ABI conformance tests

For every operation layout, tests must establish:

- family positions/ranges match the typed ABI;
- actual transaction counts match proposed counts;
- every protocol slot authenticates the declared constructor;
- ranges are disjoint;
- required families are complete;
- undeclared protocol outputs reject;
- optional families obey activation conditions;
- coordinator selection is canonical;
- witness order matches target program expectations;
- sponsor region is isolated.

### 11.2 Layout mutation vectors

Required mutation classes include:

- shift a family start index;
- alter a family count;
- swap two family ranges;
- place wrong constructor in one slot;
- insert undeclared output;
- omit required output;
- duplicate output-family claim;
- move sponsor change into protocol range;
- move protocol output into sponsor range;
- select wrong coordinator;
- use wrong target leaf/program;
- reorder witness items;
- omit witness item;
- add noncanonical optional output;
- alter transaction-level count/version/sequence.

Each mutation should identify the relations affected.

### 11.3 Relation carrier coverage

The coverage report must connect:

```text
semantic relation
    ↓
placement
    ↓
target program
    ↓
ABI family/slot
    ↓
positive and negative vector
```

A green transaction test without this chain is insufficient for relation-level
coverage.

### 11.4 Wallet construction evidence

For permissionless operations, a constructor test must demonstrate that the
ABI can be satisfied using:

- public chain facts;
- public ABI/bundle data;
- public openings where required;
- constructor-local sponsor data.

The test must not use hidden fixture state unavailable to a real arbitrary
constructor.

### 11.5 Calibration evidence

The resource report must identify:

- ABI identity;
- bundle identity;
- operation;
- branch/activation case;
- family counts;
- witness sizes;
- selected representation;
- target program;
- transaction weight;
- per-input resource cost;
- stack limits;
- crypto budget;
- target policy result.

A maximum-bound report must be produced from a valid ABI-conforming
transaction.

### 11.6 Independent implementer vectors

The canonical ABI publication should include or link canonical construction
vectors sufficient for an independent implementation to verify:

- metadata encoding;
- family order;
- witness encoding;
- target program selection;
- transaction bytes or semantic fixture;
- expected target verdict.

This improves interoperability without making the publication a first-party
semantic input.

### 11.7 Bundle/ABI release checks

Release must require:

- bundle identity matches ABI;
- target identity matches;
- architecture and realization identities match;
- compiler/backend configuration matches;
- calibrated bounds match;
- all required operations are represented;
- layout schema supported;
- witness schema supported;
- canonical ABI hash verifies;
- vectors reference the same ABI.

---

## 12. Determinism and identity consequences

### 12.1 ABI identity domain

The ABI identity should be domain-separated from:

- architecture semantic hash;
- architecture behavioural hash;
- realization identity;
- compiler configuration identity;
- target identity;
- linked-bundle hash;
- deployment-profile hash.

The ABI identity must bind:

- schema version;
- target identity;
- linked-bundle identity;
- operation layouts;
- witness schemas;
- metadata encodings;
- constructor references;
- calibrated maxima;
- representation requirements;
- canonical ordering.

### 12.2 Deterministic layout selection

Given the same:

- architecture;
- realization;
- target;
- proof plan;
- placement policy;
- backend configuration;
- calibrated bounds;

the selected layout must be identical.

Tie-breaks must be explicit.

Examples:

- family order by stable object/semantic role;
- coordinator by fixed policy;
- relation placement by stable priority;
- optional family order by stable ID;
- witness item order by role and relation ID where semantically valid.

### 12.3 Layout changes

A layout change may move:

- compiler/backend configuration identity;
- linked-bundle identity;
- ABI identity;
- vector-set identity;
- resource report;
- deployment-profile hash.

It need not move:

- architecture identity;
- realization identity;
- semantic relation IDs

when semantic behavior is unchanged.

### 12.4 Canonical publication

The ABI publication must have:

- deterministic key and array ordering;
- no timestamp;
- no host path;
- no random identifier;
- canonical numeric encoding;
- documented trailing-newline policy;
- exact schema;
- unknown-field rejection.

### 12.5 Cryptographic witness randomness

The ABI describes witness type and encoding, not one production witness.

Concrete signatures, rangeproofs, and blinding values may vary.

Canonical test vectors use fixed fixture randomness and are separately marked
non-production.

### 12.6 Layout cache keys

A future cache should bind at least:

```text
architecture identity
realization identity
compiler/backend configuration identity
target identity
proof-plan identity
calibrated bounds
bundle identity where linking is complete
```

It must not key semantic correctness only by a source file timestamp.

---

## 13. Implementation and migration

### 13.1 Phase 1: no concrete ABI yet

The realization pilots declare:

- semantic families;
- cardinalities;
- constructibility;
- representation capabilities;
- relations.

They do not fix transaction indexes.

### 13.2 Phase 2: layout requirements and placement requirements

Compiler analysis produces target-independent requirements such as:

- family must be enumerable;
- one global carrier required;
- each owner-bearing input needs local authorization;
- output closure spans all protocol-capable slots;
- permissionless witness sources must be public;
- relation needs complete family sum.

It does not yet produce final tapscript indexes unless the backend interface
explicitly owns that target lowering.

### 13.3 Phase 3: target layout primitives

The tapscript backend adds:

- fixed and contiguous range lowering;
- target introspection patterns;
- count checks;
- coordinator programs;
- witness schemas;
- resource formulas.

Target prototypes may use synthetic layouts before publication.

### 13.4 Phase 4: first public ABI

`compact-ash` produces the first complete linked transaction ABI.

The initial ABI scope may contain only:

- one target;
- one operation;
- ASH constructor;
- sponsor policy;
- witness layout;
- resource formulas.

The schema should be extensible through versioning, not through unknown generic
fields.

### 13.5 Phase 5: owner-authorized ABI

Live transfer adds:

- repeated owner-bearing input family;
- signatures;
- receipt constructor metadata;
- representation alternatives;
- output family range;
- sponsor suffix.

This phase tests whether the ABI adequately serves wallets and vectors.

### 13.6 Phase 6: STATE ABI

Maturity announcement adds:

- STATE metadata encoding;
- predecessor/successor constructor data;
- operator witness;
- state-field witness/proof items;
- sponsor region;
- target program selection.

The STATE constructor decision must be resolved first.

### 13.7 Later operations

Add operation layouts incrementally.

The ABI schema must not presume settlement's final shape before the batch-2
prototype resolves it.

### 13.8 Calibration orchestration

Avoid a Cargo cycle between linker and transaction.

Use a higher-level runner:

```text
candidate bounds
    ↓
linker candidate bundle
    ↓
transaction worst-case construction
    ↓
target measurement
    ↓
new candidate bounds
```

The final bundle and ABI are generated after calibration converges.

### 13.9 Package publication

When the transaction crate becomes public within the workspace:

- document its trust boundary;
- expose typed operation requests;
- keep raw low-level transaction assembly scoped;
- provide clear errors;
- follow ADR-010 for binaries;
- inherit ADR-011;
- publish canonical ABI/check tools if needed.

### 13.10 Root ADR promotion

Once one release bundle and wallet/vector path depend on the ABI, promote this
decision to an implemented root ADR.

The ADR should reference the actual implemented schema and commands.

---

## 14. Risks and limitations

### 14.1 ABI rigidity

A canonical layout can constrain future optimization and wallet flexibility.

Mitigation:

- layout identity is target/bundle-specific;
- schema is versioned;
- semantic relation remains independent;
- future bundles may use new ABIs;
- clients bind the exact deployment ABI.

### 14.2 Coordinator bottleneck

A coordinator may produce:

- large script;
- large witness;
- concentrated wide arithmetic;
- stack pressure;
- transaction serialization contention.

Mitigation:

- explicit placement analysis;
- local enforcement where possible;
- measured duplication;
- settlement prototype;
- resource calibration.

### 14.3 Local/global inconsistency

Local input programs and coordinator may interpret family boundaries
differently.

Mitigation:

- one typed layout;
- linked relation provenance;
- shared canonical counts;
- cross-input vectors;
- ABI-driven transaction builder.

### 14.4 Optional-family combinatorics

Many optional outputs can create several layouts for one semantic branch.

Mitigation:

- one canonical presence/absence rule;
- canonical ordering;
- conditional relation vectors;
- avoid unnecessary target variants.

### 14.5 Witness ABI evolution

Changing witness ordering can break clients even when semantic script behavior
is unchanged.

Mitigation:

- explicit ABI identity/version;
- bundle binding;
- release notes;
- canonical construction vectors;
- no inference from script.

### 14.6 ABI publication may reveal implementation details

The ABI necessarily reveals target program and transaction construction.

This is acceptable. The system does not rely on obscurity.

Do not include secret witness values.

### 14.7 Multiple backend complexity

A future Simplicity backend may not naturally use the same coordinator or
witness model.

Mitigation:

- ABI is target-specific;
- semantic layout requirements remain above backend;
- concrete `OperationLayout` types may have backend-associated forms;
- do not force Simplicity into tapscript-shaped witnesses.

### 14.8 Calibration circularity

Bounds affect layout size; layout size affects measured bounds.

Mitigation:

- explicit calibration runner;
- monotone search only where justified;
- final remeasurement;
- fail if nonmonotonic coupling invalidates assumptions;
- bind final bounds into bundle and ABI.

### 14.9 Independent implementer burden

A detailed ABI can be complex.

Mitigation:

- typed schema;
- canonical examples;
- operation-specific convenience libraries;
- stable error messages;
- focused first release;
- avoid unnecessary abstraction in publication.

### 14.10 Model/ABI order confusion

Reviewers may mistake target positional rules for abstract semantics.

Mitigation:

- every ABI field carries source provenance;
- package documentation labels implementation-only ordering;
- model conformance checks semantic relations, not target positions;
- versioning review distinguishes denotation from implementation latitude.

---

## 15. Supersession conditions

This decision may be superseded if a future target can enforce the complete
semantic transaction relation without a canonical positional ABI and the
project deliberately supports flexible layouts.

A replacement must establish:

1. how target programs authenticate arbitrary family membership;
2. how global relations are placed;
3. how wallets discover valid construction;
4. how vectors materialize mutations;
5. how calibration constructs worst cases;
6. how bundle and client compatibility are identified;
7. how deterministic release artifacts are produced;
8. how closed-asset output closure remains complete;
9. how permissionless construction remains possible.

It may also be superseded by a formally typed transaction language generated
directly from realization semantics, provided that language fulfills the same
ABI role.

This decision is not superseded merely because:

- one backend uses a different layout;
- an operation gains a new optional family;
- witness order changes under a new ABI version;
- a backend supports more flexible layouts internally;
- a client uses a convenience API;
- a future Simplicity ABI differs from tapscript;
- model output order happens to match target order.

---

## 16. References

### Normative and current source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/architecture/src/spec.rs`](../../packages/architecture/src/spec.rs)
- [`../../packages/architecture/src/validate.rs`](../../packages/architecture/src/validate.rs)
- [`../../packages/model/src/kernel.rs`](../../packages/model/src/kernel.rs)
- [`../../packages/model/src/shape.rs`](../../packages/model/src/shape.rs)
- [`../../packages/model/src/certify.rs`](../../packages/model/src/certify.rs)
- [`../../packages/model/src/manifest.rs`](../../packages/model/src/manifest.rs)
- [`../../packages/model/src/ops/relabel.rs`](../../packages/model/src/ops/relabel.rs)
- [`../../packages/model/src/ops/settlement.rs`](../../packages/model/src/ops/settlement.rs)

### Related decisions

- [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md)
- [D002: Introduce a Target-Independent Realization Layer](002-target-independent-realization-layer.md)
- [D003: Design for Multiple Backends and Implement Elements Tapscript First](003-multiple-backends-tapscript-first.md)
- [D004: Use Per-Bundle Translation Validation Instead of Initially Trusting the Compiler](004-translation-validation-over-compiler-trust.md)
- [D005: Permit Value-Representation Latitude While Keeping Closed Asset Identity Rigid](005-value-parametric-asset-rigid.md)

### Package plans

- [`../packages/compiler.md`](../packages/compiler.md)
- [`../packages/tapscript.md`](../packages/tapscript.md)
- [`../packages/linker.md`](../packages/linker.md)
- [`../packages/transaction.md`](../packages/transaction.md)
- [`../packages/vectors.md`](../packages/vectors.md)
- [`../packages/release.md`](../packages/release.md)

### Research

- [`../research/state-object-constructor.md`](../research/state-object-constructor.md)
- [`../research/public-declassification.md`](../research/public-declassification.md)
- [`../research/settlement-layout.md`](../research/settlement-layout.md)

### Roadmap

- [`../roadmap.md`](../roadmap.md)
- [`../backlog.md`](../backlog.md)

---

## 17. Decision summary

> Derive one deterministic target- and bundle-specific transaction ABI for
> every semantic operation: authenticate concrete input/output family ranges,
> assign every relation to an executable carrier, define canonical optional and
> sponsor regions, publish typed witness and metadata conventions, construct
> wallet/vector/calibration transactions from that ABI, and keep the concrete
> positional layout separate from target-independent protocol semantics.
