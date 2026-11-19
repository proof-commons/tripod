# Guide 15 — Burn, ASH, and Clear: execution guide

> **Status:** Chartered execution guide of Phase 7; entry is governed by (`gate:guide15-exec:guide-entry`) and (`task:guide15-exec:wave0`), and new semantics remain held until (`gate:guide15-exec:wave0`).
> **Phase:** Phase 7 — Burn, ASH, and Clear (`phase:roadmap:burn-clear`).
> **Entry:** The amended Phase-6 exit (`gate:phase6:exit`), the accepted public-declassification policy (`sec:public-opening:result`), and the accepted Phase-4 compact-ASH candidate at its recorded scope (`phase:roadmap:compact-ash`), subject to the entry register below.
> **Operations:** `burn`, `compact-ash`, and `clear`; the second exemplar's `SOURCE` and `MERGE` are Wave-1 declarations whose exact typed spellings Wave 1 fixes.
> **Affected packages:** `architecture`, `realization`, `compiler`, `target-elements`, `tapscript`, `linker`, `transaction`, and `vectors`; `target-elements-conformance` only for a demonstrated missing target-generic fact; `model` supplies independent conformance evidence and `labels` checks registrations.
> **Supersedes as execution direction:** `plans/guides/guide_fifteen_concept.md`; its rules, gates, questions, and semantic chapters remain the cited semantic reference, and its artifact changes only by the mints of its entry conditions and record fields.
> **Does not implement:** Redemption, request admission, attestation settlement, cycle, entitlement issuance, distribution construction, RESV or PACE succession, production key custody, production opening custody, production deployment, final calibration, or release.
> **Required result:** The Phase-7 exit gate met on its text (`gate:phase7:exit`), with a second compiled contract carried through the same pipeline as independence evidence.
> **Authority:** Attestation, Realization, typed architecture, implemented ADRs, accepted decisions, and accepted research and phase results take precedence as in `plans/guides/guide_fourteen.md`; the Phase-6 closeout is accepted only at its amended scope (`sec:phase6:exit-assessment`); (`abst:admission:scope`) supplies admission planning input; the Keyed Provenance Covenant paper is the second contract of record (`[A-abst:kpc:abstract]`).
> **Trust boundary:** Repository source and selected executors run only within the established execution trust boundary; public disposable fixtures retain the security requirements of `adr/015-public-data-and-execution-trust.md`.

## 1. Why this order · `sec:guide15-exec:order`

**Thesis (Debt closure and independence precede new semantics)** · `thesis:guide15-exec:execution-order`

Inherited debt and preflight close before the second contract tests the shared compiler boundaries, and those boundaries settle before burn, ASH, and clear depend on them; each wave admits its own evidence rather than borrowing its predecessor's conclusion (`rule:guide15-exec:baseline`).

**Fact (Inherited Safety debt)** · `fact:guide15-exec:ordering-debt`

The inherited Safety matrix has 128 outstanding rows, and the operation-region refit remains open; composing new semantics on that base would leave a later failure attributable either to the inherited boundary or to the new operation (`sec:phase6:exit-assessment`); (`T11-064`).

**Justification (Region scope precedes composition)** · `just:guide15-exec:region-first`

The region refit is exercised by the first composed operation, so it precedes every new operation rather than becoming a repair attempted after a composed failure (`T11-064`).

**Justification (Closure exactness precedes reused checks)** · `just:guide15-exec:closure-first`

The closure-exact audit gates new semantics because compact and live checks are reused by the later operations; their necessity must follow their own contracts before reuse can carry a justified boundary (`T11-063`).

**Justification (Compact negatives establish the common baseline)** · `just:guide15-exec:compact-baseline`

The compact negative half gates new semantics to establish one complete inherited baseline, rather than because the second contract semantically requires those refusals; its eventual ASH consumers still owe their own subject-bound evidence (`T4-009`); (`task:guide15:wave4`); (`task:guide15:wave10`).

**Justification (Performance debt belongs to the common baseline)** · `just:guide15-exec:performance-baseline`

The performance row gates new semantics because the debt wave closes every inherited carrier before establishing its baseline; performance measurement is not a semantic prerequisite of the second contract (`T11-110`).

**Condition (Shared target provision and exact subjects)** · `cond:guide15-exec:shared-target`

The 35 native-run-required rows and the second contract's measurements require the pinned target on the node of record; one environment, once provisioned and recorded, supports both, while each observation retains its own exact subject and evidence boundary (`sec:phase6:exit-assessment`); (`task:guide15-exec:bite-1-0`).

**Argument (A separately gated debt baseline isolates later changes)** · `arg:guide15-exec:baseline-order`

A debt wave with its own exit turns the inherited figures into an explicit baseline; later waves compare their figures against that baseline instead of treating a historical green gate as an answer to a new question.

**Argument (The minimal second contract tests semantic boundaries)** · `arg:guide15-exec:second-contract-order`

The second exemplar's SOURCE and MERGE declarations test the architecture-to-realization boundary before the larger Phase-7 declarations; its contract is the paper and its delivery requirements, package mapping and measurements are re-derived in Wave 1 (`[A-def:kpc-model:first-output]`); (`task:guide15-exec:wave1`).

**Caveat (Carrier measurements remain required)** · `cav:guide15-exec:carrier-order`

Wave 1 measures its own two-clause covenants and cannot establish the attestation record carrier's data-output policy. The carrier provisionally selected at 0.7 still requires its exact size and policy measurements in Wave 4 before declarations; refusal reopens selection by a Wave-4 ruling (`q:guide15:burn-record-carrier`); (`cond:guide15-exec:carrier-reopening`).

**Justification (Architectural failure is tested before native cost)** · `just:guide15-exec:early-boundary-test`

An independence failure is cheapest to find before burn and clear depend on the same seams; bite 1.1 tests that boundary before or beside the native campaign, and the result names which seams moved, why they moved, and which laws remained specific to each contract (`task:guide15-exec:bite-1-6`); (`crit:guide15-exec:wave1-independent`); (`rule:roadmap:parallel`).

**Record (Admission inside Phase 7)** · `rec:guide15-exec:admitted-second-contract`

The phase order does not change: the second contract is independence evidence inside Phase 7, admitted by the phase's change statement (`sec:phase7:change-statement`).

**Rule (Burn and clear depend on the second contract exit)** · `rule:guide15-exec:second-contract-dependency`

Burn, ASH, and clear delivery in Waves 2–14 and Phase 7's exit depend on Wave 1's exit, because the shared seams the second contract moves must settle before burn and clear depend on them; this dependency is accepted under (`rule:roadmap:change`) and stated in (`sec:phase7:change-statement`).

**Consequence (Later phases inherit the Phase-7 dependency)** · `cor:guide15-exec:later-phase-dependency`

The later phases wait for Phase 7 under (`rule:roadmap:sequence`), so their opening also depends on the second contract's evidence through Phase 7's exit, without changing the numbered phase order or normative version.

## 2. Entry and the register · `sec:guide15-exec:entry`

**Definition (Phase entry and guide entry)** · `def:guide15-exec:entry-boundaries`

Phase entry admits the phase on its accepted predecessors; guide entry additionally identifies and disposes the compact candidate and the concept's detailed entry and preflight requirements, without converting those future dispositions into completed implementation (`phase:roadmap:burn-clear`); (`sec:guide15:entry`); (`sec:guide15:preflight`).

**Precondition (Accepted phase-entry evidence)** · `pre:guide15-exec:phase-entry`

The Phase-7 card retains two accepted entry conditions: Phase 6's exit on its amended assessment and the accepted public-declassification policy; the amendments establish only their stated scope (`gate:phase6:exit`); (`sec:phase6:exit-assessment`); (`sec:public-opening:result`).

**Gate (Additional guide-entry obligations)** · `gate:guide15-exec:guide-entry`

Wave 0 opens only when its backlog entry record identifies the provisioned pinned target and every other entry obligation; the accepted compact candidate and all detailed preflight rows must receive their assigned dispositions before the wave exits (`task:guide15-exec:wave0`); (`tab:guide15-exec:preflight`).

**Fact (Accepted compact candidate at recorded scope)** · `fact:guide15-exec:compact-candidate`

The accepted Phase-4 compact candidate has twelve accepted and matched submissions, coverage 100 of 211, and its section-29 audit at 94 of 105, each at the candidate scope recorded by (`phase:roadmap:compact-ash`).

**Caveat (Compact acceptance does not complete negative coverage)** · `cav:guide15-exec:compact-acceptance-scope`

Those figures establish neither complete negative coverage nor final calibration; the inherited negative half closes separately at (`rec:backlog:wave0-compact-inherited-closure`), and candidate bounds retain (`rule:guide15:candidate-only`).

**Setting (Review basis and authoritative contracts)** · `setup:guide15-exec:review-basis`

The review basis comprises (`sec:admission:entry`), (`sec:admission:preflight`), (`sec:admission:findings`), (`[A-abst:kpc:abstract]`), the concept (`sec:guide15:rulings`), the Phase-7 card (`phase:roadmap:burn-clear`), the amended Phase-6 assessment and gate record (`sec:phase6:exit-assessment`); (`sec:phase6:gate-record`), Guide-11 and Guide-12 results (`sec:public-opening:result`); (`tab:guide12-exec:preflight`), typed architecture in `packages/architecture/README.md`, model transitions, realization declarations, the reviewed target, and the affected package contracts `packages/model/README.md`; (`pkg:realization:contract`); (`pkg:compiler:contract`); (`pkg:target-elements:contract`); (`pkg:tapscript:contract`); (`pkg:linker:contract`); (`pkg:transaction:contract`); (`pkg:vectors:contract`).

**Rule (The concept remains the semantic reference)** · `rule:guide15-exec:semantic-reference`

The concept remains the semantic reference for its twenty-three governing rules, seven entry gates and repository-entry procedure, twelve preflight questions, and burn, ASH, clear, STATE, event, failure, evidence, and ABI chapters (`sec:guide15:rulings`); (`sec:guide15:entry`); (`rule:guide15:repository-entry`); (`sec:guide15:preflight`).

**Definition (Finding and execution standings)** · `def:guide15-exec:register-standings`

Finding status records the audited scope: HOLDS is established policy, structure, or candidate evidence; DOES NOT HOLD is a contradicted requirement; UNDECIDED requires a choice or new observation. Execution status separately records the work still owed, initially OPEN for every entry, following (`tab:guide12-exec:preflight`) and the admission study's definitions (`def:admission:holds`), (`def:admission:does-not-hold`), (`def:admission:undecided`) and scope convention (`conv:admission:standing-scope`).

**Definition (Source kind and evidence class)** · `def:guide15-exec:evidence-classes`

Source kind distinguishes a condition bullet from a repository-record field; evidence class distinguishes inherited policy, structural implementation, observed candidate evidence, and a new Phase-7 choice. No class supplies another class's proof merely by sharing an entry row (`tab:guide15-exec:preflight`).

**Convention (Entry identifiers and cross-cutting obligations)** · `conv:guide15-exec:entry-identifiers`

The concept's condition and record-field labels key the 65 entries (`sec:guide15:entry`), with their audited standings in (`tab:admission:entry-standings`); the existing question grouping remains a navigation aid, while the entry-gate column supplies each row's actual entry category and cross-cutting obligations apply to every consuming question (`tab:guide15-exec:preflight`).

**Result (Entry standing census)** · `result:guide15-exec:entry-census`

The register contains 50 HOLDS, two DOES NOT HOLD, and thirteen UNDECIDED findings: its 55 condition bullets contribute 41, two, and twelve respectively, and its ten repository-record fields contribute nine HOLDS and one UNDECIDED (`tab:guide15-exec:preflight`).

**Rule (An inherited finding is not a fresh observation)** · `rule:guide15-exec:finding-boundary`

HOLDS never means a fresh native observation or a completed future implementation; a historical finding remains attached to its evidence subject while the guide's execution obligation remains OPEN until its own gate (`def:guide15-exec:register-standings`).

### Semantic entry without a circular prerequisite · `sec:guide15-exec:semantic-entry`

**Definition (Semantic entry has two obligations)** · `def:guide15-exec:semantic-entry-split`

The semantic-entry gate is read in two halves: known model law and complete field ownership belong to Wave 0, while typed compiler-facing burn and clear declarations belong to Wave 2; (`cond:guide15:operation-scope`) and (`cond:guide15:validated-relations`) retain DOES NOT HOLD rather than acquiring a fictitious implementation pass (`gate:guide15:semantic-entry`).

**Precondition (Model law and field ownership before new semantics)** · `pre:guide15-exec:model-law-entry`

Wave 0 identifies the complete model-law and field-owner census before any new operation is implemented, including burn records, STATE assignment, public availability, constructor continuity, signing selection, and report roles (`q:guide15:semantic-census`); (`rule:guide15:typed-source`).

**Task (Compiler-facing declarations belong to Wave 2)** · `task:guide15-exec:declaration-entry`

Wave 2 delivers and validates the missing typed burn and clear declarations against architecture and independent model conformance; model support alone does not make unsupported realization dispatch supported (`task:guide15:wave1`); (`pkg:realization:contract`).

**Obligation (Authoritative relation and assignment censuses)** · `obl:guide15-exec:entry-censuses`

Wave 0 must identify the authoritative relation and assignment censuses and name the later validation obligations for every unsupported declaration, preserving the separation of stored fields, derived quantities, and evidence roles (`q:guide15:semantic-census`); (`q:guide15:clear-assignment`).

**Rule (Consuming waves establish their own implementation evidence)** · `rule:guide15-exec:consuming-evidence`

The split applies row by row: the consuming wave must establish later evidence before accepting an implementation result, even where the inherited law or structure already HOLDS (`rule:guide15-exec:baseline`).

### The twelve controlling dispositions · `sec:guide15-exec:dispositions`

**Task (Disposition 1 — Semantic census)** · `task:guide15-exec:disposition-1`

Bite 0.7 records exact typed owners for burn, ASH, clear, STATE, records, events, accounting, and certificates, with its position accepted by a numbered Phase-7 ruling (`q:guide15:semantic-census`); (`req:guide15-exec:position-acceptance`).

**Obligation (Missing declarations and conformance)** · `obl:guide15-exec:disposition-1-proof`

Wave 2 must deliver the missing realization declarations and conformance evidence; the model census supplies authority, not that implementation result (`task:guide15:wave1`).

**Rule (Disposition 2 — Explicit-only ASH representation)** · `rule:guide15-exec:disposition-2`

Retain the accepted explicit-only public boundary; public commitments require separately accepted authenticated-opening evidence before admission (`q:guide15:ash-representation`); (`sec:public-opening:result`).

**Task (Revalidate the accepted public boundary)** · `task:guide15-exec:disposition-2-validation`

Bite 0.7 revalidates that boundary at its recorded scope and records the Phase-7 selection without treating a report attachment as an authenticated opening (`q:guide15:ash-representation`).

**Rule (Disposition 3 — Normalized-first private burn)** · `rule:guide15-exec:disposition-3`

Normalized-first is mandatory; direct-private support remains absent unless its distinct admission obligations are met and recorded, preserving the accepted private-change boundary (`q:guide15:private-burn`); (`rule:guide15:private-source-policy`); (`sec:public-opening:result`).

**Task (Disposition 4 — ASH constructor selection)** · `task:guide15-exec:disposition-4`

Bite 0.7 records intended exact equality or explicit class transitions for ASH constructors, accepted by a numbered Phase-7 ruling (`q:guide15:ash-constructor`); (`req:guide15-exec:position-acceptance`).

**Obligation (Prove actual constructor equality or admitted transitions)** · `obl:guide15-exec:disposition-4-proof`

Wave 5 proves fresh, compacted, and residual equality on actual linked subjects or the explicitly admitted transitions; a shared semantic class alone does not establish byte-level constructor equality (`task:guide15:wave4`); (`q:guide15:ash-constructor`).

**Task (Disposition 5 — Provisional burn-record carrier)** · `task:guide15-exec:disposition-5`

Bite 0.7 selects a canonical, publicly recoverable carrier with exact association and ordering, accepted provisionally by a numbered Phase-7 ruling; Wave-4 carrier measurements cannot substitute for this preflight disposition (`q:guide15:burn-record-carrier`).

**Condition (Carrier selection reopens before Wave-4 declarations)** · `cond:guide15-exec:carrier-reopening`

Wave 4 must measure the complete attestation carrier selected at 0.7, including the multiple-data-output standardness premise and exact zero-value 53-byte N-output form, with all outputs, proofs, weight and fees. Refusal of encoded size or policy reopens selection by a Wave-4 ruling before declarations. Wave-1 measurements (b) and (f) concern the two-clause covenant and do not discharge it (`q:guide15:burn-record-carrier`); (`task:guide15-exec:wave4`).

**Task (Disposition 6 — Burn-record cardinality and value law)** · `task:guide15-exec:disposition-6`

Bite 0.7 derives zero/count/ordinal/duplicate/value/aggregate rules from typed authority, retaining any rule stronger than a minimum inequality and obtaining numbered acceptance of the disposition (`q:guide15:burn-record-rule`); (`req:guide15-exec:position-acceptance`).

**Rule (Disposition 7 — Stored clear assignment and derived total)** · `rule:guide15-exec:disposition-7`

The stored assignment changes `y_l` alone by X; total Y is derived and falls by X, with other stored fields unchanged, so the two equations are not two independent stored-field writes (`q:guide15:clear-assignment`); (`rule:guide15:clear-amount`).

**Obligation (Validate the clear assignment in declarations)** · `obl:guide15-exec:disposition-7-proof`

Wave 2 must validate that stored-versus-derived distinction against independent model conformance and the complete field-owner census (`task:guide15:wave1`); (`q:guide15:clear-assignment`).

**Task (Disposition 8 — Arithmetic domains and schedule)** · `task:guide15-exec:disposition-8`

Bite 0.7 records widths, domains, sum bounds, overflow refusal, branch schedule, consumed comparisons, and authenticated minimum operands for addition, subtraction, and comparison, with numbered acceptance (`q:guide15:clear-arithmetic`); (`req:guide15-exec:position-acceptance`).

**Task (Disposition 9 — STATE generation policy)** · `task:guide15-exec:disposition-9`

Bite 0.7 records synthetic Phase-7 origin, the maturity-plus-clear subtree, no generation crossing, maturity revalidation, and no migration claim, or an explicitly accepted migration prerequisite; the selection requires a numbered ruling (`q:guide15:state-generation`); (`rule:guide15:constructor-generation`).

**Rule (Disposition 10 — Independent event and accounting roles)** · `rule:guide15-exec:disposition-10`

Keep operation acceptance, event type, record value, compact silence, clear event, accounting, and STATE succession as separate validated roles (`q:guide15:event-independence`); (`rule:guide15:event-anchors`).

**Task (Disposition 11 — Public construction evidence)** · `task:guide15-exec:disposition-11`

Bite 0.7 defines fresh-process tests using public chain data, bundle, ABI, values, metadata, and constructor data; optional sponsor-local capability cannot hide creator dependencies, and the position receives numbered acceptance (`q:guide15:permissionless`); (`req:guide15-exec:position-acceptance`).

**Rule (Disposition 12 — Internal query default)** · `rule:guide15-exec:disposition-12`

Internal record and event projection is the default, with no external format or identity implied by internal evidence consumption (`q:guide15:attestation-query`).

**Precondition (An external consumer activates interchange adjudication)** · `pre:guide15-exec:external-query`

A named external consumer activates ADR-022 before an interface or identity is implemented; the internal default does not discharge that future boundary (`[ADR022-rule:interchange:no-implementation]`); (`q:guide15:attestation-query`).

**Requirement (Numbered acceptance of position dispositions)** · `req:guide15-exec:position-acceptance`

Dispositions 1, 4, 5, 6, 8, 9, and 11, and every position-bearing UNDECIDED entry, require acceptance by a numbered ruling on the Phase-7 card in the form used by (`rule:phase6:wave13-rulings`); a bite's gate records evidence and cannot silently accept its own design choice.

**Criterion (Position decisions and run outcomes are distinct)** · `crit:guide15-exec:position-versus-observation`

(`cond:guide15:public-revalidation`), (`cond:guide15:private-route`), (`cond:guide15:public-ash`), (`cond:guide15:ash-compatibility`), (`cond:guide15:generation-policy`), (`cond:guide15:origin-nonclaims`), (`cond:guide15:signing-profiles`), (`cond:guide15:separate-evidence`), and (`data:guide15:compact-revision`) require position acceptance; (`cond:guide15:maturity-green`), (`cond:guide15:sponsor-opacity`), (`cond:guide15:compact-build`), and (`cond:guide15:compact-regression`) instead require run outcomes. The first swiftness bite also requires its own ruling and landing under (`T11-110`), and all acceptance records must be present at (`gate:guide15-exec:wave0`).

### Preflight register · `tab:guide15-exec:preflight`

**Definition (Register columns and row-specific blockers)** · `def:guide15-exec:register-columns`

Each row identifies its source entry gate, audited finding, execution status, concrete work still blocking its execution exit, Wave-0 bite, later consuming wave, evidence paths, evidence labels, source kind, evidence class, and retained evidence scope. Finding words are qualified by the scope column; OPEN is an execution status, not a failed observation (`def:guide15-exec:register-standings`).

**Rule (Preflight closure leaves later implementation obligations open)** · `rule:guide15-exec:register-exits`

A Wave-0 disposition closes preflight only; later proof remains OPEN until its consuming gate. Sponsor opacity applies to burn, compact, and clear, so (`cond:guide15:sponsor-opacity`) names Wave 7's clear evidence beside Waves 3 and 6 (`rule:guide15:sponsor-opacity`).

| Condition or field | Entry | Entry gate | Question | Finding status | Execution status | Row-specific blocker | Wave-0 bite | Later wave | Evidence paths | Evidence labels | Source kind | Evidence class | Evidence scope |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| (`cond:guide15:phase6-exited`) | §2.1 Phase-6 entry: Phase 6 has exited on the current tree | (`gate:guide15:phase6-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | RECORDED: Amended exit plus Safety’s five-conjunct reconciliation. (`sec:phase6:exit-assessment`); (`result:backlog:bite-0-5-gate-pass`) | Acceptance: Ruling 135: non-answer and native gate readings at (`gate:guide15-exec:bite-0-5-non-answer-gate`) and (`gate:guide15-exec:bite-0-5-native-gate`). No later consuming-wave obligation in this row. | 0.7, 0.5 | — | — | (`sec:phase6:exit-assessment`) | condition bullet | inherited policy | Exited on the amended gate; this does not pass unamended Guide 14 §23. |
| (`cond:guide15:clean-tree`) | §2.1 Phase-6 entry: the tree is clean | (`gate:guide15:phase6-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | RECORDED: Entry classification plus fresh baseline clean-tree result. (`rec:backlog:wave0-entry`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1, 0.7 | — | `scripts/gate-clean-tree.sh` | (`sec:phase6:gate-record`) | condition bullet | structural implementation | The gate-of-record clean-tree procedure is available; the admission inspection recorded a clean tree. |
| (`cond:guide15:operation-scope`) | §2.2 Semantic entry: `burn`, `compact-ash`, and `clear` remain in architecture and realization scope under their exact existing identifiers | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | DOES NOT HOLD | RECORDED: Exact identifiers and missing dispatch recorded; finding stays DOES NOT HOLD. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 140: (`task:guide15-exec:disposition-1`). Retained consuming obligation: Wave 2: Implement missing Burn and Clear declaration dispatch and architecture weld. | 0.7 | 2 | `packages/architecture/src/ids.rs`; `packages/realization/src/declarations/mod.rs` | — | condition bullet | new Phase-7 choice | All three architecture identifiers exist, but typed realization dispatch lacks Burn and Clear. |
| (`cond:guide15:validated-relations`) | §2.2 Semantic entry: burn, compact, and clear relations validate against architecture | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | DOES NOT HOLD | RECORDED: Complete model-law census and absent declaration validation recorded; finding stays DOES NOT HOLD. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 140: (`task:guide15-exec:disposition-1`). Retained consuming obligation: Wave 2: Validate Burn and Clear declarations against architecture and independent model conformance. | 0.7 | 2 | `packages/realization/src/declarations/mod.rs` | — | condition bullet | new Phase-7 choice | UnsupportedOperationDeclaration prevents complete burn/clear declaration validation. |
| (`cond:guide15:sponsor-opacity`) | §2.2 Semantic entry: sponsor-value opacity remains enforced | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | UNDECIDED | RECORDED: Region refit and sponsored closure. (`rec:backlog:wave0-region-refit`); (`result:backlog:bite-0-5-gate-pass`) | Acceptance: Ruling 130: region-scoped sponsored construction at (`rec:backlog:wave0-sponsored-ruling`); Ruling 135: non-answer and native gate readings at (`gate:guide15-exec:bite-0-5-non-answer-gate`) and (`gate:guide15-exec:bite-0-5-native-gate`). Retained consuming obligation: Waves 3, 6, 7: Prove operation-specific sponsor opacity for burn, compact and clear. | 0.2, 0.5 | 3, 6, 7 | `packages/compiler/src/sponsor_region.rs` | (`sec:phase6:exit-assessment`) | condition bullet | new Phase-7 choice | Inherited opacity holds in its scope; burn/clear enforcement is not delivered and the region refit remains open. |
| (`cond:guide15:target-independent`) | §2.2 Semantic entry: no target-specific type enters realization | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | RECORDED: Census confirms semantic-only realization boundary. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 140: (`task:guide15-exec:disposition-1`). Retained consuming obligation: Wave 2: Keep target-specific types outside the realization declaration boundary. | 0.7 | 2 | `packages/realization/src/lib.rs` |  (`pkg:realization:contract`); | condition bullet | inherited policy | The contract and public semantic surface retain target independence. |
| (`cond:guide15:lifecycle-obligations`) | §2.2 Semantic entry: lifecycle obligations remain explicit | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | RECORDED: Exact exit declarations and incomplete constructors recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 140: (`task:guide15-exec:disposition-1`); Ruling 142: (`task:guide15-exec:disposition-4`) and (`cond:guide15:ash-compatibility`). Retained consuming obligation: Waves 5, 11: Prove actual constructor and lifecycle closure. | 0.7 | 5 and 11 | `packages/realization/src/lifecycle.rs` | (`sec:phase6:exit-assessment`) | condition bullet | inherited policy | Lifecycle obligations and candidate incompleteness remain explicit. |
| (`data:guide15:starting-revision`) | §2.8 Repository record: starting revision | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | RECORDED: Entry’s actual `0.7.9-dev`, workspace `0.7.0-dev`. (`rec:backlog:wave0-entry`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1 | — | `Cargo.toml`; `plans/phases/README.md` | — | repository-record field | structural implementation | The study records an older starting tree; Wave 0 records the actual execution base. |
| (`data:guide15:phase6-revision`) | §2.8 Repository record: Phase-6 result revision | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | RECORDED: Entry’s retained Phase-6 evidence and amended closeout references. (`rec:backlog:wave0-entry`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1 | — | — | (`sec:phase6:gate-record`); (`sec:phase6:exit-assessment`) | repository-record field | observed candidate evidence | Evidence base 0.6.305-dev; gate-record landing 0.6.306-dev; amended closeout carried at 0.7.0-dev. |
| (`data:guide15:architecture-values`) | §2.8 Repository record: architecture schema and semantic hashes | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | RECORDED: Schema 18 and generated registered fields cited by path. (`rec:backlog:wave0-entry`); (`rec:backlog:wave0-preflight-census`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.7 | — | `packages/architecture/src/spec.rs`; `plans/registers/identities.md`; `packages/model/generated/architecture.json` | — | repository-record field | structural implementation | Schema 18; semantic and behavioural values are the registered fields in the generated architecture publication, cited by path rather than copied digests. |
| (`data:guide15:realization-version`) | §2.8 Repository record: realization version | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | RECORDED: Derived `0.7.0-dev`; no new identity. (`rec:backlog:wave0-entry`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1, 0.7 | — | `Cargo.toml`; `packages/architecture/src/spec.rs` | — | repository-record field | structural implementation | 0.7.0-dev, derived from the workspace version. |
| (`data:guide15:clean-result`) | §2.8 Repository record: clean-tree result | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | RECORDED: Fresh baseline result, not copied entry pass. (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.7 | — | `scripts/gate-clean-tree.sh` | (`sec:phase6:gate-record`) | repository-record field | observed candidate evidence | The gate-of-record clean-tree check over 0.6.305-dev is MET; this is the recorded result, not a fresh workspace gate. |
| (`cond:guide15:public-revalidation`) | §2.3 Public-declassification entry: the accepted public boundary is revalidated | (`gate:guide15:declassification-entry`) | Q2 (`q:guide15:ash-representation`) | UNDECIDED | RECORDED: Recorded explicit-only boundary revalidated. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). No later consuming-wave obligation in this row. | 0.7 | — | — | (`sec:public-opening:result`) | condition bullet | new Phase-7 choice | Accepted policy is known; Phase-7 revalidation is a Wave-0 deliverable, not a completed run here. |
| (`cond:guide15:ash-selection`) | §2.3 Public-declassification entry: the initial ASH representation is explicitly selected | (`gate:guide15:declassification-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | RECORDED: Explicit-only selection recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`); (`rule:guide15-exec:disposition-2`). Retained consuming obligation: Wave 4: Assess the actual selected Phase-7 representation forms. | 0.7 | 4 | — | (`sec:public-opening:result`) | condition bullet | inherited policy | The inherited initial public ownerless maintenance representation is explicit only. |
| (`cond:guide15:authenticated-opening`) | §2.3 Public-declassification entry: no unauthenticated opening is accepted | (`gate:guide15:declassification-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | RECORDED: Unsupported opening and refusal of scalar substitutes retained. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). No later consuming-wave obligation in this row. | 0.7 | — | — | (`sec:public-opening:result`)  (`pkg:target-elements:contract`); | condition bullet | inherited policy | Authenticated opening remains unsupported; no unauthenticated amount is an accepted substitute. |
| (`cond:guide15:public-value`) | §2.3 Public-declassification entry: no report attachment substitutes for chain-public value | (`gate:guide15:declassification-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | RECORDED: Chain-public value authority retained. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). No later consuming-wave obligation in this row. | 0.7 | — | `packages/transaction/src/view.rs` | (`sec:public-opening:result`) | condition bullet | inherited policy | The explicit public value is construction input; a report attachment supplies no opening authority. |
| (`cond:guide15:value-introspection`) | §2.6 Target entry: explicit `U` and explicit value introspection are reviewed | (`gate:guide15:target-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | RECORDED: Reviewed explicit asset/value capabilities recorded. (`rec:backlog:wave0-preflight-census`); (`pkg:target-elements:contract`) | Acceptance: Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). Retained consuming obligation: Wave 4: Assess explicit U/value introspection for each selected form. | 0.7 | 4 | `packages/target-elements/src/opcode.rs`;  |  (`pkg:target-elements:contract`); | condition bullet | structural implementation | Explicit asset/value introspection is within the reviewed target; authenticated opening is not. |
| (`data:guide15:public-policy`) | §2.8 Repository record: selected public-value policy | (`rule:guide15:repository-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | RECORDED: Explicit-only, normalized-first, private-change policy recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). No later consuming-wave obligation in this row. | 0.7 | — | — | (`sec:public-opening:result`) | repository-record field | inherited policy | Explicit-only boundary, normalized-first with private change; public-committed opening is deferred. |
| (`cond:guide15:private-route`) | §2.3 Public-declassification entry: any private-receipt burn path is classified as normalized-first or separately admitted direct-private | (`gate:guide15:declassification-entry`) | Q3 (`q:guide15:private-burn`) | UNDECIDED | RECORDED: Normalized-first selection. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 141: (`cond:guide15:public-revalidation`) and (`cond:guide15:private-route`). Retained consuming obligation: Wave 4: Assess the normalized-first route; direct-private remains separately admitted only. | 0.7 | 4 | `plans/guides/guide_fifteen_concept.md` | (`sec:public-opening:result`) | condition bullet | new Phase-7 choice | Normalized-first is the inherited route; a Phase-7 execution disposition has not been recorded. |
| (`cond:guide15:compact-build`) | §2.4 Compact-ASH entry: the Phase-4 compact-ASH operation still builds | (`gate:guide15:compact-entry`) | Q4 (`q:guide15:ash-constructor`) | UNDECIDED | RECORDED: Selected compact subject rebuilt by completed-tree baseline. (`rec:backlog:wave0-compact-inherited-closure`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.4, 0.7 | — | `packages/compiler/src/operation_plan.rs`; `plans/phases/04-compact-ash.md` | — | condition bullet | new Phase-7 choice | The operation exists and has a passing gate of record; current-base build is not rerun by this study. |
| (`cond:guide15:compact-regression`) | §2.4 Compact-ASH entry: its compiler plan, patterns, linker, ABI, vectors, and resource study remain green | (`gate:guide15:compact-entry`) | Q4 (`q:guide15:ash-constructor`) | UNDECIDED | RECORDED: Current compiler/pattern/linker/ABI/vector/resource regressions in baseline. (`rec:backlog:wave0-compact-inherited-closure`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.4, 0.7 | — | `plans/history/guide-12-completion-report.md`; `packages/vectors/src/resource_study.rs` | — | condition bullet | new Phase-7 choice | Historical candidate results do not certify a fresh regression run over this base. |
| (`cond:guide15:ash-compatibility`) | §2.4 Compact-ASH entry: its ASH constructor is compatible with burn-created and clear-residual ASH, or the incompatibility is explicitly dispositioned | (`gate:guide15:compact-entry`) | Q4 (`q:guide15:ash-constructor`) | UNDECIDED | RECORDED: Exact-equality intent and proof boundary recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 142: (`task:guide15-exec:disposition-4`) and (`cond:guide15:ash-compatibility`). Retained consuming obligation: Waves 5, 8: Prove exact constructor equality on actual linked subjects. | 0.7 | 5 and 8 | `packages/tapscript/src/bundle.rs`; `packages/linker/src/bundle.rs` | — | condition bullet | new Phase-7 choice | No compiled burn output or clear residual exists for a cross-operation constructor equality proof. |
| (`data:guide15:compact-revision`) | §2.8 Repository record: compact-ASH candidate revision | (`rule:guide15:repository-entry`) | Q4 (`q:guide15:ash-constructor`) | UNDECIDED | RECORDED: Selected candidate pins per ruling 128 and compatibility joined. (`rec:backlog:wave0-compact-inherited-closure`) | Acceptance: Ruling 128: (`data:guide15:compact-revision`). No later consuming-wave obligation in this row. | 0.1, 0.4 | — | `plans/history/guide-12-completion-report.md`; `plans/phases/04-compact-ash.md` | — | repository-record field | new Phase-7 choice | The completion record names 0.4.1-dev; the exact inherited bundle/revision for a Phase-7 rerun still needs selection. |
| (`cond:guide15:operator-signing`) | §2.1 Phase-6 entry: operator signing is finalized-byte bound | (`gate:guide15:phase6-entry`) | Q5 (`q:guide15:burn-record-carrier`) | HOLDS | RECORDED: Finalized-byte boundary retained. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 144: (`task:guide15-exec:disposition-5`) and (`cond:guide15:signing-profiles`). Retained consuming obligation: Wave 9: Prove Burn signatures bind every finalized byte role. | 0.7 | 9 | `packages/transaction/src/state_signing.rs` | (`sec:phase6:gate-record`) | condition bullet | structural implementation | §23 row 6 is MET; operator authorization binds finalized bytes. |
| (`cond:guide15:signing-profiles`) | §2.6 Target entry: selected owner and sponsor sighash profiles are reviewed | (`gate:guide15:target-entry`) | Q5 (`q:guide15:burn-record-carrier`) | UNDECIDED | RECORDED: Exact selected profiles and protected roles. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 144: (`task:guide15-exec:disposition-5`) and (`cond:guide15:signing-profiles`). Retained consuming obligation: Waves 4, 9: Assess selected profiles and execute Phase-7 signing and mutation refusals. | 0.7 | 4 and 9 | `packages/target-elements/src/authorization.rs`; `packages/transaction/src/live_signing.rs` | — | condition bullet | new Phase-7 choice | Inherited owner/sponsor profiles are reviewed; their exact Phase-7 selection and binding remain to be recorded. |
| (`cond:guide15:typed-records`) | §2.2 Semantic entry: burn record and event semantics are typed | (`gate:guide15:semantic-entry`) | Q6 (`q:guide15:burn-record-rule`) | HOLDS | RECORDED: Model record/event law recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 140: (`task:guide15-exec:disposition-1`); Ruling 143: (`task:guide15-exec:disposition-6`); Ruling 147: (`cond:guide15:separate-evidence`). Retained consuming obligation: Waves 2, 10: Deliver typed record/event declarations and independently bound reports. | 0.7 | 2 and 10 | `packages/model/src/history.rs`; `packages/model/src/certify.rs` | — | condition bullet | structural implementation | BurnRecord and BurnProjection are typed model objects; compiler-facing realization is still missing. |
| (`cond:guide15:state-owner`) | §2.1 Phase-6 entry: canonical STATE metadata has one typed owner | (`gate:guide15:phase6-entry`) | Q7 (`q:guide15:clear-assignment`) | HOLDS | RECORDED: Six stored fields and codec ownership recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 140: (`task:guide15-exec:disposition-1`). Retained consuming obligation: Wave 2: Validate complete metadata ownership against independent model conformance. | 0.7 | 2 | `packages/realization/src/state.rs`; `packages/realization/src/state_codec.rs` | — | condition bullet | structural implementation | The typed metadata owner and canonical codec are inherited; §23 row 3 is MET. |
| (`cond:guide15:clear-assignment`) | §2.2 Semantic entry: the complete STATE clear assignment is known | (`gate:guide15:semantic-entry`) | Q7 (`q:guide15:clear-assignment`) | HOLDS | RECORDED: Only stored `y_l` changes; Y derived. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 140: (`task:guide15-exec:disposition-1`); (`rule:guide15-exec:disposition-7`). Retained consuming obligation: Wave 2: Validate the stored-versus-derived assignment against independent model conformance. | 0.7 | 2 | `packages/model/src/ops/ash.rs`; `docs/attestation/realization.md` | — | condition bullet | structural implementation | ClearAsh changes only stored y_l; total Y falls by the same X because Y is derived. |
| (`cond:guide15:recon-tag`) | §2.2 Semantic entry: `tag-recon` is architecture-owned and exact | (`gate:guide15:semantic-entry`) | Q7 (`q:guide15:clear-assignment`) | HOLDS | RECORDED: Exact `TagId::Recon` ownership recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 140: (`task:guide15-exec:disposition-1`). Retained consuming obligation: Waves 2, 7: Bind declarations and destruction to the exact U/Recon tag. | 0.7 | 2 and 7 | `packages/architecture/src/ids.rs`; `packages/architecture/src/spec.rs` | — | condition bullet | structural implementation | TagId::Recon owns tag-recon; the clear destruction declaration names it. |
| (`cond:guide15:candidate-limits`) | §2.4 Compact-ASH entry: no stale candidate limit is treated as final calibration | (`gate:guide15:compact-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | RECORDED: Bound subject and non-calibration status retained. (`rec:backlog:wave0-compact-inherited-closure`); (`rec:backlog:wave0-preflight-census`); (`sec:phase6:exit-assessment`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.7 | — | `plans/phases/04-compact-ash.md`;  |  (`pkg:linker:contract`); | condition bullet | inherited policy | Candidate bounds remain candidate; no final calibration follows from phase exit. |
| (`cond:guide15:target-definition`) | §2.6 Target entry: reviewed target definition validates | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | RECORDED: Reviewed V2 definition bound and validated in baseline. (`rec:backlog:wave0-entry`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1, 0.7 | — | `packages/target-elements/src/definition.rs`;  |  (`pkg:target-elements:contract`); | condition bullet | structural implementation | Reviewed V2 definition and validator are implemented; this is a structural inherited reading. |
| (`cond:guide15:development-binding`) | §2.6 Target entry: development binding is welded to the exact target | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | RECORDED: Entry’s exact development binding retained. (`rec:backlog:wave0-entry`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1 | — | `packages/target-elements/src/weld.rs`; `packages/target-elements/src/deployment.rs` | — | condition bullet | structural implementation | Development binding validates against the exact reviewed target definition. |
| (`cond:guide15:fixed-arithmetic`) | §2.6 Target entry: checked fixed-width addition, subtraction, and comparison are reviewed | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | RECORDED: Bounded checked schedule accepted. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 145: (`task:guide15-exec:disposition-8`). Retained consuming obligation: Wave 7: Emit and execute the composed bounded checked minimum schedule. | 0.7 | 7 | `packages/target-elements/src/opcode.rs`;  |  (`pkg:tapscript:contract`); | condition bullet | structural implementation | Reviewed fixed-width primitives exist; the clear-specific schedule remains a separate preflight obligation. |
| (`cond:guide15:target-conservation`) | §2.6 Target entry: target transaction conservation remains external evidence | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | RECORDED: External target dependency distinguished from semantic proof. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 140: (`task:guide15-exec:disposition-1`). Retained consuming obligation: Wave 3: Retain target conservation as external evidence, separate from semantic proof. | 0.7 | 3 | `packages/target-elements/src/confidential.rs`;  |  (`pkg:target-elements:contract`); | condition bullet | inherited policy | Conservation is an external target dependency, never a semantic proof of burn or clear. |
| (`cond:guide15:execution-domain`) | §2.6 Target entry: execution domain and leaf version are observed | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | RECORDED: Entry/Safety domain and leaf-version observations retained. (`rec:backlog:wave0-entry`); (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1, 0.5 | — | `packages/target-elements/src/definition.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | The accepted target observations bind the execution domain and leaf version of that candidate. |
| (`cond:guide15:relay-separation`) | §2.6 Target entry: consensus and relay policy remain separate | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | RECORDED: Positive-floor evidence and relay/block distinctions retained through Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/target-elements/src/resource.rs` | (`sec:phase6:exit-assessment`) | condition bullet | inherited policy | Zero fee floors and relay bounds remain explicit; consensus acceptance does not imply ordinary relayability. |
| (`data:guide15:target-revision`) | §2.8 Repository record: target contract revision | (`rule:guide15:repository-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | RECORDED: Entry’s target/deployment/provenance fields joined. (`rec:backlog:wave0-entry`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1 | — | `packages/target-elements/src/definition.rs` | — | repository-record field | structural implementation | TargetContractVersion::V2, the supported reviewed contract. |
| (`cond:guide15:maturity-green`) | §2.1 Phase-6 entry: maturity announcement remains green under its candidate constructor generation | (`gate:guide15:phase6-entry`) | Q9 (`q:guide15:state-generation`) | UNDECIDED | RECORDED: Entry and Safety evidence plus current baseline regression. (`rec:backlog:wave0-entry`); (`result:backlog:bite-0-5-gate-pass`); (`gate:guide15-exec:baseline-repository`); (`rec:backlog:wave0-baseline`) | Acceptance: No new position ruling required; inherited evidence retained. Retained consuming obligation: Wave 7: Revalidate maturity under the expanded constructor generation. | 0.1, 0.5 | 7 | — | (`sec:phase6:gate-record`) | condition bullet | new Phase-7 choice | Acceptance is recorded over the candidate; no fresh maturity run is claimed at this base. |
| (`cond:guide15:constructor-reconstruction`) | §2.1 Phase-6 entry: predecessor and successor constructor reconstruction are implemented | (`gate:guide15:phase6-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | RECORDED: Safety’s constructor and recovery evidence retained. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1, 0.5 | — | — | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | §23 rows 4 and 10 are MET for the accepted candidate's constructor reconstruction. |
| (`cond:guide15:generation-identity`) | §2.5 STATE generation entry: the Phase-6 constructor generation is identified exactly | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | RECORDED: Exact maturity-only subtree and constructor manifest recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). No later consuming-wave obligation in this row. | 0.1, 0.7 | — | `packages/tapscript/src/state_constructor.rs` | (`sec:phase6:gate-record`) | condition bullet | structural implementation | The accepted maturity-only constructor is identified by its retained bundle and evidence, not a new generation name. |
| (`cond:guide15:clear-subtree`) | §2.5 STATE generation entry: adding clear is recognized as a static-subtree change | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | RECORDED: Distinct maturity-plus-clear static set recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). Retained consuming obligation: Wave 7: Implement and validate the expanded static subtree. | 0.7 | 7 | `packages/tapscript/src/state_constructor.rs`; `plans/guides/guide_fifteen_concept.md` | — | condition bullet | inherited policy | Adding clear changes the static leaf set and authenticated subtree; the concept draft states this explicitly. |
| (`cond:guide15:generation-policy`) | §2.5 STATE generation entry: the Phase-7 generation policy of §1.14 is accepted or replaced by an explicit migration decision | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | UNDECIDED | RECORDED: Explicit synthetic-origin acceptance. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`); Ruling 131: (`task:guide15-exec:disposition-9`). No later consuming-wave obligation in this row. | 0.7 | — | `plans/guides/guide_fifteen_concept.md`; `plans/backlog.md` | — | condition bullet | new Phase-7 choice | The concept draft proposes synthetic Phase-7 origin; no execution admission accepts it yet. |
| (`cond:guide15:committed-leaf`) | §2.5 STATE generation entry: no test attempts to spend a Phase-6 STATE with an uncommitted Phase-7 leaf | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | RECORDED: No-generation-crossing rule recorded. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). Retained consuming obligation: Wave 7: Prove refusal of uncommitted Clear leaves and generation crossing. | 0.7 | 7 | `packages/transaction/src/state_abi.rs`; `packages/linker/src/state_bundle.rs` | — | condition bullet | inherited policy | The delivered candidate has no clear leaf; its existing constructor checks do not admit an uncommitted leaf. |
| (`cond:guide15:origin-nonclaims`) | §2.5 STATE generation entry: synthetic Phase-7 origin non-claims are agreed in advance | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | UNDECIDED | RECORDED: Explicit no-migration/no-old-generation-spend clauses. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). No later consuming-wave obligation in this row. | 0.7 | — | `plans/guides/guide_fifteen_concept.md`; `plans/backlog.md` | — | condition bullet | new Phase-7 choice | Synthetic origin and no-migration non-claims need an explicit admission disposition. |
| (`cond:guide15:constructor-hashing`) | §2.6 Target entry: STATE constructor hashing and tweak requirements remain reviewed | (`gate:guide15:target-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | RECORDED: Existing hash domains and tweak/parity requirements retained under rulings 74 and 112 and the gate record. (`rule:phase6:wave9-rulings`); (`rule:phase6:wave13-rulings`); (`sec:phase6:gate-record`). New-subtree proof boundary recorded at (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`). Retained consuming obligation: Wave 7: Prove hashing and tweak requirements for the expanded committed subtree. | 0.7 | 7 | `packages/target-elements/src/definition.rs`; `packages/tapscript/src/state_constructor.rs` | — | condition bullet | structural implementation | The Phase-6 hashing and tweak requirements remain reviewed; expanded-subtree evidence is not inherited. |
| (`data:guide15:state-generation`) | §2.8 Repository record: STATE-constructor generation | (`rule:guide15:repository-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | RECORDED: Predecessor and proposed generation manifests joined. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 146: (`task:guide15-exec:disposition-9`), (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`); Ruling 131: (`task:guide15-exec:disposition-9`). No later consuming-wave obligation in this row. | 0.1, 0.7 | — | `packages/tapscript/src/state_constructor.rs` | (`sec:phase6:gate-record`) | repository-record field | structural implementation | The Phase-6 maturity-only candidate in the gate record; Phase-7 generation is not yet accepted. |
| (`cond:guide15:edge-history`) | §2.1 Phase-6 entry: root-history evidence is edge-based | (`gate:guide15:phase6-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Independent edge evidence and checkpoint limits joined through Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/vectors/src/maturity_history.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | Edge-based evidence exists; row 11 retains the checkpoint and environment conditions. |
| (`cond:guide15:exact-bindings`) | §2.1 Phase-6 entry: target, deployment, request, response, and provenance bindings remain exact | (`gate:guide15:phase6-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Entry and admitted observation bindings retained. (`rec:backlog:wave0-entry`); (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1, 0.5 | — | `packages/vectors/src/maturity_continuity.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | Inherited validated evidence binds the exact subject; no broader target identity is inferred. |
| (`cond:guide15:compact-silence`) | §2.2 Semantic entry: compact ASH remains event-silent | (`gate:guide15:semantic-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Complete inherited projection/lifecycle census. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 147: (`cond:guide15:separate-evidence`); (`rule:guide15-exec:disposition-10`). Retained consuming obligation: Wave 11: Reproduce the complete compact event and lifecycle projection. | 0.7 | 11 | `packages/realization/src/declarations/compact_ash.rs`; `packages/model/src/ops/ash.rs` | — | condition bullet | structural implementation | Compact is ownerless-lateral and produces no burn or clear projection. |
| (`cond:guide15:native-eligibility`) | §2.6 Target entry: no mock is gate-eligible | (`gate:guide15:target-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Real-target admission boundary retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1, 0.5 | — | `plans/guides/guide_fifteen_concept.md`; `packages/vectors/src/maturity_native.rs` | — | condition bullet | inherited policy | The concept draft requires real target evidence; mocks and first-party fixtures cannot discharge native rows. |
| (`cond:guide15:validated-plans`) | §2.7 Evidence entry: canonical evidence plans have no unchecked constructors | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Validated-plan construction and exact row disposition retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/vectors/src/plan.rs`; `packages/vectors/src/maturity_evidence.rs` | — | condition bullet | structural implementation | Canonical evidence plans derive through validated inputs and preserve their internal invariants. |
| (`cond:guide15:validated-coverage`) | §2.7 Evidence entry: coverage changes only from validated report wrappers | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Validated wrappers and recounted categories retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/vectors/src/plan.rs`; `packages/vectors/src/maturity_report.rs` | — | condition bullet | structural implementation | Coverage discharge consumes validated evidence/report wrappers; declared requirements are not observations. |
| (`cond:guide15:observed-refusals`) | §2.7 Evidence entry: native refusals retain observed layers and exact subjects | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Exact subjects and observed layers retained. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: Ruling 134: diagnostic attribution at (`req:w0route:recapture-constraints`); Ruling 135: non-answer and native gate readings at (`gate:guide15-exec:bite-0-5-non-answer-gate`) and (`gate:guide15-exec:bite-0-5-native-gate`). No later consuming-wave obligation in this row. | 0.5 | — | `packages/vectors/src/observed_boundary.rs`; `packages/vectors/src/maturity_mutants.rs` | — | condition bullet | structural implementation | Observed layer and bound subject remain attached to native refusals. |
| (`cond:guide15:exact-refusals`) | §2.7 Evidence entry: wrong-boundary refusals are not counted as answers | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Matching-boundary credit and wrong-boundary non-credit retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/vectors/src/observed_boundary.rs` | — | condition bullet | inherited policy | matches_boundary requires exact equality; a refusal elsewhere does not answer the row. |
| (`cond:guide15:observed-evidence`) | §2.7 Evidence entry: report-layer answerability is not counted as observed evidence | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Fifteen report-consumer dispositions joined; no target-observation promotion. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/vectors/src/maturity_evidence.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | Report-layer required and observed evidence remain distinct; 15 report-layer-required rows remain outstanding. |
| (`cond:guide15:accepted-controls`) | §2.7 Evidence entry: accepted controls ground negative target observations | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Accepted family controls and exact associations retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/vectors/src/maturity_mutants.rs` | (`sec:phase6:gate-record`) | condition bullet | structural implementation | Admitted negative observations retain accepted controls; the reading is limited to those admitted observations. |
| (`cond:guide15:rejection-artifacts`) | §2.7 Evidence entry: target rejection carries no accepted artifact | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Validated refusal response shapes retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/target-elements-conformance/src/protocol.rs`; `packages/vectors/src/operation.rs` | — | condition bullet | structural implementation | Response validation distinguishes acceptance artifacts from refusal shapes. |
| (`cond:guide15:response-shape`) | §2.7 Evidence entry: response-shape validation is exact | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Protocol-8 response validation joined to admitted evidence. (`result:backlog:bite-0-5-gate-pass`); (`rec:backlog:wave0-entry`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1, 0.5 | — | `packages/target-elements-conformance/src/protocol.rs` | — | condition bullet | structural implementation | The strict revision-8 response contract rejects incompatible shapes and subjects. |
| (`cond:guide15:retained-transcripts`) | §2.7 Evidence entry: canonical reports retain their generic execution transcripts | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Complete admitted generic transcripts retained; Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/vectors/src/maturity_report.rs`; `packages/vectors/src/maturity_continuity_report.rs` | — | condition bullet | structural implementation | Validated reports retain generic execution evidence rather than an unbound success flag. |
| (`cond:guide15:separate-evidence`) | §2.7 Evidence entry: safety, event, continuity, history, accounting, and resources remain separate | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | UNDECIDED | RECORDED: Separate role/producer/consumer contract. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 147: (`cond:guide15:separate-evidence`). Retained consuming obligation: Waves 10–12: Deliver separate validated Phase-7 evidence producers. | 0.7 | 10 through 12 | `packages/vectors/src/maturity_report.rs`; `plans/guides/guide_fifteen_concept.md` | — | condition bullet | new Phase-7 choice | Five inherited report roles are separate; Phase-7 event/accounting report production is not delivered. |
| (`data:guide15:executor-revision`) | §2.8 Repository record: executor protocol revision | (`rule:guide15:repository-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | RECORDED: Protocol 8 and adapter binding from entry; historical revisions preserved. (`rec:backlog:wave0-entry`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.1 | — | `packages/target-elements-conformance/src/protocol.rs` | — | repository-record field | structural implementation | NATIVE_PROTOCOL_SCHEMA = 8; earlier archived runs retain their own historical protocol. |
| (`cond:guide15:public-successor`) | §2.1 Phase-6 entry: public successor reconstruction is implemented | (`gate:guide15:phase6-entry`) | Q11 (`q:guide15:permissionless`) | HOLDS | RECORDED: Independently acquired public recovery evidence and import boundary; Safety. (`result:backlog:bite-0-5-gate-pass`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.5 | — | `packages/vectors/src/maturity_recovery.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | Implemented recovery consumes archived mined readback through a pinned import boundary. |
| (`cond:guide15:public-ash`) | §2.3 Public-declassification entry: public ASH remains constructible by an unrelated process | (`gate:guide15:declassification-entry`) | Q11 (`q:guide15:permissionless`) | UNDECIDED | RECORDED: Public-only fresh-process test contract. (`rec:backlog:wave0-preflight-census`) | Acceptance: Ruling 148: (`task:guide15-exec:disposition-11`) and (`cond:guide15:public-ash`). Retained consuming obligation: Waves 5, 11: Prove the complete public-only fresh-to-compact-to-clear lifecycle. | 0.7 | 5 and 11 | `packages/vectors/src/resource_study.rs`; `plans/phases/04-compact-ash.md` | — | condition bullet | new Phase-7 choice | Public compact construction is inherited; burn-to-clear public construction is not yet evidenced. |
| (`cond:guide15:compact-authorization`) | §2.4 Compact-ASH entry: the inherited operation emits no authorization primitive | (`gate:guide15:compact-entry`) | Q11 (`q:guide15:permissionless`) | HOLDS | RECORDED: Compact authorization absence revalidated. (`rec:backlog:wave0-compact-inherited-closure`) | Acceptance: Ruling 128: (`data:guide15:compact-revision`). Retained consuming obligation: Wave 11: Revalidate permissionless compact execution and separate optional sponsor authorization. | 0.4 | 11 | `packages/tapscript/src/pattern.rs`; `packages/transaction/src/abi.rs` | — | condition bullet | structural implementation | The inherited compact candidate is permissionless; sponsor authorization is a separate envelope. |
| (`cond:guide15:candidate-distinction`) | §2.1 Phase-6 entry: candidate and final states remain distinct | (`gate:guide15:phase6-entry`) | Q12 (`q:guide15:attestation-query`) | HOLDS | RECORDED: Candidate-only, no deployment/release/calibration claims. (`rec:backlog:wave0-preflight-census`); (`sec:phase6:exit-assessment`) | Acceptance: No new position ruling required; inherited evidence retained. No later consuming-wave obligation in this row. | 0.7 | — | `packages/linker/src/state_bundle.rs` | (`sec:phase6:exit-assessment`) | condition bullet | inherited policy | Accepted candidate, release-complete false, no final calibration or deployment claim. |

### Inherited evidence limits and their carriers · `sec:guide15-exec:inherited-limits`

**Fact (Amended Phase-6 exit conditions)** · `fact:guide15-exec:amended-exit`

The accepted Phase-6 exit narrows bypass, wrong-code-subtree, resource, and relation coverage without adding evidence; its other six exit conditions retain their text (`sec:phase6:exit-assessment`).

**Data (Safety census and outstanding partition)** · `data:guide15-exec:safety-census`

At charter, the Safety census was 206 rows, 78 answered and 128 outstanding: 43 first-party required, 35 native-run required, 15 report-layer required, and 35 typed non-answers, then pinned by `the_census_figures_are_unmoved_by_the_binding_readers` in `packages/vectors/src/maturity_evidence.rs` and carried by (`sec:phase6:exit-assessment`). The current census is read at (`result:backlog:close-safety-reconciliation`).

**Data (Admitted answer classes)** · `data:guide15-exec:admitted-answers`

The 78 answers comprise forty first-party discharges, one native acceptance, seven native refusals, sixteen root-history observations, and fourteen public-recovery observations; these are admitted results, not new Wave-0 observations (`sec:phase6:exit-assessment`).

**Data (Relation-link census)** · `data:guide15-exec:relation-census`

Relation links independently stand at 14 every-published, 102 declared, and 90 unlinked; the unlinked partition is 55 without a published relation, nine with several fitting relations, nine whose report is the subject, and seventeen scoped to the whole transaction, as checked by the partition and whole-transaction tests in `packages/vectors/src/maturity_safety.rs` (`sec:phase6:exit-assessment`).

**Fact (Historical Guide-14 conjunction)** · `fact:guide15-exec:historical-conjunction`

Guide 14 §23 has eighteen numbered conjuncts MET, five CONDITIONALLY MET, and its register conjunct MET; the conjunction remains historically NOT PASSED, and the amended phase exit does not alter that verdict (`sec:phase6:gate-record`).

**Inventory (Five inherited conditional conjuncts)** · `reg:guide15-exec:conditional-conjuncts`

Conjunct 5 retains the NUMS assumptions and unoffered wrong-control recipe; conjunct 7 retains the zero relay and block fee floors; conjunct 11 retains the checkpoint-bound edge; conjunct 13 retains archived readback and the pinned import boundary; conjunct 14 retains the composed-transaction region limit, each with its own new disposition owed by bite 0.5 (`sec:phase6:gate-record`).

**Data (Accepted and replay-only resource subjects)** · `data:guide15-exec:resource-subjects`

The Resources matrix admits the 53-byte variable-schedule item with observed weight 2,091; the 86-byte whole-schedule item remains replay-only against the 80-byte relay bound, and `wrong-static-subtree` remains a carried first-party case until its own disposition (`sec:phase6:gate-record`); (`sec:phase6:exit-assessment`).

**Definition (Finality of a typed non-answer reason)** · `def:guide15-exec:non-answer-finality`

Finality is a typed property of a reason: a final non-answer states why no observation can exist for its claim, such as impossibility, a claim outside the admitted contract, a build property, or an answer already held by other admitted evidence. Renaming waiting work does not make it final; no final reason names a missing node, absent provisioning, or an unperformed observable experiment.

**Data (Final and waiting non-answer rows)** · `data:guide15-exec:non-answer-census`

At charter, of the five reason classes in `packages/vectors/src/maturity_safety.rs`, TotalityAnsweredByLandedNonceEvidence had eleven final rows and PropertyOfTheBuildRatherThanAFixture one, because their answers were held by other admitted evidence; AcceptanceAwaitsRelayAdmissibility had nine waiting rows, SponsoredControlRefusedAtConstruction eleven, and SponsorArmAwaitsRegionScoping three. Wave 0 must discharge those twenty-three waiting rows through the observable route; the census reports finality beside bucket counts (`rule:phase6:wave11-rulings`); (`sec:phase6:gate-record`). The current census and the keyed charter reconciliation are read at (`result:backlog:close-safety-reconciliation`).

**Definition (Retirement of an unobservable conjunct residual)** · `def:guide15-exec:retired-residual`

RETIRED means that a conjunct residual is restated as a named assumption or a claim outside the admitted contract, with its argument, while the conjunct's observable remainder stands separately; retirement is forbidden wherever a part is observable. Existing alternative MET readings in the gate record are adopted at their exact scope without erasing the paired observable obligations (`sec:phase6:gate-record`).

**Rule (Observable obligations remain in Wave 0)** · `rule:guide15-exec:observable-retention`

An observable obligation is never retired, transferred, or narrowed out of Wave 0; a ruling removing the only form on which it can be observed must supply the form on which it is observed instead, or Wave 0 does not exit (`rule:roadmap:laws`).

**Assumption (Discrete-log hardness)** · `assum:guide15-exec:discrete-log`

The NUMS internal-key argument assumes discrete-log hardness; a finite target-refusal corpus cannot establish that no usable secret can exist (`rule:state-constructor:internal-key`); `packages/tapscript/src/state_constructor.rs`.

**Assumption (SHA-256 preimage resistance)** · `assum:guide15-exec:sha256-preimage`

The internal-key derivation also assumes SHA-256 preimage resistance, separately from discrete-log hardness, as stated by the typed residual in `packages/tapscript/src/state_constructor.rs` (`rule:state-constructor:internal-key`).

**Fact (First-party internal-key derivation check)** · `fact:guide15-exec:nums-derivation`

`StateInternalKeyPolicy` in `packages/tapscript/src/state_constructor.rs` recomputes the internal key from SHA-256 of the uncompressed generator and refuses a different key; its typed policy is unspendable with the residual discrete-log assumption, not an unconditional impossibility proof (`rule:state-constructor:internal-key`).

**Judgment (Conditional NUMS satisfaction)** · `judg:guide15-exec:nums-standing`

The NUMS part of conjunct 5 is MET under (`assum:guide15-exec:discrete-log`) and (`assum:guide15-exec:sha256-preimage`), with (`fact:guide15-exec:nums-derivation`) as first-party evidence; wrong-control and wrong-static-subtree remain separately dispositioned subjects, consistent with the alternative MET reading in (`sec:phase6:gate-record`).

**Obligation (Independent observation of the exact chain subject)** · `obl:guide15-exec:independent-observation`

Bite 0.5 must independently observe the exact STATE edge and public successor subject required by conjuncts 11 and 13; this is an obligation within the admitted subject and a non-claim beyond it, rather than an inherited limitation that may silently remain unperformed (`sec:phase6:gate-record`).

**Criterion (Observer process and import independence)** · `crit:guide15-exec:observer-independence`

The observer is a separate process with its own pinned import boundary, reads the node's chain view through the node's own interface, and shares no archived readback, fixture state, or process state with the producer; conjuncts 11 and 13 are evaluated against this criterion (`obl:guide15-exec:independent-observation`).

**Caveat (One node supplies the observed chain view)** · `cav:guide15-exec:single-node-scope`

The node of record is the only node: a second node or independent chain view is not claimed, and the observation establishes only the exact subject and environment recorded under (`crit:guide15-exec:observer-independence`).

**Caveat (Production and universal-security exclusions)** · `cav:guide15-exec:inherited-non-claims`

No production key custody, production opening custody, deployment, final calibration, release, universal security claim, or general theorem beyond the finite admitted corpus is inherited or established by this guide (`sec:phase6:exit-assessment`); (`rule:guide15:candidate-only`).

**Caveat (Guide-14 handoff remains carried candidate evidence)** · `cav:guide15-exec:handoff-scope`

Guide 14 §26's handoff remains the carried candidate evidence the accepted record calls it; closing a backlog status cannot expand that scope or turn historical evidence into a new observation (`sec:phase6:exit-assessment`).

**Table (Inherited limits and their closure carriers)** · `tab:guide15-exec:inherited-carriers`

The bindings below retain each limit or assign its exact observable discharge; a closed status alone establishes none of those observations (`rule:guide15-exec:observable-retention`).

| Inherited non-claim or residual | Wave-0 carrier | Required treatment |
|---|---|---|
| Production key and opening custody and deployment | 0.7, `T12-007` | Retain explicit non-claims; disposable fixtures and target evidence introduce no production secret interface. |
| Final calibration and release | 0.6 and 0.7, `T12-006` and `T12-007` | Retain candidate status; performance measurement promotes neither a construction bound nor a release. |
| Independent chain observation absent | 0.5, `T12-005` | Observe the exact subject under (`crit:guide15-exec:observer-independence`); retain limitations beyond it. |
| General theorem beyond the finite admitted corpus | 0.7, `T12-007` | Retain a stated non-claim with its finite corpus boundary, not a missing experiment. |
| NUMS internal-key residual | 0.5, `T12-005` | MET under the two named assumptions and first-party derivation check (`judg:guide15-exec:nums-standing`). |
| Wrong-control recipe and wrong-static-subtree | 0.5, `T12-005` | Identify the exact retained object and boundary; observe every offerable fault and justify any genuinely unobservable residual separately. |
| Positive-floor relayability and sponsored composition | 0.2 and 0.5, `T12-002` and `T12-005` | Complete the refit, ruling on ruling 58, C1, C2, and C3 with actual positive-floor observations. |
| Whole-schedule relay failure | 0.5, `T12-005` | Preserve the 86-byte form's replay-only standing under the 80-byte bound (`data:guide15-exec:resource-subjects`). |

## 3. Wave 0 — Inherited debt closure and preflight · `task:guide15-exec:wave0`

**Specification (Inherited scope without new operations)** · `spec:guide15-exec:wave0-scope`

Wave 0 owns inherited evidence closure and the concept's preparatory work; it introduces no new burn, clear, SOURCE, or MERGE declaration and preserves every historical gate record as evidence of its own scope. (`task:guide15:wave0`).

**Precondition (Provisioned target and provenance)** · `pre:guide15-exec:wave0-target`

The entry obligation is the pinned Elements toolchain on the node of record, provisioned under the dedicated execution account and directory with its provenance record beside the binaries, and its executable provenance and policy configuration retained; provisioning is outside the wave's bites. (`rule:guide15:repository-entry`).

**Requirement (Complete entry record)** · `req:guide15-exec:wave0-entry-record`

The backlog entry record must also identify the clean execution tree, exact target/deployment binding, retained compact and maturity candidates, native execution route, public fixtures, and evidence locations before the wave opens. (`rule:guide15:repository-entry`).

**Rule (A blocking entry finding prevents opening)** · `rule:guide15-exec:wave0-entry-blocker`

Bite 0.1 verifies those obligations and records entry; failure to identify or reproduce the provisioned target prevents opening and cannot be converted into a typed retirement of observable work. (`task:guide15:wave0`).

**Caveat (Future implementation scope and package authority)** · `cav:guide15-exec:implementation-scope`

Each bite below identifies future implementation scope, not additional files edited by this charter; package contracts remain authoritative over the implementation and any interface change must name its smallest sufficient generalization. (`rule:guides:authority`).

**Procedure (Wave-0 dependency graph)** · `alg:guide15-exec:wave0-graph`

Bite 0.1 precedes every other bite; 0.2 precedes 0.3 and the sponsor rows of 0.5; 0.1 precedes 0.4b's native campaign; 0.6 runs concurrently with 0.2–0.5 after entry; 0.7 runs last. Inside 0.5, the ruling revisiting ruling 58 precedes C1, C1 precedes C2, and C2 precedes C3, with the region-refit prerequisites retained under (`rule:phase6:wave11-rulings`). The wave retains its one exit (`gate:guide15-exec:wave0`).

### 0.1 Verify the provisioned target and record entry · `task:guide15-exec:bite-0-1`

**Task (bite-0-1 — Deliverables)** · `task:guide15-exec:bite-0-1-deliverables`

Verify the already provisioned pinned Elements executable, its source and build provenance, target definition, policy, and native adapter protocol, then record the backlog entry and exact inherited subjects. (`rule:guide15:repository-entry`).

**Setting (bite-0-1 — Files and packages)** · `setup:guide15-exec:bite-0-1-files-and-packages`

`packages/target-elements/src/definition.rs`, `packages/target-elements/src/deployment.rs`, `packages/target-elements-conformance/src/protocol.rs`, `packages/vectors/src/maturity_native.rs`, the retained maturity fixtures, and the Phase-7 gate record in `plans/backlog.md`. (`pkg:target-elements:contract`); (`pkg:vectors:contract`).

**Requirement (bite-0-1 — Evidence)** · `req:guide15-exec:bite-0-1-evidence`

Keep the executable/version binding, reviewed V2 definition, protocol revision 8, selected deployment, fee floors, fixture origin, and the exact request/response subjects together; an executable's presence is not a successful target observation. (`rule:guide15:repository-entry`).

**Gate (bite-0-1 — Gate)** · `gate:guide15-exec:bite-0-1-gate`

One exact entry record names all ten repository-record fields, the target validates, the existing accepted control reproduces; otherwise a recorded blocking finding blocks opening Wave 0 and nothing else, and every environment difference from inherited evidence is explicit. (`rule:guide15:repository-entry`).

**Record (bite-0-1 — Closure carrier)** · `rec:guide15-exec:bite-0-1-closure-carrier`

`T12-001` supplies the target prerequisite for `T11-064` and the native portions of `T12-005`; it does not close `T11-064` alone.

**Verification (Reproduce the inherited censuses)** · `verif:guide15-exec:entry-census-check`

The 206-row census and 14 / 102 / 90 relation-link census are reproduced from their typed sources before any evidence changes; mismatches become findings rather than edits to the expected historical counts. (`sec:phase6:exit-assessment`).

### 0.2 Refit operation regions before composition · `task:guide15-exec:bite-0-2`

**Task (bite-0-2 — Deliverables)** · `task:guide15-exec:bite-0-2-deliverables`

Make transaction-side relations quantify over the operation's claimed region, include region membership in observations, and represent a composed transaction as the union of its operations' regions. (`T11-064`).

**Setting (bite-0-2 — Files and packages)** · `setup:guide15-exec:bite-0-2-files-and-packages`

`packages/realization/src/`, `packages/compiler/src/sponsor_region.rs`, `packages/model/src/`, the affected declaration and conformance tests, and `plans/backlog.md`; contract changes belong in `plans/packages/realization.md` and `plans/packages/compiler.md` if required. (`pkg:realization:contract`); (`pkg:compiler:contract`).

**Inventory (Relations in the region refit)** · `reg:guide15-exec:region-relations`

The five relations are `AllowedObjectFamilies`, `SponsorIsolation`, `CanonicalDeltaPolicy`, `OpenFlowPolicy`, and `SponsorEnvelopeMultiplicity`; each is re-scoped or retired with an exact reason (`T11-064`).

**Definition (Refit identities and other model-scope identities)** · `def:guide15-exec:region-identities`

The six identities belonging to those relations are distinguished from the four other MODEL-SCOPE identities whose carriers disappeared with the retired partition fragment; a refit does not discharge the latter by association. (`T11-064`).

**Gate (bite-0-2 — Gate)** · `gate:guide15-exec:bite-0-2-gate`

An announcement beside a second family's existing operation is accepted by model conformance, a foreign object inside the announcement's own region is refused, and each changed relation retains a checked scope and coverage mapping. The seventeen whole-transaction-scoped unlinked rows must move to exact operation-region relations or justified non-relation dispositions; recount them separately against the inherited 14 / 102 / 90 partition (`data:guide15-exec:relation-census`).

**Record (bite-0-2 — Closure carrier)** · `rec:guide15-exec:bite-0-2-closure-carrier`

`T12-002` closes the region-refit portion of `T11-064`, supplies §23 conjunct 14's structural evidence, and leaves its positive-floor and sponsored-form obligations explicitly assigned to bite 0.5.

**Justification (Existing operations exercise the refit)** · `just:guide15-exec:existing-composition`

The gate uses existing operations to exercise composition before any new operation is delivered; it cannot depend on the second exemplar's future declarations. (`T11-064`).

### 0.3 Complete the closure-exact audit · `task:guide15-exec:bite-0-3`

**Task (bite-0-3 — Deliverables)** · `task:guide15-exec:bite-0-3-deliverables`

Audit compact and live generations against closure exactness, retain only checks needed by their own contracts, and remove each surplus check with its revised census (`T11-063`).

**Setting (bite-0-3 — Files and packages)** · `setup:guide15-exec:bite-0-3-files-and-packages`

`packages/realization/src/declarations/`, `packages/compiler/src/`, `packages/tapscript/src/`, `packages/linker/src/`, their compact/live tests, and the audit disposition in `plans/backlog.md`. (`pkg:realization:contract`); (`pkg:compiler:contract`); (`pkg:tapscript:contract`); (`pkg:linker:contract`).

**Justification (Closure checks follow their own contracts)** · `just:guide15-exec:closure-checks`

Compact's fixed-batch aggregate and live transfer's independent output-count axis remain justified where their laws require them; sponsor-region checks need their own justification rather than inheriting necessity from unrelated count checks. (`T11-063`).

**Gate (bite-0-3 — Gate)** · `gate:guide15-exec:bite-0-3-gate`

Every retained check names its closure need, every surplus check is absent from emitted and linked artifacts, relation/carrier/site counts reconcile, and compact/live positive and focused negative regressions pass. (`T11-063`).

**Record (bite-0-3 — Closure carrier)** · `rec:guide15-exec:bite-0-3-closure-carrier`

`T12-003` closes `T11-063` with an explicit before/after check census and pointers to the tests that distinguish required closure from surplus restriction.

**Obligation (Removal propagates to every carrier)** · `obl:guide15-exec:closure-impact`

Any removed relation or changed interface records the effect on compiler placement, emitted schedules, linker obligations, ABI consumers, and evidence so that an absent check does not leave a fictitious discharge. (`T11-063`).

### 0.4 Close the compact negative half and guide gaps · `task:guide15-exec:bite-0-4`

**Task (bite-0-4 — Deliverables)** · `task:guide15-exec:bite-0-4-deliverables`

Implement the three rules stated below from `plans/guides/guide_thirteen_feature_requests.md`, make the required negative subjects constructible at their declared boundaries, and complete the 74-row compact negative closure under the current 48 / 20 / 6 partition, retaining 71 outstanding of 72 and 48 / 18 / 6 as before-figures at their original evidence scope (`T4-009`).

**Setting (bite-0-4 — Files and packages)** · `setup:guide15-exec:bite-0-4-files-and-packages`

`packages/vectors/src/`, compact fixtures and evidence plans, `packages/transaction/src/`, `packages/linker/src/`, any required target-generic conformance support, and the compact gate disposition in `plans/backlog.md`; the bounded extension for the remaining compact validation entries, tests and records is specified by ruling 139 (`rule:phase7:wave0-rulings`). (`pkg:linker:contract`); (`pkg:transaction:contract`); (`pkg:vectors:contract`).

**Caveat (Compact positive evidence cannot answer negatives)** · `cav:guide15-exec:compact-negative-scope`

The admitted positive compact result remains at its Phase-4 scope; increasing the number of first-party tests cannot replace the missing mutation, funding, boundary, or native evidence that each row requires. (`T4-009`).

**Task (Part 0.4a — Three rules for the negative half)** · `task:guide15-exec:bite-0-4a`

Carry the three guide-level rules below into the compact negative work; they answer the guide gaps in `plans/guides/guide_thirteen_feature_requests.md` while keeping the 74-row campaign with its 48 / 20 / 6 partition separate, retaining 72 and 48 / 18 / 6 as before-figures (`T4-009`).

**Rule (A negative row names relation, mutation class, and boundary)** · `rule:guide15-exec:compact-negative-row`

Every compact negative row names the relation it tests, its mutation class, and the boundary at which its refusal answers that relation; a refusal at another boundary is retained as evidence of that boundary only (`pkg:vectors:contract`); `plans/guides/guide_thirteen_feature_requests.md`. A row formally closed under ruling 136 has an expected boundary that names its inherited manifest classification, and its answer is its typed formal-closure reason (`rule:phase7:wave0-rulings`).

**Rule (First-party discharge has an explicit admitted condition)** · `rule:guide15-exec:compact-first-party`

Each of the twenty first-party rows in the current 74 = 48 + 20 + 6 census, with 72 = 48 + 18 + 6 retained as the before-figure, names a retained comparison subject and checked relation sufficient to discharge its admitted claim, or an explicit out-of-scope declaration with its argument; an observable obligation cannot be removed by that declaration (`rule:guide15-exec:observable-retention`); `plans/guides/guide_thirteen_feature_requests.md`.

**Rule (ABI validation is a named negative entry point)** · `rule:guide15-exec:compact-abi-entry`

The compact negative plan names the typed ABI-validation entry point and exact malformed subject for each ABI-boundary row, makes that subject constructible without weakening earlier validators, and admits refusal only at the named boundary (`pkg:transaction:contract`); `plans/guides/guide_thirteen_feature_requests.md`.

**Task (Part 0.4b — Pinned-target compact campaign)** · `task:guide15-exec:bite-0-4b`

Dispose all 74 rows under the current 48 / 20 / 6 partition: 48 target-executable, twenty first-party, and six external-report rows, retaining 72 / 48 / 18 / 6 and the one historical native discharge leaving 47 native and 71 overall outstanding as before-figures at their original evidence scope. Run the native half only after 0.1, with exact funded subjects and accepted controls on the pinned target; report observations separately from final non-observation dispositions (`T4-009`); `plans/guides/guide_thirteen_feature_requests.md`.

**Gate (bite-0-4 — Gate)** · `gate:guide15-exec:bite-0-4-gate`

All 74 rows under the current 48 / 20 / 6 partition, with 72 / 48 / 18 / 6 retained as before-figures, have exact subjects and final dispositions, with observed refusals at matching boundaries or typed formal closure reasons; the three guide requests each have an implementing artifact and passing verification. (`T4-009`).

**Record (bite-0-4 — Closure carrier)** · `rec:guide15-exec:bite-0-4-closure-carrier`

(`T12-004`) closes the native half at its recorded 48-requirement scope; (`T4-009`) closes its remaining obligations through the separate evidence at (`rec:backlog:wave0-compact-inherited-closure`), with 51 observed and 23 formally closed requirements. (`T12-007`) still consolidates those records under (`rec:guide15-exec:bite-0-7-closure-carrier`); no closure claims 74 observed refusals, as none claimed 72.

**Requirement (Current compact pipeline regressions)** · `req:guide15-exec:compact-regressions`

The compact compiler plan, patterns, linker, ABI, vectors, and resource evidence must remain green on the selected current candidate; any changed count is reported beside the inherited figure. (`T4-009`).

### 0.5 Close Safety dispositions and the five conjuncts · `task:guide15-exec:bite-0-5`

**Task (bite-0-5 — Deliverables)** · `task:guide15-exec:bite-0-5-deliverables`

Dispose all 128 outstanding Safety rows, preserve the 78 admitted answers unless evidence requires an explicit correction, and record a final standing for all 206 rows and every relation link. (`sec:phase6:gate-record`).

**Setting (bite-0-5 — Files and packages)** · `setup:guide15-exec:bite-0-5-files-and-packages`

`packages/vectors/src/maturity_safety.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_first_party.rs`, `packages/vectors/src/maturity_mutants.rs`, the native corpus and report modules, `packages/target-elements-conformance/src/`, and `plans/backlog.md`. (`pkg:vectors:contract`). The ABI-entry routes add `transaction::validate_maturity_abi_classification`, `transaction::validate_maturity_abi_roles` and the typed request-cycle decoder in `packages/transaction/src/state_abi.rs` and `packages/transaction/src/state_request.rs`, with exports in `packages/transaction/src/lib.rs` and the three-way outcome in `packages/vectors/src/abi_validation.rs`, and the validator-side nonce comparison in `packages/tapscript/src/state_constructor.rs`, each with its tests; each is a new entry beside its owner and broadens no existing production validator (`pkg:transaction:contract`); (`pkg:tapscript:contract`). The residual routes also touch `packages/transaction/src/state_confidential.rs`, `packages/transaction/src/state_construct.rs`, `packages/transaction/src/state_finalize.rs` and `packages/transaction/src/state_signing.rs` for the balanced confidential form, the continuity, recovery, corpus and evidence modules in `packages/vectors/src/`, `scripts/elements-native-executor.py` and its capture scripts, and the admitted fixture directories under `packages/vectors/fixtures/`. The adapter request comparisons add `scripts/test-executor-request-comparisons.py`, its root and script Meson registration, `packages/vectors/src/maturity_adapter.rs`, its module and source-census registration, and the embedded `packages/vectors/fixtures/adapter-request-comparisons/COMPARISONS` fixture; the existing first-party, evidence, index and report modules admit those exact request-side comparisons without changing an adapter validator (`rec:backlog:wave0-adapter-comparisons`).

**Setting (Native mutation staging precedent)** · `setup:guide15-exec:mutation-staging`

The eight-row design in `plans/drafts/wave13-native-mutant-ceremony-study.md` supplies staging and boundary reasoning; its older standings do not replace the seven admitted refusals and unoffered recipe recorded at the accepted Phase-6 closeout. (`sec:phase6:gate-record`).

**Precondition (Sponsored obligations follow the region refit and ruling)** · `pre:guide15-exec:sponsored-order`

Bite 0.5 obtains a numbered Phase-7 ruling revisiting ruling 58 before C1, with the refit of 0.2 complete; C1 supplies the sponsored STATE builder, C2 performs its sponsored native ceremony on the provisioned target, and C3 retypes the sponsor rows after their observations. Eleven construction-refused rows depend on C1 and three region-scoping rows depend on the refit; none can be transferred out of Wave 0 (`rule:phase6:wave11-rulings`); (`rule:guide15-exec:observable-retention`).

**Criterion (Independent edge and successor observations)** · `crit:guide15-exec:conjunct-observation`

Conjuncts 11 and 13 require the separate process, pinned import boundary, node-interface reading, and absence of shared archive, fixture, or process state stated by (`crit:guide15-exec:observer-independence`); equal final cursors cannot replace equal edge histories, and no second chain view is claimed.

**Task (Part 0.5 — 43 first-party rows)** · `task:guide15-exec:bite-0-5-first-party`

The 43 first-party-required rows each need their own retained comparison object and exact relation, with final out-of-contract claims separated from observable work (`sec:phase6:gate-record`); (`rule:guide15-exec:observable-retention`). The fifteen §16.9 classification rows, the six §16.11 role rows and the §16.2 cycle-encoding row are compared at `transaction::validate_maturity_abi_classification`, `transaction::validate_maturity_abi_roles` and the typed request-cycle decoder, and the §16.3 noncanonical-nonce row at the validator-side comparison of `CandidateStateConstructor` (`rule:guide13-exec:first-party-negatives`).

**Gate (bite-0-5 — First-party gate)** · `gate:guide15-exec:bite-0-5-first-party-gate`

Each of the 43 required rows receives a retained comparison object and exact checked relation or a typed reason why its claim is outside the admitted contract; copied success flags cannot answer a row. (`sec:phase6:gate-record`).

**Task (Part 0.5 — 35 native rows)** · `task:guide15-exec:bite-0-5-native`

The 35 native-run-required rows each need exact subjects, accepted controls, and observations at their declared boundaries on the pinned target (`sec:phase6:gate-record`); (`rule:guide15-exec:observable-retention`).

**Gate (bite-0-5 — Native gate)** · `gate:guide15-exec:bite-0-5-native-gate`

Each of the 35 required rows is exercised on the pinned target at its declared boundary where observable, with accepted controls and complete subjects; infrastructure failure and a refusal at another boundary answer nothing. (`sec:phase6:gate-record`).

**Task (Part 0.5 — 15 report-layer rows)** · `task:guide15-exec:bite-0-5-report`

The fifteen report-layer-required rows each need discharge through their validated consumer or a precise final semantic reason, without claiming unobserved target behavior (`sec:phase6:gate-record`); (`rule:guide15-exec:observable-retention`).

**Gate (bite-0-5 — Report gate)** · `gate:guide15-exec:bite-0-5-report-gate`

Each of the 15 report-layer-required rows is discharged by its validated report consumer or retyped with a precise semantic reason; ability to render a report is not target evidence. (`sec:phase6:gate-record`).

**Task (Part 0.5 — 35 non-answers and relation links)** · `task:guide15-exec:bite-0-5-non-answer-relations`

Reconcile all 35 non-answers by their typed finality and all relation links against 14 / 102 / 90, preserving the distinction between the twelve final and twenty-three waiting reason rows (`sec:phase6:gate-record`); (`rule:guide15-exec:observable-retention`).

**Gate (bite-0-5 — Non-answer gate)** · `gate:guide15-exec:bite-0-5-non-answer-gate`

Each of the 35 typed non-answers receives a typed finality disposition: retain the twelve final rows on their other admitted evidence and discharge the twenty-three waiting rows through observation. No final reason names a missing node, absent provisioning, or unperformed observable experiment; count finality beside the buckets (`def:guide15-exec:non-answer-finality`); (`data:guide15-exec:non-answer-census`).

**Gate (bite-0-5 — Relation gate)** · `gate:guide15-exec:bite-0-5-relation-gate`

Recount every-published, declared, and unlinked relations against 14 / 102 / 90, assigning a declared relation or a justified non-relation disposition to every previously unlinked row. (`sec:phase6:gate-record`).

**Task (Part 0.5 — Five conditional conjuncts)** · `task:guide15-exec:bite-0-5-conjuncts`

Dispose conjuncts 5, 7, 11, 13, and 14 separately, preserving the historical table and the observable remainder of each conjunct (`sec:phase6:gate-record`); (`rule:guide15-exec:observable-retention`).

**Gate (bite-0-5 — Conjunct 5)** · `gate:guide15-exec:bite-0-5-conjunct-5`

Preserve admitted native refusals, disposition the exact wrong-control recipe and `wrong-static-subtree`, and record the NUMS part as MET under the two typed assumptions and first-party derivation check, rather than treating another rejection as an impossibility proof (`judg:guide15-exec:nums-standing`).

**Gate (bite-0-5 — Conjunct 7)** · `gate:guide15-exec:bite-0-5-conjunct-7`

Observe positive-floor relayability and the selected sponsored forms on the pinned target, with exact fees, policy, subjects, and acceptance/readback evidence; zero-floor archived acceptance cannot supply this result. (`sec:phase6:gate-record`).

**Gate (bite-0-5 — Conjunct 11)** · `gate:guide15-exec:bite-0-5-conjunct-11`

Recover the exact STATE edge from independently observed transactions with checkpoint and environment bindings; do not substitute equal final cursors for equal edge histories. (`sec:phase6:gate-record`).

**Gate (bite-0-5 — Conjunct 13)** · `gate:guide15-exec:bite-0-5-conjunct-13`

Have an unrelated process reconstruct the successor from independently observed public chain data with its declared import boundary; keep archival availability and broader deployment claims separate. (`sec:phase6:gate-record`).

**Gate (bite-0-5 — Conjunct 14)** · `gate:guide15-exec:bite-0-5-conjunct-14`

Admit the region-refit's composed observation and foreign-object refusal, then bind the observed sponsored composition to that model scope and its exact operation regions. (`sec:phase6:gate-record`).

**Gate (bite-0-5 — Gate)** · `gate:guide15-exec:bite-0-5-gate`

Every one of the five conjuncts receives a part-by-part standing: observable parts are MET by performed observations, and only unobservable residuals may be RETIRED under (`def:guide15-exec:retired-residual`). Adopt the gate record's existing alternative MET readings where available, retain their paired observable obligations, and preserve the historical Phase-6 table beside the new disposition table.

**Record (bite-0-5 — Closure carrier)** · `rec:guide15-exec:bite-0-5-closure-carrier`

`T12-005` closes the remaining `T11-064` obligations, including Wave 11's ruling on ruling 58, C1, C2, C3, and positive-floor conditions (`rule:phase6:wave11-rulings`), and records the Safety and conjunct closure that the Wave-0 gate requires.

**Bound (Whole-schedule relay exclusion)** · `bound:guide15-exec:whole-schedule-limit`

The 86-byte whole-schedule item remains replay-only under the 80-byte relay limit; use of an admitted 53-byte variable schedule is a precisely bound form choice, not a claim that the larger item relays. (`sec:phase6:gate-record`).

### 0.6 Complete the performance measurement register · `task:guide15-exec:bite-0-6`

**Task (bite-0-6 — Deliverables)** · `task:guide15-exec:bite-0-6-deliverables`

Complete `T11-110` through the measurement and reproduction obligations in `plans/research/test-performance.md`, retaining observations and tested remedies without inventing absent timings.

**Setting (bite-0-6 — Files and packages)** · `setup:guide15-exec:bite-0-6-files-and-packages`

`plans/research/test-performance.md`, the gate timing readers and scripts it names, affected test modules only when an evidenced remedy requires them, and the closing backlog record. (`sec:test-performance:tracking`).

**Requirement (bite-0-6 — Evidence)** · `req:guide15-exec:bite-0-6-evidence`

Record toolchain, node resources, cache classification, full-gate wall, suite walls, achieved parallelism, longest suites and tests, and unavailable dimensions separately (`sec:test-performance:tracking`).

**Gate (bite-0-6 — Gate)** · `gate:guide15-exec:bite-0-6-gate`

The append-only dated register contains at least two measurements, and the first proposed swiftness bite is ruled and landed with a before/after pair; it already holds three observations, two cold and one warm, with separate suite walls. Retain required cold/warm and isolated-suite dimensions or a justified typed reason for an inapplicable dimension; every adopted remedy preserves tested behavior (`T11-110`); (`tab:test-performance:register`).

**Record (bite-0-6 — Closure carrier)** · `rec:guide15-exec:bite-0-6-closure-carrier`

`T12-006` closes `T11-110` as DONE on that row's own criterion, including its ruled and landed first swiftness bite; an IN PROGRESS status cannot pass Wave 0 (`T11-110`).

**Caveat (Performance is not calibration)** · `cav:guide15-exec:performance-scope`

A performance result does not establish target calibration, and reducing verification scope to obtain a shorter wall is not a measured improvement under an unchanged gate. (`T11-110`).

### 0.7 Close preflight and publish the baseline · `task:guide15-exec:bite-0-7`

**Task (bite-0-7 — Deliverables)** · `task:guide15-exec:bite-0-7-deliverables`

Complete the semantic censuses, ASH representation, private-burn route, constructor disposition, burn-record carrier and value rule, clear assignment, arithmetic assessment, generation policy, evidence boundary, and clean-tree record inherited from (`task:guide15:wave0`).

**Setting (bite-0-7 — Files and packages)** · `setup:guide15-exec:bite-0-7-files-and-packages`

`plans/backlog.md`, the execution register, affected package contracts and research notes, `packages/architecture/src/`, `packages/model/src/`, `packages/realization/src/`, and focused validation tests where necessary to establish inherited law. (`pkg:realization:contract`).

**Invariant (Stored and derived clear fields)** · `inv:guide15-exec:clear-census`

The clear census records `y_l` as stored and Y as derived; it must not turn the two mathematical equations into two independent stored-field writes (`q:guide15:clear-assignment`).

**Specification (Carrier disposition fields)** · `spec:guide15-exec:carrier-fields`

The carrier disposition specifies canonical bytes, ordering, record identity, public recovery, association with fresh ASH, and value/cardinality law; Wave-4 OP_RETURN measurements can challenge the selected form but cannot backdate its proof, and a size or policy refusal reopens the selection by a Wave-4 ruling before declarations (`cond:guide15-exec:carrier-reopening`).

**Specification (Generation disposition fields)** · `spec:guide15-exec:generation-fields`

The generation disposition names the exact maturity-only predecessor and new maturity-plus-clear static subtree, admits synthetic origin or an explicit migration prerequisite, and retains public recovery without generation crossing. (`gate:guide15:preflight`).

**Gate (bite-0-7 — Gate)** · `gate:guide15-exec:bite-0-7-gate`

All twelve questions have explicit dispositions, all 65 register entries retain finding/evidence/execution distinctions, and every conjunct of (`gate:guide15:preflight`) is passed under (`def:guide15-exec:semantic-entry-split`), with acceptance records under (`req:guide15-exec:position-acceptance`).

**Record (bite-0-7 — Closure carrier)** · `rec:guide15-exec:bite-0-7-closure-carrier`

`T12-007` consolidates the closure records of `T11-064`, `T11-063`, `T4-009`, and `T11-110`; it cannot close one merely because its dependent preparatory row is DONE.

**Gate (Complete clean-tree baseline)** · `gate:guide15-exec:baseline-repository`

The full repository gate, including the required document and lint surfaces, runs on the completed clean tree; focused tests alone cannot establish this baseline. (`gate:guide15:batch`).

### Wave-0 exit · `gate:guide15-exec:wave0`

**Condition (wave0-exit condition 1)** · `cond:guide15-exec:wave0-exit-condition-1`

- The four backlog rows `T11-064`, `T11-063`, `T4-009`, and `T11-110` meet their own closure criteria with retained evidence; typed formal closure applies only to genuinely unobservable claims, and `T11-110` requires its first swiftness bite ruled and landed.

**Condition (wave0-exit condition 2)** · `cond:guide15-exec:wave0-exit-condition-2`

- All 206 Safety rows are answered or retyped with exact reasons: no final reason names a missing node, absent provisioning, or an unperformed observable experiment; finality is the typed property in (`def:guide15-exec:non-answer-finality`), and answered, retired, waiting, and other final dispositions are counted separately.

**Condition (wave0-exit condition 3)** · `cond:guide15-exec:wave0-exit-condition-3`

- The inherited 78 / 128 split and 43 / 35 / 15 / 35 outstanding partition have a complete before/after reconciliation with no dropped row. (`sec:phase6:exit-assessment`).

**Condition (wave0-exit condition 4)** · `cond:guide15-exec:wave0-exit-condition-4`

- The relation-link census has a complete before/after reconciliation against 14 / 102 / 90, including reasons for each remaining non-link. (`sec:phase6:exit-assessment`).

**Condition (wave0-exit condition 5)** · `cond:guide15-exec:wave0-exit-condition-5`

- The five §23 conjuncts 5, 7, 11, 13, and 14 are individually MET or have only genuinely unobservable residuals RETIRED with named assumptions or non-claims and reasons; every observable remainder is performed, preserving the historical NOT PASSED verdict at its original scope (`def:guide15-exec:retired-residual`).

**Condition (wave0-exit condition 6)** · `cond:guide15-exec:wave0-exit-condition-6`

- All twelve preflight questions are dispositioned, all 65 entry rows have explicit execution exits, and (`gate:guide15:preflight`) is passed.

**Condition (wave0-exit condition 7)** · `cond:guide15-exec:wave0-exit-condition-7`

- Every position disposition has a retained numbered acceptance ruling on the Phase-7 card, covering the seven assigned dispositions and nine position-bearing UNDECIDED entries (`req:guide15-exec:position-acceptance`).

**Condition (wave0-exit condition 8)** · `cond:guide15-exec:wave0-exit-condition-8`

- The full repository gate is green on the completed clean tree, with exact commands, results, durations, and reproducible evidence locations. (`gate:guide15:batch`).

**Condition (wave0-exit condition 9)** · `cond:guide15-exec:wave0-exit-condition-9`

- These figures, the pinned target and policy, the exact candidates, and retained non-claims are recorded as the baseline for every later wave. (`sec:phase6:exit-assessment`).

**Proposal (wave0-exit — Suggested commit)** · `proposal:guide15-exec:wave0-exit-suggested-commit`

`plans: record inherited debt closure and the Phase-7 preflight baseline`. (`gate:guide15:batch`).

## 4. Wave 1 — The Keyed Provenance Covenant · `task:guide15-exec:wave1`

**Precondition (Wave-1 entry and compilation scope)** · `pre:guide15-exec:wave1-entry`

Wave 1 opens only on the Wave-0 exit. The contract of record is the Keyed Provenance Covenant paper, beginning with its reading interface (`[A-sec:kpc:interface]`), followed by its objects and whole covenant (`[A-sec:kpc:model]`), operations (`[A-sec:kpc:operations]`) and invariants (`[A-sec:kpc:invariants]`). The wave carries that specification through declaration, analysis, emission, linking, construction and translation-validation evidence; publication of the specification alone discharges none of this implementation work.

**Specification (Exactly two covenant clauses)** · `spec:guide15-exec:whole-covenant`

Born first requires that the spent output was output 0 of its creating transaction. Continue first requires that the spending transaction's output 0 bears the same keyed covenant. These are the entire covenant; both must be checked for every consumed covenant output (`[A-def:kpc-model:first-output]`). The key is fixed at generation; incompatible systems coexist on one ledger, while an event consuming handles of different keys cannot satisfy their disjoint continuations (`[A-def:kpc-model:keyed-membership]`); (`[A-thm:kpc-invariants:key-isolation]`).

**Definition (SOURCE, MERGE and record sets)** · `def:guide15-exec:record-sets`

An event creating an output-0 handle and consuming no handle of its key is a SOURCE: the handle itself is its record and receives the singleton set. An event consuming handles is a MERGE: output 0 receives their union and introduces no record, whatever else the event creates. An event without a handle has no set to define. A later covenant-shaped output is not a handle and is never spendable; there is at most one handle of a key born per event (`[A-op:kpc-operations:source]`); (`[A-op:kpc-operations:merge]`); (`[A-def:kpc-operations:classification]`); (`[A-prop:kpc-model:single-birth]`).

**Rule (Reading and acceptance)** · `rule:guide15-exec:record-reading`

Membership is in the dependency-derived set of the read handle; no designated head or terminal selects the system. Each handle has a unique continuation, and distinct live handles have disjoint record sets. Acceptance of amount, attribution and eligibility belongs to a reader's published rule and changes no record set. A fixed predicate on immutable SOURCE contents preserves accepted-set growth along a path; arbitrary reader policies need not do so. Inclusion, retrieval and finality are named evidence assumptions, and a final reading requires finality outside the covenant (`[A-thm:kpc-invariants:unique-successor]`); (`[A-cor:kpc-interface:path-reading]`); (`[A-rem:kpc-interface:record-evidence]`); (`[A-sec:kpc:interface]`).

**Specification (Covenant exclusions)** · `spec:guide15-exec:covenant-exclusions`

For the legacy edition, there is no destruction output, burn requirement, counter, accumulator, magnitude or depth inside the system, and no designated head or terminal (`[A-sec:kpc:operations]`). For the legacy edition, other target-valid inputs and outputs remain open; no value-preservation, authorization or record-acceptance relation is added to the covenant (`[A-def:kpc-model:first-output]`); (`[A-sec:kpc:operations]`). Legacy extension may release carried value; later-position covenant-shaped outputs are not handles. The fixed-form edition retains one constant explicit deposit, excludes wrong-pair/form payments from membership and releases `(n−1)v` on MERGE of n members without constraining its destination; it does no input/output resource accounting and adds no authorization (`rule:phase7:fixed-form-rulings`).

**Inventory (Wave-1 recount)** · `reg:guide15-exec:wave1-recount`

The wave has seven bites, 1.0–1.6, and two operation declarations, SOURCE and MERGE. The former thirteen destruction-study requirement standings are replaced by eight delivery requirements: (1) born first and single birth, (2) continue first and unique continuation, (3) fixed-key separation, (4) SOURCE record identity, (5) MERGE recurrence and disjoint growth, (6) faithful membership reconstruction, (7) reader-side acceptance, and (8) explicitly published evidence assumptions including finality. Only requirements 1 and 2 are covenant clauses; the other six describe the system or its reading boundary. These requirements cite the paper above and its verification appendix (`[A-app:kpc:verification]`).

**Record (Study decision dispositions)** · `rec:guide15-exec:study-dispositions`

The old nine study decisions reduce to six realization decisions: semantic home (1.1), incidental funding and construction input bound (1.0), authenticated recurrence strategy (1.3), reader rule and evidence/report format (1.5). The old value-preservation choice is fixed by the whole covenant, genesis designation is retired because there is no designated head, and roadmap admission is already recorded by the phase change statement. The semantic-home decision still compares three options: another declaration set, a semantic profile, or a separate realization crate with a shared typed compiler input. The six decisions select realization/evidence details and cannot revise the paper's two clauses (`sec:phase7:change-statement`); (`[A-def:kpc-model:first-output]`).

**Record (Evidence and measurement recount)** · `rec:guide15-exec:evidence-recount`

Six evidence families remain, with new subjects: birth, continuation/key separation, record recurrence, reading/evidence assumptions, reader acceptance, and resources. Six target measurements remain, now (a) spent-output birth position, (b) authenticated same-key output-0 script comparison, (c) keyed constructor and spend-path closure, (d) composed clause costs, (e) transaction standardness/funding, and (f) scaling over the admitted input range. The old pair, designated-ancestry, replacement-head and bounded-consumer subjects are retired. The four old destruction-study trap arguments are retired as contract obligations; the new families test the paper's own statements. No inherited count certifies the new subjects (`[A-app:kpc:verification]`).

**Rule (Wave-1 failure route)** · `rule:guide15-exec:wave1-failure`

If this contract cannot pass the Wave-1 exit, Phase 7 stops until it can; adding a covenant clause or replacing a missing observation with a paper proof is not a failure route (`rule:roadmap:laws`); (`rule:guide15-exec:second-contract-dependency`).

### 1.0 Measure the two clauses' target obligations · `task:guide15-exec:bite-1-0`

**Task (bite-1-0 — Deliverables)** · `task:guide15-exec:bite-1-0-deliverables`

Run six isolated experiments on the Wave-0 pinned Elements target before choosing stable declarations or ABI bounds, after or alongside 1.1's boundary test. Each record binds exact bytes, target, policy, accepted controls, one-coordinate negative subjects, observation boundary, outcome and absent dimensions. Prototype observations are provisional: 1.6 repeats them on final compiler artifacts (`rule:roadmap:parallel`).

**Experiment (Measurement a — born first)** · `expt:guide15-exec:measurement-a`

Establish how an input authenticates and reads the output position of its own spent output in its creating transaction. Spend an output-0 control and refuse an otherwise matching later-output subject; distinguish the spent-output position from the current input index. Missing authentic position evidence blocks the realization, even if successor equality works (`[A-def:kpc-model:first-output]`).

**Experiment (Measurement b — continue first)** · `expt:guide15-exec:measurement-b`

Establish how output 0's complete script, including type/version and the fixed key, is compared with the spending input's authenticated covenant program. Refuse wrong-key, wrong-program and displaced-successor subjects beside a valid control. An unauthenticated caller-supplied comparison script is not the reference (`[A-def:kpc-model:first-output]`); (`[A-thm:kpc-invariants:key-isolation]`).

**Experiment (Measurement c — constructor closure)** · `expt:guide15-exec:measurement-c`

Generate two keyed covenants and establish public reconstruction, key separation, authenticated control paths and the absence of alternative spend paths bypassing either clause. Retain conditional NUMS assumptions where applicable; finite negative observations do not prove discrete-log impossibility (`judg:guide15-exec:nums-standing`).

**Experiment (Measurement d — composed clause cost)** · `expt:guide15-exec:measurement-d`

Execute both clauses together, recording program bytes, witness items, control depth, stack peaks and available execution costs. Compare predicted and observed resources, retaining mismatches as findings and unavailable dimensions as absent; separate per-input cost from fixed constructor overhead (`[A-sec:kpc:limitations]`).

**Experiment (Measurement e — transaction standardness)** · `expt:guide15-exec:measurement-e`

The reviewed target's `ExplicitZeroValueRule::SpendableRefused` refuses explicit zero-valued spendable outputs; the experiment measures the positive relay floor rather than reopening consensus positivity. Measure SOURCE and MERGE standardness with exact funding, fee outputs and policy. Check the funded accepted control and boundary cases around the relay minimum, and any proposed unrelated confidential outputs with their balance proofs. Consensus and relay outcomes remain distinct. Incidental funding and fee floors belong to the realization, never to record identity or covenant acceptance.

**Experiment (Measurement f — scaling and construction bound)** · `expt:guide15-exec:measurement-f`

Measure complete SOURCE and MERGE transaction weight and execution costs over the proposed input range, with worst-case witnesses and required fee funding. Select a measured construction bound and funding policy provisionally until 1.6; neither creates a covenant count or value comparison. The system's derived record set is not carried in the witness or stored in the handle.

**Setting (bite-1-0 — Files and packages)** · `setup:guide15-exec:bite-1-0-files-and-packages`

Use `packages/vectors/` for subjects and evidence, `packages/target-elements/` for reviewed capability/form assessments, and `packages/tapscript/` for isolated clause and constructor experiments; retain measurements in the research home below and dispositions in the wave's backlog gate record (`pkg:vectors:contract`); (`pkg:target-elements:contract`); (`pkg:tapscript:contract`).

**Gate (bite-1-0 — Gate)** · `gate:guide15-exec:bite-1-0-gate`

All six measurement records have complete subject bindings and observed controls, with the funding and input-bound decisions stated; an unavailable primitive or unresolved form blocks its consuming bite. No paper-level realization claim follows from these prototype results.

**Proposal (bite-1-0 — Suggested commit)** · `proposal:guide15-exec:bite-1-0-suggested-commit`

`test(vectors): measure Keyed Provenance Covenant target obligations`.

### 1.1 Choose the realization home and declare the contract · `task:guide15-exec:bite-1-1`

**Task (bite-1-1 — Deliverables)** · `task:guide15-exec:bite-1-1-deliverables`

Test the architecture-to-realization boundary early; compare the three semantic-home options and their costs, select the smallest justified change, and validate typed SOURCE and MERGE declarations welded to the paper's objects and laws. SOURCE classification and its singleton record belong to semantic/evidence relations, with no fictitious covenant input. MERGE declares both clauses for each covenant input and a reader-derived union, not an emitted history traversal (`[A-def:kpc-operations:classification]`).

**Setting (bite-1-1 — Files and packages)** · `setup:guide15-exec:bite-1-1-files-and-packages`

Change `packages/architecture/src/`, `packages/realization/src/`, their contracts and tests, and only the shared typed interface required by the selected semantic home. Keep model conformance independent of compiler-facing declarations (`pkg:realization:contract`); `packages/model/README.md`.

**Gate (bite-1-1 — Gate)** · `gate:guide15-exec:bite-1-1-gate`

Both declarations validate with their exact typed spellings, no fictitious attestation roots or receipt classes, and exactly the paper's two clauses. Record changed seams and justify unchanged ones with concrete reuse evidence.

**Proposal (bite-1-1 — Suggested commit)** · `proposal:guide15-exec:bite-1-1-suggested-commit`

`feat(realization): declare Keyed Provenance Covenant SOURCE and MERGE`.

### 1.2 Compile validated operation plans · `task:guide15-exec:bite-1-2`

**Task (bite-1-2 — Deliverables)** · `task:guide15-exec:bite-1-2-deliverables`

Reuse graph analysis, fact sources, constructibility, lifecycle, proof planning, placement and coverage for both declarations. Place born-first and continue-first checks at each consumed covenant output; keep SOURCE recognition, record-set reconstruction, acceptance and finality at their honest evidence boundaries. SOURCE has no covenant-input execution carrier merely because its output will later be spendable (`[A-sec:kpc:interface]`); (`[A-sec:kpc:operations]`).

**Setting (bite-1-2 — Files and packages)** · `setup:guide15-exec:bite-1-2-files-and-packages`

Use `packages/compiler/src/`, plan validators and tests, `plans/packages/compiler.md`, and the selected typed semantic boundary (`pkg:compiler:contract`).

**Gate (bite-1-2 — Gate)** · `gate:guide15-exec:bite-1-2-gate`

Every active relation has a reachable named carrier or explicit evidence obligation. Focused corruptions reject missing, duplicate, unreachable and mismatched assignments. Public construction requires no creator secret; hand-written transactions bypassing analysis supply no compiled-contract evidence.

**Proposal (bite-1-2 — Suggested commit)** · `proposal:guide15-exec:bite-1-2-suggested-commit`

`feat(compiler): plan Keyed Provenance Covenant relations`.

### 1.3 Emit both clauses and authenticated recurrence · `task:guide15-exec:bite-1-3`

**Task (bite-1-3 — Deliverables)** · `task:guide15-exec:bite-1-3-deliverables`

Emit the authenticated spent-output position check and the full same-key output-0 program comparison. Choose and justify authenticated current-input introspection or a supported linker-reference strategy, proving constructor closure and excluding alternative paths bypassing either clause. Carry the fixed key through generation and every successor; no literal fixed point is presumed. Keep any NUMS satisfaction conditional on the named assumptions (`judg:guide15-exec:nums-standing`).

**Setting (bite-1-3 — Files and packages)** · `setup:guide15-exec:bite-1-3-files-and-packages`

Use `packages/target-elements/src/transaction_form.rs`, capability assessments, `packages/tapscript/src/`, target/constructor tests and their contracts (`pkg:target-elements:contract`); (`pkg:tapscript:contract`).

**Gate (bite-1-3 — Gate)** · `gate:guide15-exec:bite-1-3-gate`

The emitted schedule has valid stack behavior and complete resource formulas. Later-born outputs, wrong-key successors, wrong scripts, versions and control recipes refuse at declared boundaries beside valid controls. Target form admission adds no covenant law and each clause has independent evidence.

**Proposal (bite-1-3 — Suggested commit)** · `proposal:guide15-exec:bite-1-3-suggested-commit`

`feat(tapscript): emit born-first and continue-first covenants`.

### 1.4 Link references and construct transactions · `task:guide15-exec:bite-1-4`

**Task (bite-1-4 — Deliverables)** · `task:guide15-exec:bite-1-4-deliverables`

Link one deterministic keyed bundle, resolve every relocation once, retain the authenticated constructor/control recipe, and construct SOURCE and MERGE through typed requests. Both forms place the new handle at output 0; SOURCE consumes no handle of the key and MERGE consumes one or more. There is no designated genesis, and the first SOURCE handle is already a record. Other target-valid inputs and outputs remain open. Funding signatures stay separate from the covenant, which requires none (`[A-def:kpc-model:first-output]`); (`[A-def:kpc-operations:classification]`).

**Setting (bite-1-4 — Files and packages)** · `setup:guide15-exec:bite-1-4-files-and-packages`

Use `packages/linker/src/`, `packages/transaction/src/`, their bundle, ABI and construction tests and contracts (`pkg:linker:contract`); (`pkg:transaction:contract`).

**Gate (bite-1-4 — Gate)** · `gate:guide15-exec:bite-1-4-gate`

Constructor/reference closure is exact; a fresh process using public data reproduces both forms. Wrong-key and wrong-position requests refuse at their declared boundaries; caller-supplied scripts or record totals cannot bypass typed construction. Reused assembly introduces no STATE law or owner authorization.

**Proposal (bite-1-4 — Suggested commit)** · `proposal:guide15-exec:bite-1-4-suggested-commit`

`feat(transaction): construct keyed SOURCE and MERGE forms`.

### 1.5 Build vectors and six evidence families · `task:guide15-exec:bite-1-5`

**Task (bite-1-5 — Deliverables)** · `task:guide15-exec:bite-1-5-deliverables`

Build independent fixtures and validators for the six families in the paper's verification appendix (`[A-app:kpc:verification]`). Birth checks output-0 membership and later-output unspendability. Continuation checks every consumed handle, unique continuation, mixed-key refusal and independent coexistence. Record recurrence checks SOURCE singleton identity, single-input and multi-input MERGE, no new record on MERGE, provenance and disjoint growth. Reading checks reconstruction, path growth, missing evidence as incomplete, conflicting identities or cycles as invalid, and the published inclusion/retrieval/finality assumptions. Acceptance checks that different reader rules change only accepted readings, including immutable-rule monotonicity and a time-varying rule with no monotonicity claim. Resources compare predicted and observed costs on exact subjects. Every negative names its own boundary; an evidence failure is not a covenant refusal.

**Decision (Reader rule and evidence format)** · `dec:guide15-exec:reader-evidence`

Publish the candidate reader's amount, attribution and eligibility rule separately from covenant membership, and select internal fixture reports unless an actual external consumer requires a maintained interchange contract under `adr/022-interchange-conventions.md`. Finality is an explicit named assumption for a final reading; no observation proves it universally. The reader rule and report format are the final two of the six realization decisions (`rec:guide15-exec:study-dispositions`).

**Setting (bite-1-5 — Files and packages)** · `setup:guide15-exec:bite-1-5-files-and-packages`

Use `packages/vectors/src/`, fixtures and report validators, independent model conformance, and realization-owned documentation; extend `packages/target-elements-conformance/` only for a demonstrated missing target-generic fact (`pkg:vectors:contract`); `packages/model/README.md`.

**Obligation (Measurement home)** · `obl:guide15-exec:measurement-home`

Retain all six target measurement records in `plans/research/state-constructor.md` as a distinct keyed-covenant constructor evidence set, with public-representation findings linked to (`sec:public-opening:measurements`) where applicable; keep gate performance in (`tab:test-performance:register`). This assignment creates no claim that measurements already exist (`sec:state-constructor:measurements`).

**Gate (bite-1-5 — Gate)** · `gate:guide15-exec:bite-1-5-gate`

All six families validate independently and trace all eight delivery requirements to the paper's statements, with positive and focused negative cases and separately published reader/evidence premises. The documentation explains why no extra clause, carried accumulator or record on MERGE is admitted.

**Proposal (bite-1-5 — Suggested commit)** · `proposal:guide15-exec:bite-1-5-suggested-commit`

`test(vectors): verify Keyed Provenance Covenant evidence families`.

### 1.6 Execute final forms and record independence · `task:guide15-exec:bite-1-6`

**Task (bite-1-6 — Deliverables)** · `task:guide15-exec:bite-1-6-deliverables`

Execute final compiler-derived SOURCE and MERGE forms on the pinned target and independently reconstruct their record sets and reader-filtered readings. Repeat all six measurements on the emitted, linked and constructed subjects; changed bytes, control paths, funding, bounds or policy require remeasurement. Record all six realization decisions and the retirement/fixing of the other three old study decisions, with a seam-by-seam independence account for architecture, realization, compiler, target, tapscript, linker, transaction and vectors.

**Setting (bite-1-6 — Files and packages)** · `setup:guide15-exec:bite-1-6-files-and-packages`

Use `packages/vectors/`, realization documentation, affected package contracts and the Phase-7 gate record in `plans/backlog.md`. Each moved seam needs its engineering reason and regression evidence; each unchanged seam needs a concrete reuse argument.

**Gate (bite-1-6 — Gate)** · `gate:guide15-exec:bite-1-6-gate`

Public deterministic inputs reproduce both final forms and their projections; all six evidence families validate and all eight requirements have final-subject evidence. Keep construction admission, consensus, relay, reconstruction and reader acceptance distinct. Record commands, outcomes, exit codes, durations, retained artifacts and absent dimensions, with the clean full repository gate green.

**Proposal (bite-1-6 — Suggested commit)** · `proposal:guide15-exec:bite-1-6-suggested-commit`

`test(vectors): execute compiled Keyed Provenance Covenant forms`.

### Wave-1 exit · `gate:guide15-exec:wave1`

**Criterion (Compiled contract and evidence)** · `crit:guide15-exec:wave1-compiled`

SOURCE and MERGE pass the same pipeline with validated declarations, plans, target forms, constructor/reference closure, typed construction and all six evidence families, covering all eight delivery requirements. The paper remains the contract of record and claims no realization by publication alone (`[A-sec:kpc:limitations]`).

**Criterion (Final measurements and decisions)** · `crit:guide15-exec:wave1-measured`

Measurements (a)–(f) bind final subjects and policy, all six realization decisions have retained dispositions, and the old nine-decision mapping is complete. No draft default supplies funding or an input bound; both are realization choices supported by observations.

**Criterion (Faithful independence and baseline)** · `crit:guide15-exec:wave1-independent`

Exactly two covenant clauses remain. SOURCE introduces its handle record; MERGE introduces none. Key separation, reader-side acceptance and finality as evidence are preserved without a designated head, terminal or carried counter. The seam record explains changes and reuse, compares figures with the Wave-0 baseline, disposes regressions and records the full repository gate on a clean tree.

**Proposal (wave1-exit — Suggested commit)** · `proposal:guide15-exec:wave1-exit-suggested-commit`

`docs(plans): record Keyed Provenance Covenant independence evidence`.

## 5. Waves 2 to 14 · `sec:guide15-exec:remaining-waves`

**Convention (Shifted numbering and retained delivery)** · `conv:guide15-exec:later-wave-numbering`

The following waves retain the concept's deliverables and suggested commits with numbering shifted by one; an earlier-supplied deliverable remains visible as inherited evidence and is revalidated for its consuming subject.

**Rule (Predecessor gates and retained verification)** · `rule:guide15-exec:later-wave-entry`

Every wave opens only after its predecessor's exit, records figures beside the Wave-0 baseline, and retains the concept's focused verification and evidence boundaries (`sec:guide15:verification`); (`sec:guide15:assurance`).

### Wave 2 — Typed burn and clear realization · `task:guide15-exec:wave2`

**Record (Wave 2 semantic and delivery reference)** · `rec:guide15-exec:wave2-reference`

The retained semantic and delivery reference is (`task:guide15:wave1`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 2 — complete typed burn declaration)** · `task:guide15-exec:wave2-deliverable-1`

- complete typed burn declaration. (`task:guide15:wave1`).

**Task (Wave 2 — complete typed clear declaration)** · `task:guide15-exec:wave2-deliverable-2`

- complete typed clear declaration. (`task:guide15:wave1`).

**Task (Wave 2 — exact event and accounting projections)** · `task:guide15-exec:wave2-deliverable-3`

- exact event and accounting projections. (`task:guide15:wave1`).

**Task (Wave 2 — exact `tag-recon` relation)** · `task:guide15-exec:wave2-deliverable-4`

- exact `tag-recon` relation. (`task:guide15:wave1`).

**Task (Wave 2 — exact residual rule)** · `task:guide15-exec:wave2-deliverable-5`

- exact residual rule. (`task:guide15:wave1`).

**Task (Wave 2 — public ASH availability)** · `task:guide15-exec:wave2-deliverable-6`

- public ASH availability. (`task:guide15:wave1`).

**Task (Wave 2 — architecture weld)** · `task:guide15-exec:wave2-deliverable-7`

- architecture weld. (`task:guide15:wave1`).

**Task (Wave 2 — model and realization conformance)** · `task:guide15-exec:wave2-deliverable-8`

- model and realization conformance. (`task:guide15:wave1`).

**Rule (Wave 2 files and packages)** · `rule:guide15-exec:wave2-settings`

Wave 2 comprises the twelve original bites at (`sec:wave2:execution`) plus bite 2.7a at (`step:backlog:wave2-bite-2-7a`), after 2.6 and before 2.8 under ruling 213. Each bite is limited to its named package/file list, new test modules and nearest census registrations; the verified paths and consumer corrections are recorded at (`tab:backlog:wave2-paths`) and (`tab:backlog:wave2-consumers`). Architecture is read-only unless a concrete missing fact and all eight consume-pipeline items are separately accepted. Model transition acceptance, target lowering, constructor generation, signing and external interchange remain outside these bites. A required additional path or nonmechanical compiler algorithm stops the bite for a revised packet.

A step builds the few tests its design names for basic confidence in the function, with their harness included in the design. A further test, wider battery or fixture rewrite discovered later is recorded as a follow-up and taken up by choice unless it exposes a product defect. The product is the purpose of the work; well-structured code, review and typed refusals already carry much of the confidence, while an ad-hoc test layer is debt. This policy changes neither what the gate must show before landing nor the independent review of a frozen-source change.

**Record (Wave 2 closure carrier)** · `rec:guide15-exec:wave2-carrier`

The Wave-2 closure record (`rec:backlog:wave2-closure`) in `plans/backlog.md` joins the twelve original packet gates in the adopted strategy (`sec:wave2:execution`) and the added union packet under ruling 213, the eight deliverables, seven consuming entry rows, dispositions 1/7/10/12, both compact reopenings, ruling 133's three reopening subjects, the six interface ledgers, baseline comparisons, full gate logs and absent dimensions. It closes no Wave-3 plan proof, Wave-4 carrier measurement, linked constructor equality, Wave-7 STATE generation, target signature, native execution, or Waves-10–12 report-production obligation. A consuming result updates current implementation standing without rewriting the historical preflight finding.

The frozen delivery judgment and Wave-3 handoff are (`rec:backlog:wave2-frozen`) and (`rec:backlog:wave2-wave3-receiver`), on `0.7.188-dev`; full-gate costs and baseline limits are (`rec:history:wave2-frozen-gate`). Wave 2 closes with the reviewed packet `0.7.189-dev` and its own import gate, including plans/labels; (`rec:history:wave2-closure-import`) records the 3,218s green and resource comparison.

**Assumption (wave2 — Inherited inputs)** · `assum:guide15-exec:wave2-inherited-inputs`

Wave 0 supplies the model-law census, region-refit evidence, and accepted generation/record dispositions from bites 0.2 and 0.7; Wave 1 supplies the validated architecture-to-realization boundary and declaration weld from 1.1, never the attestation declarations themselves. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave1`).

**Gate (wave2 — Gate)** · `gate:guide15-exec:wave2-gate`

Both complete declarations validate against architecture and independent model conformance, with exact stored-field assignments, projections, tag, residual rule, and public availability. (`task:guide15:wave1`).

**Proposal (wave2 — Suggested commit)** · `proposal:guide15-exec:wave2-suggested-commit`

`realization: define burn and clear semantics`. (`task:guide15:wave1`).

### Fixed-form Keyed Covenant track · `task:guide15-exec:fixed-form`

Adopt S0–S9 at (`sec:kffwork:execution`) with the corrections and exact paths at (`tab:backlog:fixed-form-paths`), rulings 193–212 at (`rule:phase7:fixed-form-rulings`) and their legacy ledger. Ruling 211 assigns fixed-form capture-freeze admission to S5's typed freeze owner and its consumer activation to S8 before capture; S2 supplies the realization contract and pre-admission walk, without making that identity a prerequisite of S3/S4. This is a named track beside the numbered waves; it neither renumbers them nor discharges Burn/Clear or a later wave. Its task is a complete fixed-form paper of major version 2, public triple-bound realization and fresh evidence while preserving the present edition and its captures.

**Rule (Fixed-form files and packages)** · `rule:guide15-exec:fixed-form-settings`

Each packet follows (`def:kffwork:packet`), the adopted paths and consumer census (`tab:backlog:fixed-form-consumers`). S0 changes planning only; S1 includes the labels census correction and publication tests, with code verification. S3 adds profile code 3 in architecture’s semantic namespace as the single frozen-body exception, paired with pins/history and reviewed legacy equivalence. The scheduling boundary and identity adjudication are (`rec:backlog:fixed-form-decisions`). Existing shared kernels remain shared. New paths needed without a new interface or contract change correct the packet and its census before edits; an uncovered interface, contract or ruling decision requires renewed adoption. Every frozen-source change still follows (`rule:vectors:keyed-refreeze`).

**Gate (Fixed-form track closure)** · `gate:guide15-exec:fixed-form`

Close only on the independently reviewed S1–S8 contracts, emitted/linked/constructed bytes, full reading, derived and observed floor, merge surplus, bounded contention and version-separated captures, with every acceptance and reopening disposition joined at S9. Gate D covers documents; C covers formatting, clippy, affected tests and full repository import verification; P adds the real paper build; N adds isolated native capture/admission as defined at (`def:kffwork:packet`). S1 requires C/D/P. No local documents gate establishes a code, paper or native result. Retain candidate-bound exits, logs, walls, resource deltas and absent dimensions.

**Record (Fixed-form closure carrier)** · `rec:guide15-exec:fixed-form-carrier`

(`rec:backlog:fixed-form-closure`) owns the S0–S9 statuses, verified path lists, shared consumers, adoption decisions, independent reviews and final verdict. Package contracts receive their own boundaries; a paper reader gets the final paper and the single realization record. The deployment triple, capture availability and any claimed finality remain the named external facts at (`tab:kffwork:external`). Preparation may proceed without those values; activities requiring them may not presume answers.

### Wave 3 — Compiler operation plans · `task:guide15-exec:wave3`

**Record (Wave 3 semantic and delivery reference)** · `rec:guide15-exec:wave3-reference`

The retained semantic and delivery reference is (`task:guide15:wave2`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 3 — validated burn plan)** · `task:guide15-exec:wave3-deliverable-1`

- validated burn plan. (`task:guide15:wave2`).

**Task (Wave 3 — revalidated compact-ASH plan)** · `task:guide15-exec:wave3-deliverable-2`

- revalidated compact-ASH plan. (`task:guide15:wave2`).

**Task (Wave 3 — validated clear plan)** · `task:guide15-exec:wave3-deliverable-3`

- validated clear plan. (`task:guide15:wave2`).

**Task (Wave 3 — Phase-7 cross-operation scope)** · `task:guide15-exec:wave3-deliverable-4`

- Phase-7 cross-operation scope. (`task:guide15:wave2`).

**Task (Wave 3 — relation, carrier, layout, coverage, and lifecycle censuses)** · `task:guide15-exec:wave3-deliverable-5`

- relation, carrier, layout, coverage, and lifecycle censuses. (`task:guide15:wave2`).

**Task (Wave 3 — ASH constructor closure)** · `task:guide15-exec:wave3-deliverable-6`

- ASH constructor closure. (`task:guide15:wave2`).

**Task (Wave 3 — corruption-resistant validators)** · `task:guide15-exec:wave3-deliverable-7`

- corruption-resistant validators. (`task:guide15:wave2`).

**Task (Wave 3 — independent small-instance oracles)** · `task:guide15-exec:wave3-deliverable-8`

- independent small-instance oracles. (`task:guide15:wave2`).

**Assumption (wave3 — Inherited inputs)** · `assum:guide15-exec:wave3-inherited-inputs`

Wave 0 supplies the closure-exact audit and region evidence from 0.2 and 0.3; Wave 1 supplies the validated placement and carrier assignment from 1.2 separating reader-owned from emitted relations, without supplying burn/clear plan proofs. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave2`).

Wave 2 supplies complete validated Burn/Clear declarations, assignment/projection/flow laws, public dependency and lifecycle censuses, independent model conformance, exact changed-interface ledgers and ruling-133/138 reopening records, and the rule that an explicit-only Burn does not discharge the PrivateCommitted live-receipt Burn exit; these are inputs to compiler plans and do not supply their proofs. The exact declaration, witness and graph census is (`sec:realization:burn-clear`); silent consumers and their reaching-subject obligations are (`tab:backlog:wave2-consumers`). The consuming closure obligations are joined at (`rec:backlog:wave2-wave3-receiver`).

**Gate (wave3 — Gate)** · `gate:guide15-exec:wave3-gate`

All three validated plans reconcile relation/carrier/layout/coverage/lifecycle censuses, reject focused corruptions, and agree with independent small-instance oracles. (`task:guide15:wave2`).

Under (`rule:phase7:clear-placement`), complete Clear analysis and fresh validation use the exact relation-case product without materialization, active operations agree with the retained enumerator, all active figures and five compiler capture lengths/digests remain unchanged, and measured construction/validation walls accompany the factor census. No freeze inventory or frozen source is edited. Independent review precedes shared-engine integration as well as Clear activation.

**Proposal (wave3 — Suggested commit)** · `proposal:guide15-exec:wave3-suggested-commit`

`compiler: expose burn ASH and clear target plans`. (`task:guide15:wave2`).

**Rule (Wave 3 files, packages and advancement)** · `rule:guide15-exec:wave3-settings`

Wave 3 lands the twenty-one packets in (`tab:guide15-exec:wave3-bites`) in that order, including 3.5c immediately after 3.2a. Packets 3.2d and 3.2e land between 3.2c and resumed 3.5a under (`rule:phase7:clear-placement`). Dependency-independent packets with disjoint code path sets may be prepared on two lanes concurrently; 3.1a and 3.1b qualify. Each still lands through its own full import gate, with independent review placed by ruling 236 (`rule:phase7:wave3-review-placement`). Shared planning/receipt files are reconciled and appended in landing order, never overwritten from a stale base. The per-packet file limit is the exact expansion at (`tab:guide15-exec:wave3-paths`), including registrations; packages are limited to those owning these writable files. Additional gate packages are read-only selections, not edit permission. The table's package descriptions do not widen its exact paths. No glob, nearest-registration exception or unlisted facade correction is implicit. The live packets below supersede the historical drafts for execution; governing rulings are (`rule:phase7:wave3-rulings`), with package meanings at (`sec:compiler:wave3-plans`) and (`sec:realization:wave3-admission`).

A needed extra consumer path with no interface, contract or ruling change pauses edits to that path for a loop-reviewed packet amendment and updated census, following (`rule:guide15-exec:fixed-form-settings`); it does not require a new ruling. An uncovered interface, contract, ruling, semantic owner or mechanism requires renewed adoption. Check the upper design estimate before starting and revise/split a design that exceeds it then; report a line overrun discovered during work, stopping for it only if the design changed. Core scheduling deltas follow the per-packet shares below; the repository weight rule remains authoritative. A necessary edit to any source in either complete freeze inventory stops for a paired re-pin, history row, retained evidence and independent review under (`rule:vectors:keyed-refreeze`). Do not add fields to records constructed by pinned keyed producers. Preserve the five engine-base capture digests and lengths; a formatter or identity mismatch is a finding, never permission to re-baseline. The adoption reading confirmed the two pre-code premises at (`rec:w3:precode`); actual behavior remains each producing packet's proof.

Each implementation packet has its own full import gate before advancement, in addition to its selected focused tests; independent review follows ruling 236 (`rule:phase7:wave3-review-placement`). Public activation includes the independent oracle and the mixed-scope controls; inherited laws or structure are not plan evidence (`rule:guide15-exec:consuming-evidence`). Build the few selected tests, each with its accepted control and changed subject at the owning boundary; further subjects are follow-ups unless they expose a product defect. Oracles construct expectations independently of the publication checked. Review the upper size band before a packet starts and record actual walls; twenty-one full gates cost an estimated 19,950–89,565 seconds before other work. A timeout is unfinished evidence. Later target, signing, linked equality and native/report acceptance remain outside Wave 3.

Current obligations remain in the live phase/package/backlog owners and this guide; only completed receipts enter plans/history/wave3-compiler-records.md. Guide detail is archive-class under (`rule:plans:weight`); this does not turn the history file into an execution authority. The initial 3.0 submission added 17,146 net core bytes, leaving 3,334 of its former 20 KiB wave reserve; that allowance is superseded, not claimed intact. The deliberate reserve for 3.1a–3.8 is 26 KiB (26,624 bytes): nineteen implementation packets target at most 1 KiB each for status and links, closing 3.8 targets at most 6 KiB, with 1 KiB unallocated contingency. Completed detailed evidence belongs in the history receipt; active decisions remain in their live owners or this guide. A packet exceeding its share reports the delta; the scheduling stop applies only when its own net core delta would exceed twice its share (2 KiB for an implementation packet, 12 KiB for closing). Record actual consumption and reforecast at every landing. This is a scheduling guard, not a replacement or relaxation of (`rule:plans:weight`): the hard cap still applies. The 26 KiB reservation leaves 58,871 bytes of the pre-correction 85,495-byte headroom for fixed-form and other work, before the contract deduplication below; no numeric remaining core allowance was found in the fixed-form strategy, route study or vectors contract, so their actual deltas must still be reconciled at landing. Recheck overlapping plans/backlog.md, plans/phases/07-burn-and-clear.md, plans/guides/guide_fifteen.md, plans/packages/compiler.md and plans/packages/realization.md before each landing; no other track's text is silently replaced.

A focused complete Clear analysis runs with its own 120-second budget and 12 GiB memory limit: ~/loops/tmp/w20-test.sh limits its container to 12 GiB and removes it when the run ends; the lane gate unit is limited to 24 GiB. These limits contain an analysis whose predecessor grew without bound and could terminate the whole node account. The 3.2e receipt records peak memory beside walls, separates represented cardinality from work, and compares measured analysis plus fresh validation with the 1–10-second estimate. More than 120 seconds is a stop with the available figures, never permission for a larger budget.

#### Operation-plan contract detail · `sec:guide15-exec:wave3-contracts`

Under (`rule:phase7:wave3-rulings`), retain the compact facade's public name and add validated Burn, Clear and Phase-7 sibling facades over existing attestation lowering and the shared engine. Construction is private; validators freshly derive canonical source, relation, carrier, layout, coverage and lifecycle projections and refuse focused offered changes. Public activation waits for its own independent oracle and mixed-scope controls. The exact packets and operand allocation are (`tab:guide15-exec:wave3-bites`), (`tab:guide15-exec:wave3-paths`) and (`tab:guide15-exec:wave3-operands`); unlisted interfaces or paths stop the packet.

Each relation owns only its operation's selected lifecycle rows for its object, using the output-side choice when qualified. Required input choices are operation-local; another operation or an output choice cannot repair a missing input choice. Exit eligibility uses the exit operation's input admission, not scope membership alone: Burn accepts Explicit ReceiptLive input only, so PrivateCommitted receipt-to-Burn stays unavailable and outstanding. The full candidate union may serve plan-level completeness; the scope also publishes declared obligations omitted by selected facade modes.

Existing facades derive exits per mode from the bound scope. Each mode partitions implemented/outstanding exits; live common summaries use implemented intersection and outstanding union, preserving their existing type shape. TransferLive+Burn has Explicit 2/1 and private 1/2; announcement+Clear has 2/4; compact+Clear has 2/0 and scoped release completeness. Singleton baselines stay fixed. Phase-7 consumes one bound input containing Burn, CompactAsh and Clear, rather than joining standalone plans. No mixed-scope emission is claimed: tapscript's bundle refuses the release-complete compact plan at packages/tapscript/src/bundle.rs:1804–1811. Six typed ASH producer/recognizer comparisons do not establish linked byte equality.

Side qualification is confined to key types and new enum variants; no existing enum variant or other struct in the hashed analyzed projection gains a field, and no new fields enter pinned literal-built engine_cases::Case/Tables, engine_case_facts::Facts or engine_proof/program::Records. Empty qualification preserves encodings and Debug capture bytes. The five compiler engine-base digests and lengths remain unchanged; tests/fixtures/engine_base/MANIFEST and src/tests/engine_base_capture.rs are read-only. Operation-local row ownership, absent-qualifier formatting and the Burn-only mixed visibility exception are required behavior, not implementations claimed at adoption. Ruling 234's reading confirmation is (`rec:w3:precode`).

Rulings 224, 225 and 234 at (`rule:phase7:wave3-rulings`) adopt normalized Explicit input and private output change within Wave 3; they do not admit private input or supply target proofs. The earlier proof-alternative paragraph records the present PublicArithmetic-only implementation and Wave-2 evidence limit. Packet 3.5d will add the existing ConfidentialConservation alternative in Burn's declaration and exact validation census. No new ProofKind or edit to frozen relation.rs is authorized.

Burn Ash still admits Explicit/PublicCommitted semantically. For Burn conservation only, owner lowering replaces the generic compatibility selection with exact tuple exclusions: PublicArithmetic excludes private ReceiptLive output; ConfidentialConservation excludes Explicit ReceiptLive output and PublicCommitted Ash. Input stays Explicit. For E-input/E-Ash/PrivC-change alone, case visibility is Committed; existing placement/coverage then carries target-owned ConfidentialValueConservation with no carrier arithmetic discharge. Every other mixed case retains its typed refusal. Existing explicit amounts remain public operands of other relations. The code-reading confirmation and required later checks are (`rec:w3:precode`); the global proof compatibility rule is not weakened.

Burn DataOutputs owns no representation objects; Clear DataOutputs owns State/Ash with ClearClamp dependency. Multi-object None means no single representation summary, not absence of dependence: exact sources, side activations, cases and dependencies carry it. The event/credit/context/STATE boundaries of (`sec:realization:burn-clear`) remain separate. Wave 4 receives the private-change proof requirements for target assessment and carrier fit, not an emitted or authenticated result.

#### Complete Clear placement contract · `sec:guide15-exec:wave3-placement`

Under (`rule:phase7:clear-placement`), retain every admitted carrier choice and represent Clear's complete placement set as canonical relation-case factors in engine_placement.rs. Its 56 factors have 127 options; the recounted candidate's cardinality is 17,509,995,351,216,488,448. Admission checks exact runtime-key census and local generation/validation coherence. A member selects one canonical assignment per key and exactly the union of selected local layout dependencies. Missing/extra keys, noncanonical carriers, wrong quantification and missing/surplus layout refuse at existing placement boundaries. Arbitrary malformed Records are outside the independence claim; do not silently prune them. Fresh analyzed validation reconstructs factors from bound input and compares every key, option and layout independently of offered coverage. New cross-factor capacity, co-location or selection constraints reopen this contract; no invariant is minted here.

The product supports local-option inspection, exact factor equality and single-member validation; complete Clear is never enumerated. Explicitly bounded materialization serves small equivalence subjects only, with no unbounded iterator or index. Factor cardinalities are the exact symbolic count; optional checked-u128 diagnostics report overflow explicitly. maximum_candidates continues to bound materialized candidates, with no partial result; product materialization whose count exceeds that cap refuses before expansion. Construction charges one state for the root, every factor and every option against maximum_states; counters report actual work, never the represented product as visited assignments. Existing DFS reports/refusals and active figures stay unchanged. Clear deliberately ceases to fail merely because represented cardinality exceeds maximum_candidates; public limits documentation must state this distinction.

Keep existing engine entry signatures and Vec placements for keyed and currently active attestation operations. Shared Operation/Plan/Program result containers may acquire a defaulted placement-storage type parameter; pinned Records/Case/Tables/Facts and proof records gain no fields or new error variants requiring frozen adapter edits. Shared assembly uses fixed private storage adapters, with attestation requesting products for Clear only; no owner callback, identity-based engine policy or second planner. The internal analyzed placement value has explicit enumerated/product branches; enumerated Debug delegates to the original set formatter, preserving five compiler capture lengths/digests. Product mismatch diagnostics name factor coordinates rather than counting missing complete members. Direct readers dispatch explicitly; no iterator compatibility may expand Clear. Coverage uses the existing assignment join over validated local options; operation layout remains the case-fact census, distinct from a selected member's union.

The selected controls, independent expectations and changed subjects are (`sec:clear-placement:evidence`). 3.2d leaves the old enumerator intact; 3.2e joins products into the one production pipeline and proves complete internal Clear plus active preservation. Staged 3.5a projection/field/corruption/API work survives for porting and rerun; its incomplete run is not validation evidence. Expected warm Clear analysis plus fresh validation is 1–10 seconds, bounded by four proof candidates, 224 factors/508 options per derivation and existing graph work, not product size; replace this estimate with measurements at 3.2e. The complete Clear control has the separate time/memory limits in (`rule:guide15-exec:wave3-settings`); its 120-second stop is not widened by the suite's ten-minute diagnostic ceiling. Review placement is (`rule:phase7:wave3-review-placement`).

#### Consumer, freeze and concurrency dispositions · `sec:guide15-exec:wave3-consumers`

The complete-placement consumer census and exact Clear factor recount are (`sec:clear-placement:consumers`) and (`tab:clear-placement:factors`). Its adopted dispositions preserve the real indexed keyed consumers and engine-base byte consumers; facade/coverage/layout consumers need local alternatives. The storage change does not widen this section's freeze permissions.

The inherited (`tab:backlog:wave2-consumers`) remains the semantic-match superset. ACTIVE/CASE/COMPILER/SILENT sites receive staged per-operation meanings; COVERAGE/SUPPLEMENT joins receive exact tables; OWNER/ADAPTER model acceptance stays read-only except the named Burn proof-alternative declaration/validation and shared-bound assembler changes. CONDITIONAL fold/expression remain unchanged because no new expression member is selected. SEPARATE keyed producers retain empty choices; their unpinned construction tests adapt the new generic qualifier. TRACING error names/events follow their actual owner. EVIDENCE readers update actual new-coordinate handling without new native claims. REFREEZE governs every source in both full inventories.

OperandRole/RelationMutation coordinate types need operation_plan.rs re-exports because their definition modules are private. The initial vocabulary bite does not change LayoutRequirement or RequiredSourceKind, so it drops the four unrelated tapscript paths. The early side-layout extension genuinely reaches operation_assessment, maturity_assessment, their tests and upstream re-exports; those are listed in that packet. Linker case accessors mostly forward the type and remain read-only unless a listed literal constructor needs adaptation. No transaction production or keyed capture script is selected.

Mixed-key constructors and interpreters were searched for RepresentationChoiceId, ChoiceKey, ChoiceRecord, ExecutionCaseId, WhenRepresentation and EnforceRepresentation. Shared kernel schemas and attestation case/source/layout encoders are included. Keyed production Case records have empty representation maps and do not construct ChoiceKey; keyed refusal/negative tests do, and are listed. This is why a shared key extension can avoid the pinned producers, not an assumption that only seven compiler paths matter (`packages/compiler/src/engine_keyed.rs:856`), (`packages/compiler/src/engine_keyed_fixed.rs:940`), (`packages/compiler/src/engine_keyed/program/refusal_tests.rs:540`), (`packages/compiler/src/engine_keyed_fixed/program/negative_tests.rs:20`).

Known S8b/S8c/S9 overlaps are planning receivers: plans/backlog.md and plans/phases/07-burn-and-clear.md; S9 also plans/guides/guide_fifteen.md and plans/packages/compiler.md and plans/packages/realization.md. The new receipt file avoids sharing S8's engine-history receipt. No selected vectors attestation reader is in the present fixed-form S8 source roster. Recompute if S8c expands its paths or freeze; do not overwrite another track's receipt or infer behavioral independence solely from disjoint files (`plans/backlog.md:3044–3074`).

The per-candidate retained/disclosed result from engine_proof.rs:271–278 is not copied into the existing facade projections compared for equality. Announcement's disclosure checks read the bound source realization/declassification and its operation-owned public facts, not a partner-varying candidate disclosure (`packages/compiler/src/maturity_announcement_plan.rs:1410–1438,1748–1770`). Preserve that distinction in new facades; whole-scope disclosure remains separate evidence, so no extra per-candidate ambiguity is introduced. The literal-construction invariant above also governs every consumer adaptation; editing an engine record module is not permission to add a field to a pinned-built struct.

**Table (Interface ledger, including report-anchor renewal)** · `tab:guide15-exec:wave3-interfaces`

| Missing fact / inherited type | Smallest change | Regression / identity impact | Evidence / dependency impact | Selected control / subject |
|---|---|---|---|---|
| Input-local choice and unavailable in-scope exit / LifecycleRecords, status | Operation-local required-choice membership, input eligibility table and unavailable status | Old pilot rows unchanged; keyed empty records unchanged; no serialization | Every non-available row remains outstanding; full lifecycle analysis feeds scope | Lifecycle/engine Burn-owned records / own choice removed with TransferLive present; complete analysis follows dormant lowering |
| Per-representation scoped exits / live and announcement common closures | Scope-derived per-mode rows; common implemented intersection/outstanding union | Old standalone sets preserved; new mixed status rows explicit; no byte pin change | No equality assumption across live modes; one-bound Phase-7 scope | Explicit/private TransferLive+Burn and announcement+Clear / changed status |
| Side-distinct selections / object-only choice and case keys | Optional qualifier and side map; new side activations/layout obligation | Empty maps preserve old keys; actual literal consumers listed, including keyed tests | Owner tuple policy over same engine, no second planner | E input/PrivC output / swapped side and missing output choice |
| Burn mixed proof / sole PublicArithmetic alternative | Existing ConfidentialConservation alternative plus exact owner tuple/source constraints | Existing explicit plan retained; exact Burn alternative census updated; no ProofKind change | Target-owned commitment evidence unresolved, public A not private C | Complete mixed plan / missing commitment source or wrong tuple |
| New law coordinates / OperandRole | Reuse ObjectFamilyAmount/Count, RuntimeBound and existing STATE facts where exact; add only absent data-schema/ordinal/tag/address/association/context and event-role coordinates | Preserve old codes; new Payload::Fact code domain disjoint from constructibility::fact_key, reviewed literal injectivity | No target position or arbitrary string; report anchors distinct from coordinator sources | Complete primitive-coordinate table / alias or missing anchor |
| New corruption coordinates / RelationMutation | Closed Burn/Clear coordinate enum and append-only purpose encoding | Re-export via operation_plan.rs; compact 74 negatives unchanged | Required evidence subject, not observation | Expected row / altered boundary |
| Validated sibling/scope publication / compact-only facade | Private-field constructors and fresh validators, scope includes unselected lifecycle obligations | Existing compact name and API retained; no wire format or digest | Target/graph handles absent, no unchecked constructor | Complete facade/scope / dropped relation or unselected obligation |
| Shared-bound incoming edge/seed / union merge | Existing owner rejects unsupported shared dependency with typed error | Singleton and seed-free union unchanged | Prospective admission tightening, no propagation algorithm | Seed-free / one edge / one seed |
| Report-owned event anchor / ExternalEvidenceRequirement and runtime source routing | Compiler-owned ReportObligation boundary, coordinate-keyed coverage purposes/role/projection; retain ExternalEvidence source rows outside runtime requirements | Append enum variants/codes only; no existing record fields, target capability, realization evidence member or frozen source change | Report production/verification remains outstanding; exact six-coordinate partition below | Complete Burn lowering / promote one report anchor to RuntimeCarrier |

Each row records concrete missing fact, inherited type, smallest change, regressions, identity/schema, evidence, dependencies and tests; exact path and test selection is in this guide. New amount coordinates duplicating ObjectFamilyAmount are not proposed (`packages/compiler/src/source.rs:41–85`).

**Table (Binding and DataOutputs discharge)** · `tab:guide15-exec:wave3-boundaries`

| Body | Carrier/static obligations | Report/target obligations |
|---|---|---|
| Burn DataOutputs | Canonical record schema/count/tag/association; no representation dependency | Record credit/event authenticity is not inferred from structural binding |
| BurnBinding | Owner/partition/conservation association and record closure at receipt/coordinator roles | Event-type/value authentication remains report-owned; mixed conservation target evidence stays external |
| Clear DataOutputs | Exact destruction quantity joined to clamp, schema/asset/Recon tag | Target transaction conservation and event report remain separate |
| ClearBinding | STATE succession and declared event-presence association | Event authentication, root freshness and later public recovery are not marked emitted |
| ClearClamp/Assignment/Floor/Flow/Delta | STATE/ASH public source arithmetic and exact conditional destruction/residual rows | No runtime floor counterexample fabricated after accepted clamp; target/report checks retain their owners |

This split follows concept 8.6 and the actual bound laws (`plans/guides/guide_fifteen_concept.md:1803–1833`), (`packages/realization/src/burn_clear_laws.rs:771–811`). A body may demand several evidence boundaries; no report anchor is credited as an emitted predicate.

#### Report-anchor interface renewal · `sec:guide15-exec:wave3-report-anchors`

Ruling 235 at (`proposal:phase7:report-anchor-boundary`) adopts this amendment for resumed 3.2b: read against the stop that required it and the code owners named below, it is the smallest mechanism under which a plan truthfully states that an anchor is owed by a report. It supplies the previously missing mechanism under (`rule:guide15:consume-pipeline`) and (`rule:guide15-exec:wave3-settings`); adopted rulings 221–234 retain their text. The stop was correct: packages/compiler/src/carrier.rs refuses RequiredSourceKind::ExternalEvidence for every runtime carrier, while coverage.rs requires a realization ExternalEvidenceRequirement for CoverageBoundary::ExternalEvidence. That type is defined in packages/realization/src/evaluate.rs, not relation.rs, and names only authorization and conservation. Neither an amount operand nor transaction layout proves event authenticity. The six source coordinates already landed in 3.2a retain their codes, Public visibility, Always activation and ExternalEvidence source kind; 3.2b changes their routing and coverage, not their semantic vocabulary.

Select a compiler-owned ReportObligation member appended to DischargeBoundary (also CoverageBoundary). An anchor's typed identity is its existing OperandId: the owning RelationId plus OperandRole::BurnClear(BurnClearCoordinate), qualified by the existing execution case in CoverageRequirementId. Admit only the two Burn coordinates on BurnBinding and four Clear coordinates on ClearBinding in the table below. No free string, target index, report bytes or new realization evidence kind is introduced. coverage.rs adds new CoveragePurpose variants ReportAnchorRequired { operand } and ReportAnchorChanged { operand, mutation }, EvidenceRole::ReportAnchor { operand }, and ProjectionSubject::ReportAnchors { operands }. These describe required evidence and its changed subjects, not performed verification. Generalize the internal boundary-subject dispatch to distinguish an anchor operand from a realization evidence requirement; do not coerce it into ExternalEvidenceVerified or invent a RequiredCapability.

The full relation source census retains all six anchor rows. Attestation requirement/case-facts/placement lowering splits only these exact, operation-owned coordinates from coordinator runtime sources; a generic filter dropping all ExternalEvidence sources is forbidden. Runtime requirements, carrier availability and layout contain only the coordinator subset. BurnBinding and ClearBinding can therefore each have RuntimeCarrier and ReportObligation boundaries without assigning an anchor to a carrier. Keep carrier.rs's ExternalEvidence refusal. The report boundary publishes one required row per anchor and a projection containing the complete anchor set, not merely the first positive row's operands. Report projections carry no carrier-routed sources and do not compare a runtime relation verdict. Other bodies retain the discharge table above; transaction/certificate context equality and record association remain structural/coordinator facts, never report authentication.

Fresh canonical validation at the analyzed/coverage owners derives the anchor set from the relation, case and source coordinates and checks exact ownership, row cardinality, role, boundary and projection. No missing, duplicate, foreign-operation or runtime-promoted anchor is accepted. Shared engine coverage already supports opaque owner purposes and requirement nodes; use those for these rows, without new fields in engine_case_facts::Facts, engine_cases::Case/Tables or proof/program Records, and without adding a report member to engine_source::Evidence. Keep RelationCasePlan/RelationCoveragePlan's existing external_evidence sets realization-only. Append purpose encodings after the existing codes in engine_attestation/identity.rs and append the boundary after existing variants; preserve every old identity and all five engine-base captures. This is an extension of the existing census/coverage pipeline, not a separate report registry.

| Owning body | Required report coordinates | Coordinator counterpart which cannot discharge them | Later receiver |
|---|---|---|---|
| BurnBinding | BurnEventType, BurnFreshAmountEvent | Burn-record closure, record-to-fresh-ASH association and fresh ASH amount A do not authenticate the Burn event or its value | Wave 10 report producer/verifier (`task:guide15-exec:wave10`); Wave 4 sees unresolved requirements |
| ClearBinding | ClearEventType, ClearDestroyedAmountEvent, ClearOmegaEvent, ClearResidualEvent | STATE closure, destruction X, public omega/clamp and residual amount arithmetic do not authenticate their event roles | Wave 12 report producer/verifier (`task:guide15-exec:wave12`); Wave 4 sees unresolved requirements |

The later report-producing wave receives operation/relation/case identity, the exact event coordinate, its declared changed subjects and complete projection from the plan's existing coverage publication. It must bind actual reports to those subjects and supply its own production/verification evidence. Wave 3 establishes that the requirement is owed and correctly partitioned; it emits no report, verifies no report and claims no report authenticity, public recovery or target acceptance. Mixed scopes still emit nothing. A target-only capability projection intentionally does not discharge or erase the report obligations retained in the full compiler plan (`task:guide15-exec:wave3-deliverable-8`), (`rule:guide15-exec:consuming-evidence`).

| Route | Existing role that fails / owners and cost | Downstream and freeze consequences | Decision and handoff |
|---|---|---|---|
| (a) Add realization external evidence | evaluate.rs owns ExternalEvidenceRequirement; add a realization event identity rather than depending on compiler coordinates, update its operation() match, compiler source/requirement/coverage/engine evidence conversion and target.rs evidence-role projection | ExternalEvidenceRole/capability classification would need a truthful report class rather than WholeTransactionValueConservation; tapscript operation_assessment.rs and maturity_assessment.rs, linker state evidence conversion and vectors report/coverage consumers would require a corresponding contract decision. evaluate.rs is unpinned in both inventories; relation.rs is pinned in both but this route need not edit it. Wider target/report interface and matching-owner path census would require a separate adoption | Not selected: turns an attestation report requirement into realization/target evidence plumbing with no target capability to assess. Later waves would receive a new cross-package evidence class |
| (b) Compiler report boundary | Existing ExternalReport requires a target capability and realization requirement; extend coverage.rs/placement.rs and attestation source, requirement, case-facts, placement, coverage/join and identity owners, using existing coordinate identity and generic engine owner purposes | vectors plan.rs and compact_negative_manifest.rs exhaustively match EvidenceRole: the former retains an outstanding/unreachable report, while the compact-only manifest refuses an unexpected ReportAnchor as UnknownRole, never a substrate report. Compiler coverage tests/oracle enumerate boundaries. No target evidence role changes, no frozen path touched | Selected: smallest typed extension that participates in source and coverage censuses and is impossible to credit as emission; later producers consume exact outstanding anchor subjects |
| (c) Model-scope or standalone outstanding list | ModelScope states a model boundary and cannot mean external authentication; an honest standalone list would need new plan publication/validation/accessors outside the source/coverage census | New fields in existing hashed records would violate ruling 234; a sibling side list avoids those fields but duplicates source identity and requires a separate completeness join and later consumer migration. No pinned edit is intrinsically necessary, but it leaves the existing plan coverage incomplete | Not selected: merely calling a report owed a model fact is false, while an independently validated side list recreates the chosen coverage mechanism outside its owner. Later producers would receive a second inventory to reconcile |

Consumer read-back for the selected route: tapscript operation_assessment.rs/maturity_assessment.rs match RequiredSourceKind and realization evidence, which are unchanged, not the new coverage role; linker state_carrier.rs::class_agrees checks existing boundary membership and has no exhaustive boundary match, while its StateExternalEvidenceRole stays unchanged. These paths remain read-only and no Burn/Clear emitter is added. vectors report.rs accepts only its existing typed external-evidence subjects and cannot claim the new anchor verified. vectors plan.rs and compact_negative_manifest.rs require explicit new EvidenceRole arms; plan.rs retains ExternalReportAbsent/UnreachableInCandidate; the compact-only manifest returns its existing ManifestRefusal::UnknownRole because compact cannot legitimately contain anchors. Compiler coverage.rs, engine_attestation/coverage.rs and identity.rs, tests/coverage_tests.rs and tests/coverage_oracle_tests.rs are the exhaustive match owners to adapt. Existing public enum re-exports in operation_plan.rs suffice; no new public type or struct field is selected.

#### Operand-coordinate allocation · `tab:guide15-exec:wave3-operands`

The table closes 3.0's operand ledger against packages/compiler/src/source.rs:41–85 and the actual primitive observations. These are semantic field coordinates, never transaction positions, concrete values or a new interchange schema. 3.2a uses one closed Burn/Clear coordinate vocabulary carried through OperandRole and re-exported by operation_plan.rs; lowerers supply the operation/data-family/event discriminants. Existing SourceRequirement availability and source kinds keep authentication distinct from successful proof. The names below describe required coordinates rather than authorizing duplicate roles for existing amounts, bounds or STATE fields.

| New coordinate | Existing role that cannot carry it / reason | Semantic owner and boundary |
|---|---|---|
| Data-output schema/kind | ObjectFamilyMembers enumerates objects, not the distinct data-output families; ProjectionSet enumerates projections, not permitted data schemas | DataOutputSpec.kind; packages/architecture/src/spec.rs:116–125; structural schema |
| Data-output family count | ObjectFamilyCount requires ObjectId; BurnRecord and Destruction are DataOutputKind, not object IDs | DataOutputSpec.minimum/maximum and BurnRecord family; the bound itself reuses RuntimeBound |
| Data-output tag and asset | Representation names an object mode, not Burn/Recon tag or optional data asset; ObjectFamilyMembers cannot bind these fields | DataOutputSpec.tag/asset and BurnRecord/Destruction; exact structural discriminator |
| Data-output condition | RootEffects and OpenFlowSet cannot select a data schema's conditional presence | DataOutputSpec.condition; ClearClamp dependency remains separate |
| Record ordinal | CanonicalPartition orders source/flow partitions, not contiguous record-family ordinals | BurnRecord.ordinal, packages/realization/src/burn_clear_observation.rs:54–65; semantic order, no transaction index |
| Record address | ObjectFamilyOwners names protocol owners, not independently offered record beneficiaries; repetitions are allowed | BurnRecord.address; no invented uniqueness or signer requirement |
| Data-output scalar | ObjectFamilyAmount names an object-family aggregate, not a BurnRecord claim or Destruction amount | BurnRecord.amount / Destruction.amount, packages/realization/src/burn_clear_observation.rs:54–80; no credit inferred from structure |
| Record-to-fresh-ASH association | ObjectFamilyMembers gives membership, not the exact output each record names | BurnRecord.fresh_ash; binding to the enclosing snapshot |
| Destruction-to-flow association | OpenFlowSet supplies the flow set, not the designated flow of this data output | Destruction.flow; exact normalized association |
| Observation context: transaction and certificate | CanonicalPartition and ProjectionSet contain neither producer equality token; ExternalEvidence's current variants name conservation/authorization, not context identity | BurnClearContext, packages/realization/src/burn_clear_observation.rs:16–35; equality is not authentication |
| Event-type anchor | ProjectionSet / ExprId::Fact(ProjectionPresent) carry presence only, not the distinct report's event-type authentication obligation | BurnEventType/ClearEventType: ExternalEvidence source row, ReportObligation coverage, no runtime source; packages/realization/src/burn_clear_projection.rs:15–67 and (`sec:guide15-exec:wave3-report-anchors`) |
| Event-value anchor | ObjectFamilyAmount carries the quantity, not its event role; ExternalEvidence's conservation variant cannot prove event authenticity | BurnFreshAmountEvent and ClearDestroyedAmountEvent/ClearOmegaEvent/ClearResidualEvent: ExternalEvidence source rows, ReportObligation coverage, no runtime sources; (`sec:guide15-exec:wave3-report-anchors`) |

Reuse ObjectFamilyMembers/Count/Amount for real object families, RuntimeBound for BurnRecordMax and existing bounds, ObjectFamilyOwners/ProtocolSignerSet for authorization, and ExpressionResult with existing FactId::StateField or relation ExprRole for STATE/clamp/assignment values. Fresh ASH, compact output and residual use their existing side/object amount roles plus the owning relation/event coordinate, never a duplicate scalar role. Credit verdict, event authenticity and construction observation remain distinct; no new role promotes one into another. A genuinely missing fact beyond this table returns through the interface ledger before adding it.

Existing operand role codes 0–16 are unchanged (packages/compiler/src/engine_attestation/source.rs:559–637). New closed-coordinate encoding appends after them; any Payload::Fact encoding uses a distinct leading subject code outside constructibility::fact_key's current 0–11 domain (packages/compiler/src/engine_attestation/constructibility.rs:339–382), with literal injectivity and old-pilot preservation selected at 3.2a. Field values such as an ordinal are operands, not identity codes. Side-qualified Representation and its activation/layout variants belong to 3.5c and retain the key/capture constraints of ruling 234.

#### 1. Complete delivery and receiver trace · `sec:guide15-exec:wave3-required-output`

| Deliverable | Producing packets and evidence |
|---|---|
| Validated Burn | 3.2b, 3.3a/b explicit milestone; 3.5c–f required side-qualified private-change alternative before closure |
| Revalidated compact ASH | 3.4 immediately after Burn lowering and before first activation; exact pins rerun through Clear and side-key changes |
| Validated Clear | 3.2c dormant lowering, 3.5a validator and 3.5b oracle/activation |
| Phase-7 scope | 3.6a/b from one bound scope, with declared unselected lifecycle obligations published |
| Five reconciled censuses | Each facade/activation and scope compares relation, carrier, layout, coverage and lifecycle memberships/joins |
| ASH constructor closure | 3.6a/b six typed producer/recognizer comparisons; linked equality later |
| Corruption-resistant validators | Facade/corruption packets plus activation reaching controls and the full required-coordinate table |
| Independent small-instance oracles | Each activation has its own oracle; one Guide-7 independence rule, exact bounded subset enumeration |

This is the full scope of (`task:guide15-exec:wave3`) and (`sec:guide15:compiler`). Both preflight rows are retained: operation-specific sponsor opacity for Burn, compact and Clear, and target conservation carried as external evidence rather than model-proved substrate acceptance (`plans/guides/guide_fifteen.md:242,271`). No explicit-only final narrowing is adopted.

**Table (Receiver and follow-up dispositions)** · `tab:guide15-exec:wave3-receivers`

| Receiver | Selected evidence / retained disposition |
|---|---|
| Closing B1, private receipt→Burn | 3.1a own-choice/eligibility; 3.3b live PrivateCommitted in TransferLive+Burn is unavailable while Explicit implements Burn; 3.6b publishes the row absent from selected facades |
| 2.7a B1 realization pin | 3.1a Burn+TransferLive preserves explicit-only input admission despite output/private lifecycle union |
| 2.7a B2 mixed compiler scopes, before activation | 3.3b transfer/Burn; 3.5b compact/Clear and announcement/Clear; 3.6b full three-operation scope. Allow exact partner-exit status changes and literal shared reasons, not arbitrary drift |
| 2.7a B3 local closure | 3.1a compiler required choice cannot come from company; 3.6b realization local lifecycle omission still refuses |
| 2.7a B4 and 2.8 B4 | 3.1b shared-bound incoming-edge/seed guard, seed-free control; `plans/backlog.md:740–741` |
| Provisional relation_objects | 3.2b Burn and 3.2c Clear before activation; operation-specific DataOutputs and exact two-object activations/source evidence |
| Joint role results and premises | 3.6 preserves operation/event/credit/silence/accounting/STATE distinctions, original model provenance and realization-only synthetic catalogs |
| Ruling 217 | Authorization's UnexpectedProtocolAuthorization, Constructibility blocked; 3.3/3.5 validator subjects and 3.8 handoff to Waves 5/7/8/11 |
| Rulings 133/138 | 3.7 five individually disposed subjects with actual control/refusal, leaf, successor witness, compact tree/archive, STATE digest and realization transition support |
| Later construction | 3.8 retains carrier policy, linked equality, native successor, signatures/public recovery and report duties for the actual later owners |
| 2.7a B5/B6; longer histories, random order/subsets, tracing/scaling | Deferred follow-ups, not landing claims; missing product behavior still requires disposition |
| 2.9 B1 per-key literal routes | Deferred; manifest/ledger/corpus remain distinct evidence. 2.9 B2 support selection is taken at 3.7 |
| Frozen relation comment | Leave realization relation.rs unchanged; editorial update waits for authorized re-freeze |
| Other 2.5/2.6/2.7/2.8 optional adapter/law permutations | Retain (`tab:backlog:wave2-design-followups`) statuses; no raw model transition or adapter edit selected |
| Additional subjects | Broad published-lifecycle properties, extended five-operation costs and randomized scope permutations remain deferred; exact partition and three-operation counts are selected; qualification is settled by ruling 222 |

This table disposes every (`rec:backlog:wave2-wave3-receiver`) row and the addressed deferred choices. The full inherited consumer categories have live dispositions at (`sec:guide15-exec:wave3-consumers`), not an implied licence to edit every census entry.

#### 2. Gates, estimates and whole-wave cost · `sec:guide15-exec:wave3-verification`

**Definition (Exact packet)** · `def:guide15-exec:wave3-packet`

Each packet names its goal, exact paths, focused tests, gate, output and non-claims. Needed paths use the loop-reviewed amendment route in (`rule:guide15-exec:wave3-settings`); additional mechanisms, changed semantic owners and uncovered interfaces/contracts/rulings need renewed adoption. Every packet receives a full import gate before dependent advancement; independent review follows ruling 236 (`rule:phase7:wave3-review-placement`), which governs review placement throughout the packet records below. New code uses existing structured tracing owners (`[ADR026-dec:tracing:core]`). Tests named below are selected allocations unless described as existing; no name embeds wave/bite chronology.

D is the existing plans/labels/census/generated/document surface; documents-only packets need no Rust lane. C is formatting in place, workspace Clippy with warnings denied, selected package tests under the gate profile, and existing compatibility tests for changed shared vocabularies. The full import gate remains `scripts/ci.sh` through canonical Meson; C/D alone is not that full result. Use one toolchain/target directory, record cold build separately, and format last. No compiler or gate is run locally by a boxed writer; its authorized build lane supplies these results (`AGENTS.md`), (`gate:plans:verification`).

Wave-2 full gates took 950–4,265 s (`plans/backlog.md:683–695`); the frozen gate took 1,198 s (`rec:history:wave2-frozen-gate`) and closure-import 3,218 s (`rec:history:wave2-closure-import`), an increase whose cause is not established. Budget about one hour ordinarily and reserve the observed upper wall plus queue/review overhead; never treat a one-hour estimate as proof a 4,265-second gate will fit. Scope/oracle wall estimates below are unmeasured and must be replaced by actual counts and wall.

Precedent at the reading base: announcement facade 2,125 lines, plan tests 542, corruption tests 1,366, oracle 893; live facade 2,430 and tests 1,135; compact facade 1,263, plan tests 880 and validation tests 651 (the corresponding files under `packages/compiler/src/` and `src/tests/`). The facade/validator halves therefore budget thousands of lines, with independent oracle/activation halves. Dormant lowering is cut by operation; mixed-change work is four packets, not hidden in a small Burn estimate.

**Table (Ordered packet sizes and focused costs)** · `tab:guide15-exec:wave3-bites`

| Packet | Goal | Packages | Pinned source | Estimated changed size | Focused gate / wall after build | Exact file limit |
|---|---|---|---|---|---|---|
| 3.0 | Adopt readings, interfaces and exact settings | planning | No selected edit | initial 3.0 delta +17,146 core bytes; correction/reallocation at (`rec:w3:adoption-correction`) | D; 1–5 min warm estimate | 9 |
| 3.1a | Operation-local choices and qualified exits | compiler, realization tests, tapscript, vectors | No selected edit | 500–900 lines | C: compiler, realization, tapscript, vectors; 1–3 min estimate | 25 |
| 3.1b | Shared-bound guard | realization | No selected edit | 200–400 lines | C: realization; under 2 min estimate | 13 |
| 3.2a | Owned vocabulary and public re-exports | compiler, vectors | No selected edit | 400–800 lines | C: compiler, vectors; 1–3 min estimate | 25 |
| 3.5c | Qualified selection keys and side cases | compiler, tapscript | No selected edit | 800–1,400 lines | C: compiler, tapscript plus all keyed compatibility tests; 2–6 min estimate | 78 |
| 3.2b | Dormant Burn lowering, report boundary and guard relocation | compiler, vectors | No selected edit | 1,200–2,100 lines | C: compiler, vectors; 2–8 min estimate | 49 |
| 3.4 | Compact revalidation before first activation | compiler, linker tests, vectors | No selected edit | 150–350 lines | C: compiler, linker, vectors; 2–5 min estimate | 11 |
| 3.3a | Burn explicit facade, validator and corruptions | compiler, documentation | No selected edit | 2,600–4,200 lines | C: compiler; 2–6 min estimate | 52 |
| 3.3b | Burn oracle, mixed live scope and explicit activation | compiler, realization documentation | No selected edit | 900–1,500 lines | C: compiler; oracle target under 2 min, suite 2–6 min estimate | 53 |
| 3.2c | Dormant Clear lowering with four report anchors | compiler, vectors | No selected edit | 1,200–2,000 lines | C: compiler, vectors; 2–8 min estimate | 49 |
| 3.2d | Checked placement product, old enumerator retained | compiler | No selected edit | 450–800 lines | C: compiler; 1–3 min estimate | 8 |
| 3.2e | Shared storage integration, preservation and complete Clear | compiler | No selected edit | 1,200–2,200 lines | C: compiler, read-only tapscript/linker/vectors; 2–8 min estimate | 30 |
| 3.5a | Clear facade, validator and corruptions | compiler, documentation | No selected edit | 3,000–4,800 lines | C: compiler; 2–6 min estimate | 56 |
| 3.5b | Clear oracle, mixed scopes and activation | compiler, realization documentation | No selected edit | 1,000–1,700 lines | C: compiler; oracle target under 2 min, suite 2–6 min estimate | 57 |
| 3.5d | Mixed conservation declaration and lowering | compiler, realization, planning contracts | No selected edit | 1,000–1,800 lines | C: compiler, realization; 2–6 min estimate | 78 |
| 3.5e | Mixed Burn facade and fresh validator | compiler | No selected edit | 1,500–2,700 lines | C: compiler; 2–6 min estimate | 71 |
| 3.5f | Mixed Burn oracle and alternative activation | compiler | No selected edit | 800–1,400 lines | C: compiler; oracle target under 2 min, suite 2–6 min estimate | 72 |
| 3.6a | One-bound Phase-7 facade and typed ASH closure | compiler | No selected edit | 1,500–2,500 lines | C: compiler; focused wall unmeasured, 10 min diagnostic budget | 26 |
| 3.6b | Scope oracle and complete publication | compiler, realization tests | No selected edit | 600–1,100 lines | C: compiler, realization; wall unmeasured, 10 min diagnostic budget | 29 |
| 3.7 | Candidate-bound reopenings and support pins | vectors, compiler/tapscript tests; read-only realization and target-elements-conformance | No selected edit | 100–300 lines plus 3 KiB receipt | C: vectors, compiler, tapscript, realization, tripod-target-elements-conformance; 2–10 min estimate | 10 |
| 3.8 | Frozen Wave-3 gate and Wave-4 handoff | planning | No selected edit | up to 6 KiB core; detailed receipt archive-class | D plus own full import gate; 950–4,265s historical gate range | 6 |

The twenty-one packets include nineteen implementation/test packets and two documents packets, with twenty-one gates and 19,950–89,565 gate seconds (5.54–24.88 hours) before queueing, implementation, focused checks or rework. Independent review follows ruling 236 (`rule:phase7:wave3-review-placement`). Four private-change packets contribute 3,800–17,060 seconds. Packets 3.2d/3.2e each have a 1 KiB core status/link share; their combined 2 KiB is additional to the former reserve, not charged to its contingency. These are workload estimates, not timing acceptance thresholds. A bounced gate adds another measured round; exact corrections receive the orchestrator diff reading specified by ruling 236.

Review the upper size estimate before starting each packet; revise or split before work if its design exceeds that bound. During work report line overruns; they stop the packet only when the design changed. Per-packet core scheduling shares and their double-share stop are separate from changed-code size estimates. There is no blanket 1,400-line ceiling that will predictably fire midway through a 4,000-line facade. Public activation remains atomic with the complete oracle and mixed-scope controls. New small oracles target under two minutes per module with at most 4,096 products per tiny instance; focused suites use a ten-minute diagnostic budget unless their brief names a justified larger one. A timeout is unfinished evidence. The full three-operation analysis has no measured two-to-five-minute promise: record global proof-plan count, each operation's placement-factor counts and wall, and retain factorization rather than materializing their Cartesian product (`plans/guides/guide_seven.md:56`).

#### 3. Ordered packets · `sec:guide15-exec:wave3-execution`

##### 3.0 — Adopt readings, interfaces and exact settings · `step:guide15-exec:wave3-bite-3-0`

Goal and design: Adopt or revise proposed rulings 221–234. Mint a Wave-3 settings rule beside the execution section; the Wave-2 rule is precedent only. Register plans/history/wave3-compiler-records.md and its index/census, record divergences in the drafts README, and keep the proposal bodies historical after adoption. Confirm the side-key extension, factorable mixed tuple, exact operand/evidence ledger and current full freeze inventories before code. Two explicit pre-code checks are mandatory: the proposed Debug form must preserve all five engine-base captures without manifest changes; and the Ash PublicCommitted incompatibility plus admitted-tuple Committed visibility must be expressible in the named owners. Record these decisions before implementation; failure returns the concrete missing mechanism and revised packet, never a silent re-baseline or explicit-only closure.

Focused selection: Select D checks for labels/census/weight/current owners and an independent reading of all twelve questions and receiver rows. No Rust test. The code-reading confirmations and adopted result are (`rec:w3:precode`) and (`rec:w3:adoption`); this packet does not claim implementation.

Files: exactly the 3.0 expansion at (`tab:guide15-exec:wave3-paths`). Gate: D; 1–5 min warm estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: initial 3.0 delta +17,146 core bytes; correction/reallocation at (`rec:w3:adoption-correction`).

Handoff and limits: Hands on accepted meanings, exact paths and focused subjects. The adopted decisions are in (`rule:phase7:wave3-rulings`); this records work to be done, not implementation evidence.

##### 3.1a — Operation-local choices and qualified exits · `step:guide15-exec:wave3-bite-3-1a`

Goal and design: Make required-choice membership (operation, object)-local using lifecycle-lowering/engine records; reserve the actual input-side Burn choice lowering for 3.2b after 3.5c gives it a qualified key. Add the literal input-admission eligibility table, unavailable-in-scope status and all-nonavailable outstanding filters. Keep graph reachability separate; initialize empty records without editing keyed producers.

Focused selection: Select burn_choice_cannot_be_supplied_by_transfer (Burn-owned synthetic lifecycle-lowering/engine records; remove its choice while TransferLive remains; complete analysis waits for 3.2b), exit_eligibility_is_representation_specific (literal input-admission modes and changed eligibility), and unavailable_exit_survives_internal_projection (synthetic engine/analyzed rows, with a status corruption at the validator). Pair the existing Burn-only private_input_is_not_admitted_by_the_input_change_mode_union with new transfer_company_keeps_burn_input_explicit_only in packages/realization/src/tests/burn_tests.rs. In a Burn+TransferLive realization the latter accepts Explicit receipt input, refuses a one-coordinate PrivateCommitted input at Burn's input SideRepresentation, and retains (ReceiptLive, PrivateCommitted, Burn) as a declared required exit. No claim that graph-only validate_compact_lifecycle_projection validates statuses.

Files: exactly the 3.1a expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler, realization, tapscript, vectors; 1–3 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 500–900 lines.

Handoff and limits: Hands on correct internal choices/statuses; public private-row survival waits for 3.3b and complete unselected-row publication for 3.6. No emitted/deployed usability claim.

##### 3.1b — Shared-bound guard · `step:guide15-exec:wave3-bite-3-1b`

Goal and design: Apply the adopted narrow shared-bound admission rule in the existing assembler/declassification owner. Preserve equal-visibility union and singleton semantics. Add UnsupportedSharedBoundDependency with bound, declaring operation and incoming-edge/seed kind and its diagnostic/tracing cases.

Focused selection: Select shared_bounds_without_incoming_dependencies_merge, incoming_dependency_on_shared_bound_is_refused, and seed_on_shared_bound_is_refused in derivation/disclosure tests, sharing one accepted seed-free control.

Files: exactly the 3.1b expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: realization; under 2 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 200–400 lines.

Handoff and limits: Hands on the guarded union for later mixed scopes. Its gate does not claim mixed facade compatibility or full singleton goldens.

##### 3.2a — Owned vocabulary and public re-exports · `step:guide15-exec:wave3-bite-3-2a`

Goal and design: Add only genuinely missing primitive/event/coverage coordinates, with append-only attestation encodings and disjoint Fact subject domain. Re-export carried public coordinate enums from operation_plan.rs. Retain public operation/body guards and existing source/layout kinds. Update the actual vectors exhaustive and wildcard interpretations without attaching old evidence to new coordinates. The four tapscript paths formerly here are removed because this vocabulary does not reach them.

Focused selection: Select operand_codes_are_injective_and_preserve_pilots, new_coverage_coordinates_keep_their_boundary, and existing public surface/error-name tests. Reuse ObjectFamilyAmount/Count and RuntimeBound instead of inventing duplicate Burn/ASH amount operands.

Files: exactly the 3.2a expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler, vectors; 1–3 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 400–800 lines.

Handoff and limits: Hands on typed vocabulary only, not lowered plans or target observations.

##### 3.5c — Qualified selection keys and side cases · `step:guide15-exec:wave3-bite-3-5c`

Goal and design: Add optional side qualification through choice IDs, core keys/records, case side-map, source activations and side-layout/case-fact projections. Empty/default qualifiers preserve old identities; both keyed owners still publish empty choices. Preserve the engine-base capture as well as key encodings: hand-written Debug for RepresentationChoiceId and ExecutionCaseId omits an absent qualifier or empty side map, preserving the existing derived field order, names and pretty formatting; present qualifiers remain visible. The unchanged preserve_engine_base test must reproduce all five byte lengths and SHA-256 digests in packages/compiler/tests/fixtures/engine_base/MANIFEST (`packages/compiler/src/tests/engine_base_capture.rs:155–229`). Re-baselining is rejected because it would discard the evidence that empty qualifiers preserve old identities. The capture test and MANIFEST remain read-only. Side qualification enters key types (including engine_proof::ChoiceKey, lifecycle ChoiceRecord and ExecutionCaseId) and new activation, structural/compiler requirement and layout enum variants only. Add no fields to the pinned producers' literal-built engine_cases::Case, engine_cases::Tables, engine_case_facts::Facts, engine_proof::Records or engine_program::Records (`packages/compiler/src/engine_keyed.rs:440–452,791–795,842–860`), (`packages/compiler/src/engine_keyed_fixed.rs:480–492,941–945`), (`packages/compiler/src/engine_keyed/program.rs:85`), (`packages/compiler/src/engine_keyed_fixed/program.rs:89`). No pinned producer constructs ChoiceKey or ChoiceRecord. A required new field in those records stops for a separately reviewed re-freeze packet. This packet follows 3.2a directly, before any Burn/Clear facade exists, and before actual Burn choice lowering. This is the only selected shared selection-schema extension; no new search algorithm or pinned producer edit. All literal constructors and actual new layout consumers are in the exact packet.

Focused selection: Select side_choices_do_not_alias_object_choices, unqualified_case_keys_preserve_pilot_identity, and input_choice_cannot_be_repaired_by_output_choice. Existing keyed refusal/negative constructors explicitly use the unqualified form. Re-run the unchanged preserve_engine_base, all compact baseline pins and old live/announcement projections.

Files: exactly the 3.5c expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler, tapscript plus all keyed compatibility tests; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 800–1,400 lines.

Handoff and limits: Hands on expressive side-qualified tables while private Burn selection remains refused. If a pinned producer or additional kernel constraint is required, stop and revise before continuing.

##### 3.2b — Dormant Burn lowering and guard relocation · `step:guide15-exec:wave3-bite-3-2b`

Goal and design: Interpret Burn SideRepresentation, BurnBinding and Burn DataOutputs using the complete literal source/proof/object/boundary table. Burn's actual input choice is born side-qualified using 3.5c. A relation owns only the lifecycle rows of its own operation's selected mode for that object — the output-side choice where the object is side-qualified; plan-level completeness may keep the candidate union. Change engine_requirement::owned_lifecycle and both attestation call paths to pass the owning operation and selected qualified choices, rather than filtering only (object, exit). Candidate filtering alone retains modes chosen by any operation and cannot establish this ownership (`packages/compiler/src/engine_proof.rs:279–289`), (`packages/compiler/src/engine_requirement.rs:32–47`), (`packages/compiler/src/engine_attestation/requirement.rs:115,472–482`). Remove per-stage operation checks and retain them at input.rs and engine_attestation.rs binding seams. Enable these body interpreters, including operation-aware DataOutputs dispatch, while public Burn/Clear stay refused. Convert the old synthetic direct-case/source refusal subjects to actual internal-lowering controls and record their replacement at backlog evidence rows. Do not add a test-only Boolean bypass.

Focused selection: Select burn_bodies_have_exact_owned_tables, side_admission_and_sponsor_opacity, active_carriers_and_layout_join, and live_exit_owns_only_its_selected_mode: in TransferLive+Burn, the live private Burn exit owns no Explicit row supplied by Burn's own input choice. Complete validated declaration records reach crate-private stage functions; the control must reach carrier/layout analysis rather than return UnsupportedOperationPlan. One changed carrier proves the actual placement boundary.

Apply (`sec:guide15-exec:wave3-report-anchors`) to BurnBinding now and provide the shared mechanism Clear will use. burn_bodies_have_exact_owned_tables independently pins the two anchor source rows, their report coverage and absence from every runtime/carrier/layout source set beside coordinator record closure. Add only report_anchor_cannot_be_promoted_to_runtime for the new refusal boundary: a complete internal Burn control validates; changing only one anchor's offered coverage boundary to RuntimeCarrier reaches the owning canonical analyzed/coverage validator and is refused, with no earlier UnsupportedOperationPlan. This proves partition validation, not report verification. Update the existing refusal restatement in tests/compact_validation_tests.rs with the relocated guard; adapt existing coverage-oracle exhaustive matches and the two vectors role readers without adding further subjects. The partial parked owned_lifecycle work is reusable only after review against the landing base; it does not substitute for these selected controls.

Files: exactly the 3.2b expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler, vectors, plus read-only tapscript/linker compatibility; 2–8 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,200–2,100 lines, including approximately 300–500 lines for report routing/coverage and consumer adaptation. Existing per-packet core share and stop conditions remain binding.

Handoff and limits: Hands on independently reviewed dormant Burn tables. Clear-specific bodies remain refused and no public Burn constructor succeeds yet.

##### 3.4 — Compact revalidation before first activation · `step:guide15-exec:wave3-bite-3-4`

Goal and design: Immediately revalidate compact against the modified shared owners, preserving its standalone figures and existing API. Explicitly keep event silence. This order has no dependency on Burn activation.

Focused selection: Select existing compact plan-census tests, the independent placement/coverage/assembly oracles, linker carrier pin and vectors every_coverage_requirement_has_exactly_one_row. Add compact_event_silence_is_retained only if the selected existing corruption does not already prove that coordinate.

Files: exactly the 3.4 expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler, linker, vectors; 2–5 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 150–350 lines.

Handoff and limits: Hands on exact compact baseline. Subsequent Clear and side-key changes rerun these pins; a wider-scope lifecycle result is a new row, not a replacement baseline.

##### 3.3a — Burn explicit facade, validator and corruptions · `step:guide15-exec:wave3-bite-3-3a`

Goal and design: Build the private-field Burn facade and fresh validator from dormant complete analysis, with the 32-relation source census and explicit selection. Public binding still refuses Burn. Crate-private tests derive the facade from complete owner records and freshly validated analysis; they do not manufacture a public unchecked constructor. Lowering owners are writable correction permissions. Prepare scope-derived live-transfer lifecycle summaries and per-representation equivalence normalization without publishing Burn activation.

Focused selection: Select burn_plan_control_and_corruptions for the Burn rows of the corruption table, burn_selection_refuses_private_change at the offered-selection validator returning TargetPlanRepresentationMismatch, and the public API privacy guard. Test omitted lifecycle status at the plan validator.

Files: exactly the 3.3a expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 2,600–4,200 lines.

Handoff and limits: Hands on an internally validated facade. The private change refusal is an intermediate documented limit; it cannot be the wave's final state.

##### 3.3b — Burn oracle, mixed live scope and explicit activation · `step:guide15-exec:wave3-bite-3-3b`

Goal and design: Add the independent Burn oracle, then remove only Burn's public operation guard. Body guards no longer reject its three implemented bodies; Clear remains blocked by its operation guard. Convert declared_burn_reaches_unsupported_plan_at_input_binding to public accepted/invalid controls, preserving the old claim in the receipt/backlog. Update realization README's unsupported statement. Apply the scope-derived live facade contract now, before activation is credited.

Focused selection: Select burn_plan_matches_small_oracle and transfer_burn_scope_preserves_representation_specific_exits: in one TransferLive+Burn bound scope Explicit implements Burn, PrivateCommitted retains unavailable Burn, Redeem stays outstanding, and only literal shared-bound reasons differ. The public API test checks both representation projections and notes that complete Phase-7 publication of the unselected row is supplied at 3.6b. Keep incomplete-realization-scope refusal distinct.

Files: exactly the 3.3b expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler; oracle target under 2 min, suite 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 900–1,500 lines.

Handoff and limits: Hands on a public explicit Burn plan with its oracle and correct private live exit; no mixed Burn alternative yet, and no scope-only direct-private discharge.

##### 3.2c — Dormant Clear lowering · `step:guide15-exec:wave3-bite-3-2c`

Goal and design: Complete Clear-specific bodies and Clear's quantity-dependent DataOutputs, STATE sources, six assignments, clamp/floor/destruction/residual and report splits. Keep the Clear operation check at both binding seams; stage functions already permit internal complete records. Accept None for the two-object single-selection summary only with exact source/activation/dependency tables. Preserve Burn and compact lowerers.

Focused selection: Select clear_bodies_have_exact_owned_tables, clear_data_quantity_keeps_clamp_dependency, and clear_active_carriers_and_layout_join. The DataOutputs expected object set is {State,Ash}; the independent source table and missing-edge subject must show that public quantities are retained.

Reuse the already reviewed ReportObligation mechanism from 3.2b without a second evidence kind. clear_bodies_have_exact_owned_tables independently pins all four ClearBinding anchor rows, complete report projection and coordinator/report source partition under (`sec:guide15-exec:wave3-report-anchors`). Clear's omega/destruction/residual arithmetic remains coordinator-owned; authenticating those values as a Clear event remains report-owned. No additional test is selected: the shared boundary control belongs to 3.2b and these existing Clear selections establish the operation-specific table.

Files: exactly the 3.2c expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler, vectors, plus read-only tapscript/linker compatibility; 2–8 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,200–2,000 lines. Added shared owners are correction permission, not a second implementation of the report mechanism.

Handoff and limits: Hands on complete dormant Clear lowering, not public plan evidence. Both later Clear packets own these lowerers for corrections.

##### 3.2d — Checked placement product · `step:guide15-exec:wave3-bite-3-2d`

Goal and design: Under (`rule:phase7:clear-placement`), add the checked canonical product, single-member validation and bounded expansion in the existing placement owner under (`sec:guide15-exec:wave3-placement`). Keep the old enumerator and production routes unchanged as the equivalence witness; no Clear activation.

Focused selection: placement_product_matches_exhaustive_oracle and placement_product_rejects_incoherent_factors, with literal independent controls and changed layout/quantification/census subjects at (`sec:clear-placement:evidence`). Cover all multiplicities and zero-runtime identity; no exhaustive Clear expansion.

Files: exactly the 3.2d expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler; D and full import gate; 1–3 min focused estimate. Estimated size: 450–800 changed lines. Handoff: checked product helper; orchestrator whole-diff review now, independent review includes it at 3.2e before shared integration.

##### 3.2e — Complete Clear and preserved shared consumers · `step:guide15-exec:wave3-bite-3-2e`

Goal and design: Integrate fixed storage adapters into shared assembly, restore Clear products through attestation identities and freshly validate exact factors. Keep keyed/default interfaces, active attestation members, formatting and all figures/pins. Adapt direct analyzed readers explicitly; coverage reads local options. Public Clear still refuses binding. The census at (`sec:clear-placement:consumers`) grants no extra paths.

Floor correction under (`rule:phase7:clear-floor-coverage`): packages/compiler/src/coverage.rs runtime_mutations assigns ClearFloor the existing RuntimeCarrier AmountMismatch class instead of an empty list. Keep placement.rs's runtime discharge and packages/compiler/src/engine_coverage.rs validate_plan unchanged. That census requires both signs at every active boundary; a coverage mutation names a semantic refusal obligation, not an executed malformed transaction or a world only one relation rejects. Coverage's RequireIntendedAndDependencyClosure requires intended refusal and dependency collateral; additional collateral is reported, never assumed absent (plans/guides/guide_six.md sections 7.4 and 8.5; coverage.rs collateral_policy). packages/realization/src/burn_clear_laws.rs floor requires 1 <= successor y_l + y_t <= predecessor sum and unchanged omega. The existing clear_floor_is_independent_of_clamp_and_copy control and successor y_t mutations in packages/realization/src/tests/burn_clear_relation_tests.rs demonstrate floor refusal with clamp acceptance; they do not promise that all assignments hold. Assignment implication therefore does not justify deleting the declared runtime boundary. No realization declaration, shared census, new mutation kind or placement-lowering change is selected; stored/derived fields retain (`inv:guide15-exec:clear-census`).

Clear boundary census from coverage.rs runtime_mutations/static_mutations joined to placement.rs discharges and engine_attestation/coverage.rs: Floor is the sole empty active mutation list. The table groups all 44 declared relations; sponsor-only runtime rows are inactive in the absent case and correctly have no negative. The four representation candidates use the same mutation classes.

| Clear relation group | Negative obligation at its active boundary |
|---|---|
| Six cardinalities, six recognitions, two allowed-family relations | Runtime: cardinality bound, wrong asset/object, undeclared family respectively |
| Six assignments | Runtime: AmountMismatch or ExpressionPredicateFalse according to expression presence |
| Clamp, floor | Runtime: AmountMismatch; floor added by this correction |
| Flow, binding | Runtime: DestructionFlow coordinate and AmountMismatch; binding also has four report-anchor coordinate negatives |
| Canonical delta, open flow, sponsor envelope, sponsor isolation, root, projection | Runtime: existing delta-family/source, open-flow, envelope, sponsor, root and projection mutation classes |
| Permissionless authorization, data outputs | BackendStructural: unexpected secret and six schema-coordinate mutations |
| Substrate, public constructibility | ExternalEvidence: missing/failed/identity-mismatched evidence; CompilerStatic: unavailable witness/private dependency respectively |
| Two representations, eight lifecycle exits | CompilerStatic and BackendStructural: representation support/authentication and missing exit respectively |

Focused correction selection in the already selected coverage_tests.rs: clear_floor_runtime_coverage accepts the derived floor row with its literal expected RuntimeCarrier AmountMismatch obligation, then removes only that negative and requires MissingNegative at the same relation/case/boundary through the shared census. clear_coverage_boundaries_complete independently checks every declared Clear boundary in both sponsor cases and all four candidates, including inactive sponsor controls; the complete internal analysis remains the integration control. source_tests.rs clear_bodies_have_exact_owned_tables changes only the ClearFloor relation_mutations pin from an empty vector to [(RuntimeCarrier, AmountMismatch)]. This adds one negative per floor case, two per selected proof candidate (eight across four candidates), not a new relation, carrier, factor or positive. Retain the 11 bodies, four floor operands, 56 factors/127 options and all existing-operation pins. The realization test is independent evidence of the numeric subject; compiler census tests prove obligation completeness, not execution. The floor sentence of (`rec:w3:3-2c`) records the superseded lowering. Add coverage.rs and source_tests.rs to this packet's paths; no shared coverage-owner edit is authorized. Estimate 50–100 correction lines within the existing packet band; budgets, memory receipt, gate and joint independent review remain unchanged.

Focused selection: placement_storage_preserves_active_plans and clear_product_analysis_is_complete at (`sec:clear-placement:evidence`), including changed-option refusal, low-state-budget refusal and preflight candidate-cap refusal for materialization. Run existing engine-base captures and keyed byte/pin suites unchanged. Run the complete Clear control with its own 120-second budget and 12 GiB memory limit using the diagnostic runner specified in the settings. Its receipt records construction, fresh validation and total wall, peak memory, factor/option work and represented cardinality separately, and compares the measured analysis/validation wall with the 1–10-second estimate. Exceeding 120 seconds stops with those figures; do not enlarge the budget. No count-only equivalence or partial analysis.

Files: exactly the 3.2e expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler plus read-only tapscript/linker/vectors compatibility; D and full import gate; 2–8 min focused estimate, Clear control budget 120s after build. Estimated size: 1,200–2,200 changed lines. Handoff: complete internal Clear and exact active preservation; independent review of both prerequisites before this shared change lands. Then resume 3.5a.

##### 3.5a — Clear facade, validator and corruptions · `step:guide15-exec:wave3-bite-3-5a`

Goal and design: Build the private-field Clear facade, exact 44-relation and six-field requirement projections, fresh validation and focused corruption table while public binding remains refused. Prepare scope-derived announcement and compact lifecycle behavior. Lowering correction permissions include all carrier/layout/coverage/case owners.

Resume after 3.2e lands. Preserve and port staged facade, six-field table, projection validator, corruption/API-privacy selections and announcement lifecycle work; rerun against complete product analysis. The staged incomplete run supplies no validation receipt. This packet neither bypasses complete analysis nor changes the product contract.

Focused selection: Select clear_plan_control_and_corruptions, clear_sponsor_amount_is_refused (SponsorValueRead at its source boundary), and the API privacy guard. Full/partial/refused-clamp are literal conditional requirement classes. ClearFloor's facade subject is a plan requirement corruption; its separate runtime AmountMismatch obligation remains required under (`rule:phase7:clear-floor-coverage`), with assignment collateral permitted and reported, without an isolated complete-operation world claim. Preserve Authorization failure and blocked Constructibility for signer insertion.

Files: exactly the 3.5a expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 3,000–4,800 lines.

Handoff and limits: Hands on internally validated Clear; no announcement successor execution or linked constructor claim.

##### 3.5b — Clear oracle, mixed scopes and activation · `step:guide15-exec:wave3-bite-3-5b`

Goal and design: Land the independent Clear oracle and the two required mixed pairs before removing Clear's public guard. Convert declared_clear_reaches_unsupported_plan_at_input_binding and update realization/compiler READMEs. Retain the two Unsupported errors as reserved future boundaries, with honest current tests.

After the placement amendment, derive Clear's exact local factor/option/layout census and selected-member answers independently; inherit tiny exhaustive-product evidence from 3.2d and never enumerate Clear's huge set. Mixed pairs keep enumerated partner factors and the exact lifecycle differences below. Activation review includes product evidence; shared-engine review does not replace it.

Focused selection: Select clear_plan_matches_small_oracle and clear_scopes_preserve_local_requirements: CompactAsh+Clear and AnnounceMaturity+Clear compare standalone projections except exact partner-exit changes and shared-bound reason unions. Explicitly pin compact 1→2 implemented/1→0 outstanding/scoped completeness and announcement 1→2 implemented/5→4 outstanding; retain old standalone expectations. Include Clear sponsor refusal from 3.5a in the activation selection.

Files: exactly the 3.5b expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler; oracle target under 2 min, suite 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,000–1,700 lines.

Handoff and limits: Hands on all three explicit facades and their own oracles; private-change delivery remains a required predecessor of scope closure.

##### 3.5d — Mixed conservation declaration and lowering · `step:guide15-exec:wave3-bite-3-5d`

Placement-amendment consequence for 3.5d–3.5f: keep Burn's bounded enumeration and existing explicit figures. Clear's product does not authorize a new private-change placement policy, proof compatibility change or cap re-baseline.

Goal and design: Add existing ConfidentialConservation as a Burn alternative in declaration/validation, keeping PublicArithmetic for readable cases. Lower the exact E-input/E-ASH/PrivC-change tuple and external commitment-conservation evidence, with public recognition/partition and owner-local change data. Burn's Ash domain remains {Explicit, PublicCommitted}, not fixed Explicit (`packages/realization/src/declarations/burn.rs:303–313`). PublicArithmetic excludes (Burn output ReceiptLive, PrivateCommitted); ConfidentialConservation excludes both (Burn output ReceiptLive, Explicit) and (Burn Ash, PublicCommitted). With normalized Explicit input these unary incompatibilities admit only E-input/E-Ash/PrivC-change for the confidential alternative. For that tuple alone, conserved_amount_visibility returns Committed and discharge is target-owned ConfidentialValueConservation; explicit amounts remain public operands of the other relations. Every other mixed case retains MixedRepresentationConservation. Reading only the object map would falsely classify this case as Readable; reading sides without this exception would refuse it (`packages/compiler/src/case.rs:124–151`), (`packages/compiler/src/placement.rs:303,381,565`), (`packages/compiler/src/coverage.rs:577,690`), (`packages/compiler/src/engine_attestation/coverage.rs:29`). The source table must not disclose C. Review/adopt the specific mixed-route admission in the existing contract; no global PublicArithmetic or proof compatibility weakening. Public mixed selection still refuses. 3.5d must decide and pin its mixed conservation source table explicitly: the current ConfidentialConservation branch makes every ObjectFamilyAmount an AuthenticatedCommitmentRelation with Always activation, including Explicit input and Ash (packages/compiler/src/engine_attestation/source.rs:355–365). Choose and justify either retaining those explicit amounts as public operands of the target equation or splitting the rows by side. The chosen literal table must distinguish explicit values from private change commitments without exposing C; the selected burn_mixed_tuple_selects_commitment_evidence control and changed-source subject check it. The observation that other relations keep public operands does not decide this table.

Focused selection: Select burn_mixed_tuple_selects_commitment_evidence, private_change_never_becomes_public_arithmetic, and burn_proof_alternatives_match_the_reviewed_census. Controls retain explicit Burn and reject private input, swapped sides, missing commitment evidence and cross-product mismatches through the existing finite constraint mechanism. Select admitted_burn_tuple_has_committed_visibility with the admitted tuple as control and one changed Ash mode as the refused subject; also retain the explicit Readable control and assert target-owned ConfidentialValueConservation discharge for the admitted tuple.

Files: exactly the 3.5d expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler, realization; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,000–1,800 lines.

Handoff and limits: Hands on reviewed mixed law/proof sources and capability obligations, not target verification. If unary incompatibilities cannot enforce the exact tuple, return the concrete combination-constraint gap and revised path/freeze cost.

##### 3.5e — Mixed Burn facade and fresh validator · `step:guide15-exec:wave3-bite-3-5e`

Goal and design: Extend the Burn facade with a separately validated private-change alternative, preserving explicit projections and distinct side/source/evidence rows. Include the new corruption module and exact registration. Constructor and offered-selection validator remain private/dormant for this alternative until its oracle lands; the explicit public route remains available.

Focused selection: Select mixed_burn_plan_preserves_public_ash_and_private_change, mixed_burn_plan_rejects_lost_commitment_evidence, and mixed_burn_plan_rejects_input_output_aliasing. Validate complete tuple/source/layout/coverage/lifecycle joins and no partial plan; the private receipt-to-Burn exit stays unavailable independently of private change output support.

Files: exactly the 3.5e expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,500–2,700 lines.

Handoff and limits: Hands on validated internal mixed alternative. Do not label external commitment requirements as completed target proofs or emit bytes.

##### 3.5f — Mixed Burn oracle and alternative activation · `step:guide15-exec:wave3-bite-3-5f`

Goal and design: Add independent literal side/evidence and small placement oracle, then admit the mixed offered-selection request. Review exact conditional change/no-change behavior and sponsor isolation; fresh public ASH constructor requirements remain shared with the explicit alternative. Forward both plan alternatives to Wave 4.

Focused selection: Select mixed_burn_plan_matches_small_oracle, normalized_input_with_private_change_is_admitted, and private_input_remains_refused_after_private_change_activation. Re-run explicit Burn/compact/live/announcement regressions and the private lifecycle reaching subject.

Files: exactly the 3.5f expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler; oracle target under 2 min, suite 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 800–1,400 lines.

Handoff and limits: Hands on the final Wave-3 Burn domain, including accepted private change but no direct-private input. Wave 4 still assesses complete confidential-change proofs/weight/fees/policy.

##### 3.6a — One-bound Phase-7 facade and typed ASH closure · `step:guide15-exec:wave3-bite-3-6a`

Placement-amendment consequence: consume Clear's validated factors alongside unchanged enumerated Burn/compact factors from one bound analysis. Do not form either the within-Clear or cross-operation product; report represented cardinality separately from work and preserve the exact ASH/lifecycle obligations.

Goal and design: Build scope from one BoundCompilerInput containing Burn, CompactAsh and Clear. Freshly derive all plans and six producer/recognizer comparisons. Publish full declared-representation lifecycle obligations not carried by selected facades, derived from lifecycle analysis rather than facade union; validate exact partition with carried obligations. Include private ReceiptLive→Burn even when every selected Burn input is explicit.

Focused selection: Select scope_rejects_operation_and_constructor_corruptions and unselected_representation_obligations_remain_published. A lost private row must fail the scope validator. Retain separate event/record/accounting/model/synthetic provenance.

Files: exactly the 3.6a expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler; focused wall unmeasured, 10 min diagnostic budget; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,500–2,500 lines.

Handoff and limits: Hands on privately validated scope facade and closure comparison. No linked-byte equality, combined model transition or deployed completeness claim.

##### 3.6b — Scope oracle and complete publication · `step:guide15-exec:wave3-bite-3-6b`

Placement-amendment consequence: scope completeness includes exact Clear factors; the scope oracle derives these independently and uses bounded tiny-instance expansion only. Record the complete three-operation wall and inherit the active equivalence/capture evidence from 3.2e; scope review remains independent.

Goal and design: Land independent full-scope expected relation/source/lifecycle/constructor joins and publish the validated scope. The named pair tests already landed with activation; this bite adds the full three-operation scope. Record proof-plan counts, per-operation placement-factor counts and wall; avoid a product of independent factors.

Focused selection: Select scope_matches_independent_rows and declaration_company_cannot_repair_local_lifecycle_omission. Check the exact carried/unselected obligation partition including the private route and six producer/recognizer pairs. A broad randomized property test is a follow-up, not silently included.

Files: exactly the 3.6b expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: compiler, realization; wall unmeasured, 10 min diagnostic budget; D for changed records, independent review and full repository import gate before advancement. Estimated size: 600–1,100 lines.

Handoff and limits: Hands on complete compiler scope evidence for candidate reopening. Timing beyond the finite budget is unfinished evidence and requires a reviewed domain/packet adjustment.

##### 3.7 — Candidate-bound reopenings and support pins · `step:guide15-exec:wave3-bite-3-7`

Placement-amendment consequence: retain all existing pins and native evidence boundaries. Carry the 3.2e complete-analysis/equivalence record into the reopening decision, never translate represented placement cardinality into tested or executed subjects.

Goal and design: Re-dispose both compact emitted-Clear keys and all three ruling-133 positions, binding leaf, witness, tree/archive and STATE pins. Changed compiler support is not a new emitted path. All candidate source pins and fixtures remain read-only; correction of an actual emitted validator requires revised scope.

Focused selection: Select compact_clear_declaration_reopenings_keep_both_keys; candidate_leaf_and_transition_censuses_have_no_announced_state_spend; no_announcement_byte_carries_the_successor_recipe; the_fixture_pin_is_the_output_key_this_tree_derives in target-elements-conformance; current_archive_replays_sixteen_rows_after_their_paired_controls; every_staged_subject_matches_its_pin; exact compact negative-manifest/corpus join; and the two realization announce_maturity transition refusals named in the reopening row.

Files: exactly the 3.7 expansion at (`tab:guide15-exec:wave3-paths`). Gate: C: vectors, compiler, tapscript, realization, tripod-target-elements-conformance; 2–10 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 100–300 lines plus 3 KiB receipt.

Handoff and limits: Hands on five separate dispositions, actual boundaries and supporting unchanged candidate evidence. No native successor or emitted Clear credit.

##### 3.8 — Frozen Wave-3 gate and Wave-4 handoff · `step:guide15-exec:wave3-bite-3-8`

Placement-amendment consequence: closing requires complete Clear factor validation, the independent small-product and active-equivalence controls, preserved captures/pins, measured phase walls and the additional shared-integration review. Hand off a complete set of choices, not an emitted Clear implementation.

Goal and design: Join all preceding receipts on one candidate, including this closing record's own full gate. Recount the Wave-0 baseline alongside current figures, complete compact pins, all five reopenings, mixed scope and both Burn alternatives. Keep detailed evidence in the history owner and compact current obligations in live owners.

Focused selection: Use the closing checklist below. No code fix is hidden here: reopen its producing packet. Record actual test/gate exits, wall/cold-cache/cgroup scope and real-document/reproducibility status rather than promote a mocked gate.

Files: exactly the 3.8 expansion at (`tab:guide15-exec:wave3-paths`). Gate: D plus own full import gate; 950–4,265s historical gate range; D for changed records, independent review and full repository import gate before advancement. Estimated size: up to 6 KiB core; detailed receipt archive-class.

Handoff and limits: Wave 4 receives both Burn alternatives, compact/Clear/scope plans, five censuses, ASH contracts, external evidence and carrier reopening duties. Signature/native recovery/linked equality and report production remain later-wave obligations.

#### 4. Focused corruption coverage · `tab:guide15-exec:wave3-corruptions`

These are the twenty required coordinates from (`rule:guide15:compiler-validation`), grouped into operation/scope tests, not twenty new harnesses. Each compares an accepted control with one changed coordinate at the owning boundary. A realization weld refusal and a crate-private offered-plan corruption are different subjects; record which one was actually reached.

| Coordinate | Packet and required result |
|---|---|
| Remove one operation; merge Burn/compact identities | 3.6a/b scope census/identity refusal, no scope value |
| Remove owner authorization | 3.3a missing authorization/source requirement |
| Mark Clear operator-authorized | 3.5a authorization/evidence mismatch |
| Remove public ASH availability | 3.3a/3.5a/3.6a source/public-dependency mismatch |
| Change tag-recon; remove Y−1; remove residual condition | 3.5a exact Clear law/conditional-row mismatches |
| Insert RESV | 3.5a forbidden root requirement |
| Make compact emit Burn event | 3.4 event-silence/projection refusal |
| Make Clear emit Burn record | 3.5a wrong data/event requirement |
| Remove event-value anchor; remove event-type anchor | 3.3a distinct binding/report rows, not emitted credit |
| Introduce sponsor amount | 3.2b/3.3a Burn and 3.5a/b Clear each reach SponsorValueRead; compact existing public_api.rs:587 subject retained |
| Remove lifecycle exit | 3.1a/3.3a/3.6a exact path/status/census refusal; no smaller-census success |
| Assign relation to inactive carrier | 3.2b/3.2c/3.3a placement eligibility refusal |
| Introduce target type; expose graph handle | Explicitly adopted public API-shape/privacy interpretation at each facade, not fake runtime cases |
| Duplicate relation | 3.3a multiplicity/census refusal, not deduplication |
| Change evidence boundary | 3.3a/3.5a exact role/boundary refusal, reports never counted emitted |

The additional selected receiver subjects are operation-local choice, published/unselected private obligations, partner-exit status changes, shared-bound seed/edge and mixed input/output aliasing. Randomized scopes, every field permutation, full-field singleton goldens, long histories and new native subjects remain follow-ups.

#### 5. Baselines, scope figures and closure · `tab:guide15-exec:wave3-baseline`

| Standalone compact figure | Correct pin |
|---|---|
| 23 relations / 2 cases / 46 relation-case rows | `packages/compiler/src/tests/operation_plan_tests.rs:144,209,878` |
| 216 placements | `packages/compiler/src/tests/pilot_program_tests.rs:767`; also analyzed_program_tests.rs:738 |
| 63 layout requirements | `packages/compiler/src/tests/operation_plan_tests.rs:274` |
| Coverage graph 338 nodes / 355 edges | `packages/compiler/src/tests/pilot_program_tests.rs:770–771` |
| 28 runtime carrier relation-case keys | `packages/linker/src/tests/carrier_tests.rs:32–33,48` |
| 209 coverage requirements = 135 positive + 74 negative | `packages/vectors/src/plan.rs:1521–1523` |

No packet intentionally moves those standalone counts or old encoded case identities. The side qualifier is absent for old pilots and its encoder retains old bytes. A proposed change names affected rows, reason and downstream evidence before its producing packet; updating a golden after a surprise is not disposition (`obl:guide15-exec:baseline-comparison`).

Scope-derived lifecycle is an intentional interface change, with exact expected figures at (`proposal:w3route:scope`): live Explicit in TransferLive+Burn implemented 1→2/outstanding 2→1, private live unchanged in cardinality but Burn outside→unavailable; common summary intersection/union remains 1/2. Announcement+Clear implemented 1→2, outstanding 5→4. Compact+Clear implemented 1→2, outstanding 1→0 and scoped completeness false→true. Standalone tests retain old values. Update live plan tests at :278 and lifecycle corruption/equivalence checks, announcement plan tests :484–488 and corruption/equivalence checks, compact plan test :472 and status corruptions. These are semantic status changes on wider scopes, not a new emitted leaf or candidate-byte/freeze update. Mixed-change plan counts are new and must be enumerated, never inferred by adding 32 relations twice.

At closure also recount beside Wave 0: Safety 206/177/29; observed/final/retired/waiting 177/27/2/0; classes 81/12/39/15/16/14; relation links 14/108/84; reasons 55/9/9/0/11; conjunct parts 10 MET/9 RETIRED/0 WAITING. These are retained comparison figures from `plans/backlog.md:779`, not predicted new counts. Publish actual current identities/counts and explain every movement or unchanged result; preserve T11-064/T11-063/T4-009/T11-110 on their own criteria. Compact-only figures do not satisfy this obligation (`rule:guide15-exec:later-wave-entry`), (`obl:guide15-exec:baseline-comparison`).

**Gate (Closing checklist)** · `gate:guide15-exec:wave3-closing`

“All three validated plans reconcile relation/carrier/layout/coverage/lifecycle censuses, reject focused corruptions, and agree with independent small-instance oracles.” (`gate:guide15-exec:wave3-gate`).

| Conjunct / accompanying obligation | Receipt evidence |
|---|---|
| Three validated plans | Burn explicit and private-change alternatives, existing compact API revalidated, Clear complete; private fields, fresh validators, no partial values |
| Relation census | Exact identities/bodies, multiplicity, active/inactive cases, selected domains and two-object representation/source meaning |
| Carrier census | Every active runtime obligation has reachable eligible carrier, every-member/local/coordinator roles distinct; reports never emitted |
| Layout census | Exact side/source/authentication/coordinator obligations and joins, no lost private commitment source |
| Coverage census | Positive/negative/structural/report requirements, dependency collateral and actual refusal ownership |
| Lifecycle census | Input-local choices, qualified status, declared graph retained, carried/unselected partition complete; private Burn exit published outstanding |
| Focused corruptions | Every required coordinate plus selected receivers and accepted controls, actual typed/static boundary |
| Independent oracles | Common Guide-7 rule; domains/products/branch classes; expected-side independence and actual counts/wall |
| Cross-operation/ASH closure | One bound input, all three facades, six typed constructor comparisons and exact permitted partner-status/reason changes |
| Baseline and candidate obligations | Full Wave-0 recount above, compact pins, separate new plan counts and five ruling-133/138 dispositions with support pins |
| Reviewed candidate and gates | Reviews placed by ruling 236 (`rule:phase7:wave3-review-placement`), every packet's full gate, source/records identity, final records' own gate, wall/cgroup/cache scope, skipped real-document/reproducibility dimensions stated |
| Wave-4 handoff | Both Burn alternatives and confidential-change capability/evidence duties; typed carrier/schema assumptions and affected Wave-2/3 rows for any reopening |

Wave 4 assesses complete transaction forms and confidential change proofs/weight/fees/policy; on refusal its ruling names the declarations and plans reopened. Later waves retain linked ASH equality, STATE generation, signatures/public recovery and report production. Forward Authorization/blocked Constructibility and the outstanding direct-private receipt route unchanged (`rec:backlog:wave2-wave3-receiver`), (`cond:guide15-exec:carrier-reopening`). Closure does not close Phase 7.

#### 6. Exact writable-path census · `sec:guide15-exec:wave3-paths`

**Table (Literal path sets)** · `tab:guide15-exec:wave3-paths`

Each prefix and basename expands to one exact path; no glob or “nearest registration” is implicit. A listed path is permission, not a mandate to edit it. All facade/validator and oracle/activation packets include their lowering correction owners, including engine_requirement (operation-local lifecycle ownership), carrier, placement, layout, coverage/coverage_graph, sponsor_region and attestation placement/operation/coverage/coverage_join/case_facts/cases/constructibility. Module/test/Meson registration is explicit. Read-only regression packages do not acquire write permission.

Before the placement amendment this table had 140 distinct paths (128 existing and 12 allocated new) and 769 packet-path occurrences. The adopted rows expand to 147 distinct paths and 807 occurrences; 3.2d has eight paths and 3.2e thirty, with one newly allocated placement_product_tests.rs. The floor correction adds two occurrences of paths already in the union. Mechanical expansion against both complete inventory source lists found no intersection. Recheck at each landing if either inventory changes. The report-anchor amendment expanded 3.2b and 3.2c from 43 to 49 paths each. The earlier 97/34 inventory counts omitted root Cargo.toml; complete counts include it. E/N is relative to first use. No Cargo manifest/dependency, new runner or frozen producer is selected.

| Packet | E/N | Prefix | Basenames |
|---|---|---|---|
| 3.2d | E | `plans/` | `backlog.md` |
| 3.2d | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.2d | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.2d | E | `packages/compiler/src/` | `engine_placement.rs` |
| 3.2d | E | `packages/compiler/src/tests/` | `mod.rs` |
| 3.2d | N | `packages/compiler/src/tests/` | `placement_product_tests.rs` |
| 3.2e | E | `plans/` | `backlog.md` |
| 3.2e | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.2e | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.2e | E | `packages/compiler/src/` | `engine_operation.rs`, `engine_program.rs`, `analyzed.rs`, `analyzed_operation.rs`, `analyzed_validate.rs`, `placement.rs`, `error.rs`, `target.rs`, `coverage.rs` |
| 3.2e | E | `packages/compiler/src/engine_attestation/` | `operation.rs`, `program.rs`, `placement.rs` |
| 3.2e | E | `packages/compiler/src/tests/` | `placement_product_tests.rs`, `analyzed_program_tests.rs`, `analyzed_operation_tests.rs`, `analyzed_corruption_tests.rs`, `pilot_program_tests.rs`, `analyzed_program_oracle_tests.rs`, `burn_plan_oracle_tests.rs`, `pilot_placement_tests.rs`, `pilot_coverage_tests.rs`, `engine_program_tests.rs`, `coverage_tests.rs`, `coverage_oracle_tests.rs`, `source_tests.rs` |
| 3.0 | E | `plans/` | `backlog.md` |
| 3.0 | E | `plans/guides/` | `guide_fifteen.md` |
| 3.0 | E | `plans/phases/` | `07-burn-and-clear.md` |
| 3.0 | E | `plans/packages/` | `compiler.md`, `realization.md` |
| 3.0 | N | `plans/history/` | `wave3-compiler-records.md` |
| 3.0 | E | `plans/history/` | `README.md`, `meson.build` |
| 3.0 | E | `plans/drafts/` | `README.md` |
| 3.1a | E | `plans/` | `backlog.md` |
| 3.1a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.1a | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.1a | E | `packages/compiler/src/` | `engine_lifecycle.rs`, `lifecycle.rs`, `analyzed.rs`, `analyzed_validate.rs`, `operation_plan.rs` |
| 3.1a | E | `packages/compiler/src/engine_attestation/` | `lifecycle.rs`, `proof.rs`, `requirement.rs` |
| 3.1a | E | `packages/compiler/src/tests/` | `engine_lifecycle_tests.rs`, `lifecycle_tests.rs`, `compact_validation_tests.rs`, `analyzed_program_oracle_tests.rs`, `analyzed_corruption_tests.rs`, `requirement_tests.rs`, `operation_plan_tests.rs`, `tracing_boundary_tests.rs` |
| 3.1a | E | `packages/tapscript/src/` | `bundle.rs` |
| 3.1a | E | `packages/tapscript/src/tests/` | `compact_validation_tests.rs` |
| 3.1a | E | `packages/vectors/src/` | `first_party.rs` |
| 3.1a | E | `packages/realization/src/tests/` | `burn_tests.rs`, `live_transfer_tests.rs` |
| 3.1b | E | `plans/` | `backlog.md` |
| 3.1b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.1b | E | `packages/realization/` | `README.md`, `TRACING.md` |
| 3.1b | E | `packages/realization/src/` | `derive.rs`, `declassification.rs`, `error.rs` |
| 3.1b | E | `packages/realization/src/tests/` | `derivation_tests.rs`, `disclosure_tests.rs`, `lifecycle_tests.rs`, `clear_tests.rs`, `tracing_tests.rs`, `tracing_refusal_tests.rs` |
| 3.2a | E | `plans/` | `backlog.md` |
| 3.2a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.2a | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.2a | E | `packages/compiler/src/` | `source.rs`, `coverage.rs`, `error.rs`, `operation_plan.rs` |
| 3.2a | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `identity.rs` |
| 3.2a | E | `packages/compiler/src/tests/` | `tracing_capture.rs`, `tracing_boundary_tests.rs`, `source_tests.rs`, `coverage_tests.rs` |
| 3.2a | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.2a | E | `packages/vectors/src/` | `report.rs`, `violation.rs`, `plan.rs`, `maturity_safety.rs`, `live_safety.rs`, `compact_campaign.rs`, `compact_campaign_second.rs`, `compact_negative_manifest.rs`, `compact_corpus.rs`, `first_party.rs` |
| 3.5c | E | `plans/` | `backlog.md` |
| 3.5c | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5c | E | `packages/compiler/src/` | `engine_program.rs`, `placement.rs`, `layout.rs`, `engine_cases.rs`, `analyzed_operation.rs`, `coverage.rs`, `operation_plan.rs`, `case.rs`, `analyzed_validate.rs`, `carrier.rs`, `source.rs`, `maturity_announcement_plan.rs`, `lifecycle.rs`, `coverage_graph.rs`, `requirement.rs`, `proof.rs`, `error.rs`, `engine_lifecycle.rs`, `engine_requirement.rs`, `engine_proof.rs`, `live_transfer_plan.rs`, `disclosure.rs`, `engine_source.rs`, `engine_case_facts.rs`, `engine_records.rs`, `analyzed.rs` |
| 3.5c | E | `packages/compiler/src/engine_keyed_fixed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5c | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `operation.rs`, `cases.rs`, `placement.rs`, `identity.rs`, `case_facts.rs`, `proof.rs`, `lifecycle.rs` |
| 3.5c | E | `packages/compiler/src/engine_keyed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5c | E | `packages/compiler/src/tests/` | `source_tests.rs`, `coverage_tests.rs`, `coverage_oracle_tests.rs`, `analyzed_program_oracle_tests.rs`, `engine_program_tests.rs`, `engine_lifecycle_tests.rs`, `requirement_tests.rs`, `lifecycle_tests.rs`, `proof_tests.rs`, `compact_validation_tests.rs`, `layout_tests.rs`, `placement_tests.rs`, `case_tests.rs`, `placement_oracle_tests.rs`, `carrier_tests.rs`, `pilot_placement_tests.rs`, `disclosure_tests.rs`, `analyzed_corruption_tests.rs`, `pilot_coverage_tests.rs`, `coverage_graph_tests.rs`, `oracle_tests.rs`, `maturity_announcement_oracle_tests.rs`, `operation_plan_tests.rs`, `maturity_announcement_plan_tests.rs`, `maturity_announcement_corruption_tests.rs`, `engine_proof_tests.rs`, `engine_source_tests.rs`, `engine_cases_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs` |
| 3.5c | E | `packages/tapscript/src/` | `upstream.rs`, `maturity_assessment.rs`, `operation_assessment.rs` |
| 3.5c | E | `packages/tapscript/src/tests/` | `maturity_assessment_tests.rs` |
| 3.5c | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.5c | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.2b | E | `plans/` | `backlog.md` |
| 3.2b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.2b | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.2b | E | `packages/compiler/src/` | `engine_attestation.rs`, `case.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed.rs`, `analyzed_operation.rs`, `analyzed_validate.rs`, `input.rs`, `error.rs`, `engine_requirement.rs`, `engine_case_facts.rs` |
| 3.2b | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `constructibility.rs`, `lifecycle.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs`, `identity.rs` |
| 3.2b | E | `packages/compiler/src/tests/` | `source_tests.rs`, `requirement_tests.rs`, `case_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `compact_validation_tests.rs`, `coverage_oracle_tests.rs` |
| 3.2b | E | `packages/vectors/src/` | `plan.rs`, `compact_negative_manifest.rs` |
| 3.4 | E | `plans/` | `backlog.md` |
| 3.4 | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.4 | E | `packages/compiler/` | `README.md` |
| 3.4 | E | `packages/compiler/src/` | `operation_plan.rs` |
| 3.4 | E | `packages/compiler/src/tests/` | `operation_plan_tests.rs`, `pilot_program_tests.rs`, `placement_oracle_tests.rs`, `coverage_oracle_tests.rs`, `analyzed_program_oracle_tests.rs` |
| 3.4 | E | `packages/linker/src/tests/` | `carrier_tests.rs` |
| 3.4 | E | `packages/vectors/src/` | `plan.rs` |
| 3.3a | E | `plans/` | `backlog.md` |
| 3.3a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.3a | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.3a | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_requirement.rs` |
| 3.3a | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.3a | N | `packages/compiler/src/` | `burn_operation_plan.rs` |
| 3.3a | N | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs` |
| 3.3a | E | `packages/compiler/src/tests/` | `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs` |
| 3.3a | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.3a | E | `packages/realization/` | `README.md` |
| 3.3b | E | `plans/` | `backlog.md` |
| 3.3b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.3b | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.3b | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `burn_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_requirement.rs` |
| 3.3b | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.3b | E | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs` |
| 3.3b | N | `packages/compiler/src/tests/` | `burn_plan_oracle_tests.rs` |
| 3.3b | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.3b | E | `packages/realization/` | `README.md` |
| 3.2c | E | `plans/` | `backlog.md` |
| 3.2c | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.2c | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.2c | E | `packages/compiler/src/` | `engine_attestation.rs`, `case.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed.rs`, `analyzed_operation.rs`, `analyzed_validate.rs`, `input.rs`, `error.rs`, `engine_requirement.rs`, `engine_case_facts.rs` |
| 3.2c | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `constructibility.rs`, `lifecycle.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs`, `identity.rs` |
| 3.2c | E | `packages/compiler/src/tests/` | `source_tests.rs`, `requirement_tests.rs`, `case_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `compact_validation_tests.rs`, `coverage_oracle_tests.rs` |
| 3.2c | E | `packages/vectors/src/` | `plan.rs`, `compact_negative_manifest.rs` |
| 3.5a | E | `plans/` | `backlog.md` |
| 3.5a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5a | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5a | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `engine_requirement.rs` |
| 3.5a | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.5a | N | `packages/compiler/src/` | `clear_operation_plan.rs` |
| 3.5a | N | `packages/compiler/src/tests/` | `clear_operation_plan_tests.rs` |
| 3.5a | E | `packages/compiler/src/tests/` | `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `maturity_announcement_plan_tests.rs`, `maturity_announcement_corruption_tests.rs`, `maturity_announcement_oracle_tests.rs`, `operation_plan_tests.rs` |
| 3.5a | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5a | E | `packages/realization/` | `README.md` |
| 3.5b | E | `plans/` | `backlog.md` |
| 3.5b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5b | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5b | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `clear_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `engine_requirement.rs` |
| 3.5b | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.5b | E | `packages/compiler/src/tests/` | `clear_operation_plan_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `maturity_announcement_plan_tests.rs`, `maturity_announcement_corruption_tests.rs`, `maturity_announcement_oracle_tests.rs`, `operation_plan_tests.rs` |
| 3.5b | N | `packages/compiler/src/tests/` | `clear_plan_oracle_tests.rs` |
| 3.5b | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5b | E | `packages/realization/` | `README.md` |
| 3.5d | E | `plans/` | `backlog.md` |
| 3.5d | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5d | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5d | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `burn_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_program.rs`, `engine_cases.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `lifecycle.rs`, `proof.rs`, `engine_lifecycle.rs`, `engine_requirement.rs`, `engine_proof.rs`, `disclosure.rs`, `engine_source.rs`, `engine_case_facts.rs`, `engine_records.rs` |
| 3.5d | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs`, `identity.rs` |
| 3.5d | E | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs`, `burn_plan_oracle_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs` |
| 3.5d | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5d | E | `packages/realization/` | `README.md`, `TRACING.md` |
| 3.5d | E | `packages/compiler/src/engine_keyed_fixed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5d | E | `packages/compiler/src/engine_keyed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5d | E | `packages/realization/src/declarations/` | `burn.rs` |
| 3.5d | E | `packages/realization/src/` | `burn_clear_laws.rs`, `validate.rs` |
| 3.5d | E | `packages/realization/src/tests/` | `burn_tests.rs`, `burn_clear_relation_tests.rs` |
| 3.5d | E | `plans/packages/` | `realization.md`, `compiler.md` |
| 3.5e | E | `plans/` | `backlog.md` |
| 3.5e | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5e | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5e | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `burn_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_program.rs`, `engine_cases.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `lifecycle.rs`, `proof.rs`, `engine_lifecycle.rs`, `engine_requirement.rs`, `engine_proof.rs`, `disclosure.rs`, `engine_source.rs`, `engine_case_facts.rs`, `engine_records.rs` |
| 3.5e | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs`, `identity.rs` |
| 3.5e | E | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs`, `burn_plan_oracle_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs` |
| 3.5e | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5e | E | `packages/realization/` | `README.md` |
| 3.5e | E | `packages/compiler/src/engine_keyed_fixed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5e | E | `packages/compiler/src/engine_keyed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5e | N | `packages/compiler/src/tests/` | `burn_mixed_plan_tests.rs` |
| 3.5f | E | `plans/` | `backlog.md` |
| 3.5f | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5f | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5f | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `burn_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_program.rs`, `engine_cases.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `lifecycle.rs`, `proof.rs`, `engine_lifecycle.rs`, `engine_requirement.rs`, `engine_proof.rs`, `disclosure.rs`, `engine_source.rs`, `engine_case_facts.rs`, `engine_records.rs` |
| 3.5f | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs`, `identity.rs` |
| 3.5f | E | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs`, `burn_plan_oracle_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs`, `burn_mixed_plan_tests.rs` |
| 3.5f | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5f | E | `packages/realization/` | `README.md` |
| 3.5f | E | `packages/compiler/src/engine_keyed_fixed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5f | E | `packages/compiler/src/engine_keyed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5f | N | `packages/compiler/src/tests/` | `burn_mixed_oracle_tests.rs` |
| 3.6a | E | `plans/` | `backlog.md` |
| 3.6a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.6a | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.6a | E | `packages/compiler/src/` | `lib.rs`, `operation_plan.rs`, `burn_operation_plan.rs`, `clear_operation_plan.rs`, `error.rs`, `analyzed.rs`, `analyzed_validate.rs`, `live_transfer_plan.rs`, `maturity_announcement_plan.rs`, `engine_lifecycle.rs`, `engine_requirement.rs` |
| 3.6a | N | `packages/compiler/src/` | `phase7_plan.rs` |
| 3.6a | E | `packages/compiler/src/engine_attestation/` | `requirement.rs`, `lifecycle.rs` |
| 3.6a | N | `packages/compiler/src/tests/` | `phase7_plan_tests.rs` |
| 3.6a | E | `packages/compiler/src/tests/` | `mod.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `live_transfer_plan_tests.rs`, `maturity_announcement_plan_tests.rs` |
| 3.6a | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.6b | E | `plans/` | `backlog.md` |
| 3.6b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.6b | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.6b | E | `packages/compiler/src/` | `lib.rs`, `operation_plan.rs`, `burn_operation_plan.rs`, `clear_operation_plan.rs`, `error.rs`, `analyzed.rs`, `analyzed_validate.rs`, `phase7_plan.rs`, `live_transfer_plan.rs`, `maturity_announcement_plan.rs`, `engine_lifecycle.rs`, `engine_requirement.rs` |
| 3.6b | E | `packages/compiler/src/engine_attestation/` | `requirement.rs`, `lifecycle.rs` |
| 3.6b | E | `packages/compiler/src/tests/` | `phase7_plan_tests.rs`, `mod.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `live_transfer_plan_tests.rs`, `maturity_announcement_plan_tests.rs` |
| 3.6b | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.6b | N | `packages/compiler/src/tests/` | `phase7_plan_oracle_tests.rs` |
| 3.6b | E | `packages/realization/src/tests/` | `derivation_tests.rs`, `lifecycle_tests.rs` |
| 3.7 | E | `plans/` | `backlog.md` |
| 3.7 | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.7 | E | `packages/vectors/src/` | `first_party.rs`, `compact_negative_manifest.rs`, `compact_corpus.rs`, `maturity_mutants.rs`, `maturity_safety.rs` |
| 3.7 | E | `packages/vectors/src/maturity_mutants/` | `native_staging.rs` |
| 3.7 | E | `packages/compiler/src/tests/` | `lifecycle_tests.rs` |
| 3.7 | E | `packages/tapscript/src/tests/` | `compact_validation_tests.rs` |
| 3.8 | E | `plans/` | `backlog.md` |
| 3.8 | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.8 | E | `plans/guides/` | `guide_fifteen.md` |
| 3.8 | E | `plans/phases/` | `07-burn-and-clear.md` |
| 3.8 | E | `plans/packages/` | `compiler.md`, `realization.md` |

### Wave 4 — Target and representation closure · `task:guide15-exec:wave4`

**Record (Wave 4 semantic and delivery reference)** · `rec:guide15-exec:wave4-reference`

The retained semantic and delivery reference is (`task:guide15:wave3`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 4 — public ASH representation assessment)** · `task:guide15-exec:wave4-deliverable-1`

- public ASH representation assessment. (`task:guide15:wave3`).

**Task (Wave 4 — burn owner-sighash assessment)** · `task:guide15-exec:wave4-deliverable-2`

- burn owner-sighash assessment. (`task:guide15:wave3`).

**Task (Wave 4 — clear arithmetic capability assessment)** · `task:guide15-exec:wave4-deliverable-3`

- clear arithmetic capability assessment. (`task:guide15:wave3`).

**Task (Wave 4 — event-carrier assessment)** · `task:guide15-exec:wave4-deliverable-4`

- event-carrier assessment. (`task:guide15:wave3`).

**Task (Wave 4 — STATE constructor-generation assessment)** · `task:guide15-exec:wave4-deliverable-5`

- STATE constructor-generation assessment. (`task:guide15:wave3`).

**Task (Wave 4 — exact target evidence roles)** · `task:guide15-exec:wave4-deliverable-6`

- exact target evidence roles. (`task:guide15:wave3`).

**Task (Wave 4 — private burn route disposition)** · `task:guide15-exec:wave4-deliverable-7`

- private burn route disposition. (`task:guide15:wave3`).

**Assumption (wave4 — Inherited inputs)** · `assum:guide15-exec:wave4-inherited-inputs`

Wave 0 supplies explicit-only policy, arithmetic assessment, and the pinned target from 0.1 and 0.7; Wave 1 supplies clause-cost and standardness methods from 1.0 and 1.6. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave3`).

**Gate (wave4 — Gate)** · `gate:guide15-exec:wave4-gate`

Each selected form and capability has a reviewed assessment and exact evidence role; unsupported opening and unadmitted private routes remain explicit refusals or non-claims. Measurement records must bind the complete attestation carrier transactions under (`cond:guide15-exec:carrier-reopening`), including all outputs, proofs, weight, fees and policy outcomes, including any confidential change outputs with their proofs; any refusal requires a Wave-4 ruling before target declarations, naming the Wave-2 typed declarations and Wave-3 plans it reopens. (`task:guide15:wave3`).

**Proposal (wave4 — Suggested commit)** · `proposal:guide15-exec:wave4-suggested-commit`

`target-elements: assess burn ASH and clear requirements`. (`task:guide15:wave3`).

### Wave 5 — Public ASH constructor integration · `task:guide15-exec:wave5`

**Record (Wave 5 semantic and delivery reference)** · `rec:guide15-exec:wave5-reference`

The retained semantic and delivery reference is (`task:guide15:wave4`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 5 — exact public ASH constructor)** · `task:guide15-exec:wave5-deliverable-1`

- exact public ASH constructor. (`task:guide15:wave4`).

**Task (Wave 5 — burn-created ASH output pattern)** · `task:guide15-exec:wave5-deliverable-2`

- burn-created ASH output pattern. (`task:guide15:wave4`).

**Task (Wave 5 — compact compatibility)** · `task:guide15-exec:wave5-deliverable-3`

- compact compatibility. (`task:guide15:wave4`).

**Task (Wave 5 — clear input and residual compatibility)** · `task:guide15-exec:wave5-deliverable-4`

- clear input and residual compatibility. (`task:guide15:wave4`).

**Task (Wave 5 — ownerless and permissionless guarantees)** · `task:guide15-exec:wave5-deliverable-5`

- ownerless and permissionless guarantees. (`task:guide15:wave4`).

**Task (Wave 5 — constructor mutation vectors)** · `task:guide15-exec:wave5-deliverable-6`

- constructor mutation vectors. (`task:guide15:wave4`).

**Task (Wave 5 — public construction tests)** · `task:guide15-exec:wave5-deliverable-7`

- public construction tests. (`task:guide15:wave4`).

**Assumption (wave5 — Inherited inputs)** · `assum:guide15-exec:wave5-inherited-inputs`

Wave 0 supplies the accepted constructor equality or class-transition disposition and compact baseline from 0.4 and 0.7; Wave 1 supplies the public recurrence constructor from 1.3. The native route is inherited from Phase 6 and revalidated at 0.1, while ASH aggregate and lifecycle laws remain this wave's obligations (`sec:phase6:gate-record`). (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave4`).

**Gate (wave5 — Gate)** · `gate:guide15-exec:wave5-gate`

Fresh, compacted, and residual ASH share the exact proved constructor or explicit admitted transitions, with mutations refused and unrelated-process construction reproduced. (`task:guide15:wave4`).

**Proposal (wave5 — Suggested commit)** · `proposal:guide15-exec:wave5-suggested-commit`

`tapscript: integrate the public ASH constructor`. (`task:guide15:wave4`).

### Wave 6 — Burn patterns · `task:guide15-exec:wave6`

**Record (Wave 6 semantic and delivery reference)** · `rec:guide15-exec:wave6-reference`

The retained semantic and delivery reference is (`task:guide15:wave5`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 6 — receipt recognition)** · `task:guide15-exec:wave6-deliverable-1`

- receipt recognition. (`task:guide15:wave5`).

**Task (Wave 6 — every-owner authorization)** · `task:guide15-exec:wave6-deliverable-2`

- every-owner authorization. (`task:guide15:wave5`).

**Task (Wave 6 — unique fresh ASH)** · `task:guide15-exec:wave6-deliverable-3`

- unique fresh ASH. (`task:guide15:wave5`).

**Task (Wave 6 — live-change closure)** · `task:guide15-exec:wave6-deliverable-4`

- live-change closure. (`task:guide15:wave5`).

**Task (Wave 6 — exact burn aggregate)** · `task:guide15-exec:wave6-deliverable-5`

- exact burn aggregate. (`task:guide15:wave5`).

**Task (Wave 6 — burn-record carrier)** · `task:guide15-exec:wave6-deliverable-6`

- burn-record carrier. (`task:guide15:wave5`).

**Task (Wave 6 — sponsor isolation)** · `task:guide15-exec:wave6-deliverable-7`

- sponsor isolation. (`task:guide15:wave5`).

**Task (Wave 6 — root, issuance, and destruction absence)** · `task:guide15-exec:wave6-deliverable-8`

- root, issuance, and destruction absence. (`task:guide15:wave5`).

**Task (Wave 6 — burn event placement)** · `task:guide15-exec:wave6-deliverable-9`

- burn event placement. (`task:guide15:wave5`).

**Task (Wave 6 — complete abstract schedules)** · `task:guide15-exec:wave6-deliverable-10`

- complete abstract schedules. (`task:guide15:wave5`).

**Assumption (wave6 — Inherited inputs)** · `assum:guide15-exec:wave6-inherited-inputs`

Wave 0 supplies exact burn law, region-refit evidence, and carrier disposition from 0.2 and 0.7; Wave 1 supplies clause-cost and standardness methods from 1.0 and 1.6; Wave 4 supplies the actual attestation carrier measurements. Neither supplies owner authorization or record-credit semantics. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave5`).

**Gate (wave6 — Gate)** · `gate:guide15-exec:wave6-gate`

Every burn relation has an exact emitted or evidence carrier, owner authorization binds the intended burn, and complete schedules pass stack/resource and focused refusal checks. (`task:guide15:wave5`).

**Proposal (wave6 — Suggested commit)** · `proposal:guide15-exec:wave6-suggested-commit`

`tapscript: implement owner-authorized burn`. (`task:guide15:wave5`).

### Wave 7 — Phase-7 STATE constructor and clear patterns · `task:guide15-exec:wave7`

**Record (Wave 7 semantic and delivery reference)** · `rec:guide15-exec:wave7-reference`

The retained semantic and delivery reference is (`task:guide15:wave6`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 7 — expanded maturity-plus-clear static subtree)** · `task:guide15-exec:wave7-deliverable-1`

- expanded maturity-plus-clear static subtree. (`task:guide15:wave6`).

**Task (Wave 7 — maturity regression under the new generation)** · `task:guide15-exec:wave7-deliverable-2`

- maturity regression under the new generation. (`task:guide15:wave6`).

**Task (Wave 7 — clear STATE recognition)** · `task:guide15-exec:wave7-deliverable-3`

- clear STATE recognition. (`task:guide15:wave6`).

**Task (Wave 7 — public ASH aggregate)** · `task:guide15-exec:wave7-deliverable-4`

- public ASH aggregate. (`task:guide15:wave6`).

**Task (Wave 7 — exact minimum computation)** · `task:guide15-exec:wave7-deliverable-5`

- exact minimum computation. (`task:guide15:wave6`).

**Task (Wave 7 — successor STATE reconstruction)** · `task:guide15-exec:wave7-deliverable-6`

- successor STATE reconstruction. (`task:guide15:wave6`).

**Task (Wave 7 — exact destruction)** · `task:guide15-exec:wave7-deliverable-7`

- exact destruction. (`task:guide15:wave6`).

**Task (Wave 7 — residual branch)** · `task:guide15-exec:wave7-deliverable-8`

- residual branch. (`task:guide15:wave6`).

**Task (Wave 7 — no RESV)** · `task:guide15-exec:wave7-deliverable-9`

- no RESV. (`task:guide15:wave6`).

**Task (Wave 7 — clear event)** · `task:guide15-exec:wave7-deliverable-10`

- clear event. (`task:guide15:wave6`).

**Task (Wave 7 — final-stack and resource formulas)** · `task:guide15-exec:wave7-deliverable-11`

- final-stack and resource formulas. (`task:guide15:wave6`).

**Assumption (wave7 — Inherited inputs)** · `assum:guide15-exec:wave7-inherited-inputs`

Wave 0 supplies the clear assignment and generation policy from 0.7; Wave 1 supplies the authenticated-program constructor and final-form records from 1.3 and 1.6. The Phase-6 native route is revalidated at 0.1, while STATE arithmetic and maturity-plus-clear closure remain this wave's proofs (`sec:phase6:gate-record`). (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave6`).

**Gate (wave7 — Gate)** · `gate:guide15-exec:wave7-gate`

Every authenticated minimum branch, floor, decrement, destruction, residual, and unchanged field is proved; maturity passes under the expanded subtree and cross-generation spends refuse. (`task:guide15:wave6`).

**Proposal (wave7 — Suggested commit)** · `proposal:guide15-exec:wave7-suggested-commit`

`tapscript: implement permissionless clear`. (`task:guide15:wave6`).

### Wave 8 — Linker extension · `task:guide15-exec:wave8`

**Record (Wave 8 semantic and delivery reference)** · `rec:guide15-exec:wave8-reference`

The retained semantic and delivery reference is (`task:guide15:wave7`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 8 — burn, ASH, event, destruction, and Phase-7 STATE symbols)** · `task:guide15-exec:wave8-deliverable-1`

- burn, ASH, event, destruction, and Phase-7 STATE symbols. (`task:guide15:wave7`).

**Task (Wave 8 — two-pass resolution)** · `task:guide15-exec:wave8-deliverable-2`

- two-pass resolution. (`task:guide15:wave7`).

**Task (Wave 8 — SCC disposition)** · `task:guide15-exec:wave8-deliverable-3`

- SCC disposition. (`task:guide15:wave7`).

**Task (Wave 8 — structured relocations)** · `task:guide15-exec:wave8-deliverable-4`

- structured relocations. (`task:guide15:wave7`).

**Task (Wave 8 — duplicate-sensitive trees)** · `task:guide15-exec:wave8-deliverable-5`

- duplicate-sensitive trees. (`task:guide15:wave7`).

**Task (Wave 8 — exact tree costs)** · `task:guide15-exec:wave8-deliverable-6`

- exact tree costs. (`task:guide15:wave7`).

**Task (Wave 8 — cross-operation ASH closure)** · `task:guide15-exec:wave8-deliverable-7`

- cross-operation ASH closure. (`task:guide15:wave7`).

**Task (Wave 8 — candidate linked Phase-7 bundle)** · `task:guide15-exec:wave8-deliverable-8`

- candidate linked Phase-7 bundle. (`task:guide15:wave7`).

**Assumption (wave8 — Inherited inputs)** · `assum:guide15-exec:wave8-inherited-inputs`

Wave 0 supplies the closure-exact audit and candidate bindings from 0.1 and 0.3; Wave 1 supplies the linked reference-closure record from 1.4, with every attestation symbol, relocation, constructor, and cost still derived for this bundle. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave7`).

**Gate (wave8 — Gate)** · `gate:guide15-exec:wave8-gate`

Every symbol and relocation resolves exactly once, cycles have supported dispositions, duplicate-sensitive tree/control recipes are deterministic, and complete bundle costs reconcile. (`task:guide15:wave7`).

**Proposal (wave8 — Suggested commit)** · `proposal:guide15-exec:wave8-suggested-commit`

`linker: link the burn ASH and clear candidate`. (`task:guide15:wave7`).

### Wave 9 — Candidate ABIs and burn signing · `task:guide15-exec:wave9`

**Record (Wave 9 semantic and delivery reference)** · `rec:guide15-exec:wave9-reference`

The retained semantic and delivery reference is (`task:guide15:wave8`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 9 — burn ABI)** · `task:guide15-exec:wave9-deliverable-1`

- burn ABI. (`task:guide15:wave8`).

**Task (Wave 9 — inherited compact ABI revalidated)** · `task:guide15-exec:wave9-deliverable-2`

- inherited compact ABI revalidated. (`task:guide15:wave8`).

**Task (Wave 9 — clear ABI)** · `task:guide15-exec:wave9-deliverable-3`

- clear ABI. (`task:guide15:wave8`).

**Task (Wave 9 — typed requests and public views)** · `task:guide15-exec:wave9-deliverable-4`

- typed requests and public views. (`task:guide15:wave8`).

**Task (Wave 9 — exact family layouts)** · `task:guide15-exec:wave9-deliverable-5`

- exact family layouts. (`task:guide15:wave8`).

**Task (Wave 9 — burn finalization)** · `task:guide15-exec:wave9-deliverable-6`

- burn finalization. (`task:guide15:wave8`).

**Task (Wave 9 — multi-owner signing)** · `task:guide15-exec:wave9-deliverable-7`

- multi-owner signing. (`task:guide15:wave8`).

**Task (Wave 9 — sponsor consistency)** · `task:guide15-exec:wave9-deliverable-8`

- sponsor consistency. (`task:guide15:wave8`).

**Task (Wave 9 — canonical bytes)** · `task:guide15-exec:wave9-deliverable-9`

- canonical bytes. (`task:guide15:wave8`).

**Task (Wave 9 — post-signing mutation refusal)** · `task:guide15-exec:wave9-deliverable-10`

- post-signing mutation refusal. (`task:guide15:wave8`).

**Task (Wave 9 — candidate-only status)** · `task:guide15-exec:wave9-deliverable-11`

- candidate-only status. (`task:guide15:wave8`).

**Assumption (wave9 — Inherited inputs)** · `assum:guide15-exec:wave9-inherited-inputs`

Wave 0 supplies public-construction and signing-profile dispositions from 0.7; Wave 1 supplies the typed output-0 handle ABI and construction requests from 1.4, while all-owner burn signing and exact sponsor consistency remain attestation obligations. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave8`).

**Gate (wave9 — Gate)** · `gate:guide15-exec:wave9-gate`

The final ABI and bytes are canonical, all owners authorize the same finalized burn, post-signing mutation refuses, and compact/clear public construction remains permissionless. (`task:guide15:wave8`).

**Proposal (wave9 — Suggested commit)** · `proposal:guide15-exec:wave9-suggested-commit`

`transaction: derive burn ASH and clear ABIs`. (`task:guide15:wave8`).

### Wave 10 — Burn safety and event evidence · `task:guide15-exec:wave10`

**Record (Wave 10 semantic and delivery reference)** · `rec:guide15-exec:wave10-reference`

The retained semantic and delivery reference is (`task:guide15:wave9`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 10 — canonical burn fixtures)** · `task:guide15-exec:wave10-deliverable-1`

- canonical burn fixtures. (`task:guide15:wave9`).

**Task (Wave 10 — owner, asset, class, value, change, and sponsor matrices)** · `task:guide15-exec:wave10-deliverable-2`

- owner, asset, class, value, change, and sponsor matrices. (`task:guide15:wave9`).

**Task (Wave 10 — burn-record matrix)** · `task:guide15-exec:wave10-deliverable-3`

- burn-record matrix. (`task:guide15:wave9`).

**Task (Wave 10 — event-type and value-anchor evidence)** · `task:guide15-exec:wave10-deliverable-4`

- event-type and value-anchor evidence. (`task:guide15:wave9`).

**Task (Wave 10 — accepted burn projections)** · `task:guide15-exec:wave10-deliverable-5`

- accepted burn projections. (`task:guide15:wave9`).

**Task (Wave 10 — validated burn safety and event reports)** · `task:guide15-exec:wave10-deliverable-6`

- validated burn safety and event reports. (`task:guide15:wave9`).

**Assumption (wave10 — Inherited inputs)** · `assum:guide15-exec:wave10-inherited-inputs`

Wave 0 supplies the event-role census and exact record rule from 0.7; the native route is inherited from Phase 6 and revalidated at 0.1. Wave 1 supplies the distinct validated evidence-family reports from 1.5, without substituting SOURCE membership for burn provenance (`sec:phase6:gate-record`). (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave9`).

**Gate (wave10 — Gate)** · `gate:guide15-exec:wave10-gate`

Operation acceptance, event type, and record value are independently validated; copied or over-claiming records cannot create provenance or erase a real burn event. (`task:guide15:wave9`).

**Proposal (wave10 — Suggested commit)** · `proposal:guide15-exec:wave10-suggested-commit`

`vectors: complete burn and burn-event evidence`. (`task:guide15:wave9`).

### Wave 11 — ASH maintenance evidence · `task:guide15-exec:wave11`

**Record (Wave 11 semantic and delivery reference)** · `rec:guide15-exec:wave11-reference`

The retained semantic and delivery reference is (`task:guide15:wave10`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 11 — fresh, compacted, and residual ASH fixtures)** · `task:guide15-exec:wave11-deliverable-1`

- fresh, compacted, and residual ASH fixtures. (`task:guide15:wave10`).

**Task (Wave 11 — compact target execution)** · `task:guide15-exec:wave11-deliverable-2`

- compact target execution. (`task:guide15:wave10`).

**Task (Wave 11 — permissionless unrelated-process construction)** · `task:guide15-exec:wave11-deliverable-3`

- permissionless unrelated-process construction. (`task:guide15:wave10`).

**Task (Wave 11 — event-silence checks)** · `task:guide15-exec:wave11-deliverable-4`

- event-silence checks. (`task:guide15:wave10`).

**Task (Wave 11 — constructor compatibility)** · `task:guide15-exec:wave11-deliverable-5`

- constructor compatibility. (`task:guide15:wave10`).

**Task (Wave 11 — lifecycle evidence)** · `task:guide15-exec:wave11-deliverable-6`

- lifecycle evidence. (`task:guide15:wave10`).

**Assumption (wave11 — Inherited inputs)** · `assum:guide15-exec:wave11-inherited-inputs`

Wave 0 supplies compact negative closure and public ASH policy from 0.4 and 0.7; the pinned native route is inherited from Phase 6 and revalidated at 0.1. Wave 1 supplies the public-construction ABI from 1.4, while compact event silence remains independently proved (`sec:phase6:gate-record`). (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave10`).

**Gate (wave11 — Gate)** · `gate:guide15-exec:wave11-gate`

Native compact execution preserves value and silence, and an unrelated process can reuse fresh, compacted, and residual ASH without hidden creator state. (`task:guide15:wave10`).

**Proposal (wave11 — Suggested commit)** · `proposal:guide15-exec:wave11-suggested-commit`

`vectors: verify public ASH maintenance`. (`task:guide15:wave10`).

### Wave 12 — Clear safety and STATE evidence · `task:guide15-exec:wave12`

**Record (Wave 12 semantic and delivery reference)** · `rec:guide15-exec:wave12-reference`

The retained semantic and delivery reference is (`task:guide15:wave11`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 12 — every minimum branch)** · `task:guide15-exec:wave12-deliverable-1`

- every minimum branch. (`task:guide15:wave11`).

**Task (Wave 12 — STATE assignment matrix)** · `task:guide15-exec:wave12-deliverable-2`

- STATE assignment matrix. (`task:guide15:wave11`).

**Task (Wave 12 — destruction and residual matrices)** · `task:guide15-exec:wave12-deliverable-3`

- destruction and residual matrices. (`task:guide15:wave11`).

**Task (Wave 12 — no-RESV and root closure)** · `task:guide15-exec:wave12-deliverable-4`

- no-RESV and root closure. (`task:guide15:wave11`).

**Task (Wave 12 — clear event)** · `task:guide15-exec:wave12-deliverable-5`

- clear event. (`task:guide15:wave11`).

**Task (Wave 12 — constructor continuity)** · `task:guide15-exec:wave12-deliverable-6`

- constructor continuity. (`task:guide15:wave11`).

**Task (Wave 12 — root-history edge)** · `task:guide15-exec:wave12-deliverable-7`

- root-history edge. (`task:guide15:wave11`).

**Task (Wave 12 — public successor recovery)** · `task:guide15-exec:wave12-deliverable-8`

- public successor recovery. (`task:guide15:wave11`).

**Task (Wave 12 — validated reports)** · `task:guide15-exec:wave12-deliverable-9`

- validated reports. (`task:guide15:wave11`).

**Assumption (wave12 — Inherited inputs)** · `assum:guide15-exec:wave12-inherited-inputs`

Wave 0 supplies exact independent-observation, region, and recovery records from 0.2 and 0.5; Wave 1 supplies the final-subject execution records and validated reports from 1.5 and 1.6, while clear accounting and STATE edge proofs remain new. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave11`).

**Gate (wave12 — Gate)** · `gate:guide15-exec:wave12-gate`

All clear branches and focused mutations validate their declared boundaries; one exact STATE edge, no RESV edge, exact accounting, and public recovery have separate validated reports. (`task:guide15:wave11`).

**Proposal (wave12 — Suggested commit)** · `proposal:guide15-exec:wave12-suggested-commit`

`vectors: complete clear STATE and history evidence`. (`task:guide15:wave11`).

### Wave 13 — Resource study · `task:guide15-exec:wave13`

**Record (Wave 13 semantic and delivery reference)** · `rec:guide15-exec:wave13-reference`

The retained semantic and delivery reference is (`task:guide15:wave12`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 13 — objective-specific burn, compact, and clear measurements)** · `task:guide15-exec:wave13-deliverable-1`

- objective-specific burn, compact, and clear measurements. (`task:guide15:wave12`).

**Task (Wave 13 — expanded STATE-constructor measurements)** · `task:guide15-exec:wave13-deliverable-2`

- expanded STATE-constructor measurements. (`task:guide15:wave12`).

**Task (Wave 13 — exact prediction/observation comparison)** · `task:guide15-exec:wave13-deliverable-3`

- exact prediction/observation comparison. (`task:guide15:wave12`).

**Task (Wave 13 — candidate bound study)** · `task:guide15-exec:wave13-deliverable-4`

- candidate bound study. (`task:guide15:wave12`).

**Task (Wave 13 — noncanonical timing)** · `task:guide15-exec:wave13-deliverable-5`

- noncanonical timing. (`task:guide15:wave12`).

**Task (Wave 13 — explicit non-calibration result)** · `task:guide15-exec:wave13-deliverable-6`

- explicit non-calibration result. (`task:guide15:wave12`).

**Assumption (wave13 — Inherited inputs)** · `assum:guide15-exec:wave13-inherited-inputs`

Wave 0 supplies the gate-performance register from 0.6 and the distinct inherited target-resource subjects retained at 0.5 (`data:guide15-exec:resource-subjects`); Wave 1 supplies keyed-covenant cost and standardness records from 1.0 and 1.6; Wave 4 supplies attestation carrier measurements. Complete attestation subjects require remeasurement; gate wall time is not target-resource evidence. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave12`).

**Gate (wave13 — Gate)** · `gate:guide15-exec:wave13-gate`

Complete burn/compact/clear subjects and expanded STATE paths have predicted-versus-observed resources, stated candidate bounds, timing kept noncanonical, and absent dimensions and non-calibration explicit. (`task:guide15:wave12`).

**Proposal (wave13 — Suggested commit)** · `proposal:guide15-exec:wave13-suggested-commit`

`vectors: measure the burn ASH and clear candidate`. (`task:guide15:wave12`).

### Wave 14 — Phase-7 gate and handoff · `task:guide15-exec:wave14`

**Record (Wave 14 semantic and delivery reference)** · `rec:guide15-exec:wave14-reference`

The retained semantic and delivery reference is (`task:guide15:wave13`); the implementation result belongs to this execution wave and its actual candidate.

**Task (Wave 14 — package READMEs and contracts)** · `task:guide15-exec:wave14-deliverable-1`

- package READMEs and contracts. (`task:guide15:wave13`).

**Task (Wave 14 — Phase-7 card)** · `task:guide15-exec:wave14-deliverable-2`

- Phase-7 card. (`task:guide15:wave13`).

**Task (Wave 14 — backlog gate record)** · `task:guide15-exec:wave14-deliverable-3`

- backlog gate record. (`task:guide15:wave13`).

**Task (Wave 14 — complete result matrix)** · `task:guide15-exec:wave14-deliverable-4`

- complete result matrix. (`task:guide15:wave13`).

**Task (Wave 14 — identity, schema, security, interchange, and dependency impact)** · `task:guide15-exec:wave14-deliverable-5`

- identity, schema, security, interchange, and dependency impact. (`task:guide15:wave13`).

**Task (Wave 14 — complete repository gate)** · `task:guide15-exec:wave14-deliverable-6`

- complete repository gate. (`task:guide15:wave13`).

**Task (Wave 14 — clean final tree)** · `task:guide15-exec:wave14-deliverable-7`

- clean final tree. (`task:guide15:wave13`).

**Assumption (wave14 — Inherited inputs)** · `assum:guide15-exec:wave14-inherited-inputs`

Wave 0 supplies closed debt and accepted preflight records, and Wave 1 supplies final-form second-contract independence evidence; neither replaces any conjunct of (`gate:phase7:exit`), (`gate:guide15:exit`), or (`sec:guide15:acceptance`). (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave13`).

**Gate (wave14 — Gate)** · `gate:guide15-exec:wave14-gate`

The ten bullets of the Phase-7 exit gate hold on their text, the complete result matrix and impact assessment are recorded, and the full repository gate passes on a reproducible clean tree. The named phase gate (`gate:phase7:exit`), the concept exit (`gate:guide15:exit`), and acceptance criteria (`sec:guide15:acceptance`) must all hold, including the identity-drift and byte-drift checks absent from the phase gate.

**Proposal (wave14 — Suggested commit)** · `proposal:guide15-exec:wave14-suggested-commit`

`plans: record the burn ASH and clear candidate`. (`task:guide15:wave13`).

## 6. Wave gates and the baseline rule · `rule:guide15-exec:baseline`

**Rule (Wave opening law)** · `rule:guide15-exec:wave-opening`

A wave opens only on the previous wave's exit gate; Wave 0 is the initial exception to predecessor numbering and opens only on its own backlog entry record with every entry obligation recorded.

**Obligation (Comparable baseline figures)** · `obl:guide15-exec:baseline-comparison`

Every wave's gate record states its figures beside the Wave-0 baseline; a regression against that baseline is a finding requiring an explicit disposition before advancement, not an adjustment hidden in a new total.

**Definition (A baseline comparison subject)** · `def:guide15-exec:baseline-subject`

The baseline records exact target, deployment, policy, candidate, protocol, constructor generation, evidence corpus, commands, clean-tree state, and measured dimensions so that comparisons have matching subjects.

**Definition (Separate evidence standings)** · `def:guide15-exec:baseline-standings`

Each record distinguishes answered rows, typed retirements, still-required evidence, and inapplicable claims; a retyped row never becomes an observed refusal merely because it leaves an outstanding bucket.

**Inventory (Separately countable baseline figures)** · `reg:guide15-exec:baseline-inventory`

The Safety census, relation-link census, four backlog carriers, five conjunct dispositions, compact negative matrix, twelve questions, and 65 entry rows remain separately countable throughout the later waves (`tab:guide15-exec:baseline-comparisons`).

**Condition (Changed premises require revalidation)** · `cond:guide15-exec:baseline-revalidation`

An altered target, deployment, policy, constructor, ABI, or protocol requires an explicit compatibility argument and rerun of the affected evidence; a historical success does not automatically cover the new subject.

**Rule (Focused verification and final gates)** · `rule:guide15-exec:verification-order`

Each wave retains focused verification for its changed packages and relation boundaries, then the repository gate required by its exit; the final batch gate remains (`gate:guide15:batch`) and Phase 7 exits only when (`gate:phase7:exit`), (`gate:guide15:exit`), and (`sec:guide15:acceptance`) all hold, with the second-contract dependency retained (`rule:guide15-exec:second-contract-dependency`).

**Requirement (Completion evidence in gate records)** · `req:guide15-exec:gate-record-evidence`

Gate records state commands, observed results, exit codes, durations, retained logs, and absent dimensions; a rendered report, successful process start, or returned control without completion evidence is not a passing run.

**Rule (Exact reuse arguments remain visible)** · `rule:guide15-exec:inherited-deliverables`

An inherited deliverable remains in the consuming wave's inventory with its source and exact reuse argument; changed premises require revalidation rather than deletion of the obligation.

**Specification (Independent result-matrix subjects)** · `spec:guide15-exec:result-matrix`

The complete result matrix records burn, compact, clear, representation, constructor, event, accounting, public construction, coverage, and resources independently (`tab:guide15:result`); second-contract evidence adds independence and replaces none of those rows.

**Definition (Regression requires comparable subjects)** · `def:guide15-exec:regression`

A regression is a loss of previously admitted coverage, closure, or reproducibility over the same declared subject, or failure to supply the evidence required for an explicitly changed subject; raw counts over different candidates do not establish a direction by themselves. A changed subject requires the compatibility argument and rerun below before advancement (`rule:guide15-exec:baseline`).

**Table (Baseline subjects, regression directions, and restoring evidence)** · `tab:guide15-exec:baseline-comparisons`

The figure's typed source, exact subject, adverse direction, and restoration obligation remain separate, so replacing a candidate never hides an outstanding proof (`def:guide15-exec:regression`).

| Figure and source | Comparison subject | Regression | Restoring evidence |
|---|---|---|---|
| Safety 206, 78 / 128 (`data:guide15-exec:safety-census`) | Registered rows, boundaries, controls, candidate | Lost admitted answer or new unresolved required row on the same subject | Reclassify every row and rerun affected boundary evidence |
| Relation links 14 / 102 / 90 (`data:guide15-exec:relation-census`) | Published relation identities and operation regions | Lost justified link or unaccounted changed identity | Refit mapping, recount, and composed/fault observations |
| Four debt carriers (`T11-064`); (`T11-063`); (`T4-009`); (`T11-110`) | Each row's own closing criterion | A previously satisfied criterion no longer holds | Reopen the affected criterion and reproduce its evidence |
| Five conjuncts (`reg:guide15-exec:conditional-conjuncts`) | Exact subclaims and conditional assumptions | Lost observation or invalidated assumption without disposition | Repeat affected observations and separately rule residuals |
| Compact 74 rows, partition 48 / 20 / 6; before-figures 72 / 48 / 18 / 6 (`T4-009`) | Selected compact candidate and mutation boundaries | Lost discharge or new observable gap | Current-candidate positive and focused negative campaign |
| Twelve questions (`sec:guide15:preflight`) | Accepted choices and their typed authorities | Invalidated or missing accepted disposition | Numbered acceptance and consuming proof, including carrier reopening |
| Entry 65 rows (`tab:guide15-exec:preflight`) | Finding scope and assigned execution exits | Lost evidence or unmet consuming obligation | Row-specific revalidation with finding and execution separate |
| Resource 53 / 86 / 80 bytes and weight 2,091 (`data:guide15-exec:resource-subjects`) | Exact schedule, bundle, ABI, target, and policy | Prediction mismatch or refusal of the previously admitted form | Remeasure changed forms, retaining absent dimensions and replay-only bounds |

**Open Question (Gate-wall regression standing)** · `open:guide15-exec:gate-wall-baseline`

Whether gate wall time joins the baseline as a regression figure remains open: inclusion exposes slower verification, while exclusion avoids treating node load as semantic regression. The performance register remains required evidence under (`T11-110`), without settling this question from `plans/reviews/guide_fifteen_execution_review.md`, question 9.

## 7. What this guide does not claim · `sec:guide15-exec:non-claims`

**Rule (Chartering is not wave entry)** · `rule:guide15-exec:charter-entry`

Chartering opens no wave by itself: it authorizes Wave-0 scope, whose entry obligations and provisioned target must be recorded before opening; implementation of new semantics remains held until (`gate:guide15-exec:wave0`).

**Fact (Historical closeout remains unchanged)** · `fact:guide15-exec:closeout-scope`

The accepted Phase-6 closeout is unchanged at its amended scope, with its original §23 verdict, outstanding-row figures, candidate boundaries, and finite evidence corpus preserved as historical facts (`sec:phase6:exit-assessment`).

**Caveat (Production and finite-evidence non-claims)** · `cav:guide15-exec:production-non-claims`

No production key custody, production opening custody, production deployment, final calibration, release, universal security claim, or general theorem beyond the finite admitted corpus is claimed; the lack of independent chain observation is an inherited entry limitation that bite 0.5 must discharge for its exact observed subject (`obl:guide15-exec:independent-observation`); (`sec:phase6:exit-assessment`).

**Caveat (The inherited handoff is candidate evidence)** · `cav:guide15-exec:final-handoff-scope`

Guide 14 §26's handoff remains only the carried candidate evidence recorded by (`sec:phase6:exit-assessment`); this guide does not promote it into deployment, release, calibration, or a universal security claim.

**Caveat (Planned work is not evidence)** · `cav:guide15-exec:planned-evidence`

Neither the charter nor a planned target run proves unavailable evidence; no missing-node explanation can satisfy the Wave-0 exit, and finite observations prove no unconditional NUMS impossibility; conditional satisfaction instead rests on (`judg:guide15-exec:nums-standing`).

**Caveat (Second-contract exclusions)** · `cav:guide15-exec:second-contract-limits`

The second contract supplies independence evidence inside Phase 7 under exactly born first and continue first. It has no destruction requirement, carried counter, accumulator, magnitude, depth, designated head or terminal; reader acceptance is outside the covenant (`[A-def:kpc-model:first-output]`); (`[A-rem:kpc-interface:record-evidence]`). These legacy limits do not assert fixed-form value freedom: its single retained deposit and unrestricted merge release are stated at (`spec:guide15-exec:covenant-exclusions`); S9 checks both edition scopes.

**Caveat (Measured subjects and published premises)** · `cav:guide15-exec:measurement-limits`

Its funding, construction bound, transaction standardness and clause costs remain measured realization facts. Inclusion, retrieval and finality are evidence assumptions, and acceptance is a reader's published rule. None is a universal deployment guarantee or an extra covenant clause (`task:guide15-exec:bite-1-0`); (`[A-sec:kpc:interface]`); (`[A-sec:kpc:limitations]`).

**Record (Unchanged product thesis and accepted dependency)** · `rec:guide15-exec:roadmap-position`

The roadmap's thesis is unchanged: the attestation remains the exemplar by which the compiler is accepted, and the second contract tests independence inside the phase under (`sec:phase7:change-statement`); the accepted dependency on Wave 1 is stated under (`rule:guide15-exec:second-contract-dependency`), while the numbered phase order and normative version remain unchanged.

**Rule (Execution authority remains subordinate)** · `rule:guide15-exec:authority-boundary`

The concept remains the attestation semantic reference, the admission study remains planning input, and the earlier second-exemplar study is retained only as the record of the reduction. The Keyed Provenance Covenant paper supplies Wave 1's contract; this execution guide cannot override it, a package contract, phase card, ADR or accepted decision (`[A-abst:kpc:abstract]`); (`rule:guides:authority`); (`rule:drafts:authority`).
