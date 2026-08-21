# Research Question: Confidential-to-Public Value Synchronization · `q:representation:public-opening`

> **Status:** Initial policy selected — see (`sec:public-opening:result`)
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

## Candidate matrix · `tab:public-opening:candidates`

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

## Future major: inclusion proofs, no sponsor concept · `q:public-opening:inclusion-partition`

Recorded 2026-07-24, following the F2-006 opacity closure. After it, the
sponsor-side restrictions that remain — exact membership of every ordinary
L-BTC member, the sponsor input bound, one change output, one envelope — are
not security relations. By the inspection-burden criterion no counterexample
needs them: once the branch relation authenticates the protocol region
exactly, consensus conservation forces the remainder to conserve, and
unclaimed members are protected by their owners' signatures and the
recognition spine. What the restrictions actually buy is bounded covenant
introspection (the closure scans that make script resource calibration
meaningful) and a deterministic certificate projection.

The general question for a future major revision: invert the partition from
exclusion to inclusion. The protocol would prove inclusion relations — each
protocol object and flow proves its own membership, position, and relations
at its declared ABI slot — instead of exclusion proofs that claim, scan, or
bound everything else. The sponsor concept then disappears entirely: the
non-protocol remainder is the canonical residual, conserved by consensus,
erased by the protocol projection, with no membership, multiplicity, or
shape rules of its own.

To evaluate before deciding:

- an inclusion-style replacement (or bounded-cost argument) for the one
  genuine exclusion scan, closed-asset closure — no hidden closed-asset
  output may escape recognition;
- the calibration envelope when worst-case shape is bounded only by
  consensus weight limits;
- redefinition of the certificate's sponsor projection as the residual;
- fee attribution as the residual of declared protocol-flow fees.

This moves behavioural arrays (the sponsor input bound, change cardinality,
and envelope-multiplicity rows leave the manifest), so it fires the
versioning gate and is a manifest revision — presumptively major, however it
is argued over the sponsor-erased projection. Decide it with the Phase-2/3
compiler resource model in hand, when the closure-scan cost of an emitted
script can be measured rather than assumed.

## Result · `sec:public-opening:result`

Selected 2026-08-19 by the Guide-11 batch. The initial policy is an
**explicit-only public boundary**, reached from a private value by
**owner-authorized normalization to an explicit output carrying private
change**. A public committed representation and a direct authenticated
opening are both **deferred against three named blockers**, and the public
opening capsule is **not applicable while they are deferred**.

This is a target and backend policy. It is not a claim that the semantic
relation requires an explicit amount, and it is not a disclosure-minimal
result. The conservation matrix conserved value over commitments without
reading any amount in the clear, so the explicit boundary is a choice made
in the light of what the reviewed target can authenticate, not an
arithmetic necessity. Every fact the normalization path publishes is typed
`DeploymentPolicy` and carries the alternative a deployment that wanted it
private would have to take.

### The three blockers

The review of the target's confidential-value machinery named three
independent reasons an authenticated opening has no complete on-script
form under the reviewed revision. They are typed as `OpeningBlocker`,
they are filed upstream as one capability the target lacks
(`obs:upstream:eg-020`), and they are what every deferral below cites:

1. `GeneratorNotDerivableOnScript` — the asset-generator recipe needs two
   curve maps and a point addition, and the reviewed language performs
   neither, so a program cannot derive the generator it would have to open
   against.
2. `EncodingDomainMismatch` — the confidential encodings record whether
   the y coordinate is a quadratic residue, while the curve-checking
   primitives accept only the compressed public-key prefixes, which record
   whether y is odd. No reviewed primitive converts between the two
   conventions.
3. `SuppliedParityUnbound` — a program can assemble an operand from an
   exposed x coordinate, but the parity byte it supplies is bound to
   nothing, so the relation holds for the point or for its negation.

Two of the three are parity facts, which is why the candidate proof
outline as instantiated on the reviewed primitives is refused outright
rather than merely postponed: a candidate that loses parity is rejected.
The class is deferred and not rejected, because a pattern proving the
normalization or negation the criterion asks for was not found — which is
not the same as shown impossible.

### Final representation policy matrix

Each cell states the status of the representation named by its column for
the use named by its row, about evidence that landed in this batch.

- **supported directly** — a transaction in that representation
  performing that use was constructed and accepted by a real node.
- **supported through normalization** — the same, reached only after an
  owner-authorized representation transition.
- **explicit only** — the policy for that use requires explicit
  representation, and no prototype of the use itself exists to say more.
  The uses in question are protocol operations, and operations are owned
  by later phases.
- **unsupported** — an attempt was made and could not be constructed.
- **deferred with named blocker** — the three blockers above.
- **not applicable** — the guide's own pre-filled cell.

| Object/use | Explicit | PublicCommitted | Private direct | Normalize first |
|---|---|---|---|---|
| lateral value transfer | supported directly | deferred with named blocker | supported directly | not applicable |
| public ownerless maintenance object | explicit only | deferred with named blocker | forbidden | not applicable |
| owner-authorized amount-dependent boundary | supported directly | deferred with named blocker | deferred with named blocker | supported through normalization |
| permissionless future maintenance | explicit only | deferred with named blocker | forbidden | not applicable |
| formula-bound payout boundary | explicit only | deferred with named blocker | deferred with named blocker | explicit only |
| full private input consumption | unsupported | deferred with named blocker | deferred with named blocker | unsupported |
| partial private consumption with private change | supported directly | deferred with named blocker | deferred with named blocker | supported through normalization |

Cell by cell, against landed evidence:

- **lateral value transfer.** Explicit rests on conservation row 1,
  explicit to explicit, accepted. Private direct rests on conservation
  row 2, confidential to confidential, accepted: a private lateral
  transfer is target-native confidential value and needs no boundary at
  all. The public-committed cell is the general deferral; the
  normalize-first cell is the guide's own `not applicable`, and it is
  right, because a lateral transfer has no public side to reach.

- **public ownerless maintenance object.** The private-direct cell is
  forbidden by the guide itself and is not a finding of this batch. The
  explicit cell is `explicit only` rather than `supported directly`: the
  fresh-process lifecycle proof did show an unrelated process locating,
  parsing, and reading an explicit output from public chain data alone,
  but it did not maintain that object — the second process could not
  spend it and spent its own funds. No maintenance operation exists yet
  to evidence more.

- **owner-authorized amount-dependent boundary.** The explicit cell is
  the accepted explicit output of conservation rows 1 and 4. The
  normalize-first cell is the wave's substantive result: the
  private-to-explicit-with-private-change variant is constructible, was
  built, and its nine-row mutation matrix agreed with expectations
  written and committed before the run. Both private cells are deferred
  on the three blockers.

- **permissionless future maintenance.** Same shape as public ownerless
  maintenance, and for the same reason: the lifecycle proof establishes
  that public chain data suffices to locate, parse, and verify an
  explicit object without any of the original owner's private material,
  and it does not establish that a later party can maintain the object.

- **formula-bound payout boundary.** No payout operation exists to
  construct, so no cell can claim a built use. The policy is that the
  boundary reads an explicit value; where the source is private, the
  reaching path is normalization first, and the representation transition
  that path needs is the evidenced one. The operation itself belongs to
  Guide 12 and later.

- **full private input consumption.** The one earned `unsupported` in the
  matrix, and it is a constructibility finding rather than a target
  verdict: conservation row 5, several confidential inputs paying a single
  explicit output, could not be built at all. Residual blinding has
  nowhere to go without a blinded output to absorb it, and the node says
  so — it asks for another output to blind. The target was never asked to
  judge the row, so nothing here is a rejection by the target. The
  normalize-first cell is the same construction and fails the same way:
  normalization to an explicit-only result is not available, and the
  constructible variant carries private change.

- **partial private consumption with private change.** The explicit cell
  is conservation row 4, accepted. The normalize-first cell is the nine
  row matrix again. This row and the owner-authorized boundary row are
  the two the selected policy actually rests on.

### Evidence

- Target review of the confidential-value machinery: the commitment
  relation with both terms positive and a thirty-two-byte big-endian
  opening scalar, the two parity conventions, and the three opening
  blockers. Findings `G11-C01`, `G11-C02`, `G11-C03`.
- Independent commitment oracle: sixty-eight upstream low-level vectors
  reproduce exactly, and the oracle predicts the same commitment bytes a
  real node produced for three observed openings. Findings `G11-O01`
  through `G11-O04`.
- Conservation matrix: twelve rows stated, eleven executed against a real
  node, one deferred as a typed row. Ten executed rows agreed with
  expectations written before the target was asked; the eleventh, row 5,
  reached no verdict because it could not be constructed. Findings
  `G11-W7-01` through `G11-W7-07`.
- Candidate dispositions and typed disclosure reasons. Findings
  `G11-W8-01` through `G11-W8-06`.
- Normalization prototype: the claim, the nine-row mutation matrix run
  against a real node with 9/9 agreement, and the typed safety report.
  Findings `G11-W10-01` through `G11-W10-03`.
- Fresh-process lifecycle: a real operating-system process boundary, a
  typed public handoff schema, and 18/18 row agreement over two passes —
  all nine rows the canonical matrix states, twice. Findings
  `G11-W11-01` through `G11-W11-06`.

The report roles are `Experimental` throughout. They establish what the
target does with a matrix, and nothing about a candidate being selected;
selection is this section's.

### Open residuals

- `G11-W11-06` — closed. The stale-evidence row now builds and both
  passes answer with the refusal it expects. The cause was a digest the
  wallet computed over an output-witness vector shorter than the one the
  wire form carries, and the adapter's superseding spend had not in fact
  been blinded despite being paid to a confidential address. Recorded in
  the backlog; the upstream half is drafted there for the register that
  owns it.
- `G11-H02` — issuance observations carry authority fields the
  realization evaluator does not yet enforce. This stands as a constraint
  rather than a finding of this note: no issuance realization scope may be
  added until it is enforced or externally evidenced.
- Policy-resource evidence for these paths is `UnresolvedByDesign`. The
  measurement table below is not filled, because the paths that would fill
  it are the deferred ones; the explicit path's cost is an ordinary
  transaction cost and carries no opening verifier to measure.
- `G11-W7-08` — the conservation lane records and does not gate. The
  typed report exists and is tested; the executor driver, the gate, the
  published asset, and the Meson target that would make the lane
  refusable in CI are not built.
- `G11-W7-09` — both public-committed rows of the conservation matrix
  remain deferred, one of them executed only in its explicit form.

## Handoff · `sec:public-opening:handoff`

The operation-specific initial matrix, projected from
(`sec:public-opening:result`). Operation names are owned by later phases,
so each cell states the representation policy that operation inherits, not
a built operation:

| Operation | Explicit | Private direct | Normalize first |
|---|---:|---:|---:|
| live transfer | supported directly | supported directly | n/a |
| burn | explicit only | deferred with named blocker | supported through normalization |
| compact ASH | public only | n/a | n/a |
| clear | public only | n/a | n/a |
| redemption | explicit only | deferred with named blocker | supported through normalization |

Live transfer is the one operation whose representation question this
batch answers with a built transaction on both sides. Burn and redemption
inherit the explicit boundary and the normalization path that reaches it;
neither operation is constructed here.

The remaining handoff — typed representation alternatives, target
capability status, backend patterns, transaction ABI, vectors, client
claims, calibration, and release reports — is consumed rather than
reopened by the next guide, which builds the first complete
compiler-to-target operation.
