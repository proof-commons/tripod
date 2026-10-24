# Guide 15 — Burn, ASH, and Clear: execution guide

> **Status:** Chartered execution guide of Phase 7; entry is governed by (`gate:guide15-exec:guide-entry`) and (`task:guide15-exec:wave0`), and new semantics remain held until (`gate:guide15-exec:wave0`).
> **Phase:** Phase 7 — Burn, ASH, and Clear (`phase:roadmap:burn-clear`).
> **Entry:** The amended Phase-6 exit (`gate:phase6:exit`), the accepted public-declassification policy (`sec:public-opening:result`), and the accepted Phase-4 compact-ASH candidate at its recorded scope (`phase:roadmap:compact-ash`), subject to the entry register below.
> **Operations:** `burn`, `compact-ash`, and `clear`; the second exemplar's `BURN` and `MERGE` are Wave-1 declarations whose exact typed spellings Wave 1 fixes.
> **Affected packages:** `architecture`, `realization`, `compiler`, `target-elements`, `tapscript`, `linker`, `transaction`, and `vectors`; `target-elements-conformance` only for a demonstrated missing target-generic fact; `model` supplies independent conformance evidence and `labels` checks registrations.
> **Supersedes as execution direction:** `plans/guides/guide_fifteen_concept.md`; its rules, gates, questions, and semantic chapters remain the cited semantic reference, and its artifact changes only by the mints of its entry conditions and record fields.
> **Does not implement:** Redemption, request admission, attestation settlement, cycle, entitlement issuance, distribution construction, RESV or PACE succession, production key custody, production opening custody, production deployment, final calibration, or release.
> **Required result:** The Phase-7 exit gate met on its text (`gate:phase7:exit`), with a second compiled contract carried through the same pipeline as independence evidence.
> **Authority:** Attestation, Realization, typed architecture, implemented ADRs, accepted decisions, and accepted research and phase results take precedence as in `plans/guides/guide_fourteen.md`; the Phase-6 closeout is accepted only at its amended scope (`sec:phase6:exit-assessment`); (`abst:admission:scope`) and (`abst:lineage:scope`) supply planning input.
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

The 35 native-run-required rows and the second contract's measurements require the pinned target on the node of record; one environment, once provisioned and recorded, supports both, while each observation retains its own exact subject and evidence boundary (`sec:phase6:exit-assessment`); (`sec:lineage:measurements`).

**Argument (A separately gated debt baseline isolates later changes)** · `arg:guide15-exec:baseline-order`

A debt wave with its own exit turns the inherited figures into an explicit baseline; later waves compare their figures against that baseline instead of treating a historical green gate as an answer to a new question.

**Argument (The minimal second contract tests semantic boundaries)** · `arg:guide15-exec:second-contract-order`

The second exemplar's two declarations test the architecture-to-realization boundary with the smallest contract before the larger Phase-7 declarations; its requirements and package mapping are stated in (`sec:lineage:contract`), (`sec:lineage:requirements`), (`sec:lineage:traps`), (`sec:lineage:pipeline`) and (`sec:lineage:measurements`).

**Caveat (Later carrier measurements are regression evidence)** · `cav:guide15-exec:carrier-order`

The carrier forms Wave 1 will measure supply later regression evidence for the provisional carrier selected at bite 0.7; they cannot justify that earlier selection or choose an attestation record law by analogy, and a refusal of its encoded size or policy under measurements (b) or (f) reopens it by a Wave-4 ruling before declarations (`q:guide15:burn-record-carrier`); (`expt:lineage:data-output`); (`expt:lineage:weight-bound`).

**Justification (Architectural failure is tested before native cost)** · `just:guide15-exec:early-boundary-test`

An independence failure is cheapest to find before burn and clear depend on the same seams; bite 1.1 tests that boundary before or beside the native campaign, and the result names which seams moved, why they moved, and which laws remained specific to each contract (`obs:lineage:finding-architecture`); (`def:lineage:independence`); (`rule:roadmap:parallel`).

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

Those figures establish neither complete negative coverage nor final calibration; the negative half remains an inherited obligation under (`T4-009`), and candidate bounds retain (`rule:guide15:candidate-only`).

**Setting (Review basis and authoritative contracts)** · `setup:guide15-exec:review-basis`

The review basis comprises (`sec:admission:entry`), (`sec:admission:preflight`), (`sec:admission:findings`), (`abst:lineage:scope`), the concept (`sec:guide15:rulings`), the Phase-7 card (`phase:roadmap:burn-clear`), the amended Phase-6 assessment and gate record (`sec:phase6:exit-assessment`); (`sec:phase6:gate-record`), Guide-11 and Guide-12 results (`sec:public-opening:result`); (`tab:guide12-exec:preflight`), typed architecture in `packages/architecture/README.md`, model transitions, realization declarations, the reviewed target, and the affected package contracts `packages/model/README.md`; (`pkg:realization:contract`); (`pkg:compiler:contract`); (`pkg:target-elements:contract`); (`pkg:tapscript:contract`); (`pkg:linker:contract`); (`pkg:transaction:contract`); (`pkg:vectors:contract`).

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

Wave 2 delivers and validates the missing typed burn and clear declarations against architecture and independent model conformance; model support alone does not make unsupported realization dispatch supported (`task:guide15:wave1`); (`fact:lineage:declaration-boundary`).

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

Bite 0.7 selects a canonical, publicly recoverable carrier with exact association and ordering, accepted provisionally by a numbered Phase-7 ruling; later Wave-1 form measurements cannot substitute for this preflight disposition (`q:guide15:burn-record-carrier`).

**Condition (Carrier selection reopens before Wave-4 declarations)** · `cond:guide15-exec:carrier-reopening`

If Wave 1's data-output measurement (b) or weight measurement (f) refuses the selected carrier's encoded size or policy, its selection reopens as a Wave-4 ruling before Wave 4's declarations; otherwise those measurements remain regression evidence for the selected form (`expt:lineage:data-output`); (`expt:lineage:weight-bound`).

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
| (`cond:guide15:phase6-exited`) | §2.1 Phase-6 entry: Phase 6 has exited on the current tree | (`gate:guide15:phase6-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | OPEN | Required execution: retain amended exit and separately record the five conjunct dispositions from 0.5. | 0.7, 0.5 | — | — | (`sec:phase6:exit-assessment`) | condition bullet | inherited policy | Exited on the amended gate; this does not pass unamended Guide 14 §23. |
| (`cond:guide15:clean-tree`) | §2.1 Phase-6 entry: the tree is clean | (`gate:guide15:phase6-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | OPEN | Required execution: classify initial tree changes; record a clean full repository gate. | 0.1, 0.7 | — | `scripts/gate-clean-tree.sh` | (`sec:phase6:gate-record`) | condition bullet | structural implementation | The gate-of-record clean-tree procedure is available; the admission inspection recorded a clean tree. |
| (`cond:guide15:operation-scope`) | §2.2 Semantic entry: `burn`, `compact-ash`, and `clear` remain in architecture and realization scope under their exact existing identifiers | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | DOES NOT HOLD | OPEN | Required execution: record model law and exact existing identifiers; implement missing Burn and Clear dispatch and weld. | 0.7 | 2 | `packages/architecture/src/ids.rs`; `packages/realization/src/declarations/mod.rs` | — | condition bullet | new Phase-7 choice | All three architecture identifiers exist, but typed realization dispatch lacks Burn and Clear. |
| (`cond:guide15:validated-relations`) | §2.2 Semantic entry: burn, compact, and clear relations validate against architecture | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | DOES NOT HOLD | OPEN | Required execution: record complete relation census without claiming declaration validation; validate burn and clear declarations. | 0.7 | 2 | `packages/realization/src/declarations/mod.rs` | — | condition bullet | new Phase-7 choice | UnsupportedOperationDeclaration prevents complete burn/clear declaration validation. |
| (`cond:guide15:sponsor-opacity`) | §2.2 Semantic entry: sponsor-value opacity remains enforced | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | UNDECIDED | OPEN | Required execution: refit regions; close sponsored conditions; prove burn/clear-specific opacity. | 0.2, 0.5 | 3, 6, 7 | `packages/compiler/src/sponsor_region.rs` | (`sec:phase6:exit-assessment`) | condition bullet | new Phase-7 choice | Inherited opacity holds in its scope; burn/clear enforcement is not delivered and the region refit remains open. |
| (`cond:guide15:target-independent`) | §2.2 Semantic entry: no target-specific type enters realization | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | OPEN | Required execution: confirm target-independent realization inputs; reject target types entering the declaration boundary. | 0.7 | 2 | `packages/realization/src/lib.rs` |  (`pkg:realization:contract`); | condition bullet | inherited policy | The contract and public semantic surface retain target independence. |
| (`cond:guide15:lifecycle-obligations`) | §2.2 Semantic entry: lifecycle obligations remain explicit | (`gate:guide15:semantic-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | OPEN | Required execution: census lifecycle obligations and their implementing waves; prove actual constructor closure. | 0.7 | 5 and 11 | `packages/realization/src/lifecycle.rs` | (`sec:phase6:exit-assessment`) | condition bullet | inherited policy | Lifecycle obligations and candidate incompleteness remain explicit. |
| (`data:guide15:starting-revision`) | §2.8 Repository record: starting revision | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | OPEN | Required execution: record the actual starting tree and workspace version without substituting the study's older base. | 0.1 | — | `Cargo.toml`; `plans/phases/README.md` | — | repository-record field | structural implementation | The study records an older starting tree; Wave 0 records the actual execution base. |
| (`data:guide15:phase6-revision`) | §2.8 Repository record: Phase-6 result revision | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | OPEN | Required execution: retain Phase-6 evidence and amended closeout references at their recorded scope. | 0.1 | — | — | (`sec:phase6:gate-record`); (`sec:phase6:exit-assessment`) | repository-record field | observed candidate evidence | Evidence base 0.6.305-dev; gate-record landing 0.6.306-dev; amended closeout carried at 0.7.0-dev. |
| (`data:guide15:architecture-values`) | §2.8 Repository record: architecture schema and semantic hashes | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | OPEN | Required execution: read schema 18 and registered semantic/behavioural fields by path; record any admitted change. | 0.7 | — | `packages/architecture/src/spec.rs`; `plans/registers/identities.md`; `packages/model/generated/architecture.json` | — | repository-record field | structural implementation | Schema 18; semantic and behavioural values are the registered fields in the generated architecture publication, cited by path rather than copied digests. |
| (`data:guide15:realization-version`) | §2.8 Repository record: realization version | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | OPEN | Required execution: record derived realization version 0.7.0-dev without allocating a new identity for this charter. | 0.1, 0.7 | — | `Cargo.toml`; `packages/architecture/src/spec.rs` | — | repository-record field | structural implementation | 0.7.0-dev, derived from the workspace version. |
| (`data:guide15:clean-result`) | §2.8 Repository record: clean-tree result | (`rule:guide15:repository-entry`) | Q1 (`q:guide15:semantic-census`) | HOLDS | OPEN | Required execution: perform the clean-tree gate anew and retain its result rather than copying a historical pass. | 0.7 | — | `scripts/gate-clean-tree.sh` | (`sec:phase6:gate-record`) | repository-record field | observed candidate evidence | The gate-of-record clean-tree check over 0.6.305-dev is MET; this is the recorded result, not a fresh workspace gate. |
| (`cond:guide15:public-revalidation`) | §2.3 Public-declassification entry: the accepted public boundary is revalidated | (`gate:guide15:declassification-entry`) | Q2 (`q:guide15:ash-representation`) | UNDECIDED | OPEN | Required execution: revalidate accepted public-declassification policy on its recorded evidence boundary. | 0.7 | — | — | (`sec:public-opening:result`) | condition bullet | new Phase-7 choice | Accepted policy is known; Phase-7 revalidation is a Wave-0 deliverable, not a completed run here. |
| (`cond:guide15:ash-selection`) | §2.3 Public-declassification entry: the initial ASH representation is explicitly selected | (`gate:guide15:declassification-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | OPEN | Required execution: record explicit-only ASH selection; assess the actual Phase-7 forms. | 0.7 | 4 | — | (`sec:public-opening:result`) | condition bullet | inherited policy | The inherited initial public ownerless maintenance representation is explicit only. |
| (`cond:guide15:authenticated-opening`) | §2.3 Public-declassification entry: no unauthenticated opening is accepted | (`gate:guide15:declassification-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | OPEN | Required execution: retain unsupported authenticated opening and refuse unauthenticated scalar substitutes. | 0.7 | — | — | (`sec:public-opening:result`)  (`pkg:target-elements:contract`); | condition bullet | inherited policy | Authenticated opening remains unsupported; no unauthenticated amount is an accepted substitute. |
| (`cond:guide15:public-value`) | §2.3 Public-declassification entry: no report attachment substitutes for chain-public value | (`gate:guide15:declassification-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | OPEN | Required execution: retain public-value authority in chain data and refuse report attachments as value authentication. | 0.7 | — | `packages/transaction/src/view.rs` | (`sec:public-opening:result`) | condition bullet | inherited policy | The explicit public value is construction input; a report attachment supplies no opening authority. |
| (`cond:guide15:value-introspection`) | §2.6 Target entry: explicit `U` and explicit value introspection are reviewed | (`gate:guide15:target-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | OPEN | Required execution: record explicit U/value facts and absent authenticated opening; assess each selected form. | 0.7 | 4 | `packages/target-elements/src/opcode.rs`;  |  (`pkg:target-elements:contract`); | condition bullet | structural implementation | Explicit asset/value introspection is within the reviewed target; authenticated opening is not. |
| (`data:guide15:public-policy`) | §2.8 Repository record: selected public-value policy | (`rule:guide15:repository-entry`) | Q2 (`q:guide15:ash-representation`) | HOLDS | OPEN | Required execution: record explicit-only, normalized-first, private-change policy and deferred public opening. | 0.7 | — | — | (`sec:public-opening:result`) | repository-record field | inherited policy | Explicit-only boundary, normalized-first with private change; public-committed opening is deferred. |
| (`cond:guide15:private-route`) | §2.3 Public-declassification entry: any private-receipt burn path is classified as normalized-first or separately admitted direct-private | (`gate:guide15:declassification-entry`) | Q3 (`q:guide15:private-burn`) | UNDECIDED | OPEN | Required execution: record normalized-first and absent direct-private support unless separately admitted; assess selected route. | 0.7 | 4 | `plans/guides/guide_fifteen_concept.md` | (`sec:public-opening:result`) | condition bullet | new Phase-7 choice | Normalized-first is the inherited route; a Phase-7 execution disposition has not been recorded. |
| (`cond:guide15:compact-build`) | §2.4 Compact-ASH entry: the Phase-4 compact-ASH operation still builds | (`gate:guide15:compact-entry`) | Q4 (`q:guide15:ash-constructor`) | UNDECIDED | OPEN | Required execution: 0.4 and  rebuild the accepted compact candidate and record its exact subject. | 0.4, 0.7 | — | `packages/compiler/src/operation_plan.rs`; `plans/phases/04-compact-ash.md` | — | condition bullet | new Phase-7 choice | The operation exists and has a passing gate of record; current-base build is not rerun by this study. |
| (`cond:guide15:compact-regression`) | §2.4 Compact-ASH entry: its compiler plan, patterns, linker, ABI, vectors, and resource study remain green | (`gate:guide15:compact-entry`) | Q4 (`q:guide15:ash-constructor`) | UNDECIDED | OPEN | Required execution: 0.4 and  rerun the compact pipeline and resource regressions on the selected candidate. | 0.4, 0.7 | — | `plans/history/guide-12-completion-report.md`; `packages/vectors/src/resource_study.rs` | — | condition bullet | new Phase-7 choice | Historical candidate results do not certify a fresh regression run over this base. |
| (`cond:guide15:ash-compatibility`) | §2.4 Compact-ASH entry: its ASH constructor is compatible with burn-created and clear-residual ASH, or the incompatibility is explicitly dispositioned | (`gate:guide15:compact-entry`) | Q4 (`q:guide15:ash-constructor`) | UNDECIDED | OPEN | Required execution: disposition intended equality or explicit class transitions; prove exact constructor compatibility. | 0.7 | 5 and 8 | `packages/tapscript/src/bundle.rs`; `packages/linker/src/bundle.rs` | — | condition bullet | new Phase-7 choice | No compiled burn output or clear residual exists for a cross-operation constructor equality proof. |
| (`data:guide15:compact-revision`) | §2.8 Repository record: compact-ASH candidate revision | (`rule:guide15:repository-entry`) | Q4 (`q:guide15:ash-constructor`) | UNDECIDED | OPEN | Required execution: 0.1 and  select the exact compact bundle for the Phase-7 rerun, retaining the Phase-4 result. | 0.1, 0.4 | — | `plans/history/guide-12-completion-report.md`; `plans/phases/04-compact-ash.md` | — | repository-record field | new Phase-7 choice | The completion record names 0.4.1-dev; the exact inherited bundle/revision for a Phase-7 rerun still needs selection. |
| (`cond:guide15:operator-signing`) | §2.1 Phase-6 entry: operator signing is finalized-byte bound | (`gate:guide15:phase6-entry`) | Q5 (`q:guide15:burn-record-carrier`) | HOLDS | OPEN | Required execution: select the inherited finalized-byte authorization boundary; prove burn signatures bind every final byte role. | 0.7 | 9 | `packages/transaction/src/state_signing.rs` | (`sec:phase6:gate-record`) | condition bullet | structural implementation | §23 row 6 is MET; operator authorization binds finalized bytes. |
| (`cond:guide15:signing-profiles`) | §2.6 Target entry: selected owner and sponsor sighash profiles are reviewed | (`gate:guide15:target-entry`) | Q5 (`q:guide15:burn-record-carrier`) | UNDECIDED | OPEN | Required execution: select exact owner/sponsor profiles and their bindings; assess and execute Phase-7 signing. | 0.7 | 4 and 9 | `packages/target-elements/src/authorization.rs`; `packages/transaction/src/live_signing.rs` | — | condition bullet | new Phase-7 choice | Inherited owner/sponsor profiles are reviewed; their exact Phase-7 selection and binding remain to be recorded. |
| (`cond:guide15:typed-records`) | §2.2 Semantic entry: burn record and event semantics are typed | (`gate:guide15:semantic-entry`) | Q6 (`q:guide15:burn-record-rule`) | HOLDS | OPEN | Required execution: record model record/event law; deliver typed declarations and independently bound reports. | 0.7 | 2 and 10 | `packages/model/src/history.rs`; `packages/model/src/certify.rs` | — | condition bullet | structural implementation | BurnRecord and BurnProjection are typed model objects; compiler-facing realization is still missing. |
| (`cond:guide15:state-owner`) | §2.1 Phase-6 entry: canonical STATE metadata has one typed owner | (`gate:guide15:phase6-entry`) | Q7 (`q:guide15:clear-assignment`) | HOLDS | OPEN | Required execution: record six stored fields and their typed owner; validate the complete assignment census. | 0.7 | 2 | `packages/realization/src/state.rs`; `packages/realization/src/state_codec.rs` | — | condition bullet | structural implementation | The typed metadata owner and canonical codec are inherited; §23 row 3 is MET. |
| (`cond:guide15:clear-assignment`) | §2.2 Semantic entry: the complete STATE clear assignment is known | (`gate:guide15:semantic-entry`) | Q7 (`q:guide15:clear-assignment`) | HOLDS | OPEN | Required execution: record y_l as the only changed stored field and Y as derived; prove model/realization conformance. | 0.7 | 2 | `packages/model/src/ops/ash.rs`; `docs/attestation/realization.md` | — | condition bullet | structural implementation | ClearAsh changes only stored y_l; total Y falls by the same X because Y is derived. |
| (`cond:guide15:recon-tag`) | §2.2 Semantic entry: `tag-recon` is architecture-owned and exact | (`gate:guide15:semantic-entry`) | Q7 (`q:guide15:clear-assignment`) | HOLDS | OPEN | Required execution: identify the exact architecture tag; bind declaration and destruction to that tag. | 0.7 | 2 and 7 | `packages/architecture/src/ids.rs`; `packages/architecture/src/spec.rs` | — | condition bullet | structural implementation | TagId::Recon owns tag-recon; the clear destruction declaration names it. |
| (`cond:guide15:candidate-limits`) | §2.4 Compact-ASH entry: no stale candidate limit is treated as final calibration | (`gate:guide15:compact-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | OPEN | Required execution: bind every inherited limit to candidate scope and retain non-calibration status. | 0.7 | — | `plans/phases/04-compact-ash.md`;  |  (`pkg:linker:contract`); | condition bullet | inherited policy | Candidate bounds remain candidate; no final calibration follows from phase exit. |
| (`cond:guide15:target-definition`) | §2.6 Target entry: reviewed target definition validates | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | OPEN | Required execution: bind reviewed V2 definition; validate its selected capabilities. | 0.1, 0.7 | — | `packages/target-elements/src/definition.rs`;  |  (`pkg:target-elements:contract`); | condition bullet | structural implementation | Reviewed V2 definition and validator are implemented; this is a structural inherited reading. |
| (`cond:guide15:development-binding`) | §2.6 Target entry: development binding is welded to the exact target | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | OPEN | Required execution: validate the exact development binding before admitting native observations. | 0.1 | — | `packages/target-elements/src/weld.rs`; `packages/target-elements/src/deployment.rs` | — | condition bullet | structural implementation | Development binding validates against the exact reviewed target definition. |
| (`cond:guide15:fixed-arithmetic`) | §2.6 Target entry: checked fixed-width addition, subtraction, and comparison are reviewed | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | OPEN | Required execution: prove bounds and arithmetic schedule with reviewed primitives; execute the composed minimum pattern. | 0.7 | 7 | `packages/target-elements/src/opcode.rs`;  |  (`pkg:tapscript:contract`); | condition bullet | structural implementation | Reviewed fixed-width primitives exist; the clear-specific schedule remains a separate preflight obligation. |
| (`cond:guide15:target-conservation`) | §2.6 Target entry: target transaction conservation remains external evidence | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | OPEN | Required execution: preserve conservation as external target evidence; keep semantic relation proofs separate. | 0.7 | 3 | `packages/target-elements/src/confidential.rs`;  |  (`pkg:target-elements:contract`); | condition bullet | inherited policy | Conservation is an external target dependency, never a semantic proof of burn or clear. |
| (`cond:guide15:execution-domain`) | §2.6 Target entry: execution domain and leaf version are observed | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | OPEN | Required execution: 0.1 and  reproduce domain and leaf-version observations on the selected candidate. | 0.1, 0.5 | — | `packages/target-elements/src/definition.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | The accepted target observations bind the execution domain and leaf version of that candidate. |
| (`cond:guide15:relay-separation`) | §2.6 Target entry: consensus and relay policy remain separate | (`gate:guide15:target-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | OPEN | Required execution: measure positive-floor forms separately from consensus and preserve exact relay premises. | 0.5 | — | `packages/target-elements/src/resource.rs` | (`sec:phase6:exit-assessment`) | condition bullet | inherited policy | Zero fee floors and relay bounds remain explicit; consensus acceptance does not imply ordinary relayability. |
| (`data:guide15:target-revision`) | §2.8 Repository record: target contract revision | (`rule:guide15:repository-entry`) | Q8 (`q:guide15:clear-arithmetic`) | HOLDS | OPEN | Required execution: record reviewed TargetContractVersion::V2 and its exact deployment binding. | 0.1 | — | `packages/target-elements/src/definition.rs` | — | repository-record field | structural implementation | TargetContractVersion::V2, the supported reviewed contract. |
| (`cond:guide15:maturity-green`) | §2.1 Phase-6 entry: maturity announcement remains green under its candidate constructor generation | (`gate:guide15:phase6-entry`) | Q9 (`q:guide15:state-generation`) | UNDECIDED | OPEN | Required execution: 0.1 and  bind the candidate and reproduce maturity acceptance; revalidate the expanded generation. | 0.1, 0.5 | 7 | — | (`sec:phase6:gate-record`) | condition bullet | new Phase-7 choice | Acceptance is recorded over the candidate; no fresh maturity run is claimed at this base. |
| (`cond:guide15:constructor-reconstruction`) | §2.1 Phase-6 entry: predecessor and successor constructor reconstruction are implemented | (`gate:guide15:phase6-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | OPEN | Required execution: identify the exact constructor; reproduce reconstruction on retained subjects. | 0.1, 0.5 | — | — | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | §23 rows 4 and 10 are MET for the accepted candidate's constructor reconstruction. |
| (`cond:guide15:generation-identity`) | §2.5 STATE generation entry: the Phase-6 constructor generation is identified exactly | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | OPEN | Required execution: identify the retained maturity-only bundle; record its generation boundary. | 0.1, 0.7 | — | `packages/tapscript/src/state_constructor.rs` | (`sec:phase6:gate-record`) | condition bullet | structural implementation | The accepted maturity-only constructor is identified by its retained bundle and evidence, not a new generation name. |
| (`cond:guide15:clear-subtree`) | §2.5 STATE generation entry: adding clear is recognized as a static-subtree change | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | OPEN | Required execution: record the new static leaf set as a generation change; execute expanded-subtree regressions. | 0.7 | 7 | `packages/tapscript/src/state_constructor.rs`; `plans/guides/guide_fifteen_concept.md` | — | condition bullet | inherited policy | Adding clear changes the static leaf set and authenticated subtree; the concept draft states this explicitly. |
| (`cond:guide15:generation-policy`) | §2.5 STATE generation entry: the Phase-7 generation policy of §1.14 is accepted or replaced by an explicit migration decision | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | UNDECIDED | OPEN | Required execution: accept synthetic origin with explicit non-claims or record an accepted migration prerequisite. | 0.7 | — | `plans/guides/guide_fifteen_concept.md`; `plans/backlog.md` | — | condition bullet | new Phase-7 choice | The concept draft proposes synthetic Phase-7 origin; no execution admission accepts it yet. |
| (`cond:guide15:committed-leaf`) | §2.5 STATE generation entry: no test attempts to spend a Phase-6 STATE with an uncommitted Phase-7 leaf | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | OPEN | Required execution: retain no-generation-crossing rule; test refusal of uncommitted clear leaves. | 0.7 | 7 | `packages/transaction/src/state_abi.rs`; `packages/linker/src/state_bundle.rs` | — | condition bullet | inherited policy | The delivered candidate has no clear leaf; its existing constructor checks do not admit an uncommitted leaf. |
| (`cond:guide15:origin-nonclaims`) | §2.5 STATE generation entry: synthetic Phase-7 origin non-claims are agreed in advance | (`gate:guide15:state-generation-entry`) | Q9 (`q:guide15:state-generation`) | UNDECIDED | OPEN | Required execution: bind synthetic origin and no migration to the generation disposition before target implementation. | 0.7 | — | `plans/guides/guide_fifteen_concept.md`; `plans/backlog.md` | — | condition bullet | new Phase-7 choice | Synthetic origin and no-migration non-claims need an explicit admission disposition. |
| (`cond:guide15:constructor-hashing`) | §2.6 Target entry: STATE constructor hashing and tweak requirements remain reviewed | (`gate:guide15:target-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | OPEN | Required execution: record hashing and tweak facts; prove them for the expanded committed subtree. | 0.7 | 7 | `packages/target-elements/src/definition.rs`; `packages/tapscript/src/state_constructor.rs` | — | condition bullet | structural implementation | The Phase-6 hashing and tweak requirements remain reviewed; expanded-subtree evidence is not inherited. |
| (`data:guide15:state-generation`) | §2.8 Repository record: STATE-constructor generation | (`rule:guide15:repository-entry`) | Q9 (`q:guide15:state-generation`) | HOLDS | OPEN | Required execution: identify the maturity-only constructor; disposition the distinct Phase-7 generation. | 0.1, 0.7 | — | `packages/tapscript/src/state_constructor.rs` | (`sec:phase6:gate-record`) | repository-record field | structural implementation | The Phase-6 maturity-only candidate in the gate record; Phase-7 generation is not yet accepted. |
| (`cond:guide15:edge-history`) | §2.1 Phase-6 entry: root-history evidence is edge-based | (`gate:guide15:phase6-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: retain checkpoint binding and independently observe the edge on the pinned target. | 0.5 | — | `packages/vectors/src/maturity_history.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | Edge-based evidence exists; row 11 retains the checkpoint and environment conditions. |
| (`cond:guide15:exact-bindings`) | §2.1 Phase-6 entry: target, deployment, request, response, and provenance bindings remain exact | (`gate:guide15:phase6-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: 0.1 and  bind target, deployment, request, response, and subject for every new observation. | 0.1, 0.5 | — | `packages/vectors/src/maturity_continuity.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | Inherited validated evidence binds the exact subject; no broader target identity is inferred. |
| (`cond:guide15:compact-silence`) | §2.2 Semantic entry: compact ASH remains event-silent | (`gate:guide15:semantic-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: retain compact silence; reproduce its complete event and lifecycle projection. | 0.7 | 11 | `packages/realization/src/declarations/compact_ash.rs`; `packages/model/src/ops/ash.rs` | — | condition bullet | structural implementation | Compact is ownerless-lateral and produces no burn or clear projection. |
| (`cond:guide15:native-eligibility`) | §2.6 Target entry: no mock is gate-eligible | (`gate:guide15:target-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: 0.1 and  require real target subjects and exclude mock results from native discharge. | 0.1, 0.5 | — | `plans/guides/guide_fifteen_concept.md`; `packages/vectors/src/maturity_native.rs` | — | condition bullet | inherited policy | The concept draft requires real target evidence; mocks and first-party fixtures cannot discharge native rows. |
| (`cond:guide15:validated-plans`) | §2.7 Evidence entry: canonical evidence plans have no unchecked constructors | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: preserve validated evidence-plan construction while disposing every Safety row. | 0.5 | — | `packages/vectors/src/plan.rs`; `packages/vectors/src/maturity_evidence.rs` | — | condition bullet | structural implementation | Canonical evidence plans derive through validated inputs and preserve their internal invariants. |
| (`cond:guide15:validated-coverage`) | §2.7 Evidence entry: coverage changes only from validated report wrappers | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: admit coverage changes only through validated report wrappers and recount every bucket. | 0.5 | — | `packages/vectors/src/plan.rs`; `packages/vectors/src/maturity_report.rs` | — | condition bullet | structural implementation | Coverage discharge consumes validated evidence/report wrappers; declared requirements are not observations. |
| (`cond:guide15:observed-refusals`) | §2.7 Evidence entry: native refusals retain observed layers and exact subjects | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: bind each native refusal to its observed layer, declared boundary, and exact subject. | 0.5 | — | `packages/vectors/src/observed_boundary.rs`; `packages/vectors/src/maturity_mutants.rs` | — | condition bullet | structural implementation | Observed layer and bound subject remain attached to native refusals. |
| (`cond:guide15:exact-refusals`) | §2.7 Evidence entry: wrong-boundary refusals are not counted as answers | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: refuse coverage credit for wrong-boundary failures and retain focused controls. | 0.5 | — | `packages/vectors/src/observed_boundary.rs` | — | condition bullet | inherited policy | matches_boundary requires exact equality; a refusal elsewhere does not answer the row. |
| (`cond:guide15:observed-evidence`) | §2.7 Evidence entry: report-layer answerability is not counted as observed evidence | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: discharge or retype all 15 report-layer-required rows without claiming unobserved target behavior. | 0.5 | — | `packages/vectors/src/maturity_evidence.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | Report-layer required and observed evidence remain distinct; 15 report-layer-required rows remain outstanding. |
| (`cond:guide15:accepted-controls`) | §2.7 Evidence entry: accepted controls ground negative target observations | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: retain an accepted control for each native negative family and exact subject association. | 0.5 | — | `packages/vectors/src/maturity_mutants.rs` | (`sec:phase6:gate-record`) | condition bullet | structural implementation | Admitted negative observations retain accepted controls; the reading is limited to those admitted observations. |
| (`cond:guide15:rejection-artifacts`) | §2.7 Evidence entry: target rejection carries no accepted artifact | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: validate refusal response shape and absence of accepted artifacts. | 0.5 | — | `packages/target-elements-conformance/src/protocol.rs`; `packages/vectors/src/operation.rs` | — | condition bullet | structural implementation | Response validation distinguishes acceptance artifacts from refusal shapes. |
| (`cond:guide15:response-shape`) | §2.7 Evidence entry: response-shape validation is exact | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: 0.1 and  retain revision-8 response validation or explicitly adjudicate any required change. | 0.1, 0.5 | — | `packages/target-elements-conformance/src/protocol.rs` | — | condition bullet | structural implementation | The strict revision-8 response contract rejects incompatible shapes and subjects. |
| (`cond:guide15:retained-transcripts`) | §2.7 Evidence entry: canonical reports retain their generic execution transcripts | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: retain complete generic transcripts behind every admitted canonical report. | 0.5 | — | `packages/vectors/src/maturity_report.rs`; `packages/vectors/src/maturity_continuity_report.rs` | — | condition bullet | structural implementation | Validated reports retain generic execution evidence rather than an unbound success flag. |
| (`cond:guide15:separate-evidence`) | §2.7 Evidence entry: safety, event, continuity, history, accounting, and resources remain separate | (`gate:guide15:evidence-entry`) | Q10 (`q:guide15:event-independence`) | UNDECIDED | OPEN | Required execution: define separate Phase-7 event/accounting roles; deliver their validated producers. | 0.7 | 10 through 12 | `packages/vectors/src/maturity_report.rs`; `plans/guides/guide_fifteen_concept.md` | — | condition bullet | new Phase-7 choice | Five inherited report roles are separate; Phase-7 event/accounting report production is not delivered. |
| (`data:guide15:executor-revision`) | §2.8 Repository record: executor protocol revision | (`rule:guide15:repository-entry`) | Q10 (`q:guide15:event-independence`) | HOLDS | OPEN | Required execution: record NATIVE_PROTOCOL_SCHEMA 8, preserving archived runs' historical revisions. | 0.1 | — | `packages/target-elements-conformance/src/protocol.rs` | — | repository-record field | structural implementation | NATIVE_PROTOCOL_SCHEMA = 8; earlier archived runs retain their own historical protocol. |
| (`cond:guide15:public-successor`) | §2.1 Phase-6 entry: public successor reconstruction is implemented | (`gate:guide15:phase6-entry`) | Q11 (`q:guide15:permissionless`) | HOLDS | OPEN | Required execution: reconstruct from independently observed public data through a declared import boundary. | 0.5 | — | `packages/vectors/src/maturity_recovery.rs` | (`sec:phase6:gate-record`) | condition bullet | observed candidate evidence | Implemented recovery consumes archived mined readback through a pinned import boundary. |
| (`cond:guide15:public-ash`) | §2.3 Public-declassification entry: public ASH remains constructible by an unrelated process | (`gate:guide15:declassification-entry`) | Q11 (`q:guide15:permissionless`) | UNDECIDED | OPEN | Required execution: declare public-data-only construction tests; prove the complete actual ASH lifecycle. | 0.7 | 5 and 11 | `packages/vectors/src/resource_study.rs`; `plans/phases/04-compact-ash.md` | — | condition bullet | new Phase-7 choice | Public compact construction is inherited; burn-to-clear public construction is not yet evidenced. |
| (`cond:guide15:compact-authorization`) | §2.4 Compact-ASH entry: the inherited operation emits no authorization primitive | (`gate:guide15:compact-entry`) | Q11 (`q:guide15:permissionless`) | HOLDS | OPEN | Required execution: revalidate absence of compact authorization; distinguish optional sponsor witnesses from covenant requirements. | 0.4 | 11 | `packages/tapscript/src/pattern.rs`; `packages/transaction/src/abi.rs` | — | condition bullet | structural implementation | The inherited compact candidate is permissionless; sponsor authorization is a separate envelope. |
| (`cond:guide15:candidate-distinction`) | §2.1 Phase-6 entry: candidate and final states remain distinct | (`gate:guide15:phase6-entry`) | Q12 (`q:guide15:attestation-query`) | HOLDS | OPEN | Required execution: retain candidate-only status and explicit production, release, and calibration non-claims. | 0.7 | — | `packages/linker/src/state_bundle.rs` | (`sec:phase6:exit-assessment`) | condition bullet | inherited policy | Accepted candidate, release-complete false, no final calibration or deployment claim. |

### Inherited evidence limits and their carriers · `sec:guide15-exec:inherited-limits`

**Fact (Amended Phase-6 exit conditions)** · `fact:guide15-exec:amended-exit`

The accepted Phase-6 exit narrows bypass, wrong-code-subtree, resource, and relation coverage without adding evidence; its other six exit conditions retain their text (`sec:phase6:exit-assessment`).

**Data (Safety census and outstanding partition)** · `data:guide15-exec:safety-census`

The Safety census is 206 rows, 78 answered and 128 outstanding: 43 first-party required, 35 native-run required, 15 report-layer required, and 35 typed non-answers, pinned by `the_census_figures_are_unmoved_by_the_binding_readers` in `packages/vectors/src/maturity_evidence.rs` and carried by (`sec:phase6:exit-assessment`).

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

Of the five reason classes in `packages/vectors/src/maturity_safety.rs`, TotalityAnsweredByLandedNonceEvidence has eleven final rows and PropertyOfTheBuildRatherThanAFixture one, because their answers are held by other admitted evidence; AcceptanceAwaitsRelayAdmissibility has nine waiting rows, SponsoredControlRefusedAtConstruction eleven, and SponsorArmAwaitsRegionScoping three. Wave 0 must discharge those twenty-three waiting rows through the observable route; the census reports finality beside bucket counts (`rule:phase6:wave11-rulings`); (`sec:phase6:gate-record`).

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

Wave 0 owns inherited evidence closure and the concept's preparatory work; it introduces no new burn, clear, BURN, or MERGE declaration and preserves every historical gate record as evidence of its own scope. (`task:guide15:wave0`).

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

Implement the three rules stated below from `plans/guides/guide_thirteen_feature_requests.md`, make the required negative subjects constructible at their declared boundaries, and close the remaining 71 of 72 compact negative rows (`T4-009`).

**Setting (bite-0-4 — Files and packages)** · `setup:guide15-exec:bite-0-4-files-and-packages`

`packages/vectors/src/`, compact fixtures and evidence plans, `packages/transaction/src/`, `packages/linker/src/`, any required target-generic conformance support, and the compact gate disposition in `plans/backlog.md`. (`pkg:linker:contract`); (`pkg:transaction:contract`); (`pkg:vectors:contract`).

**Caveat (Compact positive evidence cannot answer negatives)** · `cav:guide15-exec:compact-negative-scope`

The admitted positive compact result remains at its Phase-4 scope; increasing the number of first-party tests cannot replace the missing mutation, funding, boundary, or native evidence that each row requires. (`T4-009`).

**Task (Part 0.4a — Three rules for the negative half)** · `task:guide15-exec:bite-0-4a`

Carry the three guide-level rules below into the compact negative work; they answer the guide gaps in `plans/guides/guide_thirteen_feature_requests.md` while keeping the 72-row campaign separate (`T4-009`).

**Rule (A negative row names relation, mutation class, and boundary)** · `rule:guide15-exec:compact-negative-row`

Every compact negative row names the relation it tests, its mutation class, and the boundary at which its refusal answers that relation; a refusal at another boundary is retained as evidence of that boundary only (`pkg:vectors:contract`); `plans/guides/guide_thirteen_feature_requests.md`.

**Rule (First-party discharge has an explicit admitted condition)** · `rule:guide15-exec:compact-first-party`

Each of the eighteen first-party rows names a retained comparison subject and checked relation sufficient to discharge its admitted claim, or an explicit out-of-scope declaration with its argument; an observable obligation cannot be removed by that declaration (`rule:guide15-exec:observable-retention`); `plans/guides/guide_thirteen_feature_requests.md`.

**Rule (ABI validation is a named negative entry point)** · `rule:guide15-exec:compact-abi-entry`

The compact negative plan names the typed ABI-validation entry point and exact malformed subject for each ABI-boundary row, makes that subject constructible without weakening earlier validators, and admits refusal only at the named boundary (`pkg:transaction:contract`); `plans/guides/guide_thirteen_feature_requests.md`.

**Task (Part 0.4b — Pinned-target compact campaign)** · `task:guide15-exec:bite-0-4b`

Dispose all 72 rows: 48 target-executable, eighteen first-party, and six unreachable; one target-executable row is already discharged, leaving 47 native rows and 71 rows overall outstanding. Run the native half only after 0.1, with exact funded subjects and accepted controls on the pinned target; report observations separately from final non-observation dispositions (`T4-009`); `plans/guides/guide_thirteen_feature_requests.md`.

**Gate (bite-0-4 — Gate)** · `gate:guide15-exec:bite-0-4-gate`

All 72 rows have exact subjects and final dispositions, with observed refusals at matching boundaries or typed formal closure reasons; the three guide requests each have an implementing artifact and passing verification. (`T4-009`).

**Record (bite-0-4 — Closure carrier)** · `rec:guide15-exec:bite-0-4-closure-carrier`

`T12-004` closes `T4-009`; the record reports answered and formally closed counts separately, so closing the backlog row does not falsely claim 72 observed refusals.

**Requirement (Current compact pipeline regressions)** · `req:guide15-exec:compact-regressions`

The compact compiler plan, patterns, linker, ABI, vectors, and resource evidence must remain green on the selected current candidate; any changed count is reported beside the inherited figure. (`T4-009`).

### 0.5 Close Safety dispositions and the five conjuncts · `task:guide15-exec:bite-0-5`

**Task (bite-0-5 — Deliverables)** · `task:guide15-exec:bite-0-5-deliverables`

Dispose all 128 outstanding Safety rows, preserve the 78 admitted answers unless evidence requires an explicit correction, and record a final standing for all 206 rows and every relation link. (`sec:phase6:gate-record`).

**Setting (bite-0-5 — Files and packages)** · `setup:guide15-exec:bite-0-5-files-and-packages`

`packages/vectors/src/maturity_safety.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_first_party.rs`, `packages/vectors/src/maturity_mutants.rs`, the native corpus and report modules, `packages/target-elements-conformance/src/`, and `plans/backlog.md`. (`pkg:vectors:contract`).

**Setting (Native mutation staging precedent)** · `setup:guide15-exec:mutation-staging`

The eight-row design in `plans/drafts/wave13-native-mutant-ceremony-study.md` supplies staging and boundary reasoning; its older standings do not replace the seven admitted refusals and unoffered recipe recorded at the accepted Phase-6 closeout. (`sec:phase6:gate-record`).

**Precondition (Sponsored obligations follow the region refit and ruling)** · `pre:guide15-exec:sponsored-order`

Bite 0.5 obtains a numbered Phase-7 ruling revisiting ruling 58 before C1, with the refit of 0.2 complete; C1 supplies the sponsored STATE builder, C2 performs its sponsored native ceremony on the provisioned target, and C3 retypes the sponsor rows after their observations. Eleven construction-refused rows depend on C1 and three region-scoping rows depend on the refit; none can be transferred out of Wave 0 (`rule:phase6:wave11-rulings`); (`rule:guide15-exec:observable-retention`).

**Criterion (Independent edge and successor observations)** · `crit:guide15-exec:conjunct-observation`

Conjuncts 11 and 13 require the separate process, pinned import boundary, node-interface reading, and absence of shared archive, fixture, or process state stated by (`crit:guide15-exec:observer-independence`); equal final cursors cannot replace equal edge histories, and no second chain view is claimed.

**Task (Part 0.5 — 43 first-party rows)** · `task:guide15-exec:bite-0-5-first-party`

The 43 first-party-required rows each need their own retained comparison object and exact relation, with final out-of-contract claims separated from observable work (`sec:phase6:gate-record`); (`rule:guide15-exec:observable-retention`).

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

The carrier disposition specifies canonical bytes, ordering, record identity, public recovery, association with fresh ASH, and value/cardinality law; later OP_RETURN measurements can challenge the selected form but cannot backdate its proof, and a size or policy refusal under measurement (b) or (f) reopens the selection by a Wave-4 ruling before declarations (`cond:guide15-exec:carrier-reopening`).

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

## 4. Wave 1 — The second exemplar: destruction lineage · `task:guide15-exec:wave1`

**Precondition (Wave-1 entry and compilation scope)** · `pre:guide15-exec:wave1-entry`

Wave 1 opens only on the Wave-0 exit and carries the contract in (`sec:lineage:contract`), (`sec:lineage:requirements`), (`sec:lineage:traps`), (`sec:lineage:pipeline`), (`sec:lineage:measurements`) and (`sec:lineage:decisions`) through declaration, analysis, emission, linking, construction, and translation-validation evidence.

**Definition (Positive explicit destruction)** · `def:guide15-exec:destruction-output`

The contract destroys a positive explicit native amount in an OP_RETURN output containing exactly one opaque 32-byte address; its magnitude is consensus value, not a quantity encoded in the address data. (`def:lineage:destruction-output`).

**Specification (BURN pair and open carrier)** · `spec:guide15-exec:burn-pair`

BURN creates the minimal-value claim token at output 0 beside the destruction output at output 1; the pair fixes association without constraining unrelated inputs, outputs, authorization, or carrier purpose beyond target validity. (`op:lineage:burn`); (`def:lineage:claim-token`).

**Specification (MERGE successor-only condition)** · `spec:guide15-exec:merge-successor`

MERGE consumes one or more outputs carrying the fixed covenant and produces output 0 carrying the same covenant; each covenant input checks only that successor script, without a signature, value comparison, asset comparison, or closure over other outputs. (`op:lineage:merge`).

**Definition (Genesis and settled ancestry)** · `def:guide15-exec:settled-ancestry`

The designated genesis outpoint contributes no destruction record; a descendant lineage point settles the set of valid token-outpoint records in its ancestry, including a newly valid pair when BURN and MERGE coincide. (`def:lineage:genesis`).

**Rule (Side piles and duplicate credit)** · `rule:guide15-exec:side-pile-rule`

Side piles may merge into the designated head later, but their depth does not designate the canonical lineage; record identity prevents duplicate credit through shared ancestry, and malformed pairs contribute nothing. (`def:lineage:side-pile`).

**Invariant (Monotone bounded consumer acceptance)** · `inv:guide15-exec:consumer-bound`

Consumer acceptance obeys A(previous) ⊆ A(this) ⊆ L(this), with eligibility, timing, and boundary recognition in the consumer's published rule rather than the covenant; a cached commitment has no settlement authority. (`inv:lineage:accepted-sets`).

**Specification (Absent covenant laws)** · `spec:guide15-exec:covenant-exclusions`

The covenant carries no amount accumulator, count, index, aggregate commitment, ownership role, or redemption entitlement; incidental funding is not record magnitude and no third operation is introduced. (`spec:lineage:exclusions`).

**Rule (Wave-1 failure route)** · `rule:guide15-exec:wave1-failure`

If the second contract cannot be carried through Wave 1's exit, the fail-closed law stops Phase 7 until it can be carried; Wave 1 has no other failure route (`rule:roadmap:laws`); (`rule:guide15-exec:second-contract-dependency`).

### 1.0 Measure the six target obligations · `task:guide15-exec:bite-1-0`

**Task (bite-1-0 — Deliverables)** · `task:guide15-exec:bite-1-0-deliverables`

Perform the six measurements from (`sec:lineage:measurements`) on the Wave-0 pinned target using isolated target experiments permitted by (`rule:roadmap:parallel`), before selecting stable declarations or ABI bounds, after or alongside the architecture-to-realization boundary test in bite 1.1.

**Caveat (Prototype evidence does not certify final artifacts)** · `cav:guide15-exec:prototype-evidence`

The experiments may prototype the recurrence and carrier forms without claiming a delivered compiler path; bite 1.6 re-executes the final compiled forms and no prototype observation certifies a later artifact. (`gate:lineage:measurement-evidence`).

**Setting (bite-1-0 — Files and packages)** · `setup:guide15-exec:bite-1-0-files-and-packages`

Experimental fixtures and evidence in `packages/vectors/`, reviewed assessments in `packages/target-elements/`, isolated recurrence work in `packages/tapscript/`, and retained measurement records in the wave's backlog gate record. (`pkg:target-elements:contract`); (`pkg:tapscript:contract`); (`pkg:vectors:contract`).

**Experiment (Measurement a — relay funding floor)** · `expt:guide15-exec:measurement-a`

Measure exact token and successor constructor admission immediately below and at the positive relay minimum, deciding incidental funding and the selected relay route. (`expt:lineage:relay-floor`).

**Fact (Reviewed consensus positivity)** · `fact:guide15-exec:consensus-positivity`

Explicit zero-valued spendable outputs are already refused by the reviewed target's `ExplicitZeroValueRule::SpendableRefused`; this experiment measures the positive relay floor, not an unresolved consensus zero-value question. (`fact:lineage:reviewed-positivity`).

**Experiment (Measurement b — destruction data-output admission)** · `expt:guide15-exec:measurement-b`

Measure the exact explicit positive native OP_RETURN output with its 32-byte datum and encoding overhead, deciding whether the BURN pair relays and which carrier limits must be published. (`expt:lineage:data-output`).

**Definition (Separate data and witness limits)** · `def:guide15-exec:distinct-limits`

The data-output limit is independent of the reviewed 80-byte tapscript witness-item limit; reader refusal of malformed data is distinguished from target consensus or relay refusal. (`bound:lineage:witness-item`).

**Experiment (Measurement c — conflicting head spends)** · `expt:guide15-exec:measurement-c`

Establish exact replacement conditions and fee increments or first-seen behavior, deciding the admitted per-head contention argument under the observed policy. (`expt:lineage:head-conflicts`).

**Caveat (Replacement does not establish inclusion)** · `cav:guide15-exec:replacement-limit`

Replacement admission never proves eventual inclusion or a universal censorship bound; block inclusion, historical availability, and finality remain declared assumptions with exact limits. (`cav:lineage:replacement-evidence`).

**Experiment (Measurement d — mixed representations)** · `expt:guide15-exec:measurement-d`

Execute explicit native destruction and token funding with confidential unrelated outputs, binding encodings, balance proofs, and fee outputs to decide which mixed carrier forms are admitted. (`expt:lineage:mixed-representations`).

**Caveat (Mixed forms authenticate no unsupported opening)** · `cav:guide15-exec:mixed-limit`

The result cannot substitute a disclosed scalar for authenticated destruction value or claim a confidential-opening primitive absent from the reviewed target. (`obl:lineage:mixed-evidence`).

**Experiment (Measurement e — one-check implementation)** · `expt:guide15-exec:measurement-e`

Execute script introspection and equality with authenticated current-program reference, constructor control path, final stack, and wrong-script/wrong-version controls, deciding faithful target recurrence. (`expt:lineage:one-check`).

**Requirement (Recurrence measurement dimensions)** · `req:guide15-exec:recurrence-dimensions`

Record program bytes, witness items, control depth, stack peaks, and available execution costs; primitive vocabulary alone does not prove the composed recurrence pattern. (`spec:lineage:recurrence-evidence`).

**Experiment (Measurement f — transaction weight and N)** · `expt:guide15-exec:measurement-f`

Measure BURN and MERGE carriers over a proposed input range with fee funding, optional confidential outputs, and worst-case witnesses, deciding the construction bound and sweep costs. (`expt:lineage:weight-bound`).

**Verification (Predictions against observed resources)** · `verif:guide15-exec:resource-comparison`

Compare backend/linker predictions with observed weight and resources, state absent dimensions as absent, and retain prediction mismatches as findings rather than rewriting predictions to match observations. (`obl:lineage:resource-evidence`).

**Decision (Decision 2 — incidental funding)** · `dec:guide15-exec:decision-2`

Choose provisionally, until final-form remeasurement in bite 1.6, the measured default-relay floor or a separately admitted positive-value route with its deployment dependence stated; no draft minimum enters code as a fact. (`q:lineage:funding-floor`).

**Decision (Decision 4 — input bound)** · `dec:guide15-exec:decision-4`

Choose measured N provisionally until final-form remeasurement in bite 1.6 as an ABI construction bound unless an independently justified contract change requires a covenant count check; report the aggregation and worst-case-cost tradeoff. (`q:lineage:input-bound`).

**Gate (bite-1-0 — Gate)** · `gate:guide15-exec:bite-1-0-gate`

Six distinct measurement records bind target, policy, exact subject, accepted controls, negative outcomes, and deciding consequences; unresolved forms block the consuming bite. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

### 1.1 Choose the realization home and declare the contract · `task:guide15-exec:bite-1-1`

**Task (bite-1-1 — Deliverables)** · `task:guide15-exec:bite-1-1-deliverables`

Run the architecture-to-realization boundary test before the native campaign of bite 1.0, or alongside it under (`rule:roadmap:parallel`), so the cheapest informative failure is reached first; choose the semantic home from the study's three options using measured and read evidence, declare typed BURN and MERGE with exact spellings, and weld the declarations to an authoritative architecture description.

**Setting (bite-1-1 — Files and packages)** · `setup:guide15-exec:bite-1-1-files-and-packages`

`packages/architecture/src/`, `packages/realization/src/`, their contracts and tests, and any separately admitted realization interface required by the selected home. (`pkg:realization:contract`).

**Decision (Decision 1 — realization home)** · `dec:guide15-exec:decision-1`

Another declaration set shares infrastructure but broadens identifier assumptions; a semantic profile needs a selector of semantics beyond deployment evidence; a second realization crate makes separation explicit but requires a shared typed compiler input interface and maintenance. (`q:lineage:realization-home`).

**Criterion (A sufficient semantic home)** · `crit:guide15-exec:semantic-home-criterion`

The choice must demonstrate the current attestation-specific manifest boundary, validate the new operation/object/observable vocabulary, and state the smallest necessary generalization; existing deployment profiles are not a second-contract selector. (`cond:lineage:validated-input`).

**Specification (BURN relation placement)** · `spec:guide15-exec:burn-relations`

BURN declarations separate pair recognition, fixed positions, positive explicit native value, exact datum, token recognition, and immutable association from any relation actually enforced by a spending script. (`spec:lineage:burn-relations`).

**Specification (MERGE reader and emitted roles)** · `spec:guide15-exec:merge-relations`

MERGE declares successor-at-0 for every covenant input and an ancestry-set projection owned by its reader; the emitted leaf neither traverses history nor computes the union of records. (`spec:lineage:merge-relations`).

**Decision (Decision 3 — value preservation)** · `dec:guide15-exec:decision-3`

Record the charter's adoption of the contract's default of no value-preservation rule; incidental inputs may fund fees, the successor, or other outputs, and the sweep bounty is the disposition of incidental value available to a permitted sweep, not record credit or a new entitlement. (`q:lineage:value-preservation`).

**Argument (Value preservation changes the contract)** · `arg:guide15-exec:no-value-preservation`

Greater-or-equal or equality rules would change the one-check contract by adding a comparison base and introspection without authenticating record magnitude; neither is imported into these declarations. (`case:lineage:decision-value-preservation`).

**Gate (bite-1-1 — Gate)** · `gate:guide15-exec:bite-1-1-gate`

Both declarations validate against the selected architecture boundary, contain no fictitious attestation roots or receipt classes, preserve public constructibility, and introduce no third operation. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

### 1.2 Compile validated operation plans · `task:guide15-exec:bite-1-2`

**Task (bite-1-2 — Deliverables)** · `task:guide15-exec:bite-1-2-deliverables`

Reuse the compiler's graph, fact-source, constructibility, lifecycle, proof-planning, placement, and coverage responsibilities to derive new validated plans for the two declarations. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Setting (bite-1-2 — Files and packages)** · `setup:guide15-exec:bite-1-2-files-and-packages`

`packages/compiler/src/`, its plan validators and tests, `plans/packages/compiler.md`, and the semantic input boundary selected in bite 1.1. (`pkg:compiler:contract`).

**Justification (No fictitious BURN execution carrier)** · `just:guide15-exec:burn-carrier`

BURN may have no covenant input, so its record recognition cannot be attributed to an emitted leaf that never executes; pair and ancestry predicates need honest reader/evidence carriers while actual destruction relies on target consensus. (`just:lineage:burn-placement`).

**Obligation (Honest placement for every relation)** · `obl:guide15-exec:merge-placement`

MERGE's successor condition has one emitted carrier per covenant input, while consumer acceptance remains a reader obligation; placement must account for both without inventing a coordinator role (`rule:compiler:placement`). (`obl:lineage:relation-carriers`).

**Precondition (Public witness availability)** · `pre:guide15-exec:public-witness`

Public inputs are previous-output data, the fixed public constructor and spend recipe, and carrier funding; no creator secret or owner signature enters covenant witness availability. (`pre:lineage:public-construction`).

**Gate (bite-1-2 — Gate)** · `gate:guide15-exec:bite-1-2-gate`

Each active relation has a reachable, named carrier or explicit external-evidence obligation; corruption tests reject missing, duplicate, unreachable, and mismatched assignments and preserve exact lifecycle closure. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Criterion (Compilation rather than handwritten execution)** · `crit:guide15-exec:compiled-evidence`

The gate records which compiler seams changed and why; a handwritten target transaction that bypasses validated analysis does not count as a compiled second contract. (`cav:lineage:compiler-interface`).

### 1.3 Emit target forms and authenticated recurrence · `task:guide15-exec:bite-1-3`

**Task (bite-1-3 — Deliverables)** · `task:guide15-exec:bite-1-3-deliverables`

Admit exact BURN and MERGE forms against the reviewed target and emit the one-check successor pattern with a public authenticated recurrence constructor in tapscript. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Setting (bite-1-3 — Files and packages)** · `setup:guide15-exec:bite-1-3-files-and-packages`

`packages/target-elements/src/transaction_form.rs`, the reviewed capability assessments, `packages/tapscript/src/`, and their target and constructor tests. (`pkg:target-elements:contract`); (`pkg:tapscript:contract`).

**Requirement (Complete program representation)** · `req:guide15-exec:program-representation`

The output-script comparison binds the full reviewed program representation, including type or version; a superficially similar witness program from another constructor must not pass. (`obl:lineage:script-representation`).

**Invariant (Authenticated comparison target)** · `inv:guide15-exec:comparison-authentication`

The leaf checks output 0 against the actual authenticated covenant program, never an unauthenticated caller-supplied script; it performs no output-value or output-asset comparison. (`obl:lineage:constructor-paths`).

**Decision (Decision 7 — recurrence strategy)** · `dec:guide15-exec:decision-7`

An authenticated current-input program reference avoids an assumed literal fixed point but needs an exact introspection/control-path proof; a linker reference strategy must instead prove supported closure and its resource cost. If selected, that strategy owes its own recurrence execution and resource measurement under obligation (e), with final-form remeasurement at 1.6 (`expt:lineage:one-check`). (`q:lineage:recurrence-strategy`).

**Obligation (No bypass of recurrence)** · `obl:guide15-exec:recurrence-bypass`

Either strategy must exclude alternative spend paths that bypass recurrence, including a usable key path, while retaining conditional NUMS satisfaction under the two named assumptions where applicable (`judg:guide15-exec:nums-standing`); no finite refusal is promoted into a discrete-log proof. (`obl:lineage:constructor-paths`).

**Gate (bite-1-3 — Gate)** · `gate:guide15-exec:bite-1-3-gate`

The emitted schedule has valid stack behavior, exact constructor authentication and complete resource formulas, and its wrong-program, wrong-version, and wrong-control cases refuse at their declared boundaries. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Rule (Form admission does not add covenant laws)** · `rule:guide15-exec:form-admission`

The integrated form assessments distinguish consensus from relay admission and preserve measured incidental-value and N limits without turning them into added covenant laws. (`spec:lineage:form-assessments`).

### 1.4 Link references and construct transactions · `task:guide15-exec:bite-1-4`

**Task (bite-1-4 — Deliverables)** · `task:guide15-exec:bite-1-4-deliverables`

Link one deterministic lineage bundle, resolve every relocation once, retain the recurrence strategy and control recipe, and derive BURN and MERGE construction through the shared transaction pipeline. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Setting (bite-1-4 — Files and packages)** · `setup:guide15-exec:bite-1-4-files-and-packages`

`packages/linker/src/`, `packages/transaction/src/`, their candidate bundle, ABI and construction tests, and their package contracts. (`pkg:linker:contract`); (`pkg:transaction:contract`).

**Specification (Exact positional ABI boundaries)** · `spec:guide15-exec:lineage-abi`

The BURN ABI fixes token output 0 and destruction output 1, while the MERGE ABI fixes only covenant successor output 0; unrelated target-valid carrier inputs and outputs stay open. (`spec:lineage:layouts`).

**Definition (Builder ordering and witness roles)** · `def:guide15-exec:lineage-witness`

The predecessor's input ordering is a builder choice rather than a protocol rule; funding signatures and optional consumer signatures remain separate witness roles and introduce no covenant authorization. (`def:lineage:witness-roles`).

**Decision (Decision 5 — genesis designation)** · `dec:guide15-exec:decision-5`

Choose a publicly distributed designated outpoint or a reproducible deployment procedure with chain binding, stating the distribution or reconstruction cost; neither longest history nor arbitrary same-script outputs can designate genesis. (`q:lineage:genesis-designation`).

**Invariant (One genesis and recurrence constructor)** · `inv:guide15-exec:genesis-constructor`

The genesis output uses the same script but has no destruction sibling and contributes no record; token creation and successor recognition must use the same authenticated constructor. (`obl:lineage:constructor-identity`).

**Gate (bite-1-4 — Gate)** · `gate:guide15-exec:bite-1-4-gate`

Constructor/reference closure is exact, transactions finalize deterministically through typed requests, positive and malformed pair forms are distinct, and caller-supplied totals or ad hoc script literals cannot bypass the pipeline. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Caveat (Assembly reuse imports no STATE law)** · `cav:guide15-exec:assembly-reuse`

The record identifies inherited construction mechanisms and any changed interface without importing STATE codecs, STATE generations, or every-receipt-owner signing as universal requirements. (`cav:lineage:assembly-reuse`).

### 1.5 Build vectors and six evidence families · `task:guide15-exec:bite-1-5`

**Task (bite-1-5 — Deliverables)** · `task:guide15-exec:bite-1-5-deliverables`

Build public semantic fixtures, focused mutations, independent ancestry reconstruction, and distinct pair, successor, ancestry, replacement-history, consumer-rule, and resource evidence. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Setting (bite-1-5 — Files and packages)** · `setup:guide15-exec:bite-1-5-files-and-packages`

`packages/vectors/src/`, its public fixtures and report validators, realization-owned explanatory documentation, and target-generic execution support only where a missing fact requires it. (`pkg:vectors:contract`).

**Criterion (Authenticated pair evidence)** · `crit:guide15-exec:pair-evidence`

Pair evidence verifies the same transaction, fixed indices, token script, exact opaque address, and explicit positive native destruction value; wrong asset, committed value, malformed sibling, and fake token contribute zero credit. (`crit:lineage:valid-pair`).

**Criterion (Successor evidence and refusal boundary)** · `crit:guide15-exec:successor-evidence`

Successor evidence proves the common output-0 program at each covenant input and keeps script refusal distinct from malformed-pair reader refusal. (`op:lineage:merge`).

**Requirement (Complete ancestry and inclusion evidence)** · `req:guide15-exec:ancestry-evidence`

Ancestry evidence verifies inclusion proofs binding the point and its ancestry to the selected history, and covers a single token, many tokens, side-pile sweep, combined BURN/MERGE, multiple successive points in one block, shared ancestors, duplicate traversal, and missing ancestors; missing ancestry is incomplete proof. (`inv:lineage:replacement-history`).

**Requirement (Pinned replacement histories)** · `req:guide15-exec:replacement-evidence`

Replacement-history evidence binds competing spends and alternative histories to the exact pinned lineage point and policy, retaining explicit finality and availability assumptions. (`assum:lineage:finality`).

**Decision (Decision 8 — consumer publication)** · `dec:guide15-exec:decision-8`

Choose acceptance equal to L or a published monotone subset rule, and pins-only or earlier readings with their additional evidence and downward-revision behavior; enforce the consumer's own inclusion bound. (`q:lineage:consumer-publication`).

**Criterion (Consumer evidence rejects unbounded readings)** · `crit:guide15-exec:consumer-evidence`

Consumer-rule evidence rejects accepted records outside the settled set and a decrease of previously accepted records along the selected history, without moving eligibility or timing into the covenant. (`obl:lineage:consumer-readings`).

**Decision (Decision 9 — evidence consumption)** · `dec:guide15-exec:decision-9`

Use internal fixture reports unless an actual external ancestry/record consumer requires a maintained format and availability contract under `adr/022-interchange-conventions.md`; a cache remains non-authoritative either way. (`q:lineage:evidence-consumption`).

**Requirement (Resource subjects and absent dimensions)** · `req:guide15-exec:resource-evidence`

Resource evidence binds exact bundle, ABI, target, policy, and input bound; an unavailable dimension is absent, a failed child is infrastructure failure, and a policy refusal is not automatically a consensus refusal. (`obl:lineage:resource-evidence`).

**Obligation (Four arguments from the contract laws)** · `obl:guide15-exec:trap-arguments`

The realization's own documentation must argue each of the four traps using its own laws rather than a borrowed assurance claim. (`sec:lineage:traps`).

**Argument (Trap argument 1)** · `arg:guide15-exec:trap-argument-1`

Accumulator trap: no total, count, index, or commitment is stored; ancestry reconstruction computes figures and does not remove head contention (`arg:lineage:accumulator`).

**Argument (Trap argument 2)** · `arg:guide15-exec:trap-argument-2`

Burn-flag trap: the token alone supplies no credit; only the verified sibling's positive explicit native destruction amount supplies magnitude (`arg:lineage:burn-flag`).

**Argument (Trap argument 3)** · `arg:guide15-exec:trap-argument-3`

Data-ledger trap: address data never supplies quantity, and a record becomes settled through token ancestry rather than arbitrary data scanning (`arg:lineage:data-ledger`).

**Argument (Trap argument 4)** · `arg:guide15-exec:trap-argument-4`

Event-anchor trap: pair shape establishes the event while the destruction output's actual value establishes its magnitude with equality; inherited records keep their original anchors (`arg:lineage:event-anchor`).

**Task (Candidate publication artifacts for all thirteen requirements)** · `task:guide15-exec:publication-artifacts`

Bite 1.5 produces pinned-target candidate fixture artifacts for bootstrap zero with its empty-interval reason, unit cost with fees separate, and operator accrual with incidental funding distinguished from destroyed magnitude (`req:lineage:bootstrap`); (`req:lineage:unit-cost`); (`req:lineage:operator-accrual`). It also produces the candidate deployment publication set: settlement depth, finality assumptions, historical availability assumption, and the conflict policy established by measurement (c), without claiming production deployment (`obl:lineage:deployment-assumptions`).

**Criterion (The complete requirement set is exercised)** · `crit:guide15-exec:thirteen-requirements`

The Wave-1 evidence addresses all thirteen requirements: eight construction standings, three candidate publication standings, and two consumer-rule standings; fixture publications remain candidate artifacts and do not establish a production deployment (`tab:lineage:requirement-standings`); (`req:lineage:exogenous-magnitude`); (`req:lineage:neutrality`); (`req:lineage:non-transferability`); (`req:lineage:opaque-addresses`); (`req:lineage:non-negativity`); (`req:lineage:auditability`); (`req:lineage:instant-cost`); (`req:lineage:lineage`); (`req:lineage:monotone-reading`); (`req:lineage:consumer-acceptance`).

**Obligation (A durable home for the six measurement results)** · `obl:guide15-exec:measurement-home`

Retain all six target measurement records in `plans/research/state-constructor.md` as a distinct second-contract constructor-and-carrier evidence set, separating recurrence from relay floor, data-output policy, conflict policy, mixed representations, and weight; link the relevant public-representation findings to (`sec:public-opening:measurements`) and keep gate performance in (`tab:test-performance:register`). This assignment extends the research record at implementation time and does not claim those measurements already exist (`sec:state-constructor:measurements`); (`sec:lineage:measurements`).

**Gate (bite-1-5 — Gate)** · `gate:guide15-exec:bite-1-5-gate`

All six evidence families have independent subjects and validators, all thirteen requirement standings have candidate evidence under (`crit:guide15-exec:thirteen-requirements`), and the four trap arguments appear in the selected realization's documentation with executable positive and negative cases where applicable. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

### 1.6 Execute the compiled forms and record independence · `task:guide15-exec:bite-1-6`

**Task (bite-1-6 — Deliverables)** · `task:guide15-exec:bite-1-6-deliverables`

Execute final compiler-derived BURN and MERGE forms against the pinned target, independently reconstruct accepted records, and write the wave gate and architecture-independence findings. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Setting (bite-1-6 — Files and packages)** · `setup:guide15-exec:bite-1-6-files-and-packages`

`packages/vectors/`, the selected realization's documentation, affected package contracts, and the Phase-7 gate record in `plans/backlog.md`. (`pkg:vectors:contract`).

**Verification (Final-form remeasurement)** · `verif:guide15-exec:final-measurements`

Re-execute all six measurement obligations on the actual emitted, linked, and constructed artifacts; changed bytes, control depth, funding, input bound, or policy invalidate any unqualified transfer of prototype observations. (`spec:lineage:evidence-package`).

**Decision (Decision 6 — admission and first gate)** · `dec:guide15-exec:decision-6`

Record the charter's admission of the second contract inside Phase 7 without changing phase order, with its accepted dependency under (`rule:guide15-exec:second-contract-dependency`); the first gate requires both typed declarations, authenticated recurrence, separately assessed forms, and all six measurement obligations disposed on final subjects. (`q:lineage:admission`).

**Report (Independence findings and reuse evidence)** · `rep:guide15-exec:independence-report`

Record the exact seams that moved in architecture, realization, compiler, target, tapscript, linker, transaction, and vectors, with a reason and regression evidence for each change; unchanged seams need concrete reuse evidence. (`def:lineage:independence`).

**Gate (bite-1-6 — Gate)** · `gate:guide15-exec:bite-1-6-gate`

Public deterministic inputs reproduce both final forms and their accepted projections, focused negatives refuse at their own boundaries, all six report families validate, and the full repository gate is green on the completed clean tree. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Rule (Five distinct acceptance questions)** · `rule:guide15-exec:acceptance-boundaries`

The record must distinguish construction admission, target consensus, relay policy, historical reconstruction, and consumer recognition; no single successful transaction can stand for the entire contract. (`crit:lineage:evidence-families`).

### Wave-1 exit · `gate:guide15-exec:wave1`

**Condition (wave1-exit condition 1)** · `cond:guide15-exec:wave1-exit-condition-1`

- A second contract is compiled and executed through the same pipeline, with validated declarations, plans, target forms, constructor/reference closure, typed construction, all six evidence families, and candidate publication artifacts addressing all thirteen requirements (`crit:guide15-exec:thirteen-requirements`). (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Condition (wave1-exit condition 2)** · `cond:guide15-exec:wave1-exit-condition-2`

- Measurements (a)–(f) bind the final subjects and selected target policy; incidental minimum and input bound are measured facts rather than draft defaults. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Condition (wave1-exit condition 3)** · `cond:guide15-exec:wave1-exit-condition-3`

- All nine study decisions have a retained disposition at the bite assigned above, including the three realization-home options and their costs. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Condition (wave1-exit condition 4)** · `cond:guide15-exec:wave1-exit-condition-4`

- No attestation law is imported: no aggregate equation, no output closure over the carrier, and no consumer authorization in the covenant. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Condition (wave1-exit condition 5)** · `cond:guide15-exec:wave1-exit-condition-5`

- No third operation or value-preservation relation is introduced; incidental funding and its sweep bounty remain separate from destruction credit. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Condition (wave1-exit condition 6)** · `cond:guide15-exec:wave1-exit-condition-6`

- Independence findings name the seams that moved and why, and the realization's own documentation argues all four cited traps. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Condition (wave1-exit condition 7)** · `cond:guide15-exec:wave1-exit-condition-7`

- The wave's figures are compared with the Wave-0 baseline, with regressions treated as findings and the clean full repository gate recorded. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

**Proposal (wave1-exit — Suggested commit)** · `proposal:guide15-exec:wave1-exit-suggested-commit`

`plans: record the compiled destruction-lineage contract and independence evidence`. (`sec:lineage:pipeline`); (`crit:lineage:evidence-families`).

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

**Assumption (wave2 — Inherited inputs)** · `assum:guide15-exec:wave2-inherited-inputs`

Wave 0 supplies the model-law census, region-refit evidence, and accepted generation/record dispositions from bites 0.2 and 0.7; Wave 1 supplies the validated architecture-to-realization boundary and declaration weld from 1.1, never the attestation declarations themselves. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave1`).

**Gate (wave2 — Gate)** · `gate:guide15-exec:wave2-gate`

Both complete declarations validate against architecture and independent model conformance, with exact stored-field assignments, projections, tag, residual rule, and public availability. (`task:guide15:wave1`).

**Proposal (wave2 — Suggested commit)** · `proposal:guide15-exec:wave2-suggested-commit`

`realization: define burn and clear semantics`. (`task:guide15:wave1`).

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

**Gate (wave3 — Gate)** · `gate:guide15-exec:wave3-gate`

All three validated plans reconcile relation/carrier/layout/coverage/lifecycle censuses, reject focused corruptions, and agree with independent small-instance oracles. (`task:guide15:wave2`).

**Proposal (wave3 — Suggested commit)** · `proposal:guide15-exec:wave3-suggested-commit`

`compiler: expose burn ASH and clear target plans`. (`task:guide15:wave2`).

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

Wave 0 supplies explicit-only policy, arithmetic assessment, and the pinned target from 0.1 and 0.7; Wave 1 supplies the records of carrier, mixed-form, and data-output measurements from 1.0 and 1.6, requiring assessment of the actual attestation forms and any reopening under (`cond:guide15-exec:carrier-reopening`). (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave3`).

**Gate (wave4 — Gate)** · `gate:guide15-exec:wave4-gate`

Each selected form and capability has a reviewed assessment and exact evidence role; unsupported opening and unadmitted private routes remain explicit refusals or non-claims. (`task:guide15:wave3`).

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

Wave 0 supplies exact burn law, region-refit evidence, and carrier disposition from 0.2 and 0.7; Wave 1 supplies measured carrier forms and data-output policy records from 1.0 and 1.6, never owner authorization or record-credit semantics. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave5`).

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

Wave 0 supplies public-construction and signing-profile dispositions from 0.7; Wave 1 supplies the typed open-carrier ABI and construction requests from 1.4, while all-owner burn signing and exact sponsor consistency remain attestation obligations. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave8`).

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

Wave 0 supplies the event-role census and exact record rule from 0.7; the native route is inherited from Phase 6 and revalidated at 0.1. Wave 1 supplies the distinct validated evidence-family reports from 1.5, without substituting ancestry settlement for burn provenance (`sec:phase6:gate-record`). (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave9`).

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

Wave 0 supplies the gate-performance register from 0.6 and the distinct inherited target-resource subjects retained at 0.5 (`data:guide15-exec:resource-subjects`); Wave 1 supplies measured carrier forms, mixed representations, and data-output limits from 1.0 and 1.6. Complete attestation subjects require remeasurement; gate wall time is not target-resource evidence. (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`task:guide15:wave12`).

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
| Compact 72 rows (`T4-009`) | Selected compact candidate and mutation boundaries | Lost discharge or new observable gap | Current-candidate positive and focused negative campaign |
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

The second contract supplies independence evidence inside Phase 7; it introduces no third operation, aggregate covenant equation, incidental-value preservation, carrier-output closure, or consumer authorization in the covenant (`spec:lineage:exclusions`); (`crit:lineage:faithful-reuse`).

**Caveat (Measured subjects and published premises)** · `cav:guide15-exec:measurement-limits`

Its exact funding floor, input bound, relay forms, replacement premises, recurrence costs, historical availability, and finality remain bound to measured subjects and published assumptions, never universal deployment guarantees (`sec:lineage:measurements`); (`assum:lineage:availability`); (`assum:lineage:finality`).

**Record (Unchanged product thesis and accepted dependency)** · `rec:guide15-exec:roadmap-position`

The roadmap's thesis is unchanged: the attestation remains the exemplar by which the compiler is accepted, and the second contract tests independence inside the phase under (`sec:phase7:change-statement`); the accepted dependency on Wave 1 is stated under (`rule:guide15-exec:second-contract-dependency`), while the numbered phase order and normative version remain unchanged.

**Rule (Execution authority remains subordinate)** · `rule:guide15-exec:authority-boundary`

The concept remains unchanged as semantic reference, the two studies remain planning input, and this execution guide cannot override a package contract, phase card, ADR, or accepted decision (`rule:guides:authority`); (`rule:drafts:authority`).
