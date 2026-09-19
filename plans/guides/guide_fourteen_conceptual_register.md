# Guide 14 conceptual preflight register

## 1. Authority preamble

This register is the conceptual-review supplement to Guide 14. It binds Guide-14 Waves 1 through 13 without editing the guide: `plans/guides/guide_fourteen.md` is immutable, and this register is its amendment channel.

Every row begins `OPEN`. A wave with an OPEN row it owns cannot pass that wave's exit, following the guide's §4.1 blocking model. A row closes only when its owning wave presents the stated testable artifact, test, or theorem and records the evidence pointer; prose intent alone does not discharge a row.

Full discharge of this register is an added conjunct of the guide's §23 Phase-6 exit gate. Passing the guide's original numbered conjuncts without closing this register does not exit Phase 6.

The owner has ruled that Guide 14 stands: there is no withdrawal and no Guide 14b. Reviewer alternatives that would replace its construction are therefore dispositions, not silent amendments.

## 2. Rulings

### R-DOCTRINE

**STATUS: ADOPTED.**

At the top level of the compiler, STATE is presented as mutable state; at the back end, every mutation is a tracked state transformation carrying standard, named algorithmic guarantees.

This doctrine governs the whole register and the discharge evidence for every row.

### R-2 CANONICALITY

**STATUS: ADOPTED.**

The target's block graph carries multiple branches under reorganization, and the constructor cycle is a projection over that graph: a projection may be rewound and re-projected over another branch at any time, as an ordinary operation of the model.

The root-history/recovery model is branch-indexed: a typed branch context consisting of branch identity and checkpoint parameterizes every observation; the semantic-edge-to-transaction projection is per branch; rewind and reproject are typed total operations; semantic history is immutable under rewind; realization-suffix invalidation is a typed event; deterministic reprojection is stable where the predecessor realization is unchanged; competing-branch realizations are representable; and the current-view compare-and-swap token is branch-bound with submission-time fencing. Wave 10 owns this ruling and feeds Waves 7 and 9.

### R-TAXONOMY

**STATUS: ADOPTED.**

There are two levels of error. Self-expected semantics broken: the target moved to a state our semantics did not intend — a double spend, or another instance under the same keys, not initiated by us — while still moving within our model of how the target can move. Panic level: the target moved in a way that does not fit our model of how it can move, which we consider undefined.

The observation vocabulary is closed over three classes: intended transitions; modeled-but-unintended, rely-conforming environment transitions such as a competing spend, same-key instance, or reorganization-surfaced realization, with typed fencing, view invalidation, and thread-contested or thread-lost standings; and model-falsifying observations with no preimage under the abstraction relation, representable only as a poison marker carrying verbatim evidence bytes. A poison marker voids derived guarantees on that branch context and can never be absorbed into an ordinary standing. Every wave guarantee is quantified over classes 1 and 2, conditional on no class-3 observation. Waves 8 and 10 own the vocabulary, which is shared with Waves 7 and 9.

### Candidate governance

**STATUS: CANDIDATE** means the formulation is bound to its owning wave now but is not yet adopted doctrine.

For every candidate below, the adoption trigger is: the ORCHESTRATOR adopts the candidate when its owning wave's evidence confirms it, recording the adoption with its evidence pointer; evidence contradicting a candidate is an OWNER EXCEPTION escalated before any adoption or overturn.

### R-1 NONCE

**STATUS: ADOPTED.**

Bounded deterministic canonicalization has typed exhaustion: the search runs candidates 0 through 4,095, refuses a zero budget, and reports exhaustion as its own refusal (`0.6.174-dev`).

Least-nonce selection is host policy carried as evidence: every rejected lower candidate is retained with its retryable refusal, and the residual that a later admissible nonce may exist beyond the budget is stated as a constant on the evidence record; such a nonce can yield a different internally consistent encoding refused only by host policy (`0.6.174-dev`).

The selection is not claimed target-enforced, and the prototype's independent scan agrees with the production nonce exactly (`0.6.178-dev`).

The encoding it canonicalizes is a tested tiling whose slicing commutes with the fields (`0.6.172-dev`). The adopted ruling feeds Waves 7 and 12.

### R-3 ANCHOR

**STATUS: ADOPTED.**

The typed `StateThreadAnchor` carries branch and network context, starting outpoint, asset or issuance provenance, constructor generation, checkpoint policy and continuity evidence, together with explicit origin and evidence status. Its conditional theorem is that one uniquely anchored predecessor has at most one accepted continuation per selected branch, with equivocation refused (`0.6.177-dev`).

The adoption rests on the constructor and anchor tests (`0.6.174-dev`, `0.6.177-dev`).

It never rests on the synthetic origin, whose residual states that it is not genesis, trusted setup, earlier history or production STATE (`0.6.177-dev`).

### R-4 EQUIVOCATION

**STATUS: ADOPTED.**

The consensus guarantee is at-most-once ledger commit: the admitted native record contains one accepted commit after duplicate authorization was refused before any signing request (`0.6.169-dev`).

The construction and signing right is an affine capability: the scoped registry consumes one authorization, caches an identical retry, refuses a competing candidate, and retains the result in its append-only record (`0.6.163-dev`).

Operator non-equivocation is an evidence obligation rather than a consensus claim: the registry records only its scoped actions, and the admitted run supplies the first-party duplicate refusal and single native acceptance without extending that record to signatures outside the registry (`0.6.163-dev`, `0.6.169-dev`).

### R-5 RECOVERY

**STATUS: CANDIDATE.** Public recovery is named exactly deterministic reproducibility from public bytes; independently authenticated verification through a trust anchor, inclusion proofs, and implementation diversity is deferred. Wave 10 owns the ruling.

Adoption trigger: the ORCHESTRATOR adopts R-5 when Wave 10's evidence confirms it and records the evidence pointer; contradictory evidence is an OWNER EXCEPTION escalated before any adoption or overturn.

### R-6 STATE-METADATA

**STATUS: ADOPTED.**

Ruling 14 confirms that `StateMetadata` is a realization-owned projection of the model's canonical `PoolState`, with its projection law proved in model conformance and no second semantic field owner (`rule:phase6:wave2-rulings`).

### R-7 CLOSURE-EXACT

**STATUS: ADOPTED.**

An emitted and linked program carries exactly the checks the contract's closure requires and nothing more: a check beyond closure is a defect of the same kind as a missing one, because the covenant's accepted set must equal the contract's permitted set and a surplus check narrows it, refusing transactions the contract permits while charging symbols, push sites, leaf bytes and resource weight and turning an encoding choice into a deployment constraint (`[ADR024-dec:closure:exact-linking]`). Closure is compositional: every family's leaf is locally sound, constraining only its own input, its own successor and its own asset, and consensus conserves assets per transaction, so any transaction satisfying every involved leaf satisfies every involved family's invariant, and composing several operations in one transaction is permitted rather than excluded. Local soundness is therefore the stated obligation of every later family's leaf, and a family whose leaf is not locally sound is the defect rather than the transaction that composes with it.

For the STATE generation this supersedes the census entries `StateCardinalityV1`, `StateSponsorIsolationV1` and `StateIssuanceAbsenceV1` at (`tab:guide14-exec:pattern-census`) and the exact-input-count, exact-output-count, sponsor-suffix, sponsor-change-role, fee-role and no-unclassified-position clauses at (`rule:guide14-exec:coordinator`). The guide text stays archived verbatim and is not edited; this entry and ruling 40 at (`rule:phase6:wave6-rulings`) are where the supersession is recorded, and the guide's §9.7 prohibition on searching for a fixed point by repeated hashing and §17.3 refusal of constructor migration are untouched. What those clauses claimed is discharged instead by facts already in the tree: the four absence facts of `StateStructuralEvidence` collapse to the one that no second STATE object occurs, which the self-position pin at input zero, the singleton's non-reissuable declaration and consensus conservation establish together, the last two entering as named external evidence roles beside substrate conservation rather than as silent assumptions.

The discharge argument is published as a table, one row per realization relation of the announcement — 26 rows — each naming what discharges it: a positional check in the leaf, the self-position pin, consensus conservation, a named external evidence role, a semantic component, or MODEL-SCOPE. The MODEL-SCOPE class names the five relations the realization evaluates over the whole observed transaction rather than over the positions the operation claims — `AllowedObjectFamilies` on each side, `SponsorIsolation`, `CanonicalDeltaPolicy`, `OpenFlowPolicy` and `SponsorEnvelopeMultiplicity` — which the staged region-scoping refit of ruling 52 re-scopes to the operation's own region or retires. Until that refit lands those relations are enforced by no leaf, a composed transaction is accepted on-chain and outside the model, and every published closure record says so rather than implying that the model already covers it.

## 3. Conceptual preflight register

Severity is the highest rank among the cited reviews. `OWNER-BLOCKING` marks an additional owner-mandated row with no ranked-review severity.

| ID | Title | Sources | Severity | Guide sections implicated | Owning wave(s) | Testable discharge criterion | Status |
|---|---|---|---|---|---|---|---|
| `G14C-01` | Canonical semantic owner, metadata projection, and guarded update | [A-F4](../reviews/guide_fourteen_conceptual_review_a.md), [C-3](../reviews/guide_fourteen_conceptual_review_c.md), ruling R-6 | DESIGN-RISK | §§1.1, 1.4, 2.3–2.6, 6.1, 6.5, 10.5, 15.1, 15.3, 15.5, 16.3, 20.4 | 1 | Wave 1 presents one authoritative `PoolState` field owner; a non-public maturity field projection or product decomposition with `GetPut`, `PutGet`, `PutPut`, complement-preservation, and codec-commutation laws; a guarded invariant-wrapped world transition that alone is the normative semantic operation; and property or exhaustive finite-domain tests for those laws. This criterion depends on CANDIDATE R-6 and must provide the confirming evidence pointer for orchestrator adoption. | CLOSED — Ruling 14 confirms the realization-owned metadata projection and its model-conformance proof (`rule:phase6:wave2-rulings`). |
| `G14C-02` | Abstract-to-concrete refinement and named construction guarantee | [A-F7](../reviews/guide_fourteen_conceptual_review_a.md), [B-7](../reviews/guide_fourteen_conceptual_review_b.md), [C-1](../reviews/guide_fourteen_conceptual_review_c.md), [C-9](../reviews/guide_fourteen_conceptual_review_c.md), ruling R-DOCTRINE | CONCEPT-BLOCKER | Mission and thesis; §§1.1, 1.4, 1.7, 6.5, 7.2–7.4, 9.5–9.7, 10.5–10.8, 11.5, 14.3–14.7, 17.1, 23 | 2, 8, 9, 10, 13 | The owning waves publish a typed abstraction relation over semantic state, concrete constructor state, request, and bound environment; state and test the forward-simulation theorem that every accepted concrete step corresponds to exactly one allowed semantic step and related successor; state supported-step completeness only under named representability assumptions; and make each safety, continuity, history, and recovery report an evidence witness for a theorem clause. The design record names the back end as a recursive covenant over a commitment-chained single-use seal using authenticated persistent path-copying and a branch-bound predecessor version token, with all qualifications explicit. | OPEN |
| `G14C-03` | Bounded nonce canonicalization, host leastness, and exhaustion residual | [A-F1](../reviews/guide_fourteen_conceptual_review_a.md), [B-3](../reviews/guide_fourteen_conceptual_review_b.md), [B-4](../reviews/guide_fourteen_conceptual_review_b.md), [C-4](../reviews/guide_fourteen_conceptual_review_c.md), ruling R-1 | CONCEPT-BLOCKER | §§1.5, 5.3–5.4, 6.3, 6.6, 9.6, 9.8, 10.7, 12.5, 16.6–16.7, 18.1, 21–23 | 4 for candidate evidence; 7 and 12 for fed ABI and resource criteria | Wave 4 specifies a bounded deterministic normalizer with exact attempt convention, unique least host-selected witness on success, typed exhaustion, and soundness tests; proves that target execution authenticates internal consistency but not leastness; and records the later-admissible-nonce residual explicitly. Wave 7 demonstrates host-policy refusal without claiming target invalidity, and Wave 12 reports the bounded corpus and attempts without promoting empirical success to totality. The Wave-4 evidence confirms adopted R-1 while preserving those later-wave obligations. | OPEN — Wave 4 discharged the bounded normalizer, host-leastness evidence, typed exhaustion, residual and independent-scan agreement; Waves 7 and 12 remain (`0.6.174-dev`, `0.6.178-dev`). |
| `G14C-04` | Conditional state-thread uniqueness from typed provenance | [A-F2](../reviews/guide_fourteen_conceptual_review_a.md), ruling R-3 | DESIGN-RISK | §§1.3, 2.2–2.5, 5.7, 10.3, 12.4, 15.2, 17.1–17.2, 21 | 4 | Wave 4 defines a typed `StateThreadAnchor` carrying branch/network context, starting outpoint, asset or issuance provenance, constructor generation, checkpoint policy, and continuity evidence; states and tests the theorem that one uniquely anchored predecessor has at most one accepted continuation per selected branch; and records that the synthetic Phase-6 predecessor proves neither genesis nor global origin uniqueness. The constructor and anchor tests supply the evidence (`0.6.174-dev`, `0.6.177-dev`). | CLOSED |
| `G14C-05` | Affine construction right and operator non-equivocation evidence | [A-F3](../reviews/guide_fourteen_conceptual_review_a.md), ruling R-4 | DESIGN-RISK | §§1.9, 12.4–12.7, 13.1–13.3, 17.2 | 3 | Wave 3 models the version-bound construction and signing right as an affine capability, states only at-most-once ledger commit as the consensus guarantee, defines retry, replacement, timeout, and branch-reorganization outcomes for that capability, and supplies a duplicate-authorization negative plus an operator non-equivocation evidence record. The affine registry and admitted native record supply the criterion's confirming evidence pointer. | CLOSED — The affine registry and admitted native record confirm R-4 and supply the non-equivocation evidence pointer (`0.6.163-dev`, `0.6.169-dev`). |
| `G14C-06` | Layer-owned decoding, semantic, and construction refusals | [A-F5](../reviews/guide_fourteen_conceptual_review_a.md), [C-8](../reviews/guide_fourteen_conceptual_review_c.md) | DESIGN-RISK | §§1.5, 6.3–6.6, 9.6, 9.8, 10.5, 12.5 | 1, 4 | Waves 1 and 4 provide distinct closed decode, maturity-transition, and constructor-normalization error sums; types prevent decoder or semantic code from evaluating least-nonce policy; orchestration composes the sums explicitly; and tests pin precedence only if exact refusal identity is normative, otherwise pin only fail-closed success versus failure. | OPEN — The three closed sums and their policy boundary are present, but no production orchestration yet composes decode, transition and construction refusals explicitly (`0.6.146-dev`, `0.6.174-dev`). |
| `G14C-07` | Semantic and complete-environment backend determinism | [A-F6](../reviews/guide_fourteen_conceptual_review_a.md) | OBSERVATION | §§7.5, 11.2–11.4, 12.3–12.8, 15.4, 16.1 | 2, 7 | Wave 2 states that equal canonical world plus request and bound protocol context determine one semantic successor, while Wave 7 states that a bound semantic execution plus the complete deployment, branch, funding, sponsorship, fee, and signer-public environment determines exact candidate bytes. Property tests prove equality within each complete input tuple and demonstrate that distinct admitted sponsor environments need not share transaction bytes. | OPEN |
| `G14C-08` | Authenticated cuts form a feedback-edge proof | [B-1](../reviews/guide_fourteen_conceptual_review_b.md) | CONCEPT-BLOCKER | §§9.7, 11.2, 21–22; Wave 6 | 6 | Wave 6 classifies and validates every runtime cut, removes all validated cut edges, requires the residual dependency graph to be acyclic, derives a canonical topological order, and includes a negative SCC with two edge-disjoint cycles where one authenticated cut leaves one cycle and must refuse. | OPEN |
| `G14C-09` | Binding-time types make the recursive symbol safe | [B-2](../reviews/guide_fourteen_conceptual_review_b.md) | CONCEPT-BLOCKER | §§1.7–1.8, 9.5–9.7, 11.1–11.3, 12.6 | 6 | Wave 6 gives every reference a closed binding-time type, represents `StateStaticSubtree` only as a spend-time introspected or witnessed-then-authenticated strategy, proves that its root value is never serialized as a literal beneath itself, and tests the exact introspection role, reconstruction equation, and equality check while refusing ordinary link-time relocation of that symbol. | OPEN |
| `G14C-10` | Canonical linker normal form and permutation invariance | [B-5](../reviews/guide_fourteen_conceptual_review_b.md) | DESIGN-RISK | §§7.5, 11.2–11.4, 21; Wave 6 | 6 | Wave 6 defines stable keys for definitions, reference edges, SCC members and identities, condensation-DAG ties, relocation sites, leaves, equal-cost tree candidates, and serialization; rebuilds substitutions simultaneously from pristine structured input and reparses the result; and proves byte equality under permutations of definitions, references, relocations, leaves, and equal-cost ties. | OPEN |
| `G14C-11` | Branch-indexed projection, rewind, and reprojection | [B-6](../reviews/guide_fourteen_conceptual_review_b.md), [C-2](../reviews/guide_fourteen_conceptual_review_c.md), [C-7](../reviews/guide_fourteen_conceptual_review_c.md), ruling R-2 | CONCEPT-BLOCKER | §§1.3, 5.7, 10.3, 12.4, 14.6, 17.1–17.3, 21, 23 | 10; feeds 7 and 9 | Wave 10 provides the typed branch context, per-branch semantic-edge projection, total rewind and reproject operations, typed realization-suffix invalidation, immutable semantic history, branch-relative competing realizations, and branch-bound compare-and-swap token with submission-time fencing. Tests fork after one predecessor, realize competing successors, rewind one branch, preserve semantic history, invalidate only the affected realization suffix, and reproduce identical bytes when predecessor realization and complete environment are unchanged; reports call the native evidence a checkpoint-bound edge certificate rather than a complete origin history. | OPEN |
| `G14C-12` | Closed observation taxonomy and branch poison | ruling R-TAXONOMY | OWNER-BLOCKING | §§12.4, 14.2–14.7, 16.8, 17.1–17.2, 23 | 8, 10; vocabulary shared with 7 and 9 | Waves 8 and 10 implement the three closed observation classes and typed class-2 responses; encode a class-3 observation only as a branch-context poison marker carrying verbatim evidence bytes; prevent every ordinary safety, continuity, history, or recovery standing from accepting that marker; and state every guarantee with the required class-1/class-2 and no-class-3 quantifier. Tests cover competing spend, same-key instance, reorganization-surfaced realization, and a model-falsifying byte observation with no abstraction preimage. | OPEN |
| `G14C-13` | Deterministic reproducibility from public bytes | [C-5](../reviews/guide_fourteen_conceptual_review_c.md), ruling R-5 | DESIGN-RISK | §§1.8, 5.1, 14.7, 16.13, 17.2, 21, 23 | 10 | Wave 10 names the guarantee exactly deterministic reproducibility from public bytes, versions and binds the complete public handoff, reconstructs successor metadata and output 0 byte-for-byte without creator-private or ambient state, and records explicit non-claims for independent implementation, trust anchor, inclusion proof, canonical chain authentication, and static-subtree data availability. This criterion depends on CANDIDATE R-5 and must provide the confirming evidence pointer for orchestrator adoption. | OPEN |
| `G14C-14` | Codec bijection, field-layout law, and commitment assumptions | [C-6](../reviews/guide_fourteen_conceptual_review_c.md) | DESIGN-RISK | §§5.3–5.5, 6.4, 9.3–9.6, 12.8, 14.5 | 1, 4, 9 | Waves 1, 4, and 9 state and test both codec round trips, semantic injectivity across distinct semantic-value and nonce pairs, and the theorem that field slicing commutes with semantic projection; record exact tagged-hash domain separation plus collision and second-preimage assumptions; and keep exact byte equality, commitment-root equality under assumptions, and hidden-subtree-content equality as distinct evidence claims. | OPEN — Wave 4 discharged field-slicing commutation, typed field commitments and the separation of byte, root and commitment claims; Wave 9 remains (`0.6.172-dev`, `0.6.174-dev`, `0.6.178-dev`). |
| `G14C-15` | Candidate-only lifecycle and honest Phase-6 handoff | [A-F8](../reviews/guide_fourteen_conceptual_review_a.md) | OBSERVATION | §§1.16, 2.7, 9.4, 17.3, 23, 26 | 13 | Wave 13's result matrix and handoff state that only maturity announcement is compiled, later STATE operations and constructor migration remain unavailable, the successor may be intentionally inert, and no release-complete state-machine or lifecycle-wide coinduction claim is made; an audit test or plans-validator rule fails if the Phase-6 gate record promotes the candidate to final or omits those non-claims. | OPEN |

## 4. Dispositions declined or subsumed

| Review recommendation | Disposition | Reason |
|---|---|---|
| A-F1, B-3, B-4, and C-4 nonce remedies | Subsumed by `G14C-03` | R-1 selects one bounded host-policy contract and records the residual, so target leastness, probabilistic completeness, and strict-totality alternatives are not independent rows. |
| A-F4 and C-3 semantic-owner and lens remedies | Subsumed by `G14C-01` | One criterion separates the lawful field projection from the guarded world transition and prevents `StateMetadata` from becoming a second field owner. |
| A-F5 and C-8 refusal-algebra remedies | Subsumed by `G14C-06` | Both findings require the same layer-owned decode, transition, and constructor error boundary. |
| A-F7, B-7, C-1, and C-9 standard-form recommendations | Subsumed by `G14C-02` | The refinement theorem and named recursive-covenant construction are one compiler-correctness obligation, with the reports serving as theorem witnesses. |
| B-6, C-2, and C-7 chain-selection and history remedies | Superseded and subsumed by `G14C-11` | R-2 adopts the stronger branch-indexed rewind and reprojection model rather than a single-finality-policy remedy. |
| B-8 separate code-and-metadata architecture | Declined as an independent row | Guide 14 stands with no Guide 14b: the current authenticated runtime-cut strategy remains, while replacing the constructor with a companion metadata object would be a new architecture. |

Every other ranked finding owns a distinct row above. No ranked finding is dropped.

## 5. Wave coverage summary

The counts below are ownership assignments, so one cross-wave row appears under each wave that must discharge part of its criterion.

| Wave | Owned row IDs | Count |
|---:|---|---:|
| 1 | `G14C-01`, `G14C-06`, `G14C-14` | 3 |
| 2 | `G14C-02`, `G14C-07` | 2 |
| 3 | `G14C-05` | 1 |
| 4 | `G14C-03`, `G14C-04`, `G14C-06`, `G14C-14` | 4 |
| 5 | None; it consumes already-bound semantic and constructor obligations | 0 |
| 6 | `G14C-08`, `G14C-09`, `G14C-10` | 3 |
| 7 | `G14C-03`, `G14C-07` | 2 |
| 8 | `G14C-02`, `G14C-12` | 2 |
| 9 | `G14C-02`, `G14C-14` | 2 |
| 10 | `G14C-02`, `G14C-11`, `G14C-12`, `G14C-13` | 4 |
| 11 | None; sponsorship consumes the Wave-7 ABI and shared observation vocabulary | 0 |
| 12 | `G14C-03` | 1 |
| 13 | `G14C-02`, `G14C-15` | 2 |

Wave 5 closed with no owned register row and no candidate adoption: its complete pattern record and carrier projection consume already-bound semantic and constructor obligations, while `R-5 RECOVERY` retains its Wave-10 trigger (`0.6.186-dev`).

Wave 6 owns `G14C-08`, `G14C-09` and `G14C-10`, and opens with one adopted ruling and no candidate adoption: `R-7 CLOSURE-EXACT` records the closure-exact principle and the STATE-generation supersession of §10.1's three census entries and §10.2's six coordinator clauses, ruling 45 at (`rule:phase6:wave6-rulings`) binds the graph rows to typed binding times, validated cuts and a residual acyclic graph, and ruling 50 fixes the adversarial criterion each of the three rows closes on, so none of them closes on a broad success suite. `R-5 RECOVERY` remains the register's sole CANDIDATE, its trigger belonging to Wave 10 and unchanged by this record.

The register contains 12 OPEN rows and three CLOSED rows and 26 wave-ownership assignments. R-2 additionally feeds `G14C-11` constraints into Waves 7 and 9, and R-TAXONOMY shares `G14C-12` vocabulary with Waves 7 and 9 without changing the primary ownership counts above.
