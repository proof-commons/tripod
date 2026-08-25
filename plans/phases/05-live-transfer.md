# Phase 5 — Live Receipt Transfer · `phase:roadmap:live-transfer`

> **Status:** Active — the candidate pipeline is complete and the phase is
> stopped short of its exit on two typed blockers, both owned by the
> chartered intermediate guide. The entry gate is satisfied: Phase 4
> exited 2026-08-21, recorded in the backlog's section 2.8.
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
- explicit invalid cases fail at their owning boundaries — the whole first-party half of the safety matrix is discharged, twenty-five rows of twenty-five, each by driving its own owning validator twice and requiring the refusal to name the row's own class;
- unknown key types cannot satisfy authorization, because the owner-key encoding closure refuses them before a constructor exists to place them in;
- linked bundle and ABI are deterministic;
- safety and minimality reports remain separate, down to distinct schema identifiers on their first lines;
- lifecycle incompleteness is explicit: transfer is implemented, burn and redeem are recorded outstanding, and no pretend leaf is emitted for either;
- candidate bounds remain non-final, and the resource study reports prediction and observation as separate figures;
- public test material is not described as a production secret interface;
- no speculative digest is minted;
- canonical report bytes reproduce.

### What is typed-blocked · `sec:phase5:blocked`

The result is an honest stopped one rather than a full exit, and the two blockers are typed rather than narrated:

- **Owner sighash not computable.** Section 10.2 checks an owner signature with the target's own verifying primitive over the target's own taproot sighash, and section 1.7 forbids a builder asserting one. No first-party component computes that digest, so no valid live-transfer spend can be witnessed and no target can accept one. Every positive row of sections 15.1 and 15.2 carries this blocker, and the negative half is blocked behind it in turn: while nothing has been accepted, a refusal is not attributable to the row's own mutation. The native run of record is the sharpest evidence for it — the covenant ran over real coins at the linked constructors' own programs, and the target's own words were an invalid Schnorr signature. What stands between this candidate and an accepted transfer is one digest.
- **No confidential predecessor can be funded.** Section 6.3 admits a private transfer only over confidential receipt inputs, and the target-generic funding step names an explicit amount and has no confidential form. The private plan's constructors link, emit, and materialize; what cannot be created is a predecessor for them to spend.

Both are owned by [the confidential test materialization and funding concept](../guides/guide_confidential_funding_concept.md), an intermediate guide in the unnumbered category the owner ruled: it must be closed before Guide 14 is drafted.

Two consequences follow and are stated rather than left to inference. An explicit-only candidate with a typed private-plan deferral is a valid stopped result, and it is not the full Guide-13 exit; it cannot satisfy the private minimality gate. And the disclosure-minimality registry reports its verdict unanswered — five pairs built, zero supporting — because constructibility and acceptance carry those same two blockers.

## Identity, schema, and interchange · `sec:phase5:impact`

**Identity.** No digest is minted anywhere in the batch. Attestation and the realization are unchanged. Constructor identity is the typed value's equality rather than a hash; bundle and ABI identity are likewise in-process candidate values. The safety, minimality, and resource report identities are schema identifiers, not content digests.

**Schema.** The typed schema work is the validated live-transfer compiler projection, the representation plans, owner metadata, the candidate live-receipt constructor, the candidate linked bundle, the candidate ABI, the validated operation report, the safety and minimality plans and reports, the first-party negative evidence report, and the ABI-validation report role. Three rendered report schemas are versioned in their own bytes — live-transfer safety at revision 2, minimality at 1, resource at 1 — alongside the plans report schema at 3. Safety revision 2 adds the operation-vocabulary-closed census line and its outstanding spelling; a revision-1 reader summing the census lines it knew would find them short of the row count, which is the misreading a stated schema turns into a refusal.

**Interchange.** The generated reports are terminal audit closures under ADR-022: they are rendered for a reader, not consumed by another producer, and nothing downstream parses them back into a decision. A report that acquired an external consumer would activate the rest of that posture.

## Security and production non-claims · `sec:phase5:non-claims`

Every key, scalar, blinder, and opening in the batch is public disposable test material in the sense ADR-015's test-material rule fixes, and is named as such at each use. No production private key, blinder, or opening is admitted anywhere, and section 1.10's refusal of a first-party production secret-bearing interface holds: there is no generation, no storage, and no constructor taking randomness from a source a production secret could come from.

Production multi-party signing and blinding is not established. The private construction is central public-fixture construction and says so in its own recorded model; describing it as multi-party blinding would be the overclaim the model exists to prevent.

The bytes standing in every signature position authorize nothing. They are of the right width so that a transaction can be serialized and weighed, and they are called unauthorizing at every use because no first-party component computes the message they would have to be over.

No dependency was added in the batch, so ADR-011's review has no new admission to consider.

## Residuals carried forward · `sec:phase5:residuals`

Named as a set in the evidence plan rather than as prose, so that a later wave clearing one has to remove it there. None is removed by this phase, because removal needs an observed result and none exists yet:

- the selected sighash profile is not established by the review, so every semantic claim about what the digest commits to is candidate-scoped;
- the sponsor envelope's authorizing signer is not wired into the evidence lane;
- no time-locked predecessor constructor exists, so the time-locked-input row has nothing to spend;
- no confidential predecessor can be funded;
- the internal key's unspendability is unverified against a target, and the residual discrete-log assumption on it stands regardless of what any probe observes.

## Handoff · `sec:phase5:handoff`

**Sequencing.** The intermediate confidential-funding guide closes first. Guide 14 — state and maturity — is drafted after it, and consumes the following without reopening any of them: validated target-operation plans, representation-specific backend planning, owner-bound constructor metadata, per-input owner authorization, finalized-output signing, explicit and private proof separation, deterministic linking, candidate ABI construction, validated operation reports, the safety and minimality report separation, exact executor provenance and environment binding, and candidate resource measurement. It adds state constructor continuity, root succession, operator authorization, maturity announcement, state-field commitments, and the first mutable protocol constructor. Phase 5 makes none of those state or root claims.

**Named follow-up work.** Four items, each investigated and each with a report behind it:

- **Shared abstract-program analysis.** Two private wrappers measure program prefixes — one in the conformance package, one in the vectors package — and both wrap the same abstract evaluator in tapscript. The first slice adds a public analysis API beside that evaluator and side-by-side tests against both wrappers, with no consumer deleted and no canonical output changed; the wrappers are retired only after equivalence is demonstrated, and the implementation is optimized only after that.
- **Sponsor envelope signing.** The adapter capability already exists — a typed sponsor authorization operation, a fixed regtest sponsor key, deterministic signing, and a response bound to the exact finalized transaction — and the cheapest honest first step has now been taken: one integration test finalizes an explicit sponsored control over really funded receipts and a really funded sponsor coin, sends its exact sponsor request, replays the returned witness through the sponsor capability, and verifies byte binding both by comparing the adapter's echo with the bytes it was sent and by observing that the same witness bound to one mutated byte is refused. The residual stays: capability existence never clears a blocker, only an observed result does, and no sponsor-signed control has an observed result, because that step signs and replays without submitting anything. The clause that used to name the owner sighash here is retired rather than kept — the profile is established over its required set and both lanes carry observed acceptances, so the sighash is no longer what stands in the way. What stands in the way is a submission, and a submission needs a reserve asset the chain has issued, where this deployment's is a fixture constant no chain knows.
- **Internal-key unspendability probe.** Phase A performs one key-path attempt against an already-funded explicit-constructor output, using the existing generic submission wire unchanged and recording exact bytes, the one-item witness shape, the funded outpoint and program binding, target provenance, and the verbatim refusal. A refusal discharges only that the attempt was observed and refused; it establishes nothing about who knows the discrete logarithm, so the residual assumption stands regardless. Phase B adds the typed carrier and the wire correction before the named residual may be called discharged for the instances covered.
- **The rest of the safety matrix.** Every remaining row is a target-boundary row, and the repair is the two blockers above rather than more first-party tests.

## Exit gate · `gate:phase5:exit`

Phase 5 exits when:

- every owner authorizes the finalized output set;
- exact live-class closure holds;
- all closed-asset-capable outputs are classified;
- explicit transfer passes complete safety evidence;
- confidential-value transfer passes where claimed;
- confidential sponsor values pass on an observed acceptance of their own shape, the private-sponsor-values row moving with it;
- safety and minimality reports remain distinct;
- lifecycle incompleteness is explicit;
- mixed-program vectors reject;
- linked bundle and ABI are deterministic;
- resource reports pass;
- relation coverage is complete and the tree remains clean.
