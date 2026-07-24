# Research Question: Confidential-to-Public Value Synchronization · `q:representation:public-opening`

> **Status:** Open; prototype required
> **Blocks:** direct private burn to public ASH; direct private redemption;
> complete private-live-receipt lifecycle
> **Does not block:** explicit boundary operations; confidential lateral transfer
> **Affected packages:** realization, compiler, target-elements, tapscript,
> transaction, vectors, release
> **Decision:** D005
> **Imports:** (`[RZ-sec:realization:representation]`),
> (`[RZ-obl:oracle:representation]`),
> (`[RZ-obl:oracle:disclosure]`),
> (`[RZ-pin:pins:declassify]`)
> **Expected handoff:** direct public-opening policy, normalization policy, or explicit-only initial boundary profile

## Question · `sec:public-opening:question`

How can a semantic value held in a confidential target commitment become
publicly authenticated and durably usable when a boundary operation requires
public arithmetic, public event data, or later permissionless construction?

The construction must preserve:

- exact semantic value;
- explicit closed protocol asset identity;
- target confidential-transaction balance;
- residual blinding closure;
- owner authorization where required;
- public data availability;
- permissionless future lifecycle;
- canonical ABI;
- target feasibility.

Primary paths:

```text
private live receipts
    → burn
    → public fresh ASH

private live receipt
    → redemption
    → public amount-dependent payout and state effects
```

A direct path is not mandatory if owner-authorized normalization preserves the
same semantic lifecycle.

## Fixed constraints · `sec:public-opening:constraints`

### Semantic

- one consensus-enforced semantic value exists;
- metadata amount cannot override it;
- burn publishes/authenticates the fresh ASH aggregate;
- ASH must be compactable and clearable by unrelated parties;
- redemption authenticates `x` before computing payout;
- closed protocol asset identity remains explicit;
- representation changes do not alter amount, owner, class, recipient, state,
  or event semantics.

### Constructibility

Owners may provide private openings for owner-authorized burn or redemption.

After burn, future ASH maintenance may not require:

- burner key;
- burner private opening;
- burner wallet state;
- operator service;
- private indexer database.

### Target

Any opening proof must use capabilities present in the typed target and must
bind the exact commitment and explicit asset.

Low-level EC/hash operations do not constitute an approved opening pattern by
themselves.

### Signing

All output-committed signatures are generated only after:

- output commitments;
- public opening/capsule;
- records;
- change;
- sponsor outputs

are finalized.

## Terminology · `sec:public-opening:terms`

### Private committed

Amount and value blinding are not publicly available.

### Public committed

Amount and authenticated opening are public, while the target value remains
commitment-encoded.

### Explicit

Amount is directly encoded and carries no value-blinding term.

### Public opening capsule

Canonical public data binding:

- amount;
- commitment;
- asset/generator;
- object/schema;
- exact constructor instance or output ordinal;
- required proof data.

The exact fields and location are prototype results.

### Normalization

An owner-authorized value-preserving representation transition:

```text
LiveReceipt<private>
    → LiveReceipt<public committed or explicit>
```

with no state or event effect.

## Candidate matrix · `tbl:public-opening:candidates`

| Mint | Candidate | Strength | Main limitation |
|---|---|---|---|
| `candidate:opening:explicit-boundary` | Require explicit values at burn/redemption | Simple and public | No direct private lifecycle |
| `candidate:opening:normalization` | Owner normalizes before boundary operation | Separates privacy and boundary logic | Extra transaction, fee, latency |
| `candidate:opening:public-commitment` | Direct public committed successor with authenticated opening | One-step private-to-public transition | Proof, capsule, residual blinding, resources |
| `candidate:opening:explicit-plus-private-change` | Explicit boundary output; private change carries residual blinding | Efficient when change exists | Incomplete for full consumption |
| `candidate:opening:witness-history` | Opening appears only in witness history | No extra data output | Availability and canonical binding uncertain |
| `candidate:opening:data-capsule` | Separate nonspendable public capsule | Durable public data | Must bind capsule to object/future spend |
| `candidate:opening:metadata-capsule` | Opening embedded in constructor metadata | One constructor-bound object | Larger constructor and coupling |
| `candidate:opening:direct-redemption` | Owner proves private input opening at spend | No public successor needed | U destruction/blinding closure still required |

The direct leading candidate is
(`candidate:opening:public-commitment`).

The conservative fallback is
(`candidate:opening:normalization`) or
(`candidate:opening:explicit-boundary`).

## Required properties · `sec:public-opening:properties`

An accepted direct path must prove:

1. public amount binds the exact target commitment;
2. explicit closed `U` identity is checked;
3. wrong amount, blinding, generator, or commitment rejects;
4. copied opening from another output rejects;
5. confidential transaction balance closes;
6. no undeclared private `U` output absorbs value or blinding;
7. public data survives in canonical chain data;
8. a fresh unrelated constructor can recover and verify it;
9. future compaction/clear needs no private owner data;
10. output signatures commit the opening/capsule;
11. encoding is canonical and schema-bound;
12. target and policy resources fit;
13. explicit and direct/normalized paths produce equal semantic projections.

## Capsule binding · `q:representation:capsule-binding`

A capsule cannot safely commit its own final transaction ID if that creates a
self-reference.

Candidates should bind using noncircular fields such as:

- target output ordinal;
- exact value commitment;
- exact output program;
- object kind;
- metadata digest;
- constructor/bundle identity;
- creating transaction’s already-determined semantic commitment.

The prototype must reject capsule swapping between outputs.

## Threat model · `sec:public-opening:threats`

| Mutation | Required result |
|---|---|
| public amount differs from commitment | reject |
| wrong blinding factor | reject |
| wrong asset/generator | reject |
| opening copied from another output | reject |
| missing or duplicate capsule | reject or unsupported representation |
| noncanonical capsule | reject |
| capsule altered after signing | signature/constructor rejects |
| semantic amounts balance but blinders do not | target CT rejection |
| artificial private `U` change | output closure rejects |
| confidential asset output | reject |
| opening available only in private fixture | lifecycle/constructibility rejects |
| normalization changes owner/class/value | reject |
| opening from another bundle/schema | reject |

## Prototype · `sec:public-opening:prototype`

### Stage 1 — exact CT review

Confirm through typed target facts and target tests:

- value commitment format;
- explicit asset generator behavior;
- input commitment availability;
- output introspection;
- nonce and rangeproof semantics;
- CT balance;
- commitment equality;
- proof-construction library compatibility.

### Stage 2 — independent commitment reference

Construct and verify commitments through:

- explicit first-party reference;
- reviewed target/library path.

Compare exact bytes.

### Stage 3 — standalone opening verifier

Create a target program that:

- authenticates explicit asset/generator;
- receives amount and opening;
- binds them to one commitment;
- enforces amount domain;
- returns authenticated amount.

Run malformed vectors.

### Stage 4 — capsule candidates

Prototype:

1. metadata-embedded capsule;
2. separate nonspendable capsule, if future retrieval can be authenticated.

Define exact encoding and instance binding.

### Stage 5 — synthetic private-to-public transaction

Cover:

- full private consumption with no private change;
- partial consumption with private change;
- several private inputs;
- residual blinding closure;
- rangeproof construction.

### Stage 6 — fresh-process future spend

Using only:

- public chain fixture;
- linked bundle;
- ABI;
- public capsule/opening;

construct and execute a later maintenance spend.

No original owner private data may be available.

### Stage 7 — burn integration

Construct:

```text
private live receipts
    →
public committed ASH
    +
optional private live change
    +
public records
```

Then compact and clear the ASH as an unrelated actor.

### Stage 8 — redemption and normalization comparison

Compare:

- direct authenticated private redemption;
- normalization then public redemption;
- explicit-only redemption.

## Vectors · `sec:public-opening:vectors`

Required families:

### Opening

- zero/one/maximum amount;
- zero and nonzero blinding;
- wrong amount/blinding/generator;
- malformed scalar/commitment;
- wrong byte order;
- wrong output instance.

### Capsule

- wrong schema/object/ordinal/commitment/constructor;
- copied, missing, duplicate, reordered, or trailing data;
- mutation after signing;
- wrong bundle.

### CT balance

- private to public committed;
- explicit output plus valid private change;
- full private consumption;
- several private inputs;
- incorrect blinding sum;
- invalid proof;
- confidential closed asset;
- artificial undeclared `U` change.

### Lifecycle

- unrelated compaction;
- unrelated clear;
- residual ASH;
- missing public opening;
- private-fixture-only opening.

### Burn and redemption

- explicit/direct/normalized metamorphisms;
- false ASH amount;
- records above ASH;
- wrong payout/state effect;
- hidden `U` escape;
- output mutation after signing.

## Measurements · `sec:public-opening:measurements`

Measure:

- opening verifier program;
- capsule bytes;
- witness opening bytes;
- hash/EC/crypto cost;
- stack and element size;
- full and partial private burn;
- multi-owner burn;
- unrelated compact and clear;
- direct redemption;
- normalization plus boundary transaction.

Produce a decision table:

| Path | Transactions | Public amount timing | Target cost | Permissionless future use |
|---|---:|---|---:|---|

## Acceptance · `gate:public-opening:accept`

Accept direct support only when:

- exact commitment/value binding passes;
- explicit closed asset remains enforced;
- residual blinding closes for full consumption;
- capsule is canonical and instance-bound;
- fresh-process permissionless maintenance succeeds;
- owner signatures protect capsule and outputs;
- every malformed vector rejects;
- target-native transactions and policy pass;
- resource predictions agree with observations;
- explicit/direct/normalized semantic projections agree;
- lifecycle and release evidence are complete.

## Rejection · `gate:public-opening:reject`

Reject a candidate if:

- false opening passes;
- capsule can be swapped;
- confidential closed asset escapes;
- full consumption cannot close blinding;
- future maintenance needs owner secrets;
- public data is not durably recoverable;
- canonical encoding is ambiguous;
- required target capability is incomplete;
- target resources make useful ASH maintenance infeasible;
- only local library verification succeeds.

Direct rejection may still permit normalization or explicit-only support.

## Sponsor-region conservation proof · `q:public-opening:sponsor-conservation`

The F2-006 sponsor-positivity question is settled by policy (D005
sponsor-value opacity; the realization): no protocol predicate may read an
individual sponsor amount, and positivity is not a security relation. What
remains for this research note is the substrate side:

> Which Elements substrate proof establishes exact isolated sponsor-region
> conservation while preserving opacity of every individual sponsor amount?

Candidate proof routes:

- global Confidential-Transaction balance after exact protocol-flow
  cancellation (the residual-relation derivation of the kernel proof);
- a domain-separated sponsor commitment subtotal;
- explicit-value fallback selected only as deployment policy, recorded as
  deployment-policy disclosure, never as semantic necessity;
- another reviewed exact commitment proof.

The target prototype must prove conservation and family isolation without
reintroducing exact-value reads: the relation-indexed emitted-proof report
must show that sponsor paths consume no input/output value-introspection
facts unless a later reviewed policy explicitly supersedes opacity.

## Result · `sec:public-opening:result`

Pending.

## Handoff · `sec:public-opening:handoff`

The result must select an operation-specific initial matrix:

| Operation | Explicit | Private direct | Normalize first |
|---|---:|---:|---:|
| live transfer | | | n/a |
| burn | | | |
| compact ASH | public only | n/a | n/a |
| clear | public only | n/a | n/a |
| redemption | | | |

It then updates typed representation alternatives, target capability status,
backend patterns, transaction ABI, vectors, client claims, calibration, and
release reports.
