# Research: completing Wave 3 of the Guide 15 execution guide

**Abstract (Proposal status)** · `abst:wave3:scope`

This strategy proposes nineteen bounded packets at `0.7.191-dev`, with the [route study](wave3-route-study.md) as its decision record. Proposed rulings are 221–234; 220 belongs to another landing. Adoption and every future engineering result remain separate from these drafts (`rule:drafts:authority`).

## 1. Complete delivery and receiver trace · `sec:wave3:required-output`

| Deliverable | Producing packets and evidence |
|---|---|
| Validated Burn | 3.2b, 3.3a/b explicit milestone; 3.5c–f required side-qualified private-change alternative before closure |
| Revalidated compact ASH | 3.4 immediately after Burn lowering and before first activation; exact pins rerun through Clear and side-key changes |
| Validated Clear | 3.2c dormant lowering, 3.5a validator and 3.5b oracle/activation |
| Phase-7 scope | 3.6a/b from one bound scope, with declared unselected lifecycle obligations published |
| Five reconciled censuses | Each facade/activation and scope compares relation, carrier, layout, coverage and lifecycle memberships/joins |
| ASH constructor closure | 3.6a/b six typed producer/recognizer comparisons; linked equality later |
| Corruption-resistant validators | Facade/corruption packets plus activation reaching controls and the full required-coordinate table |
| Independent small-instance oracles | Each activation has its own oracle; one Guide-7 independence rule, exact bounded subset enumeration |

This is the full scope of (`task:guide15-exec:wave3`) and (`sec:guide15:compiler`). Both preflight rows are retained: operation-specific sponsor opacity for Burn, compact and Clear, and target conservation carried as external evidence rather than model-proved substrate acceptance (`plans/guides/guide_fifteen.md:242,271`). No explicit-only final narrowing is proposed.

**Table (Receiver and follow-up dispositions)** · `tab:wave3:receivers`

| Receiver | Selected evidence / retained disposition |
|---|---|
| Closing B1, private receipt→Burn | 3.1a own-choice/eligibility; 3.3b live PrivateCommitted in TransferLive+Burn is unavailable while Explicit implements Burn; 3.6b publishes the row absent from selected facades |
| 2.7a B1 realization pin | 3.1a Burn+TransferLive preserves explicit-only input admission despite output/private lifecycle union |
| 2.7a B2 mixed compiler scopes, before activation | 3.3b transfer/Burn; 3.5b compact/Clear and announcement/Clear; 3.6b full three-operation scope. Allow exact partner-exit status changes and literal shared reasons, not arbitrary drift |
| 2.7a B3 local closure | 3.1a compiler required choice cannot come from company; 3.6b realization local lifecycle omission still refuses |
| 2.7a B4 and 2.8 B4 | 3.1b shared-bound incoming-edge/seed guard, seed-free control; `plans/backlog.md:740–741` |
| Provisional relation_objects | 3.2b Burn and 3.2c Clear before activation; operation-specific DataOutputs and exact two-object activations/source evidence |
| Joint role results and premises | 3.6 preserves operation/event/credit/silence/accounting/STATE distinctions, original model provenance and realization-only synthetic catalogs |
| Ruling 217 | Authorization's UnexpectedProtocolAuthorization, Constructibility blocked; 3.3/3.5 validator subjects and 3.8 handoff to Waves 5/7/8/11 |
| Rulings 133/138 | 3.7 five individually disposed subjects with actual control/refusal, leaf, successor witness, compact tree/archive, STATE digest and realization transition support |
| Later construction | 3.8 retains carrier policy, linked equality, native successor, signatures/public recovery and report duties for the actual later owners |
| 2.7a B5/B6; longer histories, random order/subsets, tracing/scaling | Deferred follow-ups, not landing claims; missing product behavior still requires disposition |
| 2.9 B1 per-key literal routes | Deferred; manifest/ledger/corpus remain distinct evidence. 2.9 B2 support selection is taken at 3.7 |
| Frozen relation comment | Leave realization relation.rs unchanged; editorial update waits for authorized re-freeze |
| Other 2.5/2.6/2.7/2.8 optional adapter/law permutations | Retain (`tab:backlog:wave2-design-followups`) statuses; no raw model transition or adapter edit selected |
| Review follow-ups | F1 broad published-lifecycle property test deferred beyond exact partition test; F2 extended five-operation cost study deferred, three-operation counts selected at 3.6; F3 randomized scope permutations deferred; F4 owner/engine qualification comparison answered in route Q1 |

This table disposes every (`rec:backlog:wave2-wave3-receiver`) row and the addressed deferred choices. The full inherited consumer categories have dispositions at (`sec:w3route:consumers`), not an implied licence to edit every census entry.

## 2. Gates, estimates and whole-wave cost · `sec:wave3:verification`

**Definition (Exact packet)** · `def:wave3:packet`

Each packet names its goal, exact paths, focused tests, gate, output and non-claims. A needed unlisted path, additional mechanism, changed semantic owner or unreviewed interface stops that packet for a revised decision. Each implementation packet receives independent review and a full import gate before dependent advancement. New code uses existing structured tracing owners (`[ADR026-dec:tracing:core]`). Tests named below are proposed allocations unless described as existing; no name embeds wave/bite chronology.

D is the existing plans/labels/census/generated/document surface; documents-only packets need no Rust lane. C is formatting in place, workspace Clippy with warnings denied, selected package tests under the gate profile, and existing compatibility tests for changed shared vocabularies. The full import gate remains `scripts/ci.sh` through canonical Meson; C/D alone is not that full result. Use one toolchain/target directory, record cold build separately, and format last. No compiler or gate is run locally by a boxed writer; its authorized build lane supplies these results (`AGENTS.md`), (`gate:plans:verification`).

Wave-2 full gates took 950–4,265 s (`plans/backlog.md:683–695`); the frozen gate took 1,198 s (`rec:history:wave2-frozen-gate`) and closure-import 3,218 s (`rec:history:wave2-closure-import`), an increase whose cause is not established. Budget about one hour ordinarily and reserve the observed upper wall plus queue/review overhead; never treat a one-hour estimate as proof a 4,265-second gate will fit. Scope/oracle wall estimates below are unmeasured and must be replaced by actual counts and wall.

Precedent at the reading base: announcement facade 2,125 lines, plan tests 542, corruption tests 1,366, oracle 893; live facade 2,430 and tests 1,135; compact facade 1,263, plan tests 880 and validation tests 651 (the corresponding files under `packages/compiler/src/` and `src/tests/`). The facade/validator halves therefore budget thousands of lines, with independent oracle/activation halves. Dormant lowering is cut by operation; mixed-change work is four packets, not hidden in a small Burn estimate.

**Table (Ordered packet sizes and focused costs)** · `tab:wave3:bites`

| Packet | Goal | Packages | Pinned source | Estimated changed size | Focused gate / wall after build |
|---|---|---|---|---|---|
| 3.0 | Adopt readings, interfaces and exact settings | planning | No selected edit | 8–14 KiB documents | D; 1–5 min warm estimate |
| 3.1a | Operation-local choices and qualified exits | compiler, realization tests, tapscript, vectors | No selected edit | 500–900 lines | C: compiler, realization, tapscript, vectors; 1–3 min estimate |
| 3.1b | Shared-bound guard | realization | No selected edit | 200–400 lines | C: realization; under 2 min estimate |
| 3.2a | Owned vocabulary and public re-exports | compiler, vectors | No selected edit | 400–800 lines | C: compiler, vectors; 1–3 min estimate |
| 3.5c | Qualified selection keys and side cases | compiler, tapscript | No selected edit | 800–1,400 lines | C: compiler, tapscript plus all keyed compatibility tests; 2–6 min estimate |
| 3.2b | Dormant Burn lowering and guard relocation | compiler | No selected edit | 900–1,600 lines | C: compiler; 2–5 min estimate |
| 3.4 | Compact revalidation before first activation | compiler, linker tests, vectors | No selected edit | 150–350 lines | C: compiler, linker, vectors; 2–5 min estimate |
| 3.3a | Burn explicit facade, validator and corruptions | compiler, documentation | No selected edit | 2,600–4,200 lines | C: compiler; 2–6 min estimate |
| 3.3b | Burn oracle, mixed live scope and explicit activation | compiler, realization documentation | No selected edit | 900–1,500 lines | C: compiler; oracle target under 2 min, suite 2–6 min estimate |
| 3.2c | Dormant Clear lowering | compiler | No selected edit | 1,200–2,000 lines | C: compiler; 2–6 min estimate |
| 3.5a | Clear facade, validator and corruptions | compiler, documentation | No selected edit | 3,000–4,800 lines | C: compiler; 2–6 min estimate |
| 3.5b | Clear oracle, mixed scopes and activation | compiler, realization documentation | No selected edit | 1,000–1,700 lines | C: compiler; oracle target under 2 min, suite 2–6 min estimate |
| 3.5d | Mixed conservation declaration and lowering | compiler, realization, planning contracts | No selected edit | 1,000–1,800 lines | C: compiler, realization; 2–6 min estimate |
| 3.5e | Mixed Burn facade and fresh validator | compiler | No selected edit | 1,500–2,700 lines | C: compiler; 2–6 min estimate |
| 3.5f | Mixed Burn oracle and alternative activation | compiler | No selected edit | 800–1,400 lines | C: compiler; oracle target under 2 min, suite 2–6 min estimate |
| 3.6a | One-bound Phase-7 facade and typed ASH closure | compiler | No selected edit | 1,500–2,500 lines | C: compiler; focused wall unmeasured, 10 min diagnostic budget |
| 3.6b | Scope oracle and complete publication | compiler, realization tests | No selected edit | 600–1,100 lines | C: compiler, realization; wall unmeasured, 10 min diagnostic budget |
| 3.7 | Candidate-bound reopenings and support pins | vectors, compiler/tapscript tests; read-only realization and target-elements-conformance | No selected edit | 100–300 lines plus 3 KiB receipt | C: vectors, compiler, tapscript, realization, tripod-target-elements-conformance; 2–10 min estimate |
| 3.8 | Frozen Wave-3 gate and Wave-4 handoff | planning | No selected edit | 4–8 KiB receipt/live text | D plus own full import gate; 950–4,265s historical gate range |

The nineteen packets include seventeen implementation/test packets and two documents packets. Planning for one independent review and one full import gate per packet means nineteen review slots and 18,050–81,035 gate seconds (5.01–22.51 hours) before queueing, implementation, focused checks or rework. The seventeen code packets alone account for 16,150–72,505 seconds (4.49–20.14 hours). Four private-change packets contribute 3,800–17,060 seconds (1.06–4.74 hours) of that total. This is a workload estimate from the measured range, not a timing acceptance threshold. A bounced review/gate adds another measured round.

Review the upper size estimate before starting each packet; split before work if its actual design exceeds that bound. There is no blanket 1,400-line ceiling that will predictably fire midway through a 4,000-line facade. Public activation remains atomic with the complete oracle and mixed-scope controls. New small oracles target under two minutes per module with at most 4,096 products per tiny instance; focused suites use a ten-minute diagnostic budget unless their brief names a justified larger one. A timeout is unfinished evidence. The full three-operation analysis has no measured two-to-five-minute promise: record global proof-plan count, each operation's placement-factor counts and wall, and retain factorization rather than materializing their Cartesian product (`plans/guides/guide_seven.md:56`).

## 3. Ordered packets · `sec:wave3:execution`

### 3.0 — Adopt readings, interfaces and exact settings · `step:wave3:bite-3-0`

Goal and design: Adopt or revise proposed rulings 221–234. Mint a Wave-3 settings rule beside the execution section; the Wave-2 rule is precedent only. Register plans/history/wave3-compiler-records.md and its index/census, record divergences in the drafts README, and keep the proposal bodies historical after adoption. Confirm the side-key extension, factorable mixed tuple, exact operand/evidence ledger and current full freeze inventories before code. Two explicit pre-code checks are mandatory: the proposed Debug form must preserve all five engine-base captures without manifest changes; and the Ash PublicCommitted incompatibility plus admitted-tuple Committed visibility must be expressible in the named owners. Record these decisions before implementation; failure returns the concrete missing mechanism and revised packet, never a silent re-baseline or explicit-only closure.

Focused selection: Select D checks for labels/census/weight/current owners and an independent reading of all twelve questions and receiver rows. No Rust test.

Files: exactly the 3.0 expansion at (`tab:wave3:paths`). Gate: D; 1–5 min warm estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 8–14 KiB documents.

Handoff and limits: Hands on accepted meanings, exact paths and focused subjects. No implementation or adopted position follows from this draft alone.

### 3.1a — Operation-local choices and qualified exits · `step:wave3:bite-3-1a`

Goal and design: Make required-choice membership (operation, object)-local using lifecycle-lowering/engine records; reserve the actual input-side Burn choice lowering for 3.2b after 3.5c gives it a qualified key. Add the literal input-admission eligibility table, unavailable-in-scope status and all-nonavailable outstanding filters. Keep graph reachability separate; initialize empty records without editing keyed producers.

Focused selection: Select burn_choice_cannot_be_supplied_by_transfer (Burn-owned synthetic lifecycle-lowering/engine records; remove its choice while TransferLive remains; complete analysis waits for 3.2b), exit_eligibility_is_representation_specific (literal input-admission modes and changed eligibility), and unavailable_exit_survives_internal_projection (synthetic engine/analyzed rows, with a status corruption at the validator). Pair private_input_is_not_admitted_by_the_input_change_mode_union in realization burn_tests. No claim that graph-only validate_compact_lifecycle_projection validates statuses.

Files: exactly the 3.1a expansion at (`tab:wave3:paths`). Gate: C: compiler, realization, tapscript, vectors; 1–3 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 500–900 lines.

Handoff and limits: Hands on correct internal choices/statuses; public private-row survival waits for 3.3b and complete unselected-row publication for 3.6. No emitted/deployed usability claim.

### 3.1b — Shared-bound guard · `step:wave3:bite-3-1b`

Goal and design: Apply the proposed narrow shared-bound admission rule in the existing assembler/declassification owner. Preserve equal-visibility union and singleton semantics. Add UnsupportedSharedBoundDependency with bound, declaring operation and incoming-edge/seed kind and its diagnostic/tracing cases.

Focused selection: Select shared_bounds_without_incoming_dependencies_merge, incoming_dependency_on_shared_bound_is_refused, and seed_on_shared_bound_is_refused in derivation/disclosure tests, sharing one accepted seed-free control.

Files: exactly the 3.1b expansion at (`tab:wave3:paths`). Gate: C: realization; under 2 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 200–400 lines.

Handoff and limits: Hands on the guarded union for later mixed scopes. Its gate does not claim mixed facade compatibility or full singleton goldens.

### 3.2a — Owned vocabulary and public re-exports · `step:wave3:bite-3-2a`

Goal and design: Add only genuinely missing primitive/event/coverage coordinates, with append-only attestation encodings and disjoint Fact subject domain. Re-export carried public coordinate enums from operation_plan.rs. Retain public operation/body guards and existing source/layout kinds. Update the actual vectors exhaustive and wildcard interpretations without attaching old evidence to new coordinates. The four tapscript paths formerly here are removed because this vocabulary does not reach them.

Focused selection: Select operand_codes_are_injective_and_preserve_pilots, new_coverage_coordinates_keep_their_boundary, and existing public surface/error-name tests. Reuse ObjectFamilyAmount/Count and RuntimeBound instead of inventing duplicate Burn/ASH amount operands.

Files: exactly the 3.2a expansion at (`tab:wave3:paths`). Gate: C: compiler, vectors; 1–3 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 400–800 lines.

Handoff and limits: Hands on typed vocabulary only, not lowered plans or target observations.

### 3.5c — Qualified selection keys and side cases · `step:wave3:bite-3-5c`

Goal and design: Add optional side qualification through choice IDs, core keys/records, case side-map, source activations and side-layout/case-fact projections. Empty/default qualifiers preserve old identities; both keyed owners still publish empty choices. Preserve the engine-base capture as well as key encodings: hand-written Debug for RepresentationChoiceId and ExecutionCaseId omits an absent qualifier or empty side map, preserving the existing derived field order, names and pretty formatting; present qualifiers remain visible. The unchanged preserve_engine_base test must reproduce all five byte lengths and SHA-256 digests in packages/compiler/tests/fixtures/engine_base/MANIFEST (`packages/compiler/src/tests/engine_base_capture.rs:155–229`). Re-baselining is rejected because it would discard the evidence that empty qualifiers preserve old identities. The capture test and MANIFEST remain read-only. Side qualification enters key types (including engine_proof::ChoiceKey, lifecycle ChoiceRecord and ExecutionCaseId) and new activation, structural/compiler requirement and layout enum variants only. Add no fields to the pinned producers' literal-built engine_cases::Case, engine_cases::Tables, engine_case_facts::Facts, engine_proof::Records or engine_program::Records (`packages/compiler/src/engine_keyed.rs:440–452,791–795,842–860`), (`packages/compiler/src/engine_keyed_fixed.rs:480–492,941–945`), (`packages/compiler/src/engine_keyed/program.rs:85`), (`packages/compiler/src/engine_keyed_fixed/program.rs:89`). No pinned producer constructs ChoiceKey or ChoiceRecord. A required new field in those records stops for a separately reviewed re-freeze packet. This packet follows 3.2a directly, before any Burn/Clear facade exists, and before actual Burn choice lowering. This is the only selected shared selection-schema extension; no new search algorithm or pinned producer edit. All literal constructors and actual new layout consumers are in the exact packet.

Focused selection: Select side_choices_do_not_alias_object_choices, unqualified_case_keys_preserve_pilot_identity, and input_choice_cannot_be_repaired_by_output_choice. Existing keyed refusal/negative constructors explicitly use the unqualified form. Re-run the unchanged preserve_engine_base, all compact baseline pins and old live/announcement projections.

Files: exactly the 3.5c expansion at (`tab:wave3:paths`). Gate: C: compiler, tapscript plus all keyed compatibility tests; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 800–1,400 lines.

Handoff and limits: Hands on expressive side-qualified tables while private Burn selection remains refused. If a pinned producer or additional kernel constraint is required, stop and revise before continuing.

### 3.2b — Dormant Burn lowering and guard relocation · `step:wave3:bite-3-2b`

Goal and design: Interpret Burn SideRepresentation, BurnBinding and Burn DataOutputs using the complete literal source/proof/object/boundary table. Burn's actual input choice is born side-qualified using 3.5c. A relation owns only the lifecycle rows of its own operation's selected mode for that object — the output-side choice where the object is side-qualified; plan-level completeness may keep the candidate union. Change engine_requirement::owned_lifecycle and both attestation call paths to pass the owning operation and selected qualified choices, rather than filtering only (object, exit). Candidate filtering alone retains modes chosen by any operation and cannot establish this ownership (`packages/compiler/src/engine_proof.rs:279–289`), (`packages/compiler/src/engine_requirement.rs:32–47`), (`packages/compiler/src/engine_attestation/requirement.rs:115,472–482`). Remove per-stage operation checks and retain them at input.rs and engine_attestation.rs binding seams. Enable these body interpreters, including operation-aware DataOutputs dispatch, while public Burn/Clear stay refused. Convert the old synthetic direct-case/source refusal subjects to actual internal-lowering controls and record their replacement at backlog evidence rows. Do not add a test-only Boolean bypass.

Focused selection: Select burn_bodies_have_exact_owned_tables, side_admission_and_sponsor_opacity, active_carriers_and_layout_join, and live_exit_owns_only_its_selected_mode: in TransferLive+Burn, the live private Burn exit owns no Explicit row supplied by Burn's own input choice. Complete validated declaration records reach crate-private stage functions; the control must reach carrier/layout analysis rather than return UnsupportedOperationPlan. One changed carrier proves the actual placement boundary.

Files: exactly the 3.2b expansion at (`tab:wave3:paths`). Gate: C: compiler; 2–5 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 900–1,600 lines.

Handoff and limits: Hands on independently reviewed dormant Burn tables. Clear-specific bodies remain refused and no public Burn constructor succeeds yet.

### 3.4 — Compact revalidation before first activation · `step:wave3:bite-3-4`

Goal and design: Immediately revalidate compact against the modified shared owners, preserving its standalone figures and existing API. Explicitly keep event silence. This order has no dependency on Burn activation.

Focused selection: Select existing compact plan-census tests, the independent placement/coverage/assembly oracles, linker carrier pin and vectors every_coverage_requirement_has_exactly_one_row. Add compact_event_silence_is_retained only if the selected existing corruption does not already prove that coordinate.

Files: exactly the 3.4 expansion at (`tab:wave3:paths`). Gate: C: compiler, linker, vectors; 2–5 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 150–350 lines.

Handoff and limits: Hands on exact compact baseline. Subsequent Clear and side-key changes rerun these pins; a wider-scope lifecycle result is a new row, not a replacement baseline.

### 3.3a — Burn explicit facade, validator and corruptions · `step:wave3:bite-3-3a`

Goal and design: Build the private-field Burn facade and fresh validator from dormant complete analysis, with the 32-relation source census and explicit selection. Public binding still refuses Burn. Crate-private tests derive the facade from complete owner records and freshly validated analysis; they do not manufacture a public unchecked constructor. Lowering owners are writable correction permissions. Prepare scope-derived live-transfer lifecycle summaries and per-representation equivalence normalization without publishing Burn activation.

Focused selection: Select burn_plan_control_and_corruptions for the Burn rows of the corruption table, burn_selection_refuses_private_change at the offered-selection validator returning TargetPlanRepresentationMismatch, and the public API privacy guard. Test omitted lifecycle status at the plan validator.

Files: exactly the 3.3a expansion at (`tab:wave3:paths`). Gate: C: compiler; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 2,600–4,200 lines.

Handoff and limits: Hands on an internally validated facade. The private change refusal is an intermediate documented limit; it cannot be the wave's final state.

### 3.3b — Burn oracle, mixed live scope and explicit activation · `step:wave3:bite-3-3b`

Goal and design: Add the independent Burn oracle, then remove only Burn's public operation guard. Body guards no longer reject its three implemented bodies; Clear remains blocked by its operation guard. Convert declared_burn_reaches_unsupported_plan_at_input_binding to public accepted/invalid controls, preserving the old claim in the receipt/backlog. Update realization README's unsupported statement. Apply the scope-derived live facade contract now, before activation is credited.

Focused selection: Select burn_plan_matches_small_oracle and transfer_burn_scope_preserves_representation_specific_exits: in one TransferLive+Burn bound scope Explicit implements Burn, PrivateCommitted retains unavailable Burn, Redeem stays outstanding, and only literal shared-bound reasons differ. The public API test checks both representation projections and notes that complete Phase-7 publication of the unselected row is supplied at 3.6b. Keep incomplete-realization-scope refusal distinct.

Files: exactly the 3.3b expansion at (`tab:wave3:paths`). Gate: C: compiler; oracle target under 2 min, suite 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 900–1,500 lines.

Handoff and limits: Hands on a public explicit Burn plan with its oracle and correct private live exit; no mixed Burn alternative yet, and no scope-only direct-private discharge.

### 3.2c — Dormant Clear lowering · `step:wave3:bite-3-2c`

Goal and design: Complete Clear-specific bodies and Clear's quantity-dependent DataOutputs, STATE sources, six assignments, clamp/floor/destruction/residual and report splits. Keep the Clear operation check at both binding seams; stage functions already permit internal complete records. Accept None for the two-object single-selection summary only with exact source/activation/dependency tables. Preserve Burn and compact lowerers.

Focused selection: Select clear_bodies_have_exact_owned_tables, clear_data_quantity_keeps_clamp_dependency, and clear_active_carriers_and_layout_join. The DataOutputs expected object set is {State,Ash}; the independent source table and missing-edge subject must show that public quantities are retained.

Files: exactly the 3.2c expansion at (`tab:wave3:paths`). Gate: C: compiler; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,200–2,000 lines.

Handoff and limits: Hands on complete dormant Clear lowering, not public plan evidence. Both later Clear packets own these lowerers for corrections.

### 3.5a — Clear facade, validator and corruptions · `step:wave3:bite-3-5a`

Goal and design: Build the private-field Clear facade, exact 44-relation and six-field requirement projections, fresh validation and focused corruption table while public binding remains refused. Prepare scope-derived announcement and compact lifecycle behavior. Lowering correction permissions include all carrier/layout/coverage/case owners.

Focused selection: Select clear_plan_control_and_corruptions, clear_sponsor_amount_is_refused (SponsorValueRead at its source boundary), and the API privacy guard. Full/partial/refused-clamp are literal conditional requirement classes. ClearFloor is a plan requirement corruption, not an unreachable complete-operation negative. Preserve Authorization failure and blocked Constructibility for signer insertion.

Files: exactly the 3.5a expansion at (`tab:wave3:paths`). Gate: C: compiler; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 3,000–4,800 lines.

Handoff and limits: Hands on internally validated Clear; no announcement successor execution or linked constructor claim.

### 3.5b — Clear oracle, mixed scopes and activation · `step:wave3:bite-3-5b`

Goal and design: Land the independent Clear oracle and the two required mixed pairs before removing Clear's public guard. Convert declared_clear_reaches_unsupported_plan_at_input_binding and update realization/compiler READMEs. Retain the two Unsupported errors as reserved future boundaries, with honest current tests.

Focused selection: Select clear_plan_matches_small_oracle and clear_scopes_preserve_local_requirements: CompactAsh+Clear and AnnounceMaturity+Clear compare standalone projections except exact partner-exit changes and shared-bound reason unions. Explicitly pin compact 1→2 implemented/1→0 outstanding/scoped completeness and announcement 1→2 implemented/5→4 outstanding; retain old standalone expectations. Include Clear sponsor refusal from 3.5a in the activation selection.

Files: exactly the 3.5b expansion at (`tab:wave3:paths`). Gate: C: compiler; oracle target under 2 min, suite 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,000–1,700 lines.

Handoff and limits: Hands on all three explicit facades and their own oracles; private-change delivery remains a required predecessor of scope closure.

### 3.5d — Mixed conservation declaration and lowering · `step:wave3:bite-3-5d`

Goal and design: Add existing ConfidentialConservation as a Burn alternative in declaration/validation, keeping PublicArithmetic for readable cases. Lower the exact E-input/E-ASH/PrivC-change tuple and external commitment-conservation evidence, with public recognition/partition and owner-local change data. Burn's Ash domain remains {Explicit, PublicCommitted}, not fixed Explicit (`packages/realization/src/declarations/burn.rs:303–313`). PublicArithmetic excludes (Burn output ReceiptLive, PrivateCommitted); ConfidentialConservation excludes both (Burn output ReceiptLive, Explicit) and (Burn Ash, PublicCommitted). With normalized Explicit input these unary incompatibilities admit only E-input/E-Ash/PrivC-change for the confidential alternative. For that tuple alone, conserved_amount_visibility returns Committed and discharge is target-owned ConfidentialValueConservation; explicit amounts remain public operands of the other relations. Every other mixed case retains MixedRepresentationConservation. Reading only the object map would falsely classify this case as Readable; reading sides without this exception would refuse it (`packages/compiler/src/case.rs:124–151`), (`packages/compiler/src/placement.rs:303,381,565`), (`packages/compiler/src/coverage.rs:577,690`), (`packages/compiler/src/engine_attestation/coverage.rs:29`). The source table must not disclose C. Review/adopt the specific mixed-route admission in the existing contract; no global PublicArithmetic or proof compatibility weakening. Public mixed selection still refuses.

Focused selection: Select burn_mixed_tuple_selects_commitment_evidence, private_change_never_becomes_public_arithmetic, and burn_proof_alternatives_match_the_reviewed_census. Controls retain explicit Burn and reject private input, swapped sides, missing commitment evidence and cross-product mismatches through the existing finite constraint mechanism. Select admitted_burn_tuple_has_committed_visibility with the admitted tuple as control and one changed Ash mode as the refused subject; also retain the explicit Readable control and assert target-owned ConfidentialValueConservation discharge for the admitted tuple.

Files: exactly the 3.5d expansion at (`tab:wave3:paths`). Gate: C: compiler, realization; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,000–1,800 lines.

Handoff and limits: Hands on reviewed mixed law/proof sources and capability obligations, not target verification. If unary incompatibilities cannot enforce the exact tuple, return the concrete combination-constraint gap and revised path/freeze cost.

### 3.5e — Mixed Burn facade and fresh validator · `step:wave3:bite-3-5e`

Goal and design: Extend the Burn facade with a separately validated private-change alternative, preserving explicit projections and distinct side/source/evidence rows. Include the new corruption module and exact registration. Constructor and offered-selection validator remain private/dormant for this alternative until its oracle lands; the explicit public route remains available.

Focused selection: Select mixed_burn_plan_preserves_public_ash_and_private_change, mixed_burn_plan_rejects_lost_commitment_evidence, and mixed_burn_plan_rejects_input_output_aliasing. Validate complete tuple/source/layout/coverage/lifecycle joins and no partial plan; the private receipt-to-Burn exit stays unavailable independently of private change output support.

Files: exactly the 3.5e expansion at (`tab:wave3:paths`). Gate: C: compiler; 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,500–2,700 lines.

Handoff and limits: Hands on validated internal mixed alternative. Do not label external commitment requirements as completed target proofs or emit bytes.

### 3.5f — Mixed Burn oracle and alternative activation · `step:wave3:bite-3-5f`

Goal and design: Add independent literal side/evidence and small placement oracle, then admit the mixed offered-selection request. Review exact conditional change/no-change behavior and sponsor isolation; fresh public ASH constructor requirements remain shared with the explicit alternative. Forward both plan alternatives to Wave 4.

Focused selection: Select mixed_burn_plan_matches_small_oracle, normalized_input_with_private_change_is_admitted, and private_input_remains_refused_after_private_change_activation. Re-run explicit Burn/compact/live/announcement regressions and the private lifecycle reaching subject.

Files: exactly the 3.5f expansion at (`tab:wave3:paths`). Gate: C: compiler; oracle target under 2 min, suite 2–6 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 800–1,400 lines.

Handoff and limits: Hands on the final Wave-3 Burn domain, including accepted private change but no direct-private input. Wave 4 still assesses complete confidential-change proofs/weight/fees/policy.

### 3.6a — One-bound Phase-7 facade and typed ASH closure · `step:wave3:bite-3-6a`

Goal and design: Build scope from one BoundCompilerInput containing Burn, CompactAsh and Clear. Freshly derive all plans and six producer/recognizer comparisons. Publish full declared-representation lifecycle obligations not carried by selected facades, derived from lifecycle analysis rather than facade union; validate exact partition with carried obligations. Include private ReceiptLive→Burn even when every selected Burn input is explicit.

Focused selection: Select scope_rejects_operation_and_constructor_corruptions and unselected_representation_obligations_remain_published. A lost private row must fail the scope validator. Retain separate event/record/accounting/model/synthetic provenance.

Files: exactly the 3.6a expansion at (`tab:wave3:paths`). Gate: C: compiler; focused wall unmeasured, 10 min diagnostic budget; D for changed records, independent review and full repository import gate before advancement. Estimated size: 1,500–2,500 lines.

Handoff and limits: Hands on privately validated scope facade and closure comparison. No linked-byte equality, combined model transition or deployed completeness claim.

### 3.6b — Scope oracle and complete publication · `step:wave3:bite-3-6b`

Goal and design: Land independent full-scope expected relation/source/lifecycle/constructor joins and publish the validated scope. The named pair tests already landed with activation; this bite adds the full three-operation scope. Record proof-plan counts, per-operation placement-factor counts and wall; avoid a product of independent factors.

Focused selection: Select scope_matches_independent_rows and declaration_company_cannot_repair_local_lifecycle_omission. Check the exact carried/unselected obligation partition including the private route and six producer/recognizer pairs. A broad randomized property test is a follow-up, not silently included.

Files: exactly the 3.6b expansion at (`tab:wave3:paths`). Gate: C: compiler, realization; wall unmeasured, 10 min diagnostic budget; D for changed records, independent review and full repository import gate before advancement. Estimated size: 600–1,100 lines.

Handoff and limits: Hands on complete compiler scope evidence for candidate reopening. Timing beyond the finite budget is unfinished evidence and requires a reviewed domain/packet adjustment.

### 3.7 — Candidate-bound reopenings and support pins · `step:wave3:bite-3-7`

Goal and design: Re-dispose both compact emitted-Clear keys and all three ruling-133 positions, binding leaf, witness, tree/archive and STATE pins. Changed compiler support is not a new emitted path. All candidate source pins and fixtures remain read-only; correction of an actual emitted validator requires revised scope.

Focused selection: Select compact_clear_declaration_reopenings_keep_both_keys; candidate_leaf_and_transition_censuses_have_no_announced_state_spend; no_announcement_byte_carries_the_successor_recipe; the_fixture_pin_is_the_output_key_this_tree_derives in target-elements-conformance; current_archive_replays_sixteen_rows_after_their_paired_controls; every_staged_subject_matches_its_pin; exact compact negative-manifest/corpus join; and the two realization announce_maturity transition refusals named in the reopening row.

Files: exactly the 3.7 expansion at (`tab:wave3:paths`). Gate: C: vectors, compiler, tapscript, realization, tripod-target-elements-conformance; 2–10 min estimate; D for changed records, independent review and full repository import gate before advancement. Estimated size: 100–300 lines plus 3 KiB receipt.

Handoff and limits: Hands on five separate dispositions, actual boundaries and supporting unchanged candidate evidence. No native successor or emitted Clear credit.

### 3.8 — Frozen Wave-3 gate and Wave-4 handoff · `step:wave3:bite-3-8`

Goal and design: Join all preceding receipts on one candidate, including this closing record's own full gate. Recount the Wave-0 baseline alongside current figures, complete compact pins, all five reopenings, mixed scope and both Burn alternatives. Keep detailed evidence in the history owner and compact current obligations in live owners.

Focused selection: Use the closing checklist below. No code fix is hidden here: reopen its producing packet. Record actual test/gate exits, wall/cold-cache/cgroup scope and real-document/reproducibility status rather than promote a mocked gate.

Files: exactly the 3.8 expansion at (`tab:wave3:paths`). Gate: D plus own full import gate; 950–4,265s historical gate range; D for changed records, independent review and full repository import gate before advancement. Estimated size: 4–8 KiB receipt/live text.

Handoff and limits: Wave 4 receives both Burn alternatives, compact/Clear/scope plans, five censuses, ASH contracts, external evidence and carrier reopening duties. Signature/native recovery/linked equality and report production remain later-wave obligations.

## 4. Focused corruption coverage · `tab:wave3:corruptions`

These are the twenty required coordinates from (`rule:guide15:compiler-validation`), grouped into operation/scope tests, not twenty new harnesses. Each compares an accepted control with one changed coordinate at the owning boundary. A realization weld refusal and a crate-private offered-plan corruption are different subjects; record which one was actually reached.

| Coordinate | Packet and required result |
|---|---|
| Remove one operation; merge Burn/compact identities | 3.6a/b scope census/identity refusal, no scope value |
| Remove owner authorization | 3.3a missing authorization/source requirement |
| Mark Clear operator-authorized | 3.5a authorization/evidence mismatch |
| Remove public ASH availability | 3.3a/3.5a/3.6a source/public-dependency mismatch |
| Change tag-recon; remove Y−1; remove residual condition | 3.5a exact Clear law/conditional-row mismatches |
| Insert RESV | 3.5a forbidden root requirement |
| Make compact emit Burn event | 3.4 event-silence/projection refusal |
| Make Clear emit Burn record | 3.5a wrong data/event requirement |
| Remove event-value anchor; remove event-type anchor | 3.3a distinct binding/report rows, not emitted credit |
| Introduce sponsor amount | 3.2b/3.3a Burn and 3.5a/b Clear each reach SponsorValueRead; compact existing public_api.rs:587 subject retained |
| Remove lifecycle exit | 3.1a/3.3a/3.6a exact path/status/census refusal; no smaller-census success |
| Assign relation to inactive carrier | 3.2b/3.2c/3.3a placement eligibility refusal |
| Introduce target type; expose graph handle | Explicitly adopted public API-shape/privacy interpretation at each facade, not fake runtime cases |
| Duplicate relation | 3.3a multiplicity/census refusal, not deduplication |
| Change evidence boundary | 3.3a/3.5a exact role/boundary refusal, reports never counted emitted |

The additional selected receiver subjects are operation-local choice, published/unselected private obligations, partner-exit status changes, shared-bound seed/edge and mixed input/output aliasing. Randomized scopes, every field permutation, full-field singleton goldens, long histories and new native subjects remain follow-ups.

## 5. Baselines, scope figures and closure · `tab:wave3:baseline`

| Standalone compact figure | Correct pin |
|---|---|
| 23 relations / 2 cases / 46 relation-case rows | `packages/compiler/src/tests/operation_plan_tests.rs:144,209,878` |
| 216 placements | `packages/compiler/src/tests/pilot_program_tests.rs:767`; also analyzed_program_tests.rs:738 |
| 63 layout requirements | `packages/compiler/src/tests/operation_plan_tests.rs:274` |
| Coverage graph 338 nodes / 355 edges | `packages/compiler/src/tests/pilot_program_tests.rs:770–771` |
| 28 runtime carrier relation-case keys | `packages/linker/src/tests/carrier_tests.rs:32–33,48` |
| 209 coverage requirements = 135 positive + 74 negative | `packages/vectors/src/plan.rs:1521–1523` |

No packet intentionally moves those standalone counts or old encoded case identities. The side qualifier is absent for old pilots and its encoder retains old bytes. A proposed change names affected rows, reason and downstream evidence before its producing packet; updating a golden after a surprise is not disposition (`obl:guide15-exec:baseline-comparison`).

Scope-derived lifecycle is an intentional interface change, with exact expected figures at (`proposal:w3route:scope`): live Explicit in TransferLive+Burn implemented 1→2/outstanding 2→1, private live unchanged in cardinality but Burn outside→unavailable; common summary intersection/union remains 1/2. Announcement+Clear implemented 1→2, outstanding 5→4. Compact+Clear implemented 1→2, outstanding 1→0 and scoped completeness false→true. Standalone tests retain old values. Update live plan tests at :278 and lifecycle corruption/equivalence checks, announcement plan tests :484–488 and corruption/equivalence checks, compact plan test :472 and status corruptions. These are semantic status changes on wider scopes, not a new emitted leaf or candidate-byte/freeze update. Mixed-change plan counts are new and must be enumerated, never inferred by adding 32 relations twice.

At closure also recount beside Wave 0: Safety 206/177/29; observed/final/retired/waiting 177/27/2/0; classes 81/12/39/15/16/14; relation links 14/108/84; reasons 55/9/9/0/11; conjunct parts 10 MET/9 RETIRED/0 WAITING. These are retained comparison figures from `plans/backlog.md:779`, not predicted new counts. Publish actual current identities/counts and explain every movement or unchanged result; preserve T11-064/T11-063/T4-009/T11-110 on their own criteria. Compact-only figures do not satisfy this obligation (`rule:guide15-exec:later-wave-entry`), (`obl:guide15-exec:baseline-comparison`).

**Gate (Closing checklist)** · `gate:wave3:closing`

“All three validated plans reconcile relation/carrier/layout/coverage/lifecycle censuses, reject focused corruptions, and agree with independent small-instance oracles.” (`gate:guide15-exec:wave3-gate`).

| Conjunct / accompanying obligation | Receipt evidence |
|---|---|
| Three validated plans | Burn explicit and private-change alternatives, existing compact API revalidated, Clear complete; private fields, fresh validators, no partial values |
| Relation census | Exact identities/bodies, multiplicity, active/inactive cases, selected domains and two-object representation/source meaning |
| Carrier census | Every active runtime obligation has reachable eligible carrier, every-member/local/coordinator roles distinct; reports never emitted |
| Layout census | Exact side/source/authentication/coordinator obligations and joins, no lost private commitment source |
| Coverage census | Positive/negative/structural/report requirements, dependency collateral and actual refusal ownership |
| Lifecycle census | Input-local choices, qualified status, declared graph retained, carried/unselected partition complete; private Burn exit published outstanding |
| Focused corruptions | Every required coordinate plus selected receivers and accepted controls, actual typed/static boundary |
| Independent oracles | Common Guide-7 rule; domains/products/branch classes; expected-side independence and actual counts/wall |
| Cross-operation/ASH closure | One bound input, all three facades, six typed constructor comparisons and exact permitted partner-status/reason changes |
| Baseline and candidate obligations | Full Wave-0 recount above, compact pins, separate new plan counts and five ruling-133/138 dispositions with support pins |
| Reviewed candidate and gates | Every packet review/full gate, source/records identity, final records' own gate, wall/cgroup/cache scope, skipped real-document/reproducibility dimensions stated |
| Wave-4 handoff | Both Burn alternatives and confidential-change capability/evidence duties; typed carrier/schema assumptions and affected Wave-2/3 rows for any reopening |

Wave 4 assesses complete transaction forms and confidential change proofs/weight/fees/policy; on refusal its ruling names the declarations and plans reopened. Later waves retain linked ASH equality, STATE generation, signatures/public recovery and report production. Forward Authorization/blocked Constructibility and the outstanding direct-private receipt route unchanged (`rec:backlog:wave2-wave3-receiver`), (`cond:guide15-exec:carrier-reopening`). Closure does not close Phase 7.

## 6. Exact writable-path census · `sec:wave3:paths`

**Table (Literal path sets)** · `tab:wave3:paths`

Each prefix and basename expands to one exact path; no glob or “nearest registration” is implicit. A listed path is permission, not a mandate to edit it. All facade/validator and oracle/activation packets include their lowering correction owners, including engine_requirement (operation-local lifecycle ownership), carrier, placement, layout, coverage/coverage_graph, sponsor_region and attestation placement/operation/coverage/coverage_join/case_facts/cases/constructibility. Module/test/Meson registration is explicit. Read-only regression packages do not acquire write permission.

There are 140 distinct paths: 128 existing and 12 allocated new, mechanically checked against this tree. Intersections with all 98 fixed-form and all 35 legacy inventory paths are both empty. The 758 packet-path occurrences expand to that union. The earlier 97/34 inventory counts omitted root Cargo.toml; both complete counts include it. New allocations are the receipt, three facade modules and eight operation/mixed/scope test modules. E/N below is relative to first use: later consumers of an allocated file are E. No Cargo manifest/dependency or new runner is selected.

| Packet | E/N | Prefix | Basenames |
|---|---|---|---|
| 3.0 | E | `plans/` | `backlog.md` |
| 3.0 | E | `plans/guides/` | `guide_fifteen.md` |
| 3.0 | E | `plans/phases/` | `07-burn-and-clear.md` |
| 3.0 | E | `plans/packages/` | `compiler.md`, `realization.md` |
| 3.0 | N | `plans/history/` | `wave3-compiler-records.md` |
| 3.0 | E | `plans/history/` | `README.md`, `meson.build` |
| 3.0 | E | `plans/drafts/` | `README.md` |
| 3.1a | E | `plans/` | `backlog.md` |
| 3.1a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.1a | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.1a | E | `packages/compiler/src/` | `engine_lifecycle.rs`, `lifecycle.rs`, `analyzed.rs`, `analyzed_validate.rs`, `operation_plan.rs` |
| 3.1a | E | `packages/compiler/src/engine_attestation/` | `lifecycle.rs`, `proof.rs`, `requirement.rs` |
| 3.1a | E | `packages/compiler/src/tests/` | `engine_lifecycle_tests.rs`, `lifecycle_tests.rs`, `compact_validation_tests.rs`, `analyzed_program_oracle_tests.rs`, `analyzed_corruption_tests.rs`, `requirement_tests.rs`, `operation_plan_tests.rs`, `tracing_boundary_tests.rs` |
| 3.1a | E | `packages/tapscript/src/` | `bundle.rs` |
| 3.1a | E | `packages/tapscript/src/tests/` | `compact_validation_tests.rs` |
| 3.1a | E | `packages/vectors/src/` | `first_party.rs` |
| 3.1a | E | `packages/realization/src/tests/` | `burn_tests.rs`, `live_transfer_tests.rs` |
| 3.1b | E | `plans/` | `backlog.md` |
| 3.1b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.1b | E | `packages/realization/` | `README.md`, `TRACING.md` |
| 3.1b | E | `packages/realization/src/` | `derive.rs`, `declassification.rs`, `error.rs` |
| 3.1b | E | `packages/realization/src/tests/` | `derivation_tests.rs`, `disclosure_tests.rs`, `lifecycle_tests.rs`, `burn_tests.rs`, `clear_tests.rs`, `tracing_tests.rs`, `tracing_refusal_tests.rs` |
| 3.2a | E | `plans/` | `backlog.md` |
| 3.2a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.2a | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.2a | E | `packages/compiler/src/` | `source.rs`, `coverage.rs`, `error.rs`, `operation_plan.rs` |
| 3.2a | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `identity.rs` |
| 3.2a | E | `packages/compiler/src/tests/` | `tracing_capture.rs`, `tracing_boundary_tests.rs`, `source_tests.rs`, `coverage_tests.rs` |
| 3.2a | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.2a | E | `packages/vectors/src/` | `report.rs`, `violation.rs`, `plan.rs`, `maturity_safety.rs`, `live_safety.rs`, `compact_campaign.rs`, `compact_campaign_second.rs`, `compact_negative_manifest.rs`, `compact_corpus.rs`, `first_party.rs` |
| 3.5c | E | `plans/` | `backlog.md` |
| 3.5c | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5c | E | `packages/compiler/src/` | `engine_program.rs`, `placement.rs`, `layout.rs`, `engine_cases.rs`, `analyzed_operation.rs`, `coverage.rs`, `operation_plan.rs`, `case.rs`, `analyzed_validate.rs`, `carrier.rs`, `source.rs`, `maturity_announcement_plan.rs`, `lifecycle.rs`, `coverage_graph.rs`, `requirement.rs`, `proof.rs`, `error.rs`, `engine_lifecycle.rs`, `engine_requirement.rs`, `engine_proof.rs`, `live_transfer_plan.rs`, `disclosure.rs`, `engine_source.rs`, `engine_case_facts.rs`, `engine_records.rs`, `analyzed.rs` |
| 3.5c | E | `packages/compiler/src/engine_keyed_fixed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5c | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `operation.rs`, `cases.rs`, `placement.rs`, `identity.rs`, `case_facts.rs`, `proof.rs`, `lifecycle.rs` |
| 3.5c | E | `packages/compiler/src/engine_keyed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5c | E | `packages/compiler/src/tests/` | `source_tests.rs`, `coverage_tests.rs`, `coverage_oracle_tests.rs`, `analyzed_program_oracle_tests.rs`, `engine_program_tests.rs`, `engine_lifecycle_tests.rs`, `requirement_tests.rs`, `lifecycle_tests.rs`, `proof_tests.rs`, `compact_validation_tests.rs`, `layout_tests.rs`, `placement_tests.rs`, `case_tests.rs`, `placement_oracle_tests.rs`, `carrier_tests.rs`, `pilot_placement_tests.rs`, `disclosure_tests.rs`, `analyzed_corruption_tests.rs`, `pilot_coverage_tests.rs`, `coverage_graph_tests.rs`, `oracle_tests.rs`, `maturity_announcement_oracle_tests.rs`, `operation_plan_tests.rs`, `maturity_announcement_plan_tests.rs`, `maturity_announcement_corruption_tests.rs`, `engine_proof_tests.rs`, `engine_source_tests.rs`, `engine_cases_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs` |
| 3.5c | E | `packages/tapscript/src/` | `upstream.rs`, `maturity_assessment.rs`, `operation_assessment.rs` |
| 3.5c | E | `packages/tapscript/src/tests/` | `maturity_assessment_tests.rs` |
| 3.5c | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.5c | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.2b | E | `plans/` | `backlog.md` |
| 3.2b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.2b | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.2b | E | `packages/compiler/src/` | `engine_attestation.rs`, `case.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed.rs`, `analyzed_operation.rs`, `analyzed_validate.rs`, `input.rs`, `error.rs`, `engine_requirement.rs` |
| 3.2b | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `constructibility.rs`, `lifecycle.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.2b | E | `packages/compiler/src/tests/` | `source_tests.rs`, `requirement_tests.rs`, `case_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs` |
| 3.4 | E | `plans/` | `backlog.md` |
| 3.4 | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.4 | E | `packages/compiler/` | `README.md` |
| 3.4 | E | `packages/compiler/src/` | `operation_plan.rs` |
| 3.4 | E | `packages/compiler/src/tests/` | `operation_plan_tests.rs`, `pilot_program_tests.rs`, `placement_oracle_tests.rs`, `coverage_oracle_tests.rs`, `analyzed_program_oracle_tests.rs` |
| 3.4 | E | `packages/linker/src/tests/` | `carrier_tests.rs` |
| 3.4 | E | `packages/vectors/src/` | `plan.rs` |
| 3.3a | E | `plans/` | `backlog.md` |
| 3.3a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.3a | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.3a | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_requirement.rs` |
| 3.3a | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.3a | N | `packages/compiler/src/` | `burn_operation_plan.rs` |
| 3.3a | N | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs` |
| 3.3a | E | `packages/compiler/src/tests/` | `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs` |
| 3.3a | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.3a | E | `packages/realization/` | `README.md` |
| 3.3b | E | `plans/` | `backlog.md` |
| 3.3b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.3b | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.3b | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `burn_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_requirement.rs` |
| 3.3b | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.3b | E | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs` |
| 3.3b | N | `packages/compiler/src/tests/` | `burn_plan_oracle_tests.rs` |
| 3.3b | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.3b | E | `packages/realization/` | `README.md` |
| 3.2c | E | `plans/` | `backlog.md` |
| 3.2c | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.2c | E | `packages/compiler/` | `README.md`, `TRACING.md` |
| 3.2c | E | `packages/compiler/src/` | `engine_attestation.rs`, `case.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed.rs`, `analyzed_operation.rs`, `analyzed_validate.rs`, `input.rs`, `error.rs`, `engine_requirement.rs` |
| 3.2c | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `constructibility.rs`, `lifecycle.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.2c | E | `packages/compiler/src/tests/` | `source_tests.rs`, `requirement_tests.rs`, `case_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs` |
| 3.5a | E | `plans/` | `backlog.md` |
| 3.5a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5a | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5a | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `engine_requirement.rs` |
| 3.5a | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.5a | N | `packages/compiler/src/` | `clear_operation_plan.rs` |
| 3.5a | N | `packages/compiler/src/tests/` | `clear_operation_plan_tests.rs` |
| 3.5a | E | `packages/compiler/src/tests/` | `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `maturity_announcement_plan_tests.rs`, `maturity_announcement_corruption_tests.rs`, `maturity_announcement_oracle_tests.rs`, `operation_plan_tests.rs` |
| 3.5a | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5a | E | `packages/realization/` | `README.md` |
| 3.5b | E | `plans/` | `backlog.md` |
| 3.5b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5b | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5b | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `clear_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `engine_requirement.rs` |
| 3.5b | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs` |
| 3.5b | E | `packages/compiler/src/tests/` | `clear_operation_plan_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `maturity_announcement_plan_tests.rs`, `maturity_announcement_corruption_tests.rs`, `maturity_announcement_oracle_tests.rs`, `operation_plan_tests.rs` |
| 3.5b | N | `packages/compiler/src/tests/` | `clear_plan_oracle_tests.rs` |
| 3.5b | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5b | E | `packages/realization/` | `README.md` |
| 3.5d | E | `plans/` | `backlog.md` |
| 3.5d | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5d | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5d | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `burn_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_program.rs`, `engine_cases.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `lifecycle.rs`, `proof.rs`, `engine_lifecycle.rs`, `engine_requirement.rs`, `engine_proof.rs`, `disclosure.rs`, `engine_source.rs`, `engine_case_facts.rs`, `engine_records.rs` |
| 3.5d | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs`, `identity.rs` |
| 3.5d | E | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs`, `burn_plan_oracle_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs` |
| 3.5d | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5d | E | `packages/realization/` | `README.md`, `TRACING.md` |
| 3.5d | E | `packages/compiler/src/engine_keyed_fixed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5d | E | `packages/compiler/src/engine_keyed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5d | E | `packages/realization/src/declarations/` | `burn.rs` |
| 3.5d | E | `packages/realization/src/` | `burn_clear_laws.rs`, `validate.rs` |
| 3.5d | E | `packages/realization/src/tests/` | `burn_tests.rs`, `burn_clear_relation_tests.rs` |
| 3.5d | E | `plans/packages/` | `realization.md`, `compiler.md` |
| 3.5e | E | `plans/` | `backlog.md` |
| 3.5e | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5e | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5e | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `burn_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_program.rs`, `engine_cases.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `lifecycle.rs`, `proof.rs`, `engine_lifecycle.rs`, `engine_requirement.rs`, `engine_proof.rs`, `disclosure.rs`, `engine_source.rs`, `engine_case_facts.rs`, `engine_records.rs` |
| 3.5e | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs`, `identity.rs` |
| 3.5e | E | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs`, `burn_plan_oracle_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs` |
| 3.5e | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5e | E | `packages/realization/` | `README.md` |
| 3.5e | E | `packages/compiler/src/engine_keyed_fixed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5e | E | `packages/compiler/src/engine_keyed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5e | N | `packages/compiler/src/tests/` | `burn_mixed_plan_tests.rs` |
| 3.5f | E | `plans/` | `backlog.md` |
| 3.5f | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.5f | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.5f | E | `packages/compiler/src/` | `lib.rs`, `input.rs`, `engine_attestation.rs`, `case.rs`, `analyzed.rs`, `analyzed_validate.rs`, `error.rs`, `burn_operation_plan.rs`, `source.rs`, `carrier.rs`, `placement.rs`, `layout.rs`, `coverage.rs`, `coverage_graph.rs`, `requirement.rs`, `sponsor_region.rs`, `analyzed_operation.rs`, `live_transfer_plan.rs`, `engine_program.rs`, `engine_cases.rs`, `operation_plan.rs`, `maturity_announcement_plan.rs`, `lifecycle.rs`, `proof.rs`, `engine_lifecycle.rs`, `engine_requirement.rs`, `engine_proof.rs`, `disclosure.rs`, `engine_source.rs`, `engine_case_facts.rs`, `engine_records.rs` |
| 3.5f | E | `packages/compiler/src/engine_attestation/` | `source.rs`, `requirement.rs`, `proof.rs`, `foundation.rs`, `lifecycle.rs`, `constructibility.rs`, `case_facts.rs`, `cases.rs`, `placement.rs`, `operation.rs`, `coverage.rs`, `coverage_join.rs`, `identity.rs` |
| 3.5f | E | `packages/compiler/src/tests/` | `burn_operation_plan_tests.rs`, `burn_plan_oracle_tests.rs`, `mod.rs`, `source_tests.rs`, `case_tests.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `requirement_tests.rs`, `coverage_tests.rs`, `carrier_tests.rs`, `placement_tests.rs`, `layout_tests.rs`, `proof_tests.rs`, `analyzed_program_oracle_tests.rs`, `live_transfer_plan_tests.rs`, `burn_mixed_plan_tests.rs` |
| 3.5f | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.5f | E | `packages/realization/` | `README.md` |
| 3.5f | E | `packages/compiler/src/engine_keyed_fixed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5f | E | `packages/compiler/src/engine_keyed/program/` | `refusal_tests.rs`, `negative_tests.rs` |
| 3.5f | N | `packages/compiler/src/tests/` | `burn_mixed_oracle_tests.rs` |
| 3.6a | E | `plans/` | `backlog.md` |
| 3.6a | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.6a | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.6a | E | `packages/compiler/src/` | `lib.rs`, `operation_plan.rs`, `burn_operation_plan.rs`, `clear_operation_plan.rs`, `error.rs`, `analyzed.rs`, `analyzed_validate.rs`, `live_transfer_plan.rs`, `maturity_announcement_plan.rs`, `engine_lifecycle.rs`, `engine_requirement.rs` |
| 3.6a | N | `packages/compiler/src/` | `phase7_plan.rs` |
| 3.6a | E | `packages/compiler/src/engine_attestation/` | `requirement.rs`, `lifecycle.rs` |
| 3.6a | N | `packages/compiler/src/tests/` | `phase7_plan_tests.rs` |
| 3.6a | E | `packages/compiler/src/tests/` | `mod.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `live_transfer_plan_tests.rs`, `maturity_announcement_plan_tests.rs` |
| 3.6a | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.6b | E | `plans/` | `backlog.md` |
| 3.6b | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.6b | E | `packages/compiler/` | `README.md`, `TRACING.md`, `meson.build` |
| 3.6b | E | `packages/compiler/src/` | `lib.rs`, `operation_plan.rs`, `burn_operation_plan.rs`, `clear_operation_plan.rs`, `error.rs`, `analyzed.rs`, `analyzed_validate.rs`, `phase7_plan.rs`, `live_transfer_plan.rs`, `maturity_announcement_plan.rs`, `engine_lifecycle.rs`, `engine_requirement.rs` |
| 3.6b | E | `packages/compiler/src/engine_attestation/` | `requirement.rs`, `lifecycle.rs` |
| 3.6b | E | `packages/compiler/src/tests/` | `phase7_plan_tests.rs`, `mod.rs`, `tracing_capture.rs`, `tracing_boundary_tests.rs`, `live_transfer_plan_tests.rs`, `maturity_announcement_plan_tests.rs` |
| 3.6b | E | `packages/compiler/tests/` | `public_api.rs` |
| 3.6b | N | `packages/compiler/src/tests/` | `phase7_plan_oracle_tests.rs` |
| 3.6b | E | `packages/realization/src/tests/` | `derivation_tests.rs`, `lifecycle_tests.rs` |
| 3.7 | E | `plans/` | `backlog.md` |
| 3.7 | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.7 | E | `packages/vectors/src/` | `first_party.rs`, `compact_negative_manifest.rs`, `compact_corpus.rs`, `maturity_mutants.rs`, `maturity_safety.rs` |
| 3.7 | E | `packages/vectors/src/maturity_mutants/` | `native_staging.rs` |
| 3.7 | E | `packages/compiler/src/tests/` | `lifecycle_tests.rs` |
| 3.7 | E | `packages/tapscript/src/tests/` | `compact_validation_tests.rs` |
| 3.8 | E | `plans/` | `backlog.md` |
| 3.8 | E | `plans/history/` | `wave3-compiler-records.md` |
| 3.8 | E | `plans/guides/` | `guide_fifteen.md` |
| 3.8 | E | `plans/phases/` | `07-burn-and-clear.md` |
| 3.8 | E | `plans/packages/` | `compiler.md`, `realization.md` |

## 7. Proposed rulings 221–234 · `sec:wave3:rulings`

**Proposal (Register for adoption)** · `proposal:wave3:rulings`

These statements, reasons, evidence, acceptance and reopening clauses are proposed only. Ruling 220 is reserved by another landing; adoption checks the register again without overwriting another track. Detailed supporting paths and alternative costs are in the route study.

221. Proposed: Retain sibling facades over attestation lowering and the shared engine, preserving the compact public name. Reason: Existing owners already provide the required algorithms; a parallel planner or keyed facade migration has no demonstrated need. Evidence: (`rule:guide15:consume-pipeline`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: 3.0 approves the ledger; each facade freshly revalidates complete canonical projections with private construction. Reopen on an unexpressible fact or missing owner.

222. Proposed: Make relation lifecycle-row ownership operation-local, selecting only its own operation's object mode (output-side for qualified objects), while plan completeness may retain the union. Make choices operation-local, lower Burn's input SideRepresentation, and qualify exit availability by the exit's own input-admission table; publish private/unselected obligations at their actual boundaries. Reason: Company cannot supply a missing Burn choice, and selected-mode filtering otherwise hides the private route. Evidence: (`rec:backlog:wave2-wave3-receiver`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: 3.1a lifecycle/engine owned-choice omission controls, 3.2b operation-local row ownership with no foreign Explicit row in the live private exit, 3.3b TransferLive+Burn per-mode public rows and 3.6b exact carried/unselected scope partition. Reopen on input admission widened by output union, wrong eligibility or a dropped published row.

223. Proposed: Preserve equal-visibility shared-bound union but reject incoming edges and explicit seeds on multiply-declared bounds. Reason: The current seed-free boundary does not define cross-operation propagation. Evidence: (`gate:backlog:wave2-bite-2-7a`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: 3.0 explicitly adopts this prospective tightening of the cited union admission boundary to prevent unsupported reason propagation; 3.1b proves seed-free/one-edge/one-seed controls only. Mixed-pair and local-choice acceptance belongs to ruling 233/222. Reopen on a legitimate consumer needing the unsupported shape.

224. Proposed: Use operation-specific DataOutputs object sets: Burn {}, Clear {State,Ash}, retaining clamp dependencies and accepting None as the multi-object summary only with exact source/activation evidence. Reason: Clear compares destruction to a derived public clamp, unlike Burn's structurally bound records; a single-selection summary is not the dependency census. Evidence: (`tab:realization:burn-clear-graphs`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: 3.2b/3.2c independently review literal object/source/proof/activation tables before activation, including the missing clamp dependency. Reopen on a representation-dependent operand lost from the source table.

225. Proposed: Deliver explicit Burn first and private-change Burn within the same wave before scope/closure, retaining normalized explicit input. Reason: Ruling 141, disposition 3, realization's Wave-3 assignment and the Wave-4 gate already require this boundary; no final explicit-only narrowing is accepted. Evidence: (`rule:guide15-exec:disposition-3`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: Intermediate offered private selection returns TargetPlanRepresentationMismatch; Early 3.5c prepares qualified keys; 3.5d–f then validate and oracle-check the mixed alternative while private input remains refused. Reopen on a target/semantic change or a concrete mixed-route gap requiring a revised packet.

226. Proposed: Compare three independently derived ASH output contracts with both recognizers from one bound scope, retaining amount-role differences. Reason: Object-name equality is insufficient and byte equality remains a later obligation. Evidence: (`rule:guide15:object-closure`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: Six typed comparisons and a changed-constructor-coordinate refusal, plus no hidden creator-secret dependency. Reopen on changed constructor/representation/target premises.

227. Proposed: Use Guide-7 expected-data independence uniformly, bounded power-set enumeration and literal Clear branch classes. Reason: An actual result cannot supply its own expectation; 8^4 is 4,096 and static plan rows are not exercised by repeated arithmetic values. Evidence: (`task:guide15-exec:wave3-deliverable-8`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: Every activation's independent oracle records domain/product counts and does not call the production derivation of the property checked; scope records proof-plan/factor counts. Reopen on circular expected data, missing branch or a domain exceeding its reviewed budget.

228. Proposed: Every required corruption reaches its owning typed/static boundary; include Clear sponsor opacity and explicitly adopt API-shape checks for forbidden target types/graph handles. Reason: Earlier refusal, blocked collateral and absent API surface are different evidence. Evidence: (`rule:guide15:compiler-validation`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: All table coordinates, accepted controls, Authorization/blocked Constructibility and separate Burn/compact/Clear sponsor evidence are accounted for before closure. Reopen on unreachable subjects or evidence promotion.

229. Proposed: Preserve all five engine-base capture digests and byte lengths using Debug that omits only absent qualifiers/empty side maps, without editing capture or MANIFEST. Preserve standalone compact baseline figures/identities while separately publishing the intentional wider-scope lifecycle changes; recount the full Wave-0 baseline at closure. Reason: Revalidation cannot silently replace its denominator or omit the wave-entry comparison. Evidence: (`obl:guide15-exec:baseline-comparison`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: Pre-code Debug compatibility decision at 3.0, unchanged preserve_engine_base at 3.5c, correct pin table at 3.4, reruns after Clear/side-key changes and exact Wave-0 counts/identity dispositions at 3.8. Reopen on a proposed figure/identity/candidate change before its producing packet.

230. Proposed: Re-dispose both compact keys and all three ruling-133 positions on the new compiler candidate, retaining declaration-only and absent-position evidence unless actual emission changes. Reason: New plan support is not an emitted leaf or successor recipe. Evidence: (`tab:backlog:wave2-reopening-results`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: 3.7 includes the target-elements-conformance tree pin, archive/STATE/leaf/witness controls and both realization announcement transition refusals. Reopen on changed leaf, ABI, constructor, pin or offered position.

231. Proposed: Protect every path in either complete freeze inventory at each landing's base; stop for a paired re-pin/history/evidence/review packet if an edit becomes necessary. Reason: The inventories contain 98 and 35 paths, not merely the previously selected compiler subset. Evidence: (`rule:vectors:keyed-refreeze`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: Recompute all writable intersections, coordinate S8b/S8c/S9 and preserve historical evidence. The proposed 140-path union currently intersects neither inventory. Reopen on expanded freeze closure or required pinned edit.

232. Proposed: Adopt nineteen exact packets with 3.5c directly after 3.2a, actual Burn lowering first using qualified keys, and engine_requirement in both dormant and every facade/oracle correction set, a separate history receipt owner and measurement-only timing, with shared core/archive budgets respected. Reason: Facade precedent requires larger review units and separate oracle activation; known whole-wave costs must be visible. Evidence: (`rule:plans:weight`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: Wave-3 settings at 3.0; nineteen review/full-gate slots budgeted, real wall and sizes retained, and the final records pass their own gate. Reopen on unlisted path, exceeded design estimate, missing evidence or exhausted weight reserve.

233. Proposed: Derive existing facade exit sets from the bound scope and per-mode admission; build Phase-7 from one bound input, not three independent plans. Reason: Hard-coded/equal exit closures refuse the required mixed scopes and cannot express private/explicit live divergence. Evidence: (`rec:backlog:wave2-wave3-receiver`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: Operation-local lifecycle-row ownership is a prerequisite: no partner-selected row contaminates a facade. Transfer/Burn controls precede Burn activation; compact/Clear and announcement/Clear precede Clear activation. Common summaries keep their existing type shape and use implemented intersection/outstanding union; exact per-mode rows partition exits and only named reason/status differences are retained. No mixed-scope emission is claimed; tapscript refuses the scoped release-complete compact plan. Reopen on an unlisted projection difference or a consumer treating a summary as per-mode evidence.

234. Proposed: Extend shared key types and enum variants with optional side qualification, preserving empty-key encodings and all five Debug capture digests; add no fields to pinned literal-built Case, Tables, Facts or proof/program Records. Admit only E-input/E-Ash/PrivC-change using ConfidentialConservation with both output-Explicit and Ash-PublicCommitted incompatibilities, and classify only that mixed tuple's conservation visibility as Committed. Reason: Object-only keys cannot express normalized input/private output, while globally weakening proof compatibility would over-admit old owners. Evidence: (`rule:guide13-exec:mixed-representation`), (`sec:w3route:questions`) and (`sec:w3route:mixed`). Acceptance: 3.0 explicitly extends this rule's deferred admission for Burn only after the full eight-item ledger; 3.0 confirms Debug preservation and Ash exclusion/admitted-tuple visibility before code; early 3.5c preserves the unchanged capture, then 3.5d–f independently review tuple/source lowering, the Committed control and one-coordinate refusal, validator and oracle/activation. No new ProofKind, second search or pinned producer edit is selected. Reopen on changed capture bytes, inability to exclude Ash PublicCommitted, incorrect visibility/discharge or an overbroad mixed exception, failure of unary constraints to express the tuple, a required field in pinned-built records, additional owner/schema need or a frozen consumer requiring migration.

## 8. Verified premises and remaining decisions · `sec:wave3:corrections`

The following source checks establish the planning premises and their proposed owners. They are not implementation evidence.

| Item | Verified tree fact and corrected owner |
|---|---|
| Gate-cost attribution | Frozen 1,198s versus closure-import 3,218s, range 950–4,265s verified at backlog:683–695; cost section and ruling 232 distinguish them |
| Input choice ownership | Burn has SideRepresentation for receipts, lowering ignores it and engine checks object-only; Q1 and 3.1a add own input choice, operation-local check and compiler company-omission subject |
| Private obligation publication | engine_proof.rs:279–289 filters by selected mode; Q1/3.3b use the live private projection, 3.6 publishes unselected declared obligations |
| Scope-derived closures | live_transfer_plan.rs:1798–1821 also requires equality across representation closures; Q3/ruling 233 remove that restriction, derive sets and select one bound Phase-7 input |
| Activation ordering | backlog:762 says before activation; pair controls moved to 3.3b and 3.5b, full scope stays 3.6 |
| Relation dependencies | burn_clear_laws.rs:554–558 differs by operation; engine_requirement.rs:9–31 returns None for two choices; Q2/ruling 224 make both meanings explicit |
| Private-change obligation | Four accepted texts require private change; PublicArithmetic prunes it first. Q4/rulings 225/234 preserve obligation, use the offered-selection refusal and four mixed-route packets |
| Dormant reaching boundary | Stage operation guards verified in case/layout/placement/source/requirement; Q6/3.2b relocate to both binding seams and convert direct-entry evidence |
| Lowering correction ownership | Body-only guard and shared DataOutputs verified; operation-aware body interpretation plus retained public Clear guard, and all lowering correction paths in facade/oracle sets |
| Consumer registration | operation_plan re-export and realization README activation owners added; 3.7 names tripod-target-elements-conformance as read-only tree-pin owner |
| Compact baseline | All compact count literals checked at corrected paths/lines in baseline table; no invented count change |
| Freeze closure | Full inventory parsing finds 98 fixed-form and 35 legacy paths; rule covers every current member, intersection recomputed |
| Wave-entry comparison | backlog:779 contains full Wave-0 comparison figures; 3.8 and checklist include all counts and disposition |
| Sponsor opacity | Clear sponsor subject selected at 3.5a/b; existing compact public API subject retained |
| Packet estimates | Nine precedent file lengths checked; per-operation dormant stages and facade/validator versus oracle/activation split, realistic size bands and total review/gate cost |
| Lifecycle evidence boundary | compact lifecycle validator checks graph paths only; 3.1a makes synthetic internal claim, 3.3b handles actual public survival |
| Relation-row ownership | engine_requirement.rs:32–47 currently filters only object/exit; 3.2b and every correction set restrict it to the relation operation's selected mode, output-side when qualified |
| Capture identity | engine_base_capture.rs:155–229 hashes pretty Debug; 3.5c preserves all five unchanged digests by omitting only absent qualifiers and empty side maps |
| Literal-built records | Pinned keyed producers construct Case, Tables, Facts and proof/program Records; ruling 234 permits key/enum changes only, with no new fields in those structs |
| Mixed conservation | Burn Ash admits PublicCommitted; 3.5d excludes it for ConfidentialConservation and grants Committed visibility only to the admitted mixed tuple |
| Early side keys | 3.5c follows 3.2a; Burn/Clear facades are first written against qualified keys, so no later facade migration is hidden |

Private-change implementation requires the explicit shared-key extension and four packets; it cannot fit a small facade estimate. Operation-local lifecycle-row ownership is required for the stated mixed-scope figures, not merely a projection normalization.

Selected design constraints: mint wave-specific settings; use claim-only test names; construct oracle expectations independently over the bounded domains; separate body discharge owners; reuse exact existing operands with disjoint new Fact codes; preserve the literal input-admission table; revalidate compact before activation; keep shared-bound guard acceptance separate; retain reserved errors and converted historical evidence; select both announcement transition refusals; grant paths to actual consumers; measure scope proof/factor counts; and respect archive headroom plus the shared S9 core budget.

The two extra realization regression names in 3.7 are `already_announced_rejects` and `complete_rejects` at `packages/realization/src/tests/state_tests.rs:139,148`, testing the announce_maturity transition. They are read-only selections, not new writable paths. Further property, permutation and extended-cost subjects remain deferred; Q1 resolves the qualification alternative by comparing ownership and timing costs.

## 9. Landing conflicts and weight · `sec:wave3:landing`

The known S8b/S8c/S9 shared paths remain plans/backlog.md, plans/phases/07-burn-and-clear.md and, for S9, plans/guides/guide_fifteen.md plus plans/packages/compiler.md and plans/packages/realization.md. The mixed-proof packet now also writes the latter two live contracts. No selected package path intersects either source freeze or current fixed-form capture roster; any expanded S8c roster is rechecked before landing. Generic behavior can affect another owner even with disjoint pin lists, so existing keyed compatibility tests remain required.

Core remains 1,856,414 bytes, leaving 109,666 hard headroom, shared with S9. The proposed 20 KiB net live-record reserve would leave 89,186 before unrelated changes. Drafts, guides and the new detailed history record are archive-class; guide edits at adoption/closure do not consume that core reserve. At the reviewed starting draft tree archive usage was 7,334,734 / 8,388,608, leaving 1,053,874; report this revision's measured usage separately. Keep live rules with their owners and evidence in history (`rule:plans:weight`).

**Caveat (Unperformed work)** · `cav:wave3:unperformed`

No future plan, oracle, scope, mixed conservation, target capability, source re-freeze, candidate reopening or Wave-3 closing gate is asserted here. The documents gate checks these drafts and registrations. If the reviewed side-key/constraint route proves insufficient, report the concrete missing mechanism with its owners and revised cost before writing it; no explicit-only closure substitutes for the required private-change result.
