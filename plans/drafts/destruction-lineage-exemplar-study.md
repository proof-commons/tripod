# Study: A second exemplar, public destruction with a settlement lineage

Status: a design study over the tree at the Guide 15 admission study above the clippy placement fix and 0.7.0-dev; it states a second exemplar's contract and requirement set on their merits, maps the contract onto the compiler pipeline and the reviewed Elements target, lists the measurements it needs, compares it with the attestation exemplar, decides nothing for the roadmap, and ends with the decisions it leaves open.

## 0. In one breath

Public destruction writes its magnitude in native consensus value and its destination as an opaque datum. A minimal-value claim token links that destruction into a settlement lineage whose covenant checks only the successor script. A reader reconstructs settled records from ancestry, while a consumer publishes which settled records it recognizes. This second exemplar tests whether the compiler pipeline can express a contract independent of the attestation's objects and arithmetic. The reviewed Elements target requires positive value on spendable explicit outputs; relay floors and execution costs remain measurement obligations.

## 1. Why a second exemplar

The thesis in `README.md` is that a compiler is proved by what it compiles: the attestation is the exemplar carried beside the compiler, and the compiler is the product. The same ordering governs `plans/roadmap.md` and `plans/backlog.md`; a second contract must test that thesis without silently becoming a roadmap commitment.

The first exemplar demonstrates depth within one semantic family. A second exemplar supplies the first test of whether the pipeline's boundaries survive a different family: typed declaration, relation analysis, target emission, linking, transaction construction and translation-validation evidence must each accept the new contract on its own terms.

Exemplar independence means that a declaration can describe different objects and operations while preserving the responsibilities of those stages. It does not mean reusing attestation operation identifiers for unrelated meanings, bypassing analysis with handwritten bytes, or declaring success because a target script executes once.

The present realization dispatches compact ASH, live-receipt transfer and maturity announcement; other operations receive `UnsupportedOperationDeclaration` in `packages/realization/src/declarations/mod.rs`. In particular, neither the attestation's burn and clear nor this exemplar's BURN and MERGE are already complete declarations.

One covenant with one semantic check is a sharp test because incidental complexity cannot explain an inability to express it. If declaration or construction requires STATE, receipt ownership or a conserved issued asset, that requirement exposes an attestation-specific seam rather than an intrinsic need of compilation.

The one check does not make the entire correctness argument one instruction. Script recognition, constructor authentication, target encoding, ancestry traversal and evidence boundaries still need exact definitions and tests; their owners should remain visible rather than disappear into an informal simplicity claim.

This exemplar deliberately omits receipt issuance, redemption, reserve accounting, maturity, STATE arithmetic, consumer eligibility and consumer timing. It introduces no third operation and no quantity accumulator, and it does not ask Phase 7 to replace its own burn-and-clear lifecycle.

Phase 7 is Active, with Guide 15 admission still its first gate (`phase:roadmap:burn-clear`). This study is independent planning input under (`rule:drafts:authority`), and leaves whether to admit another card beside that phase to an explicit decision.

## 2. The contract

### Objects and incidental value

A destruction output is an OP_RETURN output of the native asset with an explicit value and exactly one opaque 32-byte address in its data. A valid record requires a positive destruction value; no quantity written in the data substitutes for that consensus amount.

A minimal-value claim token is an output carrying one fixed covenant script, created at output index 0 in the same transaction as its sibling destruction output at index 1. It carries positive native value at or above the admitted minimum, but its own value has no meaning as record credit.

A lineage output carries that same covenant script. The deployment designates one genesis outpoint, created without a destruction beside it; genesis establishes the lineage origin and contributes no record.

Positive incidental funding is required for the explicit spendable representation. The reviewed `ExplicitZeroValueRule::SpendableRefused` term and its source note in `packages/target-elements/src/transaction_form.rs` state that the target amount check refuses an explicit zero-valued spendable output; this is a reviewed consensus fact, not an unanswered dust question.

The minimum suitable for default relay remains a deployment measurement. An admitted positive minimum funds each constructed token and lineage output; it is not a magnitude unit and does not acquire significance because a relay policy chooses its threshold.

The covenant does not inspect or preserve value or asset. The positive-native description states the intended construction, while the script recognizes only its own program; a reader must not infer a native-asset or value guarantee from recognition of that program alone.

Consumed tokens' incidental values may flow into the successor, transaction fees or other outputs. A donation may sit on a lineage output, and a later permitted spend may redirect it; no record reader counts that donation and no covenant rule preserves it.

Neutrality concerns the record's credit and entitlements. The token confers no redemption claim, ownership role or price on its record, even though the spendable container necessarily has incidental funding that an unrestricted merge can consume.

Destruction uses OP_RETURN because the amount must become irrecoverable at the recording event. A locked covenant output would instead retain a spendable amount and make the destruction claim depend on every future exit; irreversible output semantics separate paid magnitude from the lineage's incidental funding.

No issued asset is needed: native value supplies the exogenous magnitude directly. An issued unit would introduce issuance authority and authenticity questions that the contract does not need to answer to record a destruction.

The address is exactly 32 bytes, written by the burner. It makes no identity claim, is bound to no key or role by the ledger, and remains the same datum when the record enters any later settlement point.

### Exactly two operations

BURN is any transaction whose output 0 is a minimal-value claim token and whose output 1 is a destruction output. Anyone can build it; apart from the pair's defining shape, the exemplar constrains no input, other output, authorization or carrier purpose.

Ordinary target validity still applies, including funding, authorization of unrelated inputs and the target fee-output rules. Those requirements do not create a special burner role or a third exemplar operation.

Fixed indices eliminate a search for the association: the token's transaction and sibling index identify the claimed destruction uniquely. They also make malformed pairs fail closed without interpreting an arbitrary list of payloads or selecting a favorable output.

MERGE spends one or more outputs carrying the covenant script and produces output 0 carrying the same script. Each covenant spend checks exactly that successor-at-0 condition, with no signature, value comparison, asset comparison or constraint on any other input or output.

The common successor lets many inputs agree on one continuation without adding a coordinator entitlement. A merge may ride inside any otherwise valid carrier transaction; the covenant has no knowledge of the carrier's consumer or purpose.

A transaction can satisfy both operation descriptions: it can consume lineage-script outputs while producing a new valid token/destruction pair at indices 0 and 1. The descriptions are predicates, not disjoint transaction tags; the reader must not lose either the inherited records or the newly anchored record.

One check is deliberate. Adding a value sum would preserve incidental funding rather than record magnitude, and adding consumer authorization would turn an open settlement path into a controlled service; neither follows from the primitive's requirement set.

### Settlement lineage and side piles

The settlement lineage is the sequence of merges descending from the designated genesis outpoint. Its k-th spend defines lineage point k: its depth is the index, its successor outpoint is its identity, and the spent lineage-head input identifies its predecessor.

The spend relation gives at most one successor to any particular head in a selected valid history. It places no limit on how many successive heads can be spent within one block, and no lineage index is stored in an output.

A merge that does not spend the designated head produces a side pile with the same script but without the genesis lineage's authority as a settlement boundary. Its local history may be shorter; history length alone never designates the canonical lineage.

A later merge can consume a side pile together with the head. The pile's entire record ancestry then enters the settlement lineage, so a burner can aggregate its own records without first winning access to the current head.

Let L(p) be the set of records settled at lineage point p. These are the valid claim-token records in the point's ancestry, including a valid pair created by its transaction when BURN and MERGE coincide; earlier sets are subsets of later sets along the same lineage.

A record is identified by its token outpoint, not by address or amount. Two separate destructions at the same address are distinct records; encountering the same ancestral transaction through multiple inputs never creates another copy of one record.

For each candidate pair, a reader verifies one transaction, indices 0 and 1, the covenant token, and an OP_RETURN sibling with explicit positive native value and exactly 32 bytes of address data. The contribution is that sibling's value at that address if the pair verifies, and zero otherwise.

The token's native funding is never the amount credited. A fake token, malformed sibling, unrelated data output, wrong asset or committed destruction value cannot create credit merely by joining a merge.

The lineage settles each record once as a set member. Spending a token once prevents independent token reuse, and set identity prevents double counting when historical paths share transactions; already settled ancestry remains present without being credited again.

Settlement carries no total, count, index or aggregate commitment on the ledger. Every figure is computed by a reader from ancestry, and merging never rewrites an address or transfers one record's amount into another record.

### Ledger settlement and consumer recognition

The ledger settles an atomic ancestry-defined set at each point. Its sole inclusion rule for a consumer is a bound: a consumer counting a point may count only records that this point has settled.

The consumer recognizes which points are its boundaries and which settled records it accepts. Eligibility, completeness relative to its own purpose, timing, epochs and acceptance rules belong to that consumer and are not conditions of a valid merge.

A consumer publishes its recognition rule through the provenance it requires of the carrier transaction: a data output, the carrier's act, a role's signature or another declared criterion. None of these optional mechanisms is a new ledger operation.

For successive recognized boundaries, accepted record sets obey A(previous) ⊆ A(this) ⊆ L(this). Since valid contributions are non-negative and immutable, this subset rule gives nondecreasing address totals along the selected history.

A consumer taking acceptance equal to the settled set is the instance in which no party selects which settled records count. Other consumers may select subsets, but must publish that choice and preserve the same inclusion bound.

A commitment carried by a transaction is at most a cache for the reader. It neither adds a record to ancestry nor chooses acceptance; if its bytes disagree with verified ancestry, the cache is wrong.

### Contention and censorship assumptions

Each head is contended, but anyone can build its next merge. To exclude one available record indefinitely, an excluder must win every successive head race before any admitted merge includes that record; one successful inclusion settles it permanently within the surviving history.

The honest side needs one win, whereas an excluder needs every win. Under fee competition, continuing exclusion requires continuing expenditure, while a defender pays once to win an inclusion opportunity; the burner is always able to act as a defender.

This asymmetry assumes conflicting spends can compete by fee and assumes eventual inclusion of a fee-paying transaction by block creators. It is not a bound on inclusion time, a promise that any replacement is admitted, or a guarantee that a finite budget defeats every adversary.

Actual replacement rules, admissible fee increments and package constraints determine what an effective higher bid means. Measurement (c) must establish that policy before a deployment states a fee-race claim.

Under first-seen policy the race is propagation rather than a simple fee auction: indefinite exclusion still requires winning every successive race, but the one-higher-fee argument does not follow. The deployment must state which policy supports its censorship account beside its settlement depth.

A block creator can refuse an inclusion regardless of fee, just as it can refuse the original destruction transaction. This is the ledger's own inclusion exposure, expressly retained as an assumption rather than hidden inside permissionless script access.

### Reader evidence and replacement histories

Given a lineage point and its ancestry transactions with inclusion proofs, a reader verifies the settled set and verifies that no record is missing relative to that point. It walks consumed inputs and the designated predecessor chain rather than scanning intervening blocks and totaling arbitrary data outputs.

The supplied closure must include every relevant consumed branch through which a lineage-script output can introduce records, and enough previous-output evidence to classify those branches. A missing transaction makes the proof incomplete; it never licenses treating an unknown branch as empty.

Historical transaction retrieval is the ledger's data-availability assumption. The spend relation authenticates dependencies but does not itself operate an archive or guarantee that a reader can retrieve every required byte.

A surviving point binds the same transaction ancestry and the same settled records; nothing can move a record across that point while preserving its identity and dependencies. Inclusion proofs must still bind the point and ancestry to the selected history.

In an alternative history a record is settled wherever that history settles it, possibly nowhere, and a dropped lineage point settles nothing. A different point may settle the same surviving destruction record without preserving the old boundary.

Finality is a published depth with explicit assumptions. Consumer publication rules pin readings, bound interim readings by the next recognized settled set, and restrict revisions of nonfinal readings to downward corrections no lower than the last final pin; the ledger does not manufacture those publication rules.

## 3. The requirement set and its standing

These requirements define what public destruction must provide to serve as a common reference. Their reasons are intrinsic: independent magnitude needs a real cost, neutral records need stable attribution, and shared readings need an explicit boundary between settled evidence and consumer choices.

HOLDS BY CONSTRUCTION refers to the abstract contract and a faithful realization, not an assertion that emitted bytes already exist. HOLDS BY PUBLICATION names an obligation discharged by a deployment statement; HOLDS BY CONSUMER RULE names behavior that a consumer must publish and implement; NEEDS MEASUREMENT reserves an empirical conclusion.

| Requirement | What it demands | Standing under the contract | Argument or evidence |
|---|---|---|---|
| Exogenous magnitude | Magnitude independent of any consumer's rule, so acceptance cannot manufacture paid amount. | HOLDS BY CONSTRUCTION | Credit is the explicit native value irrecoverably destroyed at output 1, never a consumer-written scalar. |
| Neutrality | A record carries no value claim, redemption or price, so recording creates no beneficiary entitlement. | HOLDS BY CONSTRUCTION | The minimal-value claim token has no holder role; its incidental funding is ignored, unpreserved and distinct from record credit. |
| Non-transferability | Recorded magnitude remains at its original address, so aggregation cannot reassign paid history. | HOLDS BY CONSTRUCTION | Every record retains its own transaction's address and amount; merge unions record identities rather than pooling their credit. |
| Settled monotone reading | Pinned address readings never decrease along a history, interim readings stay within the next pin, and permitted nonfinal revisions respect the last final pin. | HOLDS BY CONSUMER RULE | Non-negative L supplies the substrate; published A(previous) ⊆ A(this) ⊆ L(this), pin selection and revision policy supply the reading discipline. |
| Opaque permanent addresses | An enduring datum without a ledger identity claim, so attribution does not depend on a role registry. | HOLDS BY CONSTRUCTION | Exactly 32 burner-written bytes remain recoverable in the original destruction transaction. |
| Non-negativity | No record subtracts credit, so extending a settled set cannot reduce an address total. | HOLDS BY CONSTRUCTION | A verified explicit positive value contributes itself; a failed pair contributes zero. |
| Provenance auditability | Every counted unit is capped by genuine same-event destruction, so copied data cannot create magnitude. | HOLDS BY CONSTRUCTION | Equality to the sibling destruction amount and an ancestry walk authenticate each counted unit, conditional on supplied complete evidence. |
| Instant costliness and append-only record | Cost is paid at recording and surviving records cannot be edited or spent back into value. | HOLDS BY CONSTRUCTION | The native OP_RETURN output destroys value at BURN; later merges consume tokens, never destruction outputs. |
| Bootstrap capacity | An opening from the first acquisition block states its pre-acquisition bound, so earlier magnitude cannot be silently assumed. | HOLDS BY PUBLICATION | Publish zero for the empty pre-acquisition interval and the reason the interval is empty; genesis itself contributes no destruction. |
| Unit cost | A positive cost per unit in a declared denomination and terms, so magnitude has an interpretable price. | HOLDS BY PUBLICATION | Publish one native unit destroyed per unit credited and the transaction fee as a separate per-recording-event cost. |
| Operator accrual | Explain whether operation accrues destroyed magnitude to any role, so neutral accounting cannot conceal a beneficiary. | HOLDS BY PUBLICATION | Publish that merging grants no destroyed amount or record entitlement; fees go to block creators, while incidental token funding is separately disclosed. |
| Lineage | A designated origin, unique successor by spend, ancestry dependence and exactly-once settlement, so a pin selects a determinate cumulative set. | HOLDS BY CONSTRUCTION | Genesis designation plus consensus spending determines the chain; token outpoint identity and set traversal prevent duplicate record credit. |
| Consumer-defined acceptance within the ledger bound | Consumers select recognized boundaries and accepted subsets without claiming unsettled records, so acceptance remains explicit and bounded. | HOLDS BY CONSUMER RULE | Each consumer publishes its carrier-provenance rule and preserves A(previous) ⊆ A(this) ⊆ L(this); acceptance equal to L removes subset selection. |

The eight construction standings are conditional semantic claims, not target admission verdicts. The positive-value representation removes the already reviewed consensus obstruction, while its relay admission remains NEEDS MEASUREMENT under §6(a); no requirement row conceals that prerequisite.

The three publication standings require an actual deployment statement. Unit denomination, event fees and incidental minimum funding must be distinguished: destruction is the unit cost, fees are expenditure on recording, and token funding may later be redirected rather than destroyed.

Bootstrap zero bounds only the empty interval before external acquisition begins. It says nothing about subsequent acquisition, reserve availability or a cap on lifetime destruction; a deployment with a nonempty earlier interval needs a different statement and evidence.

Operator accrual concerns the destroyed magnitude and the entitlements of its record. An open merge may capture incidental funding in a permitted transaction, so the publication cannot claim that operation yields no economic benefit of any kind; it states precisely that no destroyed amount is paid to an operator.

The two consumer-rule standings cannot be delegated to the covenant. A consumer publishes its pins, accepted-set rule and interim publication policy; withholding an interim update until a supporting pin exists is one conservative implementation, while earlier publication needs evidence of the promised bound.

A deployment publishes settlement depth, the assumptions under which final pins are not revised, historical data availability and the applicable head-conflict policy together. Finality assumptions do not ensure retrieval, and retrieval assumptions do not settle competing histories.

The permissionless-inclusion argument remains conditional on eventual inclusion and the measured conflict policy. NEEDS MEASUREMENT applies to that policy and to relay and resource conclusions, rather than turning the abstract requirement table into a claim that experiments have already succeeded.

## 4. The traps

### The accumulator trap

The attestation rejects a global amount accumulator because it introduces shared mutable state and serialization (`[RZ-trap:architecture:accumulator]`). This exemplar stores no aggregate, count, index or commitment: the lineage output carries only a script and incidental funding, and a reader reconstructs all record figures.

It nevertheless has a serialized settlement head, which is a real contention cost. The distinction is not that all serialization disappears, but that BURN remains independent of that head and settlement stores no mutable quantity whose correctness must be maintained by arithmetic.

### The burn-flag trap

A token beside arbitrary data proves no paid magnitude (`[RZ-trap:architecture:burn-flag]`). Here the token alone proves none either: credit requires the verified sibling OP_RETURN output's explicit positive native amount in the same transaction.

The minimal-value claim token transports a path into settlement, not a trusted quantity. Counterfeit tokens and malformed pairs can be merged without producing positive credit; the reader's value anchor, rather than a decorative marker, defeats fabricated magnitude.

### The OP_RETURN-ledger trap

An indexer that totals attacker-chosen quantities from data creates free credit (`[RZ-trap:architecture:opreturn-ledger]`). This destruction output carries the address as data but carries magnitude as consensus value, and a reader reaches it through the settled token ancestry.

The OP_RETURN destination is deliberately unspendable, so the native amount is paid rather than merely asserted. Neither a data-only copy nor a transaction outside the selected settled ancestry can increase the reading.

### The event-anchor trap

The event-type and value anchors must establish both what happened and how much it paid (`[RZ-trap:ledger:event-anchor]`). The pair's common transaction, fixed positions, covenant token and native positive destruction shape establish the event; the destruction output's explicit value establishes the amount with equality.

MERGE does not pretend to certify a fresh burn for each consumed token. A reader preserves original pair identities and recognizes a new record only when the merge transaction itself also contains a valid pair; inherited records retain their original event anchors.

## 5. Mapping onto the pipeline

### Architecture and realization

Architecture owns the typed normative manifest, finite identifiers, validation and canonical publication, as described in `packages/architecture/README.md`. Its current manifest is explicitly the attestation realization; accepting another semantic family requires examining that binding rather than assuming its identifiers are already generic.

The new architecture description needs the two operations, the lineage-script object, destruction-pair recognition and ancestry observables. It does not need the attestation's five-root inventory, receipt classes or STATE field registry, and must not populate those registries with fictitious equivalents.

Realization owns semantic facts, expressions, relations, constructibility, lifecycle and observables (`pkg:realization:contract`). Existing typed graphs and scope validation are reusable mechanisms; the new declarations and their architecture binding do not exist in the dispatch listed in §1.

BURN's relation families are pair shape, fixed output positions, explicit positive native destruction value, exact 32-byte data, token recognition and immutable record association. The declaration must distinguish those recognition predicates from conditions that a spending covenant actually executes.

MERGE's emitted relation is successor-at-0 carrying the script for each consumed covenant input. Its semantic projection unions inherited record identities and recognizes any new valid pair, but the script does not compute that union, walk history or enforce a consumer's accepted set.

Lifecycle closure here means that a recognized covenant spend continues into the same script at output 0. It does not mean that every token eventually reaches genesis, that incidental funds are conserved inside the object class, or that an arbitrary consumer must accept a settled record.

Three homes remain possible: another declaration set inside realization, a separately selected semantic profile, or a second realization crate behind a shared compiler input boundary. The first shares infrastructure but broadens identifier assumptions; the second needs a profile that selects semantics rather than merely deployment evidence; the third makes independence visible but requires extracting a shared typed interface.

The realization dependency and validated-input rules constrain all three choices (`sec:realization:dependencies`). Architecture deployment profiles described in `packages/architecture/README.md` validate evidence against an existing manifest; they cannot be relabeled as an already implemented second-contract selector.

### Compiler analysis

Compiler owns target-independent analysis and emits no target program (`pkg:compiler:contract`). Its graph, fact-source, constructibility, lifecycle, proof-planning, placement and coverage foundations are reusable; a validated operation plan for this contract would be new.

The analysis must keep public record predicates separate from the successor script predicate. BURN has no mandatory covenant input on which to execute its pair recognition, so attributing the entire burn relation to an emitted leaf would manufacture an enforcement claim.

An ancestry reader and evidence projection can own the record predicates while target consensus owns actual destruction semantics. Each active relation needs a reachable and honestly named carrier under (`rule:compiler:placement`), even when that carrier is outside script execution.

Public constructibility is especially direct: a merge needs previous-output data, the public fixed constructor and its spend recipe, and carrier funding. No creator secret or owner signature belongs to the covenant's witness availability requirements.

The current compiler contract distinguishes internal analysis stages from published validated operation boundaries. The study therefore proposes reuse of analysis responsibilities, not a claim that a public generic compile-any-contract entry point already exists (`sec:compiler:outputs`).

### Reviewed Elements facts and transaction forms

Target-elements owns opcode, encoding, capability, resource and transaction-form facts, not exemplar semantics (`pkg:target-elements:contract`). Existing reviewed families include output introspection, confidential field encodings and fee roles; the new contract needs its own form assessments against those facts.

`packages/target-elements/README.md` describes `TransactionForm`, `FormAdmission`, `RelayCondition` and `FeeOutputContract`. Each proposed form needs separate consensus and relay rows, and a direct-submission condition cannot repair a consensus refusal.

| Proposed form | Consensus admission evidence | Relay admission evidence | Contract-specific obligation |
|---|---|---|---|
| BURN pair | Reviewed positive explicit spendable funding and unspendable destruction semantics, followed by exact-form confirmation. | NEEDS MEASUREMENT for the token dust floor and the 32-byte data output; account for target fee-output placement. | Output 0 recognizes the covenant token and output 1 is the native explicit destruction with exact address data. |
| MERGE carrier | Reviewed output-script introspection and a valid successor constructor, followed by exact-form execution evidence. | NEEDS MEASUREMENT for successor dust, conflict policy, witness limits and N-dependent weight. | Every covenant input checks the same output-0 script; no value, asset or other carrier relation is added. |

These are proposed admission rows, not existing enum variants or completed target reviews. Their measurements can support `RelayCondition` choices, including direct submission to a block producer where appropriate, while retaining the distinction between admission and eventual inclusion.

`TAPSCRIPT_STACK_ITEM_RELAY_LIMIT` is the reviewed 80-byte per-witness-item relay limit. It is not a data-output bound and gives no answer about the OP_RETURN address carrier; the latter needs its own policy evidence.

`ConfidentialFieldEncoding` separates explicit and committed asset and value representations. The reviewed output-value opcode returns the encoded field, including committed alternatives; inspection of a commitment does not reveal the numeric amount, so direct arithmetic comparison requires explicit value or an independently supported opening mechanism.

This contract chooses explicit destruction value and an identifiable native asset for record verification. It needs no opening of confidential destruction value, while unrelated carrier outputs may be confidential if the exact mixed form is admitted.

### Tapscript emission and constructor closure

Tapscript owns typed lowering, stack validation, proof patterns, concrete placement, relocations and resource formulas (`pkg:tapscript:contract`). It needs a new successor-script equality pattern and a public constructor recipe for this exemplar, rather than the attestation's amount-preserving compact pattern with predicates removed informally.

The reviewed primitives include `InspectOutputScriptPubKey`, `InspectOutputValue` and `InspectOutputAsset` in `packages/target-elements/src/opcode.rs`. MERGE needs only script inspection at output 0 and equality to the authenticated current covenant program; its leaf does not inspect output value or asset.

Script introspection returns the reviewed witness-program or digest representation, not an assumed arbitrary script byte array. The pattern must compare the full relevant representation, including its type or version, and reject a superficially similar successor from a different constructor.

The fixed covenant is a tapscript leaf in a public taproot constructor. The constructor must provide no alternative spend path that bypasses the successor condition, including no usable key path, and its witness must authenticate the actual committed leaf and control path.

Comparing to a caller-supplied unauthenticated script would not establish recurrence. Comparing the successor to the authenticated current input program is a possible reference strategy; a recursive literal commitment requires a supported constructor/reference solution rather than an assumed fixed point.

That constructor choice is technical work still required, not an extra semantic check on the carrier. Its purpose is to make the one successor condition refer to the actual covenant, while the final stack and resource schedule remain valid under the target contract.

### Linking and transaction construction

Linker owns symbols, reference graphs, cycle strategies, relocations, deterministic taptrees, control recipes and carrier preservation (`pkg:linker:contract`). Existing candidate bundles demonstrate those responsibilities for attestation operations; a lineage bundle and its exact recurrence strategy would be new.

The linker must establish that token creation and successor recognition use one constructor and that every relocation resolves exactly once. A trivial semantic relation still requires reference closure, candidate status and a control recipe; none of these belongs in an ancestry reader's ad hoc script literal.

Transaction owns final layouts, requests, constructor instantiation, assembly, witnesses and valid calibration fixtures (`pkg:transaction:contract`). A BURN request needs an address, amount and funding view; a MERGE request needs its covenant inputs and an otherwise permitted carrier construction.

The BURN ABI fixes outputs 0 and 1 and permits other target-valid outputs. The MERGE ABI fixes only the covenant successor at output 0; input ordering chosen by a builder must not become an undeclared protocol restriction on which input carries the lineage predecessor.

Funding inputs can require ordinary signatures, and a carrier can carry consumer authorization. Neither is covenant authorization; the ABI should expose these separate witness roles without importing the attestation's every-receipt-owner signing protocol.

The STATE metadata codec, STATE constructor generations, maturity signing states and exact sponsor-asset isolation relations are not needed here. Generic constructor recipes, transaction finalization, witness assembly and funding remain useful, but their reuse must not close unrelated carrier outputs by accident.

### Vectors, labels and documentation

Vectors owns semantic fixtures, focused mutations, target execution, accepted projections, relation coverage and resource evidence (`pkg:vectors:contract`). New evidence must separate successor enforcement, pair recognition, ancestry-set completeness, consumer acceptance and target admission; one successful merge cannot discharge all five.

Positive cases include one token, many tokens, a side-pile sweep, combined BURN/MERGE and multiple successive points in one block. Negative cases include a wrong successor script, missing output 0, malformed pair, wrong asset, committed destruction value, duplicate traversal and missing ancestry; only the script faults should be expected to fail at covenant execution.

An independent reader should reconstruct record sets from public transactions without relying on the builder's claimed total. Resource evidence binds the exact bundle, ABI, target and input bound, while infrastructure failures remain distinct from target refusals under (`rule:vectors:execution`).

Labels owns citation resolution and source census, not new contract semantics (`pkg:labels:contract`). This draft mints no labels; implementation could later register admitted semantic owners through the established label process, without using the draft as a substitute authority.

Documentation owns the explanatory study and its registration under (`rule:drafts:authority`). The new file joins the draft index and hand-managed Meson census in one change under (`[ADR014-rule:build:census]`); the base has ten indexed drafts and eleven census entries, becoming eleven and twelve respectively.

Public disposable fixtures remain within (`[ADR015-rule:security:public-data]`). Identity allocation follows (`[ADR021-sec:identity:procedure]`), and an actual external consumer of a record document would activate the interchange boundary under (`[ADR022-rule:interchange:no-implementation]`); this study creates neither an identity nor an interchange namespace.

## 6. Measurements and evidence

No Elements experiment is performed for this study. The following obligations must be established on the pinned Elements toolchain before an execution guide claims admitted transaction forms, a calibrated bound or a deployment-specific inclusion argument.

**(a) Relay funding floor.** Measure the dust threshold and default relay admission of the exact minimal-value claim token and successor constructor, including boundary values immediately below and at the admitted minimum; this decides construction funding and whether a special relay route is needed.

The consensus side is already reviewed: `ExplicitZeroValueRule::SpendableRefused` and its `VerifyAmounts` source note in `packages/target-elements/src/transaction_form.rs` refuse an explicit zero-valued spendable output. Measurement (a) concerns the positive relay floor alone and must not describe that consensus fact as unresolved.

**(b) Destruction data-output admission.** Measure an explicit positive native OP_RETURN output carrying the exact 32-byte datum, including its encoded script overhead and data-output policy limit; this decides whether the proposed BURN pair relays as built and what carrier constraints a deployment must publish.

The report must keep data-output limits separate from the reviewed 80-byte tapscript witness-item limit. Cases should distinguish malformed payload rejection by the reader from transaction rejection by consensus or relay; a transaction can be admitted yet contribute no record.

**(c) Conflicting head spends.** Establish whether the deployment's mempool admits fee-based replacements, the relevant replacement conditions and fee increments, or instead applies first-seen behavior; this decides which per-head censorship argument the deployment can make.

Record the exact node policy and configuration along with results for successive competing spends. Replacement admission does not establish eventual block inclusion; that remains a named assumption, and no experiment should report a universal censorship guarantee from a finite race.

**(d) Mixed representations.** Establish admission of a transaction containing explicit native destruction and explicit token funding alongside confidential unrelated outputs; this decides how broadly the merge and burn forms can share a carrier without restricting the consumer's other outputs.

The fixtures must bind encodings, balance proofs and fee outputs to the reviewed confidential facts. They must not substitute a disclosed scalar for authenticated destruction value or claim a general confidential-opening primitive that the target review leaves unsupported.

**(e) The one-check implementation.** Confirm the script-introspection and equality schedule, authenticated current-program reference, constructor control path, stack behavior and cost of the successor-at-0 leaf; this decides whether the semantic recurrence has a faithful admitted target realization.

Measure exact program bytes, witness items, control depth, stack peaks and validation cost, with wrong-script and wrong-version successors as negative cases. The primitive review is reusable evidence, but it is not a substitute for executing this composed pattern and checking its accepted projection.

**(f) Transaction weight and N.** Measure a BURN and MERGE carriers with N covenant inputs across the proposed range, including fee funding, optional confidential outputs and worst-case witnesses; this decides the construction bound, fee estimates and practical side-pile sweep size.

Compare predictions from backend and linker resource formulas against observed transaction weight and execution resources. Publish absent dimensions as absent, and make mismatches findings rather than silently adjusting the predicted figures to the observation.

The evidence package would carry public semantic fixtures, deterministic construction inputs, exact candidate bundle and ABI bindings, positive vectors, focused mutations, target admission transcripts and independently reconstructed accepted record sets. Measurement reports retain policy configuration and toolchain provenance without converting source identifiers into protocol identity.

Distinct reports should answer destruction-pair recognition, successor enforcement, ancestry completeness, replacement-history binding, consumer reading discipline and resources. A missing ancestor is an incomplete proof; a failed child process is infrastructure failure; a policy rejection is not automatically a consensus rejection.

Historical availability and finality remain published assumptions with explicit limits. The evidence can demonstrate reconstruction from supplied transactions and alternative-history fixtures, but cannot turn those demonstrations into unconditional archival availability or irreversible settlement.

## 7. Against the attestation exemplar

| Dimension | Attestation exemplar | Destruction-lineage exemplar |
|---|---|---|
| Operations | Typed compact ASH, live transfer and maturity announcement exist; Phase 7 specifies burn, compact maintenance and clear work. | BURN pair recognition and MERGE successor recurrence; neither declaration is implemented. |
| Objects | Live receipts, public ASH and root objects have distinct semantic roles and constructors. | Destruction output, minimal-value claim token and same-script lineage output, with designated genesis. |
| STATE | Canonical metadata, constructor generations and root transitions support maturity and planned clear. | No STATE metadata or mutable quantity; lineage identity is an outpoint. |
| Arithmetic in script | Compact preserves an exact aggregate; clear requires a bounded minimum and STATE decrement. | No magnitude arithmetic in the covenant; a reader sums verified immutable records. |
| Signing and authorization | Live operations require owner authorization, maturity uses operator authorization, and public maintenance is permissionless. | No covenant signature; ordinary funding and optional consumer signatures belong to the carrier. |
| Asset | A closed issued receipt asset participates in class recognition and conservation. | Native asset is destroyed directly; no issued asset and no covenant asset check. |
| Value held on the ledger | ASH holds consensus value until clear destroys the relevant amount. | Destruction is immediate; only incidental, unpreserved funding remains on lineage-script outputs. |
| Event model | Burn, compact silence and clear have distinct authenticated projections. | A verified pair anchors destruction; a lineage point settles ancestry; a consumer separately recognizes boundaries. |
| Fault classes | Ownership, object classes, aggregate arithmetic, STATE transitions, constructor continuity and sponsor isolation. | Pair recognition, recurrence authentication, duplicate or incomplete ancestry, consumer bounds and admission policy. |
| Evidence families | Safety, event, continuity, recovery, accounting and resource reports cover different attestation obligations. | Pair, successor, ancestry, replacement-history, consumer-rule and resource evidence cover different obligations. |
| What a covenant flaw could release | Depending on the affected object, spendable protocol quantities and protections around receipt or root transitions. | Incidental funding and settlement continuity; it cannot recover value already destroyed by OP_RETURN. |
| What a reader needs to hold | Authenticated event and relevant root-history evidence for the requested attestation reading. | A pinned lineage point, complete supplied ancestry with inclusion proofs, genesis designation and the consumer's published rule. |

Guide 15's compact relation preserves the exact sum of authenticated public ASH amounts (`rule:guide15:compact-relation`). MERGE shares permissionless construction and successor placement, but does not preserve incidental value and must not inherit the ASH aggregate equation.

Compact's silence requires no burn record or event, no clear event, no STATE, no issuance and no destruction (`rule:guide15:compact-projection`). MERGE deliberately imposes no comparable carrier closure: a transaction may carry an unrelated act or even a new valid destruction pair without violating the one-check covenant.

Compact also closes every output carrying the protocol asset to its one exact successor (`rule:guide15:compact-output-closure`). The lineage covenant does not close an asset domain, so copying that closure into the new declaration would change the contract rather than merely reuse an implementation.

The Guide 15 placement plan puts ASH recognition, aggregate preservation, output closure, sponsor isolation and event silence on the compact coordinator (`rule:guide15:carrier-placement`). Here each covenant input checks the common successor and reports establish record history; there is no semantic coordinator with a separate authorization role.

Guide 15's compact layout places the aggregate ASH successor at output 0 with optional sponsor change and the required target fee (`rule:guide15:compact-layout`). That positional similarity is useful ABI precedent, while the closed amount and sponsor relations remain specific to the first exemplar.

The comparison tests whether reusable mechanisms can carry different laws without importing the first exemplar's laws as universal preconditions. A successful second compilation must preserve the absence of an aggregate equation and consumer authorization just as faithfully as the first preserves required arithmetic and signatures.

The smaller covenant also sharpens assurance boundaries: less on-script logic does not eliminate constructor closure, truthful relation placement, reader completeness or target policy evidence. The compiler is tested by carrying each obligation to its proper owner, not by making every obligation execute in a leaf.

## 8. Findings for this tree

1. **The second declaration family is absent.** `packages/realization/src/declarations/mod.rs` exposes only compact ASH, live transfer and maturity announcement; a new exemplar needs actual declarations and cannot be activated by changing a transaction request alone.

2. **Architecture generality remains to be demonstrated.** `packages/architecture/README.md` identifies an attestation-specific finite manifest, while (`pkg:realization:contract`) consumes its validated architecture and identifiers; another contract tests this input boundary before it tests target bytes.

3. **Explicit positive token funding is required by the reviewed target.** The `SpendableRefused` term in `packages/target-elements/src/transaction_form.rs` rules out an explicit zero-valued spendable token, leaving the positive relay minimum as a separate measurement rather than a semantic quantity.

4. **The reviewed target provides the relevant introspection vocabulary, but no completed lineage form.** `packages/target-elements/src/opcode.rs` names output script, value and asset inspection, and `packages/target-elements/README.md` separates consensus and relay admission; the proposed forms still need composed evidence and policy measurements.

5. **Existing compact maintenance is stronger than this recurrence in materially different ways.** Guide 15 requires aggregate preservation, event silence and output closure, while this contract requires only successor-script continuation; reusing the compact implementation unchanged would add unwanted semantics (`rule:guide15:compact-relation`).

6. **Faithful compilation needs reader-owned relations as well as emitted relations.** BURN can have no covenant input, yet pair verification and ancestry completeness remain mandatory; compiler placement and vector evidence contracts provide the right responsibility split without pretending those checks run on every burn (`rule:compiler:placement`).

7. **One semantic check still needs constructor and reference closure.** Tapscript lowering and linker control recipes must authenticate the recurrent script and exclude bypass paths; existing package responsibilities supply the location of that work but not a completed constructor for this exemplar (`pkg:linker:contract`).

8. **Roadmap admission and execution evidence remain separate decisions.** Phase 7 is Active and Guide 15 admission remains its first gate; the second exemplar has neither an admitted card nor measured forms, and this draft's registration changes neither fact (`phase:roadmap:burn-clear`).

## 9. Decisions left open

1. **Realization home.** A second declaration set shares existing infrastructure but broadens attestation-specific identifier assumptions; a semantic profile requires an explicit selector beyond today's deployment-evidence profile; a second crate makes separation clear but costs a shared input interface and additional maintenance. The constraints are the architecture boundary and (`pkg:realization:contract`), and no option is selected here.

2. **Incidental funding and relay floor.** The contract takes positive native funding with no semantic amount meaning; explicit zero-valued spendable outputs are already refused by the target review. Choosing the measured default-relay floor favors ordinary propagation, while a smaller positive value with a separately admitted route adds deployment dependence; measurement (a) decides feasibility and the statement must disclose the chosen route.

3. **Whether any value-preservation rule is wanted.** None is the contract's default: funding may move to fees or elsewhere. A greater-or-equal successor rule would protect a stated funding floor or sum while allowing donations to remain, and equality would preserve an exact chosen amount or aggregate while constraining fee use and donations; either requires defining the comparison base, adding introspection and changing the one-check contract, without improving record magnitude authentication.

4. **The input bound N.** A small constructor bound simplifies worst-case witnesses and predictable fees but needs more sweeps; a larger bound improves aggregation at the cost of weight and policy pressure. The bound may be an ABI construction limit rather than a new covenant count check, and measurement (f) must distinguish those choices.

5. **Genesis designation.** A public deployment manifest can name one outpoint directly, while a published deterministic deployment procedure can explain its derivation; the former needs trustworthy distribution of the designation and the latter needs reproducible construction and chain binding. Neither allows longest-history selection or an arbitrary same-script output to replace the designated origin.

6. **Roadmap admission and first gate.** Keeping the study as a draft costs no new execution commitment but leaves independence untested; admitting a card beside Phase 7 costs capacity and requires an explicit scope and dependency decision. A possible first gate would establish a second typed declaration boundary, authenticated recurrence constructor and separately assessed BURN/MERGE forms, with the six measurement obligations disposed before any execution-guide success claim.

7. **Recurrence constructor strategy.** Authenticating the current input program for successor comparison avoids an assumed literal fixed point but requires a precise inspected-program and control-path proof; a linker reference strategy must instead demonstrate supported closure and its cost. Either must exclude alternative spend paths and retain the same semantic successor rule under (`rule:linker:references`).

8. **Consumer publication policy.** Acceptance equal to L removes discretionary subset selection but may not meet a consumer's purpose; a published subset rule adds recognition and monotonicity obligations. Publishing only at pins simplifies the interim bound, while earlier readings need additional evidence and explicit downward-revision behavior under the chosen depth and availability assumptions.

9. **Evidence and external consumption.** Internal fixture reports can establish translation-validation boundaries without an interchange identity; an external ancestry or record consumer requires an explicit format, availability contract and the ADR-022 process. The latter improves independent use but adds a maintained compatibility obligation, and neither choice turns a cached commitment into settlement authority.
