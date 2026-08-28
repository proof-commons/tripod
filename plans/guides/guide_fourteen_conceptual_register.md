# Guide 14 conceptual preflight register

## 1. Authority preamble

This register is the conceptual-review supplement to Guide 14. It binds Guide-14 Waves 1 through 13 without editing the guide: `plans/guides/guide_fourteen.md` is immutable, and this register is its amendment channel.

Every row begins `OPEN`. A wave with an OPEN row it owns cannot pass that wave's exit, following the guide's §4.1 blocking model. A row closes only when its owning wave presents the stated testable artifact, test, or theorem and records the evidence pointer; prose intent alone does not discharge a row.

Full discharge of this register is an added conjunct of the guide's §23 Phase-6 exit gate. Passing the guide's original numbered conjuncts without closing this register does not exit Phase 6.

The owner has ruled that Guide 14 stands: there is no withdrawal and no Guide 14b. Reviewer alternatives that would replace its construction are therefore dispositions, not silent amendments.

## 2. Rulings

### R-DOCTRINE

**STATUS: ADOPTED.**

> "at the top level of the compiler it should be presented as mutable state, at the back end it should be a tracked state transformation using standard algorithmic guarantees."

— owner ruling (in-chat)

This doctrine governs the whole register and the discharge evidence for every row.

### R-2 CANONICALITY

**STATUS: ADOPTED.**

> "the dag needs to include the fact there are multiple branches for target reorganizations, we have a cycle that is projected over a dag, but that may be re-winded and projected over another branch at any time."

— owner ruling (in-chat)

The root-history/recovery model is branch-indexed: a typed branch context consisting of branch identity and checkpoint parameterizes every observation; the semantic-edge-to-transaction projection is per branch; rewind and reproject are typed total operations; semantic history is immutable under rewind; realization-suffix invalidation is a typed event; deterministic reprojection is stable where the predecessor realization is unchanged; competing-branch realizations are representable; and the current-view compare-and-swap token is branch-bound with submission-time fencing. Wave 10 owns this ruling and feeds Waves 7 and 9.

### R-TAXONOMY

**STATUS: ADOPTED.**

> "there are two levels of error; self-expected semantics broken: the target moved to a state where our semantics didn't intend. (i.e. a double spend, another instance with the same private keys, not initialed by us). and panic level: the target went into something that we consider undefined. (we didn't model the target moved in a way that doesn't fit our model of how it can move."

— owner ruling (in-chat)

The observation vocabulary is closed over three classes: intended transitions; modeled-but-unintended, rely-conforming environment transitions such as a competing spend, same-key instance, or reorganization-surfaced realization, with typed fencing, view invalidation, and thread-contested or thread-lost standings; and model-falsifying observations with no preimage under the abstraction relation, representable only as a poison marker carrying verbatim evidence bytes. A poison marker voids derived guarantees on that branch context and can never be absorbed into an ordinary standing. Every wave guarantee is quantified over classes 1 and 2, conditional on no class-3 observation. Waves 8 and 10 own the vocabulary, which is shared with Waves 7 and 9.

### Candidate governance

**STATUS: CANDIDATE** means the formulation is bound to its owning wave now but is not yet adopted doctrine.

> "Register lands now with the findings bound to waves; the four ruling slots stay marked PENDING for later. You formally adopt them as the wave gives give evidence. Adopted as candidates now."

— owner ruling (in-chat), adoption wording amended by the later ruling below

> "It is your choice to adopt them when the wave gains evidence. Bring to me in exception where the evidence contradicts."

— owner ruling (in-chat)

For every candidate below, the adoption trigger is: the ORCHESTRATOR adopts the candidate when its owning wave's evidence confirms it, recording the adoption with its evidence pointer; evidence contradicting a candidate is an OWNER EXCEPTION escalated before any adoption or overturn.

### R-1 NONCE

**STATUS: CANDIDATE.** Bounded deterministic canonicalization has typed exhaustion; least-nonce selection is host policy carried as evidence and is not claimed target-enforced; the residual that a later admissible nonce can yield a different internally consistent encoding refused only by host policy is recorded explicitly. Wave 4 owns the ruling and feeds Waves 7 and 12.

Adoption trigger: the ORCHESTRATOR adopts R-1 when Wave 4's evidence confirms it and records the evidence pointer; contradictory evidence is an OWNER EXCEPTION escalated before any adoption or overturn.

### R-3 ANCHOR

**STATUS: CANDIDATE.** Uniqueness is a conditional theorem over a typed `StateThreadAnchor` provenance object; Phase 6's synthetic origin makes no genesis claim. Wave 4 owns the ruling.

Adoption trigger: the ORCHESTRATOR adopts R-3 when Wave 4's evidence confirms it and records the evidence pointer; contradictory evidence is an OWNER EXCEPTION escalated before any adoption or overturn.

### R-4 EQUIVOCATION

**STATUS: CANDIDATE.** The guarantee is at-most-once ledger commit; the construction and signing right is an affine capability; operator non-equivocation is an evidence obligation, not a consensus claim. Wave 3 owns the ruling.

Adoption trigger: the ORCHESTRATOR adopts R-4 when Wave 3's evidence confirms it and records the evidence pointer; contradictory evidence is an OWNER EXCEPTION escalated before any adoption or overturn.

### R-5 RECOVERY

**STATUS: CANDIDATE.** Public recovery is named exactly deterministic reproducibility from public bytes; independently authenticated verification through a trust anchor, inclusion proofs, and implementation diversity is deferred. Wave 10 owns the ruling.

Adoption trigger: the ORCHESTRATOR adopts R-5 when Wave 10's evidence confirms it and records the evidence pointer; contradictory evidence is an OWNER EXCEPTION escalated before any adoption or overturn.

### R-6 STATE-METADATA

**STATUS: CANDIDATE.** `StateMetadata` is a projection of the model's canonical `PoolState` with a proved lossless round-trip law and is never a second semantic field owner. Wave 1 owns the ruling.

Adoption trigger: the ORCHESTRATOR adopts R-6 when Wave 1's evidence confirms it and records the evidence pointer; contradictory evidence is an OWNER EXCEPTION escalated before any adoption or overturn.

## 3. Conceptual preflight register

Severity is the highest rank among the cited reviews. `OWNER-BLOCKING` marks an additional owner-mandated row with no ranked-review severity.

| ID | Title | Sources | Severity | Guide sections implicated | Owning wave(s) | Testable discharge criterion | Status |
|---|---|---|---|---|---|---|---|
| `G14C-01` | Canonical semantic owner, metadata projection, and guarded update | [A-F4](../reviews/guide_fourteen_conceptual_review_a.md), [C-3](../reviews/guide_fourteen_conceptual_review_c.md), R-6 owner ruling (in-chat) | DESIGN-RISK | §§1.1, 1.4, 2.3–2.6, 6.1, 6.5, 10.5, 15.1, 15.3, 15.5, 16.3, 20.4 | 1 | Wave 1 presents one authoritative `PoolState` field owner; a non-public maturity field projection or product decomposition with `GetPut`, `PutGet`, `PutPut`, complement-preservation, and codec-commutation laws; a guarded invariant-wrapped world transition that alone is the normative semantic operation; and property or exhaustive finite-domain tests for those laws. This criterion depends on CANDIDATE R-6 and must provide the confirming evidence pointer for orchestrator adoption. | OPEN |
| `G14C-02` | Abstract-to-concrete refinement and named construction guarantee | [A-F7](../reviews/guide_fourteen_conceptual_review_a.md), [B-7](../reviews/guide_fourteen_conceptual_review_b.md), [C-1](../reviews/guide_fourteen_conceptual_review_c.md), [C-9](../reviews/guide_fourteen_conceptual_review_c.md), R-DOCTRINE owner ruling (in-chat) | CONCEPT-BLOCKER | Mission and thesis; §§1.1, 1.4, 1.7, 6.5, 7.2–7.4, 9.5–9.7, 10.5–10.8, 11.5, 14.3–14.7, 17.1, 23 | 2, 8, 9, 10, 13 | The owning waves publish a typed abstraction relation over semantic state, concrete constructor state, request, and bound environment; state and test the forward-simulation theorem that every accepted concrete step corresponds to exactly one allowed semantic step and related successor; state supported-step completeness only under named representability assumptions; and make each safety, continuity, history, and recovery report an evidence witness for a theorem clause. The design record names the back end as a recursive covenant over a commitment-chained single-use seal using authenticated persistent path-copying and a branch-bound predecessor version token, with all qualifications explicit. | OPEN |
| `G14C-03` | Bounded nonce canonicalization, host leastness, and exhaustion residual | [A-F1](../reviews/guide_fourteen_conceptual_review_a.md), [B-3](../reviews/guide_fourteen_conceptual_review_b.md), [B-4](../reviews/guide_fourteen_conceptual_review_b.md), [C-4](../reviews/guide_fourteen_conceptual_review_c.md), R-1 owner ruling (in-chat) | CONCEPT-BLOCKER | §§1.5, 5.3–5.4, 6.3, 6.6, 9.6, 9.8, 10.7, 12.5, 16.6–16.7, 18.1, 21–23 | 4 for candidate evidence; 7 and 12 for fed ABI and resource criteria | Wave 4 specifies a bounded deterministic normalizer with exact attempt convention, unique least host-selected witness on success, typed exhaustion, and soundness tests; proves that target execution authenticates internal consistency but not leastness; and records the later-admissible-nonce residual explicitly. Wave 7 demonstrates host-policy refusal without claiming target invalidity, and Wave 12 reports the bounded corpus and attempts without promoting empirical success to totality. This criterion depends on CANDIDATE R-1 and Wave 4 must provide the confirming evidence pointer for orchestrator adoption. | OPEN |
| `G14C-04` | Conditional state-thread uniqueness from typed provenance | [A-F2](../reviews/guide_fourteen_conceptual_review_a.md), R-3 owner ruling (in-chat) | DESIGN-RISK | §§1.3, 2.2–2.5, 5.7, 10.3, 12.4, 15.2, 17.1–17.2, 21 | 4 | Wave 4 defines a typed `StateThreadAnchor` carrying branch/network context, starting outpoint, asset or issuance provenance, constructor generation, checkpoint policy, and continuity evidence; states and tests the theorem that one uniquely anchored predecessor has at most one accepted continuation per selected branch; and records that the synthetic Phase-6 predecessor proves neither genesis nor global origin uniqueness. This criterion depends on CANDIDATE R-3 and must provide the confirming evidence pointer for orchestrator adoption. | OPEN |
| `G14C-05` | Affine construction right and operator non-equivocation evidence | [A-F3](../reviews/guide_fourteen_conceptual_review_a.md), R-4 owner ruling (in-chat) | DESIGN-RISK | §§1.9, 12.4–12.7, 13.1–13.3, 17.2 | 3 | Wave 3 models the version-bound construction and signing right as an affine capability, states only at-most-once ledger commit as the consensus guarantee, defines retry, replacement, timeout, and branch-reorganization outcomes for that capability, and supplies a duplicate-authorization negative plus an operator non-equivocation evidence record. This criterion depends on CANDIDATE R-4 and must provide the confirming evidence pointer for orchestrator adoption. | OPEN |
| `G14C-06` | Layer-owned decoding, semantic, and construction refusals | [A-F5](../reviews/guide_fourteen_conceptual_review_a.md), [C-8](../reviews/guide_fourteen_conceptual_review_c.md) | DESIGN-RISK | §§1.5, 6.3–6.6, 9.6, 9.8, 10.5, 12.5 | 1, 4 | Waves 1 and 4 provide distinct closed decode, maturity-transition, and constructor-normalization error sums; types prevent decoder or semantic code from evaluating least-nonce policy; orchestration composes the sums explicitly; and tests pin precedence only if exact refusal identity is normative, otherwise pin only fail-closed success versus failure. | OPEN |
| `G14C-07` | Semantic and complete-environment backend determinism | [A-F6](../reviews/guide_fourteen_conceptual_review_a.md) | OBSERVATION | §§7.5, 11.2–11.4, 12.3–12.8, 15.4, 16.1 | 2, 7 | Wave 2 states that equal canonical world plus request and bound protocol context determine one semantic successor, while Wave 7 states that a bound semantic execution plus the complete deployment, branch, funding, sponsorship, fee, and signer-public environment determines exact candidate bytes. Property tests prove equality within each complete input tuple and demonstrate that distinct admitted sponsor environments need not share transaction bytes. | OPEN |
| `G14C-08` | Authenticated cuts form a feedback-edge proof | [B-1](../reviews/guide_fourteen_conceptual_review_b.md) | CONCEPT-BLOCKER | §§9.7, 11.2, 21–22; Wave 6 | 6 | Wave 6 classifies and validates every runtime cut, removes all validated cut edges, requires the residual dependency graph to be acyclic, derives a canonical topological order, and includes a negative SCC with two edge-disjoint cycles where one authenticated cut leaves one cycle and must refuse. | OPEN |
| `G14C-09` | Binding-time types make the recursive symbol safe | [B-2](../reviews/guide_fourteen_conceptual_review_b.md) | CONCEPT-BLOCKER | §§1.7–1.8, 9.5–9.7, 11.1–11.3, 12.6 | 6 | Wave 6 gives every reference a closed binding-time type, represents `StateStaticSubtree` only as a spend-time introspected or witnessed-then-authenticated strategy, proves that its root value is never serialized as a literal beneath itself, and tests the exact introspection role, reconstruction equation, and equality check while refusing ordinary link-time relocation of that symbol. | OPEN |
| `G14C-10` | Canonical linker normal form and permutation invariance | [B-5](../reviews/guide_fourteen_conceptual_review_b.md) | DESIGN-RISK | §§7.5, 11.2–11.4, 21; Wave 6 | 6 | Wave 6 defines stable keys for definitions, reference edges, SCC members and identities, condensation-DAG ties, relocation sites, leaves, equal-cost tree candidates, and serialization; rebuilds substitutions simultaneously from pristine structured input and reparses the result; and proves byte equality under permutations of definitions, references, relocations, leaves, and equal-cost ties. | OPEN |
| `G14C-11` | Branch-indexed projection, rewind, and reprojection | [B-6](../reviews/guide_fourteen_conceptual_review_b.md), [C-2](../reviews/guide_fourteen_conceptual_review_c.md), [C-7](../reviews/guide_fourteen_conceptual_review_c.md), R-2 owner ruling (in-chat) | CONCEPT-BLOCKER | §§1.3, 5.7, 10.3, 12.4, 14.6, 17.1–17.3, 21, 23 | 10; feeds 7 and 9 | Wave 10 provides the typed branch context, per-branch semantic-edge projection, total rewind and reproject operations, typed realization-suffix invalidation, immutable semantic history, branch-relative competing realizations, and branch-bound compare-and-swap token with submission-time fencing. Tests fork after one predecessor, realize competing successors, rewind one branch, preserve semantic history, invalidate only the affected realization suffix, and reproduce identical bytes when predecessor realization and complete environment are unchanged; reports call the native evidence a checkpoint-bound edge certificate rather than a complete origin history. | OPEN |
| `G14C-12` | Closed observation taxonomy and branch poison | R-TAXONOMY owner ruling (in-chat) | OWNER-BLOCKING | §§12.4, 14.2–14.7, 16.8, 17.1–17.2, 23 | 8, 10; vocabulary shared with 7 and 9 | Waves 8 and 10 implement the three closed observation classes and typed class-2 responses; encode a class-3 observation only as a branch-context poison marker carrying verbatim evidence bytes; prevent every ordinary safety, continuity, history, or recovery standing from accepting that marker; and state every guarantee with the required class-1/class-2 and no-class-3 quantifier. Tests cover competing spend, same-key instance, reorganization-surfaced realization, and a model-falsifying byte observation with no abstraction preimage. | OPEN |
| `G14C-13` | Deterministic reproducibility from public bytes | [C-5](../reviews/guide_fourteen_conceptual_review_c.md), R-5 owner ruling (in-chat) | DESIGN-RISK | §§1.8, 5.1, 14.7, 16.13, 17.2, 21, 23 | 10 | Wave 10 names the guarantee exactly deterministic reproducibility from public bytes, versions and binds the complete public handoff, reconstructs successor metadata and output 0 byte-for-byte without creator-private or ambient state, and records explicit non-claims for independent implementation, trust anchor, inclusion proof, canonical chain authentication, and static-subtree data availability. This criterion depends on CANDIDATE R-5 and must provide the confirming evidence pointer for orchestrator adoption. | OPEN |
| `G14C-14` | Codec bijection, field-layout law, and commitment assumptions | [C-6](../reviews/guide_fourteen_conceptual_review_c.md) | DESIGN-RISK | §§5.3–5.5, 6.4, 9.3–9.6, 12.8, 14.5 | 1, 4, 9 | Waves 1, 4, and 9 state and test both codec round trips, semantic injectivity across distinct semantic-value and nonce pairs, and the theorem that field slicing commutes with semantic projection; record exact tagged-hash domain separation plus collision and second-preimage assumptions; and keep exact byte equality, commitment-root equality under assumptions, and hidden-subtree-content equality as distinct evidence claims. | OPEN |
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

The register contains 15 OPEN rows and 26 wave-ownership assignments. R-2 additionally feeds `G14C-11` constraints into Waves 7 and 9, and R-TAXONOMY shares `G14C-12` vocabulary with Waves 7 and 9 without changing the primary ownership counts above.
