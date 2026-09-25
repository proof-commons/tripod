# Study: the native mutant ceremony for the seven escape rows and the foreign-subtree successor, and its bites

Status: a design study over the tree at `0.6.296-dev`: the Wave-13 opening record of `0.6.293-dev` at (`rule:phase6:wave13-rulings`) and (`sec:phase6:wave13-findings`), and the three Wave-13 bites landed on `0.6.294-dev`, `0.6.295-dev` and `0.6.296-dev`, over the Wave-13 study of `0.6.292-dev`. It is written because that study's node-bound obligation N1 asks for native mutant refusals of the seven escape rows its conjunct table names and of `successor-under-another-static-subtree`, at their declared boundaries, run by the capture driver on an Elements-provisioned compute node and admitted on the pattern of `T11-089` (`plans/drafts/wave13-gate-and-handoff-study.md:779–787`), and the opening record gave N1 no row because finding 196 states that the compute nodes in service lack an Elements toolchain (`plans/phases/06-state-and-maturity.md:1206`, `plans/phases/06-state-and-maturity.md:1352`). That estate fact no longer holds, so N1's condition holds and, under the wave's own rule that an obligation is minted as work when its condition holds, N1 becomes work. Every figure below is recounted from the source at `0.6.296-dev` or read from the assertion that pins it; nothing here was obtained by running a test, no ceremony was run, and no line of code is changed by this file. It answers the design questions as decisions D1 to D11, leaves three questions with both sides, proposes rulings 118 to 124 and findings 234 to 245 for an amendment to the Wave-13 opening record, proposes rows `T11-148` to `T11-152`, and manufactures four host bites and one node ceremony with their exact shapes. It finds that six of the seven escape rows and the foreign-subtree successor can be staged without a constructor parameter, that `wrong-control-recipe` has no position in the announcement a ceremony could offer, and that the executor adapter needs no change.

## 0. In one breath

The eight rows stand in the negative-half register `STILL_REQUIRED` with native refusals observed at zero. Seven are declared at `ScriptPathRejection` and one, `key-path-spend-attempt`, at `KeyPathRejection`. All eight intend the `LinkedConstructor` carrier and depart from the canonical sponsorless announcement. Five carry the gap that says no emission knob exists, one is a declared wall, one waits on a stager and one waits on a witness of another shape.

The tree already carries most of what a ceremony needs:

- the covenant authenticates the spent program and output zero's program through `TweakVerify`, reading each parity from the witness, so a parity or a program is a position a mutant can write;
- the host continuity tests stage four of these mutants as experimental subjects under placeholder signatures;
- the constructor publishes the metadata leaf's own control recipe, so revealing that leaf needs no derivation of its own;
- the adapter offers exact bytes and records the node's words verbatim, and its reserve funds any program;
- the operator run of record on `0.6.169-dev` is a ceremony of this exact shape on the same pinned tip: six refused offers against one funded coin, then the accepted positive control.

What the tree lacks is node-offerable bytes: stages that produce re-signed mutants, a planner and replayable evidence for the ceremony, a driver, an admission of the capture, a path from an admitted refusal to the `NativeRefusalObserved` standing, and a report line naming the answered rows.

Seven rows can be offered: four in a predecessor-constructor run and three in a successor-constructor run, because §20.5 lists those two target matrices as separate runs. One row, the predecessor's `wrong-internal-key`, needs a second funded coin whose program is derived under another internal key. `wrong-control-recipe` cannot be offered: no byte of the announcement carries the successor's control recipe, and a wrong recipe first matters when the successor is spent, where the commitment rule refuses every non-committing recipe before an opcode runs. Its register entry re-grounds as a wall.

If every refusal binds to its row, the admission moves the census as follows: native refusals observed 0 → 7, native run required 42 → 35, answered 71 → 78, outstanding 135 → 128, and the register 42 → 35. §23's fifth conjunct stays CONDITIONALLY MET on a narrower condition. The card's sixth condition still waits on `wrong-static-subtree`, a first-party row that no run touches.

Four host bites and one ceremony do the work, in this order: the stages, the harness, the driver and its lanes, the ceremony, and the admission. They land before the documents sweep `T11-146`, then `T11-141`, then the gate record `T11-147`.

## 1. The problem, exactly

§23's fifth conjunct reads "metadata leaf, key path, wrong-key, wrong-root, wrong-parity, and wrong-control escapes reject". Its sixteenth reads "every negative row is answered at its declared boundary or remains explicitly outstanding" (`plans/guides/guide_fourteen.md:3593`, `plans/guides/guide_fourteen.md:3604`). The card's own gate lists "no key-path or metadata path bypass remains" and "all wrong-code-subtree vectors reject" as its fourth and sixth conditions (`plans/phases/06-state-and-maturity.md:1456`, `plans/phases/06-state-and-maturity.md:1458`).

The Wave-13 study reads the fifth conjunct as CONDITIONALLY MET. Its condition is that seven escape rows declared at `ScriptPathRejection` or `KeyPathRejection` stand in `STILL_REQUIRED` with no native refusal, "carried by a native mutant run on an Elements-provisioned compute node" (`plans/drafts/wave13-gate-and-handoff-study.md:112`). N1 adds `successor-under-another-static-subtree`, the one row of the sixth card condition that stands at a target boundary (`plans/drafts/wave13-gate-and-handoff-study.md:144`, `plans/drafts/wave13-gate-and-handoff-study.md:781`).

The register states what moves a row: "an observed refusal recorded against an accepted control at exactly the boundary the row declared", never deletion (`packages/vectors/src/maturity_negative_half.rs:80–86`). §1.15's five conditions are decided by `native_refusal_binds_to_row`:

1. the control was accepted;
2. the mutation is attributable;
3. the intended carrier was reached;
4. the observed layer equals the declared boundary;
5. the result is bound to the exact submitted bytes.

The binding function decides three of these from the row's facts, and leaves the control's chain identity and the byte binding to the run (`packages/vectors/src/maturity_evidence.rs:1021–1086`).

§20.5 lists "predecessor constructor" and "successor constructor" among nine target matrices under "Run separately". Each run records a list of facts that ends in "wall time outside canonical report bytes" (`plans/guides/guide_fourteen.md:3341–3379`).

The question has five parts:

- for each row, what produces mutant bytes a node can refuse, and whether that needs a constructor parameter;
- where the node refuses, in what words, and what is recorded when it refuses elsewhere;
- how a run is shaped, planned, driven and recorded;
- how the capture is admitted, and what the admission moves;
- what the gate record can then cite.

**The estate.** Both compute nodes in service carry the pinned Elements toolchain: five binaries, the shallow source and the functional framework, installed under the loop's own toolchain directory. The tip is the one the variable run of record names in its `elementsd-expected-tip` and `handshake-intended-executed-tip` fields, and the capture driver pins it as `EXPECTED_ELEMENTSD_TIP` (`scripts/live-native-maturity-capture.sh:42`). The earlier ceremonies ran with `-minrelaytxfee=0` and `-blockmintxfee=0`, which the adapter passes to its node (`scripts/elements-native-executor.py:2199–2200`) and the corpus pins disclose (`packages/vectors/src/maturity_corpus.rs:228–229`). A ceremony is run by the loop from a clean suite worktree on a node through the capture driver; the capture directory is copied to the host, reviewed as text and admitted by a host bite. This paragraph states the estate, not a reading of the tree. The tree shows the node only after a run, through the admitted capture's handshake fields, which the importer binds to its pins.

## 2. Evidence tables

### 2.1 The eight rows

| Row | Matrix declaration | Register gap | Host evidence at this base |
|---|---|---|---|
| §16.5 `wrong-internal-key` | layer `LinkedConstructorProgram`, locator `ControlBlock`, `ScriptPathRejection`, carrier `LinkedConstructor`, constructibility class `ExternalReportSubjectMismatch` (`packages/vectors/src/maturity_safety.rs:1977–1985`) | `ProgramDerivationHasNoKnobForTheFault`: "no caller-chosen key reaches it" (`packages/vectors/src/maturity_negative_half.rs:422–428`) | `StateInternalKeyPolicy::new` refuses every key but the NUMS key (`packages/tapscript/src/state_constructor.rs:584–600`), pinned by `nums_is_recomputed_from_public_uncompressed_generator` (`packages/tapscript/src/tests/state_constructor_tests.rs:567–583`) |
| §16.5 `wrong-control-block` | layer `WitnessProof`, locator `ControlBlock`, `ScriptPathRejection`, `LinkedConstructor`, `ExternalReportFailed` (`packages/vectors/src/maturity_safety.rs:1986–1994`) | `RefusalIsProgramGeneric`: a non-committing block "is refused by the commitment rule before an opcode runs" (`packages/vectors/src/maturity_negative_half.rs:429–435`) | `replaced_control_block_is_refused` flips control-block bytes 0 and 33 (`packages/vectors/src/maturity_continuity.rs:4120–4150`); `replaced_predecessor_prefix_is_refused` flips witness item 5 and files it under this row (`packages/vectors/src/maturity_continuity.rs:4185–4214`) |
| §16.5 `metadata-leaf-selected-for-execution` | `WitnessProof`, `ControlBlock`, `ScriptPathRejection`, `LinkedConstructor`, `ExternalReportFailed` (`packages/vectors/src/maturity_safety.rs:2040–2048`) | `MutationStagingOwedOnTheLinkedCandidate` (`packages/vectors/src/maturity_negative_half.rs:436–442`) | the leaf is a metadata push, an empty push and `Verify`, which aborts over every admitted stack (`packages/tapscript/src/state_constructor.rs:310–332`; `metadata_leaf_ignores_arbitrary_admitted_initial_stack_contents`, `packages/tapscript/src/tests/state_constructor_tests.rs:843–867`); `the_metadata_leaf_cannot_be_declared_static` (`packages/linker/src/tests/state_taptree_tests.rs:360`) |
| §16.5 `key-path-spend-attempt` | `WitnessProof`, `WitnessStack`, `KeyPathRejection`, `LinkedConstructor`, `ExternalReportFailed` (`packages/vectors/src/maturity_safety.rs:2049–2061`) | `NoAdmittedShapeCarriesTheFault` (`packages/vectors/src/maturity_negative_half.rs:443–449`) | `key_path_policy_retains_the_residual_without_claiming_impossibility` (`packages/tapscript/src/tests/state_constructor_tests.rs:585–598`) |
| §16.6 `successor-under-another-static-subtree` | `LinkedConstructorProgram`, `LinkedProgram`, `ScriptPathRejection`, `LinkedConstructor`, `ExternalReportSubjectMismatch` (`packages/vectors/src/maturity_safety.rs:2115–2123`) | `ProgramDerivationHasNoKnobForTheFault`: "nothing here emits a bundle under a subtree supplied from outside" (`packages/vectors/src/maturity_negative_half.rs:489–495`) | `cross_bundle_successors_refuse_output_with_valid_control` (`packages/vectors/src/maturity_continuity.rs:3440–3469`) over `substituted_successor` (`packages/vectors/src/maturity_continuity.rs:2815–2865`) |
| §16.6 `wrong-internal-key` | `LinkedConstructorProgram`, `ControlBlock`, `ScriptPathRejection`, `LinkedConstructor`, `ExternalReportSubjectMismatch` (`packages/vectors/src/maturity_safety.rs:2124–2132`) | `ProgramDerivationHasNoKnobForTheFault` (`packages/vectors/src/maturity_negative_half.rs:496–502`) | none |
| §16.6 `wrong-parity` | `LinkedConstructorProgram`, `ControlBlock`, `ScriptPathRejection`, `LinkedConstructor`, `ExternalReportFailed` (`packages/vectors/src/maturity_safety.rs:2133–2141`) | `ProgramDerivationHasNoKnobForTheFault`: "no position on the candidate at which a wrong parity could be written" (`packages/vectors/src/maturity_negative_half.rs:503–509`) | `replaced_successor_prefix_is_refused` flips witness item 0 and files it under this row (`packages/vectors/src/maturity_continuity.rs:4296–4326`); `authentication_rejects_every_metadata_byte_and_wrong_parity` answers a wrong parity with the curve-relation failure (`packages/tapscript/src/tests/state_announcement_tests.rs:625–639`) |
| §16.6 `wrong-control-recipe` | `LinkedConstructorProgram`, `ControlBlock`, `ScriptPathRejection`, `LinkedConstructor`, `ExternalReportFailed` (`packages/vectors/src/maturity_safety.rs:2151–2159`) | `ProgramDerivationHasNoKnobForTheFault` (`packages/vectors/src/maturity_negative_half.rs:517–523`) | none |

The recount across these eight rows:

- **Boundaries.** Seven are declared at `ScriptPathRejection` and one at `KeyPathRejection`.
- **Common facts.** All eight intend `LinkedConstructor`, name the constructibility relation, and depart from `SponsorlessAnnouncement` (`packages/vectors/src/maturity_safety.rs:1281–1297`).
- **Gaps.** Five carry the no-knob gap; one each carries the program-generic wall, the staging gap and the missing-shape gap.
- **The shared name.** Both sides' `wrong-internal-key` share a name, which is why the register keys on the table and the name together (`packages/vectors/src/maturity_negative_half.rs:55–64`).
- **Row 5 and N1.** The Wave-13 study's row 5 names the first four rows above, the successor's `wrong-internal-key`, `wrong-parity` and `wrong-control-recipe`. N1's eighth row is `successor-under-another-static-subtree`.
- **Boundary admission.** `boundary_admits` accepts `LinkedConstructorProgram` and `WitnessProof` changes at `ScriptPathRejection`, and both at `KeyPathRejection` (`packages/vectors/src/maturity_safety.rs:1109–1125`). Every declaration above is therefore reachable.

The register's gap census is not pinned per member; `the_gap_census_accounts_for_every_row` pins only the sum 42 and the absence of `TargetRunNotYetPlanned` (`packages/vectors/src/maturity_negative_half.rs:749–765`). Recounted from `STILL_REQUIRED` at `packages/vectors/src/maturity_negative_half.rs:286–596`, the members stand at:

| Member | Rows |
|---|---|
| `MutationStagingOwedOnTheLinkedCandidate` | 19 |
| `NoAdmittedShapeCarriesTheFault` | 5 |
| `ProgramDerivationHasNoKnobForTheFault` | 9 |
| `RefusalIsProgramGeneric` | 2 |
| `NoPublishedRelationFilesTheObservation` | 7 |
| total | 42 |

### 2.2 `extra-escape-leaf` and the conjunct's words

`extra-escape-leaf` is a separate §16.5 row. Its matrix declaration:

- **layer** `StaticConstructorSchema`;
- **locator** `StaticSubtreeLeaf`;
- **boundary** `ConstructorDerivationRejection` (`packages/vectors/src/maturity_safety.rs:2031–2039`), which admits only a schema change (`packages/vectors/src/maturity_safety.rs:1067–1071`).

Its standing is first-party required. It is carried beside `wrong-static-subtree` under `TheOwnerHasNoRetainedObjectToCompareTheOfferAgainst` (`packages/vectors/src/maturity_first_party.rs:3400–3403`).

It is not declared at a script-path boundary, so it stays outside N1 on the matrix's own declaration. `observed_boundary` maps no target verdict to `ConstructorDerivationRejection` (`packages/vectors/src/observed_boundary.rs:59–88`). A node refusal of such a mutant could therefore only stand as a refusal at an unexpected boundary. Its owner is a first-party case.

The conjunct names six words, not rows. The table below is this study's reading of the words onto the matrix.

| Word | Rows | Standing at `0.6.296-dev` |
|---|---|---|
| metadata leaf | `metadata-leaf-selected-for-execution`; `metadata-leaf-duplicated`; `spendable-metadata-leaf`; `metadata-leaf-missing` | native run required; first-party discharged (`packages/vectors/src/maturity_first_party.rs:2735–2744`); first-party discharged (`packages/vectors/src/maturity_first_party.rs:2755–2764`); first-party required under `TheValueIsDerivedNotAccepted` (`packages/vectors/src/maturity_first_party.rs:3441–3444`) |
| key path | `key-path-spend-attempt` | native run required |
| wrong-key | both sides' `wrong-internal-key` | native run required |
| wrong-root | `successor-under-another-static-subtree`; `wrong-static-subtree` | native run required; first-party required (`packages/vectors/src/maturity_first_party.rs:3400–3403`) |
| wrong-parity | `wrong-parity` | native run required |
| wrong-control | `wrong-control-block`; `wrong-control-recipe`; §16.11 `control-block-from-another-program` | native run required; native run required; native run required under `RefusalIsProgramGeneric` (`packages/vectors/src/maturity_negative_half.rs:582–588`) |

### 2.3 The announcement a mutant departs from

`witness` accepts one input, one output and exactly nine witness items, and checks the first seven widths against `declared_widths` (`packages/vectors/src/maturity_continuity.rs:584–620`). The schedule changes only item 4's width: `STATE_METADATA_BYTES` under the whole schedule and `STATE_METADATA_VARIABLE_BYTES` under the variable one. The archived whole-schedule witness carries a 1,286-byte leaf and a 65-byte control block, one path element (`packages/vectors/src/maturity_continuity.rs:3653–3658`). The node-free placeholder layout names each item's source (`packages/vectors/src/maturity_continuity.rs:2733–2748`).

| Item | Bytes | Content | Read by |
|---|---|---|---|
| 0 | 1 | the successor output key's parity prefix | successor reconstruction's `authenticate`: "The witness supplies only parity; the coordinate comes from introspection" (`packages/tapscript/src/state_announcement.rs:469–476`, `packages/tapscript/src/state_announcement.rs:571–592`) |
| 1 | 4 | the successor representation nonce | successor metadata leaf |
| 2 | 8 | the announced cycle | lead window and copy-through |
| 3 | 32 | the static subtree root | both authentications; retained for the successor (`packages/tapscript/src/state_announcement.rs:477–481`) |
| 4 | metadata width | the predecessor's canonical metadata | metadata authentication |
| 5 | 1 | the predecessor output key's parity prefix | metadata authentication of input zero (`packages/tapscript/src/state_announcement.rs:607–617`) |
| 6 | 64 | the operator signature | operator authorization |
| 7 | leaf | the announcement leaf | the commitment rule |
| 8 | control | the predecessor's control recipe for that leaf | the commitment rule |

`authenticate` introspects input zero's or output zero's program and checks witness version one by equality. It then concatenates the witnessed parity with the introspected coordinate, hashes the metadata with the retained root under the Elements tags and the `InternalKey` symbol, and ends in `TweakVerify` (`packages/tapscript/src/state_announcement.rs:457–483`). The successor clause first checks output zero's asset and amount by `EqualVerify` (`packages/tapscript/src/state_announcement.rs:571–592`).

The constructor publishes the metadata leaf's own recipe: `control_recipe` for `StateLeafRole::MetadataCommitment` takes the static root as its one sibling (`packages/tapscript/src/state_constructor.rs:983–1014`). It also publishes `leaf_program`, `merkle_root`, `parity` and `output_program` (`packages/tapscript/src/state_constructor.rs:930–965`).

`StateCurveCapability::output_key` takes any internal key and merkle root (`packages/tapscript/src/state_constructor.rs:27–32`). `state_output_program_at_nonce` re-admits its policy key through `StateInternalKeyPolicy::new` before committing (`packages/tapscript/src/state_constructor.rs:1141–1152`).

### 2.4 What the tree carries, component by component

| Component | Carries | Lacks |
|---|---|---|
| host mutants | four of the rows staged as experimental subjects over coherent node-free sources, each with a designated 64-byte placeholder that "establishes neither authorization nor target acceptance" (`packages/vectors/src/maturity_continuity.rs:1340–1345`, `packages/vectors/src/maturity_continuity.rs:2723–2783`); the helpers `fixed_projection` and `prefix` are module-private (`packages/vectors/src/maturity_continuity.rs:757`, `packages/vectors/src/maturity_continuity.rs:1170`) and `changed_stack` and `changed_program` are test-module helpers (`packages/vectors/src/maturity_continuity.rs:2620–2660`) | a public stage; a signed mutant; a metadata-leaf reveal; a key-path witness; a foreign-key coin |
| planner | `MaturityAnnouncementPlanner` over the three steps `ANNOUNCEMENT_STEPS` (`packages/vectors/src/maturity_native.rs:47–51`, `packages/vectors/src/maturity_native.rs:431–447`), funding checked coin by coin (`packages/vectors/src/maturity_native.rs:623–675`), the signed submission (`packages/vectors/src/maturity_native.rs:869–900`), `expected_layer` naming only the sponsorless subject (`packages/vectors/src/maturity_native.rs:57–70`), `MaturityNativeEvidence::from_transcript` (`packages/vectors/src/maturity_native.rs:280–343`) | mutant steps; `completed_transcript` requires exactly three exchanges (`packages/vectors/src/maturity_native.rs:556–566`) |
| precedent ceremony | the operator ceremony: eleven steps with six refused offers against one funded coin before `operator-positive` (`packages/vectors/src/maturity_operator.rs:52–79`), its own declaration function (`packages/vectors/src/maturity_operator.rs:228–235`), its own test target and driver (`packages/vectors/tests/maturity_operator.rs:1–17`, `scripts/live-native-operator-capture.sh:12`) on the same pinned tip (`scripts/live-native-operator-capture.sh:36`); its admitted details include `mandatory-script-verify-flag-failed (Witness program hash mismatch)` and `(Invalid Schnorr signature)` (`packages/vectors/tests/maturity_operator.rs:963–1009`) | — |
| test support | `CeremonyId::Report` with a per-target test name (`packages/vectors/tests/common/mod.rs:146`, `packages/vectors/tests/common/mod.rs:769–780`; `packages/vectors/tests/maturity_native.rs:37`, `packages/vectors/tests/maturity_native.rs:176–177`); every submission under `Report` is auxiliary (`packages/vectors/tests/common/mod.rs:477`); `mutation_fact` keys on bare step names across ceremonies (`packages/vectors/tests/common/mod.rs:525–590`), and the writer refuses a mutation outside the refusal role (`packages/vectors/tests/common/mod.rs:1234–1239`); the timing sidecar has three fields (`packages/vectors/tests/common/mod.rs:855–858`) | — |
| executor adapter | a submission is exact bytes and nothing else (`packages/target-elements-conformance/src/protocol.rs:2160–2172`); `submit` offers them to `testmempoolaccept`, returns a script verdict at either prefix without a block, settles key path from the bytes, and retries only a non-script refusal at consensus (`scripts/elements-native-executor.py:4584–4664`); the response carries the node's words "verbatim and unmapped" (`packages/target-elements-conformance/src/protocol.rs:2896–2897`); `pay` funds any program from the reserve an issuance retains (`scripts/elements-native-executor.py:3815–3864`, `scripts/elements-native-executor.py:3872–3950`) | nothing a ceremony here needs (D4) |
| capture driver | usage, argv and environment contract (`scripts/live-native-maturity-capture.sh:4–36`); the pinned tip and a census of one test, one ceremony and no setup (`scripts/live-native-maturity-capture.sh:42–48`); the cargo invocation (`scripts/live-native-maturity-capture.sh:388–400`); the census refusals (`scripts/live-native-maturity-capture.sh:434–438`); the test name (`scripts/live-native-maturity-capture.sh:447–448`); phase walls printed to standard error only (`scripts/live-native-maturity-capture.sh:58–69`) | a second ignored test in `maturity_native` fails its one-of-one census |
| corpus importer | pins (`packages/vectors/src/maturity_corpus.rs:233–245`, `packages/vectors/src/maturity_corpus.rs:348–394`); four literals naming the target, argv, ceremony and test (`packages/vectors/src/maturity_corpus.rs:829–839`, `packages/vectors/src/maturity_corpus.rs:909`, `packages/vectors/src/maturity_corpus.rs:1008–1009`, `packages/vectors/src/maturity_corpus.rs:1565`); a three-operation grammar (`packages/vectors/src/maturity_corpus.rs:1502–1508`); replay through `MaturityNativeEvidence` (`packages/vectors/src/maturity_corpus.rs:1477–1488`); one refusal per layer (`packages/vectors/src/maturity_corpus.rs:399–428`) | a mutant roster; refusal records; a target pin |
| evidence plan | `MaturityNativeRefusal` with its eight facts (`packages/vectors/src/maturity_evidence.rs:878–987`); the binding (`packages/vectors/src/maturity_evidence.rs:1061–1107`); `from_native_refusal` (`packages/vectors/src/maturity_evidence.rs:1257–1263`); two admitted corpora read in derivation (`packages/vectors/src/maturity_evidence.rs:2009–2012`); every target-boundary row filed `NativeRunRequired` (`packages/vectors/src/maturity_evidence.rs:2385–2389`, `packages/vectors/src/maturity_evidence.rs:2421–2438`) | a path from an admitted refusal to a standing |
| safety report | schema 1 (`packages/vectors/src/maturity_report.rs:54`); sixteen census buckets (`packages/vectors/src/maturity_report.rs:833–870`); a `witness` block over first-party discharges only (`packages/vectors/src/maturity_report.rs:883–900`); `waiting` and `report_layer` blocks (`packages/vectors/src/maturity_report.rs:903–949`, `packages/vectors/src/maturity_report.rs:1449–1480`); a clause figure `native_refusals_witnessed` (`packages/vectors/src/maturity_report.rs:952–970`) | a line naming a native refusal |
| Meson lanes | the v2-r7 and v2-r8 capture contracts (`meson.build:928–952`) | the maturity and operator capture contracts |

### 2.5 The census at this base

`the_census_figures_are_unmoved_by_the_binding_readers` pins the figures (`packages/vectors/src/maturity_evidence.rs:3775–3802`):

| Bucket | Figure | Answered |
|---|---|---|
| rows | 206 | — |
| first-party discharged | 40 | yes |
| native acceptance observed | 1 | yes |
| root-history observed | 16 | yes |
| public-recovery observed | 14 | yes |
| native refusal observed | 0 | yes |
| native declared-boundary observed | 0 | yes |
| first-party required | 43 | no |
| native run required | 42 | no |
| report-layer required | 15 | no |
| outstanding under a typed non-answer | 35 | no |
| native refusal at an unexpected boundary | 0 | no |
| answered | 71 | — |

`native_declared_boundary_observed` counts "positive rows an admitted run answered at their declared boundary" (`packages/vectors/src/maturity_evidence.rs:1380–1384`). The standing it counts is minted only for the sponsorless row's historical relay refusal (`packages/vectors/src/maturity_evidence.rs:1168–1176`, `packages/vectors/src/maturity_evidence.rs:2071–2076`). A mutant refusal moves `native_refusal_observed` instead, and only through `from_native_refusal` (`packages/vectors/src/maturity_evidence.rs:1161–1167`).

The figures are also pinned by four other tests:

- `the_census_buckets_are_the_figures_the_inputs_recompute`: native run required equals the target-boundary rows less one, native refusal observed is 0, and the partition sum omits the refusal bucket (`packages/vectors/src/maturity_evidence.rs:3089–3169`);
- `required_rows_remain_outstanding_after_the_admitted_run`: the answered sum omits it too (`packages/vectors/src/maturity_evidence.rs:3283–3307`);
- `the_positive_row_names_the_accepted_run_and_its_disclosed_environment` (`packages/vectors/src/maturity_evidence.rs:3848`);
- `the_register_length_is_the_figure_the_plan_recomputes`, which holds `STILL_REQUIRED` at 42 and equal to the bucket (`packages/vectors/src/maturity_negative_half.rs:772–786`).

In the safety report:

- `the_completeness_is_partial_at_this_tip` sums the answered figure without the refusal bucket (`packages/vectors/src/maturity_report.rs:1688–1733`);
- `the_witness_stands_on_the_discharged_rows_and_names_what_it_is_owed` pins `native_refusals_witnessed` at 0 (`packages/vectors/src/maturity_report.rs:1975–2009`).

The safety matrix itself holds no standing: "Nothing here is evidence" (`packages/vectors/src/maturity_safety.rs:60–66`). Its relation standings of 14, 102 and 90 do not move with any observation (`packages/vectors/src/maturity_safety.rs:4446–4461`).

### 2.6 The conjunct and the card conditions at this base

| Item | Standing the Wave-13 study reads | What this base shows |
|---|---|---|
| §23 conjunct 5 | CONDITIONALLY MET on seven rows in `STILL_REQUIRED` | as read; the host evidence it cites for `metadata-leaf-missing` is a first-party-required row, not a discharge (F5) |
| card condition 4 | CONDITIONALLY MET, as conjunct 5, with the key-path residual retained | as read |
| card condition 6 | CONDITIONALLY MET, four rows answered and `successor-under-another-static-subtree` waiting | three rows are answered; `wrong-static-subtree` is first-party required, not answered (F6) |

### 2.7 Test counts, before

| File | Tests |
|---|---|
| `packages/vectors/src/maturity_native.rs` | 32 |
| `packages/vectors/tests/maturity_native.rs` | 10, one ignored |
| `packages/vectors/src/maturity_corpus.rs` | 42 |
| `packages/vectors/src/maturity_evidence.rs` | 22 |
| `packages/vectors/src/maturity_negative_half.rs` | 7 |
| `packages/vectors/src/maturity_report.rs` | 21 |
| `packages/vectors/src/maturity_safety.rs` | 20 |
| `packages/vectors/src/maturity_continuity.rs` | 58 |

No tracked file names a mutant stage module, a mutant ceremony test target, a mutant capture driver or a mutant run of record.

### 2.8 Findings this base holds

F1. Finding 196 states that the compute nodes in service lack an Elements toolchain (`plans/phases/06-state-and-maturity.md:1206`). Both nodes now carry the pinned toolchain, so N1's condition, as the opening record states it, holds (`plans/phases/06-state-and-maturity.md:1352`). This is an estate fact. The tree binds it only after a run, through the capture's handshake fields.

F2. Neither admitted maturity run records a duration. Each run's timing sidecar is 49 bytes (`packages/vectors/src/maturity_corpus.rs:95`, `packages/vectors/src/maturity_corpus.rs:382`), written as three lines — a schema, a ceremony id and a status — at `packages/vectors/tests/common/mod.rs:855–858`, and admitted only as those three exact lines (`packages/vectors/src/maturity_corpus.rs:1563–1567`). The variable run's `RUN-REPORT` has no timing line (`packages/vectors/fixtures/maturity-variable-run-of-record/RUN-REPORT`). The driver prints each phase's wall to standard error only (`scripts/live-native-maturity-capture.sh:58–69`). §20.5 asks every run to record "wall time outside canonical report bytes" (`plans/guides/guide_fourteen.md:3379`).

F3. The tree contradicts three register grounds:

- **`wrong-parity`.** Its ground says no candidate position carries a wrong parity. Witness item 0 is that position, and `replaced_successor_prefix_is_refused` writes it under this row.
- **`successor-under-another-static-subtree`.** Its ground says nothing emits a bundle under an outside subtree. The public link of the `Second` deployment emits one, whose root differs (`packages/vectors/src/maturity_continuity.rs:3379–3437`, `packages/vectors/src/maturity_continuity.rs:3440–3469`).
- **`wrong-control-block`.** Its wall ground describes a non-committing item 8. The tree's own host test stages item 5 under the same row (section 2.1).

F4. `wrong-control-recipe`'s ground names a knob. The announcement, however, carries no byte of the successor's control recipe: its nine items and one output are those of section 2.3, and the successor's parity is item 0, which belongs to `wrong-parity`. A recipe is read only when the successor is spent. There, every recipe that does not commit to the successor's output key draws the commitment rule's verdict before an opcode runs — the property `RefusalIsProgramGeneric` names (`packages/vectors/src/maturity_negative_half.rs:164–173`).

F5. The Wave-13 study's row 5 cites `metadata-leaf-missing`, `metadata-leaf-duplicated` and `spendable-metadata-leaf` at `packages/vectors/src/maturity_first_party.rs:2737`, `:2757` and `:3443`, in that order. At this base those lines hold `metadata-leaf-duplicated` and `spendable-metadata-leaf`, both discharged, and an assertion that `metadata-leaf-missing` is carried under `TheValueIsDerivedNotAccepted` (`packages/vectors/src/maturity_first_party.rs:3441–3444`). A carried row is first-party required, not a rejection.

F6. The Wave-13 study's card-condition-6 row counts `wrong-static-subtree` as answered at `packages/vectors/src/maturity_first_party.rs:2827` and `:3402`. At this base `:2827` is `stale-constructor-from-another-bundle`, and `:3402` asserts that `wrong-static-subtree` and `extra-escape-leaf` are carried under `TheOwnerHasNoRetainedObjectToCompareTheOfferAgainst`. The row is first-party required.

F7. A native mutant refusal cannot move `native_declared_boundary_observed`, which counts positive rows only (section 2.5).

F8. Neither the maturity driver's contract `scripts/test-live-native-maturity-capture.sh` nor the operator driver's is a Meson lane (`meson.build:928–952`). A capture contract that is not a lane passes only when somebody runs it.

F9. Under the `Report` ceremony, `CeremonyCaptureFacts::from_capture` asks `mutation_fact` about every step name (`packages/vectors/tests/common/mod.rs:388`). `mutation_fact` maps the bare name `key-path-spend-attempt` to the live `KeyPathEscape` mutant over a three-item control (`packages/vectors/tests/common/mod.rs:557–568`). `request_role` files every submission under `Report` as auxiliary, and the writer refuses a mutation outside the refusal role (`packages/vectors/tests/common/mod.rs:477`, `packages/vectors/tests/common/mod.rs:1234–1239`). A maturity step named after that row would therefore fail to write its capture.

F10. The operation-submission path consults no failure-class table. The adapter returns the node's reason as `observed_detail` (`scripts/elements-native-executor.py:4638–4648`), and the table at `scripts/elements-native-executor.py:1169–1241` serves the primitive-case path. `Witness program hash mismatch` has no table entry, yet the operator run of record holds it verbatim (`packages/vectors/tests/maturity_operator.rs:980–985`).

F11. `native_refusal_binds_to_row` compares the control, the boundary, the site's subject and the carrier. It does not compare the row's declared mutation layer (`packages/vectors/src/maturity_evidence.rs:1061–1086`). Two rows that share control, boundary, locator subject and carrier bind the same record — for example §16.5's `wrong-internal-key` and `wrong-control-block`. A refusal's row is therefore a fact its record must carry, not one the binding infers.

F12. Under the whole-metadata schedule the relay refuses the announcement's witness before any script runs, and the historical run records `bad-witness-nonstandard` (`packages/vectors/src/maturity_corpus.rs:364–366`). The adapter retries a non-script refusal at consensus (`scripts/elements-native-executor.py:4650–4658`). Once the mempool has named a non-script reason, it files the block's refusal as `consensus_rejection_before_script` (`scripts/elements-native-executor.py:4674–4722`). A script-invalid mutant under that schedule would therefore be recorded before its script.

## 3. Decisions

**D1. Seven rows are staged by writes at spend-time positions of the linked candidate, or of a coin the ceremony funds. None needs a constructor parameter. `wrong-control-recipe` has no spend-time position in the announcement and leaves N1's set.**

The ruling's test is whether a write at a spend-time position produces the row's fault. Two facts answer it for all eight rows:

- The covenant reads each program through a parity the witness supplies and a coordinate it introspects (section 2.3).
- The matrix's `ControlBlock` locator names "the control block, control recipe, internal key, parity, or leaf version of a spend path" (`packages/vectors/src/maturity_safety.rs:436–438`).

| Row | Position written | Rule the node applies | Host stage producing the bytes | Signature |
|---|---|---|---|---|
| §16.5 `wrong-internal-key` | the spent coin. A second predecessor is funded at the version-one program of `output_key(FOREIGN_INTERNAL_KEY, merkle_root)` over the honest constructor's root. The mutant spends it with the honest leaf; item 8 carries the foreign key's parity byte, the foreign key and the honest path; item 5 carries the foreign parity | the commitment holds; metadata authentication's `TweakVerify` against the leaf's NUMS `InternalKey` symbol fails | a new stage over `OracleStateCurve` (`packages/vectors/src/maturity_closure.rs:269`) and the public `merkle_root` | re-signed through `OperatorSigningRequest::freeze` over the mutant and its spent output (`packages/transaction/src/operator_signing.rs:137–193`) with the published handle (`packages/vectors/src/maturity_operator.rs:679`) |
| §16.5 `wrong-control-block` | item 5, the predecessor parity prefix the covenant pairs with the introspected input coordinate; item 8 stays honest | the commitment holds; metadata authentication's `TweakVerify` fails | the write of `replaced_predecessor_prefix_is_refused` (`packages/vectors/src/maturity_continuity.rs:4189–4191`) | the honest signature, since the script-path signature hash commits to the executing leaf and not to other witness items |
| §16.5 `metadata-leaf-selected-for-execution` | the whole witness, replaced by two items: the metadata leaf's program and its control bytes | the commitment holds; the leaf pushes the metadata and an empty item, and `Verify` aborts | the public `leaf_program` and `control_recipe(StateLeafRole::MetadataCommitment)` (`packages/tapscript/src/state_constructor.rs:930`, `packages/tapscript/src/state_constructor.rs:983–1014`) | none; the leaf reads none |
| §16.5 `key-path-spend-attempt` | the whole witness, replaced by one 64-byte item, the candidate's own item 6 | key-path Schnorr verification against the output key fails | a new stage; the live key-path probe is its precedent (`packages/vectors/tests/common/mod.rs:225`, `packages/vectors/tests/common/mod.rs:557–568`) | no new signing material |
| §16.6 `successor-under-another-static-subtree` | output zero's program, replaced by the program the honest successor metadata commits to under the `Second` deployment's static subtree, at a nonce admissible under both; items 0 and 1 are rebuilt to match | successor reconstruction's `TweakVerify` with the retained root fails | `substituted_successor`'s construction (`packages/vectors/src/maturity_continuity.rs:2815–2865`) over `maturity_sources` for `MaturityDeployment::Second` (`packages/vectors/src/maturity_closure.rs:355–362`, `packages/vectors/src/maturity_closure.rs:822`) | re-signed |
| §16.6 `wrong-internal-key` | output zero's program, replaced by the version-one program of `output_key(FOREIGN_INTERNAL_KEY, merkle_root)` over the honest successor's root; item 0 carries that key's parity | successor reconstruction's `TweakVerify` against the NUMS symbol fails | a new stage over `OracleStateCurve` and the successor constructor's public `merkle_root` | re-signed |
| §16.6 `wrong-parity` | item 0, flipped | successor reconstruction's `TweakVerify` fails | the write of `replaced_successor_prefix_is_refused` (`packages/vectors/src/maturity_continuity.rs:4300–4302`) | the honest signature |
| §16.6 `wrong-control-recipe` | none | — | — | — |

**Why these writes are the rows' faults.** For the three successor rows, a write on the candidate is byte-identical to the emission fault it names:

- An emission under a wrong parity differs from the honest one only in item 0, because the output program is an x-only key and the parity travels only as the witnessed prefix.
- An emission under a foreign key or a foreign static subtree differs in output zero's program and the witnessed prefix. The stage writes exactly those, computed from the same public derivation the emission uses.

The register's argument that "a program written over is a program that no longer matches its own commitments" (`packages/vectors/src/maturity_negative_half.rs:152–163`) concerns an arbitrary overwrite. It does not concern a program derived under the changed input. Each stage records, as its site, the row's locator and the positions it changed, and a stage test holds the two in agreement.

**Predecessor `wrong-internal-key` and a second coin.** This row cannot depart from the honest coin. The internal key the covenant authenticates against is a leaf constant, so any write that reaches it changes the leaf and draws the commitment rule. A write of another key into item 8 alone draws the same rule, which is `wrong-control-block`'s program-generic form. The row's fault lives in the spent object, so its mutant spends an object built that way. The adapter's retained reserve funds that coin at any stated program with no change (`scripts/elements-native-executor.py:3872–3950`).

**`wrong-control-block`: item 5, not item 8.** An item-8 write is the form the register calls program-generic, and its node string names only the commitment rule — the operator run recorded `Witness program hash mismatch` for a leaf its spent program does not commit to (`packages/vectors/tests/maturity_operator.rs:980–985`). The item-5 write reaches the covenant's own reading of the predecessor's control material, a parity the `ControlBlock` locator names. The tree already files that write under this row.

**`wrong-control-recipe` stays out.** A test-scoped linker knob would not help: it would emit a successor recipe that no announcement byte carries. The announcement would be byte-identical to the honest one and would be accepted (F4).

**D2. The one test-scoped derivation is the foreign internal key. It is reachable only through `StateCurveCapability::output_key` in the vectors crate, and three pins show the production constructor path refuses it.**

`FOREIGN_INTERNAL_KEY` is a published valid point other than `STATE_NUMS_KEY`. The proposed value is `STATE_GENERATOR_X` (`packages/tapscript/src/state_constructor.rs:552–565`), a point the tapscript crate already publishes. A coin under it is disposable test-network material, and its key path is never offered.

The pins:

1. A new test in the stage module, `the_foreign_internal_key_is_refused_by_the_public_constructor_path`, asserts that `StateInternalKeyPolicy::new(FOREIGN_INTERNAL_KEY, &OracleStateCurve)` returns `WrongInternalKey`. Beside it, `nums_is_recomputed_from_public_uncompressed_generator` still asserts the refusal of another key.
2. `state_output_program_at_nonce` re-admits its policy key before committing, so no public constructor entry takes the foreign key (`packages/tapscript/src/state_constructor.rs:1148`).
3. The stage bite's gate reading lists no changed file under `packages/tapscript/`, `packages/linker/` or `packages/transaction/`, so no production signature moves.

The foreign static subtree needs no knob at all: it comes from another deployment's public link.

**D3. Each row's boundary is its matrix declaration. The node's words are expected under the consensus prefix. Each row's carrier reading is a closed table declared before the run, and a refusal elsewhere is recorded and never answers.**

The adapter returns a script verdict at either prefix without retrying it (`scripts/elements-native-executor.py:4638–4648`). The operator run of record on the same tip holds its script-path and commitment refusals under `mandatory-script-verify-flag-failed` (`packages/vectors/tests/maturity_operator.rs:963–1009`). The layer is therefore `ScriptPathRejection` or `KeyPathRejection` whichever prefix the node prints, and the prefix travels in the detail as data.

On the operation path, no adapter table entry maps these strings (F10). The table entries below are cited for the class vocabulary they share with the harness, not as the mapping this path uses.

| Row | Declared boundary | Expected node words | Carrier reading |
|---|---|---|---|
| §16.5 `wrong-internal-key` | `ScriptPathRejection` | `mandatory-script-verify-flag-failed (EC scalar mult verify fail)` | executed on that script error |
| §16.5 `wrong-control-block` | `ScriptPathRejection` | the same | executed on that script error |
| §16.5 `metadata-leaf-selected-for-execution` | `ScriptPathRejection` | `mandatory-script-verify-flag-failed (Script failed an OP_VERIFY operation)` | executed on that script error |
| §16.5 `key-path-spend-attempt` | `KeyPathRejection` | `mandatory-script-verify-flag-failed (Invalid Schnorr signature)` | executed on that error at `key_path_rejection` |
| §16.6 `successor-under-another-static-subtree` | `ScriptPathRejection` | `EC scalar mult verify fail`, as above | executed on that script error |
| §16.6 `wrong-internal-key` | `ScriptPathRejection` | the same | executed on that script error |
| §16.6 `wrong-parity` | `ScriptPathRejection` | the same | executed on that script error |

Evidence for the expected words:

- **Curve relation.** The adapter's table maps `EC scalar mult verify fail` to `invalid_curve_relation` (`scripts/elements-native-executor.py:356`, `scripts/elements-native-executor.py:1183`). The host oracle answers a wrong parity in the same authentication with its curve-relation failure (`packages/tapscript/src/tests/state_announcement_tests.rs:625–639`).
- **Verify.** The table maps `Script failed an OP_VERIFY operation` to `false_verification` (`scripts/elements-native-executor.py:1223`).
- **Key path.** The operator run holds `Invalid Schnorr signature` for a refused signature, and `observed_key_path_spend` files a one-item witness at a version-one program as `key_path_rejection` (`scripts/elements-native-executor.py:320–328`, `scripts/elements-native-executor.py:6273`).

**The carrier reading is fail-closed.** Any other detail reads as `NotReached`, the binding then returns `CarrierDeparted`, and the refusal stays recorded while its row keeps waiting. A commitment string such as `Witness program hash mismatch` reads as not reached for every one of these rows, because no leaf ran. The reading is a table rather than an inference so that it is stated before the run, as the boundary is, and a reader who has only the record can recheck it.

**Recording.**

- The ceremony evidence carries, for each offer: the declared boundary beside the observed layer, the verbatim detail, the exact submitted bytes, the site, the carrier outcome, and the accepted control with its identity. These are exactly `MaturityNativeRefusal`'s facts.
- A refusal at another layer becomes `NativeRefusalAtUnexpectedBoundary` through `from_native_refusal`. It is counted in its own bucket, is never answered, keeps `every_required_row_is_answered` false (`packages/vectors/src/maturity_evidence.rs:1487–1494`), and sets the soundness clause to `ContradictedAtAnObservedBoundary` (`packages/vectors/src/maturity_report.rs:956–957`).
- A refusal whose subject, carrier or control does not bind leaves its row `NativeRunRequired`. The record stays in the admitted corpus.

**D4. The ceremony is two runs, one per §16 table. Each funds its predecessor, offers its mutants against the unspent coin, and accepts the honest announcement last, under the variable-metadata schedule. The adapter is unchanged.**

§20.5 lists the predecessor-constructor and successor-constructor matrices under "Run separately" (`plans/guides/guide_fourteen.md:3343–3349`). Each run then carries its own accepted control, its own record and its own verdicts. The shape within each run is the one the operator run of record established on the same tip.

The predecessor-constructor run has eight exchanges:

1. `issue-maturity-singleton`
2. `fund-maturity-predecessor`
3. `fund-foreign-key-predecessor`
4. `mutant-predecessor-wrong-internal-key`
5. `mutant-predecessor-wrong-control-block`
6. `mutant-metadata-leaf-selected-for-execution`
7. `mutant-key-path-spend-attempt`
8. `sponsorless`

The successor-constructor run has six:

1. `issue-maturity-singleton`
2. `fund-maturity-predecessor`
3. `mutant-successor-under-another-static-subtree`
4. `mutant-successor-wrong-internal-key`
5. `mutant-successor-wrong-parity`
6. `sponsorless`

Each offer is one `testmempoolaccept`. A refused offer mines nothing and leaves its coin unspent (`scripts/elements-native-executor.py:4610–4648`). The control is confirmed with its mined readback, as in the variable run (`packages/vectors/src/maturity_native.rs:677–716`), so it comes last. The `mutant-` prefix keeps every step name out of `mutation_fact`'s namespace (F9).

**Schedule.** Both runs use the retained `VariableMetadata` schedule, as the variable run did (`packages/vectors/tests/maturity_native.rs:183–185`). Its witness relays, so each mutant reaches its script. Under the whole schedule the mutants would be recorded before their script (F12).

**An accepted mutant.** If an offer is accepted, the adapter mines it and spends the coin that the later offers and the control need. That acceptance is an escape and a safety finding against the fifth conjunct and the fourth card condition. The planner records it and continues. Every later offer or control that spends the same coin then meets a spent input, which is no verdict about its row, and the run has no accepted control for its refusals to depart from. The evidence plan has no standing for a target accepting a negative row's mutant; if the review of a capture finds one, minting that standing under its own ruling precedes admission, and the admission bite does not land as shaped.

**No adapter change.** A submission is bytes and nothing else, so every mutant above, including the one-item witness, is carried as it is. Script verdicts are never retried, the reserve funds the foreign coin, and details arrive verbatim. The revision-8 protocol, adapter 2.1.0 and the node arguments stay as they are (`scripts/elements-native-executor.py:475`, `scripts/elements-native-executor.py:519`).

**Wall.** The variable run of record's timing sidecar holds no duration, and neither does its report (F2), so no wall figure can be read from the tree. The bound the tree does fix is the executor configuration's per-exchange timeout of 300 seconds (`packages/vectors/tests/maturity_native.rs:64–70`). That caps the exchanges at 2,400 seconds for the predecessor run and 1,800 seconds for the successor run, excluding the cargo phase. The runs' own walls are recorded by the driver under D5.

**D5. The carrier follows the operator precedent:**

- one new module holds the planner and the evidence;
- two new test targets each hold one ignored test under the generic `Report` ceremony with its own test name;
- one new driver serves both targets through a profile argument;
- the admitted run's driver and test stay byte-identical;
- both maturity capture contracts become Meson lanes;
- the new driver writes its phase walls into its `RUN-REPORT`.

**Why not a second ignored test in `maturity_native`.** That target runs every ignored test it holds under one argv, and the driver refuses any census other than one test passed (`scripts/live-native-maturity-capture.sh:434–438`).

**Why not a profile of the existing driver.** It would edit the admitted run's reproduction contract. Its pinned tip, its one-test census, its cargo argv at `scripts/live-native-maturity-capture.sh:395` and `:617`, and its test name at `:447` are what the variable run's `RUN-REPORT` binds and the importer checks (`packages/vectors/src/maturity_corpus.rs:829–839`). A profile would add a code path whose default then has to be re-proved.

**Why one new driver for both runs.** Copying the 678-line driver twice would triple one boundary. A new driver's profile touches no admitted path. The existing maturity driver and its contract stay unchanged and keep passing, which a lane makes mechanical (F8).

**Why the `Report` ceremony.** The operator run is the precedent: all eleven of its operations are auxiliary with `mutation-kind none` in its capture, and its refusal attribution lives in its own payload and replay, as the capture under `packages/vectors/fixtures/native-operator-run-of-record/` shows in its eleven `request-role auxiliary` and eleven `mutation-kind none` lines. No `CeremonyId` member and no shared capture grammar is added.

**Why phase walls in the new `RUN-REPORT`.** §20.5 requires each run's wall outside canonical report bytes. The `RUN-REPORT` is the driver's own grammar, whereas the timing sidecar is shared by every ceremony and pinned by both existing importers (`packages/vectors/src/maturity_corpus.rs:1563–1567`).

**D6. The admission is one directory per run, each of four files, admitted through the existing importer generalized by a target pin and a mutant roster, with a replay that rebuilds every mutant request.**

**Directories.** `packages/vectors/fixtures/maturity-predecessor-mutant-run-of-record/` and `packages/vectors/fixtures/maturity-successor-mutant-run-of-record/`. Each holds the four files `packages/vectors/fixtures/maturity-variable-run-of-record/` holds: one `.report.capture` named by the suite's short identifier, its `.timing` sidecar, `MANIFEST.sha256` and `RUN-REPORT`.

**Pins.** `MaturityCorpusPins` gains a ninth declared pin, `MaturityCaptureTarget`, naming the test target and the Rust test name. It replaces the literals at `packages/vectors/src/maturity_corpus.rs:829–839` and `:1008–1009`. `HISTORICAL_PINS` and `VARIABLE_PINS` state today's `maturity_native` and `the_maturity_announcement_runs_against_a_real_target`. The ceremony id and timing literal stay `report`.

The new pins state, for each run:

- the `VariableMetadata` schedule;
- the four names and sizes;
- the manifest, run and ceremony addresses as constants named `MATURITY_PREDECESSOR_MUTANT_MANIFEST_SHA256`, `MATURITY_PREDECESSOR_MUTANT_RUN_ADDRESS` and their successor counterparts;
- the `Acceptance` expectation for the control;
- the cargo path and the `IntendedTip` framework binding;
- the funding count;
- `MATURITY_DISCLOSED_FEE_FLOORS`.

`MaturityMutantCorpusPins` pairs such a pin with a `MaturityMutantTable`, whose roster and declared boundaries come from the matrix rather than the pins.

**Admission path.** `admit_maturity_mutant_corpus` reuses the census, manifest, report, header and environment layers. It reads a roster-length operation grammar and replays through `MaturityMutantEvidence::from_transcript`, which rebuilds every funding and mutant request from the stages and compares it with the recorded request. The stages are deterministic, since signing takes zero auxiliary randomness (`packages/vectors/src/maturity_native.rs:874–878`). It also reads the driver's phase-wall lines as noncanonical facts.

**Accessors.** `maturity_predecessor_mutant_run_of_record()` and `maturity_successor_mutant_run_of_record()` admit once each, as the two existing accessors do (`packages/vectors/src/maturity_corpus.rs:1597–1604`, `packages/vectors/src/maturity_corpus.rs:1642–1649`). `replay_maturity_mutant_run_of_record` replays under a stated selection.

**Refusals.** Every refusal is an existing `MaturityCorpusImportRefusal` member:

- a roster or request that differs from replay is `Derivation` of `TranscriptStepMismatch`;
- a payload that differs from replay is `PayloadDisagreement`;
- a phase-wall line outside its grammar is `RunReportGrammar`.

The harness adds `MaturityNativePlanRefusal::MutantStage`, carrying a stage refusal whole, so a stage that refuses at replay reaches `Derivation` unchanged.

**D7. The evidence plan classifies an admitted mutant refusal only through the binding. The census, the register and every pinning test move by the figures the run produces; the figures below assume all seven bind.**

`derive_maturity_evidence_plan_with` reads both mutant accessors beside the two existing corpora and refuses with `NativeCorpusImportRefused` as they do. For a row whose standing is `NativeRunRequired` and whose refusal an admitted mutant evidence carries:

| `native_refusal_binds_to_row` returns | The row's standing |
|---|---|
| `Bound` | `from_native_refusal`, hence `NativeRefusalObserved` |
| `ObservedElsewhere` | `from_native_refusal`, hence `NativeRefusalAtUnexpectedBoundary` |
| `AnotherSubject`, `CarrierDeparted` or `AnotherControl` | `NativeRunRequired`, with the record retained in the corpus |

The record's row comes from the evidence, never from the binding (F11).

| Figure | Before | After |
|---|---|---|
| native refusal observed | 0 | 7 |
| native run required | 42 | 35 |
| answered | 71 | 78 |
| outstanding | 135 | 128 |
| native declared-boundary observed | 0 | 0 |
| native refusal at an unexpected boundary | 0 | 0 |
| `STILL_REQUIRED` | 42 | 35 |

The partition then reads 40 + 43 + 35 + 1 + 7 + 15 + 16 + 14 + 35 = 206.

**Register.** The seven answered entries leave `STILL_REQUIRED`. `wrong-control-recipe` re-grounds to `RefusalIsProgramGeneric`, with the ground that its recipe appears in no announcement byte and is read only when the successor is spent, where every non-committing recipe draws the commitment rule's verdict before an opcode runs. The recounted census:

| Member | Rows |
|---|---|
| `MutationStagingOwedOnTheLinkedCandidate` | 18 |
| `NoAdmittedShapeCarriesTheFault` | 4 |
| `ProgramDerivationHasNoKnobForTheFault` | 4 |
| `RefusalIsProgramGeneric` | 2 |
| `NoPublishedRelationFilesTheObservation` | 7 |
| total | 35 |

**Tests that move:**

- `the_census_figures_are_unmoved_by_the_binding_readers` pins 35, 78 and 7 and adds the refusal bucket to its partition;
- `the_census_buckets_are_the_figures_the_inputs_recompute` subtracts the observed refusals and adds the bucket to its sum;
- `required_rows_remain_outstanding_after_the_admitted_run` adds the bucket to the answered sum;
- `the_positive_row_names_the_accepted_run_and_its_disclosed_environment` pins 7;
- `the_register_length_is_the_figure_the_plan_recomputes` and `the_gap_census_accounts_for_every_row` pin 35.

`maturity_safety.rs` does not change.

**D8. The safety report moves to schema 2 with a `native_refusal` row block, because the admission changes its rendered grammar and not only its figures.**

Without a new block, the admission changes only values: the census figures, seven fewer `waiting` lines, and `clause_native_refusals_witnessed`. On that reading, schema 1, which names the grammar, would stand.

The rows, however, would then appear in no block. The clause would count seven native refusals that its canonical bytes never name — a figure without its list, which the register's own design rejects (`packages/vectors/src/maturity_negative_half.rs:4–21`). Ruling 106 has the gate record transcribe rows by name from the validated rendering (`plans/phases/06-state-and-maturity.md:368`).

The block therefore adds one line per answered native refusal: `native_refusal`, followed by the section, the row, the declared boundary, the observed layer and the carrier outcome. Following the report's rule, the line carries no detail text and no archived address (`packages/vectors/src/maturity_report.rs:1482–1493`). A new line kind is a new grammar, so `MATURITY_SAFETY_REPORT_SCHEMA` moves from 1 to 2, and the validation recomputes the block from the plan as it recomputes the `witness` block.

**D9. The study proposes these readings; the gate record decides them on the admission's evidence.**

If all seven bind:

- **§23 conjunct 5** reads CONDITIONALLY MET on a narrower condition. Seven of the eight rows are refused natively at their declared boundaries. `wrong-control-recipe` has no offerable position (D1). The key-path refusal establishes a refused signature and "nothing whatever about who knows the internal key's discrete logarithm" (`packages/target-elements-conformance/src/protocol.rs:1358–1366`).
- **Card condition 4** has both bypass paths refused natively. Beside that stand two host facts: the metadata leaf always aborts over every admitted stack, and the key path carries its residual. The condition reads CONDITIONALLY MET, narrowed to that residual.
- **Card condition 6** reads CONDITIONALLY MET, narrowed to `wrong-static-subtree`'s first-party case, since that row stays first-party required (F6).

The gate record cites these carriers:

- the tag `T11-152` lands on;
- the accessors `maturity_predecessor_mutant_run_of_record` and `maturity_successor_mutant_run_of_record`;
- `the_census_figures_are_unmoved_by_the_binding_readers` with its moved figures;
- `each_admitted_mutant_refusal_answers_its_row_at_its_declared_boundary`;
- the rendered `native_refusal` block.

The fifth conjunct's host evidence drops `metadata-leaf-missing` as a rejection (F5).

**D10. The order is `T11-148` → `T11-149` → `T11-150` → `T11-151` → `T11-152` → `T11-146` → `T11-141` → `T11-147`.**

Each host step depends on the one before it. The ceremony runs at `T11-150`'s landing tag, and the admission lands its capture. The documents sweep `T11-146` moves after the admission: that bite restates the vectors crate's run and row sentences and the lane roster (`plans/backlog.md:1339`), and the admission changes both — two runs of record, a report schema and two lanes. The sweep then reads the crate once, at its final Phase-6 scope. `T11-141` keeps its place after the sweep, and `T11-147` after both, so the gate record cites the fifth conjunct on evidence rather than on a standing. The other order keeps the sweep where ruling 114 put it and has `T11-152` restate the vectors README and contract sentences it makes stale. That spreads one reading over two bites, which is the split ruling 109 avoided.

**D11. What a native mutant run of record establishes:** that a target at the pinned tip and configuration refused these seven exact byte strings, each at its row's declared boundary, beside an accepted control of the row's own shape. It establishes nothing more:

- no general theorem over mutants that were not offered;
- nothing about positive-fee-floor relay, which stays N2 under Wave 11's C1;
- no acceptance beyond the admitted corpus;
- nothing about a node other than the tip, adapter and arguments the pins name;
- nothing about `wrong-control-recipe`;
- nothing about the internal key's discrete logarithm.

Every figure stands under the plan's class-one and class-two quantifier (`packages/vectors/src/maturity_evidence.rs:1496–1500`).

## 4. Open questions

**OQ1. Whether §20.5's "Run separately" requires two runs, or allows one ceremony covering both tables.**

For two runs:

- the guide lists the two matrices as separate runs, and each run records its own facts (`plans/guides/guide_fourteen.md:3343–3379`);
- a failure in one table then leaves the other's record whole;
- each table gets its own accepted control.

For one ceremony:

- one funded predecessor serves every offer, as the operator run's did;
- it costs one node boot, one issuance and one control fewer;
- "separately" can be read as separate records within one run.

D4 proceeds on two runs. If the ruling reads one ceremony, the two tables fold into one planner, one test target and one directory, and nothing else in the bites changes.

**OQ2. Whether a refusal by the covenant's constructor-authentication clauses executes the `LinkedConstructor` carrier the rows declare, or the `AnnouncementLeaf` carrier.**

For `AnnouncementLeaf`: the matrix lists "predecessor metadata authentication" and "successor constructor reconstruction" among that carrier's relations (`packages/vectors/src/maturity_safety.rs:309–329`). The clauses that refuse five of the seven mutants are those two.

For `LinkedConstructor`:

- The carrier names "static-subtree continuity, and internal-key and leaf-version policy". The relation these rows violate is enforced in the running program exactly by `TweakVerify` over the constructor's commitment.
- The leaf's own owner vocabulary names the two clauses "Predecessor constructor binding" and "Successor constructor binding" (`packages/tapscript/src/state_announcement.rs:67–78`).
- Under the strict reading, no target could ever execute a `LinkedConstructor` carrier at `ScriptPathRejection`, so seven declarations would declare verdicts no run can produce. That is the confusion `boundary_admits` exists to prevent (`packages/vectors/src/maturity_safety.rs:1045–1053`).

D3's carrier table proceeds on the `LinkedConstructor` reading. Under the other reading, five rows stay `NativeRunRequired` with their refusals recorded, and only `metadata-leaf-selected-for-execution` and `key-path-spend-attempt` bind.

**OQ3. Whether the fifth conjunct and the fourth card condition read MET after the admission.**

For MET:

- `wrong-control-recipe` is no escape of the announcement, because no announcement byte carries it, and BIP-341 refuses every non-committing recipe when the successor is spent.
- A discrete-log assumption is a limit no bite or run discharges, so it belongs among the non-claims beside conjunct 19's.

For CONDITIONALLY MET:

- No refusal of `wrong-control-recipe` is observed by any run or validator.
- The Wave-13 record keeps conjuncts 11 and 13 conditional on limits the host evidence states about itself (`plans/drafts/wave13-gate-and-handoff-study.md:341–346`, `plans/phases/06-state-and-maturity.md:366`), and the key-path residual is such a limit.

D9 proposes the conditional reading, and the gate record takes it on the admission's evidence.

### Proposed findings for the amendment

The amendment records F1 to F12 as findings 234 to 245, each relocated and recounted at its own base.

### Proposed rulings for the amendment

The house form numbers rulings on the phase card; these continue from 117 (`plans/phases/06-state-and-maturity.md:390`).

118. N1's condition holds: both compute nodes carry the pinned Elements toolchain at the tip `EXPECTED_ELEMENTSD_TIP` names, which finding 196's statement no longer describes. N1 receives rows `T11-148` to `T11-152`, and N2 keeps its Wave-11 C1 condition. The estate carries the fact; each admitted capture's handshake fields bind it to the pins after the run (`plans/phases/06-state-and-maturity.md:1206`, `plans/phases/06-state-and-maturity.md:1352`, `scripts/live-native-maturity-capture.sh:42`, `plans/drafts/wave13-gate-and-handoff-study.md:779–787`).

119. Seven rows are staged in the vectors crate at spend-time positions, or on a coin the ceremony funds, with no production crate changed:

- §16.5 `wrong-internal-key` spends a coin whose program commits under `FOREIGN_INTERNAL_KEY`;
- `wrong-control-block` writes the predecessor parity item;
- `metadata-leaf-selected-for-execution` reveals the metadata leaf through its public recipe;
- `key-path-spend-attempt` offers one 64-byte item;
- `successor-under-another-static-subtree` writes output zero under the `Second` deployment's subtree;
- §16.6 `wrong-internal-key` writes output zero under the foreign key;
- `wrong-parity` writes the successor parity item.

Output-changing and coin-changing mutants are re-signed with the published handle. The foreign key is refused by the public constructor path, as a test pins. `wrong-control-recipe` has no announcement position and leaves N1 (`packages/tapscript/src/state_announcement.rs:457–483`, `packages/tapscript/src/state_constructor.rs:983–1014`, `packages/tapscript/src/state_constructor.rs:1141–1152`, `packages/vectors/src/maturity_continuity.rs:4185–4214`, `packages/vectors/src/maturity_continuity.rs:4296–4326`).

120. Each mutant's boundary is its matrix row's refusing layer. The expected node words and each row's carrier reading are a closed table declared before the run. A refusal at another layer is recorded as `NativeRefusalAtUnexpectedBoundary`, and a refusal failing subject, carrier or control leaves its row waiting with its record retained (`packages/vectors/src/maturity_evidence.rs:1061–1086`, `packages/vectors/src/maturity_evidence.rs:1257–1263`, `packages/vectors/tests/maturity_operator.rs:963–1009`).

121. The ceremony is a predecessor-constructor run of eight exchanges and a successor-constructor run of six. Each offers its mutants against its unspent coin and accepts the honest announcement last, under the variable schedule, through the unchanged revision-8 adapter. Each runs from its own test target through one new driver with two profiles, while the admitted runs' driver and its contract stay byte-identical. Both maturity capture contracts are Meson lanes, and the new driver records its phase walls in its `RUN-REPORT` (`plans/guides/guide_fourteen.md:3341–3379`, `packages/vectors/src/maturity_operator.rs:52–79`, `scripts/live-native-maturity-capture.sh:434–448`, `meson.build:928–952`).

122. The two captures are admitted under their own pins through the importer, generalized by a capture-target pin and a mutant roster, with every mutant request rebuilt at replay. The plan files each bound refusal as `NativeRefusalObserved`. The register loses the answered entries and re-grounds `wrong-control-recipe` as program-generic. The safety report moves to schema 2 with a `native_refusal` block (`packages/vectors/src/maturity_corpus.rs:1581–1604`, `packages/vectors/src/maturity_negative_half.rs:286–596`, `packages/vectors/src/maturity_report.rs:54`, `packages/vectors/src/maturity_report.rs:1449–1480`).

123. `T11-146` depends on `T11-152` instead of `T11-145`, so the sweep reads the vectors crate and the lane roster at their final Phase-6 scope. `T11-141` and `T11-147` keep their dependencies, and `T11-147` cites the admission's tag, accessors and census test for §23's fifth conjunct and the card's fourth and sixth conditions (`plans/phases/06-state-and-maturity.md:384`, `plans/backlog.md:1339–1340`).

124. The gate record reads §23's fifth conjunct and the card's fourth condition as CONDITIONALLY MET on the narrowed conditions of D9 and states the MET reading beside them. It reads the card's sixth condition as CONDITIONALLY MET on `wrong-static-subtree`'s first-party case, and cites no carried first-party row as a rejection (`plans/guides/guide_fourteen.md:3593`, `plans/phases/06-state-and-maturity.md:1456–1458`, `packages/vectors/src/maturity_first_party.rs:3400–3403`, `packages/vectors/src/maturity_first_party.rs:3441–3444`).

### Proposed rows

- `T11-148` | TODO | Bite 1 adds `packages/vectors/src/maturity_mutants.rs` (new), its `pub mod maturity_mutants;` line in `packages/vectors/src/lib.rs` and its census line in `packages/vectors/meson.build`, and widens `fixed_projection` and `prefix` in `packages/vectors/src/maturity_continuity.rs` to `pub(crate)`. `MaturityMutantRow` names the seven staged rows with their section, matrix name, step name, site and declared layer read from the matrix. `FOREIGN_INTERNAL_KEY` fixes the foreign point, and `stage_maturity_mutant` produces each mutant's exact bytes over a signed honest submission, re-signing through `OperatorSigningRequest::freeze` where the change reaches a signed field. `foreign_key_predecessor_program` states the second coin's program. Nine tests pin positions, host refusals, re-signed verification, the public constructor's refusal of the foreign key, the metadata leaf's commitment, the key-path shape, the declarations, determinism, and the absence of a successor-recipe position. DONE when every stage's bytes differ from the honest submission exactly at its declared positions and the host projector or commitment recomputation reaches the row's clause. Gate: nine test markers in the new module, 58 unchanged in continuity, one Meson census entry, and no changed file under `packages/tapscript/`, `packages/linker/` or `packages/transaction/`. Depends on the amendment under `T11-142`.
- `T11-149` | TODO | Bite 2 adds `packages/vectors/src/maturity_mutant_ceremony.rs` (new) with its `lib.rs` and census lines, and the test targets `packages/vectors/tests/maturity_predecessor_mutants.rs` and `packages/vectors/tests/maturity_successor_mutants.rs` (new) with their census lines. It also adds `MaturityNativePlanRefusal::MutantStage` in `packages/vectors/src/maturity_native.rs`. Over bite 1's `MaturityMutantTable`, `PREDECESSOR_MUTANT_STEPS`, `SUCCESSOR_MUTANT_STEPS`, `MaturityMutantCeremonyPlanner`, `declared_layer` from the matrix, the closed carrier table and `MaturityMutantEvidence::from_transcript` produce one `MaturityNativeRefusal` per offer. Each target holds one ignored real-target test and five scripted tests. DONE when scripted runs settle both rosters, a scripted refusal at the declared layer yields a bound record, a refusal elsewhere and an unexpected detail yield unbound records, and no step name has a live mutation fact. Gate: eight test markers in the new module, six in each target with one ignored in each, three Meson census entries, 32 markers unchanged in `maturity_native.rs`, and the variable run of record still admitting. Depends on `T11-148`.
- `T11-150` | TODO | Bite 3 adds `scripts/live-native-maturity-mutant-capture.sh` and `scripts/test-live-native-maturity-mutant-capture.sh` (new) and wires two Meson lanes, `live-native-maturity-capture-contract` and `live-native-maturity-mutant-capture-contract`, in `meson.build`. The driver takes a profile, `predecessor-constructor` or `successor-constructor`, and a new capture directory. It keeps the maturity driver's environment contract, pinned tip and one-test, one-ceremony, no-setup census, selects the profile's test target and test name, and writes `phase-wall-ms` lines into its `RUN-REPORT`. DONE when both contracts pass as lanes and `scripts/live-native-maturity-capture.sh` is byte-identical to its base. Gate: both new lanes green, the lane count 52 → 54, and `git diff` empty for the existing driver and its contract. Depends on `T11-149`.
- `T11-151` | TODO | The ceremony: both driver profiles run on an Elements-provisioned compute node from a clean suite worktree at `T11-150`'s landing tag, under the environment contract of section 5. Its deliverable is two capture directories, each of four files and concluding `eligible yes`, copied to the host and reviewed as text; its landing is `T11-152`. DONE when both directories exist, every offer's recorded layer and detail have been read against D3's table, and each control is accepted with its mined readback. Gate: the driver's own refusals, which conclude `eligible yes`, and the review; an accepted mutant or an unexpected detail is a finding recorded before admission. Depends on `T11-150`.
- `T11-152` | TODO | Bite 4 admits both directories under `packages/vectors/fixtures/` with eight census lines in `packages/vectors/meson.build`, and edits `packages/vectors/src/maturity_corpus.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_negative_half.rs` and `packages/vectors/src/maturity_report.rs` as D6 to D8 state. DONE when both runs admit, both earlier runs still admit, each bound refusal stands `NativeRefusalObserved`, and the census, register and report pin the run's figures. Gate: the test counts of section 5, the census figures the run produces, the report at schema 2, and exactly the named files changed. Depends on `T11-151`.

## 5. The bites

Each bite is one named commit under a row the amendment mints, and each is gated. The counts before are section 2.7's. The counts after are each gate's obligations, recounted at the base the bite lands on. Every gate row is a reading the gate prints.

### Bite 1 — the mutant stages (`T11-148`)

**Files:**

- `packages/vectors/src/maturity_mutants.rs` (new);
- its `pub mod maturity_mutants;` line in `packages/vectors/src/lib.rs`;
- its census line `'src/maturity_mutants.rs',` in `packages/vectors/meson.build`, between `'src/maturity_measurements.rs',` and `'src/maturity_native.rs',` (`packages/vectors/meson.build:64–65`);
- `packages/vectors/src/maturity_continuity.rs`, where `fixed_projection` and `prefix` become `pub(crate)` with no body change.

**Types and functions:**

- `MaturityMutantRow`, seven members: `PredecessorWrongInternalKey`, `PredecessorWrongControlBlock`, `MetadataLeafSelectedForExecution`, `KeyPathSpendAttempt`, `SuccessorUnderAnotherStaticSubtree`, `SuccessorWrongInternalKey` and `SuccessorWrongParity`. Each has `section()`, `row()`, `step()` (the `mutant-` step names of D4), `site()` (the row's `MaturityMutationLocator`), and `declared_layer()` read from `rows()` by section and name.
- `MaturityMutantTable`: `PredecessorConstructor` or `SuccessorConstructor`, with `rows()`.
- `FOREIGN_INTERNAL_KEY`.
- `foreign_key_predecessor_program(bundle) -> Result<Vec<u8>, MaturityMutantStageRefusal>`.
- `stage_maturity_mutant(row, context: &MaturityMutantContext) -> Result<MaturityStagedMutant, MaturityMutantStageRefusal>`. The context carries the target, the honest bundle, the finalized announcement and its signed bytes, and, for the foreign-key row, the funded foreign outpoint.
- `MaturityStagedMutant`: `row`, `bytes`, `changed_witness_items`, `changed_output`, `spent_outpoint`.
- `MaturityMutantStageRefusal`, which carries the closure, transaction, tapscript and signing refusals whole.

**Reused, with visibility at `0.6.296-dev`:**

- `OracleStateCurve` (public, `packages/vectors/src/maturity_closure.rs:269`);
- `maturity_sources` and `MaturityDeployment::Second` (public, `packages/vectors/src/maturity_closure.rs:355–362`, `packages/vectors/src/maturity_closure.rs:822`);
- `OPERATOR_HANDLE` and `OperatorVerifier` (public, `packages/vectors/src/maturity_operator.rs:679–688`);
- `OperatorSigningRequest::freeze` and `message` (public, `packages/transaction/src/operator_signing.rs:137–193`);
- `leaf_program`, `merkle_root`, `parity`, `output_program` and `control_recipe` (public, `packages/tapscript/src/state_constructor.rs:930–1014`);
- `StateInternalKeyPolicy::new` (public, `packages/tapscript/src/state_constructor.rs:584`).

| Test | What it asserts |
|---|---|
| `each_stage_changes_exactly_its_declared_positions` (new) | each mutant's bytes differ from the honest submission at its declared witness items, output and outpoint and nowhere else |
| `each_stage_reaches_its_rows_host_clause` (new) | the predecessor-parity mutant refuses with `PredecessorPrefix`, the successor-parity mutant with `SuccessorPrefix`, both output mutants with `OutputProgram`, and the foreign-key mutant, projected over its own funded coin, with `PredecessorProgram`, each over the variable-schedule source |
| `re_signed_mutants_verify_under_the_published_operator_key` (new) | each re-signed mutant's item 6 verifies under `OperatorVerifier` over its own frozen bytes |
| `the_foreign_internal_key_is_refused_by_the_public_constructor_path` (new) | `StateInternalKeyPolicy::new` refuses `FOREIGN_INTERNAL_KEY` with `WrongInternalKey` |
| `the_metadata_leaf_reveal_commits_to_the_funded_program` (new) | the reveal's recomputed output key equals the funded predecessor's program key, and the revealed leaf aborts over the empty stack |
| `the_key_path_mutant_is_one_item_over_equal_witnessless_bytes` (new) | one 64-byte item; the witnessless serialization equals the control's |
| `every_stage_declares_its_rows_matrix_facts` (new) | each member's layer, locator, boundary, carrier and control equal its matrix row's, and the boundary admits the layer |
| `the_stages_are_deterministic` (new) | two stagings over one honest submission produce equal bytes |
| `no_announcement_byte_carries_the_successor_recipe` (new) | the announcement has nine items and one output, and no item equals the successor constructor's control bytes |

**Counts:** the new module has 9 tests; `packages/vectors/src/maturity_continuity.rs` stays at 58. Estimate: production 260–420 lines; tests 200–320.

| Reading | Must be |
|---|---|
| `grep -c '#\[test\]' packages/vectors/src/maturity_mutants.rs` | 9 |
| `grep -c '#\[test\]' packages/vectors/src/maturity_continuity.rs` | 58 |
| `grep -c 'maturity_mutants' packages/vectors/meson.build` | 1 |
| `git diff --name-only` against the bite's base | exactly the four files, none under `packages/tapscript/`, `packages/linker/` or `packages/transaction/` |

What this bite does not establish: no node verdict, and no claim that the host clause a mutant reaches is the one a node reaches. The metadata-leaf and key-path mutants reach no host clause the projector owns; their evidence is the commitment recomputation and the witness shape.

### Bite 2 — the ceremony harness (`T11-149`)

**Files:**

- `packages/vectors/src/maturity_mutant_ceremony.rs` (new);
- its `lib.rs` line and census line;
- `packages/vectors/src/maturity_native.rs`, one `MaturityNativePlanRefusal` member;
- `packages/vectors/tests/maturity_predecessor_mutants.rs` and `packages/vectors/tests/maturity_successor_mutants.rs` (new);
- their two census lines in `packages/vectors/meson.build`, beside `'tests/maturity_native.rs',` (`packages/vectors/meson.build:293`).

**Types and functions:**

- `PREDECESSOR_MUTANT_STEPS: [&str; 8]` and `SUCCESSOR_MUTANT_STEPS: [&str; 6]`, as D4 lists them.
- `declared_layer(step) -> Option<ObservedOutcomeLayer>`, read from the matrix row the step stages; funding steps declare none; `sponsorless` declares `Accepted`. `expected_layer` in `maturity_native.rs` is unchanged, as the operator module keeps its own declaration function.
- `carrier_execution(row, observed_layer, detail) -> MaturityCarrierExecution`, D3's closed table.
- `MaturityMutantCeremonyPlanner::new(identity, branch, table)`, implementing `TargetOperationPlanner`. It relinks after issuance, builds and signs the honest announcement after the predecessor's funding, funds the foreign coin in the predecessor table, and stages each offer through bite 1.
- `MaturityMutantEvidence::from_transcript(identity, branch, table, exchanges) -> Result<MaturityMutantEvidence, MaturityNativePlanRefusal>`, returning one `MaturityNativeRefusal` per offer with its row and the accepted control `MaturityAcceptedControl::new(SponsorlessAnnouncement, identity)`.
- `MaturityNativePlanRefusal::MutantStage(Box<MaturityMutantStageRefusal>)`.

**Each test target:**

- one `#[ignore]` real-target test, `the_maturity_predecessor_mutants_are_refused_before_their_control_is_accepted` or its successor counterpart, on the form of `packages/vectors/tests/maturity_native.rs:164–224`: same environment, `CeremonyId::Report` with its own test name, the payload written before the gates;
- five scripted tests, listed in the table below.

| Test | Where | What it asserts |
|---|---|---|
| `every_step_declares_the_layer_its_row_declares` (new) | module | each mutant step's declaration is its matrix row's refusing layer, funding declares none, and the control declares acceptance |
| `the_carrier_table_is_closed_and_fails_closed` (new) | module | the expected detail reads executed; a commitment string and an unknown string read not reached |
| `a_bound_record_carries_every_fact_the_row_needs` (new) | module | a scripted refusal at the declared layer binds under `native_refusal_binds_to_row` |
| `a_refusal_elsewhere_is_recorded_and_does_not_bind` (new) | module | a scripted relay refusal yields `ObservedElsewhere` and keeps its observation |
| `an_accepted_mutant_is_recorded_as_an_escape` (new) | module | a scripted acceptance of an offer is retained with its layer, and a later offer on the same coin carries no verdict about its row |
| `the_funding_steps_admit_only_accepted_coins` (new) | module | a refused funding stops the planner with `FundingNotAccepted` |
| `replay_rebuilds_every_mutant_request` (new) | module | an altered recorded mutant refuses with `TranscriptStepMismatch` |
| `the_two_rosters_partition_the_seven_rows` (new) | module | the predecessor and successor tables hold four and three rows, disjoint, and together `MaturityMutantRow`'s members |
| `scripted_ceremony_settles_its_roster` (new) | each target | scripted responses settle every exchange and the evidence carries one record per offer |
| `scripted_payload_renders_every_record_in_order` (new) | each target | the payload lists each offer's declared and observed layers and carrier outcome |
| `scripted_capture_is_written_before_the_gates` (new) | each target | the enhanced capture is written with every operation auxiliary |
| `no_mutant_step_is_read_as_a_live_mutation` (new) | each target | `mutation_fact` returns neither kind nor locator for every step |
| `shared_capture_names_preserve_the_legacy_roster` (new) | each target | naming the target's own test for the `Report` ceremony leaves every other ceremony's test name unchanged, as `packages/vectors/tests/maturity_native.rs:557–585` checks for its own name |

**Counts:** the new module has 8 tests; each target has 6, one of them ignored; `packages/vectors/src/maturity_native.rs` stays at 32; `packages/vectors/tests/maturity_native.rs` stays at 10. Estimate: production 380–600 lines; tests 420–640.

| Reading | Must be |
|---|---|
| `grep -c '#\[test\]' packages/vectors/src/maturity_mutant_ceremony.rs` | 8 |
| `grep -c '#\[test\]' packages/vectors/tests/maturity_predecessor_mutants.rs` | 6 |
| `grep -c '#\[test\]' packages/vectors/tests/maturity_successor_mutants.rs` | 6 |
| `grep -c '#\[ignore' packages/vectors/tests/maturity_predecessor_mutants.rs` | 1 |
| `grep -c '#\[ignore' packages/vectors/tests/maturity_successor_mutants.rs` | 1 |
| `grep -c '#\[test\]' packages/vectors/src/maturity_native.rs` | 32 |
| `grep -c 'maturity_mutant_ceremony\|maturity_predecessor_mutants\|maturity_successor_mutants' packages/vectors/meson.build` | 3 |
| `git diff --name-only` against the bite's base | exactly the six files |

What this bite does not establish: no node verdict. The scripted runs establish transcript consistency, not an executor's provenance.

### Bite 3 — the driver and the lanes (`T11-150`)

**Files:**

- `scripts/live-native-maturity-mutant-capture.sh` (new);
- `scripts/test-live-native-maturity-mutant-capture.sh` (new);
- `meson.build`.

The scripts census does not list shell sources (`scripts/meson.build:1–4`).

**Shape:**

- Usage: `scripts/live-native-maturity-mutant-capture.sh <profile> <new-capture-directory>`, with the profile `predecessor-constructor` or `successor-constructor`.
- The profile selects the cargo argv `cargo test -p tripod-vectors --test maturity_predecessor_mutants -- --ignored --test-threads=1` or its successor counterpart, and the one test name that argv selects.
- The environment contract, `EXPECTED_ELEMENTSD_TIP`, `EXPECTED_TEST_COUNT=1`, `EXPECTED_CEREMONY_COUNT=1`, `EXPECTED_SETUP_COUNT=0` and `CEREMONY_IDS='report'` are the maturity driver's (`scripts/live-native-maturity-capture.sh:13–48`).
- The `RUN-REPORT` adds `phase-wall-ms <phase> <milliseconds>` for the preflight, cargo, census and manifest phases, which finish before the report is written.
- The contract test runs both profiles over synthetic captures and a mock cargo, in the form of `scripts/test-live-native-maturity-capture.sh`.
- `meson.build` wires this contract and the existing maturity contract as integration lanes in the form of `meson.build:941–952`.

**Counts:** Meson lanes 52 → 54.

| Reading | Must be |
|---|---|
| the lane `live-native-maturity-mutant-capture-contract` | green |
| the lane `live-native-maturity-capture-contract` | green |
| `git diff --stat` against the bite's base for `scripts/live-native-maturity-capture.sh` and `scripts/test-live-native-maturity-capture.sh` | empty |
| `git diff --name-only` against the bite's base | exactly the three files |

What this bite does not establish: the driver's boundary, not the harness's grammar. The contract runs no node.

### The ceremony (`T11-151`)

A run performed by the loop, not a code change. Its deliverable is two capture directories; its landing is bite 4.

**Environment contract:**

- **Invocation.** From a clean suite worktree at `T11-150`'s landing tag on an Elements-provisioned compute node: `scripts/live-native-maturity-mutant-capture.sh predecessor-constructor <new-capture-directory>`, then `scripts/live-native-maturity-mutant-capture.sh successor-constructor <new-capture-directory>`, each into a new directory.
- **Required:** `TRIPOD_LIVE_EXECUTOR`, `TRIPOD_LIVE_NETWORK_ID`, `TRIPOD_LIVE_GENESIS_ID`, `ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD`, `ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI`, `ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK`, `ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID`, `ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP`, `ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET` and `TMPDIR`.
- **Optional:** `TRIPOD_CAPTURE_CARGO`, `TRIPOD_CAPTURE_SHA256SUM`, `ELEMENTS_NATIVE_EXECUTOR_PYTHON` and `ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY`.
- **Derived by the driver:** `TRIPOD_LIVE_SUITE_COMMIT`, `TRIPOD_LIVE_SUITE_TREE`, `TRIPOD_LIVE_SUITE_SHORT_SHA` and `TRIPOD_LIVE_REPORT_DIR`.
- **Executor.** The revision-8 adapter through `scripts/elements-native-executor.sh`, whose node runs with the disclosed zero fee floors.
- **Hand-off.** Each directory is copied to the host and reviewed as text: every offer's layer and detail against D3's table, and the control's acceptance and readback.
- **Wall.** The run's wall is the driver's `phase-wall-ms` lines and the loop's own timing of each invocation.

### Bite 4 — the admission (`T11-152`)

**Files:**

- the eight files of the two directories under `packages/vectors/fixtures/`, and their eight census lines in `vectors_fixture_files` (`packages/vectors/meson.build:97–107`);
- `packages/vectors/src/maturity_corpus.rs`;
- `packages/vectors/src/maturity_evidence.rs`;
- `packages/vectors/src/maturity_negative_half.rs`;
- `packages/vectors/src/maturity_report.rs`;
- `packages/vectors/meson.build`.

**Shape:**

- **Importer (D6).** `MaturityCaptureTarget` on `MaturityCorpusPins`, the two new pins, `MaturityMutantCorpusPins`, `admit_maturity_mutant_corpus`, `ValidatedMaturityMutantCorpus`, the two accessors and `replay_maturity_mutant_run_of_record`.
- **Classification (D7).** `derive_maturity_evidence_plan_with` reads both accessors and applies `mutant_standing(row, refusal)`.
- **Register (D7).** Seven entries are removed; `wrong-control-recipe` is re-grounded; the doc line "forty-two" at `packages/vectors/src/maturity_negative_half.rs:285` and the §16.5 and §16.6 comments at `:384–386` and `:450–453` are restated to the rows that remain; the gap member documentation at `:143–163` drops the key-path witness, internal key, parity, control recipe and static subtree it no longer carries.
- **Report (D8).** Schema 2, `MaturityNativeRefusalRow`, the `native_refusal` block, and its recomputation in validation.

**Tests:**

| Test | File | What it asserts |
|---|---|---|
| `both_mutant_runs_admit_through_their_pins` (new) | corpus | each directory admits and its replay equals its evidence |
| `a_moved_byte_is_refused_at_its_layer_in_each_mutant_run` (new) | corpus | a changed name, size, manifest, report, capture hash, payload and request each refuse with its own member |
| `the_earlier_runs_admit_under_their_restated_targets` (new) | corpus | both existing archives admit with their pins' `maturity_native` target |
| `phase_walls_are_read_and_bind_nothing` (new) | corpus | the phase-wall lines parse and no admission layer compares them |
| `each_admitted_mutant_refusal_answers_its_row_at_its_declared_boundary` (new) | evidence | each of the seven records binds and stands `NativeRefusalObserved` |
| `an_unbound_refusal_leaves_its_row_waiting` (new) | evidence | a record failing carrier or subject leaves `NativeRunRequired` |
| `the_native_refusal_block_names_every_observed_refusal` (new) | report | the rendered block lists the seven rows in matrix order with their boundaries and layers |
| `the_validation_refuses_an_altered_native_refusal_line` (new) | report | a changed line fails validation naming its item |

The pinned tests of D7 move to the run's figures, and `the_witness_stands_on_the_discharged_rows_and_names_what_it_is_owed` and `the_completeness_is_partial_at_this_tip` add the refusal bucket to their sums.

**Counts:** corpus 42 → 46; evidence 22 → 24; report 21 → 23; negative half stays at 7. The census figures are D7's if every refusal binds, and otherwise the run's. Estimate: production 360–560 lines; tests 260–420; eight fixture files.

| Reading | Must be |
|---|---|
| `grep -c '#\[test\]' packages/vectors/src/maturity_corpus.rs` | 46 |
| `grep -c '#\[test\]' packages/vectors/src/maturity_evidence.rs` | 24 |
| `grep -c '#\[test\]' packages/vectors/src/maturity_report.rs` | 23 |
| `grep -c '#\[test\]' packages/vectors/src/maturity_negative_half.rs` | 7 |
| `grep -n 'MATURITY_SAFETY_REPORT_SCHEMA: u32' packages/vectors/src/maturity_report.rs` | `= 2` |
| `the_census_figures_are_unmoved_by_the_binding_readers` | 206 rows; native refusal observed 7; native run required 35; answered 78 |
| `grep -c 'mutant-run-of-record' packages/vectors/meson.build` | 8 |
| the full gate | green |
| `git diff --name-only` against the bite's base | exactly the thirteen files |

What this bite does not establish: nothing beyond D11. No plan file moves; the card, the register and the conjunct table belong to `T11-147`.

### Sequencing

`T11-148`, `T11-149`, `T11-150`, the ceremony `T11-151` and `T11-152`, in that order. Then `T11-146`, `T11-141` and `T11-147`. `T11-143`, `T11-144` and `T11-145` have landed on `0.6.294-dev`, `0.6.295-dev` and `0.6.296-dev`, and `T11-147` disposes their rows.

## 6. What this study does not establish

**No compilation, no run.** It establishes no compilation and no node verdict. Every type, test and changed line is checked by reading against the items that exist at `0.6.296-dev`: the visibility of each reused function, the witness layout, the covenant's clauses, and the adapter's submission path. That shows each shape is expressible over what the tree publishes. It shows nothing about whether the bites build, whether the gates accept them, whether any named test passes, or what words the node prints. D3's expected words are expectations, read from the adapter's vocabulary, the host oracle and the operator run of record on the same tip.

**Checks a bite makes first:**

- that `OperatorSigningRequest::freeze` admits a census whose output program or spent output differs from the construction the announcement's own signing opened over;
- that `substituted_successor`'s search finds a common admissible nonce under the runtime-asset link and `Second`'s subtree;
- that the variable-schedule source reaches the same host clauses as the whole-schedule sources the continuity tests use;
- that the continuity projector refuses the foreign-key mutant at `PredecessorProgram` rather than at an earlier comparison.

**The estate.** The estate fact of section 1 is stated, not observed: no node was contacted. The tree binds it after a run through the admitted capture.

**The open questions.** It decides none of OQ1 to OQ3. Each turns on a reading of the guide, the matrix's carrier vocabulary or the conjunct's words, which a ruling or the gate record makes.

**What the run cannot show.** It proves no general theorem, claims nothing about `wrong-control-recipe`, the leaf-version rows, the rows outside the eight or the internal key's discrete logarithm, and moves no conjunct. It mints no backlog row and edits no plan beyond this file, the directory's index and its census line. An amendment to the Wave-13 opening record carries the rulings, the rows and findings F1 to F12.
