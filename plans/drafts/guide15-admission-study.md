# Study: Admission of Guide 15 as the Phase-7 execution guide

Status: a design study over the tree at the clippy placement fix above 0.7.0-dev; it audits the Guide 15 concept draft against the Phase-7 card, the Phase-6 closeout and the tree, decides nothing, and ends with the decision page the backlog's next-gate line reserves for the project's owner. The study's own gate is the documentation suite and the labels tests, and it claims no workspace or native-target pass of its own; where the audit relies on such a pass it cites the gate of record over the tip it names.

## 0. In one breath

The Guide 15 concept draft is a 4,654-line proposal with twenty-three governing rulings, eight entry procedures or gates, twelve preflight questions, fourteen suggested waves and twenty-nine numbered main sections. Admission is still the first Phase-7 gate: the card is Active, but no execution guide is chartered. The entry audit below distinguishes 55 literal condition bullets from ten repository record fields and gives all 65 an evidence-backed standing: 50 HOLDS, two DOES NOT HOLD and thirteen UNDECIDED. Phase 6 exited on its amended gate while Guide 14 §23 remains NOT PASSED on five conditional conjuncts; 128 of 206 Safety-matrix rows remain outstanding and ninety remain unlinked to a published relation. The evidence establishes an inherited compact-ASH candidate and an explicit public-boundary policy, but does not establish complete typed burn/clear declarations or an accepted Phase-7 constructor-generation decision. The decision page preserves the choice of admission form and amendments while making the existing default explicit: no wave opens unanswered.

## 1. What admission is

### The precedent and its limits

Guide 14 has two distinct artifacts: `plans/guides/guide_fourteen_concept.md` remains a concept draft, while `plans/guides/guide_fourteen.md` begins “Draft execution guide; not yet executed” and explicitly supersedes the concept as execution direction.

The execution header retains Phase, entry, affected packages, required result, exclusions, authority and trust boundary, but turns entry into the amended Phase-5 exit, the accepted STATE-constructor prototype and its own preflight gate; it also identifies the primary typed operation and human operation name.

The additional Initial review basis names the supplied 51-file static-review selection archived as review 10, with every review row a hypothesis until reproduced against the complete working tree; that qualification is part of the precedent rather than a licence to copy its findings into another phase.

A preflight register records a concrete hypothesis, priority and required disposition, then distinguishes finding status CONFIRMED, REFUTED or RECLASSIFIED from execution status DONE or OPEN with a named blocker (`tab:guide14-exec:preflight`).

A wave plan assigns deliverables, exits and commit-sized work in dependency order; Guide 14 has implementation waves at (`sec:guide14-exec:waves`), whereas its concept draft has suggested waves at (`sec:guide14:waves`).

An execution guide therefore needs more than an amended status word: its questions become recorded dispositions or named blockers, its scope becomes executable obligations, and its verification and evidence boundaries attach to those obligations.

### What the opening record actually reconciled

Row `T11-001` in `plans/history/backlog-history.md` reconciled three maintained declarations: the Phase-6 card's status block, the backlog header and phase-table cell, and the guides index's explanation and executed-guide registration.

The row's evidence was already present: the execution guide was archived under `T6-001`, Wave 0 had run and closed under `T6-002` and `T6-031`, and the conceptual preflight register bound Waves 1–13 under `T8-001`; the next wave it named was Wave 1.

That was retrospective reconciliation of declarations with an established execution track. Guide 15 admission is prospective: the backlog explicitly requires admission before any wave opens, so that precedent cannot be used to pretend a Guide-15 wave has already run.

The guides index separates (`tab:guides:executed`) from (`tab:guides:concepts`); the guide rule makes a guide subordinate to the backlog, package contracts, phase card, ADRs and accepted decisions (`rule:guides:authority`).

### Two forms of admission

Promotion in place would amend `plans/guides/guide_fifteen_concept.md` into execution direction, change its status and header, integrate dispositions and waves, and reconcile its index classification; its benefit is a stable path, while its cost is that the path still says concept and no separate unchanged concept artifact remains in the active tree.

The backlog's phrase “admission of Guide 15” is compatible with promotion in place, but the Guide-14 pair supplies no instance of that file-level method; this audit does not invent an in-place precedent.

A separate execution-guide file would preserve the concept draft, explicitly supersede it as execution direction, and register the new artifact in the executed-guide table once chartered; this follows the Guide-14 shape, at the cost of another maintained text and an explicit supersession boundary.

Either form needs coherent changes to the guide, `plans/guides/README.md`, the Phase-7 card's status and entry wording, and `plans/backlog.md`'s next-gate line and phase row; its current-condition narrative must remain consistent with those declarations.

Those are prospective admission changes, not changes made by this study. The only integrations of this study are its draft-index row and alphabetical draft-census entry.

## 2. The entry conditions at this base

### How to read the standings

HOLDS means the cited tree establishes the stated structural fact or accepted policy at the scope in Note; a historical observation remains attached to its named candidate and gate. DOES NOT HOLD means the tree contradicts the requirement as written. UNDECIDED means an admission disposition, exact candidate selection or fresh verification is still needed; it is not a failed experiment.

The 55 condition bullets are transcribed from §§2.1–2.7 of the Guide 15 concept draft. Its §2.8 is a procedure, not another bullet list, so the remaining ten rows audit its record fields for determinability from the tree.

Repository entry first asks for `git status --porcelain=v1 --untracked-files=all` and `git log -1 --oneline`, then requires a stop and classification of every change if the tree is dirty (`rule:guide15:repository-entry`). Those commands establish the local observation; they do not constitute the full repository gate.

The clean-tree record below cites the gate's own clean-tree check. A determinable record field may hold while its proposed Phase-7 use remains undecided: knowing the existing constructor does not accept a new generation, and knowing the historical compact candidate does not choose the bundle for revalidation.

| Condition | Bullet | Standing | Evidence | Note |
|---|---|---|---|---|
| §2.1 Phase-6 entry | Phase 6 has exited on the current tree | HOLDS | (`sec:phase6:exit-assessment`) | Exited on the amended gate; this does not pass unamended Guide 14 §23. |
| §2.1 Phase-6 entry | maturity announcement remains green under its candidate constructor generation | UNDECIDED | (`sec:phase6:gate-record`) | Acceptance is recorded over the candidate; no fresh maturity run is claimed at this base. |
| §2.1 Phase-6 entry | canonical STATE metadata has one typed owner | HOLDS | packages/realization/src/state.rs; packages/realization/src/state_codec.rs | The typed metadata owner and canonical codec are inherited; §23 row 3 is MET. |
| §2.1 Phase-6 entry | predecessor and successor constructor reconstruction are implemented | HOLDS | (`sec:phase6:gate-record`) | §23 rows 4 and 10 are MET for the accepted candidate's constructor reconstruction. |
| §2.1 Phase-6 entry | operator signing is finalized-byte bound | HOLDS | packages/transaction/src/state_signing.rs; (`sec:phase6:gate-record`) | §23 row 6 is MET; operator authorization binds finalized bytes. |
| §2.1 Phase-6 entry | root-history evidence is edge-based | HOLDS | packages/vectors/src/maturity_history.rs; (`sec:phase6:gate-record`) | Edge-based evidence exists; row 11 retains the checkpoint and environment conditions. |
| §2.1 Phase-6 entry | public successor reconstruction is implemented | HOLDS | packages/vectors/src/maturity_recovery.rs; (`sec:phase6:gate-record`) | Implemented recovery consumes archived mined readback through a pinned import boundary. |
| §2.1 Phase-6 entry | target, deployment, request, response, and provenance bindings remain exact | HOLDS | packages/vectors/src/maturity_continuity.rs; (`sec:phase6:gate-record`) | Inherited validated evidence binds the exact subject; no broader target identity is inferred. |
| §2.1 Phase-6 entry | candidate and final states remain distinct | HOLDS | packages/linker/src/state_bundle.rs; (`sec:phase6:exit-assessment`) | Accepted candidate, release-complete false, no final calibration or deployment claim. |
| §2.1 Phase-6 entry | the tree is clean | HOLDS | scripts/gate-clean-tree.sh; (`sec:phase6:gate-record`) | The gate-of-record clean-tree procedure is available; initial inspection found this checkout clean. |
| §2.2 Semantic entry | `burn`, `compact-ash`, and `clear` remain in architecture and realization scope under their exact existing identifiers | DOES NOT HOLD | packages/architecture/src/ids.rs; packages/realization/src/declarations/mod.rs | All three architecture identifiers exist, but typed realization dispatch lacks Burn and Clear. |
| §2.2 Semantic entry | burn, compact, and clear relations validate against architecture | DOES NOT HOLD | packages/realization/src/declarations/mod.rs | UnsupportedOperationDeclaration prevents complete burn/clear declaration validation. |
| §2.2 Semantic entry | the complete STATE clear assignment is known | HOLDS | packages/model/src/ops/ash.rs; docs/attestation/realization.md | ClearAsh changes only stored y_l; total Y falls by the same X because Y is derived. |
| §2.2 Semantic entry | `tag-recon` is architecture-owned and exact | HOLDS | packages/architecture/src/ids.rs; packages/architecture/src/spec.rs | TagId::Recon owns tag-recon; the clear destruction declaration names it. |
| §2.2 Semantic entry | burn record and event semantics are typed | HOLDS | packages/model/src/history.rs; packages/model/src/certify.rs | BurnRecord and BurnProjection are typed model objects; compiler-facing realization is still missing. |
| §2.2 Semantic entry | compact ASH remains event-silent | HOLDS | packages/realization/src/declarations/compact_ash.rs; packages/model/src/ops/ash.rs | Compact is ownerless-lateral and produces no burn or clear projection. |
| §2.2 Semantic entry | sponsor-value opacity remains enforced | UNDECIDED | packages/compiler/src/sponsor_region.rs; (`sec:phase6:exit-assessment`) | Inherited opacity holds in its scope; burn/clear enforcement is not delivered and the region refit remains open. |
| §2.2 Semantic entry | no target-specific type enters realization | HOLDS | plans/packages/realization.md; packages/realization/src/lib.rs | The contract and public semantic surface retain target independence. |
| §2.2 Semantic entry | lifecycle obligations remain explicit | HOLDS | packages/realization/src/lifecycle.rs; (`sec:phase6:exit-assessment`) | Lifecycle obligations and candidate incompleteness remain explicit. |
| §2.3 Public-declassification entry | the accepted public boundary is revalidated | UNDECIDED | (`sec:public-opening:result`) | Accepted policy is known; Phase-7 revalidation is a Wave-0 deliverable, not a completed run here. |
| §2.3 Public-declassification entry | the initial ASH representation is explicitly selected | HOLDS | (`sec:public-opening:result`) | The inherited initial public ownerless maintenance representation is explicit only. |
| §2.3 Public-declassification entry | any private-receipt burn path is classified as normalized-first or separately admitted direct-private | UNDECIDED | (`sec:public-opening:result`); plans/guides/guide_fifteen_concept.md | Normalized-first is the inherited route; a Phase-7 execution disposition has not been recorded. |
| §2.3 Public-declassification entry | no unauthenticated opening is accepted | HOLDS | (`sec:public-opening:result`); plans/packages/target-elements.md | Authenticated opening remains unsupported; no unauthenticated amount is an accepted substitute. |
| §2.3 Public-declassification entry | public ASH remains constructible by an unrelated process | UNDECIDED | packages/vectors/src/resource_study.rs; plans/phases/04-compact-ash.md | Public compact construction is inherited; burn-to-clear public construction is not yet evidenced. |
| §2.3 Public-declassification entry | no report attachment substitutes for chain-public value | HOLDS | (`sec:public-opening:result`); packages/transaction/src/view.rs | The explicit public value is construction input; a report attachment supplies no opening authority. |
| §2.4 Compact-ASH entry | the Phase-4 compact-ASH operation still builds | UNDECIDED | packages/compiler/src/operation_plan.rs; plans/phases/04-compact-ash.md | The operation exists and has a passing gate of record; current-base build is not rerun by this study. |
| §2.4 Compact-ASH entry | its compiler plan, patterns, linker, ABI, vectors, and resource study remain green | UNDECIDED | plans/history/guide-12-completion-report.md; packages/vectors/src/resource_study.rs | Historical candidate results do not certify a fresh regression run over this base. |
| §2.4 Compact-ASH entry | its ASH constructor is compatible with burn-created and clear-residual ASH, or the incompatibility is explicitly dispositioned | UNDECIDED | packages/tapscript/src/bundle.rs; packages/linker/src/bundle.rs | No compiled burn output or clear residual exists for a cross-operation constructor equality proof. |
| §2.4 Compact-ASH entry | the inherited operation emits no authorization primitive | HOLDS | packages/tapscript/src/pattern.rs; packages/transaction/src/abi.rs | The inherited compact candidate is permissionless; sponsor authorization is a separate envelope. |
| §2.4 Compact-ASH entry | no stale candidate limit is treated as final calibration | HOLDS | plans/phases/04-compact-ash.md; plans/packages/linker.md | Candidate bounds remain candidate; no final calibration follows from phase exit. |
| §2.5 STATE generation entry | the Phase-6 constructor generation is identified exactly | HOLDS | packages/tapscript/src/state_constructor.rs; (`sec:phase6:gate-record`) | The accepted maturity-only constructor is identified by its retained bundle and evidence, not a new generation name. |
| §2.5 STATE generation entry | adding clear is recognized as a static-subtree change | HOLDS | packages/tapscript/src/state_constructor.rs; plans/guides/guide_fifteen_concept.md | Adding clear changes the static leaf set and authenticated subtree; the concept draft states this explicitly. |
| §2.5 STATE generation entry | the Phase-7 generation policy of §1.14 is accepted or replaced by an explicit migration decision | UNDECIDED | plans/guides/guide_fifteen_concept.md; plans/backlog.md | The concept draft proposes synthetic Phase-7 origin; no execution admission accepts it yet. |
| §2.5 STATE generation entry | no test attempts to spend a Phase-6 STATE with an uncommitted Phase-7 leaf | HOLDS | packages/transaction/src/state_abi.rs; packages/linker/src/state_bundle.rs | The delivered candidate has no clear leaf; its existing constructor checks do not admit an uncommitted leaf. |
| §2.5 STATE generation entry | synthetic Phase-7 origin non-claims are agreed in advance | UNDECIDED | plans/guides/guide_fifteen_concept.md; plans/backlog.md | Synthetic origin and no-migration non-claims need an explicit admission disposition. |
| §2.6 Target entry | reviewed target definition validates | HOLDS | packages/target-elements/src/definition.rs; plans/packages/target-elements.md | Reviewed V2 definition and validator are implemented; this is a structural inherited reading. |
| §2.6 Target entry | development binding is welded to the exact target | HOLDS | packages/target-elements/src/weld.rs; packages/target-elements/src/deployment.rs | Development binding validates against the exact reviewed target definition. |
| §2.6 Target entry | explicit `U` and explicit value introspection are reviewed | HOLDS | packages/target-elements/src/opcode.rs; plans/packages/target-elements.md | Explicit asset/value introspection is within the reviewed target; authenticated opening is not. |
| §2.6 Target entry | checked fixed-width addition, subtraction, and comparison are reviewed | HOLDS | packages/target-elements/src/opcode.rs; plans/packages/tapscript.md | Reviewed fixed-width primitives exist; the clear-specific schedule remains a separate preflight obligation. |
| §2.6 Target entry | target transaction conservation remains external evidence | HOLDS | packages/target-elements/src/confidential.rs; plans/packages/target-elements.md | Conservation is an external target dependency, never a semantic proof of burn or clear. |
| §2.6 Target entry | selected owner and sponsor sighash profiles are reviewed | UNDECIDED | packages/target-elements/src/authorization.rs; packages/transaction/src/live_signing.rs | Inherited owner/sponsor profiles are reviewed; their exact Phase-7 selection and binding remain to be recorded. |
| §2.6 Target entry | STATE constructor hashing and tweak requirements remain reviewed | HOLDS | packages/target-elements/src/definition.rs; packages/tapscript/src/state_constructor.rs | The Phase-6 hashing and tweak requirements remain reviewed; expanded-subtree evidence is not inherited. |
| §2.6 Target entry | execution domain and leaf version are observed | HOLDS | (`sec:phase6:gate-record`); packages/target-elements/src/definition.rs | The accepted target observations bind the execution domain and leaf version of that candidate. |
| §2.6 Target entry | consensus and relay policy remain separate | HOLDS | packages/target-elements/src/resource.rs; (`sec:phase6:exit-assessment`) | Zero fee floors and relay bounds remain explicit; consensus acceptance does not imply ordinary relayability. |
| §2.6 Target entry | no mock is gate-eligible | HOLDS | plans/guides/guide_fifteen_concept.md; packages/vectors/src/maturity_native.rs | The concept draft requires real target evidence; mocks and first-party fixtures cannot discharge native rows. |
| §2.7 Evidence entry | canonical evidence plans have no unchecked constructors | HOLDS | packages/vectors/src/plan.rs; packages/vectors/src/maturity_evidence.rs | Canonical evidence plans derive through validated inputs and preserve their internal invariants. |
| §2.7 Evidence entry | coverage changes only from validated report wrappers | HOLDS | packages/vectors/src/plan.rs; packages/vectors/src/maturity_report.rs | Coverage discharge consumes validated evidence/report wrappers; declared requirements are not observations. |
| §2.7 Evidence entry | native refusals retain observed layers and exact subjects | HOLDS | packages/vectors/src/observed_boundary.rs; packages/vectors/src/maturity_mutants.rs | Observed layer and bound subject remain attached to native refusals. |
| §2.7 Evidence entry | wrong-boundary refusals are not counted as answers | HOLDS | packages/vectors/src/observed_boundary.rs | matches_boundary requires exact equality; a refusal elsewhere does not answer the row. |
| §2.7 Evidence entry | report-layer answerability is not counted as observed evidence | HOLDS | packages/vectors/src/maturity_evidence.rs; (`sec:phase6:gate-record`) | Report-layer required and observed evidence remain distinct; 15 report-layer-required rows remain outstanding. |
| §2.7 Evidence entry | accepted controls ground negative target observations | HOLDS | packages/vectors/src/maturity_mutants.rs; (`sec:phase6:gate-record`) | Admitted negative observations retain accepted controls; the reading is limited to those admitted observations. |
| §2.7 Evidence entry | target rejection carries no accepted artifact | HOLDS | packages/target-elements-conformance/src/protocol.rs; packages/vectors/src/operation.rs | Response validation distinguishes acceptance artifacts from refusal shapes. |
| §2.7 Evidence entry | response-shape validation is exact | HOLDS | packages/target-elements-conformance/src/protocol.rs | The strict revision-8 response contract rejects incompatible shapes and subjects. |
| §2.7 Evidence entry | canonical reports retain their generic execution transcripts | HOLDS | packages/vectors/src/maturity_report.rs; packages/vectors/src/maturity_continuity_report.rs | Validated reports retain generic execution evidence rather than an unbound success flag. |
| §2.7 Evidence entry | safety, event, continuity, history, accounting, and resources remain separate | UNDECIDED | packages/vectors/src/maturity_report.rs; plans/guides/guide_fifteen_concept.md | Five inherited report roles are separate; Phase-7 event/accounting report production is not delivered. |
| §2.8 Repository record | starting revision | HOLDS | Cargo.toml; plans/phases/README.md | The clippy placement fix above 0.7.0-dev. |
| §2.8 Repository record | Phase-6 result revision | HOLDS | (`sec:phase6:gate-record`); (`sec:phase6:exit-assessment`) | Evidence base 0.6.305-dev; gate-record landing 0.6.306-dev; amended closeout carried at 0.7.0-dev. |
| §2.8 Repository record | architecture schema and semantic hashes | HOLDS | packages/architecture/src/spec.rs; plans/registers/identities.md; packages/model/generated/architecture.json | Schema 18; semantic and behavioural values are the registered fields in the generated architecture publication, cited by path rather than copied digests. |
| §2.8 Repository record | realization version | HOLDS | Cargo.toml; packages/architecture/src/spec.rs | 0.7.0-dev, derived from the workspace version. |
| §2.8 Repository record | target contract revision | HOLDS | packages/target-elements/src/definition.rs | TargetContractVersion::V2, the supported reviewed contract. |
| §2.8 Repository record | executor protocol revision | HOLDS | packages/target-elements-conformance/src/protocol.rs | NATIVE_PROTOCOL_SCHEMA = 8; earlier archived runs retain their own historical protocol. |
| §2.8 Repository record | compact-ASH candidate revision | UNDECIDED | plans/history/guide-12-completion-report.md; plans/phases/04-compact-ash.md | The completion record names 0.4.1-dev; the exact inherited bundle/revision for a Phase-7 rerun still needs selection. |
| §2.8 Repository record | STATE-constructor generation | HOLDS | packages/tapscript/src/state_constructor.rs; (`sec:phase6:gate-record`) | The Phase-6 maturity-only candidate in the gate record; Phase-7 generation is not yet accepted. |
| §2.8 Repository record | selected public-value policy | HOLDS | (`sec:public-opening:result`) | Explicit-only boundary, normalized-first with private change; public-committed opening is deferred. |
| §2.8 Repository record | clean-tree result | HOLDS | scripts/gate-clean-tree.sh; (`sec:phase6:gate-record`) | The gate-of-record clean-tree check over 0.6.305-dev is MET; this is the recorded result, not a fresh workspace gate. |

### Counts and the eight whole-gate readings

The 65 data rows comprise 50 HOLDS, two DOES NOT HOLD and thirteen UNDECIDED. The 55 condition bullets contribute 41 HOLDS, two DOES NOT HOLD and twelve UNDECIDED; the ten repository fields contribute nine HOLDS and one UNDECIDED.

Phase-6 entry has nine HOLDS and one UNDECIDED: the phase-exit fact holds on the amended assessment, but “remains green” is not established by this documentation study's verification scope (`gate:guide15:phase6-entry`). The whole gate needs candidate-specific revalidation without reopening or enlarging the completed closeout.

Semantic entry has six HOLDS, two DOES NOT HOLD and one UNDECIDED (`gate:guide15:semantic-entry`). Its literal complete realization scope and validated burn/clear declarations do not exist, although model semantics do; this gate does not hold as a conjunction.

Public-declassification entry has three HOLDS and three UNDECIDED (`gate:guide15:declassification-entry`). The accepted explicit policy is determinate, but Phase-7 revalidation, route disposition and full public lifecycle evidence still prevent an unconditional whole-gate pass.

Compact-ASH entry has two HOLDS and three UNDECIDED (`gate:guide15:compact-entry`). An inherited accepted candidate is different from a current-base regression result or an equality proof across constructors that have not yet been compiled.

STATE-constructor generation entry has three HOLDS and two UNDECIDED (`gate:guide15:state-generation-entry`). The current constructor and the effect of adding a leaf are known; acceptance of synthetic origin or an explicit migration alternative remains an admission decision.

Target entry has nine HOLDS and one UNDECIDED (`gate:guide15:target-entry`). The reviewed V2 substrate is inherited, but the Phase-7 signing-profile selection and operation-specific use must be fixed before implementation can claim this gate complete.

Evidence entry has nine HOLDS and one UNDECIDED (`gate:guide15:evidence-entry`). The inherited validation boundaries hold in their scopes; new event and accounting evidence cannot be assumed to exist because five maturity report types already exist.

Repository entry has nine determinable fields and one UNDECIDED selection. The compact candidate must be identified for the actual execution baseline; observations of cleanliness and full-gate status must retain their own scope and timing rather than becoming timeless guarantees.

### The amended Phase-6 evidence boundary

The amended bypass condition covers the admitted refusals and retains the NUMS key-path residual and unoffered wrong-control recipe. The amended wrong-code-subtree condition covers vectors actually offered to a target and retains wrong-static-subtree as a carried first-party case.

The amended resource condition covers the accepted 53-byte variable item and observed weight 2,091, while the 86-byte whole-schedule item remains replay-only against the 80-byte relay bound. The amended coverage condition requires complete disposition, including every outstanding row, rather than complete answers.

These are the four narrowed conditions of (`sec:phase6:exit-assessment`), with no evidence added by the amendments; the other six conditions retain their original text. The full repository gate of record over 0.6.305-dev is 54 Ok, zero Fail, exit zero, with its clean-tree check MET (`sec:phase6:gate-record`).

That gate's passing run is inherited evidence at its named tip. It neither proves the unamended §23 conjunction nor supplies a new workspace or native-target pass for this study.

## 3. The header discrepancy

### What the two headers say

The Guide 15 concept draft's Entry line names the Phase-6 exit gate, the accepted public-declassification policy, and the accepted compact-ASH candidate. The Phase-7 card's Entry line names only the first two, and its status explicitly says “both entry conditions hold” (`plans/guides/guide_fifteen_concept.md`); (`plans/phases/07-burn-and-clear.md`).

The concept draft's detailed entry conditions are stronger than either short header: they additionally require typed scope, constructor policy, target readiness, evidence boundaries and repository records. A header is an index to these obligations, not their discharge.

### Is compact ASH accepted?

The Phase-4 card is Exited and records twelve accepted and matched live submissions, coverage 100 of 211, a section-29 audit at 94 of 105, and a 47-of-47 local gate with byte-identical document reproduction (`plans/phases/04-compact-ash.md`).

The Guide-12 register is a set of sixteen review hypotheses, not an automatic certificate of acceptance (`tab:guide12-exec:preflight`). Its completion record reports fifteen CONFIRMED and DONE and R08 RECLASSIFIED and DONE, and the Phase-4 card supplies the phase-level accepted result (`plans/history/guide-12-completion-report.md`).

Thus an accepted compact-ASH candidate exists at its declared scope. That is not final calibration, a complete negative matrix, a production deployment, or proof that burn-created and clear-residual ASH share its exact constructor; `plans/packages/vectors.md` still records one discharged compact negative row and 71 outstanding.

The historical completion record is over 0.4.1-dev and explicitly distinguishes its audit from deferred repository-gate verdicts; the card's accepted exit cannot be replaced by treating every completion-template field as independently passed.

### Alternative A: align the card upward

The card could say: “The three inherited entry results hold: Phase 6 exited on its amended gate, the explicit public-declassification policy is accepted, and the Phase-4 compact-ASH candidate is accepted at its recorded scope; detailed Guide-15 entry and preflight obligations remain to be dispositioned before implementation.”

Its Entry line would add the accepted compact-ASH candidate. The concept draft's Entry line would retain all three and explicitly qualify Phase 6 by the amended assessment and compact ASH by its Phase-4 candidate scope.

This alternative makes the phase header expose all three inherited results, while avoiding the false implication that every detailed entry bullet is already true. It does not convert the historical compact acceptance into a Phase-7 compatibility proof.

### Alternative B: keep phase entry separate from guide entry

The card could retain its two Entry conditions and say: “Both phase-entry conditions hold on the amended Phase-6 exit and accepted public policy; admission of Guide 15 also requires disposition of the inherited compact-ASH candidate and the concept draft's detailed entry and preflight requirements.”

The concept draft's Entry line could say: “The Phase-7 card's two entry results, plus the accepted Phase-4 compact-ASH candidate at its recorded scope, subject to detailed entry and preflight disposition.”

This alternative treats the third result as a guide-specific prerequisite rather than a missing phase-entry condition. The consequence is an explicit distinction between opening the phase and chartering its execution guide, which the current “Active” status already permits.

Neither alternative changes whether the Phase-4 candidate exists or whether Phase 6 exited. No header is edited here; the admission call chooses the coherent declaration set and its execution consequences.

## 4. The preflight questions

### 3.1 Exact semantic field ownership

Answerable in part: architecture owns the operation, quantity, tag and projection identifiers in `packages/architecture/src/ids.rs`; `BurnReceipts` owns requested fresh ASH, receipt change and records in `packages/model/src/ops/burn.rs`; `CompactAsh` and `ClearAsh` compute aggregate, decrement and residual in `packages/model/src/ops/ash.rs`; the kernel/certifier and `packages/model/src/history.rs` own derived events and certificates. The complete compiler-facing census is not delivered because `packages/realization/src/declarations/mod.rs` lacks burn and clear; the admission decision must assign its completion before those plans consume it (`q:guide15:semantic-census`).

### 3.2 ASH representation

Answerable for the inherited initial mode: explicit value is selected at (`sec:public-opening:result`), including public ownerless maintenance, and retained by Guide-12's representation rule (`rule:guide12-exec:representation`). Public-committed ASH would require a new accepted result addressing generator derivation, encoding-domain mismatch and unbound supplied parity; a concept-draft preference cannot make that result exist (`q:guide15:ash-representation`).

### 3.3 Private burn route

Answerable as the existing default: normalized-first with private change is the accepted route, while direct private burn remains deferred; the public-declassification result explicitly does not make full private consumption without residual blinding available. The admission decision must record that scope or require the separate direct-private evidence before claiming it; no positive direct-private class follows from confidential conservation alone (`q:guide15:private-burn`); (`plans/research/public-declassification.md`).

### 3.4 ASH constructor continuity

Needs a disposition: the inherited constructor exists in `packages/tapscript/src/bundle.rs` and `packages/linker/src/bundle.rs`, but there are no compiled burn and clear counterparts to compare. Admission must require exact equality under one target, deployment, representation and bundle, or explicitly charter class transitions; shared semantic Meta::Ash is not byte-level constructor equality (`q:guide15:ash-constructor`).

### 3.5 Burn record carrier

Needs a decision: model records and canonical ordering exist in `packages/model/src/ops/burn.rs`, `packages/model/src/kernel.rs` and `packages/model/src/certify.rs`, but no target burn ABI selects the carrier. Admission must choose the chain-public encoding and recovery boundary, including its signature coverage, fresh-ASH association and resistance to copied-payload provenance; the concept draft's list of candidate carriers is not a selection (`q:guide15:burn-record-carrier`).

### 3.6 Burn-record cardinality and value rule

Answerable semantically: zero records are allowed; `BurnRecordMax` bounds count; each amount is positive; indices start at zero and are contiguous in output order; duplicate or skipped indices reject; arbitrary address partitioning is permitted rather than an invented one-record-per-address restriction. `packages/model/src/ledger.rs` keeps over-claim a credit verdict, with aggregate comparison in u128: records exceeding fresh ASH receive no credit without erasing the authenticated burn; exact target encoding still needs implementation (`q:guide15:burn-record-rule`); (`docs/attestation/realization.md`).

### 3.7 Clear STATE assignment

Answerable as the authoritative law: `ClearAsh` changes only y_l to y_l minus X and copies the other five stored fields in `packages/model/src/ops/ash.rs`; `PoolState::y` derives Y as y_l plus y_t in `packages/model/src/pool.rs`. Therefore Y also decreases by X, but it is not an additional stored assignment; the six-field `StateMetadata` in `packages/realization/src/state.rs` has no independent Y field. The missing typed realization transition must implement that law rather than materialize a seventh semantic field (`q:guide15:clear-assignment`).

### 3.8 Clear arithmetic lowering

Answerable at the operation level: the authoritative clamp uses bounded summation, subtraction and minimum comparisons, with no multiplication or division; model checked arithmetic rejects overflow and zero progress in `packages/model/src/ops/ash.rs`. The reviewed target primitives exist in `packages/target-elements/src/opcode.rs`, but admission must require the exact operand encoding, signedness, bounds, branch schedule and consumed comparisons before calling a target lowering proven; existence of primitives alone does not discharge the schedule (`q:guide15:clear-arithmetic`).

### 3.9 STATE generation transition

Needs an admission decision: the Guide 15 concept draft proposes a new synthetic Phase-7 origin, maturity and clear in the expanded static subtree, maturity revalidation, no cross-generation spend and no migration claim. `packages/tapscript/src/state_constructor.rs` and the Phase-6 constructor evidence make the subtree identity material; continuity from the accepted Phase-6 STATE would instead require an explicit migration operation and evidence plan before clear (`q:guide15:state-generation`).

### 3.10 Event projection independence

Answerable at the model boundary: `packages/model/src/certify.rs`, `packages/model/src/history.rs` and `packages/model/src/ledger.rs` distinguish burn provenance, record acceptance, compact silence, clear projection and accounting, while the inherited maturity reports already separate safety, continuity, history, recovery and resources. A complete Phase-7 report implementation is absent, so admission must require separate validated roles and exact bindings rather than interpret a generic success as all seven answers (`q:guide15:event-independence`).

### 3.11 Permissionless public construction

Answerable for inherited compact ASH at its recorded candidate scope, using public construction inputs and an optional sponsor-local capability (`plans/phases/04-compact-ash.md`); (`packages/transaction/src/view.rs`). Clear has no delivered target ABI or independent construction demonstration; admission must make fresh-process clear and residual-ASH reuse explicit evidence obligations, with any creator-only opening or metadata dependency a blocker (`q:guide15:permissionless`).

### 3.12 Attestation query boundary

The tree contains an internal attestation consumer in `packages/model/src/ledger.rs` and its indexer conformance tests, but this audit finds no Phase-7 external burn-report interchange consumer in the delivered pipeline. The default is internal event and record validation without a new namespace or report digest; a named external consumer would require ADR-022 adjudication before its interface is implemented (`q:guide15:attestation-query`); (`adr/022-interchange-conventions.md`).

### The preflight conjunction

The preflight gate does not hold at this base: the twelve questions do not all have execution dispositions, the burn/clear realization census is incomplete, and the carrier, constructor and generation obligations remain unresolved (`gate:guide15:preflight`).

That result bars a claim that implementation can begin immediately under the concept draft's literal preflight wording. It does not decide whether admission may charter a bounded Wave 0 to complete dispositions; the execution guide must make the ordering explicit.

## 5. What an execution guide adds

### A register that controls entry

The execution artifact needs a preflight register with each of the twelve questions, the 65 audited entries, exact evidence, finding status, execution status, blocker and required exit; Guide 14's distinction between hypothesis and reproduced finding is the useful precedent.

The register must explicitly distinguish inherited policy, structural implementation, observed candidate evidence and new Phase-7 choices. A planned positive case or report field cannot be used to mark its own entry requirement as already satisfied.

The semantic-entry contradiction needs a disposition: either an authorized preparatory tranche creates the missing declarations before implementation entry, or the execution guide explicitly separates known model law from the typed declarations Wave 1 delivers; leaving both requirements unchanged creates a circular gate.

### A review basis grounded in this tree

The initial review basis should name the Guide 15 concept draft, the Phase-7 card, the amended Phase-6 assessment and gate record, Guide-12/Guide-11 results, typed architecture, realization declarations, model transitions, target contract and the affected package contracts.

Guide 14's review 10 and 51-file selection are historical evidence for its own register; they are not a new review of burn and clear. The four supplied candidate discrepancies and the additional findings below need explicit dispositions without making editorial repairs prerequisites to unrelated semantic proofs.

The governing-ruling census runs from (`rule:guide15:consume-pipeline`) through (`rule:guide15:exactness`): twenty-three concept-draft rulings establish proposed constraints, including operation separation, public ASH, exact authorization, constructor generations, independent event anchors, failure layers and candidate-only status.

### The concept draft's complete section inventory

Sections 1–3 cover governing rulings, entry conditions and preflight questions; §§4–9 cover the semantic burn contract, public ASH and compact maintenance, semantic clear contract, Phase-7 STATE constructor generation, compiler plans, and target assessment and proof patterns.

Sections 10–15 cover linking and constructor closure, candidate transaction and witness ABIs, burn signing and authorization, evidence architecture, semantic fixtures and projections, and the required vector matrix.

Sections 16–21 cover event and attestation integrity, resource evidence, relation-indexed coverage, assurance boundaries, suggested implementation waves, and focused verification.

Sections 22–29 cover the full batch gate, acceptance criteria, rejection criteria, identity/schema/security/interchange/dependency impact, final Phase-7 result matrix, Guide-15 exit checklist, completion report template, and handoff after Guide 15.

This inventory is the scope of the 4,654-line concept draft in `plans/guides/guide_fifteen_concept.md`, not a statement that any of those sections has been executed.

### Wave and bite shapes

The concept draft proposes fourteen waves at (`sec:guide15:waves`); an execution guide must turn each into a bounded dependency, deliverable, verification and exit rather than simply retain its suggested commit.

| Wave | Proposed execution bite shape | Evidence boundary before advancement |
|---|---|---|
| 0 | Revalidate Phase 6 and close the preflight register | Exact candidate baseline, policy and generation dispositions; no inherited non-claim silently discharged |
| 1 | Typed burn and clear declarations, event/accounting projections and architecture weld | Independent model conformance and exact semantic census before lowering |
| 2 | Validated burn/clear plans and compact regression | Relation, carrier, layout, lifecycle and coverage censuses plus corruption oracles |
| 3 | Target and representation closure | Reviewed arithmetic, explicit value and signing requirements; unsupported alternatives stated |
| 4 | Public ASH constructor integration | Exact fresh/aggregate/residual constructor compatibility or explicit transitions |
| 5 | Burn patterns | Owner authorization, live-only inputs, one fresh ASH, records and isolated sponsor flow |
| 6 | Expanded STATE constructor and clear patterns | Exact decrement, floor, residual, destruction and unchanged fields; maturity revalidated |
| 7 | Linker extension | Deterministic symbols, tree/control recipes and complete carrier closure |
| 8 | Candidate ABIs and burn signing | Finalized-byte binding, canonical records, permissionless clear and public construction |
| 9 | Burn safety and event evidence | Independent event-type and value anchors, copied records and over-claim behavior |
| 10 | ASH maintenance evidence | Compact silence, constructor continuity and unrelated-process reuse |
| 11 | Clear safety and STATE evidence | Exact accounting, one STATE edge, no RESV edge and residual closure |
| 12 | Resource study | Separate prediction/observation, exact subjects and absent dimensions, no final calibration |
| 13 | Phase-7 gate and handoff | Card's ten exit bullets, complete result matrix, explicit remaining non-claims |

The execution plan must settle whether Wave 0 is authorized preflight work or is itself blocked by every literal entry bullet. The current backlog authorizes neither interpretation before admission.

Bites should follow ownership boundaries: a semantic declaration and its conformance proof, a compiler plan and its validator/oracles, a target pattern and its exact schedule, a linked bundle and its closure checks, an ABI/signing boundary and its mutants, then bound report producers and their evidence.

A bite that changes an inherited interface must identify the missing fact, smallest sufficient generalization, identity/schema impact and compact/live/maturity regressions; the concept draft already requires this at (`rule:guide15:consume-pipeline`).

### Package contracts that would move

| Package | Existing contract and delivered boundary | Required Phase-7 addition if chartered |
|---|---|---|
| architecture | packages/architecture/src/ids.rs and packages/architecture/src/spec.rs already name burn, compact, clear, tags and projections | Correct only an established omission; do not invent identities for absent consumers |
| realization | plans/packages/realization.md; three typed declarations are delivered | Burn/clear declarations, complete assignments, relations, event/accounting and lifecycle projections |
| compiler | plans/packages/compiler.md; validated target-operation plans and maturity requirements | Burn/clear plans, public availability, exact carriers and operation-specific validators |
| target-elements | plans/packages/target-elements.md; reviewed V2 contract and capability census | Exact operation assessments and any justified missing capability, without promoting unsupported opening |
| tapscript | plans/packages/tapscript.md; compact patterns and candidate STATE constructor | Burn/clear patterns, public ASH constructor relation and expanded static subtree |
| linker | plans/packages/linker.md; compact/live/maturity candidate bundles | Cross-operation ASH closure and Phase-7 STATE graph, relocations and complete costs |
| transaction | plans/packages/transaction.md; compact and maturity ABIs, finalized signing states | Burn/clear construction, canonical records, all-owner signing and public residual reuse |
| vectors | plans/packages/vectors.md; inherited evidence plans and five maturity reports | Burn-event, clear-accounting, safety, continuity, public-construction and resource evidence |
| target-elements-conformance | packages/target-elements-conformance/src/protocol.rs | Change only if existing target-generic execution, signing, constructor, proof or readback interfaces cannot express a required fact |
| model | packages/model/src/ops/burn.rs and packages/model/src/ops/ash.rs | Supply independent conformance evidence; do not make compiler-facing semantics depend on the model |

The Phase-7 card's Packages header names only tapscript, linker, transaction and vectors, whereas the concept draft names seven primary packages plus conditional conformance support. The actual missing realization declarations and compiler plans make that scope difference substantive.

### Evidence that remains separate

Burn needs owner authorization and a fresh-event anchor independent of record-credit acceptance. Compact needs no burn event and no state edge. Clear needs exact destruction, accounting and STATE succession without treating the clear event as burn provenance (`rule:phase7:event-authentication`).

Every required vector at (`sec:phase7:vectors`) must acquire a declared refusing boundary, an accepted control where applicable, and a retained subject; wrong-boundary rejection, construction refusal, semantic failure and target rejection remain different outcomes.

Representation evidence must preserve explicit U, authenticated public value and unrelated-process construction; resource evidence must bind complete transactions, selected sponsor forms, residual branches and deepest paths without treating unavailable observations as zero (`sec:phase7:representation`); (`sec:phase7:resources`).

The Phase-7 exit gate has ten bullets, covering independent event anchors, over-claim behavior, compact silence, authenticated clear values, floor/RESV rules, public lifecycle, no recovery/escape, projection agreement, reports and reproducibility (`gate:phase7:exit`). An execution guide must not replace that conjunction with one successful target transaction.

### Phase-6 non-claims carried into entry

Guide 14 §23 has eighteen MET numbered conjuncts and five CONDITIONALLY MET, with the register conjunct MET; its conjunction remains NOT PASSED (`sec:phase6:gate-record`).

Conjunct 5 retains the key-path residual and unoffered wrong-control recipe. Conjunct 7 retains zero relay and block fee floors on the accepted sponsorless control. Conjunct 11 retains the checkpoint-bound edge and its environment. Conjunct 13 retains archived readback and the pinned recovery import boundary. Conjunct 14 retains R-7's composed-transaction region limit.

The 206 Safety rows comprise 78 answered and 128 outstanding: 43 first-party-required, 35 native-run-required, 15 report-layer-required and 35 typed non-answers. The independent relation-link census is fourteen every-published, 102 declared and ninety unlinked (`sec:phase6:exit-assessment`).

The amended gate claims complete disposition, not complete answers; every_required_row_is_answered remains false. No Phase-7 admission can turn a named outstanding row into observed evidence by inheriting a green phase status.

The open carriers remain `T11-064` for region scoping and positive-floor/sponsored conditions, `T11-063` for the closure-exact audit, `T4-009` for compact negative coverage and filed guide gaps, and `T11-110` for test-performance measurement (`plans/backlog.md`); (`sec:phase6:exit-assessment`).

These are entry facts with their existing carriers, not automatically Phase-7 deliverables. An execution guide must state which facts constrain a new claim and which remain outside its charter, preserving no production custody, deployment, calibration, release, independent chain observation or general theorem beyond the finite admitted corpus.

## 6. Findings for this tree

1. **The realization appendix introduction has a stale architecture schema number.** `docs/attestation/realization.md` says “It is architecture schema 17 and is authoritative on every enumerated structural fact”; `packages/architecture/src/spec.rs` sets ARCHITECTURE_SCHEMA_VERSION to 18. Confirmed: the explanatory sentence lags the typed manifest, and this study neither rewrites it nor treats schema 17 as current.

2. **The Wave-13 closing row retains an obsolete outstanding-row count.** `T11-147` in `plans/backlog.md` says “with all 135 outstanding rows by name”; (`sec:phase6:gate-record`) and (`sec:phase6:exit-assessment`) name 128. Confirmed: seven admitted refusals moved the count from 135 to 128, as the Wave-13 findings state; the stale task wording cannot override the gate record.

3. **The patch-blind version derivation test repeats one assertion.** Lines 108 and 109 of `packages/architecture/src/tests/versioning_gate_tests.rs` both read `assert_eq!(spec::tracked_realization_version("0.7.0-dev"), "0.7.0-dev");`. Confirmed: the duplicate supplies no distinct input case; the same test also checks 1.2.3 to 1.2.0, so duplication alone does not establish that patch-blind behavior is untested.

4. **The short entry headers name different prerequisite sets.** The concept draft includes “the accepted compact-ASH candidate”; the Phase-7 card names only the amended Phase-6 exit and accepted public-declassification policy and says “both entry conditions hold”. Confirmed: the inherited compact candidate exists, but the headers do not explain whether its admission prerequisite is phase-level or guide-specific; section 3 gives both coherent alternatives without editing either file.

5. **Literal semantic entry and the proposed implementation order conflict at this base.** The concept draft requires all three operations in realization scope and validated relations at entry, while `packages/realization/src/declarations/mod.rs` returns UnsupportedOperationDeclaration for burn and clear and concept-draft Wave 1 delivers their complete typed declarations. This is a new substantive admission finding: the execution artifact must explicitly distinguish authoritative model law, preparatory disposition and delivered compiler-facing semantics before it can assign a non-circular opening wave.

6. **The concept draft's clear equations are not two stored-field assignments.** Its §6.4 conditionally says successor STATE carries Y minus X and Y_L minus X; `packages/model/src/ops/ash.rs` changes y_l alone, `packages/model/src/pool.rs` derives Y, and `packages/realization/src/state.rs` stores six semantic fields without Y. The equations agree as mathematics, so an inconsistent clear law is refuted; the admission obligation is to preserve that distinction when translating the concept draft into a typed assignment census.

7. **The card's package summary understates the concept draft's implementation scope.** `plans/phases/07-burn-and-clear.md` names four packages, while the concept draft names seven primary packages and conditional conformance support; `plans/packages/realization.md` and `plans/packages/compiler.md` confirm that new declarations and plans would cross those additional contracts. This is a scope-alignment finding for admission, not authorization for package edits in this study.

8. **Repository entry is procedural, unlike the seven preceding condition lists.** The concept draft's §2.8 contains two commands, ten record fields and a dirty-tree stop requirement, not a bullet list. The audit therefore has 55 literal bullet rows plus ten determinability rows; treating all 65 as already-executed checks would misstate both the source and the evidence.

These eight findings comprise four confirmed supplied candidates and four additional clarifications or discrepancies. They do not promote the concept draft, reopen Phase 6, repair unrelated files, or discharge any native evidence obligation.

## 7. Decision page

**In one breath** — The admission question reserved by the backlog's next-gate line for the project's owner is whether to charter Guide 15 as Phase 7's execution guide unchanged, with the named entry/header/preflight/scope amendments, or only after specified prerequisites are satisfied.

**Analogy** — The choice is whether a proposed building plan is ready to become the work order: accepted foundations can be reused, but a survey of their limits is not permission to build the next floor, and an unresolved inspection cannot be certified by changing the plan's cover.

**Desk view** — Admit as is would change execution authority and reconcile the guide/index/card/backlog declarations, with proposed Wave 0 first in order, but the literal semantic-entry and preflight failures would still block an unconditional wave opening; it cannot honestly mean immediate Wave-1 implementation. Admit with named amendments would choose promotion in place or a superseding execution file, adopt one header alternative, record the twelve preflight dispositions and candidate baseline, distinguish preflight entry from Wave-1 typed-declaration delivery, fix the generation/carrier/representation obligations, align package scope and preserve all inherited non-claims; its first authorized wave could then be bounded Wave 0 revalidation and disposition, with implementation held until its explicit exit. Defer would leave the concept draft and all execution declarations unchanged pending the exact candidate/regression record, semantic-entry ordering, constructor-generation choice, burn-record carrier and complete preflight disposition; it opens no wave.

**If it sits** — The existing default remains: Phase 7 is Active, Guide 15 is a concept draft, no execution guide is chartered, and no wave opens; the accepted Phase-6 exit, public policy and compact-ASH result remain exactly at their recorded scopes (`plans/backlog.md`); (`plans/phases/07-burn-and-clear.md`).
