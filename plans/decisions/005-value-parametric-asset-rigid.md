# D005: Permit Value-Representation Latitude While Keeping Closed Asset Identity Rigid

> **Status:** ACCEPTED
> **Scope:** Target-independent representation semantics, compiler proof
> planning, permissionless constructibility, target enforcement, and
> representation evidence for the initial Elements backend
> **Decision class:** Representation
> **Applies to:** `realization`, `model` conformance, `compiler`,
> `target-elements`, `tapscript`, future `simplicity`, `linker`,
> `transaction`, `vectors`, and `release`
> **Depends on:** [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md);
> [D002: Introduce a Target-Independent Realization Layer](002-target-independent-realization-layer.md);
> [D003: Design for Multiple Backends and Implement Elements Tapscript First](003-multiple-backends-tapscript-first.md);
> [D004: Use Per-Bundle Translation Validation Instead of Initially Trusting the Compiler](004-translation-validation-over-compiler-trust.md)
> **Supersedes:** none
> **Superseded by:** none
> **Related normative constraints:** the representation-conformance section,
> consensus-value authority, disclosure frontier, permissionless liveness,
> declassification derivation, reader firewall, representation oracle
> obligations, and representation pins in
> `docs/attestation/realization.md`
> **Related research:**
> [`../research/public-declassification.md`](../research/public-declassification.md),
> [`../research/state-object-constructor.md`](../research/state-object-constructor.md),
> [`../research/settlement-layout.md`](../research/settlement-layout.md)
> **Promoted ADR:** none
> **Machine-consumed by the toolchain:** no

---

## 1. Context

The executable model represents every UTXO with one semantic asset and one
semantic value:

```rust
pub struct Utxo {
    pub asset: Asset,
    pub value: Sat,
    pub meta: Meta,
}
```

At the abstract level, a value exists independently of how a concrete target
encodes or proves it.

An Elements transaction may represent a value using:

- an explicit consensus value;
- a confidential value commitment;
- a commitment whose opening is published or otherwise authenticated.

Those encodings may denote the same abstract amount.

The target-independent realization therefore needs to distinguish:

```text
what semantic value exists
```

from:

```text
how one target proves the relation involving that value
```

At the same time, the protocol's closed assets are not arbitrary fungible
sidecar assets. Their identities determine whether an output belongs to the
canonical protocol state.

Closed protocol assets include:

```text
U
ENT
DIST_CTL
PID
PACE
ENT_AUTH
DIST_AUTH
```

A target output that can secretly carry one of these assets without being
classified by the covenant can violate:

- closed-asset closure;
- receipt accounting;
- issuance discipline;
- authority uniqueness;
- root identity;
- lifecycle enforcement;
- public auditability.

Value representation and asset identity therefore require different policies.

### 1.1 Semantic values are not necessarily explicit encodings

Some operations need the numerical value itself.

Examples include:

- cycle issuance;
- redemption payout;
- admission principal;
- settlement allocation;
- clear decrement;
- public STATE assignments;
- public event values.

Other operations need only a relation such as exact preservation or aggregate
conservation.

Examples include:

- live receipt transfer;
- time-locked receipt transfer;
- receipt relabel;
- other lateral value-preserving movement.

A target may establish preservation through:

- explicit arithmetic;
- commitment equality;
- confidential transaction conservation;
- another target proof with equivalent semantic force.

Requiring explicit values at every seam may be safe but disclose more than the
semantic relation needs.

### 1.2 Closed asset identity has a different failure mode

Suppose a transfer consumes explicit `U` but permits an output whose
confidential asset commitment is not classified.

That output could secretly carry `U`. Target consensus may accept its asset
surjection proof because `U` is among the transaction inputs.

The target transaction can therefore move closed `U` into an output that the
protocol cannot recognize as:

- receipt;
- ASH;
- distribution vault;
- destruction;
- another declared `U` destination.

This breaks exact object closure even if the value commitments balance.

An induction from explicit genesis does not prevent the first such hidden
escape. The transaction creating the confidential asset output is itself the
escape.

The initial backend therefore cannot treat asset confidentiality as analogous
to value confidentiality.

### 1.3 Public availability differs from explicit encoding

Some semantic values must be publicly usable by future permissionless
constructors.

For example:

- ASH must be compactable and clearable by anyone;
- admission must be constructible from public request facts;
- settlement must be constructible without entitlement-owner secrets;
- public STATE values must support future state transitions;
- public event values must be independently auditable.

A publicly usable value might be encoded as:

- an explicit value;
- a public commitment together with an authenticated public opening.

The semantic requirement is public authenticated availability, not necessarily
one exact wire representation.

This distinction matters when a confidential input is fully consumed.

An explicit output has no value-blinding term. The confidential input's
residual blinding contribution may need to be routed into another commitment
for the transaction's confidential-value balance to remain valid.

A public commitment with a published opening may therefore be required at a
confidential-to-public synchronization seam.

The exact target pattern remains prototype-dependent.

### 1.4 Verification differs from constructibility

A script may be able to verify an opening that only an owner knows.

That does not make the operation permissionless.

A permissionless operation must be constructible by an arbitrary party using:

- public chain data;
- public openings or proof capsules;
- the constructor's own sponsor funds.

The representation plan must therefore account for witness availability, not
merely target verification capability.

### 1.5 Safety differs from disclosure minimality

A fully explicit backend may correctly enforce every semantic transition. It
can still reveal values that the semantic relation does not require.

Conversely, accepting confidential representations does not by itself prove
safety. A backend might preserve privacy while allowing:

- wrong asset identity;
- an unclassified output;
- wrong recipient;
- missing owner authorization;
- hidden protocol-value escape.

The project needs two separate evidence axes:

1. **safety**
   Every accepted concrete transaction denotes an authorized semantic
   transition.

2. **disclosure minimality**
   The backend does not require unnecessary disclosure when a supported
   lower-disclosure proof establishes the same relation.

Neither axis subsumes the other.

---

## 2. Decision

The target-independent realization treats value as a semantic fact whose
concrete representation may vary when the selected target proof establishes the
same relation.

The initial Elements deployment is:

> **value-representation-parametric and closed-asset-identity-rigid.**

The central decisions are:

1. **Semantic value and target representation are distinct.**
   An abstract amount is not defined by whether it is encoded explicitly or as
   a commitment.

2. **The realization may permit several value-representation modes.**
   The initial conceptual modes are:
   - `PrivateCommitted`;
   - `PublicCommitted`;
   - `Explicit`.

3. **Representation modes form disclosure/availability classes, not distinct
   economic values.**
   Changing representation must not change the semantic amount, authorization,
   recipient, lifecycle, or public projection except for explicitly declared
   disclosure.

4. **Closed protocol asset identity remains explicit at every initial Elements
   protocol seam.**
   No confidential or unclassified asset output may silently carry a closed
   protocol asset.

5. **Open/sponsor value may remain confidential where the semantic relation
   does not read it and target consensus plus authorization and output closure
   preserve the intended flow.**

6. **The initial sponsor asset remains explicitly L-BTC.**
   This decision does not authorize arbitrary foreign or confidential sponsor
   assets.

7. **Public semantic values may use explicit or publicly authenticated
   committed representation.**
   The selected target representation must remain publicly constructible and
   auditable where required.

8. **Permissionless lifecycle paths may not require private witness material
   unavailable to arbitrary constructors.**

9. **Declassification is derived from typed semantic dependencies.**
   Representation policy does not come from an authored
   `declassification.json`.

10. **Safety and disclosure minimality receive separate compiler checks and
    separate evidence.**

11. **A target backend may support only a subset of representation modes.**
    Unsupported modes fail compilation or require an explicitly declared
    normalization path. They do not weaken semantic relations.

12. **Representation choices do not change protocol denotation when they
    preserve the same target-independent state, transition, invariant,
    observable, and constructibility relation.**
    A representation change that alters an abstract capability or public
    observable requires normative versioning review.

The exact concrete proof patterns are selected by compiler/backend policy and
bound to target and bundle identities.

---

## 3. Representation vocabulary

### 3.1 Value representations

The initial target-independent representation classes are conceptually:

```rust
pub enum ValueRepresentation {
    PrivateCommitted,
    PublicCommitted,
    Explicit,
}
```

> Illustrative vocabulary; exact Rust type names are not frozen by this
> decision.

Their intended meanings are:

| Mode | Amount publicly known | Commitment algebra retained | Typical use |
|---|---:|---:|---|
| `PrivateCommitted` | no | yes | Owner-controlled lateral receipt value or sponsor value. |
| `PublicCommitted` | yes, through a published authenticated opening | yes | Public synchronization where residual blinding must remain represented. |
| `Explicit` | yes, directly encoded | no value-blinding term | Public state, public workflow values, arithmetic seams where direct introspection is selected. |

These modes do not define different abstract `Sat` values.

### 3.2 Asset representations

The initial conceptual target vocabulary may distinguish:

```rust
pub enum AssetRepresentation {
    Explicit,
    Confidential,
}
```

> Illustrative vocabulary; not frozen.

For the initial Elements backend:

```text
closed protocol asset seam:
    Explicit required

ordinary sponsor/open asset seam:
    explicit L-BTC identity required by current profile
```

A later deployment may add a target proof for confidential asset
classification only through a new accepted decision and deployment evidence.

### 3.3 Metadata representations

Metadata visibility is relation-specific.

Examples include:

- receipt owner;
- receipt class;
- request pool ID;
- refund key;
- receipt owner destination;
- request principal;
- entitlement target cycle;
- distribution counters;
- maturity state;
- burn-record address and amount.

The realization should describe semantic availability and public observability,
not one final byte encoding.

Concrete metadata commitments and openings are backend and transaction-ABI
concerns.

### 3.4 Representation requirements

A semantic seam may declare requirements such as:

```text
must be public
may remain private
must preserve commitment exactly
must support public opening
must support permissionless reconstruction
must support owner-authorized normalization
asset identity must be explicit
```

These requirements are distinct from the backend's selected proof method.

---

## 4. Required consequences

### 4.1 The realization records semantic representation latitude

For each relevant fact or object family, the realization must be able to state:

- semantic value domain;
- whether numerical value is read by the operation;
- whether value is part of a public observable;
- whether future permissionless construction needs the value;
- whether exact commitment preservation is acceptable;
- whether aggregate confidential conservation is acceptable;
- whether authenticated public opening is acceptable;
- whether normalization is allowed;
- whether closed asset identity must remain explicit;
- required lifecycle exits.

The realization must not select one target opcode or confidential transaction
construction.

### 4.2 The compiler derives disclosure demand

The compiler derives disclosure from the semantic dependency graph.

A value becomes public when at least one demand applies:

```text
D_state:
    value determines a public state assignment

D_live:
    a permissionless constructor needs the value or opening

D_audit:
    value enters a public event, interface, or audit projection
```

The exact analysis may refine these categories, but every disclosure must have
typed provenance.

The compiler must reject:

- a public disclosure with no accepted reason when minimality policy requires a
  supported less-disclosing proof;
- a private value required by unsupported arithmetic;
- a permissionless path requiring an unavailable private opening;
- a public event whose target representation is not publicly authenticated;
- a lifecycle path made unreachable by representation.

### 4.3 The compiler represents proof alternatives

For a relation such as value conservation, the compiler may retain alternatives:

```text
explicit arithmetic
confidential transaction conservation
```

For exact value preservation:

```text
explicit equality
commitment equality
```

For a value needed later in public arithmetic:

```text
explicit at rest
authenticated opening at spend
public-committed synchronization
owner-authorized normalization
```

An alternative is selected only when:

- the target supports it;
- witness availability is sufficient;
- lifecycle remains reachable;
- resource limits permit it;
- deployment policy allows it;
- disclosure policy is satisfied.

### 4.4 The target package exposes exact capabilities

The typed Elements target must separately advertise capabilities such as:

- explicit asset inspection;
- explicit value inspection;
- confidential value conservation;
- value-commitment equality;
- authenticated value opening, if a concrete verified pattern exists;
- constructor/program verification;
- target rangeproof and transaction-construction support.

The target must not advertise `AuthenticatedValueOpening` merely because low
level EC operations exist. A complete target proof pattern and evidence are
required.

### 4.5 Closed asset identity is checked at every protocol seam

For the initial Elements backend, every input and output capable of carrying a
closed protocol asset must be classified.

The backend must prevent:

- confidential asset output secretly carrying `U`;
- confidential asset output secretly carrying `ENT`;
- confidential asset output secretly carrying `DIST_CTL`;
- confidential root/authority asset output;
- foreign asset replacing a closed protocol asset;
- closed asset entering an undeclared object family;
- closed asset leaving through sponsor change or unclassified output.

The exact closure proof may be distributed across:

- explicit asset checks;
- operation-family layout;
- canonical output classification;
- exact flow/issuance partition;
- target confidential transaction semantics.

No unclassified output is presumed harmless when it could carry a closed asset.

### 4.6 Consensus-value authority remains representation-neutral

The invariant that consensus value is authoritative does not require every
target value to be explicit.

It requires:

- one semantic value;
- no independent metadata scalar that overrides it;
- any opening or commitment proof binds to the consensus-enforced value;
- public arithmetic reads an authenticated value;
- equality/conservation claims use an approved target proof;
- metadata partitions exhaust the actual semantic source value.

Therefore:

```text
explicit consensus value:
    acceptable when selected

binding confidential commitment:
    acceptable for relations that do not require public numerical arithmetic

metadata face value unrelated to either:
    prohibited
```

### 4.7 Safety and minimality analyses are separate

The compiler should produce conceptually separate results:

```text
SafetyPlan
DisclosurePlan
```

or equivalent typed distinctions.

Safety asks:

- which semantic relations must be enforced;
- which asset/object closures must hold;
- which authorization and recipient relations apply;
- which target proof establishes each relation.

Minimality asks:

- which facts are disclosed;
- why each disclosure is required;
- whether a supported lower-disclosure proof exists;
- whether representation metamorphisms preserve acceptance.

A compiler must not mark a plan safe merely because it is private, nor minimal
merely because it is safe.

### 4.8 Evidence is two-sided

For each **required public** seam:

- blind or hide the required fact without an approved proof;
- require compiler planning failure or target rejection;
- test malformed or mismatched public openings;
- compare public projection.

For each **confidentiality-permitted** seam:

- construct a lower-disclosure valid representation;
- require target acceptance;
- require the same abstract successor;
- require the same public semantic projection.

For each **closed asset** seam:

- try confidential/unclassified asset substitution;
- require rejection independently of value confidentiality.

### 4.9 Permissionless constructibility is representation-sensitive

For each permissionless operation, the realization/compiler must prove that
required target witnesses are available from:

- public chain data;
- public opening;
- deployment constant;
- constructor-local sponsor data.

A permissionless operation must not require:

- receipt-owner value opening;
- entitlement-owner secret;
- operator secret;
- private blinding factor retained by the previous creator;
- unpublished rangeproof witness;
- inaccessible off-chain metadata.

A representation that violates this requirement is unsupported even when the
script could verify it if the secret were supplied.

### 4.10 Representation lifecycle analysis is required

For every supported object representation, the compiler must verify a path to
each required semantic lifecycle exit.

Examples:

```text
LiveReceipt<PrivateCommitted>
    → transfer
    → burn or redeem through supported declassification/normalization

TimeLockedReceipt<representation>
    → transfer
    → permissionless relabel after maturity

Ash<publicly usable representation>
    → compact
    → clear

DepositRequest<admissible representation>
    → cancel or permissionless admission

Entitlement<representation>
    → permissionless settlement
```

A representation that strands the object is not supported merely because one
creation transaction can produce it.

### 4.11 Normalization is explicit semantic/toolchain policy

A target may support a private object by requiring an authorized normalization
before an operation needing public arithmetic.

For example:

```text
private live receipt
    → owner-authorized self-transfer preserving semantic value
    → public or publicly opened live receipt
    → redeem
```

Normalization must be:

- declared as a lifecycle path;
- authorized correctly;
- non-value-changing;
- constructible by the authorized party;
- represented in transaction ABI;
- covered by vectors;
- visible to clients.

It must not be an undocumented wallet convention.

### 4.12 Sponsor policy remains conservative

For the initial backend:

```text
sponsor asset:
    explicit L-BTC

sponsor value:
    may be confidential where supported

sponsor change asset:
    explicit L-BTC

sponsor change value:
    may be confidential where supported
```

Sponsor authorization must commit the required output set.

Sponsor value must not alter:

- formula-bound payout;
- request refund;
- reserve successor;
- admission principal;
- issuance amount;
- canonical closed-asset conservation.

This decision does not authorize arbitrary foreign sidecars.

### 4.13 Public committed values require authenticated openings

A `PublicCommitted` value is not merely a commitment accompanied by an
unauthenticated number in metadata.

The public opening must be bound to the commitment through a target proof or
consensus-enforced relation accepted by the backend.

Future permissionless constructors must be able to obtain and verify the
opening.

The opening format and target proof remain under
[`../research/public-declassification.md`](../research/public-declassification.md).

### 4.14 Release binds supported representation modes

A deployment release must state and bind:

- representation modes supported per object/operation seam;
- target capabilities used;
- proof patterns used;
- normalization paths;
- public-opening format where applicable;
- representation vector report;
- target dependency evidence.

A green model alone cannot establish these target representation claims.

---

## 5. Initial operation classification

The table below records the planned initial representation posture. It is an
implementation guide and must remain subordinate to the typed realization and
accepted prototype results.

| Operation | Value facts required publicly by semantics | Confidentiality latitude |
|---|---|---|
| `create-request` | Request facts needed by the selected client/admission profile; malformed private requests may remain inert. | Client funding/change value may be confidential where request construction and later lifecycle remain valid. |
| `cancel-request` | Full-value refund relation and refund destination. | A shape-restricted confidential refund may be possible, but is not assumed until explicitly implemented and tested. |
| `admit-deposits` | Principal, service-budget partition, active-backing update, entitlement values. | Initial profile is public/explicit or publicly opened at admission. |
| `cycle` | STATE, reserve carry, issuance, class split, fee split, control/vault values. | Protocol values are public in the initial profile; sponsor value may remain confidential. |
| `settle-distribution` | Entitlement principal, control counters, class allocations, vault value, receipt allocations. | Initial profile is public; private entitlement arithmetic is deferred. |
| `transfer-live-receipts` | No economy-determining numerical value must be published by the abstract lateral relation. | Private committed values are intended to be supported using CT conservation. |
| `transfer-time-locked-receipts` | Same lateral relation as live transfer. | Value privacy is semantically allowed, but support depends on permissionless relabel lifecycle. |
| `redeem` | Receipt amount `x`, payout `p`, and public state delta. | Receipt may be private while held; spend requires authenticated declassification or normalization. |
| `receipt-relabel` | Owner, class transition, exact value preservation, maturity state. | Commitment equality may preserve amount privacy, subject to permissionless rangeproof/constructor feasibility. |
| `burn` | Fresh ASH aggregate and public burn records; public accounting/event facts. | Source receipt and live-change amounts may remain private if CT balance and public ASH synchronization are valid. |
| `compact-ash` | ASH values must be publicly usable by arbitrary constructors. | No private opening dependency is allowed. |
| `clear` | ASH aggregate, public state decrement, residual ASH. | ASH must be explicit or publicly committed with authenticated public opening. |
| `announce-maturity` | Public STATE fields and maturity schedule. | Sponsor value may remain confidential; state values are public. |

This table does not freeze the concrete target encoding. Package plans and
research decisions will refine implementation support.

---

## 6. Prohibited consequences

### 6.1 No confidential closed asset identity in the initial backend

The initial Elements backend must reject:

- confidential asset commitments at closed protocol seams;
- unclassified outputs that may carry a closed asset;
- sponsor outputs that may conceal closed protocol assets;
- confidential root or authority identities;
- a proof plan that relies on off-chain assumptions to classify a closed asset.

Supporting confidential closed asset identity later requires:

- a complete target proof of asset class;
- object-closure integration;
- constructibility analysis;
- safety vectors;
- target evidence;
- a new accepted decision or superseding record.

### 6.2 No blanket “all values confidential” policy

The backend must not blind every value by default.

Values required by public state, public event, public audit, or permissionless
construction must remain publicly and authentically available.

Privacy does not override liveness or auditability.

### 6.3 No blanket “all values explicit” policy when minimality is claimed

A deployment may initially choose a more explicit representation profile when
that profile remains semantically correct and the release claim says so.

However, the compiler must not claim disclosure minimality for an operation if:

- a supported lower-disclosure proof exists;
- policy requires it;
- the emitted program unnecessarily rejects the lower-disclosure form.

Safety and minimality claims must remain accurate.

### 6.4 No unauthenticated metadata opening

A metadata field such as:

```text
value = 100
```

does not prove that a confidential commitment carries value 100.

Any public opening must be target-authenticated.

### 6.5 No owner-secret dependency on permissionless paths

The following cannot require an owner's private value opening in the accepted
permissionless profile:

- admission;
- delayed cycle;
- settlement;
- receipt relabel;
- ASH compaction;
- clear.

If a private representation makes one path owner-dependent, either:

- add a valid public capsule/opening;
- add a semantics-preserving normalization before the permissionless state;
- reject that representation;
- or perform normative review if changing permissionless behavior is proposed.

### 6.6 No hidden representation-driven semantic change

A backend representation must not change:

- accepted abstract operation;
- economic formula;
- recipient;
- owner authorization;
- state assignment;
- event value;
- audit quantity;
- lifecycle capability.

For example, making private time-locked receipts owner-assisted at relabel would
remove a permissionless abstract capability and is not an implementation-only
choice.

### 6.7 No conflation of public commitment with privacy

A `PublicCommitted` value has a commitment representation but a publicly known
amount.

It must not be described as amount-private.

Its benefit may be:

- routing residual blinding;
- preserving commitment algebra;
- supporting later target relations.

### 6.8 No conflation of asset and value confidentiality

A value commitment does not imply an asset commitment, and an asset commitment
does not imply a value commitment.

Compiler facts, target capabilities, vectors, and reports must track the axes
separately.

### 6.9 No confidential target fact without lifecycle analysis

The compiler must not support a private representation merely because its
creation and immediate transfer work.

Required exits must also work.

### 6.10 No authored declassification table

The target backend must not decide disclosure by maintaining an operation-name
allowlist such as:

```text
burn reveals ASH
redeem reveals amount
transfer reveals nothing
```

as an independent policy source.

Those results must derive from typed realization dependencies. Target mappings
may implement the derived result and must be validated for complete coverage.

### 6.11 No privacy overclaim

User-facing documentation must not imply that all protocol amounts are
confidential.

The initial target is expected to expose many protocol values because of:

- public state;
- public workflow;
- public attestation;
- permissionless maintenance;
- direct arithmetic requirements.

Any privacy claim must name the exact object, operation, representation mode,
and target proof.

---

## 7. Safety analysis

### 7.1 Primary refinement property

For a target representation relation `R_T(c, s)`, target acceptance must imply
an authorized abstract successor:

```text
R_T(concrete_before, semantic_before)
and
target accepts concrete transaction
    ⇒
there exists semantic_after such that:
    semantic operation accepts
    semantic invariant holds
    R_T(concrete_after, semantic_after)
```

Representation latitude changes `R_T`; it does not weaken the semantic
transition.

### 7.2 Closed-asset output closure

For each operation touching a closed asset, the concrete transaction must
classify every output capable of carrying that asset.

The target proof must establish exact membership in declared categories such
as:

```text
U:
    live receipt
    time-locked receipt
    ASH
    distribution vault
    declared destruction
    authorized issuance destination

ENT:
    deposit entitlement
    declared destruction

DIST_CTL:
    distribution control
    declared destruction

root/authority assets:
    exact declared successor or terminal rule
```

No residual “other output” category may carry a closed asset.

### 7.3 Confidential value conservation

Where selected, confidential transaction conservation may establish a semantic
value equation only when:

- relevant asset identities are classified;
- input/output families are closed;
- issuance and destruction are separately accounted;
- target CT semantics are typed in the target contract and
  deployment-tested;
- sponsor/open flows cannot absorb closed value;
- public state/event deltas are separately pinned.

CT balance alone does not prove recipient or object closure.

### 7.4 Commitment equality

Commitment equality may establish exact value preservation for a local
relation, such as receipt relabel, only when:

- commitments are for the same explicitly classified asset;
- owner and class metadata satisfy the semantic transition;
- the successor constructor is authenticated;
- no additional value-bearing outputs escape;
- target commitment-equality semantics are tested;
- permissionless construction remains possible.

### 7.5 Authenticated opening

An authenticated opening may establish a numerical value when:

- the proof binds to the exact consensus commitment;
- the asset is correctly classified;
- the opening is canonical;
- range/domain checks apply;
- the witness is available to the authorized constructor;
- future lifecycle requirements are satisfied.

A target package advertises this capability only after a concrete proof pattern
exists.

### 7.6 Normalization

Normalization is safe only when it:

- preserves semantic value;
- preserves asset identity;
- preserves owner unless the owner authorizes change;
- changes only representation or explicitly allowed metadata;
- has exact object closure;
- does not bypass lifecycle restrictions;
- is separately represented and tested.

---

## 8. Disclosure minimality analysis

### 8.1 Minimality criterion

A selected proof plan is minimally demanding under one target and deployment
policy when no supported alternative:

- establishes the same semantic relations;
- preserves required constructibility and lifecycle;
- satisfies resources and policy;
- reveals strictly less semantic information.

This is target- and policy-relative.

It is not a universal claim that one representation is the minimum possible
under every cryptographic system.

### 8.2 Disclosure provenance

Every disclosed fact must record one or more reasons, such as:

```text
PublicStateDependency
PermissionlessConstructibility
PublicAuditDependency
TargetSafetyRequirement
DeploymentPolicyOverride
```

> Illustrative vocabulary; not frozen.

`TargetSafetyRequirement` should be used carefully. For example, explicit
closed-asset identity is required for initial backend safety, even when asset
identity is not an amount disclosure.

A policy override may select a more explicit safe mode, but the release must
not then claim semantic minimality for that seam.

### 8.3 Representation metamorphisms

For relations that should be representation-independent, evidence should
transform one valid concrete representation into another while preserving:

- semantic input;
- semantic operation;
- authorization;
- semantic successor;
- public projection.

Examples:

- explicit live-receipt transfer → confidential-value transfer;
- one valid commitment blinding → another valid blinding;
- explicit sponsor value → confidential sponsor value;
- equivalent commitment-preserving relabel, if supported.

### 8.4 Minimality failure classes

The compiler/evidence system should distinguish:

- target lacks lower-disclosure capability;
- target supports capability but policy disables it;
- proof exceeds resource limits;
- witness unavailable;
- lifecycle path fails;
- backend unnecessarily emits a reveal;
- backend unnecessarily rejects a valid lower-disclosure representation.

These have different consequences and should not collapse into “privacy
unsupported.”

---

## 9. Alternatives considered

### 9.1 Require every protocol value to be explicit

#### Proposal

Treat every protocol input and output value as explicit, regardless of whether
the semantic relation performs arithmetic on it.

#### Advantages

- simplest compiler and transaction builder;
- straightforward public audit;
- direct script arithmetic;
- easier debugging;
- no CT proof-planning complexity;
- no rangeproof construction for protocol objects.

#### Rejection as the universal architecture

It unnecessarily constrains lateral value-preserving relations such as receipt
transfer and potentially relabel.

It also prevents the compiler from expressing target-independent representation
latitude already permitted by the realization's representation discipline.

An initial deployment may still choose explicit values at selected seams, but
the semantic and compiler architecture must not define explicit encoding as
the abstract value itself.

### 9.2 Permit confidential asset identity everywhere

#### Proposal

Use Elements asset commitments for closed protocol assets and rely on
surjection proofs plus confidential transaction balance.

#### Advantages

- stronger asset privacy;
- more uniform CT usage;
- potentially hides protocol holdings.

#### Rejection for the initial backend

A confidential asset output can hide `U`, `ENT`, `DIST_CTL`, or an authority
asset in an unclassified object.

The current target design has no complete proof that an arbitrary confidential
asset commitment belongs to one declared protocol class while preserving exact
object closure.

This would create a hidden closed-asset escape.

### 9.3 Treat all confidential outputs as inert open objects

#### Proposal

Declare any confidential-asset output inert and outside protocol state.

#### Advantages

- aligns with open-object immunity;
- simple recognition rule;
- appears to avoid classifying confidential assets.

#### Rejection

An inert confidential output may secretly carry a closed protocol asset
obtained from a protocol input. Calling it inert does not undo the loss from
canonical accounting.

Open-object inertness is safe only for genuinely open assets, not an
unclassified output that may contain a closed asset.

### 9.4 Derive asset closure from explicit genesis alone

#### Proposal

Because genesis creates closed assets explicitly, assume confidential closed
assets cannot appear later.

#### Advantages

- avoids checking every output asset representation;
- simple induction argument.

#### Rejection

The first accepted transaction that creates an unclassified confidential asset
output from a closed asset input breaks the induction.

The transaction is the origin of the hidden asset.

### 9.5 Permit private values wherever script can verify them

#### Proposal

If a target program can verify an opening or proof, permit the private
representation.

#### Advantages

- maximizes target privacy;
- simple capability rule;
- uses cryptographic proofs directly.

#### Rejection

Script verifiability does not imply permissionless constructibility.

If only the previous owner knows the opening, a permissionless settlement,
relabel, compaction, or clear may become impossible.

Witness availability and lifecycle are independent requirements.

### 9.6 Require public values to use explicit wire encoding

#### Proposal

Whenever a value must be public, require Elements explicit encoding.

#### Advantages

- direct introspection;
- simplest future construction;
- no opening format;
- no ambiguity about visibility.

#### Rejection as the only semantic option

A confidential input that is fully consumed may need to route residual
blinding into a value commitment. A public commitment with authenticated
opening can keep the value public while preserving CT balance.

The exact target pattern remains research-dependent, so the semantic layer
should require public authenticated availability rather than hardcode one wire
encoding.

### 9.7 Accept owner-assisted permissionless maintenance

#### Proposal

Allow private receipt or entitlement owners to provide openings when
permissionless maintenance is needed.

#### Advantages

- easier confidential representations;
- owners already possess relevant secrets;
- avoids public openings.

#### Rejection

This changes the capability from permissionless to owner-assisted.

Lost or uncooperative owners could block shared-state cleanup, violating the
current realization's liveness and no-ransom relation.

### 9.8 Allow arbitrary sponsor sidecar assets

#### Proposal

Permit any sponsor-owned open asset input/output pair as long as protocol
closed assets and formula-bound outputs remain correct.

#### Advantages

- flexible transaction composition;
- potentially better wallet privacy;
- avoids unnecessary sponsor asset restrictions.

#### Rejection for the initial profile

The current architecture declares sponsor flow through plain L-BTC. Broadening
to arbitrary sidecars changes the accepted concrete relation and complicates
closure analysis.

A future `OpenSidecar` policy may be considered through a separate decision and
normative compatibility review.

### 9.9 Make privacy a protocol guarantee

#### Proposal

Require confidential representation for supported holdings and transfers.

#### Advantages

- stronger user-facing privacy commitment;
- avoids explicit-value fallback;
- clearer deployment promise.

#### Rejection

The current realization prioritizes public auditability and permits multiple
representation modes. Many protocol values are intentionally public.

The project can support privacy where semantically and operationally sound
without claiming universal confidentiality.

---

## 10. Assurance and evidence consequences

### 10.1 Safety report

The safety report must include:

- explicit closed-asset identity checks;
- exact output-family closure;
- wrong-asset vectors;
- confidential-asset escape vectors;
- value conservation;
- recipient and authorization vectors;
- malformed commitment/opening vectors;
- state/event delta checks;
- lifecycle-path checks where representation affects safety.

### 10.2 Minimality report

The minimality report must include, for every claimed lower-disclosure seam:

- selected representation;
- semantic relation;
- target proof;
- accepted representation vector;
- reference representation vector;
- equal public semantic projection;
- witness availability;
- lifecycle reachability;
- reason a still-lower mode is unsupported or not selected.

### 10.3 Target dependency evidence

Representation claims require separately named target evidence for the proof
methods actually selected, including where applicable:

- confidential value conservation;
- value-commitment equality;
- authenticated opening;
- explicit value introspection;
- explicit asset introspection;
- rangeproof/transaction construction;
- sighash output commitment.

### 10.4 Independent audit consequence

A confidential value representation may shift some audit claims from direct
public summation to verification of commitment/conservation relations.

This trust-surface shift must be named in deployment documentation.

The attestation indexer and receipt-accounting auditor must still receive every
public fact their interfaces require.

### 10.5 Release profile consequence

The release must bind:

- representation-plan identity;
- selected proof methods;
- target identity;
- vector reports;
- public-opening schema where used;
- normalization paths;
- target dependency evidence;
- linked-bundle and ABI identities.

A report generated for an explicit-only bundle cannot validate a confidential
bundle.

### 10.6 Model evidence remains semantic

The model's one-value-per-UTXO representation is semantic evidence.

It does not prove:

- a target commitment binds correctly;
- CT balance is enforced;
- a rangeproof can be constructed;
- an opening is public;
- a confidential asset cannot escape;
- a permissionless constructor has the witness.

Those remain target/backend/transaction/deployment evidence.

---

## 11. Determinism and identity consequences

### 11.1 Representation plan identity

The selected representation and proof plan must have a deterministic identity
binding:

- architecture identity;
- realization identity;
- operation and fact IDs;
- target identity;
- representation mode per seam;
- selected proof alternative;
- normalization paths;
- public-opening format;
- compiler policy;
- canonical ordering.

### 11.2 Realization identity

The realization identity may bind the **allowed semantic representation
capabilities** and lifecycle requirements.

It must not bind one backend's:

- commitment bytes;
- prefix bytes;
- rangeproof;
- blinding factor;
- transaction nonce;
- concrete opening serialization.

### 11.3 Concrete randomness

Confidential transaction construction may require cryptographic randomness.

The semantic relation and representation-plan identity remain deterministic.
Concrete transactions may differ when randomness is an explicit construction
input.

Canonical release vectors must use:

- fixed deterministic test randomness; or
- committed concrete transaction bytes.

Production wallet randomness must not be derived from deterministic release
test seeds.

### 11.4 Publication reports

Canonical representation reports must not include secret:

- blinding factors;
- owner private keys;
- unpublished openings;
- rangeproof witness material.

Public openings may be included only when the selected representation declares
them public.

### 11.5 Proof selection

Given the same:

- realization;
- target;
- deployment policy;
- compiler configuration;
- resource constraints,

proof selection must be deterministic.

Equal-cost alternatives require a canonical tie-break.

---

## 12. Implementation and migration

### 12.1 Phase 1: semantic representation declarations

The two pilot operations establish the initial semantic distinction.

#### `compact-ash`

Expected declaration:

- ASH values are publicly usable;
- operation is permissionless;
- no private owner witness is allowed;
- exact ownerless conservation holds;
- closed `U` asset identity is rigid.

#### `transfer-live-receipts`

Expected declaration:

- value-preserving lateral movement;
- explicit closed `U` asset identity;
- owner authorization;
- value may use explicit or confidential proof alternatives;
- no public state or event value is derived from denominations.

### 12.2 Phase 2: disclosure and lifecycle analysis

The compiler adds:

- disclosure-demand propagation;
- proof alternatives;
- witness availability;
- lifecycle reachability;
- deterministic proof-plan identity;
- safety/minimality distinction.

### 12.3 Phase 3: typed target capabilities

`target-elements` must source-pin:

- confidential value conservation;
- value commitment behavior;
- explicit asset/value prefix interpretation;
- target opening capability if used;
- resource and policy limits.

Do not assume direct opening support before the concrete target pattern is
implemented and tested.

### 12.4 Phase 5: private-value live transfer

The live-transfer milestone should demonstrate:

- explicit `U` identity;
- private committed input/output values;
- CT conservation;
- owner signatures;
- exact constructor closure;
- sponsor isolation;
- equal semantic result to an explicit transfer.

This is the first release-quality minimality test.

### 12.5 Phase 7: public ASH synchronization

Before burn/clear integration, resolve:

- explicit ASH versus public-committed ASH;
- authenticated opening;
- residual blinding routing;
- public constructor availability;
- canonical transaction ABI.

The accepted result updates the research note and may produce a new decision
record.

### 12.6 Later operations

Add representation support operation by operation.

Do not claim a representation mode globally merely because one operation
supports it.

### 12.7 Client-facing ABI

The transaction package must expose:

- required representation per input/output family;
- required public openings;
- owner/operator/sponsor witness roles;
- normalization path;
- constructor requirements;
- target proof material.

Wallets must not reverse-engineer representation requirements from script.

### 12.8 Release documentation

Human-facing release documentation should state:

- which holdings or flows have amount privacy;
- which protocol values are public;
- which assets remain explicit;
- what target assumptions protect commitments;
- which operations declassify values;
- which permissionless paths use public openings;
- what normalization may be required.

Avoid general statements such as “transactions are confidential.”

---

## 13. Risks and limitations

### 13.1 Transaction-construction complexity

Confidential value support adds:

- blinding-factor management;
- rangeproof generation;
- balance equations;
- public-opening handling;
- deterministic test fixtures;
- wallet APIs;
- error modes.

The transaction package may become one of the largest target-specific
components.

### 13.2 Privacy may be narrow

The initial deployment may provide value privacy primarily for:

- owner-controlled lateral receipt transfer;
- sponsor values;
- selected commitment-preserving transitions.

Many protocol values remain public by design.

This limited scope must be documented honestly.

### 13.3 Representation combinations may interact

A transaction with several object families may not support every independent
combination of representation modes.

Proof planning may need operation-wide compatibility checks.

For example:

- a confidential input may require a committed output for blinding balance;
- a public state delta may force a public synchronization output;
- a permissionless branch may reject an otherwise valid owner-private
  representation.

### 13.4 Rangeproof availability

A script can verify output relations while a transaction constructor still
cannot generate a valid rangeproof without private data.

This is especially relevant to permissionless receipt relabel and settlement.

### 13.5 Public openings enlarge data surfaces

A published opening may:

- increase witness size;
- create canonicalization requirements;
- need indexing and long-term availability;
- reveal transaction linkage;
- become a new parser surface.

It must be treated as a typed ABI, not an informal metadata convention.

### 13.6 Confidentiality may change audit trust

Direct public summation can become commitment-proof verification.

That introduces reliance on:

- commitment binding;
- CT consensus;
- proof correctness;
- target implementation.

This is a named deployment residual, not an invariant failure.

### 13.7 Over-general representation IR

A universal representation system may become more complex than the operations
need.

Mitigation:

- implement the minimum modes required by pilots;
- add capability vocabulary incrementally;
- keep target-specific proof details below the compiler boundary;
- do not add speculative cryptographic schemes.

### 13.8 Asset-rigidity cost

Explicit asset identity may reveal object classes and reduce privacy.

This is accepted for initial safety.

A future confidential-asset design must prove exact protocol classification
rather than merely requesting greater privacy.

---

## 14. Supersession conditions

This decision may be superseded if the project adopts a target proof that
safely supports confidential closed-asset identity.

A replacement must establish:

1. exact closed-asset class proof;
2. canonical object-family closure;
3. issuance and authority integrity;
4. no hidden closed-asset escape;
5. public audit compatibility;
6. permissionless constructibility;
7. lifecycle reachability;
8. relation-indexed safety vectors;
9. target substrate evidence;
10. deployment-profile representation;
11. normative denotation review.

It may also be superseded by a deployment policy that intentionally requires
all protocol values explicit.

Such a replacement must distinguish:

- dropping a minimality claim;
- changing compiler policy;
- changing abstract representation latitude;
- changing protocol observables or capabilities.

This decision is not superseded merely because:

- the first backend initially supports explicit values for some operations;
- authenticated opening is deferred;
- Simplicity supports different proof methods;
- one deployment disables confidential transfer;
- public committed values use one target-specific encoding;
- a wallet chooses explicit transactions voluntarily.

---

## 15. References

### Normative and current source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/architecture/src/spec.rs`](../../packages/architecture/src/spec.rs)
- [`../../packages/architecture/src/ids.rs`](../../packages/architecture/src/ids.rs)
- [`../../packages/model/src/object.rs`](../../packages/model/src/object.rs)
- [`../../packages/model/src/asset.rs`](../../packages/model/src/asset.rs)
- [`../../packages/model/src/kernel.rs`](../../packages/model/src/kernel.rs)
- [`../../packages/model/src/shape.rs`](../../packages/model/src/shape.rs)
- [`../../packages/model/src/recognition.rs`](../../packages/model/src/recognition.rs)
- [`../../packages/model/src/quiescence.rs`](../../packages/model/src/quiescence.rs)

### Related decisions

- [D001: Typed Rust Is Normative](001-typed-rust-is-normative.md)
- [D002: Introduce a Target-Independent Realization Layer](002-target-independent-realization-layer.md)
- [D003: Design for Multiple Backends and Implement Elements Tapscript First](003-multiple-backends-tapscript-first.md)
- [D004: Use Per-Bundle Translation Validation Instead of Initially Trusting the Compiler](004-translation-validation-over-compiler-trust.md)
- [D006: Canonical Transaction-Layout ABI](006-canonical-transaction-layout-abi.md)

### Package plans

- [`../packages/realization.md`](../packages/realization.md)
- [`../packages/compiler.md`](../packages/compiler.md)
- [`../packages/target-elements.md`](../packages/target-elements.md)
- [`../packages/tapscript.md`](../packages/tapscript.md)
- [`../packages/transaction.md`](../packages/transaction.md)
- [`../packages/vectors.md`](../packages/vectors.md)
- [`../packages/release.md`](../packages/release.md)

### Research

- [`../research/public-declassification.md`](../research/public-declassification.md)
- [`../research/settlement-layout.md`](../research/settlement-layout.md)

---

## 16. Decision summary

> Treat values as target-independent semantic amounts whose concrete
> representation may be explicit, publicly committed with an authenticated
> opening, or privately committed when a supported proof preserves the same
> relation. Keep every closed protocol asset identity explicit and classified
> in the initial Elements backend; preserve permissionless construction and
> lifecycle exits; derive disclosure from typed dependencies; and validate
> safety with rejecting closure vectors separately from disclosure minimality
> with accepting representation metamorphisms.
