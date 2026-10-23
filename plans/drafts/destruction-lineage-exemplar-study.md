# Study: A second exemplar, public destruction with a settlement lineage

**Abstract (Status and scope)** · `abst:lineage:scope`

This labeled design study states a public-destruction exemplar's own contract and requirements, maps them onto the compiler pipeline and reviewed Elements target, specifies measurements, compares the attestation exemplar and retains open alternatives. Destruction writes magnitude in native consensus value and destination as an opaque datum; a positively funded claim token connects it to a settlement lineage whose covenant checks only the successor script. Readers reconstruct ancestry and consumers publish recognition rules. Positive spendable funding is required by (`fact:lineage:positive-funding`); relay floors and execution costs remain the experiments of (`sec:lineage:measurements`). The study supplies planning input under (`rule:drafts:authority`); the charter separately admits independence evidence inside Phase 7 (`sec:phase7:change-statement`).

## 1. Why a second exemplar · `sec:lineage:motivation`

**Thesis (Compilation establishes the product)** · `thesis:lineage:compiler-product`

The thesis in `README.md` is that a compiler is proved by what it compiles: the attestation is the exemplar carried beside the compiler, and the compiler is the product. The same ordering governs the phase dependencies (`rule:roadmap:dependencies`) and roadmap change rule (`rule:roadmap:change`); a second contract must test that thesis without silently becoming a roadmap commitment.

**Motivation (Independence beyond one semantic family)** · `mot:lineage:second-family`

The first exemplar demonstrates depth within one semantic family. A second exemplar supplies the first test of whether the pipeline's boundaries survive a different family: typed declaration, relation analysis, target emission, linking, transaction construction and translation-validation evidence must each accept the new contract on its own terms. (`pkg:realization:contract`); (`pkg:compiler:contract`); (`pkg:tapscript:contract`); (`pkg:linker:contract`); (`pkg:transaction:contract`); (`pkg:vectors:contract`).

**Definition (Exemplar independence)** · `def:lineage:independence`

Exemplar independence means that a declaration can describe different objects and operations while preserving the responsibilities of those stages. It does not mean reusing attestation operation identifiers for unrelated meanings, bypassing analysis with handwritten bytes, or declaring success because a target script executes once.

**Fact (Existing declaration boundary)** · `fact:lineage:declaration-boundary`

The present realization dispatches compact ASH, live-receipt transfer and maturity announcement; other operations receive `UnsupportedOperationDeclaration` in `packages/realization/src/declarations/mod.rs`. In particular, neither the attestation's burn and clear nor this exemplar's BURN and MERGE are already complete declarations. (`pkg:realization:contract`).

**Justification (A minimal recurrence tests architectural seams)** · `just:lineage:minimal-test`

One covenant with one semantic check is a sharp test because incidental complexity cannot explain an inability to express it. If declaration or construction requires STATE, receipt ownership or a conserved issued asset, that requirement exposes an attestation-specific seam rather than an intrinsic need of compilation.

**Caveat (One check is not the entire assurance argument)** · `cav:lineage:assurance-scope`

The one check does not make the entire correctness argument one instruction. Script recognition, constructor authentication, target encoding, ancestry traversal and evidence boundaries still need exact definitions and tests; their owners should remain visible rather than disappear into an informal simplicity claim.

**Specification (Deliberate exclusions)** · `spec:lineage:exclusions`

This exemplar deliberately omits receipt issuance, redemption, reserve accounting, maturity, STATE arithmetic, consumer eligibility and consumer timing. It introduces no third operation and no quantity accumulator, and it does not ask Phase 7 to replace its own burn-and-clear lifecycle (`phase:roadmap:burn-clear`).

**Caveat (Study standing and execution authority)** · `cav:lineage:planning-authority`

The study's admission assessment treats Phase 7 as Active and Guide 15 admission as its first gate (`phase:roadmap:burn-clear`); its own alternatives remain open under (`q:lineage:admission`) and its authority remains planning input (`rule:drafts:authority`). The execution charter separately admits the second contract inside Phase 7 and places it in Wave 1 (`sec:phase7:change-statement`); (`task:guide15-exec:wave1`). (`sec:phase6:exit-assessment`).

## 2. The contract · `sec:lineage:contract`

**Definition (Destruction output)** · `def:lineage:destruction-output`

A destruction output is an OP_RETURN output of the native asset with an explicit value and exactly one opaque 32-byte address in its data. A valid record requires a positive destruction value; no quantity written in the data substitutes for that consensus amount.

**Definition (Minimal-value claim token)** · `def:lineage:claim-token`

A minimal-value claim token is an output carrying one fixed covenant script, created at output index 0 in the same transaction as its sibling destruction output at index 1. It carries positive native value at or above the admitted minimum, but its own value has no meaning as record credit.

**Definition (Lineage output)** · `def:lineage:lineage-output`

A lineage output carries the same fixed covenant script as the claim token (`def:lineage:claim-token`). Recognition of this program establishes the recurrence class, while the designated origin separately establishes settlement authority (`def:lineage:genesis`).

**Definition (Designated genesis)** · `def:lineage:genesis`

The deployment designates one genesis outpoint, created without a destruction beside it. Genesis establishes the lineage origin and contributes no record; an arbitrary output with the same script cannot replace the designated origin (`q:lineage:genesis-designation`).

**Fact (Explicit spendable zero is refused)** · `fact:lineage:positive-funding`

Positive incidental funding is required for the explicit spendable representation. The reviewed `ExplicitZeroValueRule::SpendableRefused` term and its source note in `packages/target-elements/src/transaction_form.rs` state that the target amount check refuses an explicit zero-valued spendable output; this is a reviewed consensus fact, not an unanswered dust question.

**Condition (Admitted positive relay funding)** · `cond:lineage:relay-funding`

The minimum suitable for default relay remains a deployment measurement. An admitted positive minimum funds each constructed token and lineage output; it is not a magnitude unit and does not acquire significance because a relay policy chooses its threshold. This standing is NEEDS MEASUREMENT under (`expt:lineage:relay-floor`).

**Caveat (Script recognition does not certify asset or value)** · `cav:lineage:recognition-limits`

The covenant does not inspect or preserve value or asset. The positive-native description states the intended construction, while the script recognizes only its own program; a reader must not infer a native-asset or value guarantee from recognition of that program alone.

**Definition (Incidental funding and sweep bounty)** · `def:lineage:sweep-bounty`

Consumed tokens' incidental values may flow into the successor, transaction fees or other outputs. A donation may sit on a lineage output, and a later permitted spend may redirect it; no record reader counts that donation and no covenant rule preserves it. The unpreserved incidental value is a sweep bounty, never an accumulated quantity or record entitlement (`just:lineage:one-check`).

**Principle (Record credit is independent of container funding)** · `prin:lineage:funding-neutrality`

Neutrality concerns the record's credit and entitlements. The token confers no redemption claim, ownership role or price on its record, even though the spendable container necessarily has incidental funding that an unrestricted merge can consume.

**Justification (Destruction occurs at recording)** · `just:lineage:immediate-destruction`

Destruction uses OP_RETURN because the amount must become irrecoverable at the recording event. A locked covenant output would instead retain a spendable amount and make the destruction claim depend on every future exit; irreversible output semantics separate paid magnitude from the lineage's incidental funding.

**Justification (Native magnitude needs no issued asset)** · `just:lineage:native-magnitude`

No issued asset is needed: native value supplies the exogenous magnitude directly. An issued unit would introduce issuance authority and authenticity questions that the contract does not need to answer to record a destruction.

**Definition (Opaque permanent address)** · `def:lineage:address`

The address is exactly 32 bytes, written by the burner. It makes no identity claim, is bound to no key or role by the ledger, and remains the same datum when the record enters any later settlement point.

**Operation (BURN)** · `op:lineage:burn`

BURN is any transaction whose output 0 is a minimal-value claim token and whose output 1 is a destruction output. Anyone can build it; apart from the pair's defining shape, the exemplar constrains no input, other output, authorization or carrier purpose.

**Precondition (Ordinary target validity)** · `pre:lineage:target-validity`

Ordinary target validity still applies, including funding, authorization of unrelated inputs and the target fee-output rules. Those requirements do not create a special burner role or a third exemplar operation. The target obligations are specified by (`pkg:target-elements:contract`).

**Justification (Fixed positions determine the association)** · `just:lineage:fixed-pair`

Fixed indices eliminate a search for the association: the token's transaction and sibling index identify the claimed destruction uniquely. They also make malformed pairs fail closed without interpreting an arbitrary list of payloads or selecting a favorable output.

**Operation (MERGE)** · `op:lineage:merge`

MERGE spends one or more outputs carrying the covenant script and produces output 0 carrying the same script. Each covenant spend checks exactly that successor-at-0 condition, with no signature, value comparison, asset comparison or constraint on any other input or output.

**Corollary (A common successor needs no coordinator entitlement)** · `cor:lineage:open-carrier`

The common successor lets many inputs agree on one continuation without adding a coordinator entitlement. A merge may ride inside any otherwise valid carrier transaction; the covenant has no knowledge of the carrier's consumer or purpose.

**Case (BURN and MERGE coincide)** · `case:lineage:combined-operation`

A transaction can satisfy both operation descriptions: it can consume lineage-script outputs while producing a new valid token/destruction pair at indices 0 and 1. The descriptions are predicates, not disjoint transaction tags; the reader must not lose either the inherited records or the newly anchored record.

**Justification (Exactly one covenant condition)** · `just:lineage:one-check`

One check is deliberate. Adding a value sum would preserve incidental funding rather than record magnitude, and adding consumer authorization would turn an open settlement path into a controlled service; neither follows from the primitive's requirement set.

**Definition (Settlement lineage and point)** · `def:lineage:settlement-point`

The settlement lineage is the sequence of merges descending from the designated genesis outpoint. Its k-th spend defines lineage point k: its depth is the index, its successor outpoint is its identity, and the spent lineage-head input identifies its predecessor.

**Invariant (A head has at most one successor in a selected history)** · `inv:lineage:unique-successor`

The spend relation gives at most one successor to any particular head in a selected valid history. It places no limit on how many successive heads can be spent within one block, and no lineage index is stored in an output.

**Proof (Successor uniqueness follows from spending)** · `pf:lineage:unique-successor`

Two distinct successors of the same head would require two accepted spends of that outpoint in the same selected valid history. The selected history admits no such double spend, so its spend relation gives at most one successor (`inv:lineage:unique-successor`).

**Caveat (Depth is neither block height nor stored state)** · `cav:lineage:depth`

Several successive heads may be spent within one block, so the index of a lineage point is not its block height. No lineage index is stored in an output; the reader derives the depth from the designated predecessor chain (`def:lineage:settlement-point`).

**Definition (Side pile)** · `def:lineage:side-pile`

A merge that does not spend the designated head produces a side pile with the same script but without the genesis lineage's authority as a settlement boundary. Its local history may be shorter; history length alone never designates the canonical lineage.

**Corollary (Sweeping a side pile admits its ancestry)** · `cor:lineage:side-pile-sweep`

A later merge can consume a side pile together with the head. The pile's entire record ancestry then enters the settlement lineage, so a burner can aggregate its own records without first winning access to the current head.

**Notation (Settled record set)** · `ntn:lineage:settled-set`

Let L(p) be the set of records settled at lineage point p. These are the valid claim-token records in the point's ancestry, including a valid pair created by its transaction when BURN and MERGE coincide; earlier sets are subsets of later sets along the same lineage.

**Invariant (Settled sets grow along one lineage)** · `inv:lineage:set-growth`

For successive points p and q on the same selected lineage, L(p) is a subset of L(q). This is a relation between ancestry-defined sets, not a balance equation over incidental funding (`ntn:lineage:settled-set`); (`def:lineage:sweep-bounty`).

**Proof (Continuation retains inherited ancestry)** · `pf:lineage:set-growth`

A successor spends the preceding head, so its ancestry includes that head and every dependency already reachable through it. Additional consumed branches and a newly valid pair may add record identities but cannot remove inherited ones; set union proves (`inv:lineage:set-growth`).

**Definition (Record identity)** · `def:lineage:record-identity`

A record is identified by its token outpoint, not by address or amount. Two separate destructions at the same address are distinct records; encountering the same ancestral transaction through multiple inputs never creates another copy of one record.

**Criterion (A pair contributes authenticated destruction)** · `crit:lineage:valid-pair`

For each candidate pair, a reader verifies one transaction, indices 0 and 1, the covenant token, and an OP_RETURN sibling with explicit positive native value and exactly 32 bytes of address data. The contribution is that sibling's value at that address if the pair verifies, and zero otherwise.

**Counterexample (A same-script token with a malformed sibling)** · `cex:lineage:false-credit`

The token's native funding is never the amount credited. A fake token, malformed sibling, unrelated data output, wrong asset or committed destruction value cannot create credit merely by joining a merge. For example, a token beside a sibling with only 31 address bytes fails (`crit:lineage:valid-pair`), even if its script can participate in MERGE.

**Invariant (Record identities are settled exactly once)** · `inv:lineage:exactly-once`

The lineage settles each record once as a set member. Spending a token once prevents independent token reuse, and set identity prevents double counting when historical paths share transactions; already settled ancestry remains present without being credited again.

**Proof (Spending and set identity prevent duplicate credit)** · `pf:lineage:exactly-once`

Each record is keyed by its token outpoint (`def:lineage:record-identity`). Traversing two branches that reach that outpoint inserts the same member twice into a set and therefore changes the set only once; the selected history also forbids two independent spends of the token. Together these facts establish (`inv:lineage:exactly-once`).

**Invariant (Settlement stores no aggregate and rewrites no record)** · `inv:lineage:immutable-records`

Settlement carries no total, count, index or aggregate commitment on the ledger. Every figure is computed by a reader from ancestry, and merging never rewrites an address or transfers one record's amount into another record.

**Principle (Ledger settlement and consumer recognition are separate)** · `prin:lineage:two-layers`

The ledger settles an atomic ancestry-defined set at each point, while a consumer separately recognizes boundaries and accepted records. The separation is governed by (`def:lineage:consumer-recognition`) and yields (`bound:lineage:acceptance`); neither consumer eligibility nor carrier authorization becomes a MERGE condition.

**Bound (Acceptance cannot exceed settled ancestry)** · `bound:lineage:acceptance`

At every consumer-recognized point p, the accepted set A(p) must be a subset of L(p). This bound permits a consumer to decline settled records but forbids it to count an unsettled record (`ntn:lineage:settled-set`); (`def:lineage:consumer-recognition`).

**Definition (Consumer recognition)** · `def:lineage:consumer-recognition`

The consumer recognizes which points are its boundaries and which settled records it accepts. Eligibility, completeness relative to its own purpose, timing, epochs and acceptance rules belong to that consumer and are not conditions of a valid merge.

**Specification (Publication of a recognition rule)** · `spec:lineage:recognition-publication`

A consumer publishes its recognition rule through the provenance it requires of the carrier transaction: a data output, the carrier's act, a role's signature or another declared criterion. None of these optional mechanisms is a new ledger operation.

**Invariant (Successive accepted sets remain monotone and bounded)** · `inv:lineage:accepted-sets`

For successive recognized boundaries, accepted record sets obey A(previous) ⊆ A(this) ⊆ L(this). Since valid contributions are non-negative and immutable, this subset rule gives nondecreasing address totals along the selected history.

**Corollary (Address totals do not decrease along the selected history)** · `cor:lineage:address-monotonicity`

For successive recognized boundaries satisfying (`inv:lineage:accepted-sets`), the sum of accepted destruction values at each address is nondecreasing. This conclusion concerns the selected history and the published acceptance rule, not unconditional finality across replacement histories.

**Proof (Immutable non-negative contributions preserve totals)** · `pf:lineage:address-monotonicity`

Every previously accepted member remains accepted, keeps its original address and contributes its original positive destruction value; new members contribute non-negative amounts (`crit:lineage:valid-pair`); (`inv:lineage:immutable-records`). Thus extending the accepted set cannot reduce an address sum, proving (`cor:lineage:address-monotonicity`).

**Case (Acceptance equals the settled set)** · `case:lineage:accept-all`

A consumer taking acceptance equal to the settled set is the instance in which no party selects which settled records count. Other consumers may select subsets, but must publish that choice and preserve the same inclusion bound.

**Caveat (A commitment is only a cache)** · `cav:lineage:cache-authority`

A commitment carried by a transaction is at most a cache for the reader. It neither adds a record to ancestry nor chooses acceptance; if its bytes disagree with verified ancestry, the cache is wrong.

**Claim (Indefinite exclusion requires every successive head race)** · `claim:lineage:exclusion-races`

Each head is contended, but anyone can build its next merge. To exclude one available record indefinitely, an excluder must win every successive head race before any admitted merge includes that record; one successful inclusion settles it permanently within the surviving history.

**Proof (One surviving inclusion defeats continued exclusion)** · `pf:lineage:exclusion-races`

If any accepted successor includes the available record, ancestry monotonicity retains it at every later point on that surviving lineage (`inv:lineage:set-growth`). Therefore exclusion throughout that lineage entails defeating every opportunity that would include it, which proves (`claim:lineage:exclusion-races`) without asserting when such an opportunity is admitted.

**Assumption (Conflicting spends can compete by fee)** · `assum:lineage:fee-competition`

The deployment permits conflicting head spends to compete by fee under its actual replacement conditions. This premise needs the policy evidence of (`expt:lineage:head-conflicts`); permissionless covenant access alone does not establish it.

**Assumption (A fee-paying transaction eventually receives inclusion)** · `assum:lineage:eventual-inclusion`

A suitably funded transaction has an eventual inclusion opportunity under the deployment assumptions. Block creators retain inclusion discretion, so this premise is published rather than proved by the covenant or by a finite replacement experiment (`cav:lineage:block-discretion`).

**Argument (Exclusion pays repeatedly while inclusion needs one success)** · `arg:lineage:cost-asymmetry`

The honest side needs one win, whereas an excluder needs every win. Under fee competition, continuing exclusion requires continuing expenditure, while a defender pays once to win an inclusion opportunity; the burner is always able to act as a defender. (`assum:lineage:fee-competition`); (`assum:lineage:eventual-inclusion`).

**Caveat (No universal time or budget guarantee)** · `cav:lineage:inclusion-limits`

This asymmetry is conditional on (`assum:lineage:fee-competition`) and (`assum:lineage:eventual-inclusion`). It is not a bound on inclusion time, a promise that any replacement is admitted, or a guarantee that a finite budget defeats every adversary.

**Condition (Deployment policy admits the proposed competition)** · `cond:lineage:conflict-policy`

Actual replacement rules, admissible fee increments and package constraints determine what an effective higher bid means. Experiment (`expt:lineage:head-conflicts`) must establish that policy before a deployment states a fee-race claim.

**Case (First-seen contention)** · `case:lineage:first-seen`

Under first-seen policy the race is propagation rather than a simple fee auction: indefinite exclusion still requires winning every successive race, but the one-higher-fee argument does not follow. The deployment must state which policy supports its censorship account beside its settlement depth.

**Caveat (Block creators retain inclusion discretion)** · `cav:lineage:block-discretion`

A block creator can refuse an inclusion regardless of fee, just as it can refuse the original destruction transaction. This is the ledger's own inclusion exposure, expressly retained as an assumption rather than hidden inside permissionless script access.

**Procedure (Reconstruct records at a selected lineage point)** · `alg:lineage:reader`

Given a lineage point, the designated genesis and ancestry transactions with inclusion proofs, first authenticate the selected history and the predecessor chain; then walk every relevant consumed input, obtaining previous-output evidence to classify branches. Check each reached candidate pair by (`crit:lineage:valid-pair`), insert its token outpoint into a visited set, and union each verified record once, including a new pair at a combined BURN/MERGE point. Refuse a complete reading when any dependency is absent (`crit:lineage:ancestry-completeness`); otherwise sum the immutable contributions by address. This procedure reconstructs the settled set without scanning intervening blocks and totaling arbitrary data outputs.

**Precondition (Complete ancestry and classification evidence)** · `pre:lineage:ancestry-evidence`

The supplied closure must include every relevant consumed branch through which a lineage-script output can introduce records, and enough previous-output evidence to classify those branches. A missing transaction makes the proof incomplete; it never licenses treating an unknown branch as empty.

**Criterion (Missing ancestry makes reconstruction incomplete)** · `crit:lineage:ancestry-completeness`

A reconstruction is complete only if every relevant consumed branch and the previous-output evidence needed to classify it are supplied. An unknown branch is not an empty branch, and a missing dependency prevents the complete-set conclusion (`pre:lineage:ancestry-evidence`).

**Assumption (Historical transaction availability)** · `assum:lineage:availability`

Historical transaction retrieval is the ledger's data-availability assumption. The spend relation authenticates dependencies but does not itself operate an archive or guarantee that a reader can retrieve every required byte.

**Invariant (A surviving point binds its original ancestry)** · `inv:lineage:replacement-history`

A surviving point binds the same transaction ancestry and the same settled records; nothing can move a record across that point while preserving its identity and dependencies. Inclusion proofs must still bind the point and ancestry to the selected history.

**Case (The selected point survives)** · `case:lineage:surviving-point`

When the same point survives with the same authenticated dependencies, its ancestry and record identities remain fixed (`inv:lineage:replacement-history`). Inclusion proofs must nevertheless establish membership of that point and its ancestors in the history being read.

**Case (The selected point disappears)** · `case:lineage:dropped-point`

In an alternative history a record is settled wherever that history settles it, possibly nowhere, and a dropped lineage point settles nothing. A different point may settle the same surviving destruction record without preserving the old boundary.

**Case (A different point settles the surviving record)** · `case:lineage:replacement-point`

A replacement history may retain the destruction transaction while settling its record at a different point. The record remains identified by its token outpoint, but its former settlement boundary does not survive merely because the destruction does (`def:lineage:record-identity`).

**Assumption (Published settlement depth and finality premises)** · `assum:lineage:finality`

Finality is a published depth with explicit assumptions. Consumer publication rules pin readings, bound interim readings by the next recognized settled set, and restrict revisions of nonfinal readings to downward corrections no lower than the last final pin; the ledger does not manufacture those publication rules.

**Bound (Interim readings and downward revisions respect pins)** · `bound:lineage:publication-revisions`

An interim reading is bounded by the next recognized settled set; permitted downward revisions of nonfinal readings cannot go below the last final pin. These are consumer publication obligations under declared depth and availability assumptions, not additional MERGE predicates (`q:lineage:consumer-publication`).

## 3. The requirement set and its standing · `sec:lineage:requirements`

**Motivation (A common reference needs intrinsic guarantees)** · `mot:lineage:requirement-basis`

These requirements define what public destruction must provide to serve as a common reference. Their reasons are intrinsic: independent magnitude needs a real cost, neutral records need stable attribution, and shared readings need an explicit boundary between settled evidence and consumer choices.

**Definition (Standing vocabulary)** · `def:lineage:standings`

HOLDS BY CONSTRUCTION refers to the abstract contract and a faithful realization, not an assertion that emitted bytes already exist. HOLDS BY PUBLICATION names an obligation discharged by a deployment statement; HOLDS BY CONSUMER RULE names behavior that a consumer must publish and implement; NEEDS MEASUREMENT reserves an empirical conclusion.

**Requirement (Exogenous magnitude)** · `req:lineage:exogenous-magnitude`

Magnitude independent of any consumer's rule, so acceptance cannot manufacture paid amount. Standing: HOLDS BY CONSTRUCTION. Credit is the explicit native value irrecoverably destroyed at output 1, never a consumer-written scalar. The standing rests on (`crit:lineage:valid-pair`).

**Requirement (Neutrality)** · `req:lineage:neutrality`

A record carries no value claim, redemption or price, so recording creates no beneficiary entitlement. Standing: HOLDS BY CONSTRUCTION. The minimal-value claim token has no holder role; its incidental funding is ignored, unpreserved and distinct from record credit. The standing rests on (`prin:lineage:funding-neutrality`).

**Requirement (Non-transferability)** · `req:lineage:non-transferability`

Recorded magnitude remains at its original address, so aggregation cannot reassign paid history. Standing: HOLDS BY CONSTRUCTION. Every record retains its own transaction's address and amount; merge unions record identities rather than pooling their credit. The standing rests on (`inv:lineage:immutable-records`).

**Requirement (Settled monotone reading)** · `req:lineage:monotone-reading`

Pinned address readings never decrease along a history, interim readings stay within the next pin, and permitted nonfinal revisions respect the last final pin. Standing: HOLDS BY CONSUMER RULE. Non-negative L supplies the substrate; published A(previous) ⊆ A(this) ⊆ L(this), pin selection and revision policy supply the reading discipline. The standing rests on (`bound:lineage:publication-revisions`).

**Requirement (Opaque permanent addresses)** · `req:lineage:opaque-addresses`

An enduring datum without a ledger identity claim, so attribution does not depend on a role registry. Standing: HOLDS BY CONSTRUCTION. Exactly 32 burner-written bytes remain recoverable in the original destruction transaction. The standing rests on (`def:lineage:address`).

**Requirement (Non-negativity)** · `req:lineage:non-negativity`

No record subtracts credit, so extending a settled set cannot reduce an address total. Standing: HOLDS BY CONSTRUCTION. A verified explicit positive value contributes itself; a failed pair contributes zero. The standing rests on (`crit:lineage:valid-pair`).

**Requirement (Provenance auditability)** · `req:lineage:auditability`

Every counted unit is capped by genuine same-event destruction, so copied data cannot create magnitude. Standing: HOLDS BY CONSTRUCTION. Equality to the sibling destruction amount and an ancestry walk authenticate each counted unit, conditional on supplied complete evidence. The standing rests on (`crit:lineage:ancestry-completeness`).

**Requirement (Instant costliness and append-only record)** · `req:lineage:instant-cost`

Cost is paid at recording and surviving records cannot be edited or spent back into value. Standing: HOLDS BY CONSTRUCTION. The native OP_RETURN output destroys value at BURN; later merges consume tokens, never destruction outputs. The standing rests on (`just:lineage:immediate-destruction`).

**Requirement (Bootstrap capacity)** · `req:lineage:bootstrap`

An opening from the first acquisition block states its pre-acquisition bound, so earlier magnitude cannot be silently assumed. Standing: HOLDS BY PUBLICATION. Publish zero for the empty pre-acquisition interval and the reason the interval is empty; genesis itself contributes no destruction. The standing rests on (`def:lineage:genesis`).

**Requirement (Unit cost)** · `req:lineage:unit-cost`

A positive cost per unit in a declared denomination and terms, so magnitude has an interpretable price. Standing: HOLDS BY PUBLICATION. Publish one native unit destroyed per unit credited and the transaction fee as a separate per-recording-event cost. The standing rests on (`obl:lineage:cost-publication`).

**Requirement (Operator accrual)** · `req:lineage:operator-accrual`

Explain whether operation accrues destroyed magnitude to any role, so neutral accounting cannot conceal a beneficiary. Standing: HOLDS BY PUBLICATION. Publish that merging grants no destroyed amount or record entitlement; fees go to block creators, while incidental token funding is separately disclosed. The standing rests on (`cav:lineage:operator-benefit`).

**Requirement (Lineage)** · `req:lineage:lineage`

A designated origin, unique successor by spend, ancestry dependence and exactly-once settlement, so a pin selects a determinate cumulative set. Standing: HOLDS BY CONSTRUCTION. Genesis designation plus consensus spending determines the chain; token outpoint identity and set traversal prevent duplicate record credit. The standing rests on (`inv:lineage:exactly-once`).

**Requirement (Consumer-defined acceptance within the ledger bound)** · `req:lineage:consumer-acceptance`

Consumers select recognized boundaries and accepted subsets without claiming unsettled records, so acceptance remains explicit and bounded. Standing: HOLDS BY CONSUMER RULE. Each consumer publishes its carrier-provenance rule and preserves A(previous) ⊆ A(this) ⊆ L(this); acceptance equal to L removes subset selection. The standing rests on (`inv:lineage:accepted-sets`).

**Table (Requirement standings)** · `tab:lineage:requirement-standings`

The thirteen requirements retain eight construction standings, three publication standings and two consumer-rule standings. No row claims a completed measurement; deployment admission remains subject to (`expt:lineage:relay-floor`), (`expt:lineage:data-output`) and the remaining experiments in (`sec:lineage:measurements`).

| Requirement | What it demands | Standing under the contract | Argument or evidence |
|---|---|---|---|
| Exogenous magnitude (`req:lineage:exogenous-magnitude`) | Magnitude independent of any consumer's rule, so acceptance cannot manufacture paid amount. | HOLDS BY CONSTRUCTION | Credit is the explicit native value irrecoverably destroyed at output 1, never a consumer-written scalar. |
| Neutrality (`req:lineage:neutrality`) | A record carries no value claim, redemption or price, so recording creates no beneficiary entitlement. | HOLDS BY CONSTRUCTION | The minimal-value claim token has no holder role; its incidental funding is ignored, unpreserved and distinct from record credit. |
| Non-transferability (`req:lineage:non-transferability`) | Recorded magnitude remains at its original address, so aggregation cannot reassign paid history. | HOLDS BY CONSTRUCTION | Every record retains its own transaction's address and amount; merge unions record identities rather than pooling their credit. |
| Settled monotone reading (`req:lineage:monotone-reading`) | Pinned address readings never decrease along a history, interim readings stay within the next pin, and permitted nonfinal revisions respect the last final pin. | HOLDS BY CONSUMER RULE | Non-negative L supplies the substrate; published A(previous) ⊆ A(this) ⊆ L(this), pin selection and revision policy supply the reading discipline. |
| Opaque permanent addresses (`req:lineage:opaque-addresses`) | An enduring datum without a ledger identity claim, so attribution does not depend on a role registry. | HOLDS BY CONSTRUCTION | Exactly 32 burner-written bytes remain recoverable in the original destruction transaction. |
| Non-negativity (`req:lineage:non-negativity`) | No record subtracts credit, so extending a settled set cannot reduce an address total. | HOLDS BY CONSTRUCTION | A verified explicit positive value contributes itself; a failed pair contributes zero. |
| Provenance auditability (`req:lineage:auditability`) | Every counted unit is capped by genuine same-event destruction, so copied data cannot create magnitude. | HOLDS BY CONSTRUCTION | Equality to the sibling destruction amount and an ancestry walk authenticate each counted unit, conditional on supplied complete evidence. |
| Instant costliness and append-only record (`req:lineage:instant-cost`) | Cost is paid at recording and surviving records cannot be edited or spent back into value. | HOLDS BY CONSTRUCTION | The native OP_RETURN output destroys value at BURN; later merges consume tokens, never destruction outputs. |
| Bootstrap capacity (`req:lineage:bootstrap`) | An opening from the first acquisition block states its pre-acquisition bound, so earlier magnitude cannot be silently assumed. | HOLDS BY PUBLICATION | Publish zero for the empty pre-acquisition interval and the reason the interval is empty; genesis itself contributes no destruction. |
| Unit cost (`req:lineage:unit-cost`) | A positive cost per unit in a declared denomination and terms, so magnitude has an interpretable price. | HOLDS BY PUBLICATION | Publish one native unit destroyed per unit credited and the transaction fee as a separate per-recording-event cost. |
| Operator accrual (`req:lineage:operator-accrual`) | Explain whether operation accrues destroyed magnitude to any role, so neutral accounting cannot conceal a beneficiary. | HOLDS BY PUBLICATION | Publish that merging grants no destroyed amount or record entitlement; fees go to block creators, while incidental token funding is separately disclosed. |
| Lineage (`req:lineage:lineage`) | A designated origin, unique successor by spend, ancestry dependence and exactly-once settlement, so a pin selects a determinate cumulative set. | HOLDS BY CONSTRUCTION | Genesis designation plus consensus spending determines the chain; token outpoint identity and set traversal prevent duplicate record credit. |
| Consumer-defined acceptance within the ledger bound (`req:lineage:consumer-acceptance`) | Consumers select recognized boundaries and accepted subsets without claiming unsettled records, so acceptance remains explicit and bounded. | HOLDS BY CONSUMER RULE | Each consumer publishes its carrier-provenance rule and preserves A(previous) ⊆ A(this) ⊆ L(this); acceptance equal to L removes subset selection. |

**Caveat (Construction standing is conditional)** · `cav:lineage:construction-standing`

The eight construction standings are conditional semantic claims, not target admission verdicts. The positive-value representation removes the already reviewed consensus obstruction, while its relay admission remains NEEDS MEASUREMENT under (`expt:lineage:relay-floor`); no requirement row conceals that prerequisite.

**Obligation (Publish denomination and costs separately)** · `obl:lineage:cost-publication`

The three publication standings require an actual deployment statement. Unit denomination, event fees and incidental minimum funding must be distinguished: destruction is the unit cost, fees are expenditure on recording, and token funding may later be redirected rather than destroyed. (`req:lineage:unit-cost`); (`req:lineage:operator-accrual`).

**Bound (Bootstrap zero covers only an empty interval)** · `bound:lineage:bootstrap-interval`

Bootstrap zero bounds only the empty interval before external acquisition begins. It says nothing about subsequent acquisition, reserve availability or a cap on lifetime destruction; a deployment with a nonempty earlier interval needs a different statement and evidence.

**Caveat (Operation may capture incidental funding)** · `cav:lineage:operator-benefit`

Operator accrual concerns the destroyed magnitude and the entitlements of its record. An open merge may capture incidental funding in a permitted transaction, so the publication cannot claim that operation yields no economic benefit of any kind; it states precisely that no destroyed amount is paid to an operator.

**Obligation (Consumers implement their published readings)** · `obl:lineage:consumer-readings`

The two consumer-rule standings cannot be delegated to the covenant. A consumer publishes its pins, accepted-set rule and interim publication policy; withholding an interim update until a supporting pin exists is one conservative implementation, while earlier publication needs evidence of the promised bound.

**Obligation (Publish distinct deployment assumptions)** · `obl:lineage:deployment-assumptions`

A deployment publishes settlement depth, the assumptions under which final pins are not revised, historical data availability and the applicable head-conflict policy together. Finality assumptions do not ensure retrieval, and retrieval assumptions do not settle competing histories.

**Caveat (Inclusion and resource claims need evidence)** · `cav:lineage:empirical-standing`

The permissionless-inclusion argument remains conditional on eventual inclusion and the measured conflict policy. NEEDS MEASUREMENT applies to that policy and to relay and resource conclusions, rather than turning the abstract requirement table into a claim that experiments have already succeeded. (`expt:lineage:head-conflicts`); (`expt:lineage:weight-bound`).

## 4. The traps · `sec:lineage:traps`

**Argument (Avoiding the amount accumulator)** · `arg:lineage:accumulator`

The attestation rejects a global amount accumulator because it introduces shared mutable state and serialization (`[RZ-trap:architecture:accumulator]`). This exemplar stores no aggregate, count, index or commitment: the lineage output carries only a script and incidental funding, and a reader reconstructs all record figures.

**Caveat (Settlement still has contention)** · `cav:lineage:serialized-head`

It nevertheless has a serialized settlement head, which is a real contention cost. The distinction is not that all serialization disappears, but that BURN remains independent of that head and settlement stores no mutable quantity whose correctness must be maintained by arithmetic.

**Argument (A burn flag cannot establish paid magnitude)** · `arg:lineage:burn-flag`

A token beside arbitrary data proves no paid magnitude (`[RZ-trap:architecture:burn-flag]`). Here the token alone proves none either: credit requires the verified sibling OP_RETURN output's explicit positive native amount in the same transaction.

**Justification (The token carries a path rather than trusted credit)** · `just:lineage:token-path`

The minimal-value claim token transports a path into settlement, not a trusted quantity. Counterfeit tokens and malformed pairs can be merged without producing positive credit; the reader's value anchor, rather than a decorative marker, defeats fabricated magnitude.

**Argument (Consensus value defeats data-only credit)** · `arg:lineage:data-ledger`

An indexer that totals attacker-chosen quantities from data creates free credit (`[RZ-trap:architecture:opreturn-ledger]`). This destruction output carries the address as data but carries magnitude as consensus value, and a reader reaches it through the settled token ancestry.

**Justification (Unspendability pays the recorded cost)** · `just:lineage:unspendable-anchor`

The OP_RETURN destination is deliberately unspendable, so the native amount is paid rather than merely asserted. Neither a data-only copy nor a transaction outside the selected settled ancestry can increase the reading.

**Argument (The pair supplies both event and value anchors)** · `arg:lineage:event-anchor`

The event-type and value anchors must establish both what happened and how much it paid (`[RZ-trap:ledger:event-anchor]`). The pair's common transaction, fixed positions, covenant token and native positive destruction shape establish the event; the destruction output's explicit value establishes the amount with equality.

**Invariant (Inherited records retain their event anchors)** · `inv:lineage:inherited-anchors`

MERGE does not pretend to certify a fresh burn for each consumed token. A reader preserves original pair identities and recognizes a new record only when the merge transaction itself also contains a valid pair; inherited records retain their original event anchors.

## 5. Mapping onto the pipeline · `sec:lineage:pipeline`

**Obligation (Architecture supplies a validated semantic boundary)** · `obl:lineage:architecture`

Architecture owns the typed normative manifest, finite identifiers, validation and canonical publication, as described in `packages/architecture/README.md`. Its current manifest is explicitly the attestation realization; accepting another semantic family requires examining that binding rather than assuming its identifiers are already generic.

**Specification (The second architecture vocabulary)** · `spec:lineage:architecture-vocabulary`

The new architecture description needs the two operations, the lineage-script object, destruction-pair recognition and ancestry observables. It does not need the attestation's five-root inventory, receipt classes or STATE field registry, and must not populate those registries with fictitious equivalents. (`op:lineage:burn`); (`op:lineage:merge`); (`def:lineage:lineage-output`); (`crit:lineage:valid-pair`).

**Obligation (Realization declares the semantic relations)** · `obl:lineage:realization`

Realization owns semantic facts, expressions, relations, constructibility, lifecycle and observables (`pkg:realization:contract`). Existing typed graphs and scope validation are reusable mechanisms; the new declarations and their architecture binding do not exist in the dispatch listed in (`fact:lineage:declaration-boundary`).

**Specification (BURN recognition relations)** · `spec:lineage:burn-relations`

BURN's relation families are pair shape, fixed output positions, explicit positive native destruction value, exact 32-byte data, token recognition and immutable record association. The declaration must distinguish those recognition predicates from conditions that a spending covenant actually executes. (`crit:lineage:valid-pair`); (`rule:compiler:placement`).

**Specification (MERGE emission and projection)** · `spec:lineage:merge-relations`

MERGE's emitted relation is successor-at-0 carrying the script for each consumed covenant input. Its semantic projection unions inherited record identities and recognizes any new valid pair, but the script does not compute that union, walk history or enforce a consumer's accepted set. (`op:lineage:merge`); (`ntn:lineage:settled-set`).

**Definition (Lifecycle closure for this contract)** · `def:lineage:lifecycle-closure`

Lifecycle closure here means that a recognized covenant spend continues into the same script at output 0. It does not mean that every token eventually reaches genesis, that incidental funds are conserved inside the object class, or that an arbitrary consumer must accept a settled record.

**Question (Which semantic home carries the declaration)** · `q:lineage:semantic-home`

Three homes remain possible: another declaration set inside realization, a separately selected semantic profile, or a second realization crate behind a shared compiler input boundary. The first shares infrastructure but broadens identifier assumptions; the second needs a profile that selects semantics rather than merely deployment evidence; the third makes independence visible but requires extracting a shared typed interface.

**Condition (Validated architecture remains required)** · `cond:lineage:validated-input`

The realization dependency and validated-input rules constrain all three choices (`sec:realization:dependencies`). Architecture deployment profiles described in `packages/architecture/README.md` validate evidence against an existing manifest; they cannot be relabeled as an already implemented second-contract selector.

**Obligation (Compiler analysis preserves target independence)** · `obl:lineage:compiler`

Compiler owns target-independent analysis and emits no target program (`pkg:compiler:contract`). Its graph, fact-source, constructibility, lifecycle, proof-planning, placement and coverage foundations are reusable; a validated operation plan for this contract would be new.

**Justification (BURN recognition cannot be assigned to a nonexistent input)** · `just:lineage:burn-placement`

The analysis must keep public record predicates separate from the successor script predicate. BURN has no mandatory covenant input on which to execute its pair recognition, so attributing the entire burn relation to an emitted leaf would manufacture an enforcement claim. (`rule:compiler:placement`).

**Obligation (Every relation has an honest carrier)** · `obl:lineage:relation-carriers`

An ancestry reader and evidence projection can own the record predicates while target consensus owns actual destruction semantics. Each active relation needs a reachable and honestly named carrier under (`rule:compiler:placement`), even when that carrier is outside script execution.

**Precondition (Public construction needs no covenant secret)** · `pre:lineage:public-construction`

Public constructibility is especially direct: a merge needs previous-output data, the public fixed constructor and its spend recipe, and carrier funding. No creator secret or owner signature belongs to the covenant's witness availability requirements. (`rule:compiler:constructibility`).

**Caveat (Reuse is not an existing generic entry point)** · `cav:lineage:compiler-interface`

The current compiler contract distinguishes internal analysis stages from published validated operation boundaries. The study therefore proposes reuse of analysis responsibilities, not a claim that a public generic compile-any-contract entry point already exists (`sec:compiler:outputs`).

**Obligation (Target facts remain separate from exemplar semantics)** · `obl:lineage:target-elements`

Target-elements owns opcode, encoding, capability, resource and transaction-form facts, not exemplar semantics (`pkg:target-elements:contract`). Existing reviewed families include output introspection, confidential field encodings and fee roles; the new contract needs its own form assessments against those facts.

**Specification (Separate consensus and relay assessments)** · `spec:lineage:form-assessments`

`packages/target-elements/README.md` describes `TransactionForm`, `FormAdmission`, `RelayCondition` and `FeeOutputContract`. Each proposed form needs separate consensus and relay rows, and a direct-submission condition cannot repair a consensus refusal. (`rule:target-elements:resources`); (`expt:lineage:relay-floor`); (`expt:lineage:data-output`).

**Table (Proposed transaction-form assessments)** · `tab:lineage:forms`

BURN and MERGE each need a separate consensus verdict, relay verdict and contract-specific evidence. The following proposed rows distinguish those subjects under (`pkg:target-elements:contract`), with actual admissions left to (`sec:lineage:measurements`).

| Proposed form | Consensus admission evidence | Relay admission evidence | Contract-specific obligation |
|---|---|---|---|
| BURN pair | Reviewed positive explicit spendable funding and unspendable destruction semantics, followed by exact-form confirmation. | NEEDS MEASUREMENT for the token dust floor and the 32-byte data output; account for target fee-output placement. | Output 0 recognizes the covenant token and output 1 is the native explicit destruction with exact address data. |
| MERGE carrier | Reviewed output-script introspection and a valid successor constructor, followed by exact-form execution evidence. | NEEDS MEASUREMENT for successor dust, conflict policy, witness limits and N-dependent weight. | Every covenant input checks the same output-0 script; no value, asset or other carrier relation is added. |

**Caveat (Proposed forms are not admitted variants)** · `cav:lineage:proposed-forms`

These are proposed admission rows, not existing enum variants or completed target reviews. Their measurements can support `RelayCondition` choices, including direct submission to a block producer where appropriate, while retaining the distinction between admission and eventual inclusion.

**Bound (Witness item relay limit)** · `bound:lineage:witness-item`

`TAPSCRIPT_STACK_ITEM_RELAY_LIMIT` is the reviewed 80-byte per-witness-item relay limit. It is not a data-output bound and gives no answer about the OP_RETURN address carrier; the latter needs its own policy evidence. (`pkg:target-elements:contract`); (`expt:lineage:data-output`).

**Fact (Committed fields do not expose numeric values)** · `fact:lineage:committed-fields`

`ConfidentialFieldEncoding` separates explicit and committed asset and value representations. The reviewed output-value opcode returns the encoded field, including committed alternatives; inspection of a commitment does not reveal the numeric amount, so direct arithmetic comparison requires explicit value or an independently supported opening mechanism. (`tab:elements-ref:ct-encodings`); (`tab:elements-ref:ct-introspection`); (`tab:elements-ref:ct-opening-gap`).

**Specification (Explicit destruction with optional confidential carriers)** · `spec:lineage:mixed-carrier`

This contract chooses explicit destruction value and an identifiable native asset for record verification. It needs no opening of confidential destruction value, while unrelated carrier outputs may be confidential if the exact mixed form is admitted. (`expt:lineage:mixed-representations`).

**Obligation (Tapscript lowers authenticated recurrence)** · `obl:lineage:tapscript`

Tapscript owns typed lowering, stack validation, proof patterns, concrete placement, relocations and resource formulas (`pkg:tapscript:contract`). It needs a new successor-script equality pattern and a public constructor recipe for this exemplar, rather than the attestation's amount-preserving compact pattern with predicates removed informally.

**Fact (Reviewed output introspection vocabulary)** · `fact:lineage:introspection`

The reviewed primitives include `InspectOutputScriptPubKey`, `InspectOutputValue` and `InspectOutputAsset` in (`tab:elements-ref:opcodes`). MERGE needs only script inspection at output 0 and equality to the authenticated current covenant program; its leaf does not inspect output value or asset. (`tab:elements-ref:opcodes`); (`sec:elements-ref:introspection`).

**Obligation (Compare the complete script representation)** · `obl:lineage:script-representation`

Script introspection returns the reviewed witness-program or digest representation, not an assumed arbitrary script byte array. The pattern must compare the full relevant representation, including its type or version, and reject a superficially similar successor from a different constructor. (`sec:elements-ref:introspection`); (`rule:tapscript:representation`).

**Obligation (Constructor paths cannot bypass recurrence)** · `obl:lineage:constructor-paths`

The fixed covenant is a tapscript leaf in a public taproot constructor. The constructor must provide no alternative spend path that bypasses the successor condition, including no usable key path, and its witness must authenticate the actual committed leaf and control path. (`rule:tapscript:constructors`); (`rule:linker:constructors`).

**Counterexample (An unauthenticated comparison target permits substitution)** · `cex:lineage:script-substitution`

Comparing to a caller-supplied unauthenticated script would not establish recurrence. Comparing the successor to the authenticated current input program is a possible reference strategy; a recursive literal commitment requires a supported constructor/reference solution rather than an assumed fixed point. The script must compare against an authenticated program; an attacker-supplied replacement program could otherwise satisfy equality while changing the successor covenant (`obl:lineage:constructor-paths`).

**Caveat (Constructor closure realizes the existing condition)** · `cav:lineage:constructor-work`

That constructor choice is technical work still required, not an extra semantic check on the carrier. Its purpose is to make the one successor condition refer to the actual covenant, while the final stack and resource schedule remain valid under the target contract.

**Obligation (Linker closes symbols and control recipes)** · `obl:lineage:linker`

Linker owns symbols, reference graphs, cycle strategies, relocations, deterministic taptrees, control recipes and carrier preservation (`pkg:linker:contract`). Existing candidate bundles demonstrate those responsibilities for attestation operations; a lineage bundle and its exact recurrence strategy would be new.

**Obligation (Creation and recognition share one constructor)** · `obl:lineage:constructor-identity`

The linker must establish that token creation and successor recognition use one constructor and that every relocation resolves exactly once. A trivial semantic relation still requires reference closure, candidate status and a control recipe; none of these belongs in an ancestry reader's ad hoc script literal. (`rule:linker:relocations`); (`rule:linker:references`).

**Obligation (Transaction builds final layouts and witnesses)** · `obl:lineage:transaction`

Transaction owns final layouts, requests, constructor instantiation, assembly, witnesses and valid calibration fixtures (`pkg:transaction:contract`). A BURN request needs an address, amount and funding view; a MERGE request needs its covenant inputs and an otherwise permitted carrier construction.

**Specification (Open BURN and MERGE layouts)** · `spec:lineage:layouts`

The BURN ABI fixes outputs 0 and 1 and permits other target-valid outputs. The MERGE ABI fixes only the covenant successor at output 0; input ordering chosen by a builder must not become an undeclared protocol restriction on which input carries the lineage predecessor. (`op:lineage:burn`); (`op:lineage:merge`); (`rule:transaction:ordering`).

**Definition (Distinct witness roles)** · `def:lineage:witness-roles`

Funding inputs can require ordinary signatures, and a carrier can carry consumer authorization. Neither is covenant authorization; the ABI should expose these separate witness roles without importing the attestation's every-receipt-owner signing protocol. (`rule:transaction:signing`).

**Caveat (Reusable assembly imports no STATE or sponsor law)** · `cav:lineage:assembly-reuse`

The STATE metadata codec, STATE constructor generations, maturity signing states and exact sponsor-asset isolation relations are not needed here. Generic constructor recipes, transaction finalization, witness assembly and funding remain useful, but their reuse must not close unrelated carrier outputs by accident. (`rule:transaction:constructors`); (`rule:transaction:stages`).

**Obligation (Vectors separate five acceptance questions)** · `obl:lineage:vectors`

Vectors owns semantic fixtures, focused mutations, target execution, accepted projections, relation coverage and resource evidence (`pkg:vectors:contract`). New evidence must separate successor enforcement, pair recognition, ancestry-set completeness, consumer acceptance and target admission; one successful merge cannot discharge all five.

**Specification (Positive cases and focused negatives)** · `spec:lineage:vector-cases`

Positive cases include one token, many tokens, a side-pile sweep, combined BURN/MERGE and multiple successive points in one block. Negative cases include a wrong successor script, missing output 0, malformed pair, wrong asset, committed destruction value, duplicate traversal and missing ancestry; only the script faults should be expected to fail at covenant execution. (`crit:lineage:valid-pair`); (`op:lineage:merge`); (`crit:lineage:ancestry-completeness`).

**Obligation (Independent reconstruction binds resource subjects)** · `obl:lineage:independent-reader`

An independent reader should reconstruct record sets from public transactions without relying on the builder's claimed total. Resource evidence binds the exact bundle, ABI, target and input bound, while infrastructure failures remain distinct from target refusals under (`rule:vectors:execution`).

**Obligation (Labels resolve citations without conferring semantics)** · `obl:lineage:labels`

Labels owns citation resolution and source census, not new contract semantics (`pkg:labels:contract`). This study mints planning labels in area lineage, classified by (`[ADR020-judg:kinds:classification]`) and validated by (`[ADR020-judg:kinds:head-validation]`); admitted semantic owners still require their own established registration and cannot acquire authority from the study.

**Obligation (Documentation retains its existing registration)** · `obl:lineage:documentation`

Documentation owns this explanatory study and its registration under (`rule:drafts:authority`). Its initial registration increased the indexed drafts from ten to eleven and the directory census entries from eleven to twelve; rewriting the same registered file adds neither an index entry nor a census entry (`[ADR014-rule:build:census]`). The existing registrations remain in `plans/drafts/README.md` and `plans/drafts/meson.build`.

**Condition (Public evidence and identity boundaries)** · `cond:lineage:evidence-boundaries`

Public disposable fixtures remain within (`[ADR015-rule:security:public-data]`). Identity allocation follows (`[ADR021-sec:identity:procedure]`), and an actual external consumer of a record document would activate the interchange boundary under (`[ADR022-rule:interchange:no-implementation]`); this study creates neither an identity nor an interchange namespace.

## 6. Measurements and evidence · `sec:lineage:measurements`

**Gate (Evidence before admission claims)** · `gate:lineage:measurement-evidence`

No Elements experiment is performed for this study. The following obligations must be established on the pinned Elements toolchain before an execution guide claims admitted transaction forms, a calibrated bound or a deployment-specific inclusion argument.

**Experiment (Relay funding floor)** · `expt:lineage:relay-floor`

Measure the dust threshold and default relay admission of the exact minimal-value claim token and successor constructor, including boundary values immediately below and at the admitted minimum; this decides construction funding and whether a special relay route is needed.

**Fact (Consensus positivity is already reviewed)** · `fact:lineage:reviewed-positivity`

The consensus side is already reviewed: `ExplicitZeroValueRule::SpendableRefused` and its `VerifyAmounts` source note in `packages/target-elements/src/transaction_form.rs` refuse an explicit zero-valued spendable output. Experiment (`expt:lineage:relay-floor`) concerns the positive relay floor alone and must not describe that consensus fact as unresolved. (`fact:lineage:positive-funding`).

**Experiment (Destruction data-output admission)** · `expt:lineage:data-output`

Measure an explicit positive native OP_RETURN output carrying the exact 32-byte datum, including its encoded script overhead and data-output policy limit; this decides whether the proposed BURN pair relays as built and what carrier constraints a deployment must publish.

**Obligation (Distinguish payload refusal from admission refusal)** · `obl:lineage:data-output-evidence`

The report must keep data-output limits separate from the reviewed 80-byte tapscript witness-item limit. Cases should distinguish malformed payload rejection by the reader from transaction rejection by consensus or relay; a transaction can be admitted yet contribute no record. (`bound:lineage:witness-item`).

**Experiment (Conflicting head spends)** · `expt:lineage:head-conflicts`

Establish whether the deployment's mempool admits fee-based replacements, the relevant replacement conditions and fee increments, or instead applies first-seen behavior; this decides which per-head censorship argument the deployment can make.

**Caveat (Observed replacement is not eventual inclusion)** · `cav:lineage:replacement-evidence`

Record the exact node policy and configuration along with results for successive competing spends. Replacement admission does not establish eventual block inclusion; that remains a named assumption, and no experiment should report a universal censorship guarantee from a finite race.

**Experiment (Mixed representations)** · `expt:lineage:mixed-representations`

Establish admission of a transaction containing explicit native destruction and explicit token funding alongside confidential unrelated outputs; this decides how broadly the merge and burn forms can share a carrier without restricting the consumer's other outputs.

**Obligation (Bind mixed-form fixtures to authenticated encodings)** · `obl:lineage:mixed-evidence`

The fixtures must bind encodings, balance proofs and fee outputs to the reviewed confidential facts. They must not substitute a disclosed scalar for authenticated destruction value or claim a general confidential-opening primitive that the target review leaves unsupported. (`tab:elements-ref:ct-conservation`); (`tab:elements-ref:ct-opening-gap`).

**Experiment (The one-check implementation)** · `expt:lineage:one-check`

Confirm the script-introspection and equality schedule, authenticated current-program reference, constructor control path, stack behavior and cost of the successor-at-0 leaf; this decides whether the semantic recurrence has a faithful admitted target realization.

**Specification (Recurrence execution dimensions and controls)** · `spec:lineage:recurrence-evidence`

Measure exact program bytes, witness items, control depth, stack peaks and validation cost, with wrong-script and wrong-version successors as negative cases. The primitive review is reusable evidence, but it is not a substitute for executing this composed pattern and checking its accepted projection. (`rule:tapscript:resources`); (`rule:linker:resources`).

**Experiment (Transaction weight and N)** · `expt:lineage:weight-bound`

Measure BURN and MERGE carriers with N covenant inputs across the proposed range, including fee funding, optional confidential outputs and worst-case witnesses; this decides the construction bound, fee estimates and practical side-pile sweep size.

**Obligation (Retain prediction mismatches and absent dimensions)** · `obl:lineage:resource-evidence`

Compare predictions from backend and linker resource formulas against observed transaction weight and execution resources. Publish absent dimensions as absent, and make mismatches findings rather than silently adjusting the predicted figures to the observation. (`rule:tapscript:resources`); (`rule:linker:resources`).

**Specification (The evidence package)** · `spec:lineage:evidence-package`

The evidence package would carry public semantic fixtures, deterministic construction inputs, exact candidate bundle and ABI bindings, positive vectors, focused mutations, target admission transcripts and independently reconstructed accepted record sets. Measurement reports retain policy configuration and toolchain provenance without converting source identifiers into protocol identity. (`[ADR021-rule:identity:admission-order]`); (`rule:vectors:execution`).

**Criterion (Evidence families answer distinct questions)** · `crit:lineage:evidence-families`

Distinct reports should answer destruction-pair recognition, successor enforcement, ancestry completeness, replacement-history binding, consumer reading discipline and resources. A missing ancestor is an incomplete proof; a failed child process is infrastructure failure; a policy rejection is not automatically a consensus rejection.

**Caveat (Finite evidence cannot establish unconditional availability)** · `cav:lineage:evidence-limits`

Historical availability and finality remain published assumptions with explicit limits. The evidence can demonstrate reconstruction from supplied transactions and alternative-history fixtures, but cannot turn those demonstrations into unconditional archival availability or irreversible settlement. (`assum:lineage:availability`); (`assum:lineage:finality`).

## 7. Against the attestation exemplar · `sec:lineage:comparison`

**Table (Comparison with the attestation exemplar)** · `tab:lineage:comparison`

The twelve dimensions below compare different contract laws and their evidence owners. Existing attestation obligations are governed by (`pkg:realization:contract`), (`pkg:vectors:contract`) and the compact laws discussed immediately below; this exemplar is governed by (`sec:lineage:contract`).

| Dimension | Attestation exemplar | Destruction-lineage exemplar |
|---|---|---|
| Operations | Typed compact ASH, live transfer and maturity announcement exist; Phase 7 specifies burn, compact maintenance and clear work. | BURN pair recognition and MERGE successor recurrence; neither declaration is implemented. (`fact:lineage:declaration-boundary`); (`phase:roadmap:burn-clear`). |
| Objects | Live receipts, public ASH and root objects have distinct semantic roles and constructors. | Destruction output, minimal-value claim token and same-script lineage output, with designated genesis. (`[RZ-sec:architecture:one-asset]`); (`def:lineage:destruction-output`); (`def:lineage:genesis`). |
| STATE | Canonical metadata, constructor generations and root transitions support maturity and planned clear. | No STATE metadata or mutable quantity; lineage identity is an outpoint. (`[RZ-sec:kernel:certificate]`); (`inv:lineage:immutable-records`). |
| Arithmetic in script | Compact preserves an exact aggregate; clear requires a bounded minimum and STATE decrement. | No magnitude arithmetic in the covenant; a reader sums verified immutable records. (`rule:guide15:compact-relation`); (`[RZ-sec:operations:clear]`); (`alg:lineage:reader`). |
| Signing and authorization | Live operations require owner authorization, maturity uses operator authorization, and public maintenance is permissionless. | No covenant signature; ordinary funding and optional consumer signatures belong to the carrier. (`rule:transaction:signing`); (`def:lineage:witness-roles`). |
| Asset | A closed issued receipt asset participates in class recognition and conservation. | Native asset is destroyed directly; no issued asset and no covenant asset check. (`[RZ-sec:architecture:one-asset]`); (`just:lineage:native-magnitude`). |
| Value held on the ledger | ASH holds consensus value until clear destroys the relevant amount. | Destruction is immediate; only incidental, unpreserved funding remains on lineage-script outputs. (`[RZ-sec:architecture:burn-ledger]`); (`def:lineage:sweep-bounty`). |
| Event model | Burn, compact silence and clear have distinct authenticated projections. | A verified pair anchors destruction; a lineage point settles ancestry; a consumer separately recognizes boundaries. (`[RZ-sec:operations:burn]`); (`rule:guide15:compact-projection`); (`[RZ-sec:operations:clear]`); (`prin:lineage:two-layers`). |
| Fault classes | Ownership, object classes, aggregate arithmetic, STATE transitions, constructor continuity and sponsor isolation. | Pair recognition, recurrence authentication, duplicate or incomplete ancestry, consumer bounds and admission policy. (`pkg:realization:contract`); (`pkg:vectors:contract`); (`crit:lineage:evidence-families`). |
| Evidence families | Safety, event, continuity, recovery, accounting and resource reports cover different attestation obligations. | Pair, successor, ancestry, replacement-history, consumer-rule and resource evidence cover different obligations. (`pkg:vectors:contract`); (`crit:lineage:evidence-families`). |
| What a covenant flaw could release | Depending on the affected object, spendable protocol quantities and protections around receipt or root transitions. | Incidental funding and settlement continuity; it cannot recover value already destroyed by OP_RETURN. (`[RZ-sec:architecture:one-asset]`); (`just:lineage:immediate-destruction`). |
| What a reader needs to hold | Authenticated event and relevant root-history evidence for the requested attestation reading. | A pinned lineage point, complete supplied ancestry with inclusion proofs, genesis designation and the consumer's published rule. (`[RZ-sec:ledger:model]`); (`[RZ-sec:kernel:replay]`); (`pre:lineage:ancestry-evidence`). |

**Observation (Similar successor placement carries different arithmetic)** · `obs:lineage:compact-arithmetic`

Guide 15's compact relation preserves the exact sum of authenticated public ASH amounts (`rule:guide15:compact-relation`). MERGE shares permissionless construction and successor placement, but does not preserve incidental value and must not inherit the ASH aggregate equation.

**Observation (MERGE has no compact event-silence law)** · `obs:lineage:compact-silence`

Compact's silence requires no burn record or event, no clear event, no STATE, no issuance and no destruction (`rule:guide15:compact-projection`). MERGE deliberately imposes no comparable carrier closure: a transaction may carry an unrelated act or even a new valid destruction pair without violating the one-check covenant.

**Observation (MERGE closes no asset domain)** · `obs:lineage:compact-closure`

Compact also closes every output carrying the protocol asset to its one exact successor (`rule:guide15:compact-output-closure`). The lineage covenant does not close an asset domain, so copying that closure into the new declaration would change the contract rather than merely reuse an implementation.

**Observation (The recurrence needs no semantic coordinator)** · `obs:lineage:compact-coordinator`

The Guide 15 placement plan puts ASH recognition, aggregate preservation, output closure, sponsor isolation and event silence on the compact coordinator (`rule:guide15:carrier-placement`). Here each covenant input checks the common successor and reports establish record history; there is no semantic coordinator with a separate authorization role.

**Observation (Positional ABI precedent does not import amount laws)** · `obs:lineage:compact-layout`

Guide 15's compact layout places the aggregate ASH successor at output 0 with optional sponsor change and the required target fee (`rule:guide15:compact-layout`). That positional similarity is useful ABI precedent, while the closed amount and sponsor relations remain specific to the first exemplar.

**Criterion (Reuse must preserve both required and absent laws)** · `crit:lineage:faithful-reuse`

The comparison tests whether reusable mechanisms can carry different laws without importing the first exemplar's laws as universal preconditions. A successful second compilation must preserve the absence of an aggregate equation and consumer authorization just as faithfully as the first preserves required arithmetic and signatures.

**Observation (A smaller script still needs exact assurance boundaries)** · `obs:lineage:assurance-boundaries`

The smaller covenant also sharpens assurance boundaries: less on-script logic does not eliminate constructor closure, truthful relation placement, reader completeness or target policy evidence. The compiler is tested by carrying each obligation to its proper owner, not by making every obligation execute in a leaf.

## 8. Findings for this tree · `sec:lineage:findings`

**Fact (The second declaration family is absent)** · `fact:lineage:finding-declarations`

`packages/realization/src/declarations/mod.rs` exposes only compact ASH, live transfer and maturity announcement; a new exemplar needs actual declarations and cannot be activated by changing a transaction request alone. (`fact:lineage:declaration-boundary`).

**Observation (Architecture generality remains to be demonstrated)** · `obs:lineage:finding-architecture`

`packages/architecture/README.md` identifies an attestation-specific finite manifest, while (`pkg:realization:contract`) consumes its validated architecture and identifiers; another contract tests this input boundary before it tests target bytes.

**Fact (Positive explicit token funding is required)** · `fact:lineage:finding-funding`

The `SpendableRefused` term in `packages/target-elements/src/transaction_form.rs` rules out an explicit zero-valued spendable token, leaving the positive relay minimum as a separate measurement rather than a semantic quantity. (`fact:lineage:positive-funding`); (`expt:lineage:relay-floor`).

**Observation (Primitive vocabulary does not establish a composed form)** · `obs:lineage:finding-target`

(`tab:elements-ref:opcodes`) names output script, value and asset inspection, and (`rule:target-elements:resources`) separates consensus and relay admission; the proposed forms still need composed evidence and policy measurements. (`sec:elements-ref:introspection`); (`expt:lineage:one-check`).

**Observation (Compact maintenance has different stronger laws)** · `obs:lineage:finding-compact`

Guide 15 requires aggregate preservation, event silence and output closure, while this contract requires only successor-script continuation; reusing the compact implementation unchanged would add unwanted semantics (`rule:guide15:compact-relation`).

**Observation (Faithful compilation includes reader-owned relations)** · `obs:lineage:finding-placement`

BURN can have no covenant input, yet pair verification and ancestry completeness remain mandatory; compiler placement and vector evidence contracts provide the right responsibility split without pretending those checks run on every burn (`rule:compiler:placement`).

**Observation (One check still needs constructor closure)** · `obs:lineage:finding-constructor`

Tapscript lowering and linker control recipes must authenticate the recurrent script and exclude bypass paths; existing package responsibilities supply the location of that work but not a completed constructor for this exemplar (`pkg:linker:contract`).

**Observation (Admission and execution evidence are separate)** · `obs:lineage:finding-admission`

The study distinguishes roadmap admission from measured execution: its admission alternatives do not themselves establish a card or admitted forms (`q:lineage:admission`). Phase 7 is Active (`phase:roadmap:burn-clear`), and the charter separately admits independence evidence inside that phase and places it in Wave 1 (`sec:phase7:change-statement`); (`task:guide15-exec:wave1`). The study supplies no target measurements and does not discharge (`gate:guide15-exec:wave1`).

## 9. Decisions left open · `sec:lineage:decisions`

**Question (Realization home)** · `q:lineage:realization-home`

A second declaration set shares existing infrastructure but broadens attestation-specific identifier assumptions; a semantic profile requires an explicit selector beyond today's deployment-evidence profile; a second crate makes separation clear but costs a shared input interface and additional maintenance. The constraints are the architecture boundary and (`pkg:realization:contract`), and no option is selected here. The charter leaves this selection to bite 1.1 within (`task:guide15-exec:wave1`), whose completed contract must meet (`gate:guide15-exec:wave1`).

**Case (Declaration set or semantic profile or separate crate)** · `case:lineage:decision-realization-home`

A declaration set shares infrastructure while broadening identifier assumptions; a semantic profile must select semantics beyond deployment evidence; a separate crate needs a maintained shared compiler interface. Each alternative remains subject to the same validated architecture boundary (`q:lineage:realization-home`).

**Question (Incidental funding and relay floor)** · `q:lineage:funding-floor`

The contract takes positive native funding with no semantic amount meaning; explicit zero-valued spendable outputs are already refused by the target review. Choosing the measured default-relay floor favors ordinary propagation, while a smaller positive value with a separately admitted route adds deployment dependence; experiment (`expt:lineage:relay-floor`) decides feasibility and the statement must disclose the chosen route. (`fact:lineage:positive-funding`); (`expt:lineage:relay-floor`).

**Case (Default relay or a separately admitted positive route)** · `case:lineage:decision-funding-floor`

Default-relay funding favors ordinary propagation; a smaller positive value requires a separately admitted route and disclosed deployment dependence. Neither alternative reopens the reviewed zero-value refusal (`expt:lineage:relay-floor`).

**Question (Whether any value-preservation rule is wanted)** · `q:lineage:value-preservation`

None is the contract's default: funding may move to fees or elsewhere. A greater-or-equal successor rule would protect a stated funding floor or sum while allowing donations to remain, and equality would preserve an exact chosen amount or aggregate while constraining fee use and donations; either requires defining the comparison base, adding introspection and changing the one-check contract, without improving record magnitude authentication. The charter separately adopts the no-preservation default for Wave 1 (`task:guide15-exec:wave1`); the alternatives here remain descriptions of contract changes.

**Case (No preservation or an added inequality or equality)** · `case:lineage:decision-value-preservation`

No preservation keeps the one-check contract. An inequality protects its declared comparison base while permitting excess donations; equality preserves an exact base while limiting fee use and donations. Either addition changes the contract and cannot improve destruction-value authentication (`q:lineage:value-preservation`).

**Question (The input bound N)** · `q:lineage:input-bound`

A small constructor bound simplifies worst-case witnesses and predictable fees but needs more sweeps; a larger bound improves aggregation at the cost of weight and policy pressure. The bound may be an ABI construction limit rather than a new covenant count check, and experiment (`expt:lineage:weight-bound`) must distinguish those choices. (`expt:lineage:weight-bound`).

**Case (Small or large construction bound)** · `case:lineage:decision-input-bound`

A smaller bound trades more sweeps for predictable witnesses and fees; a larger bound trades fewer sweeps for weight and policy pressure. An ABI bound need not introduce a covenant count check (`expt:lineage:weight-bound`).

**Question (Genesis designation)** · `q:lineage:genesis-designation`

A public deployment manifest can name one outpoint directly, while a published deterministic deployment procedure can explain its derivation; the former needs trustworthy distribution of the designation and the latter needs reproducible construction and chain binding. Neither allows longest-history selection or an arbitrary same-script output to replace the designated origin.

**Case (Direct designation or reproducible deployment)** · `case:lineage:decision-genesis-designation`

A directly published outpoint needs trustworthy distribution of its designation; a deterministic deployment procedure needs reproducible construction and chain binding. Neither admits selection by longest history (`def:lineage:genesis`).

**Question (Roadmap admission and first gate)** · `q:lineage:admission`

Keeping the study as a draft costs no new execution commitment but leaves independence untested; admitting a card beside Phase 7 costs capacity and requires an explicit scope and dependency decision. A possible first gate would establish a second typed declaration boundary, authenticated recurrence constructor and separately assessed BURN/MERGE forms, with the six measurement obligations disposed before any execution-guide success claim. The charter separately admits the second contract inside Phase 7 and assigns its execution to Wave 1 (`sec:phase7:change-statement`); (`task:guide15-exec:wave1`).

**Case (Remain planning input or admit execution work)** · `case:lineage:decision-admission`

Retaining a draft makes no execution commitment but leaves independence untested; admitting execution consumes capacity and requires explicit dependencies. The charter makes that separate admission inside Phase 7, with final-form evidence required at (`gate:guide15-exec:wave1`).

**Question (Recurrence constructor strategy)** · `q:lineage:recurrence-strategy`

Authenticating the current input program for successor comparison avoids an assumed literal fixed point but requires a precise inspected-program and control-path proof; a linker reference strategy must instead demonstrate supported closure and its cost. Either must exclude alternative spend paths and retain the same semantic successor rule under (`rule:linker:references`).

**Case (Authenticated current program or supported linker reference)** · `case:lineage:decision-recurrence-strategy`

Current-program authentication needs an exact introspection and control-path proof; a linker-reference construction needs supported closure and measured cost. Both must exclude bypass paths and preserve the same successor condition (`obl:lineage:constructor-paths`).

**Question (Consumer publication policy)** · `q:lineage:consumer-publication`

Acceptance equal to L removes discretionary subset selection but may not meet a consumer's purpose; a published subset rule adds recognition and monotonicity obligations. Publishing only at pins simplifies the interim bound, while earlier readings need additional evidence and explicit downward-revision behavior under the chosen depth and availability assumptions.

**Case (All settled records or a subset and pins-only or early publication)** · `case:lineage:decision-consumer-publication`

Acceptance equal to L removes subset discretion; a subset rule must publish recognition and monotonicity. Pins-only publication simplifies the interim bound, while early readings need evidence and downward-revision behavior (`bound:lineage:publication-revisions`).

**Question (Evidence and external consumption)** · `q:lineage:evidence-consumption`

Internal fixture reports can establish translation-validation boundaries without an interchange identity; an external ancestry or record consumer requires an explicit format, availability contract and the ADR-022 process. The latter improves independent use but adds a maintained compatibility obligation, and neither choice turns a cached commitment into settlement authority. (`[ADR022-rule:interchange:no-implementation]`).

**Case (Internal reports or an external format contract)** · `case:lineage:decision-evidence-consumption`

Internal fixtures establish validation boundaries without an interchange identity; an actual external consumer adds a maintained format, compatibility and availability obligation. Neither makes a cached commitment authoritative (`cav:lineage:cache-authority`).
