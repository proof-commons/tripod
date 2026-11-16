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

The realization owner is (`[DOC-sec:keyed-fixed:scope]`), including the exact transcript/leaf/commitment, assumptions, envelope, floor and source-qualified environment. The fixed-form identity walk is pre-admission at (`rec:identities:keyed-fixed-capture-stop`): no typed validated fixed-form inventory/encoding or present consumer yet warrants admission. Ruling 211 at (`rule:phase7:fixed-form-rulings`) assigns admission to S5's freeze owner and its deadline-bound consumer migration to S8 before fixed-form capture. S3 and S4 neither read nor re-pin a fixed-form freeze; S3's frozen semantic.rs edit belongs to the already admitted legacy class. Their other packet and integration dependencies remain unchanged.

The maintained clarification of (`sec:w1b16:packet-domain`) is (`[DOC-sec:keyed-fixed:environment]`) and the Wave-1 packet in packages/vectors/README.md. Observations used one Elements Core v28.99.0 development build on fresh isolated elementsregtest nodes with default-challenge harness blocks and no block signing, not release, Liquid or signed-block production. The sealed per-session provenance retains exact source, binary, toolchain and image bindings. Only explicit policy-asset successors were observed; legacy fully-confidential public continuation remains unresolved. These limits preserve the archived record and supply no fixed-form native evidence. S8 must retain effective policy and its own environment; S9 still owns the required-tip description reconciliation.

Under (`rule:phase7:fixed-form-rulings`), the joined gate remains (`rec:backlog:fixed-form-closure`). Vectors owns the ADR-021 adjudication of both capture-freeze classes under (`rule:backlog:identity-admission`) and (`[ADR021-req:identity:admission-record]`), with S0 treatment at (`rec:backlog:fixed-form-decisions`). The legacy selected-source FreezeManifest and envelope are admitted as provenance at (`rec:identities:keyed-capture-freeze`): the explicit owned encoding warrants the existing recipe while preserving the sealed capture bytes and bindings and reproducing the reviewed history reconstructions. Accepted ruling 210 at (`rule:phase7:fixed-form-rulings`) discharges this admission by a vectors code step ahead of the documents-only S2 packet. The present-class admission prerequisite is satisfied; Wave 2’s relation.rs and S3’s semantic.rs re-pins may proceed under (`rule:vectors:keyed-refreeze`) when their other packet and landing dependencies hold. Each retains its own history/override and equivalence obligations. Preserve `keyed-capture-freeze-v1`; projection/encoding/domain changes require a new recipe. S2 owns the fixed-form realization recipe and pre-admission walk; S5 supplies the distinct capture-freeze subject, encoder, inventory and property warrants, and S8 activates its declared consumers before capture under ruling 211. The legacy admission does not select the fixed-form recipe, and an unwarranted property remains unadmitted. The Wave-2 packet correction and S3 landing dependency remain at (`rec:backlog:fixed-form-decisions`).

S3’s semantic.rs source/pin/history landing is additive with legacy behavior unchanged, not editorial. Its history entry supplies previous/current digest, complete diff, before/after legacy manifest/identity/attestation and byte/replay/negative controls, and affected-evidence disposition. Retained native captures stay bound to the old freeze, with relation.rs and semantic.rs each added to RETAINED_CAPTURE by its own re-pin landing at its capture-time digest; current source identity changes. Unchanged byte controls justify retaining exact-transaction native observations; a changed byte or invalid premise reopens affected evidence and cannot be answered by updating a pin. The 35-path legacy census is not enlarged with fixed-form children; S5 creates a separate inventory. No new digest value or regression pass is asserted by S0.

S7 owns the new vectors-only model/reader/evidence siblings, complete dependency closure, authenticated member fields and SOURCE-only attribution; it introduces no partial-chain API or external report schema. S8 owns new capture/admission/corpus subjects and artifact census, keeping legacy copy evidence separately named and replayed against the old leaf. The existing historical extraction tool stays unchanged; no duplicate extraction mechanism is introduced. New progress/timing uses tracing, not expansion of the printed history exception. S2’s maintained legacy environment clarification cites (`sec:w1b16:packet-domain`) and packages/vectors/README.md; the archived draft remains untouched. Exact files, shared consumers and literal registry receivers are (`tab:backlog:fixed-form-paths`), (`tab:backlog:fixed-form-sweep`).
