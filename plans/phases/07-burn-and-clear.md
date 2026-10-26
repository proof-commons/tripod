# Phase 7 — Burn, ASH, and Clear · `phase:roadmap:burn-clear`

> **Status:** Active — Guide 15 is chartered as the execution guide at `plans/guides/guide_fifteen.md`; Wave 0 is the first authorized wave and opens on its backlog entry record with its provisioned pinned target and other entry obligations recorded; implementation of new semantics is held until its exit (`gate:guide15-exec:wave0`). Both phase-entry conditions hold on the amended Phase-6 exit (`sec:phase6:exit-assessment`) and accepted public policy (`sec:public-opening:result`); the guide will record the required dispositions of the accepted compact-ASH candidate at its recorded scope and the concept's detailed entry and preflight requirements in (`tab:guide15-exec:preflight`).
> **Entry:** (`gate:phase6:exit`) on its amended assessment and accepted public-declassification policy; these remain the two phase-entry conditions, distinct from the execution guide's compact-candidate and detailed entry/preflight dispositions.
> **Packages:** architecture, realization, compiler, target-elements, tapscript, linker, transaction, vectors; target-elements-conformance only for required target-generic evidence support, with model conformance and labels verification.
> **Operations:** `burn`, `compact-ash`, `clear`

## Change statement · `sec:phase7:change-statement`

Phase 7 carries destruction lineage as a second compiled contract for independence evidence inside Guide 15: inherited debt and preflight close in Wave 0, and burn, ASH, and clear delivery in Waves 2–14 and Phase 7's exit depend on Wave 1's exit because the shared seams the second contract moves must settle before burn and clear depend on them; later phases consequently depend on that evidence through Phase 7 under (`rule:roadmap:sequence`), while the numbered phase order is unchanged (`rule:roadmap:change`); (`task:guide15-exec:wave0`); (`task:guide15-exec:wave1`); (`rule:guide15-exec:second-contract-dependency`).

The supporting evidence is (`abst:admission:scope`), (`sec:admission:entry`), (`sec:admission:decisions`) and (`abst:lineage:scope`), (`sec:lineage:requirements`), (`sec:lineage:measurements`) and (`sec:lineage:decisions`); the latter states the contract's own BURN and MERGE laws, six measurement obligations, package responsibilities, and nine decisions, with no assumed positive relay minimum or input bound.

The attestation delivery requires realization and compiler contracts and the target-elements, tapscript, linker, transaction, and vectors pipeline contracts (`task:guide15:wave1`); (`task:guide15:wave2`); (`pkg:realization:contract`); (`pkg:compiler:contract`); (`pkg:target-elements:contract`); (`pkg:tapscript:contract`); (`pkg:linker:contract`); (`pkg:transaction:contract`); (`pkg:vectors:contract`); the second contract additionally tests the architecture boundary in `packages/architecture/README.md` and those shared package responsibilities (`sec:lineage:pipeline`). Labels retains registration and citation checking, target-elements-conformance changes only for a demonstrated target-generic evidence need, and model evidence remains independent of compiler-facing semantics (`obl:lineage:labels`); `packages/target-elements-conformance/README.md`; `packages/model/README.md`.

Affected research notes are `plans/research/public-declassification.md`, `plans/research/state-constructor.md`, `plans/research/wide-arithmetic.md`, `plans/research/compiler-algorithms.md`, `plans/research/linker-algorithms.md`, and `plans/research/test-performance.md`; the six second-contract measurement results enter `plans/research/state-constructor.md` as a distinct constructor-and-carrier evidence set, with related public-representation findings linked to (`sec:public-opening:measurements`) and gate performance kept separate in (`tab:test-performance:register`) under (`obl:guide15-exec:measurement-home`). Later findings enter each note at its own scope without promoting candidate results into calibration.

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
