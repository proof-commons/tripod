# Draft: Guide 15 Concept — Owner-Authorized Burn, Public ASH, and Permissionless Clear

> **Status:** Concept draft; not an execution guide
> **Phase:** Phase 7 — Burn, ASH, and Clear
> **Entry:** (`gate:phase6:exit`), the accepted public-declassification policy, and the accepted compact-ASH candidate
> **Primary semantic operations:** `burn`, `compact-ash`, and `clear`
> **Affected packages:** `realization`, `compiler`, `target-elements`, `tapscript`, `linker`, `transaction`, `vectors`; `target-elements-conformance` only where inherited target-generic execution, constructor, signing, proof, or readback interfaces remain sufficient
> **Evidence support:** the target-executor, finalized-signing, STATE-constructor, root-history, public-recovery, safety-report, and resource-comparison boundaries established by Phases 4 through 6
> **May affect after acceptance:** package contracts, the Phase-7 card, backlog, compact-ASH candidate status, STATE-constructor generation policy, burn-event evidence, public-value representation policy, Meson graph, and dependency graph
> **Supersedes as concept direction:** none
> **Does not implement:** redemption, request admission, settlement, cycle, entitlement issuance, distribution construction, RESV succession, PACE succession, production wallets, production private-key custody, production opening custody, production deployment, final calibration, or release
> **Required result:** one complete candidate pipeline in which owners may convert live receipts into one fresh public ASH object under exact authorization and event anchoring, public ASH may compact permissionlessly without creating an attestation event, and public ASH may clear permissionlessly through one exact STATE succession that destroys the correct amount of `U`, preserves the `Y − 1` floor, emits the exact clear projection, and leaves every non-claim and lifecycle limitation explicit
> **Authority:** Attestation, Realization, typed architecture, implemented ADRs, accepted decisions, the accepted public-declassification result, the accepted compact-ASH result, and the completed Phase-6 result take precedence over this concept
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015

---

## Mission · `sec:guide15:mission`

Guide 15 joins three operations that share one public maintenance object while preserving the distinctions between them:

```text
owner-authorized live receipts
    ↓ burn
one fresh public ASH
    ↓ compact-ash, zero or more times
one public aggregate ASH
    ↓ clear against current STATE
STATE successor + optional residual ASH
```

The three operations have different semantic owners and different evidence:

```text
burn:
    owners authorize declassification and conversion into fresh ASH

compact-ash:
    permissionless public aggregation, no STATE and no attestation event

clear:
    permissionless public consumption of ASH through exact STATE succession
```

The shared `ASH` constructor must not erase those differences.

A valid burn does not prove clear.

A valid clear does not authenticate a burn event.

A compacted ASH is not a fresh burn anchor.

A target accepting any of the three proves only that it accepted those bytes. Separate projections establish the operation, object provenance, STATE transition, event relation, and accounting result.

Guide 15 therefore extends the candidate pipeline along two axes at once:

1. from owner-authorized live receipts to publicly maintainable ASH; and
2. from root-free operations to the first permissionless operation that mutates canonical STATE.

The successful Phase-7 result is the conjunction:

\[\text{burn safety}\land\text{ASH public constructibility}\land\text{compact safety}\land\text{clear safety}\land\text{STATE continuity}\land\text{event integrity}\land\text{accounting integrity}\]

---

## One-line thesis · `rem:guide15:thesis`

> Guide 15 succeeds when every fresh ASH is created from an exactly authorized live-receipt burn, every compaction preserves public value without manufacturing burn provenance, every clear consumes authenticated public ASH and advances canonical STATE by the exact bounded decrement, and no target balance rule, event record, sponsor payment, constructor transition, or final cursor is allowed to stand in for another relation.

---

# 1. Governing rulings · `sec:guide15:rulings`

## 1.1 Guide 15 consumes the earlier pipeline · `rule:guide15:consume-pipeline`

Guide 15 consumes rather than redesigns without a concrete need:

- typed architecture and realization ownership;
- validated compiler target-operation plans;
- target-independent relation, source, constructibility, lifecycle, placement, layout, and coverage analysis;
- typed target capability assessment;
- exact target transaction and witness encoding;
- typed tapscript pattern records;
- abstract stack and failure-state validation;
- candidate relocatable bundles;
- deterministic taptree linking;
- candidate transaction ABIs;
- finalized transaction signing;
- owner authorization over exact finalized bytes;
- target-generic sponsor authorization;
- strict executor framing and response-shape validation;
- exact target, deployment, request, response, and provenance binding;
- target acceptance plus independent semantic projection;
- STATE constructor continuity;
- root-history edge validation;
- public metadata recovery;
- separate safety and resource reports;
- candidate-versus-final type separation.

An inherited interface changes only when burn, ASH, or clear presents a fact it cannot express.

Every such change records:

1. the concrete missing fact;
2. the inherited type that cannot carry it;
3. the smallest sufficient generalization;
4. regressions against compact ASH, live transfer, and maturity announcement;
5. identity and schema impact;
6. evidence impact;
7. dependency impact;
8. focused positive and negative tests.

## 1.2 Typed Rust remains the semantic source · `rule:guide15:typed-source`

First-party semantics derive from validated typed Rust values.

No package reconstructs burn, ASH, clear, STATE, event, or accounting meaning by parsing:

- this concept;
- an execution guide;
- realization prose;
- Attestation LaTeX;
- model source text;
- source comments;
- generated architecture publications;
- target scripts;
- reports from previous runs.

Generated reports and documents are one-way publications.

The target backend may define exact byte encodings for ASH objects, STATE metadata, event records, and target transactions. Those encodings represent typed semantic values; they do not become competing semantic sources.

## 1.3 The three operations remain distinct · `rule:guide15:operation-separation`

Burn, compact ASH, and clear are not modes of one operation.

They differ in authority, root effects, event effects, and lifecycle:

| Operation | Authority | STATE | Fresh ASH | Residual ASH | Event |
|---|---|---:|---:|---:|---|
| burn | every consumed receipt owner | absent | exactly one | not applicable | burn event and canonical burn records |
| compact ASH | permissionless | absent | none | exactly one aggregate successor | none |
| clear | permissionless | exact succession | none | iff \(B-X>0\) | clear event |

The compiler, backend, linker, ABI, vector, and report layers retain distinct operation identities.

A shared helper or constructor does not merge their relations.

## 1.4 Closed protocol asset identity remains explicit · `rule:guide15:asset-rigidity`

Every receipt and ASH object carries exact explicit `U`.

No confidential or unclassified asset field may carry `U`.

No target confidential-value balance substitutes for:

- exact asset identity;
- exact object class;
- exact constructor;
- exact family membership;
- exact event provenance;
- exact destruction classification.

An output carrying explicit `U` outside every admitted family is a closed-asset escape and rejects.

## 1.5 ASH value is public and authentically available · `rule:guide15:public-ash`

The initial Phase-7 ASH value is public.

The reference candidate uses an explicit target value unless a public-committed representation with an authenticated public opening has independently passed the accepted public-declassification gate before implementation begins.

A public ASH constructor must be usable by an unrelated process without:

- an owner secret;
- a creator-local opening;
- a blinding factor;
- an operator secret;
- a private database;
- a hidden wallet descriptor;
- temporary creator state.

A merely committed amount with no authenticated public opening is not public ASH.

A report stating an amount is not an opening and cannot make an object publicly constructible.

## 1.6 Burn is the authorized public boundary · `rule:guide15:burn-authorization`

Burn is owner-authorized.

Every consumed live-receipt owner authorizes the exact finalized transaction that:

- consumes the owner’s receipt;
- selects the burned amount;
- fixes every live-receipt change output;
- fixes the fresh ASH amount and constructor;
- fixes every canonical burn record;
- fixes optional sponsor inputs and change where covered;
- fixes the fee role;
- fixes transaction version and lock time;
- fixes all target proofs covered by the selected signature profile.

Burn may publish value because the owners authorize the public result.

A public amount appearing in the transaction is not, by itself, evidence that every owner authorized its publication.

## 1.7 Private receipt handling follows the accepted declassification policy · `rule:guide15:private-source-policy`

The accepted initial public boundary is explicit and owner-authorized.

Guide 15 therefore begins with one mandatory route:

```text
private live receipt
    ↓ accepted owner-authorized normalization
explicit live receipt
    ↓ burn
fresh explicit ASH
```

Direct private-receipt burn is optional and is not inferred from target confidential conservation.

It may be admitted only if a focused preflight establishes all of:

- every private source owner authorizes the same finalized burn;
- the exact public fresh-ASH amount is bound to the committed inputs;
- any private receipt change remains correctly blinded;
- closed-asset and live-class closure remain exact;
- target CT conservation is retained as external evidence;
- the direct route has the same semantic burn projection as the normalized route;
- no opening enters a production-capable interface;
- positive direct-private burn evidence exists.

Without that separate admission, Guide 15 supports private receipts through normalization and does not claim direct private burn.

## 1.8 Compact ASH is inherited, not reinterpreted · `rule:guide15:compact-inheritance`

The accepted compact-ASH candidate is the starting point for Phase 7’s maintenance path.

Guide 15 revalidates:

- exact explicit `U`;
- public positive ASH amounts;
- nonempty bounded ASH input family;
- one exact aggregate successor;
- permissionless construction;
- no owner authorization;
- sponsor isolation;
- no roots;
- no issuance;
- no destruction;
- no burn or clear event;
- deterministic linking and ABI;
- exact resource projection.

Guide 15 may generalize compact ASH only where the ASH created by burn or retained by clear cannot be consumed by the inherited constructor.

If all ASH objects share one constructor, that equality is checked by exact typed and byte-level comparison. It is not assumed from the shared class name.

## 1.9 Clear is permissionless · `rule:guide15:clear-permissionless`

Clear consumes public ASH and canonical STATE without an ASH owner or protocol operator signature.

Every operand needed to construct the clear must be public and authentic:

- current STATE;
- current STATE metadata;
- current canonical root;
- public ASH amounts;
- ASH constructors;
- exact input family;
- target and deployment;
- linked clear program;
- candidate ABI;
- optional sponsor capability.

No party that burned value must remain online for later compaction or clear.

No operator secret is introduced merely because clear mutates STATE.

## 1.10 Clear uses exact authenticated ASH values · `rule:guide15:clear-values`

Let the clear consume ASH inputs with public semantic amounts \(b_0,\ldots,b_{n-1}\).

The aggregate available amount is:

\[B=\sum_{i=0}^{n-1}b_i\]

Each \(b_i\) is authenticated from the actual ASH object.

The transaction must not:

- accept an amount from caller prose;
- trust an unauthenticated report;
- substitute a value from another ASH object;
- count one ASH twice;
- omit one consumed ASH;
- aggregate a sponsor value into \(B\);
- infer \(B\) from output balance alone.

## 1.11 Clear amount is exact · `rule:guide15:clear-amount`

Let predecessor STATE carry:

- total active quantity \(Y\);
- live-class quantity \(Y_L\).

The clear amount is:

\[X=\min(B,Y_L,Y-1)\]

and a valid clear requires:

\[X>0\]

The intended successor assignments are revalidated against typed realization before implementation. The expected assignment is:

\[Y'=Y-X\]

\[Y_L'=Y_L-X\]

All other semantic STATE fields remain unchanged unless the authoritative realization states an additional clear-owned assignment.

The concept does not silently make the expected assignment authoritative. Wave 0 compares it with the typed source and records any difference before implementation.

The \(Y-1\) term preserves the nonzero floor:

\[Y'\ge1\]

No branch may reduce \(Y\) to zero.

## 1.12 Clear destroys exactly what it removes · `rule:guide15:clear-destruction`

Clear destroys exactly \(X\) units of explicit `U` under the architecture-owned `tag-recon` destruction class.

It creates residual ASH exactly when:

\[B-X>0\]

When residual exists:

\[\operatorname{value}(\mathrm{ASH}_{\mathrm{residual}})=B-X\]

When \(B-X=0\), no residual ASH output exists.

A zero-valued placeholder is not an absent residual.

The destruction, STATE decrement, and residual equation must agree:

\[B=X+(B-X)\]

A target value tally proving total transaction balance does not prove the destruction tag, the STATE decrement, or the residual class.

## 1.13 STATE continuity is exact · `rule:guide15:state-continuity`

Clear consumes one canonical STATE predecessor and creates one canonical STATE successor.

The successor:

- preserves the selected static constructor relation;
- carries the exact clear-derived semantic metadata;
- uses the canonical representation nonce;
- is output under the exact constructor;
- becomes the new canonical STATE root.

No RESV edge occurs.

The root-history report validates the clear edge in sequence and rejects a final cursor restored after an invalid intermediate edge.

## 1.14 Constructor generations do not migrate silently · `rule:guide15:constructor-generation`

Phase 6’s candidate STATE constructor contains only the operations implemented at that generation. It carries no placeholder clear leaf.

Adding clear therefore changes the static STATE subtree.

Guide 15 must not claim that a Phase-6 STATE object under the maturity-only subtree can be spent by a Phase-7 clear leaf it never committed to.

The initial Phase-7 policy is:

```text
Phase-7 candidate generation:
    static subtree contains every STATE operation implemented
    for this candidate generation, including inherited maturity
    announcement and the new clear operation

Phase-6 target STATE:
    historical candidate evidence under the prior generation

Phase-7 native origin:
    a new synthetic STATE predecessor under the Phase-7 generation

cross-generation migration:
    not implemented and not claimed
```

Within the Phase-7 generation, predecessor and successor static subtrees must match exactly.

The maturity-announcement operation is revalidated under the expanded Phase-7 subtree so adding clear does not silently break the inherited STATE operation.

A future release requires one final multi-operation STATE constructor or an explicit migration mechanism. This concept supplies neither.

## 1.15 Event type and event value are independent anchors · `rule:guide15:event-anchors`

Burn attestation requires two independent anchors.

### Event-type anchor

The transaction is a canonical burn:

- live-receipt inputs;
- every-owner authorization;
- exactly one fresh ASH;
- exact allowed change;
- no STATE;
- no clear;
- no compaction;
- no unrelated event;
- canonical burn record structure.

### Value anchor

The accepted burn-record total does not exceed the fresh ASH created by that burn.

Let canonical burn records carry accepted values \(r_0,\ldots,r_{k-1}\), under the exact typed record domain.

At minimum:

\[\sum_{j=0}^{k-1}r_j\le A_{\mathrm{fresh}}\]

where \(A_{\mathrm{fresh}}\) is the fresh ASH amount.

Any stricter relation in Attestation, realization, or model remains authoritative and is derived in Wave 0.

Neither anchor substitutes for the other.

Correct record values on a compaction transaction do not make it a burn.

A genuine burn with over-claiming records remains a burn event whose records are invalid; the invalid records do not erase the underlying burn provenance.

## 1.16 Freshness is a provenance relation · `rule:guide15:fresh-ash`

A fresh ASH is an ASH created directly by burn.

An ASH successor created by compact or clear is not fresh, even when:

- it has the same amount;
- it uses the same constructor;
- it occupies the same output position;
- its transaction carries copied burn-record bytes;
- its ancestor was fresh.

Freshness is derived from the creating operation and its accepted projection, not from a field an ASH object carries indefinitely.

Compaction preserves value and ASH class. It does not preserve “fresh burn output” as a current-event fact.

## 1.17 Compact ASH is attestation-silent · `rule:guide15:compact-silent`

Compact ASH emits no burn event, no burn records, and no clear event.

Copied burn-record payloads, burn-like output ordering, or an amount equal to a prior fresh ASH do not create attestation provenance.

A compact transaction may preserve historical chain ancestry, but the current operation projects no new burn event.

## 1.18 Clear event is distinct · `rule:guide15:clear-event`

Clear emits the exact clear projection owned by realization.

It does not emit:

- a burn event;
- burn records;
- a distribution residue event;
- a redemption event;
- a maturity-announcement event.

The clear event binds the actual clear operation, the exact STATE succession, and the exact destruction result.

A transaction destroying `U` without the clear relation is not a clear event.

## 1.19 Sponsor opacity survives all three operations · `rule:guide15:sponsor-opacity`

Optional sponsorship remains outside burn, compact, and clear semantics.

No protocol relation, compiler plan, target program, canonical report, or future identity requires an individual sponsor amount to be:

- explicit for protocol use;
- decoded;
- opened;
- compared with zero;
- proved positive;
- publicly aggregated;
- included in \(A_{\mathrm{fresh}}\), \(B\), or \(X\);
- emitted in diagnostics;
- emitted in canonical reports.

Sponsor safety derives from exact region membership, disjointness, sponsor-owner authorization, multiplicity limits, and whole-transaction target conservation.

A sponsor amount cannot satisfy a short burn, compact, or clear relation.

## 1.20 Failure layers remain distinct · `rule:guide15:failure-layers`

Guide 15 distinguishes:

```text
semantic-request rejection

compiler-plan rejection

constructor derivation rejection

backend-emission rejection

linker rejection

ABI/construction rejection

owner-signing refusal

sponsor-signing refusal

executor infrastructure failure

consensus rejection before script

key-path rejection

script-path rejection

relay-policy rejection

accepted target transaction

report-layer semantic-projection rejection

report-layer burn-event rejection

report-layer clear-event rejection

report-layer constructor-continuity rejection

report-layer root-history rejection

report-layer accounting rejection
```

A negative row is answered only when the observed boundary equals the declared boundary and the intended carrier was reached.

Formally:

\[\operatorname{ObservedBoundary}(r)=\operatorname{DeclaredBoundary}(r)\]

An earlier refusal is a finding, not a pass.

## 1.21 Candidate and final states remain distinct · `rule:guide15:candidate-only`

Guide 15 may produce:

```text
ValidatedBurnOperationPlan
ValidatedClearOperationPlan
ValidatedPhase7OperationScope

CandidatePublicAshConstructor
CandidatePhase7StateConstructor
CandidateBurnTapscriptPlan
CandidateClearTapscriptPlan
CandidateRelocatablePhase7Bundle
CandidateLinkedPhase7Bundle

CandidateBurnAbi
CandidateCompactAshAbi
CandidateClearAbi

CandidateBurnSafetyReport
CandidateAshMaintenanceReport
CandidateClearSafetyReport
CandidateBurnEventReport
CandidateStateContinuityReport
CandidateRootHistoryReport
CandidateAccountingReport
CandidatePhase7ResourceReport
```

It must not produce or imply:

```text
FinalAshConstructor
FinalStateConstructor
FinalLinkedBundle
FinalTransactionAbi
ProductionBurnService
ProductionClearService
ProductionStateDeployment
ValidatedDeploymentRelease
```

## 1.22 No speculative identity · `rule:guide15:no-speculative-identity`

Typed in-process values use exact typed comparison.

Exact transaction, program, constructor, control-block, event, and report bytes use exact byte comparison where those bytes are the subject.

Guide 15 mints no:

```text
AshConstructorHash
BurnPlanHash
ClearPlanHash
Phase7BundleHash
Phase7AbiHash
BurnEventReportHash
ClearReportHash
AccountingReportHash
```

A digest enters only after ADR-021 admits a real consumer and distinct decision.

No candidate type reserves a future digest field.

## 1.23 Exactness and deterministic failure · `rule:guide15:exactness`

Guide 15 uses:

- checked bounded integers;
- exact finite sets and maps;
- exact target bytes;
- exact relation and carrier censuses;
- exact value aggregation;
- exact STATE assignments;
- exact event and accounting projections;
- exact or checked tree-cost arithmetic;
- explicit complexity bounds;
- target-native evidence where the claim concerns the target.

An exhausted search returns a typed failure and no partial result.

The implementation must not:

- wrap or saturate an aggregate;
- clamp a burn or clear amount;
- omit a source to fit a bound;
- keep a best partial candidate;
- fabricate a residual output;
- turn an unavailable observation into zero;
- treat an infrastructure failure as target rejection;
- alter expected semantics to match an unexpected acceptance.

---

# 2. Entry conditions · `sec:guide15:entry`

## 2.1 Phase-6 entry · `gate:guide15:phase6-entry`

Required:

- Phase 6 has exited on the current tree;
- maturity announcement remains green under its candidate constructor generation;
- canonical STATE metadata has one typed owner;
- predecessor and successor constructor reconstruction are implemented;
- operator signing is finalized-byte bound;
- root-history evidence is edge-based;
- public successor reconstruction is implemented;
- target, deployment, request, response, and provenance bindings remain exact;
- candidate and final states remain distinct;
- the tree is clean.

A Phase-6 concept, typed stopped result, or unreviewed local branch is not entry.

## 2.2 Semantic entry · `gate:guide15:semantic-entry`

Required:

- `burn`, `compact-ash`, and `clear` remain in architecture and realization scope under their exact existing identifiers;
- burn, compact, and clear relations validate against architecture;
- the complete STATE clear assignment is known;
- `tag-recon` is architecture-owned and exact;
- burn record and event semantics are typed;
- compact ASH remains event-silent;
- sponsor-value opacity remains enforced;
- no target-specific type enters realization;
- lifecycle obligations remain explicit.

## 2.3 Public-declassification entry · `gate:guide15:declassification-entry`

Required:

- the accepted public boundary is revalidated;
- the initial ASH representation is explicitly selected;
- any private-receipt burn path is classified as normalized-first or separately admitted direct-private;
- no unauthenticated opening is accepted;
- public ASH remains constructible by an unrelated process;
- no report attachment substitutes for chain-public value.

## 2.4 Compact-ASH entry · `gate:guide15:compact-entry`

Required:

- the Phase-4 compact-ASH operation still builds;
- its compiler plan, patterns, linker, ABI, vectors, and resource study remain green;
- its ASH constructor is compatible with burn-created and clear-residual ASH, or the incompatibility is explicitly dispositioned;
- the inherited operation emits no authorization primitive;
- no stale candidate limit is treated as final calibration.

## 2.5 STATE-constructor generation entry · `gate:guide15:state-generation-entry`

Required:

- the Phase-6 constructor generation is identified exactly;
- adding clear is recognized as a static-subtree change;
- the Phase-7 generation policy of §1.14 is accepted or replaced by an explicit migration decision;
- no test attempts to spend a Phase-6 STATE with an uncommitted Phase-7 leaf;
- synthetic Phase-7 origin non-claims are agreed in advance.

## 2.6 Target entry · `gate:guide15:target-entry`

Required:

- reviewed target definition validates;
- development binding is welded to the exact target;
- explicit `U` and explicit value introspection are reviewed;
- checked fixed-width addition, subtraction, and comparison are reviewed;
- target transaction conservation remains external evidence;
- selected owner and sponsor sighash profiles are reviewed;
- STATE constructor hashing and tweak requirements remain reviewed;
- execution domain and leaf version are observed;
- consensus and relay policy remain separate;
- no mock is gate-eligible.

## 2.7 Evidence entry · `gate:guide15:evidence-entry`

Required:

- canonical evidence plans have no unchecked constructors;
- coverage changes only from validated report wrappers;
- native refusals retain observed layers and exact subjects;
- wrong-boundary refusals are not counted as answers;
- report-layer answerability is not counted as observed evidence;
- accepted controls ground negative target observations;
- target rejection carries no accepted artifact;
- response-shape validation is exact;
- canonical reports retain their generic execution transcripts;
- safety, event, continuity, history, accounting, and resources remain separate.

## 2.8 Repository entry · `rule:guide15:repository-entry`

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

Record:

```text
starting revision
Phase-6 result revision
architecture schema and semantic hashes
realization version
target contract revision
executor protocol revision
compact-ASH candidate revision
STATE-constructor generation
selected public-value policy
clean-tree result
```

If the tree is not clean, stop and classify every change.

---

# 3. Preflight questions · `sec:guide15:preflight`

## 3.1 Exact semantic field ownership · `q:guide15:semantic-census`

What exact typed sources own:

- burn amount;
- live-receipt change;
- fresh ASH;
- burn records;
- burn event;
- compact aggregate;
- clear amount;
- clear STATE assignments;
- `tag-recon` destruction;
- residual ASH;
- clear event;
- transition certificate?

The execution guide must cite typed owners rather than restating prose as code.

## 3.2 ASH representation · `q:guide15:ash-representation`

Is the initial ASH representation:

```text
explicit value
```

or:

```text
public commitment plus authenticated public opening
```

The initial recommendation is explicit value, matching the accepted public-declassification policy and existing compact-ASH candidate.

A public-committed mode enters only if the three previously recorded target blockers have been resolved and target-native evidence exists.

A type with a commitment and an unauthenticated amount beside it is rejected.

## 3.3 Private burn route · `q:guide15:private-burn`

Does Phase 7 support:

```text
normalized-first private burn only
```

or also:

```text
direct private-receipt burn
```

The normalized-first route is mandatory.

Direct private burn requires a separate accepted result satisfying §1.7. If that result is absent, the execution guide must not include direct-private positive classes or claim private burn lifecycle completion.

## 3.4 ASH constructor continuity · `q:guide15:ash-constructor`

Do fresh, compacted, and residual ASH share one exact constructor?

If yes, prove:

```text
burn output constructor
=
compact input/output constructor
=
clear input/residual constructor
```

under one target, deployment, representation, and linked bundle.

If not, define explicit class transitions. Do not treat different constructor programs as one class because their semantic amounts are both public.

## 3.5 Burn record carrier · `q:guide15:burn-record-carrier`

Where do canonical burn records live on the target?

Candidates include:

- an unspendable data output;
- a typed witness publication;
- another already accepted public record carrier.

The selected carrier must provide:

- exact canonical bytes;
- stable ordering;
- association with the fresh ASH;
- target-visible or chain-public recoverability;
- no creator-local state;
- no ability for copied payload bytes to manufacture burn provenance.

The request may select record semantics where architecture permits. It may not select an arbitrary target carrier.

## 3.6 Burn-record cardinality and value rule · `q:guide15:burn-record-rule`

What exact typed rule governs:

- whether zero records are allowed;
- maximum record count;
- ordinal base;
- ordering;
- duplicate identities;
- skipped ordinals;
- individual value domain;
- aggregate accepted value;
- relation to fresh ASH?

The value-anchor rule must be derived from authority and not weakened to the minimum inequality in §1.15 if the semantic source is stronger.

## 3.7 Clear STATE assignment · `q:guide15:clear-assignment`

Does typed realization confirm:

\[Y'=Y-X\]

\[Y_L'=Y_L-X\]

with every other field unchanged?

If not, record the exact assignment and why before implementation.

No target pattern is written while this remains unresolved.

## 3.8 Clear arithmetic lowering · `q:guide15:clear-arithmetic`

Can the reviewed fixed-width arithmetic implement:

\[B=\sum b_i\]

\[X=\min(B,Y_L,Y-1)\]

without wide multiplication or division?

The expected answer is yes: addition, subtraction, and comparison only.

The execution guide nevertheless records:

- exact operand widths;
- signed versus unsigned interpretation;
- sum bounds;
- overflow behavior;
- branch schedule;
- final equality relations;
- proof that every comparison result is consumed;
- proof that the chosen minimum is one of the authenticated operands.

## 3.9 STATE generation transition · `q:guide15:state-generation`

How is the Phase-7 expanded static subtree introduced?

The initial answer is the generation policy in §1.14:

- no cross-generation spend;
- new synthetic Phase-7 STATE origin;
- maturity and clear both present in the Phase-7 static subtree;
- maturity revalidated under that subtree;
- no migration claim.

If the project instead requires continuity from the Phase-6 accepted STATE, an explicit migration operation and evidence plan are prerequisites. It cannot be improvised inside clear.

## 3.10 Event projection independence · `q:guide15:event-independence`

Can the evidence architecture keep separate:

- burn operation acceptance;
- burn event-type anchor;
- burn-record value anchor;
- compact event silence;
- clear event;
- clear accounting;
- STATE succession?

If not, use separate report types. Do not place optional fields in one report whose meaning changes according to which fields happen to be present.

## 3.11 Permissionless public construction · `q:guide15:permissionless`

Can a fresh unrelated process construct compact and clear using only:

- public chain data;
- public linked bundle and ABI;
- public ASH values;
- public STATE metadata;
- public constructor data;
- constructor-local sponsor capability where selected?

Any hidden creator dependency is a blocker.

## 3.12 Attestation query boundary · `q:guide15:attestation-query`

Which downstream attestation or audit result consumes burn records?

If no external consumer exists yet, Guide 15 validates the event and record projection internally and does not mint an interchange namespace or report digest.

If a real external consumer exists, ADR-022 activates for that record before implementation.

## 3.13 Preflight gate · `gate:guide15:preflight`

Implementation begins only when:

- every question above has an explicit disposition;
- the complete semantic relation and field censuses are known;
- the ASH representation is selected;
- the private burn route is selected;
- burn-record carrier and value rules are exact;
- clear STATE assignments are exact;
- clear arithmetic fits reviewed target capabilities;
- STATE generation policy is explicit;
- event roles remain separate;
- permissionless public construction has no hidden secret;
- no production secret interface is introduced;
- focused tests and repository checks pass;
- the tree is clean.

---

# 4. Semantic burn contract · `sec:guide15:burn`

## 4.1 Burn inputs · `def:guide15:burn-inputs`

Burn consumes a nonempty bounded family of live receipts.

For each input \(i\):

```text
object:
    live receipt

asset:
    explicit U

owner:
    authenticated from the receipt constructor

semantic amount:
    xᵢ > 0

authorization:
    one target-valid authorization at that input
```

Let:

\[I=\sum_{i=0}^{n-1}x_i\]

Every source appears exactly once.

No ASH, time-locked receipt, STATE, RESV, request, entitlement, vault, control, or bare `U` input may join the receipt family.

## 4.2 Burn outputs · `def:guide15:burn-outputs`

Burn creates exactly one fresh ASH with amount \(A\):

\[A>0\]

It may create live-receipt change outputs with amounts \(c_0,\ldots,c_{m-1}\):

\[c_j>0\]

Let:

\[C=\sum_{j=0}^{m-1}c_j\]

The exact burn relation is:

\[I=A+C\]

No second fresh ASH is allowed.

No zero ASH or zero live-receipt placeholder is allowed.

Every output carrying `U` is either:

- the unique fresh ASH; or
- an admitted live-receipt change output.

## 4.3 Burn change · `rule:guide15:burn-change`

Change is a live receipt, not ASH.

Its owner and constructor are selected through the typed burn request and candidate ABI under the same owner and constructor rules as live transfer.

Burn change does not become a second burn amount.

A transaction that shortens fresh ASH and increases live change by the same amount is value-conserving and still semantically wrong if the request fixed another burn amount.

Owner signatures commit to the exact split.

## 4.4 Every owner authorizes · `rule:guide15:burn-owners`

Let \(O\) be the set of distinct owners of the consumed receipts.

Every accepted burn satisfies:

\[O\subseteq\operatorname{Authorized}(T)\]

The target realization remains one authorization per concrete receipt input where the target message is input-specific.

Reports distinguish:

```text
distinct semantic owners
receipt inputs
owner signatures
```

A repeated owner controlling three inputs is one semantic owner and three concrete authorizations.

## 4.5 Burn records · `def:guide15:burn-records`

Burn may carry the canonical record family defined by the authoritative semantic source.

Each record has:

- exact schema;
- exact operation association;
- canonical ordinal;
- canonical subject or attestation key;
- typed value;
- exact target carrier;
- exact association with the unique fresh ASH;
- no mutable free-form metadata.

The record family is ordered canonically and duplicate-sensitive.

The target transaction carries no record the semantic request did not authorize.

## 4.6 Event projection · `rule:guide15:burn-event`

The burn event derives from the accepted transaction and semantic projection.

It binds:

- burn operation identity;
- consumed live-receipt family;
- unique fresh ASH;
- fresh ASH amount;
- canonical record family;
- optional live change;
- absence of STATE and other root effects;
- absence of clear and residue events.

A bare record payload without the event-type anchor produces no accepted record.

## 4.7 Burn provenance survives invalid records · `rule:guide15:burn-provenance`

If the transaction is a genuine burn but its record family over-claims, the evidence system reports separately:

```text
burn event:
    present

record acceptance:
    refused
```

It must not erase burn provenance merely because the value anchor failed.

Conversely, valid-looking records on a non-burn transaction are refused and create no burn event.

## 4.8 Burn sponsorship · `rule:guide15:burn-sponsor`

Optional sponsorship is disjoint from the receipt and ASH ranges.

The sponsor cannot:

- fund the fresh ASH;
- reduce the burned amount;
- satisfy a missing receipt source;
- become burn-record value;
- alter event provenance.

Whole-transaction conservation remains a target rule; burn semantics independently enforce \(I=A+C\).

---

# 5. Public ASH and compact maintenance · `sec:guide15:ash`

## 5.1 ASH object · `def:guide15:ash-object`

An ASH object carries:

```text
object:
    ASH

asset:
    explicit U

semantic amount:
    public, positive, and authenticated

constructor:
    exact candidate public-ASH constructor

authorization:
    none on the protocol path

required exits:
    compact
    clear
```

The object carries no owner.

A public amount is part of the object’s construction contract and is available to any later constructor.

## 5.2 Fresh and aggregate ASH · `rule:guide15:ash-provenance`

The object class may be common while creation provenance differs:

```text
fresh ASH:
    created directly by burn

aggregate ASH:
    created by compact

residual ASH:
    created by clear
```

If architecture distinguishes these as separate object variants, the exact typed distinction is preserved.

If architecture uses one ASH object class, provenance remains an event/history fact rather than disappearing.

## 5.3 Compact relation · `rule:guide15:compact-relation`

Compact consumes a nonempty bounded ASH family satisfying the inherited minimum.

Let the consumed public amounts be \(a_0,\ldots,a_{k-1}\).

The successor amount is:

\[A'=\sum_{i=0}^{k-1}a_i\]

The operation creates exactly one ASH successor carrying \(A'\).

Every arithmetic operation is checked.

No amount comes from sponsor inputs.

## 5.4 Compact permissionlessness · `rule:guide15:compact-permissionless`

Compact requires no owner, operator, or creator authorization.

Construction uses public ASH fields and public candidate artifacts only.

A hidden signature primitive, cadence condition, or private opening is a violation of the operation.

## 5.5 Compact silence · `rule:guide15:compact-projection`

Compact projects:

```text
ASH sources:
    consumed

ASH successor:
    created

value:
    exactly preserved

burn event:
    absent

burn records:
    absent

clear event:
    absent

STATE:
    absent

destruction:
    absent

issuance:
    absent
```

Copied record bytes cannot alter this projection.

## 5.6 Compact output closure · `rule:guide15:compact-output-closure`

Every output carrying `U` is the exact compact successor.

Sponsor change and fee carry only their admitted open asset.

No live receipt, fresh-burn marker, residual event object, or bare `U` output is permitted.

---

# 6. Semantic clear contract · `sec:guide15:clear`

## 6.1 Clear inputs · `def:guide15:clear-inputs`

Clear consumes:

```text
exactly one current canonical STATE predecessor
+
a nonempty bounded ASH family
+
optional isolated sponsor inputs
```

No owner authorization is required for the STATE or ASH inputs.

The STATE input is the canonical coordinator.

Every ASH input’s amount is public and authenticated.

## 6.2 Aggregate ASH · `rule:guide15:clear-aggregate`

For ASH inputs \(b_0,\ldots,b_{n-1}\):

\[B=\sum_{i=0}^{n-1}b_i\]

The sum is exact and independently recomputed from the accepted transaction’s actual inputs.

A duplicate input, omitted input, wrong constructor, wrong asset, or private/unavailable amount rejects.

## 6.3 Clear amount · `rule:guide15:clear-formula`

For predecessor STATE values \(Y\) and \(Y_L\):

\[X=\min(B,Y_L,Y-1)\]

Required:

\[X>0\]

The formula is evaluated from authenticated target values.

A witness-supplied \(X\) is either absent or checked against the recomputation before use.

## 6.4 STATE successor · `rule:guide15:clear-state`

Subject to Wave-0 semantic confirmation, successor STATE carries:

\[Y'=Y-X\]

\[Y_L'=Y_L-X\]

and for every other semantic field \(f\):

\[f(S')=f(S)\]

The successor remains operational under the authoritative maturity and sealing rules.

Clear does not announce maturity, complete maturity, advance cycle, alter reserve accounting, or change operator policy unless typed realization explicitly assigns such an effect.

## 6.5 Nonzero floor · `rule:guide15:clear-floor`

Because \(X\le Y-1\):

\[Y'\ge1\]

A candidate that clears to zero is invalid even if target value conservation holds.

The floor is semantic and independently projected after acceptance.

## 6.6 Destruction · `rule:guide15:clear-destruction-exact`

Clear records exact destruction:

```text
asset:
    U

amount:
    X

tag:
    tag-recon
```

No other destruction is present.

The destruction amount must equal the STATE decrement and the non-residual portion of ASH.

A wrong tag with the right amount rejects.

A right tag with the wrong amount rejects.

## 6.7 Residual ASH · `rule:guide15:clear-residual`

Residual ASH is conditional:

```text
if B - X > 0:
    exactly one residual ASH with value B - X

if B - X = 0:
    no residual ASH
```

The residual uses the exact public ASH constructor.

It is immediately usable by an unrelated future compact or clear constructor.

## 6.8 No RESV participation · `rule:guide15:no-resv`

Clear consumes and creates no RESV.

A reserve input, reserve output, reserve succession edge, or reserve termination edge rejects.

Whole-transaction sponsor funds are not RESV and must not be classified as one.

## 6.9 Clear event · `rule:guide15:clear-projection`

The clear event binds:

- operation identity;
- STATE predecessor;
- STATE successor;
- ASH source family;
- aggregate \(B\);
- clear amount \(X\);
- exact `tag-recon` destruction;
- residual presence and amount;
- absence of RESV;
- absence of burn records and burn event;
- exact transition certificate.

## 6.10 Clear permissionlessness · `rule:guide15:clear-construction`

An unrelated process constructs clear from public chain data.

No burner, ASH creator, receipt owner, or operator participates.

A sponsor may fund the target fee through the inherited isolated capability and contributes no semantic operand.

---

# 7. Phase-7 STATE constructor generation · `sec:guide15:state-constructor`

## 7.1 Expanded static subtree · `candidate:guide15:phase7-state-constructor`

The Phase-7 candidate STATE constructor contains the complete implemented STATE operation set for this generation:

```text
metadata commitment leaf
+
static subtree:
    maturity-announcement leaves
    clear leaves
    exact shared support leaves
```

No later-operation placeholder is included.

The metadata schema, internal-key policy, branch-side policy, nonce policy, and target construction rule are inherited from Phase 6 unless a concrete incompatibility is found.

## 7.2 Maturity regression under the expanded tree · `rule:guide15:maturity-regression`

Adding clear changes the static subtree root and every STATE constructor program in the candidate generation.

The inherited maturity operation is therefore:

- re-emitted;
- re-linked;
- assigned new Phase-7 constructor bytes;
- re-run through abstract validation;
- re-run through focused constructor continuity;
- re-run against a target where the Phase-7 gate relies on it.

This is not identity drift in a supposedly unchanged bundle. It is a new candidate generation.

The Phase-6 run remains historical evidence about the Phase-6 generation.

## 7.3 No generation crossing · `rule:guide15:no-generation-crossing`

A Phase-6 predecessor and Phase-7 successor cannot form an accepted continuity edge without an explicit migration operation.

Guide 15 does not implement one.

All Phase-7 STATE native vectors begin from a synthetic predecessor already committed to the Phase-7 static subtree.

## 7.4 Clear constructor continuity · `rule:guide15:clear-continuity`

Within the Phase-7 generation, clear proves:

```text
predecessor metadata:
    exact current STATE

successor metadata:
    exact clear assignment

static subtree:
    identical

metadata schema:
    identical

internal key:
    identical

leaf version:
    identical

constructor policy:
    identical
```

Only the semantic fields assigned by clear and the representation nonce needed to canonically encode the successor may differ.

## 7.5 Public recovery · `rule:guide15:clear-public-recovery`

An unrelated process recovers the clear successor using the public carrier selected in Phase 6:

1. locate the accepted clear transaction;
2. identify output 0;
3. recover predecessor metadata and clear inputs;
4. recompute \(B\);
5. recompute \(X\);
6. derive successor semantic STATE;
7. recover or recompute successor representation nonce;
8. reconstruct the Phase-7 successor constructor;
9. compare it with output 0.

No creator-local state is admitted.

---

# 8. Compiler plans · `sec:guide15:compiler`

## 8.1 Operation plans · `rule:guide15:operation-plans`

Guide 15 derives or confirms three validated operation plans:

```text
ValidatedBurnOperationPlan
ValidatedCompactAshOperationPlan
ValidatedClearOperationPlan
```

and one Phase-7 scope projection comparing their shared objects and relations.

No unchecked constructor exists for a validated plan.

## 8.2 Burn relation families · `tab:guide15:burn-relations`

The exact realization-derived census includes, where applicable:

- nonempty bounded live-receipt inputs;
- live-class input recognition;
- exact explicit `U`;
- every-owner authorization;
- unique fresh ASH;
- optional live-receipt change;
- exact \(I=A+C\);
- canonical source and destination partition;
- canonical burn record family;
- event-type anchor;
- event-value anchor;
- sponsor isolation;
- root absence;
- issuance absence;
- destruction absence;
- burn event;
- transition certificate;
- lifecycle to compact and clear.

## 8.3 Compact relation families · `tab:guide15:compact-relations`

The inherited census includes:

- nonempty bounded ASH inputs;
- public amount availability;
- explicit `U`;
- exact ASH constructor;
- permissionless path;
- exact aggregate;
- one successor;
- canonical partition;
- sponsor isolation;
- root absence;
- issuance and destruction absence;
- event silence;
- compact and clear lifecycle.

## 8.4 Clear relation families · `tab:guide15:clear-relations`

The exact census includes:

- one current STATE predecessor;
- nonempty bounded ASH inputs;
- exact public ASH values;
- exact \(B\);
- exact \(X=\min(B,Y_L,Y-1)\);
- \(X>0\);
- exact STATE assignments;
- one STATE successor;
- static-constructor continuity;
- exact `tag-recon` destruction;
- residual ASH iff positive;
- exact residual value;
- no RESV;
- root closure;
- sponsor isolation;
- issuance absence;
- burn-event absence;
- exact clear event;
- transition certificate;
- permissionless construction;
- public successor recovery;
- lifecycle obligations.

## 8.5 Cross-operation object closure · `rule:guide15:object-closure`

The compiler verifies:

```text
burn creates an ASH constructor compact and clear recognize

compact creates an ASH constructor compact and clear recognize

clear residual creates an ASH constructor compact and clear recognize
```

This closure is checked across operation plans rather than inferred from matching object identifiers.

## 8.6 Carrier placement · `rule:guide15:carrier-placement`

Expected carriers include:

```text
burn receipt leaves:
    local receipt recognition
    owner authorization

burn coordinator:
    exact counts
    input/output closure
    aggregate burn relation
    burn record closure
    sponsor isolation
    absence relations

compact coordinator:
    ASH recognition
    aggregate preservation
    output closure
    sponsor isolation
    event silence

clear STATE coordinator:
    STATE recognition
    ASH recognition and aggregate
    min formula
    STATE successor
    destruction
    residual
    sponsor isolation
    absence relations

reports:
    current-root freshness
    burn event/value anchors
    root-history sequence
    accounting projection
    public recovery

target:
    selected signatures
    transaction conservation
    taproot commitment validity
```

Every active relation has a reachable carrier.

A report-owned relation is not marked emitted.

## 8.7 Compiler validation · `rule:guide15:compiler-validation`

Required corruptions include:

- remove one operation;
- merge burn and compact identities;
- remove owner authorization;
- mark clear operator-authorized;
- remove public ASH availability;
- change `tag-recon`;
- remove \(Y-1\);
- remove residual condition;
- insert RESV;
- make compact emit burn event;
- make clear emit burn record;
- remove event-value anchor;
- remove event-type anchor;
- introduce sponsor amount;
- remove lifecycle exit;
- assign relation to inactive carrier;
- introduce target type;
- expose graph handle;
- duplicate relation;
- change evidence boundary.

Every corruption produces a typed refusal and no partial plan.

---

# 9. Target assessment and proof patterns · `sec:guide15:target-patterns`

## 9.1 Required capability families · `rule:guide15:target-assessment`

Assess at least:

- receipt, ASH, and STATE recognition;
- current-input index;
- input/output count;
- input/output asset, value, and program inspection;
- checked addition and subtraction;
- fixed-width comparison;
- byte-string equality;
- canonical event-record parsing or hashing;
- owner signature verification;
- selected owner sighash;
- STATE constructor hashing and tweak verification;
- sponsor isolation;
- transaction conservation;
- issuance absence;
- exact target resources.

Every requirement receives an explicit target disposition.

## 9.2 Burn patterns · `rule:guide15:burn-patterns`

The burn backend is expected to contain:

```text
local live-receipt recognition
per-input owner authorization
burn coordinator role
burn cardinality
fresh ASH output recognition
live-change closure
exact explicit burn aggregate
burn-record closure
sponsor isolation
root/issuance/destruction absence
final truth
```

The coordinator authenticates the transaction-global relation.

Each receipt input independently authenticates itself and its owner.

## 9.3 Compact patterns · `rule:guide15:compact-patterns`

The existing compact-ASH patterns are reused where exact.

No owner or operator signature primitive may enter them.

Any changed ASH constructor or event policy causes a full re-assessment rather than silent reuse of old pattern identities.

## 9.4 Clear patterns · `rule:guide15:clear-patterns`

The clear backend is expected to contain:

```text
STATE coordinator role
exact clear shape
predecessor STATE authentication
ASH input recognition
public ASH aggregate
clear minimum computation
successor STATE reconstruction
exact destruction
conditional residual ASH
sponsor isolation
RESV/root/issuance/event absence
clear event commitment
final truth
```

## 9.5 Minimum computation · `rule:guide15:minimum-pattern`

The target program computes or verifies \(X\) exactly.

One admissible schedule is:

1. compute checked \(B\);
2. read \(Y_L\);
3. read \(Y\) and compute checked \(Y-1\);
4. select or verify the minimum;
5. require \(X>0\);
6. use the same \(X\) for STATE decrement, destruction, and residual.

The implementation must prevent three independently supplied values from occupying those three roles.

One authenticated \(X\) is reused.

## 9.6 Burn record patterns · `rule:guide15:record-patterns`

The target-side record pattern authenticates the record carrier’s exact structure where target enforcement is feasible.

Report-layer validation independently reconstructs:

- record order;
- record values;
- association with fresh ASH;
- event type;
- aggregate bound.

Target parsing alone does not establish event provenance.

## 9.7 Final stack · `rule:guide15:final-stack`

Every complete program:

- ends on one canonical true item;
- consumes every arithmetic success flag;
- consumes every comparison result;
- leaves no alternate-stack residue;
- retains no non-aborting failure satisfying final truth;
- admits no unknown-key unverified success;
- remains within target script, operation, stack, element, and validation limits.

---

# 10. Linking and constructor closure · `sec:guide15:linking`

## 10.1 Linker reuse · `rule:guide15:linker-reuse`

Guide 15 extends the linker only for concrete needs:

- fresh-ASH creation references;
- shared ASH constructor closure;
- burn record carrier;
- Phase-7 STATE static subtree;
- clear operation leaves;
- exact `tag-recon` references;
- event and accounting carrier closure;
- operation-specific resource formulas.

No general dynamic linker is introduced.

## 10.2 Typed symbols · `rule:guide15:symbols`

Expected symbol roles include:

```text
U asset
live-receipt constructor family
public ASH constructor
STATE singleton asset and amount
STATE metadata schema
Phase-7 STATE static subtree
burn operation programs
compact operation programs
clear operation programs
tag-recon
burn-record schema and carrier
burn-event discriminator
clear-event discriminator
owner profile
STATE leaf version
unspendable internal key
sponsor reserve asset
sponsor-change program and version
fee program digest
candidate bounds
```

A symbol exists only while an emitted or linked consumer uses it.

## 10.3 Two-pass resolution · `rule:guide15:two-pass`

Pass one collects and validates every typed definition.

Pass two resolves every reference, constructs the frozen reference graph, computes SCCs, and applies cycle policy.

Every mandatory reference resolves exactly once.

Duplicate definitions reject.

No “last value wins” behavior is admitted.

## 10.4 Shared ASH constructor closure · `rule:guide15:ash-link-closure`

The linker compares every operation’s ASH constructor references.

Required equalities are exact over:

- target;
- representation;
- ownerless policy;
- leaf version;
- internal key;
- static leaf set;
- linked symbols;
- witness program;
- control recipes.

If provenance changes without constructor bytes changing, that fact remains in event/history evidence and is not forced into constructor identity.

## 10.5 Phase-7 STATE tree · `rule:guide15:state-tree`

The Phase-7 STATE tree includes maturity and clear leaves under one deterministic static subtree.

It rejects:

- missing inherited maturity leaf;
- missing clear leaf;
- extra future-operation leaf;
- duplicate leaf;
- conflicting weight;
- source-order-dependent tree;
- depth overflow;
- exact-cost overflow.

## 10.6 Candidate linked bundle · `def:guide15:linked-bundle`

The result retains:

- exact three-operation scope;
- exact target projection;
- public ASH constructor;
- Phase-7 STATE constructor;
- burn, compact, and clear programs;
- event carriers;
- relation placements;
- ABI handoffs;
- resource formulas;
- outstanding target evidence;
- lifecycle obligations;
- candidate-only status.

No bundle digest is minted.

---

# 11. Candidate transaction and witness ABIs · `sec:guide15:abi`

## 11.1 Burn input layout · `rule:guide15:burn-input-layout`

```text
inputs 0..n-1:
    live receipt family

inputs n..n+s-1:
    optional sponsor suffix
```

Input 0 is the burn coordinator.

Receipt inputs are canonically ordered.

Duplicates and sponsor overlap reject before sorting.

## 11.2 Burn output layout · `rule:guide15:burn-output-layout`

The initial candidate layout is:

```text
output 0:
    unique fresh ASH

next range:
    optional live-receipt change in typed request order

next range:
    canonical burn-record carriers

next optional role:
    sponsor change

final role where required:
    target fee
```

The exact order is candidate ABI, not semantic identity, unless the semantic source makes an ordinal observable.

Every `U` output belongs to the fresh ASH or live-change range.

## 11.3 Compact layout · `rule:guide15:compact-layout`

The inherited compact ABI remains:

```text
ASH input family
optional sponsor suffix

output 0:
    aggregate ASH successor

optional sponsor change
target fee where required
```

Any existing candidate-specific differences are retained exactly.

## 11.4 Clear input layout · `rule:guide15:clear-input-layout`

```text
input 0:
    current canonical STATE coordinator

inputs 1..n:
    ASH family

remaining inputs:
    optional sponsor suffix
```

No caller chooses another coordinator.

The exact ASH range is authenticated by counts and constructor checks.

## 11.5 Clear output layout · `rule:guide15:clear-output-layout`

```text
output 0:
    canonical STATE successor

next optional role:
    residual ASH iff B - X > 0

next role:
    clear-event carrier where the selected projection requires one

next optional role:
    sponsor change

final role where required:
    target fee
```

A zero residual contributes no output.

The clear-event carrier cannot satisfy the residual ASH role.

## 11.6 Burn request · `def:guide15:burn-request`

A typed burn request may select:

- receipt input outpoints;
- intended burn amount or exact change partition, according to realization ownership;
- live-change owners and values;
- canonical burn-record semantic entries;
- admitted representation route;
- optional sponsor capability;
- optional sponsor change under policy;
- public test-only construction inputs where required.

It may not select:

- input owners;
- input classes;
- input assets;
- fresh ASH class;
- fresh ASH constructor bytes;
- burn event truth;
- target output positions;
- coordinator;
- fee role;
- transaction version;
- witness order.

## 11.7 Clear request · `def:guide15:clear-request`

A clear request may select:

- ASH input outpoints;
- optional sponsor capability;
- optional sponsor change under policy.

It does not select:

- current STATE;
- \(B\);
- \(X\);
- STATE successor metadata;
- destruction amount;
- destruction tag;
- residual amount;
- residual presence;
- successor constructor;
- root cursor;
- event result;
- transaction positions.

Those derive from public authenticated inputs and the candidate ABI.

## 11.8 Finalization · `rule:guide15:finalization`

Before any burn owner signs:

- every receipt input is fixed;
- fresh ASH is fixed;
- live change is fixed;
- burn records are fixed;
- sponsor region is fixed;
- fee role is fixed;
- target proofs are fixed;
- version and lock time are fixed.

Clear and compact are permissionless and require no protocol owner-signing transition.

Sponsor signing, where selected, still occurs over exact finalized bytes.

## 11.9 Canonical transaction decoding · `rule:guide15:canonical-transaction`

External bytes enter through a strict parser.

Successful decoding satisfies:

\[\operatorname{decode}(b).\operatorname{encode}()=b\]

The parser rejects:

- nonminimal compact sizes;
- unsupported field encodings;
- confidential closed asset;
- unsupported proof forms;
- peg-ins;
- issuance-bearing bytes;
- inconsistent witness vectors;
- superfluous witness sections;
- trailing bytes.

## 11.10 Candidate ABI status · `rule:guide15:abi-status`

The three ABI values remain candidate-specific and carry no digest.

They record the exact constructor generation and representation policy they belong to.

---

# 12. Burn signing and authorization · `sec:guide15:signing`

## 12.1 One finalized burn · `rule:guide15:one-finalized-burn`

Every receipt owner signs the same protected burn transaction.

The signing request binds:

- exact finalized bytes;
- exact protected preimage;
- input index;
- spent output;
- receipt owner;
- executing leaf;
- control block;
- deployment;
- selected profile.

No owner receives a partially fixed output set.

## 12.2 Signing response · `rule:guide15:signing-response`

A successful response carries:

- authorization bytes;
- returned type byte where applicable;
- exact candidate binding;
- input position;
- no private key.

The transaction layer validates:

- expected owner;
- expected input;
- expected profile;
- exact byte binding;
- one response per required input;
- no unexpected response.

## 12.3 Burn signing faults · `rule:guide15:signing-faults`

Required:

- missing owner;
- wrong owner;
- one omitted owner;
- unrelated signer;
- duplicate response;
- one repeated owner’s concrete input omitted;
- empty signature;
- malformed signature;
- unknown key encoding;
- wrong profile;
- signature over another burn;
- fresh ASH changed after signing;
- change changed after signing;
- burn record changed after signing;
- sponsor input added after signing;
- fee changed after signing.

No partial owner set reaches submission.

---

# 13. Evidence architecture · `sec:guide15:evidence`

## 13.1 Canonical Phase-7 evidence plan · `rule:guide15:evidence-plan`

The canonical evidence plan has private fields and no unchecked constructor.

It derives from:

- validated burn, compact, and clear operation plans;
- exact linked Phase-7 bundle;
- exact candidate ABIs;
- canonical semantic fixtures;
- canonical safety mutations;
- canonical event mutations;
- exact target and deployment binding;
- expected executor provenance;
- root-history and public-recovery requirements.

Ad hoc cases remain experimental.

## 13.2 Burn safety report · `def:guide15:burn-safety-report`

The report answers:

```text
Did every valid burn preserve owner, class, asset, value,
change, record, sponsor, and absence relations, and did every
invalid burn fail at its declared boundary?
```

It carries no clear or compaction verdict by implication.

## 13.3 ASH maintenance report · `def:guide15:ash-report`

The report answers:

```text
Can public ASH be compacted and later consumed without owner
or creator secrets, with exact value preservation and no false event?
```

It includes fresh, compacted, and residual ASH constructor compatibility.

## 13.4 Clear safety report · `def:guide15:clear-safety-report`

The report answers:

```text
Did every valid clear compute the exact aggregate and minimum,
advance canonical STATE correctly, destroy exactly X under
tag-recon, create the exact residual, and exclude RESV and
every unrelated object or event?
```

## 13.5 Burn-event report · `def:guide15:burn-event-report`

The burn-event report validates the two independent anchors:

```text
event type
record value
```

It carries:

- accepted burn transaction;
- unique fresh ASH;
- canonical record family;
- event-type result;
- per-record result;
- aggregate record-value result;
- exact refusal for over-claim;
- explicit preservation of burn provenance where records fail.

## 13.6 Clear-event report · `def:guide15:clear-event-report`

The clear-event report binds:

- accepted clear transaction;
- exact STATE edge;
- exact ASH sources;
- \(B\);
- \(X\);
- exact destruction;
- exact residual;
- absence of burn provenance;
- clear event projection.

## 13.7 STATE continuity report · `def:guide15:continuity-report`

The continuity report uses the Phase-6 report pattern and additionally binds the Phase-7 static subtree containing clear.

It does not infer continuity across Phase-6 and Phase-7 generations.

## 13.8 Root-history report · `def:guide15:history-report`

The root-history report carries the ordered Phase-7 STATE edge sequence.

For a clear vector it validates exactly one edge from the synthetic Phase-7 predecessor to the clear successor.

It states that no pre-synthetic history is established.

## 13.9 Accounting report · `def:guide15:accounting-report`

The accounting report separately validates:

- burn-event record totals;
- fresh ASH amount;
- compact value preservation;
- clear aggregate \(B\);
- clear amount \(X\);
- STATE decrement;
- destruction amount and tag;
- residual amount;
- event type;
- noninterference of residue with later monetary or attestation computation where required by the model.

An event report does not silently satisfy accounting.

## 13.10 Public-construction report · `def:guide15:public-construction-report`

An unrelated process:

1. discovers public ASH;
2. reads its amount and constructor;
3. builds a compact or clear request;
4. reconstructs the Phase-7 STATE successor where applicable;
5. produces the same candidate bytes from equal explicit inputs;
6. uses no creator-private state.

## 13.11 Resource report · `def:guide15:resource-report`

The resource report remains separate.

It compares exact predictions with target observations over the same bytes.

Missing observations remain absent.

Timing remains noncanonical.

## 13.12 Validated report boundary · `rule:guide15:validated-reports`

Coverage consumes validated wrappers built from exact generic execution transcripts.

Public APIs do not accept caller-authored tuples such as:

```text
case
+
Accepted
+
ProjectionMatched
```

as canonical evidence.

Every native refusal records and checks the observed layer against the row’s declared boundary.

---

# 14. Semantic fixtures and projections · `sec:guide15:fixtures`

## 14.1 Burn fixture · `rule:guide15:burn-fixture`

A positive burn fixture begins from a model-valid world with:

- nonempty live receipts;
- exact owners;
- explicit or normalized-public values under the selected policy;
- no ASH or root input;
- exact intended fresh ASH;
- optional live change;
- canonical burn records;
- optional sponsor context.

The model transition runs through the invariant wrapper.

## 14.2 Compact fixture · `rule:guide15:compact-fixture`

A positive compact fixture may consume ASH from several origins:

- directly fresh from burn;
- previously compacted;
- residual from clear.

The fixture records provenance and verifies that provenance does not alter the aggregate relation.

## 14.3 Clear fixture · `rule:guide15:clear-fixture`

A positive clear fixture begins from:

- a synthetic but canonical Phase-7 STATE root;
- public ASH inputs under the exact constructor;
- nontrivial \(Y\), \(Y_L\), and \(B\);
- optional sponsor context.

Separate fixtures exercise each minimum branch:

```text
B is minimum
Y_L is minimum
Y - 1 is minimum
ties among minima
```

## 14.4 Accepted burn projection · `rule:guide15:accepted-burn`

For every accepted burn compare:

- exact consumed live-receipt family;
- exact owners;
- exact owner authorizations;
- exact input amounts;
- exact fresh ASH;
- exact live change;
- \(I=A+C\);
- explicit `U`;
- fresh provenance;
- burn records;
- event anchors;
- sponsor region;
- no roots;
- no issuance or destruction;
- transition certificate.

## 14.5 Accepted compact projection · `rule:guide15:accepted-compact`

Compare:

- exact ASH sources;
- exact aggregate successor;
- explicit `U`;
- exact amount preservation;
- permissionless path;
- no STATE;
- no destruction;
- no event or record;
- sponsor isolation.

## 14.6 Accepted clear projection · `rule:guide15:accepted-clear`

Compare:

- exact current STATE predecessor;
- exact ASH sources;
- exact \(B\);
- exact \(X\);
- exact successor STATE;
- exact changed and unchanged fields;
- exact STATE root edge;
- no RESV;
- exact destruction;
- exact residual condition and amount;
- exact clear event;
- no burn record or event;
- sponsor region;
- transition certificate.

## 14.7 Positive class witnesses · `rule:guide15:positive-witnesses`

Every positive class evaluates a typed predicate.

Examples:

```text
B-minimum clear:
    B ≤ Y_L and B ≤ Y - 1 and X = B

Y_L-minimum clear:
    Y_L ≤ B and Y_L ≤ Y - 1 and X = Y_L

floor-preserving clear:
    Y - 1 ≤ B and Y - 1 ≤ Y_L and X = Y - 1

full clear of ASH:
    B = X and no residual exists

partial clear:
    B > X and residual = B - X

fresh burn:
    creating operation is burn and exactly one ASH is created

event-silent compact:
    projection contains no burn or clear event
```

A class name alone cannot make a fixture canonical.

---

# 15. Required vector matrix · `sec:guide15:vectors`

## 15.1 Positive burn cases · `tab:guide15:burn-positive`

Required:

- one receipt, full burn;
- one receipt, partial burn with live change;
- several receipts, full burn;
- several receipts, partial burn;
- repeated owner;
- several distinct owners;
- minimum positive fresh ASH;
- representative burn records;
- maximum candidate burn-record count;
- sponsorless;
- sponsored;
- sponsor change present;
- sponsor change absent;
- normalized-first private source where private receipt lifecycle is claimed;
- direct private burn only if separately admitted.

## 15.2 Burn authorization faults · `tab:guide15:burn-auth-faults`

Required:

- missing owner signature;
- wrong owner;
- one omitted owner;
- duplicated signature substituted for another owner;
- unrelated signer;
- repeated owner with one concrete input signature omitted;
- empty signature;
- malformed signature;
- unknown key type;
- wrong profile;
- signature over another transaction;
- fresh ASH changed after signing;
- live change changed after signing;
- burn records changed after signing;
- sponsor region changed after signing.

## 15.3 Burn class and asset faults · `tab:guide15:burn-object-faults`

Required:

- time-locked receipt input;
- ASH input;
- STATE or root input;
- wrong receipt constructor;
- stale receipt constructor;
- wrong explicit asset;
- confidential asset commitment;
- bare `U` input;
- no fresh ASH;
- two fresh ASH outputs;
- fresh output under wrong constructor;
- fresh ASH with confidential or unavailable value;
- live change under ASH constructor;
- ASH output under live constructor;
- hidden `U` output;
- unclassified `U` output.

## 15.4 Burn value faults · `tab:guide15:burn-value-faults`

Required:

- fresh ASH one below the authorized amount;
- fresh ASH one above;
- live change one below;
- live change one above;
- value moved between fresh ASH and change after signing;
- zero fresh ASH;
- zero live-change placeholder;
- aggregate overflow;
- omitted receipt source;
- duplicated receipt source;
- output claimed twice;
- sponsor amount used to balance a short protocol relation;
- private source opened without the accepted declassification route.

## 15.5 Burn-record faults · `tab:guide15:burn-record-faults`

Required:

- record under-claim at every admitted boundary;
- record exact claim;
- record one above fresh ASH;
- aggregate over-claim split across several individually valid records;
- duplicate record;
- skipped ordinal;
- reordered records;
- wrong record schema;
- wrong event discriminator;
- record associated with another fresh ASH;
- record copied to compact transaction;
- record copied to clear transaction;
- bare record payload without burn event;
- burn event with no valid type anchor;
- genuine burn with invalid records, retaining burn provenance and refusing record acceptance.

## 15.6 Positive compact cases · `tab:guide15:compact-positive`

Required:

- minimum admitted ASH input count;
- three ASH inputs;
- candidate maximum inputs;
- equal amounts;
- unequal amounts;
- ASH directly from burn;
- ASH from prior compact;
- residual ASH from clear;
- mixed provenance inputs;
- sponsorless;
- sponsored;
- repeated construction with equal bytes.

## 15.7 Compact faults · `tab:guide15:compact-faults`

Required:

- zero inputs;
- one input where the inherited minimum is two;
- above candidate maximum;
- duplicate outpoint;
- wrong ASH asset;
- wrong constructor;
- private or unavailable amount;
- sum one below;
- sum one above;
- aggregate overflow;
- zero successor;
- two successors;
- live receipt substituted for ASH;
- fresh-burn event added;
- copied burn records;
- clear event added;
- STATE input or output;
- issuance;
- destruction;
- sponsor overlap;
- authorization unexpectedly required.

## 15.8 Positive clear cases · `tab:guide15:clear-positive`

Required:

- \(B\) uniquely minimum;
- \(Y_L\) uniquely minimum;
- \(Y-1\) uniquely minimum;
- \(B=Y_L<Y-1\);
- \(B=Y-1<Y_L\);
- \(Y_L=Y-1<B\);
- all three equal;
- full ASH consumption with no residual;
- partial ASH consumption with residual;
- one ASH input;
- several ASH inputs;
- candidate maximum ASH inputs;
- residual usable by later compact;
- residual usable by later clear;
- sponsorless;
- sponsored;
- sponsor change present;
- sponsor change absent;
- unrelated-process construction.

## 15.9 Clear formula faults · `tab:guide15:clear-formula-faults`

Required:

- \(X=B-1\) where \(B\) is minimum;
- \(X=B+1\);
- \(X=Y_L-1\) where \(Y_L\) is minimum;
- \(X=Y_L+1\);
- \(X=Y-2\) where \(Y-1\) is minimum;
- \(X=Y\);
- \(X=0\);
- caller-supplied \(X\) differs from recomputation;
- aggregate \(B\) one below actual inputs;
- aggregate \(B\) one above;
- omitted ASH input;
- duplicated ASH input;
- checked-add overflow;
- malformed amount width;
- comparison result left unchecked;
- different \(X\) used for STATE, destruction, and residual.

## 15.10 Clear STATE faults · `tab:guide15:clear-state-faults`

Required:

- stale STATE predecessor;
- wrong STATE asset;
- wrong STATE amount;
- wrong predecessor metadata;
- wrong current-root binding;
- \(Y\) decremented by wrong amount;
- \(Y_L\) decremented by wrong amount;
- \(Y_T\) changed;
- \(Q\) changed;
- cycle changed;
- maturity changed;
- another unaffected field changed;
- no STATE successor;
- two STATE successors;
- STATE termination;
- successor under wrong static subtree;
- Phase-6 predecessor paired with Phase-7 successor without migration;
- representation nonce noncanonical;
- final root cursor restored after invalid intermediate edge.

## 15.11 Clear destruction and residual faults · `tab:guide15:clear-output-faults`

Required:

- missing destruction;
- duplicate destruction;
- wrong destruction amount;
- wrong destruction asset;
- wrong destruction tag;
- residual omitted when \(B-X>0\);
- residual present when \(B-X=0\);
- residual one below;
- residual one above;
- residual under wrong constructor;
- residual as live receipt;
- zero-valued residual placeholder;
- second residual;
- hidden `U` output;
- destruction routed into sponsor flow.

## 15.12 Clear root and event faults · `tab:guide15:clear-root-event-faults`

Required:

- RESV input;
- RESV output;
- RESV succession;
- PACE edge;
- authority edge;
- receipt input or output;
- entitlement, vault, or control object;
- issuance;
- burn event;
- burn record;
- distribution residue event;
- wrong clear event;
- clear event naming another STATE edge;
- omitted transition certificate;
- certificate naming another predecessor;
- certificate naming another successor.

## 15.13 Sponsor faults · `tab:guide15:sponsor-faults`

Required for each operation where sponsorship is admitted:

- protocol/sponsor overlap;
- two sponsor envelopes;
- foreign sponsor asset;
- missing sponsor authorization;
- empty sponsor offer;
- sponsor change in protocol range;
- protocol object in sponsor suffix;
- fee/change substitution;
- unclassified sponsor member;
- report publishes sponsor amount;
- report publishes sponsor opening;
- balanced protocol corruption compensated by sponsor change;
- zero-valued ordinary sponsor member under exact roles;
- confidential sponsor values where target policy permits.

## 15.14 ABI and linker faults · `tab:guide15:abi-faults`

Required:

- wrong coordinator;
- duplicate coordinator;
- no coordinator;
- family overlap;
- family gap;
- wrong total input count;
- wrong total output count;
- wrong transaction version;
- wrong sequence;
- witness reorder;
- control block from another program;
- unresolved `U` symbol;
- unresolved ASH constructor;
- unresolved STATE static root;
- unresolved `tag-recon`;
- relocation omitted;
- relocation applied twice;
- duplicate tree leaf;
- conflicting leaf role or weight;
- source-order-dependent tree;
- stale bundle paired with new ABI;
- raw transaction bypassing safe construction;
- target bytes changed after ABI validation.

## 15.15 Protocol and report faults · `tab:guide15:protocol-faults`

Required:

- blank record;
- oversized record;
- unterminated record;
- malformed JSON;
- unknown field;
- wrong schema;
- wrong environment;
- wrong provenance;
- accepted response missing required artifact;
- rejected response carrying acceptance artifact;
- infrastructure response carrying target observation;
- report summary edited;
- failed row removed;
- duplicate passing row;
- wrong-boundary refusal counted as answered;
- event-answerable row counted without validated event evidence;
- canonical report includes wall time;
- diagnostic includes caller path, raw child text, or injected line.

---

# 16. Event and attestation integrity · `sec:guide15:events`

## 16.1 Event sequence · `rule:guide15:event-sequence`

The event report retains an ordered typed sequence.

For a transaction in this guide, the sequence is operation-specific:

```text
burn:
    one burn event, with canonical record family

compact:
    no event

clear:
    one clear event
```

An event list assembled from caller labels is not evidence.

## 16.2 Burn credit · `rule:guide15:burn-credit`

Attestation credit is accepted only for canonical records whose:

- transaction is an accepted burn;
- event-type anchor holds;
- record schema and ordinal hold;
- value anchor holds;
- fresh ASH association holds;
- chain checkpoint is current under the report’s policy.

Copied or stale records do not acquire current credit merely because their bytes remain public.

## 16.3 Over-claim noninterference · `rule:guide15:overclaim-noninterference`

An over-claiming record is invalid.

Its invalidity must not:

- erase the burn event;
- alter the fresh ASH amount;
- create clear provenance;
- alter STATE;
- alter unrelated valid record entries unless the authoritative record policy makes the whole family atomic.

The exact family-atomicity rule is derived during preflight.

## 16.4 Residual noninterference · `rule:guide15:residual-noninterference`

Residual ASH from clear is future protocol value and not burn credit, fee value, redemption value, or attestation residue.

A later compact or clear may consume it.

Its presence must not retroactively change the prior clear event or burn records.

## 16.5 Checkpoint and reorg binding · `rule:guide15:event-checkpoint`

Event evidence binds:

- network;
- genesis;
- block hash;
- block height;
- transaction identity;
- relevant outpoints;
- target contract;
- linked bundle;
- ABI;
- event schema.

A reorg that removes or changes the event transaction stales the observation.

---

# 17. Resource evidence · `sec:guide15:resources`

## 17.1 Candidate dimensions · `rule:guide15:candidate-dimensions`

Evaluate candidate values for:

```text
burn receipt-input maximum
burn live-change maximum
burn-record maximum
compact ASH-input maximum
clear ASH-input maximum
Phase-7 STATE tree depth
sponsor-input maximum
```

These remain candidate assignments.

## 17.2 Complete measurements · `rule:guide15:measurements`

Measure:

### Burn

- one-input full burn;
- one-input partial burn;
- multi-input multi-owner burn;
- maximum live change;
- maximum record family;
- sponsorless and sponsored;
- largest owner witness set.

### Compact

- minimum inputs;
- candidate maximum inputs;
- mixed-origin ASH;
- sponsorless and sponsored;
- deepest control path.

### Clear

- each minimum branch;
- full consumption;
- residual branch;
- candidate maximum ASH family;
- expanded Phase-7 STATE constructor;
- public-recovery witness;
- sponsorless and sponsored;
- maximum sponsor candidate.

## 17.3 Resource dimensions · `tab:guide15:resource-dimensions`

Record separately:

- burn coordinator bytes;
- burn member bytes;
- compact coordinator and member bytes;
- clear STATE program bytes;
- ASH constructor bytes;
- Phase-7 STATE constructor bytes;
- burn-record carrier bytes;
- metadata leaf bytes;
- taptree depth;
- control bytes;
- owner-signature bytes;
- sponsor-witness bytes;
- public metadata and event witness bytes;
- initial witness items;
- peak main stack;
- peak alternate stack;
- largest element;
- arithmetic operations;
- hash operations;
- tweak checks;
- validation budget;
- transaction weight;
- virtual size;
- consensus verdict;
- relay-policy verdict;
- construction and execution time as noncanonical diagnostics.

## 17.4 Prediction and observation · `rule:guide15:resource-comparison`

Backend, linker, and ABI predictions are compared with target observations over the same exact bytes.

A mismatch:

- records both values;
- returns a typed report failure;
- emits no later dependent operation;
- cannot coexist with overall success.

No absent observation is read as zero.

## 17.5 Phase-7 resource result · `rule:guide15:resource-result`

Phase 7 may conclude:

```text
the tested burn, public-ASH, compact, clear, and expanded STATE
candidate fit the reviewed target under the measured candidate policy
```

It must not claim final production bounds or final STATE-tree layout.

---

# 18. Relation-indexed coverage · `sec:guide15:coverage`

For every operation relation-case retain:

```text
operation identity
relation identity
execution case
representation
sponsor case
activation
selected proof
carrier
positive requirement
negative requirement
declared evidence boundary
observed evidence boundary
target verdict where applicable
semantic projection where applicable
event projection where applicable
STATE projection where applicable
root-history projection where applicable
accounting projection where applicable
mutation locator
collateral closure
validated observation provenance
```

## 18.1 Burn coverage · `rule:guide15:burn-coverage`

Burn requires separate coverage for:

- receipt recognition;
- every-owner authorization;
- exact `U`;
- fresh ASH uniqueness;
- live change;
- exact aggregate;
- event type;
- record value;
- sponsor isolation;
- root and event absence.

One accepted burn does not discharge every relation unless every relation’s row explicitly cites that same transaction as an instance of its own class.

## 18.2 Compact coverage · `rule:guide15:compact-coverage`

Compact coverage includes:

- public ASH availability;
- permissionless path;
- aggregate preservation;
- output closure;
- event silence;
- sponsor isolation;
- no roots or destruction.

Event silence requires an actual validated projection, not merely the absence of an event field from the request.

## 18.3 Clear coverage · `rule:guide15:clear-coverage`

Clear coverage includes separate rows for:

- ASH aggregate;
- each minimum branch;
- nonzero progress;
- STATE decrement;
- nonzero floor;
- constructor continuity;
- destruction;
- residual branch;
- no-residual branch;
- no RESV;
- root history;
- clear event;
- public recovery;
- sponsor isolation.

## 18.4 Boundary equality · `rule:guide15:coverage-boundaries`

A target refusal answers a row only when:

- an accepted control exists;
- the mutation is attributable;
- the intended carrier executed;
- observed and declared boundaries match.

A balance-layer refusal cannot discharge a script arithmetic relation whose script never ran.

A report-layer mismatch cannot be filed as target rejection.

## 18.5 First-party evidence · `rule:guide15:first-party-evidence`

Compiler-static, constructor, linker, ABI, and signing-flow rows may be discharged first-party only by:

- canonical malformed typed input;
- exact owning validator;
- accepted control;
- typed refusal naming the intended class;
- focused negative test;
- validated first-party evidence record.

A type making a fault unrepresentable may be a structural standing only where the row’s own gate asks whether the fault is representable. It is not automatically a refusal.

## 18.6 External evidence · `rule:guide15:external-evidence`

Target transaction conservation, selected sighash semantics, and other target-owned rules remain explicit external evidence.

Each report binds exact target, deployment, bundle, ABI, transaction, case, and provenance.

Mocks and abstract validators do not satisfy these roles.

---

# 19. Assurance boundaries · `sec:guide15:assurance`

## 19.1 Realization · `rem:guide15:realization-assurance`

Establishes target-independent burn, compact, clear, STATE, event, and accounting semantics.

Does not establish target bytes, constructor continuity, signatures, or node behavior.

## 19.2 Compiler · `rem:guide15:compiler-assurance`

Establishes complete relation, source, constructibility, lifecycle, placement, layout, and coverage analysis.

Does not establish emitted programs or target acceptance.

## 19.3 Target contract · `rem:guide15:target-assurance`

Establishes reviewed target capabilities and external evidence requirements.

Does not prove a complete burn or clear.

## 19.4 Tapscript · `rem:guide15:tapscript-assurance`

Establishes typed target patterns, abstract schedules, and concrete carrier placement.

Does not establish linking, transaction construction, event provenance, or root history.

## 19.5 Linker · `rem:guide15:linker-assurance`

Establishes exact symbols, references, trees, relocations, constructor closure, and carrier reachability.

Does not establish semantic operations or target verdicts.

## 19.6 Transaction · `rem:guide15:transaction-assurance`

Establishes candidate-ABI-consistent requests, outputs, finalization, signing handoffs, and canonical bytes.

Does not establish production custody or target acceptance.

## 19.7 Safety evidence · `rem:guide15:safety-assurance`

Establishes finite candidate-specific accepting and rejecting observations.

Does not prove universal compiler or target correctness.

## 19.8 Event evidence · `rem:guide15:event-assurance`

Establishes burn and clear event projections for the tested transactions.

Does not establish a complete production attestation indexer.

## 19.9 Root-history evidence · `rem:guide15:history-assurance`

Establishes the tested Phase-7 clear edge from a synthetic Phase-7 predecessor.

Does not establish protocol genesis, prior root history, cross-generation migration, production finality, or universal reorg safety.

## 19.10 Phase-7 result · `rem:guide15:phase-assurance`

Establishes one candidate burn/ASH/clear pipeline.

It does not establish redemption, admission, settlement, cycle, final constructors, final calibration, production operation services, or release.

---

# 20. Suggested implementation waves · `sec:guide15:waves`

Each wave ends formatted, focused-test green, documented, and committed before the next begins.

## Wave 0 — Revalidate Phase 6 and close preflight · `task:guide15:wave0`

**Deliverables**

- Phase-6 gate confirmed on the working tree;
- complete burn, ASH, clear, STATE, event, and accounting semantic censuses;
- public ASH representation selected;
- normalized-first versus direct-private burn disposition;
- shared ASH constructor disposition;
- burn-record carrier and value rule;
- clear STATE assignment;
- arithmetic-lowering assessment;
- Phase-7 STATE generation policy;
- inherited evidence boundary checks;
- clean-tree record.

**Suggested commit**

```text
plans: charter the burn ASH and clear tranche
```

## Wave 1 — Typed burn and clear realization · `task:guide15:wave1`

**Deliverables**

- complete typed burn declaration;
- complete typed clear declaration;
- exact event and accounting projections;
- exact `tag-recon` relation;
- exact residual rule;
- public ASH availability;
- architecture weld;
- model and realization conformance.

**Suggested commit**

```text
realization: define burn and clear semantics
```

## Wave 2 — Compiler operation plans · `task:guide15:wave2`

**Deliverables**

- validated burn plan;
- revalidated compact-ASH plan;
- validated clear plan;
- Phase-7 cross-operation scope;
- relation, carrier, layout, coverage, and lifecycle censuses;
- ASH constructor closure;
- corruption-resistant validators;
- independent small-instance oracles.

**Suggested commit**

```text
compiler: expose burn ASH and clear target plans
```

## Wave 3 — Target and representation closure · `task:guide15:wave3`

**Deliverables**

- public ASH representation assessment;
- burn owner-sighash assessment;
- clear arithmetic capability assessment;
- event-carrier assessment;
- STATE constructor-generation assessment;
- exact target evidence roles;
- private burn route disposition.

**Suggested commit**

```text
target-elements: assess burn ASH and clear requirements
```

## Wave 4 — Public ASH constructor integration · `task:guide15:wave4`

**Deliverables**

- exact public ASH constructor;
- burn-created ASH output pattern;
- compact compatibility;
- clear input and residual compatibility;
- ownerless and permissionless guarantees;
- constructor mutation vectors;
- public construction tests.

**Suggested commit**

```text
tapscript: integrate the public ASH constructor
```

## Wave 5 — Burn patterns · `task:guide15:wave5`

**Deliverables**

- receipt recognition;
- every-owner authorization;
- unique fresh ASH;
- live-change closure;
- exact burn aggregate;
- burn-record carrier;
- sponsor isolation;
- root, issuance, and destruction absence;
- burn event placement;
- complete abstract schedules.

**Suggested commit**

```text
tapscript: implement owner-authorized burn
```

## Wave 6 — Phase-7 STATE constructor and clear patterns · `task:guide15:wave6`

**Deliverables**

- expanded maturity-plus-clear static subtree;
- maturity regression under the new generation;
- clear STATE recognition;
- public ASH aggregate;
- exact minimum computation;
- successor STATE reconstruction;
- exact destruction;
- residual branch;
- no RESV;
- clear event;
- final-stack and resource formulas.

**Suggested commit**

```text
tapscript: implement permissionless clear
```

## Wave 7 — Linker extension · `task:guide15:wave7`

**Deliverables**

- burn, ASH, event, destruction, and Phase-7 STATE symbols;
- two-pass resolution;
- SCC disposition;
- structured relocations;
- duplicate-sensitive trees;
- exact tree costs;
- cross-operation ASH closure;
- candidate linked Phase-7 bundle.

**Suggested commit**

```text
linker: link the burn ASH and clear candidate
```

## Wave 8 — Candidate ABIs and burn signing · `task:guide15:wave8`

**Deliverables**

- burn ABI;
- inherited compact ABI revalidated;
- clear ABI;
- typed requests and public views;
- exact family layouts;
- burn finalization;
- multi-owner signing;
- sponsor consistency;
- canonical bytes;
- post-signing mutation refusal;
- candidate-only status.

**Suggested commit**

```text
transaction: derive burn ASH and clear ABIs
```

## Wave 9 — Burn safety and event evidence · `task:guide15:wave9`

**Deliverables**

- canonical burn fixtures;
- owner, asset, class, value, change, and sponsor matrices;
- burn-record matrix;
- event-type and value-anchor evidence;
- accepted burn projections;
- validated burn safety and event reports.

**Suggested commit**

```text
vectors: complete burn and burn-event evidence
```

## Wave 10 — ASH maintenance evidence · `task:guide15:wave10`

**Deliverables**

- fresh, compacted, and residual ASH fixtures;
- compact target execution;
- permissionless unrelated-process construction;
- event-silence checks;
- constructor compatibility;
- lifecycle evidence.

**Suggested commit**

```text
vectors: verify public ASH maintenance
```

## Wave 11 — Clear safety and STATE evidence · `task:guide15:wave11`

**Deliverables**

- every minimum branch;
- STATE assignment matrix;
- destruction and residual matrices;
- no-RESV and root closure;
- clear event;
- constructor continuity;
- root-history edge;
- public successor recovery;
- validated reports.

**Suggested commit**

```text
vectors: complete clear STATE and history evidence
```

## Wave 12 — Resource study · `task:guide15:wave12`

**Deliverables**

- objective-specific burn, compact, and clear measurements;
- expanded STATE-constructor measurements;
- exact prediction/observation comparison;
- candidate bound study;
- noncanonical timing;
- explicit non-calibration result.

**Suggested commit**

```text
vectors: measure the burn ASH and clear candidate
```

## Wave 13 — Phase-7 gate and handoff · `task:guide15:wave13`

**Deliverables**

- package READMEs and contracts;
- Phase-7 card;
- backlog gate record;
- complete result matrix;
- identity, schema, security, interchange, and dependency impact;
- complete repository gate;
- clean final tree.

**Suggested commit**

```text
plans: record the burn ASH and clear candidate
```

---

# 21. Focused verification · `sec:guide15:verification`

## 21.1 Working cadence · `rule:guide15:rust-cadence`

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Focused runs supplement but do not replace complete workspace execution.

## 21.2 Model and realization · `tab:guide15:model-tests`

```sh
cargo test -p tripod-model burn
cargo test -p tripod-model ash
cargo test -p tripod-model clear
cargo test -p tripod-model root
cargo test -p tripod-model realization_conformance

cargo test -p tripod-realization
RUSTDOCFLAGS='-D warnings' \
  cargo doc -p tripod-realization --no-deps
```

Focused areas:

```text
burn aggregate
owner authorization
fresh ASH
burn records
event anchors
compact preservation
clear minimum
STATE decrement
Y - 1 floor
tag-recon destruction
residual ASH
no RESV
event separation
sponsor opacity
lifecycle
```

## 21.3 Compiler · `tab:guide15:compiler-tests`

```sh
cargo test -p tripod-compiler
RUSTDOCFLAGS='-D warnings' \
  cargo doc -p tripod-compiler --no-deps
```

Focused areas:

```text
burn plan
compact revalidation
clear plan
relation census
ASH constructor closure
STATE generation
carrier placement
layout
coverage
event roles
accounting roles
lifecycle
determinism
corruption validation
no target positions
no digest
```

## 21.4 Target and tapscript · `tab:guide15:target-tests`

```sh
cargo test -p tripod-target-elements
cargo test -p tripod-tapscript

RUSTDOCFLAGS='-D warnings' \
  cargo doc -p tripod-target-elements --no-deps

RUSTDOCFLAGS='-D warnings' \
  cargo doc -p tripod-tapscript --no-deps
```

Focused areas:

```text
explicit value and asset introspection
checked addition and subtraction
fixed-width comparison
burn owner signatures
event carrier
public ASH constructor
clear minimum
expanded STATE constructor
metadata leaf
tweak and parity
sponsor isolation
final stack
resource formulas
```

## 21.5 Linker and transaction · `tab:guide15:linker-transaction-tests`

```sh
cargo test -p tripod-linker
cargo test -p tripod-transaction

RUSTDOCFLAGS='-D warnings' \
  cargo doc -p tripod-linker --no-deps

RUSTDOCFLAGS='-D warnings' \
  cargo doc -p tripod-transaction --no-deps
```

Focused areas:

```text
typed burn and clear symbols
two-pass resolution
STATE constructor generation
duplicate leaf refusal
exact tree cost
ASH constructor equality
burn ABI
clear ABI
family ranges
owner finalization
record finalization
post-signing mutation
residual presence
canonical decode and encode
candidate status
```

## 21.6 Vectors and conformance · `tab:guide15:vectors-tests`

```sh
cargo test -p tripod-vectors
cargo test -p tripod-target-elements-conformance

RUSTDOCFLAGS='-D warnings' \
  cargo doc -p tripod-vectors --no-deps
```

Focused areas:

```text
canonical evidence plans
positive class witnesses
burn safety
burn event
record over-claim
compact silence
clear formula
STATE continuity
root history
public recovery
accounting
boundary equality
response shape
resource prediction
report determinism
```

## 21.7 Real target matrices · `tab:guide15:target-matrices`

Run separately:

```text
burn semantic safety
burn owner authorization
burn records and event anchors
public ASH constructor
compact ASH
clear arithmetic
clear STATE continuity
clear destruction and residual
root history
public recovery
sponsor forms
candidate resources
```

Each run records:

```text
source revision
target contract revision
deployment binding
network and genesis
executor provenance
protocol revision
candidate bundle
candidate ABI
STATE constructor generation
ASH representation
candidate bounds
case, relation, event, and edge censuses
exact submitted bytes
target verdicts
semantic projections
event projections
accounting projections
resource observations
infrastructure errors
wall time outside canonical report bytes
```

## 21.8 Documentation · `tab:guide15:documentation-tests`

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new tracked file enters its nearest Meson census in the same commit.

---

# 22. Full batch gate · `gate:guide15:batch`

After every coherent implementation series:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run real target matrices separately and serialized where shared-node contention would make a parallel result ambiguous.

Record:

```text
target and deployment
network and genesis
executor provenance
protocol revision
Phase-7 STATE constructor generation
public ASH representation
linked bundle
burn, compact, and clear ABIs
candidate bounds
case counts
relation counts
burn-record counts
event counts
root-edge counts
failures
infrastructure errors
report byte reproducibility
```

If dependencies changed:

```sh
cargo tree -e features
cargo metadata
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

# 23. Acceptance criteria · `sec:guide15:acceptance`

Accept the Phase-7 candidate only when:

## Entry and inheritance

- Phase 6 remains green;
- all preflight questions are explicitly answered;
- inherited compact-ASH and STATE-constructor boundaries are revalidated;
- STATE constructor generation policy is explicit;
- no Phase-6 object is spent through an uncommitted Phase-7 leaf;
- no unexplained identity or byte drift exists.

## Burn

- burn consumes a nonempty bounded live-receipt family;
- every input authenticates live class, exact `U`, owner, constructor, and representation;
- every concrete receipt input satisfies its owner authorization;
- all owners authorize one finalized output set;
- exactly one positive fresh ASH is created;
- optional live change is exact;
- \(I=A+C\) holds;
- every `U` output is classified;
- burn records are canonical;
- event-type and value anchors pass independently;
- record over-claim rejects without erasing burn provenance;
- no root, issuance, destruction, compact event, or clear event appears;
- sponsor value remains opaque.

## Public ASH and compact

- ASH amount is public and authentic;
- fresh, compacted, and residual ASH constructors are exactly compatible where claimed;
- unrelated processes can construct compact and clear;
- compact preserves the exact aggregate;
- compact requires no owner, operator, or creator secret;
- compact emits no burn record, burn event, or clear event;
- compact creates exactly one successor;
- no closed-asset escape exists.

## Clear

- one current canonical STATE is consumed;
- a nonempty bounded ASH family is consumed;
- every ASH value is authenticated;
- \(B\) is exact;
- \(X=\min(B,Y_L,Y-1)\) is exact;
- \(X>0\);
- successor STATE assignments match realization;
- every unaffected field is equal;
- the \(Y-1\) floor is preserved;
- exactly \(X\) is destroyed under `tag-recon`;
- residual ASH exists iff \(B-X>0\);
- residual value is exactly \(B-X\);
- no RESV or other root participates;
- clear is genuinely permissionless;
- clear event and transition certificate are exact;
- no burn event or record appears;
- root-history edge and constructor continuity pass;
- unrelated-process successor reconstruction passes.

## Evidence and resources

- safety, event, continuity, history, public construction, accounting, and resource reports remain separate;
- canonical coverage derives from validated transcripts;
- every native refusal matches its declared boundary;
- every positive class has an executable witness;
- target acceptance and semantic projection both pass;
- prediction equals target observation;
- absent observations remain absent;
- candidate parameters remain non-final;
- lifecycle incompleteness is explicit;
- canonical report bytes reproduce;
- no speculative digest is minted;
- no production secret interface is introduced;
- all repository gates pass;
- the final tree is clean.

---

# 24. Rejection criteria · `sec:guide15:rejection`

Reject or typed-stop the candidate if:

- one operation is merged into another semantically;
- target details enter realization or compiler core;
- public ASH depends on an unauthenticated opening;
- private burn is claimed from malformed-transaction rejection alone;
- a receipt owner is omitted;
- different owners sign different finalized burns;
- fresh ASH or records change after signing;
- a non-burn transaction creates burn credit;
- record over-claim erases genuine burn provenance;
- compact emits burn or clear provenance;
- compact requires owner or operator authorization;
- an ASH amount comes from creator memory;
- an ASH source is omitted or duplicated;
- \(B\) wraps or saturates;
- \(X\) is caller-selected without exact verification;
- \(X\) differs among STATE, destruction, and residual relations;
- clear reduces \(Y\) to zero;
- residual ASH presence or value is wrong;
- destruction amount, asset, or tag is wrong;
- RESV participates;
- a Phase-6 constructor is treated as if it committed a Phase-7 clear leaf;
- cross-generation continuity is claimed without migration;
- target acceptance substitutes for semantic, event, constructor, root-history, or accounting evidence;
- a wrong-boundary refusal is counted as answered;
- report answerability substitutes for validated report evidence;
- sponsor amount becomes a protocol operand;
- construction failure counts as target rejection;
- infrastructure failure carries target observations;
- missing resource data is encoded as zero;
- candidate and final statuses are conflated;
- a digest is added without an admitted consumer;
- any required gate fails or leaves a dirty tree.

---

# 25. Identity, schema, security, interchange, and dependency impact · `sec:guide15:impact`

## 25.1 Identity impact

Expected:

```text
Attestation version:
    unchanged

realization version:
    unchanged unless a semantic correction independently requires movement

architecture schema and semantic hashes:
    unchanged if burn, ASH, clear, event, and STATE meanings are already complete

compiler identity:
    none minted

ASH constructor identity:
    none minted for in-process candidate use

Phase-7 STATE constructor identity:
    none minted

bundle identity:
    none minted

ABI identities:
    none minted

event and report identities:
    none minted

deployment profile:
    dormant
```

Target-computed transaction identities remain observation locators, not newly designed content identities.

## 25.2 Schema impact

Expected typed schema work:

```text
validated burn operation plan
validated clear operation plan
Phase-7 cross-operation scope
public ASH constructor closure
Phase-7 STATE constructor generation
burn target patterns
clear target patterns
burn ABI
clear ABI
burn-record carrier
burn and clear event projections
burn safety report
ASH maintenance report
clear safety report
STATE continuity report
root-history report
public-construction report
accounting report
resource report
```

Every schema migration is explicit.

No historical report or executor protocol revision is widened silently.

## 25.3 Security impact

Permitted:

- public ASH values;
- public STATE metadata;
- public constructor data;
- public disposable owner and sponsor test material;
- public deterministic fixture values;
- disposable development-chain credentials confined to disposable node state.

Not permitted:

- production receipt-owner private keys;
- production sponsor private keys;
- production openings or blinders;
- production wallet seeds;
- production node credentials;
- production deployment authority;
- arbitrary secret input through command-line arguments;
- publication of raw child or request text as typed diagnostics.

Direct private burn, if admitted, remains a public-fixture target experiment and not production custody.

## 25.4 Interchange impact

Typed in-process values are not interchange documents.

Native executor frames remain under the existing protocol.

Rendered reports remain terminal audit closures until a real external consumer exists.

A burn-record or event document consumed outside the repository activates ADR-022 before that consumer merges.

## 25.5 Dependency impact

No new third-party dependency is expected.

A proposal requires:

- concrete consumer;
- exact source and version;
- features;
- transitive graph;
- licence;
- MSRV;
- unsafe and FFI boundary;
- build scripts;
- determinism implications;
- advisory status;
- lockfile impact;
- public API leakage;
- independent-oracle implications.

No semantic package gains a dependency on target conformance.

---

# 26. Final Phase-7 result matrix · `tab:guide15:result`

The completion report fills the final column.

| Boundary | Required result | Actual result |
|---|---|---|
| architecture/realization | burn, ASH, clear, event, and STATE semantics complete | |
| compiler | validated burn and clear target plans | |
| target assessment | every capability and external role classified | |
| ASH representation | public and authentically constructible | |
| burn authorization | every receipt input authorized | |
| burn aggregate | exact fresh ASH and change relation | |
| burn records | canonical and value-bounded | |
| burn event | type and value anchors both pass | |
| compact ASH | exact, permissionless, event-silent | |
| Phase-7 STATE constructor | maturity and clear static subtree | |
| clear arithmetic | exact \(B\) and \(X\) | |
| clear STATE | exact successor and nonzero floor | |
| destruction | exact `tag-recon` amount | |
| residual ASH | exact conditional output | |
| root history | one valid clear succession edge | |
| public recovery | unrelated process reconstructs successor | |
| sponsor isolation | exact and amount-opaque | |
| linking | deterministic candidate bundle | |
| transaction | deterministic candidate ABIs | |
| target execution | positive cases accepted, negatives rejected at declared boundaries | |
| accounting | event, destruction, STATE, and residual projections agree | |
| coverage | every relation-case disposed honestly | |
| resources | prediction equals observation | |
| lifecycle | redemption, admission, settlement, and cycle outstanding | |
| secret boundary | public disposable test material only | |
| identity | no speculative digest | |
| release | not claimed | |

---

# 27. Guide-15 exit checklist · `gate:guide15:exit`

## Entry and inheritance

- [ ] Phase-6 gate passes on the current tree;
- [ ] complete semantic sources are identified;
- [ ] every preflight question has an explicit disposition;
- [ ] compact-ASH inheritance is revalidated;
- [ ] public-declassification policy is revalidated;
- [ ] Phase-7 STATE generation policy is explicit;
- [ ] no unexplained identity drift exists;
- [ ] entry tree is clean.

## Burn

- [ ] nonempty bounded live-receipt inputs;
- [ ] exact live class and explicit `U`;
- [ ] every concrete owner authorization executes;
- [ ] all owners sign one finalized transaction;
- [ ] unique positive fresh ASH;
- [ ] exact live change;
- [ ] exact aggregate;
- [ ] canonical burn records;
- [ ] event-type anchor;
- [ ] record-value anchor;
- [ ] over-claim preserves burn provenance while refusing record credit;
- [ ] no root, issuance, destruction, compact event, or clear event;
- [ ] sponsor relation exact and amount-opaque.

## Public ASH and compact

- [ ] public ASH amount is authentic;
- [ ] no creator-private dependency;
- [ ] fresh, compacted, and residual constructors are compatible where claimed;
- [ ] compact aggregate exact;
- [ ] compact permissionless;
- [ ] one compact successor;
- [ ] no burn records or events;
- [ ] no clear event;
- [ ] no STATE, issuance, or destruction;
- [ ] residual ASH is usable by later maintenance.

## Clear

- [ ] one current canonical STATE predecessor;
- [ ] nonempty bounded ASH family;
- [ ] every ASH amount authenticated;
- [ ] aggregate \(B\) exact;
- [ ] minimum \(X\) exact;
- [ ] every minimum branch covered;
- [ ] \(X>0\);
- [ ] successor STATE exact;
- [ ] every unaffected field preserved;
- [ ] \(Y-1\) floor preserved;
- [ ] exact `tag-recon` destruction;
- [ ] residual iff positive;
- [ ] exact residual value;
- [ ] no RESV or other root;
- [ ] clear permissionless;
- [ ] clear event exact;
- [ ] transition certificate exact;
- [ ] no burn provenance;
- [ ] public successor recovery passes.

## Constructor and linking

- [ ] Phase-7 static subtree contains maturity and clear;
- [ ] no future-operation placeholder;
- [ ] metadata leaf remains unspendable;
- [ ] predecessor and successor constructor authentication pass;
- [ ] internal-key policy exact;
- [ ] branch ordering canonical;
- [ ] representation nonce deterministic;
- [ ] tweak totality exact;
- [ ] duplicate leaves reject;
- [ ] references resolve exactly once;
- [ ] SCC strategies explicit;
- [ ] tree costs exact or checked;
- [ ] declaration order does not change bytes;
- [ ] cross-generation continuity is not claimed.

## ABI and signing

- [ ] burn, compact, and clear layouts exact;
- [ ] coordinators unique;
- [ ] family ranges complete and disjoint;
- [ ] every closed-asset-capable output classified;
- [ ] burn outputs finalized before signing;
- [ ] owner response census exact;
- [ ] post-signing mutation rejects;
- [ ] sponsored and sponsorless forms do not silently downgrade;
- [ ] canonical decode/encode equality holds;
- [ ] candidate status remains non-final.

## Evidence

- [ ] canonical evidence plans have no unchecked constructor;
- [ ] every positive class has a typed witness;
- [ ] target acceptance and semantic projection both pass;
- [ ] native refusals match declared boundaries;
- [ ] unexpected earlier refusals remain visible;
- [ ] burn safety and burn event reports remain separate;
- [ ] clear safety, STATE continuity, root history, event, and accounting remain separate;
- [ ] operation reports retain generic transcripts;
- [ ] target and deployment binding exact;
- [ ] provenance validates;
- [ ] no infrastructure error counts as a target result;
- [ ] no mock satisfies the gate;
- [ ] canonical report bytes reproduce.

## Resources

- [ ] burn cases measured;
- [ ] compact cases measured;
- [ ] every clear minimum branch measured;
- [ ] expanded STATE constructor measured;
- [ ] owner and sponsor witnesses measured;
- [ ] control depth and bytes measured;
- [ ] prediction equals observation;
- [ ] absent observations remain absent;
- [ ] mismatch fails the report;
- [ ] candidate bounds remain non-final;
- [ ] timing remains noncanonical.

## Repository

- [ ] package READMEs current;
- [ ] package contracts current;
- [ ] phase index, roadmap, active card, and backlog agree;
- [ ] Phase-7 card current;
- [ ] every new file is in its Meson census;
- [ ] dependency review recorded;
- [ ] security impact recorded;
- [ ] identity, schema, and interchange impact recorded;
- [ ] `cargo fmt --all` passes;
- [ ] Clippy passes with warnings denied;
- [ ] workspace tests pass;
- [ ] `scripts/ci.sh` passes;
- [ ] Meson compile passes;
- [ ] Meson tests pass;
- [ ] advisory status passed or explicitly skipped;
- [ ] document reproducibility passed or explicitly deferred under policy;
- [ ] `git diff --check` passes;
- [ ] final repository status is empty.

---

# 28. Completion report template · `sec:guide15:report-template`

```text
Guide 15 result
===============

Starting state:
    source revision:
    working tree:
    Phase-6 result:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    realization version:
    target contract revision:
    executor protocol revision:
    compact-ASH candidate revision:
    STATE constructor generation:
    public-value policy:

Preflight:
    semantic census:
    ASH representation:
    private burn route:
    ASH constructor compatibility:
    burn-record carrier:
    burn-record value rule:
    clear STATE assignment:
    clear arithmetic:
    Phase-7 STATE generation:
    event-role separation:
    permissionless construction:
    clean tree:

Realization:
    burn operation:
    compact operation:
    clear operation:
    burn relation census:
    compact relation census:
    clear relation census:
    event census:
    accounting census:
    lifecycle:
    architecture weld:

Compiler:
    burn plan:
    compact plan:
    clear plan:
    Phase-7 scope:
    relation censuses:
    source requirements:
    authorization:
    constructibility:
    lifecycle:
    carriers:
    layouts:
    coverage:
    external evidence:
    graph handles exposed:
    target types exposed:
    identity minted:

Target assessment:
    reviewed target:
    deployment binding:
    explicit asset:
    explicit values:
    arithmetic:
    comparisons:
    signatures:
    sighash:
    event carrier:
    STATE constructor:
    target conservation:
    sponsor support:
    missing primitives:
    external evidence:

ASH:
    representation:
    amount availability:
    constructor:
    fresh output:
    compact successor:
    clear residual:
    constructor equality:
    public construction:
    owner dependency:
        none
    creator-private dependency:
        none

Burn tapscript:
    input recognition:
    owner authorization:
    coordinator:
    cardinality:
    fresh ASH:
    live change:
    aggregate:
    burn records:
    sponsor isolation:
    root absence:
    issuance absence:
    destruction absence:
    final stack:
    resources:

Compact tapscript:
    inherited patterns:
    changes:
    aggregate:
    permissionless path:
    event silence:
    final stack:
    resources:

Phase-7 STATE constructor:
    metadata schema:
    metadata leaf:
    static subtree:
    maturity leaves:
    clear leaves:
    branch side:
    representation nonce:
    tweak totality:
    internal key:
    key-path result:
    Phase-6 crossing:
        not claimed
    constructor continuity:
    public recovery:

Clear tapscript:
    STATE recognition:
    ASH recognition:
    aggregate B:
    minimum X:
    positive progress:
    STATE successor:
    Y - 1 floor:
    tag-recon destruction:
    residual:
    no RESV:
    clear event:
    sponsor isolation:
    final stack:
    resources:

Linker:
    symbols:
    definitions:
    references:
    SCCs:
    cycle strategies:
    relocations:
    duplicate-leaf result:
    ASH constructor closure:
    STATE tree:
    exact tree cost:
    carrier closure:
    deterministic rebuild:
    candidate bundle:

Burn ABI:
    input layout:
    output layout:
    request:
    change:
    records:
    sponsor region:
    finalization:
    owner signing:
    witness roles:
    canonical bytes:
    candidate status:

Compact ABI:
    input layout:
    output layout:
    public construction:
    sponsor region:
    canonical bytes:
    candidate status:

Clear ABI:
    input layout:
    output layout:
    current STATE view:
    ASH view:
    request:
    B derivation:
    X derivation:
    successor metadata:
    destruction:
    residual:
    event:
    sponsor region:
    witness roles:
    canonical bytes:
    candidate status:

Burn evidence:
    positive cases:
    owner faults:
    object faults:
    value faults:
    record faults:
    sponsor faults:
    accepted projections:
    native refusals:
    unexpected-boundary refusals:
    first-party refusals:
    infrastructure errors:
    canonical bytes:

Burn event:
    event-type anchor:
    record-value anchor:
    fresh ASH association:
    record count:
    accepted record total:
    over-claim result:
    burn provenance retained:
    canonical bytes:

ASH maintenance:
    fresh ASH consumed:
    compacted ASH consumed:
    residual ASH consumed:
    compact aggregate:
    permissionless:
    event silence:
    unrelated process:
    lifecycle:
    canonical bytes:

Clear evidence:
    B-minimum:
    Y_L-minimum:
    Y-minus-one-minimum:
    tied minima:
    full consumption:
    residual:
    STATE projection:
    destruction projection:
    event projection:
    sponsor forms:
    accepted target identities:
    native refusals:
    unexpected-boundary refusals:
    infrastructure errors:
    canonical bytes:

STATE continuity:
    predecessor metadata:
    successor metadata:
    static subtree:
    predecessor program:
    successor program:
    representation nonce:
    parity:
    control path:
    target comparison:
    residual assumptions:
    canonical bytes:

Root history:
    synthetic origin:
    predecessor cursor:
    successor cursor:
    edge count:
    edge sequence:
    stale predecessor:
    invalid intermediate edge:
    STATE termination:
    RESV edge:
    other root edges:
    checkpoint:
    reorg binding:
    prior-history claim:
        none
    canonical bytes:

Accounting:
    burn input total:
    fresh ASH:
    live change:
    record total:
    compact input total:
    compact successor:
    clear aggregate B:
    clear amount X:
    STATE decrement:
    destruction amount:
    destruction tag:
    residual amount:
    equations agree:
    residue noninterference:
    canonical bytes:

Resources:
    burn coordinator:
    burn members:
    compact programs:
    clear program:
    ASH constructor:
    Phase-7 STATE constructor:
    burn-record carrier:
    metadata leaf:
    control depth:
    control bytes:
    owner signatures:
    sponsor witnesses:
    initial witness items:
    peak main stack:
    peak alternate stack:
    largest element:
    arithmetic:
    hashes:
    tweaks:
    validation budget:
    burn weight:
    compact weight:
    clear weight:
    prediction mismatch:
    candidate bounds:
    final calibration claim:
        none

Lifecycle:
    burn:
        candidate implemented
    compact ASH:
        candidate implemented
    clear:
        candidate implemented
    redemption:
        outstanding
    request admission:
        outstanding
    settlement:
        outstanding
    cycle:
        outstanding
    constructor migration:
        outstanding
    release-complete:
        false

Security:
    public ASH:
    public STATE metadata:
    public disposable owner material:
    public disposable sponsor material:
    production private keys accepted:
        no
    production openings accepted:
        no
    production wallets accepted:
        no
    future secret-bearing review required:

Identity impact:
    Attestation:
    realization:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    compiler identity:
        none
    ASH constructor identity:
        none
    STATE constructor identity:
        none
    bundle identity:
        none
    ABI identities:
        none
    report identities:
        none
    deployment profile:
        dormant

Schema and interchange:
    typed schema changes:
    executor revision:
    report schemas:
    external burn-record consumer:
    ADR-022 allocation:
        none unless explicitly activated

Dependency impact:
    first-party changes:
    third-party additions:
    licences:
    MSRV:
    unsafe/FFI:
    advisories:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets -- -D warnings:
    cargo test --workspace:
    architecture:
    model:
    realization:
    compiler:
    target-elements:
    tapscript:
    linker:
    transaction:
    vectors:
    target-elements-conformance:
    Rustdoc:
    protocol cross-language:
    burn native matrix:
    burn-event matrix:
    compact native matrix:
    clear native matrix:
    STATE continuity matrix:
    root-history matrix:
    public-construction matrix:
    sponsor matrix:
    resource matrix:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Phase-7 verdict:
    accepted candidate
    / typed stopped
    / public-ASH path deferred
    / STATE-generation path deferred
    / target path rejected
    / blocked

Residuals:

Next phase:
```

---

# 29. Handoff after Guide 15 · `sec:guide15:handoff`

If Guide 15 succeeds, Phase 7 has:

- owner-authorized conversion of live receipts into fresh public ASH;
- canonical burn records with independent event-type and value anchors;
- permissionless public ASH compaction;
- permissionless clear against canonical STATE;
- exact \(X=\min(B,Y_L,Y-1)\);
- exact STATE decrement;
- exact `tag-recon` destruction;
- exact residual ASH;
- a Phase-7 STATE constructor generation containing maturity and clear;
- target-evidenced STATE succession by a permissionless operation;
- separate burn, clear, event, history, accounting, and resource evidence.

The next phase should implement wide arithmetic and redemption:

```text
Guide 16 — Wide Arithmetic and Redemption
```

Guide 16 consumes rather than reopens:

- canonical STATE metadata;
- Phase-7 STATE constructor-generation policy;
- authenticated STATE predecessor and successor reconstruction;
- public metadata recovery;
- finalized owner signing;
- exact explicit `U` classification;
- public ASH representation and lifecycle;
- destruction tagging;
- root-history evidence;
- sponsor isolation;
- target/deployment/provenance binding;
- relation-indexed coverage;
- exact resource prediction and observation.

It adds:

- wide exact floor arithmetic;
- one live-receipt owner redemption;
- formula-bound L-BTC payout;
- exact `tag-redeem` destruction;
- STATE decrement;
- RESV succession or terminal exhaustion;
- sealed-state transition;
- post-sealing non-revival;
- representation-specific redemption or normalization paths.

Guide 15 makes none of those redemption, RESV, sealing, payout, or wide-arithmetic claims.

---

## Closing statement · `rem:guide15:closing`

> Burn, ASH, and clear form one lifecycle only if their boundaries remain different. Burn must prove that every owner authorized one public fresh aggregate and its records; compact must preserve that public value without manufacturing a new event; clear must consume authenticated ASH permissionlessly, compute the exact bounded decrement, preserve the nonzero STATE floor, destroy exactly the cleared `U`, emit the right residual and event, and advance one canonical STATE edge under the Phase-7 constructor generation. Guide 15 succeeds only when those relations are welded without letting target conservation replace authorization, copied record bytes replace provenance, final cursor equality replace root history, sponsor value replace protocol value, or a new STATE subtree pretend that an older constructor committed to leaves it never contained.
