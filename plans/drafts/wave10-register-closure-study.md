# Study: Wave 10's open register clauses and premise defects and their bites

Status: a design study over the tree at `0.6.266-dev`, written because the Wave-10 closure record at (`sec:phase6:wave10-findings`) stops on three code findings and three open register clauses, and a bite cannot be cut against a finding whose repair is described in one sentence or a clause whose closing evidence names artifacts no type in the tree carries. Findings 164 and 165 are defects in the root-history checkpoint's two declared premises, finding 166 is the creator-side projection of the root-history edge on which §23's eleventh conjunct at (`gate:guide14-exec:exit`) stays conditional, and the open clauses are `G14C-11`, `G14C-13` and Wave 10's clause of `G14C-12` in `plans/guides/guide_fourteen_conceptual_register.md`. Every figure below is recounted from the source at `0.6.266-dev` or read from the assertion that pins it; nothing here was obtained by running a test, and no line of production code is changed by this file. It answers eight design questions as numbered decisions D1 to D8, adds two decisions for the stale comments and the backlog rows, leaves two questions with the guide's text, and manufactures seven code bites and one closure row, each with its exact shapes.

## 0. In one breath

The three findings are smaller than the exit they block and the three clauses are further along than their Status cells read: the evidence derivation already replays the accepted archive through the variable-metadata planner and projects a continuity whose own bundle is the premise the checkpoint should declare, the ABI over that bundle's view is two existing calls away, the public handoff already carries the five published parameters a declared bundle can be checked against, the recovery process already reads the two outpoints a root edge carries from the published bytes, the recovery report's residual vocabulary already has the form a typed non-claim takes, and the modelled prefix already rewinds, extends and stales an observation. What is absent is a binder that checks the declared premises against anything public, a report item that reads their provenance, an edge built from the handoff, five non-claims, and the branch-indexed layer the register rows name: realizations of edges on modelled branches, the typed invalidation a rewind produces, semantic history held outside any branch, a compare-and-swap token fenced at submission, and the class-2 standings a thread takes when the environment moves. The bites therefore repair the checkpoint first — declaring the bundle and ABI that produced the accepted bytes and binding them against the public handoff rather than the creator-side continuity — then build the edge from the handoff alone, type the five recovery non-claims, add the branch layer and its class-2 observations over modelled branches, and last retype two stale comments; all three clauses can close on that evidence because their criteria name model operations and report bytes, and each closing pointer states that no chain was observed.

## 1. The problem, exactly

§17.2 binds ten facts, the last two the linked candidate and the candidate ABI, and states that "A reorg that removes or changes the transaction stales the observation" (`plans/guides/guide_fourteen.md:2731–2748`). The landed checkpoint carries those two as `MaturityDeclaredPremise` values (`packages/vectors/src/maturity_history.rs:481–482`), and its constructor stores either member of the two-member `MaturityPremiseProvenance` without reading it (`packages/vectors/src/maturity_history.rs:596–637`, `packages/vectors/src/maturity_corpus.rs:140–145`). The history report's renderer writes `checkpoint_linked_candidate declared deployment-declaration` and `checkpoint_candidate_abi declared deployment-declaration` as constants (`packages/vectors/src/maturity_history_report.rs:586–593`), and none of its nineteen items compares a premise (`:116–136`): a checkpoint whose premises were declared from executor arguments validates and renders byte-identically. That is finding 164 (`plans/phases/06-state-and-maturity.md:1028`).

The evidence derivation declares, as those premises, the plan's bundle linked under the retained whole-metadata selection and the ABI derived over that bundle's view (`packages/vectors/src/maturity_evidence.rs:1987–1995`, `:2014–2026`), while the edge's continuity is projected from the replayed planner's bundle under the retained variable-metadata selection (`:1885–1890`, `:1948`). `binds` asks three questions — identity, successor, predecessor — and compares neither premise (`packages/vectors/src/maturity_history.rs:644–677`), and `the_checkpoint_binds_eight_facts_and_declares_two` asserts that the declared ABI's schedule differs from the continuity's while the checkpoint binds (`:1124–1128`). The premises the history report validates over are not the accepted transaction's. That is finding 165 (`plans/phases/06-state-and-maturity.md:1030`).

`StateRootEdge::from_continuity` takes a validated continuity (`packages/vectors/src/maturity_history.rs:114`), whose projection input carries the retained bundle and the caller's deployment identity (`packages/vectors/src/maturity_continuity.rs:205–218`) and whose `retained_context` refuses unless the bundle's first retained instance and that identity agree with the witnessed bytes (`:680–690`). No edge is built from the public handoff, so §23's eleventh conjunct — "exactly one STATE root edge is independently recovered" (`plans/guides/guide_fourteen.md:3599`) — is CONDITIONALLY MET on the disclosed environment and on this projection. That is finding 166 (`plans/phases/06-state-and-maturity.md:1032`, `:1060`), and with 164 and 165 it is one of the three grounds on which the fourth Wave-10 exit, "no creator-private state is used" (`plans/guides/guide_fourteen.md:3141`), is NOT MET for root history (`plans/phases/06-state-and-maturity.md:1066`).

The register holds a wave with an open owned row at its exit (`plans/guides/guide_fourteen_conceptual_register.md:7`), and three clauses Wave 10 owns stand open (`plans/phases/06-state-and-maturity.md:1068`). `G14C-11` asks for branch-relative competing realizations, typed realization-suffix invalidation, a branch-bound compare-and-swap token with submission-time fencing, and fork, rewind and byte-identical reproduction tests (`plans/guides/guide_fourteen_conceptual_register.md:119`). Wave 10's clause of `G14C-12` asks for typed class-2 responses and the competing-spend, same-key-instance and reorganization-surfaced-realization tests (`:120`). `G14C-13` asks for explicit non-claims beside the reconstruction (`:121`). The landed evidence for each is the modelled prefix, the stale refusals and the recovery report; none of the named artifacts exists.

## 2. Evidence tables

### 2.1 The three findings and their anchors at this base

| Finding | Anchor | What the tree holds there |
|---|---|---|
| 164 | `packages/vectors/src/maturity_history_report.rs:586–593` | the two premise lines written as constants |
| 164 | `packages/vectors/src/maturity_history_report.rs:116–136`, `ALL` at `:140–160` | nineteen items, none reading a premise |
| 164 | `packages/vectors/src/maturity_history.rs:596–637` | `bind`, storing either provenance; its `# Panics` section at `:593–595` and `unreachable!` at `:619–621` |
| 164 | `packages/vectors/src/maturity_corpus.rs:140–145`, `:149–172` | `MaturityPremiseProvenance`, two members and no `name()`; the generic carrier with `declared`, `value` and `provenance` |
| 165 | `packages/vectors/src/maturity_evidence.rs:1987–1995` | the plan's bundle under `retained_whole_metadata()` and its ABI over `validated_view` |
| 165 | `packages/vectors/src/maturity_evidence.rs:2014–2027` | both declared as `DeploymentDeclaration` premises of the checkpoint |
| 165 | `packages/vectors/src/maturity_evidence.rs:1885–1890`, `:1948` | the replayed planner under `Retained(StateWitnessSchedule::VariableMetadata)` and its bundle as the projection input |
| 165 | `packages/vectors/src/maturity_history.rs:644–677` | `binds`: identity, successor, predecessor, no premise |
| 165 | `packages/vectors/src/maturity_history.rs:1124–1128` | `assert_ne!` over the declared ABI's schedule and the continuity's |
| 166 | `packages/vectors/src/maturity_history.rs:114–122` | `StateRootEdge::from_continuity` |
| 166 | `packages/vectors/src/maturity_history.rs:43–59` | the certificate's sole constructor, reading the continuity; its `# Panics` section at `:39–41` |
| 166 | `packages/vectors/src/maturity_continuity.rs:205–218`, `:680–690` | the projection input's bundle and identity; `retained_context` |

### 2.2 The three clauses' criteria against the tree

| Clause | Criterion item | What the tree carries | What is absent |
|---|---|---|---|
| `G14C-11` | typed branch context, rewind and reproject | `BranchContext` (`packages/transaction/src/operator_right.rs:28–58`); `ModelledBranchPrefix` with `anchored`, `from_checkpoint`, `extend`, `rewind`, `reproject` (`packages/vectors/src/maturity_history.rs:537–541`, `:780–887`); the stale refusals (`:565–585`) | — |
| `G14C-11` | per-branch semantic-edge projection; immutable semantic history | the semantic transition is read by recovery (`RecoveredSuccessor`, `packages/vectors/src/maturity_recovery.rs:204–211`) and by the projector (`packages/vectors/src/maturity_continuity.rs:2127–2144`) | a semantic edge held apart from its realizations, and a projection of a thread onto one branch |
| `G14C-11` | typed realization-suffix invalidation | `rewind` returns a shorter prefix; the operator-right registry marks scopes invalidated in its record (`packages/transaction/src/operator_right.rs:795–809`) | an event naming the realizations a rewind removed |
| `G14C-11` | branch-relative competing realizations | `StateThreadAnchor::check_continuations` admits one accepted continuation per branch identifier and refuses a second on one branch as `Equivocation` (`packages/transaction/src/operator_right.rs:239–270`); the validator refuses a reused predecessor (`packages/vectors/src/maturity_history.rs:275–311`) | realizations placed on modelled branches; a second successor of the accepted predecessor |
| `G14C-11` | branch-bound compare-and-swap token with submission-time fencing | `RightScope` binds a branch and a predecessor, and the registry refuses a stale branch at issue and dispatch once a caller invokes `reorganize` (`packages/transaction/src/operator_right.rs:298–304`, `:433`, `:753–768`, `:817–823`) | a token that reads a branch view instead of a caller's reorganization call |
| `G14C-11` | reports call the evidence a checkpoint-bound edge certificate | the history report's checkpoint and its `no-archive-origin-beyond-admission` residual (`packages/vectors/src/maturity_history_report.rs:44–69`) | — |
| `G14C-12` | typed class-2 responses; three environment tests | `MaturityObservationClass::ModeledUnintendedEnvironmentTransition` (`packages/vectors/src/maturity_evidence.rs:227–243`), the quantifier (`:330–345`) and the poison marker (`:288–321`); no layer produces a class-2 observation, and every continuity is class 1 (`packages/vectors/src/maturity_continuity.rs:2170–2174`) | thread standings, environment-transition kinds and a classifier |
| `G14C-13` | explicit non-claims | four residuals: no chain observation, no root-cursor freshness, no archive origin beyond admission, no leastness of the witnessed nonce (`packages/vectors/src/maturity_recovery_report.rs:60–82`) | independent implementation, trust anchor, inclusion proof, canonical chain authentication, static-subtree data availability |

### 2.3 The landed types and functions the bites reuse, with visibility at this base

| Item | Where | Visibility and shape |
|---|---|---|
| `StateRootEdge`, `predecessor()`, `successor()`, `operation()`, `certificate()` | `packages/vectors/src/maturity_history.rs:104–143` | public type, private fields, four `const` accessors |
| `MaturityTransitionCertificate` | `:24–100` | public type, eight private fields, eight `const` accessors |
| `validate_state_root_history` | `:267–410` | `pub fn (&[StateRootEdge], Outpoint) -> Result<Outpoint, MaturityRootHistoryRefusal>` |
| `MaturityRootFact::kind`, `MaturityRootFactKind::DeclaredPremise` | `:453–467`, `:429–434` | public, `const` |
| `ModelledBranchBlock::new`, `height()`, `identity()`, `transactions()` | `:744–772` | public; `new` refuses an all-zero identity |
| `ModelledBranchPrefix::anchored`, `extend`, `rewind`, `identifier()`, `tip_height()`, `blocks()` | `:780–922` | public; `context()` at `:894` carries a `# Panics` section and no bite calls it |
| `MaturityModelledBranchRefusal` | `:555–561` | public, five variants |
| `ValidatedMaturityContinuity::bundle`, `schedule`, `branch`, `predecessor`, `requested_cycle`, `expected_successor` | `packages/vectors/src/maturity_continuity.rs:2102–2144` | public, `const` |
| `submitted_transaction_identities` | `packages/vectors/src/maturity_continuity.rs:1657–1665` | public |
| `PublicAnnouncementHandoff` and its eight accessors | `packages/vectors/src/maturity_recovery.rs:57–150` | public; `bytes()`, `state_output_index()`, `schedule()`, `bounds()`, `static_subtree()`, `internal_key()`, `contract()` |
| `RecoveredSuccessor` and its accessors | `packages/vectors/src/maturity_recovery.rs:204–261` | public, six private fields, constructed only at `:511–518` |
| `recover_public_successor` | `packages/vectors/src/maturity_recovery.rs:365–519` | `pub fn (PublicAnnouncementHandoff) -> Result<RecoveredSuccessor, MaturityRecoveryRefusal>` |
| `accepted_public_handoff` | `packages/vectors/src/maturity_recovery_report.rs:837–925` | `pub fn (&ValidatedMaturityCorpus) -> Result<PublicAnnouncementHandoff, MaturityPublicRecoveryReportRefusal>` |
| `validated_view` | `packages/vectors/src/maturity_evidence.rs:2206–2237` | module-private, `(&ReviewedElementsTapscriptDefinition, &CandidateLinkedMaturityBundle) -> Result<ValidatedMaturityStateView, MaturityEvidenceRefusal>` |
| `derive_maturity_announcement_abi` | `packages/transaction/src/state_abi.rs:756–792` | public; `CandidateMaturityAnnouncementAbi::schedule()` at `:654` and `contract()` at `:660` |
| `CandidateLinkedMaturityBundle::contract`, `deployment`, `policy`, `static_subtree`, `record` | `packages/linker/src/state_bundle.rs:529–562` | public, `const` |
| `StateLinkDeploymentParameters::lead_bounds`, `operator`; `StateLeadBounds::bounds` | `packages/linker/src/state_deployment.rs:244`, `:262`, `:113` | public, `const` |
| the policy's `internal_key` | `packages/linker/src/state_bundle.rs:321` | public, `const`, returning `StateInternalKeyPolicy` |
| `OperatorDeploymentBinding::key` | `packages/linker/src/operator_deployment.rs:156` | public, `const`, returning `&OperatorKey`; `OperatorKey` derives `Clone, Debug, PartialEq, Eq` (`packages/tapscript/src/operator_authorization.rs:193–194`) |
| `StateStaticSubtree::root`, `leaves` | `packages/tapscript/src/state_constructor.rs:418`, `:423` | public |
| `Outpoint::new`, `txid()`, `index()`; `TargetInput::outpoint`; `TargetTransaction::inputs`, `decode` | `packages/transaction/src/bytes.rs:556`, `:565`, `:571`, `:685`, `:1205`, `:1388` | public; `new` is fallible, `decode` returns `Result<Self, TransactionRefusal>` |
| `AnnouncementLeadBounds::window` | `packages/realization/src/state.rs:161` | `pub fn window(self, current: Cycle) -> Result<(Cycle, Cycle), MaturityTransitionRefusal>`, the pair being the earliest and latest announcement cycles |
| `MaturityAnnouncementPlanner::new`, `next_step`, `bundle`, `submission_bytes` | `packages/vectors/src/maturity_native.rs:435`, `:475`, `:493` | public |
| `OPERATOR_HANDLE`, `OperatorVerifier` | `packages/vectors/src/maturity_operator.rs:679`, `:688` | public |

### 2.4 Every call site of the binder, the report validator and the premise declarations

The rewrite in bite 1 changes the operands of `MaturityRootCheckpoint::bind`, `MaturityRootCheckpoint::binds` and `validate_maturity_root_history_report` from a continuity to a handoff. The inventory below is every call site in `packages/`, recounted by searching for each name; no other site exists.

| Site | Now | After bite 1 |
|---|---|---|
| `packages/vectors/src/maturity_evidence.rs:2014–2027` (production) | `bind(&edge, &continuity, accepted_corpus, <plan bundle>, <plan ABI>)` | `bind(&edge, &handoff, accepted_corpus, linked, abi)` with `(linked, abi)` from `checkpoint_premises_of(&continuity)?`, the handoff built first |
| `packages/vectors/src/maturity_evidence.rs:2037–2046` (production) | `validate_maturity_root_history_report(.., &continuity, ..)` | `.., &handoff, ..` |
| `packages/vectors/src/maturity_evidence.rs:2047–2049` (production) | `accepted_public_handoff` after both history calls | moved above the checkpoint, the one handoff serving history and recovery |
| `packages/vectors/src/maturity_history.rs:1010–1020` (test) | `static ABI` over the plan's whole-metadata ABI | replaced by `static WHOLE_METADATA_PREMISES`, the plan's bundle and ABI, read only by the new premise test |
| `packages/vectors/src/maturity_history.rs:1022–1038` (test `declarations`) | the continuity's bundle beside the plan's ABI | `checkpoint_premises_of(continuity).expect("accepted premises")` |
| `packages/vectors/src/maturity_history.rs:1040–1048` (test `checkpoint`) | `bind(edge, continuity, corpus, ..)` | gains `handoff: &PublicAnnouncementHandoff`; `bind(edge, handoff, corpus, ..)` |
| `packages/vectors/src/maturity_history.rs:1128` | `binds(&edge, &continuity)` | `binds(&edge, &handoff)` |
| `packages/vectors/src/maturity_history.rs:1178–1185` | historical `bind` refusing the outstanding acceptance | `bind(&historical_edge, &historical_handoff, historical_corpus, ..)`, the same refusal |
| `packages/vectors/src/maturity_history.rs:1200–1202` | `binds(&historical_edge, &historical)` | `binds(&historical_edge, &historical_handoff)`; `TransactionIdentityDisagrees { checkpoint, handoff }` |
| `packages/vectors/src/maturity_history.rs:1215–1222` | accepted corpus over the historical continuity | `bind(&historical_edge, &historical_handoff, accepted_corpus, ..)`; `SubmittedBytesDisagree { checkpoint, handoff }`, the second operand the digest of the historical bytes |
| `packages/vectors/src/maturity_history.rs:1246–1248` | changed successor refuses `SuccessorOutpointDisagrees` | refuses `SuccessorIsNotTheHandoffsStateOutput`, because the checkpoint's successor is now the edge's and the handoff decides it; `SuccessorOutpointDisagrees` is asserted by `checkpoint.binds(&wrong_successor, &handoff)` on the accepted checkpoint |
| `packages/vectors/src/maturity_history.rs:1266–1273` | changed predecessor | `PredecessorOutpointDisagrees { checkpoint, handoff }`, the second operand input 0 of the handoff's bytes |
| `packages/vectors/src/maturity_history.rs:1297` | `checkpoint(&edge, &continuity, corpus)` | `checkpoint(&edge, &continuity, &handoff, corpus)` |
| `packages/vectors/src/maturity_history_report.rs:657–679` (test `checkpoint`) | the continuity's bundle beside `PLAN.abi()` | `checkpoint_premises_of(continuity)`; `bind(edge, handoff, ..)` |
| `packages/vectors/src/maturity_history_report.rs:681–698` (test `assembled`) | returns the continuity | returns the accepted handoff in its place |
| `packages/vectors/src/maturity_history_report.rs:700–714` (test `validate`) | `continuity` operand | `handoff` operand; every caller at `:719`, `:751`, `:753`, `:778`, `:795–803`, `:811`, `:819`, `:827`, `:837`, `:845`, `:857`, `:870`, `:882`, `:890`, `:923` passes the handoff |
| `packages/vectors/src/maturity_history_report.rs:866–874` | the historical continuity refuses at the checkpoint item | the historical handoff refuses there, `TransactionIdentityDisagrees { .. }` |
| `packages/vectors/src/maturity_recovery_report.rs:1258–1297` (cross-role test) | the continuity's bundle beside `plan.abi()`; `bind` and `validate` over the continuity | `checkpoint_premises_of(&continuity)`; `bind` and `validate` over the test's own handoff from `accepted()` |
| `packages/vectors/src/maturity_evidence.rs:3150–3185` (test) | `PLAN.bundle()` and `PLAN.abi()`; `bind` and `validate` over the continuity | `checkpoint_premises_of(&continuity)`; the handoff built at `:3186` moves above `bind`, and both calls take it |

The historical handoff the history tests need is restated, not built by `accepted_public_handoff`, which refuses the historical corpus for its outstanding acceptance (`packages/vectors/src/maturity_recovery_report.rs:841–850`): `PublicAnnouncementHandoff::new` over `archived().input().submitted_bytes`, a locator naming the historical identity recomputed by `submitted_transaction_identities`, output index 0 and the accepted handoff's five published parameters. `archived()` is crate-visible at `packages/vectors/src/maturity_continuity.rs:2290`, and `input()` exposes the bytes at `:2248–2257`.

### 2.5 Test counts, before

| File | Tests |
|---|---|
| `packages/vectors/src/maturity_history.rs` | 21 |
| `packages/vectors/src/maturity_history_report.rs` | 9 |
| `packages/vectors/src/maturity_recovery.rs` | 13 |
| `packages/vectors/src/maturity_recovery_report.rs` | 11 |
| `packages/vectors/tests/maturity_recovery.rs` | 3 |
| `packages/vectors/src/maturity_evidence.rs` | 22 |
| `packages/vectors/src/maturity_corpus.rs` | 42 |
| `packages/vectors/src/maturity_safety.rs` | 20 |
| `packages/vectors/src/matrix.rs` | 11 |
| `packages/vectors/src/observed_boundary.rs` | 3 |
| `packages/vectors/src/maturity_negative_half.rs` | 7 |
| `packages/vectors/src/maturity_continuity.rs` | 58 |
| `packages/vectors/src/maturity_continuity_report.rs` | 31 |
| `packages/vectors/src/maturity_report.rs` | 20 |
| `packages/vectors/tests/maturity_continuity.rs` | 16 |
| `packages/vectors/src/maturity_native.rs` | 32 |

The census figures stand at 206 rows, 71 answered, 35 typed non-answers, 16 root-history observed and 14 public-recovery observed (`packages/vectors/src/maturity_evidence.rs:3770–3773`), `EvidenceBoundary::ALL.len()` at 15 (`packages/vectors/src/matrix.rs:1495`) and `STILL_REQUIRED.len()` at 42 (`packages/vectors/src/maturity_negative_half.rs:775`). No bite below moves any of them.

### 2.6 Contradictions this base holds

Six statements about the tree read differently from the tree, and each changes a shape below; a seventh reading closes the section. `G14C-13`'s criterion names five non-claims — independent implementation, trust anchor, inclusion proof, canonical chain authentication and static-subtree data availability (`plans/guides/guide_fourteen_conceptual_register.md:121`) — while its Status cell, the register exit paragraph of the closure record and finding 169 name four, omitting canonical chain authentication (`plans/phases/06-state-and-maturity.md:1038`, `:1068`); a row closes on its criterion (`plans/guides/guide_fourteen_conceptual_register.md:7`), so D4 types five and the closure row retypes the count. The history report carries no golden canonical-bytes test: its byte tests are `independent_assemblies_render_identical_bytes` and `the_canonical_bytes_carry_no_volatile_key_and_no_raw_text` (`packages/vectors/src/maturity_history_report.rs:748`, `:888`), so D2 states the rendered lines rather than a digest. `MaturityPremiseProvenance` has no `name()` (`packages/vectors/src/maturity_corpus.rs:140–145`). The history tests' `declarations` already declare the continuity's bundle and only the ABI from the plan (`packages/vectors/src/maturity_history.rs:1022–1038`), while production and the evidence test declare both from the plan (`packages/vectors/src/maturity_evidence.rs:2018–2025`, `:3157–3164`). `StateStaticSubtree` equality is structural over tree, root and leaves (`packages/tapscript/src/state_constructor.rs:383–388`), and no assertion at this base shows the linked subtree (`packages/linker/src/state_bundle.rs:975–991`) whole-value equal to the handoff's subtree rebuilt from witness item seven (`packages/vectors/src/maturity_recovery_report.rs:875–890`) — the bite-3 landing compared roots only — so every comparison below compares roots. And three landed test names become false under the bites: `the_other_archives_facts_do_not_bind_the_accepted_continuity` and `a_checkpoint_the_continuity_does_not_bind_refuses_at_the_checkpoint_item` once the binder takes a handoff, and `the_accepted_handoffs_recovery_report_validates_and_renders_four_residuals_in_order` once the report carries nine; the bites rename them and the closure row retypes the phase record's citations of them (`plans/phases/06-state-and-maturity.md:1050`, `:1054`). One more reading qualifies the premise that the evidence path carries no panic: `packages/vectors/src/maturity_evidence.rs` carries no `# Panics` section, `.expect(` or `.unwrap()` above its test module at `:2456`, but `packages/vectors/src/maturity_corpus.rs` documents five `# Panics` sections in production text (`:1567`, `:1584`, `:1604`, `:1628`, `:1645`), each propagating the planner's documented panic (`packages/vectors/src/maturity_native.rs:432–433`, `:787`), and `packages/vectors/src/maturity_history.rs` carries three (`:39–41`, `:593–595`, `:891–892`); no design below adds one, and bite 1 removes the one on `bind`.

## 3. Decisions

**D1. The evidence derivation declares the bundle the accepted continuity was projected under and the ABI derived over that bundle's view, and the checkpoint binds against the public handoff: its three existing questions are answered from the handoff's bytes, and two new comparisons check each premise's published parameters against the handoff's.** The declaration needs no new derivation: `validated_view` builds a view from any bundle's first retained instance and the deployment's stated positions (`packages/vectors/src/maturity_evidence.rs:2206–2237`), `PublicMaturityStateView::validate` checks only the commitment at the stated nonce and reads no schedule (`packages/transaction/src/state_view.rs:430–463`), and `derive_maturity_announcement_abi` reads the schedule from the bundle's record (`packages/transaction/src/state_abi.rs:756–792`). A crate-private `checkpoint_premises_of(continuity)` composes them over `continuity.bundle()`, so the derivation and the four test sites of section 2.4 declare one value from one definition. The comparison is where this decision departs from the finding's second wording, which compared the declared bundle with the continuity's: in the derivation that comparison would be circular, since the declared bundle is the continuity's, and it would make the binder read the creator's retained instance, which is the ground on which the fourth exit fails for root history. The public handoff carries the five parameters a published bundle fixes — the witness schedule, the lead bounds, the static subtree, the internal key and the contract revision (`packages/vectors/src/maturity_recovery.rs:57–68`) — and recovery reconstructs output 0 from exactly those (`:492–510`), so a declared bundle whose published parameters equal the handoff's is a bundle under which the accepted bytes reproduce, which is what "the premises are the ones that produced the accepted bytes" can mean for values archive admission never pins (`packages/vectors/src/maturity_corpus.rs:1731–1736`). The ABI carries two of them, schedule and contract (`packages/transaction/src/state_abi.rs:654–662`). Static subtrees compare by root, not by value (section 2.6): the root is what witness item three and the output's tweak commit to, and a whole-value comparison would compare the in-memory parse of a leaf program. The three existing questions stay, each now answered from the handoff: the identity recomputed from the handoff's bytes, the successor as that identity at the handoff's STATE output index, the predecessor as input 0; the readback digest is compared with the handoff's bytes at construction. Taking the successor from the edge and checking it against the handoff removes the `Outpoint::new` call whose failure `bind` could only declare unreachable, so the landed `unreachable!` and its `# Panics` section (`packages/vectors/src/maturity_history.rs:593–595`, `:619–621`) leave with the rewrite, a hygiene gain for a function the evidence derivation calls; every new reading that can fail — decoding the handoff's bytes, reading input 0 — returns a typed refusal. The test that pinned the defect is retyped to pin its repair: the declared ABI's schedule and the declared bundle's record schedule both equal the handoff's `VariableMetadata`, and a new test declares the plan's whole-metadata bundle and ABI and refuses at the schedule. After this decision the root-history report validates against the handoff, the admitted archive and two declarations and names no continuity.

**D2. The root-history report gains one recomputed item comparing both carried provenances with the kind §17.2's roster declares, and the renderer writes the carried provenance by a new total `name()`.** The item `CheckpointPremiseProvenance` follows `CheckpointSuccessor`, moving the roster from 19 to 20; it compares `MaturityRootFactKind::DeclaredPremise` over each carried provenance with `MaturityRootFact::LinkedCandidate.kind()` and `MaturityRootFact::CandidateAbi.kind()` (`packages/vectors/src/maturity_history.rs:453–467`), so the expected kind is read from the one roster that states it rather than restated as a constant, and it refuses through the existing `ItemDiffers`, which keeps `failed_item` total with no new arm (`packages/vectors/src/maturity_history_report.rs:216–225`). Placing the check in the report rather than the binder follows what each answers: the binder asks whether facts agree, and the provenance is a claim the report publishes about how two facts were obtained. `MaturityPremiseProvenance::name` maps `ExecutorArguments` to `executor-arguments` and `DeploymentDeclaration` to `deployment-declaration`, and the renderer writes `checkpoint_linked_candidate declared {}` and `checkpoint_candidate_abi declared {}` from the carried values. For the accepted archive the two lines render as exactly `checkpoint_linked_candidate declared deployment-declaration` and `checkpoint_candidate_abi declared deployment-declaration`, the landed text, and the canonical bytes gain one line, `recomputed checkpoint-premise-provenance`, immediately after `recomputed checkpoint-successor`, because the renderer walks the validated set in the enum's derived order (`packages/vectors/src/maturity_history_report.rs:613–615`). A report whose premise carries `ExecutorArguments` no longer validates, so the rendered name can only read `deployment-declaration` from a validated report; rendering the carried value rather than the constant keeps the bytes a function of the report the validation passed.

**D3. A root edge can be built from the public handoff alone, and the evidence derivation validates and reports that edge.** Nothing `StateRootEdge` or its certificate carries needs the retained context (`packages/vectors/src/maturity_history.rs:24–33`, `:104–109`). The predecessor is input 0 of the published bytes, which recovery decodes (`packages/vectors/src/maturity_recovery.rs:370–374`); the successor is the identity recovery recomputes and checks against the locator (`:375–381`) at the STATE output index it checks against the architecture (`:383–395`); the projection, operation, root and use are the constants the landed constructor writes (`packages/vectors/src/maturity_history.rs:49–53`); and both static subtrees are the handoff's published subtree, authenticated on the predecessor side as far as witness item three equals its root (`packages/vectors/src/maturity_recovery.rs:426–433`) and on the successor side by output 0 equalling the program reconstructed over it (`:492–510`). `RecoveredSuccessor` therefore gains the two outpoints recovery already reads, and `StateRootEdge::from_public_handoff` takes the handoff by value, runs `recover_public_successor` and refuses through `MaturityRecoveryRefusal`, so an edge exists exactly when public recovery succeeds and no second refusal vocabulary is minted. Under `validate_state_root_history` the handoff-built edge meets §17.1's six clauses as follows (`plans/guides/guide_fourteen.md:2716–2729`): clause 1 against the starting cursor the derivation states as the edge's predecessor, which the checkpoint then checks against input 0 (D1); clause 2 because the successor's transaction identity is the announcement's own and differs from the predecessor's; clause 3 because the certificate's operation and projection are the constants; clause 4 because the certificate's endpoints are the edge's by construction and both are read from the transaction's bytes; clause 5 because `RootId::State` with `RootUse::Succession` is the operation's declared root and output 0 is its declared STATE position, which recovery has already required; clause 6 because the cursor advances only on return. §17.3's `state_bundle_continuity` receives two copies of one published subtree and so cannot refuse; for this edge the no-migration rule is decided by recovery — item three against the published root, output 0 against the program over it — and the study states that rather than counting the linker's call as the check. Equality with the projected edge is asserted field by field with static roots compared (section 2.6). The edge discharges the eleventh conjunct's second condition; what stays unpublished for a subtree carrying support leaves is D4's data-availability non-claim.

**D4. The public-recovery report carries five new typed non-claims as members of `MaturityPublicRecoveryResidual`, the criterion's five, and a census fact reading the handoff's subtree shape beside the data-availability non-claim.** A residual in this report already is a typed non-claim with a canonical position, a recomputed item per position and a render line (`packages/vectors/src/maturity_recovery_report.rs:60–82`, `:678–698`, `:813–815`); a second vocabulary would need a second vector, a second item family and a second render prefix to say the same kind of thing, and `no_role_spelling_crosses_between_the_reports` would have to learn a second name set (`:1246–1322`). The members, in the criterion's order: `NoIndependentImplementation`, `NoTrustAnchor`, `NoInclusionProof`, `NoCanonicalChainAuthentication`, `NoStaticSubtreeDataAvailability`, named `no-independent-implementation`, `no-trust-anchor`, `no-inclusion-proof`, `no-canonical-chain-authentication` and `no-static-subtree-data-availability`, taking `ALL` from 4 to 9. Canonical chain authentication is typed separately from the existing `no-chain-observation` because the two retire independently: a process that queried a node would observe a chain without authenticating it as canonical, and C-5's remedy lists the trust anchor, inclusion evidence and active-chain rules as distinct obligations (`plans/reviews/guide_fourteen_conceptual_review_c.md:225–233`). Data availability is a residual because it is a property of who publishes a subtree's leaves, not of the bytes; the census gains `static_leaves`, recomputed from `handoff.static_subtree().leaves().len()`, which reads 1 for the one-leaf singleton finding 148 records, so a reader sees that this handoff's whole subtree is witness item seven and that the non-claim stands for a subtree the report does not have. Items move from 26 to 32: five residual items and `CensusStaticLeaves`.

**D5. Class-2 responses are a thread vocabulary of their own, not members of §14.2's row standings: a thread standing of three members, three environment-transition kinds fixed to class 2, and a classifier over a thread's realizations on one modelled branch.** §14.2's standing vocabulary answers whether a row's mutation failed at its declared boundary (`plans/guides/guide_fourteen.md:2157–2188`) and has no thread member; the register's R-TAXONOMY names the thread standings — "typed fencing, view invalidation, and thread-contested or thread-lost standings" (`plans/guides/guide_fourteen_conceptual_register.md:37`) — so they belong to a thread, and a row standing that could read "lost" would let an environment event stand as an answer. `MaturityThreadStanding` has `Intended`, `Contested` and `Lost`; `MaturityEnvironmentTransition` has `CompetingSpend`, `SameKeyInstance` and `ReorganizationSurfacedRealization`, each with `class()` returning `MaturityObservationClass::ModeledUnintendedEnvironmentTransition` (`packages/vectors/src/maturity_evidence.rs:227–243`) and a total `standing()`; no new type has a field of `MaturityBranchPoisonMarker`'s type, so a class-3 observation stays representable only as the marker (`:288–321`). A competing spend is a realization spending the thread's cursor that is not one of the thread's intended edges; a reorganization-surfaced realization is the same after an invalidation that removed an intended realization at that cursor; a same-key instance is a realization under the thread's operator key that does not continue the thread. The key is read from each realization's projected continuity — `continuity.bundle().deployment().operator().key()` (`packages/linker/src/state_deployment.rs:262`, `packages/linker/src/operator_deployment.rs:156`) — because the handoff carries no operator key and no reader in the tree extracts one from a published leaf; the evidence pointer says so. The three tests draw on bytes the tree can supply without a chain: a competing successor built and signed in process from the accepted predecessor at the window's earliest cycle, on the pattern of the native module's independent builders (`packages/vectors/src/maturity_native.rs:1196–1288`), which differs from the planner's earliest-plus-one (`:813`); the node-free candidate as the same-key instance, built by the planner under the same published signer handle (`packages/vectors/src/maturity_continuity.rs:2437–2494`, `packages/vectors/src/maturity_native.rs:831`); and a rewind followed by re-extension along the competing branch.

**D6. `G14C-11`'s artifacts live in a new module `packages/vectors/src/maturity_branch.rs` over the landed modelled prefix: a branch view carrying realizations, a thread carrying semantic history and a starting cursor, a rewind returning a typed invalidation event, a per-branch projection of the thread, and a compare-and-swap token the thread mints from a view and fences against the view current at submission.** R-2 states the model: "the semantic-edge-to-transaction projection is per branch; rewind and reproject are typed total operations; semantic history is immutable under rewind; realization-suffix invalidation is a typed event; deterministic reprojection is stable where the predecessor realization is unchanged; competing-branch realizations are representable; and the current-view compare-and-swap token is branch-bound with submission-time fencing" (`plans/guides/guide_fourteen_conceptual_register.md:29`). A realization pairs a block height with an edge, its semantic edge and its operator key; a view is a prefix and its realizations; the thread's semantic history is a field no view operation takes, so a rewind cannot reach it — immutability is a property of the signatures rather than of a test that happens to pass. Competing realizations are two views sharing an anchor block under different identifiers, each projecting the thread to its own successor, while the union refuses as a reused predecessor under the landed validator (`packages/vectors/src/maturity_history.rs:275–311`) — the same one-per-branch shape R-3's `check_continuations` states over accepted continuations (`packages/transaction/src/operator_right.rs:239–270`). The token binds the branch identifier, a checkpoint block and the thread's cursor; the fence refuses another branch, a checkpoint block no longer in the view, and a moved cursor, and passes a view extended by blocks that realize nothing of the thread, which is what makes it a compare-and-swap on the cursor rather than an equality on the tip — the predecessor outpoint as the version token, in C's reading (`plans/reviews/guide_fourteen_conceptual_review_c.md:144`). The operator-right registry's stale-branch refusal is not reused because it fences only after a caller invokes `reorganize` with a whole branch context (`packages/transaction/src/operator_right.rs:795–823`); the token reads the view. Byte-identical reproduction replays the accepted archive through a fresh planner and projects the replayed bytes, which is the reprojection R-2 calls stable where the predecessor realization and the complete environment are unchanged.

**D7. Seven code bites and one closure row, the two premise defects first as one bite.** The order is bite 1 (D1, D2), bite 2 (D3), bite 3 (D4), bites 4a and 4b (D6), bite 5 (D5) and bite 6 (D9); D6's layer is two bites because its realizations, views and thread measure above 600 production lines at landed scale together with its token and fence, and the token is a separate gate. The cut and its estimates are section 5's; every estimate is stated at landed scale against the Wave-10 bites' measured undercounts — documented accessors, total name mappings and staged builders took bite 2 to 1.16 times the top of its range, bite 3 to 1.37, 4a to 618 against 420–600, 4b to 925 against 480–680 and 4c to 572 inserted against 220–340 (`plans/drafts/wave10-root-history-and-recovery-study.md`, sections 2.10 to 2.12) — and no bite's top exceeds 600 production lines.

**D8. All three clauses can close on modelled branches, pinned admitted bytes and host-built candidates, and each closing pointer states what the modelled evidence is.** `G14C-13`'s criterion is a report property — "records explicit non-claims" (`plans/guides/guide_fourteen_conceptual_register.md:121`) — and its non-claims are exactly the statements that no chain, anchor or diversity stands behind the bytes, so it needs no environment to close. `G14C-11` and `G14C-12` name model operations and tests of the model's responses: R-2 makes rewind and reprojection "an ordinary operation of the model" (`:27`), and R-TAXONOMY places class 2 where "the target moved to a state our semantics did not intend — a double spend, or another instance under the same keys, not initiated by us — while still moving within our model of how the target can move" (`:35`), with the response vocabulary on the class-2 list at `:37`. The chain-selection and finality remedies of B-6, C-2 and C-7 are superseded by R-2's branch-indexed model rather than owed beside it (`:133`). Each pointer states: the branches are modelled; the competing successor is a host-built, signed candidate never submitted to a target; the same-key comparison reads keys from projected continuities; no chain was observed; and the fence is a typed operation no production submitter calls, because no layer here holds a branch view of a real target to fence against. What a later environment would add — an observed fork, a competing spend on a real branch — is not a register requirement and stays under the `no-chain-observation` residual both reports already carry.

**D9. The two stale comments of findings 171 and 172 are retyped in their own last bite, comments only.** The standings test's comment speaks of a twelfth acceptance-awaiting row while asserting eleven that include §16.1's last (`packages/vectors/src/maturity_safety.rs:3789–3792`), the sponsorless row's comment describes a relay refusal its standing no longer stands on (`:1430–1433`), and the reason's documentation says neither route to an acceptance has landed although the constant-elision route closed on `0.6.245-dev` (`:223–237`). A comment that contradicts its assertion is read as the assertion's intent by the next editor; retyping three comments changes no line a gate reads. Finding 167's header spellings are not cut: aligning them changes the older reports' canonical bytes and first needs a `name()` accessor neither `TargetContractVersion` nor `ExecutionDomain` carries (`plans/phases/06-state-and-maturity.md:1034`).

**D10. Each bite lands under its own backlog row, `T11-119` to `T11-125`, and `T11-126` is the closure row that alone edits the phase record and the register.** The rows follow the `T11-113` to `T11-118` pattern (`plans/backlog.md:1306–1311`): status, the work in its own terms, a DONE-when criterion, the gate and the dependency. Keeping the record and the register out of the code bites keeps each bite's file list exact and puts every evidence pointer in one record written over landed tags.

## 4. Open questions

**OQ1. Whether §14.2's standing vocabulary is meant to be the only standing vocabulary.** §14.2 lists eleven evidence standings and no thread standing (`plans/guides/guide_fourteen.md:2157–2173`), while R-TAXONOMY names "thread-contested or thread-lost standings" (`plans/guides/guide_fourteen_conceptual_register.md:37`). For one vocabulary: a reader of §14.2 might take its list as the closed set of every standing the phase publishes, and a second type named for standings could read as an addition to it. For two: §14.2's members answer a matrix row's question at a declared boundary, a thread standing answers what the environment did to one thread on one branch, and a row whose standing could read "lost" would let an environment event stand as an answer. D5 proceeds on two, naming the new type for threads so it cannot be mistaken for a row standing; whether §14.2 intends its list as exhaustive over every standing is a question about the guide's text.

**OQ2. Whether Wave 10's fourth exit governs the root-history path.** The exit list reads "no creator-private state is used" directly after "an unrelated process reconstructs output 0" (`plans/guides/guide_fourteen.md:3138–3141`), and the closure record applied it to both paths (`plans/phases/06-state-and-maturity.md:1066`). After bites 1 and 2 the root-history path builds its edge from the handoff and checks its declared premises against published parameters, but the checkpoint still carries the declared linked candidate whole, retained instance included, because §17.2 binds the "linked candidate" (`plans/guides/guide_fourteen.md:2743`); section 5 states the root-history half as CONDITIONALLY MET on that ground. Whether §17.2's linked candidate is the whole bundle value or the parameters a deployment publishes, and so whether the carried instance is a use of creator-private state, is the guide's question.

## 5. The bites

Each bite is one named commit under its own backlog row, gated. Line estimates are changed or added lines at landed scale — the Wave-10 bites undercounted documented accessors, total name mappings and staged builders, and each estimate below names which of the three it carries — and no bite's top exceeds 600 production lines. The counts before are section 2.5's; the counts after are each gate's obligations, recounted at the base the bite lands on. Every new tracked file joins its directory's `meson.build` in the same commit. Every gate row is a reading the bite's gate prints, with the figure it must be; a reading "above the test module" means every matching line number is below the line of the module's `mod tests`.

### Bite 1 — the checkpoint's premises declared and bound against the public handoff (`T11-119`)

Files: `packages/vectors/src/maturity_history.rs`, `packages/vectors/src/maturity_history_report.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/src/maturity_corpus.rs`, and `packages/vectors/src/maturity_recovery_report.rs` for its cross-role test only. Five files, none new.

Shape: the derivation declares the continuity's bundle and the ABI over its view through `checkpoint_premises_of`; `MaturityRootCheckpoint::bind` and `binds` take the handoff in place of the continuity, answer identity, successor and predecessor from the handoff's bytes and then compare each premise's published parameters with the handoff's; the history report validates against the handoff, gains `CheckpointPremiseProvenance`, and renders the carried provenance by `MaturityPremiseProvenance::name` (D1, D2).

Reused, with visibility at `0.6.266-dev`: section 2.3's rows for the handoff, the bundle's accessors, the ABI's `schedule()` and `contract()`, `submitted_transaction_identities`, `TargetTransaction::decode` and `inputs`, `StateStaticSubtree::root`, `MaturityRootFact::kind`, `validated_view` and `derive_maturity_announcement_abi`; `accepted_public_handoff` (public) in the tests; `sha256` as the history module already imports it (`packages/vectors/src/maturity_history.rs:10`); `ValidatedMaturityCorpus::evidence()` with `branch()` as the derivation reads it (`packages/vectors/src/maturity_evidence.rs:1884`).

New type:

| Type | Members and derives |
|---|---|
| `MaturityPremiseDisagreement` | five variants, each `{ declared: T, published: T }`, in comparison order: `WitnessSchedule` over `StateWitnessSchedule`, `LeadBounds` over `AnnouncementLeadBounds`, `StaticRoot` over `Digest32`, `InternalKey` over `StateInternalKeyPolicy`, `ContractRevision` over `TargetContractVersion`; derives `Clone, Copy, Debug, PartialEq, Eq`, which every operand type already derives |

`MaturityRootCheckpointRefusal` moves from five variants to ten; its derives stay `Clone, Debug, PartialEq, Eq`:

| Variant | Operands | `fact()` |
|---|---|---|
| `AcceptanceIsOutstanding` | `routes: [MaturityAcceptanceRoute; 2]`, unchanged | `None` |
| `TransactionIdentityDisagrees` | `checkpoint: Txid`, `handoff: Txid` — the second operand renamed from `continuity`, because it is now the identity recomputed from the handoff's bytes | `TransactionIdentity` |
| `SubmittedBytesDisagree` | `checkpoint: Digest32`, `handoff: Digest32`, renamed likewise | `TransactionIdentity` |
| `HandoffBytesUndecodable` (new) | `offered: usize` | `TransactionIdentity` |
| `SuccessorOutpointDisagrees` | `checkpoint: Outpoint`, `edge: Outpoint`, unchanged | `SuccessorOutpoint` |
| `SuccessorIsNotTheHandoffsStateOutput` (new) | `checkpoint: Outpoint`, `identity: Txid`, `index: u32` | `SuccessorOutpoint` |
| `HandoffCarriesNoInput` (new) | none | `PredecessorOutpoint` |
| `PredecessorOutpointDisagrees` | `checkpoint: Outpoint`, `handoff: Outpoint`, renamed likewise | `PredecessorOutpoint` |
| `LinkedCandidateDisagrees` (new) | `MaturityPremiseDisagreement` | `LinkedCandidate` |
| `CandidateAbiDisagrees` (new) | `MaturityPremiseDisagreement` | `CandidateAbi` |

`fact()` joins arms sharing a result with `|`, as it does now (`packages/vectors/src/maturity_history.rs:513–524`), because clippy refuses two arms with identical bodies.

Every function the bite adds or rewrites, with its exact signature:

| Function | Signature | What it does |
|---|---|---|
| `MaturityPremiseProvenance::name` | `pub const fn name(self) -> &'static str`, `#[must_use]` | total over both members: `executor-arguments`, `deployment-declaration` |
| `MaturityRootCheckpoint::bind` | `pub fn bind(edge: &StateRootEdge, handoff: &PublicAnnouncementHandoff, corpus: &ValidatedMaturityCorpus, linked_candidate: MaturityDeclaredPremise<CandidateLinkedMaturityBundle>, candidate_abi: MaturityDeclaredPremise<CandidateMaturityAnnouncementAbi>) -> Result<Self, MaturityRootCheckpointRefusal>` | refuses an outstanding acceptance, then `sha256(readback.bytes())` against `sha256(handoff.bytes())`; assembles with `transaction_identity: readback.identity()`, `predecessor: edge.predecessor()`, `successor: edge.successor()` and `branch: corpus.evidence().branch()`; then asks `binds`; an `# Errors` section and no `# Panics` section |
| `MaturityRootCheckpoint::binds` | `pub fn binds(&self, edge: &StateRootEdge, handoff: &PublicAnnouncementHandoff) -> Result<(), MaturityRootCheckpointRefusal>` | in order: `TargetTransaction::decode(handoff.bytes())`, refusing `HandoffBytesUndecodable { offered: handoff.bytes().len() }`; the identity from `submitted_transaction_identities(&transaction, handoff.bytes()).identity()` against `self.transaction_identity`; `self.successor` against `edge.successor()`; `self.successor.txid()` and `.index()` against that identity and `handoff.state_output_index()`; `transaction.inputs().first()`, refusing `HandoffCarriesNoInput`, and its `outpoint()` against `self.predecessor`; then the two premise comparisons; an `# Errors` section |
| `linked_candidate_disagreement` | `fn linked_candidate_disagreement(bundle: &CandidateLinkedMaturityBundle, handoff: &PublicAnnouncementHandoff) -> Option<MaturityPremiseDisagreement>` | module-private; the first of `bundle.record().schedule()`, `bundle.deployment().lead_bounds().bounds()`, `*bundle.static_subtree().root()`, `bundle.policy().internal_key()` and `bundle.contract()` that differs from the handoff's `schedule()`, `bounds()`, `*static_subtree().root()`, `internal_key()` and `contract()` |
| `candidate_abi_disagreement` | `fn candidate_abi_disagreement(abi: &CandidateMaturityAnnouncementAbi, handoff: &PublicAnnouncementHandoff) -> Option<MaturityPremiseDisagreement>` | module-private; the first of `abi.schedule()` and `abi.contract()` that differs from the handoff's |
| `checkpoint_premises_of` | `pub(crate) fn checkpoint_premises_of(continuity: &ValidatedMaturityContinuity) -> Result<(MaturityDeclaredPremise<CandidateLinkedMaturityBundle>, MaturityDeclaredPremise<CandidateMaturityAnnouncementAbi>), MaturityEvidenceRefusal>` | `closure_target()` mapped to `TargetBindingUnavailable`, `validated_view(&target, continuity.bundle())?`, `derive_maturity_announcement_abi(&target, &view)` mapped to `CandidateAbiRefused`, both values declared `DeploymentDeclaration`; placed after `validated_view` with an `# Errors` section naming the three refusals |
| `validate_maturity_root_history_report` | its fourth parameter becomes `handoff: &PublicAnnouncementHandoff`; seven parameters, as now | `CheckpointBinding` asks `report.checkpoint.binds(last_edge, handoff)`; after `CheckpointSuccessor`, `CheckpointPremiseProvenance` compares `MaturityRootFactKind::DeclaredPremise(report.checkpoint.linked_candidate().provenance())` with `MaturityRootFact::LinkedCandidate.kind()` and the ABI's with `MaturityRootFact::CandidateAbi.kind()`, refusing `ItemDiffers` |
| `render_maturity_root_history_report` | unchanged signature | the two premise lines write `checkpoint_linked_candidate declared {}` and `checkpoint_candidate_abi declared {}` with the carried provenance's `name()` |

Existing lines that change, by file and line at `0.6.266-dev`:

| Where | Now | After |
|---|---|---|
| `packages/vectors/src/maturity_corpus.rs:139–145` | the enum alone | an `impl` with `name()` follows it |
| `packages/vectors/src/maturity_history.rs:5–20` | imports | gain `AnnouncementLeadBounds`, `StateInternalKeyPolicy`, `StateWitnessSchedule`, `TargetContractVersion`, `TargetTransaction` and `PublicAnnouncementHandoff` |
| `packages/vectors/src/maturity_history.rs:486–525` | five refusal variants and `fact()` | the table above, with `MaturityPremiseDisagreement` declared after the refusal |
| `packages/vectors/src/maturity_history.rs:587–637` | `bind` over the continuity, with `# Panics` at `:593–595` and `unreachable!` at `:619–621` | the row above; both removed |
| `packages/vectors/src/maturity_history.rs:639–677` | `binds`, three comparisons over the continuity | the row above, then the two private helpers after the `impl` block |
| `packages/vectors/src/maturity_history_report.rs:13`, `:18–21` | imports `ValidatedMaturityContinuity` | imports `PublicAnnouncementHandoff`, and `MaturityRootFact` and `MaturityRootFactKind` beside the checkpoint types |
| `:116–136`, `:140–160`, `:164–186` | nineteen items, `ALL: &'static [Self; 19]`, nineteen names | `CheckpointPremiseProvenance` after `CheckpointSuccessor`, `ALL: &'static [Self; 20]`, the name `checkpoint-premise-provenance` |
| `:351–371`, `:430–443` | the continuity operand and the binding over it | the handoff operand; the new item after `:443` |
| `:586–593` | two constant lines | the carried names |
| `packages/vectors/src/maturity_evidence.rs:1956–1971` | the doc says the checkpoint binds the plan's bundle and ABI | it binds the bundle the accepted continuity was projected under and the ABI over its view, checked against the public handoff; the plan's own bundle and ABI at `:1987–1995` stay the §14.1 inputs the first-party evidence is taken over |
| `:2011–2056` | continuity, edge, checkpoint over the plan's premises, history report, then the handoff and the recovery report | continuity, edge, the handoff (moved from `:2047–2049`), `checkpoint_premises_of(&continuity)?`, `bind(&edge, &handoff, ..)`, the history report validated over `&handoff`, the recovery report over the same handoff |

Section 2.4 lists every test site the operand change reaches. The tests, by name and the fact each asserts:

| Test | What it asserts |
|---|---|
| `the_checkpoint_binds_eight_facts_and_declares_two` (retyped, name kept) | the eight carried or derived facts; the declared bundle equals `continuity.bundle()`; in place of the `assert_ne!` at `packages/vectors/src/maturity_history.rs:1124–1128`, the declared ABI's schedule and the declared bundle's record schedule both equal `handoff.schedule()`, which is `StateWitnessSchedule::VariableMetadata`; `binds(&edge, &handoff)` is `Ok(())`; the roster assertions unchanged |
| `the_other_archives_facts_do_not_bind_the_accepted_handoff` (renamed from `..._accepted_continuity`) | the historical acceptance refuses first; the historical handoff refuses `TransactionIdentityDisagrees` with both identities; binding it against the accepted corpus refuses `SubmittedBytesDisagree` with both digests; a changed successor refuses `SuccessorIsNotTheHandoffsStateOutput` at `bind` and `SuccessorOutpointDisagrees` at `binds` on the accepted checkpoint; a changed predecessor refuses `PredecessorOutpointDisagrees` naming input 0; each refusal's `fact()` |
| `a_premise_that_did_not_produce_the_accepted_bytes_refuses_the_binding` (new, history) | the plan's whole-metadata bundle refuses `LinkedCandidateDisagrees(WitnessSchedule { declared: WholeMetadata, published: VariableMetadata })`; the right bundle beside the plan's ABI refuses `CandidateAbiDisagrees(WitnessSchedule { .. })`; the right premises against a handoff restated with lead bounds `(4, 7)` refuse `LinkedCandidateDisagrees(LeadBounds { .. })`, which fixes the comparison order; `fact()` names `LinkedCandidate` and `CandidateAbi` |
| `the_accepted_archives_history_report_validates_and_renders_three_residuals_in_order` (extended) | as now, and the rendered text contains `checkpoint_linked_candidate declared deployment-declaration\n`, `checkpoint_candidate_abi declared deployment-declaration\n` and `recomputed checkpoint-premise-provenance\n` |
| `a_checkpoint_another_handoff_does_not_bind_refuses_at_the_checkpoint_item` (renamed from `a_checkpoint_the_continuity_does_not_bind_...`) | validating the accepted report against the historical handoff refuses `CheckpointRefused` carrying `TransactionIdentityDisagrees` |
| `a_premise_declared_from_executor_arguments_refuses_at_the_provenance_item` (new, history report) | a report over a checkpoint whose linked-candidate premise carries `ExecutorArguments` refuses `ItemDiffers { item: CheckpointPremiseProvenance }`, and so does one whose ABI premise carries it |
| `the_recomputation_inventory_is_complete_and_distinctly_named` | `ALL` has twenty members with twenty distinct names |

Exhaustive matches the members reach, and no others: over `MaturityRootCheckpointRefusal`, `fact()` alone; the test matchers name single variants or use `{ .. }` (`packages/vectors/src/maturity_history_report.rs:872`) and survive the operand renames. Over `MaturityRootHistoryRecomputedItem`, `name()` and `ALL`; `failed_item` matches the refusal (`:216–225`). `MaturityPremiseProvenance` is compared with `==` everywhere and matched nowhere. The checkpoint's struct literal is `bind`'s alone.

Counts: tests in `packages/vectors/src/maturity_history.rs` 21 → 22, in `packages/vectors/src/maturity_history_report.rs` 9 → 10; every other file of section 2.5 unchanged; no census figure moves. Estimate: production 300–420, tests 260–380, carrying one total name mapping, one five-parameter comparison helper and ten documented variants.

Gate:

| Reading | Must be |
|---|---|
| `grep -c '#\[test\]'` over history and history report | 22 and 10 |
| `grep -c '#\[test\]'` over the other fourteen files of section 2.5 | each unchanged |
| `grep -c 'unreachable!' packages/vectors/src/maturity_history.rs` | 2, from 3 |
| `grep -n '# Panics' packages/vectors/src/maturity_history.rs` | two lines, above `MaturityTransitionCertificate::from_continuity` and `ModelledBranchPrefix::context`, none above `bind` |
| `grep -n 'ValidatedMaturityContinuity' packages/vectors/src/maturity_history_report.rs` | no line above the test module |
| `grep -n 'declared deployment-declaration' packages/vectors/src/maturity_history_report.rs` | no line above the test module |
| `grep -n '\.expect(\|\.unwrap()' packages/vectors/src/maturity_evidence.rs` | no line above the test module |
| `grep -c 'checkpoint_premises_of' packages/vectors/src/maturity_evidence.rs` | at least 3: the definition, the derivation and the evidence test |
| `grep -c '#\[allow'` over the five files | 0 in each; every `#[expect]` carries a `reason` |
| `git diff --name-only` against the bite's base | exactly the five files |
| `the_census_figures_are_unmoved_by_the_binding_readers` | every figure unchanged |

What this bite does not establish: the premises remain declarations, checked only through the parameters a deployment publishes; the declared bundle is carried whole, retained instance included; the edge is still the continuity's projection until bite 2; no chain is observed and no row standing moves.

### Bite 2 — the root edge built from the public handoff (`T11-120`)

Files: `packages/vectors/src/maturity_recovery.rs`, `packages/vectors/src/maturity_history.rs`, `packages/vectors/src/maturity_evidence.rs`, `packages/vectors/tests/maturity_recovery.rs`. Four files, none new.

Shape: recovery keeps the two outpoints it reads; the edge and its certificate are built from them and the handoff's subtree by a constructor taking the handoff alone; the derivation validates and reports that edge (D3).

Reused: `recover_public_successor`, `RecoveredSuccessor`, `Outpoint::new`, `TargetInput::outpoint`, `MaturityContinuityRefusal::InputCount` (`packages/vectors/src/maturity_continuity.rs:601`), `validate_state_root_history`; the certificate's private fields, which the new constructor writes from inside `packages/vectors/src/maturity_history.rs`.

`RecoveredSuccessor` gains two private fields after `contract`, `predecessor: Outpoint` and `successor: Outpoint`, and two `const` accessors `predecessor(&self) -> Outpoint` ("Input 0 of the published transaction") and `successor(&self) -> Outpoint` ("The recomputed identity at the stated STATE output index"); its derives are unchanged. No refusal variant is added: inside `recover_public_successor`, after the output-index check at `packages/vectors/src/maturity_recovery.rs:393–395`, `Outpoint::new(recomputed, stated)` maps its refusal to `WrongOutputIndex { stated, declared }`, and after the witness reading at `:397–406`, `transaction.inputs().first().map(TargetInput::outpoint)` maps absence to `MalformedPublicWitness` carrying `InputCount { actual: 0 }`; both are unreachable after the checks before them and are typed rather than asserted. The struct literal at `:511–518` gains both fields, and `:19` imports `Outpoint` and `TargetInput`.

| Function | Signature | What it does |
|---|---|---|
| `StateRootEdge::from_public_handoff` | `pub fn from_public_handoff(handoff: PublicAnnouncementHandoff) -> Result<Self, MaturityRecoveryRefusal>` | clones `handoff.static_subtree()` before moving the handoff into `recover_public_successor`, then writes the certificate with the landed constants, `recovered.predecessor()` and `recovered.successor()`, and the published subtree on both sides; the edge's endpoints read from the certificate as `from_continuity` reads them; an `# Errors` section, and no lint expectation because the handoff is consumed |
| `MaturityEvidenceRefusal::RootHistoryEdgeRefused` (new member) | `RootHistoryEdgeRefused(Box<MaturityRecoveryRefusal>)`, after `AcceptedContinuityRefused` | "The public handoff refused the root-history edge's recovery."; twenty members become twenty-one; the enum is `#[non_exhaustive]` and matched by no arm in `packages/` |

Existing lines that change: `packages/vectors/src/maturity_evidence.rs:2012`, `StateRootEdge::from_continuity(&continuity)`, becomes `StateRootEdge::from_public_handoff(handoff.clone())` mapped to `RootHistoryEdgeRefused`, after the handoff bite 1 moved above it; the continuity stays in the derivation for `checkpoint_premises_of`, because the premises are the creator's declarations and the replay is how the derivation obtains the declared values; `:1624–1691` gains the member; `packages/vectors/tests/maturity_recovery.rs:6–34` gains `use vectors::maturity_history::StateRootEdge;` and `use vectors::maturity_history::validate_state_root_history;` after the corpus line, and the expected list at `:166–196` gains the same two lines in the same place.

Tests:

| Test | What it asserts |
|---|---|
| `the_handoff_built_edge_agrees_with_the_projected_edge_field_by_field` (new, history) | over the accepted archive, the handoff-built edge's predecessor, successor and operation and its certificate's projection, operation, root, use, predecessor and successor equal the projected edge's, and both static roots equal the projected edge's; `validate_state_root_history` over the one handoff-built edge from its predecessor returns its successor at output 0; `StateRootEdge::from_public_handoff` binds to `fn(PublicAnnouncementHandoff) -> Result<StateRootEdge, MaturityRecoveryRefusal>`; a handoff restated with output index 1 refuses `WrongOutputIndex { stated: 1, declared: 0 }` |
| `the_accepted_archives_root_edge_is_built_from_the_handoff_alone` (new, binary) | from the binary's own `accepted_handoff()`, the edge validates from its predecessor to a cursor whose `txid()` is the readback identity and whose `index()` is 0 |
| `the_binarys_import_list_excludes_the_bundle_the_identity_and_both_planners` | the pinned list has 31 lines and the forbidden fragments stay absent |

Under `validate_state_root_history` the handoff-built edge meets §17.1's clauses as D3 states; the test asserts the whole run, and §17.3's inequality receives two copies of one subtree, so its check for this edge is recovery's item-three and output-0 comparisons.

Exhaustive matches: over `MaturityRecoveryRefusal`, `row()` (`packages/vectors/src/maturity_recovery.rs:317–337`) — unchanged, no variant added; `RecoveredSuccessor`'s literal is `recover_public_successor`'s alone; over `MaturityEvidenceRefusal`, none.

Counts: history 22 → 23, the binary 3 → 4, recovery 13 and evidence 22 unchanged. Estimate: production 80–130, tests 90–150, carrying two documented accessors.

Gate:

| Reading | Must be |
|---|---|
| `grep -c '^use ' packages/vectors/tests/maturity_recovery.rs` | 31 |
| `grep -c '#\[test\]'` over history, the binary, recovery and evidence | 23, 4, 13 and 22 |
| `grep -n 'StateRootEdge::from_continuity' packages/vectors/src/maturity_evidence.rs` | no line above the test module |
| `grep -c 'RootHistoryEdgeRefused' packages/vectors/src/maturity_evidence.rs` | 2 |
| `git diff --name-only` | exactly the four files |

What this bite does not establish: the edge's static subtree is the published one, whose leaves beyond the witnessed leaf are not available from the handoff (D4's non-claim); the checkpoint's premises stay declarations; no chain is observed.

### Bite 3 — the five recovery non-claims and the subtree census (`T11-121`)

Files: `packages/vectors/src/maturity_recovery_report.rs` alone.

Shape: `MaturityPublicRecoveryResidual` gains the criterion's five non-claims after `WitnessedNonceWithoutLeastness`; each gains a recomputed item at its position; the census gains `static_leaves` with its item; the renderer writes the census line, and the residual and item lines follow from the rosters (D4).

| Member | Name | Recomputed item | Item name |
|---|---|---|---|
| `NoIndependentImplementation` | `no-independent-implementation` | `IndependentImplementationResidual` | `independent-implementation-residual` |
| `NoTrustAnchor` | `no-trust-anchor` | `TrustAnchorResidual` | `trust-anchor-residual` |
| `NoInclusionProof` | `no-inclusion-proof` | `InclusionProofResidual` | `inclusion-proof-residual` |
| `NoCanonicalChainAuthentication` | `no-canonical-chain-authentication` | `CanonicalChainAuthenticationResidual` | `canonical-chain-authentication-residual` |
| `NoStaticSubtreeDataAvailability` | `no-static-subtree-data-availability` | `StaticSubtreeAvailabilityResidual` | `static-subtree-availability-residual` |
| census `static_leaves: usize` | — | `CensusStaticLeaves`, after `CensusRows` | `census-static-leaves` |

Existing lines that change: `:58–82`, the residual enum, `ALL` from `&'static [Self; 4]` to `&'static [Self; 9]` and `name()` gaining five arms; `:158–177`, the census gaining the field and `pub const fn static_leaves(&self) -> usize`; `:208–235`, `:239–266` and `:270–299`, the items, `ALL` from `&'static [Self; 26]` to `&'static [Self; 32]` and six name arms; `:519–555`, assembly computing `let static_leaves = handoff.static_subtree().leaves().len();` before the struct literal, because the literal moves `handoff` at `:537` before it reaches `census`; `:694–698`, five position items after `LeastnessResidual` at positions 4 to 8; `:709–713`, `CensusStaticLeaves` comparing the stated census with `handoff.static_subtree().leaves().len()`; `:816–818`, `static_leaves {}` after `rows {}`. The first residual item's length check already compares against `ALL.len()` and so carries the new roster (`:679–680`).

For the accepted handoff the canonical bytes gain five lines after `residual witnessed-nonce-without-leastness` — `residual no-independent-implementation`, `residual no-trust-anchor`, `residual no-inclusion-proof`, `residual no-canonical-chain-authentication`, `residual no-static-subtree-data-availability` — the line `static_leaves 1` after `rows 14`, and six `recomputed` lines in roster order, five after `recomputed leastness-residual` and `recomputed census-static-leaves` after `recomputed census-rows`.

Tests:

| Test | What it asserts |
|---|---|
| `the_accepted_handoffs_recovery_report_validates_and_renders_nine_residuals_in_order` (renamed from `..._four_residuals_in_order`) | 32 recomputed items; nine residual lines in canonical order |
| `the_recomputation_inventory_is_complete_and_distinctly_named` | 32 members, 32 distinct names |
| `no_role_spelling_crosses_between_the_reports` (extended) | each of the five new residual lines is in the recovery bytes and absent from the history bytes |
| `the_static_subtree_non_claim_stands_beside_a_one_leaf_census` (new) | the accepted report's `static_leaves()` is 1 and its bytes carry `static_leaves 1\n` and `residual no-static-subtree-data-availability\n`; a census stating 2 refuses `ItemDiffers { item: CensusStaticLeaves }`; removing the last residual refuses at `ChainObservationResidual`, whose length check carries the roster; swapping the independent-implementation and trust-anchor residuals refuses at `IndependentImplementationResidual` |

Exhaustive matches: over `MaturityPublicRecoveryResidual`, `name()` alone; over `MaturityPublicRecoveryRecomputedItem`, `name()` and `ALL`; the census literal is assembly's alone (`:538–542`). No other file names either enum.

Counts: recovery report 11 → 12; every other file unchanged. Estimate: production 110–170, tests 70–120, carrying two total name mappings of five and six arms.

Gate:

| Reading | Must be |
|---|---|
| `grep -c '#\[test\]' packages/vectors/src/maturity_recovery_report.rs` | 12 |
| `grep -c "&'static \[Self; 9\]\|&'static \[Self; 32\]" packages/vectors/src/maturity_recovery_report.rs` | 2 |
| `grep -c '"no-independent-implementation"\|"no-trust-anchor"\|"no-inclusion-proof"\|"no-canonical-chain-authentication"\|"no-static-subtree-data-availability"'` over the recovery report and the history report | 5 and 0 |
| `git diff --name-only` | exactly the one file |

What this bite does not establish: a non-claim states an absence and discharges nothing it names; the report still observes no chain; and `G14C-13` closes on these bytes only through the closure row's pointer.

### Bite 4a — realizations on modelled branches, the invalidation event and the thread's semantic history (`T11-122`)

Files: a new `packages/vectors/src/maturity_branch.rs` (production and tests), `packages/vectors/src/lib.rs`, `packages/vectors/meson.build`. Three files, one new.

Shape: a realization places an edge at a block height of a modelled branch with the operator key the caller states for it; a branch view is a modelled prefix and its realizations; a thread holds a starting cursor, its operator key and its intended steps, each a semantic edge beside the edge built to realize it; `realize` extends a view by one block and its realizations, `rewind` returns the shorter view and a typed invalidation event naming exactly the realizations it removed, and `project` reads the thread's chain on one view, its cursor and the semantic steps that view does not realize (D6).

Reused: `ModelledBranchBlock` with `new`, `height()`, `identity()`, `transactions()`; `ModelledBranchPrefix` with `anchored`, `extend`, `rewind`, `identifier()`, `tip_height()`, `blocks()`; `MaturityModelledBranchRefusal`; `StateRootEdge` and its accessors; `validate_state_root_history` and `MaturityRootHistoryRefusal`; `RecoveredSuccessor::predecessor_metadata`, `requested_cycle` and `successor_semantics`; `ValidatedMaturityContinuity::predecessor`, `requested_cycle` and `expected_successor`; `OperatorKey` — all public, rows of section 2.3. `ModelledBranchPrefix::context` is not called.

New types, all fields private, none an `Option`:

| Type | Fields and derives |
|---|---|
| `MaturitySemanticEdge` | `predecessor: StateMetadata`, `requested_cycle: Cycle`, `successor: StateMetadata`; `Clone, Copy, Debug, PartialEq, Eq` |
| `MaturityRealization` | `height: u32`, `edge: StateRootEdge`, `operator: OperatorKey`; `Clone, Debug, PartialEq, Eq` |
| `MaturityBranchView` | `prefix: ModelledBranchPrefix`, `realizations: Vec<MaturityRealization>`; `Clone, Debug, PartialEq, Eq` |
| `MaturityRealizationSuffixInvalidation` | `branch: Digest32`, `rewound_to: u32`, `invalidated: Vec<MaturityRealization>`; `Clone, Debug, PartialEq, Eq` |
| `MaturityIntendedStep` | `semantic: MaturitySemanticEdge`, `edge: StateRootEdge`; `Clone, Debug, PartialEq, Eq` |
| `MaturityThread` | `starting_cursor: Outpoint`, `operator: OperatorKey`, `steps: Vec<MaturityIntendedStep>`; `Clone, Debug, PartialEq, Eq` |
| `MaturityThreadProjection` | `branch: Digest32`, `realized: Vec<MaturityRealization>`, `cursor: Outpoint`, `unrealized: Vec<MaturitySemanticEdge>`; `Clone, Debug, PartialEq, Eq` |
| `MaturityBranchRefusal` | `Prefix(MaturityModelledBranchRefusal)`, `RealizationHeightDiffers { realization: u32, block: u32 }`, `RealizationAbsentFromItsBlock { height: u32, transaction: Txid }`, `ThreadSequenceRefused(Box<MaturityRootHistoryRefusal>)`; `Clone, Debug, PartialEq, Eq` |

Every function, with its exact signature:

| Function | Signature | What it does |
|---|---|---|
| `MaturitySemanticEdge::from_recovered` | `pub const fn from_recovered(recovered: &RecoveredSuccessor) -> Self` | the decoded predecessor's semantics, the witnessed cycle and the derived successor, from the public process |
| `MaturitySemanticEdge::from_continuity` | `pub const fn from_continuity(continuity: &ValidatedMaturityContinuity) -> Self` | the same three from a projection |
| its accessors | `predecessor() -> StateMetadata`, `requested_cycle() -> Cycle`, `successor() -> StateMetadata` | three, `const`, by value |
| `MaturityRealization::new` | `pub const fn new(height: u32, edge: StateRootEdge, operator: OperatorKey) -> Self` | a caller-stated realization, like a modelled block's transaction list; its accessors `height()`, `edge() -> &StateRootEdge`, `operator() -> &OperatorKey`, all `const` |
| `MaturityBranchView::forked` | `pub const fn forked(prefix: ModelledBranchPrefix) -> Self` | a view with no realization, the fork point being the prefix's anchor block |
| `MaturityBranchView::realize` | `pub fn realize(&self, block: ModelledBranchBlock, realizations: Vec<MaturityRealization>) -> Result<Self, MaturityBranchRefusal>` | refuses a realization whose height is not the block's and one whose edge's successor transaction is not in the block, before the prefix moves; then `extend` mapped to `Prefix`; returns a new view with the realizations appended; `# Errors` |
| `MaturityBranchView::rewind` | `pub fn rewind(&self, height: u32) -> Result<(Self, MaturityRealizationSuffixInvalidation), MaturityBranchRefusal>` | the prefix's `rewind` mapped to `Prefix`; realizations above `height` leave the view and are named, in order, by the event, whose `branch` is the view's identifier and `rewound_to` the height; `# Errors` |
| the view's and the event's accessors | `prefix() -> &ModelledBranchPrefix`, `realizations() -> &[MaturityRealization]`; `branch() -> &Digest32`, `rewound_to() -> u32`, `invalidated() -> &[MaturityRealization]` | the slice readers not `const` |
| `MaturityThread::anchored` | `pub const fn anchored(starting_cursor: Outpoint, operator: OperatorKey) -> Self` | a thread with no step |
| `MaturityThread::record` | `pub fn record(&self, semantic: MaturitySemanticEdge, edge: StateRootEdge) -> Self` | a new thread with one step appended; no view operation takes a thread by value or by mutable reference, so a rewind cannot reach semantic history |
| `MaturityThread::project` | `pub fn project(&self, view: &MaturityBranchView) -> Result<MaturityThreadProjection, MaturityBranchRefusal>` | walks the view's realizations in order, collecting each whose edge is one of the thread's intended edges and whose predecessor is the running cursor; validates the collected edges from the starting cursor with `validate_state_root_history` when any were collected, mapping a refusal to `ThreadSequenceRefused`; the unrealized steps are those whose edge was not collected; `# Errors` |
| the thread's, step's and projection's accessors | `starting_cursor()`, `operator()`, `steps()`; `semantic()`, `edge()`; `branch()`, `realized()`, `cursor()`, `unrealized()` | `const` except the slice readers |

Module and census lines: `pub mod maturity_branch;` enters `packages/vectors/src/lib.rs` after `pub mod matrix;` (`:50`) and before `pub mod maturity_closure;`; `'src/maturity_branch.rs',` enters the Rust-source census of `packages/vectors/meson.build` before `'src/maturity_closure.rs',` (`:54`).

Tests, in the new module, over the accepted archive's projection from `variable_archived()` (`packages/vectors/src/maturity_continuity.rs:2344`), the readback's block height and hash, and a competing successor built in the test module: from the accepted predecessor's funded coin, retained instance, branch and bundle, a sponsorless request at the window's earliest cycle, where the planner asks for the earliest plus one (`packages/vectors/src/maturity_native.rs:805–813`), constructed, finalized and signed under `OPERATOR_HANDLE` exactly as the native module's independent builders are (`:1196–1288`, with the import paths at `:29–43`), then projected with the accepted source's bundle, identity and funded coin under `MaturityByteSource::NodeFreeSubmitReady`. The operator keys are read from each projection's `bundle().deployment().operator().key()`. Branch A's identifier is `identifier: [0xa1; 32]`, B's `identifier: [0xb1; 32]`, and both anchor at one block one height below the readback's with `identity: [0x60; 32]`.

| Test | What it asserts |
|---|---|
| `a_fork_after_one_predecessor_realizes_competing_successors_on_their_own_branches` | A realizes the accepted edge in the readback's block and B the competing edge in a block with `identity: [0xb2; 32]` at the same height; the accepted thread projects on A to the accepted successor with no unrealized step and on B to its starting cursor with the accepted semantic edge unrealized; the competitor's thread, recording the competing step, projects on B to the competing successor; the two semantic edges share their predecessor and differ in requested cycle and successor; `validate_state_root_history` over both edges from the predecessor refuses `OutpointReused` at position 1, first seen at position 0 on the predecessor side |
| `rewinding_one_branch_invalidates_only_its_realization_suffix_and_keeps_semantic_history` | A, extended past the accepted block by an empty block, rewound to the anchor height returns an event naming A, the anchor height and exactly the accepted realization; B is equal to itself before the rewind; the thread's steps are unchanged; the thread projects on the rewound view to its starting cursor with the accepted semantic edge unrealized; rewinding above the tip refuses `Prefix(RewoundAboveTheTip { .. })` |
| `re_realizing_the_same_block_reproduces_identical_bytes_and_an_identical_view` | a fresh planner replay of the accepted archive — the corpus's identity, branch and `Retained(StateWitnessSchedule::VariableMetadata)`, stepped through its exchanges as `variable_archived()` steps (`packages/vectors/src/maturity_continuity.rs:2349–2371`) — yields submission bytes equal to the readback's; their projection's edge equals the original edge; re-realizing the rewound A with the same block and that edge yields a view equal to A before the rewind |
| `a_realization_outside_its_block_refuses_before_the_prefix_moves` | a realization stated at another height refuses `RealizationHeightDiffers`, one whose successor transaction the block does not list refuses `RealizationAbsentFromItsBlock`, and a block above the tip's successor refuses `Prefix(NonContiguousExtension { .. })` |

Exhaustive matches: none outside the new module; `MaturityModelledBranchRefusal` is carried whole, not matched.

Counts: tests in `packages/vectors/src/maturity_branch.rs` at 4; every file of section 2.5 unchanged; `packages/vectors/meson.build` gains one entry. Estimate: production 380–520, tests 380–520, carrying some twenty documented accessors and the staged competing builder in the tests.

Gate:

| Reading | Must be |
|---|---|
| `grep -c '#\[test\]' packages/vectors/src/maturity_branch.rs` | 4 |
| `grep -n 'Option<' packages/vectors/src/maturity_branch.rs` | no struct field |
| `grep -n '\.expect(\|\.unwrap()\|# Panics\|unreachable!\|panic!' packages/vectors/src/maturity_branch.rs` | no line above the test module |
| `grep -c 'context()' packages/vectors/src/maturity_branch.rs` | 0 above the test module |
| `grep -c '#\[allow' packages/vectors/src/maturity_branch.rs` | 0; every `#[expect]` carries a `reason` |
| `git diff --name-only` | exactly the three files |

What this bite does not establish: every block and realization is caller-stated, so a view is a model and not a chain view; the competing successor is a host-built candidate never submitted to a target; the token and the class-2 vocabulary are bites 4b and 5.

### Bite 4b — the branch-bound compare-and-swap token and its fence (`T11-123`)

Files: `packages/vectors/src/maturity_branch.rs` alone.

Shape: a view reports the live cursor reached from an outpoint by any realization that spends it, whoever made it; a thread mints a token from a view — the view's identifier, its tip block and that live cursor — and fences the token against the view current at submission, refusing another branch, a checkpoint block the view no longer carries or carries changed, and a moved cursor (D6).

| Item | Shape |
|---|---|
| `MaturityBranchViewToken` | `branch: Digest32`, `checkpoint_height: u32`, `checkpoint_block: Digest32`, `cursor: Outpoint`; `Clone, Copy, Debug, PartialEq, Eq`; four `const` accessors |
| `MaturityFencingRefusal` | `AnotherBranch { minted: Digest32, presented: Digest32 }`, `CheckpointBlockRemoved { height: u32, minted: Digest32, tip: u32 }`, `CheckpointBlockChanged { height: u32, minted: Digest32, current: Digest32 }`, `CursorMoved { minted: Outpoint, current: Outpoint }`; `Clone, Copy, Debug, PartialEq, Eq` — the two block refusals mirror the landed observation refusals' removed and changed cases (`packages/vectors/src/maturity_history.rs:570–579`) |
| `MaturityBranchRefusal::PrefixCarriesNoBlock` (new member) | `{ branch: Digest32 }`; unreachable while `anchored` admits one block, and typed rather than asserted |
| `MaturityBranchView::live_cursor` | `pub fn live_cursor(&self, from: Outpoint) -> Outpoint` — follows the realizations in order, each whose predecessor is the running outpoint advancing it to its successor |
| `MaturityThread::token` | `pub fn token(&self, view: &MaturityBranchView) -> Result<MaturityBranchViewToken, MaturityBranchRefusal>` — the tip from `view.prefix().blocks().last()`, refusing `PrefixCarriesNoBlock`; the cursor `view.live_cursor(self.starting_cursor)`; `# Errors` |
| `MaturityThread::fence` | `pub fn fence(&self, token: &MaturityBranchViewToken, view: &MaturityBranchView) -> Result<(), MaturityFencingRefusal>` — in order the branch identifier, the block at the token's height (absent, then changed), then the live cursor; `# Errors` naming the four refusals and the statement that the fence is the call a submitter makes immediately before handing bytes to a target |

Test, one in the module:

| Test | What it asserts |
|---|---|
| `a_token_fences_another_branch_a_lost_checkpoint_and_a_moved_cursor` | a token the accepted thread mints on A's anchor-only view fences that view and the view extended by an empty block without refusal, which is a compare-and-swap on the cursor and not an equality on the tip; the view realizing the accepted edge refuses `CursorMoved { minted: predecessor, current: accepted successor }`; B's anchor-only view refuses `AnotherBranch`; a token minted after an empty block with `identity: [0xa2; 32]` at the accepted height refuses `CheckpointBlockRemoved` once the view is rewound to the anchor and `CheckpointBlockChanged` once the rewound view is extended by a block with `identity: [0xa3; 32]` at that height |

Exhaustive matches: none; `MaturityBranchRefusal` is matched nowhere at 4a's tip.

Counts: `packages/vectors/src/maturity_branch.rs` 4 → 5. Estimate: production 150–230, tests 90–140, carrying four documented accessors.

Gate:

| Reading | Must be |
|---|---|
| `grep -c '#\[test\]' packages/vectors/src/maturity_branch.rs` | 5 |
| the panic reading of bite 4a | no line above the test module |
| `grep -rn '\.fence(' packages/vectors` | lines in `packages/vectors/src/maturity_branch.rs`'s test module only: no production submitter calls the fence (the labels crate's `nested_fence` and `unclosed_fence` do not match the leading dot) |
| `git diff --name-only` | exactly the one file |

What this bite does not establish: no submitter in the tree calls the fence, because no layer here holds a branch view of a real target; the token fences against a modelled view, and its placement at submission is the caller's contract, stated in its documentation.

### Bite 5 — class-2 observations of a thread on a modelled branch (`T11-124`)

Files: `packages/vectors/src/maturity_branch.rs` alone.

Shape: the thread classifies every realization on a view that is neither its own chain nor unrelated — a spend of an outpoint its chain reached is a competing spend, or a reorganization-surfaced realization when an invalidation event the caller presents removed the thread's intended realization at that predecessor; a realization under the thread's operator key that continues nothing of the thread is a same-key instance — and takes the standing the worst of them gives (D5).

| Item | Shape |
|---|---|
| `MaturityThreadStanding` | `Intended`, `Contested`, `Lost`; `Clone, Copy, Debug, Eq, PartialEq`; `pub const fn name(self) -> &'static str` total: `intended`, `contested`, `lost` |
| `MaturityEnvironmentTransition` | `CompetingSpend`, `SameKeyInstance`, `ReorganizationSurfacedRealization`; `Clone, Copy, Debug, Eq, PartialEq`; `pub const ALL: &'static [Self; 3]`; `pub const fn name(self) -> &'static str` total: `competing-spend`, `same-key-instance`, `reorganization-surfaced-realization`; `pub const fn class(self) -> MaturityObservationClass`, always `ModeledUnintendedEnvironmentTransition`; `pub const fn standing(self) -> MaturityThreadStanding` total: `Lost`, `Contested`, `Lost` |
| `MaturityEnvironmentRealization` | `realization: MaturityRealization`, `transition: MaturityEnvironmentTransition`; `Clone, Debug, PartialEq, Eq`; two accessors |
| `MaturityThreadObservation` | `projection: MaturityThreadProjection`, `environment: Vec<MaturityEnvironmentRealization>`, `standing: MaturityThreadStanding`; `Clone, Debug, PartialEq, Eq`; three accessors |
| `MaturityThread::observe` | `pub fn observe(&self, view: &MaturityBranchView, invalidations: &[MaturityRealizationSuffixInvalidation]) -> Result<MaturityThreadObservation, MaturityBranchRefusal>` — projects the thread; the outpoints its chain reached are the starting cursor and each realized edge's successor; each other realization in order is a competing spend or a reorganization-surfaced realization when its predecessor is one of those outpoints, a same-key instance when its operator equals the thread's, and unrelated otherwise; the standing is `Lost` if any classified transition's standing is, else `Contested` if any is, else `Intended`; `# Errors` |

No type here has a field of `MaturityBranchPoisonMarker`'s type, and none offers a conversion into one, so a class-3 observation stays representable only as the marker (`packages/vectors/src/maturity_evidence.rs:268–321`).

Tests, four in the module:

| Test | What it asserts |
|---|---|
| `a_competing_spend_of_the_predecessor_loses_the_thread_and_fences_its_token` | on B realizing the competing edge the accepted thread observes one `CompetingSpend`, standing `Lost`, its projection's cursor the starting cursor; a token the thread minted on B's anchor-only view refuses `CursorMoved { minted: predecessor, current: competing successor }` |
| `a_same_key_instance_contests_the_thread_without_moving_its_cursor` | the operator keys of the accepted and node-free projections are equal, the node-free candidate being built by the planner under the same published signer handle (`packages/vectors/src/maturity_continuity.rs:2437–2494`); A realizing the accepted edge and, one block later, the node-free edge gives one `SameKeyInstance`, standing `Contested`; the thread's cursor and the view's live cursor from the predecessor are both the accepted successor |
| `a_reorganization_surfaced_realization_loses_the_thread_and_names_the_invalidation` | A realizing the accepted edge, rewound to the anchor and re-extended with B's competing block, observed with the rewind's event gives one `ReorganizationSurfacedRealization`, standing `Lost`, and without it one `CompetingSpend`: the distinction rests on the typed event; the event names exactly the accepted realization and the thread's steps are unchanged |
| `every_environment_transition_is_class_two_under_the_quantifier` | each member of `ALL` has class `ModeledUnintendedEnvironmentTransition`, which `is_quantified_over()`, and the standing the table states; the three names and the three standing names are distinct; `MaturityEvidenceCensus::default().quantifier().excluded()` is `ModelFalsifyingWithNoPreimage` (`packages/vectors/src/maturity_evidence.rs:1327`, `:1497`) |

Exhaustive matches: `name()`, `class()` and `standing()` over the two new enums; `MaturityObservationClass` is returned, not matched.

Counts: `packages/vectors/src/maturity_branch.rs` 5 → 9. Estimate: production 230–340, tests 250–360, carrying three total name mappings and one total standing mapping.

Gate:

| Reading | Must be |
|---|---|
| `grep -c '#\[test\]' packages/vectors/src/maturity_branch.rs` | 9 |
| `grep -c 'MaturityBranchPoisonMarker' packages/vectors/src/maturity_branch.rs` | 0 above the test module |
| the panic reading of bite 4a | no line above the test module |
| `git diff --name-only` | exactly the one file |

What this bite does not establish: every environment transition is constructed in the model from bytes the tree holds; none was observed on a target, the same-key comparison reads keys from projected continuities because no public reader of an operator key exists, and no row standing and no census figure moves.

### Bite 6 — the two stale comments (`T11-125`)

Files: `packages/vectors/src/maturity_safety.rs` alone, comments only.

| Where | Now | After |
|---|---|---|
| `:223–237`, the documentation of `AcceptanceAwaitsRelayAdmissibility` | says the spend is stopped by a standardness width and that neither route to an acceptance has landed | says the constant-elision route closed on `0.6.245-dev` and its accepted archive answers the sponsorless row, that the rows standing here ask for an acceptance of their own shape that no admitted run supplies, that their classification belongs to the accepted-byte classification study's first two open questions, and — kept — that a relaxed node policy is not a route |
| `:1430–1433`, above the sponsorless row | says the row is the one a run answers and its spend is stopped by a width | says the row is the one §16.1 row an admitted run answers: the accepted archive carries its acceptance, so its standing is `NativeAcceptanceObserved` while its declared boundary stays the relay layer the historical run observed |
| `:3789–3791`, in `each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names` | eleven rows and a twelfth | eleven positive rows ask for an acceptance no admitted run supplies, §16.1's last, four-term row among them |

Gate: every added and removed line of `git diff -U0` is a `//` or `///` line; the twenty tests of the file unchanged; `git diff --name-only` names the one file. Estimate: 20–40 changed comment lines and no production code line.

What this bite does not establish: it changes no assertion and no standing; finding 167's header spellings stay as landed.

### The closure row (`T11-126`)

After bite 6 the closure row records the landed evidence and alone edits `plans/phases/06-state-and-maturity.md`, `plans/guides/guide_fourteen_conceptual_register.md` and the backlog rows' dispositions: findings for each bite; the Status cells of `G14C-11`, `G14C-13` and `G14C-12` with evidence pointers carrying D8's qualifiers; the count of `G14C-13`'s non-claims corrected from four to five in its Status cell, in the closure record's register-exit paragraph and in finding 169 (`plans/phases/06-state-and-maturity.md:1038`, `:1068`); the citations of the three renamed tests retyped (`:1050`, `:1054`); the fourth exit and §23's eleventh conjunct restated on the landed tags; and the register exit decided on that evidence.

### The exits and register clauses these bites reach

The first exit stays MET. §23's eleventh conjunct loses its second condition at bite 2 — the edge is built from the public handoff alone — and stays CONDITIONALLY MET on its first, the disclosed zero-fee-floor environment at (`sec:phase6:wave9-findings`). The fourth exit's root-history half moves at bites 1 and 2 from NOT MET on three grounds to CONDITIONALLY MET on one: the edge is public, the premises are the ones under which the accepted bytes reproduce and the report reads their provenance, and the checkpoint carries the declared linked candidate whole, retained instance included, because §17.2 binds it and no verdict reads the instance (OQ2). The recovery half stays MET with its landed condition.

Each clause closes on the sentence its Status cell states, satisfied by the named bite and carried by the closure row's pointer with D8's qualifiers. `G14C-13`: "The row closes when the report carries typed non-claims for independent implementation, trust anchor, inclusion proof and static-subtree data availability beside its four residuals" (`plans/guides/guide_fourteen_conceptual_register.md:121`) — satisfied by bite 3, over the criterion's five rather than the sentence's four. `G14C-11`: "The row closes when competing branch realizations, typed realization-suffix invalidation, the branch-bound compare-and-swap token with submission-time fencing and the fork, rewind and byte-identical reproduction tests land; the modelled prefix observes no chain" (`:119`) — satisfied by bites 4a and 4b. Wave 10's clause of `G14C-12`: "the clause closes when the typed class-2 standings and the three environment tests land" (`:120`) — satisfied by bite 5, whose token-fencing assertion reuses bite 4b.

## 6. What this study does not establish

It establishes no compilation and no run: every type, field, signature, refusal variant, test and changed line is checked by reading against the items that exist at `0.6.266-dev` — the visibility of each reused function, the derives of each carried type, the exhaustive matches each new member reaches and the assertions that pin every figure — which shows each shape is expressible over values the tree publishes and shows nothing about whether the modules build, whether the lint gate accepts their argument counts and expectations, or whether any of the tests it names passes. Four readings are checks a code bite makes first, because a reading is cheaper than a rewrite: that the variable-metadata bundle's internal-key policy, lead bounds and static root equal the accepted handoff's, which bite 1's retyped test asserts by binding; that `checkpoint_premises_of` derives an ABI over the replayed bundle's view, which the same test asserts by the schedule; that the competing successor at the window's earliest cycle projects under the accepted source's bundle and funded coin, which bite 4a's fork test asserts; and that the node-free projection's operator key equals the accepted one's, which bite 5's same-key test asserts first.

It observes no chain. A modelled branch, a modelled fork and a modelled rewind are models; the competing successor is a host-built candidate never submitted; the same-key instance is a node-free candidate; the fence is called by no production submitter. It establishes no root-cursor freshness, no origin beyond the admission of pinned bytes, no relayability under a positive fee floor, no resource closure and no artifact promotion, which keeps its gate at (`rule:guide14-exec:candidate-only`). It answers none of the eleven acceptance-awaiting positive rows and moves no row standing and no census figure. Finding 167's header spellings are not cut, because aligning them changes the older reports' canonical bytes and needs an accessor neither `TargetContractVersion` nor `ExecutionDomain` carries. It settles neither OQ1 nor OQ2, both questions about the guide's text. It closes no register row: the rows close when the bites land and the closure row records their pointers. It changes no line of code; it joins the directory's census in `plans/drafts/meson.build` and the README's index in the same commit that adds the backlog rows `T11-119` to `T11-126`, and the documentation gate runs over this file when it lands and over each bite when the bite lands.
