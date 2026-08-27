# Phase 5 — Live Receipt Transfer · `phase:roadmap:live-transfer`

> **Status:** Exited — the amended exit gate is met on the assessment
> re-recorded below: assessed at the binding-closeout wave's tip with the full closing gate green
> and countersigned by the external adversarial review this card's
> assessment cites, whose two remaining majors the binding-closeout wave
> then closed. The exit campaign's arcs are recorded in the backlog's
> sections 2.10 through 2.21; the owner flipped this status 2026-08-27.
> The entry gate was satisfied by Phase 4's exit of 2026-08-21 (backlog
> section 2.8).
> **Entry:** (`gate:phase4:exit`)
> **Packages:** realization, compiler, tapscript, linker, transaction, vectors
> **Decision:** D005 value representation

## Goal · `sec:phase5:goal`

Validate:

- every-owner authorization;
- live-class closure;
- closed-asset output closure;
- split/merge conservation;
- explicit and confidential value proof alternatives;
- sponsor isolation;
- separate safety and disclosure-minimality evidence.

## Deliverables · `sec:phase5:deliverables`

### Constructors

Link a live-receipt constructor binding:

- explicit `U` asset;
- owner metadata;
- live class;
- selected value representation;
- transfer operation programs.

### Programs

Emit:

- local live receipt recognition;
- owner authorization on every input;
- canonical coordinator;
- complete input/output family ranges;
- live output closure;
- exact aggregate value conservation;
- sponsor isolation;
- representation-specific target proof.

### ABI

Support typed requests for:

- receipt inputs;
- destination owners and values;
- explicit or supported confidential representation;
- optional sponsor.

All signature-committed outputs are finalized before signing.

## Required safety vectors · `sec:phase5:safety`

- empty input/output;
- above input/output bounds;
- wrong or missing owner;
- one omitted owner in a multi-owner transfer;
- time-locked input;
- time-locked output;
- ASH or other undeclared output;
- wrong explicit asset;
- confidential asset commitment;
- unclassified closed `U`;
- aggregate value mismatch;
- duplicate output claim;
- sponsor overlap;
- output mutation after signing;
- wrong constructor metadata;
- transfer/burn program mixture.

## Required minimality vectors · `sec:phase5:minimality`

Compare semantically equivalent:

- explicit one-to-one transfer;
- explicit split;
- explicit merge;
- confidential-value one-to-one transfer;
- confidential-value split/merge where supported;
- confidential sponsor value where supported.

Require:

- identical semantic value relation;
- identical owner/class result;
- identical public protocol projection;
- target acceptance;
- no confidential closed-asset identity.

## Lifecycle · `sec:phase5:lifecycle`

A private live receipt is not release-complete until it retains supported paths
to:

```text
transfer
burn
redeem
```

Phase 5 records future lifecycle obligations explicitly. It does not overclaim
direct private burn or redemption support.

## Resource evidence · `sec:phase5:resources`

Measure:

- one input/one output;
- maximum candidate families;
- multi-owner signatures;
- explicit and confidential values;
- sponsorless/sponsored;
- largest proof and witness forms.

## Candidate pipeline · `sec:phase5:candidate-pipeline`

The Guide-13 batch built the live-transfer candidate end to end. Every stage exists, is reachable through its own package's public surface, and is exercised by tests that recompute rather than assert: the compiler's validated target-operation plan, the static live-receipt constructor, backend emission of the transfer program set, deterministic linking into a candidate bundle with its taptree and carrier closure, the candidate transaction ABI, construction and owner-authorization, the canonical evidence plan over the complete section-15 safety matrix, the disclosure-minimality pair registry, and the resource study with its two lanes.

Nothing in that pipeline is final. Every constructor, bundle, ABI, bound, and lifecycle claim is candidate-only, and no identity is minted for any of them.

### What holds · `sec:phase5:holds`

The acceptance lines that hold on the current tree:

- live-transfer semantic scope is exact, and every relation survives every package boundary;
- every target requirement is assessed and every selected proof is realization-approved;
- every active relation-case has a reachable carrier;
- the request cannot select a program, a constructor, or any of the fourteen structurally absent facets;
- explicit invalid cases fail at their owning boundaries — the whole first-party half of the safety matrix is discharged, twenty-six rows of twenty-six, each by driving its own owning validator twice and requiring the refusal to name the row's own class, the twenty-sixth being the time-locked-input row retyped to this half when its declared target boundary was shown to demand an observation the target's program-generic commitment refusal can never carry;
- unknown key types cannot satisfy authorization, because the owner-key encoding closure refuses them before a constructor exists to place them in;
- linked bundle and ABI are deterministic;
- safety and minimality reports remain separate, down to distinct schema identifiers on their first lines;
- lifecycle incompleteness is explicit: transfer is implemented, burn and redeem are recorded outstanding, and no pretend leaf is emitted for either;
- candidate bounds remain non-final, and the resource study reports prediction and observation as separate figures;
- public test material is not described as a production secret interface;
- no speculative digest is minted;
- canonical report bytes reproduce.

### What was typed-blocked, and how each blocker discharged · `sec:phase5:blocked`

The phase stood for a time as an honest stopped result on two typed blockers, both owned by [the confidential test materialization and funding guide](../guides/guide_confidential_funding.md) — the intermediate guide the owner chartered and which has since executed to completion, its restart order Completed across all seven steps. Both blockers are discharged on observations rather than narration:

- **The owner sighash is computable and established.** The digest was computed, the profile established over its required set, and both lanes carry observed acceptances whose signatures verify against independently recomputed per-input messages. The explicit positive table stands complete at sixteen of sixteen observed acceptances, the private positive table is answered through observed acceptances of each row's own shape, and the negative half opened behind them with refusals made attributable by their accepted controls, mutants offered first.
- **Confidential predecessors are funded routinely.** The funding stage creates blinded-output predecessors — including the three-output non-canceling form built so that a merge's forced blinder is provably nonzero — and every private acceptance since spends one. The blinded-value sponsor coin has its own funding stage beside them.

The consequence paragraph that stood here is retired by the outcomes it predicted: the private minimality gate is satisfiable and the disclosure-minimality registry answers per pair — five pairs built, five supporting on recomputed conjuncts — with the report inheriting rather than escaping the disclosure that no pair member was itself submitted as a pair.

## Identity, schema, and interchange · `sec:phase5:impact`

**Identity.** No digest is minted anywhere in the batch. Attestation and the realization are unchanged. Constructor identity is the typed value's equality rather than a hash; bundle and ABI identity are likewise in-process candidate values. The safety, minimality, and resource report identities are schema identifiers, not content digests.

**Schema.** The typed schema work is the validated live-transfer compiler projection, the representation plans, owner metadata, the candidate live-receipt constructor, the candidate linked bundle, the candidate ABI, the validated operation report, the safety and minimality plans and reports, the first-party negative evidence report, and the ABI-validation report role. Three rendered report schemas are versioned in their own bytes — live-transfer safety at revision 2, minimality at 1, resource at 1 — alongside the plans report schema at 3. Safety revision 2 adds the operation-vocabulary-closed census line and its outstanding spelling; a revision-1 reader summing the census lines it knew would find them short of the row count, which is the misreading a stated schema turns into a refusal.

**Interchange.** The generated reports are terminal audit closures under ADR-022: they are rendered for a reader, not consumed by another producer, and nothing downstream parses them back into a decision. A report that acquired an external consumer would activate the rest of that posture.

## Security and production non-claims · `sec:phase5:non-claims`

Every key, scalar, blinder, and opening in the batch is public disposable test material in the sense ADR-015's test-material rule fixes, and is named as such at each use. No production private key, blinder, or opening is admitted anywhere, and section 1.10's refusal of a first-party production secret-bearing interface holds: there is no generation, no storage, and no constructor taking randomness from a source a production secret could come from.

Production multi-party signing and blinding is not established. The private construction is central public-fixture construction and says so in its own recorded model; describing it as multi-party blinding would be the overclaim the model exists to prevent.

Signature positions in accepted live-lane runs carry real signatures over recomputed messages, verified per input out of the node's own copy of the transaction. Where placeholder bytes remain — sizing lanes and candidates that never reach a target — they are still called unauthorizing at each use. Every signing key remains ADR-015 public disposable test material; one fixed regtest key answering a request is a wire demonstration, not a ceremony, and production multi-party signing remains unestablished as stated above.

No dependency was added in the batch, so ADR-011's review has no new admission to consider.

## Residuals · `sec:phase5:residuals`

Named as a set in the evidence plan rather than as prose, so that a wave clearing one has to remove it there — and the carried set is now EMPTY. Each member left by its own site's rule, with the variant kept in the vocabulary and its clearing sections written at the defining site:

- the sighash profile was established over its required set, with observed acceptances on both lanes verifying against independently recomputed messages;
- the sponsor envelope's signer cleared on the first observed sponsor-signed acceptance — run first, verdict second;
- the confidential-predecessor residual cleared when the funding stage created spendable blinded predecessors and acceptances consumed them;
- the time-locked-predecessor residual retired by GROUND CORRECTION rather than observation: the row that demanded an attributable spend was mis-typed — the class is enforced by leaf commitment, the target's refusal is program-generic and can never name the lock, and the ruling retyped the row first-party where it discharges by its own validator; the retirement cites no run and claims none.

What is not a residual and never was one: the internal key's discrete-log assumption stands regardless of what any probe observes — the key-path attempt was observed and refused, which discharges only that. Phase B has since landed the typed carrier and the wire correction, so the refusal is now filed under its own name and behind an accepted control; the assumption is exactly where it was before either phase ran, and no phase of this probe can move it.

## Handoff · `sec:phase5:handoff`

**Sequencing.** The intermediate confidential-funding guide has closed — its restart order Completed, its archive in the guides directory. Guide 14 — state and maturity — is drafted after this phase's exit, and consumes the following without reopening any of them: validated target-operation plans, representation-specific backend planning, owner-bound constructor metadata, per-input owner authorization, finalized-output signing, explicit and private proof separation, deterministic linking, candidate ABI construction, validated operation reports, the safety and minimality report separation, exact executor provenance and environment binding, and candidate resource measurement. It adds state constructor continuity, root succession, operator authorization, maturity announcement, state-field commitments, and the first mutable protocol constructor. Phase 5 makes none of those state or root claims.

**Named follow-up work.** Four items, each investigated and each with a report behind it:

- **Shared abstract-program analysis.** Two private wrappers measured program prefixes — one in the conformance package, one in the vectors package — and both wrapped the same abstract evaluator in tapscript. The first slice adds a public analysis API beside that evaluator and side-by-side tests against both wrappers, with no consumer deleted and no canonical output changed; the wrappers are retired only after equivalence is demonstrated, and the implementation is optimized only after that. SLICES A AND B ARE DONE. Slice A landed `tapscript::stack::program_stack_profile` beside the evaluator and demonstrated it side by side against both wrappers over each wrapper's own inputs; slice B moved both call sites onto it, deleted the conformance wrapper `peak_main_stack` outright, deleted the vectors study's private copy of the prefix loop, and converted the two side-by-side tests into direct tests of the shared analysis. No canonical output moved: the study's pinned seventeen-row measured table recomputes with every `peak` figure unchanged, the prototype projection's anchored peak still recomputes to nine, and no emitter, record or recorded digest carried either figure. SLICE C — optimizing the implementation, which still revalidates the whole program once per prefix — REMAINS OPEN, and is now a change to one function rather than to three.
- **Sponsor envelope evidence, delivered and bounded.** The submission this item waited on happened, and the arc behind it ran to the exit row: sponsor-signed acceptances stand on both lanes, with change and without, with an explicit sponsor value and with a blinded one whose committed change the arithmetic itself demanded. What remains is the standing non-claim, not work: one fixed regtest key answering requests is a wire, never production multi-party signing.
- **Internal-key unspendability probe, phase B — DELIVERED.** Phase A was performed and its refusal recorded; a refusal discharges only that the attempt was observed and refused, and establishes nothing about who knows the discrete logarithm. Phase B — the typed key-path carrier and the wire correction — was open "before the named assumption may be called discharged for the instances covered", and it is now closed. The carrier is `ObservedOutcomeLayer::KeyPathRejection`, spelled `key_path_rejection` on the wire and read as its own verdict; the wire correction is the reviewed adapter's classification, which now settles a mandatory-script refusal from the witness the bytes carry and the program each spent output pays instead of from the refusal text alone, because a taproot key-path signature failure wears the same wrapper a leaf failure does. The probe re-ran against the pinned node and re-derived every figure phase A recorded, the target's verbatim words included, with the verdict filed under the key-path name; and it now offers its refusal beside an accepted control — the SAME candidate, spent by its script path, the two witnessless serializations compared byte for byte — so the §15.4 `key-path-escape` row is answered as an observed refusal against a control identity a reader can check. **The precondition is removed and nothing more is licensed.** This item's own first sentence governs what a refusal can do, and it is unchanged by the refusal having a better name: the discrete-log assumption is NOT discharged, for these instances or any, and the residual rule holds as written (`rule:exclusions:nonclaims`). One thing was escalated rather than decided by that wave: the §15.4 row still declared `EvidenceBoundary::ScriptPathRejection` as its expected boundary, which the phase-B observation shows is the wrong layer for a key-path spend, and retyping a matrix row is a matrix erratum owed to a ruling — so the wave reported it and left it. THE RULING CAME AND THE ERRATUM IS CLOSED. The thirteenth boundary member `EvidenceBoundary::KeyPathRejection` is minted on the twelfth's precedent and the row DECLARES it, with its provenance recorded beside it; the finding is `R5-010` in the backlog's section 5.10 register, DONE. The row's ANSWER did not move and no observation was reclassified.
- **The negative half of the private table.** The explicit table's negative rows opened behind their accepted controls; the private table's positives are answered and its remaining negative rows follow the same attributability rule — accepted control on the same chain, mutants first. Named work, not a blocker: every needed control now exists.
- **The two stale records the form register found.** The run-of-record cardinality arrays carry six entries for an eight-member private table, and that table's own doc still says all six; filed by the census-forms wave for the owning files' next wave.
- **A paired-projection materialization — DELIVERED by the pairs arc.** The projection-equality standing asked for one fixture materialized twice under section 16.1's pair rule, and said so rather than borrowing the two independent acceptances that do not satisfy it. The arc chartered on the owner's pairs ruling built it: one semantic fixture read by both members, one issued asset, both materializations submitted to one node and both accepted — the explicit member and the private member — with the eleven section 6.6 terms compared from the node's own copies and the private member withholding two of them. What remains is the standing narrowness, not work: one pair's members were submitted and the other four still cite an acceptance of each member's own shape.

## Exit gate · `gate:phase5:exit`

Phase 5 exits when:

- every owner authorizes the finalized output set;
- exact live-class closure holds;
- all closed-asset-capable outputs are classified;
- explicit transfer's safety evidence is COMPLETELY DISPOSED on the explicit disposition standard: every row of the section 15 matrix is either ANSWERED by the gate its own row was given — a target verdict, a determinism recomputation, a paired relation, a first-party discharge or fact, or this workspace's own report bytes — or TYPED against a named gap that says what is missing and why, cross-footed against the classifier by set equality in both directions with no silent row. THIS IS NOT SECTION 13.5'S ANSWERED-BAR AND DOES NOT CLAIM IT. That bar is `every_required_row_is_answered`, it is FALSE here, and it is honestly false: rows still standing at `NativeRunRequired` are final typed stops rather than work in flight, each one waiting on a transaction this demonstration deployment does not emit, a shape the architecture does not admit, a covenant clause or wire field that does not exist, or a verdict that would not separate the row from one already driven. A row this gate calls disposed may therefore be a row no target ever answered, and this row is signed knowing that;
- confidential-value transfer passes where claimed;
- confidential sponsor values pass on an observed acceptance of their own shape, the private-sponsor-values row moving with it;
- fee-bearing transfer is supported in all four forms — sponsored and sponsorless, explicit and confidential — each passing on an observed acceptance of its own shape, with every fee output explicit as consensus requires;
- representation-crossing transfer is supported in both directions — explicit inputs to confidential outputs, and confidential inputs to explicit outputs with the blinded absorber output consensus requires of a nonzero input blinder sum — each direction passing on an observed acceptance of its own shape, and the confidential-to-fee-only corner staying labeled consensus-impossible rather than attempted;
- the form register's remaining unsupported-here cells are honestly removed, on the precedent the six completed removals set — each of the two walls that ask removal, the registry's sole committed sponsor-change role and the solving role confined to the destinations, taken along its own filed path as a NARROWING and not a relaxation, the freed form declared by a role rather than inferred so that what was refused still draws its old refusal unless it names the form it wants, every recorded digest re-deriving because the declaration rides in a role code the transcript already emits, each removal recorded in the same four-stage arc — the row that took it, what structurally changed in the registry's own terms, and a run-of-record identity where a node accepted a freed shape, claimed nowhere that nothing ran — and the freed cells re-verdicted by the unchanged cascade rather than relabeled, any wall a removal uncovers censused as its own limitation with its own path;
- safety and minimality reports remain distinct;
- lifecycle incompleteness is explicit;
- mixed-operation programs are excluded by OPERATION-VOCABULARY CLOSURE rather than by a rejection, and no vector rejects one because none can be built: the architecture admits no value that names such a program — the compiler's projection fixes the operation to live transfer before any program is planned, the leaf-role vocabulary the constructor and the linker consume names transfer roles only, and a request cannot select a program at all — so no layer is ever offered one and no verdict exists to record. Section 15.4's `mixed-operation-program` is the SOLE row of the matrix naming no layer, held at exactly one by test, and its standing is neither answered nor evidence: it sits outside section 4.2's refusal denominator, on that section's own instruction that a requirement whose policy cannot be met belongs outside the denominator rather than permanently outstanding inside it;
- linked bundle and ABI are deterministic;
- resource reports pass;
- relation coverage is complete and the tree remains clean.

## Exit assessment · `sec:phase5:exit-assessment`

RE-RECORDED against the AMENDED gate, at the gate-repair wave's tip. The first recording of this assessment was audited by an external adversarial review and FAILED on two rows: row 4 asserted "complete safety evidence" beside a completeness predicate the tree's own classifier reports as false, and row 12 asserted that mixed-program vectors reject where the architecture can state no such vector and no layer ever gave a verdict. The owner accepted both of the review's amendments, and the two rows above now say what this workspace can support. Both amendments NARROW what is claimed; neither adds evidence, and no row moved in the matrix to make either true. The review is archived at [plans/reviews/review-9-0.5.17-dev.md](../reviews/review-9-0.5.17-dev.md) and its findings register is backlog §5.10.

Every gate row is met ON ITS AMENDED TEXT, with the safety-evidence row's standard stated in the row itself and section 13.5's answered-bar reported below as the separate — and false — figure it is.

Owner authorization, live-class closure, and closed-asset classification stand from the positive half; per-input owner authorization is verified against an independently recomputed message on every acceptance, and the covenant's output inspection is reached past the signature gate. Confidential-value transfer, confidential sponsor values (the private-sponsor-values row moving with it), the fee matrix in all four forms, and representation-crossing in both directions (exit and entry both, the blinded absorber proven and the fee-only corner labeled consensus-impossible) all pass on observed acceptances of their own shape. The unsupported-here removal is complete: both removal-asking walls are down along their filed paths as narrowings, the register holds no unsupported-here cell, each removal is recorded in the four-stage arc with a `proven_by` honestly `None` where the newly uncovered materializer-projection wall — censused as its own limitation — stands between the freed vocabulary and a node, and no recorded digest moved. Safety and minimality reports stay distinct, lifecycle incompleteness stays explicit, the linked bundle and ABI are deterministic, resource reports pass, and relation coverage is complete against a clean tree. Mixed-operation programs are excluded BY CLOSURE and not by refusal: the compiler projects one operation, the leaf-role vocabulary the constructor and the linker consume carries only that operation's roles, and no request can select a program — so `mixed-operation-program` is the sole row of the matrix naming no layer, pinned at exactly one by `live_safety`'s own test, and it is recorded as neither answered nor evidence. Nothing was submitted for it and nothing refused it, and this row is signed on that fact rather than on a rejection.

**Explicit transfer's safety evidence is completely disposed** on the explicit disposition standard row 4 now carries. The explicit positive table is sixteen of sixteen. The section 15 matrix — 108 rows — partitions with no remainder and no silent row, cross-footed against the classifier by set equality in both directions: EIGHTY-TWO ARE ANSWERED — 34 first-party discharged, 24 native-run observed, 17 native-refusal observed, 1 determinism, 1 paired relation, 3 first-party fact, 2 report-layer, with zero first-party-undischarged, zero infrastructure-blocked and zero experimental; ONE IS VOCABULARY-CLOSED, the `mixed-operation-program` row above, which is not answered and sits outside the coverage denominator on section 4.2's own instruction; and TWENTY-FIVE ARE STILL REQUIRED.

**Section 13.5's answered-bar is FALSE, and it is stated here as its own figure rather than folded into the disposition.** That bar is `every_required_row_is_answered`, it asks that no row be waiting on a target-native run, and twenty-five rows are. The plan's own test refuses to call itself complete on exactly this ground, and nothing in this assessment overrides it. What the disposition standard claims, and all it claims, is that no row is unconsidered: each of the twenty-five is a FINAL TYPED STOP with a named gap, not work in flight. Nine need a transaction the demonstration deployment does not emit (the issuance, destruction, root, burn and transition-certificate facets; a balanced added flow; and three sponsor-region faults needing a sponsored successor); five have no admitted shape carrying the fault; three do not separate from a row already driven (copied-commitment behind private-ct-imbalance, and the two leaf-arrangement partners behind the two that drove, each partner's arrangement carrying a second failing input so it has no separating fact of its own); two have no covenant clause constraining the mutated field independent of the owner signature; two have no wire field carrying the fault; and four are singles — a witness-surgery stage that would still draw an already-established program-generic verdict; an underdetermined row class; a row typing in question; and witness-reorder, inexpressible because the witness stack is written from a fixed order constant. The two singles this card's handoff carried, the section 16.1 paired-projection materialization and the internal-key phase B probe, are no longer among them: both were driven to observations of their own and left the register, which is why it reads twenty-five where the closing arc's own gate record reads twenty-seven.

**The exit gate (`gate:phase5:exit`) is MET on its amended text**, and the amendment is what makes the signature sound rather than a formality: the two rows that failed audit failed because they claimed more than this workspace can support, and a gate met by narrowing a claim to the truth is the honest form of the same gate. Every row carries its evidence; the safety matrix is completely disposed with no silent gap; section 13.5's bar is false and said so; the tree is clean and the suite green. The twenty-five typed rows are named follow-up work carried honestly in the register, consistent with this card's handoff. What is NOT claimed, plainly: that every safety row was observed against a target, that the canonical completeness predicate holds, or that any layer ever refused a mixed-operation program. The Guide-14 drafting decision is the owner's.
