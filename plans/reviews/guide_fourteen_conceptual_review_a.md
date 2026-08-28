# Guide 14 conceptual review A — compiling mutable STATE to a UTXO target

## §1. Question restated in my own terms

The governing question is whether Guide 14 describes one abstraction-preserving compilation:

```text
canonical semantic world W
    -- announce-maturity(a) -->
canonical semantic world W'

                    refined by

predecessor STATE UTXO
    -- one atomic target transaction -->
successor STATE UTXO
```

At the semantic level, the operation should be ordinary mutation of the one canonical STATE: select no predecessor manually, change only maturity, and leave all other fields and roots alone. At the target level, the immutable predecessor and successor must be distinct state versions connected by a tracked, authorized, atomic transition. The review therefore asks four things:

1. whether target representation is absent from the semantic operation;
2. whether the target construction is a sound and complete refinement of that operation;
3. whether the compiler seam preserves the distinction; and
4. whether the guide states the standard guarantees of the mechanisms it uses, especially atomicity, ownership, version uniqueness, totality, and determinism.

Overall judgment: the guide largely succeeds at presenting mutation at the top, provided the invariant-wrapped whole-world model execution remains the normative semantic entry point. Its back end is a strong soundness design for one accepted transaction. It does not yet establish a globally unique live state thread from its synthetic origin, does not supply host-side affine ownership of a current-state capability, and does not provide total target realization. The last point is currently obscured by calling bounded deterministic rejection sampling “tweak totality”; that is the one conceptual blocker against the owner's demand for named standard guarantees.

## §2. The guide's actual design as I read it

### 2.1 Semantic presentation

The guide declares typed Rust the sole semantic source and makes encoded bytes a one-way target representation (§1.1). It defines STATE as one canonical root rather than an output class (§1.3), specifies the operation as a single-field update (§1.4), and separates semantic fields from representation fields (§1.5). The request selects only an announced cycle and optional sponsorship form; it cannot select the predecessor, successor, root cursor, constructor, nonce, program, or transaction positions (§2.3). The successor is derived with maturity changed and every other semantic field copied (§2.4), while the root projection is one STATE succession and absence of every other root effect (§§2.5–2.6).

Section 6 introduces a semantic `StateMetadata`, a separate representation nonce, a canonical codec, and a typed `announce_maturity` function. The function is a total decision procedure in the programming-language sense: every call returns either a complete successor or a typed refusal, with no partial successor (§6.5). It performs no nonce search. Section 15 then places expected semantics on the invariant-wrapped model path: the fixture retains predecessor world, request, successor world, and an appended transition certificate (§§15.1, 15.3). In the established substrate, that path is already a pure state-passing abstract machine transition over the complete `World`; errors leave the old world unchanged and successful execution rechecks the global invariant. Thus immutable Rust values do not prevent the semantic interface from representing mutation. Functional state passing is a standard implementation of mutable abstract state.

There is, however, an important naming boundary. The §6.5 function transforms detached metadata, not the canonical world. It is the local field-delta function. The actual semantic operation is the invariant-wrapped world transition of §15.1. The guide relies on this distinction but never states it as an API rule.

### 2.2 Compiler seam

The seam is principally between §6.5 and §7:

- §6.5 ends with a semantic successor and explicitly returns no representation nonce.
- §7 carries the semantic relation census into a validated compiler plan, but excludes opcodes, stack indices, transaction positions, program bytes, control blocks, graph handles, and a nonce result (§7.1).
- §15.3 forbids the backend constructor from generating expected semantics, while §15.4 materializes target bytes only from a linked bundle, candidate ABI, typed request, current-state view, signer, and optional sponsor capability.

This is a conventional source-semantics/validated-IR/backend split. Representation requirements in the compiler plan are not themselves leakage: constructibility and carrier obligations belong in a compiler IR so long as they do not become semantic facts.

The copy-through design also sits on the correct side. The semantic rule is complete record preservation (§1.4 and §6.5); the emitted program derives a prefix/suffix rewrite or per-field comparisons from the typed schema (§10.5). The semantic layer need not know that the target realizes the update by reconstructing bytes. The representation nonce is likewise erased by semantic projection and introduced only after the semantic transition (§§1.5, 6.3, 12.5).

One seam ambiguity remains: §6.6 places `NoncanonicalRepresentationNonce` and `RepresentationSearchExhausted` in the “metadata” refusal vocabulary, although the strict decoder of §6.4 has only bytes and cannot decide whether a nonce is the first constructor-admissible one without the static subtree, internal key, branch policy, and tweak rules. Those are constructor-normal-form decisions, not semantic decoding decisions.

### 2.3 Backend realization

The backend uses a state-carrying continuation covenant:

1. The predecessor target program commits to canonical semantic metadata plus its representation nonce and to one static operation subtree (§§9.2–9.5).
2. The announcement leaf authenticates the actual consumed program and derives successor semantic metadata rather than accepting a successor blob (§§9.5–9.6).
3. It reconstructs output 0 under the same static subtree, internal key, leaf version, and tweak relation (§§9.6, 10.7).
4. Transaction cardinality and role closure require one STATE input and one STATE output, with all other protocol roots and economic objects absent (§§10.2, 10.8, 12.1–12.2).
5. All protected bytes are frozen before operator signing (§§12.7, 13.1–13.3).
6. Constructor continuity, root history, semantic projection, and public recovery are checked independently (§§14.3–14.7, 17.1–17.2).

The paired predecessor/successor recipes are therefore more than “same key” continuity. They define an authenticated representation relation on both sides of the semantic transition (§1.7), while the carrier-closure equality prevents an obligation from disappearing between plan, emitted program, linker, and ABI (§11.5). This is a strong forward-simulation design.

The target claim is nevertheless conditional. Current-root freshness may remain report-layer evidence (§10.3), construction accepts a public current-STATE view (§12.4), and the native predecessor is explicitly synthetic with no genesis or prior-history claim (§§5.7, 15.2). The transaction can prove that the outpoint it spends continues correctly. This phase does not prove that no other live STATE thread exists on the target.

## §3. Standard concepts and algorithms the design instantiates

| Standard concept | Guide instantiation | Obligation status |
|---|---|---|
| Abstract state machine / labeled transition system | Whole semantic world before and after, operation label `announce-maturity`, and transition certificate (§§1.4, 2.5–2.6, 15.1) | Strong, provided the whole-world execution is normative |
| State-passing semantics / State monad | Immutable predecessor world produces a fresh successor world; failure preserves the old world | Standard and appropriate; source-level mutability does not require in-place host mutation |
| Functional record update / lens law | Maturity is replaced and the frame of every other field is preserved (§§1.4, 6.5) | Strong intent; should be tied to the sole authoritative STATE schema by a lossless projection law |
| Refinement mapping / forward simulation | Semantic transition is computed independently; target bytes, constructor continuity, root edge, and public recovery must all project back to it (§§1.14, 14.3–14.7, 15.3) | Strong; the guide should state the commuting diagram as its compiler-correctness theorem |
| SSA state versions | Predecessor and successor outpoints are immutable names for two versions, connected by one typed edge (§§14.6, 17.1) | Strong for accepted transactions |
| Affine ownership | A UTXO can be accepted as spent at most once; consuming it authorizes no second accepted successor on the same canonical chain | Ledger provides affine, not linear, use. Host construction and signing are not affine |
| Optimistic concurrency / compare-and-swap | The predecessor outpoint is the expected version; a stale competing spend fails after another transition commits (§§12.4, 13.1) | Implicit and conditional on a canonical chain view; not named |
| State-thread token / extended-UTXO state machine | Singleton STATE asset and exact amount identify the continuation output (§§2.2–2.4, 10.3, 10.7) | Transaction-local preservation is strong; global uniqueness is assumed from an anchor not established in this phase |
| Continuation covenant / recursive covenant | The consumed commitment authenticates one static subtree; the leaf checks a successor under that same subtree with updated metadata (§§1.7, 9.5–9.7) | Strong soundness design; migration is deliberately excluded |
| Atomic transaction | One transaction consumes predecessor and creates successor, with complete input/output closure and finalized signing (§§10.2, 10.8, 12.1–12.7) | Atomic on ledger acceptance; evidence publication and finality are separate |
| Canonical serialization | Strict decode/re-encode equality and fixed domain, schema, widths, order, and discriminants (§6.4) | Strong for one semantic value plus one supplied representation nonce |
| Canonical rejection sampling | Search nonces from zero and choose the first candidate satisfying branch and tweak predicates (§§5.3–5.4, 6.3) | Deterministic, bounded, sound on success, but not total |
| Deterministic compilation | Equal typed inputs, stable tie-breaks, exact linking, first admissible nonce, and canonical target bytes (§§7.5, 11.2–11.4, 12.5, 12.8) | Strong only after all target environment and sponsor inputs are included in the input tuple |

The guide's guarantee set can be summarized precisely:

| Guarantee | Actual level achieved |
|---|---|
| Atomicity | Adopted for an accepted target transaction. Not an atomic commit of chain acceptance, report storage, and finality. |
| Exactly one predecessor/successor | Adopted as per-transaction cardinality. |
| Exactly-once consumption | Not adopted. UTXO consensus supplies at-most-once acceptance; liveness, abandonment, retry, and reorg prevent a general exactly-once claim. |
| No aliasing of live states | Adopted in the semantic model invariant and within one transaction. Conditional at the target because unique origin/current-root authority is outside this phase. |
| Total semantic decision procedure | Adopted: valid successor or typed refusal, no partial result (§6.5). |
| Total realization of every semantically valid transition | Not adopted: nonce exhaustion, capability failure, resource bounds, and target limitations can stop construction (§§9.8, 18.1). |
| Deterministic realization | Adopted conditional on the complete backend environment; not uniquely determined by semantic STATE and announced cycle alone because sponsorship and deployment inputs may vary. |

## §4. Findings, ranked

### F1 — CONCEPT-BLOCKER: bounded nonce grinding is not “tweak totality”

**Guide sections implicated:** §§5.3–5.4, 6.3, 6.6, 9.8, 18.1, 21 (Constructor).

The selected algorithm is deterministic bounded rejection sampling: enumerate nonces, accept the least candidate satisfying branch-side and tweak predicates, or return `RepresentationSearchExhausted`. Its standard guarantees are termination within a bound, determinism, canonical minimality, and soundness of an accepted candidate. It does not guarantee that every valid semantic STATE has a target representation. The admitted exhaustion result proves the realization function is partial over semantically valid inputs.

Calling this “totality” combines two different statements: the search procedure always terminates, and the constructor always succeeds. The first is true; the second is not. Nor does the guide state a probabilistic completeness theorem: no explicit search bound, random-oracle assumption, per-attempt admissibility model, independence argument, or upper bound on residual failure appears in the guide. The accepted prototype's finite corpus did not exercise the exceptional path, which is test evidence, not a completeness proof.

**Recommendation:** choose and name one standard contract before implementation:

- **Partial canonical normalization:** define `normalize : SemanticState -> Result<Representation, Exhausted>`, claim bounded termination, least-witness canonicality, and soundness only; make target representability an explicit compiler precondition/residual.
- **Probabilistic completeness:** retain rejection sampling but state the hash model, search bound, predicate acceptance probability, dependence assumptions, and a quantified failure bound. Do not call it total.
- **Strict totality:** replace the construction with one for which every source-valid state has a proved target representation.

Until the owner selects one, the guide does not meet the doctrine's requirement that the backend carry a named standard guarantee rather than a private weakened variant.

### F2 — DESIGN-RISK: the state-thread uniqueness theorem stops at a trusted synthetic anchor

**Guide sections implicated:** §§1.3, 2.2–2.5, 5.7, 10.3, 12.4, 15.2, 17.1–17.2, 21 (Semantic STATE).

The singleton asset, exact amount, one-input/one-output census, unchanged continuation constructor, and UTXO spend rule form the usual state-thread-token discipline. Given a unique current thread token, they preserve uniqueness and serialize accepted successors. Guide 14, however, begins from a synthetically funded predecessor and explicitly disclaims genesis and earlier root history. Current-root freshness may be report-layer evidence. Therefore the guide proves conditional continuation, not the global premise that there is only one live target STATE.

An outpoint is a unique version name, but a public current-state view is not by itself proof that no second synthetic or otherwise forged state-shaped output exists. The wording “exactly one current canonical STATE root” is valid as a semantic precondition; it is not established by this phase's target construction.

**Recommendation:** define a typed `StateThreadAnchor` or equivalent provenance object whose contract states exactly how uniqueness is established: unique asset genesis/issuance authority, starting outpoint, constructor generation, network/genesis identity, and chain-finality policy. Make every current-state handle derive from that anchor plus a contiguous verified edge sequence. If Phase 6 deliberately cannot establish the anchor, state the backend theorem conditionally: “assuming one unique anchored predecessor, an accepted transaction preserves one unique continuation.”

### F3 — DESIGN-RISK: consensus is affine, but construction and signing remain freely duplicable

**Guide sections implicated:** §§1.9, 12.4–12.7, 13.1–13.3, 17.2.

The UTXO ledger prevents two competing transactions from both consuming the same outpoint on one canonical chain. That is an affine at-most-once property at commit. The public current-state view and the guide's staged construction types do not appear to be affine: the same view may be reused to build and authorize several different announcement candidates. Finalized bytes are protected against mutation, but no rule prevents operator equivocation across two independently finalized candidates using the same predecessor.

This does not break ledger safety; only one double spend can win. It does weaken the claimed ownership discipline, complicates retries and audit evidence, and allows the operator to create multiple valid authorizations for one state version. Standard exactly-once processing cannot be obtained from UTXOs alone.

**Recommendation:** state the intended guarantee as at-most-once ledger commit. If operator non-equivocation is required, add a version-keyed authorization rule or service-side compare-and-swap/idempotency record keyed by the predecessor outpoint, and represent the construction right as an affine capability consumed when signing begins. Separately define how failed submission, replacement, timeout, and reorg affect that capability.

### F4 — DESIGN-RISK: §6.5 can be mistaken for the top-level mutation even though it transforms detached metadata

**Guide sections implicated:** §§1.1, 1.4, 2.3–2.6, 6.1, 6.5, 15.1, 15.3.

The established invariant-wrapped model path is an excellent top-level mutable-state presentation: it selects the canonical root from the world, applies the request atomically, derives the certificate, returns a successor world, and rechecks the global invariant. By contrast, the public-looking function in §6.5 accepts arbitrary predecessor metadata and bounds. It knows neither canonical root identity nor world invariants and cannot itself implement the semantic operation described in §§2.2–2.6.

This is not a problem if §6.5 is a private local delta used inside the whole-world transition. It becomes a conceptual leak if compiler clients treat it as the semantic operation and later bolt canonicality, authorization, and root succession onto it in transaction/report layers.

There is a related schema risk: §6.1 “introduces or confirms” a second `StateMetadata` value even though the authoritative semantic STATE already exists. Copying the field census into a parallel semantic structure would weaken §1.1 unless the relationship is an explicit lossless refinement rather than a manually synchronized duplicate.

**Recommendation:** define two named functions and their laws:

1. a non-public local metadata delta that changes maturity and satisfies a frame law for all other fields; and
2. the normative canonical-world transition `delta : World × Request -> Result<(World, Certificate), Refusal>`.

Require the compiler to consume a bound execution of the second, not an arbitrary result of the first. Define the source-STATE-to-metadata projection as an isomorphism or proved lossless lens derived from the single typed field owner. Keep architecture-owned bounds inside an authenticated semantic environment rather than as caller-selected policy.

### F5 — DESIGN-RISK: syntactic decoding and constructor normal-form validation are conflated

**Guide sections implicated:** §§1.5, 6.3–6.6, 9.6, 10.5, 12.5.

The separation doctrine is otherwise strong, but the metadata refusal vocabulary includes nonce canonicality and search exhaustion. A strict byte decoder can validate domain, revision, widths, discriminants, reserved fields, and round-trip encoding. It cannot decide least-admissible nonce without target constructor context. If that check enters the codec, static subtree, branch ordering, internal key, and tweak policy leak into a layer named “typed STATE metadata.”

**Recommendation:** use three explicit types and validators:

- decoded semantic metadata plus an uninterpreted representation nonce;
- syntactically canonical encoded metadata;
- constructor-normalized metadata, created only by the target normalizer with proof that the nonce is the least admissible candidate.

Move nonce noncanonicality and search exhaustion to the constructor/normalization refusal family. Semantic projection should accept only the semantic component and never need constructor policy.

### F6 — OBSERVATION: byte determinism must be parameterized by the complete environment

**Guide sections implicated:** §§7.5, 11.2–11.4, 12.3–12.8, 15.4, 16.1.

The guide has unusually strong deterministic mechanisms: strict encodings, stable typed ordering, exact-cost tree construction, two-pass linking, least admissible nonce, and freeze-before-signing. Yet the same semantic mutation may legitimately have sponsorless and sponsored transactions, different sponsor inputs, fee shapes, and deployment bindings. Therefore “equal typed inputs produce equal candidate bytes” is true only when “inputs” includes the complete target environment and funding envelope.

**Recommendation:** state two distinct properties: semantic determinism (`W` and request determine `W'`) and backend determinism (semantic bound execution plus complete deployment/funding environment determine exact candidate bytes). Do not require a unique transaction byte string for one semantic transition across different sponsor environments.

### F7 — OBSERVATION: the paired constructor is a strong continuation-covenant refinement

**Guide sections implicated:** §§1.7, 9.5–9.7, 10.5–10.8, 11.5, 14.5–14.7.

The paired recipes adopt the important obligations of a standard state-carrying UTXO machine: authenticate the predecessor datum commitment, derive rather than witness the successor datum, preserve the static validator, require one continuation output, bind authorization to finalized bytes, and independently project the target step back to semantic state. The explicit SCC cut rule is also preferable to an improvised fixed-point construction.

**Recommendation:** name the mechanism in the guide as a continuation covenant over an affine state-thread token, and state a forward-simulation theorem:

```text
if target_step(P, tx, Q) accepts
and P projects to semantic W,
then semantic_delta(W, request) = W'
and Q projects to W'.
```

State separately the converse/compiler-progress theorem, which is exactly where target representability and nonce exhaustion arise.

### F8 — OBSERVATION: compiling only one transition is a justified but explicit divergence

**Guide sections implicated:** §§1.16, 2.7, 9.4, 17.3, 26.

A conventional closed state-machine compiler would compile every operation available from every state and would define migration between validator generations. Guide 14 deliberately emits only maturity-announcement leaves, forbids placeholders, forbids subtree migration, and labels the result candidate-only. An announced successor may therefore be intentionally inert until later phases supply the rest of the lifecycle.

This divergence is justified for a research/phase gate because it avoids granting unreviewed future authority. It would not be acceptable as a release-complete mutable-state compiler. The guide states that limitation honestly.

## §5. Open questions for the owner

1. Is the invariant-wrapped whole-world execution the only normative top-level operation, with §6.5 required to remain an internal metadata delta?
2. Does the owner accept a conditional Phase-6 theorem based on a synthetic `StateThreadAnchor`, or must Guide 14 establish target-level uniqueness of the singleton STATE asset before claiming one canonical root?
3. Which nonce-search contract is intended: partial canonical normalization, probabilistic completeness with an explicit bound, or strict totality? What exact search bound and residual are acceptable?
4. May the operator sign two distinct finalized transactions spending the same predecessor? If not, which component owns non-equivocation and retry/reorg state?
5. What chain-finality rule turns a checkpointed current-state view into the authoritative mutable state, and how is a reorg rollback reflected in host state and authorization records?
6. Should `NoncanonicalRepresentationNonce` be impossible at semantic decode and exist only at constructor normalization?
7. Is the new `StateMetadata` an alias/newtype/projection of the existing canonical STATE schema, and what exact round-trip law prevents two semantic field owners?
8. Is byte determinism required only for identical complete deployment/funding environments, or is a single canonical sponsorship selection intended for each semantic request?

## §6. Wall-time table

All shell commands were read-only, returned exit code 0, and were run from the repository working directory. `/usr/bin/time` reported elapsed wall seconds. Values displayed as `0.00` are below the tool's two-decimal resolution. The only write was this required report, performed with the patch tool outside the repository. No toolchain, network, server, commit, or repository edit was used. Deviations from the brief: **NONE**.

| Step | Timed command / purpose | Wall time (s) |
|---:|---|---:|
| 1 | `git status --short --branch` — verify clean branch | 0.01 |
| 2 | `git rev-parse HEAD` — verify review revision `0.6.145-dev…` | 0.00 |
| 3 | `rg` guide headings — map all sections | 0.00 |
| 4 | `sed -n 1,698p` — mission, rulings, and exact scope | 0.00 |
| 5 | `sed -n 698,1281p` — ground truth, decisions, metadata, compiler | 0.00 |
| 6 | `sed -n 1282,2051p` — target, constructor, program, linker, ABI | 0.00 |
| 7 | `sed -n 2052,2832p` — signing, evidence, fixtures, vectors, history, resources | 0.00 |
| 8 | `sed -n 2833,3226p` — implementation waves | 0.00 |
| 9 | `sed -n 3429,3732p` — acceptance, rejection, exit, impact | 0.00 |
| 10 | `sed -n 3227,3428p` — verification section | 0.00 |
| 11 | `sed -n 3733,4162p` — report template, handoff, closing | 0.00 |
| 12 | `rg` semantic identifiers in architecture/realization/model | 0.00 |
| 13 | Read `packages/model/src/ops/maturity.rs` | 0.00 |
| 14 | Locate model STATE, world, and transaction-builder definitions | 0.00 |
| 15 | Read `packages/model/src/pool.rs` | 0.00 |
| 16 | Read current-root selection in `packages/model/src/world.rs` | 0.00 |
| 17 | Read transaction-builder consume/emit operations | 0.00 |
| 18 | Read atomic builder commit pipeline | 0.00 |
| 19 | Read builder completion tail | 0.00 |
| 20 | Locate constructor-research and guide references | 0.00 |
| 21 | Read constructor totality alternatives | 0.00 |
| 22 | Read constructor prototype residuals | 0.00 |
| 23 | Read accepted constructor result and nonce policy | 0.00 |
| 24 | Read architecture STATE root-use and projection definitions | 0.00 |
| 25 | Read first half of architecture announce-maturity operation | 0.00 |
| 26 | Read second half of architecture announce-maturity operation | 0.00 |
| 27 | Locate singleton STATE/root use across model and architecture | 0.00 |
| 28 | Read global root-exactness invariant | 0.00 |
| 29 | Read normative atomic state-transition API | 0.00 |
| 30 | Read transition-certificate structure | 0.00 |
| 31 | `wc -l plans/guides/guide_fourteen.md` — confirm 4,162 lines | 0.00 |
| 32 | `ls -ld` report directory — verify destination exists | 0.00 |
| 33 | `sed -n 1,260p` on the report — complete read-back | 0.00 |
| 34 | `wc -l -w -c` on the report — verify materialization | 0.00 |
| 35 | Constructed-pattern `rg` scan — verify prohibited sequence absent | 0.00 |
| 36 | `rg` required section headings — verify §§1–6 structure | 0.00 |
| 37 | `git status --short --branch` — verify repository unchanged | 0.00 |
| 38 | `sed -n 210,270p` on the amended report — final table read-back | 0.00 |
| 39 | Constructed-pattern `rg` scan after amendment | 0.00 |
| 40 | `git status --short --branch` — final repository check | 0.00 |
