# Research: concrete routes for the Keyed Provenance Covenant

**Abstract (Proposal status and scope)** · `abst:w1route:scope`

This research draft studies the target, constructor, evidence and semantic-boundary routes needed by Wave 1. It proposes, binds nothing, changes no rule and accepts no position. The execution guide's Wave-1 section remains governing, and the paper remains its contract; neither these schedules nor their proposed tests are completed observations. The companion strategy orders the work without promoting a route into acceptance (`task:guide15-exec:wave1`), (`gate:guide15-exec:wave1`), (`[A-def:kpc-model:first-output]`), (`sec:wave1:execution`), (`rule:drafts:authority`).

**Convention (Source readings and proposed files)** · `conv:w1route:sources`

Code citations identify the inspected base by path and line; the drafts index records the authored-at patch tag. Facts describe existing code, while proposals describe future changes and evidence. Proposed module names are delivery homes to test against package boundaries, not existing APIs. All negative subjects belong to the repository's own conformance fixtures and name the boundary at which an answer is sought (`rule:guide15-exec:finding-boundary`).

## 1. Born first · `sec:w1route:birth`

**Fact (Two different indices)** · `fact:w1route:outpoint`

`InspectInputOutpoint` consumes an input index and pushes transaction identity, four-byte output index and flags, in that order (`packages/target-elements/src/opcode.rs:1001`). The index is fixed-width little endian (`packages/target-elements/src/encoding.rs:321`). `PushCurrentInputIndex` selects the input being validated (`packages/target-elements/src/opcode.rs:1162`). Both capabilities are Reviewed, which means source-reviewed rather than node-observed (`packages/target-elements/src/capability.rs:13`, `packages/target-elements/src/capability.rs:502`, `packages/target-elements/src/capability.rs:510`). The primitive census establishes multi-item behavior without pinning deployment-dependent outpoint bytes (`packages/target-elements-conformance/src/census/introspection.rs:12`, `packages/target-elements-conformance/src/census/introspection.rs:90`).

**Proposal (Birth fragment)** · `proposal:w1route:birth-fragment`

Emit the schedule below as a typed fragment with an empty initial and final stack. Compare the index as bytes, not as a ScriptNumber: numeric zero's empty encoding is not the required four zero bytes. The inspected flags are discarded, not compared with zero; a flags restriction would add a third covenant condition. The separate target flag field already has a typed weld for issuance reporting (`packages/target-elements/src/weld.rs:498`). Validate the proposed schedule against the reviewed stack contracts before native execution (`packages/tapscript/src/stack.rs:539`), (`[A-def:kpc-model:first-output]`).

```text
PushCurrentInputIndex
InspectInputOutpoint
Drop
PushBytes(00 00 00 00)
EqualVerify
Drop
```

**Argument (The proposed stack reading)** · `arg:w1route:birth-stack`

After inspection the deepest-to-top items are txid, birth index and flags. Dropping flags exposes the index; byte equality consumes it and the four-byte constant, and the last Drop removes txid. The fragment leaves no Boolean. A standalone experiment appends true for the target's final success rule; the full leaf appends true only after both clause fragments. Stack rearrangement and byte equality are Reviewed (`packages/target-elements/src/capability.rs:853`, `packages/target-elements/src/capability.rs:871`). This derivation is an implementation argument awaiting the target control/refusal evidence, not an observation.

**Fact (Funding routes differ)** · `fact:w1route:birth-funding`

The primitive convenience `fund` puts its test output at position 0, change at 1 and fee at 2 (`scripts/elements-native-executor.py:2882`). The operation `issue` and `pay` routes create consecutive same-program outputs and report their outpoints (`scripts/elements-native-executor.py:3870`, `scripts/elements-native-executor.py:3939`, `scripts/elements-native-executor.py:3981`, `scripts/elements-native-executor.py:4027`). The latter requires an asset reserve issued earlier in the same run; arbitrary native-asset payment through it is not established. Complete transaction submission is already supported (`scripts/elements-native-executor.py:8042`).

**Proposal (Four birth-position subjects)** · `proposal:w1route:birth-subjects`

Fund two equal-value, same-program outputs at positions 0 and 1 in one creating transaction, then offer their spends as distinct subjects. Use fresh equivalent funding for each current-input placement so successful controls do not consume another row's coin. Keep two inputs in every spending form: one covenant input and one independently authorized funding input. Exchange their ordering to place the covenant at current input 0 or 1; recompute funding signatures and transaction identity as declared collateral. Keep the successor at output 0, with identical program, amounts, asset and fee policy across each control/negative pair. Fix the funding input's exact outpoint, including its own birth position, within each pair; choose that position nonzero for A2/A3 so accepted A2 alone refutes inspection of input 0 (`expt:guide15-exec:measurement-a`).

| Subject | Control / single semantic coordinate | Expected observation boundary | What remains absent |
|---|---|---|---|
| Birth A0 | Spend birth-position 0 at current input 0 | Accepted script spend, plus separately recorded relay/consensus outcome | No claim about other input indices from this row |
| Birth A1 | Against A0, spend the otherwise matching birth-position 1 output at current input 0 | Born-first EqualVerify refusal at script execution | No record reading for the later output |
| Birth A2 | Spend fresh birth-position 0 control at current input 1 | Accepted script spend | No requirement that covenant inputs occupy input 0 |
| Birth A3 | Against A2, spend matching birth-position 1 output at current input 1 | Born-first EqualVerify refusal | No inference from an unrelated funding/signature refusal |

**Caveat (Mutation identity and flags)** · `cav:w1route:birth-collateral`

The changed outpoint and resulting transaction identities are unavoidable encodings of the changed birth position, not additional semantic mutations. The parent transaction bytes authenticate both output positions. Initially use ordinary non-issuance funding; do not claim that these four rows test every flag combination. Add an optional birth-position-0 control whose covenant input carries issuance, with required balance/signature changes declared as collateral, to exercise index/flag separation. Read and bind the pinned target's index/flag separation before admitting the experiment. If another admitted input form changes the outpoint interpretation, add its own control and exact target fact rather than silently masking bits or banning that form. In particular, if the deployment admits peg-ins and exposes the claim program as the spent program, a peg-in under the keyed program is no handle: include that reading-family subject, and classify an event consuming no handles as SOURCE even if the peg-in passes both clauses. A missing authentic birth-position route blocks the consuming bite (`gate:guide15-exec:bite-1-0-gate`), (`[A-def:kpc-model:keyed-membership]`).

## 2. Continue first · `sec:w1route:continuation`

**Fact (Authenticated equality already has a schedule)** · `fact:w1route:program-equality`

`require_program_matches_this_input` reads the current input's program and an indexed program, then compares version and payload with two verifying equalities (`packages/tapscript/src/pattern.rs:741`). Here payload means the `WitnessProgram` item, and output program means witness version plus payload (`packages/target-elements/src/encoding.rs:170`). Introspection returns witness payload with version, or a script digest with a non-witness marker (`packages/target-elements/src/opcode.rs:820`). The ASH successor wrapper adds asset, explicit-value and amount-domain checks (`packages/tapscript/src/pattern.rs:823`); those are not part of this paper's whole covenant (`[A-def:kpc-model:first-output]`).

**Proposal (Extract only the equality)** · `proposal:w1route:continuation-fragment`

Give the authenticated comparison a target-generic fragment entry in `packages/tapscript/src/`, or a contract-local wrapper over a shared helper, selected by the boundary test. Keep the existing ASH wrapper and its tests intact. The new fragment compares output 0 against the executing input, with no caller-supplied reference script, asset comparison, value comparison or coordinator-index condition. For the selected witness-version-one constructor, matching payload and version identifies the complete output program (`expt:guide15-exec:measurement-b`). Equality means the same keyed covenant because each handle authenticates the published recipe for its key; an identical leaf under a foreign internal key or larger tree continues that foreign program and neither enters nor receives continuation from the key's handles (`[A-def:kpc-model:keyed-membership]`).

```text
PushCurrentInputIndex
InspectInputScriptPubKey
PushScriptNumber(0)
InspectOutputScriptPubKey
Rotate
EqualVerify
EqualVerify
```

**Proposal (Continuation subject matrix)** · `proposal:w1route:continuation-subjects`

First capture wrong-program, wrong-version and displaced-successor subjects on a single-key leaf, with an accepted birth-position-0 input and identical output-0 program; this capture is key-free in that it makes no key-separation claim. After the constructor step supplies its prototype two-key recipe, capture the keyed control under k and the wrong-key, mixed-key and coexistence rows. Refused transactions precede their same-coin control where possible; accepted rows consume distinct equivalent coins. Every mutation below changes one declared successor coordinate, with transaction identities, fee balancing and unrelated signatures recorded as collateral. Stage negative bytes below safe typed construction so a construction refusal cannot be mislabeled a native covenant refusal (`task:guide15-exec:bite-1-3-deliverables`).

| Subject | Changed coordinate from control | Expected boundary / attribution |
|---|---|---|
| Wrong key | Replace output-0 program by the same recipe under a distinct fixed key | Payload equality refuses; constructor tests separately establish key separation |
| Wrong program | Keep the single key but replace the successor by a deliberately different leaf recipe's valid output program | Payload equality refuses; this is not a second production constructor |
| Wrong version | Keep the same 32-byte payload but encode output 0 with witness version 0 instead of 1 | Version equality refuses if ordinary target form admission permits the output; an earlier form refusal remains a separate finding |
| Displaced successor | Exchange output 0 with an existing different-program output, keeping the genuine successor at position 1 | Output-0 comparison refuses; record the exchange as one position permutation |
| Mixed keys | Consume birth-position-0 handles under k and l, choosing output 0 under k | The l input refuses; use a same-key two-input control and valid funding |
| Independent coexistence | Separate accepted transactions for k and l on one history | Both accept; no mixed-key continuation claim |

**Caveat (Independent clause attribution)** · `cav:w1route:clause-attribution`

Wrong-key and wrong-program rows can reach the same equality instruction while answering different subject claims. Record the exact site and shared-site limitation; do not invent opcode attribution from an undifferentiated script error. Birth negatives keep successor equality valid; continuation negatives keep birth position valid. If version or displaced-output subjects fail before script, retain that observation and repair the test subject until the declared clause boundary is reachable, or stop that obligation (`req:guide15-exec:gate-record-evidence`).

## 3. Keyed constructor and path closure · `sec:w1route:constructor`

**Fact (The existing reference alternatives are unequal)** · `fact:w1route:reference-strategies`

`SelfCommitmentStrategy::IdentityIntrospection` avoids a self-referential literal and checks that no contradictory literal remains. `NotStated` refuses. `ExternallyAuthenticatedCommitment` carries `SelfCommitmentEqualityUndischarged`; its documentation says that a literal inside the leaves committing to that same program is not a completed resolution (`packages/linker/src/deployment.rs:40`). Existing linker tests check sound introspection, contradictory literals and outstanding equality (`packages/linker/src/tests/link_tests.rs:100`, `packages/linker/src/tests/link_tests.rs:122`, `packages/linker/src/tests/link_tests.rs:180`).

**Proposal (One public keyed leaf)** · `proposal:w1route:key-recipe`

Represent the semantic key as an opaque fixed 32-byte public value, not a signing key. Commit it in one canonical tapscript leaf as `PushBytes(key); Drop`, followed by born-first, continue-first and a final true. Use the reviewed leaf version and one fixed NUMS internal-key policy; no metadata leaf, alternate operation leaf, owner-signature path or generation transition is proposed. The leaf literal changes the tapleaf and resulting output program while adding no spending predicate. The constructor descriptor publishes key bytes, leaf bytes/version, internal-key derivation, tree shape, output program and control recipe. Exact domain separation and encoding are part of the proposed recipe to ratify after two-key construction evidence, not an existing target guarantee (`[A-def:kpc-model:keyed-membership]`), (`task:guide15-exec:bite-1-3-deliverables`).

**Argument (Why current-input introspection is preferred)** · `arg:w1route:recurrence-choice`

Both clauses can execute from authenticated transaction facts; the output-key literal need not occur inside its own leaf. The fixed semantic key is ordinary committed data and is not that recursive output-key literal. This reuses a demonstrated equality schedule and the introspection principle: the keyed leaf removes the self-reference edge rather than resolving a cycle, while avoiding the STATE dynamic-metadata constructor. `LinkDeploymentParameters` carries `CompactAshSymbols`, so a keyed linker parameter form is a bite-1.4 seam (`packages/linker/src/deployment.rs:84`). A reference strategy becomes necessary only if the selected target cannot authenticate the current program or the selected constructor requires a different authenticated relation; either finding reopens the proposal and needs a demonstrated closure strategy, not a supplied value with unresolved equality (`fact:w1route:reference-strategies`), (`proposal:w1route:continuation-fragment`).

**Proposal (Complete path census and controls)** · `proposal:w1route:path-census`

The proposed bundle lists exactly one script leaf containing both clauses and the single taproot key path. Public reconstruction under k must reproduce identical leaf, root, output key and control bytes; reconstruction under l must differ in output program. Authenticate each control block against the actual funded predecessor. A fresh process must reconstruct without creator state. Propose a tapscript-owned keyed curve-capability interface, following `StateCurveCapability`, whose supplied implementation computes the tweak and point sum from the public descriptor (`packages/tapscript/src/state_constructor.rs:27`); the transaction crate deliberately computes neither (`packages/transaction/src/taproot.rs:16`). Reuse public leaf/branch hashing and control machinery, not the private STATE hash helper or metadata law (`packages/transaction/src/taproot.rs:99`, `packages/transaction/src/taproot.rs:113`, `packages/transaction/src/taproot.rs:126`, `packages/tapscript/src/state_constructor.rs:787`), (`gate:guide15-exec:bite-1-4-gate`).

| Subject | Control and changed coordinate | Boundary / limit |
|---|---|---|
| Repeated public construction | Same key and complete public descriptor in unrelated process | Exact-byte equality; no secret required |
| Distinct key | Change only public key bytes, regenerate dependent commitments | Output programs differ; finite test plus second-preimage resistance of the target's tagged SHA-256 chain for a generated key, and collision resistance where both keys are adversarial, not universal injectivity proof |
| Foreign internal key offered | Same descriptor except supplied internal key | Constructor-policy refusal against canonical NUMS control; no claim that an unrelated output with a spendable key belongs to this system |
| Wrong control parity | Flip only parity bit in an otherwise correct spend control | Target commitment refusal; both clause bytes unchanged |
| Uncommitted leaf | Offer a modified leaf under the original control block | Target commitment refusal before its clauses can count as executed |
| Extra leaf in descriptor | Add an alternate leaf to the one-leaf recipe | Typed constructor census refusal; structural proof accounts for the full committed tree |
| Key-path attempt | Offer a key-path witness against the same funded constructor | Native key-path refusal, separately classified; no proof of absent discrete logarithm |

**Caveat (Conditional closure and semantic recognition)** · `cav:w1route:constructor-assumptions`

The NUMS argument retains discrete-log hardness and SHA-256 preimage resistance under the guide's conditional standing; finite failed key-path attempts do not prove impossibility (`judg:guide15-exec:nums-standing`). Key separation additionally assumes second-preimage resistance of the target's tagged SHA-256 chain for separation from a generated key, and collision resistance where both keys are adversarial; these are proposed premises for acceptance alongside, not consequences of, the guide's preimage premise (`assum:guide15-exec:sha256-preimage`). Reader membership must authenticate the registered public recipe, not infer it from a key-shaped literal in an arbitrary script. If a key change leaves the output program equal, or a permitted path bypasses either clause, the proposal fails rather than acquiring an extra authorization condition (`rule:guide15-exec:wave1-failure`).

## 4. Cost, standardness, funding and scaling · `sec:w1route:resources`

**Fact (Available observations)** · `fact:w1route:resource-observations`

The typed stack engine validates schedules and projects resources (`packages/tapscript/src/stack.rs:539`, `packages/tapscript/src/stack.rs:700`, `packages/tapscript/src/stack.rs:1714`). Native `resources_for` reports transaction weight from the node and restates script length/initial item count from the fixture; stack peaks, largest element and validation budget remain null (`scripts/elements-native-executor.py:6920`). The earlier STATE prototype's 699 script bytes, 924 witness bytes and 36/36 native rows describe another constructor, with no retained repository report for that historical run (`tab:state-constructor:resources`), (`sec:state-constructor:evidence`).

**Proposal (Measurement d packet)** · `proposal:w1route:cost`

Measure isolated birth, isolated continuation and the complete keyed leaf on exact bytes. Record static instruction/resource predictions, linked script bytes, witness serialization, control depth and complete node-read weight, separating per-input repetitions from fixed constructor data. Compare predicted versus observed only where both share a dimension and definition. Keep native stack peaks and validation budget absent; static estimates occupy separate fields. Focused oversized witness and altered resource-report subjects test target policy or report validation, respectively, not the covenant's two clauses (`expt:guide15-exec:measurement-d`).

**Fact (Zero and fees have different owners)** · `fact:w1route:funding-boundaries`

Explicit zero spendable outputs are consensus-refused (`packages/target-elements/src/transaction_form.rs:456`). `FeeOutputContract` recognizes the fee by empty program and explicit fields, while form reviews separate consensus and relay (`packages/target-elements/src/transaction_form.rs:191`, `packages/target-elements/src/transaction_form.rs:288`). Wave-0 sponsored weights 2,544 and 2,809 yielded fees 128 and 141 under their measured policy; sponsorless controls were direct-block accepted after relay refusal. Those STATE measurements do not establish a keyed-handle relay floor (`rec:backlog:wave0-sponsored-observation`).

**Proposal (Measurement e funding route)** · `proposal:w1route:funding`

Prefer positive explicit policy-asset handles, ordinary default relay and an explicit fee output for the first candidate. Obtain native-asset sponsor coins through existing sponsor funding, then construct and submit a SOURCE paying the proposed keyed program at output 0; the resulting readback supplies its exact spendable outpoint. MERGE consumes these public handles and may add funding inputs. The generic `pay` reserve route can stage the isolated birth experiment in a separately declared asset, but cannot be assumed to fund arbitrary native-asset handles. Demonstrate the preferred route before claiming no adapter extension is needed (`scripts/elements-native-executor.py:3981`, `scripts/elements-native-executor.py:4351`, `scripts/elements-native-executor.py:4717`), (`expt:guide15-exec:measurement-e`).

**Proposal (Funding and standardness comparisons)** · `proposal:w1route:funding-subjects`

Freeze the policy asset, fee policy, constructor and one accepted funded control. Vary the handle amount around a discovered positive admission boundary while keeping a sufficient fee fixed and balancing ordinary change; vary fee around the reported floor while keeping the handle funded. Preserve complete bytes, node policy readings, relay result and separately attempted consensus result for each point. If no positive output-amount threshold is found beyond consensus positivity, report the tested range and no discovered threshold rather than inventing dust. A changed policy, program size or input count reopens the floor result (`expt:guide15-exec:measurement-e`).

**Proposal (Open outputs and confidential collateral)** · `proposal:w1route:open-outputs`

Include accepted controls with unrelated ordinary outputs and with value redistributed away from the consumed handle into fees or other outputs while the successor remains target-valid. These distinguish two-clause faithfulness from accidental value preservation or output closure. If unrelated confidential outputs are admitted by the chosen construction surface, build the balanced proofs and obtain their own control; a corrupted proof then tests target proof validation, not keyed membership. The existing confidential sponsor route is precedent for balance and exact readback, not automatic coverage of arbitrary carrier outputs (`rec:backlog:wave0-confidential-route-admission`), (`[A-def:kpc-model:first-output]`).

**Proposal (Measurement f range)** · `proposal:w1route:scaling`

Measure SOURCE with zero covenant inputs and MERGE with one, two, representative interior and proposed maximum covenant inputs, including worst admitted funding witnesses, fee funding and other permitted output forms. Select N only after complete weights and policy results are retained. The typed constructor refuses N+1 as construction policy beside an admitted N control; this refusal is not a covenant count condition. For resource limits, compare a just-admitted complete transaction with its nearest stageable overweight neighbor at the target's actual boundary. Derive any scaling formula from the measured subject family, retain mismatches and disclose dimensions never observed (`expt:guide15-exec:measurement-f`).

**Caveat (No hidden range restriction)** · `cav:w1route:range`

The adapter funding request accepts 1–255 outputs (`scripts/elements-native-executor.py:8132`); this is not the covenant's input maximum. Many handles may require several SOURCE creations rather than one multi-output transaction, because later covenant-shaped outputs are not handles. Therefore the scaling campaign creates N distinct output-0 handles and cannot count N same-program siblings as N valid inputs. Record preparation cost separately from MERGE cost (`[A-prop:kpc-model:single-birth]`).

## 5. The semantic home and independent model · `sec:w1route:semantic-home`

**Fact (The current boundary is concretely typed)** · `fact:w1route:semantic-seams`

`OperationId` and `ObjectId` enumerate attestation operations and objects (`packages/architecture/src/ids.rs:99`, `packages/architecture/src/ids.rs:118`). `validate_expected_id_coverage` requires every registered asset, root, object, operation and other census member (`packages/architecture/src/validate.rs:620`). `DOCUMENT` binds the attestation anchor set (`packages/architecture/src/spec.rs:404`). Realization dispatch has only compact ASH, live transfer and maturity announcement (`packages/realization/src/declarations/mod.rs:11`). Compiler `bind_input` consumes concrete `ScopedRealizationSpec` and architecture IDs and revalidates them (`packages/compiler/src/input.rs:11`, `packages/compiler/src/input.rs:171`). `DeploymentProfile` is a network/evidence profile, not a semantic selector (`packages/architecture/src/deployment.rs:175`).

**Table (Three options and their costs)** · `tab:w1route:semantic-options`

The following change footprints are engineering inferences from the cited seams, not completed implementations or permitted weakening of the first contract (`rec:guide15-exec:study-dispositions`).

| Option | Changes and reuse | Cost / disconfirming evidence |
|---|---|---|
| Another declaration set | Add typed SOURCE/MERGE/handle identities and declarations in architecture/realization; reuse graphs, validators and compiler input | Current all-ID census requires attestation ballast. If a standalone second manifest needs selective census and binding, this has become a semantic profile rather than dispatch-only reuse |
| Explicit semantic profile | Add profile-owned vocabulary/census/specification binding and validation in architecture; profile-aware realization dispatch/relations and compiler input; reuse existing graph/proof/placement engines | More than an operation arm, but avoids a second semantic framework. Refute on a new profile/keyed-operation conditional outside the named owner seams of the boundary test or failure to isolate first-contract validation; count compile-forced relation arms separately |
| Separate realization crate with shared typed compiler input | Add crate/workspace/Meson/contract registrations; extract common validated semantic vocabulary and input boundary; retain owner-specific validators | Clean separation but extraction still must remove concrete attestation ID/census assumptions. Prefer only if the boundary test shows profile isolation cannot stay at semantic ownership boundaries |

**Proposal (Test a semantic profile first)** · `proposal:w1route:semantic-profile`

Recommend an explicit keyed-contract semantic profile in the existing architecture and realization packages, with exact typed keyed operations and handles, profile-owned census and paper binding. It is the smallest candidate that acknowledges the demonstrated all-ID barrier without inventing STATE/receipt objects or another crate before extraction is justified. Do not reuse `DeploymentProfile` as the selector. Trace changes through `packages/architecture/src/ids.rs:99`, `packages/architecture/src/spec.rs:289`, `packages/architecture/src/validate.rs:620`, `packages/realization/src/operation.rs:13`, `packages/realization/src/validate.rs:17` and `packages/compiler/src/input.rs:171`; record schema/canonical identity impacts before accepting it. The recommendation is conditional on the next boundary test (`task:guide15-exec:bite-1-1-deliverables`).

**Proposal (Boundary test before semantic-home commitment)** · `proposal:w1route:boundary-test`

Decide identifier representation as the probe's first coordinate: shared `OperationId`/`ObjectId` members require profile-scoped census and exhaustive-match changes, while profile-scoped identifiers require corresponding `Relation`, `OperationRealization` and `bind_input` types (`packages/realization/src/relation.rs:5`, `packages/realization/src/relation.rs:49`, `packages/realization/src/operation.rs:13`). Record a baseline census of existing operation-keyed conditionals in analysis, proof, placement and coverage, starting with `CompactAsh | TransferLive` in `packages/compiler/src/placement.rs:266`. The named owner seams are the architecture validation entry (`packages/architecture/src/validate.rs:194`), `derive_operation` (`packages/realization/src/declarations/mod.rs:11`) and `bind_input` (`packages/compiler/src/input.rs:171`). A new conditional keyed on profile or keyed-operation identity outside those seams refutes the profile; count new `Relation` variants and their compile-forced discharge arms separately as relation vocabulary (`packages/compiler/src/placement.rs:285`).

Construct the smallest standalone keyed manifest and two declarations, with no roots, receipts, attestation amount law or covenant authorization. SOURCE has no covenant input and produces singleton evidence; MERGE has one-or-more handles with both clauses per consumed member and reader-derived union. Revalidate through the owner's validator and existing compiler input/analysis boundary. Require an explicit evidence obligation for SOURCE classification and reading, never a synthetic execution carrier. Corrupt the profile binding, remove a clause, duplicate a relation, introduce a foreign operation and move a reader relation to a runtime carrier; each must refuse at its named owner. Re-run first-contract derivation/census/identity regressions and retain every intentional schema change. Apply the conditional-delta test above; if it refutes the profile, compare the shared-input extraction alternative before stable declarations land (`gate:guide15-exec:bite-1-1-gate`), (`gate:guide15-exec:bite-1-2-gate`).

**Proposal (Independent keyed-history oracle)** · `proposal:w1route:model-home`

Place a finite public-history reference model in a proposed `packages/vectors/src/keyed_history_model.rs`, separately from the compiler-facing declarations and production reader. It defines output identities, immutable events, keys, birth positions and single consumption directly from the paper; SOURCE singleton and MERGE union are computed in dependency order. It imports neither the realization evaluator nor compiler plans to decide expected results. A sibling reader consumes public byte-derived events and evidence and is compared with that oracle. Propose a vectors source-boundary test `keyed_history_oracle_import_boundary` that refuses imports of realization, compiler, linker, transaction and the production reader in the oracle module, including aliases and fully qualified references. This is bite-1.5 work that may be prepared early, not a bite-1.1 deliverable. It avoids forcing a second contract into the attestation model's sealed transition vocabulary (`packages/model/README.md:3`, `packages/model/README.md:42`), while preserving independent model conformance (`setup:guide15-exec:bite-1-5-files-and-packages`).

**Criterion (What would move the model)** · `crit:w1route:model-extraction`

The proposed vector-owned oracle is suitable only while its consumers are conformance fixtures and it remains independent of declaration evaluation. A runtime consumer requiring a maintained general history API, or inability to keep the oracle independent in that package, reopens a separate model crate; mere reuse of public identifier encodings does not justify copying the declaration algorithm. Missing dependencies produce incomplete, conflicting identities/cycles invalid, and no set is returned for later-shaped outputs (`[A-sec:kpc:interface]`).

## 6. Formally labelled sub-steps for bite 1.0 · `sec:w1route:sub-steps`

**Proposal (Sub-step order and packets)** · `proposal:w1route:step-order`

After or alongside the semantic boundary test, prepare the exact target/policy manifest, then stage each experiment, capture it and admit its evidence. The labels below name proposed bounded units, not new gates. Birth and key-free continuation preparation/capture can run together on a single-key leaf; constructor capture follows their reviewed contracts. The keyed control and wrong-key, mixed-key and coexistence rows consume the constructor step's prototype two-key recipe in a second continuation capture, before cost/standardness/scaling. Final evidence is repeated by bite 1.6, even if prototype bytes happen to match (`task:guide15-exec:bite-1-0-deliverables`), (`task:guide15-exec:bite-1-6-deliverables`).

| Step | Input and files | Evidence / stop predicate |
|---|---|---|
| (`step:w1route:target-packet`) | Wave-0 target binding; vectors campaign manifest and target assessment readers | Exact target, policy, subject schema, coordinate census and available observation fields; freeze exact bytes per experiment once its fragment exists; missing binding stops capture |
| (`step:w1route:birth`) | Reviewed introspection; proposed tapscript fragment and vectors birth subjects | A0–A3, flags/encoding reading, exact parent/spend bytes; authentic birth evidence required |
| (`step:w1route:continuation`) | Equality helper; single-key leaf, tapscript comparison and vectors successor subjects | Key-free first capture: accepted control plus wrong program/version/position; no key-separation claim or ASH predicates |
| (`step:w1route:constructor`) | Two-key public descriptor; tapscript/linker reference checks and vectors paths | Reproducibility, full path census, conditional key-path closure; unresolved equality stops |
| (`step:w1route:keyed-continuation`) | Constructor step's prototype two-key recipe; vectors keyed subjects | Second capture: keyed control, wrong key, mixed keys and independent coexistence |
| (`step:w1route:cost`) | Composed keyed leaf and resource predictions | Exact emitted/witness/control figures and native weight; unavailable native dimensions typed absent |
| (`step:w1route:funding`) | Exact SOURCE/MERGE layouts; target forms, vectors funding/fee subjects | Default-policy controls and boundary measurements, separate consensus/relay, balanced proofs where claimed |
| (`step:w1route:scaling`) | Accepted funded form and candidate input range | Distinct SOURCE handles, complete range costs, provisional N and funding choice |
| (`step:w1route:admission`) | Six complete records and exact controls | Validated measurement packet in research/backlog homes; unresolved primitive/form blocks consumer |

**Step (Bind the target and subject packet)** · `step:w1route:target-packet`

Bind reviewed target/deployment, executable provenance, protocol/executor, policy settings, subject schema and coordinate census to the inherited entry. Freeze exact bytes, controls, changes, collateral, declared boundaries and missing dimensions per experiment once its fragment exists and before its native run. This releases preparation, not an acceptance claim (`rec:backlog:wave0-entry`), (`gate:guide15-exec:bite-1-0-gate`).

**Step (Stage and observe birth position)** · `step:w1route:birth`

Deliver the typed fragment and four subjects of (`proposal:w1route:birth-subjects`), record the target's authentic index/flags interpretation and admit each control/refusal pair independently. Stop if the later output cannot reach the clause boundary with an accepted matching control.

**Step (Stage and observe key-free continuation)** · `step:w1route:continuation`

Deliver the equality-only extraction and first capture on a single-key leaf: wrong program, wrong version and displaced successor beside its accepted control (`proposal:w1route:continuation-subjects`). A native earlier-layer refusal is retained but cannot close the claimed script row. Keep preexisting ASH behavior regression-tested; key-dependent rows wait for the constructor step.

**Step (Close the keyed constructor)** · `step:w1route:constructor`

Deliver the two-key public recipe, independent reconstruction and complete path inventory of (`proposal:w1route:path-census`). Compose both clauses; require the key changes only committed identity and introduces no authorization or value law.

**Step (Stage and observe keyed continuation)** · `step:w1route:keyed-continuation`

Use the constructor step's prototype two-key recipe for the keyed control and wrong-key, mixed-key and independent-coexistence capture (`proposal:w1route:continuation-subjects`). Join this second capture to the first without promoting its single-key evidence into a key-separation claim.

**Step (Measure composed costs)** · `step:w1route:cost`

Deliver the comparison packet of (`proposal:w1route:cost`), separating static predictions, serialized quantities, node observations and unavailable fields. Attribute per-input versus fixed overhead on the exact composed constructor.

**Step (Measure funding and policy boundaries)** · `step:w1route:funding`

Deliver (`proposal:w1route:funding-subjects`) under the preferred explicit default-relay route or a separately justified replacement. No funding number is accepted by this draft; a failed positive control holds the consuming plan and ABI open.

**Step (Measure scaling and propose the bound)** · `step:w1route:scaling`

Deliver (`proposal:w1route:scaling`) with distinct valid handles, complete funding/witness cost and admitted endpoints. State the proposed bound and its evidence envelope; no carried history or semantic count condition is introduced.

**Step (Admit the provisional measurement set)** · `step:w1route:admission`

Validate all six families of target measurement records against exact subjects and controls, publish funding/N recommendations and explicit absent dimensions, and retain the distinct keyed set in `plans/research/state-constructor.md`. The consuming backlog record links each disposition; public-representation findings link to (`sec:public-opening:measurements`), while gate performance stays at (`tab:test-performance:register`). This is the assigned measurement home, not permission to claim observations before they occur (`obl:guide15-exec:measurement-home`).

## 7. Ambiguities and readings · `sec:w1route:readings`

**Table (Readings that need acceptance or evidence)** · `tab:w1route:ambiguities`

| Ambiguity | Recommended reading and reason | Evidence that reopens it |
|---|---|---|
| Birth index versus current input | Authenticate the spent outpoint index; exercise current indices 0 and 1 | Pinned target returns an encoding/flag interaction inconsistent with the proposed byte comparison |
| Flags | Discard the separate flags field; do not impose zero flags | Target facts show a different authentic extraction is necessary for an admitted form |
| Complete script comparison | Compare version plus witness payload, using the actual executing input | A selected representation needs more than the reviewed pair |
| Fixed key | Opaque 32-byte public constant committed in the sole leaf; not a signature requirement | Collision/aliasing, incompatible recipe domain or bypass path discovered |
| Native funding | Sponsor-funded typed SOURCE for policy-asset candidate; issued reserve only for explicitly scoped prototypes | Positive default-policy control cannot be built through existing routes |
| Number of input handles | N distinct SOURCE output-0 handles; no later sibling counts | No overturn within the paper; a contrary count is a fixture error |
| Semantic home | Profile first, separate crate if isolation fails; no fake attestation objects | New profile/keyed-operation conditional outside the named owner seams against the baseline census; count relation vocabulary arms separately |
| Independent model | Vector-owned finite oracle separate from reader and declaration evaluator | A genuine runtime consumer or independence failure needs extraction |
| Exact relay minimum and N | Remain unknown until measurements; choose no inherited numeric default | New bytes, witness envelope or policy reopen an observed selection |
| Finality depth | Publish a named realization assumption before claiming final; nonfinal reads remain distinct | Replacement-history policy or evidence assumptions change |
