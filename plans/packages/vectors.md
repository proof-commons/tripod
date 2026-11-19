# Translation-Validation Vectors · `pkg:vectors:contract`

> **Status:** Candidate — compact-ASH fixtures, evidence plan and target execution, the live-transfer safety, minimality and resource studies, and the maturity run-of-record corpora, continuity, recovery, resource and refinement checks and five candidate-scope reports are delivered; one of 72 compact-ASH negative coverage rows is discharged and 71 remain outstanding; the bite-1.5 offline evidence packet is closed at (`rec:w1b15:closed`)
> **Phase:** [Phase 4](../phases/04-compact-ash.md) onward
> **Package:** `tripod-vectors`
> **Library:** `vectors`
> **Direct dependencies:** `architecture`, `compiler`, `linker`, `realization`, `tapscript`, `target-elements`, `target-elements-conformance`, `transaction`
> **Decision:** [D004](../decisions/004-translation-validation.md)

## Purpose · `sec:vectors:purpose`

`vectors` compares target-independent semantic expectations with concrete
target behavior for one exact linked bundle and ABI.

It owns:

- semantic fixtures;
- model-to-realization fact projection;
- semantic vectors;
- focused mutations;
- ABI-driven target materialization;
- target-native execution;
- accepted semantic projection comparison;
- relation activation and carrier coverage;
- representation safety and minimality reports;
- resource measurements;
- substrate reports;
- deterministic shrinking;
- independent-observer report comparison;
- canonical evidence schemas and identities.

It owns evidence, not semantics.

The keyed finite-history model is conformance-only standard-library code compiled by external tests and an isolated harness. Public construction fixtures and their event/content-address association live in `packages/vectors/tests/keyed_history/`; their expected sets are declared before construction, not taken from builder totals or declaration evaluation. The source/dependency review, refusal boundaries and supplied-parent limits are recorded at (`rec:w1b15:histories`). The conformance-library byte reader reconstructs classification from actual parent outputs and applies the recurrence, with explicit incomplete, invalid and non-handle outcomes and missing evidence retained for non-handle events too; its permitted substrate calls, public surface, conditional boundary subjects and per-instance external premises are recorded at (`rec:w1b15:reader`). It adds no production reader or chain-evidence claim and never consumes constructor roles or the model.

## Dependencies · `sec:vectors:dependencies`

Broad direct dependencies are permitted because this package compares assurance
boundaries.

No direct `tapscript` dependency is required when final programs and provenance
are exposed through `linker`.

Forbidden dependency:

```text
release
```

Release consumes vector reports, not the reverse.

## Typed inputs · `sec:vectors:inputs`

The package consumes:

- architecture and realization identities;
- executable model behavior;
- analyzed compiler relation/coverage scope;
- exact target;
- exact linked bundle;
- exact transaction ABI;
- explicit deterministic seeds and test fixtures;
- external observer reports through typed adapters.

## Typed outputs · `sec:vectors:outputs`

Separate reports cover:

- model/realization conformance;
- compiler-analysis completeness;
- backend pattern behavior;
- exact bundle target execution;
- relation coverage;
- representation safety;
- representation minimality;
- resource measurement;
- substrate claims;
- script integration;
- event projection;
- attestation query;
- receipt accounting.

Reports remain distinct even when one runner invokes several checks.

> Illustrative boundary; names and exact fields are not frozen.

```rust
pub fn execute_vectors(
	vectors: &TargetVectorRegistry,
	executor: &mut impl TargetExecutor,
) -> Result<TargetExecutionReport, VectorError>;
```

## Semantic fixtures · `rule:vectors:fixtures`

A positive fixture begins from a model-valid world produced through the model’s
declared valid path.

A corruption fixture is explicitly labeled as fault evidence.

Fixture identity binds:

- architecture/realization;
- operation;
- initial semantic state;
- request;
- order/checkpoint context;
- explicit fixture parameters.

It excludes target bytes and bundle identity.

## Expected result · `rule:vectors:expected`

Expected semantic behavior derives from:

- realization relations;
- executable model;
- typed expected projection adapters.

It must not derive from the candidate backend being tested.

Shared helpers are documented and are not described as independent evidence.

## Mutations · `rule:vectors:mutations`

A mutation records:

- source valid vector;
- intended relation;
- changed semantic facts or concrete fields;
- mutation layer;
- collateral relations;
- expected result;
- exact concrete transaction.

Mutation layers include:

- semantic fact;
- ABI/layout;
- target transaction;
- witness/proof;
- linked constructor/program.

Construction failure is not target-rejection evidence unless constructor
validation is the explicit claim.

## Target execution · `rule:vectors:execution`

Release-used execution runs against the selected target environment.

Reports distinguish:

- consensus result;
- policy/mempool result;
- package result;
- local diagnostic interpreter result;
- infrastructure failure.

Infrastructure failure never counts as expected rejection.

## Accepted projection · `rule:vectors:projection`

For accepted transactions, compare applicable:

- created object families;
- assets and public values;
- owners and classes;
- STATE/root successors;
- canonical deltas;
- open flows and fees;
- data outputs;
- event projections;
- disclosed facts;
- target-independent protocol observable.

Verdict equality alone is insufficient.

## Coverage · `rule:vectors:coverage`

Before execution, require equality among:

```text
realization relation scope
compiler relation scope
linked carrier scope
coverage requirement scope
```

Positive coverage requires active relation, reachable carrier, target
acceptance, and matching projection.

Target-negative coverage requires intended violation, target execution, and expected rejection; first-party refusal requires the exact owning validator to reject a retained malformed subject and accept its focused control, while missing entries remain explicit obligations in `packages/vectors/src/compact_negative_manifest.rs`.

Supplemental ABI requirements are indexed separately from compiler-derived coverage and use explicitly named test staging against the transaction-owned field validator; constructor pinning alone is not a malformed-input refusal.

Existing-entry STATE comparisons retain both offers in `packages/vectors/src/maturity_first_party.rs` and dispatch through the declared transaction, linker or conformance entry. Scripted protocol comparisons establish first-party evidence only. Missing typed contracts and Python request-admission routes remain required at (`rec:backlog:wave0-first-party-entries`).

Focused mutations list collateral relations honestly.

## Representation reports · `rule:vectors:representation`

Safety and minimality remain separate.

Safety vectors reject:

- wrong asset;
- confidential closed asset;
- wrong class;
- wrong recipient;
- value imbalance;
- missing authorization;
- malformed proof/opening;
- unclassified output.

Minimality vectors accept supported lower-disclosure representations with the
same semantic projection.

## Permissionless evidence · `rule:vectors:permissionless`

A permissionless construction vector runs from a public construction view plus
sponsor-local data.

It must not access hidden owner/operator fixture state.

## Resources · `rule:vectors:resources`

For each measured transaction, report:

- target/bundle/ABI;
- operation and activation case;
- family counts;
- representation;
- predicted resources;
- observed resources;
- target and policy verdict;
- measurement tool identity.

One fixture may maximize only one resource dimension.

## Independent observers · `rule:vectors:observers`

The package validates separate external candidate reports for:

1. raw events;
2. canonical query;
3. receipt accounting.

A model wrapper or second invocation is a harness test double, not independent
deployment evidence.

Each candidate declares implementation identity and shared dependencies.

## Shrinking · `rule:vectors:shrinking`

Unexpected failures are minimized deterministically when practical.

The report preserves:

- original failure;
- shrink policy and seed;
- minimized regression;
- relation and mutation provenance.

## Identity · `rule:vectors:identity`

Vector-set and report identities bind all applicable:

- architecture;
- realization;
- compiler;
- target;
- bundle;
- ABI;
- coverage policy;
- fixture/mutation set;
- tool version;
- explicit seeds.

They exclude timing, temporary paths, host identity, and secrets.

Archive candidate identity binds deployment and executed program commitments separately from optional source revision; matching source provenance alone establishes no current acceptance.

## Assurance boundary · `sec:vectors:assurance`

The package establishes finite bundle-specific translation evidence.

An earlier candidate's capture has typed historical standing for that candidate; retained producing programs support exact replay, and unavailable current-candidate replay is explicit, without promoting archived acceptance to changed bytes.

The compact archive admits once through `compact_corpus::run_of_record` with current-candidate standing bound to `FIXTURE_MERKLE_ROOT`, `FIXTURE_REFERENCE_OUTPUT_KEY` and `PINNED_PROGRAM` under ruling 128 (`rule:phase7:wave0-rulings`), replaying the same schedule that produced the capture and crediting a row only after its paired control and an exact boundary match. A second archive of the versioned second schedule admits the same way through `compact_corpus::second_run_of_record` under its own pins and the same candidate standing, and `compact_corpus::joined_campaign` requires equal keys and pins across both, so a row observed in either archive counts once and a key neither observes keeps only a tested formal closure. The admitted dispositions bind the record's table (`tab:backlog:compact-dispositions`) through `record_dispositions_bind_exactly_to_the_current_manifest`; changed candidate bytes reopen admission rather than inherit its credit.

The additive `validate_compact_substrate_report` entry compares an offered report with an immutable `AdmittedCompactSubmission` retained after successful archive replay. `project_compact_substrate_report` projects an accepted control offline; `prepare_compact_substrate_report_comparison` binds its external substrate requirement, sponsor identity, deployment and candidate premises and checks the offered transaction's bytes, identities and settlement readback, including the admitted transaction's settlement location. Preparation refuses malformed operands separately from the three comparison classes: absent report, failed claimed outcome and a report for another exact transaction. The coherent transaction binding is one identity coordinate; a mismatched identity string alone is malformed evidence. The accepted controls and exact refused subjects cover both sponsor identities at the external-report boundary without changing the external evidence role, archive admission, derivation, emission, acceptance or closure accounting. Explicit ledger admission supplies closure accounting separately; the backlog closure is recorded separately at (`rec:backlog:wave0-compact-inherited-closure`) under ruling 139 (`rule:phase7:wave0-rulings`).

The explicit compact closure ledger joins the twenty first-party and six external-report negative requirements to exactly one typed evidence record per complete manifest key. Admission checks both directions, rejects missing, duplicate, surplus and wrongly keyed readings separately, reconstructs each expected comparison, and requires an accepted control beside its exact one-coordinate refused subject. The ledger is separate from plan derivation and archive admission; a freshly derived plan still discharges only its two canonical dependency rows, and the admitted archives retain their existing acceptance and refusal boundaries (`rule:guide15-exec:compact-first-party`).

The twenty first-party closures comprise eighteen observed comparisons and two formal candidate-specific dispositions. The unavailable-witness pair is formally closed because Compact's permissionless authorization selects `PermissionlessPrivateDependency`, making the requested `ConstructibilityWitnessUnavailable` class unreachable under that authorization; the supporting observation receives no credit as the requested class. The two emitted-Clear observations accept retention of the outstanding declaration and refuse its removal, without claiming an emitted Clear path. Clear's realization landing in Wave 2 reopens those dispositions and their evidence compatibility (`rule:phase7:wave0-rulings`) (`cond:wave0:baseline-freeze`).

The six external closures are report-consumer observations over immutable admitted submissions: absence, failed claimed outcome and another exact transaction, for both sponsor identities. Malformed reports and mismatched premises fail preparation separately. These comparisons add no native execution claim, alter no external evidence role, and authorize no capture, emission or acceptance change. The manifest's supplied-entry mapping moves missing-entry standing from 24 = 18 + 6 to zero, `compact_first_party_dispositions`' own comparisons move first-party standing from 2 discharged / 18 retained to 20 closed / 0 retained, and the ledger explicitly admits the 26 keys, while preserving 74 negatives, the 48 / 20 / 6 partition and 209 = 135 + 74 requirements.

**Table (Sponsored maturity archive standing)** · `tab:vectors:sponsored-maturity-archive`

| Archive | Admission | Fee policy and standing |
|---|---|---|
| `packages/vectors/fixtures/maturity-sponsored-run-of-record/` | Four immutable files; `MATURITY_SPONSORED_MANIFEST_SHA256` and `MATURITY_SPONSORED_RUN_ADDRESS` | Relay and mempool floors `0.00000100 BTC/kvB` reported by the node; block default `0.00000100 BTC/kvB` declared by the pinned source, no override. Replay: 69 operations, 35 fundings, 17 signings, 17 submissions, 12 accepted and five refused; zero-valued funding separately refused. Fees 128 and 141 units; exact routes and readback at (`rec:backlog:wave0-sponsored-observation`). |
| `packages/vectors/fixtures/maturity-native-run-of-record/` | Four immutable files; `MATURITY_NATIVE_MANIFEST_SHA256` and `MATURITY_NATIVE_RUN_ADDRESS` | Default relay and mempool floors `0.00000100 BTC/kvB` reported by the node; block floor `0.00000100 BTC/kvB` declared by the pinned source without override. Exact replay: 137 operations, 63 fundings, 27 signings, 47 submissions, 21 accepted and 26 refused; row bindings and phase durations at (`rec:backlog:wave0-native-observation`). |
| `packages/vectors/fixtures/maturity-observer-run-of-record/` | Four immutable files; `MATURITY_OBSERVER_MANIFEST_SHA256`, `MATURITY_OBSERVER_RUN_ADDRESS` and `MATURITY_OBSERVER_CEREMONY_SHA256` | Separate public-input observation of the exact variable sponsorless subject; six refusals followed by validated history and recovery reports, under node-default fee policy and direct-block inclusion. Process isolation, checkpoint provenance and retained non-claims at (`rec:backlog:wave0-independent-observation`). |
| `packages/vectors/fixtures/maturity-retained-run-of-record/` | Four immutable files; `MATURITY_RETAINED_MANIFEST_SHA256` and `MATURITY_RETAINED_RUN_ADDRESS` | Exact replay: nine operations, five fundings and four submissions. Both controls accept; the semantic subject refuses at the script path; the two-successor subject accepts by direct block with matched readback. Node-default policy has the same distinct relay and block provenance as the native archive. Whole-archive admission precedes one refusal answer and one final undecided-coordinate reason at (`rec:backlog:wave0-native-route-capture-driver`). |
| `packages/vectors/fixtures/maturity-confidential-run-of-record/` | Four immutable files; `MATURITY_CONFIDENTIAL_MANIFEST_SHA256`, `MATURITY_CONFIDENTIAL_RUN_ADDRESS` and `MATURITY_CONFIDENTIAL_CEREMONY_SHA256` | Fourteen exchanges: six funding operations, six signing operations and two submissions. Explicit control and confidential subject agree through relay and ordinary template selection with exact readback under default policy. Whole-archive admission binds one final unread-sponsor-value reason, with no accepted-transaction layer for the fault row (`rec:backlog:wave0-confidential-route-admission`). |
| `packages/vectors/fixtures/maturity-diagnostic-run-of-record/` | Four immutable files; `MATURITY_DIAGNOSTIC_MANIFEST_SHA256`, `MATURITY_DIAGNOSTIC_RUN_ADDRESS` and private `MATURITY_DIAGNOSTIC_CEREMONY_SHA256` | Exact replay: 48 exchanges, 17 fundings, zero signings and 31 submissions; seven accepted family controls, 23 script-path refusals and one consensus refusal. All 23 campaign rows bind 25 claims. The attribution ledger records 24 agreements, one difference and zero absences without changing any standing or census (`rec:backlog:wave0-diagnostic-route-admission`). |

It does not prove universal compiler correctness, cryptographic hardness,
target implementation correctness, or final deployment release.

## Exit gate · `gate:vectors:compact-ash`

The first bundle report exits when:

- all compact-ASH relation censuses agree;
- every relation has reachable positive and negative coverage;
- valid ABI transactions execute successfully;
- focused mutations reject at the intended evidence layer;
- accepted target projection matches the model;
- permissionless construction uses public data only;
- predicted and observed resources agree;
- reports are deterministic, identity-bound, and secret-free.

## Error vocabulary · `sec:vectors:errors`

See [`errors/vectors.md`](errors/vectors.md).

## Open questions · `sec:vectors:open`

- Where do shared evidence-envelope types live?
- Which exact target runner and RPC adapter are used?
- How are public model fixtures exposed without test-only internals?
- What low-level mutation API remains inaccessible to production callers?
- What coverage policy governs duplicated carriers and collateral mutations?
- How are external observer tools invoked and their independence declared?
- Which canonical binary vectors are committed versus generated for release?


## Keyed reader rule and report boundary · `rule:vectors:keyed-reading`

The conformance-library `keyed_evidence` layer consumes the byte reader's memberships and missing evidence. Its all-record, fixed immutable and changing rules have report-bound identities; annotation values remain SOURCE provenance with unassigned attribution, never record credit. Complete-handle and whole-supplied-history requests have distinct completeness scopes. All four parent facts and the per-key inclusion limit remain named assumptions. Finality matching has an explicitly hypothetical fixture type and an externally accepted deployment type, with no activation data selected here. Internal schema 1 now separates shared headers and family bodies; only complete outcomes contain sets, and reporting requests can retain incomplete and non-handle outcomes. Required finality accepts only the sealed deployment type, which has no public producer before activation; hypothetical controls never satisfy it. The shape, refusal order and recomputing validator are specified at (`rec:w1b15:reading-review`); all six family validators now consume a sealed byte/evidence context as recorded at (`rec:w1b15:families`). No report pass bit supplies independent recurrence or native execution evidence.

The family review at (`rec:w1b15:families-review`) distinguishes byte-model replay consistency from the harness's independent graph/literal comparison, and measured-term weight formulas from pinned `elements` weight evidence. Its sole reader amendment refuses nonempty member scriptSig before witness shape. Typed preparation and validation boundaries preserve owner errors, resources headers reflect body observations, and full archive rows remain in the sealed context. A complete model reading still relies on inclusion for native transaction validity; sequence/lock time and transaction-wide validity remain final-subject work for bite 1.6.

The bite-1.5 packet at (`rec:w1b15:gate`) inventories the conformance reader, internal six-family frame and typed boundaries, their source/dependency independence, conditional premises and exact-byte preservation. It is closed at (`rec:w1b15:closed`). Generation/validation share derivation helpers; replay and recurrence re-folding are consistency checks, with model/literal and Elements second sources described at (`rec:w1b15:families-review`). A complete member reading establishes local authenticated model evidence, not containing-transaction validity or native acceptance. Finality requires a deployment declaration unavailable from the public API pending activation; fixture declarations cannot satisfy Required. Bite 1.6 receives exact subjects, relation evidence and all remaining native/policy/transaction-validity debts.


The final-subject hand-off is discharged for its observed domain at (`rec:w1b16:packet`), including the exact-byte native transaction-validity evidence and retained deployment-premise limits.

## Supplied-key freeze inventory · `sec:vectors:keyed-freeze`

The conformance-only `keyed_final_manifest` surface is specified by (`rec:w1b16:freeze`): a mutable `FreezeManifest` offer, `FrozenItem`, `ComputedItem`, `SourceIdentity`, `Envelope`, `FreezeRefusal` and its exhaustive `FreezeBoundary`. `FreezeManifest::capture_encoding` owns the fallible canonical preimage and refuses paths outside ``[A-Za-z0-9/._-]`` as `FrozenInventory`; it does not replace validation. `validate_manifest` binds the reviewed source inventory and per-key template identity; `parse_key` checks textual width and hexadecimal form. The literal `predicted_leaf` helper and second-key phrase/digest are test evidence, not an exported production format. Twelve frozen categories contain 35 selected source-file identities; sixteen categories remain per-instance. These pins do not claim a transitive workspace or executable digest. No signing authority, native verdict or chain fact is minted.

The external `keyed_final` family constructs two SOURCE events and their MERGE through public entries under two supplied keys, compares literal grammar and first-party conformance arithmetic with production, and reproduces all snapshot coordinates in a fresh process using recorded ordinary witnesses. Freshness separates state, not shared reconstruction code. Literal membership expectations are compared with the unchanged byte reader. Edge data values, the three link scopes, wrong bindings and commitment refusals have controls at their owners. Public semantic keys require no secrecy. Generic sizes and schedules follow from fixed-width data dropped before both clauses; per-key cryptographic and external premises remain conditional. No production code, previous vector, archive or pin is regenerated.


### Re-freeze and provenance · `rule:vectors:keyed-refreeze`

A change to pinned source and its re-pin land together in one gate-passing landing, independently reviewed before import. The mandatory freeze-history entry names each path, previous and new digest, the source diff, whether it changes behaviour or is editorial, and which evidence reopens under (`rec:w1b16:adoption`). A changed template, target, policy or envelope reopens its affected evidence; a new key alone does not. The initial review expansion and literal digest table are at (`tab:w1b16:freeze-history`). A re-pin is never routine permission to change a reviewed function; no check forbids paired source/pin commits, and no red intermediate main commit is required.

The present provenance class is admitted at (`rec:identities:keyed-capture-freeze`); its admission prerequisite is satisfied for Wave-2 `relation.rs` and fixed-form S3 `semantic.rs` re-pins, subject to their other packet and landing dependencies. For an authorized re-pin, update the paired source and current digest in `packages/vectors/src/keyed_final_manifest/pins.rs`, retain that path in `RETAINED_CAPTURE` at its capture-time digest (the previous value of its first post-capture row, unchanged by later re-pins), append the previous/current row and its classification in `packages/vectors/tests/keyed_final/freeze_history.rs` together with the corresponding `ENCODING_DIGESTS` row of two literal per-key digests, and land the plan record of the complete diff, before/after controls and affected-evidence disposition together. Source identity and affected equivalence evidence reopen; unchanged exact submitted bytes and valid premises can retain native observations, but changed bytes or invalid premises reopen affected observations and are not cured by updating a pin. Current-source validation, history replay, retained mapping/census and sealed corpus comparisons must still pass. Inventory or rendering changes exceed this digest-only migration; preserve historical captures and require a separately admitted recipe/transition where applicable.

The source identities attest bytes compiled into vectors via relative-path inclusion in its build checkout. They do not inspect runtime source files or identify an executable, toolchain or resolved dependencies. This unpublished crate requires the workspace layout. A moved or copied checkout must rebuild vectors before a control counts; a copied old binary still holds the old bytes. `Cargo.lock` is untracked and the workspace `sha2` requirement is a version range used by both commitment and oracle. Step 2's executor provenance must retain the commit, resolved lock and toolchain of the binary actually run. Shared SHA-256 is explicitly outside the prediction's independence claim.

The source census is a literal module/path-declaration check with accepted and missing-child controls, not macro or alias expansion. Non-test child files of pinned Rust files are pinned. Generic program/instruction encoding (`packages/tapscript/src/program.rs`, `instruction.rs`) and opcode/push tables (`packages/target-elements/src/opcode.rs`, `push.rs`) are outside the identity and protected for this template by independent literal prediction. Reader, stack model and conformance arithmetic are instruments to be pinned at step 5, not implicitly frozen template code. The twelve categories now contain 35 pins; sixteen variable categories are tied to exhaustive packet/input/snapshot destructuring. The selection offer helper is pure and untraced; emission remains the checked owner boundary.

Historical replay selects operation shape rules from the recorded closed revision set, independently of the live handshake. Revision eight's archives keep their bytes, adapter identities and admissions; revision nine's explicit sponsor funding carries canonical parent readback. A session may not mix revisions. Current request-comparison identity is rendered separately from the old comparison archive. The scaling replay change and its sole source re-pin are recorded at (`tab:w1b16:bridge-refreeze`); no native evidence is reopened. The ordinary-edge signing-authority choice is resolved by (`rec:w1b16:bridge-authority`); its original stop remains a historical record.

### Attestation vocabulary re-freeze · `rec:vectors:burn-clear-refreeze`

The attestation `Relation` vocabulary in `packages/realization/src/relation.rs` adds SideRepresentation, DataOutputs, BurnBinding, ClearClamp, ClearAssignment, ClearFloor, ClearFlow, ClearBinding and ClearCanonicalDeltaPolicy. This is a behaviour change confined to attestation vocabulary; keyed relation types and the selected template are unchanged. Under (`rule:vectors:keyed-refreeze`), source identity reopens and native observations remain retained only if every existing keyed byte pin passes unchanged.

| Source | Previous / capture-time digest | Current digest | History and encoding controls |
|---|---|---|---|
| `packages/realization/src/relation.rs` | `REFREEZES[3].previous`; retained in `RETAINED_CAPTURE` | `REFREEZES[3].current` | `packages/vectors/tests/keyed_final/freeze_history.rs`: previous/current continuity, retained mapping, and `ENCODING_DIGESTS[4]` for both keys |

The source diff adds only the nine attestation enum bodies and their fields. Before/after controls are the unchanged keyed source declarations and byte predictions, retained capture reconstruction and sealed corpus comparisons. `check_encoding_history` compares the owned encoding and capture producer against both new literal digests, then reconstructs every earlier row without changing its expectations. The retained source digest remains the previous value of this path's first post-capture row. The new source pin does not replace any native byte pin or capture-time identity, and the full import gate remains the landing condition.

### Fixed-form profile re-freeze · `rec:vectors:keyed-fixed-profile-refreeze`

S3 adds only `KeyedFixedForm = 3` after `KeyedProvenance = 2` in packages/architecture/src/semantic.rs. Every existing discriminant and encoder arm is unchanged; no legacy object or operation union is extended. This is additive vocabulary and supported-entry behaviour, not editorial. The paired source history is `REFREEZES[4]` and the two-key owned-encoder snapshot is `ENCODING_DIGESTS[5]` in packages/vectors/tests/keyed_final/freeze_history.rs. The capture-time previous digest is retained in `RETAINED_CAPTURE` in packages/vectors/src/keyed_final_manifest/pins.rs.

The complete frozen-source diff is that one enum member. Before/after controls are the unchanged independent 165-byte schema-2 grammar and literal digest, all legacy namespace encodings, the frozen attestation canonical/publication fixture, current-source validation, history continuity, retained capture reconstruction and sealed corpus comparisons. New controls independently pin schema-3 grammar, parameter census and cross-schema refusals, exact four/six declaration rows, positive-value admission and member pair/form/position. Source identity and equivalence reopen; the legacy template, submitted bytes, target, policy and envelope do not change. Native observations remain evidence only of their retained exact bytes and premises, conditional on unchanged legacy replay/negative checks. No fixed-form freeze is read, re-pinned or admitted; that remains S5.

The encoding pair is obtained from the owned capture encoder through the freeze-history assertion in the node gate, then retained literally and replayed against every earlier snapshot. Independent import review must rebuild it. The lane gate receipt, rather than this record alone, establishes execution of the controls; full import verification remains required.

### Dependency-line re-freeze · `tab:vectors:dependency-refreeze`

The root manifest is pinned in the Dependencies category, so moving the workspace's dependency release lines under (`rule:backlog:dependency-currency`) is a change to pinned source and lands with its re-pin under (`rule:vectors:keyed-refreeze`). The literal row is `REFREEZES[2]` in `packages/vectors/tests/keyed_final/freeze_history.rs`.

| Pinned path | Previous SHA-256 | New SHA-256 | Source diff | Classification and reopened evidence |
|---|---|---|---|---|
| `Cargo.toml` | `REFREEZES[2].previous` | `REFREEZES[2].current` | `lzma-rust2` 0.24.0 to 0.25.0, `num-bigint` 0.4 to 0.5, `toml` 0.8 to 1, `sha2` 0.10 to 0.11, `nix` and `textwrap` from `"0"` to their `0.32` and `0.16` lines, and class comments beside `bitcoin_hashes`, `elements`, `secp256k1-zkp` and `petgraph` | Build-graph change to a frozen condition of the bundle identity in (`rec:w1b16:adoption`), its dependency versions, which reopens the evidence that condition affects. No template, target, policy or envelope source changes, and the slow-class version requirements and `packages/transaction/Cargo.toml` are unchanged. Reopened: source identity, and the byte equivalence of the template's dependent values on the new graph. Not reopened: native observations, for the reason below. |

The native observations judged exact submitted bytes, which the sealed capture manifests bind by digest, and each capture's provenance retains the resolved lock of the binary actually run. The new graph is equivalent for that evidence only if every first-party byte pin it computes is unchanged: the freeze family's literal leaf and independent commitment predictions under both keys and their fresh replays, the retained corpus's decoded, manifest and subject digests with its history readings and fresh reproduction of commitments and final transaction bytes, and the other package byte pins. This re-freeze stands on a gate in which they pass unchanged; a changed byte pin is a finding about the upgrade, not a further re-pin. Commitment and oracle still share one `sha2` range, which stays outside the prediction's independence claim.

The retained final native corpus stays bound to the freeze of its capture. `retained_capture_freeze_digest` validates the current inventory, so included-source drift still refuses, then digests `FreezeManifest::retained_capture_offer` with the rendering of `capture_freeze_digest`: the current pins, with each path in `RETAINED_CAPTURE_PINS` at its capture-time digest. The history check requires that table to equal, for each path re-frozen after that capture, the previous digest of its first later row, so the capture-time value is derived from the history given the asserted capture marker, which only the corpus's sealed freeze checks; a duplicated path and the marker one row either side are refused beside the table; it also refuses the newest row with its previous or its current digest one coordinate away. The source census reads the same table, `RETAINED_CAPTURE` in `pins.rs`, and requires each entry to name a pinned source whose capture-time digest is not its current pin. The corpus compares its sessions and rows with the capture-time digest and refuses the current freeze for the same session at the `Freeze` coordinate. A new capture carries the current freeze. The construction covers digest re-pins only: a path added to or removed from the inventory after the capture makes the retained digest differ from the sealed one, and the corpus refuses until the construction carries that change. This preserves the existing rendering, role and consumers; its migration behaviour is part of the admission at (`rec:identities:keyed-capture-freeze`).

### Final public-construction bridge · `sec:vectors:keyed-final-bridge`

`keyed_final_native` is a public conformance boundary, not a production planner. `freeze_final_subject` consumes a sealed request and structural result; `complete_final_subject` binds admission to frozen bytes and capability snapshots. `FinalSubject` exposes read-only complete bytes, digest, census and member receipts, policy provenance and its expectation outside the executor request. The optional spent-output assertion uses the bound decoded asset/value/program. The production constructor remains the owner of signature verification and policy admission. No receipt is detached from its parent census. Private-field doctests forbid constructing either sealed bridge value directly.

Capture provenance checks agreement of retained observations; it cannot establish that an operator ran the stated executable. The capture runner must retain the actual commit, resolved lock and toolchain of its binary, handshake, image and node identities, and the per-key checked manifest digest. Missing measurements stay absent. Test-local diagnostic rendering is no maintained interchange contract. Mock histories and transport replay establish the offline path only; capture commands, pilots and native validation belong to the next steps, as recorded at (`rec:w1b16:bridge`). Existing archives retain revision-bound replay and bytes.

The revision-nine bridge requires `bind_capture_journal` before archived operation evidence is admitted. It binds the recorded handshake and packet revision to every request/response and authenticates accepted sponsor parents. A `BoundCaptureJournal` is only that local binding, not complete archive admission or native validity. Historical revision-eight archive-specific admission remains unchanged; generic shape replay does not authorize revision-nine evidence by itself. The bridge also pins a parent accepted by the funding decoder but refused by construction for peg-in form. Review dispositions are at (`rec:w1b16:protocol-review`).
The final capture provenance carrier uses `Option` for each measured coordinate and rejects missing measurements on either side, including equal missing values. The capture binary, adapter interpreter and Elements daemon have distinct identities. Runner measurements (source commit, resolved lock, compiler, binary/script/image digests and host) are distinguished from retained handshake/self-reported fields in the field documentation. Agreement is not authentication; the revision-nine adapter additionally binds to its rendered comparison digest.

The bridge review at (`rec:w1b16:bridge-review`) pins reachable sealed-value mispairings, records the three defensive guards and defines the orchestrator-only capture dispatch. A native pilot must exercise actual framework decoding; no mock result discharges that obligation.

### Final capture support · `sec:vectors:keyed-final-capture`

The four clause capture targets and three reviewed shell wrappers are specified at (`rec:w1b16:clauses-support`); two additional envelope targets and their per-case fresh sessions are specified at (`rec:w1b16:envelope-support`). The conformance-only `CaptureManifest`, `CaptureRow`, `CaptureTarget`, `CaptureSchema`, `CaptureBoundary` and exhaustive `CaptureRefusal` describe untrusted per-run evidence. `bind_capture_manifest` binds provenance and recorded-revision journal shape before checking the complete schema/key census, freeze identity, exact bytes, expected boundary and observation joins. A full retained-capture admission additionally replays the public construction against the recorded responses and compares its requests and manifest rows. This reader remains test-local; its JSON is no stable interchange contract. A matching digest alone is never proof of a sealed origin or of native execution.

Preparation copies a named clean source tree without shared Git-object dependencies, builds in the retained image with a warm target, and records the actual resolved lock, toolchain and binary. Runtime checks bind those measurements to the mounted files; handshake observations must agree before a capture counts. The completed subject-manifest digest is recorded after all rows exist, while each row and submitted byte file is flushed before its own submission. Mock and native share subject schemas rather than transaction digests. Existing mock pins, archives, admission readers, production entries and freeze pins remain unchanged. Unit resource snapshots name their scope; unavailable final-subject dimensions remain absent. No native acceptance or Wave-1 closure follows from this offline support landing.


The envelope support retains SOURCE preparation separately from MERGE execution and distinguishes declared constructor policy, reported node policy and actual target outcomes. The maximum is a policy envelope, not a script law. The large 1565-member subject receives a typed preflight refusal before its separately labelled structural serialization; native acceptance is not inherited from construction. The full retained reader requires exact per-run manifests, recorded signatures, journal binding and independently decoded node weights. Segment-only offline framing is not exported as a full admission: every prefix has its own exact replay, and small controls also traverse the unsplit reader. A new native landing requires its own pilot and target-first wall review; previous-commit pilot equality is never waived.


## Fixed-form keyed track · `sec:vectors:fixed-form`

The realization owner is (`[DOC-sec:keyed-fixed:scope]`). S5 supplies the typed validated 46-source freeze, owned binary encoder and independent controls; its separate provenance admission and before-first-capture S8 migration are (`rec:identities:keyed-fixed-capture-stop`). The initial structural envelope leaves maximum members explicitly unadmitted for S6/S8. Source and inventory changes retain reviewed re-freeze obligations; legacy admission and retained artifacts remain unchanged. Code admission supplies no native observation.

The maintained clarification of (`sec:w1b16:packet-domain`) is (`[DOC-sec:keyed-fixed:environment]`) and the Wave-1 packet in packages/vectors/README.md. Observations used one Elements Core v28.99.0 development build on fresh isolated elementsregtest nodes with default-challenge harness blocks and no block signing, not release, Liquid or signed-block production. The sealed per-session provenance retains exact source, binary, toolchain and image bindings. Only explicit policy-asset successors were observed; legacy fully-confidential public continuation remains unresolved. These limits preserve the archived record and supply no fixed-form native evidence. S8 must retain effective policy and its own environment; S9 still owns the required-tip description reconciliation.

Under (`rule:phase7:fixed-form-rulings`), the joined gate remains (`rec:backlog:fixed-form-closure`). Vectors owns the ADR-021 adjudication of both capture-freeze classes under (`rule:backlog:identity-admission`) and (`[ADR021-req:identity:admission-record]`), with S0 treatment at (`rec:backlog:fixed-form-decisions`). The legacy selected-source FreezeManifest and envelope are admitted as provenance at (`rec:identities:keyed-capture-freeze`): the explicit owned encoding warrants the existing recipe while preserving the sealed capture bytes and bindings and reproducing the reviewed history reconstructions. Accepted ruling 210 at (`rule:phase7:fixed-form-rulings`) discharges this admission by a vectors code step ahead of the documents-only S2 packet. The present-class admission prerequisite is satisfied; Wave 2’s relation.rs and S3’s semantic.rs re-pins may proceed under (`rule:vectors:keyed-refreeze`) when their other packet and landing dependencies hold. Each retains its own history/override and equivalence obligations. Preserve `keyed-capture-freeze-v1`; projection/encoding/domain changes require a new recipe. S2 owns the fixed-form realization recipe and pre-admission walk; S5 supplies the distinct capture-freeze subject, encoder, inventory and property warrants, and S8 activates its declared consumers before capture under ruling 211. The legacy admission does not select the fixed-form recipe, and an unwarranted property remains unadmitted. The Wave-2 packet correction and S3 landing dependency remain at (`rec:backlog:fixed-form-decisions`).

S3’s semantic.rs source/pin/history landing is additive with legacy behavior unchanged, not editorial. Its history entry supplies previous/current digest, complete diff, before/after legacy manifest/identity/attestation and byte/replay/negative controls, and affected-evidence disposition. Retained native captures stay bound to the old freeze, with relation.rs and semantic.rs each added to RETAINED_CAPTURE by its own re-pin landing at its capture-time digest; current source identity changes. Unchanged byte controls justify retaining exact-transaction native observations; a changed byte or invalid premise reopens affected evidence and cannot be answered by updating a pin. The 35-path legacy census is not enlarged with fixed-form children; S5 creates a separate inventory. No new digest value or regression pass is asserted by S0.

S5's shared-model source update is editorial: the new public model re-exports the existing ProgramFact, exposing rows() to the public error-documentation lint. Its complete source diff adds three documentation lines immediately before that method: its payload/version order, an Errors heading, and the existing unavailable/invalid-witness refusal domain. No signature or implementation changes. This helper was pinned by no freeze before S5; the first fixed-form freeze includes the documented source. No previous landed fixed-form freeze or affected capture exists, so this initial landing creates no re-freeze history row under (`rule:vectors:keyed-refreeze`). The helper remains outside the legacy 35-source freeze.

The first per-triple snapshot uses the full ordered grammar and computed-code table at (`rec:identities:keyed-fixed-capture-stop`), implemented by packages/vectors/src/keyed_fixed_manifest/encoding.rs. Its test triple has a 4249-byte encoding; its exact identity token is registered in lint/hash-citation-families.tsv. The independent encoding and literal identity control is supplemented by a synthetic stale-source subject, not an intermediate lane state presented as history. Codes 9–15 name recomputed results; their full values are bound through the pinned producers, not through code-list bytes. The descriptor and envelope separately encode the values named by codes 1–8 and the admitted maximum.

S8's pre-capture re-freeze at (`rec:identities:keyed-fixed-s8-migration`) retains that S5 snapshot and extends the current whole-file inventory to 98 paths, 15 computed codes and `Some(1299)` members for the ruled constructor envelope. The 55 literal history rows are 52 source additions and paired previous/current changes for packages/vectors/Cargo.toml, manifest.rs and encoding.rs; the Cargo dependency move is a build-input change while the other two vectors sources change inventory/envelope admission and computed-code encoding. The added paths cover S6 construction and shared byte/signing sources, S7 reading/model/evidence, the transaction manifest and amount-form owner, conformance proof/protocol/executor provenance and direct module dependencies, the unchanged adapter, the three sibling wrappers, and the fixed-form capture/native/test sources. The first-campaign test-triple encoding is 8853 bytes and its independently reconstructed capture-time identity is `6591fbb37cfc1fbcf1ffe14a5212a14bc919416f937dc51d314818070028c063`; the previous 4249-byte value remains historical and cross-edition values refuse. The diff is behavioral for current inventory/envelope admission and additive for computed-code encoding; no S5 fixed-form native evidence existed to relabel. All first-edition capture bytes, pins and scripts stay unchanged. Future source edits stale their pins and require paired history, independent byte controls and affected-evidence disposition before capture.

The sealed S8a pilot used the preceding 98-source, 8853-byte identity `02abd95456364268db586511be03b32cc864f833753cda1d4a643921430d3b12`. The post-pilot correction changes three already pinned sources: the capture owner joins negative MERGEs to their actual MERGE control, requires same-preparation provenance and relay-and-template acceptance, and checks floor labels against the freeze; its journal test adds those refusal controls; its native child omits a zero-valued fee output from the zero-fee probe. `POST_PILOT` in `packages/vectors/tests/keyed_fixed_freeze_history.rs` pairs each pilot digest with its new digest and reconstructs both literal encodings. Before the correction, the pilot sealed and its node answers remain evidence of the subjects it actually submitted; after the correction, the stricter journal admission and changed zero-fee bytes require a new current capture for campaign claims. The pilot archive retains its capture-time identity and is never relabelled with the current one. No first-edition pin or source changes, and the 55 S5-to-current rows still record the full inventory transition.

The pilot-retention correction changes two more pinned whole files: `packages/vectors/tests/keyed_fixed_capture/native.rs` retains a structured runtime-provenance record and typed handshake, emits JSON timing, and hashes the four executable/source files once per family; `scripts/keyed-fixed-capture/capture-unit.sh` copies the node logs into the seal and marks cgroup inclusion when the retained container membership proves it. The wrapper contract test exercises node-log copies and sealing but is not a capture-consumed source. `POST_S8A2` pairs both prior digests with their current digests and reconstructs the intermediate 98-source, 8853-byte identity `24660fb26a6da460107ddd3b795746fb99a353e8ee9cdaf585dcd6fbca1030a5`; `POST_PILOT` then reconstructs the sealed pilot identity. The current identity above changes only because the two consumed source bytes change; the inventory, code list, admitted maximum and encoding length are unchanged. Before controls are the sound sealed pilot, the previous identity reconstruction and the wrapper's lifecycle test; after controls are the independent current pin/encoding rebuild, historical reconstruction, node gate and wrapper log-seal control. The pilot remains bound to its original bytes, while S8a2 has no native campaign archive and gains none by re-pinning. The new current capture is required for campaign claims; no old observation is relabelled.

Ruling 219 narrows the first campaign timing record's claim without changing a capture source: `phase-timing.jsonl` holds the calling thread's transport and admission microsecond walls and family and executor events. The planner's worker thread emitted per-exchange timing and `capture freeze identity bound` events outside that thread-local subscriber, so the first campaign derives per-exchange time from sub-second exchange file times in pax archives. Ruling 220 moved subscriber propagation, its wrapper assertion and watcher fixes to the campaign repair. The second run's unrestricted process-wide DEBUG subscriber flooded the clauses and scaling timing files with construction events, and scaling hit the executor's 600-second whole-run watchdog. The current re-freeze limits the subscriber to `vectors::native_test`, `vectors::keyed_fixed_capture` and `target_elements_conformance::executor` at DEBUG or higher, checks timing size and line count against exchanges, and gives scaling a 2040-second inner bound derived from four times the first run's measured 509 seconds, rounded up to a minute. The 3600-second scaling unit guard stays fixed. `POST_TIMING` pairs the prior and current digests of the pinned native child and `run-one.sh`; the 98-source, 8853-byte encoding now yields `6c49332fab715b72e4d85d94935df6f4c52034d027110e203c5eea83f7a97092`. This changes the harness's timing record and scaling stop point, not covenant behaviour, submitted bytes or order. Before controls are the second run's sealed clauses and stopped scaling packets with their measured counts and the reconstructed `REPAIR_IDENTITY`; after controls are the valid-JSON wrapper refusals at both timing ceilings, the independent pin and encoding rebuild, the 67-row history test and the green node gate. Earlier sealed packets keep their capture-time scope and identity; the next run repeats all four families under one new identity.

The campaign repair changed five pinned whole files: the capture census and adapter-layer admission owner, its offline control, its native child, and the two capture wrappers. `POST_CAMPAIGN` pairs each prior and then-current digest, bringing the then-literal S5-to-repair history to 65 rows. The ordered inventory remained 98 sources, the computed list 15 codes, and the independent encoding 8853 bytes for the record triple. The repair's identity is now the historical `REPAIR_IDENTITY` asserted by `fixed_capture_refreeze_history`; reversing its five rows reconstructs the first campaign identity `6591fbb37cfc1fbcf1ffe14a5212a14bc919416f937dc51d314818070028c063`, then the earlier S8a2 and pilot rows reconstruct their prior identities. The first campaign's policy and histories seals remain bound to their capture-time bytes; its clauses and scaling packets are stopped evidence and receive no credit under the repair identity. Before controls were the retained stop packets' exact request/response and census bytes, the earlier historical reconstructions and the wrapper lifecycle test. After controls were the repaired census and layer/reason joins, the full node-free row projection, the independently encoded repair identity, the history test and the node gate. The adapter and first-edition sources remain unchanged; the changed prediction labels the pinned adapter's mixed-input response while the node's `Invalid Schnorr signature` reason and the constructed key-path witness preserve the subject's claimed refusal.

The S8 dependency census follows the named capture inputs through their module declarations and direct path consumers. In addition to the S8 construction, reader, proof, protocol and wrapper paths already named, the capture consumes `packages/transaction/Cargo.toml` and `packages/target-elements/src/transaction_form.rs` (the amount bound checked by every constructor subject), plus conformance `constructor/mod.rs`, its non-test children `canonical.rs`, `metadata.rs`, `metadata_leaf.rs` and `totality.rs`, `confidential_oracles.rs`, `error.rs`, `fixture.rs`, `prototype.rs`, `prototype_validate.rs`, `sponsor_parent.rs`, `test_material.rs` and `validate.rs`; these receive whole-file pins and literal addition rows before capture. The new native integration child `packages/vectors/tests/keyed_fixed_capture/native.rs` receives its own pin. The manifest's `pins.rs` is represented by the encoded ordered inventory rather than hashed into itself. Generic transaction `abi.rs`, `error.rs`, `request.rs`, `synthetic.rs`, `view.rs` and `trace_support.rs` are not independent subject selectors: the frozen construction output and descriptor bind their values, while the runtime binary and resolved lock are recorded separately. This census claims the listed direct consumed sources, not the entire transitive workspace; a later discovered consumed selector requires a packet correction before capture. The corpus consumer source remains separately recorded outside the inventory so its own identity does not depend on itself. Historical corpus reconstruction checks source bytes against a retained mapping and compares the resulting recipe digest with a separately reviewed capture-time identity; it does not compare that mapping's code list, descriptor or envelope with a later current offer.

S7 owns the new vectors-only model/reader/evidence siblings, complete dependency closure, authenticated member fields and SOURCE-only attribution; it introduces no partial-chain API or external report schema. S8 owns new capture/admission/corpus subjects and artifact census, keeping legacy copy evidence separately named and replayed against the old leaf. The existing historical extraction tool stays unchanged; no duplicate extraction mechanism is introduced. New progress/timing uses tracing, not expansion of the printed history exception. S2’s maintained legacy environment clarification cites (`sec:w1b16:packet-domain`) and packages/vectors/README.md; the archived draft remains untouched. Exact files, shared consumers and literal registry receivers are (`tab:backlog:fixed-form-paths`), (`tab:backlog:fixed-form-sweep`).

The S7 candidate places byte/address indexing, qualification, recurrence and typed refusals in `packages/vectors/src/keyed_fixed_history_reader.rs` and its five children. Its symbolic recurrence in `packages/vectors/src/keyed_fixed_history_model.rs` does not call the reader, and `packages/vectors/src/keyed_fixed_evidence.rs` keeps acceptance predicates and external premises separate. The seven selected checks are `packages/vectors/tests/keyed_fixed_history.rs` under (`tab:backlog:fixed-form-s7-tests`). The retained first-edition reader and both freeze inventories are unchanged.

### S8b historical decoder re-freeze · `rec:vectors:keyed-fixed-s8b-refreeze`

The two paired pinned-source rows in `POST_S8B` are `packages/vectors/src/keyed_fixed_manifest.rs`: `604553f78b941ce48d3d6de8b5c8d6715188b02b08fa08df897d406b99c37ab8` → `df9e03a79ffbf8e8abda6289627e2a90a10be0dacda26f80c21f6de0270b801a`, and `packages/vectors/src/keyed_fixed_manifest/encoding.rs`: `b2de0ec0df98d539aee105fc638dbb5a65aae02d6bf300300d246e3b26a7f8ce` → `ac2a88e7231f7ca0a605c36df05e949720a376eb12e9c71a4e8495991beb2c02`. The complete diff adds an inverse bounded grammar and a reviewed historical complete-freeze route; it changes neither current producer nor encoding grammar, source membership, computed codes, envelope, descriptor recipe or capture/native source. The two pre-change source byte snapshots remain in the source fixture. The new record-triple identity is `df1ba2a6697b772471a34dcb7ee51637adb2e306edb2e43a6f656e94d55876f9`; reversing the two rows reconstructs the sealed campaign identity `6c49332fab715b72e4d85d94935df6f4c52034d027110e203c5eea83f7a97092`, then all earlier identities by their retained rows. Current validation refuses the historical inventory while the authenticated historical route supplies `ValidatedFreeze` to the unchanged journal binder. Source identity and consumer admission reopen under (`rule:vectors:keyed-refreeze`); exact historical bytes and premises retain the native observations conditionally, with no new native conclusion.

### S8c sponsor and capture re-freeze · `rec:vectors:keyed-fixed-s8c-refreeze`

Under (`rule:vectors:keyed-refreeze`), `POST_S8C` pairs the initial nine whole-file source transitions. Their previous bytes are retained in the indexed source snapshot catalog; the older campaign source versions and four identity anchors remain separate. Inventory stays at 98 sources, the independent grammar at 8,853 bytes, 15 computed codes and the admitted maximum at 1,299. The pre-review record-triple identity was `2b6172a80e767470f5a8f174c8497b227baea6d508e58b03849771d7445f809c`.

| Pinned source path | Previous SHA-256 | Current SHA-256 | Diff class and reopened evidence |
|---|---|---|---|
| `packages/transaction/src/keyed_fixed_sponsor.rs` | `9e1d8099e79ceebd38acc761c569750179bc04a5c87beaedee5bfffda65fa79c` | `7fff69f695f6cde48276f159c57dd0c8c141f984b465df6800e4419c5e1b8736` | Behavior: hybrid output form; sponsor equivalence and signatures. |
| `packages/vectors/src/keyed_fixed_capture.rs` | `05a989e20afd2e2ef71bc360cda162e0b240d260440bed94e43b1b0842798923` | `f2f41ba3367950ea16e7477a829a27fed6c0ecff81abab0c947b71318e02057c` | Behavior: family census and live identity; capture admission. |
| `packages/vectors/src/keyed_fixed_manifest.rs` | `df9e03a79ffbf8e8abda6289627e2a90a10be0dacda26f80c21f6de0270b801a` | `28369c40d7310a03a8aac4986b97d8070de694637747730dde8e8b2c75d7a508` | Behavior: epoch resolution; source identity. |
| `packages/vectors/src/keyed_fixed_manifest/encoding.rs` | `ac2a88e7231f7ca0a605c36df05e949720a376eb12e9c71a4e8495991beb2c02` | `4af4f248a690346804564639931589d0c31c090069cf6aa915d46d7c559d2e88` | Behavior: trusted source count; historical grammar admission. |
| `packages/vectors/tests/keyed_fixed_capture.rs` | `dfa86b3852443c7ad9e3865ebd109a1c0ff8499365b80f1a5ebd61dbec1b6374` | `92abb4f41880b82f71a3cca77f74d4f3bd931cb29cb2f8c360df1c3efafc79a9` | Test producer: proof and packet controls; pre-capture evidence. |
| `packages/vectors/tests/keyed_fixed_capture/native.rs` | `02faf1def0dd6382dffcd65819714731144db87a170dec0066d90e7c0a8a4590` | `0c0f2376c7add055670f041d1e1f1fae6cbfd77e3f7fcdf252d2793a93f1f0b3` | Capture producer: eleven subjects and live joins; native premise. |
| `scripts/keyed-fixed-capture/capture-prepare.sh` | `e0c0f86d1fce29ae7f37b0d3802147122c34835a43d164c620d3d37d0ebd66de` | `bc9e831825d771221824743ed86b9497e44270c2d794bd80f0b67f2839afd782` | Driver behavior: selected family preparation; provenance. |
| `scripts/keyed-fixed-capture/capture-unit.sh` | `fd79053c54ce1f3055ee45a55be02396e31af4a39e6eaf82657b553ffcc8f314` | `17fd2f382b40f085a67d48da7a9f20314db72dec35aa70f66fe0be78a2750c17` | Driver behavior: one-unit seal and waiter; lifecycle. |
| `scripts/keyed-fixed-capture/run-one.sh` | `3926322dbecbd66d4f090927bde81e7582744bae6f2814bc5196a6d85ad1f285` | `2eeffba0e0beb34e9e2a1a244662361993d3d21fb20363512e01f34d696ef69e` | Capture dispatch: exact new family entries; native premise. |

These are behavior-changing or test/capture-behavior transitions, not editorial substitutions. Reversing the review transition and then these nine rows yields S8b's `df1ba2a6697b772471a34dcb7ee51637adb2e306edb2e43a6f656e94d55876f9` and the earlier chain, while current validation refuses every historical inventory. The retained 220 complete S8b rows and their supporting payment signatures replay from unchanged submitted and parent bytes; the 4,000 archive-addressed support rows are not re-admitted. S8c's new identities are pinned separately by triple; no second-capture observation is claimed before the reviewed unit and offline joins pass (`rec:phase7:fixed-form-s8c-pre-capture`).

The pre-review S8c candidate anchors were K42/policy asset at value 60 `ce559f68c9cf7da5e600584a5e670f6ebaf012b3853da2cf07412bf05f76a3cc`, value 17 `34fda99767f0b5fe7a427a65f66cbd6972cd861b0cedd2e30411ee426eead5b3` and value 18 `b7d7f5be0706275968099d4b29aadde3fc4a3e8b6bebd80cd0fc8eace3a74400`. The pre-review source fixture was 42,872 compressed bytes with eleven indexed historical versions; its SHA-256 was `6cbd7adc9eb035c8df4e17ff54f9c04eb6acaa31debed2156f0860e74886c34d`, and the census SHA-256 was `f32699ff5c994e8a8388dc6a1c3640087dae816b73a2cdc7f3fd7f0409c4cf1e`. Campaign3 family fixture payloads and original archive digests remain unchanged.

The reviewed `POST_S8C_REVIEW` pairs `packages/vectors/src/keyed_fixed_manifest.rs` `28369c40d7310a03a8aac4986b97d8070de694637747730dde8e8b2c75d7a508` → `f9686eb4392a941ac06cc93d16674738355e4b5aeac08a26bc465900421e839b` for the immutable 98-entry Sponsor map and `packages/vectors/tests/keyed_fixed_capture/native.rs` `0c0f2376c7add055670f041d1e1f1fae6cbfd77e3f7fcdf252d2793a93f1f0b3` → `bb3ada1c5b2c34c03660447ff46fc67f3bad0dcf53b4f21cda053e10368c7b83` for the local mocked producer and funding-output index. Both previous whole files are retained in the checked shared source fixture. The current record-triple identity is `d8ca459f01ee509f7fdabdd5c92c880bd7bba618fe4b10b90fb853ea09b45380`; the K42/policy-asset anchors at values 60, 17 and 18 are `17a7b1da3385f79d2ec5a3587e8f35c25bba87cc760aae2c35d35e8e40c94480`, `3353d21e195b40379ec6d4923f1393e8cee785ad25587c01987da07d7775f331` and `5980c072960539d7b0c28dbab308987548fd51fc9e9514b1265aed603990a9e1`. The source fixture now has thirteen indexed versions, 46,162 compressed bytes and SHA-256 `f41879a59d53d653611c96ed19c5a201906701967b3d2baf0d7de271dd12a2b5`; the checked census SHA-256 is `fe2345f624b912e4b50230d043f365752e7a33db45b2c4073e2fe37ddb5c51d6`. The freeze-history test checks the literal Sponsor map against today's pins, reverses this review pair to the previous S8c identity, then reverses the nine earlier pairs and every older chain. The epoch test reconstructs the three current Sponsor anchors and four retained campaign3 encodings, and the mocked eleven-row packet checks the producer against the offline joins without claiming a node observation.

The nonce disposition (`sec:s8c:nonce-disposition`) adds the behavioral `POST_S8C_NONCE` re-freeze. Each previous whole file is retained in the checked shared source history; none of the failed run's submitted rows becomes successful evidence.

| Source path | Previous SHA-256 | Corrected SHA-256 | Reopened evidence |
|---|---|---|---|
| `packages/vectors/src/keyed_fixed_capture.rs` | `f2f41ba3367950ea16e7477a829a27fed6c0ecff81abab0c947b71318e02057c` | `bdf1067723ab490b4da0e9398376495f410d662b3982dae75a027d0013beceb2` | Six-row Sponsor census and role/order admission. |
| `packages/vectors/src/keyed_fixed_manifest.rs` | `f9686eb4392a941ac06cc93d16674738355e4b5aeac08a26bc465900421e839b` | `6ce6d0c3d8fae1db6f2096c9bfb082c3100a928e0b36baee955a12c44422d34b` | Separate corrected epoch resolution and old-epoch reconstruction. |
| `packages/vectors/tests/keyed_fixed_capture.rs` | `92abb4f41880b82f71a3cca77f74d4f3bd931cb29cb2f8c360df1c3efafc79a9` | `1f90cafbaa85b2a42b8df0de9cfc6ee596b07da723e0f4c0a1317f8e40bf5044` | Four-form signature domain and packet controls. |
| `packages/vectors/tests/keyed_fixed_capture/native.rs` | `bb3ada1c5b2c34c03660447ff46fc67f3bad0dcf53b4f21cda053e10368c7b83` | `45dd11fcf67f6b04cc22d55ef1451ca4b3170213f5344105d4f2097c5bbc762e` | Response/timing retention before comparison and reduced producer order. |

The failed Sponsor epoch keeps its K42 anchors at values 60, 17 and 18 above. The corrected epoch uses the same 98-source, 8,853-byte grammar and one selected K42/value-60 anchor, `3da39a6415b17e78adb2faa1a112c0eb6dab71598d815407d6885964a2ed3353`; its record-triple identity is `d73282219b633ff8f2b30380e2274761fcfa3b3b6c51d4c5ad77afea14a7c5a5`. The shared source fixture now contains seventeen indexed versions, 49,811 compressed bytes and SHA-256 `7c12b59b69f776625fe0f01d209fb70a619720028987d95cae62963976d11d6b`; its checked census SHA-256 is `d2612b64cec3fb5fa7eed10dcaf43ff128615c741fc66751d064d3bb25f6384a`. Reversing the four rows reconstructs the failed epoch without admitting its family; reversing the older rows still reconstructs campaign3 and every earlier inventory, each of which current validation refuses. The 220 retained S8b records and supporting payment signatures replay from unchanged bytes. The failed run archive is a diagnostic address only: `refs/handoff/w20c1/s8c-capture-failed-1608`, 84,244 bytes, SHA-256 `4c95cc108a359f4b16c964e95b82ca67f463e49bc96b889da273844dfe5368f5`. The successful nine-row packet and offline admission remain pending.

The implementation review's census finding adds `POST_S8C_CENSUS`, one behavioral paired row: `packages/vectors/tests/keyed_fixed_capture/native.rs` previous SHA-256 `45dd11fcf67f6b04cc22d55ef1451ca4b3170213f5344105d4f2097c5bbc762e` → current `404ec7fd4ec4e01f6a2e82cd2e4a6d025324a6e3f4f90bf3e6b05ca4d5fb339b`. The previous whole file is the eighteenth checked source-history version, and the ordered history now has 85 rows. This changes the producer's census writer and checker, so its previous corrected record-triple identity `d73282219b633ff8f2b30380e2274761fcfa3b3b6c51d4c5ad77afea14a7c5a5` moves to `b882bef6bc70ca3d881e1b028e5f4c954a19bcd80e8200862af447c1d182f894`; the selected K42/value-60 anchor moves from `3da39a6415b17e78adb2faa1a112c0eb6dab71598d815407d6885964a2ed3353` to `bdeb5788ff13893ddb100ec5c24a19e234adbb594678a43442ec5d349c563c85`. The failed Sponsor epoch and every earlier anchor stay fixed. The shared source fixture is now 50,338 compressed bytes, SHA-256 `686fa6ba70238135db6d802ebcb221bc6d31b5ce9f06929fc2e20d3ad809f5d1`; its census SHA-256 is `176697aace3fa413c4ede2bb08d0e01899d4c301a5ba40811adac94460223002`. The changed source is resolved by exact path and digest in history, current validation refuses its previous inventory, and no native observation has used either corrected anchor.
