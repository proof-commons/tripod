# Phase 7 — Burn, ASH, and Clear · `phase:roadmap:burn-clear`

> **Status:** Active — Guide 15 is chartered as the execution guide at `plans/guides/guide_fifteen.md`; Wave 0 is the first authorized wave and opens on its backlog entry record with its provisioned pinned target and other entry obligations recorded; implementation of new semantics is held until its exit (`gate:guide15-exec:wave0`). Both phase-entry conditions hold on the amended Phase-6 exit (`sec:phase6:exit-assessment`) and accepted public policy (`sec:public-opening:result`); the guide will record the required dispositions of the accepted compact-ASH candidate at its recorded scope and the concept's detailed entry and preflight requirements in (`tab:guide15-exec:preflight`).
> **Entry:** (`gate:phase6:exit`) on its amended assessment and accepted public-declassification policy; these remain the two phase-entry conditions, distinct from the execution guide's compact-candidate and detailed entry/preflight dispositions.
> **Packages:** architecture, realization, compiler, target-elements, tapscript, linker, transaction, vectors; target-elements-conformance only for required target-generic evidence support, with model conformance and labels verification.
> **Operations:** `burn`, `compact-ash`, `clear`

## Change statement · `sec:phase7:change-statement`

Phase 7 carries the Keyed Provenance Covenant as a second compiled contract for independence evidence inside Guide 15: inherited debt and preflight close in Wave 0, and burn, ASH, and clear delivery in Waves 2–14 and Phase 7's exit depend on Wave 1's exit because the shared seams the second contract moves must settle before burn and clear depend on them; later phases consequently depend on that evidence through Phase 7 under (`rule:roadmap:sequence`), while the numbered phase order is unchanged (`rule:roadmap:change`); (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`rule:guide15-exec:second-contract-dependency`).

The supporting admission evidence is (`abst:admission:scope`), (`sec:admission:entry`) and (`sec:admission:decisions`). The second contract of record is the Keyed Provenance Covenant paper (`[A-abst:kpc:abstract]`); (`[A-def:kpc-model:first-output]`); (`[A-def:kpc-operations:classification]`). Wave 1 derives eight delivery requirements, six realization decisions, six evidence families and six target measurements from that contract, with the former study counts explicitly disposed at (`reg:guide15-exec:wave1-recount`), (`rec:guide15-exec:study-dispositions`) and (`rec:guide15-exec:evidence-recount`).

The attestation delivery requires realization and compiler contracts and the target-elements, tapscript, linker, transaction, and vectors pipeline contracts (`task:guide15:wave1`); (`task:guide15:wave2`); (`pkg:realization:contract`); (`pkg:compiler:contract`); (`pkg:target-elements:contract`); (`pkg:tapscript:contract`); (`pkg:linker:contract`); (`pkg:transaction:contract`); (`pkg:vectors:contract`); the second contract additionally tests the architecture boundary in `packages/architecture/README.md` and those shared package responsibilities (`task:guide15-exec:wave1`). Labels retains registration and citation checking, target-elements-conformance changes only for a demonstrated target-generic evidence need, and model evidence remains independent of compiler-facing semantics (`rule:labels-index:check`); `packages/target-elements-conformance/README.md`; `packages/model/README.md`.

Affected research notes are `plans/research/public-declassification.md`, `plans/research/state-constructor.md`, `plans/research/wide-arithmetic.md`, `plans/research/compiler-algorithms.md`, `plans/research/linker-algorithms.md`, and `plans/research/test-performance.md`; the six second-contract measurement results enter `plans/research/state-constructor.md` as a distinct keyed-covenant constructor evidence set, with related public-representation findings linked to (`sec:public-opening:measurements`) and gate performance kept separate in (`tab:test-performance:register`) under (`obl:guide15-exec:measurement-home`). Later findings enter each note at its own scope without promoting candidate results into calibration.

The backlog effect is the Phase-7 task table in `plans/backlog.md`, with a DONE charter import and TODO Wave-0 bites carrying the four inherited debt rows; the next gate is (`gate:guide15-exec:wave0`). Versioning impact is none: the workspace remains `0.7.0-dev`, with `0.7.N-dev` steps named as landed when citations require them under the existing identity rules.

## Goal · `sec:phase7:goal`

Implement the share-nothing owner-authorized burn path and the permissionless
public ASH maintenance path while preserving event provenance, closed-asset
closure, and conservative accounting.

## Deliverables · `sec:phase7:deliverables`

### Burn

Implement:

- nonempty bounded live-receipt inputs;
- every-owner authorization;
- no ASH or root input;
- exactly one fresh positive ASH;
- optional positive live receipt change;
- exact `U` relation;
- canonical burn records;
- optional isolated sponsor flow;
- burn event projection;
- no clear or residue projection.

### Public ASH

Implement the accepted ASH representation:

- explicit or public-committed value;
- authenticated public opening where needed;
- explicit `U` asset;
- durable public construction data;
- compact and clear lifecycle.

### Clear

Implement:

```text
B = sum(ASH inputs)
X = min(B, Y_L, Y - 1)
```

Require:

- nonempty bounded ASH input family;
- public/openable ASH values;
- canonical STATE succession;
- no RESV edge;
- `X > 0`;
- exact `tag-recon` destruction;
- residual ASH iff `B-X > 0`;
- clear event projection;
- all unaffected state fields unchanged;
- optional isolated sponsor flow;
- permissionless construction.

## Event authentication · `rule:phase7:event-authentication`

Burn credit requires both:

1. event-type anchor:
   a canonical burn transition with live inputs and one fresh ASH;

2. value anchor:
   accepted record total does not exceed fresh ASH.

This implements (`[RZ-sec:ledger:authentication]`).

Compaction and clear must never produce a burn event.

## Required vectors · `sec:phase7:vectors`

### Burn

- valid full/partial burn;
- multi-input and multi-owner;
- missing owner;
- time-locked input;
- ASH input;
- wrong change class;
- undeclared `U` output;
- no/two ASH outputs;
- wrong ASH amount;
- record under-claim;
- record over-claim;
- duplicate/skipped record ordinal;
- record mutation after signing;
- bare record payload without burn event;
- confidential closed asset.

### Compact ASH

- value-preserving compaction;
- no burn event;
- public opening use;
- malformed/missing opening rejection.

### Clear

- exact clear;
- oversized ASH partial clear;
- residual ASH;
- no clear to zero supply;
- zero-progress rejection;
- wrong decrement;
- wrong destruction amount/tag;
- RESV input/output substitution;
- wrong STATE successor;
- private owner secret requirement;
- residual opening unavailable to a fresh constructor.

## Representation evidence · `sec:phase7:representation`

If private source receipts are supported:

- compare explicit and private burn semantic projections;
- prove fresh ASH public authentication;
- prove residual blinding closure;
- prove unrelated future compaction and clear;
- keep explicit `U` asset identity.

If direct private burn is not accepted, require the documented normalization or
explicit-only path and do not claim direct support.

## Resource evidence · `sec:phase7:resources`

Measure:

- maximum burn inputs/change/records candidates;
- maximum ASH compact batch;
- maximum ASH clear batch;
- public-opening verification;
- residual branch;
- maximum sponsor candidate;
- deepest control paths.

## Exit gate · `gate:phase7:exit`

Phase 7 exits when:

- burn event type and record value anchors are independent and complete;
- over-claiming invalidates records without erasing burn provenance;
- compact ASH is attestation-silent;
- clear uses actual authenticated ASH values;
- clear preserves the `Y-1` floor and RESV non-use;
- ASH lifecycle is publicly constructible;
- no owner recovery or closed-asset escape exists;
- model/target event projections agree;
- all representation, resource, and relation reports pass;
- output remains reproducible and the tree remains clean.

## Numbered rulings · `sec:phase7:rulings`

This section is the record that holds Phase 7's numbered rulings in the Phase-6 form, each taken on the merits and open to revision at the candidate audit. Numbering continues across the two cards so that each ruling number is unambiguous: the Phase-6 card ends at ruling 127, and Phase 7 begins at ruling 128 (`rule:phase6:wave13-rulings`); the section supplies the prerequisite at (`pre:wave0:ruling-location`).

### Wave-0 rulings · `rule:phase7:wave0-rulings`

The Wave-0 compact-candidate selection is ruled here on the merits as ruling 128, open to revision at the candidate audit like every ruling on this card. It is taken over the closure records of bites 0.2, 0.3 and 0.4a, whose classification, byte elision and negative manifest establish the candidate and its outstanding evidence, rather than over the task text alone (`rec:backlog:wave0-closure-audit`); (`rec:backlog:wave0-closure-elision`); (`rec:backlog:wave0-compact-negative-manifest`).

128. The compact-ASH candidate selected for the native campaign in bite 0.4's second half and the compact revalidation required by bite 0.5 on the pinned target is the program the tree builds at `0.7.16-dev` (`step:wave0:bite-0-4-native-campaign`); (`task:guide15-exec:bite-0-4b`); (`rec:backlog:wave0-entry`); (`rec:backlog:wave0-closure-elision`). Its identity is fixed by `FIXTURE_MERKLE_ROOT` and `FIXTURE_REFERENCE_OUTPUT_KEY` in `packages/target-elements-conformance/src/reference.rs`, and `PINNED_PROGRAM` in `packages/vectors/src/bundle.rs`. Its compact census is 20 symbols, previously 22, under `the_symbol_table_is_exactly_the_guide_list_and_every_relocation_is_backed`; 55 relocations, previously 61, under `the_measured_shape_of_the_candidate_is_exactly_this`; and 60 pushed sites, previously 66, under `the_compact_demonstration_has_exactly_sixty_pushed_symbol_sites`, all in `packages/tapscript/src/tests/bundle_tests.rs`. Its linked total is 3,861 bytes, previously 3,975, under `the_linked_script_bytes_are_the_pre_link_ones_moved_by_the_resolved_widths` in `packages/linker/src/tests/link_tests.rs`; the before figures remain beside the moved assertions, and the census is recorded at (`rec:backlog:wave0-closure-elision`). Its exact leaf bytes and roots are held in `packages/vectors/fixtures/closure-compact-live.golden`, regenerated and compared by `closure_leaf_bytes_and_roots_match_the_fixture` in `packages/vectors/src/first_party.rs`. The closure-exact audit classified every retained check by its closure law, and the elision removed the sponsor-change program and version equalities made surplus by the sponsor's own all-inputs-all-outputs signature; every retained leaf instruction has a stated closure reason, with the regional residuals named as residuals at their fragments (`rec:backlog:wave0-closure-audit`); (`rec:backlog:wave0-closure-elision`). The rejected alternative is the inherited candidate proposed at (`data:backlog:entry-compact-revision`): it carries those surplus destination equalities and its former pins no longer identify the current program, so selecting it would run the campaign against a program the closure audit has superseded. Its Phase-4 compact positive remains historical acceptance at its original scope, and the negative inventory's historical native discharge remains evidence only for its producing candidate; neither credits this candidate (`rec:backlog:wave0-closure-elision`); (`rec:backlog:wave0-compact-negative-manifest`). The entry proposal remains the record of what was proposed; this ruling supplies the selection it deferred for (`data:guide15:compact-revision`). Every compact-ASH native observation admitted under Wave 0 must name this candidate by the pins above; the STATE subjects of bite 0.5 keep the maturity control's own candidate standing recorded at (`rec:backlog:wave0-entry`) and are not selected by this ruling. This ruling accepts the candidate selection and no native observation; a later change to its bytes reopens this ruling rather than inheriting its acceptances.

129. At `0.7.20-dev`, the retained warm observation occupies 891.8 s and exposes independent capture and executor work after serial Cargo barriers (`sec:test-performance:causes`); (`sec:test-performance:recommendations`). The shared-output audit establishes fresh temporary fixtures and no shared package or serial-Cargo output for `executor-boundaries`, `live-native-v2-r7-capture-contract` and `live-native-v2-r8-capture-contract`. Add `priority: 1` to exactly those three `test()` declarations in root `meson.build`, changing their implicit priority from 0 and scheduling them with the initial parallel work. Change no dependency declaration, command, argument, suite, timeout or parallelism setting. Preserve all 56 checks, both package profiles, every test outcome and timing stream, the serial Cargo barriers, and the final serial `clean-tree` check at priority −100. Preserve fixture construction, executor teardown and budget comments. Acceptance requires the same measurement sequence with observed cache classification, a lower comparable warm command wall, equal executed and ignored counts per package, unchanged attribution, and no new failure or skip (`sec:test-performance:constraints`); (`sec:test-performance:tracking`). The observable remainder is contention and variance, the unchanged expensive package tests, and timing dimensions the retained artifacts cannot separate; a wall supplies no calibration claim (`cav:guide15-exec:performance-scope`).

130. Ruling 58’s construction refusal rests on the reduced STATE leaf carrying no sponsor check and on sponsor relations awaiting operation regions; the first ground stands under ruling 40, while the completed region refit changes the second by giving `SponsorIsolation` the announcement’s own fee-sponsor flows, exact membership and input-member authorization, scoped to the announcement’s operation region (`rule:phase6:wave6-rulings`); (`rule:phase6:wave7-rulings`); (`rec:backlog:wave0-region-refit`). C1 builds the typed STATE request’s sponsored arm with and without sponsor change: the coordinator remains at input zero, sponsor inputs occupy the suffix beginning at one, the authenticated STATE successor remains at output zero, and the sponsor offer supplies the fee and optional change, with every sponsor input authorized by its own signer over the finalized transaction. Construction retains the complete spent-output census, freezes protected bytes before signing and preserves the sponsor-local capability boundary; confidential sponsor values require a balanced form with its proofs where the selected policy claims support. The STATE leaf retains its existing local checks and singleton premises; region-scoped `SponsorIsolation` supplies model membership and authorization constraints, while `SponsorEnvelopeMultiplicity` remains model-scope resource policy enforced by no leaf, and sponsor-value conservation remains a target obligation (`tab:backlog:region-relations`). C1 exercises missing and surplus membership, protocol overlap, absent member authorization and excess envelopes at the refitted observation boundary, malformed or empty offers and foreign sponsor assets at construction, and missing or mismatched sponsor responses and protected-field mutation at finalization or signing; none of those refusals substitutes for a target refusal at another boundary (`step:wave0:bite-0-5-c1`). C2 owes funded acceptance of the selected forms and the nine waiting positive controls under explicit positive relay and block fee floors, retaining exact fee amounts, policy, submitted subjects, acceptance and transaction readback, together with each observable negative at its declared boundary; zero-floor acceptance supplies none of that result (`gate:guide15-exec:bite-0-5-conjunct-7`). C3 retypes the fourteen sponsor rows and the two staged signing waits by section and row name only after their required evidence is admitted, preserving each former reason beside its disposition; region-only findings remain distinguished from target behavior (`rec:backlog:wave0-sponsored-ruling`); (`rec:backlog:wave0-safety-native-staging`); (`gate:guide15-exec:bite-0-5-non-answer-gate`). This ruling selects construction and supplies no observation: conjunct 14 still requires binding the sponsored composition to its exact operation regions, and removing this form requires a replacement route retaining every observable obligation (`pre:guide15-exec:sponsored-order`); (`gate:guide15-exec:bite-0-5-conjunct-14`); (`rule:guide15-exec:observable-retention`).

131. The position of (`task:guide15-exec:disposition-9`) is (`case:wave0:generation-preferred`): synthetic origin, no generation crossing, maturity revalidation and no migration claim for this candidate. The one-member constructor generation admits no operator-key migration relation, so `16.4 / stale-operator` is outside the admitted contract and takes `NativeCoordinateAbsentFromCandidate`; re-signing under another published signer remains the admitted wrong-operator subject (`rec:backlog:wave0-native-route-settlement`). The intended expanded static leaf set and its maturity revalidation remain owed. The source subject's execution-exit reference remains owed in the open generation-policy entry of (`tab:guide15-exec:preflight`), joined by (`step:wave0:bite-0-7-join-entry-exits`); this ruling supplies its accepted position without closing that execution exit. This scope disposition partially satisfies (`gate:guide15-exec:bite-0-5-non-answer-gate`), credits no observation and remains open to revision at the candidate audit.

132. The admitted announcement derives successor metadata and has no independent successor-metadata witness among its seven ABI roles, so `16.11 / predecessor-and-successor-metadata-witnesses-exchanged` is outside the admitted contract and takes `NativeCoordinateAbsentFromCandidate` (`step:wave0:bite-0-5-native-routes`); (`rec:backlog:wave0-native-route-settlement`). The keyed settlement table and candidate-premise test retain this build property; a third witness schedule and changed candidate are not selected. This scope disposition partially satisfies (`gate:guide15-exec:bite-0-5-non-answer-gate`) and supplies no refusal or acceptance observation. Any later schedule change reopens the premise and admitted-archive compatibility under (`cond:wave0:baseline-freeze`); the ruling remains open to revision at the candidate audit.

133. No operation on the admitted candidate spends an announced state: the only STATE transition produces announced metadata, the announcement leaf requires unannounced predecessor maturity, the metadata commitment leaf aborts and the linked subtree contains no support leaf. Thus `16.6 / wrong-control-recipe` has no successor control position and takes `NativeCoordinateAbsentFromCandidate` (`step:wave0:bite-0-5-native-routes`); (`rec:backlog:wave0-native-route-settlement`). The two-announcement chain has no accepted second control and replaces no missing observation; its former preference is refuted. The announcement module declares cycle, clear, redeem, admit-deposits and receipt-relabel STATE exits that the dispatch leaves unrealized. Realizing any of these exits, including the maturity-plus-clear expansion retained as owed by ruling 131, reopens this premise and the row's finality. The announcement-byte coordinate remains retired, and the successor recipe construction part of (`gate:guide15-exec:bite-0-5-conjunct-5`) is RETIRED with the same precise non-claim: no operation on the candidate spends an announced state; no successor control position exists. This build-property disposition partially satisfies (`gate:guide15-exec:bite-0-5-non-answer-gate`), leaves the two observable constructions under (`gate:guide15-exec:bite-0-5-native-gate`) open, and remains open to revision at the candidate audit.

134. The bare layer verdict cannot show which check of the leaf refused, so the row's own check is not shown to be the one that answered. Diagnostic recapture therefore remains Wave 0 work for the twenty-five claims named in (`req:w0route:recapture-constraints`), supported by twenty-three distinct staged subjects pinned by `shared_pair_keys_map_to_one_subject` for the two shared pairs in `packages/vectors/src/maturity_mutants/native_staging.rs` and `diagnostic_expectations_match_exact_staged_keys_and_shared_subjects` for the count in `packages/vectors/src/maturity_native.rs`; (`step:wave0:bite-0-5-diagnostic-recapture`); (`gate:guide15-exec:bite-0-5-native-gate`). The executor records only a closed-vocabulary script-error key or `unclassified` inside the existing observation detail; a separate capture binds each subject to its accepted family control and direct-block result, and separate admission records its class beside the row's expected class. The retained archive's bare script-path refusal for `16.6 / successor-from-wrong-semantic-metadata` owes attribution on the same ground. Its re-signed subject joins the diagnostic schedule with `InvalidCurveRelation` expected at successor reconstruction's `TweakVerify`; the table covers twenty-five claims on twenty-three subjects (`rec:backlog:wave0-diagnostic-route-capture-driver`). A different recorded class is a finding beside the unchanged standing. This ruling preserves the existing boundary credit, every row standing, census and staged pin; it credits no new observation, supplies no exact instruction-site attribution from a class shared by several checks, and closes neither the native gate nor the bite 0.5 packet.

135. (a) The "through observation" clause of (`gate:guide15-exec:bite-0-5-non-answer-gate`) is read with its second sentence and (`def:guide15-exec:non-answer-finality`): a waiting row is discharged by a performed observation at its declared boundary or by a final reason on admitted grounds that names neither a missing node, absent provisioning nor an unperformed observable experiment. The thirty-five charter keys comprise twelve retained finals and twenty-three former waits; fifteen waits are observed and eight are final, giving fifteen observed and twenty final charter rows, with zero retired or waiting (`result:backlog:close-safety-reconciliation`). Seven of the eight new finals are pre-target ABI-layout claims: three construction, three region and one zero-role claim, whose answers are held by the landed transaction tests before target submission; each former reason remains beside its disposition at (`tab:backlog:sponsored-retyping-ledger`). The confidential final is bound to admitted explicit/confidential agreement at (`rec:backlog:wave0-confidential-route-admission`). The seven additional coordinate finals give twenty-seven current typed finals and zero typed waits; finality adds no observed answer. The non-answer gate is MET. Each final premise and its archive compatibility reopen under (`cond:wave0:baseline-freeze`), retaining the candidate-property readings of rulings 131–133. (b) The boundary in (`gate:guide15-exec:bite-0-5-native-gate`) is the declared layer, reached by every credited refusal in its admitted archive before diagnostic recapture. The separate admission at (`rec:backlog:wave0-diagnostic-route-admission`) binds twenty-five claims on twenty-three subjects and records twenty-four agreements, one difference and zero absent claims. The difference is `16.11 / witness-item-reorder`: its `None` / `UnnameableFirstCheck` expectation meets the recorded stack-size refusal read as `StackUnderflow` inside the leaf at the script-path boundary. This is a finding beside its unchanged `NativeRefusalObserved` standing, not a refusal at another boundary. Attribution adds no answer and moves no standing, census or pin, as ruling 134 requires. The frozen manifest binds the expected-class table, which is not revised after admission to remove the finding. The native gate is SATISFIED for bite 0.5. Ruling 134's non-claim remains: a class shared by several checks supplies no unique instruction-site attribution. Both parts remain open to revision at the candidate audit.

136. The twenty classification questions listed at (`rec:backlog:wave0-compact-campaign`) retain their published manifest identities and expected-boundary fields, and their typed formal closures are final at the selected candidate under (`gate:guide15-exec:bite-0-4-gate`). For these formally closed rows, `ScriptPathRejection` identifies the inherited manifest classification and asserts neither an observable refusing coordinate nor an observed script refusal; this reading reconciles (`rule:guide15-exec:compact-negative-row`) with the typed answers. The two sponsor-authorization closures and eighteen absent-coordinate closures remain distinct: the latter comprise six canonical-delta, two open-flow, four sponsor-isolation, two root and four projection requirements. Compiler coverage, archive bytes, key sets, closure tests and the 48 / 20 / 6 partition are unchanged. This reading is taken because the alternative fourth classification class moves coverage, test bindings and archive compatibility for no new evidence. The disposition remains open to revision at the candidate audit.

137. The two compact `constructibility-witness-unavailable` identities receive a formal candidate-specific disposition because `compiler::validate_required_dependency` refuses the private dependency as `PermissionlessPrivateDependency` under Compact's own permissionless authorization, making the requested `ConstructibilityWitnessUnavailable` class unreachable under that authorization. The observed refusal at the more specific class remains beside the formal disposition as evidence and receives no credit as the requested class. `the_same_case_does_not_discharge_the_neighbouring_class` and `another_operations_authorization_cannot_answer_compact_witness_coverage` in `packages/vectors/src/first_party.rs` retain the exact class and operation boundaries. No production validator is refactored to produce a differently labelled refusal: that alternative would change the validation boundary to satisfy a classification rather than supply new evidence. This is the argued out-of-scope declaration of (`rule:guide15-exec:compact-first-party`); the requested class is unreachable under the candidate's authorization, so no observable obligation is removed (`rule:guide15-exec:observable-retention`). First-party closure counts this pair as two formal rows beside the observed rows, retaining its candidate premise under (`cond:wave0:baseline-freeze`); the disposition remains open to revision at the candidate audit.

138. The two compact `missing-clear-exit-emitted` identities close through a declaration comparison that accepts the emitted candidate retaining its outstanding Clear lifecycle declaration and refuses removal of that declaration. This is the admitted claim because the candidate carries the outstanding obligation but has no emitted Clear path; no emitted Clear path exists in this candidate and no emitted-path claim is made. An invented accepting Clear path would answer a different candidate and supplies no evidence for this pair. The required comparison is performed at the typed emitted-lifecycle entry and admitted by the explicit ledger at (`rec:backlog:wave0-compact-inherited-closure`). Clear's realization landing in Wave 2 reopens the disposition and its evidence compatibility in the candidate-bound form of ruling 133 under (`cond:wave0:baseline-freeze`); the disposition remains open to revision at the candidate audit.

139. (`T4-009`) closes as its own bounded work before (`task:guide15-exec:bite-0-7`), while (`T12-004`) stays DONE at its recorded 48-native scope; completed native evidence cannot discharge the remaining entries merely by dependency. The allowed set for the remaining (`T4-009`) work, extending (`setup:guide15-exec:bite-0-4-files-and-packages`), is exactly `packages/compiler/src/constructibility.rs`, `packages/compiler/src/lifecycle.rs`, `packages/compiler/src/operation_plan.rs`, `packages/compiler/src/sponsor_region.rs`, `packages/compiler/src/error.rs`, `packages/compiler/src/lib.rs`, `packages/compiler/src/tests/operation_plan_tests.rs`, `packages/compiler/src/tests/lifecycle_tests.rs`, `packages/compiler/src/tests/mod.rs`, `packages/compiler/meson.build`, `packages/tapscript/src/pattern.rs`, `packages/tapscript/src/bundle.rs`, `packages/tapscript/src/lib.rs`, `packages/tapscript/src/tests/bundle_tests.rs`, `packages/tapscript/src/tests/pattern_tests.rs`, `packages/tapscript/src/tests/mod.rs`, `packages/tapscript/meson.build`, `packages/vectors/src/first_party.rs`, `packages/vectors/src/compact_negative_manifest.rs`, `packages/vectors/src/plan.rs`, `packages/vectors/src/report.rs`, `packages/vectors/src/compact_corpus.rs`, `packages/vectors/src/error.rs`, `packages/vectors/src/lib.rs`, `plans/backlog.md`, `plans/guides/guide_fifteen.md`, `plans/phases/07-burn-and-clear.md`, `plans/drafts/wave0-completion-strategy.md`, `plans/packages/compiler.md`, `plans/packages/tapscript.md`, `plans/packages/vectors.md`, and the new test files `packages/compiler/src/tests/compact_validation_tests.rs` and `packages/tapscript/src/tests/compact_validation_tests.rs`. The new entries are additive validators that change no emission or acceptance semantics; any such change is outside this work and needs its own review. The current gate and baseline wording moves to 74 and 48 / 20 / 6, retaining 72 / 48 / 18 / 6 as before-figures so comparisons preserve their denominators. The extension supplies no Burn or Clear implementation licence and leaves (`setup:guide15-exec:bite-0-7-files-and-packages`) unchanged; (`rec:guide15-exec:bite-0-7-closure-carrier`) still requires each inherited closure on its own evidence. This bounded extension is taken because the missing entries need explicit package authority without reopening the completed native half or expanding preflight into implementation; it remains open to revision at the candidate audit.

140. Accept (`task:guide15-exec:disposition-1`) through the complete typed-owner census: architecture owns operation, object, tag, quantity and projection identifiers; model transitions and certification own the executable inherited law; realization owns semantic metadata and the delivered declaration graphs. Clear stores `y_l` and derives Y. Burn and Clear declaration dispatch and validation remain Wave-2 obligations; (`cond:guide15:operation-scope`) and (`cond:guide15:validated-relations`) retain DOES NOT HOLD. No target type or new semantic law is introduced. Evidence: (`rec:backlog:wave0-preflight-census`) binds `packages/architecture/src/ids.rs` and `spec.rs`, the transitions in `packages/model/src/ops/burn.rs` and `ops/ash.rs`, `PoolState` in `packages/model/src/pool.rs`, `StateMetadata` in `packages/realization/src/state.rs`, and `derive_operation` in `packages/realization/src/declarations/mod.rs`; `clear_changes_only_the_stored_y_l_and_the_derived_total` in `packages/model/src/tests/ash_clear_tests.rs` supplies the focused stored-field check. Reopen at the candidate audit if an owner, relation, field or boundary is omitted or changed.

141. Accept (`cond:guide15:public-revalidation`) at the recorded explicit-only target boundary and accept (`cond:guide15:private-route`) as normalized-first with private change. PublicCommitted and authenticated direct-private opening remain deferred on the three recorded opening blockers. Semantic representation vocabulary is unchanged; no scalar attachment supplies authentication and no Burn execution result is claimed. Evidence: (`rec:backlog:wave0-preflight-census`) joins (`sec:public-opening:result`) to `RepresentationMode` in `packages/realization/src/domain.rs` and compact's representation declaration in `packages/realization/src/declarations/compact_ash.rs`. Reopen at the candidate audit on changed target capability, authenticated opening evidence or selected policy.

142. Accept (`task:guide15-exec:disposition-4`) and (`cond:guide15:ash-compatibility`) as the requirement that fresh, compacted and clear-residual ASH use an exactly equal constructor under one target, deployment, representation and linked bundle. The existing compact constructor supplies the inherited subject; semantic `Meta::Ash` equality supplies no cross-operation linked-byte proof. Waves 5 and 8 retain that proof. Any proposed class transition requires explicit edge obligations and renewed acceptance. Evidence: the constructor recipe and class table at (`rec:backlog:wave0-preflight-census`) join the ASH lifecycle in `packages/architecture/src/spec.rs`, emissions in `packages/model/src/ops/burn.rs` and `ops/ash.rs`, compact lifecycle declarations in `packages/realization/src/declarations/compact_ash.rs`, and ruling 128's selected bundle with `LinkedConstructor` in `packages/linker/src/bundle.rs`. Reopen at the candidate audit on a proposed class transition or changed target, deployment, representation or linked constructor recipe.

143. Accept (`task:guide15-exec:disposition-6`) as zero through the configured `BurnRecordMax` records, contiguous zero-based ordinals, positive individually valid amounts, and identity bound to the burn transaction and record ordinal. Repeated addresses remain permitted. A valid burn produces exactly one fresh ASH; aggregate record claims at most that ASH value receive credit. Underclaim is donation; overclaim remains a valid raw burn whose record set receives no credit, including sums outside the individual amount domain. Record credit is derived and never stored as authority. Evidence: (`rec:backlog:wave0-preflight-census`) binds `derive_burn_records` in `packages/model/src/certify.rs`, `BurnTransaction::records_accepted` and `validate_burn_records` in `packages/model/src/ledger.rs`, and the genuine-burn, donation, overclaim, beyond-domain overclaim and ordinal tests in `packages/model/src/tests/burn_attestation_tests.rs`. Reopen at the candidate audit on a change to typed record or accounting authority.

144. Accept (`task:guide15-exec:disposition-5`) and (`cond:guide15:signing-profiles`) through the census packet’s exact candidate carrier and protected-byte table. Select a chain-public unspendable data-output carrier, ordered by canonical record ordinal, associated with the unique fresh ASH by authenticated Burn recognition. A copied payload alone establishes no Burn provenance. Select finalized all-inputs/all-outputs authorization for the applicable receipt and sponsor inputs, binding fresh ASH, change, records, sponsor region and fee. Preserve ruling 143’s value law. Size or policy refusal under the attestation carrier measurements retained in Wave 4 (`cond:guide15-exec:carrier-reopening`) reopens the carrier by a Wave-4 ruling before declarations. Evidence: the proposed carrier packet at (`tab:backlog:preflight-carrier`) and signing-role table at (`rec:backlog:wave0-preflight-census`) bind `BurnRecord` in `packages/model/src/history.rs`, the serialization functions in `packages/transaction/src/bytes.rs`, `protected_preimage` in `packages/transaction/src/live_finalize.rs`, `authorize_live_transfer` in `packages/transaction/src/live_signing.rs`, and `SighashProfile` in `packages/transaction/src/sponsor.rs`. Encoding and costs are proposed and analytically derived, not a measured Burn transaction. Reopen at the candidate audit on changed carrier or signing premises, including size or policy refusal under the attestation carrier measurements retained in Wave 4 (`cond:guide15-exec:carrier-reopening`); the required Wave-4 ruling precedes declarations.

145. Accept (`task:guide15-exec:disposition-8`) through the recorded checked fixed-width schedule. Authenticate nonnegative operands below `2^51`; check each accumulated sum against the semantic amount domain; derive Y, require positive Y, derive `Y−1`, select the smaller authenticated operand twice, require positive X, and derive residual and successor `y_l`. Every arithmetic success flag is consumed by verification and every comparison boolean by its selection or verification; no witness chooses X. Check `B=X+residual`, `y_l=y_l′+X`, and derived `Y′=Y−X`, with other stored fields unchanged. No multiplication, division or wider arithmetic is required. Evidence: (`tab:backlog:preflight-clear-schedule`) in (`rec:backlog:wave0-preflight-census`) binds `ClearAsh::apply` in `packages/model/src/ops/ash.rs`, `Sat` in `packages/model/src/scalar.rs`, `ProtocolAmount` in `packages/realization/src/domain.rs`, and `arithmetic_opcodes`, `comparison_opcodes`, `conversion_opcodes`, `stack_opcodes`, `verification_opcodes` and `byte_string_opcodes` in `packages/target-elements/src/opcode.rs`, together with the abstract validator in `packages/tapscript/src/stack.rs`. Reopen at the candidate audit on changed bounds, operand encoding or opcode contract.

146. Retain ruling 131’s acceptance of (`task:guide15-exec:disposition-9`) and explicitly accept (`cond:guide15:generation-policy`) and (`cond:guide15:origin-nonclaims`): the retained maturity-only constructor is the predecessor subject; the proposed static set contains maturity and Clear, with metadata commitment retained separately. The new generation starts synthetically, admits no old-generation spend through an uncommitted Clear leaf, requires maturity revalidation and public recovery, and claims no migration. Evidence: ruling 131 and the generation manifest at (`rec:backlog:wave0-preflight-census`) bind `StateMetadata` and its codec in `packages/realization/src/state.rs` and `state_codec.rs`, `CandidateStateConstructor` in `packages/tapscript/src/state_constructor.rs`, and `linked_subtree` in `packages/linker/src/state_bundle.rs`. Reopen at the candidate audit on changed static leaves or a proposed migration.

147. Accept (`cond:guide15:separate-evidence`) as separate claims for operation acceptance, Burn event recognition, record-value credit, compact silence, Clear event, Clear accounting and STATE succession. Raw event equality, query equality, accounting equality, history, continuity, safety and resource claims retain their own producers and premises; one report’s success cannot discharge another role. Phase-7 producers remain owed in Waves 10–12. Evidence: (`rec:backlog:wave0-preflight-census`) binds `TransitionCertificate`, `BurnProjection` and `ClearProjection` in `packages/model/src/history.rs`, `RecognizedAttestationEvent`, `compare_attestation_events` and `compare_attestation_query` in `packages/model/src/ledger.rs`, and `ReceiptAccountingAuditProjection` in `packages/model/src/audit.rs`. Reopen at the candidate audit on merged roles or changed observation bindings.

148. Accept (`task:guide15-exec:disposition-11`) and (`cond:guide15:public-ash`) as a fresh-process construction test contract using public chain data, linked bundle and ABI, ASH values, STATE metadata and public constructor data, plus only constructor-local sponsor capability where selected. No ASH creator secret is an input. Existing compact evidence is inherited only at its recorded scope; full fresh-to-compact-to-clear construction remains Waves 5 and 11 work. Evidence: the fresh-process input and secret-dependency census at (`rec:backlog:wave0-preflight-census`) binds the permission classes in `packages/architecture/src/spec.rs`, `CompactAsh` and `ClearAsh` in `packages/model/src/ops/ash.rs`, the constructibility graph in `packages/realization/src/declarations/compact_ash.rs`, its validation in `packages/realization/src/constructibility.rs`, and public STATE reconstruction through `CandidateStateConstructor` in `packages/tapscript/src/state_constructor.rs`. Reopen at the candidate audit on any hidden dependency or changed constructor input.

### Wave-1 rulings · `rule:phase7:wave1-rulings`

These nine rulings state the contract and its engineering boundary, each on its merits and open to revision at the candidate audit. They authorize no claim that the planned realization or evidence already exists.

149. The Keyed Provenance Covenant paper is Wave 1's contract of record (`[A-abst:kpc:abstract]`); (`[A-app:kpc:verification]`). A single registered specification lets declarations, proofs and evidence cite the same statements; the execution guide assigns work against it without changing its laws. Reopen at the candidate audit on a proposed paper edition or an unresolved conflict between a declaration and its cited specification.

150. The whole covenant is exactly born first and continue first: the spent output was output 0 of its creating transaction, and the spending transaction's output 0 bears the same keyed covenant (`[A-def:kpc-model:first-output]`). Both checks are necessary to obtain single birth and unique continuation; no additional condition is justified by convenience of a realization. Reopen at the candidate audit on a paper edition that changes either clause.

151. The key is fixed when the covenant is generated, and incompatible systems coexist on one ledger (`[A-def:kpc-model:keyed-membership]`); (`[A-thm:kpc-invariants:key-isolation]`). Disjoint authenticated membership prevents a handle from changing systems through a successor while permitting independent histories for different keys. Reopen at the candidate audit on changed key-generation or authenticated-membership premises.

152. The record is the SOURCE handle itself: an event creating a handle while consuming none of its key assigns that handle its singleton record (`[A-op:kpc-operations:source]`). Fresh output identity already binds the creating event, so no separate marker or recognition-dependent record is needed. Reopen at the candidate audit on changed SOURCE identity or output-freshness premises.

153. MERGE assigns output 0 the union of consumed record sets and introduces no record (`[A-op:kpc-operations:merge]`); (`[A-def:kpc-operations:classification]`). Classification depends only on consumed handles, so other created data cannot poison reconstruction or add a record during a merge. Reopen at the candidate audit on changed input classification or record-set recurrence.

154. The system has no destruction output, burn requirement, counter, accumulator, magnitude, depth, designated head or terminal (`[A-sec:kpc:operations]`); the whole covenant adds no destruction condition (`[A-def:kpc-model:first-output]`). Record sets and successors are derived from history; carrying a quantity or selecting a terminal would add a mechanism the two clauses do not need. Reopen at the candidate audit on a proposed additional mechanism or covenant condition.

155. Acceptance of amount, attribution and eligibility belongs to the reader's published rule; it changes no record set and the covenant checks none of it (`[A-rem:kpc-interface:record-evidence]`). A fixed predicate on immutable SOURCE contents preserves path growth, while a changing reader rule has no such unconditional guarantee; keeping the distinction avoids mistaking an application policy for a spending condition. Reopen at the candidate audit on changed reader acceptance rules or immutable-content premises.

156. Finality is a named evidence assumption alongside inclusion and retrieval, required for a reading claimed final and enforced by no covenant clause (`[A-sec:kpc:interface]`). Reconstruction proves the set for a supplied valid history; it cannot prove that no replacement history will later be accepted. Reopen at the candidate audit on changed inclusion, retrieval or finality assumptions.

157. The earlier destruction-lineage study is superseded as Wave 1's contract and retained as the record of the reduction (`task:guide15-exec:wave1`); `plans/drafts/destruction-lineage-exemplar-study.md`. Its destruction-pair, designated-genesis and consumer-bound premises cannot govern the new evidence. Its separate attestation carrier measurement obligation survives in Wave 4 under (`cond:guide15-exec:carrier-reopening`), so retiring the old contract does not silently discharge an attestation proof. Carrier evidence arrives after Wave 2’s typed declarations and Wave 3’s plans; a Wave-4 refusal must name those artifacts among what it reopens at (`gate:guide15-exec:wave4-gate`). Reopen at the candidate audit on a proposed use of the superseded study as contract authority or a changed carrier measurement subject.

### Wave-1 realization rulings · `rule:phase7:wave1-realization-rulings`

These eight rulings choose realization routes for Wave 1, each on its merits and open to revision at the candidate audit. Where a ruling names an acceptance condition, its route is the one the next bites implement and measure, and acceptance as the realization's mechanism, home or figure waits for that condition. None claims that the realization or its evidence exists.

158. Rule now to test the explicit semantic profile first, because the all-ID census makes dispatch-only reuse insufficient and immediate crate extraction costs more. Acceptance condition for the home: the standalone boundary test disposes the identifier choice, baseline conditional census and first-contract regressions without refuting the profile. Evidence: (`proposal:w1route:boundary-test`) binds the census in `packages/architecture/src/validate.rs:620`, architecture validation entry at `packages/architecture/src/validate.rs:194`, `derive_operation` at `packages/realization/src/declarations/mod.rs:11`, `bind_input` at `packages/compiler/src/input.rs:171` and existing operation branches at `packages/compiler/src/placement.rs:266`. Reopen at the candidate audit on cross-profile leakage or a new profile/keyed-operation conditional outside architecture validation entry, derive_operation and bind_input; count new relation variants and compile-forced discharge arms separately.

159. Place the independent finite keyed-history oracle in vectors as bite-1.5 work, with the proposed import-boundary test; its conformance-only consumers do not justify a new model crate. Evidence: (`proposal:w1route:model-home`) and (`crit:w1route:model-extraction`) distinguish the proposed vector-owned oracle from the sealed model vocabulary in `packages/model/README.md:3` and the existing reader in `packages/vectors/src/maturity_continuity.rs:1958`. Reopen at the candidate audit on an actual model API consumer or failed oracle independence evidence.

160. Rule now on the paper reading: flags are no born-first coordinate, and the reviewed index is compared as four bytes rather than a script number, without an input-0 or zero-flags law. Acceptance condition for current-input outpoint inspection as the realization mechanism: admit A0–A3 and the pinned index/flag reading. Evidence: (`proposal:w1route:birth-fragment`), (`proposal:w1route:birth-subjects`) and (`cav:w1route:birth-collateral`) bind `InspectInputOutpoint` at `packages/target-elements/src/opcode.rs:1001`, current-input selection at `packages/target-elements/src/opcode.rs:1162` and index encoding at `packages/target-elements/src/encoding.rs:321`. Reopen at the candidate audit on contrary encoding/flag facts, an admitted-form discrepancy or failure of the four-subject control matrix.

161. Rule now to investigate authenticated current-program equality and a sole key-committing public leaf, because this removes self-reference without additional spending predicates. Acceptance condition for the mechanism: admit measurements b and c, including the full leaf/key-path census, conditional NUMS premises and proposed tagged SHA-256 second-preimage resistance for a generated key and collision resistance where both keys are adversarial, alongside the guide's preimage premise. Evidence: (`proposal:w1route:key-recipe`), (`arg:w1route:recurrence-choice`), (`cav:w1route:constructor-assumptions`) and (`assum:guide15-exec:sha256-preimage`) bind the equality schedule at `packages/tapscript/src/pattern.rs:741`, reference strategies at `packages/linker/src/deployment.rs:40` and supplied curve capability at `packages/tapscript/src/state_constructor.rs:27`. Reopen at the candidate audit on key aliasing, unauthenticated comparison, an alternative spending path or demonstrated need for another supported reference strategy.

162. Prefer positive explicit policy-asset funding and ordinary default relay; this separates incidental funding from record semantics. Acceptance condition for the exact floor/fee recipe: admitted 1.0(e) observations, repeated on final bytes. Evidence: (`proposal:w1route:funding`) binds target-form assessment at `packages/target-elements/src/transaction_form.rs:288` and the sponsor funding/submission routes at `scripts/elements-native-executor.py:4351` and `scripts/elements-native-executor.py:4717`. Reopen at the candidate audit on changed form, asset, policy or balance-proof requirements.

163. Keep N+1 refusal labeled construction policy, so adapter/ASH limits do not become covenant laws. Acceptance condition for the construction input bound: complete measured SOURCE/MERGE range and worst-witness evidence. Evidence: (`proposal:w1route:scaling`) binds construction/weight checks at `packages/transaction/src/construct.rs:897` and the native resource observations at `scripts/elements-native-executor.py:6920`. Reopen at the candidate audit on changed witness/funding/output envelope or a failed admitted endpoint.

164. Prefer the all-authenticated-record reader with optional explicit-amount annotation and unassigned attribution, and separately test immutable and changing filters; this demonstrates reader independence without inventing record credit. Acceptance condition for final readings: publish and accept exact inclusion/retrieval/finality premises from the pinned deployment's policy. Evidence: (`proposal:wave1:reader-rule`) distinguishes reader acceptance from membership and names the finality declaration's producer; independent reading precedents are at `packages/vectors/src/maturity_continuity.rs:1958` and `packages/vectors/src/maturity_recovery.rs:416`. Reopen at the candidate audit on a real consumer predicate or changed history/replacement premises.

165. Keep internal typed evidence reports unless an actual external consumer warrants ADR-022 interchange; this avoids maintaining an unused wire contract while preserving exact subject validation. Evidence: (`tab:wave1:decisions`) records the consumer-dependent format choice under (`dec:guide15-exec:reader-evidence`), with the typed report precedent at `packages/vectors/src/report.rs:429`. Reopen at the candidate audit on a named consumer and compatibility obligation.
