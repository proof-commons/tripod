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


## Supplied-key freeze inventory · `sec:vectors:keyed-freeze`

The conformance-only `keyed_final_manifest` surface is specified by (`rec:w1b16:freeze`): a mutable `FreezeManifest` offer, `FrozenItem`, `ComputedItem`, `SourceIdentity`, `Envelope`, `FreezeRefusal` and its exhaustive `FreezeBoundary`. `validate_manifest` binds the reviewed source inventory and per-key template identity; `parse_key` checks textual width and hexadecimal form. The literal `predicted_leaf` helper and second-key phrase/digest are test evidence, not an exported production format. Twelve frozen categories contain 35 selected source-file identities; sixteen categories remain per-instance. These pins do not claim a transitive workspace or executable digest. No signing authority, native verdict or chain fact is minted.

The external `keyed_final` family constructs two SOURCE events and their MERGE through public entries under two supplied keys, compares literal grammar and first-party conformance arithmetic with production, and reproduces all snapshot coordinates in a fresh process using recorded ordinary witnesses. Freshness separates state, not shared reconstruction code. Literal membership expectations are compared with the unchanged byte reader. Edge data values, the three link scopes, wrong bindings and commitment refusals have controls at their owners. Public semantic keys require no secrecy. Generic sizes and schedules follow from fixed-width data dropped before both clauses; per-key cryptographic and external premises remain conditional. No production code, previous vector, archive or pin is regenerated.


### Re-freeze and provenance · `rule:vectors:keyed-refreeze`

A change to pinned source and its re-pin land together in one gate-passing landing, independently reviewed before import. The mandatory freeze-history entry names each path, previous and new digest, the source diff, whether it changes behaviour or is editorial, and which evidence reopens under (`rec:w1b16:adoption`). A changed template, target, policy or envelope reopens its affected evidence; a new key alone does not. The initial review expansion and literal digest table are at (`tab:w1b16:freeze-history`). A re-pin is never routine permission to change a reviewed function; no check forbids paired source/pin commits, and no red intermediate main commit is required.

The source identities attest bytes compiled into vectors via relative-path inclusion in its build checkout. They do not inspect runtime source files or identify an executable, toolchain or resolved dependencies. This unpublished crate requires the workspace layout. A moved or copied checkout must rebuild vectors before a control counts; a copied old binary still holds the old bytes. `Cargo.lock` is untracked and workspace `sha2 = "0.10"` is a version range used by both commitment and oracle. Step 2's executor provenance must retain the commit, resolved lock and toolchain of the binary actually run. Shared SHA-256 is explicitly outside the prediction's independence claim.

The source census is a literal module/path-declaration check with accepted and missing-child controls, not macro or alias expansion. Non-test child files of pinned Rust files are pinned. Generic program/instruction encoding (`packages/tapscript/src/program.rs`, `instruction.rs`) and opcode/push tables (`packages/target-elements/src/opcode.rs`, `push.rs`) are outside the identity and protected for this template by independent literal prediction. Reader, stack model and conformance arithmetic are instruments to be pinned at step 5, not implicitly frozen template code. The twelve categories now contain 35 pins; sixteen variable categories are tied to exhaustive packet/input/snapshot destructuring. The selection offer helper is pure and untraced; emission remains the checked owner boundary.
