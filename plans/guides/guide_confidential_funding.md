# Draft: Intermediate Guide — Confidential Test Materialization and Funding, Execution

> **Status:** Draft execution guide; not yet executed; working document outside the plans census until closeout
> **Phase:** Phase 5 — Live Receipt Transfer; this guide completes the arc's private half
> **Category:** INTERMEDIATE GUIDE — an unnumbered category of its own; it must be CLOSED before Guide 14 is drafted
> **Concept of record:** [the confidential test materialization and funding concept](guide_confidential_funding_concept.md); its three accepted rulings are carried verbatim below and are never reopened here
> **Entry:** the reproduced `NoConfidentialPredecessorCanBeFunded` blocker, the recorded Phase-5 stopped result, and the three accepted charter rulings
> **Affected packages:** `target-elements-conformance`, `transaction`, `vectors`; the native Elements executor where it implements the first-party funding wire
> **Supersedes as execution direction:** the confidential-funding concept draft, and scalar-only funding as the route to Guide-13 private evidence
> **Does not implement:** the owner sighash, production wallets, production opening custody, production blinding, production signing or blinding coordination, privacy guarantees, final calibration, production deployment, or release
> **Required result:** one reviewed candidate-only protocol that creates, mines, reads back, spends, and evidences explicit-protocol-asset/confidential-value receipts with valid rangeproofs, empty surjection proofs, transaction-wide balanced value blinders, proof-bearing output witnesses finalized before owner signing, and honest evidence boundaries
> **Authority:** Attestation, the typed architecture, implemented ADRs, accepted decisions, and the concept's accepted rulings take precedence over this guide
> **Review basis:** the accepted concept, four lane-authored angle documents treated as proposals, and a direct reading of `packages/target-elements-conformance`, `packages/transaction`, and `packages/vectors` at tree `0.5.1-dev`; every identifier below is marked EXISTS or NEW against that reading
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015, and any selected secret-bearing interface must pass ADR-015's separate design gate before implementation

---

## Mission · `sec:guide-ctf-exec:mission`

This guide translates the accepted confidential-funding concept into exact Rust surfaces, wire records, file targets, refusal vocabularies, tests, and wave gates, so that a Guide-13 private receipt becomes creatable, spendable, and evidencable in the candidate pipeline without any claim the work has not earned.

It is an execution guide and not a second concept. Where the concept ruled, this guide carries the ruling and names the code that implements it; where the concept left a naming, a placement, or an external dependency open, this guide either resolves it against the repository and says how, or records it as a pending charter decision with options and a recommendation. Nothing is chosen silently.

The document is sized to six waves. It is deliberately smaller than Guide 13, because the arc it completes is one funding protocol, one materializer, one handoff, and one evidence restart — not a whole phase.

---

## One-line thesis · `rem:guide-ctf-exec:thesis`

> One accepted custody model, one fixture registry, one representation-tagged funding arm, and one transaction-wide materializer are enough to mine and read back the exact explicit-asset/confidential-value receipt form and freeze it before the separately reviewed owner sighash — and every report this guide produces must refuse to claim transfer, minimality, or production properties that those four things do not establish.

---

# 1. Governing rulings · `sec:guide-ctf-exec:rulings`

The three accepted rulings are reproduced first, in force, before any type is named. A wave that finds them inconvenient has found a defect in its own design, not in the ruling.

## 1.1 Custody is deterministic central public fixtures · `rule:guide-ctf-exec:custody`

**Ruling: ACCEPTED — deterministic central public fixtures.** One party holds every opening, every opening is a published test fixture, the material is disposable and destroyed with the chain, and nothing here generates, retains, or transports a production secret. The three nonrecommended models — disposable wallet-held, adapter-local opaque handles, and cooperative or distributed blinding — are CLOSED as custody directions for this guide and may not be reintroduced by a wave, a helper, or a test lane.

The model already has a typed name in the repository: `ConfidentialConstructionModel::CentralPublicFixtureConstruction` in `packages/transaction/src/live_private.rs` (EXISTS), which is also `ConfidentialConstructionModel::EXPECTED` and the only model `SelectedConstructionModel::record` admits. This guide does not mint a second spelling of the accepted model; it reuses that one, and any new custody profile type is a selection *within* it rather than beside it.

Wave 0 elaborates opening ownership, lifetime, lookup authority, diagnostics, production separation, and ADR-015 disposition. Those are elaborations of an accepted model, never a reopened choice.

## 1.2 Serialized form, not crate boundary, invokes ADR-022 · `rule:guide-ctf-exec:form-boundary`

The custody ruling carries one binding addition on form, and it is carried here word for word in effect: an asset this work makes or consumes as plain Rust types — an in-process, typed API surface — carries no interchange obligation; the moment it takes a serialized or archival form it is bound by ADR-022 under that ADR's own scope rules, including its ADR-010 exclusions for first-party command-line streams and the native executor protocol.

The criterion is the form and the consumer, not the crate, the filename, or whether the value is public. A fixture, opening, funding record, or report that exists as bytes to be stored or exchanged is an interchange document; the same value passed as a typed argument is not.

One closure is exempt: a terminal artifact generated with no consumer, existing purely for audit, does not invoke ADR-022. A serialized form becomes an interchange document at the moment something consumes it, not at the moment it is written. Phase 5 already classifies its rendered reports that way, and this guide inherits the classification rather than restating it.

## 1.3 Both reproducibility contracts are first-class · `rule:guide-ctf-exec:determinism`

**Ruling: ACCEPTED — both contracts, elegantly supported.** Neither contract is the revision of the other. The reproducibility contract is a typed, per-ceremony selection; one evidence schema carries the selected contract as a typed field rather than two parallel schemas; every validated report names the contract its run was under; a report under one contract can never claim the other's guarantees; and no run mixes contracts silently.

Byte identity is the REFERENCE contract. It is what the deterministic central public fixtures exist to serve, it satisfies the mandatory deterministic-public-fixture-openings row, and it is the contract the proven server rerun already demonstrates.

Recorded randomness is the contract under which wallet-held or adapter-held randomness is admissible. Its comparisons are semantic-fixture equality, retained per-run bytes, verified openings, and target projections. The external-wallet and adapter-handle *construction* models become reachable under it without any weakening of the byte-identity lane — and this is not a reopening of §1.1, because §1.1 rules who holds the openings for this guide's own ceremonies while this rule states which comparison a run may claim.

Elegance is a design obligation, not a preference. The two contracts share the ceremony, the wire protocol, the fixture registry, and the report vocabulary, and differ only where the contract genuinely differs. A wave that supports one contract by distorting the other has failed this rule even if its tests pass.

## 1.4 The canonical request is a public-fixture handle and digest · `rule:guide-ctf-exec:canonical-request`

**Ruling: ACCEPTED — the canonical public-fixture handle and digest.** The confidential funding request carries the case identity, the profiles, and a binding digest, and never an amount. The canonical report retains the exact request and the exact response unmodified. Validation binds handle, digest, profiles, request, response, and the mined transaction. An unknown handle is a construction refusal, before any cryptographic work.

Nothing is redacted and no existing evidence rule is revised. The two rejected alternatives — canonical redaction with a binding envelope, and noncanonical exact attachment — are CLOSED as evidence directions for this guide.

The handle is a public deterministic test identity and not an adapter-local secret handle. Its spelling encodes no amount, its digest detects drift, and lookup failure is a construction refusal rather than a target verdict.

## 1.5 The owner sighash is external parallel work · `rule:guide-ctf-exec:external-sighash`

This guide does not implement, select, review, or declare accepted the owner sighash. `LiveInfrastructureBlocker::OwnerSighashNotComputable` (EXISTS, `packages/vectors/src/live_evidence.rs`) and `LiveInfrastructureBlocker::SighashProfileUnreviewed` (EXISTS, same file) remain independently authoritative and are not cleared by anything in Waves 0 through 3.

Wave 4's entry condition is the independent acceptance of that separately reviewed profile. Until it is recorded, Wave 4 does not start, and a typed stopped result at the end of Wave 3 is a valid outcome of this guide.

Digest review and materializer development proceed in parallel. An explicit-only signing success does not establish proof-bearing signing, and a valid confidential funding transaction does not establish any owner digest.

## 1.6 Candidate vocabulary only · `rule:guide-ctf-exec:candidate-only`

Every type, profile, registry, record, and report this guide adds is candidate-only. No architecture operation, phase, release identity, or digest is minted. No interface is described as final, stable, production-capable, or released, and no wave may promote one by adding a version number to it.

The result of a wave that cannot reach its target is a typed stopped result carrying its blocker. A typed stopped result is valid; an overstated one is not.

## 1.7 Construction, infrastructure, and target verdicts stay distinct · `rule:guide-ctf-exec:failure-layers`

`ObservedOutcomeLayer` (EXISTS, `packages/target-elements-conformance/src/protocol.rs`) already separates the two non-verdicts — `FixtureConstructionFailure` and `ExecutorInfrastructureFailure` — from the four verdicts `ConsensusRejectionBeforeScript`, `ScriptPathRejection`, `RelayPolicyRejection`, and `Accepted`. This guide adds no second spelling of that distinction.

Every refusal this guide mints is a *construction* refusal or a *protocol* refusal, and never a target verdict. A pre-target refusal is never a target verdict, and a fixture-lookup failure is not evidence about a chain.

A refusal record may carry no funded observation. A refusal that carried an asset, an outpoint, a transaction identity, or raw mined bytes would be a verdict wearing a refusal's name.

## 1.8 Evidence roles never substitute for one another · `rule:guide-ctf-exec:role-separation`

Funding evidence, CT-conservation evidence, owner-signature evidence, safety evidence, minimality evidence, resource evidence, and lifecycle status are seven distinct roles. Each requires its own relation and its own validation. One ceremony may record several of them, and the summary must still preserve the separation.

Confidential funding capability is not a matrix pass. A blocker moves on an observed result and never on a capability existing — the rule the existing `SponsorEnvelopeSignerAbsent` documentation already states, applied unchanged.

## 1.9 One computation is never both observation and expectation · `rule:guide-ctf-exec:independent-origin`

The workspace already owns this rule and already paid for it. `packages/target-elements-conformance/Cargo.toml` records, at length, that adopting the same zero-knowledge secp256k1 bindings the target vendors would make an oracle and the thing it checks "one opinion wearing two hats", and that the first-party bignum commitment oracle exists precisely so the independence claim is unqualified.

This guide inherits that classification exactly:

| Origin | Owner | Claim class |
|---|---|---|
| Materialized commitment | the confidential proof materializer supplied to `transaction` | the construction's own output; never its own expectation |
| Recomputed commitment | `target_elements_conformance::commitment_oracle` (EXISTS) | first-party arithmetic over published constants; independence unqualified |
| Reference commitment | `secp256k1-zkp` / `elements` (EXISTS as workspace dependencies) | conformance-to-the-target's-own-implementation evidence, not independence |
| Read-back commitment | the native adapter's raw mined-transaction readback | the target's observation |

A comparison requires two different origins from that table. A comparison of a value with itself, of a value with a helper both sides call, or of a materialized value with a reference computed by the same library the materializer used, is not evidence and must be refused at the boundary that assembles it.

---

# 2. Entry conditions · `sec:guide-ctf-exec:entry`

## 2.1 Blocker entry · `gate:guide-ctf-exec:blocker-entry`

The guide starts only when the two Phase-5 typed blockers are reproduced rather than quoted: `LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded` is present in `carried_residuals()` and carried by the rows `derive_live_evidence_plan()` produces, and `OwnerSighashNotComputable` is present and unmoved by its own distinct mechanism — it is the positive-row blocker every positive safety row carries, not a member of `carried_residuals()` (Wave-0 precision).

Reproduction means running the existing evidence lane and reading the blocker census, not reading this paragraph.

## 2.2 Ruling entry · `gate:guide-ctf-exec:ruling-entry`

The three accepted rulings of §1.1 through §1.4 are recorded in the concept and are carried here. No wave may begin that requires a fourth ruling not recorded in §4.

## 2.3 Target entry · `gate:guide-ctf-exec:target-entry`

The target-side facts the concept verified at Elements tip `b7fc5d080a` are entry facts and are re-checked, not assumed, before Wave 2 submits anything:

| Fact | Elements location | Consequence for this guide |
|---|---|---|
| A confidential value paired with an explicit asset uses the unblinded asset generator, requires a rangeproof, and requires the surjection-proof field to be empty | `src/confidential_validation.cpp:368-403` | the exact target form of §7.1 |
| An empty or invalid rangeproof is rejected | `src/script/sigcache.cpp:131-163` | a rangeproof is mandatory, not optional |
| A rangeproof whose proven minimum is zero is rejected on a spendable output, and the target's own signer proves a minimum of one on spendable programs | `src/script/sigcache.cpp:157-164`, `src/blind.cpp:263-264` | the materializer proves a minimum of at least one on both protocol outputs; observed live by Wave 2's first submission bouncing on exactly this rule before the fix |
| Value-commitment prefixes `0x08` and `0x09` are admitted | `src/primitives/confidential.h:13-84`, `:129-154` | both parities are reachable and both must be exercised |
| `SIGHASH_ALL` incorporates the serialized output set and the hash of the output-witness vector | `src/script/interpreter.cpp:2414-2425`, `:2568-2578`, `:2741-2744` | proofs must be final before any owner signs |
| No reviewed stock RPC produces the hybrid form; the common blinding path draws fresh blinders, blinds the asset, and generates a surjection proof | `src/blind.cpp:557-627` | a first-party deterministic materializer is required |

Calling `blindrawtransaction`, retrying ordinary wallet blinding until an output happens to look useful, or mutating a fully blinded asset back to explicit after proof construction does not meet the charter and is a rejection condition, not a fallback.

## 2.4 Repository entry · `rule:guide-ctf-exec:repository-entry`

The tree is clean, the workspace builds, and the three affected packages' existing focused suites are green before Wave 1 changes a wire type. The dependency directions recorded in §3.3 are re-read from the three `Cargo.toml` files rather than assumed from this guide.

---

# 3. Repository ground truth · `sec:guide-ctf-exec:ground-truth`

Every identifier this guide names is marked EXISTS or NEW here, once, so that no later section has to be trusted about it.

## 3.1 What exists and is reused · `tab:guide-ctf-exec:existing`

| Identifier | File | Shape as of tree `0.5.1-dev` |
|---|---|---|
| `NATIVE_PROTOCOL_SCHEMA` | `target-elements-conformance/src/protocol.rs` | `u32 = 4` |
| `HandshakeRequest` | same | `{ schema: u32 }`, defaulting to the constant |
| `ExecutorHandshake` | same | carries `protocol_schema` and `capabilities: BTreeSet<ExecutorCapability>` |
| `ExecutorCapability` | same | thirteen variants including `TestFundingCeremony`, `TargetTransactionSubmission`, `ConfidentialConservation` |
| `OperationSubject` | same | `#[serde(untagged)]` enum of four boxed subjects: `Funding`, `Submission`, `SponsorFunding`, `SponsorSigning` |
| `TargetFundingSubject` | same | a STRUCT: `{ issue_asset: bool, asset: Option<String>, output_program: Vec<u8>, outputs: u8, amount_per_output: u64 }` |
| `FundedOutput` | same | `{ outpoint: WireOutpoint, asset: String, amount_satoshis: u64, script: String }` |
| `WireOutpoint` | same | `{ txid: String, vout: u32 }` |
| `NativeOperationRequest` / `NativeOperationResponse` | same | flat records with `schema`, `case`, and per-kind members |
| `ObservedOutcomeLayer` | same | two non-verdicts and four verdicts |
| `OperationCaseId` | same | `{ operation: OperationStepKind, step: String }` |
| `commitment_oracle` | `target-elements-conformance/src/commitment_oracle/` | first-party bignum commitment arithmetic: `commitment`, `commitment_point`, `blinded_asset_generator`, `parse_commitment`, `serialized_asset_generator`, `COMMITMENT_PREFIX_BASE`, `SEMANTIC_AMOUNT_BOUND` |
| `ValidatedNativeConformanceReport` | `target-elements-conformance/src/validate.rs` | precedent for a validated record whose only constructor is a validator |
| `TargetTransaction` | `transaction/src/bytes.rs` | `{ version, inputs, outputs, lock_time, witnesses }`; `encode`, `encode_without_witness`, `decode` |
| `TargetOutput` | same | `{ asset: AssetField, value: ValueField, nonce: NonceField, program: Vec<u8> }` — no output-witness member |
| `ValueField` / `AssetField` / `NonceField` | same | `Explicit(u64)` or `Commitment([u8; 33])`; `Null` for the nonce |
| `COMMITMENT_BYTES`, `NULL_PREFIX` | same | `33`, `0x00` |
| `PrivateValueCapability` | `transaction/src/live_private.rs` | per-output trait returning `Option<[u8; 33]>`; the role the concept replaces |
| `ConfidentialConstructionModel`, `SelectedConstructionModel`, `PrivateConstructionNonClaim` | same | the accepted custody model and its typed non-claims |
| `PublicTestRandomness`, `ProtocolValue`, `LiveReceiptDestination` | `transaction/src/live_request.rs` | the existing public-fixture and value types |
| `finalize_live_transfer`, `LiveFinalization`, `CandidateLiveTransferTransaction` | `transaction/src/live_construct.rs` | the construction entry point and its results |
| `FinalizedLiveTransfer`, `LiveSigningRequest` | `transaction/src/live_finalize.rs` | private fields, crate-only constructor, `protected_bytes`, `signing_request`, `check_offered` |
| `ProtectedDatum` | `tapscript/src/authorization.rs` | fifteen members including `ProofFields` and `ExplicitValuesOrCommitments` |
| `SighashDimension` | `target-elements/src/authorization.rs` | ten dimensions including `AllOutputs` and `SpentOutputs` |
| `LiveInfrastructureBlocker`, `carried_residuals`, `blocker_census`, `derive_live_evidence_plan` | `vectors/src/live_evidence.rs` | the blocker vocabulary and evidence plan |
| `LiveSafetyRow`, `required_safety_matrix`, `rows_of` | `vectors/src/live_safety.rs` | the safety matrix this guide's Wave 5 restarts against |
| `secp256k1-zkp`, `elements` | workspace dependencies | adopted reference-implementation oracles; already in the lock file |

## 3.2 What changes · `tab:guide-ctf-exec:changes`

| Change | File | Why it is a change and not an addition |
|---|---|---|
| `NATIVE_PROTOCOL_SCHEMA` 4 → 5 | `protocol.rs` | the confidential arm adds an untagged `OperationSubject` variant and a non-defaulted response member, so a revision-4 executor cannot parse or produce a revision-5 exchange |
| `OperationSubject` gains a fifth variant | `protocol.rs` | the concept's "distinct arm", expressed in the enum that already carries the arms |
| `NativeOperationResponse` gains one confidential member | `protocol.rs` | the response arm the request arm must match |
| `TargetTransaction` gains an output-witness census | `transaction/src/bytes.rs` | today the encoder writes two `NULL_PREFIX` bytes per output unconditionally and the decoder refuses any nonempty proof |
| `TargetTransaction::has_witness` | same | today it is `witnesses.iter().any(!is_null)`; a confidential funding transaction may have null input witnesses and nonempty output witnesses, and would serialize without the section that carries its proofs |
| `TransactionRefusal::SuperfluousWitnessRecord` condition | same | today an all-null input-witness section is refused as superfluous; with nonempty output witnesses it is required |
| `TransactionRefusal::RangeProofRefused` / `SurjectionProofRefused` | `transaction/src/error.rs` | both currently fire on any nonempty proof field; they must become form-conditional, refusing a rangeproof where the profile forbids one and a surjection proof always for the hybrid form |
| `FinalizedLiveTransfer::protected_bytes` | `transaction/src/live_finalize.rs` | today it is `encode_without_witness()`, which omits the output-witness vector the target's `SIGHASH_ALL` covers — the empty-vector case `G11-W11-06` already diagnosed |
| `ProtectedDatum::ProofFields` dimension mapping | `tapscript/src/authorization.rs` | today it maps to `SpentOutputs` alone, naming only the input side's anchoring; the created outputs' proof fields are covered too |
| `PrivateValueCapability` consumers | `transaction/src/live_construct.rs` | the per-output role is retired from private complete-transaction claims |

The last four rows are PREFLIGHT FINDINGS of this guide's own reading and are recorded as such in §12 Wave 0. They are stated as hypotheses to reproduce, not as established defects: each is a claim about the tree at `0.5.1-dev` that a wave must confirm by running code before repairing. The `protected_bytes` row is the exception in one respect only — the target behaviour it rests on is already established by the runnable diagnosis at `scripts/diagnose-taproot-output-witness-digest.py`, so what Wave 0 reproduces there is the first-party half.

Wave 0 confirmed all four by running code (T5-020), and three precisions from that wave bind the repairs. The proof-fields repair of §9.3 is a TYPE change, not a value edit: coverage becomes set-valued, the public `carrier` signature moves with it, and the totality argument in `selected_owner_profile` moves too. The protected-bytes repair's serialization parts are strictly downstream of the output-witness census row above, which must land first — Wave 3 orders them so. And the two proof-field refusals are asymmetric in repair: for the hybrid form the surjection refusal is already the required behaviour, and only the rangeproof half becomes form-conditional.

## 3.3 Dependency directions, which are load-bearing · `rule:guide-ctf-exec:dependency-directions`

Read from the three manifests rather than assumed:

- `target-elements-conformance` library depends on `cli-common`, `tapscript`, `target-elements`, and third-party crates. It does NOT depend on `transaction` or `vectors` in its library graph; both are dev-dependencies only, and the manifest states that nothing there may be promoted without revisiting the boundary.
- `transaction` depends on `linker`, `sha2`, and `target-elements`. Its manifest states that `target-elements-conformance` is deliberately absent and that the absence is load-bearing, because that package is the independent oracle its output is compared against.
- `vectors` depends on both `transaction` and `target-elements-conformance`, and is therefore the one library where both are reachable. Its manifest records that what crosses the conformance edge is the executor boundary and nothing else, and that no expectation crosses.

Three consequences follow and are binding on every wave:

1. the transaction-wide materializer lives in `transaction` and cannot call the commitment oracle directly; its cryptographic collaborators are TRAITS defined in `transaction` and implemented outside it, exactly as `PrivateValueCapability` and `LiveCurveCapability` already are;
2. the independent commitment recomputation is the existing `commitment_oracle` in `target-elements-conformance`, and the comparison of a materialized commitment against a recomputed one is assembled where both are reachable;
3. no wave adds a library edge between `transaction` and `target-elements-conformance` in either direction. A wave that finds it needs one has found a design error in its own placement.

## 3.4 New dependencies · `rule:guide-ctf-exec:new-dependencies`

None is required, and this is a finding rather than an assumption. Rangeproof generation and verification are already reachable through `secp256k1-zkp`, an existing workspace dependency with an existing first-party consumer and an existing lock-file entry, adopted as a reference-implementation oracle.

That adoption fixes the claim class and this guide does not widen it. A rangeproof generated with `secp256k1-zkp` and accepted by an Elements node is conformance-to-the-target's-own-implementation evidence. It is not independent evidence, and no report may describe it as independent. The independent claim in this guide belongs to the commitment arithmetic, where the first-party oracle owns it.

ADR-011 therefore has no new admission to consider. A wave that reaches for a new cryptographic crate has left the guide and must stop and file a charter decision.

---

# 4. Pending charter decisions · `sec:guide-ctf-exec:pending-decisions`

Each entry below is a question this guide could not close from the concept or the repository. Each carries options, a recommendation, and an explicit pending ruling, in the concept's own format. A wave may not proceed past the boundary an entry names until its ruling is recorded.

The seventeen questions the angle documents raised are answered in full in §20; the four that could not be closed appear here.

## 4.1 The accepted-result type the external sighash work exposes · `rule:guide-ctf-exec:pending-sighash-result`

Wave 4's entry condition is that the parallel owner-sighash profile has reached its own accepted result. What that result *is*, as a value this guide's handoff can bind to, belongs to that work and not to this one.

| Option | What this guide would consume | Consequence |
|---|---|---|
| An accepted-profile token plus opaque authorization bytes | a typed marker that the profile is accepted, and a byte string per owner | matches the existing `LiveSigningRequest` shape, which already carries a preimage and never a digest; smallest coupling |
| A digest-producing capability trait | a trait this guide's handoff calls to obtain the message | imports the digest design into this guide, which §1.5 forbids |
| A fully authorized candidate handed back | the other work returns a signed candidate | moves finalization out of this guide and blurs the ownership the concept fixed |

**Recommendation.** The first. The handoff hands out protected bytes and target spent-output data and takes back opaque authorization bytes bound to those exact protected bytes, which is the boundary `LiveSigningRequest` and `FinalizedLiveTransfer::check_offered` already implement for the explicit lane. Nothing about the digest crosses.

**Ruling: ACCEPTED — option B of the owner-sighash concept's accepted-result decision.** The ordered deepening produced the chartered concept at plans/guides/guide_owner_sighash_concept.md, whose accepted-result entry supersedes this one's three options with five and shows the first option above not implementable as written: `LiveSigningRequest` carries no spent-output data, genesis hash, output-witness vector, leaf hash, codeseparator position, or annex disposition, and for the proof-bearing lane the output-witness omission is unrecoverable at any length. The accepted result is protected bytes, the signing-input census, and opaque authorization bytes plus the returned hash-type byte, exact-byte bound to one candidate identity; the census carries observed target data plus candidate structure and never an opening, blinder, nonce input, key, or proof input. Wave 4 consumes that concept's Wave-5 handoff and nothing narrower.

## 4.2 The ADR-022 allocation for a consumed funding record · `rule:guide-ctf-exec:pending-adr022`

Under §1.2 the funding record and the fixture catalogue are plain typed values in process, and become interchange documents at the moment something consumes their serialized form. Within Waves 0 through 5 no consumer exists: the rendered outputs are terminal audit closures on the Phase-5 pattern.

| Option | Consequence |
|---|---|
| Declare no allocation now; record the trigger and classify the rendered forms as terminal audit closures | matches Phase 5's existing classification and the accepted exemption; costs nothing until a consumer appears |
| Allocate an ADR-022 namespace, version, canonical encoding, theory, and decoder now | pays the whole cost for a consumer that does not exist, and mints a schema identity §1.6 forbids |
| Allocate only a reserved namespace | a half-allocation is a claim that a document exists; the register would carry a name nothing writes |

**Recommendation.** The first, with the trigger stated in the guide and enforced by a test: any code path that parses a rendered funding record back into a decision activates the full ADR-022 posture before it merges.

**Ruling: ACCEPTED — no allocation now, with the enforced trigger.** Rendered funding records are terminal audit closures on the Phase-5 classification; the trigger stands in this guide and is enforced by a test: any code path that parses a rendered funding record back into a decision activates the full ADR-022 posture before it merges. The other two options are closed as directions for this guide, and the closeout disposition is no longer provisional.

## 4.3 Recorded-randomness bootstrapping · `rule:guide-ctf-exec:pending-recorded-bootstrap`

Under byte identity the fixture manifest binds openings, so handle and digest can be registered before the run. Under recorded randomness the openings do not exist until the run produces them, yet §1.4 requires the request to carry a handle and a digest and nothing else.

| Option | What the digest binds under recorded randomness | Consequence |
|---|---|---|
| Semantic-only transcript | version, framed handle, profile codes, roles, amounts, programs, asset, and fixed order; no opening, no blinder, no counter | one transcript function with a contract-tagged domain and a contract-determined member set; the request stays handle-and-digest-only; per-run openings land in the run attachment |
| Two-stage digest | a pre-run semantic digest and a post-run opening digest, both in the record | two digests in one field's place, and a report that names one could be read as the other |
| Recorded randomness deferred out of this guide | one contract implemented, one designed | contradicts §1.3, which requires both designed together and forbids bolting one on afterwards |

**Recommendation.** The first. It preserves the accepted single shared field, keeps central custody intact, and keeps the request free of amounts and openings; the contract tag in the transcript domain is what stops a semantic-only digest being mistaken for a byte-identity one.

**Ruling: ACCEPTED — the semantic-only transcript.** The recorded-randomness digest binds the semantic transcript with the contract tag inside the domain; per-run openings land in the run attachment and are checked by the four comparison surfaces; the request stays handle-and-digest-only. The other two options are closed, and Wave 1 may serialize recorded-randomness requests under this ruling.

## 4.4 Where the reproducibility contract is selected · `rule:guide-ctf-exec:pending-contract-selection`

§1.3 fixes that the contract is a per-ceremony typed selection carried unchanged. It does not fix who selects it, and two candidates are defensible.

| Option | Consequence |
|---|---|
| The ceremony plan in `vectors` selects it and the request carries it | the evidence lane owns what its own report will claim; the executor never chooses the contract it is judged under |
| The executor advertises supported contracts and the harness picks the intersection | an executor that supported only recorded randomness could silently move a run off the reference contract |

**Recommendation.** The first, with the executor advertising which contracts it supports so that an unsupported selection is refused before construction rather than downgraded after it. Advertisement constrains; it does not choose.

**Ruling: ACCEPTED — the ceremony plan selects.** The ceremony plan in `vectors` selects the contract and the request carries it; the executor advertises supported contracts, and an unsupported selection is a typed refusal before construction. Advertisement constrains and never chooses. The intersection alternative is closed.

---

# 5. The funding wire · `sec:guide-ctf-exec:wire`

## 5.1 The confidential arm is a fifth operation subject · `rule:guide-ctf-exec:arm-placement`

The concept fixes that the confidential member is a distinct arm and not a boolean beside `amount_per_output`. This guide fixes where that arm lives, and the answer is not the one the wire angle proposed.

`TargetFundingSubject` is not re-shaped into an enum. It is already a variant of `OperationSubject`, which is `#[serde(untagged)]` for a stated reason: the kind is in the case identity, and a second discriminator could disagree with the first. Making the funding subject itself a tagged union would put a second discriminator inside a variant of an untagged enum, which is precisely the shape that file refuses.

The distinct arm is therefore a fifth `OperationSubject` variant, with its own step kind and its own capability, exactly as the sponsor steps were added:

```text
OperationStepKind::FundConfidential                      NEW
ExecutorCapability::ConfidentialValueTestFunding         NEW
OperationSubject::ConfidentialFunding(Box<TargetConfidentialFundingSubject>)   NEW
```

`OperationSubject::kind` and `OperationSubject::required_capability` gain the two arms, so the correspondence between subject, kind, and capability continues to be stated in exactly one place.

Untagged deserialization stays unambiguous because the member sets are disjoint under `deny_unknown_fields`: a confidential subject carries `destinations` and `binding`, which no other subject declares, and declares no `output_program`, `outputs`, or `amount_per_output`. That disjointness is a correctness property and gets its own test, not a comment.

## 5.2 Request types · `def:guide-ctf-exec:request-types`

All NEW, in `packages/target-elements-conformance/src/protocol.rs`, all `#[serde(deny_unknown_fields)]`:

```text
TargetConfidentialFundingSubject
    issue_asset: bool
    asset: Option<String>
    destinations: Vec<ConfidentialFundingDestination>
    binding: ConfidentialFundingBinding

ConfidentialFundingDestination
    output_program: Vec<u8>

ConfidentialFundingBinding
    fixture_handle: ConfidentialFixtureHandle
    fixture_digest: ConfidentialFixtureDigest
    profiles: ConfidentialFundingProfiles

ConfidentialFundingProfiles
    representation: FundingRepresentationProfile
    custody: FundingCustodyProfile
    materializer: FundingMaterializerProfile
    reproducibility_contract: ReproducibilityContract
```

`issue_asset` and `asset` keep the existing explicit arm's meaning and its invariant — the asset is absent exactly when the step issues it — because the protocol asset question is identical in both arms and a second spelling of it would be a second chance to get it wrong.

The request carries no amount, no count, no opening, no blinder, and no nonce or proof input. The destination count is the destination list's length, and what each destination holds is a fact of the registered fixture, not of the request. This is §1.4 expressed in a type: there is no member an amount could be written into.

`FundingRepresentationProfile` (NEW) has one variant in this guide, `ExplicitAssetConfidentialValue`, naming the exact hybrid tuple of §2.3. `FundingCustodyProfile` (NEW) has one variant, `CentralPublicFixtures`, which is the accepted model of §1.1. `FundingMaterializerProfile` (NEW) has one variant, `GuideCtfDeterministicV1`. Each is a closed enum with one member rather than a bare marker, so that a second profile is an added variant a peer must advertise rather than a silent change of meaning.

## 5.3 Response types · `def:guide-ctf-exec:response-types`

`NativeOperationResponse` (EXISTS) gains two members and keeps every existing one unchanged:

```text
confidential_funded_outputs: Vec<ConfidentialFundedOutput>     NEW
mined_readback: Option<MinedFundingReadback>                   NEW
```

Neither is `#[serde(default)]`, and that is deliberate. The sponsor witness members were defaulted precisely so that adding them was not a revision; these are not defaulted precisely so that adding them IS one, which is what §1.4's "confidential requests require the new schema" asks for and what keeps a revision-4 executor from answering a confidential request with silence in the new members.

```text
ConfidentialFundedOutput                                       NEW
    outpoint: WireOutpoint
    explicit_asset: String
    value_commitment: [u8; 33]
    nonce: [u8; 33]
    script: String
    output_witness_index: u32
    surjection_proof: Vec<u8>
    rangeproof: Vec<u8>

MinedFundingReadback                                           NEW
    transaction_id: String
    witness_transaction_id: String
    block_hash: String
    block_height: u32
    raw_transaction: Vec<u8>
```

Two spellings are inherited from the existing `FundedOutput` rather than improved on: the script comes back as a `String` in the target's own rendering, and the asset comes back as a `String` in the target's own spelling. The wire angle proposed byte vectors for both; adopting that would give the two arms two different renderings of one target fact, and a comparison across arms would then be a comparison of two authored spellings.

The proof fields are declared in the target's own serialization order — surjection proof first, then rangeproof — so that a reader of the record and a reader of the raw bytes are reading the same order. For the hybrid form the surjection proof is always empty and the rangeproof is never empty, and both are still declared, because an absent field cannot be observed to be empty.

The angle document's `TargetFundingResponse`, `FundingOutcome`, `FundingResult`, `FundingAcceptance`, `ExplicitFundedOutput`, `FundingRefusalResponse`, and `FundingRefusalLayer` are DROPPED. `NativeOperationResponse` already carries schema, case, and observed layer; `ObservedOutcomeLayer` already separates non-verdicts from verdicts; and `FundedOutput` already is the explicit arm's output record. Adding that parallel tree would restructure the explicit arm the concept says keeps its original schema, and would mint a second layer vocabulary beside the one the workspace already validates against.

## 5.4 Negotiation and revision 5 · `rule:guide-ctf-exec:negotiation`

`NATIVE_PROTOCOL_SCHEMA` becomes `5`, and the two implementations — this harness and the reviewed native adapter — bump together in one change. A revision-4 executor is refused at the handshake exactly as revision 3 was refused for revision 4, and for the same recorded reason: a revision only one side moved to reproduces the two-sided disagreement the revision mechanism exists to end.

There is ONE binary and one schema constant. No compatibility entry point, no dual-vocabulary serializer, and no per-record revision negotiation. The concept's "old executors receive only their old explicit schema" is honoured by the fact that a revision-4 executor is handed nothing at all under revision 5, which is a stronger guarantee than translating for it would be; and "compatibility may translate old explicit requests only to the explicit arm, never confidential requests backward" is honoured vacuously, because no translation exists to be misused.

`ExecutorHandshake` (EXISTS) gains one member:

```text
confidential_funding: Option<ConfidentialFundingAdvertisement>   NEW

ConfidentialFundingAdvertisement                                 NEW
    representation_profiles: BTreeSet<FundingRepresentationProfile>
    custody_profiles: BTreeSet<FundingCustodyProfile>
    materializer_profiles: BTreeSet<FundingMaterializerProfile>
    reproducibility_contracts: BTreeSet<ReproducibilityContract>
```

The advertisement is `Option` and absent by default, so the capability and the advertisement cannot disagree: an executor advertising `ConfidentialValueTestFunding` without the record, or the record without the capability, is refused at the handshake. Every selected profile must appear in the advertised set BEFORE the request is written. Advertisement constrains the selection and never makes it (§4.4).

Unknown tags, unknown profiles, and undeclared members fail strict framing before construction, with no ignore, no default, and no fallback. An untagged record under revision 5 is unknown, not explicit.

## 5.5 Typed wire refusals · `def:guide-ctf-exec:wire-refusals`

One closed enum, `ConfidentialFundingRefusal` (NEW), no catch-all variant, every variant a construction or protocol refusal and never a target verdict (§1.7). The concept requires at least eleven distinctions; the table marks them.

| Variant | Payload | Concept-required | Meaning |
|---|---|---|---|
| `ProtocolSchemaUnsupported` | `{ required, offered }` | | the peer does not speak revision 5 |
| `ConfidentialFundingCapabilityAbsent` | | yes | the capability is not advertised |
| `CapabilityAdvertisementDisagrees` | | | capability and advertisement record contradict each other |
| `HybridRepresentationUnsupported` | | yes | the exact hybrid tuple is not advertised |
| `UnknownRepresentationTag` | `{ tag }` | yes | an unassigned arm tag |
| `FundingProfileUnsupported` | `{ kind, name }` | yes | a profile unknown or unadvertised |
| `UnknownWireMember` | `{ record, member }` | yes | strict framing failure |
| `UnknownFixtureHandle` | `{ handle }` | yes | registry lookup failed |
| `FixtureDigestMismatch` | `{ handle }` | yes | the registered digest differs |
| `DestinationSetEmpty` | | | a confidential request with no destination |
| `DeterministicMaterializationRefused` | `{ cause }` | yes | the materializer refused, with its own typed cause |
| `ResponseArmMismatch` | `{ requested, returned }` | yes | including explicit fallback from a confidential request |
| `ResponseFixtureBindingMismatch` | | | handle or digest differs from the request |
| `ResponseProfileBindingMismatch` | | | profile binding differs from the request |
| `OutputCountMismatch` | `{ requested, observed }` | | |
| `OutputProgramMismatch` | `{ index }` | | |
| `ExplicitAssetMismatch` | `{ index }` | | the explicit protocol asset differs |
| `ConfidentialAssetReturned` | `{ index }` | yes | an asset commitment came back |
| `ExplicitValueReturned` | `{ index }` | yes | a scalar came back |
| `CommitmentEncodingInvalid` | `{ index }` | | not an admitted prefix or not a point |
| `NonceEncodingInvalid` | `{ index }` | | absent or profile-invalid |
| `RangeproofMissing` | `{ index }` | yes | the field is empty |
| `RangeproofInvalid` | `{ index }` | yes | independent verification failed |
| `SurjectionProofUnexpected` | `{ index }` | yes | the field carries bytes |
| `DuplicateFundedOutpoint` | `{ outpoint }` | | one coin reported twice |
| `RawTransactionDecodeFailed` | | | the readback bytes do not decode |
| `MinedInclusionUnverified` | | | the named block does not contain the transaction |
| `MinedReadbackMismatch` | `{ fact, index }` | yes | a projection differs, with the specific variants above taking precedence |
| `RefusalCarriesFundedObservation` | | | a refusal carried an asset, an outpoint, an identity, or raw bytes |

`ExplicitValueReturned` and `ConfidentialAssetReturned` are separate variants and not one "wrong form" variant, because they are the two halves of the hybrid tuple and a run that confused them would have learned the opposite of what it reported.

The angle documents spelled the fixture refusals three different ways — `FixtureHandleUnknown`, `UnknownFixtureHandle`, and `FixtureUnknown`. One spelling is fixed here, `UnknownFixtureHandle`, matching the adjective-first shape the existing protocol refusals already use.

## 5.6 Readback binding · `req:guide-ctf-exec:readback-binding`

The response binds nothing by echo. Every output fact in the validated record is derived from decoding `raw_transaction` and from an independent target readback, and a request echo can never stand in for either. Five conditions, all recomputed:

1. the transaction identity and the witness transaction identity recomputed from the decoded bytes equal the reported identities and the outpoints' transaction, and the reported block contains that transaction at the reported height;
2. the response position, the output index, and the output-witness index name one unique output, with the count and the order exactly as the fixture fixed them;
3. the decoded asset field is the explicit protocol asset, the decoded value field is the exact commitment reported, and the nonce field and the output program are byte-equal to the reported ones;
4. the rangeproof is byte-equal to the reported one, is nonempty, and verifies against that commitment, the unblinded asset generator, and the output program; the surjection proof is byte-equal and empty;
5. the arm, the handle, the digest, and the profiles equal the request and the advertisement, while every output fact stays readback-derived.

The distinction between (5) and (1) through (4) is the whole binding: what the caller asked for is checked against what the caller sent, and what the chain holds is checked against the chain.

## 5.7 Canonical request evidence · `rule:guide-ctf-exec:canonical-request-evidence`

The serialized confidential request carries the asset choice, the ordered destination programs, the public handle, the digest, and the profiles. It carries no amount and no opening, and the handle's spelling encodes neither (§6.2).

Resolution happens before materialization, through the registry of §6.1, and an unknown handle or a drifted digest is a construction refusal at that point. Nothing downstream may repair, default, or retry a lookup.

Validation binds handle, digest, profiles, the exact request, the exact response, the raw transaction, the mining observation, and the representation. Nothing is redacted, and the exact request and response are retained unmodified, which is what makes this ruling compatible with Guide-13's existing exact-request evidence rather than a revision of it.

The native pipe carrying these records remains an ADR-010 exclusion. A separately stored fixture catalogue or opening attachment is not the pipe and is classified by §1.2 on its own form and consumer.

---

# 6. Public fixtures, custody, and derivation · `sec:guide-ctf-exec:fixtures`

## 6.1 Registry and its authority · `def:guide-ctf-exec:fixture-registry`

The registry is the only thing with lookup authority. No environment value, no wallet, no request member other than handle and digest, and no response echo may resolve a fixture.

```text
ConfidentialFixtureManifest      NEW    the complete public description of one case
ConfidentialFixtureRegistry      NEW    register(manifest) -> Result<(), RegistrationRefusal>
                                        freeze() -> FrozenConfidentialFixtureRegistry
FrozenConfidentialFixtureRegistry NEW   resolve(&handle, &digest) -> Result<&ResolvedFixture, ...>
ResolvedFixture                  NEW    ordered roles, amounts, programs, asset, order, and openings
```

Freezing is a type transition and not a flag. Registration is possible only before the freeze and lookup only after it, so a late write is a compile-time impossibility in the common case and a typed refusal in the dynamic one. Only the frozen registry supplies openings, and it supplies them to the materializer and to nothing else.

Registration validates handle grammar, material class, manifest closure, exhaustive and disjoint roles, positive semantic amounts, the explicit protocol asset on every protocol member, bounded counters, opening well-formedness where the contract carries them, and the digest. A duplicate handle is refused as `DuplicateFixtureHandle` even when the two manifests are equal, because a registry that silently accepted an equal re-registration would accept an unequal one on the day the two drifted.

Every value in the registry is public disposable test material in the sense of ADR-015's test-material rule (its "Disposable test-network material" clause), and the class is a typed member of the manifest rather than a claim in a doc comment — `PublicDisposableTestMaterial` (NEW) is this guide's name for that class, not the ADR's (Wave-0 precision). The existing `PrivateConstructionNonClaim::NoOpeningIsSecret` (EXISTS) states the same fact on the construction side, and the two are held equal by a test rather than by intent.

`FixtureDiagnostic` (NEW) may expose handle, digest, role, counter, and refusal kind, and may expose nothing else — no amount, no opening, no derivation material, no attachment, and no free text. Diagnostics are subject to the canonical exclusions of §10.3 like every other output.

## 6.2 Handle identity · `def:guide-ctf-exec:fixture-handle`

```text
ConfidentialFixtureHandle(String)     NEW, validated newtype
ConfidentialFixtureDigest([u8; 32])   NEW
```

The grammar is fixed: `"ctf-v1/"` followed by a case name of three to sixty-three lowercase letters and interior hyphens, ending in a letter, with no digit and no adjacent hyphens. The one digit in the whole spelling is the grammar's version, which is what lets a later grammar be a different prefix rather than a reinterpretation of this one.

The case name encodes no amount, no unit, no asset, no opening, no derivation value, no digest fragment, no retry result, and no transaction identity. The first slice's handle is `ctf-v1/predecessor-dual-parity`.

A handle is a public deterministic test identity. It is not a capability, not a secret, and not an architecture or release identity, and no digest minted here is one either (§1.6).

## 6.3 Digest and drift · `rule:guide-ctf-exec:fixture-digest`

The digest is a tagged SHA-256 over a framed transcript, under the tag `tripod/guide-ctf/fixture-digest/v1`, computed with the workspace's existing `sha2` dependency and no new one.

The transcript binds, in fixed order and with big-endian framing: the grammar version, the framed handle, the contract tag, the profile codes, the retry limit, the chosen counter where the contract has one, the output count, and then each output in fixed order. Each output binds its role, the explicit protocol asset, the output program, and — under byte identity only — the semantic amount, the value blinder, the nonce input, the proof seed, and the resulting commitment prefix.

The contract tag is inside the transcript rather than beside it. That is what makes a semantic-only recorded-randomness digest impossible to mistake for a byte-identity one, and it is why §4.3's recommendation costs one field rather than a second digest.

Unknown codes, trailing bytes, duplicate roles, wrong lengths, and reordering all refuse. Registration and lookup both recompute the digest; neither repairs, truncates, nor falls back.

The transcript is an ephemeral hash preimage and a plain typed value. Storing or exchanging it changes its form and brings §1.2 into play.

## 6.4 Deterministic derivation · `rule:guide-ctf-exec:derivation`

Under byte identity, every scalar the ceremony needs is derived, and the derivation is domain-separated by role and by case.

```text
FixtureDerivationProfile::GuideCtfV1      NEW
tag: "tripod/guide-ctf/derive/v1"
roles: ValueBlinder | NonceSecret | RangeproofSeed
asset blinders: fixed at zero, never derived
```

Each preimage frames the case identity, the output index, the derivation role, the parity counter, and the scalar counter, all as big-endian integers with explicit lengths, so that no two preimages can collide by concatenation.

A value-blinder or nonce hash is read as a big-endian scalar and must be nonzero and below the group order; `commitment_oracle::read_scalar` (EXISTS) already implements exactly that reading and its `ScalarDefect` vocabulary, and the derivation reuses it rather than restating the bound. A rangeproof seed uses all thirty-two hash bytes and is not a scalar. Placement (Wave-0 precision): `read_scalar` lives in `target-elements-conformance`, which `transaction` may never reach by a library edge (§3.3), so the reuse is assembled where both are reachable — the §8.8 pattern — and a derivation implemented inside `transaction` takes the reading through a trait implemented outside it, never through the forbidden edge.

Bounds are constants and are part of the contract: `MAX_PARITY_COUNTER: u16 = 4095` and `MAX_SCALAR_COUNTER: u8 = 255` (both NEW). Search moves upward from zero without wrapping, without skipping, without randomness, and without concurrency. Exhaustion is a typed refusal and never a retry with a different source.

One output per transaction is marked `Balancing` by the fixture and the rest are derived. The balancing blinder is then solved as the confidential input blinder sum minus the other outputs' blinder sum, modulo the group order. For the first predecessor the input contributes zero, output zero is `Primary`, output one is `Balancing`, and the two blinders are ordered additive inverses — which is the concept's opposite-sum requirement expressed as an instance of the general rule rather than as a special case.

The parity search runs the parity counter from zero until the fixed-order serialized commitment prefixes are exactly `0x08` then `0x09`. `target_elements::ConfidentialFieldEncoding::admits_prefix` and `committed_prefixes` (both EXIST) are the target-side authority on which prefixes are admitted, and the search compares against them rather than against a literal written here twice.

Order is fixed by the fixture. It is never fixed by a commitment prefix, an amount, or the order a retry happened to finish in.

## 6.5 Derivation refusals · `def:guide-ctf-exec:derivation-refusals`

`FixtureDerivationRefusal` (NEW), closed, no catch-all:

| Variant | Payload | Meaning |
|---|---|---|
| `ScalarSearchExhausted` | `{ role, attempts }` | the scalar counter reached its bound for one role |
| `DegenerateBalancingScalar` | | the solved balancing blinder is zero or out of range |
| `BlinderBalanceMismatch` | | the independent recheck of the two sums disagreed |
| `IdentityValueCommitment` | `{ output }` | a commitment landed on the group identity |
| `ParitySearchExhausted` | `{ attempts }` | the parity counter reached `MAX_PARITY_COUNTER` without the required prefix pair |
| `RangeproofConstructionFailed` | `{ output }` | proof generation refused; no randomness is added and no retry follows |

The last variant's rule is the one that matters most: a proof failure never becomes a source of entropy. Under byte identity there is exactly one deterministic answer, and if it does not work the ceremony refuses rather than searching until it does.

## 6.6 One reproducibility field · `def:guide-ctf-exec:reproducibility-field`

```text
ReproducibilityContract   NEW, closed: ByteIdentity | RecordedRandomness
```

Placement is fixed here and the reasoning is a dependency fact. The contract is named by the wire record in `target-elements-conformance`, by the materializer profiles in `transaction`, and by the evidence record; those two packages share no library edge in either direction, and the only package both already depend on is `target-elements`, which carries no dependencies at all and already owns `confidential.rs`. The enum therefore lives in `target-elements` as a candidate ceremony vocabulary, and both sides name the same value.

The alternative — one enum in each package, held equal by a census test — is two authored spellings of one closed vocabulary, which is the defect the workspace's typed-source rule exists to prevent. It is recorded here as the fallback if the owner declines to widen `target-elements`, and it is the weaker option.

The widening is explicit, never silent (the §16.2 pattern, applied here by Wave-0 finding): the `target-elements` contract states reviewed target facts, and `ReproducibilityContract` is first-party ceremony vocabulary, so the wave that lands it also widens that package's contract note in the same change, naming the enum as candidate ceremony vocabulary carried beside the reviewed facts. A silent addition is a rule break even though it compiles.

The field is required, never optional, and never inferred. It is selected before lookup and carried unchanged through request, materialization, and report. Absence is `ContractNotDeclared`, disagreement between two carriers of it is `ContractMismatch`, and comparing a value produced under one contract with a value produced under the other is `CrossContractComparison`. All three are refusals, and none is a downgrade.

One schema carries the contract, the handle, the digest, and the surface verdicts. Predicates are read from the enum, never from an optional field's presence and never from a parallel schema.

## 6.7 The byte-identity contract · `rule:guide-ctf-exec:byte-identity`

Equal inputs produce equal bytes. Concretely: equal fixture catalogue and manifest, equal funding inputs, equal target and deployment binding, equal asset, equal ordered destination programs, equal profiles, equal transaction fields, equal fees, equal sponsor facts, and equal external authorization bytes produce byte-identical funding and successor transactions.

A byte-identity run recomputes the fixture from its manifest, compares both transactions byte for byte, and reports `MaterializedBytes`. Mining placement, block identity, timestamps, and log output are excluded from the comparison, because none of them is a property of the construction.

Any wallet choice, any unrecorded nonce, any unstable proof, any reordering, any retry race, and any differing authorization bytes produce `ByteIdentityNotSatisfied`. That refusal is never a downgrade to the other contract: a run that selected byte identity and did not achieve it has failed, not changed contracts.

This is the reference contract, and it is the contract that satisfies the mandatory deterministic-public-fixture-openings row of the safety matrix.

## 6.8 The recorded-randomness contract · `rule:guide-ctf-exec:recorded-randomness`

Everything the byte-identity contract fixes is preserved except the opening source and cross-run byte equality: the fixture semantics, the handle, the digest check, the output order, every construction check, and the exact target form are identical. Wallet- or adapter-held randomness is admissible HERE only as an opening source under this contract's comparisons — this restates §1.3's guard and reopens none of §1.1's closed custody models.

```text
RetainedRunBytes    NEW
    funding_transaction
    proof_finalized_successor
    submitted_successor
```

Bytes are captured once, bound to a run identity, and compared only within that run. They and the public per-run opening attachment are retained until run closure and then discarded with the run, unless an archival consumer appears, at which point §1.2 applies to them.

Four comparison surfaces, and all four are compared:

| Surface | What is compared |
|---|---|
| `SemanticFixtureProjection` | roles, amounts, programs, asset, and order |
| `RetainedRunBytes` | the three retained transactions, within the run |
| `VerifiedOpeningProjection` | recomputed commitments and proof inputs against the recorded openings |
| `TargetReadbackProjection` | mined fields, proof shapes, outpoint, and witness transaction identity |

Cross-run byte inequality is admissible under this contract and under no other. Inequality on any other surface is `ComparisonSurfaceMismatch { surface }` and is a failure.

A recorded-randomness run reports `FixtureInputsOnly` and may never report `MaterializedBytes`, even when two runs happened to agree byte for byte. Chance equality is not a contract.

## 6.9 ADR-022 form classification · `tab:guide-ctf-exec:form-classification`

§1.2 applied to this guide's own assets, once, so no wave has to decide it twice:

| Asset | Form in Waves 0–5 | Classification |
|---|---|---|
| Handles, digests, manifests, resolved fixtures, openings, registries, derivation values, contracts, projections, refusals | plain typed values in process | no ADR-022 obligation |
| The digest transcript | ephemeral hash preimage | no obligation while ephemeral; storing or exchanging it creates one |
| Native executor request and response frames | first-party executor protocol | ADR-010 exclusion, unchanged |
| First-party command-line streams | first-party streams | ADR-010 exclusion, unchanged |
| A separately stored fixture catalogue or opening attachment | serialized, if a consumer exists | interchange document; §4.2 governs |
| Rendered funding and evidence reports | serialized, no consumer in this guide | terminal audit closure; exempt while unconsumed |

Classification follows form and consumer. It does not follow the crate, the filename, the file extension, or whether the value is public.

---

# 7. The predecessor slice · `sec:guide-ctf-exec:predecessor-slice`

## 7.1 Exact shape · `rule:guide-ctf-exec:slice-shape`

One funding transaction, and every clause below is a requirement rather than an example:

- one explicit protocol-asset input, contributing a zero value blinder;
- exactly two protocol outputs, both carrying the explicit protocol asset and both carrying positive confidential values whose semantic sum equals the explicit input amount;
- zero asset blinders on both outputs, and therefore no surjection proof on either;
- two deterministic output value blinders that are ordered additive inverses, so their sum is zero and matches the input blinder sum;
- a bounded deterministic fixture-counter search producing serialized value commitments whose prefixes are, in fixed order, one `0x08` and one `0x09`;
- a deterministic nonce field on each output, and a valid rangeproof on each bound to that output's value commitment, the unblinded asset generator for the explicit asset, and the output program;
- submission, acceptance, mining, and raw readback of the proof-bearing bytes from the target.

Semantic amounts stay inside the bound the commitment oracle already fixes; `commitment_oracle::SEMANTIC_AMOUNT_BOUND` and `is_semantic_amount` (both EXIST) are that bound's single source and fixture registration checks against them.

## 7.2 The non-protocol region · `rule:guide-ctf-exec:non-protocol-region`

Any policy-asset fee or change member belongs to an explicitly classified `NonProtocolFundingRegion` (NEW) and sits outside both two-output balance equations — the semantic one and the blinder one.

A member that is unclassified, that carries the protocol asset, that occupies a protocol position, or that changes the protocol count, order, fields, or witness census is refused. Classification is exhaustive: there is no member the classifier may decline to place.

The region exists so that a fee can be paid without the fee becoming an argument about whether the two-output equation closed. A transaction whose balance depends on which region a member was put in has been classified wrongly, not balanced cleverly.

## 7.3 Independent agreement · `req:guide-ctf-exec:slice-agreement`

Recomputation and raw readback must agree, for both outputs, on every field of a fixed census:

```text
FundingAgreementField                       NEW
    Asset
    Commitment
    Parity
    Nonce
    Program
    ProofShape
    Outpoint
    WitnessTransactionIdentity
```

`ProofShape` covers four things and not one: the rangeproof is present, the rangeproof verifies, the surjection proof is empty, and the rangeproof binds to this output's commitment, generator, and program rather than to some other output's.

`WitnessTransactionIdentity` binds the mined index and the witness-bearing bytes, so an agreement recorded against a non-witness identity cannot be mistaken for agreement about the proofs.

The agreement is between two origins from §1.9's table and never between a value and itself.

## 7.4 Slice report boundary · `rule:guide-ctf-exec:slice-report-boundary`

The slice reports exactly six things:

1. confidential funding capability and schema negotiation;
2. deterministic materialization under the selected custody profile;
3. the exact hybrid representation of both mined outputs;
4. both accepted commitment parities;
5. valid proof-bearing predecessor outputs and target readback;
6. a stable opening reference usable by the later transaction-wide materializer.

It reports no transfer, no authorization, no CT transfer conservation, no Guide-13 acceptance, no minimality, no production privacy, and no matrix discharge.

It clears exactly one blocker, `NoConfidentialPredecessorCanBeFunded`, and only after validated readback. It never clears `OwnerSighashNotComputable` and never clears `SighashProfileUnreviewed`. Clearance happens at the blocker's owning boundary — by removing it from `carried_residuals()` in `packages/vectors/src/live_evidence.rs` and updating the rows that carried it — and nowhere else.

---

# 8. Transaction-wide materializer · `sec:guide-ctf-exec:materializer`

## 8.1 The entry point · `def:guide-ctf-exec:materializer-api`

One function, in `packages/transaction/src/live_materialize.rs` (NEW file), and it is the only complete-private construction API:

```text
materialize_confidential_candidate(
    intent:   &ConfidentialConstructionIntent,
    fixtures: &FrozenConfidentialFixtureView,
    crypto:   &dyn ConfidentialProofMaterializer,
    checker:  &dyn IndependentCommitmentCheck,
) -> Result<MaterializedConfidentialCandidate, MaterializationRefusal>
```

`ConfidentialProofMaterializer` and `IndependentCommitmentCheck` are TRAITS declared in `transaction`, for the reason §3.3 fixes: `transaction` may not depend on `target-elements-conformance`, and the existing `PrivateValueCapability` and `LiveCurveCapability` already establish injection as this crate's way of needing cryptography without owning it.

`FrozenConfidentialFixtureView` (NEW) is the narrow read-only projection of §6.1's frozen registry that `transaction` can name without depending on wherever the registry is built. It supplies roles, amounts, programs, asset, fixed order, the balancing designation, and openings, and it supplies no registration and no freezing.

There is no per-output entry point. The old `PrivateValueCapability` remains for the per-output path it already serves and is retired from every private complete-transaction claim; a request that reaches this function with a per-output substitute is refused as `PerOutputMaterializationRefused`.

## 8.2 The intent · `def:guide-ctf-exec:intent`

```text
ConfidentialConstructionIntent                NEW
    inputs:              Vec<ConfidentialInputIntent>
    destinations:        Vec<ConfidentialDestinationIntent>
    non_protocol_region: NonProtocolFundingRegion
    profiles:            ConfidentialMaterializationProfiles

ConfidentialInputIntent                       NEW
    outpoint, target-observed fields, opening reference, explicit amount, zero blinder

ConfidentialDestinationIntent                 NEW
    semantic amount, explicit asset, output program, fixture identity and digest, role

ConfidentialMaterializationProfiles           NEW
    reproducibility_contract, custody_profile, materializer_profile,
    proof_profile, nonce_profile, order_profile, retry_profile
```

The intent is consumed whole. There is no builder that can be half-applied and no stage that may be called on a partial intent, because a transaction-wide balance problem cannot be solved on a subset of its own terms.

One contract per run. Byte identity is the reference (§1.3), and mixing within a run is `CrossContractComparison` rather than a warning.

## 8.3 Pre-construction validation · `req:guide-ctf-exec:preflight`

Everything below happens before any cryptographic work, and none of it reports a private subtotal:

1. every handle and digest resolves uniquely against the frozen view, and every predecessor opening recomputes the commitment the target was observed to hold;
2. input outpoints are unique; the input and output families are complete, nonempty, and classified into exhaustive disjoint roles; every protocol member carries the explicit protocol asset with a zero asset blinder;
3. semantic conservation closes across the protocol region, with the non-protocol region excluded from both equations;
4. every destination satisfies the target's proof policy, and the fixture's own output order and family placement are present and consistent with the intent's order;
5. every selected profile is one this build supports, and the combination is one this build supports — an unsupported combination refuses rather than defaulting to a supported neighbour.

A preflight refusal names what was missing. It never names a subtotal, an amount, or an opening, because a refusal that leaked one would have published exactly what §1.4 keeps off the wire.

## 8.4 Ordered materialization · `task:guide-ctf-exec:stages`

Seven stages, in this order, with no stage reordered for convenience:

1. derive deterministic output value blinders for every output except the fixture-designated balancing output;
2. solve the balancing output's blinder so the output blinder sum equals the confidential input blinder sum under the explicit asset generator; refuse an invalid or degenerate solved scalar rather than nudging it;
3. fix every protocol asset blinder at zero, emit the explicit protocol asset, and refuse any confidential-asset result outright;
4. construct each value commitment through `ConfidentialProofMaterializer`, then independently recompute it through `IndependentCommitmentCheck` and compare the two;
5. derive deterministic nonce material and generate a nonempty rangeproof for each confidential value, bound to that value's commitment, the unblinded asset generator, and the output program;
6. serialize the ordered output-witness vector, one entry per output, each carrying an empty surjection proof and its rangeproof;
7. freeze inputs, outputs, nonce fields, output witnesses, version, locktime, sponsor region, and fee region into one `ProofFinalizedCandidate` before any external signing input exists.

Stage 4 is not optional and is not a debug assertion. It is the place §1.9 is enforced, and a build that skipped it would have one opinion and would report it twice.

Stage 7 is the boundary the whole guide is arranged around: after it, nothing protected moves.

## 8.5 The result · `def:guide-ctf-exec:materialized-result`

```text
MaterializedConfidentialCandidate     NEW
    proof_finalized:        ProofFinalizedCandidate
    profiles:               ConfidentialMaterializationProfiles
    opening_binding_census: OpeningBindingCensus
    signer_inputs:          Vec<ProofFinalizedSignerInput>

OpeningBindingCensus                  NEW
    one verified fixture reference per confidential input, and no opening

ProofFinalizedSignerInput             NEW
    outpoint, required spent-output fields, position, byte binding, role
```

No member of the result exposes an opening, a blinder, a nonce input, or a proof input. `SignerInputWouldExposeOpening` is a refusal and not a lint, because the type that would have carried it must never be constructed at all.

## 8.6 Freeze and encoding · `rule:guide-ctf-exec:freeze`

`ProofFinalizedCandidate` (NEW) has private fields, a constructor visible only to the materializer, immutable accessors, and a region census:

```text
ProofFinalizedRegion    NEW
    Inputs | Outputs | NonceFields | OutputWitnesses | Version | LockTime | SponsorRegion | FeeRegion
```

Insertion, removal, replacement, reordering, proof regeneration, proof repair, and reblinding all return `PostFinalizationMutation { region }` and no candidate. Every output field, the output count, the output order, and every proof is protected.

The encoding law is exact: decoding the candidate's bytes and re-encoding them reproduces those bytes, including nonempty rangeproofs and empty surjection proofs. Loss, an alternate encoding of the same value, wrong emptiness, and fragments all refuse.

Meeting that law requires the four changes §3.2 records, and Wave 3 owns them:

- `TargetTransaction` gains an output-witness census, one entry per output, each an empty surjection proof and a possibly-nonempty rangeproof; `TargetOutput` stays as it is, because the output witness is a transaction-level vector in the target's serialization and modelling it as an output field would misplace it;
- `has_witness` becomes true when any input witness is non-null OR any output witness is nonempty, because a confidential funding transaction may legitimately have null input witnesses and proofs that must be serialized;
- `SuperfluousWitnessRecord` fires only when the flagged section carries neither a non-null input witness nor a nonempty output witness, so the existing round-trip law survives the addition rather than being weakened by it;
- `RangeProofRefused` and `SurjectionProofRefused` become form-conditional: a surjection proof is refused for the hybrid form always, and a rangeproof is refused where the form forbids one and required where the form requires one.

The current decoder refuses every nonempty proof unconditionally. That is correct for the explicit-only forms it was written for, and it is exactly what Wave 3 must change, so it is named here rather than discovered later.

## 8.7 Typed materialization refusals · `def:guide-ctf-exec:materialization-refusals`

`MaterializationRefusal` (NEW), closed, no catch-all. The concept requires the first group; the rest is what an adversarial reading of the stages adds.

| Group | Variants |
|---|---|
| Concept-required | `PredecessorOpeningMissing { outpoint }`, `PredecessorOpeningMismatch { outpoint }`, `SemanticValueImbalance`, `ValueBlinderImbalance`, `InvalidScalar { role }`, `InvalidCommitment { output }`, `BoundedParitySearchExhausted`, `NonceMaterializationFailed { output }`, `RangeproofMaterializationFailed { output }`, `SerializationRoundTripMismatch`, `PostFinalizationMutation { region }` |
| Fixture binding | `UnknownFixtureHandle { handle }`, `FixtureDigestMismatch { handle }`, `FixtureBindingAmbiguous { handle }`, `FixtureOutputOrderMismatch` |
| Family and asset | `DuplicateInputOutpoint { outpoint }`, `IncompleteFamilyClassification`, `ProtocolAssetMismatch { member }`, `ConfidentialProtocolAsset { member }`, `NonzeroProtocolAssetBlinder { member }` |
| Region | `NonProtocolRegionOverlapsProtocol { member }`, `NonProtocolRegionAffectsProtocolBalance` |
| Profiles | `UnsupportedProfileCombination` |
| Independence | `IndependentCommitmentMismatch { output }`, `IndependentCommitmentOriginNotDistinct { output }` |
| Proof shape | `ProofBindingMismatch { output }`, `RangeproofEmpty { output }`, `UnexpectedSurjectionProof { output }` |
| Census | `OutputWitnessCensusMismatch`, `OpeningBindingCensusMismatch`, `SignerInputWouldExposeOpening` |
| Substitution | `PerOutputMaterializationRefused` |

Every one is a construction refusal. None is a target verdict, and none may be re-reported as one (§1.7).

`role` and `member` are not new vocabularies. `role` is the derivation role of §6.4 for scalar refusals and the fixture output role for output refusals; `member` is the family classification's own member identity. The arbitration is that each of the two words has exactly one meaning in this guide, and neither is re-minted per refusal.

## 8.8 Independent cryptographic ownership · `rule:guide-ctf-exec:independence`

Three opaque newtypes, each constructible only inside the module that owns its origin:

```text
MaterializedCommitment    NEW, constructible only by the proof materializer's adapter
RecomputedCommitment      NEW, constructible only by the independent checker's adapter
ReadBackCommitment        NEW, constructible only by the adapter that decodes mined bytes
```

A comparison function takes two different types. It is therefore impossible to compare a materialized value with itself or a readback with a readback, and the property is carried by the type system rather than by a reviewer's attention.

The independent checker's only admitted implementation is an adapter over `target_elements_conformance::commitment_oracle` — the first-party bignum arithmetic whose independence claim the workspace already holds unqualified. `secp256k1-zkp` may implement the proof materializer, and may NOT implement the independent checker, because it binds the same C library the target vendors and its agreement would be conformance evidence rather than independence (§1.9).

Placement follows §3.3 and has one consequence Wave 3 must record openly. The trait is declared in `transaction`; the adapter that implements it lives in `vectors`, the one library that can see both `transaction` and `target-elements-conformance`. The `vectors` manifest currently records that what crosses its conformance edge is the executor boundary "and nothing else"; Wave 3 WIDENS that record explicitly to name a second boundary, the commitment oracle, and states why it is not the forbidden direction: the forbidden direction is obtaining an expectation from the component that will produce the observation, and the oracle produces no observation — it computes curve arithmetic over published constants and never touches a node.

A wave that widens the boundary silently, by adding a use statement and no manifest note, has broken the rule even though the code compiles.

## 8.9 What the per-output role loses · `rule:guide-ctf-exec:per-output-retirement`

`PrivateValueCapability` (EXISTS) cannot see a predecessor opening, cannot balance across destinations, returns only a commitment, fixes a null nonce, and cannot carry a proof. Those five limits are why the concept replaces the role rather than extending it.

After Wave 3 no private complete-transaction claim rests on it. The private branch of `finalize_live_transfer` takes the transaction-wide path, and `PrivateConstructionNonClaim::NoRangeProofIsProducedOrChecked` (EXISTS) is retired for the confidential lane only where a proof is genuinely produced and checked — and stays in force everywhere else, because retiring a non-claim by scope is honest and retiring it by wish is not.

---

# 9. Owner-sighash handoff · `sec:guide-ctf-exec:sighash-handoff`

## 9.1 What this guide does and does not own · `rule:guide-ctf-exec:handoff-ownership`

This guide owns proof finalization, the protected byte set, the handoff request, the binding of authorization responses to one candidate, and the refusal of protected mutation. It owns no digest, no profile selection, and no acceptance of either.

The interaction is nevertheless fixed rather than left open, because the target fixes it: `SIGHASH_ALL` incorporates both the serialized output set and the hash of the output-witness vector (§2.3), so the exact rangeproof and surjection-proof fields must be final before any receipt owner signs.

## 9.2 The mandatory order · `rule:guide-ctf-exec:handoff-order`

```text
resolve and verify predecessor openings
    ↓
materialize all inputs, outputs, commitments, nonces, and output witnesses
    ↓
freeze one proof-finalized candidate
    ↓
hand the exact protected transaction to the separately reviewed owner-sighash component
    ↓
collect every required owner authorization over that same candidate
    ↓
submit without changing any protected byte
```

Integration waits until proof finalization AND the selected owner-sighash profile are both independently accepted. Neither alone is entry.

## 9.3 The protected-bytes repair · `rule:guide-ctf-exec:protected-bytes`

This is the sharpest finding of the guide's reading of the tree, Wave 3 owns it, and it is not a new hypothesis — the workspace has already diagnosed the target behaviour it rests on.

`scripts/diagnose-taproot-output-witness-digest.py` (EXISTS) is the recorded, runnable diagnosis behind `G11-W11-06`: this target's taproot digest commits the transaction's OUTPUT WITNESS vector and hashes it at whatever length the vector happens to have, not at one entry per output. A transaction carrying no witness deserializes with an EMPTY output-witness vector and its signer hashes the empty string there; serializing a transaction that has any witness grows the vector to one entry per output, and consensus then hashes one entry per output. The two digests differ, the signature is reported complete, and the target refuses the spend as invalid.

`FinalizedLiveTransfer::protected_bytes` is currently computed as `protected.encode_without_witness()`, which omits the entire witness section and therefore presents exactly the empty-vector case that diagnosis names. `ProtectedDatum::ProofFields` (EXISTS) already declares that the proof fields are protected data, and the dimension mapping in `tapscript` currently sends it to `SighashDimension::SpentOutputs` alone — naming the input side's anchoring and not the created outputs' proofs.

For an explicit-only candidate the gap is invisible, because there are no proofs to omit and the empty vector is what the wire will carry anyway. For a proof-bearing candidate it is the difference between a declared commitment and an actual one: an owner would be asked to bind to a preimage that does not contain the rangeproofs the target's digest covers, and the resulting signature would be complete and invalid in exactly the way `G11-W11-06` records.

The repair has three parts, and each is a hypothesis to reproduce before it is applied:

1. the private representation's protected preimage includes the output-witness vector, so the bytes an owner binds to contain the proofs;
2. `ProtectedDatum::ProofFields` maps to the created outputs' dimension as well as the spent outputs', so the declared coverage matches the target rule the concept verified;
3. a test drives the difference directly: mutating one rangeproof byte changes the protected bytes, and a candidate whose protected bytes did not change when a proof changed is refused.

`ProofFinalizedCandidate` therefore EXTENDS the finalized form rather than replacing it or wrapping it. Replacing it would fork the signing path for two representations that share every other protected datum; wrapping it would leave the inner value's proof-free protected bytes authoritative, which is the exact loss the wrapper would exist to prevent.

## 9.4 Handoff states and refusals · `def:guide-ctf-exec:handoff-states`

```text
ProofFinalizedCandidate -> SigningStarted -> FullyAuthorizedCandidate -> SubmitReadyPrivateCandidate
```

Each transition is a type transition, so a candidate cannot be in two states and no flag can disagree with a state. From `SigningStarted` onward, proof repair, reblinding, regeneration, and any protected mutation are impossible to express.

`SighashHandoffRefusal` (NEW): `ProfileNotAccepted`, `WrongCandidate`, `MissingOwner`, `MutationAfterSigningStarted`.

The handoff supplies target spent-output data and no openings; this guide imposes no digest need for openings, and a request for one is a design change belonging to the other work rather than a gap in this one.

The existing `FinalizedLiveTransfer::check_offered` (EXISTS) is the pattern for `WrongCandidate`: an authorization is checked by comparing exact bytes, not by trusting that the signer looked.

---

# 10. Evidence · `sec:guide-ctf-exec:evidence`

## 10.1 The validated funding record · `def:guide-ctf-exec:funding-record`

`ValidatedConfidentialFundingRecord` (NEW) lives in `packages/target-elements-conformance/`, beside the existing `ValidatedNativeConformanceReport` and the conservation report. The placement is a dependency fact and not a preference: the record binds target, deployment, handshake, capability, schema, exact request, exact response, and raw readback, all of which that package owns and none of which crosses the narrow executor boundary `vectors` is permitted to import. Putting it in `vectors` would push the wire records across that boundary, which the boundary's own manifest note forbids.

Its only constructor is `validate_confidential_funding_record` (NEW). A raw wire record cannot become a validated record, and a caller-authored outcome cannot discharge one.

It binds: the target and deployment bindings; the handshake, capability, and schema bindings; the selected profiles as one `FundingProfiles` value; the exact request and the exact response, unredacted; the fixture handle and digest; the witness-bearing transaction; the submission and mining observations; the raw readback; two validated outputs in fixture order, each carrying output index, outpoint, explicit asset, value commitment, commitment prefix, nonce field, output program, rangeproof bytes, surjection-proof bytes, and its witness-transaction binding; the observed outcome layer as the existing `ObservedOutcomeLayer`; the exhaustive exclusions and non-claims; and a recomputed summary.

`FundingOutputForm` (NEW) fixes the four form facts as a census: `ConfidentialCommitment`, `ExplicitProtocolAsset`, `ValidRangeproof`, `EmptySurjectionProof`. A record naming three of them is the thing the census exists to make unsayable.

`independent_commitments: [CommitmentCheck; 2]` (NEW) carries, per output, the checker's identity, the recomputed commitment, the observed commitment, and the verdict — assembled inside this package, where the oracle and the readback both live and are different origins.

The record contains no amount and no opening. That is a property of its members, not a rule about how to fill them in.

## 10.2 Validation recomputes, in order · `rule:guide-ctf-exec:record-validation`

Eight recomputations, each with its own refusal, so a failed record names the step it failed:

| Step | What is recomputed | Refusal |
|---|---|---|
| 1 | request and response arms match; no fallback occurred | `FundingRecordRefusal::Arm` |
| 2 | handle, digest, profiles, request, response, transaction, and attachment all bind | `Fixture` |
| 3 | output order equals the fixture's fixed order | `Order` |
| 4 | asset, value, nonce, program, and witness form are exactly the hybrid tuple | `Representation` |
| 5 | the commitment prefixes are the admitted pair | `PrefixMask` |
| 6 | proof shape from the readback: rangeproof present and valid, surjection proof empty | `ProofShape` |
| 7 | submission, mining, readback, and outputs agree | `Readback` |
| 8 | the summary recomputes from the members rather than being carried | `Summary` |

Plus `CommitmentCheck` for a failed independent comparison and `CanonicalBoundary` for a record that carried an excluded value.

The prefix check reads the admitted pair from `target_elements::ConfidentialFieldEncoding` (EXISTS) rather than from a literal, so a change in the target's admitted prefixes is a change in one place.

## 10.3 Canonical exclusions · `rule:guide-ctf-exec:exclusions`

`CanonicalFundingExclusion` (NEW), exhaustive: `PrivateAmounts`, `FixtureOpenings`, `ValueBlinders`, `NonceInputs`, `ProofInputs`, `WalletData`, `Credentials`, `EnvironmentValues`, `Diagnostics`.

Public fixture data lives in its chartered attachment. It is not smuggled into a schema that excludes it, and the schema is not loosened so that it fits.

A canonical report under `ByteIdentity` may claim compared funding, successor, and report bytes, and may never claim a randomness property. A canonical report under `RecordedRandomness` may claim semantic fixtures, retained run bytes, fixture binding, and target projections, and may never claim deterministic bytes, deterministic derivation, or the reference row. `ReproducibilityClaimMismatch` (NEW) is the refusal, and no run mixes contracts.

## 10.4 Role separation, typed · `rule:guide-ctf-exec:evidence-roles`

```text
CandidateEvidenceRole    NEW
    Funding | CtConservation | OwnerSignature | Safety | Minimality | Resource | Lifecycle

EvidenceDisposition      NEW
    Validated | Blocked | NotRun | NotApplicable
```

One ceremony carries a map from role to disposition, and each entry is settled independently. A ceremony may not fill one role's entry from another's evidence; `EvidenceAssemblyRefusal::RoleSubstitution { offered_role, required_role }` (NEW) is what happens when it tries.

This is §1.8 made checkable. A summary reading "the funding ceremony succeeded, so the transfer is evidenced" cannot be constructed, because the two roles are two entries and only one of them was written.

## 10.5 Wave-5 restart order · `task:guide-ctf-exec:restart-order`

The order is mandatory, and each step's entry is the previous step's observed result:

1. one accepted sponsorless private one-to-one control, before any negative case;
2. both predecessor commitment parities exercised in complete accepted successors;
3. target CT conservation recorded against a balance-valid control;
4. wrong-blinder, missing-rangeproof, and malformed-rangeproof cases run from that control, each attributed to its own layer;
5. the remaining positive private shapes, only where each one accepts;
6. sponsor cases, only after their independent signer dependency closes by observation;
7. disclosure-minimality pairs last, only after both sides of each pair accept.

A negative case run before step 1 is not evidence about its own mutation, which is exactly what `LiveInfrastructureBlocker::NoAcceptingControlExists` (EXISTS) already records.

## 10.6 Proof-negative attribution · `rule:guide-ctf-exec:proof-negative`

A proof-negative mutates exactly one field of a control that is balance-valid in every other respect. A transaction already failing commitment balance cannot attribute its refusal to the rangeproof, and `ProofNegativeAttributionRefusal::ControlNotBalanceValid` (NEW) is the refusal that says so.

The rule is not a nicety. A run that took an imbalanced transaction's rejection as rangeproof evidence would have recorded a target fact it never observed.

---

# 11. Quantified unblocking and non-claims · `sec:guide-ctf-exec:nonclaims`

## 11.1 What is cleared, exactly · `rule:guide-ctf-exec:unblocking`

Confidential funding alone clears ONE carried residual: `NoConfidentialPredecessorCanBeFunded`. It establishes that a guide-shaped predecessor can exist on chain and that the ceremony can retain a verified opening reference.

It moves ZERO Guide-13 matrix rows. Every positive private row remains blocked by the independent owner-sighash component, and sponsor cases retain their signer dependency where claimed.

The materializer makes ten private-positive fixtures constructible and enables the CT and proof mutations, and it still moves zero positive rows before owner authorization and target acceptance. Constructibility is not acceptance.

The four residuals `carried_residuals()` holds today are `SighashProfileUnreviewed`, `SponsorEnvelopeSignerAbsent`, `PredecessorConstructorAbsent`, and `NoConfidentialPredecessorCanBeFunded`. This guide removes exactly the last one, at the end of Wave 2, and only after validated readback. `PredecessorConstructorAbsent` is about a time-locked predecessor and is a different question that this guide does not touch.

## 11.2 The rows Wave 5 may move, and the ones it may not · `tab:guide-ctf-exec:row-delta`

The ten positive private classes of the Guide-13 safety matrix, with their disposition under this guide:

| Class | May enter the Wave-5 delta | Gate |
|---|---|---|
| one-to-one | yes | external sighash accepted, then observed acceptance |
| split | yes | same |
| merge | yes | same |
| many-to-many representative | yes | same |
| several distinct owners | yes | same |
| both commitment parity forms | yes | Waves 2 and 3 make both constructible; acceptance still needed |
| deterministic public fixture openings | yes | satisfied by the byte-identity contract (§6.7) |
| target CT conservation | yes | needs a balance-valid accepted control first |
| projection equality with paired explicit cases | yes | needs both sides accepted |
| private sponsor values where claimed | NO | blocked independently by `SponsorEnvelopeSignerAbsent`, which this guide does not clear |

Nine of ten are reachable after the external sighash closes; the tenth is not this guide's to reach. A closeout that moved the sponsor row would be claiming a signer wiring that does not exist.

## 11.3 Non-claims, carried by the result · `rem:guide-ctf-exec:non-claims`

`CandidateFundingNonClaim` (NEW) is exhaustive, is carried by every validated record and every report, and is checked rather than written down:

| Variant | What is not established |
|---|---|
| `ProductionCustodyNotEstablished` | production opening, key, nonce, seed, blinder, wallet, or credential custody |
| `ProductionCryptographyNotEstablished` | production-quality randomness, blinding, proof generation, signing, erasure, or side-channel resistance |
| `ProductionMultiOwnerProtocolNotEstablished` | a production multi-owner blinding or signing protocol |
| `PrivacyNotEstablished` | owner anonymity, graph privacy, count privacy, timing privacy, wallet privacy, universal transaction confidentiality |
| `PublicFixturesNotSecret` | that public deterministic fixture openings are secret from an observer |
| `StockRpcHybridSupportNotEstablished` | that the stock Elements RPC surface supports the selected hybrid representation |
| `FundingDoesNotProveTransferSafetyMinimalityOrResource` | that funding evidence proves a live transfer, a safety relation, a disclosure-minimality relation, or a resource result |
| `MalformedRejectionDoesNotReplaceControl` | that malformed private rejections substitute for an accepted positive private transaction |
| `OwnerSighashNotEstablishedHere` | that the owner-sighash profile is implemented or reviewed by this guide |
| `CandidateInterfaceNotFinal` | that a candidate interface is final, stable, production-capable, or released |

The result is candidate-only. A secret-bearing selection stops at ADR-015 rather than weakening the boundary.

---

# 12. Implementation waves · `sec:guide-ctf-exec:waves`

Six waves, in the concept's dependency order, which is mandatory and not a suggestion. Each wave ends formatted, focused-test green, documented, and committed before the next begins.

## Wave 0 — Elaborate custody and reproducibility · `task:guide-ctf-exec:wave0`

**Entry.** §2.1 and §2.2. The three rulings are recorded and reproduced, not renegotiated.

**Deliverables**

- the custody ruling CARRIED forward: deterministic central public fixtures, with the ADR-022 form boundary of §1.2, mapped onto the existing `ConfidentialConstructionModel::CentralPublicFixtureConstruction`;
- opening owner, lifetime, process boundary, lookup authority, diagnostics, production separation, and ADR-015 disposition, elaborated within the accepted model;
- the reproducibility ruling CARRIED forward: both contracts first-class, byte identity as the reference, one shared typed field;
- reviewed fixture domain separators, derivation inputs, bounded retry constants, and the typed contract field's semantics for both contracts;
- the canonical-request ruling CARRIED forward, with its validation binding elaborated;
- the four preflight findings of §3.2 REPRODUCED and dispositioned CONFIRMED, REFUTED, or RECLASSIFIED, each by running code;
- the affected package and dependency boundary of §3.3 recorded, with no implementation by implication;
- the four pending charter decisions of §4 filed for ruling.

**Exit.** Nothing is implemented, every finding has a disposition, and every pending decision is filed.

**Suggested commit**

```text
plans: elaborate confidential test-material custody and reproducibility
```

## Wave 1 — Revise the funding wire · `task:guide-ctf-exec:wave1`

**Entry.** Wave 0 exited. §4.3's ruling is not required to start, and is required before a recorded-randomness request is serialized.

**Deliverables**

- `OperationStepKind::FundConfidential`, `ExecutorCapability::ConfidentialValueTestFunding`, and the fifth `OperationSubject` arm;
- the request types of §5.2 and the response members of §5.3;
- `NATIVE_PROTOCOL_SCHEMA` at 5, bumped in the harness and the reviewed native adapter in one change;
- `ConfidentialFundingAdvertisement` on the handshake, with capability and advertisement held consistent;
- `ReproducibilityContract` in `target-elements`, named by both sides;
- the complete `ConfidentialFundingRefusal` vocabulary of §5.5;
- readback binding per §5.6, with no fact taken from a request echo;
- mock-executor and protocol contract cases for every arm, every mismatch, and the untagged-record case.

**Exit.** Every refusal variant is reachable from a test, and no test reaches a target.

**Suggested commit**

```text
target-elements-conformance: type confidential funding
```

## Wave 2 — Prove one confidential predecessor · `task:guide-ctf-exec:wave2`

**Entry.** Wave 1 exited. §2.3's target facts re-checked against the deployment in use.

**Deliverables**

- the fixture registry, handle grammar, digest, and derivation recipe of §6.1 through §6.5;
- one deterministic funding transaction meeting §7.1 exactly;
- one `0x08` and one `0x09` commitment under the bounded fixture recipe, in fixed order;
- deterministic nonce fields, valid rangeproofs, and empty surjection proofs;
- target acceptance, mining, raw readback, and independent commitment comparison per §7.3;
- `ValidatedConfidentialFundingRecord` and its validator, carrying all non-claims;
- `NoConfidentialPredecessorCanBeFunded` removed from `carried_residuals()` and from the rows that carried it, at that boundary and after validated readback only.

**Exit.** One mined predecessor exists, one validated record binds it, and exactly one residual moved.

**Suggested commit**

```text
vectors: record confidential predecessor funding
```

## Wave 3 — Materialize private transactions transaction-wide · `task:guide-ctf-exec:wave3`

**Entry.** Wave 2 exited with a mined predecessor and a stable opening reference.

**Deliverables**

- `materialize_confidential_candidate` and the intent, view, and profile types of §8.1 and §8.2;
- the preflight of §8.3 and the seven stages of §8.4, in order;
- the four serialization changes of §8.6, each with a round-trip test;
- the protected-bytes repair of §9.3, with the mutating-one-proof-byte test;
- `ProofFinalizedCandidate`, its region census, and every post-finalization mutation refusal;
- the three origin-tagged commitment newtypes and the independence rule of §8.8, including the explicit widening of the `vectors` manifest note;
- focused valid, imbalance, blinder, proof, and serialization cases;
- the per-output role retired from private complete-transaction claims per §8.9.

**Exit.** A proof-finalized candidate exists whose bytes round-trip exactly and whose protected preimage changes when a proof changes.

**Suggested commit**

```text
transaction: finalize confidential candidate witnesses
```

## Wave 4 — Close the external sighash handoff · `task:guide-ctf-exec:wave4`

**Entry condition.** The separately reviewed owner-sighash profile has reached its own accepted result, AND §4.1's ruling is recorded. Neither alone is entry, and a Wave-4 start without both is a rejection condition.

**Deliverables**

- proof-finalized candidate bytes enter the signing request unchanged;
- output-witness commitment independently tested by the sighash work, not by this guide;
- valid, wrong-owner, wrong-candidate, post-proof-mutation, and post-signing-mutation cases;
- every required owner signs the same protected candidate;
- one complete sponsorless private candidate becomes submit-ready;
- an explicit statement, in the code and in the report, that the digest implementation belongs to the parallel sighash work.

**Exit.** One submit-ready private candidate exists, and no digest was designed here.

**Suggested commit**

```text
transaction: bind private candidates to reviewed owner signing
```

## Wave 5 — Restart Guide-13 evidence · `task:guide-ctf-exec:wave5`

**Entry.** Wave 4 exited with a submit-ready candidate.

**Deliverables**

- the restart order of §10.5, executed in order and stopped honestly wherever a step does not accept;
- the evidence-role map of §10.4 filled per ceremony, with no substitution;
- target CT conservation, wrong-blinder, missing-rangeproof, and malformed-rangeproof cases with attributable layers;
- semantic projections and exact report exclusions validated;
- Guide-13 blockers and matrix rows updated from observed evidence only, per §11.2;
- the closeout report of §19.

**Exit.** The exit checklist of §18 passes, or a typed stopped result records exactly which step did not.

**Suggested commit**

```text
vectors: restart private live-transfer evidence
```

---

# 13. Verification · `sec:guide-ctf-exec:verification`

Reproduction-test convention (stated at Wave 0): a reproduction module states its convention in its own module header. Where a repair property is known in advance, an ignored test stating that property means CONFIRMED and a running one means REFUTED, as Guide 13 fixed; a characterization with no repair property is instead a set of running measurements, and the header says so, so a running test there is never misread as a refutation.

## 13.1 Working cadence · `rule:guide-ctf-exec:cadence`

For Rust tranches:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Focused package tests run during development; the full workspace and Meson gate run once per coherent batch, on the content-scoped cadence the backlog records. Documentation-only tranches run the documentation, label, and census checks rather than the full Rust suite.

Every run reports its wall time. A step that did not report how long it took is a process defect in the harness and is reported as a finding, not shrugged off.

## 13.2 Focused package commands · `tab:guide-ctf-exec:focused-tests`

```sh
cargo test --locked -p tripod-target-elements
cargo test --locked -p tripod-tapscript
cargo test --locked -p tripod-transaction
cargo test --locked -p tripod-vectors
cargo test --locked -p tripod-target-elements-conformance
```

For changed public package documentation:

```sh
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p tripod-transaction --no-deps
```

## 13.3 Required focused regressions · `tab:guide-ctf-exec:regressions`

Each line is one test that must exist and must fail if its rule is removed:

```text
confidential funding subject and explicit funding subject are untagged-disjoint
a revision-4 executor is refused at the handshake, not translated for
capability without advertisement, and advertisement without capability, both refuse
an unadvertised profile refuses before construction
an untagged record under revision 5 is unknown, not explicit
explicit fallback from a confidential request is a response arm mismatch
a request echo cannot replace readback for any output fact
a refusal carrying a funded observation is refused
an unknown handle and a drifted digest both refuse before cryptographic work
derivation is role-separated and case-separated
the parity search finds 0x08 then 0x09 without reordering
a degenerate scalar and an identity commitment are typed refusals
a rangeproof failure adds no randomness and triggers no retry
equal inputs reproduce funding and successor bytes
byte identity never downgrades to recorded randomness silently
recorded randomness compares all four surfaces
a contract mismatch refuses before comparison
a report under one contract cannot claim the other's guarantees
the materializer refuses semantic imbalance and blinder imbalance separately
the materializer refuses an invalid balancing scalar
the materializer refuses an empty or cross-bound rangeproof
the materializer refuses an unexpected surjection proof
a proof-finalized candidate round-trips exact bytes
the decoder refuses a dropped rangeproof
a proof-finalized candidate refuses mutation in each protected region
an independent origin cannot self-attest
protected bytes change when one rangeproof byte changes
the handoff refuses post-signing mutation
one ceremony keeps its evidence roles distinct
a proof-negative refuses an imbalanced control
minimality waits for two acceptances
the closeout records one residual, zero pre-sighash rows, and all non-claims
```

## 13.4 Real target runs · `tab:guide-ctf-exec:target-runs`

Run separately, never folded into one verdict:

```text
confidential funding negotiation matrix
predecessor slice mining and readback
both-parity predecessor materialization
private candidate submission after external sighash
private safety matrix restart
private minimality pairs
```

Each run records the target build and provenance, the deployment binding, the selected profiles and contract, exact request and response bytes, the mined transaction, the observed layer per submission, and its own wall time. A run whose target provenance is unrecorded produces no evidence.

Compute happens on the server lane. A gate or matrix run started on the interactive host is a process defect regardless of how long it took.

## 13.5 Full batch gate · `gate:guide-ctf-exec:batch`

After a coherent implementation batch:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

When dependencies changed — and §3.4 says they should not:

```sh
cargo tree --locked -e features
cargo metadata --locked
cargo audit
```

A missing advisory tool is recorded as skipped, never as passed.

Documentation and census:

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Final check, which must be empty:

```sh
git status --porcelain=v1 --untracked-files=all
```

---

# 14. Wave report contracts · `sec:guide-ctf-exec:report-contracts`

Every wave's report carries the same eight items, in this order, and a report missing one is incomplete rather than brief:

1. what was implemented, by file and by type, with EXISTS or NEW marked as §3 marks it;
2. per-step wall times, including every test and gate run;
3. which rulings the wave carried and which pending decisions it needed and did or did not have;
4. every typed refusal variant added, and whether each is reachable from a test;
5. blockers: which were carried, which moved, and the observed result that moved them — a blocker that moved without an observation is a finding against the wave;
6. matrix rows: how many moved, which, and on what evidence; zero is the expected answer before Wave 5;
7. non-claims carried by the wave's own output;
8. anything surprising, stated plainly, including findings that refute this guide.

A wave that could not reach its target reports a typed stopped result naming the blocker. That is a valid wave outcome. Reporting partial success as success is not.

---

# 15. Acceptance criteria · `sec:guide-ctf-exec:acceptance`

The guide is accepted only when all of the following hold:

- the three rulings are in force, unweakened and unreopened, and every pending decision of §4 that a completed wave needed was ruled before that wave started;
- the wire union has one exact hybrid meaning, a fail-closed migration, and no path by which a confidential request is answered explicitly;
- the first slice specifies and achieves deterministic opposite-sum value blinders, zero asset blinders, both commitment parities, valid rangeproofs, empty surjection proofs, mining, and raw readback;
- the transaction-wide capability consumes every opening, balances the blinders, serializes the proofs, and freezes the complete protected candidate before signing;
- the protected preimage covers the output-witness vector, demonstrated by a test in which changing one proof byte changes the protected bytes;
- the owner-sighash boundary is external, parallel, and ordered strictly after proof finalization;
- funding-only evidence and zero-row movement before the external closure are explicit and checked, not narrated;
- the six waves preserved the required dependency order;
- the production and privacy non-claims are carried by the result rather than left to implication;
- every comparison in the result is between two distinct origins from §1.9's table.

Implementation acceptance additionally requires one mined predecessor, one accepted sponsorless private one-to-one successor after external sighash closure, independently validated observations, and no stronger claim than those three support.

---

# 16. Rejection criteria · `sec:guide-ctf-exec:rejection`

Stop the guide, and reject the wave, when any of the following appears:

- custody becomes implicit, ADR-015 is bypassed, or nondeterminism lands without an accepted contract revision;
- stock blinding is claimed to produce the hybrid form, or a confidential request exposes an amount or an opening;
- a legacy explicit fallback answers a confidential request, an unbound mined response is accepted, or per-output-only materialization is accepted for a complete private transaction;
- any protocol asset blinder is nonzero, any protocol asset is confidential, a rangeproof is absent or invalid, or a surjection proof is present;
- a protected byte changes after signing starts, or a proof is repaired, reblinded, or regenerated at any point after finalization;
- funding capability is counted as a matrix pass, or a blocker moves without an observed result;
- a malformed rejection replaces a positive control, or a proof-negative is attributed from an imbalanced control;
- one computation supplies both the observation and the expectation, or a reference-implementation agreement is reported as independent evidence;
- a new cryptographic dependency is added, or a library edge appears between `transaction` and `target-elements-conformance`;
- any production, privacy, stability, or release claim appears in code, test names, or reports.

---

# 17. Identity, schema, security, and dependency impact · `sec:guide-ctf-exec:impact`

**Identity.** No architecture operation, phase, release identity, or digest is minted. The fixture digest is a drift-detection value over public test material and is explicitly not an identity; the guide's own labels are working labels outside the plans census until closeout. All new types remain subordinate to existing identities.

**Schema.** The native protocol moves from revision 4 to revision 5 in one change on both sides, for the reason §5.4 gives. Old explicit records retain their original schema and their original meaning. No report schema is versioned by this guide unless the selected evidence policy requires it, and if one is, the version is stated in the report's own bytes rather than inferred.

**Security.** Every scalar, blinder, opening, nonce input, and proof seed in this guide is public disposable test material in ADR-015's own sense, is named as such at each use, is unrelated to production, and is destroyed with the chain. Real secret retention or transport requires ADR-015's separate design gate. Diagnostics and canonical reports exclude openings without exception.

**Dependencies.** None is added; §3.4 records why, and the absence is verified by `cargo tree` rather than asserted. The dependency directions of §3.3 are unchanged in the library graph. The one deliberate boundary change is the `vectors` manifest note of §8.8, which widens a recorded boundary explicitly rather than silently.

**Handoff.** The closeout report of §19 is written to be consumed by Guide 14 without reopening any ruling and without importing any digest design. The existing Guide-14 concept draft is KEPT as queued.

---

# 18. Exit checklist · `gate:guide-ctf-exec:exit`

## Rulings and decisions

- the three accepted rulings are quoted in force and unweakened;
- every pending decision a completed wave needed was ruled before that wave started;
- unruled decisions are recorded as unruled, with the waves they gate named.

## Wire

- schema 5 on both sides, bumped in one change;
- capability and advertisement consistent, and every selected profile advertised;
- every arm mismatch, unknown tag, unknown member, and unadvertised profile refuses;
- no output fact in any record came from a request echo.

## Fixtures

- registry frozen before funding, and the freeze is a type transition;
- handle grammar enforced, and no handle encodes an amount;
- digest recomputed at registration and at lookup, with no repair path;
- both reproducibility contracts implemented, and neither distorts the other.

## Materialization

- one mined predecessor with prefixes `0x08` and `0x09` in fixed order;
- both outputs carry valid rangeproofs and empty surjection proofs;
- blinders sum to zero and asset blinders are zero;
- the candidate's bytes round-trip exactly, proofs included;
- every protected region refuses mutation;
- every commitment comparison is between two distinct origins.

## Signing

- protected bytes cover the output-witness vector, proven by the one-byte test;
- Wave 4 started only after the external accepted result was recorded;
- no digest was designed, selected, or accepted here.

## Evidence

- funding evidence is its own role and discharged nothing else;
- exactly one residual cleared, at its owning boundary, after validated readback;
- zero matrix rows moved before the external closure;
- the sponsor row did not move;
- every non-claim is carried by the record rather than by prose.

## Repository

- formatted, clippy-clean, workspace tests green, Meson gate green;
- no new dependency, no new library edge between `transaction` and `target-elements-conformance`;
- every run's wall time reported;
- working tree clean.

---

# 19. Closeout report template · `sec:guide-ctf-exec:closeout`

`ConfidentialFundingCloseoutReport` (NEW), written for Guide 14 to consume:

| Member | Content |
|---|---|
| `disposition` | `Completed` or `TypedStopped { blocker }` |
| `rulings` | the three accepted rulings, quoted, plus each pending decision's ruling or its unruled status |
| `contracts` | which reproducibility contract each ceremony ran under |
| `funding` | the validated funding records, by handle |
| `target_facts` | target build, provenance, deployment binding, mined identities, block heights |
| `sighash_result` | the external accepted result consumed, or its absence |
| `roles` | the evidence-role map per ceremony |
| `cleared_residuals` | must equal exactly `{ NoConfidentialPredecessorCanBeFunded }` |
| `blockers` | the residuals still carried, unchanged |
| `pre_sighash_matrix_delta` | must equal `0` |
| `wave5_matrix_delta` | the rows moved on observed evidence, per §11.2 |
| `exclusions` | the canonical exclusions applied |
| `non_claims` | the complete `CandidateFundingNonClaim` census |
| `wall_times` | per-wave and per-gate durations |

Three invariants are checked rather than trusted: `cleared_residuals` holds exactly one member; funding never clears `OwnerSighashNotComputable`; and `pre_sighash_matrix_delta` is zero.

A missing external result yields `TypedStopped`, which is a valid closeout. Guide 14 consumes this record without reopening any ruling and without inheriting any digest design.

---

# 20. Open-question ledger · `sec:guide-ctf-exec:open-questions`

The seventeen questions the four angle documents raised, each RESOLVED with its source or carried into §4 as PENDING.

| # | Source | Question | Disposition |
|---|---|---|---|
| W1 | wire | One binary for schemas 4 and 5, or compatibility entry points? | RESOLVED — one binary, one constant at 5, revision-4 refused at the handshake. Source: `protocol.rs`'s own revision doctrine, which refuses a revision-3 executor rather than reconciling it and requires both implementations to bump together (§5.4). |
| W2 | wire | Which newtypes, profile identifiers, and registry owners do the fixture and materializer lanes assign? | RESOLVED — fixed by arbitration in §5.2, §6.1, and §6.2; registry owned where the ceremony is built, `ReproducibilityContract` in `target-elements` on the dependency reading of §6.6. |
| W3 | wire | Does readback report witness identity and block height, and which failures stay outside the wire refusal? | RESOLVED — both are reported, because the concept's binding names the witness-bearing transaction identity and the mining observation (§5.3, §5.6). Outside the vocabulary: transport and framing failures, already owned by the existing protocol error type, and target verdicts, which are `ObservedOutcomeLayer` observations and not refusals (§1.7). |
| F1 | fixtures | Confirm crate placement and dependency direction with the materializer lane. | RESOLVED — read from the three manifests: materializer in `transaction`, oracle in `target-elements-conformance`, wiring in `vectors`, no new library edge (§3.3, §8.8). |
| F2 | fixtures | Choose an ADR-022 namespace, version, and catalogue transport if a consumer exists. | PENDING — §4.2. Recommendation: no allocation while no consumer exists, with the trigger stated and tested. |
| F3 | fixtures | Align the rangeproof seed and nonce inputs with the proof API's byte-identity support. | RESOLVED — `secp256k1-zkp` is already a workspace dependency with an existing consumer and lock entry, so no new dependency arises; its claim class is reference-implementation conformance and not independence, which fixes what the proofs may be reported as (§3.4, §1.9). Seeds are thirty-two hash bytes and are not scalars (§6.4). |
| F4 | fixtures | Fix the authorization-byte boundary without importing owner-sighash design. | RESOLVED — the existing `LiveSigningRequest` and `FinalizedLiveTransfer::check_offered` are the boundary: a preimage out, opaque authorization bytes back, compared by exact bytes (§9.4). |
| F5 | fixtures | Choose recorded-randomness bootstrapping while preserving central custody and a handle-and-digest-only request. | PENDING — §4.3. Recommendation: a semantic-only transcript with the contract tag inside the domain. |
| M1 | materializer | Which package independently owns the independent commitment checker? | RESOLVED — `target-elements-conformance`'s existing `commitment_oracle`, whose unqualified independence claim that package's manifest already records and defends at length (§1.9, §8.8). |
| M2 | materializer | Does the proof-finalized candidate replace or wrap the finalized live transfer without proof loss? | RESOLVED — neither: it EXTENDS it, because the current protected bytes are `encode_without_witness()` and a wrapper would leave that proof-free preimage authoritative — the empty-vector case `G11-W11-06` already diagnosed (§9.3). |
| M3 | materializer | Which vocabulary supplies `role` and `member` without duplicates? | RESOLVED — arbitrated in §8.7: `role` is the derivation role or the fixture output role, `member` is the family classification's member identity; neither is re-minted per refusal. |
| M4 | materializer | Does the fixture lane expose the balancing-output designation and fixed order compatibly? | RESOLVED — yes, by design: `FrozenConfidentialFixtureView` supplies fixed order and the balancing designation, and the first slice's opposite-sum blinders are an instance of the general balancing rule rather than a special case (§6.4, §8.1). |
| E1 | evidence | Which package owns the validated record while hiding raw wire records? | RESOLVED — `target-elements-conformance`, beside `ValidatedNativeConformanceReport`, because the record's members are that package's own and moving it to `vectors` would push wire records across a boundary whose own note admits only the executor types (§10.1). |
| E2 | evidence | Which wire revision and capability identifiers replace the opaque bindings? | RESOLVED — schema 5 and `ExecutorCapability::ConfidentialValueTestFunding` (§5.1, §5.4). |
| E3 | evidence | Do serialized records activate ADR-022 through an external consumer, or remain terminal audit output? | RESOLVED as the working classification — terminal audit closures, on the Phase-5 precedent that a report acquiring an external consumer activates the rest of the posture; the allocation question itself is §4.2 (§6.9). |
| E4 | evidence | Which accepted-result type or profile will the parallel sighash work expose? | PENDING — §4.1. Recommendation: an accepted-profile token plus opaque authorization bytes bound to exact protected bytes. |
| E5 | evidence | Which Guide-13 rows may enter the Wave-5 delta, and which sponsor rows stay blocked? | RESOLVED — nine of the ten positive private classes may, after the external closure and on observed acceptance; the private-sponsor-values row may not, because `SponsorEnvelopeSignerAbsent` is an independent blocker this guide does not clear (§11.2). |

A fourth pending decision, §4.4, is this guide's own and was not raised by any angle document: §1.3 fixes that the reproducibility contract is a per-ceremony typed selection but not who selects it.

---

## Closing statement · `rem:guide-ctf-exec:closing`

This guide is finished when one confidential predecessor has been mined and read back, one private candidate has been frozen with its proofs before any owner signed, one accepted sponsorless private successor exists on the far side of work this guide does not own, and every report says exactly that and nothing more.

If the external sighash does not close, the honest result is a typed stopped one with three of the six waves complete and one residual cleared. That is a smaller claim than the guide set out to make, and it is the only kind of claim worth carrying into Guide 14.
