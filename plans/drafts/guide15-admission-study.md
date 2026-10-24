# Study: Admission of Guide 15 as the Phase-7 execution guide

**Abstract (Status and scope)** · `abst:admission:scope`

This labeled design study records admission as assessed over the clippy placement fix above 0.7.0-dev; it audits the concept against the Phase-7 card, the Phase-6 closeout and the tree, decides nothing, and retains the admission decision page. Its verification covers the documentation suite and labels tests, with no workspace or native-target pass of its own. The Guide 15 concept draft as assessed is a 4,654-line proposal with twenty-three governing rulings, eight entry procedures or gates, twelve preflight questions, fourteen suggested waves and twenty-nine numbered main sections. At the assessed base, admission is the first Phase-7 gate: the card is Active, but no execution guide is chartered. The entry audit below distinguishes 55 literal condition bullets from ten repository record fields and gives all 65 an evidence-backed standing: 50 HOLDS, two DOES NOT HOLD and thirteen UNDECIDED. Phase 6 exited on its amended gate while Guide 14 §23 remains NOT PASSED on five conditional conjuncts; 128 of 206 Safety-matrix rows remain outstanding and ninety remain unlinked to a published relation. The evidence establishes an inherited compact-ASH candidate and an explicit public-boundary policy, but does not establish complete typed burn/clear declarations or an accepted Phase-7 constructor-generation decision. The decision page preserves the choice of admission form and amendments while making the existing default explicit: no wave opens unanswered. The subsequent charter has its own authority and evidence obligations; its existence does not revise these historical assessments. (`rule:drafts:authority`); (`sec:guide15:rulings`); (`sec:guide15:entry`); (`sec:guide15:preflight`); (`sec:guide15:waves`); (`sec:phase6:exit-assessment`); (`sec:phase6:gate-record`); (`sec:phase7:change-statement`).

## 1. What admission is · `sec:admission:admission`

**Definition (Admission as execution authority)** · `def:admission:admission`

Admission is the explicit act that makes a proposed guide execution direction under the maintained phase and backlog declarations, with an identified scope, entry, exits and evidence obligations. It does not itself prove that an entry condition holds or that any wave has executed. (`rule:guides:authority`); (`rule:roadmap:laws`).

### The precedent and its limits · `sec:admission:precedent`

**Fact (Separate concept and execution artifacts)** · `fact:admission:guide14-artifacts`

Guide 14 has two distinct artifacts: `plans/guides/guide_fourteen_concept.md` remains a concept draft, while `plans/guides/guide_fourteen.md` begins “Draft execution guide; not yet executed” and explicitly supersedes the concept as execution direction. (`tab:guides:concepts`); (`tab:guides:executed`).

**Fact (Execution header and inherited entry)** · `fact:admission:guide14-header`

The execution header retains Phase, entry, affected packages, required result, exclusions, authority and trust boundary, but turns entry into the amended Phase-5 exit, the accepted STATE-constructor prototype and its own preflight gate; it also identifies the primary typed operation and human operation name. (`gate:phase5:exit`); (`sec:state-constructor:result`); (`def:guide14-exec:operation`); (`gate:guide14-exec:preflight`).

**Fact (The bounded initial review basis)** · `fact:admission:guide14-review`

The additional Initial review basis names the supplied 51-file static-review selection archived as review 10, with every review row a hypothesis until reproduced against the complete working tree. The header is retained in `plans/guides/guide_fourteen.md`. (`tab:guide14-exec:preflight`).

**Caveat (Review hypotheses do not transfer between phases)** · `cav:admission:review-transfer`

The qualification that each review row remains a hypothesis until reproduced is part of the precedent rather than a licence to copy its findings into another phase. (`tab:guide14-exec:preflight`).

**Fact (Finding status and execution status are distinct)** · `fact:admission:guide14-register`

A preflight register records a concrete hypothesis, priority and required disposition, then distinguishes finding status CONFIRMED, REFUTED or RECLASSIFIED from execution status DONE or OPEN with a named blocker (`tab:guide14-exec:preflight`).

**Observation (Suggested waves become gated execution work)** · `obs:admission:wave-precedent`

A wave plan assigns deliverables, exits and commit-sized work in dependency order; Guide 14 has implementation waves at (`sec:guide14-exec:waves`), whereas its concept draft has suggested waves at (`sec:guide14:waves`).

**Requirement (Execution authority needs dispositions and evidence boundaries)** · `req:admission:execution-content`

An execution guide therefore needs more than an amended status word: its questions become recorded dispositions or named blockers, its scope becomes executable obligations, and its verification and evidence boundaries attach to those obligations. (`rule:guides:authority`); (`gate:guide14-exec:preflight`).

### What the opening record actually reconciled · `sec:admission:opening-record`

**Fact (The three reconciled declarations)** · `fact:admission:opening-declarations`

Row (`T11-001`) reconciled three maintained declarations: the Phase-6 card's status block, the backlog header and phase-table cell, and the guides index's explanation and executed-guide registration.

**Fact (Execution evidence preceded reconciliation)** · `fact:admission:opening-evidence`

The row's evidence was already present: the execution guide was archived under (`T6-001`), Wave 0 had run and closed under (`T6-002`) and (`T6-031`), and the conceptual preflight register bound Waves 1–13 under (`T8-001`); the next wave it named was Wave 1. (`T11-001`).

**Result (Retrospective reconciliation supplies no prospective wave evidence)** · `result:admission:precedent-limit`

That was retrospective reconciliation of declarations with an established execution track. Guide 15 admission is prospective: the backlog explicitly requires admission before any wave opens, so that precedent cannot be used to pretend a Guide-15 wave has already run. The subsequent charter separately records the admitted scope without claiming that chartering executes a wave. (`T11-001`); (`rule:guide15-exec:baseline`); (`sec:phase7:change-statement`).

**Fact (Guide registration and subordinate authority)** · `fact:admission:guide-authority`

The guides index separates (`tab:guides:executed`) from (`tab:guides:concepts`); the guide rule makes a guide subordinate to the backlog, package contracts, phase card, ADRs and accepted decisions (`rule:guides:authority`).

### Two forms of admission · `sec:admission:admission-forms`

**Definition (Promotion in place)** · `def:admission:promotion`

Promotion in place means retaining the concept artifact as the execution artifact while changing its authority, entry dispositions and execution obligations; it is one possible file-level realization of admission. (`def:admission:admission`).

**Case (Promote the concept artifact)** · `case:admission:promotion`

Promotion in place would amend the concept artifact into execution direction, change its status and header, integrate dispositions and waves, and reconcile its index classification; its benefit is a stable path, while its cost is that the path still says concept and no separate unchanged concept artifact remains in the active tree. (`sec:guide15:rulings`); (`tab:guides:concepts`).

**Caveat (The precedent does not demonstrate promotion in place)** · `cav:admission:promotion-precedent`

The backlog's phrase “admission of Guide 15” is compatible with promotion in place, but the Guide-14 pair supplies no instance of that file-level method; this audit does not invent an in-place precedent. (`T11-001`); (`T12-008`).

**Definition (Superseding execution artifact)** · `def:admission:supersession`

A superseding execution artifact preserves the concept as a distinct reference and assigns execution direction to a separate guide under an explicit supersession boundary. (`tab:guides:executed`); (`tab:guides:concepts`).

**Case (Preserve the concept and charter a separate guide)** · `case:admission:supersession`

A separate execution-guide file would preserve the concept draft, explicitly supersede it as execution direction, and register the new artifact in the executed-guide table once chartered; this follows the Guide-14 shape, at the cost of another maintained text and an explicit supersession boundary. (`tab:guides:executed`).

**Requirement (Reconcile the maintained execution declarations)** · `req:admission:declaration-coherence`

Either form needs coherent changes to the guide, the guides index (`tab:guides:executed`), the Phase-7 card's status and entry wording (`phase:roadmap:burn-clear`), and the backlog's next-gate line and phase row (`T12-008`); its current-condition narrative must remain consistent with those declarations. (`rule:guides:authority`); (`rule:roadmap:change`).

**Caveat (The study records admission choices without exercising authority)** · `cav:admission:study-authority`

Those are prospective admission changes in the assessment, not authority exercised by this study. Its original integrations are its draft-index row and alphabetical draft-census entry in `plans/drafts/meson.build`; the subsequent separate execution charter is recorded independently. (`tab:drafts:index`); (`rule:drafts:authority`); (`sec:phase7:change-statement`).

## 2. The entry conditions at this base · `sec:admission:entry`

### How to read the standings · `sec:admission:reading-standings`

**Definition (HOLDS at the cited scope)** · `def:admission:holds`

HOLDS means the cited tree establishes the stated structural fact or accepted policy at the scope in Note; a historical observation remains attached to its named candidate and gate.

**Definition (DOES NOT HOLD as contradiction)** · `def:admission:does-not-hold`

DOES NOT HOLD means the tree contradicts the requirement as written.

**Definition (UNDECIDED as an outstanding disposition)** · `def:admission:undecided`

UNDECIDED means an admission disposition, exact candidate selection or fresh verification is still needed; it is not a failed experiment.

**Convention (Condition bullets and repository record fields)** · `conv:admission:entry-units`

The 55 condition bullets are transcribed from §§2.1–2.7 of the Guide 15 concept draft. Its §2.8 is a procedure, not another bullet list, so the remaining ten rows audit its record fields for determinability from the tree. Their condition and field labels now supply stable row keys without changing the assessed text or standings. (`sec:guide15:entry`); (`rule:guide15:repository-entry`).

**Fact (Repository entry observes and classifies the checkout)** · `fact:admission:repository-procedure`

Repository entry first asks for `git status --porcelain=v1 --untracked-files=all` and `git log -1 --oneline`, then requires a stop and classification of every change if the tree is dirty (`rule:guide15:repository-entry`).

**Caveat (Repository observations do not establish the full gate)** · `cav:admission:repository-observation`

Those commands establish the local observation; they do not constitute the full repository gate. (`rule:guide15:repository-entry`); (`gate:guide15:batch`).

**Convention (Determinability does not accept a future execution choice)** · `conv:admission:standing-scope`

The clean-tree record below cites the gate's own clean-tree check. A determinable record field may hold while its proposed Phase-7 use remains undecided: knowing the existing constructor does not accept a new generation, and knowing the historical compact candidate does not choose the bundle for revalidation. (`sec:phase6:gate-record`).

**Table (Entry conditions and repository record standings)** · `tab:admission:entry-standings`

| Condition or field | Bullet | Standing | Evidence | Note |
|---|---|---|---|---|
| (`cond:guide15:phase6-exited`) | Phase 6 has exited on the current tree | HOLDS | (`sec:phase6:exit-assessment`) | Exited on the amended gate; this does not pass unamended Guide 14 §23. |
| (`cond:guide15:maturity-green`) | maturity announcement remains green under its candidate constructor generation | UNDECIDED | (`sec:phase6:gate-record`) | Acceptance is recorded over the candidate; no fresh maturity run is claimed at this base. |
| (`cond:guide15:state-owner`) | canonical STATE metadata has one typed owner | HOLDS | `packages/realization/src/state.rs`; `packages/realization/src/state_codec.rs` | The typed metadata owner and canonical codec are inherited; §23 row 3 is MET (`sec:phase6:gate-record`). |
| (`cond:guide15:constructor-reconstruction`) | predecessor and successor constructor reconstruction are implemented | HOLDS | (`sec:phase6:gate-record`) | §23 rows 4 and 10 are MET for the accepted candidate's constructor reconstruction. |
| (`cond:guide15:operator-signing`) | operator signing is finalized-byte bound | HOLDS | `packages/transaction/src/state_signing.rs`; (`sec:phase6:gate-record`) | §23 row 6 is MET; operator authorization binds finalized bytes. |
| (`cond:guide15:edge-history`) | root-history evidence is edge-based | HOLDS | `packages/vectors/src/maturity_history.rs`; (`sec:phase6:gate-record`) | Edge-based evidence exists; row 11 retains the checkpoint and environment conditions. |
| (`cond:guide15:public-successor`) | public successor reconstruction is implemented | HOLDS | `packages/vectors/src/maturity_recovery.rs`; (`sec:phase6:gate-record`) | Implemented recovery consumes archived mined readback through a pinned import boundary. |
| (`cond:guide15:exact-bindings`) | target, deployment, request, response, and provenance bindings remain exact | HOLDS | `packages/vectors/src/maturity_continuity.rs`; (`sec:phase6:gate-record`) | Inherited validated evidence binds the exact subject; no broader target identity is inferred. |
| (`cond:guide15:candidate-distinction`) | candidate and final states remain distinct | HOLDS | `packages/linker/src/state_bundle.rs`; (`sec:phase6:exit-assessment`) | Accepted candidate, release-complete false, no final calibration or deployment claim. |
| (`cond:guide15:clean-tree`) | the tree is clean | HOLDS | `scripts/gate-clean-tree.sh`; (`sec:phase6:gate-record`) | The gate-of-record clean-tree procedure is available; the original admission inspection recorded a clean checkout. |
| (`cond:guide15:operation-scope`) | `burn`, `compact-ash`, and `clear` remain in architecture and realization scope under their exact existing identifiers | DOES NOT HOLD | `packages/architecture/src/ids.rs`; `packages/realization/src/declarations/mod.rs` | All three architecture identifiers exist, but typed realization dispatch lacks Burn and Clear (`fact:lineage:declaration-boundary`). |
| (`cond:guide15:validated-relations`) | burn, compact, and clear relations validate against architecture | DOES NOT HOLD | `packages/realization/src/declarations/mod.rs` | UnsupportedOperationDeclaration prevents complete burn/clear declaration validation (`fact:lineage:declaration-boundary`). |
| (`cond:guide15:clear-assignment`) | the complete STATE clear assignment is known | HOLDS | `packages/model/src/ops/ash.rs`; (`[RZ-sec:operations:clear]`) | ClearAsh changes only stored y_l; total Y falls by the same X because Y is derived. |
| (`cond:guide15:recon-tag`) | `tag-recon` is architecture-owned and exact | HOLDS | `packages/architecture/src/ids.rs`; `packages/architecture/src/spec.rs` | TagId::Recon owns tag-recon; the clear destruction declaration names it. |
| (`cond:guide15:typed-records`) | burn record and event semantics are typed | HOLDS | `packages/model/src/history.rs`; `packages/model/src/certify.rs` | BurnRecord and BurnProjection are typed model objects; compiler-facing realization is still missing. |
| (`cond:guide15:compact-silence`) | compact ASH remains event-silent | HOLDS | `packages/realization/src/declarations/compact_ash.rs`; `packages/model/src/ops/ash.rs` | Compact is ownerless-lateral and produces no burn or clear projection. |
| (`cond:guide15:sponsor-opacity`) | sponsor-value opacity remains enforced | UNDECIDED | `packages/compiler/src/sponsor_region.rs`; (`sec:phase6:exit-assessment`) | Inherited opacity holds in its scope; burn/clear enforcement is not delivered and the region refit remains open. |
| (`cond:guide15:target-independent`) | no target-specific type enters realization | HOLDS | (`pkg:realization:contract`); `packages/realization/src/lib.rs` | The contract and public semantic surface retain target independence. |
| (`cond:guide15:lifecycle-obligations`) | lifecycle obligations remain explicit | HOLDS | `packages/realization/src/lifecycle.rs`; (`sec:phase6:exit-assessment`) | Lifecycle obligations and candidate incompleteness remain explicit. |
| (`cond:guide15:public-revalidation`) | the accepted public boundary is revalidated | UNDECIDED | (`sec:public-opening:result`) | Accepted policy is known; Phase-7 revalidation is a Wave-0 deliverable, not a completed run here. |
| (`cond:guide15:ash-selection`) | the initial ASH representation is explicitly selected | HOLDS | (`sec:public-opening:result`) | The inherited initial public ownerless maintenance representation is explicit only. |
| (`cond:guide15:private-route`) | any private-receipt burn path is classified as normalized-first or separately admitted direct-private | UNDECIDED | (`sec:public-opening:result`); (`sec:guide15:entry`) | Normalized-first is the inherited route; a Phase-7 execution disposition is absent at the assessed base; the charter separately retains normalized-first (`rule:guide15-exec:disposition-3`). |
| (`cond:guide15:authenticated-opening`) | no unauthenticated opening is accepted | HOLDS | (`sec:public-opening:result`); (`pkg:target-elements:contract`) | Authenticated opening remains unsupported; no unauthenticated amount is an accepted substitute. |
| (`cond:guide15:public-ash`) | public ASH remains constructible by an unrelated process | UNDECIDED | `packages/vectors/src/resource_study.rs`; (`phase:roadmap:compact-ash`) | Public compact construction is inherited; burn-to-clear public construction is not yet evidenced. |
| (`cond:guide15:public-value`) | no report attachment substitutes for chain-public value | HOLDS | (`sec:public-opening:result`); `packages/transaction/src/view.rs` | The explicit public value is construction input; a report attachment supplies no opening authority. |
| (`cond:guide15:compact-build`) | the Phase-4 compact-ASH operation still builds | UNDECIDED | `packages/compiler/src/operation_plan.rs`; (`phase:roadmap:compact-ash`) | The operation exists and has a passing gate of record; current-base build is not rerun by this study. |
| (`cond:guide15:compact-regression`) | its compiler plan, patterns, linker, ABI, vectors, and resource study remain green | UNDECIDED | `plans/history/guide-12-completion-report.md`; `packages/vectors/src/resource_study.rs` | Historical candidate results do not certify a fresh regression run over this base. |
| (`cond:guide15:ash-compatibility`) | its ASH constructor is compatible with burn-created and clear-residual ASH, or the incompatibility is explicitly dispositioned | UNDECIDED | `packages/tapscript/src/bundle.rs`; `packages/linker/src/bundle.rs` | No compiled burn output or clear residual exists for a cross-operation constructor equality proof. |
| (`cond:guide15:compact-authorization`) | the inherited operation emits no authorization primitive | HOLDS | `packages/tapscript/src/pattern.rs`; `packages/transaction/src/abi.rs` | The inherited compact candidate is permissionless; sponsor authorization is a separate envelope. |
| (`cond:guide15:candidate-limits`) | no stale candidate limit is treated as final calibration | HOLDS | (`phase:roadmap:compact-ash`); (`pkg:linker:contract`) | Candidate bounds remain candidate; no final calibration follows from phase exit. |
| (`cond:guide15:generation-identity`) | the Phase-6 constructor generation is identified exactly | HOLDS | `packages/tapscript/src/state_constructor.rs`; (`sec:phase6:gate-record`) | The accepted maturity-only constructor is identified by its retained bundle and evidence, not a new generation name. |
| (`cond:guide15:clear-subtree`) | adding clear is recognized as a static-subtree change | HOLDS | `packages/tapscript/src/state_constructor.rs`; (`sec:guide15:entry`) | Adding clear changes the static leaf set and authenticated subtree; the concept draft states this explicitly. |
| (`cond:guide15:generation-policy`) | the Phase-7 generation policy of §1.14 is accepted or replaced by an explicit migration decision | UNDECIDED | (`sec:guide15:entry`); `plans/backlog.md` | The concept draft proposes synthetic Phase-7 origin; no execution admission accepts it at the assessed base; the charter separately assigns its disposition (`task:guide15-exec:disposition-9`). |
| (`cond:guide15:committed-leaf`) | no test attempts to spend a Phase-6 STATE with an uncommitted Phase-7 leaf | HOLDS | `packages/transaction/src/state_abi.rs`; `packages/linker/src/state_bundle.rs` | The delivered candidate has no clear leaf; its existing constructor checks do not admit an uncommitted leaf. |
| (`cond:guide15:origin-nonclaims`) | synthetic Phase-7 origin non-claims are agreed in advance | UNDECIDED | (`sec:guide15:entry`); `plans/backlog.md` | Synthetic origin and no-migration non-claims need an explicit admission disposition; the charter separately assigns it (`task:guide15-exec:disposition-9`). |
| (`cond:guide15:target-definition`) | reviewed target definition validates | HOLDS | `packages/target-elements/src/definition.rs`; (`pkg:target-elements:contract`) | Reviewed V2 definition and validator are implemented; this is a structural inherited reading. |
| (`cond:guide15:development-binding`) | development binding is welded to the exact target | HOLDS | `packages/target-elements/src/weld.rs`; `packages/target-elements/src/deployment.rs` | Development binding validates against the exact reviewed target definition. |
| (`cond:guide15:value-introspection`) | explicit `U` and explicit value introspection are reviewed | HOLDS | `packages/target-elements/src/opcode.rs`; (`pkg:target-elements:contract`) | Explicit asset/value introspection is within the reviewed target; authenticated opening is not. |
| (`cond:guide15:fixed-arithmetic`) | checked fixed-width addition, subtraction, and comparison are reviewed | HOLDS | `packages/target-elements/src/opcode.rs`; (`pkg:tapscript:contract`) | Reviewed fixed-width primitives exist; the clear-specific schedule remains a separate preflight obligation. |
| (`cond:guide15:target-conservation`) | target transaction conservation remains external evidence | HOLDS | `packages/target-elements/src/confidential.rs`; (`pkg:target-elements:contract`) | Conservation is an external target dependency, never a semantic proof of burn or clear. |
| (`cond:guide15:signing-profiles`) | selected owner and sponsor sighash profiles are reviewed | UNDECIDED | `packages/target-elements/src/authorization.rs`; `packages/transaction/src/live_signing.rs` | Inherited owner/sponsor profiles are reviewed; their exact Phase-7 selection and binding remain to be recorded. |
| (`cond:guide15:constructor-hashing`) | STATE constructor hashing and tweak requirements remain reviewed | HOLDS | `packages/target-elements/src/definition.rs`; `packages/tapscript/src/state_constructor.rs` | The Phase-6 hashing and tweak requirements remain reviewed; expanded-subtree evidence is not inherited. |
| (`cond:guide15:execution-domain`) | execution domain and leaf version are observed | HOLDS | (`sec:phase6:gate-record`); `packages/target-elements/src/definition.rs` | The accepted target observations bind the execution domain and leaf version of that candidate. |
| (`cond:guide15:relay-separation`) | consensus and relay policy remain separate | HOLDS | `packages/target-elements/src/resource.rs`; (`sec:phase6:exit-assessment`) | Zero fee floors and relay bounds remain explicit; consensus acceptance does not imply ordinary relayability. |
| (`cond:guide15:native-eligibility`) | no mock is gate-eligible | HOLDS | (`sec:guide15:entry`); `packages/vectors/src/maturity_native.rs` | The concept draft requires real target evidence; mocks and first-party fixtures cannot discharge native rows. |
| (`cond:guide15:validated-plans`) | canonical evidence plans have no unchecked constructors | HOLDS | `packages/vectors/src/plan.rs`; `packages/vectors/src/maturity_evidence.rs` | Canonical evidence plans derive through validated inputs and preserve their internal invariants. |
| (`cond:guide15:validated-coverage`) | coverage changes only from validated report wrappers | HOLDS | `packages/vectors/src/plan.rs`; `packages/vectors/src/maturity_report.rs` | Coverage discharge consumes validated evidence/report wrappers; declared requirements are not observations. |
| (`cond:guide15:observed-refusals`) | native refusals retain observed layers and exact subjects | HOLDS | `packages/vectors/src/observed_boundary.rs`; `packages/vectors/src/maturity_mutants.rs` | Observed layer and bound subject remain attached to native refusals. |
| (`cond:guide15:exact-refusals`) | wrong-boundary refusals are not counted as answers | HOLDS | `packages/vectors/src/observed_boundary.rs` | matches_boundary requires exact equality; a refusal elsewhere does not answer the row. |
| (`cond:guide15:observed-evidence`) | report-layer answerability is not counted as observed evidence | HOLDS | `packages/vectors/src/maturity_evidence.rs`; (`sec:phase6:gate-record`) | Report-layer required and observed evidence remain distinct; 15 report-layer-required rows remain outstanding. |
| (`cond:guide15:accepted-controls`) | accepted controls ground negative target observations | HOLDS | `packages/vectors/src/maturity_mutants.rs`; (`sec:phase6:gate-record`) | Admitted negative observations retain accepted controls; the reading is limited to those admitted observations. |
| (`cond:guide15:rejection-artifacts`) | target rejection carries no accepted artifact | HOLDS | `packages/target-elements-conformance/src/protocol.rs`; `packages/vectors/src/operation.rs` | Response validation distinguishes acceptance artifacts from refusal shapes. |
| (`cond:guide15:response-shape`) | response-shape validation is exact | HOLDS | `packages/target-elements-conformance/src/protocol.rs` | The strict revision-8 response contract rejects incompatible shapes and subjects. |
| (`cond:guide15:retained-transcripts`) | canonical reports retain their generic execution transcripts | HOLDS | `packages/vectors/src/maturity_report.rs`; `packages/vectors/src/maturity_continuity_report.rs` | Validated reports retain generic execution evidence rather than an unbound success flag. |
| (`cond:guide15:separate-evidence`) | safety, event, continuity, history, accounting, and resources remain separate | UNDECIDED | `packages/vectors/src/maturity_report.rs`; (`sec:guide15:entry`) | Five inherited report roles are separate; Phase-7 event/accounting report production is not delivered. |
| (`data:guide15:starting-revision`) | starting revision | HOLDS | Cargo.toml; `plans/phases/README.md` | The clippy placement fix above 0.7.0-dev. |
| (`data:guide15:phase6-revision`) | Phase-6 result revision | HOLDS | (`sec:phase6:gate-record`); (`sec:phase6:exit-assessment`) | Evidence base 0.6.305-dev; gate-record landing 0.6.306-dev; amended closeout carried at 0.7.0-dev. |
| (`data:guide15:architecture-values`) | architecture schema and semantic hashes | HOLDS | `packages/architecture/src/spec.rs`; (`reg:identities:ownership`); `packages/model/generated/architecture.json` | Schema 18; semantic and behavioural values are the registered fields in the generated architecture publication, cited by path rather than copied digests. |
| (`data:guide15:realization-version`) | realization version | HOLDS | Cargo.toml; `packages/architecture/src/spec.rs` | 0.7.0-dev, derived from the workspace version. |
| (`data:guide15:target-revision`) | target contract revision | HOLDS | `packages/target-elements/src/definition.rs` | TargetContractVersion::V2, the supported reviewed contract. |
| (`data:guide15:executor-revision`) | executor protocol revision | HOLDS | `packages/target-elements-conformance/src/protocol.rs` | NATIVE_PROTOCOL_SCHEMA = 8; earlier archived runs retain their own historical protocol. |
| (`data:guide15:compact-revision`) | compact-ASH candidate revision | UNDECIDED | `plans/history/guide-12-completion-report.md`; (`phase:roadmap:compact-ash`) | The completion record names 0.4.1-dev; the exact inherited bundle/revision for a Phase-7 rerun still needs selection. |
| (`data:guide15:state-generation`) | STATE-constructor generation | HOLDS | `packages/tapscript/src/state_constructor.rs`; (`sec:phase6:gate-record`) | The Phase-6 maturity-only candidate in the gate record; Phase-7 generation is not accepted at the assessed base; the charter separately assigns its disposition (`task:guide15-exec:disposition-9`). |
| (`data:guide15:public-policy`) | selected public-value policy | HOLDS | (`sec:public-opening:result`) | Explicit-only boundary, normalized-first with private change; public-committed opening is deferred. |
| (`data:guide15:clean-result`) | clean-tree result | HOLDS | `scripts/gate-clean-tree.sh`; (`sec:phase6:gate-record`) | The gate-of-record clean-tree check over 0.6.305-dev is MET; this is the recorded result, not a fresh workspace gate. |

### Counts and the eight whole-gate readings · `sec:admission:entry-readings`

**Result (Entry census and its two components)** · `result:admission:entry-counts`

The 65 data rows comprise 50 HOLDS, two DOES NOT HOLD and thirteen UNDECIDED. The 55 condition bullets contribute 41 HOLDS, two DOES NOT HOLD and twelve UNDECIDED; the ten repository fields contribute nine HOLDS and one UNDECIDED. (`tab:admission:entry-standings`).

**Judgment (Phase-6 entry requires candidate revalidation)** · `judg:admission:phase6-entry`

Phase-6 entry has nine HOLDS and one UNDECIDED: the phase-exit fact holds on the amended assessment, but “remains green” is not established by this documentation study's verification scope (`gate:guide15:phase6-entry`). The whole gate needs candidate-specific revalidation without reopening or enlarging the completed closeout. (`tab:admission:entry-standings`).

**Judgment (Semantic entry fails its literal conjunction)** · `judg:admission:semantic-entry`

Semantic entry has six HOLDS, two DOES NOT HOLD and one UNDECIDED (`gate:guide15:semantic-entry`). Its literal complete realization scope and validated burn/clear declarations do not exist, although model semantics do; this gate does not hold as a conjunction. (`tab:admission:entry-standings`).

**Judgment (Public-declassification entry retains three dispositions)** · `judg:admission:declassification-entry`

Public-declassification entry has three HOLDS and three UNDECIDED (`gate:guide15:declassification-entry`). The accepted explicit policy is determinate, but Phase-7 revalidation, route disposition and full public lifecycle evidence still prevent an unconditional whole-gate pass. (`tab:admission:entry-standings`).

**Judgment (Compact entry retains regression and constructor obligations)** · `judg:admission:compact-entry`

Compact-ASH entry has two HOLDS and three UNDECIDED (`gate:guide15:compact-entry`). An inherited accepted candidate is different from a current-base regression result or an equality proof across constructors that have not yet been compiled. (`tab:admission:entry-standings`).

**Judgment (STATE generation entry retains an admission choice)** · `judg:admission:generation-entry`

STATE-constructor generation entry has three HOLDS and two UNDECIDED (`gate:guide15:state-generation-entry`). The current constructor and the effect of adding a leaf are known; acceptance of synthetic origin or an explicit migration alternative remains an admission decision. (`tab:admission:entry-standings`).

**Judgment (Target entry retains signing-profile selection)** · `judg:admission:target-entry`

Target entry has nine HOLDS and one UNDECIDED (`gate:guide15:target-entry`). The reviewed V2 substrate is inherited, but the Phase-7 signing-profile selection and operation-specific use must be fixed before implementation can claim this gate complete. (`tab:admission:entry-standings`).

**Judgment (Evidence entry lacks the new report roles)** · `judg:admission:evidence-entry`

Evidence entry has nine HOLDS and one UNDECIDED (`gate:guide15:evidence-entry`). The inherited validation boundaries hold in their scopes; new event and accounting evidence cannot be assumed to exist because five maturity report types already exist. (`tab:admission:entry-standings`).

**Judgment (Repository entry retains candidate selection)** · `judg:admission:repository-entry`

Repository entry has nine determinable fields and one UNDECIDED selection. The compact candidate must be identified for the actual execution baseline; observations of cleanliness and full-gate status must retain their own scope and timing rather than becoming timeless guarantees. (`tab:admission:entry-standings`).

### The amended Phase-6 evidence boundary · `sec:admission:phase6-boundary`

**Fact (Amended bypass and wrong-code conditions retain their limits)** · `fact:admission:phase6-refusal-amendments`

The amended bypass condition covers the admitted refusals and retains the NUMS key-path residual and unoffered wrong-control recipe. The amended wrong-code-subtree condition covers vectors actually offered to a target and retains wrong-static-subtree as a carried first-party case. (`sec:phase6:exit-assessment`).

**Fact (Amended resources and coverage retain incomplete evidence)** · `fact:admission:phase6-resource-amendments`

The amended resource condition covers the accepted 53-byte variable item and observed weight 2,091, while the 86-byte whole-schedule item remains replay-only against the 80-byte relay bound. The amended coverage condition requires complete disposition, including every outstanding row, rather than complete answers. (`sec:phase6:exit-assessment`).

**Fact (Four amendments add no observations)** · `fact:admission:phase6-amendment-scope`

These are the four narrowed conditions of (`sec:phase6:exit-assessment`), with no evidence added by the amendments; the other six conditions retain their original text.

**Fact (The repository gate of record passed at its retained scope)** · `fact:admission:phase6-recorded-gate`

The full repository gate of record over 0.6.305-dev is 54 Ok, zero Fail, exit zero, with its clean-tree check MET (`sec:phase6:gate-record`).

**Bound (Inherited gate evidence establishes no broader pass)** · `bound:admission:inherited-gate`

That gate's passing run is inherited evidence at its named tip. It neither proves the unamended §23 conjunction nor supplies a new workspace or native-target pass for this study. (`sec:phase6:gate-record`); (`sec:phase6:exit-assessment`).

## 3. The header discrepancy · `sec:admission:headers`

### What the two headers say · `sec:admission:header-statements`

**Observation (Different short prerequisite sets)** · `obs:admission:header-discrepancy`

The Guide 15 concept draft's Entry line names the Phase-6 exit gate, the accepted public-declassification policy, and the accepted compact-ASH candidate. The Phase-7 card's Entry line names only the first two, and its status explicitly says “both entry conditions hold” (`sec:guide15:entry`); (`phase:roadmap:burn-clear`); the subsequent charter separately states their different entry roles (`def:guide15-exec:entry-boundaries`).

**Caveat (A header indexes obligations without discharging them)** · `cav:admission:header-scope`

The concept draft's detailed entry conditions are stronger than either short header: they additionally require typed scope, constructor policy, target readiness, evidence boundaries and repository records. A header is an index to these obligations, not their discharge. (`sec:guide15:entry`).

### Is compact ASH accepted? · `sec:admission:compact-acceptance`

**Fact (The accepted Phase-4 candidate result)** · `fact:admission:compact-result`

The Phase-4 card is Exited and records twelve accepted and matched live submissions, coverage 100 of 211, a section-29 audit at 94 of 105, and a 47-of-47 local gate with byte-identical document reproduction (`phase:roadmap:compact-ash`).

**Fact (Review hypotheses and accepted completion differ)** · `fact:admission:compact-review`

The Guide-12 register is a set of sixteen review hypotheses, not an automatic certificate of acceptance (`tab:guide12-exec:preflight`). Its completion record reports fifteen CONFIRMED and DONE and R08 RECLASSIFIED and DONE, and the Phase-4 card supplies the phase-level accepted result (`plans/history/guide-12-completion-report.md`). (`rem:guide12-completion:counting`).

**Bound (Compact acceptance retains its finite scope)** · `bound:admission:compact-scope`

Thus an accepted compact-ASH candidate exists at its declared scope. That is not final calibration, a complete negative matrix, a production deployment, or proof that burn-created and clear-residual ASH share its exact constructor; (`pkg:vectors:contract`) still records one discharged compact negative row and 71 outstanding. (`gate:vectors:compact-ash`).

**Caveat (A completion template is not a gate verdict)** · `cav:admission:completion-scope`

The historical completion record is over 0.4.1-dev and explicitly distinguishes its audit from deferred repository-gate verdicts; the card's accepted exit cannot be replaced by treating every completion-template field as independently passed. (`rem:guide12-completion:counting`).

### Alternative A: align the card upward · `sec:admission:alternative-a`

**Case (Alternative A aligns the phase header upward)** · `case:admission:alternative-a`

The card could say: “The three inherited entry results hold: Phase 6 exited on its amended gate, the explicit public-declassification policy is accepted, and the Phase-4 compact-ASH candidate is accepted at its recorded scope; detailed Guide-15 entry and preflight obligations remain to be dispositioned before implementation.” (`phase:roadmap:compact-ash`); (`sec:phase6:exit-assessment`).

**Specification (Alternative A entry wording)** · `spec:admission:alternative-a-wording`

Its Entry line would add the accepted compact-ASH candidate. The concept draft's Entry line would retain all three and explicitly qualify Phase 6 by the amended assessment and compact ASH by its Phase-4 candidate scope. (`sec:guide15:entry`).

**Argument (Three named results need not imply complete entry)** · `arg:admission:alternative-a-argument`

This alternative makes the phase header expose all three inherited results, while avoiding the false implication that every detailed entry bullet is already true. It does not convert the historical compact acceptance into a Phase-7 compatibility proof. (`tab:admission:entry-standings`).

### Alternative B: keep phase entry separate from guide entry · `sec:admission:alternative-b`

**Case (Alternative B distinguishes phase and guide entry)** · `case:admission:alternative-b`

The card could retain its two Entry conditions and say: “Both phase-entry conditions hold on the amended Phase-6 exit and accepted public policy; admission of Guide 15 also requires disposition of the inherited compact-ASH candidate and the concept draft's detailed entry and preflight requirements.” (`sec:guide15:entry`).

**Specification (Alternative B guide wording)** · `spec:admission:alternative-b-wording`

The concept draft's Entry line could say: “The Phase-7 card's two entry results, plus the accepted Phase-4 compact-ASH candidate at its recorded scope, subject to detailed entry and preflight disposition.” (`sec:guide15:entry`).

**Argument (An active phase need not have a chartered guide)** · `arg:admission:alternative-b-argument`

This alternative treats the third result as a guide-specific prerequisite rather than a missing phase-entry condition. The consequence is an explicit distinction between opening the phase and chartering its execution guide, which the current “Active” status already permits. (`def:guide15-exec:entry-boundaries`).

**Caveat (Header alternatives preserve inherited results)** · `cav:admission:header-invariants`

Neither alternative changes whether the Phase-4 candidate exists or whether Phase 6 exited. The assessment leaves selection of the coherent declaration set and its execution consequences to admission; the subsequent charter adopts the phase-versus-guide distinction (`def:guide15-exec:entry-boundaries`). (`sec:phase6:exit-assessment`); (`phase:roadmap:compact-ash`).

## 4. The preflight questions · `sec:admission:preflight`

### 3.1 Exact semantic field ownership · `sec:admission:semantic-ownership`

**Question (Exact semantic field ownership)** · `q:admission:semantic-ownership`

Which typed sources own each operation, field, record, event and certificate, and which complete compiler-facing census remains to be delivered? (`q:guide15:semantic-census`).

**Observation (Assessment of Exact semantic field ownership)** · `obs:admission:semantic-ownership-assessment`

Answerable in part: architecture owns the operation, quantity, tag and projection identifiers in `packages/architecture/src/ids.rs`; `BurnReceipts` owns requested fresh ASH, receipt change and records in `packages/model/src/ops/burn.rs`; `CompactAsh` and `ClearAsh` compute aggregate, decrement and residual in `packages/model/src/ops/ash.rs`; the kernel/certifier and `packages/model/src/history.rs` own derived events and certificates. The complete compiler-facing census is not delivered because `packages/realization/src/declarations/mod.rs` lacks burn and clear; the admission decision must assign its completion before those plans consume it (`q:guide15:semantic-census`). The subsequent charter separately assigns model-law entry and later declarations (`def:guide15-exec:semantic-entry-split`).

### 3.2 ASH representation · `sec:admission:ash-representation`

**Question (ASH representation)** · `q:admission:ash-representation`

Which initial ASH representation is accepted, and what distinct evidence would admit a public-committed alternative? (`q:guide15:ash-representation`).

**Fact (Assessment of ASH representation)** · `fact:admission:ash-representation-assessment`

Answerable for the inherited initial mode: explicit value is selected at (`sec:public-opening:result`), including public ownerless maintenance, and retained by Guide-12's representation rule (`rule:guide12-exec:representation`). Public-committed ASH would require a new accepted result addressing generator derivation, encoding-domain mismatch and unbound supplied parity; a concept-draft preference cannot make that result exist (`q:guide15:ash-representation`).

### 3.3 Private burn route · `sec:admission:private-burn`

**Question (Private burn route)** · `q:admission:private-burn`

Does the admitted route normalize private receipts first, or is a direct-private route separately supported? (`q:guide15:private-burn`).

**Fact (Assessment of Private burn route)** · `fact:admission:private-burn-assessment`

Answerable as the existing default: normalized-first with private change is the accepted route, while direct private burn remains deferred; the public-declassification result explicitly does not make full private consumption without residual blinding available. The admission decision must record that scope or require the separate direct-private evidence before claiming it; no positive direct-private class follows from confidential conservation alone (`q:guide15:private-burn`); (`sec:public-opening:result`). The subsequent charter retains that normalized-first route (`rule:guide15-exec:disposition-3`).

### 3.4 ASH constructor continuity · `sec:admission:ash-continuity`

**Question (ASH constructor continuity)** · `q:admission:ash-continuity`

Do fresh, compacted and residual ASH share one exact constructor, or must class transitions be admitted explicitly? (`q:guide15:ash-constructor`).

**Condition (Assessment of ASH constructor continuity)** · `cond:admission:ash-continuity-assessment`

Needs a disposition: the inherited constructor exists in `packages/tapscript/src/bundle.rs` and `packages/linker/src/bundle.rs`, but there are no compiled burn and clear counterparts to compare. Admission must require exact equality under one target, deployment, representation and bundle, or explicitly charter class transitions; shared semantic Meta::Ash is not byte-level constructor equality (`q:guide15:ash-constructor`). The subsequent charter separately assigns selection and proof (`task:guide15-exec:disposition-4`), (`obl:guide15-exec:disposition-4-proof`).

### 3.5 Burn record carrier · `sec:admission:record-carrier`

**Question (Burn record carrier)** · `q:admission:record-carrier`

Which chain-public carrier, signature coverage and recovery boundary bind canonical burn records to fresh ASH? (`q:guide15:burn-record-carrier`).

**Observation (Assessment of Burn record carrier)** · `obs:admission:record-carrier-assessment`

Needs a decision: model records and canonical ordering exist in `packages/model/src/ops/burn.rs`, `packages/model/src/kernel.rs` and `packages/model/src/certify.rs`, but no target burn ABI selects the carrier. Admission must choose the chain-public encoding and recovery boundary, including its signature coverage, fresh-ASH association and resistance to copied-payload provenance; the concept draft's list of candidate carriers is not a selection (`q:guide15:burn-record-carrier`). The subsequent charter separately assigns provisional selection and its reopening condition (`task:guide15-exec:disposition-5`), (`cond:guide15-exec:carrier-reopening`).

### 3.6 Burn-record cardinality and value rule · `sec:admission:record-rule`

**Question (Burn-record cardinality and value rule)** · `q:admission:record-rule`

What are the authoritative count, ordinal, amount and aggregate-credit rules for burn records? (`q:guide15:burn-record-rule`).

**Fact (Assessment of Burn-record cardinality and value rule)** · `fact:admission:record-rule-assessment`

Answerable semantically: zero records are allowed; `BurnRecordMax` bounds count; each amount is positive; indices start at zero and are contiguous in output order; duplicate or skipped indices reject; arbitrary address partitioning is permitted rather than an invented one-record-per-address restriction. `packages/model/src/ledger.rs` keeps over-claim a credit verdict, with aggregate comparison in u128: records exceeding fresh ASH receive no credit without erasing the authenticated burn; exact target encoding still needs implementation (`q:guide15:burn-record-rule`); (`[RZ-sec:operations:burn]`), (`[RZ-sec:realization:ledger]`).

### 3.7 Clear STATE assignment · `sec:admission:clear-assignment`

**Question (Clear STATE assignment)** · `q:admission:clear-assignment`

Which stored STATE fields change at clear, and which resulting quantities are derived? (`q:guide15:clear-assignment`).

**Fact (Assessment of Clear STATE assignment)** · `fact:admission:clear-assignment-assessment`

Answerable as the authoritative law: `ClearAsh` changes only y_l to y_l minus X and copies the other five stored fields in `packages/model/src/ops/ash.rs`; `PoolState::y` derives Y as y_l plus y_t in `packages/model/src/pool.rs`. Therefore Y also decreases by X, but it is not an additional stored assignment; the six-field `StateMetadata` in `packages/realization/src/state.rs` has no independent Y field. The missing typed realization transition must implement that law rather than materialize a seventh semantic field (`q:guide15:clear-assignment`). The stored-versus-derived law is also stated in (`[RZ-sec:operations:clear]`).

### 3.8 Clear arithmetic lowering · `sec:admission:clear-arithmetic`

**Question (Clear arithmetic lowering)** · `q:admission:clear-arithmetic`

Which exact domains, bounds and branch schedule realize the authoritative clear arithmetic on the reviewed target? (`q:guide15:clear-arithmetic`).

**Observation (Assessment of Clear arithmetic lowering)** · `obs:admission:clear-arithmetic-assessment`

Answerable at the operation level: the authoritative clamp uses bounded summation, subtraction and minimum comparisons, with no multiplication or division; model checked arithmetic rejects overflow and zero progress in `packages/model/src/ops/ash.rs`. The reviewed target primitives exist in `packages/target-elements/src/opcode.rs`, but admission must require the exact operand encoding, signedness, bounds, branch schedule and consumed comparisons before calling a target lowering proven; existence of primitives alone does not discharge the schedule (`q:guide15:clear-arithmetic`).

### 3.9 STATE generation transition · `sec:admission:state-generation`

**Question (STATE generation transition)** · `q:admission:state-generation`

Does Phase 7 admit synthetic origin under an expanded subtree, or require an explicit migration before clear? (`q:guide15:state-generation`).

**Judgment (Assessment of STATE generation transition)** · `judg:admission:state-generation-assessment`

Needs an admission decision: the Guide 15 concept draft proposes a new synthetic Phase-7 origin, maturity and clear in the expanded static subtree, maturity revalidation, no cross-generation spend and no migration claim. `packages/tapscript/src/state_constructor.rs` and the Phase-6 constructor evidence make the subtree identity material; continuity from the accepted Phase-6 STATE would instead require an explicit migration operation and evidence plan before clear (`q:guide15:state-generation`). The subsequent charter separately assigns the generation disposition (`task:guide15-exec:disposition-9`).

### 3.10 Event projection independence · `sec:admission:event-independence`

**Question (Event projection independence)** · `q:admission:event-independence`

Which separate validated roles establish operation acceptance, provenance, credit, silence, accounting and succession? (`q:guide15:event-independence`).

**Observation (Assessment of Event projection independence)** · `obs:admission:event-independence-assessment`

Answerable at the model boundary: `packages/model/src/certify.rs`, `packages/model/src/history.rs` and `packages/model/src/ledger.rs` distinguish burn provenance, record acceptance, compact silence, clear projection and accounting, while the inherited maturity reports already separate safety, continuity, history, recovery and resources. A complete Phase-7 report implementation is absent, so admission must require separate validated roles and exact bindings rather than interpret a generic success as all seven answers (`q:guide15:event-independence`).

### 3.11 Permissionless public construction · `sec:admission:public-construction`

**Question (Permissionless public construction)** · `q:admission:public-construction`

Can unrelated processes construct clear and reuse residual ASH from public inputs alone? (`q:guide15:permissionless`).

**Condition (Assessment of Permissionless public construction)** · `cond:admission:public-construction-assessment`

Answerable for inherited compact ASH at its recorded candidate scope, using public construction inputs and an optional sponsor-local capability (`phase:roadmap:compact-ash`); (`packages/transaction/src/view.rs`). Clear has no delivered target ABI or independent construction demonstration; admission must make fresh-process clear and residual-ASH reuse explicit evidence obligations, with any creator-only opening or metadata dependency a blocker (`q:guide15:permissionless`).

### 3.12 Attestation query boundary · `sec:admission:query-boundary`

**Question (Attestation query boundary)** · `q:admission:query-boundary`

Is record and event consumption internal, or does an identified external consumer activate interchange adjudication? (`q:guide15:attestation-query`).

**Observation (Assessment of Attestation query boundary)** · `obs:admission:query-boundary-assessment`

The tree contains an internal attestation consumer in `packages/model/src/ledger.rs` and its indexer conformance tests, but this audit finds no Phase-7 external burn-report interchange consumer in the delivered pipeline. The default is internal event and record validation without a new namespace or report digest; a named external consumer would require ADR-022 adjudication before its interface is implemented (`q:guide15:attestation-query`); (`[ADR022-rule:interchange:no-implementation]`).

### The preflight conjunction · `sec:admission:preflight-conjunction`

**Gate (The preflight conjunction is not discharged)** · `gate:admission:preflight`

The preflight gate does not hold at this base: the twelve questions do not all have execution dispositions, the burn/clear realization census is incomplete, and the carrier, constructor and generation obligations remain unresolved (`gate:guide15:preflight`). (`tab:admission:entry-standings`).

**Caveat (Admission does not imply immediate implementation)** · `cav:admission:preflight-order`

That result bars a claim that implementation can begin immediately under the concept draft's literal preflight wording. It does not decide whether admission may charter a bounded Wave 0 to complete dispositions; the execution guide must make the ordering explicit. The subsequent charter separately fixes preparatory scope and retains every observable Wave-0 obligation until performed. (`spec:guide15-exec:wave0-scope`); (`rule:guide15-exec:observable-retention`).

## 5. What an execution guide adds · `sec:admission:execution-obligations`

### A register that controls entry · `sec:admission:entry-register`

**Requirement (An entry register records both findings and execution)** · `req:admission:register-content`

The execution artifact needs a preflight register with each of the twelve questions, the 65 audited entries, exact evidence, finding status, execution status, blocker and required exit; Guide 14's distinction between hypothesis and reproduced finding is the useful precedent. (`tab:guide14-exec:preflight`); (`tab:guide15-exec:preflight`).

**Criterion (Evidence classes cannot certify themselves)** · `crit:admission:register-evidence`

The register must explicitly distinguish inherited policy, structural implementation, observed candidate evidence and new Phase-7 choices. A planned positive case or report field cannot be used to mark its own entry requirement as already satisfied. The subsequent charter carries the audited entries with separate execution obligations (`tab:guide15-exec:preflight`). (`def:guide15-exec:evidence-classes`).

**Obligation (Dispose the semantic-entry circularity)** · `obl:admission:semantic-order`

The semantic-entry contradiction needs a disposition: either an authorized preparatory tranche creates the missing declarations before implementation entry, or the execution guide explicitly separates known model law from the typed declarations Wave 1 delivers; leaving both requirements unchanged creates a circular gate. The subsequent charter separately assigns model law to Wave 0 and typed declarations to Wave 2 (`def:guide15-exec:semantic-entry-split`). (`cond:guide15:operation-scope`); (`cond:guide15:validated-relations`); (`task:guide15:wave1`).

### A review basis grounded in this tree · `sec:admission:review-basis`

**Specification (Review authority and affected contracts)** · `spec:admission:review-content`

The initial review basis should name the Guide 15 concept draft, the Phase-7 card, the amended Phase-6 assessment and gate record, Guide-12/Guide-11 results, typed architecture, realization declarations, model transitions, target contract and the affected package contracts. (`sec:guide15:rulings`); (`phase:roadmap:burn-clear`); (`sec:phase6:exit-assessment`); (`sec:phase6:gate-record`); (`setup:guide15-exec:review-basis`).

**Caveat (An inherited review does not audit a new operation)** · `cav:admission:review-scope`

Guide 14's review 10 and 51-file selection are historical evidence for its own register; they are not a new review of burn and clear. The four supplied candidate discrepancies and the additional findings below need explicit dispositions without making editorial repairs prerequisites to unrelated semantic proofs. (`tab:guide14-exec:preflight`).

**Fact (Twenty-three governing rulings)** · `fact:admission:ruling-census`

The governing-ruling census runs from (`rule:guide15:consume-pipeline`) through (`rule:guide15:exactness`): twenty-three concept-draft rulings establish proposed constraints, including operation separation, public ASH, exact authorization, constructor generations, independent event anchors, failure layers and candidate-only status. (`sec:guide15:rulings`).

### The concept draft's complete section inventory · `sec:admission:concept-inventory`

**Table (The complete concept section inventory)** · `tab:admission:concept-inventory`

| Sections | Proposed subject | Source |
|---|---|---|
| 1–9 | Sections 1–3 cover governing rulings, entry conditions and preflight questions; §§4–9 cover the semantic burn contract, public ASH and compact maintenance, semantic clear contract, Phase-7 STATE constructor generation, compiler plans, and target assessment and proof patterns. | (`sec:guide15:rulings`); (`sec:guide15:entry`); (`sec:guide15:preflight`); (`sec:guide15:burn`); (`sec:guide15:ash`); (`sec:guide15:clear`); (`sec:guide15:state-constructor`); (`sec:guide15:compiler`); (`sec:guide15:target-patterns`) |
| 10–15 | Sections 10–15 cover linking and constructor closure, candidate transaction and witness ABIs, burn signing and authorization, evidence architecture, semantic fixtures and projections, and the required vector matrix. | (`sec:guide15:linking`); (`sec:guide15:abi`); (`sec:guide15:signing`); (`sec:guide15:evidence`); (`sec:guide15:fixtures`); (`sec:guide15:vectors`) |
| 16–21 | Sections 16–21 cover event and attestation integrity, resource evidence, relation-indexed coverage, assurance boundaries, suggested implementation waves, and focused verification. | (`sec:guide15:events`); (`sec:guide15:resources`); (`sec:guide15:coverage`); (`sec:guide15:assurance`); (`sec:guide15:waves`); (`sec:guide15:verification`) |
| 22–29 | Sections 22–29 cover the full batch gate, acceptance criteria, rejection criteria, identity/schema/security/interchange/dependency impact, final Phase-7 result matrix, Guide-15 exit checklist, completion report template, and handoff after Guide 15. | (`gate:guide15:batch`); (`sec:guide15:acceptance`); (`sec:guide15:rejection`); (`sec:guide15:impact`); (`tab:guide15:result`); (`gate:guide15:exit`); (`sec:guide15:report-template`); (`sec:guide15:handoff`) |

**Bound (The concept inventory is proposed scope)** · `bound:admission:inventory-scope`

This inventory is the scope of the concept as assessed at 4,654 lines, not a statement that any of those sections has been executed. (`sec:guide15:rulings`); (`sec:guide15:handoff`).

### Wave and bite shapes · `sec:admission:wave-shapes`

**Requirement (Suggested waves need executable exits)** · `req:admission:wave-exits`

The concept draft proposes fourteen waves at (`sec:guide15:waves`); an execution guide must turn each into a bounded dependency, deliverable, verification and exit rather than simply retain its suggested commit.

**Table (Proposed wave and bite shapes)** · `tab:admission:wave-shapes`

| Wave | Proposed execution bite shape | Evidence boundary before advancement |
|---|---|---|
| 0 (`task:guide15:wave0`) | Revalidate Phase 6 and close the preflight register | Exact candidate baseline, policy and generation dispositions; no inherited non-claim silently discharged |
| 1 (`task:guide15:wave1`) | Typed burn and clear declarations, event/accounting projections and architecture weld | Independent model conformance and exact semantic census before lowering |
| 2 (`task:guide15:wave2`) | Validated burn/clear plans and compact regression | Relation, carrier, layout, lifecycle and coverage censuses plus corruption oracles |
| 3 (`task:guide15:wave3`) | Target and representation closure | Reviewed arithmetic, explicit value and signing requirements; unsupported alternatives stated |
| 4 (`task:guide15:wave4`) | Public ASH constructor integration | Exact fresh/aggregate/residual constructor compatibility or explicit transitions |
| 5 (`task:guide15:wave5`) | Burn patterns | Owner authorization, live-only inputs, one fresh ASH, records and isolated sponsor flow |
| 6 (`task:guide15:wave6`) | Expanded STATE constructor and clear patterns | Exact decrement, floor, residual, destruction and unchanged fields; maturity revalidated |
| 7 (`task:guide15:wave7`) | Linker extension | Deterministic symbols, tree/control recipes and complete carrier closure |
| 8 (`task:guide15:wave8`) | Candidate ABIs and burn signing | Finalized-byte binding, canonical records, permissionless clear and public construction |
| 9 (`task:guide15:wave9`) | Burn safety and event evidence | Independent event-type and value anchors, copied records and over-claim behavior |
| 10 (`task:guide15:wave10`) | ASH maintenance evidence | Compact silence, constructor continuity and unrelated-process reuse |
| 11 (`task:guide15:wave11`) | Clear safety and STATE evidence | Exact accounting, one STATE edge, no RESV edge and residual closure |
| 12 (`task:guide15:wave12`) | Resource study | Separate prediction/observation, exact subjects and absent dimensions, no final calibration |
| 13 (`task:guide15:wave13`) | Phase-7 gate and handoff | Card's ten exit bullets, complete result matrix, explicit remaining non-claims |

**Question (Whether preparatory Wave 0 can open)** · `q:admission:wave0-authority`

The execution plan must settle whether Wave 0 is authorized preflight work or is itself blocked by every literal entry bullet. The backlog at the assessed base authorizes neither interpretation before admission; the subsequent charter assigns preparatory scope separately (`spec:guide15-exec:wave0-scope`). (`gate:guide15:preflight`); (`T12-008`).

**Specification (Bites follow ownership and proof boundaries)** · `spec:admission:bite-shapes`

Bites should follow ownership boundaries: a semantic declaration and its conformance proof, a compiler plan and its validator/oracles, a target pattern and its exact schedule, a linked bundle and its closure checks, an ABI/signing boundary and its mutants, then bound report producers and their evidence. (`rule:guide15:consume-pipeline`).

**Obligation (An inherited-interface change needs its own warrant)** · `obl:admission:interface-change`

A bite that changes an inherited interface must identify the missing fact, smallest sufficient generalization, identity/schema impact and compact/live/maturity regressions; the concept draft already requires this at (`rule:guide15:consume-pipeline`).

### Package contracts that would move · `sec:admission:package-contracts`

**Table (Package contracts and required additions)** · `tab:admission:package-contracts`

| Package | Existing contract and delivered boundary | Required Phase-7 addition if chartered |
|---|---|---|
| architecture | `packages/architecture/src/ids.rs` and `packages/architecture/src/spec.rs` already name burn, compact, clear, tags and projections | Correct only an established omission; do not invent identities for absent consumers |
| realization | (`pkg:realization:contract`); three typed declarations are delivered (`fact:lineage:declaration-boundary`) | Burn/clear declarations, complete assignments, relations, event/accounting and lifecycle projections |
| compiler | (`pkg:compiler:contract`); validated target-operation plans and maturity requirements | Burn/clear plans, public availability, exact carriers and operation-specific validators |
| target-elements | (`pkg:target-elements:contract`); reviewed V2 contract and capability census | Exact operation assessments and any justified missing capability, without promoting unsupported opening |
| tapscript | (`pkg:tapscript:contract`); compact patterns and candidate STATE constructor | Burn/clear patterns, public ASH constructor relation and expanded static subtree |
| linker | (`pkg:linker:contract`); compact/live/maturity candidate bundles | Cross-operation ASH closure and Phase-7 STATE graph, relocations and complete costs |
| transaction | (`pkg:transaction:contract`); compact and maturity ABIs, finalized signing states | Burn/clear construction, canonical records, all-owner signing and public residual reuse |
| vectors | (`pkg:vectors:contract`); inherited evidence plans and five maturity reports | Burn-event, clear-accounting, safety, continuity, public-construction and resource evidence |
| target-elements-conformance | `packages/target-elements-conformance/src/protocol.rs` | Change only if existing target-generic execution, signing, constructor, proof or readback interfaces cannot express a required fact |
| model | `packages/model/src/ops/burn.rs` and `packages/model/src/ops/ash.rs` | Supply independent conformance evidence; do not make compiler-facing semantics depend on the model |

**Observation (The package header understates required contracts)** · `obs:admission:package-scope`

The Phase-7 card's Packages header names only tapscript, linker, transaction and vectors, whereas the concept draft names seven primary packages plus conditional conformance support. The actual missing realization declarations and compiler plans make that scope difference substantive. The subsequent charter separately aligns the affected contracts (`setup:guide15-exec:review-basis`). (`pkg:realization:contract`); (`pkg:compiler:contract`); (`sec:phase7:change-statement`).

### Evidence that remains separate · `sec:admission:separate-evidence`

**Rule (Burn compact and clear retain distinct evidence)** · `rule:admission:operation-evidence`

Burn needs owner authorization and a fresh-event anchor independent of record-credit acceptance. Compact needs no burn event and no state edge. Clear needs exact destruction, accounting and STATE succession without treating the clear event as burn provenance (`rule:phase7:event-authentication`).

**Obligation (Vectors retain boundary control and subject)** · `obl:admission:vector-evidence`

Every required vector at (`sec:phase7:vectors`) must acquire a declared refusing boundary, an accepted control where applicable, and a retained subject; wrong-boundary rejection, construction refusal, semantic failure and target rejection remain different outcomes.

**Specification (Representation and resources retain exact subjects)** · `spec:admission:representation-resources`

Representation evidence must preserve explicit U, authenticated public value and unrelated-process construction; resource evidence must bind complete transactions, selected sponsor forms, residual branches and deepest paths without treating unavailable observations as zero (`sec:phase7:representation`); (`sec:phase7:resources`).

**Gate (All ten Phase-7 exit conjuncts remain required)** · `gate:admission:phase7-exit`

The Phase-7 exit gate has ten bullets, covering independent event anchors, over-claim behavior, compact silence, authenticated clear values, floor/RESV rules, public lifecycle, no recovery/escape, projection agreement, reports and reproducibility (`gate:phase7:exit`). An execution guide must not replace that conjunction with one successful target transaction.

### Phase-6 non-claims carried into entry · `sec:admission:inherited-nonclaims`

**Fact (The unamended conjunction remains not passed)** · `fact:admission:phase6-conjunction`

Guide 14 §23 has eighteen MET numbered conjuncts and five CONDITIONALLY MET, with the register conjunct MET; its conjunction remains NOT PASSED (`sec:phase6:gate-record`).

**Fact (The five conditional conjuncts retain their limitations)** · `fact:admission:phase6-conjunct-limits`

Conjunct 5 retains the key-path residual and unoffered wrong-control recipe. Conjunct 7 retains zero relay and block fee floors on the accepted sponsorless control. Conjunct 11 retains the checkpoint-bound edge and its environment. Conjunct 13 retains archived readback and the pinned recovery import boundary. Conjunct 14 retains R-7's composed-transaction region limit. (`sec:phase6:gate-record`).

**Result (Safety and relation-link censuses remain distinct)** · `result:admission:safety-census`

The 206 Safety rows comprise 78 answered and 128 outstanding: 43 first-party-required, 35 native-run-required, 15 report-layer-required and 35 typed non-answers. The independent relation-link census is fourteen every-published, 102 declared and ninety unlinked (`sec:phase6:exit-assessment`).

**Bound (Disposition is not complete answers)** · `bound:admission:disposition-limit`

The amended gate claims complete disposition, not complete answers; every_required_row_is_answered remains false. No Phase-7 admission can turn a named outstanding row into observed evidence by inheriting a green phase status. (`sec:phase6:exit-assessment`).

**Fact (Inherited work retains its existing carriers)** · `fact:admission:inherited-carriers`

The open carriers remain (`T11-064`) for region scoping and positive-floor/sponsored conditions, (`T11-063`) for the closure-exact audit, (`T4-009`) for compact negative coverage and filed guide gaps, and (`T11-110`) for test-performance measurement; (`sec:phase6:exit-assessment`).

**Caveat (Inherited entry facts are not automatic new deliverables)** · `cav:admission:nonclaim-scope`

These are entry facts with their existing carriers, not automatically Phase-7 deliverables. An execution guide must state which facts constrain a new claim and which remain outside its charter, preserving no production custody, deployment, calibration, release, independent chain observation or general theorem beyond the finite admitted corpus. (`sec:phase6:exit-assessment`); (`rule:guide15:candidate-only`).

## 6. Findings for this tree · `sec:admission:findings`

**Fact (The realization introduction retains a stale schema number)** · `fact:admission:finding-schema`

The realization appendix (`[RZ-app:realization:architecture]`) says “It is architecture schema 17 and is authoritative on every enumerated structural fact”; `packages/architecture/src/spec.rs` sets ARCHITECTURE_SCHEMA_VERSION to 18. Confirmed: the explanatory sentence lags the typed manifest, and this study neither rewrites it nor treats schema 17 as current. (`reg:identities:ownership`).

**Fact (The Wave-13 closing row retains the earlier outstanding count)** · `fact:admission:finding-count`

(`T11-147`) says “with all 135 outstanding rows by name”; (`sec:phase6:gate-record`) and (`sec:phase6:exit-assessment`) name 128. Confirmed: seven admitted refusals moved the count from 135 to 128, as the Wave-13 findings state; the stale task wording cannot override the gate record. (`T11-147`); (`sec:phase6:wave13-findings`).

**Observation (The version derivation test repeats one assertion)** · `obs:admission:finding-test`

Lines 108 and 109 of `packages/architecture/src/tests/versioning_gate_tests.rs` both read `assert_eq!(spec::tracked_realization_version("0.7.0-dev"), "0.7.0-dev");`. Confirmed: the duplicate supplies no distinct input case; the same test also checks 1.2.3 to 1.2.0, so duplication alone does not establish that patch-blind behavior is untested.

**Observation (The short entry headers name different prerequisite sets)** · `obs:admission:finding-headers`

The concept draft includes “the accepted compact-ASH candidate”; the Phase-7 card names only the amended Phase-6 exit and accepted public-declassification policy and says “both entry conditions hold”. Confirmed: the inherited compact candidate exists, but the headers do not explain whether its admission prerequisite is phase-level or guide-specific; (`sec:admission:headers`) gives both coherent alternatives; the subsequent charter separately adopts the phase-versus-guide distinction (`def:guide15-exec:entry-boundaries`). (`sec:admission:headers`).

**Observation (Literal semantic entry conflicts with the proposed delivery order)** · `obs:admission:finding-entry`

The concept draft requires all three operations in realization scope and validated relations at entry, while `packages/realization/src/declarations/mod.rs` returns UnsupportedOperationDeclaration for burn and clear and concept-draft Wave 1 delivers their complete typed declarations. This is a new substantive admission finding: the execution artifact must explicitly distinguish authoritative model law, preparatory disposition and delivered compiler-facing semantics before it can assign a non-circular opening wave. (`cond:guide15:operation-scope`); (`cond:guide15:validated-relations`); (`fact:lineage:declaration-boundary`); (`def:guide15-exec:semantic-entry-split`).

**Fact (The clear equations are not two stored-field assignments)** · `fact:admission:finding-clear`

The concept's clear-state rule (`rule:guide15:clear-state`) conditionally says successor STATE carries Y minus X and Y_L minus X; `packages/model/src/ops/ash.rs` changes y_l alone, `packages/model/src/pool.rs` derives Y, and `packages/realization/src/state.rs` stores six semantic fields without Y. The equations agree as mathematics, so an inconsistent clear law is refuted; the admission obligation is to preserve that distinction when translating the concept draft into a typed assignment census. (`rule:guide15:clear-state`); (`[RZ-sec:operations:clear]`).

**Observation (The package summary understates implementation scope)** · `obs:admission:finding-packages`

(`phase:roadmap:burn-clear`) names four packages, while the concept draft names seven primary packages and conditional conformance support; (`pkg:realization:contract`) and (`pkg:compiler:contract`) confirm that new declarations and plans would cross those additional contracts. This is a scope-alignment finding for admission, not authorization for package edits in this study. The subsequent charter names its affected contracts separately (`setup:guide15-exec:review-basis`). (`sec:phase7:change-statement`).

**Fact (Repository entry records fields rather than another condition list)** · `fact:admission:finding-repository`

The concept draft's §2.8 contains two commands, ten record fields and a dirty-tree stop requirement, not a bullet list. The audit therefore has 55 literal bullet rows plus ten determinability rows; treating all 65 as already-executed checks would misstate both the source and the evidence. (`rule:guide15:repository-entry`); (`conv:admission:entry-units`).

**Result (The findings delimit admission without exercising it)** · `result:admission:findings-scope`

These eight findings comprise four confirmed supplied candidates and four additional clarifications or discrepancies. They do not promote the concept draft, reopen Phase 6, repair unrelated files, or discharge any native evidence obligation. (`rule:drafts:authority`); (`sec:admission:entry`); (`sec:admission:headers`).

## 7. Decision page · `sec:admission:decisions`

**Question (Which admission disposition is warranted)** · `q:admission:admission-decision`

The admission question reserved by the backlog's next-gate line is whether to charter Guide 15 as Phase 7's execution guide unchanged, with the named entry/header/preflight/scope amendments, or only after specified prerequisites are satisfied. (`T12-008`); (`rule:drafts:authority`).

**Explanation (A proposed plan and an authorized work order)** · `expl:admission:admission-analogy`

The choice is whether a proposed building plan is ready to become the work order: accepted foundations can be reused, but a survey of their limits is not permission to build the next floor, and an unresolved inspection cannot be certified by changing the plan's cover.

**Case (Admit without the named amendments)** · `case:admission:admit-unchanged`

Admit as is would change execution authority and reconcile the guide/index/card/backlog declarations, with proposed Wave 0 first in order, but the literal semantic-entry and preflight failures would still block an unconditional wave opening; it cannot honestly mean immediate Wave-1 implementation. (`gate:admission:preflight`).

**Case (Admit with explicit amendments)** · `case:admission:admit-amended`

Admit with named amendments would choose promotion in place or a superseding execution file, adopt one header alternative, record the twelve preflight dispositions and candidate baseline, distinguish preflight entry from Wave-1 typed-declaration delivery, fix the generation/carrier/representation obligations, align package scope and preserve all inherited non-claims; its first authorized wave could then be bounded Wave 0 revalidation and disposition, with implementation held until its explicit exit. (`sec:admission:headers`); (`sec:admission:preflight`); (`sec:admission:execution-obligations`).

**Case (Defer pending specified prerequisites)** · `case:admission:defer`

Defer would leave the concept draft and all execution declarations unchanged pending the exact candidate/regression record, semantic-entry ordering, constructor-generation choice, burn-record carrier and complete preflight disposition; it opens no wave. (`tab:admission:entry-standings`).

**Condition (Without admission no wave opens)** · `cond:admission:default`

The default at the assessed base remains: Phase 7 is Active, Guide 15 is a concept draft, no execution guide is chartered, and no wave opens; the accepted Phase-6 exit, public policy and compact-ASH result remain exactly at their recorded scopes; (`phase:roadmap:burn-clear`). The subsequent charter separately chooses a superseding guide and phase-versus-guide entry, while retaining its own Wave-0 entry and exit obligations and the accepted dependence of Waves 2–14 on Wave 1. (`def:guide15-exec:entry-boundaries`); (`gate:guide15-exec:guide-entry`); (`rule:guide15-exec:observable-retention`); (`rule:guide15-exec:second-contract-dependency`).
