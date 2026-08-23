# Draft: Guide 13 — End-to-End Live Receipt Transfer

> **Status:** Draft execution guide; not yet executed
> **Phase:** Phase 5 — Live Receipt Transfer
> **Entry:** recorded Phase-4 exit, (`gate:phase4:exit`), plus the preflight gate below
> **Primary typed operation:** `architecture::OperationId::TransferLive`
> **Human operation name:** `transfer-live-receipts`
> **Affected packages:** `realization`, `compiler`, `target-elements`, `tapscript`, `linker`, `transaction`, `vectors`; `target-elements-conformance` only for target-generic executor mechanics
> **Supersedes as execution direction:** the Guide-13 concept draft
> **Does not implement:** burn, clear, redemption, STATE, RESV, cycle, settlement, production wallets, production private-key custody, production blinding-factor custody, production multi-party signing or blinding, final calibration, production deployment, or release
> **Required result:** one complete candidate live-receipt transfer pipeline with exact relation preservation, every-owner authorization, live-class and closed-asset closure, explicit split/merge conservation, a separately assessed private-committed plan, real target execution, validated safety evidence, separate disclosure-minimality evidence, exact resource comparison, and explicit lifecycle and production non-claims
> **Authority:** Attestation, Realization, typed architecture, implemented ADRs, accepted decisions, the accepted Guide-11 representation policy, and the recorded Phase-4 result take precedence
> **Review basis:** two static passes over the supplied 154-file selection at tree `0.4.2-dev`; every review finding below remains a hypothesis until reproduced against the complete working tree
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015

---

## Mission · `sec:guide13-exec:mission`

Guide 13 extends the complete compiler-to-target path from compact ASH—the first ownerless, public, root-free operation—to the first owner-authorized and representation-parametric operation:

```text
typed architecture
    ↓
target-independent live-transfer realization
    ↓
validated compiler analysis
    ↓
validated live-transfer target-operation plan
    ↓
reviewed target assessment
    ↓
representation-specific tapscript plans
    ↓
candidate relocatable bundle
    ↓
deterministic linking
    ↓
candidate transaction and witness ABI
    ↓
finalized multi-owner signing flow
    ↓
complete Elements transactions
    ↓
real target execution
    ↓
accepted semantic projection
    ↓
separate safety and disclosure-minimality evidence
```

The semantic operation consumes one or more live receipts and creates one or more live receipts:

```text
RECEIPT_L inputs
    ↓
RECEIPT_L outputs
```

It may split, merge, redistribute, and change physical denominations and owners, provided that:

- every consumed receipt owner authorizes the same finalized protected transaction;
- every protocol input and output remains in the live receipt class;
- every protocol input and output carries explicit closed asset `U`;
- aggregate semantic `U` value is conserved exactly;
- every closed-asset-capable position is classified;
- no ASH, time-locked receipt, vault, control, entitlement, bare `U`, root, or other undeclared protocol object appears;
- no issuance or destruction occurs;
- no specialized event is projected;
- sponsor value remains isolated and opaque;
- each selected value representation preserves the same target-independent relation;
- lifecycle incompleteness remains visible.

Phase 5 introduces two evidence questions that must remain separate:

```text
LiveTransferSafety

LiveTransferDisclosureMinimality
```

Safety asks whether an invalid transfer can pass. Minimality asks whether a supported lower-disclosure representation can perform the same valid semantic transfer. Neither result may satisfy the other.

---

## One-line thesis · `rem:guide13-exec:thesis`

> Guide 13 succeeds when every consumed live-receipt owner authorizes one finalized, class-closed and closed-asset-rigid transfer; explicit split and merge preserve exact semantic value; any admitted private-committed plan preserves the same relation without exact amount inspection; sponsor value remains erased; target acceptance and semantic agreement are checked separately; and safety, minimality, lifecycle, resource, and production claims remain distinct.

---

# 1. Governing rulings · `sec:guide13-exec:rulings`

## 1.1 Consume the Phase-4 pipeline · `rule:guide13-exec:consume-phase4`

Guide 13 consumes rather than redesigns without a concrete need:

- the validated compiler target-operation boundary;
- abstract target requirement projection;
- typed target capability assessment;
- typed tapscript pattern records;
- candidate relocatable bundles;
- typed linker symbols, references, and relocations;
- deterministic taptree construction;
- candidate linked-bundle state;
- candidate transaction ABI framework;
- canonical versus experimental evidence subjects;
- target-generic operation execution;
- process-group supervision and timeout handling;
- exact target/deployment transcript binding;
- executor environment and provenance records;
- strict response-shape validation;
- resource prediction and target observation comparison;
- candidate-versus-final type separation.

A Phase-4 interface changes only when live transfer presents a fact it cannot express. Each generalization records:

1. the concrete missing fact;
2. why the Phase-4 type cannot carry it;
3. the smallest sufficient extension;
4. compact-ASH regression impact;
5. identity and schema impact;
6. focused compatibility tests.

No Phase-4 boundary is replaced merely to make the live-transfer implementation look independent.

## 1.2 No relation disappears · `rule:guide13-exec:relation-census`

For the exact live-transfer scope, require equality among:

```text
realization relation census
=
compiler analyzed relation census
=
compiler target-operation-plan relation census
=
backend selected-proof relation census
=
backend emitted-placement relation census
=
linked reachable-carrier relation census
=
ABI relation census
=
safety evidence relation census
=
minimality evidence relation census where minimality applies
```

Every equality is duplicate-sensitive and checked in both directions.

A package may add target-owned structural obligations. It may not:

- remove a semantic relation;
- merge two relation identities into one row;
- invent a relation;
- silently weaken authorization;
- silently widen representation support;
- omit an unresolved lifecycle exit;
- convert external target evidence into a local backend claim;
- discharge evidence from caller-authored status values.

## 1.3 Value-parametric and asset-rigid · `rule:guide13-exec:value-parametric`

Guide 13 implements D005 directly:

```text
semantic value:
    representation-parametric where supported

closed protocol asset identity:
    explicit and rigid at every seam
```

Every live-receipt input and output carries explicit `U`.

No confidential asset commitment may carry `U`.

No unclassified output may carry `U`.

No value-conservation proof—including target Confidential-Transaction conservation—substitutes for exact closed-asset, object-family, class, or recipient closure.

## 1.4 Target acceptance and semantic agreement remain separate · `rule:guide13-exec:two-verdicts`

A target accepting a transaction proves only that the target accepted those bytes under the observed environment.

It does not prove that:

- the transaction represented live receipt transfer;
- every input owner was the intended owner;
- every output was a live receipt;
- every `U` output was classified;
- every semantic relation was retained;
- the accepted output multiset matched the request;
- the report’s relation and case censuses were complete.

Every accepted target transaction receives an independent semantic-projection comparison.

The successful condition is:

\[\text{target accepted} \land \text{observed projection}=\text{expected live-transfer projection}\]

Acceptance without a comparison does not discharge coverage. A comparison that differed records a semantic finding and does not discharge coverage.

## 1.5 Safety and minimality remain separate · `rule:guide13-exec:two-evidence-axes`

The safety question is:

```text
Can an invalid asset, class, owner, authorization, value,
constructor, family, output set, or sponsor relation be accepted?
```

The minimality question is:

```text
Can a supported lower-disclosure representation perform the same
semantic transfer without adding disclosure the relation does not need?
```

Safety requires focused accepting and rejecting evidence.

Minimality requires paired accepting evidence and equal semantic projections.

The following implications are prohibited:

```text
explicit transfer is safe
    ⇏ explicit transfer is disclosure-minimal

private transfer is accepted
    ⇏ confidential closed-asset identity is safe

target CT conservation holds
    ⇏ every owner authorized

one-to-one private transfer works
    ⇏ private split works

private split works
    ⇏ private merge works

all malformed private cases reject
    ⇏ any valid private case exists

same target verdict
    ⇏ same semantic projection
```

## 1.6 Every owner authorizes one finalized transaction · `rule:guide13-exec:every-owner`

Let \(I\) be the consumed live-receipt inputs and let \(\operatorname{owner}(i)\) be the owner authenticated from input \(i\).

Every accepted transfer satisfies:

\[\{\operatorname{owner}(i)\mid i\in I\}\subseteq \operatorname{Authorized}(T)\]

where \(T\) is the exact finalized target transaction.

Two levels remain distinct:

```text
semantic authorization:
    every distinct consumed owner is represented

target realization:
    every consumed receipt input satisfies its own spending predicate
```

One owner may control several receipt inputs. The semantic owner set may contain that owner once, while the target realization may require one signature per input because the target message is input-specific.

Reports distinguish:

```text
distinct semantic owners
concrete receipt inputs
concrete owner signatures
```

Three signatures from one owner are not three independent owners.

## 1.7 Signature commitment is exact · `rule:guide13-exec:signature-commitment`

No protocol-owner signing request exists until all protected data is fixed:

- receipt inputs;
- destination entries;
- destination owners;
- destination semantic values;
- explicit values or confidential commitments;
- receipt constructors;
- representation plan;
- sponsor inputs where the profile commits them;
- sponsor change;
- target fee role;
- rangeproof and surjection-proof fields covered by the selected digest;
- transaction version;
- locktime;
- issuance fields, including their absence;
- relevant taproot script-path fields.

The initial candidate uses an all-inputs, all-outputs profile unless a narrower profile is separately proved to preserve every required commitment.

The selected profile is observed from the finalized witness or independently recomputed from the exact signing request. It is not accepted merely because the builder says which profile it intended.

## 1.8 Unknown-key forward compatibility is closed · `rule:guide13-exec:owner-key-encoding`

The reviewed signature primitive may succeed without verification for an unrecognized nonempty public-key encoding.

Therefore every live-receipt constructor and owner-authorization pattern independently authenticates the approved owner-key encoding before relying on signature success.

An accepted owner check requires both:

```text
owner key has the approved encoding
∧
signature verifies against that key
```

A forward-compatibility success path is not authorization.

Required negative cases include:

- empty key;
- unknown nonempty key type;
- malformed approved key;
- approved key belonging to another owner;
- valid signature against another key;
- valid signature over another transaction.

## 1.9 Sponsor opacity survives owner authorization · `rule:guide13-exec:sponsor-opacity`

Protocol-owner signatures do not make sponsor values protocol data.

No compiler plan, target program, ABI, canonical report, or future release identity may require an individual sponsor amount to be:

- explicit for protocol use;
- decoded;
- opened;
- compared with zero;
- proved positive;
- publicly aggregated;
- emitted in a diagnostic;
- emitted in a canonical report;
- included in semantic identity.

The sponsor relation remains:

```text
exact sponsor-region membership
+
protocol/sponsor disjointness
+
sponsor-owner target authorization
+
at most one sponsor envelope
+
whole-transaction target conservation
+
independent enforcement of the complete U transfer relation
```

The balanced-theft mutation is mandatory:

```text
decrease one protocol receipt output
increase sponsor change by the same amount
preserve total target balance
keep every sponsor amount positive
```

The transaction must fail because the `U` transfer relation is wrong, not because a sponsor amount failed a positivity check.

## 1.10 Public test material is not a production secret interface · `rule:guide13-exec:test-material`

Private-committed transfer tests may use cryptographically secret-shaped material that is public disposable test data under ADR-015:

- fixed owner scalars;
- fixed input openings;
- fixed value blinders;
- fixed nonce seeds;
- fixed rangeproof seeds;
- deterministic test signing nonces.

Such material must:

- authorize nothing on a production network;
- be clearly labeled test-only;
- be confined to fixtures or disposable development state;
- be reproducible from explicit test inputs where the selected materializer permits;
- never derive from production material;
- never be accepted through a production-capable interface.

Guide 13 does not introduce a first-party production interface for:

- owner private keys;
- production signing nonces;
- production input openings;
- production output blinders;
- wallet seeds;
- node credentials;
- production multi-party blinding.

Any such interface requires the separate ADR-015 future-secret design before it lands.

## 1.11 Construction failure, infrastructure failure, and target rejection remain distinct · `rule:guide13-exec:failure-layers`

Reports distinguish at least:

```text
semantic-request rejection
compiler-plan rejection
backend-emission rejection
linker rejection
ABI/construction rejection
signer refusal
confidential-materialization refusal
executor infrastructure failure
consensus rejection before script
script-path rejection
relay-policy rejection
accepted transaction
report-layer semantic-projection rejection
report-layer disclosure-minimality rejection
```

No failure layer is inferred from the expected result.

A target-negative claim requires a complete target transaction and an observed target verdict.

## 1.12 Candidate and final states remain distinct · `rule:guide13-exec:candidate-state`

Guide 13 may construct:

```text
ValidatedLiveTransferOperationPlan
CandidateLiveTransferTapscriptPlan
CandidateRelocatableLiveTransferBundle
CandidateLinkedLiveTransferBundle
CandidateLiveTransferAbi
ValidatedCandidateLiveTransferSafetyReport
ValidatedCandidateLiveTransferMinimalityReport
```

It must not construct or claim:

```text
FinalLiveReceiptConstructor
FinalLinkedBundle
FinalTransactionAbi
ProductionConfidentialTransfer
ProductionSignerInterface
ProductionWalletInterface
ValidatedDeploymentRelease
```

The live-receipt candidate remains lifecycle-incomplete because burn and redemption are absent.

## 1.13 No speculative identity · `rule:guide13-exec:no-speculative-identity`

Typed in-process values use exact typed comparison.

Exact target bytes use exact byte comparison when the bytes themselves are the subject.

Guide 13 mints no:

```text
LiveTransferPlanHash
LiveReceiptConstructorHash
LiveTransferBundleHash
LiveTransferAbiHash
SafetyReportHash
MinimalityReportHash
```

Any proposed digest must first pass the identity-adjudication procedure adopted by ADR-021, including a present consumer and a distinct decision.

No candidate type reserves a future digest field.

## 1.14 No external interchange publication by implication · `rule:guide13-exec:interchange-boundary`

Phase-5 typed reports and repository-internal report assets are not automatically externally consumed interchange documents.

If Guide 13 introduces a document intended for consumption outside the repository by a reader that did not invoke the producing command, that document becomes ADR-022’s first consumer and must carry the registry, namespace allocation, encoder, decoder, versioning, and acceptance work activated there.

No report becomes an ADR-022 document merely because JSON is convenient.

---

# 2. Entry conditions · `sec:guide13-exec:entry`

## 2.1 Phase-4 handoff · `gate:guide13-exec:phase4-entry`

Before live-transfer implementation begins:

- the Phase-4 gate record is located and checked against the current tree;
- active phase indexes, roadmap, package contracts, and backlog agree on the Phase-4 result and Phase-5 activation;
- compact ASH still builds and its focused regressions pass;
- the compiler target-operation boundary remains validated;
- tapscript patterns remain typed;
- linker and transaction candidate boundaries remain available;
- exact target/deployment transcripts remain retained;
- operation reports cannot be forged from caller-authored observations;
- candidate and final states remain distinct;
- the tree is clean.

A historical Phase-4 completion record is evidence about its named tree, not a green claim about the current checkout.

## 2.2 Semantic entry · `gate:guide13-exec:semantic-entry`

Required:

- typed architecture validation passes;
- `OperationId::TransferLive` remains in architecture and realization scope;
- the live-transfer realization validates against architecture;
- compiler analysis remains complete for live transfer;
- realization and compiler relation censuses agree;
- explicit and private-committed value modes remain realization-approved;
- explicit closed `U` remains mandatory;
- sponsor-value opacity remains enforced;
- transfer, burn, and redemption lifecycle obligations remain explicit;
- no target-specific type enters realization or compiler relation identity.

## 2.3 Target entry · `gate:guide13-exec:target-entry`

Required:

- reviewed target definition validates;
- development binding is welded to that exact target projection;
- owner-key encoding rules are reviewed;
- signature success and failure behavior is reviewed;
- the selected sighash profile has target-native evidence;
- input/output asset and program introspection is reviewed;
- explicit value introspection is reviewed;
- CT conservation is retained as external target evidence;
- confidential transaction materialization is available if the private plan is attempted;
- execution domain and leaf version are observed;
- consensus and relay-policy verdicts remain separate;
- no mock is gate-eligible.

## 2.4 Evidence entry · `gate:guide13-exec:evidence-entry`

Required:

- canonical subjects cannot be caller-forged;
- coverage cannot be discharged from caller-authored tuples;
- operation reports bind exact target, deployment, requests, responses, and executor provenance;
- executor requests contain no expected result;
- all response types reject role- and verdict-contradictory records;
- every failed canonical case fails its owning gate;
- infrastructure failures carry no target observations;
- protocol records are strict and bounded in both directions;
- canonical report bytes exclude ambient timing and host data;
- safety and minimality use separate validated report types.

## 2.5 Repository entry · `rule:guide13-exec:repository-entry`

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

Record:

```text
starting revision
working-tree status
Phase-4 result revision
architecture schema and semantic identity
realization version
target contract revision
native protocol revision
compact-ASH candidate bundle/ABI status
live-transfer compiler projection
```

If the tree is not clean, stop and classify every change.

---

# 3. Preflight review register · `tab:guide13-exec:preflight`

The two static reviews of tree `0.4.2-dev…` supplied the following hypotheses. They are not current-tree facts until reproduced.

| ID | Priority | Finding | Required disposition |
|---|---:|---|---|
| `G13-R01` | P0 | Canonical coverage can be discharged from caller-authored `Accepted + Matched` tuples or caller-authored mutation refusals, without a validated target transcript. | Make discharge consume a validated operation report; make tuple-level helpers non-public. |
| `G13-R02` | P1 | The transaction decoder accepts a witness-flagged transaction whose entire witness section is empty, then re-encodes it differently. | Reject superfluous witness records and require exact successful decode/re-encode equality. |
| `G13-R03` | P1 | A resource-prediction mismatch is recorded in the planner transcript but does not return `PlanRefused`; execution may return success while the report records refusal. | Propagate mismatch as an actual planner and executor failure; emit no later step. |
| `G13-R04` | P1 | `CandidateShapeSet::new` accepts shapes outside its declared bounds; emitted specializations and advertised bounds can disagree. | Make construction fallible and validate every member against the exact bounds. |
| `G13-R05` | P1 | Public shape constructors admit total input counts that overflow the `u8` sponsor-range arithmetic. | Use a consistent wider index domain or reject totals above the admitted index domain. |
| `G13-R06` | P1 | The Python executor logs raw child stderr and caller-supplied framework paths despite ADR-010’s omission policy. | Remove raw child text and argv-derived paths from first-party diagnostics; add marker tests. |
| `G13-R07` | P2 | The taproot constructor oracle rejects a zero tweak even though \(Q=P+0G=P\) is valid. | Accept zero below the group order and retain the separate identity-result check. |
| `G13-R08` | P2 | The “exact” taptree construction and oracles use saturating `u64` arithmetic. | Use checked or exact arithmetic and return typed overflow, never a saturated optimum. |
| `G13-R09` | P2 | Active phase, package, and backlog documents disagree about the implemented Phase-4 state. | Reconcile active owners without rewriting archived history; extend plan checks. |
| `G13-R10` | P0 | The canonical class `sponsor-change-present` is materialized with sponsor change absent and can still receive a matching projection. | Add sponsor-change presence to semantic fixtures, materialization, projections, and class witnesses. |
| `G13-R11` | P1 | `EQUAL; VERIFY` over known unequal literals can retain an impossible abstract success path because computed Boolean values lose literal knowledge. | Propagate exact computed Boolean facts and add composed-verification regressions. |
| `G13-R12` | P1 | The live compact-ASH lane discards the subject-bound generic execution transcript and publishes an unvalidated, timing-dependent summary. | Introduce a validated operation report binding the generic transcript to the operation-specific result; remove timing from canonical bytes. |
| `G13-R13` | P1 | A request marked sponsored can silently select and return the sponsorless form when the supplied sponsor capability offers no inputs. | Require exact request/capability/offer/shape-form equivalence; reject an empty sponsor offer. |
| `G13-R14` | P1 | Lifecycle and operation response validators admit role- or verdict-contradictory records. | Validate exact allowed fields for every role and outcome. |
| `G13-R15` | P2 | Duplicate taptree declarations use last-value-wins, making conflicting weight or role resolution declaration-order-dependent. | Reject duplicate leaves and derive program role from leaf role. |
| `G13-R16` | P2 | Duplicate public-output views and sponsor inputs are silently collapsed. | Make constructors fallible and reject duplicate or conflicting outpoints. |
| `G13-R17` | P2 | Evidence-critical `ALL` arrays cannot prove enum completeness; omitted variants may vanish from “complete census” APIs. | Generate enums and censuses from one source, or narrow the completeness claim. |
| `G13-R18` | P2 | The Python side of protocol revision 4 skips blank input records, accepts handshake extras, and has no record-size bound. | Implement strict bounded request-side framing and cross-language malformed-record tests. |

## 3.1 Preflight disposition · `rule:guide13-exec:preflight-disposition`

Each row receives one reproduction status:

```text
CONFIRMED
REFUTED
RECLASSIFIED
```

and one execution status:

```text
DONE
OPEN with named blocker
```

“Existing tests pass” does not close a row unless a focused test reaches the reported branch.

## 3.2 Blocking rule · `gate:guide13-exec:preflight`

No live-transfer compiler, constructor, backend, ABI, or evidence implementation begins while a confirmed P0 or P1 row remains open.

A confirmed P2 row may proceed only until the package boundary it affects:

- `G13-R07`, `G13-R08`, and `G13-R15` close before linker reuse;
- `G13-R17` closes before the live-transfer compiler-plan census is accepted;
- `G13-R18` closes before any real target execution;
- `G13-R09` closes before Phase 5 is described as active.

All confirmed rows close before the Guide-13 exit gate.

---

# 4. Phase-4 evidence handoff repair · `sec:guide13-exec:phase4-handoff`

Guide 12 ended with negative coverage at 1 of 72 and named three guide-level gaps. Guide 13 must not inherit them silently while claiming complete relation-indexed coverage.

## 4.1 Typed mutation links · `rule:guide13-exec:typed-mutation-links`

Every canonical negative vector states:

- exact source semantic fixture;
- exact relation intended to be violated;
- exact compiler mutation class;
- exact changed semantic and target fields;
- expected evidence boundary;
- required carrier;
- dependency collateral;
- representation and sponsor case.

A class name alone is navigation and never evidence.

Plan derivation resolves each declaration against the compiler coverage plan and requires exactly one matching requirement.

An underdetermined vector remains experimental or blocked. It does not receive a guessed relation.

## 4.2 First-party negative evidence · `rule:guide13-exec:first-party-negatives`

The execution guide records one explicit policy for compiler-static and backend-structural negative requirements.

A first-party negative row is discharged only by:

- a canonical malformed typed input;
- the exact owning first-party validator;
- a typed refusal naming the intended class;
- a focused positive control;
- a focused negative test;
- a validated first-party evidence report.

A refusal from another layer does not satisfy it.

If this policy is rejected, those requirements are explicitly outside the target-execution coverage denominator rather than left permanently outstanding inside it.

## 4.3 ABI validation role · `rule:guide13-exec:abi-validation-role`

Guide 13 introduces or confirms a typed ABI-validation result for classes whose boundary precedes target execution.

Such a result distinguishes:

```text
safe constructor refused
unsafe raw mutation exists
target was not asked
```

A pre-target class is never submitted merely because a target executor is available.

## 4.4 Validated operation report · `rule:guide13-exec:validated-operation-report`

Coverage discharge consumes a validated report whose private constructor binds:

- target projection;
- deployment projection;
- exact executor handshake;
- observed environment;
- executor trust declaration;
- expected provenance;
- exact sent operation requests;
- exact operation responses;
- operation-specific semantic fixtures;
- accepted bytes;
- target-observed input coins;
- independently recomputed semantic projections;
- resource comparisons;
- report summary.

The low-level APIs accepting raw:

```text
ObservedOutcomeLayer
ProjectionComparison
NegativeMutation
```

are crate-private test machinery.

No external caller can create canonical observed coverage by choosing those enums.

---

# 5. Semantic live-transfer contract · `sec:guide13-exec:semantics`

## 5.1 Inputs and outputs · `def:guide13-exec:operation`

Let a transfer consume \(n\) live receipts and create \(m\) live receipts:

\[1\le n\le N_{\mathrm{in}}\]

\[1\le m\le N_{\mathrm{out}}\]

where \(N_{\mathrm{in}}\) and \(N_{\mathrm{out}}\) are candidate assignments for architecture-owned bounds.

Each input \(i\) carries:

```text
object:
    RECEIPT_L

asset:
    explicit U

owner:
    oᵢ

semantic amount:
    xᵢ > 0

representation:
    selected explicit or private-committed plan
```

Each output \(j\) carries:

```text
object:
    RECEIPT_L

asset:
    explicit U

owner:
    pⱼ

semantic amount:
    yⱼ > 0

representation:
    selected explicit or private-committed plan
```

The exact semantic conservation relation is:

\[\sum_{i=0}^{n-1}x_i=\sum_{j=0}^{m-1}y_j\]

## 5.2 Transfer freedom · `rule:guide13-exec:transfer-freedom`

The request may choose:

- destination owners;
- output count within the admitted bound;
- output denominations;
- physical grouping;
- an admitted representation plan.

It may not change:

- aggregate semantic value;
- asset `U`;
- live class;
- required owner authorization;
- root state;
- issuance;
- destruction;
- specialized event semantics.

## 5.3 Split and merge · `rule:guide13-exec:split-merge`

The semantic relation admits:

```text
one-to-one
one-to-many split
many-to-one merge
many-to-many redistribution
```

provided every input and output relation holds.

No implementation claims general split or merge support from one-to-one evidence alone.

## 5.4 Live-class closure · `rule:guide13-exec:live-closure`

Every accepted protocol input and output is `RECEIPT_L`.

Forbidden as transfer inputs or outputs:

```text
RECEIPT_T
ASH
DISTRIBUTION_VAULT
DISTRIBUTION_CONTROL
DEPOSIT_ENTITLEMENT
DEPOSIT_REQUEST
STATE
RESV
bare-key U
foreign object carrying U
```

Class is authenticated from the constructor, not inferred from amount, owner, transaction position, or selected leaf alone.

## 5.5 Owner authorization · `rule:guide13-exec:owner-authorization`

For each consumed receipt input:

- predecessor constructor is authenticated;
- owner metadata is canonical;
- owner key uses the approved encoding;
- owner signature is present;
- signature verifies;
- signature binds the finalized protected transaction;
- empty, malformed, unknown-type, wrong-owner, and wrong-transaction signatures reject.

## 5.6 Canonical `U` flow · `rule:guide13-exec:u-flow`

The one canonical protocol flow is:

```text
asset:
    U

kind:
    lateral

sources:
    every live receipt input exactly once

destinations:
    every live receipt output exactly once

issuance:
    none

destruction:
    none
```

No source or destination participates in another canonical flow, sponsor flow, issuance, or destruction.

Aggregate balance alone is insufficient: family membership, class, owner routing, and destination closure remain independent relations.

## 5.7 Roots and projections · `rule:guide13-exec:roots-projections`

Every architecture root is forbidden.

Required semantic projection:

```text
transition certificate
```

Forbidden specialized projections:

```text
burn
clear
distribution residue
```

No burn record, ASH, destruction, or authority object appears.

## 5.8 Sponsor flow · `rule:guide13-exec:sponsor-flow`

The only open flow is an optional fee-sponsor envelope.

The `U` transfer never funds the L-BTC fee.

Sponsor roles may include:

- ordinary L-BTC sponsor inputs;
- optional sponsor change;
- target fee output.

The semantic projection retains sponsor membership, sponsor-change presence, and the sponsor relation’s verdict where those are part of the vector class. It retains no individual sponsor amount.

---

# 6. Representation plans · `sec:guide13-exec:representations`

## 6.1 Representation vocabulary · `def:guide13-exec:representation-plan`

The Phase-5 representation-plan type has exactly the admitted modes:

```rust
pub enum LiveTransferRepresentationPlan {
    Explicit,
    PrivateCommitted,
}
```

`PublicCommitted` remains absent while Guide 11 leaves authenticated public opening deferred.

Mixed explicit/private representation is absent unless the preflight admits it explicitly.

## 6.2 Explicit plan · `candidate:guide13-exec:explicit-plan`

The explicit plan uses:

- explicit `U` asset;
- explicit receipt input values;
- explicit receipt output values;
- checked aggregate target arithmetic;
- exact live-class and constructor closure;
- per-input owner authorization;
- optional sponsor envelope.

The coordinator computes input and output sums independently and verifies equality.

Every arithmetic success flag is consumed immediately.

Every amount remains within the semantic domain.

## 6.3 Private-committed plan · `candidate:guide13-exec:private-plan`

The private plan uses:

- explicit `U` asset;
- confidential receipt input values;
- confidential receipt output values;
- target CT conservation as external target evidence;
- complete input and output family closure;
- complete live-class closure;
- per-input owner authorization;
- optional sponsor envelope.

No local target program claims to implement CT conservation merely because target consensus eventually accepts the transaction.

The private relation is sound only when:

```text
every U input is one live receipt source
every U output is one live receipt destination
no U issuance exists
no U destruction exists
no unclassified U output exists
target CT conservation holds
```

Under those conditions, whole-transaction conservation over exact explicit asset `U` is the semantic aggregate transfer relation.

## 6.4 Private amount opacity · `rule:guide13-exec:private-opacity`

The private plan must not:

- open a receipt amount in script;
- compare a receipt amount with zero as a protocol predicate;
- publish an exact receipt amount in ordinary diagnostics;
- publish fixture openings in canonical reports;
- order outputs by private amount or blinding material;
- require a public subtotal;
- add a public equality witness duplicating target CT conservation.

Positive semantic value is established by valid object construction and target proof policy, not by disclosing the amount.

## 6.5 Mixed representation · `rule:guide13-exec:mixed-representation`

The initial scope is:

```text
homogeneous explicit:
    required

homogeneous private committed:
    required for the full Guide-13 exit where target construction succeeds

mixed:
    unsupported unless separately admitted
```

An ad hoc mixed transaction accepted by the target does not widen the ABI.

If private construction remains blocked, Guide 13 may record:

```text
explicit-only candidate
private plan deferred
```

That is a valid stopped result and not a passing full Phase-5 exit.

## 6.6 Representation equivalence · `rule:guide13-exec:representation-equivalence`

Paired explicit and private fixtures compare:

- input semantic amount multiset;
- destination owner/value multiset;
- distinct input-owner set;
- live class;
- exact explicit `U`;
- authorization result;
- absence of roots;
- absence of issuance and destruction;
- lateral flow;
- sponsor relation and shape;
- transition-certificate projection.

Target bytes, commitments, proofs, witness sizes, and resource use may differ.

---

# 7. Live-receipt constructor · `sec:guide13-exec:constructor`

## 7.1 Constructor meaning · `rule:guide13-exec:constructor-meaning`

The candidate constructor binds:

- exact explicit `U`;
- live receipt object family;
- canonical owner metadata;
- live class;
- selected representation plan;
- static transfer program set;
- target contract revision;
- leaf version;
- internal-key policy;
- candidate ABI schema.

Value remains in the target value field and is not duplicated into metadata.

## 7.2 Owner metadata · `rule:guide13-exec:owner-metadata`

Owner metadata has one canonical encoding.

It rejects:

- wrong width;
- omitted owner;
- malformed owner key;
- unknown key type;
- alternate encoding of the same key;
- owner bytes not committed by the constructor;
- key types admitted only through the target’s forward-compatibility path.

`OwnerKey` is a public authorization identity and contains no private key material.

## 7.3 Live class is structural · `rule:guide13-exec:live-class`

The live and time-locked constructors are distinct.

A live-transfer leaf cannot spend a time-locked constructor.

A time-locked output constructor cannot satisfy a live destination role.

Class is not inferred from target position, value representation, amount, or owner.

## 7.4 Lifecycle status · `rule:guide13-exec:constructor-lifecycle`

The Phase-5 candidate may contain only implemented transfer leaves.

It contains no spendable placeholder for burn, redemption, or normalization.

It records:

```text
implemented:
    transfer-live-receipts

outstanding:
    burn
    redeem

release-complete:
    false
```

## 7.5 Internal key and key path · `rule:guide13-exec:internal-key`

Use the inherited public unspendable internal-key policy after the zero-tweak and exact-oracle preflight findings are closed.

There is no:

- owner internal key;
- operator key;
- release key;
- generated-and-discarded key;
- caller-selected key;
- accepted key-path escape.

## 7.6 Constructor derivation · `rule:guide13-exec:constructor-derivation`

Each output constructor derives from:

```text
destination owner
+
live class
+
selected representation
+
linked static transfer program set
+
target constants
```

The request cannot supply arbitrary constructor bytes or control paths.

Changing owner changes the constructor deterministically.

Changing representation changes the constructor only where the selected ABI permits it.

---

# 8. Compiler target-operation plan · `sec:guide13-exec:compiler-plan`

## 8.1 Plan type · `rule:guide13-exec:operation-plan`

Expose the smallest compiler-owned validated live-transfer projection needed by target packages.

It retains:

- exact operation identity;
- architecture and realization binding;
- execution cases;
- representation plans;
- relation requirements;
- source requirements;
- constructibility;
- lifecycle obligations;
- abstract carriers;
- layout requirements;
- coverage requirements;
- capability requirements;
- external evidence roles.

It contains no:

- graph index;
- search count;
- target opcode;
- target position;
- script byte;
- filesystem path;
- environment value;
- digest.

## 8.2 Plan alternatives · `rule:guide13-exec:plan-alternatives`

Compiler core retains every feasible target-independent plan.

The Phase-5 filter selects:

```text
operation:
    transfer-live-receipts

class:
    live only

closed asset:
    explicit U

authorization:
    every input owner

representations:
    explicit
    private committed where admitted

lifecycle:
    transfer implemented
    burn and redeem outstanding
```

Mixed representation is retained only if explicitly admitted.

## 8.3 Plan validation · `gate:guide13-exec:compiler-plan`

Required regressions include:

- removed relation rejects;
- added relation rejects;
- altered owner source rejects;
- altered representation rejects;
- altered lifecycle exit rejects;
- altered carrier rejects;
- altered layout rejects;
- altered coverage rejects;
- omitted capability rejects;
- omitted evidence role rejects;
- declaration permutations produce equal projections;
- external callers cannot construct validated state;
- `ALL` censuses remain complete by construction;
- no sponsor amount can enter the projection;
- no digest field exists.

---

# 9. Target assessment · `sec:guide13-exec:target-assessment`

## 9.1 Every requirement is classified · `rule:guide13-exec:assessment`

The target adapter assesses every requirement from the validated plan.

Expected capability families include:

- authenticated object recognition;
- authenticated live-class closure;
- authenticated family cardinality;
- authenticated canonical partition;
- explicit closed-asset closure;
- owner authorization;
- output-committing sighash;
- exact explicit amount arithmetic;
- public constructibility for explicit fixtures;
- private transaction materialization;
- target CT conservation;
- sponsor isolation;
- root and projection absence.

Each remains in one explicit state:

```text
Unsupported
MissingTargetPrimitives
BackendPatternRequired
BackendStructural
ExternalEvidenceRequired
CompleteBackendPattern
```

## 9.2 Signature profile · `rule:guide13-exec:sighash-profile`

The selected profile must establish:

- all receipt inputs committed;
- all receipt outputs committed;
- output asset fields committed;
- output value or commitment fields committed;
- output programs committed;
- relevant output witnesses committed;
- issuance fields committed, including absence;
- transaction version and locktime committed;
- required script-path data committed.

Target-native tests must include:

- valid owner signature;
- missing signature;
- wrong owner;
- output changed after signing;
- input added after signing;
- output removed after signing;
- wrong sighash byte;
- signature bound to another transaction.

## 9.3 CT evidence · `rule:guide13-exec:ct-evidence`

CT conservation remains external target evidence.

No opcode, abstract stack result, local curve primitive, or commitment oracle substitutes for real complete-transaction evidence.

The independent commitment oracle may validate fixture expectations. It does not establish target acceptance.

---

# 10. Tapscript patterns · `sec:guide13-exec:patterns`

Every pattern states:

- semantic owner;
- reviewed target prerequisites;
- typed instruction fragment;
- stack precondition;
- successful states;
- non-aborting failures;
- aborting failures;
- source requirements;
- witness role;
- constructibility;
- disclosure;
- resource formula;
- positive and negative vectors.

## 10.1 Local predecessor recognition · `rule:guide13-exec:input-recognition`

Every receipt input proves:

```text
asset = exact linked U
constructor = exact linked live-receipt constructor
owner metadata = canonical
class = live
representation = selected plan
current input lies in the receipt range
```

## 10.2 Per-input owner authorization · `rule:guide13-exec:input-owner`

Every receipt input:

- reads the owner from authenticated metadata;
- authenticates the owner-key encoding;
- verifies the signature;
- uses the selected sighash profile;
- rejects empty signature;
- rejects invalid signature;
- rejects wrong key;
- rejects unknown nonempty key success;
- commits the finalized transaction.

## 10.3 Coordinator and members · `rule:guide13-exec:coordinator`

Input 0 is the coordinator.

The coordinator performs transaction-global checks:

- exact input and output counts;
- complete protocol ranges;
- live-class output closure;
- explicit `U` closure;
- representation-specific conservation;
- sponsor isolation;
- roots absent;
- issuance absent;
- destruction absent;
- specialized events absent;
- exact role order.

Every receipt input, including the coordinator, separately performs local predecessor recognition and owner authorization.

Member leaves are valid only at nonzero receipt positions.

## 10.4 Output constructor closure · `rule:guide13-exec:output-closure`

Every protocol output is:

```text
RECEIPT_L
explicit U
canonical destination-owner metadata
selected representation
linked constructor
```

No output carrying `U` remains outside the destination range.

## 10.5 Explicit conservation · `rule:guide13-exec:explicit-conservation`

The explicit coordinator computes:

\[\sum_i x_i\]

and:

\[\sum_j y_j\]

using exact checked target arithmetic.

Every addition’s success flag is consumed immediately.

The final equality is verified.

Known computed Boolean values remain exact through composed instructions, so a false equality followed by `VERIFY` has no abstract success path.

## 10.6 Private conservation · `rule:guide13-exec:private-conservation`

The private plan does not inspect receipt amounts.

It establishes complete explicit-`U`, live-class, owner, constructor, issuance, destruction, and output closure, then retains CT conservation as an external target requirement.

No private-conservation backend pattern ID is minted for target consensus behavior.

## 10.7 Sponsor isolation · `rule:guide13-exec:sponsor-isolation`

The coordinator authenticates:

- exact sponsor suffix;
- sponsor-change presence or absence;
- target fee role;
- protocol/sponsor disjointness;
- no `U` in sponsor roles;
- no second sponsor envelope;
- no sponsor amount operand.

The semantic fixture explicitly represents sponsor-change presence so that `sponsor-change-present` cannot be materialized as absent.

## 10.8 Absence relations · `rule:guide13-exec:absence`

The candidate admits no:

```text
STATE
RESV
PACE
ENT_AUTH
DIST_AUTH
issuance
destruction
burn record
burn projection
clear projection
residue projection
```

## 10.9 Final stack · `rule:guide13-exec:final-stack`

Every program:

- reaches exactly one canonical true item on success;
- checks every arithmetic and verification result;
- leaves no alternate-stack residue;
- has no non-aborting failure state satisfying final truth;
- remains within target stack, element, and validation-budget limits.

---

# 11. Linking · `sec:guide13-exec:linking`

## 11.1 Linker reuse · `rule:guide13-exec:linker-reuse`

Guide 13 extends the linker only for concrete needs:

- owner-parameterized constructors;
- representation-specific leaves;
- signature witness roles;
- explicit/private resource formulas;
- safety/minimality provenance.

The symbol and relocation model otherwise remains unchanged.

## 11.2 Symbols · `rule:guide13-exec:symbols`

Expected typed roles include:

```text
U asset
owner metadata schema
live receipt constructor
explicit coordinator program
explicit member program
private coordinator program
private member program
representation plan
target leaf version
unspendable internal key
candidate input bound
candidate output bound
candidate sponsor bound
selected sighash profile
target fee role
```

## 11.3 Representation-specific leaves · `rule:guide13-exec:representation-leaves`

Explicit and private coordinators are distinct unless one complete typed proof establishes a sound shared program.

Member leaves may be shared only if:

- predecessor recognition remains complete;
- owner authorization is identical;
- representation-specific obligations remain present;
- target-native vectors execute both modes through the shared leaf.

No attacker-selected in-script dispatch selects a semantic representation.

## 11.4 Deterministic taptree · `rule:guide13-exec:taptree`

The taptree input:

- rejects duplicate leaf declarations;
- derives program role from leaf role;
- uses exact checked cost arithmetic;
- has one stable leaf census;
- records exact leaf weights;
- applies a stable tie-break;
- enforces the declared depth policy;
- compares small instances with an exact independent oracle.

## 11.5 Carrier closure · `rule:guide13-exec:carrier-closure`

The linker compares:

```text
compiler-required carriers
backend-emitted carriers
linked reachable carriers
```

for every execution case and representation plan.

A carrier reachable only in the explicit plan does not satisfy the private plan.

An external CT requirement cannot be reassigned to a local target program.

## 11.6 Candidate linked bundle · `rule:guide13-exec:linked-bundle`

The linked candidate retains:

- exact live-transfer scope;
- exact target projection;
- exact shape bounds;
- explicit/private plan status;
- linked constructors;
- linked programs;
- selected sighash profile;
- concrete placements;
- ABI handoff;
- exact resource formulas;
- unresolved external evidence;
- outstanding burn and redemption lifecycle;
- candidate-only status.

No digest is minted.

---

# 12. Candidate transaction ABI and construction · `sec:guide13-exec:abi`

## 12.1 Input layout · `rule:guide13-exec:input-layout`

The initial input layout is:

```text
inputs 0..n-1:
    live receipt family

inputs n..n+s-1:
    optional sponsor suffix
```

Receipt and sponsor inputs are each sorted by canonical outpoint order.

Input 0 is the coordinator.

Duplicates, conflicting public views, and protocol/sponsor overlap reject before sorting.

## 12.2 Output layout · `rule:guide13-exec:output-layout`

The initial output layout is:

```text
outputs 0..m-1:
    live receipt destinations in typed request order

next optional role:
    sponsor change

final target role where required:
    fee output
```

Request order is ABI presentation order and is committed by every owner signature.

It is not semantic object identity. The semantic projection compares the destination multiset.

Private output ordering never depends on private amount, opening, or blinding material.

## 12.3 Typed request · `rule:guide13-exec:request`

A request may select:

- live receipt input outpoints;
- ordered destination entries;
- destination owners;
- destination semantic values;
- one admitted representation plan;
- optional sponsor capability;
- optional sponsor-change destination;
- explicit public test randomness where needed.

It may not select:

- predecessor owner;
- predecessor class;
- predecessor asset;
- output class;
- output asset;
- constructor bytes;
- target program;
- coordinator;
- family positions;
- witness order;
- target fee role;
- issuance;
- destruction;
- specialized projection.

## 12.4 Destination type · `def:guide13-exec:destination`

Illustratively:

```rust
pub struct LiveReceiptDestination {
    owner: OwnerKey,
    value: ProtocolAmount,
}
```

Target-specific blinding and signing material lives in separate authorized or test-only capabilities and is not part of semantic destination identity.

## 12.5 Form exactness · `rule:guide13-exec:form-exactness`

Construction requires:

```text
request asks sponsored
⇔ sponsor capability supplied
⇔ sponsor offer is nonempty
⇔ sponsored shape selected
⇔ sponsored transaction form reported
```

and:

```text
request asks sponsorless
⇒ no sponsor capability
⇒ no sponsor offer
⇒ no sponsor change
⇒ no fee-sponsor role
⇒ sponsorless shape selected
```

An empty sponsor capability never downgrades a sponsored request into the sponsorless form.

## 12.6 Output finalization · `rule:guide13-exec:finalization`

Before owner signing begins:

1. every input is fixed;
2. every destination is fixed;
3. every owner is fixed;
4. every semantic value is fixed;
5. every explicit value or commitment is fixed;
6. every output constructor is fixed;
7. sponsor change is fixed;
8. fee role is fixed;
9. target proofs covered by the sighash are fixed;
10. version and locktime are fixed.

After this boundary only witness fields explicitly excluded from the message may move.

## 12.7 Multi-owner signing · `rule:guide13-exec:multi-owner-signing`

All owners sign the same finalized protected transaction.

Construction rejects:

- missing owner;
- duplicate response;
- unexpected signer;
- wrong owner;
- wrong input;
- wrong sighash profile;
- response bound to different bytes;
- output mutation after signing;
- input extension after signing;
- output omission after signing.

No partial owner set proceeds to target submission.

## 12.8 Confidential fixture construction · `rule:guide13-exec:confidential-construction`

The selected test materializer records one construction model:

```text
central public-fixture construction
cooperative owner opening exchange
distributed blinding protocol
external wallet materializer
```

The initial expected model is:

```text
central public-fixture construction
```

That demonstrates target feasibility and semantic equivalence. It does not demonstrate a production multi-owner privacy protocol.

## 12.9 Candidate ABI status · `rule:guide13-exec:abi-status`

The result is:

```text
CandidateLiveTransferAbi
```

It is not final and carries no digest.

---

# 13. Evidence architecture · `sec:guide13-exec:evidence`

## 13.1 Canonical evidence plan · `rule:guide13-exec:evidence-plan`

The live-transfer evidence plan has private fields and no unchecked constructor.

It derives from:

- compiler coverage;
- exact linked bundle;
- exact candidate ABI;
- canonical safety mutation registry;
- canonical minimality pair registry;
- exact target and deployment;
- exact executor provenance expectation.

Ad hoc cases remain experimental.

## 13.2 Safety report · `def:guide13-exec:safety-report`

The safety report answers:

```text
Did every valid transfer preserve the exact semantic relation,
and did every required invalid transfer fail at its owning boundary?
```

It carries typed:

- target and deployment;
- candidate bundle and ABI;
- representation plan;
- relation, case, mutation, and coverage censuses;
- exact sent requests;
- exact responses;
- target verdicts;
- semantic projections;
- first-party refusals;
- resource observations;
- lifecycle obligations;
- executor provenance.

## 13.3 Minimality report · `def:guide13-exec:minimality-report`

The minimality report answers:

```text
Does the private-committed plan perform the same semantic transfer
with less exact amount disclosure than the explicit plan?
```

Each row contains:

- one semantic fixture;
- explicit materialization;
- private materialization;
- both target verdicts;
- both semantic projections;
- disclosure sets;
- constructibility assumptions;
- lifecycle status;
- resource comparison.

## 13.4 Positive private evidence is mandatory for each claimed shape · `rule:guide13-exec:private-positive`

Private support cannot be inferred from rejecting malformed private transactions.

Before claiming:

```text
private one-to-one
```

at least one valid private one-to-one transaction accepts.

Before claiming:

```text
private split
```

at least one valid private split accepts.

Before claiming:

```text
private merge
```

at least one valid private merge accepts.

Before claiming general many-to-many private transfer, a representative valid many-to-many transaction accepts and exact structural coverage spans the admitted bounds.

## 13.5 Report validation · `rule:guide13-exec:report-validation`

The safety and minimality gates accept validated report wrappers, not raw report DTOs.

Validation recomputes:

- role;
- schema;
- target/deployment binding;
- exact request and response census;
- response shape;
- case census;
- relation census;
- mutation links;
- target verdict comparison;
- semantic projection;
- disclosure comparison;
- resource comparison;
- executor provenance;
- summary.

Canonical bytes exclude:

- wall-clock time;
- elapsed time;
- hostname;
- username;
- process ID;
- temporary path;
- executor path;
- raw child stderr;
- environment values;
- test-only private fixture material not required by the report schema.

Timing belongs in a separate noncanonical diagnostic report.

## 13.6 Report separation · `rule:guide13-exec:report-separation`

The following do not substitute for one another:

```text
safety report
minimality report
CT conservation report
owner-signature report
resource report
lifecycle status
```

No report digest is introduced.

---

# 14. Semantic fixtures and projections · `sec:guide13-exec:fixtures`

## 14.1 Model-valid origins · `rule:guide13-exec:model-fixtures`

Positive semantic fixtures begin from model-valid worlds produced through the declared transition path or an explicitly approved synthetic setup preserving the invariant.

Each fixture binds:

- exact predecessor world;
- exact transfer request;
- exact canonical order;
- exact successor world;
- exact appended certificate.

Model execution runs through the invariant wrapper.

## 14.2 Expected semantics · `rule:guide13-exec:expected-semantics`

Expected owners, classes, values, authorization, and certificate facts derive from:

- realization relations;
- executable model;
- typed projection adapters.

The candidate backend does not generate its own expected semantic result.

## 14.3 Target materialization · `rule:guide13-exec:target-materialization`

Target materialization derives from:

```text
candidate linked bundle
+
candidate ABI
+
typed live-transfer request
+
public target input view
+
test-only owner-signing capability
+
test-only confidential construction capability
```

Semantic fixtures contain no:

- target position;
- script;
- tapleaf;
- control block;
- signature bytes;
- private key;
- blinding factor;
- target transaction bytes.

## 14.4 Accepted projection · `rule:guide13-exec:accepted-projection`

For each accepted transaction compare:

- exact consumed live receipt family;
- exact created live receipt multiset;
- exact distinct input-owner set;
- exact destination-owner multiset;
- exact semantic values;
- exact aggregate conservation;
- live class;
- explicit `U`;
- every-owner authorization;
- no roots;
- no issuance;
- no destruction;
- lateral `U` flow;
- sponsor-region membership;
- sponsor-change presence where the class claims it;
- no specialized event;
- transition certificate.

Private exact amounts may be known to the public fixture evaluator without appearing in the canonical target report.

## 14.5 Positive class witnesses · `rule:guide13-exec:class-witnesses`

A positive vector class carries an executable typed predicate proving that the fixture exhibits the property named.

Examples:

```text
split:
    outputs ≥ 2

merge:
    inputs ≥ 2 and outputs = 1

multiple owners:
    distinct owner count ≥ 2

sponsor-change present:
    sponsor region present and exact change role present

canonical input normalization:
    fixture order differs from ABI order and projection still matches

repeatable bytes:
    repeated equal explicit inputs produce equal canonical bytes
```

A class name alone cannot make a fixture canonical.

---

# 15. Required safety matrix · `sec:guide13-exec:safety-matrix`

The execution implementation transcribes the complete matrix into typed classes with polarity, mutation layer, expected boundary, intended relation, and collateral.

## 15.1 Positive explicit classes · `tab:guide13-exec:positive-explicit`

At minimum:

- one input to one output;
- one input split into two;
- several inputs merged into one;
- several inputs to several outputs;
- repeated owner;
- several distinct owners;
- one destination owner;
- several destination owners;
- semantic boundary values;
- canonical input normalization;
- sponsorless;
- sponsored;
- sponsor change present;
- sponsor change absent;
- candidate maximum inputs;
- candidate maximum outputs.

## 15.2 Positive private classes · `tab:guide13-exec:positive-private`

Where private support is claimed:

- one-to-one;
- split;
- merge;
- many-to-many representative;
- several distinct owners;
- private sponsor values where claimed;
- both commitment parity forms;
- deterministic public fixture openings;
- target CT conservation;
- projection equality with paired explicit cases.

## 15.3 Owner and signature faults · `tab:guide13-exec:owner-faults`

At minimum:

- missing owner signature;
- wrong owner;
- one omitted owner;
- unrelated signer;
- duplicated signature substituted for another owner;
- wrong sighash profile;
- signature over another transaction;
- output changed after signing;
- input added after signing;
- output removed after signing;
- unknown key type;
- empty signature;
- malformed signature;
- repeated owner with one concrete input signature omitted.

## 15.4 Class, asset, and constructor faults · `tab:guide13-exec:object-faults`

At minimum:

- time-locked input;
- time-locked output;
- ASH input or output;
- vault, control, entitlement, or bare-`U` output;
- wrong owner metadata;
- malformed live metadata;
- wrong constructor schema;
- mixed operation program;
- stale constructor;
- wrong explicit asset;
- confidential asset commitment;
- unclassified `U`;
- sponsor or fee role carrying `U`;
- foreign asset under receipt-shaped program;
- key-path escape;
- malformed control path.

## 15.5 Value and partition faults · `tab:guide13-exec:value-faults`

At minimum:

- output total one below input;
- output total one above input;
- zero receipt output;
- amount outside semantic domain;
- explicit arithmetic overflow;
- private CT imbalance;
- wrong private blinding balance;
- malformed rangeproof;
- malformed surjection proof;
- copied commitment;
- private output omitted;
- hidden private `U` output;
- representation mismatch;
- mixed representation under homogeneous-only ABI;
- omitted source;
- duplicated source;
- duplicated destination;
- output claimed through two flows;
- issuance;
- destruction;
- value routed into ASH or time-locked receipt;
- second offsetting `U` flow.

## 15.6 Sponsor faults · `tab:guide13-exec:sponsor-faults`

At minimum:

- sponsor/protocol overlap;
- two sponsor envelopes;
- foreign sponsor asset;
- missing sponsor authorization;
- empty sponsor offer for a sponsored request;
- sponsor change in protocol range;
- fee/change substitution;
- sponsor member unclassified;
- report publishes sponsor amount;
- report publishes sponsor opening;
- balanced theft;
- zero-valued ordinary sponsor member under exact roles;
- confidential sponsor values where claimed.

## 15.7 Root, event, ABI, and linker faults · `tab:guide13-exec:structural-faults`

At minimum:

- any root input or output;
- burn record or specialized event;
- omitted transition certificate;
- wrong coordinator;
- two coordinators;
- no coordinator;
- member/coordinator leaf exchange;
- receipt/sponsor range exchange;
- destination order changed after signing;
- witness reorder;
- control block from another program;
- unresolved or duplicate relocation;
- wrong `U` or owner-schema relocation;
- explicit plan paired with private ABI;
- private plan paired with explicit program;
- target bytes changed after ABI validation;
- raw transaction bypassing safe construction.

---

# 16. Disclosure-minimality matrix · `sec:guide13-exec:minimality-matrix`

## 16.1 Paired cases · `rule:guide13-exec:paired-minimality`

Compare semantically equivalent:

- explicit and private one-to-one;
- explicit and private split;
- explicit and private merge;
- explicit and private many-to-many where claimed;
- explicit sponsor and private sponsor where claimed.

Each pair begins from one semantic fixture.

## 16.2 Pair acceptance · `rule:guide13-exec:minimality-acceptance`

A pair supports minimality only when:

- both materializations are constructible;
- both target transactions accept;
- both semantic projections equal the expected transfer;
- owner and live class agree;
- explicit `U` agrees;
- family closure agrees;
- private exact receipt amounts are absent from public protocol output and canonical reports;
- no private opening enters the report;
- no secret dependency exists outside the declared test construction model;
- lifecycle status is equal.

## 16.3 Disclosure classes · `rule:guide13-exec:disclosure-classes`

Record separately:

```text
semantic disclosure
target-safety disclosure
deployment-policy disclosure
transaction-shape leakage
```

For the private plan:

```text
exact input receipt amounts:
    private

exact output receipt amounts:
    private

aggregate semantic transfer amount:
    not published by protocol

owners:
    public constructor metadata

asset U:
    explicit

input and output counts:
    public transaction shape
```

Every additional exact amount disclosure requires a typed reason.

## 16.4 Minimality failure · `rule:guide13-exec:minimality-failure`

Private minimality fails when:

- private materialization rejects;
- exact receipt values enter protocol predicates;
- a public subtotal is required;
- fixture openings appear in the canonical report;
- undeclared owner-private data is required;
- semantic output changes;
- a required exit is lost;
- a hard target limit blocks the private plan while explicit fits;
- confidential asset identity appears.

A failed minimality result says nothing negative about explicit safety.

## 16.5 Privacy non-claims · `rem:guide13-exec:privacy-nonclaims`

A successful private transfer does not establish:

- owner anonymity;
- transaction-graph privacy;
- count privacy;
- timing privacy;
- production wallet privacy;
- production side-channel resistance;
- production multi-party blinding;
- universal transaction confidentiality.

---

# 17. Lifecycle · `sec:guide13-exec:lifecycle`

## 17.1 Required exits · `rule:guide13-exec:required-exits`

A live receipt requires:

```text
transfer
burn
redeem
```

Phase 5 implements transfer only.

Both explicit and private candidates remain lifecycle-incomplete.

## 17.2 Private lifecycle · `rule:guide13-exec:private-lifecycle`

Guide 11 selected an explicit public boundary for amount-dependent public operations.

Therefore:

```text
transfer:
    candidate supported

direct private burn:
    deferred

direct private redemption:
    deferred

normalization:
    accepted target research path
    not an architecture operation

release-complete:
    false
```

Guide 13 does not silently add normalization to architecture or realization.

## 17.3 Constructor status · `rule:guide13-exec:lifecycle-constructor`

No pretend burn, redemption, or normalization leaf is emitted.

Missing exits remain typed obligations.

## 17.4 Minimality and lifecycle · `rule:guide13-exec:lifecycle-minimality`

The minimality report may conclude:

```text
equal for transfer semantics
not equal for complete live-receipt lifecycle
```

until the required exits exist.

A green private transfer is not a general claim that private live receipts are production-supported.

---

# 18. Resource study · `sec:guide13-exec:resources`

## 18.1 Candidate enumeration · `rule:guide13-exec:candidate-bounds`

Evaluate candidate values for:

```text
TRANSFER_INPUT_MAX
TRANSFER_OUTPUT_MAX
FEE_SPONSOR_INPUT_MAX
```

Initial research candidates may include:

```text
receipt inputs:
    1, 2, 4, 8, 16, 32, 64

receipt outputs:
    1, 2, 4, 8, 16, 32, 64

sponsor inputs:
    0, 1, 2, 4, 8, 16
```

Candidate shape sets validate that every member lies within their declared bounds.

Total transaction positions use a numeric domain that cannot overflow accepted shapes.

## 18.2 Complete measurements · `rule:guide13-exec:measurements`

Measure complete transactions for:

- explicit one-to-one;
- explicit split;
- explicit merge;
- explicit many-to-many;
- private one-to-one;
- private split and merge where claimed;
- maximum input family;
- maximum output family;
- maximum distinct owners;
- repeated owner;
- sponsorless;
- sponsored;
- sponsor change present and absent;
- largest proof forms;
- deepest control path.

## 18.3 Resource dimensions · `tab:guide13-exec:resource-dimensions`

Record separately:

- coordinator bytes;
- member bytes;
- constructor bytes;
- taptree depth;
- control bytes;
- owner-signature witness bytes;
- confidential proof bytes;
- initial witness items;
- peak main stack;
- peak alternate stack;
- largest element;
- validation budget;
- complete transaction weight;
- virtual size;
- consensus verdict;
- relay-policy verdict;
- construction and execution time as noncanonical diagnostics.

## 18.4 Prediction and observation · `rule:guide13-exec:resource-comparison`

Backend, linker, and ABI predictions are compared with target observations over the same exact bytes.

A mismatch:

- records both figures;
- returns a typed planner failure;
- stops further operation steps;
- fails the resource report;
- cannot coexist with an overall successful execution result.

No absent observation is read as zero or agreement.

## 18.5 Candidate-only result · `rule:guide13-exec:resource-result`

Phase 5 may record:

```text
candidate bounds fit this tested candidate bundle and ABI
```

It must not record final production bounds.

---

# 19. Relation-indexed coverage · `sec:guide13-exec:coverage`

For every relation-case and representation plan, retain:

```text
relation identity
execution case
representation
activation
selected proof
carrier
positive requirement
negative requirement
evidence boundary
target verdict where applicable
semantic projection where applicable
mutation declaration
dependency collateral
validated observation provenance
```

## 19.1 Explicit coverage · `rule:guide13-exec:explicit-coverage`

Every active explicit relation-case receives:

- valid explicit transaction;
- focused invalid case;
- reachable carrier;
- target or owning first-party verdict;
- accepted semantic projection where applicable.

## 19.2 Private coverage · `rule:guide13-exec:private-coverage`

Private relations use the evidence class that owns them:

- object, class, owner, and asset relations: target programs;
- value conservation: target CT evidence;
- proof construction: transaction evidence;
- disclosure: minimality evidence;
- lifecycle: explicit incomplete status.

No one report row stands for all classes.

## 19.3 Authorization coverage · `rule:guide13-exec:authorization-coverage`

Required cases include:

- approved key and valid signature;
- missing signature;
- wrong owner;
- invalid signature;
- empty signature;
- unknown key form;
- output mutation;
- input mutation;
- incomplete owner set;
- repeated owner with one concrete input witness omitted;
- sponsor-owner omission.

## 19.4 Conditional coverage · `rule:guide13-exec:conditional-coverage`

For sponsor and representation conditions, require:

```text
inactive valid
active valid
active invalid
```

For representation:

```text
explicit plan:
    explicit arithmetic active
    CT conservation inactive

private plan:
    explicit arithmetic inactive
    CT conservation active
```

## 19.5 External evidence · `rule:guide13-exec:external-evidence`

CT conservation and selected sighash semantics remain explicit external target evidence.

Each report binds:

- exact target;
- exact development deployment;
- exact candidate bundle;
- exact candidate ABI;
- exact transaction bytes;
- exact representation and sponsor case;
- exact executor provenance.

Mocks and abstract validators do not satisfy these roles.

---

# 20. Implementation waves · `sec:guide13-exec:waves`

Each wave ends formatted, focused-test green, documented, and committed before the next begins.

## Wave 0 — Reproduce and close the preflight · `task:guide13-exec:wave0`

**Deliverables**

- reproduce `G13-R01` through `G13-R18`;
- assign CONFIRMED, REFUTED, or RECLASSIFIED;
- close every confirmed P0/P1;
- schedule and close P2 rows before their consuming boundary;
- reconcile active Phase-4/Phase-5 planning state;
- record current identities and clean-tree state.

**Suggested commits**

```text
vectors: bind canonical coverage to validated operation reports
transaction: reject noncanonical and contradictory construction inputs
target-conformance: enforce strict operation response and framing contracts
linker: close exact taptree input and arithmetic invariants
plans: reconcile the Phase-4 handoff
```

## Wave 1 — Complete the evidence handoff · `task:guide13-exec:wave1`

**Deliverables**

- validated operation-report type;
- exact request/response binding;
- expected executor provenance at operation gate;
- canonical timing-free bytes;
- typed mutation-to-requirement links;
- first-party negative evidence policy;
- ABI-validation report role;
- sponsor-change class witness.

**Suggested commit**

```text
vectors: complete the validated operation evidence boundary
```

## Wave 2 — Live-transfer compiler plan · `task:guide13-exec:wave2`

**Deliverables**

- exact `TransferLive` target-operation plan;
- explicit/private alternatives;
- owner, class, value, sponsor, lifecycle, layout, and coverage projections;
- enum/census generation from one source;
- corruption-resistant validator;
- public API tests.

**Suggested commit**

```text
compiler: expose the validated live-transfer target plan
```

## Wave 3 — Signature and CT target closure · `task:guide13-exec:wave3`

**Deliverables**

- selected owner sighash profile;
- target-native valid signature case;
- input and output mutation cases;
- owner-key encoding closure;
- CT conservation evidence role;
- mixed-representation disposition;
- reviewed public test materializer.

**Suggested commit**

```text
target-elements: close live-transfer signature and CT requirements
```

## Wave 4 — Live-receipt constructor · `task:guide13-exec:wave4`

**Deliverables**

- canonical owner metadata;
- structural live class;
- explicit/private representation role;
- static transfer leaf set;
- inherited internal-key policy;
- no key-path escape;
- candidate lifecycle;
- constructor mutation cases.

**Suggested commit**

```text
tapscript: implement the candidate live-receipt constructor
```

## Wave 5 — Recognition and owner authorization · `task:guide13-exec:wave5`

**Deliverables**

- exact predecessor recognition;
- owner-key encoding enforcement;
- per-input owner authorization;
- coordinator/member placement;
- composed literal/result soundness;
- unknown-key negative cases;
- complete stack schedules.

**Suggested commit**

```text
tapscript: implement live-transfer input authorization
```

## Wave 6 — Explicit transfer plan · `task:guide13-exec:wave6`

**Deliverables**

- complete family ranges;
- destination constructor closure;
- exact explicit aggregate conservation;
- split and merge;
- sponsor isolation;
- root and event absence;
- candidate relocatable explicit bundle.

**Suggested commit**

```text
tapscript: emit the explicit live-transfer plan
```

## Wave 7 — Private-committed plan · `task:guide13-exec:wave7`

**Deliverables**

- complete explicit-`U` closure;
- private value preservation through external CT evidence;
- no amount reads;
- deterministic public test material;
- valid private one-to-one, split, and merge where claimed;
- private relocatable bundle;
- explicit production non-claims.

**Suggested commit**

```text
tapscript: emit the private live-transfer plan
```

If private construction is not feasible, stop with a typed private-plan deferral. Do not weaken the private claim or call an explicit-only result the full Guide-13 exit.

## Wave 8 — Linker extension · `task:guide13-exec:wave8`

**Deliverables**

- owner-parameterized constructors;
- representation-specific symbols;
- exact relocations;
- duplicate-free taptree input;
- exact tree-cost arithmetic;
- deterministic tree;
- carrier closure per plan;
- candidate linked live-transfer bundle.

**Suggested commit**

```text
linker: link the live-transfer candidate bundle
```

## Wave 9 — Candidate ABI and signing flow · `task:guide13-exec:wave9`

**Deliverables**

- canonical input order;
- request-order destinations;
- exact sponsor/request/form consistency;
- output finalization;
- owner signing requests;
- multi-owner response validation;
- explicit transaction construction;
- private test construction;
- candidate-only ABI.

**Suggested commit**

```text
transaction: derive the live-transfer candidate ABI
```

## Wave 10 — Safety evidence · `task:guide13-exec:wave10`

**Deliverables**

- canonical safety plan;
- complete explicit matrix;
- private matrix for claimed scope;
- owner/class/asset/value/sponsor coverage;
- accepted semantic projections;
- first-party negative reports;
- validated safety report;
- zero required infrastructure errors.

**Suggested commit**

```text
vectors: complete live-transfer safety evidence
```

## Wave 11 — Disclosure-minimality evidence · `task:guide13-exec:wave11`

**Deliverables**

- paired explicit/private fixtures;
- equal expected semantics;
- both target verdicts;
- projection comparison;
- disclosure comparison;
- shape leakage;
- construction assumptions;
- lifecycle limitation;
- validated minimality report.

**Suggested commit**

```text
vectors: compare live-transfer representations
```

## Wave 12 — Resource study · `task:guide13-exec:wave12`

**Deliverables**

- finite input/output/sponsor enumeration;
- explicit/private measurements;
- multi-owner signature measurements;
- exact tree and transaction resource arithmetic;
- prediction/observation equality;
- useful candidate;
- explicit non-calibration result.

**Suggested commit**

```text
vectors: measure live-transfer candidate bounds
```

## Wave 13 — Phase-5 gate and handoff · `task:guide13-exec:wave13`

**Deliverables**

- package READMEs and contracts;
- Phase-5 card;
- backlog gate record;
- security and production non-claims;
- identity, schema, interchange, and dependency impact;
- complete repository gate;
- clean tree.

**Suggested commit**

```text
plans: record the live-transfer candidate pipeline
```

---

# 21. Verification · `sec:guide13-exec:verification`

## 21.1 Working cadence · `rule:guide13-exec:cadence`

For Rust tranches:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Run focused package tests during development. Run the full workspace and Meson gate once per coherent batch, consistent with the content-scoped cadence recorded in the backlog.

Documentation-only tranches run the documentation, label, and census checks rather than the full Rust suite after every edit.

## 21.2 Focused package commands · `tab:guide13-exec:focused-tests`

```sh
cargo test --locked -p tripod-model transfer
cargo test --locked -p tripod-model realization_conformance

cargo test --locked -p tripod-realization
cargo test --locked -p tripod-compiler
cargo test --locked -p tripod-target-elements
cargo test --locked -p tripod-tapscript
cargo test --locked -p tripod-linker
cargo test --locked -p tripod-transaction
cargo test --locked -p tripod-vectors
cargo test --locked -p tripod-target-elements-conformance
```

For changed public package documentation:

```sh
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p <package> --no-deps
```

## 21.3 Required focused regressions · `tab:guide13-exec:required-regressions`

The preflight series includes direct regressions for:

```text
caller-authored Accepted + Matched cannot discharge canonical coverage
caller-authored target refusal cannot discharge canonical negative coverage
all-empty witness section rejects
decoded canonical transaction re-encodes byte-identically
weight mismatch returns PlanRefused and emits no later step
shape outside CandidateShapeSet bounds rejects
total input range cannot overflow
raw child stderr marker appears in no first-party output
zero taproot tweak is admitted
tree cost overflow returns a typed refusal
sponsor-change-present contains an actual change role
known unequal EQUAL followed by VERIFY has no success
operation report retains exact generic execution transcript
empty sponsor offer rejects a sponsored request
lifecycle response cannot carry the opposite role’s fields
rejected operation response cannot carry success artifacts
duplicate tree leaf rejects
duplicate public view rejects
duplicate sponsor input rejects
enum and ALL census cannot drift
Python and Rust reject the same blank, oversized, unknown-field,
and unterminated records
```

## 21.4 Real target matrices · `tab:guide13-exec:target-matrices`

Run separately:

```text
explicit live-transfer safety matrix
private live-transfer safety matrix
owner-authorization matrix
constructor and class-closure matrix
sponsor matrix
explicit/private minimality matrix
candidate resource matrix
```

Each run records:

```text
target contract revision
deployment binding
network and genesis
executor declaration
adapter and framework versions
binary-reported revision
intended tip
upstream base
local-topic census
candidate bundle and ABI typed subjects
representation plan
candidate bounds
case, relation, and claim censuses
failures and infrastructure errors
canonical report-byte reproducibility
```

## 21.5 Full batch gate · `gate:guide13-exec:batch`

After a coherent implementation batch:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

When dependencies changed:

```sh
cargo tree --locked -e features
cargo metadata --locked
cargo audit
```

A missing advisory tool is recorded as skipped, never passed.

When document policy requires it:

```sh
scripts/check-document-reproducibility.sh
```

Documentation and census:

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Final check:

```sh
git status --porcelain=v1 --untracked-files=all
```

It must be empty.

---

# 22. Acceptance criteria · `sec:guide13-exec:acceptance`

The full Phase-5 candidate is accepted only when:

- Phase 4 remains green on the current tree;
- all confirmed preflight findings are closed;
- live-transfer semantic scope is exact;
- every relation survives every package boundary;
- every target requirement is assessed;
- every selected proof is realization-approved;
- every active relation-case has a reachable carrier;
- every receipt input authenticates live class, exact `U`, owner, constructor, and representation;
- every receipt output authenticates live class, exact `U`, destination owner, constructor, and representation;
- every consumed owner authorizes the same finalized transaction;
- unknown key types cannot satisfy authorization;
- explicit one-to-one, split, merge, and many-to-many cases accept;
- explicit invalid cases fail at their owning boundaries;
- private one-to-one accepts;
- private split and merge accept before those scopes are claimed;
- private protocol paths inspect no exact receipt amount;
- every closed-asset-capable output is classified;
- no confidential asset carries `U`;
- sponsor membership and sponsor-change presence are exact;
- sponsor value remains opaque;
- balanced theft rejects for the protocol relation;
- no root, issuance, destruction, ASH, or specialized event appears;
- target acceptance and semantic projection both pass;
- safety and minimality reports remain separate;
- canonical coverage is transcript-derived and complete for the claimed scope;
- lifecycle incompleteness is explicit;
- linked bundle and ABI are deterministic;
- resource prediction equals observation;
- candidate bounds remain non-final;
- public test material is not described as a production secret interface;
- no speculative digest is minted;
- canonical report bytes reproduce;
- all repository gates pass;
- the final tree is clean.

An explicit-only candidate with a typed private-plan deferral is a valid stopped result. It is not the full Guide-13 exit and cannot satisfy the private minimality gate.

---

# 23. Rejection criteria · `sec:guide13-exec:rejection`

Reject the candidate if:

- one semantic relation disappears;
- target details enter compiler core;
- caller-authored statuses can discharge canonical evidence;
- a target capability is treated as a completed proof;
- one owner is omitted;
- different owners sign different protected transactions;
- output or input mutation after signing passes;
- unknown key encoding satisfies owner authorization;
- a time-locked receipt enters or leaves;
- ASH or another undeclared `U` object enters or leaves;
- aggregate balance hides a wrong owner, class, recipient, or object family;
- confidential asset identity is accepted;
- hidden `U` output escapes closure;
- private transfer requires amount inspection;
- rejection-only evidence is presented as private support;
- one-to-one evidence is generalized to split or merge;
- explicit safety is presented as minimality;
- CT conservation is presented as owner authorization;
- sponsor amount becomes protocol data;
- sponsor positivity substitutes for transfer correctness;
- a sponsored request silently becomes sponsorless;
- `sponsor-change-present` contains no change;
- a role-contradictory protocol response is accepted;
- a noncanonical transaction encoding acquires a semantic projection;
- construction failure counts as target rejection;
- ad hoc vectors become gate-eligible;
- accepted target projection differs from expected semantics;
- lifecycle incompleteness is hidden;
- central public-fixture construction is described as production multi-party blinding;
- a mock satisfies the gate;
- resource mismatch coexists with successful execution;
- candidate and final states are conflated;
- a digest is added without an admitted consumer;
- any full gate fails or leaves the tree dirty.

---

# 24. Identity, schema, security, and dependency impact · `sec:guide13-exec:impact`

Expected identity impact:

```text
Attestation:
    unchanged

realization:
    unchanged unless semantic correction independently requires movement

architecture schema and semantic identity:
    unchanged unless semantic correction independently requires movement

compiler identity:
    none minted

constructor identity:
    none minted

bundle identity:
    none minted for in-process candidate use

ABI identity:
    none minted for in-process candidate use

safety/minimality report identities:
    none minted

deployment profile:
    dormant
```

Expected typed schema work:

```text
validated live-transfer compiler projection
live-transfer representation plan
owner metadata
candidate live-receipt constructor
candidate linked live-transfer bundle
candidate live-transfer ABI
validated operation report
live-transfer safety plan and report
live-transfer minimality plan and report
first-party negative evidence report
ABI-validation report role
```

Expected security impact:

```text
public disposable test signing material:
    admitted

public disposable test blinders and openings:
    admitted

production private keys:
    not admitted

production blinders and openings:
    not admitted

production multi-party signing and blinding:
    not established
```

Expected dependency impact:

```text
reuse Phase-4 first-party packages

no new dependency without a concrete consumer and ADR-011 review

if an Elements transaction or cryptographic dependency is proposed:
    exact version, features, licence, MSRV, transitive graph,
    build scripts, unsafe/FFI, determinism, advisories,
    and lockfile impact are reviewed before admission
```

Any externally consumed report document additionally activates ADR-022.

---

# 25. Exit checklist · `gate:guide13-exec:exit`

## Entry and preflight

- [ ] Phase-4 result revalidated on the current tree;
- [ ] active planning documents agree;
- [ ] all `G13-R01`–`G13-R18` rows dispositioned;
- [ ] all confirmed blockers closed before their consuming wave;
- [ ] no unexplained identity drift;
- [ ] tree clean.

## Compiler

- [ ] validated live-transfer plan has no unchecked constructor;
- [ ] scope exact;
- [ ] relation census exact;
- [ ] explicit/private plan census exact;
- [ ] owner-source requirements complete;
- [ ] constructibility complete;
- [ ] lifecycle explicit;
- [ ] carrier, layout, and coverage censuses exact;
- [ ] enum and `ALL` censuses cannot drift;
- [ ] no graph handles or target positions leak;
- [ ] no digest minted.

## Constructor and programs

- [ ] owner metadata canonical;
- [ ] live class structural;
- [ ] explicit `U` mandatory;
- [ ] confidential `U` rejected;
- [ ] predecessor and destination constructors exact;
- [ ] every input owner predicate executes;
- [ ] unknown-key success closed;
- [ ] explicit conservation exact;
- [ ] private plan reads no exact amount;
- [ ] sponsor relation and change presence exact;
- [ ] roots, issuance, destruction, and events absent;
- [ ] composed abstract execution sound;
- [ ] final stack valid;
- [ ] no key-path escape.

## Linking and ABI

- [ ] every symbol typed;
- [ ] every reference resolves;
- [ ] duplicate leaves reject;
- [ ] tree costs exact;
- [ ] representation-specific carriers reachable;
- [ ] shape members lie within declared bounds;
- [ ] index arithmetic cannot overflow;
- [ ] input order canonical;
- [ ] destination order explicit;
- [ ] private order independent of private values;
- [ ] coordinator unique;
- [ ] sponsor form cannot silently downgrade;
- [ ] output set finalized before signing;
- [ ] candidate bundle and ABI non-final.

## Authorization and confidential construction

- [ ] all owners sign the same finalized transaction;
- [ ] every input satisfies its own predicate;
- [ ] selected sighash observed;
- [ ] missing owner rejects;
- [ ] output/input mutation rejects;
- [ ] test signing material public and disposable;
- [ ] test blinding material public and disposable;
- [ ] no production secret interface introduced;
- [ ] centralized fixture construction not described as production protocol.

## Safety evidence

- [ ] validated operation report binds exact generic transcript;
- [ ] canonical safety plan cannot be caller-forged;
- [ ] positive explicit cases accept;
- [ ] required explicit negatives reject;
- [ ] positive private cases accept where claimed;
- [ ] required private negatives reject;
- [ ] split and merge each have positive evidence;
- [ ] owner/class/asset/value/sponsor coverage complete;
- [ ] accepted projections match;
- [ ] pre-target and target failures separate;
- [ ] zero required infrastructure failures;
- [ ] no mock satisfies the gate.

## Minimality evidence

- [ ] safety and minimality report types distinct;
- [ ] paired cases share one semantic fixture;
- [ ] both explicit and private transactions accept;
- [ ] both projections equal expected semantics;
- [ ] private exact amounts unpublished;
- [ ] no confidential asset carries `U`;
- [ ] disclosure reasons typed;
- [ ] shape leakage recorded;
- [ ] lifecycle limitation explicit;
- [ ] no universal privacy claim.

## Resources

- [ ] explicit one-to-one measured;
- [ ] explicit split and merge measured;
- [ ] private one-to-one measured;
- [ ] private split and merge measured where claimed;
- [ ] multi-owner signatures measured;
- [ ] sponsorless and sponsored measured;
- [ ] sponsor change present and absent measured;
- [ ] candidate maxima measured;
- [ ] prediction and observation agree;
- [ ] mismatch causes actual planner failure;
- [ ] consensus and policy remain separate;
- [ ] candidate bounds remain non-final.

## Repository

- [ ] READMEs and package contracts current;
- [ ] Phase-5 card current;
- [ ] backlog current;
- [ ] every new file in Meson census;
- [ ] dependency review recorded;
- [ ] security impact recorded;
- [ ] identity/schema/interchange impact recorded;
- [ ] formatting passes;
- [ ] Clippy passes with warnings denied;
- [ ] workspace tests pass;
- [ ] `scripts/ci.sh` passes;
- [ ] Meson compile and tests pass;
- [ ] advisory status passed or explicitly skipped;
- [ ] document reproducibility passed or deferred under explicit policy;
- [ ] `git diff --check` passes;
- [ ] final repository status empty.

---

# 26. Completion report template · `sec:guide13-exec:completion-report`

```text
Guide 13 result
===============

Starting state:
    source revision:
    working tree:
    Phase-4 result:
    architecture schema and semantic identity:
    realization version:
    target contract revision:
    executor protocol revision:
    live-transfer compiler projection:

Preflight:
    G13-R01:
    G13-R02:
    G13-R03:
    G13-R04:
    G13-R05:
    G13-R06:
    G13-R07:
    G13-R08:
    G13-R09:
    G13-R10:
    G13-R11:
    G13-R12:
    G13-R13:
    G13-R14:
    G13-R15:
    G13-R16:
    G13-R17:
    G13-R18:
    clean tree:

Compiler:
    target-operation type:
    scope:
    relation census:
    explicit plan:
    private plan:
    mixed plan:
    owner requirements:
    constructibility:
    lifecycle:
    carriers:
    layout:
    coverage:
    external evidence:
    graph handles:
    identity minted:

Target assessment:
    owner-key encoding:
    signature semantics:
    sighash profile:
    CT conservation:
    value and asset introspection:
    missing primitives:
    unsupported capabilities:
    structural obligations:
    external evidence:

Live constructor:
    owner metadata:
    class encoding:
    representation encoding:
    leaf set:
    internal key:
    key-path result:
    outstanding lifecycle:

Explicit tapscript:
    input recognition:
    output closure:
    owner authorization:
    coordinator/member:
    conservation:
    sponsor isolation:
    absence relations:
    stack result:
    resources:

Private tapscript:
    input recognition:
    output closure:
    owner authorization:
    value inspection:
    CT evidence handoff:
    sponsor isolation:
    stack result:
    supported shapes:
    resources:

Linker:
    symbols:
    relocations:
    duplicate-leaf result:
    exact cost domain:
    explicit/private leaves:
    taptree:
    carrier closure:
    deterministic rebuild:
    candidate bundle:

Transaction ABI:
    input layout:
    output layout:
    output-order policy:
    shape bounds:
    coordinator:
    sponsor form:
    sponsor change:
    fee role:
    request fields:
    finalization:
    signing requests:
    witness order:
    candidate status:

Signing:
    distinct owners:
    receipt inputs:
    signatures:
    repeated owners:
    selected sighash:
    missing-owner result:
    post-signing mutation result:
    input-extension result:
    production-key claim:
        none

Confidential construction:
    materializer:
    construction model:
    public fixture keys:
    public fixture blinders:
    one-to-one:
    split:
    merge:
    many-to-many:
    sponsor:
    CT proof construction:
    production blinding claim:
        none

Safety evidence:
    validated operation report:
    explicit positive cases:
    explicit failures:
    private positive cases:
    private failures:
    owner authorization:
    class closure:
    asset closure:
    value conservation:
    sponsor isolation:
    accepted projections:
    relation coverage:
    first-party negative coverage:
    infrastructure errors:
    canonical report bytes:

Minimality evidence:
    paired cases:
    explicit accepted:
    private accepted:
    projection equality:
    amount disclosure:
    shape leakage:
    constructibility difference:
    lifecycle difference:
    canonical report bytes:
    universal privacy claim:
        none

Resources:
    explicit one-to-one:
    explicit split:
    explicit merge:
    private one-to-one:
    private split:
    private merge:
    maximum owners and signatures:
    sponsorless:
    sponsored:
    sponsor change present:
    sponsor change absent:
    coordinator/member bytes:
    constructor and control bytes:
    proof and witness bytes:
    peak stack:
    validation budget:
    transaction weight:
    candidate input/output/sponsor bounds:
    final calibration claim:
        none

Lifecycle:
    transfer:
        candidate implemented
    burn:
        outstanding
    redeem:
        outstanding
    normalization:
        research path only / architecture status
    release-complete:
        false

Security:
    production signing keys accepted:
        no
    production blinders accepted:
        no
    public disposable test material:
    future secret-bearing design required:

Identity and interchange:
    Attestation:
    realization:
    architecture:
    compiler identity:
        none
    constructor identity:
        none
    bundle identity:
        none
    ABI identity:
        none
    report identities:
        none
    ADR-022 external document:
        none / explicitly implemented
    deployment profile:
        dormant

Dependency impact:
    first-party changes:
    third-party additions:
    Cargo.lock:
    licences:
    MSRV:
    unsafe/FFI:
    advisories:

Verification:
    formatting:
    Clippy:
    workspace tests:
    model:
    realization:
    compiler:
    target-elements:
    tapscript:
    linker:
    transaction:
    vectors:
    target executor:
    Rustdoc:
    plan and label checks:
    Meson lint:
    scripts/ci.sh:
    Meson compile:
    Meson tests:
    explicit target matrix:
    private target matrix:
    owner matrix:
    minimality matrix:
    resource matrix:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Phase-5 verdict:
    accepted full candidate
    / explicit-only candidate
    / private plan deferred
    / rejected target path
    / blocked

Residuals:

Next phase:
```

---

# 27. Handoff · `sec:guide13-exec:handoff`

If the full Guide-13 gate passes, Phase 5 has one complete owner-authorized transfer candidate.

The next guide should implement STATE and maturity:

```text
Guide 14 — STATE and Maturity
```

Guide 14 consumes rather than reopens:

- validated target-operation plans;
- representation-specific backend planning;
- owner-bound constructor metadata;
- per-input owner authorization;
- finalized-output signing;
- explicit/private proof separation;
- deterministic linking;
- candidate ABI construction;
- validated operation reports;
- safety/minimality report separation;
- exact executor provenance and environment binding;
- candidate resource measurement.

It adds:

- STATE constructor continuity;
- root succession;
- operator authorization;
- maturity announcement;
- state-field commitments;
- the first mutable protocol constructor.

Guide 13 makes none of those STATE or root claims.

---

## Closing statement · `rem:guide13-exec:closing`

> Live receipt transfer is the first phase where authorization, representation, disclosure, and collaboration become simultaneous implementation dimensions. The phase succeeds only when every consumed owner approves one finalized transaction; every protocol input and output remains an exact live receipt carrying explicit closed `U`; split and merge preserve one semantic value relation; any private plan preserves that relation without exact amount inspection; sponsor value remains erased; target acceptance and semantic agreement are checked separately; safety and minimality are evidenced by distinct validated reports; and every constructor, bundle, ABI, bound, lifecycle claim, and secret-handling claim remains visibly candidate-only and non-production.

