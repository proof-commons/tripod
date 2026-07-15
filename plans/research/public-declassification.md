# Research Question: Confidential-to-Public Value Synchronization

> **Status:** OPEN / PROTOTYPE REQUIRED
> **Blocks:** direct confidential burn-to-public-ASH support; direct redemption
> of confidential live receipts; any representation path that consumes a
> confidential semantic value and must expose a publicly usable amount without
> losing commitment balance; final public-opening ABI; complete lifecycle
> support for private live receipts
> **Does not block:** explicit-value `compact-ash`; explicit-value burn and
> redemption; confidential lateral live-receipt transfer; owner-authorized
> normalization as a fallback if separately implemented and accepted
> **Affected packages:** `realization`, `compiler`, `target-elements`,
> `tapscript`, `linker`, `transaction`, `vectors`, and `release`
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-source.md),
> [D002](../decisions/002-realization-layer.md),
> [D003](../decisions/003-tapscript-first.md),
> [D004](../decisions/004-translation-validation.md),
> [D005](../decisions/005-value-representation.md),
> [D006](../decisions/006-transaction-abi.md)
> **Related normative constraints:** semantic value is independent of target
> representation; consensus value is authoritative; closed asset identity is
> explicit in the initial backend; burn publishes a fresh ASH aggregate;
> redemption publishes amount-dependent state and payout effects;
> permissionless ASH compaction and clear require public construction data;
> declassification is derived from typed dependencies; safety and disclosure
> minimality are separate; representation lifecycle exits must remain reachable
> in `docs/attestation/realization.md`
> **Expected decision output:** an operation-specific accepted representation
> policy for confidential-to-public transitions, likely selecting one or more
> of direct public-commitment synchronization, owner-authorized normalization,
> or explicit-only initial support; plus a canonical public-opening/proof schema
> if direct synchronization is accepted
> **Machine-consumed by the toolchain:** no

---

## 1. Question

How can the initial Elements backend consume a semantic value represented by a
confidential target commitment and make that value publicly and durably usable
when the abstract operation requires public synchronization?

The construction must simultaneously:

1. bind the public amount to the exact consensus-enforced value commitment of
   the consumed or created protocol object;
2. preserve explicit closed protocol asset identity;
3. preserve target confidential-transaction balance, including residual
   blinding terms;
4. expose enough public information for an unrelated future permissionless
   constructor to build the next operation;
5. require no retained owner secret on permissionless lifecycle paths;
6. use one canonical typed encoding;
7. reject false amounts, false blinders, wrong commitments, wrong assets,
   replayed openings, malformed capsules, and missing public data;
8. preserve the same target-independent semantic successor and public
   observables as the explicit-value representation;
9. fit target script, witness, transaction-weight, proof, crypto, and policy
   limits;
10. produce deterministic linked recipes and ABI descriptions;
11. support relation-indexed safety and minimality evidence.

The two primary operation paths are:

### Burn

```text
private committed live receipts
    ↓ owner-authorized burn
fresh publicly usable ASH aggregate
    +
optional private live receipt change
    +
public burn records
```

The ASH amount must be publicly available because:

- it anchors record acceptance;
- anyone may compact ASH;
- anyone may clear ASH;
- clear derives a public STATE decrement from it.

### Redemption

```text
private committed live receipt
    ↓ owner-authorized redemption
public/authenticated receipt amount x
    +
formula-bound public payout p
    +
public STATE and RESV successor effects
    +
receipt destruction
```

The redemption relation must authenticate `x` before proving:

```text
p = floor(x * Ω / Y)
```

and the state/reserve deltas.

A direct private-to-public path may not be the only acceptable answer. An
owner-authorized normalization transaction may be selected when it preserves
the same semantic value and lifecycle while avoiding an impractical direct
proof.

The research must decide the supported initial deployment profile rather than
assuming every theoretically possible representation is required immediately.

---

## 2. Why the answer matters

### 2.1 Private transfer is incomplete without lifecycle exits

Confidential live-receipt transfer may preserve value without revealing
denominations.

That representation is not deployment-complete unless the resulting receipt
can still reach its required exits:

```text
transfer
burn
redeem
```

A representation that transfers privately but cannot later burn or redeem is a
stranded object representation.

The compiler lifecycle analysis must therefore know one supported path from a
private receipt to every required exit.

### 2.2 Burn crosses the public accounting boundary

Burn is semantically lateral at the immediate operation boundary:

```text
live receipt U
    →
ASH U
```

Actual `U` destruction occurs later at clear.

Nevertheless, burn declassifies a fresh aggregate:

```text
F(T) = fresh ASH value
```

because the public attestation record is accepted only when:

```text
sum(record amounts) <= F(T)
```

A false public ASH amount would grant incorrect attestation or corrupt later
clear accounting.

### 2.3 Clear must be constructible by anyone

If the ASH output remains confidential but only the burner knows its opening,
then an unrelated clearer cannot:

- compute the ASH batch total;
- construct the clear amount;
- prove the STATE decrement;
- construct residual ASH.

That would turn a permissionless maintenance path into an owner-assisted one.

The selected ASH representation must therefore publish or encode all data
needed by arbitrary future constructors.

### 2.4 Explicit outputs may not absorb confidential blinding residue

A confidential value commitment can be described schematically as:

```text
C = vH + rG
```

where:

- `v` is semantic value;
- `r` is value blinding;
- `H` is an asset-specific value generator;
- `G` is the blinding generator.

A fully explicit value output carries no value-blinding term.

If confidential inputs are fully consumed and all outputs for that asset are
explicit, their residual blinding contribution may have nowhere to go in the
target confidential-transaction balance.

One output may need to remain commitment-encoded even when its amount is
public.

This motivates the `PublicCommitted` representation:

```text
commitment encoded
+
amount publicly known
+
opening publicly authenticated
```

### 2.5 Metadata amount alone is unsafe

Publishing:

```text
ash_amount = 100
```

in an unauthenticated data field does not prove the ASH commitment carries
value 100.

The amount must bind to the exact consensus commitment or explicit value.

Otherwise an attacker could:

- create low-value ASH;
- claim high public ASH;
- obtain false attestation;
- later clear too much recorded supply.

### 2.6 The answer affects target proof planning

The result determines whether the compiler can select:

```text
DirectPrivateBurnToPublicAsh
DirectPrivateRedemption
OwnerAuthorizedNormalization
ExplicitOnlyAtBoundary
```

and which target capabilities each plan requires.

The target package must not advertise a complete authenticated-opening
capability until this construction is implemented and tested.

### 2.7 The answer affects the public transaction ABI

The selected construction may add:

- value amount;
- blinding factor;
- opening proof;
- public opening capsule;
- output commitment;
- metadata field;
- deterministic representation nonce;
- normalization operation;
- extra public-committed output;
- additional target program;
- additional resource formula.

Those choices affect clients, vectors, linking, calibration, and release.

---

## 3. Existing constraints

### 3.1 Semantic constraints

- One semantic value exists per protocol object.
- Metadata may not override consensus value.
- Burn consumes live receipts only.
- Burn output closure permits one fresh ASH plus optional live receipt change.
- Burn records are public and capped by fresh ASH.
- ASH is ownerless and publicly maintainable.
- Clear consumes ASH and publicly decrements `Y_L`.
- Redemption consumes one live receipt and pays a formula-bound amount.
- Redemption's receipt amount and payout affect public state.
- Permissionless operations must remain constructible without owner secrets.
- Representation changes must preserve semantic operation and public
  observables.
- Closed protocol asset identity remains explicit in the initial backend.

### 3.2 Target constraints

The exact Elements target must define and verify every primitive relied upon,
including where selected:

- input value introspection;
- output value introspection;
- confidential commitment encodings;
- confidential transaction conservation;
- commitment equality;
- explicit asset identity;
- value generators;
- scalar multiplication or tweak verification;
- hash and byte operations;
- target witness limits;
- target crypto budget;
- output nonce and rangeproof behavior;
- input UTXO data actually available to script and the transaction builder.

The target documentation currently notes that some input fields, including
input nonce information, may not be available through the same introspection
path as value commitments. The prototype must use exact reviewed target facts
rather than assume every output field survives into spend-time introspection.

### 3.3 Asset policy

For all protocol receipt and ASH outputs in the initial Elements profile:

```text
asset identity = explicit U
```

A confidential asset commitment is not accepted as a closed-asset proof.

The public-opening problem concerns the value axis only.

### 3.4 Constructibility constraints

For burn:

- owners may contribute private receipt openings/blinding data during the burn
  signing process;
- all owners must authorize the finalized output set;
- the resulting ASH opening/capsule must become public.

For clear and compaction:

- no burner/owner secret may be required;
- all required opening data must be recoverable from public chain history or a
  canonical public index.

For redemption:

- the current owner may provide private opening data;
- payout and state effects become public;
- an owner-authorized normalization step is semantically possible only if it
  preserves value, asset, owner, class, and lifecycle.

### 3.5 Signature constraints

All required owner signatures must commit the final economic output set.

Therefore:

- representation and blinding must be finalized before signing requests;
- public opening data carried in outputs must be signed where output
  commitment is required;
- a third party must not be able to alter the opening and invalidate records or
  redirect value after owners sign.

### 3.6 Canonical publication constraints

If public openings are stored in transaction data:

- they need one canonical encoding;
- ordinal/location rules need one ABI;
- future constructors need deterministic retrieval;
- unknown/malformed encodings reject;
- target/indexer/publication reports bind exact bytes.

### 3.7 Release constraints

A deployment claiming direct confidential burn/redemption must bind:

- selected public-opening proof pattern;
- target capability evidence;
- transaction ABI;
- representation-safety report;
- representation-minimality report;
- lifecycle/constructibility report;
- exact linked bundle and target identities.

---

## 4. Definitions and terminology

### 4.1 Private committed value

A target value commitment whose semantic amount and value blinding are not
publicly available.

The current owner or authorized transaction constructor may know the opening.

### 4.2 Public committed value

A target value commitment whose semantic amount and sufficient authenticated
opening data are publicly available.

It retains commitment algebra but provides no amount privacy.

The exact public opening may include:

- amount;
- value blinding factor;
- asset generator/class identity;
- proof data;
- domain/instance binding.

### 4.3 Explicit value

A target value encoded directly in the target's explicit amount form.

It contains no value-blinding term.

### 4.4 Opening

Typed data proving or enabling verification that one target commitment
represents one semantic amount under the relevant asset generator.

An opening is not authentic merely because it contains an amount and blinding
factor.

### 4.5 Public opening capsule

A canonical public data object containing all nonsecret information required by
future constructors and target programs to authenticate a public committed
value.

The capsule may be:

- part of constructor metadata;
- a nonspendable data output;
- a canonical event field;
- another target-supported public commitment structure.

Its exact location is unresolved.

### 4.6 Residual blinding

The value-blinding contribution remaining when confidential inputs are consumed
and some outputs are explicit.

The target transaction's commitment balance must route this contribution into
one or more commitment-valued outputs.

### 4.7 Normalization

An authorized semantics-preserving operation changing only concrete
representation or allowed metadata.

Example:

```text
LiveReceipt<PrivateCommitted>
    →
LiveReceipt<PublicCommitted or Explicit>
```

while preserving:

- explicit `U` asset;
- amount;
- owner;
- live class;
- no state change;
- no burn/attestation event.

### 4.8 Boundary operation

An operation whose semantic dependencies require a value to cross from private
representation into public state, public event, public audit, or
permissionless construction.

Burn and redemption are initial boundary operations.

---

## 5. Required properties

A production confidential-to-public construction must satisfy all applicable
properties below.

### 5.1 Exact value binding

The public amount must bind to the exact target consensus commitment/value of
the relevant protocol object.

No independent metadata face amount is accepted.

### 5.2 Explicit closed asset identity

Every relevant receipt, ASH, destruction, or synchronization output must
expose and match explicit `U` asset identity under the initial target policy.

### 5.3 Complete value conservation

The target transaction must satisfy exact semantic conservation:

#### Burn

```text
sum(live receipt input values)
    =
fresh ASH value
    +
sum(live receipt change values)
```

#### Redemption

```text
receipt U value
    =
declared U destruction/synchronization relation
```

and the public payout/state relation must use the same authenticated receipt
amount.

The exact on-chain destruction representation remains backend-specific.

### 5.4 Residual blinding closure

Every confidential input blinding contribution must be accounted for in the
target transaction.

No hidden or unbalanced blinding term may be ignored because the semantic
amount equation balances.

### 5.5 Public durability

A public opening/capsule required by future operations must remain available
from public canonical chain data.

It must not depend on:

- owner wallet backup;
- private indexer database;
- operator server;
- ephemeral transaction-construction memory;
- unpublished witness data not preserved by the chain.

### 5.6 Permissionless future construction

An unrelated actor must be able to construct:

```text
compact-ash
clear
```

from the ASH output and public opening/capsule without contacting the burner.

### 5.7 Canonical encoding

The public opening/capsule must have one canonical encoding under the selected
ABI.

Alternate encodings must reject or be explicitly represented as equivalent.

### 5.8 Instance binding

An opening must bind to:

- exact outpoint or target output;
- exact value commitment;
- exact asset generator/class;
- constructor/object kind;
- schema/domain;
- target/bundle identity where needed.

An opening copied from another output must not verify.

### 5.9 Authorization integrity

For owner-authorized boundary operations:

- every owner signs the finalized output set;
- public opening/capsule is included in the protected output projection;
- a third party cannot replace or remove the opening after signing;
- normalization cannot change owner/value/class without authorization.

### 5.10 Lifecycle completeness

The selected private representation must retain all required exits through:

- direct boundary operation; or
- an accepted normalization path.

### 5.11 Public observable equality

The confidential-to-public path must produce the same abstract/public result as
the corresponding explicit-value path.

### 5.12 Target feasibility

The complete target transaction and future permissionless spend must fit:

- target consensus;
- relay/standardness policy;
- rangeproof/surjection-proof constraints;
- witness limits;
- stack limits;
- crypto budget;
- transaction weight;
- calibrated operation bounds.

### 5.13 Secret-safe reporting

Canonical release artifacts must not include owner-private input openings
unless the selected operation explicitly declassifies them.

Public output openings may be included because they are intentionally public.

---

## 6. Candidate approaches

No candidate is accepted until prototype and measurement complete.

---

### Candidate A — Explicit-at-rest boundary profile

#### Construction

Require receipt and ASH values to be explicit before burn or redemption.

Private receipt transfer may be unsupported in the initial deployment, or
private receipts must normalize first.

#### Advantages

- simplest target arithmetic;
- simplest audit;
- simplest permissionless ASH maintenance;
- no public-opening proof;
- no residual-blinding issue at the boundary object itself if no confidential
  inputs remain;
- lowest implementation risk.

#### Risks

- weak disclosure minimality;
- private transfer may strand receipt exits unless normalization exists;
- an explicit-only deployment cannot claim private receipt lifecycle support;
- owner must reveal through a separate step if private receipts exist.

#### Current status

Safe baseline candidate. May be accepted for an initial explicit-only profile
if private receipt support is deferred honestly.

---

### Candidate B — Owner-authorized normalization before burn or redemption

#### Construction

Use an owner-authorized value-preserving transaction:

```text
private live receipt
    →
public committed or explicit live receipt
```

Then burn or redeem through an existing public-value operation.

#### Required normalization relation

```text
input and output asset = explicit U
input and output class = live
input and output owner = same
input semantic value = output semantic value
no state change
no event
no destruction
```

Potential target proof:

- commitment equality if output remains commitment-encoded;
- authenticated opening plus explicit output;
- another exact target relation.

#### Advantages

- separates privacy boundary from complex burn/redemption operations;
- owner consent is available;
- target program can focus on one-to-one exact preservation;
- lifecycle remains explicit in client UX;
- boundary operations remain simpler;
- no permissionless owner secret is needed because normalization is
  owner-authorized.

#### Risks

- two transactions;
- extra fee;
- latency and contention;
- normalization can reveal value before burn/redemption;
- public output must remain valid and available;
- explicit output still faces residual-blinding balance unless another
  commitment output carries it;
- amount-preserving proof and rangeproof construction remain required;
- users may fail to normalize.

#### Current status

Strong fallback candidate.

---

### Candidate C — Direct public-committed synchronization output

#### Construction

The boundary transaction creates a `PublicCommitted` output whose:

- asset is explicit `U`;
- value remains commitment-encoded;
- amount is public;
- opening data is public and authenticated;
- value blinding absorbs the required confidential-input residual;
- constructor metadata or public capsule makes the opening available later.

For burn, this output is fresh ASH.

For redemption, it may be a dedicated unspendable U destruction or
synchronization output, depending on the accepted target destruction design.

#### Advantages

- direct one-transaction private-to-public transition;
- preserves CT balance;
- public amount supports permissionless future operations;
- amount is bound to commitment;
- no pre-normalization transaction;
- clean semantic interpretation of `PublicCommitted`.

#### Risks

- target opening verification may be complex;
- public blinding factor can create linkability;
- public capsule must be permanently available;
- input/output commitment generators and asset handling must be exact;
- transaction builder must produce valid rangeproofs;
- script may need expensive EC arithmetic;
- input nonce information may be unavailable at spend time;
- future clear must verify the opening from public data;
- destruction output semantics for redemption may require additional design.

#### Current status

Leading direct candidate. Requires complete prototype.

---

### Candidate D — Explicit synchronization output plus private change

#### Construction

Create the public boundary value as an explicit output and route residual
blinding into another confidential output, typically legitimate live receipt
change.

#### Advantages

- boundary amount directly explicit;
- simple future permissionless use;
- no public-opening proof for the boundary output;
- potentially efficient when private change exists.

#### Risks

- does not work when the confidential input is fully consumed and no private
  change exists;
- attacker/caller must not create artificial value-bearing change that changes
  semantics;
- burn-all and redeem-all cases remain unresolved;
- sponsor change is a different asset and cannot absorb `U` blinding;
- operation semantics may not permit another private `U` output.

#### Current status

Useful conditional optimization, not a complete lifecycle solution.

It may be part of Candidate C or B but cannot be the only supported direct
boundary plan.

---

### Candidate E — Public opening carried only in transaction witness

#### Construction

The boundary transaction or future spend provides the opening in witness data,
but no canonical public capsule is committed in the creating transaction's
outputs/data.

#### Advantages

- no extra public data output;
- simple creating transaction;
- opening may be available in full witness history.

#### Risks

- future constructor must reliably retrieve historical witness;
- target input introspection may not expose every needed field;
- opening may not be bound canonically to the object constructor;
- pruning/indexing availability assumptions;
- public audit tooling must retain the witness;
- future script still must verify the opening;
- ambiguity between several historical openings.

#### Current status

Not accepted without a durable canonical retrieval and binding design.

---

### Candidate F — Public opening capsule in nonspendable data output

#### Construction

Create:

- the public committed protocol output;
- a separate canonical nonspendable data output carrying the opening/capsule.

The protocol output constructor may commit the capsule hash or outpoint
relation.

#### Advantages

- public durable chain data;
- clear separation between spendable object and opening;
- easy independent indexing;
- capsule may carry amount, blinding, schema, and object binding.

#### Risks

- additional output and transaction weight;
- target script must bind capsule to protocol output;
- future input program must authenticate capsule data from history or witness;
- input introspection does not automatically retrieve creating transaction data;
- unrelated fake capsules must remain inert;
- may require a public indexer/proof-of-origin;
- could create a new standing semantic object if relied upon incorrectly.

#### Current status

Candidate storage mechanism for Candidate C, not independently sufficient.

---

### Candidate G — Public opening embedded in object constructor metadata

#### Construction

The protocol object's metadata commitment includes:

- public amount;
- public blinding/opening fields;
- schema;
- object/instance binding.

The target output remains commitment-valued.

#### Advantages

- one object constructor binds program and opening;
- no separate capsule output;
- future constructor obtains opening from public constructor metadata;
- integrates with metadata-dependent constructor research.

#### Risks

- metadata is not automatically available to future scripts unless supplied
  and authenticated;
- constructor becomes larger;
- public blinding may affect metadata/schema identity;
- target commitment reconstruction more expensive;
- public opening authenticity still needs target proof;
- hash-to-scalar and constructor totality interactions;
- object program identity varies with opening data.

#### Current status

Leading storage candidate to compare with Candidate F.

Depends on the state/object-constructor result.

---

### Candidate H — Direct spend-time opening proof without a public committed successor

#### Construction

For redemption, owner supplies an opening of the private input value directly
to the target program.

The receipt is destroyed; no future protocol object needs the opening.

The target verifies the input commitment opening and uses `x` in public
arithmetic.

#### Advantages

- no normalization;
- no public successor object;
- opening need only exist in redemption witness;
- owner is present and authorized;
- public payout/state effects naturally reveal `x`.

#### Risks

- exact target opening proof may be expensive or unavailable;
- value commitment generator/input data must be available;
- residual input blinding still needs to be balanced by transaction outputs or
  an unspendable committed destruction/synchronization output;
- destruction representation must close `U`;
- opening proof must bind to the exact input commitment;
- target crypto budget may be significant.

#### Current status

Promising redemption-specific candidate, but not a burn/ASH solution.

---

### Candidate I — General zero-knowledge proof

#### Construction

Provide a proof that:

- private input values satisfy semantic conservation;
- public boundary value/state delta is correct;
- closed asset identities remain correct.

#### Advantages

- strongest privacy flexibility;
- one proof may cover several relations;
- could avoid publishing blinding factors.

#### Risks

- not an approved current target capability;
- large implementation and trust surface;
- proving/witness availability;
- target verification costs;
- new cryptographic assumptions;
- proof-system deployment.

#### Current status

Out of scope for the initial Elements backend. Possible future backend work.

---

## 7. Threat and failure model

The attacker can control or influence:

- confidential input commitments;
- claimed public amount;
- claimed blinding factor;
- public opening/capsule bytes;
- output value representation;
- output nonce;
- rangeproof/surjection proof;
- target output ordering;
- object metadata;
- selected operation program;
- witness ordering;
- record amounts;
- live receipt change;
- sponsor outputs;
- copied opening from another object;
- stale opening from another bundle/schema;
- normalization destination;
- target constructor parameters.

The attacker cannot break:

- target commitment binding;
- target CT consensus;
- target signature security;
- explicit closed-asset identity checks;
- linked program hash/constructor security;
- cryptographic assumptions named by deployment.

### 7.1 False public amount

Commitment carries value `v`, public opening claims `v' != v`.

Required result:

```text
reject
```

### 7.2 False blinding factor

Claimed amount correct, blinding wrong.

Required result:

```text
reject
```

### 7.3 Wrong commitment instance

Opening is valid for another output.

Required result:

```text
reject
```

### 7.4 Wrong asset generator

Opening uses another asset generator or hides another asset.

Required result:

```text
reject
```

Explicit `U` identity remains separately enforced.

### 7.5 Copied capsule

Copy an authentic public capsule from another ASH/receipt.

Required result:

```text
reject
```

### 7.6 Capsule omitted

Create public committed output without durable opening.

Required result:

```text
creating transaction rejects
or
representation is not accepted as publicly usable
```

### 7.7 Capsule mutable after signing

Alter public opening/capsule after owners sign.

Required result:

```text
signature or constructor relation rejects
```

### 7.8 Residual blinding imbalance

Semantic amounts balance but target commitment blinders do not.

Required result:

```text
target CT proof/consensus rejects
```

### 7.9 Artificial private change

Create private `U` change not permitted by the operation merely to absorb
blinding.

Required result:

```text
output-family closure rejects
```

### 7.10 Confidential closed asset

Create confidential asset output at ASH/receipt/destruction seam.

Required result:

```text
reject
```

### 7.11 Unavailable future opening

Burn succeeds, but unrelated clear cannot reconstruct/verify ASH.

Required result:

```text
candidate representation rejected by lifecycle/constructibility gate
```

This is a design rejection even if the burn transaction itself is target-valid.

### 7.12 Noncanonical opening

Supply alternate field order, integer encoding, trailing data, duplicate field,
or unsupported schema.

Required result:

```text
reject
```

### 7.13 Wrong bundle/schema

Use opening/capsule created under another constructor schema or linked bundle.

Required result:

```text
reject
```

### 7.14 Unauthorized normalization

Normalization changes:

- amount;
- owner;
- class;
- asset;
- output family

without semantic authorization.

Required result:

```text
reject
```

### 7.15 Permissionless secret dependence

A future clearer needs owner-private blinding data not present publicly.

Required result:

```text
compiler/lifecycle support rejects the representation before release
```

---

## 8. Prototype design

The prototype should compare both a direct path and a normalization fallback.

### 8.1 Prototype location

Preferred:

- experimental modules in `transaction::elements::confidential`;
- experimental target patterns in `tapscript::patterns::commitment`;
- target-native vectors in `vectors::representation`;
- no production release path until accepted.

### 8.2 Stage A — Exact target CT fact review

Before coding, source-pin and test:

- value commitment format;
- asset generator derivation;
- CT balance;
- input value commitment availability;
- output value introspection;
- input/output nonce availability;
- rangeproof requirements;
- commitment equality support;
- EC/scalar operations available to tapscript;
- target crypto budget;
- transaction library proof construction.

Record which data a future spend can inspect and which must be supplied as
witness.

### 8.3 Stage B — Independent off-chain commitment reference

Implement or use two checked paths to:

- construct value commitment from amount, blinding, and explicit asset
  generator;
- verify opening;
- compare expected target bytes;
- compute CT balance.

At least one path should be an independent explicit reference rather than only
the transaction library that produces the candidate.

### 8.4 Stage C — Synthetic opening-verification pattern

Create a simple target program that:

1. inspects or receives an exact commitment;
2. receives amount and blinding;
3. reconstructs or verifies the commitment;
4. checks amount domain;
5. checks explicit asset/generator identity;
6. returns the authenticated amount for later arithmetic.

Test target-native positive and negative cases.

### 8.5 Stage D — Public capsule representation

Prototype at least:

1. metadata-embedded opening;
2. separate nonspendable capsule, if target retrieval/binding is plausible.

For each, define:

- canonical encoding;
- instance binding;
- constructor relation;
- future retrieval path;
- target verification path;
- resource cost.

Reject candidates that rely on unavailable historical fields without a
specified public index/proof.

### 8.6 Stage E — Synthetic private-to-public transaction

Construct a target transaction with:

- one confidential explicit-`U` input;
- one public committed explicit-`U` output;
- public opening/capsule;
- valid target CT balance;
- valid rangeproof;
- target program verifying opening.

Exercise:

- full consumption with no private change;
- partial consumption with private change;
- multiple confidential inputs;
- multiple owner inputs if tooling permits.

### 8.7 Stage F — Unrelated future spend

In a new process/fixture containing only public chain data and bundle/ABI:

1. discover the public committed output;
2. recover its public opening/capsule;
3. verify the opening;
4. construct a future spend;
5. execute target program successfully.

Do not provide the original owner's private fixture state.

This is mandatory permissionless constructibility evidence.

### 8.8 Stage G — Burn integration

Implement private live receipt burn:

```text
private receipt inputs
    →
public committed ASH
    +
optional private live change
    +
public records
```

Verify:

- exact semantic conservation;
- record total bounded by authenticated ASH amount;
- owner signatures commit outputs/capsule;
- unrelated actor can compact ASH;
- unrelated actor can clear ASH;
- no confidential closed-asset escape.

### 8.9 Stage H — Redemption direct path

Prototype direct private receipt redemption:

- authenticate input opening;
- compute wide floor payout through accepted arithmetic pattern;
- create public state/reserve successors;
- create payout;
- close `U` value/blinding through accepted destruction/synchronization form;
- owner signs finalized outputs.

If direct redemption is too costly or target-infeasible, retain normalization
as the supported path.

### 8.10 Stage I — Normalization fallback

Implement owner-authorized:

```text
private live receipt
    →
public committed or explicit live receipt
```

Then demonstrate burn and redemption through public operations.

Measure:

- two-transaction cost;
- target complexity;
- client interaction;
- disclosure;
- lifecycle.

### 8.11 Stage J — Representation-policy comparison

Compare candidates across:

- semantic safety;
- target feasibility;
- constructibility;
- lifecycle;
- disclosure;
- transaction count;
- witness size;
- target crypto budget;
- transaction weight;
- implementation complexity;
- auditability.

### 8.12 Stage K — Production handoff

Only after decision acceptance:

- stabilize public-opening schema;
- stabilize target proof pattern;
- stabilize constructor recipe;
- stabilize transaction ABI;
- add permanent vectors;
- add target evidence requirement;
- add release report schema.

---

## 9. Public opening capsule candidates

This section refines Candidate C storage options.

### 9.1 Capsule fields

A candidate capsule may include:

```text
domain separator
capsule schema version
object kind
target/bundle constructor identity
creating transaction/output identity or canonical output ordinal
explicit asset ID or asset-generator binding
public amount
public value blinding factor
value commitment digest or exact commitment
metadata digest
optional proof data
```

Exact fields depend on the selected proof.

### 9.2 Instance identity circularity

A creating transaction cannot commit its final txid inside one of its own
outputs without creating a circular dependency.

Therefore capsule binding should prefer fields available before final txid,
such as:

- output ordinal;
- exact output commitment;
- constructor metadata;
- transaction semantic domain;
- bundle/constructor identity;
- another noncircular commitment.

A later indexer can associate capsule and output by transaction position.

Do not require self-committed txid unless an explicit noncircular construction
exists.

### 9.3 Capsule-to-output binding

Possible bindings:

1. capsule contains exact output value commitment and output index;
2. object constructor metadata commits capsule hash;
3. capsule commits object constructor output program and ordinal;
4. both object and capsule derive from one shared semantic commitment.

The prototype must reject swapping capsules between outputs.

### 9.4 Public blinding factor

Publishing a value blinding factor is acceptable only when:

- amount privacy is intentionally relinquished;
- target security assumptions remain valid;
- the factor does not reveal unrelated input/output secrets;
- no private key material is derivable;
- future public construction benefits.

The prototype must review whether revealing one output blinder creates linkage
or balance information beyond the declared public amount.

### 9.5 Canonical availability

The capsule must be retrievable deterministically by independent tools.

Potential rule:

```text
one capsule immediately following the protocol output
```

or another D006-bound layout.

The exact target layout belongs in the transaction ABI.

---

## 10. Normalization relation

If normalization is selected, it needs a typed semantic and target contract.

### 10.1 Semantic relation

For one or more private live receipts:

```text
sum input value = sum output value
input asset/output asset = U
input class/output class = live
owner mapping = identity or owner-authorized redistribution according to the
selected normalization operation
no state effect
no burn/clear/redemption event
```

The safest initial normalization is one-to-one, same owner, same value.

### 10.2 Authorization

Every current owner authorizes.

No operator permission.

### 10.3 Target proof

Possible proof:

- exact commitment equality for one-to-one same-blinding output;
- authenticated input opening plus public committed output;
- CT conservation plus complete one-output closure.

The exact proof must be selected and tested.

### 10.4 Lifecycle

The normalized output must be:

- transferable;
- burnable;
- redeemable;
- recognized by the same live-receipt semantic class.

### 10.5 Client UX

Normalization is a real extra transaction and must be visible to clients.

Do not describe direct confidential redemption support when the deployment
requires normalization.

### 10.6 Representation downgrade

Normalization intentionally reveals the value.

The minimality report must state that privacy ends at normalization.

---

## 11. Test and vector plan

### 11.1 Commitment-opening unit vectors

- amount zero where permitted by helper but reject for positive protocol object;
- amount one;
- maximum semantic amount;
- blinding zero;
- representative nonzero blinders;
- wrong amount;
- wrong blinding;
- wrong asset generator;
- malformed scalar;
- scalar outside group order;
- malformed commitment;
- wrong commitment prefix;
- wrong byte order;
- wrong output instance.

### 11.2 Public capsule vectors

- valid capsule;
- wrong schema;
- wrong object kind;
- wrong output ordinal;
- wrong commitment;
- wrong constructor identity;
- copied capsule;
- duplicate capsule;
- missing capsule;
- reordered fields;
- trailing data;
- noncanonical amount;
- invalid blinding;
- capsule after wrong protocol output;
- capsule altered after owner signature.

### 11.3 CT balance vectors

- private input to public committed output;
- private input to explicit output plus valid private change;
- full private consumption with no change;
- several private inputs;
- incorrect output blinding sum;
- invalid rangeproof;
- wrong explicit asset;
- confidential asset output;
- sponsor L-BTC unable to absorb `U` blinding;
- artificial undeclared `U` change.

### 11.4 Permissionless lifecycle vectors

Using only public chain view:

- compact one public committed ASH;
- compact several ASH outputs from different burners;
- clear public committed ASH;
- produce residual public committed ASH;
- clear residual again;
- reject missing opening;
- reject opening available only from private fixture.

### 11.5 Burn vectors

Positive:

- one private receipt fully burned;
- one private receipt partially burned with private live change;
- several private receipts;
- several owners;
- public records at exact ASH amount;
- under-claiming records;
- sponsorless/sponsored.

Negative:

- false ASH amount;
- false public opening;
- record total above ASH;
- capsule not signed/committed;
- confidential ASH with no public opening;
- confidential asset ASH;
- hidden `U` output;
- wrong live change;
- mixed capsule from another burn.

### 11.6 Redemption vectors

Positive:

- direct private redemption if supported;
- normalization then redemption;
- exact division;
- nonzero rounding remainder;
- partial and sealing cases where implementation stage permits.

Negative:

- wrong input opening;
- wrong `x`;
- wrong payout;
- wrong state decrement;
- residual blinding imbalance;
- hidden `U` escape;
- wrong destruction/synchronization output.

### 11.7 Representation metamorphisms

Compare:

```text
explicit receipt burn
private receipt direct burn
private receipt normalize + public burn
```

where supported.

Require equal target-independent:

- burned semantic amount;
- fresh ASH amount;
- records;
- state effect after later clear;
- public attestation projection.

Likewise compare explicit and private/normalized redemption.

### 11.8 Constructibility isolation

Run future compaction/clear in a fresh process with:

- public chain fixture;
- linked bundle;
- transaction ABI;
- no burner key;
- no burner private opening;
- no original blinding wallet state.

### 11.9 Secret-leak vectors

Verify canonical reports and diagnostics do not include:

- private input blinding factors;
- owner private keys;
- signing nonces;
- unpublished openings.

Public opening/capsule data may appear where intentionally public.

---

## 12. Measurement plan

### 12.1 Commitment proof pattern

Measure:

- script bytes;
- witness amount/blinding bytes;
- capsule bytes;
- EC/hash operations;
- crypto budget;
- peak stack;
- maximum element;
- target operation cost.

### 12.2 Public committed burn

Measure complete transactions for:

- one full burn with no change;
- partial burn with private change;
- multi-input/multi-owner burn;
- maximum record count candidate;
- maximum sponsor count candidate;
- deepest control path.

### 12.3 Future ASH operations

Measure:

- compact batch using public openings;
- clear batch using public openings;
- residual ASH construction;
- opening verification repeated per ASH input;
- maximum candidate ASH batch.

The public-opening proof may dominate clear/compaction resources and therefore
affect `ASH_BATCH_MAX`.

### 12.4 Direct redemption

Measure:

- input opening proof;
- wide arithmetic;
- STATE/RESV constructor checks;
- payout;
- destruction/synchronization output;
- sponsor;
- sealing/nonsealing branches.

### 12.5 Normalization path

Measure:

- normalization transaction;
- subsequent public burn;
- subsequent public redemption;
- total user fees/weight and witness complexity.

### 12.6 Predicted versus observed

Compare backend/linker/transaction formulas with target-native measurement.

### 12.7 Privacy versus cost table

Produce a deterministic comparison table:

| Path | Transactions | Public amount timing | Script cost | Witness cost | Permissionless future use | Status |
|---|---:|---|---:|---:|---|---|

The table is decision evidence, not protocol semantics.

---

## 13. Acceptance criteria

The research may accept different constructions for burn and redemption.

### 13.1 Mandatory safety criteria

For any accepted direct path:

- [ ] public amount binds the exact target commitment/value;
- [ ] wrong amount rejects;
- [ ] wrong blinding rejects;
- [ ] wrong commitment/output instance rejects;
- [ ] explicit `U` asset identity is enforced;
- [ ] confidential closed-asset identity rejects;
- [ ] exact semantic value conservation holds;
- [ ] residual blinding is closed;
- [ ] undeclared private `U` change rejects;
- [ ] canonical opening/capsule encoding is enforced;
- [ ] owner signatures commit the opening/capsule and output set;
- [ ] target-native vectors pass.

### 13.2 Burn lifecycle criteria

For direct private burn support:

- [ ] fresh ASH amount is public and authenticated;
- [ ] public opening/capsule is durable in chain data;
- [ ] unrelated actor can discover and verify it;
- [ ] unrelated actor can construct compaction;
- [ ] unrelated actor can construct clear;
- [ ] residual ASH remains publicly usable;
- [ ] record cap uses the same authenticated ASH amount;
- [ ] no owner secret is required after burn;
- [ ] complete burn/compact/clear target vectors pass.

### 13.3 Redemption criteria

For direct private redemption support:

- [ ] input opening binds exact receipt commitment;
- [ ] authenticated amount feeds exact wide arithmetic;
- [ ] payout and STATE/RESV effects use the same amount;
- [ ] `U` value and residual blinding are closed through an accepted
      destruction/synchronization design;
- [ ] owner authorization protects the final output set;
- [ ] complete target transaction fits selected limits.

### 13.4 Normalization fallback criteria

If normalization is selected:

- [ ] exact amount/asset/owner/class preservation holds;
- [ ] owner authorizes;
- [ ] normalized output has canonical public representation;
- [ ] burn and redemption work afterward;
- [ ] no event/state effect occurs during normalization;
- [ ] lifecycle and client behavior are documented;
- [ ] release does not claim direct private burn/redemption;
- [ ] representation minimality report reflects the extra disclosure/transaction.

### 13.5 Constructibility criteria

- [ ] every permissionless future witness is public;
- [ ] public data can be obtained by an independent fresh process;
- [ ] no private fixture state is used;
- [ ] target/bundle/ABI identities bind the opening;
- [ ] malformed/missing public data fails closed.

### 13.6 Resource criteria

- [ ] direct or normalization path fits target hard limits;
- [ ] required policy/standardness passes;
- [ ] public opening verification fits `compact-ash` and `clear` candidate
      batches;
- [ ] predicted and observed resources agree;
- [ ] final bounds remain subject to calibration.

### 13.7 Evidence handoff

- [ ] accepted path is recorded by new or updated decision;
- [ ] realization proof alternatives are updated;
- [ ] compiler target/disclosure/lifecycle rules are updated;
- [ ] target capability status is updated;
- [ ] backend patterns stabilize;
- [ ] transaction ABI stabilizes;
- [ ] permanent vectors exist;
- [ ] release report requirements exist;
- [ ] residual privacy/audit assumptions are documented.

---

## 14. Rejection criteria

Reject a candidate for the initial deployment when any unresolved condition
below holds.

### 14.1 Value-binding rejection

- false amount/blinding passes;
- opening does not bind exact consensus commitment;
- opening can be copied across outputs;
- wrong asset generator passes;
- metadata amount can override consensus value.

### 14.2 Closed-asset rejection

- confidential/unclassified asset can carry `U`;
- artificial private `U` output can absorb value/blinding;
- output family closure is incomplete.

### 14.3 Constructibility rejection

- future clear/compaction requires burner secret;
- public opening is not durably recoverable;
- capsule depends on private indexer state;
- permissionless builder cannot construct valid rangeproof/successor;
- representation strands required lifecycle exit.

### 14.4 Authorization rejection

- capsule/opening can be modified after signing;
- normalization changes owner/value/class without authorization;
- multi-owner burn cannot coordinate one finalized output set safely.

### 14.5 Target rejection

- required opening proof cannot be implemented with reviewed target
  primitives;
- target input data needed by the verifier is unavailable;
- mandatory negative target vector accepts;
- exact target-native test cannot execute the construction.

### 14.6 Resource rejection

- opening proof exceeds target crypto budget;
- public capsule exceeds required policy;
- direct burn/redemption exceeds transaction limits;
- clear/compaction with opening verification cannot support a useful calibrated
  batch;
- predicted/observed cost cannot be reconciled.

### 14.7 Canonicality rejection

- several uncontrolled public opening encodings pass;
- capsule/output pairing is ambiguous;
- representation depends on self-referential txid;
- deterministic construction cannot be defined.

### 14.8 Assurance rejection

- only the transaction library verifies its own opening without target-native
  proof;
- prototype and production bytes differ without revalidation;
- report contains private input secrets;
- exact target/bundle/ABI identities are not bound.

If direct private support is rejected, explicit or normalization-only support
may still be accepted.

---

## 15. Result

Pending.

When populated, this section must include:

- exact repository revision;
- exact target identity;
- exact backend/linker/transaction revisions;
- candidate paths tested;
- selected public-opening schema;
- selected capsule location/binding;
- selected normalization relation if any;
- target proof/program identities;
- fixed and generated vector summary;
- unrelated-future-constructor result;
- predicted/observed resource table;
- complete transaction hashes;
- report hashes;
- accepted/rejected operation representation matrix.

Required result matrix:

| Operation/path | Explicit | Private direct | Normalize then public | Accepted initial support |
|---|---:|---:|---:|---|
| live transfer | | | n/a | |
| burn | | | | |
| compact ASH | | n/a | n/a | |
| clear | | n/a | n/a | |
| redemption | | | | |

Do not describe direct private support without a passing target-native and
lifecycle result.

---

## 16. Decision and implementation handoff

Pending.

Possible accepted outputs include:

### Outcome A — Direct public committed ASH; normalization for redemption

Potential decision:

```text
D009: Use Publicly Opened Value Commitments for ASH and Owner-Authorized
Normalization for Redemption
```

### Outcome B — Direct public committed ASH and direct private redemption

Potential decision:

```text
D009: Use Authenticated Public Commitment Openings at Value-Declassification
Boundaries
```

### Outcome C — Explicit-only initial deployment

Potential decision:

```text
D009: Restrict the Initial Elements Deployment to Explicit Protocol Values
at Boundary Operations
```

This outcome must state whether confidential live transfer is:

- unsupported;
- supported only with normalization before exits;
- deferred entirely.

The handoff must update:

- realization representation/proof alternatives;
- compiler disclosure/lifecycle analysis;
- target capability registry;
- tapscript patterns;
- transaction ABI;
- vectors;
- resource calibration;
- release claims;
- user-facing privacy documentation.

---

## 17. Residual risks

Any accepted construction may retain the following risks.

### 17.1 Public amount and linkage

A public committed opening intentionally removes amount privacy and may reveal
linkage through public blinding/capsule data.

### 17.2 Commitment assumptions

Correctness relies on target commitment binding, generator derivation, and CT
consensus.

### 17.3 Public data availability

Permissionless lifecycle depends on full public chain/witness/capsule
availability and correct indexing.

### 17.4 Target implementation

Opening verification and CT balance rely on the exact target source and
deployment.

### 17.5 Client complexity

Multi-owner confidential burns may require interactive proof and signing
coordination.

### 17.6 Resource coupling

Public opening verification may reduce practical ASH and settlement-related
batch bounds.

### 17.7 Normalization friction

A two-transaction normalization path adds fee, latency, and UX burden.

### 17.8 Representation diversity

Supporting explicit, public committed, and private forms increases backend,
transaction, vector, and client complexity.

### 17.9 Audit trust shift

Confidential lateral movement changes some audit evidence from direct value
summation to commitment/conservation verification.

### 17.10 Future target revisions

A new target implementation or policy may alter proof size, opening
availability, or accepted transaction form.

---

## 18. References

### Normative and current source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/model/src/object.rs`](../../packages/model/src/object.rs)
- [`../../packages/model/src/kernel.rs`](../../packages/model/src/kernel.rs)
- [`../../packages/model/src/ops/burn.rs`](../../packages/model/src/ops/burn.rs)
- [`../../packages/model/src/ops/ash.rs`](../../packages/model/src/ops/ash.rs)
- [`../../packages/model/src/ops/redeem.rs`](../../packages/model/src/ops/redeem.rs)
- [`../../packages/model/src/ops/transfer.rs`](../../packages/model/src/ops/transfer.rs)
- [`../../packages/model/src/shape.rs`](../../packages/model/src/shape.rs)
- [`../../packages/model/src/invariant.rs`](../../packages/model/src/invariant.rs)
- [`../../packages/model/src/tests/burn_attestation_tests.rs`](../../packages/model/src/tests/burn_attestation_tests.rs)
- [`../../packages/model/src/tests/ash_clear_tests.rs`](../../packages/model/src/tests/ash_clear_tests.rs)
- [`../../packages/model/src/tests/redemption_tests.rs`](../../packages/model/src/tests/redemption_tests.rs)

### Decisions

- [D001](../decisions/001-typed-rust-source.md)
- [D002](../decisions/002-realization-layer.md)
- [D003](../decisions/003-tapscript-first.md)
- [D004](../decisions/004-translation-validation.md)
- [D005](../decisions/005-value-representation.md)
- [D006](../decisions/006-transaction-abi.md)

### Package plans

- [`../packages/realization.md`](../packages/realization.md)
- [`../packages/compiler.md`](../packages/compiler.md)
- [`../packages/target-elements.md`](../packages/target-elements.md)
- [`../packages/tapscript.md`](../packages/tapscript.md)
- [`../packages/linker.md`](../packages/linker.md)
- [`../packages/transaction.md`](../packages/transaction.md)
- [`../packages/vectors.md`](../packages/vectors.md)
- [`../packages/release.md`](../packages/release.md)

### Related research

- [`state-object-constructor.md`](state-object-constructor.md)
- [`wide-arithmetic.md`](wide-arithmetic.md)

### Target reference

- [`../reference/elements-tapscript.md`](../reference/elements-tapscript.md)

### Roadmap

- [`../roadmap.md`](../roadmap.md)

---

## 19. One-line research contract

> Determine, against one exact Elements target and complete transaction
> lifecycle, whether a confidential protocol value can cross into a publicly
> authenticated, durably available, permissionlessly usable representation
> while preserving explicit closed-asset identity, exact semantic value,
> confidential-transaction blinding balance, owner authorization, canonical
> ABI, and target resource limits; compare direct public commitments with
> owner-authorized normalization and explicit-only support; and do not claim
> private burn or redemption support until the exact opening, lifecycle,
> target-native, and resource evidence passes.
