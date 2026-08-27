# Phase 5 — Live Receipt Transfer · `phase:roadmap:live-transfer`

> **Status:** Active — the candidate pipeline is complete, both typed
> blockers the intermediate guide owned are discharged on observed
> acceptances, the carried residual set is empty, and the exit campaign's
> arcs are recorded in the backlog's sections 2.10 through 2.19. The
> entry gate is satisfied: Phase 4 exited 2026-08-21, recorded in the
> backlog's section 2.8.
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

What is not a residual and never was one: the internal key's discrete-log assumption stands regardless of what any probe observes — the phase-A key-path attempt was observed and refused, which discharges only that; the typed carrier of phase B remains named follow-up work below.

## Handoff · `sec:phase5:handoff`

**Sequencing.** The intermediate confidential-funding guide has closed — its restart order Completed, its archive in the guides directory. Guide 14 — state and maturity — is drafted after this phase's exit, and consumes the following without reopening any of them: validated target-operation plans, representation-specific backend planning, owner-bound constructor metadata, per-input owner authorization, finalized-output signing, explicit and private proof separation, deterministic linking, candidate ABI construction, validated operation reports, the safety and minimality report separation, exact executor provenance and environment binding, and candidate resource measurement. It adds state constructor continuity, root succession, operator authorization, maturity announcement, state-field commitments, and the first mutable protocol constructor. Phase 5 makes none of those state or root claims.

**Named follow-up work.** Four items, each investigated and each with a report behind it:

- **Shared abstract-program analysis.** Two private wrappers measure program prefixes — one in the conformance package, one in the vectors package — and both wrap the same abstract evaluator in tapscript. The first slice adds a public analysis API beside that evaluator and side-by-side tests against both wrappers, with no consumer deleted and no canonical output changed; the wrappers are retired only after equivalence is demonstrated, and the implementation is optimized only after that.
- **Sponsor envelope evidence, delivered and bounded.** The submission this item waited on happened, and the arc behind it ran to the exit row: sponsor-signed acceptances stand on both lanes, with change and without, with an explicit sponsor value and with a blinded one whose committed change the arithmetic itself demanded. What remains is the standing non-claim, not work: one fixed regtest key answering requests is a wire, never production multi-party signing.
- **Internal-key unspendability probe, phase B.** Phase A was performed and its refusal recorded; a refusal discharges only that the attempt was observed and refused, and establishes nothing about who knows the discrete logarithm. Phase B — the typed key-path carrier and the wire correction — remains open before the named assumption may be called discharged for the instances covered.
- **The negative half of the private table.** The explicit table's negative rows opened behind their accepted controls; the private table's positives are answered and its remaining negative rows follow the same attributability rule — accepted control on the same chain, mutants first. Named work, not a blocker: every needed control now exists.
- **The two stale records the form register found.** The run-of-record cardinality arrays carry six entries for an eight-member private table, and that table's own doc still says all six; filed by the census-forms wave for the owning files' next wave.
- **A paired-projection materialization — DELIVERED by the pairs arc.** The projection-equality standing asked for one fixture materialized twice under section 16.1's pair rule, and said so rather than borrowing the two independent acceptances that do not satisfy it. The arc chartered on the owner's pairs ruling built it: one semantic fixture read by both members, one issued asset, both materializations submitted to one node and both accepted — the explicit member and the private member — with the eleven section 6.6 terms compared from the node's own copies and the private member withholding two of them. What remains is the standing narrowness, not work: one pair's members were submitted and the other four still cite an acceptance of each member's own shape.

## Exit gate · `gate:phase5:exit`

Phase 5 exits when:

- every owner authorizes the finalized output set;
- exact live-class closure holds;
- all closed-asset-capable outputs are classified;
- explicit transfer passes complete safety evidence;
- confidential-value transfer passes where claimed;
- confidential sponsor values pass on an observed acceptance of their own shape, the private-sponsor-values row moving with it;
- fee-bearing transfer is supported in all four forms — sponsored and sponsorless, explicit and confidential — each passing on an observed acceptance of its own shape, with every fee output explicit as consensus requires;
- representation-crossing transfer is supported in both directions — explicit inputs to confidential outputs, and confidential inputs to explicit outputs with the blinded absorber output consensus requires of a nonzero input blinder sum — each direction passing on an observed acceptance of its own shape, and the confidential-to-fee-only corner staying labeled consensus-impossible rather than attempted;
- the form register's remaining unsupported-here cells are honestly removed, on the precedent the six completed removals set — each of the two walls that ask removal, the registry's sole committed sponsor-change role and the solving role confined to the destinations, taken along its own filed path as a NARROWING and not a relaxation, the freed form declared by a role rather than inferred so that what was refused still draws its old refusal unless it names the form it wants, every recorded digest re-deriving because the declaration rides in a role code the transcript already emits, each removal recorded in the same four-stage arc — the row that took it, what structurally changed in the registry's own terms, and a run-of-record identity where a node accepted a freed shape, claimed nowhere that nothing ran — and the freed cells re-verdicted by the unchanged cascade rather than relabeled, any wall a removal uncovers censused as its own limitation with its own path;
- safety and minimality reports remain distinct;
- lifecycle incompleteness is explicit;
- mixed-program vectors reject;
- linked bundle and ABI are deterministic;
- resource reports pass;
- relation coverage is complete and the tree remains clean.

## Exit assessment · `sec:phase5:exit-assessment`

Assessed tree clean, the full suite green (48 of 48 meson lanes, gate §2.20). Every gate row is met. The safety-evidence row rests on a completed disposition rather than on universal observation, and the twenty-seven typed rows behind it are named here rather than left implicit.

Owner authorization, live-class closure, and closed-asset classification stand from the positive half; per-input owner authorization is verified against an independently recomputed message on every acceptance, and the covenant's output inspection is reached past the signature gate. Confidential-value transfer, confidential sponsor values (the private-sponsor-values row moving with it), the fee matrix in all four forms, and representation-crossing in both directions (exit and entry both, the blinded absorber proven and the fee-only corner labeled consensus-impossible) all pass on observed acceptances of their own shape. The unsupported-here removal is complete: both removal-asking walls are down along their filed paths as narrowings, the register holds no unsupported-here cell, each removal is recorded in the four-stage arc with a `proven_by` honestly `None` where the newly uncovered materializer-projection wall — censused as its own limitation — stands between the freed vocabulary and a node, and no recorded digest moved. Safety and minimality reports stay distinct, lifecycle incompleteness stays explicit, mixed-program vectors reject, the linked bundle and ABI are deterministic, resource reports pass, and relation coverage is complete against a clean tree.

**Explicit transfer passes complete safety evidence** on the disposition standard the phase set. The explicit positive table is sixteen of sixteen. The section 15 matrix — 108 rows — is completely disposed: eighty-one observed or first-party-established, twenty-seven honestly typed against named gaps, cross-footed against the classifier both directions with no silent row. The twenty-seven are named work, not silent gaps: nine need a transaction the demonstration deployment does not emit (the issuance, destruction, root, burn and transition-certificate facets; a balanced added flow; and three sponsor-region faults needing a sponsored successor); five have no admitted shape carrying the fault; three do not separate from a row already driven (copied-commitment behind private-ct-imbalance, and two leaf-arrangement partners behind the two that drove); two have no covenant clause constraining the mutated field independent of the owner signature; two have no wire field carrying the fault; and six are singles — the section 16.1 paired-projection materialization and the internal-key phase B probe, both already named in this card's handoff; a witness-surgery stage that would still draw an already-established program-generic verdict; an underdetermined row class; a row typing in question; and witness-reorder, inexpressible because the witness stack is written from a fixed order constant.

**The exit gate (`gate:phase5:exit`) is MET.** Every gate row carries its evidence; the safety matrix is completely disposed with no silent gap; the tree is clean and the suite green. The twenty-seven typed rows are named follow-up work carried honestly in the register, consistent with this card's handoff, which lists the negative half and the paired-projection and key-path probes as named work rather than blockers. The Guide-14 drafting decision is the owner's.
