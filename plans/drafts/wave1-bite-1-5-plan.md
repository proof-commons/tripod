# Wave 1 bite-1.5 landing plan · `sec:w1b15:plan`

This sequence reads `0.7.115-dev` and receives (`rec:w1b14:closed`), (`rec:w1b13:closed`) and (`rec:w1b13:members`). Its adoption and decisions are recorded at (`rec:w1b15:adoption`); the proposals below retain their reasoning. It implements (`proposal:wave1:bite-1-5`) and (`task:guide15-exec:bite-1-5-deliverables`) under rulings 159, 164 and 165 at (`rule:phase7:wave1-realization-rulings`). It specifies evidence machinery, not another covenant, production history API or realization claim. No build, test or native execution is run for this study; its documents gate validates the plan only.

**Disposition (Adopted sequence and decisions)** · `rec:w1b15:adoption`

The five-step sequence is adopted for bite 1.5, with the three recommendations and the finality decision below. These dispositions govern the implementation; the original proposals remain as the record of the reasoning.

(`q:w1b15:homes`) is adopted: the expected-set model, byte/evidence reader and six validators live in vectors, with no production reader export and no new model crate. The model uses standard-library data only and calls none of the shared substrate operations. The reader may use the canonical decoder, stripped serialization, hashes, target field types, first-party conformance curve reconstruction and bounded byte model; it never calls `keyed_request`, `keyed_construct`, `bind_keyed_members`, compiler plans or declaration evaluators. Ruling 159 requires an independent finite comparison: this split makes what decides the answer checkable. Step 1's import-boundary control is backed by source and dependency review, since a spelling scan proves nothing about macros or aliases.

(`q:w1b15:reader`) is adopted: every authenticated SOURCE record is eligible, with no amount threshold and attribution unassigned. An explicit amount is an annotation with parent-byte provenance; unknown is never zero. The fixed subset uses explicit asset and a value threshold fixed in its rule identity; the changing rule toggles one named record between two observation epochs. Membership is fixed for a handle and grows along one valid continuation path; all-record and fixed-subset accepted sets grow along that path, while the changing rule has no monotonicity claim. No claim crosses replacement histories, changed premises, changed parameters or incomplete readings. This is the smallest rule set separating membership from acceptance and exhibiting both monotone and non-monotone cases.

(`q:w1b15:reports`) is adopted: internal typed Rust reports, one per family, use the framing below. Each validator recomputes from subject bytes and evidence, trusting no other family's verdict, declared total or report digest. Test-local JSON may carry public inputs between processes and establishes no interchange contract. This follows (`dec:guide15-exec:reader-evidence`): no actual external consumer exists, and a real consumer reopens ruling 165.

(`q:w1b15:finality`) is decided: finality is a typed reading premise, never a value selected by this repository. Step 3 publishes the declaration form: deployment identity, policy version, history anchor, depth or replacement rule, inclusion and retrieval sources, and unspentness-observation scope. A reading is nonfinal unless it names an accepted declaration matching its deployment, anchor and policy version; a requested final reading without one refuses at the reader-evidence boundary. Fixtures use a declaration marked hypothetical, establishing nothing about any deployment. An actual declaration is deployment data owed at activation, just as a key value is deployment data, not plan content. The gates require separately published reader/evidence premises and finality as a named assumption, not a deployment; Wave 1 claims no production activation, and construction evidence cannot select a depth. The published form, refusal without it and hypothetical-declaration controls meet the packet's finality clause; absence of deployment data is not a blocker. An actual deployment or gate text demanding a concrete declaration reopens this question.

Independent review follows step 1 for model independence and fixture separation, step 2 for classification completeness and graph validation, step 3 for the reader rule, premise form and report boundary, step 4 for validator independence and resource units, and the gate packet at step 5.

The test-time bounds are adopted: a 10-second subject timeout and a 60-second family-process cap, every new process with its own timeout and printed duration, and the 253-member subject built once per process. Report a breach instead of widening a bound.

## Ownership and independence · `sec:w1b15:independence`

**Proposed homes** · `tab:w1b15:homes`

| Component | Proposed files and authority | Independence boundary |
|---|---|---|
| Expected-set model | `packages/vectors/src/keyed_history_model.rs`; finite events, output identities, keys, first birth and single consumption directly from the paper | Standard-library data only; no realization evaluator, compiler, linker, transaction, reader or builder result decides an expected set. Compute sets in a topological fold. |
| Public history fixtures | `packages/vectors/tests/keyed_history/fixture.rs` and bounded fixture data | Predeclare the event graph independently, then construct bytes with the public bite-1.4 entries. A checked bijection associates symbolic event names with resulting content addresses; it cannot change the expected edges, classification or sets. |
| Byte/evidence reader | `packages/vectors/src/keyed_history_reader.rs` | Reconstruct by dependency traversal from canonical bytes, authenticated programs and explicit evidence; never read builder totals, model sets, request roles or compiler/declaration evaluation as authority. |
| Six validators | `packages/vectors/src/keyed_evidence/` with separate birth, continuation, recurrence, reading, acceptance and resources validators | Shared framing and subject binding are allowed; each family rechecks its own witnesses and claims, not another family's success flag. No catch-all boolean certifies six families. |
| Contracts and reports | Vectors/realization READMEs and `plans/packages/{vectors,realization}.md`, plus this plan | Internal typed reports and test-local fixtures; no stable descriptor or maintained wire interchange, and no production dependency on vectors. |

The reader may call transaction's existing canonical decoder, stripped serialization and hashes for syntax/content addressing; target types for field interpretation; existing first-party conformance curve reconstruction for public recipe authentication; and the bounded byte model for the two clauses. Cross-check selected transaction ids, output order, values and witness extraction with the pinned Elements comparison in vectors. These are shared substrate operations, explicitly not independent recurrence evidence. The model must not call any of them. The reader must not call `keyed_request`, `keyed_construct`, `bind_keyed_members`, compiler plans or declaration evaluators to classify SOURCE/MERGE or obtain coverage. The fixture builder may call those production entries; only its bytes and separately supplied public evidence cross into the reader. Keep the conformance arithmetic independent of the production native-linked curve wrapper; neither comparison proves chain inclusion.

Ruling 159's import-boundary control should test forbidden imports, qualified paths and aliases, backed by a dependency/source review and a separately compiled standard-library-only model test harness on the gate node. A spelling scan alone is not a proof about macros or alias expansion. Prefer no macros or external aliases in the model. Do not extend `packages/model` or target-elements-conformance without a demonstrated consumer or missing target-generic fact (`proposal:w1route:model-home`).

## Checks and limits · `gate:w1b15:landing`

Every landing keeps attestation paths, the ten realization/compiler length/digest pins, canonical architecture publication, all existing vectors and every keyed, linked, committed and constructed byte exact; no expectation is regenerated. Rerun `preserve_engine_base`, the canonical publication control, the existing recipe/reproduction controls and archive admissions alongside the affected-package tests. No shared engine, emitted grammar, production construction algorithm, new covenant predicate, accumulator, STATE metadata or owner-authorization law belongs here. Report a discovered production defect before changing it.

Each code step passes formatting, clippy with warnings denied, affected-package tests and rustdoc, registration/census, labels and plans checks; the explicit step gates below supplement that common bar. The final packet covers architecture, realization, compiler, model, target-elements, tapscript, linker, transaction, target-elements-conformance and vectors, plus lint and plans. Future briefs supply exact remote commands. All execution is offline: construction/replay, finite histories, curve operations and bounded byte evaluation. No step launches native Elements; final-byte execution and policy observations remain bite 1.6.

Every newly launched test process has its own timeout, exact selection, exactly-one-child assertion where applicable, exit status and printed duration. Use the existing ignored-child capture pattern; parent and child each have a bound. Target a few seconds per subject, use the adopted 10-second subject timeout and 60-second family-process cap, and report a breach rather than silently widening it. The 253-member subject is built/decoded once per process and reused immutably across validators; index parents once, not once per member. Gate queue/build time is separate from subject duration. File estimates include tests, registrations and documentation; overlapping sets are not additive.

## Five landings · `sec:w1b15:steps`

### 1. Independent expected sets and public histories · `step:w1b15:histories`

**Scope: 9–13 files.** Add the model, fixture builder and `tests/keyed_history` target, with `lib.rs`, Meson and vectors documentation. Reuse the public recipe, sealed requests, construction and recorded-signature replay from `packages/vectors/tests/keyed_reproduction/fixture.rs`; extract only test support needed by both consumers, retaining its old comparisons. Build SOURCE, single-input MERGE, disjoint multi-input MERGE, a longer path, two-key coexistence and the existing two-/253-member subjects. Synthetic sponsor parents remain explicitly supplied fixture premises, not an invented chain or admitted history. No secret is needed to reproduce a record; disposable fixture signing remains ordinary funding support only.

**Controls and boundaries.** Independently assert SOURCE's singleton `(key, handle)`, MERGE's union with no new record, unique SOURCE provenance, disjoint live sets and path growth. Same local identities under different keys stay separate. Duplicate consumption and conflicting event identities refuse at the model's valid-history boundary; a cycle refuses at its graph boundary. Add a later same-key output through structural construction: it has no membership set, while output 0 is the accepted twin. Do not label the structural extra-output subject admitted under the three-output policy envelope. Test the model import boundary and the symbolic-to-byte identity bijection, including duplicate/mismatched associations.

**Gate and handoff.** Vectors plus transaction/linker/tapscript and the common preservation checks. Discharges ruling 159's finite oracle and the expected side of requirements 1–5; hands independently declared graphs, exact public bytes, policy/fee/census inputs and expected sets to step 2. Review model independence and fixture separation before the reader relies on them.

### 2. Byte-derived reconstruction and evidence outcomes · `step:w1b15:reader`

**Scope: 9–14 files.** Add `keyed_history_reader.rs`, its evidence/identity helpers and tests, registrations and realization-owned documentation. Reuse only the substrate operations permitted above. Classify every input from its actual parent output and authenticated key/birth position; never let missing parent or unknown membership silently mean SOURCE. Authenticate the selected handle, output order and each consumed handle's two clauses; reconstruct dependency sets independently of the model. Preserve relation/member identities as evidence for later discharge, without importing the producer's coverage as a verdict.

**Controls and named outcomes.** Complete closures equal the model for every control; reorder retrieval responses without changing results. Missing parent, missing inclusion/retrieval assertion or unresolved input classification yields `Incomplete` at evidence completeness. Conflicting identity, duplicate consumption and cycles yield `Invalid` at evidence consistency, not a covenant refusal. A later-shaped output yields `NotHandle`, never an empty set. Wrong-key association refuses at key authentication; later birth at the born-first model boundary; wrong successor/version at continuation; wrong parity/uncommitted leaf at commitment authentication. Keep a passing member beside the one changed member. An unsupported decoder form is `UnsupportedForm`, not invalid covenant semantics.

Malformed evidence is an untrusted fixture envelope, not a sealed construction result. A byte mutation retaining the old txid fails content addressing. Conflicting aliases and cyclic references can be supplied to the evidence-graph validator before content authentication; do not claim to construct a self-consistent cyclic txid graph or an admitted transaction. Test both the public byte refusal and the internal graph refusal at their actual boundaries. Local content addressing alone establishes none of inclusion, single-consumption history or finality.

**Gate and handoff.** Vectors, transaction, target-elements and target-elements-conformance plus the common checks. Covers requirements 1–6 and the reading family, with all four inherited parent premises retained. Hands typed complete/incomplete/invalid/non-handle outcomes and per-relation evidence to step 3. Independent review is especially valuable here: classification completeness and graph validation are the trust boundary, not the fixture builder's success.

### 3. Reader policy, premises and internal report schema · `step:w1b15:reading`

**Scope: 8–12 files.** Add acceptance/premise helpers and internal report framing under `keyed_evidence/`, tests and the vectors/realization contracts. Reuse the reader's sets unchanged. Publish the adopted rule, report format and typed finality-declaration form, with refusal without a matching accepted declaration and hypothetical-declaration controls. Actual deployment data is owed at activation, not to complete this step (`rec:w1b15:adoption`).

**Controls and boundaries.** All-authenticated-record eligibility, a fixed immutable subset and a changing filter return different accepted sets over the same membership set. Fixed-set growth is checked only along extensions of one valid history. The changing filter has an explicit removal counterexample and no monotonicity promise. Unknown amount is not zero; absent attribution is unassigned, not an owner. Missing evidence refuses a requested complete/final reading at the reader-evidence boundary, never at covenant execution. A finality declaration for another deployment, history anchor or policy version does not authorize a final claim. Fixtures may exercise an explicitly hypothetical declaration; they do not establish the actual deployment premise.

**Gate and handoff.** Vectors and realization plus the common checks. Covers requirements 7–8, rulings 164–165 and the last two realization decisions (`rec:guide15-exec:study-dispositions`). Hands versioned internal typed reports, separately published assumptions and accepted-set rules to all six validators. Review the reader rule, premise form and report boundary before family reports freeze.

### 4. Six independently checked family reports · `step:w1b15:families`

**Scope: 12–18 files.** Complete six validator modules, focused mutants and bounded test drivers, report registrations and vectors documentation. Reuse exact histories, reader results, model traces, extracted leaf contexts and the existing resource archive unchanged. Validators receive the underlying subject bytes/evidence plus a proposed report and recompute their family checks. They do not accept declared totals, producer receipts or a report digest as proof of its assertions.

**Controls and boundaries.** Each family has an accepted report and one-coordinate mutants: birth position; one member's continuation/key; SOURCE identity or a spurious record on MERGE; missing dependency/premise; an acceptance filter incorrectly changing membership; and a wrong resource count/unit/subject binding. Every refusal identifies the family-report validation boundary and its underlying observation boundary. Change the report's subject bytes, family, relation id, expected outcome, premise, missing field and evidence reference separately; no family may validate by trusting another report's pass bit. Corrupted assignment/omitted-clause cases remain internal where sealed requests make them unreachable publicly.

**Resources.** Compare grammar-derived predictions from the unchanged tapscript projector against independent byte-model operation/stack observations and direct serialized sizes: separate birth, continuation and composition; script/control versus execution arguments; CompactSize transitions (including 252/253); 88-byte script stack, 91-byte member witness and 255-unit member contribution; complete transaction weight and supplied non-member terms. Reuse existing overflow and resource-dimension controls. Native elapsed time, CPU, stalls, validation charging and relay policy are not newly observed here: bind archived observations to their own exact subjects and mark unavailable final-subject dimensions absent. A measured-envelope maximum is not a script law. Hand final-subject measurement obligations to bite 1.6 and keep records in (`obl:guide15-exec:measurement-home`).

**Gate and handoff.** Vectors, tapscript, transaction and target-elements/conformance plus the common checks. Produces all six reports and the eight-row trace below, with explicit conditional evidence. Independent review should examine validator independence, resource units and the separation between model observations and native measurements. Hands complete offline subjects and report validators to the packet and bite 1.6.

### 5. Gate packet and review · `step:w1b15:close`

**Scope: 6–10 files.** Update this plan, vectors/realization READMEs and contracts, Phase-7 and backlog records; add only a demonstrably missing small control. Assemble `rec:w1b15:gate` with a row per clause of the three gates, named tests/records, source and dependency inventories, measured subject durations and a covering full-package log. State production reachability: internal conformance readers/reports do not create a production chain reader, signing authority or native acceptance. Preserve existing backlog follow-ups.

**Gate and handoff.** Run the full package set in (`gate:w1b15:landing`); record command, start/end, wall, queue, exit and scoped battery resource deltas. Offer the packet for independent review under (`gate:wave1:bite-1-5-packet`) and (`gate:guide15-exec:bite-1-5-gate`); write a separate closing disposition only after review. The finality clause is met by the published premise form, refusal without a matching accepted declaration and hypothetical-declaration controls; an actual deployment declaration is not required here. A missing required control remains open. Hand byte-bound relation evidence, validators and absent metrics to bite 1.6; no full-wave realization claim follows from this packet.

## Reader and report decisions · `sec:w1b15:decisions`

**Proposal (Conformance homes)** · `q:w1b15:homes`

Adopt the vector-owned model, byte reader and six validators in (`tab:w1b15:homes`), with no production reader export and no new model crate. This follows ruling 159 and keeps the independent algorithm outside the declaration/constructor path. Reopen on an actual runtime consumer or a failed independence check, not merely because identifiers are shared.

**Proposal (Candidate reader rule)** · `q:w1b15:reader`

Adopt all authenticated SOURCE records as eligible, with no amount threshold and attribution explicitly unassigned. An explicit SOURCE output-0 value/asset is an optional annotation with its parent-byte provenance, never current handle value, record credit or entitlement; a committed amount remains unknown. Unsupported confidential construction is not introduced to test this distinction: use an internal annotation subject if needed. For the fixed subset, use the immutable predicate “SOURCE output-0 has explicit asset A and explicit value at least T”, with A and T fixed in the report's rule identity and no claim that T is a funding floor. For the changing rule, toggle a named record's eligibility between two observation epochs. Membership of a fixed handle never changes; membership along one valid continuation path grows; all-record and fixed-immutable accepted sets grow along that path; the changing accepted set need not. No monotonicity claim crosses replacement histories, changed premises, changed rule parameters or incomplete readings (`proposal:wave1:reader-rule`), (`[A-rem:kpc-interface:record-evidence]`).

**Proposal (Internal reports)** · `q:w1b15:reports`

Adopt internal Rust report types, one per family, with shared framing: subject bytes/content identity, key and handle, history/policy provenance, family and paper/requirement ids, rule identity where relevant, evidence references and explicit assumptions, observation boundary, typed outcome, observed/predicted quantities with units and absent dimensions. Validators bind every claim to its underlying subject and reject missing/contradictory fields. Retained test-local JSON may transport public inputs between processes; it is not ADR-022 interchange or a stable production descriptor. No external consumer is named by the present brief; a real consumer and compatibility obligation would reopen ruling 165.

**Decision (Typed finality premise and deployment data)** · `q:w1b15:finality`

Under (`rec:w1b15:adoption`), step 3 publishes a typed declaration form binding deployment identity, policy version, history anchor, depth or replacement rule, inclusion/retrieval sources and unspentness-observation scope. Finality remains a named assumption, not something a finite test proves. A reading is nonfinal without a matching accepted declaration, and a requested final reading without one refuses at the reader-evidence boundary. Hypothetical fixture declarations test this contract without establishing any deployment premise. The actual declaration is deployment data owed at activation; neither this plan nor construction evidence selects a confirmation count. The published form and controls meet this bite's finality clause without a deployment. An actual deployment or a gate requiring a concrete declaration reopens the question; per-instance missing premises remain visible, including on admitted artifacts.

## Requirement and family trace · `tab:w1b15:requirements`

These eight rows join (`tab:wave1:requirement-trace`) to (`tab:wave1:family-trace`); resources is a cross-cutting sixth family, not a ninth delivery requirement or a new covenant clause.

| Requirement | Paper anchor | Families and step evidence |
|---|---|---|
| 1. Born first and single birth | (`[A-def:kpc-model:first-output]`), (`[A-prop:kpc-model:single-birth]`) | Birth; continuation also checks both clauses. Steps 1–2 output-0/later-non-handle and per-member subjects; step 4 resource accounting binds the same bytes. |
| 2. Continue first and unique continuation | (`[A-thm:kpc-invariants:unique-successor]`) | Continuation and birth; steps 1–2 unique consumption/successor and each consumed handle's comparisons; resources covers execution subjects. |
| 3. Fixed-key separation | (`[A-def:kpc-model:keyed-membership]`), (`[A-thm:kpc-invariants:key-isolation]`) | Continuation; steps 1–2 two-key coexistence, mixed/wrong-key boundary and independent reconstruction. |
| 4. SOURCE record identity | (`[A-op:kpc-operations:source]`), (`[A-thm:kpc-invariants:source-provenance]`) | Record recurrence; steps 1–2 singleton keyed handle and unique provenance; step 4 forged report record refuses. |
| 5. MERGE recurrence and disjoint growth | (`[A-op:kpc-operations:merge]`), (`[A-def:kpc-operations:classification]`), (`[A-thm:kpc-invariants:set-membership-growth]`) | Record recurrence and reading; steps 1–2 single/multiple union, no new record, live disjointness and path growth. |
| 6. Faithful membership reconstruction | (`[A-def:kpc-interface:membership-reading]`), (`[A-thm:kpc-interface:membership-correctness]`), (`[A-cor:kpc-interface:path-reading]`) | Reading; step 2 independent model/reader comparison, incomplete/invalid/non-handle cases and complete classification. |
| 7. Reader-side acceptance | (`[A-rem:kpc-interface:record-evidence]`) | Acceptance; step 3 filters preserve membership, fixed-rule growth and changing-rule counterexample. |
| 8. Published evidence assumptions | (`[A-sec:kpc:interface]`), (`[A-sec:kpc:limitations]`) | Reading and all report headers; step 3 per-instance premises/finality, step 4 resources' conditional and absent dimensions. Resources compares exact subjects without supplying a paper bound. |

## Inherited obligations and next boundary · `tab:w1b15:handoff`

| Received item | Owning step and disposition |
|---|---|
| Bite-1.3 history recognition, inclusion/retrieval/finality (`rec:w1b13:closed`), (`rec:w1b13:members`) | Steps 1–3 finite history recognition and published evidence premises; local receipts and offline assertions are not chain evidence. |
| Four parent facts on every bite-1.4 artifact | Steps 2–3 retain inclusion, unspentness, retrieval and finality with provenance and missing-evidence outcomes. Historical membership does not require every ancestor to remain unspent; unspentness claims name the live handle and observation prefix. Finality is required for a final claim. |
| SOURCE four/MERGE three evidence rows and per-member relation evidence | Steps 2 and 4 bind evidence to the exact relation, member, key and bytes; no producer coverage receipt alone discharges it. |
| Multi-member and one-wrong-member subjects | Steps 1–2 reuse serialized two-/253-member controls and model boundaries, preserving the unchanged passing member; step 4 validates their report evidence. |
| Later same-key outputs | Steps 1–2 structural extra-output subject is `NotHandle` with no set; attempted spend fails born-first model evaluation. This is not relay rejection or native execution. |
| Public-input packet, SOURCE/two-/253-member constructed bytes, fee and census | Step 1 reuses test-local reproduction and recorded signatures; step 4 binds resources to exact subjects. No descriptor is promoted to a stable format. |
| NUMS/discrete-log, preimage, second-preimage/collision and target assumptions | Steps 2–4 retain named conditional authentication premises; no unconditional key-path impossibility or independent native-backend claim. |
| `MemberExecutionAndRelationEvidence`, including structural results | Step 4 supplies only the relation/model side. Bite 1.6 records final-subject discharge when both this relation evidence and its own native execution exist on the same final bytes. |
| Final-byte policy and (a)–(f) measurements, funding and scaling envelope | Step 4 lists absent final observations; bite 1.6 repeats them, observes inclusion/unspentness for its subjects and records `FinalBytePolicyObservation`. Rulings 162/163/180 remain construction/measurement policy, not membership rules. |

MERGE adds no record because complete classification selects a union of consumed handles' SOURCE records; other transaction contents are not a SOURCE event for that key. Reader acceptance cannot alter that recurrence. Nothing in these reports adds a covenant clause, carried accumulator, authorization condition or finite MERGE law. The six target measurement records retain their research home at (`obl:guide15-exec:measurement-home`); gate performance belongs in (`tab:test-performance:register`), not a fabricated native resource report.

## Record (Independent finite histories) · `rec:w1b15:histories`

Step 1 implements (`step:w1b15:histories`) on `0.7.116-dev` under (`rec:w1b15:adoption`). The new model at `packages/vectors/src/keyed_history_model.rs` is compiled only by external tests and an isolated standard-library harness, not exported by the vectors library. This deliberately omits the scoped `lib.rs` registration: step 2’s reader will be a library module and cannot import this external-test model; that separation must remain. Public-byte fixtures and association checks live in `packages/vectors/tests/keyed_history/`; their external registration is in `tests/maturity_native.rs` and the integration-test exclusion census in `packages/vectors/meson.build`. No production code, declaration evaluator, covenant clause or expected preservation value changes.

**Model surface and paper statements** · `tab:w1b15:model-statements`

| Surface / statement | Paper authority | Independent control |
|---|---|---|
| `Key`, `EventId`, `Output { event, position }`, `Handle { key, output }`, `Event { id, inputs, outputs }` | (`[A-def:kpc-model:history]`), (`[A-def:kpc-model:keyed-membership]`) | `model_checks.rs::statements` distinguishes identical local output names under two keys; complete declarations precede any construction. Identities and keyed output membership are model premises, not hashes or decoded claims. |
| `analyze` topologically folds the graph; `History::order` is deterministic under declaration permutation | (`[A-def:kpc-model:history]`), (`[A-def:kpc-operations:classification]`) | Reverse declaration order gives the same history; the complete topological order is checked before any clause fold, so a cycle precedes clause refusals. |
| `History::records` gives a SOURCE singleton keyed handle and a MERGE union, introducing no MERGE record | (`[A-op:kpc-operations:source]`), (`[A-op:kpc-operations:merge]`), (`[A-thm:kpc-invariants:source-provenance]`) | Literal singleton/union expectations in `model_checks.rs::statements` and `mod.rs::assert_expected`; removing one declared dependency changes the union and fails the original equality. No builder total supplies the expectation. |
| `History::live` and `successor` expose disjoint live sets, unique continuation and path growth | (`[A-thm:kpc-invariants:unique-successor]`), (`[A-thm:kpc-invariants:set-membership-growth]`), (`[A-thm:kpc-invariants:key-isolation]`) | Disjoint two-key frontiers and the multi-step path; repeated consumption refuses rather than manufacturing a second successor. |
| Non-handle outputs have no set (`None`), not an empty set; consuming a later same-key output refuses born-first | (`[A-prop:kpc-model:single-birth]`), (`[A-def:kpc-model:first-output]`) | `model_checks.rs::refusals` pairs output 0 with position 1, and permits ordinary later output consumption; the `later` byte fixture has identical keyed programs at positions 0 and 1 but only the first has a model set. |

The model uses one `std::collections` import and ordinary standard-library operations, with built-in derives and no custom macros, external aliases, decoder, hash, curve, process call or substrate operation. Its eight `Refusal` variants and four `Boundary` values describe declared-model failures; they are never target or native rejection reports. `History` holds private sets/frontier/successors/order and the declared output census with read-only projections. `History::query` returns `OutputStatus::{MissingEvent, MissingOutput, NotHandle, WrongKey(Key), Handle(Handle)}`; only a declared ordinary or later output is `NotHandle`. `records` remains the optional set lookup. The model receives graph-local outputs and inputs; ordinary external sponsor funding remains a separately stated fixture premise, not a claim that this reduced graph authenticates a full chain.

**Refusal inventory by boundary** · `tab:w1b15:history-refusals`

| Boundary | Conditions / variants | Accepted twin and negative |
|---|---|---|
| Valid-history premises | `DuplicateEvent`, `ConflictingEvent`, `MissingOutput`, `DuplicateConsumption` | A SOURCE followed by MERGE; repeat the same declaration, bind its name to changed contents, change one input to an absent position, or consume one output twice within/across events. |
| Dependency graph | `MissingEvent`, `Cycle` | The same acyclic control; name an absent parent or add the back edge. No claim to construct a hash-consistent cyclic transaction graph. |
| Born-first model clause | `LaterOutputConsumed` | Position-zero input accepted; change only its birth position to a declared later covenant output. |
| Continue-first model clause | `WrongSuccessor { input, spender }` | Same-key successor accepted; change only the successor key. |
| Test fixture association | `DuplicateName`, `DuplicateAddress`, `UnknownName`, `MissingName`, `MissingBytes`, `AddressMismatch`, `InvalidBytes`, `InputEdges`, `OutputKeys` | `association_subjects` checks an accepted complete association, then one changed coordinate per variant, and restores the accepted control. These are fixture-binding refusals, not evidence that production can manufacture an invalid sealed request. |

All eight model refusal variants and their exact `boundary()` values are exercised by `tests/keyed_history/model_checks.rs::refusals`. The independent harness also checks source identity, recurrence, provenance, key separation, disjointness, growth, non-handles and permutation in `statements`. Model outcomes are not reader verdicts; the exact step-2 mapping is recorded below, rather than inferred from the four boundary names.

**Public-byte subjects** · `tab:w1b15:history-subjects`

| Child subject | Declared graph and public material | Standing |
|---|---|---|
| `source` | One public SOURCE, no consumed keyed handle | Singleton record under supplied sponsor-parent premises. |
| `single` | Public SOURCE then one-input MERGE | Same singleton, no added MERGE record. |
| `ordinary-edge` | A SOURCE followed by a SOURCE spending the earlier change output at index 1 as its sponsor | A graph-local input is not automatically a member; ignoring its index fails the ordered association subject. Both SOURCE sets remain live. |
| `multi` | Two public SOURCEs then disjoint two-input MERGE | Literal two-record union and a single live handle. |
| `path` | Two SOURCEs, a single-input MERGE, a two-input MERGE and another single-input MERGE | Dependency-derived path growth; consumed intermediate handles retain their fixed sets. |
| `two-keys` | Two independent SOURCE/MERGE paths under distinct keys | Disjoint final sets and programs recognized through the shared public leaf recipe with independent conformance tweak arithmetic. |
| `wide` | 253 public SOURCEs then one 253-member MERGE | Full declared-to-constructed association, literal 253-record union; no finite MERGE law. |
| `later` | Structural SOURCE with another same-key program at output 1 | Never labelled admitted under the measured three-output envelope; no set for position 1, and its model spend refuses born-first. |
| `existing-two`, `existing-253` | The unchanged reproduction cases, replayed from their recorded ordinary signatures | Supplied member parents are declared exactly as two outputs both bearing key 42. The association checks those bytes and the constructed MERGE’s ordered member-parent output-zero inputs followed by its sponsor, and keyed/ordinary/ordinary outputs. Position 1 of every boundary parent is a later non-handle. Their own input bytes remain boundary premises, not constructed complete SOURCE histories; `wide` supplies that complete keyed graph. |
| `association` | The `multi` graph with independent byte binding and focused mutants | Exact name/address bijection, ordered keyed input edges and output-key census; the immutable association checker cannot edit expected sets; each mutated coordinate refuses beside the accepted association. |

`fixture::build` freezes and analyzes the declared graph before constructing bytes. It uses public bind/plan/emission/link/commitment/request/construction entries through the existing reproduction helper. Only test support is shared: the reproduction fixture module is imported once in the external target, disposable signature setup is extracted, and immutable commitments are cached by key/scope. The original reproduction comparisons remain. The public-input packet, including fee, policy provenance/asset and input census, is generated once per subject; unsigned construction for funding-signature setup and recorded-signature reconstruction are distinct passes, not fresh secret-dependent reproduction. No secret is consumed by the replay path. Wide subjects are not rebuilt separately for each model assertion.

Sponsor parent transactions are supplied fixture premises. Their txids, sequences and explicit amounts are distinct, and selected sponsor vouts are nonzero; graph events also have distinct output amounts and input sequences. Parent decoding is indexed once during fixture construction, and the association observer independently decodes/indexes each constructed event once. The observer computes content addresses from canonical bytes and compares ordered input outpoints against the declared edges plus the supplied sponsor outpoint, then compares output keys against programs reconstructed from the shared public leaf recipe with conformance tweak arithmetic. This is shared substrate authentication, not independent recurrence or recipe-correctness evidence. It never rewrites edges, classification or model sets. Its permutation control changes association offer order only; its wrong address, changed edge and changed output-key subjects demonstrate that the comparison can fail. This observer is a fixture association check, not step 2's history reader, chain inclusion validation or independent recurrence algorithm.

**Independence evidence and limits.** `keyed_history_oracle_import_boundary` scans listed forbidden imports, qualified paths and aliases as a fast first check. It also rejects `cfg`, `cfg_attr`, `mod`, `path`, string literals and comments other than whole-line comments, so no configuration or nested source escapes the isolated compilation. Twelve rejecting mutants each isolate one denied token or literal condition, including a conditional import of unlisted `elements`, an `architecture` alias and an import behind a string containing comment delimiters. The harness then invokes `rustc --edition=2024 -Dwarnings` without `--extern` or a dependency search path and includes the actual model source. Together these controls show that the model names nothing outside `std` in any configuration; the scan performs no macro or alias expansion, and semantic independence remains a matter for source/dependency review. Review confirms one `std::collections` import, built-in derives, standard collections and identity type names used only as invariant diagnostics, no substrate calls and no hidden builder answer. The isolated compile sees every model item that the integration tests see. Compilation occurs only on the gate node. Ruling 159’s conformance-only home remains intact.

**Timing and correction evidence.** Each child subject has an unchanged 10-second timeout; the family child has a 60-second cap. The isolated compiler process has a 60-second timeout and the model executable a 10-second timeout, each printing duration and exit. Child tests are ignored in ordinary selection and launched explicitly with `--ignored --exact`; line-anchored assertions require exactly one test at each level. The first gate at `/var/home/tripod/loops/tmp/gate-w20b1-20261112T120358Z.log` was red solely because the outer assertion counted the ten relayed child harness markers as its own (11 instead of 1). Every subject child passed. Prefixing relayed output corrected that harness failure without changing a model, byte or expectation. That gate took 341 seconds after a 45-second slot wait; its package-test journal retains the actual child results. Final measured durations and the corrected covering run are recorded below.

**Reader handoff.** Step 2’s byte reader receives only `Fixture::reader_inputs() -> ReaderFixture`: canonical event bytes, checked identity association, semantic keys, sponsor premises (`parent`, `txid`, `vout`), boundary-parent event names, and per-event `PolicyReading` (`fee`, `asset`, `provenance`, `rates`, `minimums`, `ceilings`, `package_before`, `signature_bound`). It receives no declared graph, expected sets, construction packet or SOURCE/member flags. A source-field inventory lock and a mutation of construction flags pin that separation. The comparison harness retains graphs and expected sets separately. Recorded signatures are in the public transaction bytes; structural later-output setup clears the obsolete template witness. The complete `wide` graph is not replaced by a boundary-premise regression. Inclusion, unspentness, retrieval, finality and relation evidence stay open per instance; no model result is native acceptance, an admitted history or a final reading. Step 2 owes independent byte/evidence classification and reconstruction.

**Measured offline subject walls** · `tab:w1b15:history-walls`

These are process walls after the review fixes, including construction, assertions and child startup, not native measurements or a performance claim. The raw transcript is retained in `/var/home/tripod/loops/tmp/gate-w20b1-20261112T130201Z.histories-preservation.tar` (`vectors-test-maturity_native.json`); every subject exited zero under the unchanged 10-second timeout. The family child took 3.393143 seconds under 60 seconds. The standard-library-only compile took 2.011692 seconds under 60 seconds; its executable took 0.011576 seconds under 10 seconds and printed 0.003380 seconds for its model controls. No bound was widened.

| Subject | Wall, seconds |
|---|---:|
| SOURCE | 0.050802 |
| Single-input MERGE history | 0.046885 |
| Ordinary graph-local change edge | 0.190096 |
| Disjoint multi-input MERGE history | 0.104838 |
| Longer path | 0.104787 |
| Two keys coexisting | 0.160969 |
| 253 public SOURCEs followed by MERGE | 1.530123 |
| Later same-key structural output | 0.100048 |
| Existing two-member reproduction | 0.092548 |
| Existing 253-member reproduction | 0.185944 |
| Association controls and negatives | 0.811188 |

**Code verification.** `/var/home/tripod/loops/tmp/gate-w20b1-20261112T121755Z.log` ran from 12:17:55Z to 12:21:30Z on 2026-11-12: `== green`, exit 0, 215-second gate wall after 392 seconds queued. It covered vectors, transaction, linker and tapscript, with 27 package lanes, 17 lint lanes and six checks green, plus formatting, clippy and rustdoc. Battery walls were setup 1, fmt 3, clippy 2, package tests 154, doc 29, lint 24 and checks 2 seconds. The log retains each battery's resource samples in its own `tripod-gate-w20b1-20261112T121755Z.service` cgroup, not node-wide attribution. Both gates used the supplied slot-governed transient unit and a foreground SSH wait through the end marker; the first red round is retained above rather than erased.

Fast checks first repaired formatting, duplicate fixture inclusion and local lint findings. One repeat fetched the preceding tip because checking started before push completion; it repeated old findings and is not evidence about the corrected tree. The corrected code's fast check passed in 71 seconds (fmt 4, clippy 2, rustdoc 65). There are no production-source changes in architecture, realization, compiler, tapscript, linker, transaction or target-elements/conformance; frozen vector expectations and archives are untouched. The package gate reran existing vector/reproduction controls. The final step gate additionally includes architecture and compiler to rerun the adopted common-bar projection pins and canonical publication comparison, without broadening code scope; its final-tree run `/var/home/tripod/loops/tmp/gate-w20b1-20261112T122415Z.log` finished `== green`, exit 0, in 239 seconds (12:24:15Z–12:28:14Z, zero slot wait), with 29 package lanes, 17 lint lanes and six checks. Its preserved package journals are `/var/home/tripod/loops/tmp/gate-w20b1-20261112T122415Z.preservation.tar`; the ten-pin control `tests::engine_base_capture::preserve_engine_base` and `tests::keyed_profile_tests::attestation_canonical_bytes_and_publication_equal_the_opening_base` both passed.


**Review closure and precedence** · `rec:w1b15:histories-review`

The history review changes test evidence and records only. `analyze` checks id-sorted declaration groups first (the lowest duplicate id wins; any differing contents in its group yield `ConflictingEvent`, otherwise `DuplicateEvent`), then all missing events, all missing output positions, duplicate consumption, and complete topology. Each scan uses event-id order and declared input order. Only then are born-first and continue-first clauses folded in minimum-ready-id topological order and declared input order. A cycle plus an independently wrong successor pins graph-before-clause precedence; declaration permutations pin duplicate precedence. The immutable event index and dependency adjacency avoid repeated parent and ready scans. Missing completed parent sets are explicit `expect` invariants with a `# Panics` contract, not silent omissions.

`model_checks.rs::refusals` pins both mixed-key successor choices, ordinary-first and zero-output spenders, self-reference, an empty event and a missing position, and ordinary graph-local consumption as a new SOURCE. Every refusal’s boundary is asserted. `assert_expected` pins both intermediate path sets, all four successor edges, both key-specific SOURCE singletons and the final two-key frontier. `assert_invariants` supplements independent literals for every accepted fixture: disjoint live sets, their union equal to graph-declared SOURCE handles, per-edge growth and a successor for each consumed handle. No-new-record assertions now inspect model sets. The association’s immutability follows from its shared reference, not a tautological before/after assertion.

The association now distinguishes missing offers (`MissingName`) from missing byte entries (`MissingBytes`). Both have subjects. An attempted nonminimal CompactSize encoding refuses decoding and is reported as `InvalidBytes`; the decoder’s minimal CompactSize, witness-flag, superfluous-witness and trailing-byte checks admit no known decode/reencode mismatch in this explicit envelope. Therefore the re-encoding equality remains defensive, not a separately reachable public noncanonical form. The ordinary-edge subject spends the preceding change output at index 1; changing only the declared index to 0 refuses `InputEdges`. Its fixture classification follows declared output keys, and keyed member edges explicitly require same-key position zero. The existing two- and 253-member fixtures now carry exact checked associations rather than unbound expected sets. Their two-output boundary parents remain supplied premises.

**Model refusal to reader outcome** · `tab:w1b15:history-reader-outcomes`

| Model refusal | Boundary | Required step-2 evidence outcome |
|---|---|---|
| `DuplicateEvent` | Valid history | Invalid duplicate event declaration; conservative refusal of duplicate evidence. |
| `ConflictingEvent` | Valid history | Invalid evidence binding one event identity to conflicting contents. |
| `MissingEvent` | Graph | Incomplete dependency evidence, never a covenant refusal. |
| `MissingOutput` | Valid history | Invalid referenced position after the parent is available. |
| `DuplicateConsumption` | Valid history | Invalid history evidence claiming multiple consumption. |
| `Cycle` | Graph | Invalid cyclic dependency evidence. |
| `LaterOutputConsumed` | Born first | Invalid claimed keyed history at the born-first boundary; the declared later output itself queries as `NotHandle`. |
| `WrongSuccessor` | Continue first | Invalid claimed keyed history at the continuation boundary, preserving input and spender identities. |

These are comparison outcomes for the declared subject and its supplied evidence, not native execution verdicts. `OutputStatus::MissingEvent` is incomplete, `MissingOutput` is invalid after parent retrieval, `WrongKey` is a mismatched key association, and only `NotHandle` establishes a declared non-handle. No four-boundary enum split is needed because the mapping is by refusal variant. The reader must not infer a final or admitted history from any of these test fixtures.


**Review verification.** The corrected code’s covering run `/var/home/tripod/loops/tmp/gate-w20b1-20261112T130201Z.log` finished `== green`, exit 0: 13:02:01Z–13:07:12Z on 2026-11-12, 311 seconds, zero slot wait. It covered vectors, transaction, linker, tapscript, architecture and compiler, 29 package lanes, 17 lint lanes and six checks. Battery walls were setup 0, fmt 3, clippy 7, package tests 239, rustdoc 29, lint 30 and checks 1 seconds. Per-battery start/end/delta counters are retained in that log under its own `tripod-gate-w20b1-20261112T130201Z.service` cgroup, including containers, not attributed from node-wide samples. The supplied slot-governed systemd unit ran with a foreground SSH waiter through its end marker; it was not abandoned or relaunched. The preserved journals confirm all ten projection pins through `tests::engine_base_capture::preserve_engine_base` and canonical publication through `tests::keyed_profile_tests::attestation_canonical_bytes_and_publication_equal_the_opening_base`. Existing reproduction comparisons and vector tests passed; no production source, archive, emitted byte or expected value changed. Fast-check correction rounds applied the formatter’s patches and factored helpers to satisfy the existing line-count and invariant-diagnostic lints; the fourth check passed fmt, clippy and rustdoc in 38 seconds. The documentation commit and final tip are checked after this code run, with their final verdict in the step report.

## Byte-derived history reader · `rec:w1b15:reader`

Step 2 implements (`step:w1b15:reader`) under (`rec:w1b15:adoption`). `packages/vectors/src/keyed_history_reader.rs` and its five helpers reconstruct finite histories from bytes and explicit evidence; the external comparison is in `tests/keyed_history/reader.rs`. The model remains exactly as imported at `0.7.117-dev`. This is a public **conformance-library** boundary, with ADR-026 tracing, not a production chain reader, an admission entry or a new maintained interchange format. No production package depends on it. A complete local membership set establishes neither chain acceptance nor finality.

**Boundary-parent disposition.** The existing two-member and 253-member reproduction subjects remain unchanged. Their supplied member parents spend outputs whose creating transactions are absent. A boundary event name describes a fixture cut; it cannot establish that event as SOURCE. The successor therefore has `Membership::Incomplete`, naming precisely the missing input dependency of each of the two or 253 boundary parents. The harness checks that these missing dependencies belong to exactly the declared boundary events, and no others, and compares each address and vout with the independently decoded parent input. The model's sets are conditional on its SOURCE declarations at those cuts; they are not complete byte-derived answers. Local content addresses, canonical commitment, both member clauses, successor key, input order and the later same-key output's `NotHandle` remain checkable and are checked. The complete 253-SOURCE `wide` history supplies the large complete reconstruction comparison. Step 3's premises and step 4's reports must retain incomplete standing for the two cut subjects; bite 1.6 receives them with completion dependent on their ancestors' evidence. No evidence was invented to make an equality pass.

A sponsor parent's own inputs are outside the history and need not be followed merely to classify the actual output spent by an event. `source` supplies a nonzero-index ordinary sponsor output whose bytes suffice for that classification; `ordinary-edge` spends an earlier event's change output at index 1 and still creates a SOURCE singleton. Neither the fixture's sponsor label nor a graph-local edge determines membership.

**Inputs, surface and permitted calls** · `tab:w1b15:reader-surface`

| Part | Inventory and authority |
|---|---|
| Untrusted input | `Input`: `bytes`, `association`, `keys`, `retrieved`, `boundary_premises`, `assertions`. `Fixture::reader_inputs()` supplies these only, with no declared graph, expected set, construction packet or source/member flag. `Assertions` adds explicit hypothetical inclusion/retrieval presence to that projection; it is not evidence of their truth. |
| Identity and premises | Distinct newtypes `Address` and `Key`, plus `EventId`, `Output`, `Handle`, `Member` and `ByteSource::{Event, Retrieved}`. `retrieved` is bytes by address, with no event label or output selection. Policy readings remain test-local beside the reading for step 3, absent from `Input` and `Reading`. |
| Entry and result | `read(&Input) -> Result<Reading, Refusal>`; immutable `Reading` exposes `memberships`, `live`, `successors`, `relations`, `leaves`, `unexamined`, `obligations`, `boundary_premises` and `query`. Local live means unconsumed in these supplied bytes, not chain unspentness. |
| Completeness and queries | `Membership::{Complete, Incomplete}`, `Missing`; `Query::{Handle, NotHandle, MissingEvent, MissingOutput, WrongKey, UnresolvedKey}`. A non-handle has no record set, never an empty set. |
| Per-relation evidence | `Relation::{BornFirst, ContinueFirst}`, `RelationEvidence`: event/input, content-addressed member and key, exact leaf/control bytes and each fragment's own counters. Two rows per authenticated consumed handle; `LeafEvidence` carries whole-leaf counters once per member through `leaves()`. No producer coverage is accepted. `unexamined()` retains each non-keyed-first-output event's missing set, including empty sets. |
| Remaining premises | `Obligation` names event/input/outpoint and each `ParentFact::{Inclusion, Unspentness, Retrieval, Finality}`. All four remain per-instance external obligations even when assertion presence is supplied. Finality declaration and acceptance rules belong to step 3. |
| Invalid evidence | `Boundary` and `Refusal`, inventoried below; no wrapper converts these into target admission or native covenant execution verdicts. |

The identity helper uses transaction's canonical decoder and serialization and the existing first-party conformance SHA-256 for stripped content addresses (the transaction package does not export its internal hash function). Authentication uses the existing public recipe descriptor and first-party conformance curve reconstruction, target field types and reviewed target definition. The bounded opcode evaluator lives in private `keyed_stack_core`; its instruction kernel and previous expectations remain unchanged. Test-oracle wrappers retain tracing, while the reader uses untraced entries. An unavailable input-program fact refuses explicitly instead of being guessed. These operations are **shared substrate, not independent recurrence evidence**. The reader never calls request/construction, member binding, compiler planning or declaration evaluation to classify or obtain sets. `fold.rs` derives the singleton/union, frontier and successors directly from actual parent outputs. The external test additionally compares canonical bytes, txids, outpoints, witness stacks and output script/value fields against pinned `elements`; that comparison is outside the reader.

**Independence limit.** The reader decides from bytes and evidence, while the model decides from the separately declared graph. The fold, topology, grouping and most precedence mirror the model's algorithms and the same author's interpretation of SOURCE/MERGE. A shared misreading of the recurrence would pass every comparison; step 1's independently written literal expectations and source review, rather than import isolation, guard that risk. The comparison checks byte-to-graph translation and evidence outcomes, not an independently discovered recurrence theorem.

**Classification and precedence.** Identity declarations are grouped in sorted event order; duplicate and conflicting identities, alias collisions and unassociated bytes refuse. Canonical decoding precedes available-parent output-domain checks, duplicate consumption and a complete deterministic topological order. Graph checks precede content-address authentication deliberately, so an untrusted cyclic-reference subject can refuse as a graph without claiming a self-consistent cyclic content-address graph. After hashes and public recipe reconstruction, every actual input output is classified: a recognized key program at position zero is a member; the same program at a later position refuses born-first; ordinary output bytes contribute no records; an input program outside all supplied keys contributes no consumed handle for the key being read. Its status as a handle of an unsupplied key cannot be excluded by these bytes: that exclusion rests on inclusion in one valid history, whose continue-first law would forbid spending it to this successor. An unrecognized witness-v1 first output remains `Query::UnresolvedKey`; it no longer creates `Missing::Classification`, which is removed. Every known member must authenticate its exact leaf/control and satisfy both clauses. Only an event with no member and no missing evidence can receive a SOURCE singleton. A complete MERGE unions completed parent sets and creates no record. A missing parent, assertion or membership is retained per handle, or in `unexamined` for an event without a recognized keyed first output; no event silently loses missing evidence.

Each event and supplied parent is decoded/indexed once per derivation; duplicate retrieval premises reuse the indexed canonical parent. Reordering evidence yields the same reading. Recipe material is reconstructed once per key. Separate permutations and negative subjects deliberately invoke fresh derivations; the 253-member fixture itself is built once per family process.

**Outcomes by boundary and reachability** · `tab:w1b15:reader-outcomes`

| Boundary / standing | Variants | Controls / reachability |
|---|---|---|
| `EvidenceConsistency` | `DuplicateEvent`, `ConflictingEvent`, `ConflictingAlias`, `UnassociatedBytes`, `MissingOutput`, `DuplicateConsumption`, `Cycle`, `ConflictingParent` | Association, available-output, within/across-event consumption, graph and retrieval-conflict subjects. Retrieval is ordered by address, not arbitrary labels; `SponsorOutput` is retired. |
| `ContentAddressing` | `MalformedBytes`, `NonCanonical`, `ContentAddress` | Malformed bytes and stale addresses refuse. `ByteSource` names either the event or retrieved address; successful decode/reencode disagreement remains defensive. |
| `UnsupportedForm` | `UnsupportedForm` | Issuance/peg-in subjects; unsupported proof forms retain the decoder's typed error, not covenant invalidity. |
| `KeyAuthentication` | `Recipe`, `TargetDefinition` | Defensive public-recipe/target failures retain their owner error; no invented reachable malformed recipe offer. |
| `BornFirst` | `LaterBirth` | The later same-key output is consumed beside the accepted (incomplete-membership) boundary twin. |
| `Continuation` | `WrongSuccessor`, `WrongVersion` | Other-key, changed-program, displaced output-zero and version subjects. |
| `CommitmentAuthentication` | `WitnessShape`, `UncommittedLeaf`, `ControlVersion`, `WrongParity`, `ControlRecipe` | One member's witness coordinate changes beside its accepted twin. |
| `ModelEvaluation` | `Model` | Defensive bounded-model failure after canonical authentication; preserve the model's kind. |
| Non-refusal completeness | `Missing::{Parent, Event, Inclusion, Retrieval, Membership}` in `Membership::Incomplete` or `unexamined` | Withheld parent/assertions, retrieval-only known parent, absent associated event and ordinary-first-output missing dependency. No `Classification` remains. |
| Non-refusal query | `Query::{Handle, NotHandle, MissingEvent, MissingOutput, WrongKey, UnresolvedKey}` | Accepted-subject queries plus the explicit empty-key subject; adding an unused key changes no reading. |

`Boundary::KeyAuthentication` names recipe reconstruction; query wrong-key is separately typed rather than collapsed into `Refusal`. A recognized member's other-key successor is a continuation failure. The model's eight variants map exactly as (`tab:w1b15:history-reader-outcomes`) specifies on the corresponding single-fault byte/evidence subjects: duplicate/conflicting event, absent event, absent position, repeated consumption, cycle, later birth and wrong successor. The pinned cycle-plus-wrong-successor subject agrees on graph precedence, including evidence permutation. Cyclic references are deliberately injected before authentication; the public byte mutation separately pins content addressing. No content-address-consistent cycle or admitted invalid transaction is claimed. The general precedence rule is that the reader never stops at unsupplied evidence: any refusal it can evaluate outranks incompleteness, so the model's `MissingEvent` precedence agrees only on single-fault subjects. `mapped` computes the model result on the matching mixed graph, alongside the reader's decisive supplied-member failure; this is not described as one exceptional combination. Reader precedence, which step 4 must preserve, is: sorted identity grouping and alias checks; event decoding and retrieval-address consistency/decoding; available-output domain; duplicate consumption; minimum-ready complete topology; content authentication; then minimum-ready event and input order. Within one member, later birth precedes continuation, continuation precedes commitment (shape, leaf, version, parity, control), and commitment precedes model evaluation. `precedence` pins a wrong-version plus missing-witness subject and two independently bad events in reverse evidence order. The cycle subject pins graph-before-address/clauses. Neither model nor reader ordering is bent to make a multi-fault equality claim.

**Comparison against the model** · `tab:w1b15:reader-comparison`

All comparison helpers below are in `packages/vectors/tests/keyed_history/reader.rs`, invoked by the bounded `keyed_history::history_subject_child` through `keyed_histories_have_bounded_public_subjects`. They see the model only outside the library reader.

| Subject | Model answer | Byte-reader answer / comparison |
|---|---|---|
| `source` | One SOURCE singleton | Equal complete singleton, frontier and queries; ordinary sponsor parent ancestry is not needed. |
| `single` | Singleton through one MERGE | Equal set, consumed/frontier handles and successor. |
| `ordinary-edge` | Two live SOURCE singletons despite a graph-local edge | Equal; the actual index-1 change output is ordinary. |
| `multi` | Two SOURCE records in one MERGE | Equal union, frontier, two successor edges and four local clause rows. |
| `path` | Growing sets through three continuations | Equal every intermediate set, frontier and successor, not only the final total. |
| `two-keys` | Separate keyed singleton paths | Equal disjoint sets and key-sensitive queries. |
| `wide` | Union of 253 explicitly declared SOURCE records | Equal complete union, frontier and all successor edges; 506 member-clause rows. |
| `later` | Output 0 singleton, output 1 not a handle | Equal, with no set for the later same-key output; structural subject, not three-output policy admission. |
| `existing-two` | Conditional union under two boundary SOURCE declarations | Incomplete with precisely two boundary-parent missing input dependencies; four local clause rows and all parent output-1 non-handles checked. |
| `existing-253` | Conditional union under 253 boundary SOURCE declarations | Incomplete with precisely 253 boundary-parent missing input dependencies; 506 local clause rows and all parent output-1 non-handles checked. |

`compare_relation_identities` compares every event, actual input index, keyed member outpoint and clause identity against declared edges joined to the independent decoder, not only a total. `compare` also reverses association offers and requires identical results. The conditional comparison never injects model declarations into reader input. `completeness` proves that a boundary name alone cannot complete a reading. `consistency`, `continuation`, `commitment` and `boundary_subjects` each retain an accepted twin; the negative envelope is test-local, not an alternative constructor format. Missing/duplicate producer assignments and omitted producer clauses remain the earlier sealed-boundary controls, not publicly fabricated request defects.

**Independence and tracing.** `keyed_reader_import_boundary_and_isolated_compile` scans all six reader source files, forbids hidden configurations/modules/path attributes and producer/model spellings, and has rejecting mutants. It compiles those same files with only the recipe/core and explicitly named substrate externs; the external-test model is absent. Exact Cargo dependency fingerprints select the artifacts so different feature-state units cannot be confused. Source/dependency review confirms that `identity` calls canonical decode/serialization/hash; `authentication` calls recipe/curve/model substrate and hashes non-witness programs for the evaluator's `Other` fact; `fold` applies recurrence. The exact extern-path allowlist rejects producer surfaces including `transaction::keyed_inputs`, rather than relying on their names staying in a denylist. This is a spelling/module boundary and isolated-compilation control, not macro expansion, alias/call-graph analysis or a proof of semantic independence; review remains necessary because a permitted substrate crate also exports forbidden higher-level operations. No extra production dependency is added. `keyed_reader_trace_has_one_completion_or_refusal` launches exactly one ignored child under its own timeout, checks the span and exactly one completed or refused event with `refusal_kind`. Only test-oracle wrappers retain the historical `vectors::keyed_stack_model` tracing target; reader member evaluation has no per-member span or event. The capture uses MERGE and asserts their absence. Its model evidence is not native timing.

**Test budget and handoff.** Every new child is bounded at 10 seconds and the existing family process at 60 seconds, with line-anchored exactly-one-test checks and printed duration. The isolated compiler has its own 60-second timeout. No bound is widened. Step 3 receives typed complete/incomplete membership, invalid evidence and exact query non-handle outcomes, per-relation and once-per-leaf local model evidence, non-handle-event missing sets and all four parent premises. Uninterpreted policy readings are supplied separately. It must not turn the two incomplete cut histories into accepted/final readings. Inclusion, unspentness, retrieval and finality are not established by local hashes; relation evidence and native execution remain distinct later discharge requirements.


**Measured offline walls** · `tab:w1b15:reader-walls`

These are process walls including fixture construction, comparison and child startup, not native measurements or a performance claim. Raw successful output and preservation results are retained in `/var/home/tripod/loops/tmp/gate-w20b1-20261112T141842Z.reader-preservation.tar`. All subjects exited zero below their unchanged 10-second bound. The family process took 12.293505 seconds under its 60-second cap; the reader-only compilation took 2.984035 seconds under 60 seconds, and the trace child 0.119342 seconds under 10 seconds. No bound was widened.

| Subject | Wall, seconds |
|---|---:|
| `source` | 0.642751 |
| `single` | 0.682742 |
| `ordinary-edge` | 1.126907 |
| `multi` | 0.990125 |
| `path` | 0.552835 |
| `two-keys` | 0.755915 |
| `wide` | 1.560471 |
| `later` | 0.935554 |
| `existing-two` | 0.472258 |
| `existing-253` | 0.592051 |
| `association` | 2.006840 |
| `reader-negatives` | 1.935363 |

**Verification and corrections.** The code commit's fast check passed fmt (clean), clippy and rustdoc in 82 seconds. Earlier fast rounds applied formatter output and fixed local lint findings; no expectation was regenerated. Three code-gate correction rounds are retained: `/var/home/tripod/loops/tmp/gate-w20b1-20261112T134000Z.log` (626 seconds, red) exposed the isolated compiler's outdated Cargo artifact-directory assumption; `/var/home/tripod/loops/tmp/gate-w20b1-20261112T135935Z.log` (383 seconds, red) then exposed external module-path resolution and split metadata requirements. Exact copied source layout and matching `.rmeta` artifacts fixed those harness failures. `/var/home/tripod/loops/tmp/gate-w20b1-20261112T140851Z.log` (396 seconds, red) passed isolated compilation but found the new mixed-fault subject used an output swap that changed the witness version before its intended program comparison; changing only a program byte fixes the subject, not its expected boundary or the reader. Those failures are not evidence of production defects or native refusals.

The covering code run `/var/home/tripod/loops/tmp/gate-w20b1-20261112T141842Z.log` ran 14:18:42Z–14:26:20Z on 2026-11-12, `== green`, exit 0, 458 seconds with zero slot wait. It covered vectors, transaction, target-elements, target-elements-conformance, architecture and compiler: all 29 package lanes, 17 lint lanes and six checks passed. Battery walls were setup 2, fmt 8, clippy 19, package tests 360, rustdoc 33, lint 34 and checks 2 seconds. The log retains each battery's start/end/delta samples in its own `tripod-gate-w20b1-20261112T141842Z.service` cgroup, including container descendants, not node-wide attribution. All rounds used the supplied slot-governed transient systemd unit with a foreground SSH wait through the end marker; none was abandoned or interpreted as complete after a lost connection.

The retained journals show `tests::engine_base_capture::preserve_engine_base` (all ten exact projection pins) and `tests::keyed_profile_tests::attestation_canonical_bytes_and_publication_equal_the_opening_base` passing. The model source is unchanged. The bounded core's `evaluate`, `fragment` and `leaf` function bodies are byte-identical to the earlier evaluator; the only new fact case explicitly refuses unavailable program evidence. There are no source changes in architecture, realization, compiler, tapscript, linker, transaction or target-elements/conformance, and no existing expected vector or archive changes. Existing vector, reproduction and keyed construction controls passed. The documentation commit is fast-checked and the final tip receives the same full package-set gate; its final-tree verdict is reported with the step.


## Reader review disposition · `rec:w1b15:reader-review`

The per-key classification decision follows (`[A-sec:kpc:interface]`), its reconstruction of consumed handles for the key being read, and (`[A-thm:kpc-interface:membership-correctness]`). Supplied parent bytes decide whether their actual spent output equals that key's authenticated first-output program. An unknown taproot input is outside this key's consumed handles; excluding an unsupplied other key's handle relies on the already-carried inclusion premise, not on asserting its absence from bytes. `reader_review::classification` pins three controls: ordinary taproot funding is complete beside the existing SOURCE; a genuine other-key first output also reads complete when that key is omitted, while the model knowing both keys refuses; adding the other key makes the same bytes refuse at continuation. The latter is an explicit inclusion-premise limit, not an admitted invalid history. There is no contradiction with the paper's conditional theorem and no new covenant law.

| Finding | Closure |
|---|---|
| 1 | `Reading::unexamined` retains missing sets for all events without recognized keyed output zero; `query_and_unexamined` withholds the actual parent beside its ordinary-first-output twin. |
| 2 | Per-key rule above; `Missing::Classification` removed, unresolved output queries retained, inclusion limit tested against the model. |
| 3 | `retrieved: BTreeMap<Address, Vec<u8>>`; `ByteSource::Retrieved` names malformed/misaddressed bytes by address, no label or selected vout. Reader projection and its exclusion lock remain. |
| 4 | Exhaustive `boundary()` and the eight-boundary inventory above; completeness and queries have separate non-refusal rows. |
| 5 | Empty keys yield `UnresolvedKey`, empty memberships/frontier and the non-handle carrier; an unused key leaves `multi` unchanged. |
| 6 | The independence-limit paragraph names the shared algorithms and shared-misreading risk. Package contract wording is narrowed accordingly. |
| 7 | General refusal-over-incompleteness rule and full local precedence; continuation-before-commitment and minimum-ready-event controls. |
| 8 | New README paragraphs are single lines. |
| 9 | Fragment counters are birth (5 operations, peak 3, width 32), continuation (6, 4, 32); whole leaf (12, 4, 32) appears once per member. `counters` compares slices with the public clause encodings. No count is regenerated. |
| 10 | `successors` documents byte-true edges from an associated but unsupplied event that has no membership; retrieval-only event control remains. |
| 11 | Policy is absent from `Input` and `Reading`; test-local policy readings travel beside them for step 3. |
| 12 | Exact substrate-path allowlist plus isolated compilation; `keyed_inputs`, grouped producer imports and an unlisted extern are rejecting mutants. No macro/alias expansion claim. |
| 13 | `mapped`, `domains_and_consumption`, and `cycles_and_clauses` run matching declared mutants for all eight graph-expressible model refusals; the mixed-fault model result is computed too. |
| 14 | Untraced evaluator entries in the member loop; test-only historical target documented and absence captured. Instruction kernel and test expectations unchanged. |
| 15 | Cross-event duplicate consumption, obligation identities from `elements`, address-order retrieval permutation, and the unchanged first member's actual witness are asserted. Label permutation no longer applies: the labels were removed. |
| 16 | Accepted incomplete twin, both hash call sites and all-event incompleteness stated accurately. The original 18-file step exceeded the 9–14 estimate because of the five reader helpers and shared core extraction; realization-owned documentation is deferred to step 3 or the gate packet, where its reader rule and report premises settle. |
| 17 | Final successor's unresolved event set is compared literally with `fixture.boundary_premises`, as well as model records and exact independently decoded missing outpoints. |
| 18 | Distinct `Key` and `Address` newtypes; classification-outpoint expansion is obsolete because the variant is retired. `Parent` carries address and vout; `Membership` carries address and actual input index after position-zero authentication. |
| 19 | Each derivation prints its event/key counts and wall inside the bounded subject. The measurements below separate measured reader work from fixture/comparison/process remainder and retain final-gate evidence. |

`reader-review` is another bounded 10-second subject; it builds its SOURCE and multi-input fixtures once, plus the focused cut and separate-branch controls. All prior subject bytes and expected model sets remain unchanged. The model file is untouched. No production package outside vectors changes and no native execution occurs.


**Previously omitted final-tip evidence.** The pre-review final-tree run was `/var/home/tripod/loops/tmp/gate-w20b1-20261112T142833Z.log`: 2026-11-12 14:28:33Z–14:35:45Z, 432 seconds, zero slot wait, `== green`, exit 0. Its family wall was 10.135061 seconds, distinct from the code-commit run's 12.293505 seconds above. Its preserved journals are `/var/home/tripod/loops/tmp/gate-w20b1-20261112T142833Z.reader-preservation.tar`. These are different observations on the same reader code, not a speed claim. Resource deltas below are microseconds from that gate's own `tripod-gate-w20b1-20261112T142833Z.service` cgroup, including container descendants; “some/full” are the kernel pressure counters, not inferred CPU utilization. Raw start/end samples, memory levels and remaining counters are in the log.

| Battery | Wall s | CPU usage / user / system μs | CPU pressure some / full μs | Memory pressure some / full μs | I/O pressure some / full μs |
|---|---:|---|---|---|---|
| setup | 0 | +2202737 / +957277 / +1245460 | +8181 / +3551 | +0 / +0 | +61 / +59 |
| fmt | 4 | +6104193 / +4639620 / +1464573 | +56794 / +14721 | +0 / +0 | +266 / +264 |
| clippy | 13 | +31938504 / +25447995 / +6490508 | +237831 / +212708 | +0 / +0 | +7262 / +7019 |
| package-tests | 344 | +3127376697 / +3065333645 / +62043053 | +77932602 / +36178237 | +195874 / +92467 | +50091 / +36719 |
| doc | 36 | +40161869 / +26476100 / +13685769 | +18902 / +17030 | +97160 / +97160 | +3836 / +3833 |
| lint-suite | 32 | +109300063 / +94428620 / +14871442 | +54331 / +30990 | +0 / +0 | +4890 / +3912 |
| checks | 2 | +3602609 / +2039990 / +1562619 | +10031 / +7948 | +0 / +0 | +343 / +341 |


**Review verification and wall attribution.** The final reader-and-record tip's covering run `/var/home/tripod/loops/tmp/gate-w20b1-20261112T153434Z.log` ran 2026-11-12T15:34:34Z–2026-11-12T15:39:51Z, `== green`, exit 0, 317 seconds, with zero slot wait. Its bounded family took 7.701056 seconds. These measurements are transcribed after that run; the amendment changes only this evidence record and receives the same full gate. The code commit also passed `/var/home/tripod/loops/tmp/gate-w20b1-20261112T152719Z.log` in 330 seconds, exit 0, zero queue wait, with family wall 9.273395 seconds. Its separate journals remain retained. Raw journals and exact preservation controls are retained beside the log as `.reader-review-preservation.tar`.

The new instrumentation measures the reader calls inside each subject, rather than attributing the historical wall increase to node load or asserting a speedup. The following table partitions each child wall into the sum of measured derivations and the remainder (fixture construction, literal/model comparisons, process startup and output). The recipe column counts offered keys across calls: successful calls reconstruct each once, but early refusals may stop before reconstruction, so it is an upper bound for negative subjects. Prior step-1 timings lacked this instrumentation; their difference cannot be causally apportioned retrospectively. The table does identify the additional work and its current cost. The two accepted-comparison calls deliberately rebuild recipes for the permutation check. All walls are model/offline measurements, not native execution.

| Subject | Child wall s | Reader calls | Offered recipes | Reader wall sum s | Remainder s |
|---|---:|---:|---:|---:|---:|
| `source` | 0.183789 | 2 | 2 | 0.101229 | 0.082560 |
| `single` | 0.252042 | 2 | 2 | 0.100224 | 0.151818 |
| `ordinary-edge` | 0.132947 | 2 | 2 | 0.044683 | 0.088264 |
| `multi` | 0.120314 | 2 | 2 | 0.048263 | 0.072051 |
| `path` | 0.160511 | 2 | 2 | 0.068492 | 0.092019 |
| `two-keys` | 0.481164 | 2 | 4 | 0.087130 | 0.394034 |
| `wide` | 2.105907 | 2 | 2 | 0.158360 | 1.947547 |
| `later` | 0.267612 | 2 | 2 | 0.097604 | 0.170008 |
| `existing-two` | 0.326228 | 2 | 2 | 0.068110 | 0.258118 |
| `existing-253` | 0.549463 | 2 | 2 | 0.259087 | 0.290376 |
| `association` | 0.553923 | 2 | 2 | 0.091281 | 0.462642 |
| `reader-negatives` | 1.296804 | 37 | 37 | 1.091275 | 0.205529 |
| `reader-review` | 1.254849 | 24 | 25 | 1.048261 | 0.206588 |

No timeout was widened. This round's resource deltas are microseconds in its own `tripod-gate-w20b1-20261112T153434Z.service` cgroup, including containers. Raw start/end values, memory levels and all remaining counters are retained in the log; pressure is kernel stall accounting, not utilization.

| Battery | Wall s | CPU usage / user / system μs | CPU pressure some / full μs | Memory pressure some / full μs | I/O pressure some / full μs |
|---|---:|---|---|---|---|
| setup | 0 | +1768953 / +790296 / +978657 | +3645 / +2819 | +0 / +0 | +121 / +121 |
| fmt | 4 | +5064819 / +3998308 / +1066510 | +3478 / +3390 | +0 / +0 | +57 / +57 |
| clippy | 8 | +23180005 / +17677983 / +5502022 | +13607 / +5528 | +0 / +0 | +26075 / +25438 |
| package-tests | 227 | +2787340254 / +2722770213 / +64570040 | +24552261 / +4861621 | +200580 / +127995 | +73046 / +60822 |
| doc | 42 | +44681806 / +30221762 / +14460044 | +650960 / +529411 | +0 / +0 | +10887 / +10876 |
| lint-suite | 33 | +114651938 / +100938945 / +13712992 | +56734 / +17517 | +0 / +0 | +17188 / +15083 |
| checks | 2 | +3566998 / +2006064 / +1560934 | +10740 / +4314 | +0 / +0 | +342 / +342 |

The two review correction gates were `/var/home/tripod/loops/tmp/gate-w20b1-20261112T151034Z.log` (361 seconds, red, exit 1) and `/var/home/tripod/loops/tmp/gate-w20b1-20261112T151828Z.log` (347 seconds, red, exit 1). Both passed the functional reader subjects and failed only the new spelling lock: first it treated a primitive associated path as an external crate; then it treated the local variable `transaction` as a qualified external path. The corrected scanner distinguishes an unqualified identifier from a path before applying the exact allowlist, while still refusing grouped external imports. Its real-source acceptance, all rejecting mutants and the isolated compile pass in the covering green run. Fast checks also applied formatter output and resolved local lint findings; the final code check passed fmt, clippy and rustdoc in 45 seconds. No expected set or byte was regenerated.

The retained green journals show the ten exact projection pins (`tests::engine_base_capture::preserve_engine_base`) and the canonical publication comparison (`tests::keyed_profile_tests::attestation_canonical_bytes_and_publication_equal_the_opening_base`) passing. The model and every source outside vectors are unchanged; the bounded evaluator's opcode-execution body is byte-identical. Existing vector and reproduction tests pass with unchanged expectations. All server rounds used the supplied slot-governed transient unit and a foreground SSH waiter through the recorded end marker, with zero queue wait; none was abandoned or restarted after a lost connection. No native execution was run.


## Reader rules, premises and internal report form · `rec:w1b15:reading`

This is the pre-review record. The report shape, finality acceptance and refusal order below are superseded by (`rec:w1b15:reading-review`); its byte/model evidence remains valid.

Step 3 implements (`step:w1b15:reading`) under (`rec:w1b15:adoption`), rulings 164–165 and (`dec:guide15-exec:reader-evidence`). `packages/vectors/src/keyed_evidence/` adds acceptance/premise helpers and report framing. No classification, recurrence, membership, reader outcome, constructor or byte recipe changes. The paper's reconstruction follows the queried handle's reached dependencies (`[A-sec:kpc:interface]`); therefore a requested complete handle reading depends on that closure, while a whole-supplied-history claim also checks every incomplete handle and nonempty unexamined set. An unrelated missing dependency is retained in a closure report, not silently discharged. The controls exercise both scopes without changing the reader.

**Rules and typed annotation** · `tab:w1b15:reading-rules`

| Rule identity | Shared multi-input membership | Accepted set | Growth statement |
|---|---|---|---|
| `AllSourceRecords` | SOURCE handles 10 and 11, key 42 | `{10, 11}` | Fixed all-record rule grows only along one valid continuation path. |
| `Fixed { asset: [3; 32], threshold: 1011 }` | Same set | `{11}` | Predicate reads immutable SOURCE output 0; A and T are part of identity. T is not a funding floor. |
| `Changing { record: 11, enabled_epoch: 1, disabled_epoch: 2 }` | Same set, observed at epoch 2 | `{10}` | No monotonicity promise. At epoch 1 the same rule identity accepts `{10, 11}`; epoch 2 removes 11. |

`reading::rules` compares the three accepted sets with independently written literals and asserts identical membership. SOURCE annotations are literal values 1010 and 1011 from their creating bytes, not the MERGE's 1020-unit successor. `reading::growth` takes actual prefixes of the once-built `path` fixture: all-record results are `{10}`, `{10}`, `{10,11}`, `{10,11}` at handles 10, 20, 30, 40; the fixed rule gives `{}`, `{}`, `{11}`, `{11}`. This claims nothing across replacement histories, changed premises, changed rule parameters or incomplete readings (`[A-rem:kpc-interface:record-evidence]`), (`[A-cor:kpc-interface:path-reading]`). No general monotonicity checker is exported.

`Amount`, `Asset` and `Epoch` are distinct newtypes; reader `Address`, `Key` and `Handle` stay distinct. `Value::Unknown` is not explicit zero; `AssetValue::Unknown` is not an asset identity. Reasons are `Absent`, `Committed` and `Unsupported`. `Attribution` has only `Unassigned`; no owner is inferred. `Annotation` records the SOURCE handle, creating address and exact creating bytes plus optional explicit value/asset via these enums. It is neither current handle value, record credit nor entitlement. The internal `committed_amount_is_unknown_and_never_explicit_zero_or_an_owner` subject pairs explicit zero accepted by threshold zero with a committed unknown excluded by that predicate, while all-record eligibility remains. It introduces no confidential transaction construction. The internal annotation-guard subjects are deliberately not reader-accepted histories.

**Surface and framing** · `tab:w1b15:reading-surface`

| Part | Inventory / authority |
|---|---|
| Public boundaries | `reading_report(&Input, &Request) -> Result<Report, Refusal>` and `validate_report(&Input, &Request, &Report) -> Result<(), Refusal>`. Each calls the existing reader once; its immutable `Reading` is consumed without modifying any set or classification. |
| Request | Queried handle, `Scope::{HandleClosure, WholeHistory}`, `ReadingFamily::{Reading, Acceptance}`, full `Rule`, separate observed epoch, history/policy `Provenance`, named `PremiseSources`, finality request and optional declaration offer. Policy readings are uninterpreted named caller readings beside the reader, not observations or acceptance parameters. |
| Provenance types | `Deployment`, `PolicyVersion`, `HistoryAnchor`, `EvidenceSource`; these distinguish identities with similar representations. A history anchor and its truth are externally supplied provenance; matching is not chain authentication. |
| Shared schema | `SCHEMA = 1`; `Family::{Birth, Continuation, Recurrence, Reading, Acceptance, Resources}`; separate `PaperId` and `RequirementId`, plus `Trace`. Only reading and acceptance reports are generated/validated in this step. The other four family semantics remain step 4. |
| Subject and references | `Subject` contains handle (including key), creating address and exact complete bytes. `EvidenceReferences` retains all offered event bytes, sorted associations, address-keyed retrieval bytes, assertion presence, boundary names and supplied keys. Witness bytes are retained separately from stripped content identity. |
| Report claims | Required subject/provenance/trace/rule/evidence/assumptions, offline observation boundary, typed outcome, unchanged membership, accepted set, SOURCE annotations, unexamined missing sets and observed/predicted quantities. Required optional fields allow missing-field subjects; they do not make partial reports valid. |
| Outcomes | `Outcome::{Complete, Incomplete, Invalid, NotHandle}` is the common framing vocabulary. This complete-reading entry returns only `Complete`, with explicit scope and finality standing; incomplete/invalid/non-handle requests return typed refusals retaining the reader's details. Step 4 can frame those observations, never turn them into sets. |
| Quantities | `Dimension`, `Quantity`, `Measurement`, `Unit`, `Absence`. Observed record count, accepted-record count and complete subject-byte length have explicit units. Predictions are absent (`NoIndependentPrediction`); operations, stack items, weight, native wall/CPU/stall, validation charge, deployment and package bounds are `NotObservedHere`, never zero. This is framing, not the resources-family proof. |

`validate_report` freshly invokes the reader and re-derives annotations/filter/framing from bytes and request provenance. It trusts no proposed set, total, digest, producer role or other family's verdict. It shares the derivation helper with report generation: the binding comparison is not a second implementation of the acceptance rule. Independently written literal sets, source annotations, paper-label/requirement trace controls and negative subjects guard a common misreading. Tests bind the trace constants to the paper's label definitions and the eight-row trace in (`tab:w1b15:requirements`). Schema 1 and the twelve-dimensional census are pinned literally. No JSON codec, stable descriptor or maintained interchange promise is introduced; test-local transport remains public-input support only.

**Premises and finality** · `tab:w1b15:reading-premises`

| Assumption / requirement | Carrier and control |
|---|---|
| Inclusion, unspentness, retrieval, finality | One `Assumption::Parent` for every exact reader obligation, with its named source. `premise_identities` obtains expected event/input/outpoint identities independently through `elements`, and checks all four facts. Assertion presence never discharges them. Historical inputs need not remain unspent forever; the named observation scope matters. |
| Per-key inclusion limit | `NoUnsuppliedKeyHandle` names the offered keys and inclusion source. The exclusion of a consumed handle of an unsupplied key rests on one valid history, not bytes proving its absence. |
| Missing own closure | Requested completeness refuses `Incomplete { handle, missing }` at reader evidence. The unchanged two-member and 253-member cut subjects retain exactly two/253 unresolved dependencies; no SOURCE boundary name supplies missing ancestry. |
| Unrelated incomplete supplied evidence | Closure reports retain `unexamined` and still name every input's obligations. Whole-history requests refuse either an incomplete other handle or a nonempty unexamined set. The ordinary and keyed event-99 subjects pin both cases beside the complete SOURCE. Concealing unexamined evidence in a report refuses. |
| Finality declaration | `Declaration` names deployment, policy version, history anchor, depth or replacement rule, inclusion and retrieval sources, unspentness-observation scope and the source asserting external acceptance. The library checks matching/nonempty fields, not the truth or prudence of deployment policy. |
| Finality standing | No offer is `Nonfinal`; requiring one refuses `FinalityMissing`. `HypotheticalDeclaration` and `DeploymentDeclaration` are distinct types, carried through `DeclarationOffer` and `Finality::{Hypothetical, Assumed}` without implicit promotion. A hypothetical match exercises the contract but establishes no deployment premise. The report mutant that promotes it to `Assumed` refuses. |
| Matching controls | Matching hypothetical replacement-rule declaration passes. One-coordinate deployment, anchor and policy-version changes refuse separately; empty acceptance/inclusion/retrieval sources, unspentness scope or replacement rule also refuse beside it. No confirmation count or actual deployment datum is selected. |

The actual accepted declaration is owed at activation. Its absence is not a Wave-1 deployment claim or a blocker for publishing this form. `Finality::Assumed` remains an explicitly external premise, not a locally authenticated fact. A requested final reading with incomplete membership refuses the missing membership before considering finality; no matching declaration can complete a cut history.

**Refusals by boundary** · `tab:w1b15:reading-refusals`

| Boundary | Variants / reachability | Named controls |
|---|---|---|
| `ReaderEvidence` | `Reader` preserves the reader's exact owner refusal; `Query` preserves the non-handle/missing/wrong-key query; `Incomplete`, `Unexamined`, `EmptyProvenance(field)` | `completeness`, `conditional`, `premise_identities`, `reading_reports::provenance`; corrupted underlying bytes refuse through `Reader`. |
| `ReaderEvidence` finality | `FinalityMissing`, `FinalityDeployment`, `FinalityAnchor`, `FinalityPolicy`, `FinalityAcceptanceSource`, `FinalityInclusionSource`, `FinalityRetrievalSource`, `FinalityUnspentnessScope`, `FinalityRule` | `reading::finality`, one coordinate at a time beside the hypothetical matching declaration. |
| `ReaderRule` | `SameEpoch`, `UnknownEpoch`, `ToggleRecordMissing` | `rule_refusals` beside the valid two-epoch rule. |
| `Annotation` | `AnnotationMissing`, `AnnotationDecode` retain annotation identity and decoder error. Defensive after a complete fresh reader result; no invented public malformed-annotation entry. | Internal `annotation_internal_missing_and_decode_guards_preserve_owner_errors`; public annotation corruption is separately `AnnotationsMismatch`. |
| `ReportFraming` | `MissingField(field)`; `SchemaMismatch`, `FamilyMismatch`, `SubjectBytesMismatch`, `SubjectIdentityMismatch`, `SubjectHandleMismatch`, `ProvenanceMismatch`, `TraceMismatch`, `RuleMismatch`, `EvidenceMismatch`, `AssumptionsMismatch`, `OutcomeMismatch`, `ObservationMismatch`, `MembershipMismatch`, `UnexaminedMismatch`, `AcceptedMismatch`, `AnnotationsMismatch` | `reading_reports::{missing, contradictions}`, `completeness`, finality-promotion mutant. Every proposed mismatch is checked beside its freshly validated control. |
| `Quantity` | `QuantityMissing`, `QuantityUnit`, `QuantityValue`, `QuantityAvailability`, `QuantityOverflow` | `reading_reports::quantities`; the private wide-integer conversion pairs maximum with maximum-plus-one. On this node, real collection sizes cannot reach the overflow branch; no public gigantic-history subject is claimed. |

`Refusal::boundary()` and `refusal_kind()` are exhaustive. None of these is a covenant-execution refusal. A report cannot add an observation kind outside the singleton offline-reader vocabulary; that is a type invariant, not a dead runtime guard. Its epoch is separately bound and a one-coordinate epoch mismatch refuses. Step-4 semantic validators must not treat this framing pass as their own verdict, and must preserve the reader's precedence at (`rec:w1b15:reader-review`).

**Tracing, deferred documentation and preservation.** Both exported entries are public conformance-library boundaries under the vectors contract, with INFO spans, one distinct completed/refused event and typed refusal kind. Their internal derivation is untraced at this layer; the nested existing reader retains its own boundary event. `reading_and_report_boundaries_have_distinct_single_events` launches exactly one ignored capture child with its own 10-second timeout and checks all four messages and both span names. No per-record span is introduced. The small step-2 item is documented rather than changed: fragment evaluations are computed birth then continuation, but their `Result`s are retained; whole-leaf failure is propagated first, then birth, then continuation. All are the same defensive model-evaluation stage after commitment authentication. No reader source change or new subject is needed.

The realization-owned documentation deferred from step 2 is now (`rule:realization:keyed-reading-premises`); vectors ownership is (`rule:vectors:keyed-reading`). The code/test split uses seven small source modules and three external test modules, rather than hiding the surface, framing and controls in one file; registrations and tracing plus the package records bring the step above the original 8–12-file estimate. It stays entirely within the adopted vectors/tests/contracts scope. The model, reader and all production packages outside vectors remain unchanged, as do every keyed/linked/committed/constructed byte and existing expectation. No native execution or deployment activation occurs.

**Handoff to step 4.** Reading/acceptance report generation, fresh field binding, rule identities, SOURCE annotations, explicit premise/finality standings, complete/incomplete/invalid/non-handle vocabulary, and absent metric dimensions are available. The other family validators must recompute their own claims from bytes and evidence. The two cut reproduction subjects remain incomplete in reports and later native handoff until ancestry evidence exists. Bite 1.6 still owns final-byte policy observations and the final native/member-evidence discharge; local accepted sets establish neither admission nor finality.


**Measured controls and verification.** The code gate `/var/home/tripod/loops/tmp/gate-w20b1-20261112T162056Z.log` ran 2026-11-12T16:20:56Z–16:28:42Z: `== green`, exit 0, 466 seconds, zero slot wait. It covered vectors, realization, transaction, target-elements, target-elements-conformance, architecture and compiler, plus lint and plans. Raw journals are retained in `/var/home/tripod/loops/tmp/gate-w20b1-20261112T162056Z.reading-preservation.tar`. The documentation tip receives the same package gate after this transcription.

| New subject | Process wall s | Bound s | Result |
|---|---:|---:|---|
| `reading-rules` | 1.269755 | 10 | exit 0 |
| `reading-premises` | 1.369697 | 10 | exit 0 |
| `reading-reports` | 1.728189 | 10 | exit 0 |
| `reading-conditional` | 0.302821 | 10 | exit 0 |
| Trace capture | 0.356358 | 10 | exit 0 |
| Whole history family, including existing subjects | 13.700919 | 60 | exit 0 |

These are offline process walls including fixture setup, reader reconstruction, assertions and child startup, not native costs or a speed claim. Each child has its own timeout and prints its wall. The two cut fixtures are each built once in the conditional subject; no bound was widened. Fast-check correction rounds applied the formatter's emitted patches and repaired local lint findings (visibility, constant helpers, documentation, layout and test idioms). The final epoch-identity check passed Clippy and rustdoc in 64 seconds; its formatting patch was applied mechanically before the code gate, whose formatter was clean. No failed functional gate or changed expected byte was hidden.

Both `tests::engine_base_capture::preserve_engine_base` (all ten compiler pins) and `tests::keyed_profile_tests::attestation_canonical_bytes_and_publication_equal_the_opening_base` passed. Existing vector and reproduction controls passed unchanged. The model, reader, architecture, compiler, realization and transaction sources are unchanged; the realization change is its package contract only. There is no regenerated expectation. The 18-file change consists of 14 code/test/registration/tracing files and four documentation files.

The supplied slot-governed systemd unit ran with its original foreground SSH waiter through the end marker; it was neither abandoned nor relaunched. Per-battery deltas below are microseconds in the gate's own `tripod-gate-w20b1-20261112T162056Z.service` cgroup, including containers. They are not node-wide attribution. Raw start/end samples, memory levels and remaining counters are in the retained log; pressure means kernel stall accounting.

| Battery | Wall s | CPU usage / user / system μs | CPU pressure some / full μs | Memory pressure some / full μs | I/O pressure some / full μs |
|---|---:|---|---|---|---|
| setup | 1 | +2064807 / +833007 / +1231800 | +55753 / +6225 | +0 / +0 | +462 / +445 |
| fmt | 6 | +7045672 / +5668811 / +1376862 | +47027 / +7210 | +0 / +0 | +22408 / +22380 |
| clippy | 24 | +54925328 / +47733028 / +7192300 | +308244 / +100310 | +40730 / +40730 | +79903 / +77522 |
| package-tests | 356 | +3624811480 / +3533165528 / +91645952 | +55791321 / +20341449 | +125710 / +91295 | +605604 / +527225 |
| doc | 34 | +43603364 / +30212231 / +13391132 | +12349 / +11041 | +0 / +0 | +18617 / +18213 |
| lint-suite | 43 | +137996598 / +121499195 / +16497403 | +54567 / +19622 | +0 / +0 | +28754 / +26052 |
| checks | 2 | +3702499 / +2073975 / +1628524 | +4140 / +3462 | +0 / +0 | +401 / +389 |


## Reader-rule review disposition · `rec:w1b15:reading-review`

The review changes only vectors report/premise/acceptance code and controls, registrations and these documents. Reader, model and every existing byte and expected set remain unchanged. The adopted phrase “parent-byte provenance” means the record's **creating** transaction bytes; actual input parents keep their existing meaning. Validation binds offers to the request; it authenticates neither external provenance, sources, declaration, rule, epoch and scope nor the truth of assertions. Shared derivation is not a second semantic implementation.

| Finding | Closure and named control |
|---|---|
| 1 | Both bodies retain `incomplete: MissingHandles` beside `unexamined: MissingEvents`. `reading::completeness` pins keyed event 99's literal missing set and refuses concealment as `IncompleteMismatch`. |
| 2 | `Outcome<T>` contains sets only through `Complete(T)`; incomplete, invalid and non-handle variants have none. `Completeness::Report` produces/validates both unchanged cut subjects and a non-handle. Forged complete standing refuses `OutcomeMismatch`; `Required` keeps its missing-evidence refusal. |
| 3 | `Report { header, body }` splits common framing from Reading/Acceptance bodies; only Acceptance holds rule and epoch. The other-family body subject refuses `BodyMismatch`. `Body` and `Observation` are non-exhaustive for step 4; no other family implementation or archived observation is invented. |
| 4 | Required finality refuses `FinalityHypothetical` before coordinate checks. Matching and coordinate controls run under `NotRequested`. In-crate synthetic deployment control alone exercises required acceptance. |
| 5 | Header trace carries requirement 8 and both interface/limitations anchors. Each body's trace is the union of its own requirement row (6 or 7). `reading_reports::trace` extracts each claimed table row's anchors, compares exact label sets and checks their defining paper files, including `10_limitations.tex`. |
| 6 | `HistoryAnchor(String)` is an opaque external chain position. Matching binds only caller provenance, never supplied bytes or a chain fact. |
| 7 | `reading_review::precedence` pins missing closure against absent/mismatched finality, whole-history combined failures against single failures, and a bad rule against missing finality. Required hypothetical precedence is pinned before mismatched deployment. The full order is below. |
| 8 | Annotation fields are `creating` and `creating_bytes`, matching their existing doc meaning. |
| 9 | Changing eligibility is total over memberships: an absent named record is not accepted. Two epochs remain. `reading::growth` adds literals for prefix 30 enabled `{10,11}` and prefix 40 disabled `{10}`, and accepts the earlier prefix without record 11. The test asserts the intended record exists when demonstrating removal. |
| 10 | `DeploymentDeclaration` has a private field and only an in-crate test constructor until an activation route exists. The E0451 privacy doctest and `synthetic_deployment_is_required_and_hypothetical_cannot_promote` cover privacy, required acceptance and promotion refusal. This is synthetic, not deployment data. |
| 11 | Nonempty declaration inclusion/retrieval sources must equal `PremiseSources`; separate `FinalityInclusionMismatch` and `FinalityRetrievalMismatch` subjects sit beside the match. |
| 12 | A matching `Depth(0)` form passes: it means a deployment elects finality at inclusion. This library checks form, not prudence, and selects no count. |
| 13 | `Unknown::Absent` is removed; `Unsupported` documents future target-field variants. SOURCE annotations are creation values, not successor readings or credits; a directly queried SOURCE is the coincident case. |
| 14 | `premise_identities` now covers multi and both event-99 forms. `reading_review::literals` independently assembles exact SOURCE evidence references (including nonempty boundary names and keys) and the ordered full assumption list. README and this record state validation's limits. |
| 15 | Defensive queried-subject lookup uses `SubjectMissing`, separate from missing SOURCE annotation. Its synthetic lookup control refuses missing bytes/association. Quantity overflow is unreachable from real sizes on every supported target with at most 64-bit `usize`, though the wide-integer helper has a maximum-plus-one subject. Refused events carry the wrapped reader's `inner_refusal_kind`, pinned by the capture child and TRACING rows. |
| 16 | The dimension census traverses an exhaustive successor match; a new variant requires a new arm. Quantity interpretation also matches every dimension explicitly. |
| 17 | `WholeHistory { handles, events }` retains all failures, tested with a keyed event and a separate ordinary missing-parent event, plus their single-fault twins. Further controls query the incomplete handle itself and retain the entire aggregate, including its own missing set. An associated but unsupplied unused event lies outside supplied history, with a control. |

**Final report/request shape.** `Header` contains schema, family, subject, provenance, common trace, evidence, assumptions, observation and observed/predicted quantities. `ReadingBody<CompleteReading>` contains family trace, outcome and missing maps. `CompleteReading` contains scope, finality and membership. `AcceptanceBody` adds rule and observed epoch to `ReadingBody<AcceptedReading>`; `AcceptedReading` contains the complete reading, accepted set and SOURCE annotations. `Outcome<T>` is `Complete(T)`, `Incomplete { handle, missing }`, `Invalid(reader_refusal)` or `NotHandle(handle)`. Noncomplete outcomes have no record-set or annotation field and their record-count quantities are absent, not zero. Reading has no acceptance rule or accepted-record quantity. `Completeness::{Required, Report}` is separate from handle-closure versus whole-history scope. Reporting relaxes the queried-handle completeness demand only when finality is not required; required finality still refuses incomplete evidence first. It does not turn a failing whole-history claim into success. Invalid reader results remain typed refusals even in reporting mode, with a corrupted-content control. `Outcome::Invalid` is shared vocabulary for step 4, not a new report-producing route here. Every produced report therefore retains the successful reader's full parent-obligation list.

**Refusal order.** First validate nonempty request provenance, then invoke the unchanged reader whose own order remains at (`rec:w1b15:reader-review`). A demanding closure request next checks its queried handle's query/completeness. For a whole-history request, missing-evidence maps are aggregated including the queried handle itself if incomplete; missing/wrong-key/non-handle query refusals still precede that whole-history claim. A reporting request frames incomplete/non-handle observations instead; a noncomplete observation cannot confer required finality. Defensive subject lookup precedes body assembly. Complete Acceptance then derives SOURCE annotations, checks the two-epoch rule and applies it; Reading has neither annotation nor rule checks. Finality follows: absent offer under Required; hypothetical offer under Required; acceptance-source presence; deployment, anchor, policy version; inclusion and retrieval presence; inclusion and retrieval equality; unspentness scope; nonempty replacement rule. Zero depth is accepted. Successful declarations match the caller's external anchor, not anything authenticated from transaction bytes. The four two-fault controls preserve this order, with aggregate whole-history evidence replacing the former first-failure choice.

Field validation derives first, then checks header schema/family/subject and required components, observation and quantities, body family, acceptance rule/epoch if any, body trace, incomplete/unexamined maps and outcome. Complete payload comparisons check scope/finality, membership, accepted set and annotations in that order. Refusal-to-boundary mapping is exhaustive. `SubjectMissing` is a defensive report-framing refusal, and `WholeHistory` and finality refusals are reader evidence; neither is covenant execution. Missing annotation retains the documented internal grouping of unavailable creating material; no public invalid fixture is invented for it.

**Step-3 final-tip evidence.** `/var/home/tripod/loops/tmp/gate-w20b1-20261112T163100Z.log` ran 2026-11-12T16:31:00Z–16:38:49Z, `== green`, exit 0, 469 seconds, zero queue wait. Its family wall was 16.900047 seconds, distinct from the earlier code gate's 13.700919 seconds. Its preserved journals are the same stem with `.reading-preservation.tar`. New subject walls were rules 1.004042, premises 1.644486, reports 1.991176, conditional 0.260953 and trace 0.275501 seconds; all bounds held. Per-battery start/end/delta counters remain in that gate's own `tripod-gate-w20b1-20261112T163100Z.service` cgroup, containers included. No native result or performance claim follows.


**Correction rounds.** Fast checks repaired formatter output, exhaustive refusal mapping, helper placement/size and test lint findings after the type split. Two attempted checks returned `CHECK-BUSY` (exit 3), launched no battery and were retried only after the existing foreground check ended. The named code commit's final fast check passed with clean formatting, Clippy and rustdoc in 47 seconds (4/6/37 seconds). The first full gate, `/var/home/tripod/loops/tmp/gate-w20b1-20261112T171046Z.log`, ran 17:10:46Z–17:17:49Z, 423 seconds, red, exit 1, zero queue wait: all package tests passed, and both failures were the same unstaged formatter hunk in `reading_review.rs` (the dedicated fmt battery and the lint suite's cargo-fmt lane). The emitted formatter hunk had been applied locally but omitted from the named commit; amending it fixes the tree without changing a subject or expectation. This failure is retained rather than counted as green.


**Review verification.** The corrected code's covering run `/var/home/tripod/loops/tmp/gate-w20b1-20261112T171851Z.log` ran 2026-11-12T17:18:51Z–17:23:07Z, `== green`, exit 0, reported wall 255 seconds, zero queue wait. It covered vectors, realization, transaction, target-elements, target-elements-conformance, architecture and compiler, plus lint and plans. Journals are retained as `/var/home/tripod/loops/tmp/gate-w20b1-20261112T171851Z.reading-review-preservation.tar`. A final direct family-local trace corruption control is added beside the existing header-trace control; it and this documentation receive the same final package gate. This table is the measured code run, not a prediction of that final run.

| Subject | Process wall s | Bound s | Verdict |
|---|---:|---:|---|
| `reading-review` | 0.567608 | 10 | exit 0 |
| `reading-rules` | 0.658737 | 10 | exit 0 |
| `reading-premises` | 1.001418 | 10 | exit 0 |
| `reading-reports` | 1.627795 | 10 | exit 0 |
| `reading-conditional` | 1.216347 | 10 | exit 0 |
| Trace capture | 0.303611 | 10 | exit 0 |
| Whole family, including existing subjects | 10.363905 | 60 | exit 0 |

Each process has its own timeout and prints its duration. These are offline walls including fixture construction, repeated reader calls, report binding and process startup; no causal performance claim or native measurement is made. Both cut fixtures are built once per conditional process, and no bound was widened. The independent literals and exact existing reproduction comparisons pass. `tests::engine_base_capture::preserve_engine_base` reruns all ten pins, and `tests::keyed_profile_tests::attestation_canonical_bytes_and_publication_equal_the_opening_base` reruns canonical publication; both passed. Reader/model and all production packages outside vectors are byte-identical to the step's base; no vector expectation changed.

The supplied gate starts its own slot-governed transient systemd unit; the lane retained the original foreground SSH waiter through its end marker for both rounds, without abandonment or relaunch. The following deltas are microseconds from the gate's own `tripod-gate-w20b1-20261112T171851Z.service` cgroup, including containers, not from node-wide samples. Raw start/end counters, memory levels and remaining fields stay in the log.

| Battery | Wall s | CPU usage / user / system μs | CPU pressure some / full μs | Memory pressure some / full μs | I/O pressure some / full μs |
|---|---:|---|---|---|---|
| setup | 0 | +1892403 / +852950 / +1039453 | +2269 / +2160 | +0 / +0 | +1029 / +1029 |
| fmt | 3 | +4714071 / +3546651 / +1167421 | +3951 / +3164 | +0 / +0 | +91 / +90 |
| clippy | 2 | +2564588 / +1408481 / +1156106 | +4985 / +3893 | +0 / +0 | +1018 / +1019 |
| package-tests | 186 | +2605629487 / +2559994437 / +45635051 | +21095899 / +4712743 | +0 / +0 | +2649 / +982 |
| doc | 34 | +35814120 / +23105214 / +12708906 | +428485 / +413959 | +0 / +0 | +107 / +101 |
| lint-suite | 26 | +96070873 / +85222276 / +10848597 | +102253 / +48352 | +0 / +0 | +309 / +302 |
| checks | 2 | +3397174 / +1809887 / +1587286 | +7518 / +4688 | +0 / +0 | +218 / +194 |


The source-and-documentation gate `/var/home/tripod/loops/tmp/gate-w20b1-20261112T172558Z.log` ran 17:25:58Z–17:30:54Z, reported wall 296 seconds, red, exit 1, zero queue wait. All package tests, formatting, Clippy, rustdoc and lint suites passed; the final label check refused a malformed citation group in this record. The citation is corrected. Final inspection also removed an unrequested conversion of reader errors into invalid reports: both request modes now preserve `Refusal::Reader`, so every report produced here carries the reader's four facts per input. `Outcome::Invalid` remains shared framing for step 4. A reporting-mode corrupted-byte control pins this limit; reader/model code and all existing bytes remain unchanged.


**Final review-tree run.** `/var/home/tripod/loops/tmp/gate-w20b1-20261112T174012Z.log` ran 2026-11-12T17:40:12Z–17:45:32Z, wall 320 seconds, zero slot wait, `== green`, exit 0, over the full requested package set plus lint/plans. This run includes the final whole-history aggregate control (also querying the incomplete handle), the reporting-mode reader-refusal control and the family-local trace mutant. The final code fast check passed in 47 seconds and the documentation fast check in 57 seconds. The whole-history test initially exceeded Clippy's function-size bound; removing its redundant intermediate expected map restored the bound without changing its final expected value. No bounds were widened.

Journals are retained at `/var/home/tripod/loops/tmp/gate-w20b1-20261112T174012Z.reading-review-preservation.tar`. The ten-pin capture and canonical publication comparison both passed; reader/model, other production packages and all pre-existing vector expectations remain unchanged. Subject walls below include fixture construction and repeated report validation, not just the reader. Each subject has its own 10-second timeout; the family has 60 seconds. This transcription is the only change after that run and receives final verification before publication.

| Final subject | Wall s | Verdict |
|---|---:|---|
| `reading-review` | 0.579759 | exit 0 |
| `reading-rules` | 0.543122 | exit 0 |
| `reading-premises` | 1.322257 | exit 0 |
| `reading-reports` | 1.549225 | exit 0 |
| `reading-conditional` | 0.659146 | exit 0 |
| Trace capture | 0.130665 | exit 0 |
| Whole family | 8.542735 | exit 0 |

The unchanged supplied gate (`#!/bin/bash`, `set -o pipefail`) ran through its slot-governed transient unit with the original foreground SSH waiter held until `== green`. Resource deltas below belong to its own `tripod-gate-w20b1-20261112T174012Z.service` cgroup, containers included; raw start/end samples remain in the log. They are not node-wide costs or native measurements.

| Battery | Wall s | CPU usage / user / system μs | CPU pressure some / full μs | Memory pressure some / full μs | I/O pressure some / full μs |
|---|---:|---|---|---|---|
| setup | 1 | +1891458 / +868387 / +1023070 | +3054 / +2514 | +0 / +0 | +76 / +76 |
| fmt | 3 | +5189411 / +4068709 / +1120702 | +2874 / +2764 | +0 / +0 | +46 / +46 |
| clippy | 10 | +23676351 / +18208596 / +5467754 | +10134 / +4892 | +0 / +0 | +5304 / +5303 |
| package-tests | 240 | +2888765667 / +2822475867 / +66289799 | +31502166 / +9093904 | +136695 / +93001 | +15859 / +5389 |
| doc | 32 | +34718367 / +22998273 / +11720095 | +20385 / +20094 | +86675 / +86676 | +34667 / +34666 |
| lint-suite | 31 | +107882764 / +93639692 / +14243071 | +39451 / +22204 | +1 / +0 | +20215 / +19153 |
| checks | 1 | +3361317 / +1926993 / +1434324 | +5424 / +4623 | +0 / +0 | +872 / +871 |
