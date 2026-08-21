# Draft: Guide 13 Concept — End-to-End Live Receipt Transfer

> **Status:** Concept draft; not an execution guide
> **Phase:** Phase 5 — Live Receipt Transfer
> **Entry:** (`gate:phase4:exit`)
> **Primary semantic operation:** `transfer-live-receipts`
> **Affected packages:** `realization`, `compiler`, `target-elements`, `tapscript`, `linker`, `transaction`, `vectors`
> **Evidence support:** the target-executor and validated-report boundary established by Phase 4; `target-elements-conformance` only where the inherited executor interface remains target-generic
> **May affect after acceptance:** package contracts, Phase-5 card, backlog, representation research, signer boundary, Meson graph, dependency graph
> **Supersedes as concept direction:** none
> **Does not implement:** burn, clear, redemption, STATE, RESV, cycle, settlement, production wallets, production private-key custody, production blinding-factor custody, final calibration, production deployment, or release
> **Required result:** one complete candidate compiler-to-target implementation of live receipt transfer, with every-owner authorization, live-class closure, explicit closed `U`, exact split/merge conservation, sponsor isolation, an explicit safety result, a separate disclosure-minimality result, real target execution, complete relation-indexed coverage, and an explicit non-production lifecycle status
> **Authority:** Attestation, Realization, typed architecture, implemented ADRs, accepted decisions, the accepted Guide-11 representation policy, and the Phase-4 compact-ASH result take precedence over this concept
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015

---

## Mission · `sec:guide13:mission`

Guide 13 extends the complete compiler-to-target pipeline from one ownerless public operation to the first owner-authorized, representation-parametric protocol operation:

```text
typed architecture
    ↓
target-independent live-transfer realization
    ↓
validated compiler analysis
    ↓
validated live-transfer target-operation plan
    ↓
reviewed Elements target assessment
    ↓
representation-specific tapscript proof plans
    ↓
candidate relocatable bundle
    ↓
deterministic linking
    ↓
candidate transaction and witness ABI
    ↓
owner-authorized complete Elements transactions
    ↓
real target execution
    ↓
accepted semantic projection
    ↓
separate safety and minimality evidence
```

The operation consumes one or more live receipts and creates one or more live receipts:

```text
RECEIPT_L inputs
    ↓
RECEIPT_L outputs
```

It may split, merge, redistribute, and change physical denominations and owners, provided that:

- every consumed receipt owner authorizes the finalized transaction;
- every protocol input and output remains live class;
- every protocol input and output carries explicit closed asset `U`;
- aggregate semantic `U` value is conserved exactly;
- every closed-asset-capable position is classified;
- no ASH, time-locked receipt, vault, control, bare `U`, or other undeclared object appears;
- no root participates;
- no issuance or destruction occurs;
- no specialized event projection occurs;
- sponsor value remains isolated and opaque;
- the selected value representation preserves the same semantic relation.

Phase 5 is the first pipeline phase in which target acceptance is insufficient in two independent ways:

1. an accepted transaction may still violate the owner/class/object relation the protocol intended;
2. a safe explicit implementation may still reveal more than a supported confidential implementation requires.

Therefore Guide 13 produces two distinct evidence results:

```text
LiveTransferSafety

LiveTransferDisclosureMinimality
```

Neither may satisfy the other.

---

## One-line thesis · `rem:guide13:thesis`

> Guide 13 succeeds when every consumed live-receipt owner authorizes one finalized, class-closed, closed-asset-rigid transfer; explicit and supported confidential value plans preserve the same semantic split/merge relation; sponsor-local value remains outside protocol predicates; and safety, constructibility, lifecycle, disclosure, target execution, and resource evidence remain separate and honest.

---

# 1. Governing rulings · `sec:guide13:rulings`

## 1.1 Guide 13 consumes the Phase-4 pipeline · `rule:guide13:consume-phase4`

Guide 13 must consume rather than redesign without cause:

- the validated compiler target-operation boundary;
- abstract target requirement projection;
- typed target capability assessment;
- typed tapscript pattern interface;
- relocatable bundle interface;
- typed linker symbols and relocations;
- deterministic taptree policy;
- candidate linked-bundle state;
- candidate transaction ABI framework;
- canonical operation evidence-plan boundary;
- target-executor supervision and strict protocol;
- exact target/deployment transcript binding;
- validated report trust states;
- exact report-census validation;
- resource prediction and observation comparison;
- candidate-versus-final type separation.

A Phase-4 interface may change only when live transfer presents a concrete requirement it cannot express. The change records:

- the missing fact;
- why the compact-ASH interface cannot carry it;
- the smallest generalization;
- compact-ASH regression impact;
- identity and schema impact.

Guide 13 must not replace a working Phase-4 boundary merely to make the live-transfer implementation aesthetically independent.

## 1.2 Every relation survives the pipeline · `rule:guide13:relation-census`

For the live-transfer scope, require exact equality among:

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
minimality evidence relation census, where applicable
```

Safety and minimality may have different evidence obligations. They may not have different semantic relation ownership.

A package may add target-owned structural obligations. It may not:

- drop a semantic relation;
- merge two relation identities into one report row;
- silently weaken authorization;
- silently widen representation support;
- omit an unresolved lifecycle exit;
- turn an external consensus claim into a backend-local claim.

## 1.3 Value-parametric, asset-rigid · `rule:guide13:value-parametric`

Guide 13 implements D005 directly:

```text
value proof:
    representation-parametric where supported

closed protocol asset identity:
    explicit and rigid everywhere
```

Every live receipt input and output carries explicit `U`.

No confidential asset commitment may carry `U`.

No unclassified output may carry `U`.

No value-conservation proof, including target Confidential-Transaction conservation, substitutes for explicit closed-asset classification.

## 1.4 Publicly equivalent does not mean byte-identical · `rule:guide13:semantic-equivalence`

The explicit and confidential transfer plans may differ in:

- target value encoding;
- value witness material;
- blinding factors;
- rangeproofs and surjection proofs;
- transaction bytes;
- witness sizes;
- disclosure;
- target resource use;
- construction protocol.

They are semantically equivalent only when they agree on the target-independent transfer projection:

```text
consumed live receipt objects
consumed owners
created live receipt objects
created destination owners
semantic values
aggregate U conservation
live class
closed U identity
authorization result
absence of roots
absence of issuance and destruction
absence of specialized events
sponsor-region verdict
```

Semantic equivalence is evaluated through typed values, not through equal target bytes.

## 1.5 Safety and minimality are distinct · `rule:guide13:two-evidence-axes`

The safety question is:

```text
Can an invalid asset, class, owner, authorization, amount,
constructor, family, or output set be accepted?
```

The minimality question is:

```text
Can a lower-disclosure supported representation perform the same
semantic transfer without adding a disclosure the relation does not need?
```

Safety is demonstrated by focused rejecting vectors.

Minimality is demonstrated by accepting a supported lower-disclosure plan and comparing its semantic projection with the explicit plan.

These implications are forbidden:

```text
explicit transfer is safe
    ⇏ explicit transfer is disclosure-minimal

confidential transfer is accepted
    ⇏ confidential closed-asset identity is safe

confidential transfer conserves value
    ⇏ every owner authorized

one-to-one confidential transfer works
    ⇏ confidential split and merge work

all invalid confidential cases reject
    ⇏ a valid confidential case exists

same target verdict
    ⇏ same semantic projection
```

## 1.6 Every owner authorizes the finalized transaction · `rule:guide13:every-owner`

Let \(I\) be the consumed live-receipt inputs and \(\operatorname{owner}(i)\) the owner committed by input \(i\).

Every accepted transfer satisfies:

\[\{\operatorname{owner}(i)\mid i\in I\}\subseteq \operatorname{Authorized}(T)\]

where \(T\) is the complete finalized target transaction.

At the semantic layer, repeated ownership may collapse into a set of required owners.

At the target layer, each input’s own program may require its own signature. Those are different statements:

```text
semantic authorization:
    every distinct consumed owner is represented

target realization:
    every consumed receipt input satisfies its own owner predicate
```

The target realization may be stronger by requiring one valid signature per consumed receipt input, even when several inputs share one owner. That implementation choice is recorded and measured; it is not reinterpreted as a semantic requirement that one owner sign several logically distinct approvals.

## 1.7 Signatures bind the complete protected output set · `rule:guide13:signature-commitment`

All protocol-owner signatures are requested only after:

- every receipt input is fixed;
- every destination owner is fixed;
- every destination amount or commitment is fixed;
- every receipt constructor is fixed;
- sponsor inputs are fixed where the selected sighash profile commits them;
- sponsor change is fixed;
- target fee role is fixed;
- every rangeproof and surjection-proof commitment field covered by the selected digest is fixed;
- transaction version and locktime are fixed;
- the selected tapleaf and control data are fixed where the target digest commits them.

The initial candidate should use the reviewed all-outputs, non-anyone-can-pay signature profile unless an alternative is separately shown to preserve:

- complete owner input commitment;
- complete protected output commitment;
- issuance absence;
- representation commitments;
- sponsor isolation.

A signature profile permitting unreviewed input extension or output omission is rejected.

## 1.8 No hidden owner selection · `rule:guide13:no-hidden-owner-policy`

The transfer request may choose destination owners and semantic amounts because transfer semantics authorize those choices.

The backend may not invent:

- a recipient;
- a denomination;
- an owner aggregation rule;
- a fee recipient;
- a representation transition;
- an output omitted from the owners’ signatures.

Where several owners collaborate, all sign the same finalized protected transaction. The target builder may not sign one owner against one output view and another owner against a different output view.

## 1.9 Sponsor opacity survives owner authorization · `rule:guide13:sponsor-opacity`

Owner authorization over protocol receipts does not make sponsor values protocol data.

A target program, ABI, canonical report, or future identity must not require an individual sponsor amount to be:

- explicit for protocol use;
- decoded;
- opened;
- compared with zero;
- proved positive;
- publicly aggregated;
- emitted in a diagnostic;
- emitted in a canonical report.

The concrete sponsor relation remains:

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

A balanced-theft mutation remains mandatory:

```text
shorten a protocol receipt output
increase sponsor change by the same amount
preserve whole-transaction balance
keep every sponsor amount positive
```

The target must reject because the protocol transfer relation is wrong, not because sponsor positivity failed.

## 1.10 No production secret interface enters by accident · `rule:guide13:secret-boundary`

A confidential transfer requires material that is secret in a production setting:

- input value openings;
- input value blinding factors;
- output value blinding factors;
- rangeproof randomness;
- owner signing keys;
- possibly asset blinding data if an unsupported confidential asset path is attempted.

Current first-party packages are public-data packages under ADR-015.

Therefore Phase 5 distinguishes:

```text
public disposable test material
    admitted for target and semantic evidence

production secret-bearing interface
    not admitted without a separate reviewed security design
```

Guide 13 may prove confidential transfer semantics using deterministic public development fixtures whose keys and blinders authorize nothing of value.

Guide 13 must not claim:

- production wallet support;
- production multi-party blinding;
- production signer support;
- production opening custody;
- production nonce safety.

A package or executable that accepts genuine production secrets requires the ADR-015 future-secret design before the interface lands.

## 1.11 Candidate and final states remain distinct · `rule:guide13:candidate-state`

Guide 13 may produce:

```text
ValidatedLiveTransferOperationPlan

CandidateLiveTransferTapscriptPlan

CandidateRelocatableLiveTransferBundle

CandidateLinkedLiveTransferBundle

CandidateLiveTransferAbi

CandidateLiveTransferSafetyReport

CandidateLiveTransferMinimalityReport
```

It must not produce:

```text
FinalLiveReceiptConstructor

FinalLinkedBundle

FinalTransactionAbi

ProductionConfidentialTransfer

ProductionSignerInterface

ValidatedDeploymentRelease
```

The candidate live-receipt constructor is lifecycle-incomplete because burn and redemption remain absent.

## 1.12 No speculative identity · `rule:guide13:no-speculative-identity`

Typed in-process values use exact typed comparison.

Exact program, transaction, and report bytes use exact byte comparison where those bytes are the subject.

Guide 13 mints no:

```text
LiveTransferPlanHash
LiveReceiptConstructorHash
LiveTransferBundleHash
LiveTransferAbiHash
SafetyReportHash
MinimalityReportHash
```

A digest is admitted only when a real cache, process, publication, distribution, deployment, or signature consumer appears and ADR-016 admission is satisfied.

No candidate type reserves a future digest field.

---

# 2. Entry conditions · `sec:guide13:entry`

Guide 13 begins only when all applicable entry conditions hold.

## 2.1 Phase-4 entry · `gate:guide13:phase4-entry`

Required:

- Phase 4 has passed on the current tree;
- compact ASH has one complete candidate compiler-to-target path;
- the compiler target-operation boundary is public and validated;
- the tapscript pattern interface is typed;
- the linker and transaction ABI boundaries exist;
- target execution uses exact target/deployment-bound transcripts;
- operation evidence subjects are canonical;
- resource prediction includes exact program bytes and complete transactions;
- candidate and final states remain distinct;
- the tree is clean.

Guide 13 must not begin against a Phase-4 concept or partially implemented boundary described as complete.

## 2.2 Semantic entry · `gate:guide13:semantic-entry`

Required:

- architecture release validation passes;
- live transfer remains in architecture and realization scope;
- live-transfer realization validates against architecture;
- compiler analysis for live transfer remains complete;
- realization and compiler relation censuses agree;
- explicit and private-committed value modes remain admitted by D005;
- explicit closed `U` remains mandatory;
- sponsor-value opacity remains enforced;
- live-receipt lifecycle obligations remain explicit;
- no target-specific type has entered realization or compiler core.

## 2.3 Target entry · `gate:guide13:target-entry`

Required:

- reviewed target definition validates;
- development binding is welded to that exact target;
- selected signature and sighash profile is reviewed and target-evidenced;
- input and output asset/program introspection is reviewed;
- explicit value introspection is reviewed;
- confidential-value conservation is available as external target evidence where the private plan uses it;
- CT proof materialization is available in the development executor;
- execution domain and leaf version are observed;
- policy and consensus verdicts remain separate;
- no mock is gate-eligible.

## 2.4 Evidence entry · `gate:guide13:evidence-entry`

Required:

- canonical operation evidence plans cannot be caller-forged;
- executor requests contain no expected outcome;
- transcripts retain exact target, deployment, and sent subjects;
- every failed canonical case fails the gate;
- infrastructure failure never counts as target rejection;
- non-target outcomes carry no target observations;
- exact executable provenance is validated;
- protocol records are strict and bounded;
- report bytes reproduce;
- safety and minimality have separate report roles.

## 2.5 Repository entry · `rule:guide13:repository-entry`

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

Record:

```text
starting revision
Phase-4 result revision
architecture schema and semantic hash
realization version
target contract revision
executor protocol revision
compact-ASH bundle/ABI candidate state
live-transfer compiler projection
clean-tree result
```

If the tree is not clean, stop and classify every change before continuing.

---

# 3. Preflight questions · `sec:guide13:preflight`

The execution guide derived from this concept must begin with a focused preflight review. At minimum it answers the following questions.

## 3.1 Phase-4 interface reuse · `q:guide13:phase4-reuse`

Can the Phase-4 compiler plan, relocatable bundle, linker, ABI, and evidence boundaries express:

- protocol owner authorization;
- multiple protocol output owners;
- split and merge;
- representation-specific proofs;
- distinct safety and minimality reports;
- confidential transaction construction;
- multi-owner signing?

Any missing dimension is recorded before implementation begins.

## 3.2 Signature profile · `q:guide13:signature-profile`

Does the selected target profile commit:

- every receipt input;
- every receipt output;
- every output asset;
- every output value or commitment;
- every output program;
- every output witness committed by the reviewed digest;
- transaction version and locktime;
- issuance absence;
- relevant taproot script-path data?

Can the profile be observed from the finalized witness rather than merely asserted by the builder?

## 3.3 Confidential construction boundary · `q:guide13:confidential-construction`

Can the development materializer construct:

- private one-to-one transfer;
- private split;
- private merge;
- several private inputs from different owners;
- several private outputs;
- private sponsor input/change where claimed?

Which material must be shared among collaborators to close blinding?

Does the current test path centralize all public test openings, and if so, which production claim is explicitly excluded?

## 3.4 Constructor metadata · `q:guide13:receipt-constructor`

How does the candidate constructor authenticate:

- live class;
- owner;
- static transfer program set;
- selected value representation;
- explicit `U`;
- target schema?

Does changing owner require a new constructor instance derived from canonical metadata?

Does any synthetic Guide-10 metadata field remain, and if so, why is it semantically necessary?

## 3.5 Confidential proof ownership · `q:guide13:confidential-proof`

Which layer owns the private transfer value proof?

Candidate outcomes include:

```text
external target consensus conservation

backend-local commitment-equality pattern

authenticated public opening

owner-authorized normalization
```

For lateral private transfer, the expected initial result is external target conservation plus complete closed-asset and object closure. The preflight must prove that this is sufficient and must not label target conservation as a local tapscript pattern.

## 3.6 Mixed representation · `q:guide13:mixed-representation`

Does Phase 5 support transactions mixing:

- explicit and private inputs;
- explicit and private outputs;
- explicit input to private output;
- private input to explicit output?

The default concept scope is:

```text
homogeneous explicit transfer:
    required

homogeneous private-committed transfer:
    required where target construction is supported

mixed representation:
    deferred until explicitly selected and covered
```

A mixed transaction must not enter because the target accepts it incidentally.

## 3.7 Output ordering · `q:guide13:output-order`

How are output positions determined without making private values part of a public canonical-order rule?

The initial candidate should preserve the typed request’s destination order as ABI presentation order.

It must not sort confidential outputs by private amount or blinding material.

The semantic projection compares an owner/class/value multiset and does not treat physical output order as protocol meaning unless a later decision makes it observable.

## 3.8 Evidence-role separation · `q:guide13:evidence-roles`

Can one report type keep separate:

- explicit safety;
- private safety;
- representation equivalence;
- disclosure minimality;
- confidential transaction consensus evidence;
- signer authorization evidence;
- resource evidence;
- lifecycle incompleteness?

If not, use separate report types rather than one report with optional fields whose absence changes its meaning.

## 3.9 Preflight gate · `gate:guide13:preflight`

Implementation begins only when:

- every question above has an explicit disposition;
- the selected target signature profile is exact;
- private test-material handling is explicitly public/test-only;
- no production secret interface is introduced;
- mixed representation scope is explicit;
- output-order policy is explicit;
- Phase-4 interface changes are the minimum required;
- safety and minimality report roles are separate;
- focused tests and full repository checks pass;
- the tree is clean.

---

# 4. Required authority · `sec:guide13:authority`

Guide 13 is governed by:

```text
Attestation
Realization
typed architecture
implemented ADRs
accepted decisions
accepted Guide-11 representation result
accepted Phase-4 compact-ASH pipeline
package contracts
Phase-5 card
this concept's eventual execution guide
```

Relevant accepted decisions include:

- D001 typed Rust source;
- D002 target-independent realization;
- D003 tapscript first;
- D004 translation validation;
- D005 value-parametric representation with explicit closed assets;
- D006 canonical transaction and witness ABI;
- D007 Petgraph substrate;
- D008 exact semantic mathematics.

Relevant realization obligations include:

- live and time-locked classes remain structurally distinct;
- live transfer is class-closed;
- every consumed owner authorizes;
- aggregate `U` value is preserved;
- recipient safety is not aggregate conservation;
- exact canonical and open-flow partitions remain separate;
- sponsor values remain opaque;
- no root participates;
- no issuance or destruction occurs;
- no specialized event is emitted;
- actual target value remains authoritative;
- private representation must retain lifecycle exits.

No package parses these documents as semantic input.

---

# 5. Semantic live-transfer contract · `sec:guide13:semantics`

## 5.1 Inputs and outputs · `def:guide13:operation`

Let the operation consume \(n\) live receipts and create \(m\) live receipts:

\[1\le n\le N_{\mathrm{in}}\]

\[1\le m\le N_{\mathrm{out}}\]

where \(N_{\mathrm{in}}\) and \(N_{\mathrm{out}}\) are Phase-5 candidate assignments for the architecture-owned transfer bounds.

Each input \(i\) carries:

```text
object:
    RECEIPT_L

asset:
    explicit U

owner:
    oᵢ

semantic value:
    xᵢ > 0

representation:
    selected explicit or private-committed mode
```

Each output \(j\) carries:

```text
object:
    RECEIPT_L

asset:
    explicit U

owner:
    pⱼ

semantic value:
    yⱼ > 0

representation:
    selected explicit or private-committed mode
```

The exact conservation relation is:

\[\sum_{i=0}^{n-1}x_i=\sum_{j=0}^{m-1}y_j\]

## 5.2 Semantic freedom · `rule:guide13:transfer-freedom`

The transfer request may change:

- destination owners;
- output count;
- denominations;
- physical grouping;
- representation where the selected candidate explicitly supports a transition.

It may not change:

- aggregate semantic value;
- asset `U`;
- live class;
- operation family;
- required authorization;
- root state;
- issuance;
- destruction;
- event semantics.

## 5.3 Split and merge · `rule:guide13:split-merge`

The semantic relation admits:

```text
one-to-one
one-to-many split
many-to-one merge
many-to-many redistribution
```

provided:

- every input is live;
- every output is live;
- every input owner authorizes;
- exact aggregate value is preserved;
- every output is recognized and classified;
- no unauthorized value escapes into another object family.

No implementation may claim general split/merge support from one-to-one evidence alone.

## 5.4 Class closure · `rule:guide13:live-closure`

Every accepted protocol input and output is `RECEIPT_L`.

The following are forbidden as transfer inputs or outputs:

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

A time-locked receipt can neither enter nor leave a live transfer.

Class closure is structural and independently tested in both directions.

## 5.5 Owner authorization · `rule:guide13:owner-authorization`

For each consumed receipt input:

- the owner is authenticated from the exact predecessor constructor;
- the owner’s target key encoding is valid;
- the owner’s signature verifies under the selected sighash profile;
- the signature commits the finalized protected transaction;
- an unknown target key type cannot enter the forward-compatibility success path.

The last item is load-bearing: the reviewed target may treat an unknown nonempty key encoding as success without verification. Therefore the live-receipt constructor must constrain owner-key encoding independently of the signature primitive’s success result.

## 5.6 Canonical `U` flow · `rule:guide13:u-flow`

The one canonical flow is:

```text
asset:
    U

kind:
    lateral

sources:
    every live-receipt input exactly once

destinations:
    every live-receipt output exactly once

destructions:
    none

issuance:
    none
```

The value relation is exact aggregate conservation.

No source or destination appears in:

- another canonical flow;
- an issuance;
- a sponsor flow;
- a destruction.

## 5.7 Root and projection policy · `rule:guide13:roots-projections`

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

The transfer emits no burn record and creates no ASH.

## 5.8 Sponsor flow · `rule:guide13:sponsor-flow`

The only open flow is the optional fee-sponsor region.

The transfer’s `U` value never funds the L-BTC fee.

The sponsor region may contain:

- ordinary L-BTC sponsor inputs;
- optional sponsor change;
- target fee role.

The protocol relation retains sponsor membership and verdict, not individual sponsor values.

---

# 6. Representation plans · `sec:guide13:representations`

## 6.1 Representation profile · `def:guide13:representation-profile`

Guide 13 distinguishes at least:

```rust
pub enum LiveTransferRepresentationPlan {
    Explicit,
    PrivateCommitted,
}
```

No `PublicCommitted` variant enters Phase 5 while Guide 11 leaves it deferred.

No mixed variant enters until (`q:guide13:mixed-representation`) is decided positively.

## 6.2 Explicit plan · `candidate:guide13:explicit-plan`

The explicit plan uses:

- explicit `U` asset;
- explicit input values;
- explicit output values;
- checked aggregate target arithmetic;
- one exact canonical `U` flow;
- complete live-receipt output closure;
- per-input owner signatures;
- optional sponsor region.

The coordinator computes both input and output sums and checks equality.

Every arithmetic success flag is consumed immediately.

Every amount re-enters the semantic amount domain.

## 6.3 Private-committed plan · `candidate:guide13:private-plan`

The private plan uses:

- explicit `U` asset;
- confidential input values;
- confidential output values;
- target Confidential-Transaction conservation as external evidence;
- complete protocol input/output classification;
- complete live-class closure;
- per-input owner signatures;
- optional sponsor region.

The protocol program does not decode or open receipt values merely to re-prove conservation the target already enforces.

The private plan is sound only when the complete transaction relation establishes:

```text
every U input is a live receipt source
every U output is a live receipt destination
no issuance of U exists
no destruction of U exists
no unclassified U output exists
target CT conservation holds
```

Then target conservation over explicit asset `U` is exactly the semantic transfer aggregate relation.

## 6.4 No amount-inspection back door · `rule:guide13:private-opacity`

A private transfer program must not:

- open a receipt amount;
- compare a receipt amount with zero as a protocol check;
- make amount explicit for diagnostics;
- use amount-dependent output order;
- require a public subtotal;
- require a public equality witness duplicating CT conservation.

Positive semantic value remains a requirement of valid receipt construction and target rangeproof policy. It is not re-established by disclosing the exact value.

## 6.5 Representation equivalence · `rule:guide13:representation-equivalence`

For paired explicit and private fixtures, compare:

- same semantic input multiset;
- same semantic destination multiset;
- same owner set;
- same live class;
- same explicit `U`;
- same authorization result;
- same absence of roots, issuance, destruction, and events;
- same sponsor-role verdict;
- same accepted target-independent successor projection.

The target bytes, commitments, proofs, and resource use may differ.

## 6.6 Mixed representation · `rem:guide13:mixed-representation`

Mixed explicit/private transactions are not implied by the existence of both homogeneous plans.

If deferred, every public type and report states:

```text
explicit homogeneous:
    supported candidate

private homogeneous:
    supported candidate where evidenced

mixed:
    unsupported in Phase 5
```

A target accepting an ad hoc mixed transaction does not widen the candidate ABI.

---

# 7. Live-receipt constructor · `sec:guide13:constructor`

## 7.1 Constructor meaning · `rule:guide13:constructor-meaning`

The candidate constructor binds:

- exact explicit `U` asset;
- live receipt object family;
- owner metadata;
- live class;
- selected value-representation plan;
- static transfer program set;
- target contract revision;
- leaf version;
- internal-key policy;
- candidate ABI schema.

Value remains a target transaction field and is not duplicated as metadata.

## 7.2 Owner metadata · `rule:guide13:owner-metadata`

Owner metadata has one canonical encoding.

It must reject:

- wrong width;
- unknown key encoding;
- alternate encoding of the same key;
- malformed key;
- omitted owner;
- owner bytes outside the constructor commitment;
- target forward-compatibility key forms not approved by the live-receipt policy.

The owner key is public authorization identity, not private key material.

## 7.3 Live class is structural · `rule:guide13:class-constructor`

The constructor distinguishes live from time-locked receipts.

A live-transfer leaf cannot be valid under the time-locked constructor.

A time-locked output program cannot satisfy a live destination role.

Class is not inferred from target value, amount, owner, or leaf selection alone.

## 7.4 Candidate lifecycle constructor · `rule:guide13:candidate-constructor`

The Phase-5 constructor may contain only the implemented transfer leaves.

It must not include spendable placeholders for burn or redemption.

Therefore it records:

```text
implemented:
    transfer-live-receipts

outstanding:
    burn
    redeem

release-complete:
    false
```

The constructor is candidate infrastructure and not the final live-receipt constructor.

## 7.5 Internal key · `rule:guide13:internal-key`

Use the inherited public unspendable internal-key policy if its assumptions remain valid.

No:

- owner key;
- operator key;
- release key;
- wallet key;
- generated-and-discarded key;
- caller-selected key.

No accepted key-path escape exists.

## 7.6 Constructor continuity · `rule:guide13:constructor-continuity`

Each input program is authenticated as the constructor over its committed owner and selected representation.

Each output program is reconstructed from:

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

The transfer may change owner and representation only where the typed request and selected plan permit it.

The output constructor cannot be caller-supplied as arbitrary bytes.

---

# 8. Candidate transaction ABI · `sec:guide13:abi`

## 8.1 Input layout · `rule:guide13:input-layout`

The initial candidate input layout is:

```text
inputs 0..n-1:
    live receipt family

inputs n..n+s-1:
    optional sponsor suffix
```

Receipt inputs are sorted by canonical outpoint order.

Sponsor inputs are sorted by canonical outpoint order inside their suffix.

Input 0 is the canonical coordinator.

Duplicates and protocol/sponsor overlap reject before sorting.

## 8.2 Output layout · `rule:guide13:output-layout`

The initial candidate output layout is:

```text
outputs 0..m-1:
    live receipt destinations, in typed request order

next optional role:
    sponsor change

final target role where required:
    fee output
```

The request order is ABI presentation order and is committed by every owner signature.

It is not semantic output identity. Accepted semantic projection compares the destination owner/class/value multiset.

Private output order must not derive from private amount or blinding data.

## 8.3 Coordinator · `rule:guide13:coordinator`

The first receipt input is the coordinator.

The coordinator enforces transaction-global relations:

- exact input and output family counts;
- complete protocol family ranges;
- live-class output closure;
- explicit closed-asset closure;
- exact aggregate value proof for the selected representation;
- sponsor-region isolation;
- root absence;
- issuance and destruction absence;
- projection policy;
- exact role order.

Every receipt input, coordinator included, separately enforces its own owner authorization and local predecessor recognition.

## 8.4 Typed request · `rule:guide13:request`

A live-transfer request may select:

- receipt input outpoints;
- ordered destination entries;
- destination owners;
- destination semantic values;
- one admitted representation plan;
- optional sponsor capabilities;
- optional sponsor-change destination;
- explicit public test randomness for confidential fixtures.

It may not select:

- input owner;
- input class;
- input asset;
- output class;
- output asset;
- output constructor bytes;
- target program;
- coordinator;
- family positions;
- witness order;
- target fee role;
- issuance;
- destruction;
- specialized event.

## 8.5 Destination entries · `def:guide13:destination`

Illustrative typed form:

```rust
pub struct LiveReceiptDestination {
    pub owner: OwnerKey,
    pub value: SemanticAmount,
}
```

For confidential construction, the request may additionally select an approved public representation preference. Private opening and blinding material remains in a separate test or authorized construction capability and never in the semantic destination identity.

## 8.6 Output finalization · `rule:guide13:finalization`

Before any owner signature request is produced:

1. every input is fixed;
2. every output role is fixed;
3. every destination owner is fixed;
4. every destination semantic value is fixed;
5. every explicit amount or confidential commitment is fixed;
6. every output constructor is fixed;
7. sponsor change is fixed;
8. fee role is fixed;
9. target proofs are fixed where covered by the sighash;
10. transaction version and locktime are fixed.

A post-signing edit invalidates every affected signature and must be observed as such in target evidence.

## 8.7 Candidate status · `rule:guide13:abi-status`

The result is:

```text
CandidateLiveTransferAbi
```

It is not final, and no ABI digest is minted by default.

---

# 9. Owner signing and confidential construction · `sec:guide13:construction`

## 9.1 Signer boundary · `rule:guide13:signer-boundary`

The transaction library holds no production private key.

A signing request binds:

- exact target;
- candidate linked bundle;
- candidate ABI;
- operation;
- finalized transaction;
- input outpoint;
- owner public identity;
- target sighash profile;
- target message;
- expected witness role.

The response contains a target signature and no private key material.

Phase-5 tests may use public disposable test signing scalars under ADR-015.

## 9.2 Multi-owner signing · `rule:guide13:multi-owner`

A multi-owner transfer is an interactive construction protocol.

All owners sign the same finalized protected transaction.

The builder rejects:

- one owner missing;
- one signature bound to another transaction;
- one signature under another sighash profile;
- duplicate signature response;
- unexpected signer;
- owner/signature mismatch;
- post-signing output mutation;
- input added after signatures;
- output removed after signatures.

The operation does not proceed with a partial owner set.

## 9.3 Same owner, several inputs · `rule:guide13:repeated-owner`

Where one owner controls several receipt inputs:

- the semantic owner requirement may appear once in the owner set;
- each target input still satisfies its own spending condition;
- the candidate may request one signature per input if the target digest is input-specific;
- reports distinguish distinct semantic owners from concrete signature count.

No report inflates “three signatures from Alice” into “three independent owners authorized.”

## 9.4 Confidential test material · `rule:guide13:test-confidential-material`

Private-committed transfer evidence uses public disposable test material:

- fixed test owner scalars;
- fixed input openings;
- fixed value blinders;
- fixed nonce seeds;
- fixed rangeproof seeds;
- fixed output blinding policy.

The material:

- authorizes nothing of value;
- is confined to a disposable development chain;
- is labeled test-only;
- is never derived from production material;
- is reproducible from explicit fixture inputs;
- is destroyed with the environment where practical.

Its use does not create a production secret-bearing interface.

## 9.5 Multi-owner blinding limitation · `rem:guide13:blinding-protocol`

A centralized test harness may know every public fixture opening and construct a balanced confidential transaction.

That demonstrates:

- target feasibility;
- commitment conservation;
- semantic equivalence;
- finite safety evidence.

It does not demonstrate a privacy-preserving production protocol among mutually distrusting owners.

The Phase-5 result must state which construction model was used:

```text
central public-fixture construction

cooperative owner opening exchange

distributed blinding protocol

external wallet materializer
```

Only the first is assumed by this concept.

Production multi-party blinding remains a separate design and security boundary.

## 9.6 Full output closure under confidentiality · `rule:guide13:confidential-closure`

For private transfer, target conservation is sound only over the complete explicit-`U` family.

The candidate therefore authenticates:

- every input carrying `U`;
- every output carrying `U`;
- every live constructor;
- every target role;
- issuance absence;
- destruction absence;
- no hidden `U` output;
- no confidential asset commitment.

A hidden output of another asset cannot absorb `U`.

A hidden confidential `U` output is prohibited by explicit asset closure before target conservation is used.

---

# 10. Tapscript patterns · `sec:guide13:patterns`

Each pattern states:

- semantic owner;
- target prerequisites;
- typed instructions;
- initial and final stack;
- non-aborting failures;
- aborting failures;
- witness roles;
- source requirements;
- constructibility;
- disclosure;
- resource formula;
- positive and negative vectors.

## 10.1 Local receipt recognition · `rule:guide13:input-recognition`

Every receipt input proves:

```text
asset = exact linked U
program = exact linked live-receipt constructor
owner metadata = canonical
class = live
representation = selected plan
current input lies in the receipt range
```

A target script match without exact `U` is insufficient.

## 10.2 Per-input owner authorization · `rule:guide13:input-owner`

Every receipt input:

- obtains the owner from authenticated predecessor metadata;
- constrains the owner key to the recognized encoding;
- verifies the owner signature;
- uses the selected output-committing sighash profile;
- rejects empty signature;
- rejects invalid signature;
- rejects unknown-key success-without-verification;
- commits the finalized transaction.

The target signature primitive’s success is not sufficient unless key encoding is independently authenticated.

## 10.3 Output constructor closure · `rule:guide13:output-constructors`

The coordinator verifies every protocol output is:

```text
RECEIPT_L
explicit U
canonical owner metadata
selected value representation
linked constructor
```

No target output carrying `U` remains outside the receipt range.

## 10.4 Explicit conservation pattern · `rule:guide13:explicit-conservation`

The explicit coordinator computes:

\[\sum_i x_i\]

and:

\[\sum_j y_j\]

with checked target arithmetic, then requires equality.

Every addition:

- uses exact target width;
- checks its success flag immediately;
- rejects overflow;
- re-establishes the amount domain.

The final equality is verified, not left as a branchable truth value.

## 10.5 Private conservation plan · `rule:guide13:private-conservation`

The private plan does not open values in script.

It establishes:

```text
explicit U classification
+
complete live-receipt input family
+
complete live-receipt output family
+
no U issuance
+
no U destruction
+
target CT conservation
```

The value-conservation claim remains external target evidence. No tapscript pattern ID claims to implement it.

## 10.6 Coordinator uniqueness · `rule:guide13:coordinator-uniqueness`

The coordinator program is valid only at receipt input 0.

Member programs are valid only at later receipt positions.

Exactly one coordinator executes in every valid transfer.

## 10.7 Sponsor isolation · `rule:guide13:sponsor-isolation`

The coordinator authenticates:

- exact sponsor suffix;
- exact sponsor-change role;
- exact target fee role;
- protocol/sponsor disjointness;
- no closed protocol asset in sponsor roles;
- no second sponsor envelope;
- sponsor amount absent from protocol operands.

Sponsor input authorization remains in sponsor programs.

## 10.8 Root, issuance, destruction, and event absence · `rule:guide13:absence`

The candidate bundle and ABI admit no:

```text
STATE
RESV
PACE
ENT_AUTH
DIST_AUTH
issuance record
destruction record
burn record
burn projection
clear projection
residue projection
```

Absence is checked in compiler plan, backend bundle, linker output, ABI, and target vector layers.

## 10.9 Final stack · `rule:guide13:final-stack`

Every receipt program:

- leaves one canonical true result on success;
- checks every arithmetic and verification result;
- leaves no alternate-stack residue;
- has no non-aborting failure state satisfying final truth;
- remains within target stack and element limits.

---

# 11. Linking · `sec:guide13:linking`

## 11.1 Inherited linker · `rule:guide13:linker-reuse`

Guide 13 extends the Phase-4 linker only where live transfer requires:

- owner-parameterized constructors;
- representation-specific program roles;
- multiple target programs per operation;
- signature witness roles;
- confidential proof resources;
- distinct safety/minimality provenance.

The symbol and relocation model remains unchanged unless a concrete typed need requires generalization.

## 11.2 Symbols · `rule:guide13:symbols`

Expected typed symbols include:

```text
U asset identifier
live receipt constructor
explicit transfer coordinator
explicit transfer member
private transfer coordinator
private transfer member
owner metadata schema
representation profile
target leaf version
unspendable internal key
candidate input bound
candidate output bound
candidate sponsor bound
selected sighash profile
target fee role
```

## 11.3 Representation-specific leaves · `rule:guide13:representation-leaves`

The explicit and private plans use distinct coordinator leaves unless one complete typed pattern proves that a shared leaf enforces both without attacker-selected dispatch.

Member leaves may be shared only if:

- local predecessor recognition is representation-parametric;
- owner authorization is identical;
- no representation-specific fact is silently omitted;
- target-native vectors cover both forms through the shared leaf.

## 11.4 Deterministic tree · `rule:guide13:taptree`

The live-receipt candidate tree contains the complete Phase-5 transfer leaf set.

The selected deterministic tree policy records:

- exact leaf set;
- leaf weights;
- stable tie-break;
- maximum depth;
- representation-specific roles;
- candidate lifecycle status.

For the small leaf set, exhaustive enumeration checks the declared objective.

## 11.5 Carrier closure · `rule:guide13:carrier-closure`

The linker compares compiler-required, backend-emitted, and linked-reachable carriers for every representation plan and execution case.

A carrier existing only in the explicit plan does not satisfy a private-plan relation.

A private-plan external conservation requirement cannot be “carried” by a local target program merely because the transaction eventually passes consensus.

## 11.6 Candidate linked bundle · `rule:guide13:linked-bundle`

The candidate bundle retains:

- exact live-transfer scope;
- exact target projection;
- candidate bounds;
- explicit and private plan statuses;
- linked constructors;
- linked programs;
- selected sighash profile;
- concrete placements;
- ABI handoff;
- resource formulas;
- external target evidence;
- outstanding burn and redemption lifecycle;
- explicit non-production status.

No linked-bundle digest is minted by default.

---

# 12. Evidence architecture · `sec:guide13:evidence`

## 12.1 Canonical plan · `rule:guide13:evidence-plan`

Introduce a canonical, validated live-transfer evidence plan with private fields and no unchecked constructor.

It derives from:

- compiler coverage;
- candidate linked bundle;
- candidate ABI;
- canonical safety mutation registry;
- canonical minimality pair registry;
- exact target/deployment binding.

Ad hoc live-transfer vectors produce experimental reports only.

## 12.2 Safety report · `def:guide13:safety-report`

The safety report answers:

```text
Did every required invalid transfer fail at the owning boundary,
and did every valid transfer preserve the exact semantic relation?
```

It carries:

- exact candidate bundle and ABI subject as typed content;
- target and deployment;
- representation plan;
- case census;
- relation census;
- mutation census;
- target observations;
- semantic projections;
- construction/target/report-layer distinction;
- executor provenance;
- resource observations;
- explicit lifecycle incompleteness.

## 12.3 Minimality report · `def:guide13:minimality-report`

The minimality report answers:

```text
Does the private-committed plan perform the same semantic transfer
with strictly less amount disclosure than the explicit plan?
```

It carries paired cases:

- one semantic fixture;
- explicit target materialization;
- private target materialization;
- both target verdicts;
- both semantic projections;
- disclosure sets;
- resource comparison;
- constructibility assumptions;
- lifecycle status.

A pair passes only when:

- both target transactions accept;
- both semantic projections equal the expected transfer;
- private amount remains absent from public protocol outputs and reports;
- private plan adds no secret dependency outside its declared construction model.

## 12.4 No missing positive private case · `rule:guide13:private-positive`

Private safety cannot be established by rejection-only evidence.

At least one valid private one-to-one transfer must accept.

To claim private split, at least one valid private split must accept.

To claim private merge, at least one valid private merge must accept.

To claim general private many-to-many transfer, at least one representative many-to-many case must accept and every bound family must be covered by exact structural vectors.

## 12.5 Report role separation · `rule:guide13:report-separation`

The safety report cannot satisfy minimality.

The minimality report cannot satisfy safety.

The target CT conservation report cannot satisfy either by itself.

The owner-signature report cannot satisfy value conservation.

The resource report cannot satisfy semantic projection.

No report digest is introduced in Phase 5.

---

# 13. Semantic fixtures and projections · `sec:guide13:fixtures`

## 13.1 Model-valid origins · `rule:guide13:model-fixtures`

Positive fixtures begin from model-valid worlds produced through the declared model path or an explicitly approved synthetic setup preserving the invariant.

A live transfer fixture binds:

- exact predecessor world;
- exact transfer request;
- exact canonical order;
- exact successor world;
- exact appended certificate.

The executable-model transition runs through the invariant wrapper.

## 13.2 Expected semantics · `rule:guide13:expected-semantics`

Expected semantic behavior derives from:

- realization relations;
- executable model;
- typed projection adapters.

The candidate target backend does not generate the expected owner, class, amount, or authorization result used to judge itself.

## 13.3 Target materialization · `rule:guide13:target-materialization`

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
test-only signing and confidential construction capabilities
```

The semantic fixture contains no target position, script, control path, signature byte, or blinder.

## 13.4 Accepted projection · `rule:guide13:accepted-projection`

For every accepted transaction compare:

- exact consumed live receipt set;
- exact created live receipt multiset;
- exact input owners;
- exact destination owners;
- exact semantic values;
- exact aggregate conservation;
- live class;
- explicit `U`;
- every-owner authorization result;
- no root;
- no issuance;
- no destruction;
- lateral `U` flow;
- sponsor-region verdict;
- no specialized event;
- required transition certificate.

For public disposable confidential fixtures, the semantic harness may know the fixture amounts. The canonical target report need not publish private-plan amounts when the report’s role does not require them.

---

# 14. Required safety vectors · `sec:guide13:safety`

## 14.1 Positive explicit cases · `tab:guide13:positive-explicit`

Required:

- one input to one output;
- one input split into two outputs;
- several inputs merged into one output;
- several inputs to several outputs;
- same owner on all inputs;
- several distinct input owners;
- one destination owner;
- several destination owners;
- input/output values at semantic boundaries;
- canonical input reordering;
- sponsorless transaction;
- sponsored transaction;
- sponsor change present;
- sponsor change absent;
- candidate maximum input count;
- candidate maximum output count.

## 14.2 Positive private cases · `tab:guide13:positive-private`

Where private support is claimed:

- one private input to one private output;
- private split;
- private merge;
- private many-to-many representative;
- several distinct owners;
- private sponsor input/change where claimed;
- both target value-commitment parities;
- output blinding factors under deterministic public fixtures;
- exact target CT conservation;
- same semantic projection as explicit paired case.

## 14.3 Cardinality faults · `tab:guide13:cardinality-faults`

Required:

- zero receipt inputs;
- zero receipt outputs;
- input count above candidate maximum;
- output count above candidate maximum;
- sponsor count above candidate maximum;
- duplicate input;
- duplicate output role;
- family range overlap;
- family range gap;
- unexpected protocol output.

## 14.4 Owner faults · `tab:guide13:owner-faults`

Required:

- missing owner signature;
- wrong owner signature;
- one omitted owner in a multi-owner transfer;
- signature from an unrelated owner;
- duplicated signature substituted for another owner;
- signature under wrong sighash profile;
- signature over another transaction;
- output mutation after signing;
- input addition after signing;
- unknown public-key encoding that would otherwise succeed unverified;
- empty signature;
- malformed signature;
- same owner on several inputs with one required concrete input signature omitted.

## 14.5 Class and constructor faults · `tab:guide13:class-faults`

Required:

- time-locked receipt input;
- time-locked receipt output;
- ASH input;
- ASH output;
- distribution vault output;
- entitlement output;
- bare `U` output;
- wrong owner metadata;
- wrong live metadata;
- wrong constructor schema;
- transfer/burn program mixture;
- transfer/relabel program mixture;
- constructor from another candidate bundle;
- malformed control path;
- key-path escape attempt.

## 14.6 Asset faults · `tab:guide13:asset-faults`

Required:

- wrong explicit asset;
- confidential asset commitment;
- unclassified closed `U`;
- output carrying `U` outside receipt family;
- sponsor role carrying `U`;
- target fee role carrying `U`;
- foreign open asset under receipt-shaped program;
- copied `U` constructor with another asset.

## 14.7 Value faults · `tab:guide13:value-faults`

Required:

- aggregate output one below input;
- aggregate output one above input;
- one output zero;
- one amount outside semantic domain;
- explicit arithmetic overflow;
- private CT imbalance;
- wrong private blinding balance;
- malformed rangeproof;
- malformed surjection proof;
- copied value commitment;
- private output omitted;
- hidden private `U` output;
- explicit/private representation mismatch with selected plan;
- mixed representation under a homogeneous-only ABI.

## 14.8 Canonical-partition faults · `tab:guide13:partition-faults`

Required:

- omit one receipt source;
- cite one source twice;
- cite one destination twice;
- claim one output through two flows;
- leave one `U` output unwitnessed;
- add issuance;
- add destruction;
- route value into ASH;
- route value into time-locked receipt;
- preserve aggregate total while stealing from one owner’s intended output before authorization;
- add another `U` flow that offsets the first.

## 14.9 Sponsor faults · `tab:guide13:sponsor-faults`

Required:

- sponsor/protocol overlap;
- two sponsor envelopes;
- foreign sponsor asset;
- missing sponsor authorization;
- sponsor change in protocol output range;
- target fee role substituted for sponsor change;
- sponsor change substituted for fee role;
- sponsor member unclassified;
- report contains sponsor amount;
- report contains sponsor opening;
- balanced theft with all sponsor amounts positive;
- zero-valued ordinary sponsor member under exact role structure;
- confidential sponsor values where claimed.

## 14.10 Root and event faults · `tab:guide13:absence-faults`

Required:

- add STATE input;
- add RESV input;
- add PACE input;
- add authority input;
- create root-shaped output;
- emit burn record;
- emit `tag-burn`;
- emit redeem destruction;
- emit clear destruction;
- emit residue;
- claim burn projection;
- claim clear projection;
- omit semantic transition certificate.

## 14.11 ABI and linking faults · `tab:guide13:abi-faults`

Required:

- wrong coordinator;
- two coordinators;
- no coordinator;
- member leaf at coordinator;
- coordinator leaf at member;
- receipt and sponsor positions exchanged;
- output request order changed after signing;
- witness items reordered;
- control block from another program;
- unresolved relocation;
- wrong `U` relocation;
- wrong owner-schema relocation;
- explicit plan paired with private ABI;
- private plan paired with explicit program;
- target bytes changed after ABI validation;
- ad hoc raw transaction bypasses safe construction.

---

# 15. Required minimality vectors · `sec:guide13:minimality`

## 15.1 Paired cases · `rule:guide13:paired-minimality`

At minimum compare semantically equivalent:

- explicit one-to-one and private one-to-one;
- explicit split and private split;
- explicit merge and private merge;
- explicit many-to-many and private many-to-many where supported;
- explicit sponsor and confidential sponsor where supported.

Each pair starts from one target-independent semantic fixture.

## 15.2 Pair acceptance · `rule:guide13:minimality-acceptance`

A pair supports minimality only when:

- both transactions are constructible;
- both target transactions accept;
- both semantic projections match the same expected transfer;
- both preserve owner and live class;
- both preserve explicit `U`;
- both satisfy complete family closure;
- the private plan publishes no exact receipt amount;
- no private opening enters the canonical report;
- no owner-private dependency is introduced beyond the declared construction model;
- lifecycle status is equal.

## 15.3 Disclosure comparison · `rule:guide13:disclosure-comparison`

Record separately:

```text
semantic disclosure
target-safety disclosure
deployment-policy disclosure
transaction-shape leakage
```

For the private plan, expected amount disclosure is:

```text
receipt exact input amounts:
    private

receipt exact output amounts:
    private

aggregate semantic transfer amount:
    not published by protocol

owners:
    public constructor metadata

asset U:
    explicit

input/output counts:
    public transaction shape
```

Any extra exact amount disclosure requires one typed reason.

## 15.4 Minimality failure · `rule:guide13:minimality-failure`

A private pair fails minimality if:

- the private transaction rejects;
- it needs explicit receipt values for protocol predicates;
- it needs a public subtotal;
- the report publishes fixture openings;
- it needs owner-private data unavailable to the declared constructor;
- it changes semantic output;
- it loses an exit the explicit form retains;
- it exceeds a hard target limit while the explicit form fits;
- it uses confidential asset identity.

A failed minimality result does not imply the explicit transfer is unsafe.

## 15.5 No universal privacy claim · `rem:guide13:privacy-nonclaim`

A successful private transfer demonstrates confidentiality of the tested receipt amounts under the target’s commitment and proof assumptions.

It does not establish:

- owner anonymity;
- transaction graph privacy;
- output-count privacy;
- timing privacy;
- sponsor privacy beyond the erased-value claim;
- production wallet privacy;
- production side-channel resistance;
- universal transaction confidentiality.

---

# 16. Lifecycle · `sec:guide13:lifecycle`

## 16.1 Required exits · `rule:guide13:required-exits`

A live receipt requires:

```text
transfer
burn
redeem
```

Phase 5 implements transfer only.

Therefore both explicit and private candidate receipts remain lifecycle-incomplete.

## 16.2 Private lifecycle · `rule:guide13:private-lifecycle`

Guide 11 selected an explicit public boundary for amount-dependent operations.

A private live receipt therefore needs an owner-authorized normalization path before direct public burn or redemption, unless a later target result supersedes the Guide-11 deferral.

Phase 5 does not implement normalization as a protocol operation unless architecture and realization are deliberately extended under their versioning rules.

Accordingly the candidate private live-receipt status is:

```text
transfer:
    candidate supported

burn:
    direct private support deferred

redeem:
    direct private support deferred

normalization:
    accepted target research path,
    not a current architecture operation

release-complete:
    false
```

## 16.3 Constructor consequence · `rule:guide13:lifecycle-constructor`

The Phase-5 live-receipt constructor does not contain pretend burn, redeem, or normalization leaves.

It records the missing exits as typed obligations.

A valid transfer output under the candidate constructor cannot be described as a final production live receipt.

## 16.4 Lifecycle equality in minimality · `rule:guide13:lifecycle-minimality`

The private plan cannot be described as equally supported merely because transfer works.

The minimality report states:

```text
equal for transfer semantics

not equal for complete live-receipt lifecycle
```

until the required exits exist.

This distinction prevents a green private transfer from becoming a general “private live receipts supported” claim.

---

# 17. Resource evidence · `sec:guide13:resources`

## 17.1 Candidate bounds · `rule:guide13:candidate-bounds`

Phase 5 evaluates candidate values for:

```text
TRANSFER_INPUT_MAX
TRANSFER_OUTPUT_MAX
FEE_SPONSOR_INPUT_MAX
```

Candidate values are not final deployment calibration.

Initial deterministic candidates may include:

```text
receipt inputs:
    1, 2, 4, 8, 16, 32, 64

receipt outputs:
    1, 2, 4, 8, 16, 32, 64

sponsor inputs:
    0, 1, 2, 4, 8, 16
```

The useful Phase-5 candidate supports split, merge, and at least two distinct input owners.

## 17.2 Complete transaction measurements · `rule:guide13:measurements`

Measure:

- one input/one output;
- one input/multiple outputs;
- multiple inputs/one output;
- multiple inputs/multiple outputs;
- candidate maximum input family;
- candidate maximum output family;
- maximum distinct-owner signatures;
- repeated owner across several inputs;
- explicit values;
- confidential values;
- sponsorless;
- sponsored;
- sponsor change present;
- sponsor change absent;
- largest rangeproof and surjection-proof forms;
- deepest candidate control path.

## 17.3 Resource dimensions · `tab:guide13:resource-dimensions`

Record separately:

- coordinator program bytes;
- member program bytes;
- constructor bytes;
- taptree depth;
- control bytes;
- signature witness bytes;
- confidential proof bytes;
- initial witness items;
- peak main stack;
- peak alternate stack;
- maximum item;
- validation budget;
- transaction weight;
- consensus verdict;
- relay-policy verdict;
- construction time as diagnostic only;
- target execution time as diagnostic only.

Timing never enters semantic identity or acceptance.

## 17.4 Explicit/private comparison · `rule:guide13:resource-comparison`

For paired semantic fixtures compare:

```text
explicit transaction weight
private transaction weight

explicit witness bytes
private witness and proof bytes

explicit stack
private stack

explicit construction requirements
private construction requirements
```

A private plan may be semantically and minimally preferable while more expensive. Resource cost and disclosure are separate objectives.

## 17.5 Prediction and observation · `rule:guide13:resource-prediction`

Backend, linker, and ABI resource predictions are compared with real target observations.

A mismatch fails the resource report even if the target accepted.

No one transaction is claimed to maximize every dimension unless proved.

## 17.6 Phase-5 result · `rule:guide13:resource-result`

Phase 5 may record:

```text
candidate transfer bounds fit the tested candidate bundle and ABI
```

It must not record final production transfer bounds.

Final calibration waits for the exact final bundle and ABI in the release phase.

---

# 18. Relation-indexed coverage · `sec:guide13:coverage`

For every relation-case and representation plan, require:

```text
relation identity
execution case
representation plan
activation
selected proof
carrier
positive requirement
negative requirement
expected evidence boundary
target verdict where applicable
accepted semantic projection where applicable
mutation layer
collateral relation closure
```

## 18.1 Explicit coverage · `rule:guide13:explicit-coverage`

Every active explicit relation-case receives:

- valid explicit transaction;
- focused invalid transaction;
- reachable carrier;
- target verdict;
- accepted semantic projection.

## 18.2 Private coverage · `rule:guide13:private-coverage`

Every private relation-case receives appropriate evidence:

- object, class, owner, and closed-asset relations through target programs;
- value conservation through target CT evidence;
- proof construction through transaction evidence;
- disclosure through minimality evidence;
- lifecycle through explicit incomplete status.

No one report row is allowed to stand for all four.

## 18.3 Authorization coverage · `rule:guide13:authorization-coverage`

Every distinct authorization seam has positive and negative coverage:

- recognized key and valid signature;
- missing signature;
- wrong owner;
- invalid signature;
- empty signature;
- unknown-key form;
- output mutation;
- input mutation;
- incomplete owner set;
- sponsor-owner omission.

## 18.4 Conditional coverage · `rule:guide13:conditional-coverage`

Where sponsor or representation conditions activate a relation, require:

```text
inactive valid
active valid
active invalid
```

Examples:

```text
no sponsor:
    sponsor relation inactive and transaction valid

sponsor present:
    sponsor relation active and valid

sponsor present with overlap:
    sponsor relation active and invalid
```

and:

```text
explicit plan:
    explicit arithmetic active

private plan:
    explicit arithmetic inactive
    external CT conservation active
```

## 18.5 External evidence · `rule:guide13:external-evidence`

Target CT conservation and selected signature/sighash semantics remain explicit evidence roles.

The operation report binds them to:

- exact target;
- exact deployment;
- exact candidate bundle;
- exact candidate ABI;
- exact transaction bytes;
- exact case.

A mock or abstract validator does not satisfy either.

---

# 19. Assurance boundaries · `sec:guide13:assurance`

## 19.1 Realization · `rem:guide13:realization-assurance`

Establishes target-independent live-transfer semantics.

Does not establish target representation, signature bytes, or CT behavior.

## 19.2 Compiler · `rem:guide13:compiler-assurance`

Establishes complete relation, proof, source, constructibility, lifecycle, placement, layout, and coverage analysis.

Does not establish backend or target behavior.

## 19.3 Target contract · `rem:guide13:target-assurance`

Establishes reviewed static target facts and evidence requirements.

Does not execute a transfer.

## 19.4 Tapscript · `rem:guide13:tapscript-assurance`

Establishes typed deterministic patterns and concrete relation placement.

Does not establish linking, complete transactions, or target acceptance.

## 19.5 Linker · `rem:guide13:linker-assurance`

Establishes deterministic constructor/program assembly and carrier reachability.

Does not establish owner signing or CT construction.

## 19.6 Transaction · `rem:guide13:transaction-assurance`

Establishes candidate-ABI-consistent construction and test signing/materialization.

Does not establish production key custody, production blinding, or target acceptance.

## 19.7 Safety evidence · `rem:guide13:safety-assurance`

Establishes finite candidate-specific rejecting and accepting evidence.

Does not prove universal compiler or target correctness.

## 19.8 Minimality evidence · `rem:guide13:minimality-assurance`

Establishes that a tested lower-disclosure plan preserves the same tested semantic transfer.

Does not establish complete private live-receipt lifecycle or universal privacy.

## 19.9 Phase-5 result · `rem:guide13:phase-assurance`

Establishes one candidate live-transfer pipeline.

It does not establish:

- burn;
- redemption;
- normalization as a protocol operation;
- final live-receipt constructor;
- final calibration;
- production wallet support;
- production signer support;
- production confidential transaction support;
- deployment release.

---

# 20. Suggested implementation waves · `sec:guide13:waves`

Each wave ends formatted, focused-test green, documented, and committed before the next begins.

## Wave 0 — Revalidate the Phase-4 handoff · `task:guide13:wave0`

**Deliverables**

- Phase-4 gate confirmed on the working tree;
- inherited compiler, linker, ABI, executor, report, and resource interfaces reviewed;
- preflight questions answered;
- Guide-11 representation policy imported without reopening;
- no identity drift.

**Suggested commit**

```text
plans: charter the live-transfer end-to-end tranche
```

## Wave 1 — Live-transfer compiler operation plan · `task:guide13:wave1`

**Deliverables**

- exact live-transfer target-operation plan;
- explicit and private plan alternatives;
- owner, class, value, sponsor, lifecycle, layout, and coverage projections;
- no graph handles;
- corruption-resistant validator;
- public API tests.

**Suggested commit**

```text
compiler: expose the validated live-transfer target plan
```

## Wave 2 — Signature and CT target closure · `task:guide13:wave2`

**Deliverables**

- selected owner sighash profile;
- target-native valid signature case;
- output/input mutation cases;
- explicit closed-asset checks;
- confidential conservation evidence role;
- mixed-representation disposition;
- development confidential materializer review.

**Suggested commit**

```text
target-elements: close live-transfer signature and CT requirements
```

## Wave 3 — Live-receipt constructor · `task:guide13:wave3`

**Deliverables**

- canonical owner metadata;
- structural live class;
- explicit/private representation role;
- static transfer leaf set;
- inherited unspendable internal key;
- no key-path escape;
- candidate lifecycle status;
- constructor mutation vectors.

**Suggested commit**

```text
tapscript: implement the candidate live-receipt constructor
```

## Wave 4 — Local authorization and recognition patterns · `task:guide13:wave4`

**Deliverables**

- predecessor recognition;
- owner-key encoding enforcement;
- per-input owner authorization;
- coordinator/member role checks;
- final-stack schedules;
- unknown-key negative vectors.

**Suggested commit**

```text
tapscript: implement live-transfer input authorization
```

## Wave 5 — Explicit transfer plan · `task:guide13:wave5`

**Deliverables**

- complete family ranges;
- output constructor closure;
- exact explicit aggregate conservation;
- split and merge;
- sponsor isolation;
- root/event absence;
- relocatable explicit plan;
- complete abstract schedules.

**Suggested commit**

```text
tapscript: emit the explicit live-transfer plan
```

## Wave 6 — Private-committed transfer plan · `task:guide13:wave6`

**Deliverables**

- complete explicit-`U` closure;
- private value preservation through target CT evidence;
- no amount reads;
- deterministic public test material;
- positive private one-to-one, split, and merge cases where claimed;
- private plan relocatable bundle;
- explicit production non-claims.

**Suggested commit**

```text
tapscript: emit the private live-transfer plan
```

## Wave 7 — Linker extension · `task:guide13:wave7`

**Deliverables**

- owner-parameterized constructors;
- representation-specific symbols and leaves;
- selected target programs;
- deterministic live-receipt taptree;
- carrier closure for both plans;
- candidate linked live-transfer bundle.

**Suggested commit**

```text
linker: link the live-transfer candidate bundle
```

## Wave 8 — Candidate live-transfer ABI · `task:guide13:wave8`

**Deliverables**

- input/output family order;
- request-order destination policy;
- coordinator derivation;
- sponsor suffix;
- output finalization;
- owner signing requests;
- explicit construction;
- private test construction;
- exact target transaction bytes;
- candidate-only status.

**Suggested commit**

```text
transaction: derive the live-transfer candidate ABI
```

## Wave 9 — Canonical safety evidence · `task:guide13:wave9`

**Deliverables**

- canonical live-transfer safety plan;
- complete explicit positive and negative cases;
- private positive and negative cases where claimed;
- every-owner authorization coverage;
- class and output closure;
- accepted semantic projections;
- validated safety report.

**Suggested commit**

```text
vectors: complete live-transfer safety evidence
```

## Wave 10 — Disclosure-minimality evidence · `task:guide13:wave10`

**Deliverables**

- paired explicit/private fixtures;
- exact semantic equivalence;
- disclosure comparison;
- target acceptance for both sides;
- constructibility assumptions;
- lifecycle-equivalence limitation;
- validated minimality report.

**Suggested commit**

```text
vectors: compare live-transfer representations
```

## Wave 11 — Resource study · `task:guide13:wave11`

**Deliverables**

- finite input/output/sponsor candidate enumeration;
- explicit/private measurements;
- multi-owner signature measurements;
- prediction/observation equality;
- useful Phase-5 candidate;
- explicit non-calibration result.

**Suggested commit**

```text
vectors: measure live-transfer candidate bounds
```

## Wave 12 — Phase-5 gate and handoff · `task:guide13:wave12`

**Deliverables**

- package READMEs;
- package contracts;
- Phase-5 card;
- backlog gate record;
- identity, schema, security, and dependency impact;
- complete repository gate;
- clean final tree.

**Suggested commit**

```text
plans: record the live-transfer candidate pipeline
```

---

# 21. Focused verification · `sec:guide13:verification`

## 21.1 Working cadence · `rule:guide13:rust-cadence`

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Focused runs supplement but do not replace complete workspace execution.

## 21.2 Realization · `tab:guide13:realization-tests`

```sh
cargo test --locked -p tripod-realization
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-realization --no-deps
```

Focused areas:

```text
live relation census
owner authorization
class closure
explicit/private alternatives
sponsor opacity
lifecycle exits
architecture weld
```

## 21.3 Compiler · `tab:guide13:compiler-tests`

```sh
cargo test --locked -p tripod-compiler
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-compiler --no-deps
```

Focused areas:

```text
live target-operation plan
representation alternatives
owner/source requirements
constructibility
lifecycle
carrier census
layout census
coverage census
determinism
no graph handles
no digest
```

## 21.4 Target · `tab:guide13:target-tests`

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

Focused areas:

```text
owner-key encoding
signature behavior
unknown-key behavior
selected sighash profile
input/output commitment
CT conservation
explicit U
resource and evidence contracts
reviewed trust state
```

## 21.5 Tapscript · `tab:guide13:tapscript-tests`

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

Focused areas:

```text
live constructor
owner metadata
local recognition
per-input signature checks
coordinator/member schedules
explicit aggregate arithmetic
private amount non-inspection
output closure
sponsor isolation
root/event absence
final stack
typed serialization
resource formulas
```

## 21.6 Linker · `tab:guide13:linker-tests`

```sh
cargo test --locked -p tripod-linker
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-linker --no-deps
```

Focused areas:

```text
owner-parameterized constructor
representation-specific symbols
relocation
taptree determinism
carrier closure
candidate lifecycle status
candidate/final separation
```

## 21.7 Transaction · `tab:guide13:transaction-tests`

```sh
cargo test --locked -p tripod-transaction
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-transaction --no-deps
```

Focused areas:

```text
live-transfer ABI
canonical input ordering
request output order
output finalization
multi-owner signing
sighash profile
post-signing mutations
explicit construction
private test construction
sponsor capability
secret/public boundary
```

## 21.8 Vectors · `tab:guide13:vectors-tests`

```sh
cargo test --locked -p tripod-vectors
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-vectors --no-deps
```

Focused areas:

```text
canonical safety plan
canonical minimality plan
owner mutations
class mutations
asset mutations
value mutations
sponsor mutations
accepted projections
relation coverage
report role separation
resource prediction
report determinism
```

## 21.9 Target executor · `tab:guide13:executor-tests`

Run the inherited executor package’s complete suite.

Focused areas:

```text
valid multi-owner signatures
missing owner
output mutation after signing
private CT construction
CT imbalance
proof corruption
strict protocol
exact transcript subjects
environment binding
provenance
timeout cleanup
no expected answer in request
```

## 21.10 Model · `tab:guide13:model-tests`

```sh
cargo test --locked -p tripod-model transfer
cargo test --locked -p tripod-model realization_conformance
```

No backend change may alter model acceptance.

## 21.11 Documentation · `tab:guide13:documentation-tests`

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new file enters its nearest Meson census in the same commit.

---

# 22. Full batch gate · `gate:guide13:batch`

After every coherent implementation series:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run real target matrices separately:

```text
explicit live-transfer safety matrix
private live-transfer safety matrix
owner-authorization matrix
constructor and class-closure matrix
sponsor matrix
explicit/private minimality matrix
candidate resource matrix
```

Record:

```text
target contract revision
deployment binding
network and genesis
executor declaration
adapter version
framework revision
binary-reported revision
intended tip
upstream base
local topic census
candidate linked bundle
candidate ABI
representation plan
candidate bounds
case count
relation count
claim count
failures
infrastructure errors
report byte reproducibility
```

If dependencies changed:

```sh
cargo tree --locked -e features
cargo metadata --locked
cargo audit
```

A missing advisory tool is recorded as skipped.

Run document reproducibility when policy requires it:

```sh
scripts/check-document-reproducibility.sh
```

Final check:

```sh
git status --porcelain=v1 --untracked-files=all
```

The result must be empty.

---

# 23. Acceptance criteria · `sec:guide13:acceptance`

Accept the Phase-5 candidate only when:

- Phase 4 remains green;
- live-transfer semantic scope is exact;
- realization and compiler relation censuses agree;
- every target requirement is assessed;
- every selected proof is realization-approved;
- every active relation-case has a reachable carrier;
- every receipt input authenticates live class, exact `U`, owner, constructor, and selected representation;
- every receipt output authenticates live class, exact `U`, owner, constructor, and selected representation;
- every consumed owner authorizes the finalized protected transaction;
- unknown key encodings cannot satisfy authorization without verification;
- split and merge conserve exact semantic value;
- explicit transfer passes complete safety evidence;
- private transfer has at least one accepting positive case where claimed;
- private split and merge have accepting cases before those capabilities are claimed;
- private value remains uninspected by protocol predicates;
- every closed-asset-capable output is classified;
- no confidential asset carries `U`;
- sponsor membership is exact and amount-opaque;
- balanced theft rejects;
- no root, issuance, destruction, or specialized event exists;
- explicit and private paired projections agree where minimality is claimed;
- safety and minimality reports remain separate;
- lifecycle incompleteness is explicit;
- linked bundle and ABI are deterministic;
- resource prediction equals observation;
- candidate bounds remain non-final;
- no production secret interface is overclaimed;
- no speculative digest is minted;
- report bytes reproduce;
- full repository gates pass;
- final tree is clean.

---

# 24. Rejection criteria · `sec:guide13:rejection`

Reject the candidate if:

- one semantic relation disappears between packages;
- target details enter compiler core;
- a target capability is treated as a completed proof;
- a required carrier is missing;
- one consumed owner is omitted;
- different owners sign different output sets;
- output mutation after signing passes;
- an unknown key encoding satisfies owner authorization;
- a time-locked receipt enters or leaves;
- ASH or another undeclared `U` object enters or leaves;
- aggregate balance hides a wrong recipient or object family;
- confidential asset identity is accepted;
- a hidden `U` output escapes closure;
- private transfer requires exact amount inspection;
- private safety is inferred from rejection-only evidence;
- one-to-one private evidence is generalized to split or merge;
- explicit safety is presented as minimality;
- target CT conservation is presented as owner authorization;
- sponsor amount becomes protocol data;
- sponsor positivity substitutes for transfer correctness;
- construction failure counts as target rejection for a target relation;
- an ad hoc vector becomes gate-eligible;
- accepted target projection differs from model semantics;
- lifecycle incompleteness is hidden;
- centralized public-fixture blinding is described as a production multi-owner protocol;
- a mock satisfies the gate;
- resource predictions disagree;
- candidate and final types are conflated;
- a report digest is added without an admitted consumer;
- any full gate fails or leaves the tree dirty.

---

# 25. Identity, schema, security, and dependency impact · `sec:guide13:impact`

Expected identity impact:

```text
Attestation version:
    unchanged

realization version:
    unchanged

architecture schema:
    unchanged

architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

anchor-set hash:
    unchanged

compiler identity:
    none minted

live-receipt constructor identity:
    none minted

bundle identity:
    none minted for in-process candidate use

ABI identity:
    none minted for in-process candidate use

safety report identity:
    none minted

minimality report identity:
    none minted

deployment-profile identity:
    remains dormant
```

Expected schema work:

```text
live-transfer compiler operation projection
representation-specific tapscript plan
candidate live-receipt constructor
candidate linked live-transfer bundle
candidate live-transfer ABI
live-transfer safety evidence plan/report
live-transfer minimality evidence plan/report
```

Expected security impact:

```text
public disposable test signing material:
    admitted

public disposable test blinders/openings:
    admitted

production private keys:
    not admitted

production blinders/openings:
    not admitted

production multi-party signing/blinding:
    not established
```

Expected dependency impact:

```text
reuse Phase-4 linker, transaction, and vectors packages

possible extension of the selected Elements transaction dependency

no new third-party dependency without a concrete consumer and ADR-011 review
```

Every schema migration is explicit. No old report or protocol revision is silently widened.

---

# 26. Final Phase-5 result matrix · `tab:guide13:result`

The completion report fills the final column.

| Boundary | Required result | Actual result |
|---|---|---|
| realization | live-transfer semantics complete | |
| compiler | validated live-transfer target plan | |
| target assessment | every capability and evidence role classified | |
| constructor | owner-bound live candidate constructor | |
| explicit tapscript | complete transfer relation | |
| private tapscript | complete closure and external CT handoff where claimed | |
| linker | deterministic candidate bundle | |
| transaction | deterministic candidate ABI and finalized signing flow | |
| owner authorization | every owner signs protected output set | |
| explicit safety | complete and passing | |
| private safety | complete for claimed scope | |
| minimality | explicit/private equivalence where claimed | |
| sponsor isolation | exact and amount-opaque | |
| target execution | positive cases accepted, negative cases rejected | |
| semantic projection | every accepted case matches | |
| coverage | every relation-case complete | |
| resources | prediction equals observation | |
| lifecycle | burn and redemption outstanding | |
| secret boundary | public test material only | |
| identity | no speculative digest | |
| release | not claimed | |

---

# 27. Guide-13 exit checklist · `gate:guide13:exit`

## Entry and inheritance

- [ ] Phase-4 gate passes on the current tree;
- [ ] inherited compiler, linker, ABI, executor, report, and resource interfaces are reviewed;
- [ ] every preflight question has an explicit answer;
- [ ] Guide-11 representation policy is consumed without reopening;
- [ ] no identity drift is unexplained.

## Compiler

- [ ] live-transfer target-operation plan has no unchecked constructor;
- [ ] scope is exact;
- [ ] relation census is exact;
- [ ] representation plans are explicit;
- [ ] owner-source requirements are complete;
- [ ] constructibility is complete;
- [ ] lifecycle is explicit;
- [ ] carrier, layout, and coverage censuses are exact;
- [ ] no graph handles or target positions leak;
- [ ] no compiler digest is minted.

## Constructor and programs

- [ ] owner metadata is canonical;
- [ ] live class is structural;
- [ ] explicit `U` is mandatory;
- [ ] confidential `U` rejects;
- [ ] predecessor recognition is exact;
- [ ] successor constructor derives from typed owner metadata;
- [ ] every input owner predicate executes;
- [ ] unknown key success without verification is closed;
- [ ] explicit aggregate arithmetic is exact;
- [ ] private plan reads no exact amount;
- [ ] sponsor isolation is exact;
- [ ] root, issuance, destruction, and event absence hold;
- [ ] final stack and failure states validate;
- [ ] no key-path escape exists.

## Linking and ABI

- [ ] every symbol is typed;
- [ ] every mandatory reference resolves;
- [ ] representation-specific leaves remain reachable;
- [ ] taptree is deterministic;
- [ ] carrier closure holds for each plan;
- [ ] input ordering is canonical;
- [ ] destination order is explicit ABI presentation;
- [ ] private amount does not determine public output order;
- [ ] coordinator is unique;
- [ ] sponsor suffix is exact;
- [ ] fee and sponsor-change roles are distinct;
- [ ] output set is finalized before signing;
- [ ] candidate bundle and ABI remain non-final.

## Authorization and confidential construction

- [ ] every owner signs the same finalized protected transaction;
- [ ] every input satisfies its own owner predicate;
- [ ] selected sighash profile is observed;
- [ ] missing owner rejects;
- [ ] output mutation rejects;
- [ ] input extension rejects where prohibited;
- [ ] test signing material is public and disposable;
- [ ] test blinding material is public and disposable;
- [ ] no production secret interface is introduced;
- [ ] centralized fixture construction is not described as a production protocol.

## Safety evidence

- [ ] canonical safety plan has no unchecked constructor;
- [ ] explicit positive cases accept;
- [ ] every required explicit negative case rejects;
- [ ] private positive cases accept where claimed;
- [ ] every required private negative case rejects;
- [ ] split and merge each have positive evidence before being claimed;
- [ ] every-owner authorization coverage is complete;
- [ ] class closure coverage is complete;
- [ ] closed-asset closure coverage is complete;
- [ ] sponsor coverage is complete;
- [ ] accepted semantic projections match;
- [ ] construction and target failures remain separate;
- [ ] zero required infrastructure errors;
- [ ] no mock satisfies the gate.

## Minimality evidence

- [ ] safety and minimality reports are distinct;
- [ ] paired cases derive from one semantic fixture;
- [ ] both explicit and private transactions accept;
- [ ] semantic projections agree;
- [ ] private exact amounts remain unpublished;
- [ ] no confidential asset carries `U`;
- [ ] disclosure reasons are typed;
- [ ] shape leakage is recorded;
- [ ] lifecycle incompleteness is equal and explicit;
- [ ] no universal privacy claim is made.

## Resources

- [ ] one-to-one explicit measured;
- [ ] split explicit measured;
- [ ] merge explicit measured;
- [ ] private one-to-one measured;
- [ ] private split/merge measured where claimed;
- [ ] multi-owner signatures measured;
- [ ] sponsorless and sponsored measured;
- [ ] candidate maxima measured;
- [ ] predicted and observed resources agree;
- [ ] consensus and policy remain separate;
- [ ] candidate bounds remain non-final.

## Repository

- [ ] package READMEs are current;
- [ ] package contracts are current;
- [ ] Phase-5 card is current;
- [ ] backlog is current;
- [ ] every new file is in the Meson census;
- [ ] dependency review is recorded;
- [ ] security impact is recorded;
- [ ] identity and schema impact is recorded;
- [ ] `cargo fmt --all` passes;
- [ ] Clippy passes with warnings denied;
- [ ] workspace tests pass;
- [ ] `scripts/ci.sh` passes;
- [ ] Meson compile passes;
- [ ] Meson tests pass;
- [ ] advisory status is passed or explicitly skipped;
- [ ] document reproducibility is passed or explicitly deferred under policy;
- [ ] `git diff --check` passes;
- [ ] final repository status is empty.

---

# 28. Completion report template · `sec:guide13:report-template`

```text
Guide 13 result
===============

Starting state:
    source revision:
    working tree:
    Phase-4 result:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    realization version:
    target contract revision:
    executor protocol revision:
    live-transfer compiler projection:

Preflight:
    Phase-4 interface reuse:
    signature profile:
    confidential construction boundary:
    constructor metadata:
    confidential proof ownership:
    mixed representation:
    output ordering:
    evidence roles:
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
    graph handles exposed:
    identity minted:

Target assessment:
    reviewed target:
    deployment binding:
    owner-key encoding:
    signature semantics:
    sighash profile:
    CT conservation:
    value introspection:
    asset introspection:
    missing primitives:
    unsupported capabilities:
    structural obligations:
    external evidence:

Live constructor:
    owner metadata:
    class encoding:
    representation encoding:
    static leaf set:
    internal key:
    key-path result:
    transfer continuity:
    outstanding lifecycle:

Tapscript explicit plan:
    input recognition:
    output recognition:
    owner authorization:
    coordinator:
    cardinality:
    value conservation:
    class closure:
    closed-asset closure:
    sponsor isolation:
    absence relations:
    stack result:
    resource formula:

Tapscript private plan:
    input recognition:
    output recognition:
    owner authorization:
    coordinator:
    value inspection:
    CT evidence handoff:
    class closure:
    closed-asset closure:
    sponsor isolation:
    stack result:
    resource formula:
    supported transfer shapes:

Linker:
    symbols:
    relocations:
    explicit leaves:
    private leaves:
    taptree:
    carrier closure:
    candidate bundle:
    deterministic rebuild:

Transaction ABI:
    input layout:
    output layout:
    output-order policy:
    coordinator:
    sponsor suffix:
    sponsor change:
    fee role:
    representation plans:
    request fields:
    output finalization:
    signing requests:
    witness order:
    candidate bounds:
    candidate status:

Signing:
    owner count:
    signature count:
    repeated owners:
    selected sighash:
    missing-owner result:
    post-signing mutation result:
    input-extension result:
    production-key claim:
        none

Confidential construction:
    materializer:
    public fixture keys:
    public fixture blinders:
    one-to-one:
    split:
    merge:
    many-to-many:
    sponsor:
    CT proof construction:
    centralized fixture limitation:
    production blinding claim:
        none

Safety evidence:
    explicit cases:
    explicit failures:
    private cases:
    private failures:
    owner authorization:
    class closure:
    asset closure:
    value conservation:
    sponsor isolation:
    accepted projection:
    relation coverage:
    infrastructure errors:
    report bytes:

Minimality evidence:
    paired cases:
    explicit accepted:
    private accepted:
    semantic projection equality:
    amount disclosure:
    shape leakage:
    constructibility difference:
    lifecycle difference:
    report bytes:
    universal privacy claim:
        none

Resources:
    explicit one-to-one:
    explicit split:
    explicit merge:
    private one-to-one:
    private split:
    private merge:
    maximum owners:
    maximum signatures:
    sponsorless:
    sponsored:
    coordinator bytes:
    member bytes:
    constructor bytes:
    control bytes:
    proof bytes:
    witness bytes:
    peak stack:
    validation budget:
    transaction weight:
    candidate input bounds:
    candidate output bounds:
    candidate sponsor bounds:
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

Identity impact:
    Attestation:
    realization:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    anchor-set hash:
    compiler identity:
        none
    constructor identity:
        none
    bundle identity:
        none unless separately admitted
    ABI identity:
        none unless separately admitted
    report identities:
        none
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
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
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
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    real explicit safety matrix:
    real private safety matrix:
    real owner matrix:
    real minimality matrix:
    real resource matrix:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Phase-5 verdict:
    accepted candidate / explicit-only candidate / private plan deferred /
    rejected target path / blocked

Residuals:

Next phase:
```

---

# 29. Handoff after Guide 13 · `sec:guide13:handoff`

If Guide 13 succeeds, Phase 5 has one complete owner-authorized transfer candidate.

The next phase should implement STATE and maturity:

```text
Guide 14 — STATE and Maturity
```

The next guide consumes rather than reopens:

- validated compiler target-operation plans;
- representation-specific backend planning;
- owner-bound constructor metadata;
- per-input owner authorization;
- finalized-output signing flow;
- explicit and confidential value-proof separation;
- deterministic linking;
- candidate ABI framework;
- safety/minimality report separation;
- exact executor provenance and environment binding;
- candidate resource study.

It adds:

- STATE constructor continuity;
- root succession;
- operator authorization;
- maturity announcement;
- cadence and cycle-index distinctions;
- state-field commitments;
- the first mutable protocol constructor.

Guide 13 itself adds none of those root or STATE claims.

---

## Closing statement · `rem:guide13:closing`

> Live receipt transfer is the first phase where authorization, representation, and disclosure all become first-class implementation dimensions. The phase succeeds only when every consumed owner approves one finalized transaction; every input and output remains an exact live receipt carrying explicit closed `U`; split and merge conserve one semantic value relation; the private plan, where supported, preserves that relation without exact amount inspection; sponsor value remains erased; safety and minimality are evidenced separately; and the resulting constructor, bundle, ABI, reports, and bounds remain visibly candidate-only and lifecycle-incomplete.
