# Compiler tracing engineering design

## Purpose and shape · `sec:tracing-compiler:purpose`

`tripod-compiler` is a library that binds a validated realization and explicit operation scope, analyzes target-independent proof and placement alternatives, and returns typed target requirements or validated operation plans; it emits no target program and has no binary or stream-owning API. Tracing is a core dependency (`[ADR026-dec:tracing:core]`).

`input.rs` owns `CompilationScope`, `AnalysisPolicy`, `ProofSearchLimits`, `BoundCompilerInput`, and `bind_input`; `analyzed.rs` orchestrates the complete analysis and validation; `engine_proof.rs`, `engine_placement.rs`, and `search_counter.rs` implement bounded exact searches; `target.rs`, `operation_plan.rs`, `live_transfer_plan.rs`, and `maturity_announcement_plan.rs` project and validate public results; `error.rs` defines typed `CompileError` failures. The other modules derive foundation, graph, source, disclosure, lifecycle, coverage, carrier, and layout facts, while `src/tests` and `tests/public_api.rs` exercise these boundaries.

Public entry points include `bind_input`, `target::analyze_target_requirements`, and the compact-ash, live-transfer, and maturity-announcement planners; public validation functions return `Result` and never print errors. Workspace callers in `vectors`, `tapscript`, `transaction` tests, and `linker` tests consume the typed API; `tapscript::upstream` also re-exports compiler plan types. Search exhaustion, infeasible plans, invalid scope, graph cycles, and validation mismatches are typed `CompileError` results; the caller owns any terminal command diagnostic.

`input/owner.rs` stores the immutable bound owners and read-only accessors; `engine_identity.rs` centralizes the pure const namespace/side encoders and delegates semantic identifier encoding to architecture. These moves add no new execution stage or telemetry-only work: construction remains inside the existing `bind_input`/`bind_keyed_input` spans and encoding remains inside the existing lowering spans. Lowering and its completion events stay with the private owner modules; no per-identifier span or measured duration is added.

## Output inventory · `sec:tracing-compiler:inventory`

This inventory records the design-time baseline before compiler instrumentation: `println!` had two test sites, while `eprintln!`, `print!`, `eprint!`, direct `stdout()`/`stderr()`, and `tracing` calls had zero sites. No production code wrote a standard stream at that baseline, and the two test prints have since been removed.

| Baseline item | Location | Class | Baseline behavior |
|---|---|---|---|
| Proof search report fields | `src/proof.rs:83-104` | counter | Stores visited, complete, feasible, and five cause-specific rejection counts; diagnostic, excluded from semantic identity. |
| Local proof rejection increment | `src/proof.rs:108-116` | counter | Selects one cause-specific field and saturates it. |
| Proof feasible total | `src/proof.rs:373-374` | counter | Converts the final candidate count to `u64`. |
| Proof state and candidate admission | `src/proof.rs:431-435,589-592` | counter | Checks the state and candidate budgets; exhaustion returns typed errors. |
| Proof completion and rejection increments | `src/proof.rs:451,469,482,543,569-574` | counter | Records local, representation, complete-assignment, source, and disclosure outcomes. |
| Shared checked and saturating increment helpers | `src/search_counter.rs:38-56` | counter | Budgeted visits cannot overflow; diagnostic counts saturate. |
| Placement search report fields | `src/placement.rs:796-807` | counter | Stores visited, complete, feasible, and retained-option counts; diagnostic, excluded from stable projection. |
| Placement initial and final totals | `src/placement.rs:1009-1031` | counter | Counts retained options and feasible placements. |
| Placement state, completion, and candidate admission | `src/placement.rs:1522-1539` | counter | Checks state and candidate budgets and records complete assignments; exhaustion returns typed errors. |
| Analysis execution report and accumulation | `src/analyzed.rs:115-120,274-289,558-575` | counter | Returns proof counts and per-operation placement counts; combines placement counts by saturating addition. |
| Manual oracle wall timer | `src/tests/analyzed_program_oracle_tests.rs:40,1271-1274` | timing | `Instant` measures the ignored combined-scope placement regression. |
| Manual oracle factor print | `src/tests/analyzed_program_oracle_tests.rs:1324-1329` | counter | `println!` writes factor, product, and global counts as human text when the ignored test runs with capture disabled. |
| Manual oracle elapsed print | `src/tests/analyzed_program_oracle_tests.rs:1341-1345` | timing | `println!` writes plan count and elapsed seconds as human text when the ignored test runs with capture disabled. |

At the design-time baseline there were zero diagnostic stream writes, zero progress stream writes, and zero standalone production timers; search traversal had no progress reporting. The two test prints were the only standard-output items and were not structured results.

## Standard output and standard error · `sec:tracing-compiler:streams`

The library returns typed values and writes nothing to standard output or standard error. `ProofSearchReport`, `PlacementSearchReport`, and `AnalysisExecutionReport` remain typed return data because callers may need their exact values; tracing mirrors aggregate observations without changing result identity. The ignored oracle asserts factor, product, global, and plan counts without printing or measuring elapsed time; the test harness measures its wall time.

Executable callers own their result schema: an explicit JSON object and newline or documented NDJSON on standard output, with an empty result stream on failure before publication. All first-party program stderr bytes, including early help, usage, TTY refusal, panic, and diagnostics, come from `cli-common`'s subscriber; compiler diagnostics are `tracing` events, never direct writes. This library never installs a subscriber or panic hook. A binary's first statement calls `cli_common::install_command_subscriber(command_name)` before argument parsing; `cli-common` owns the always-on control route, filtered diagnostic route, shared locked stderr writer, filter reload, redaction, and terminal command record. A program command span begins after parsing; `RUST_LOG` is honored before parsing and takes precedence over parsed `--debug` thereafter, and a malformed directive falls back to `info` with a value-free warning.

ADR-010 requires structured result stdout and subscriber-produced JSON stderr (`[ADR010-rule:output:streams]`, `[ADR010-rule:output:json]`, `[ADR010-rule:output:diagnostics]`, `[ADR010-rule:output:implementation]`). `cli-common` emits control records through the subscriber with their established JSON shape and shares one locked writer with diagnostics and panic records. Its explicit bootstrap and legacy startup delegates install the subscriber before argument parsing; ADR-010 assigns child stdout and stderr to explicit file routes owned by `execwrap`, and the command alone publishes its result. Compiler has no binary or direct stream handling; executable owners own subprocess tests of their program contracts.

## Spans · `sec:tracing-compiler:spans`

| Name | Target | Level | Entry fields and types | Duration measured |
|---|---|---|---|---|
| `bind_input` | `compiler::input` | DEBUG | `scope_operation_count: u64` | Revalidation and scope binding can dominate small requests. |
| `analyze_target_requirements` | `compiler::target` | INFO | `scope_operation_count: u64` | Measures public requirement projection and its scoped analysis. |
| `plan_target_operation` | `compiler::plan` | INFO | `operation_code: u16`, `family: &'static str`, `scope_operation_count: u64` | Measures a complete public compact, live, or state plan, including derivation and validation. |
| `analyze_scoped_program` | `compiler::analysis` | DEBUG | `scope_operation_count: u64`, `proof_state_limit: u64`, `placement_state_limit: u64` | Separates target-independent analysis from its public INFO parent. |
| `enumerate_feasible_plans` | `compiler::proof` | DEBUG | `state_limit: u64`, `candidate_limit: u64` | Test oracle: Isolates bounded proof search from preparation and assembly.|
| `enumerate_feasible_placements` | `compiler::placement` | DEBUG | `state_limit: u64`, `candidate_limit: u64`, `relation_case_count: u64` | Test oracle: Isolates each exact placement search under its parent operation.|
| `place_feasible_proof_plans` | `compiler::placement` | DEBUG | `proof_plan_count: u64` | Test oracle: Measures placement and full-set validation across candidate plans.|
| `analyze_candidate_operations_reported` | `compiler::analysis` | DEBUG | `operation_count: u64` | Test oracle: Measures per-operation factor construction without materializing a cross-operation product.|
| `validate_scoped_analyzed_program` | `compiler::analysis` | DEBUG | `proof_plan_count: u64` | Separates final independent validation from initial derivation. |
| `lower_relations` | `compiler::engine_attestation` | DEBUG | `source_node_count: u64`, `source_edge_count: u64` | Converts attestation relation declarations and dependencies into shared records. |
| `validate` | `compiler::engine_case_facts` | DEBUG | `source_count: u64`, `carrier_count: u64` | Checks source, carrier and layout facts for one selected relation case. |
| `select` | `compiler::engine_cases` | DEBUG | `case_count: u64`, `row_count: u64` | Selects case rows for the candidate proofs and representations. |
| `analyze_constructibility` | `compiler::engine_constructibility` | DEBUG | `node_count: u64`, `case_count: u64` | Builds and validates construction dependencies and operation cases. |
| `derive` | `compiler::engine_coverage` | DEBUG | `boundary_count: u64`, `subject_count: u64` | Derives coverage obligations from boundaries and their subjects. |
| `validate_census` | `compiler::engine_coverage` | DEBUG | `planned_count: u64`, `covered_count: u64` | Checks that covered subjects equal the planned subject census. |
| `definition_census` | `compiler::engine_coverage_graph` | DEBUG | `plan_count: u64` | Collects coverage definitions from operation plans. |
| `derive_dependencies` | `compiler::engine_coverage_graph` | DEBUG | `plan_count: u64`, `prerequisite_count: u64` | Derives coverage dependencies from plans and prerequisite relations. |
| `prerequisites` | `compiler::engine_coverage_graph` | DEBUG | `edge_count: u64` | Collects relation prerequisites from dependency edges. |
| `collateral_dependencies` | `compiler::engine_coverage_graph` | DEBUG | `requirement_count: u64` | Derives collateral dependencies for requirement bundles. |
| `build` | `compiler::engine_coverage_graph` | DEBUG | `node_count: u64`, `edge_count: u64` | Assembles and validates the complete coverage graph. |
| `attach_requirements` | `compiler::engine_coverage_join` | DEBUG | `projection_count: u64` | Joins requirement bundles to carrier projections. |
| `alternatives` | `compiler::engine_coverage_join` | DEBUG | `eligibility_count: u64` | Expands coverage alternatives from placement eligibility. |
| `validate` | `compiler::engine_coverage_join` | DEBUG | `plan_count: u64`, `layout_count: u64` | Checks coverage plans against carrier layouts. |
| `validate_predicates` | `compiler::engine_foundation` | DEBUG | `predicate_count: u64`, `expression_count: u64` | Checks predicate expression identities, Boolean types and scope. |
| `lower_predicates` | `compiler::engine_keyed` | DEBUG | `scope_count: u64` | Reads keyed declaration shells and lowers predicate records. |
| `read_declaration_shells` | `compiler::engine_keyed` | DEBUG | `scope_count: u64` | Reads all in-scope shell fields before constructing empty stage records. |
| `lower_relations` | `compiler::engine_keyed` | DEBUG | `scope_operation_count: u64` | Lowers scoped keyed relations and lifecycle records. |
| `lower_sources` | `compiler::engine_keyed` | DEBUG | `scope_count: u64` | Lowers keyed reader obligations and declaration-derived funding regions. |
| `lower_placements` | `compiler::engine_keyed` | DEBUG | `scope_count: u64` | Lowers keyed runtime placement eligibility. |
| `lower_coverage` | `compiler::engine_keyed` | DEBUG | `relation_count: u64` | Lowers keyed relation coverage subjects and dependencies. |
| `analyze_lifecycle` | `compiler::engine_lifecycle` | DEBUG | `choice_count: u64`, `node_count: u64`, `edge_count: u64` | Analyzes lifecycle dependencies, choices and exit requirements. |
| `analyze` | `compiler::engine_operation` | DEBUG | `plan_count: u64` | Assembles operation factors with placement and coverage analyses. |
| `enumerate` | `compiler::engine_placement` | DEBUG | `obligation_count: u64`, `state_limit: u64`, `candidate_limit: u64` | Enumerates bounded placements for runtime obligations. |
| `lower_bound_owner` | `compiler::engine_program` | DEBUG | None | Test oracle: Lowers all stages through the statically bound owner.|
| `analyze` | `compiler::engine_program` | DEBUG | `scope_count: u64` | Runs stage analyses, joins, proof search and operation assembly. |
| `join_proof_stages` | `compiler::engine_program` | DEBUG | `obligation_count: u64` | Compares proof inputs with analyzed lifecycle and constructibility outputs before search. |
| `rederive_bound_owner` | `compiler::engine_program` | DEBUG | None | Test oracle: Rederives from the retained owner and compares the offered program.|
| `enumerate` | `compiler::engine_proof` | DEBUG | `relation_count: u64`, `choice_count: u64`, `state_limit: u64`, `candidate_limit: u64` | Enumerates feasible proof and representation assignments under search limits. |
| `analyze_relations` | `compiler::engine_records` | DEBUG | `source_node_count: u64`, `source_edge_count: u64`, `scope_operation_count: u64` | Builds and validates the scoped relation dependency graph. |
| `derive` | `compiler::engine_requirement` | DEBUG | `table_count: u64`, `selected_count: u64` | Selects requirement bundles for candidate proofs. |
| `validate_aggregate` | `compiler::engine_requirement` | DEBUG | `relation_count: u64`, `source_count: u64`, `evidence_count: u64` | Checks aggregate capabilities, sources and evidence against relation bundles. |
| `analyze_sources` | `compiler::engine_source` | DEBUG | `table_count: u64`, `region_count: u64` | Validates source-table rows and funding-region restrictions. |
| `lower_case_facts` | `compiler::engine_attestation` | DEBUG | `source_count: u64`, `layout_count: u64` | Lowers attestation sources and layouts into case facts. |
| `lower_cases` | `compiler::engine_attestation` | DEBUG | `scope_count: u64` | Lowers attestation relation cases and their compatibility payloads. |
| `lower_layout_scopes` | `compiler::engine_attestation` | DEBUG | `row_count: u64` | Lowers carrier layout scopes for case rows. |
| `lower_constructibility` | `compiler::engine_attestation` | DEBUG | `node_count: u64` | Lowers attestation construction nodes, dependencies and cases. |
| `lower_predicates` | `compiler::engine_attestation` | DEBUG | `relation_count: u64`, `expression_count: u64` | Lowers attestation predicate and expression identities. |
| `lower_lifecycle` | `compiler::engine_attestation` | DEBUG | `source_node_count: u64`, `source_edge_count: u64` | Lowers attestation lifecycle nodes, dependencies and representation choices. |
| `lower_operation_payload` | `compiler::engine_attestation` | DEBUG | None | Prepares owner payloads used to restore operation projections. |
| `restore_operation` | `compiler::engine_attestation` | DEBUG | `case_count: u64` | Restores attestation operation projections from shared case results. |
| `lower_placement` | `compiler::engine_attestation` | DEBUG | `plan_count: u64`, `eligibility_count: u64` | Lowers attestation placement plans and eligibility rows. |
| `publish_bound_program` | `compiler::engine_attestation` | DEBUG | None | Publishes the legacy program projection from common orchestration. |
| `lower_proofs` | `compiler::engine_attestation` | DEBUG | `scope_count: u64` | Lowers attestation proof alternatives, requirements and disclosure records. |
| `prepare_requirement_payloads` | `compiler::engine_attestation` | DEBUG | None | Prepares owner payloads for requirement projection restoration. |
| `restore_requirement_payloads` | `compiler::engine_attestation` | DEBUG | `relation_count: u64` | Restores selected attestation requirement bundles. |
| `lower_requirement_tables` | `compiler::engine_attestation` | DEBUG | `relation_count: u64` | Lowers per-relation requirement alternatives into shared tables. |
| `lower_sources` | `compiler::engine_attestation` | DEBUG | `scope_count: u64` | Lowers attestation source tables and funding regions. |
| `analyze_bound_records` | `compiler::analysis` | DEBUG | None | Analyzes records lowered by the bound owner. |
| `bind_keyed_input` | `compiler::input` | DEBUG | `scope_operation_count: u64` | Validates and binds the keyed manifest, realization and scope. |

All spans use static literal targets and lower-snake-case names, with no per-state or per-candidate span. The three planner entry points share `plan_target_operation`; `family` is exactly `compact`, `live`, or `state`. A subscriber emits one close record for every enabled span with entry fields and `busy_ns: u64` and `idle_ns: u64`, on by default; nested durations are inclusive and must not be summed as a wall total. Compiler never computes a measured duration field or an `Instant` solely for telemetry.

## Events · `sec:tracing-compiler:events`

| Target | Level | Fixed message | Fields and types | Emission point |
|---|---|---|---|---|
| `compiler::input` | DEBUG | `input binding completed` | `outcome: &'static str`; `scope_operation_count: u64` when known; `refusal_kind: &'static str` on refusal or `error_kind: &'static str` on failure | Once on every `bind_input` return, inside its span. |
| `compiler::target` | INFO on completion or failure; DEBUG on refusal | `target requirements analyzed` | `outcome: &'static str`; `operation_count: u64`, `proof_plan_count: u64` when known; `refusal_kind` or `error_kind` as applicable | Once on every public requirements return, inside its span. |
| `compiler::plan` | INFO on completion or failure; DEBUG on refusal | `target operation planned` | `outcome: &'static str`; `operation_code: u16`, `family: &'static str`, `proof_plan_count: u64` when known; `refusal_kind` or `error_kind` as applicable | Once on each compact, live, or state planner return, inside its span. |
| `compiler::proof` | DEBUG | `proof search completed` | `outcome: &'static str`; `visited_state_count: u64`, `complete_assignment_count: u64`, `feasible_assignment_count: u64`, `capability_rejection_count: u64`, `source_rejection_count: u64`, `constructibility_rejection_count: u64`, `representation_rejection_count: u64`, `disclosure_rejection_count: u64` when observed; `refusal_kind` or `error_kind` as applicable | Test oracle: Once on every proof search exit, including state or candidate exhaustion.|
| `compiler::placement` | DEBUG | `placement search completed` | `outcome: &'static str`; `visited_state_count: u64`, `complete_assignment_count: u64`, `feasible_placement_count: u64`, `retained_option_count: u64` when observed; `refusal_kind` or `error_kind` as applicable | Test oracle: Once on every placement search exit, including state or candidate exhaustion.|
| `compiler::engine_placement` | DEBUG | `placement search completed` | `outcome: &'static str`; `refusal_kind: &'static str` on refusal; `visited_state_count`, `complete_assignment_count`, `feasible_placement_count`, `retained_option_count`: `u64` | Once on every production placement-search return, including census, eligibility, assignment and budget refusals. |
| `compiler::engine_proof` | DEBUG | `proof search completed` | `outcome: &'static str`; `refusal_kind: &'static str` on refusal; `visited_state_count`, `complete_assignment_count`, `feasible_assignment_count`, `capability_rejection_count`, `source_rejection_count`, `constructibility_rejection_count`, `representation_rejection_count`, `disclosure_rejection_count`: `u64` | Once on every production proof-search exit, including fixed-source refusal, no feasible plan, and state/candidate exhaustion. |
| `compiler::input` | DEBUG | `keyed input binding completed` | `outcome`, optional `refusal_kind`: static strings | Once per experimental keyed binding return. |
| `compiler::engine_attestation` | DEBUG | `relation records lowered` | `outcome: &'static str`, `identity_count: u64` | Successful relation lowering. |
| `compiler::engine_attestation` | DEBUG | `owner record refusal restored` | `stage: Debug`, `relation_count: u64`, `unknown_relation_count: u64`, `outcome: &'static str` | Owner adapter restores public refusal subjects without exposing private records. |
| `compiler::engine_keyed` | DEBUG | `relation records lowered` | `outcome: &'static str`, `source_node_count: u64` | Successful keyed relation lowering. |
| `compiler::engine_keyed` | DEBUG | `declaration shell read` | `outcome: &'static str`, `field: Debug` on refusal, `field_count: u64` | Unsupported nonempty fields are refused before lowering; completed means every observed shell field is empty. |
| `compiler::engine_attestation` | DEBUG | `lifecycle records lowered` | `outcome: &'static str`, `choice_count: u64` | Successful lifecycle lowering. |

The production placement search uses `compiler::engine_placement`; the legacy `compiler::placement` wrappers are test oracles. The production proof search uses `compiler::engine_proof`; the legacy `compiler::proof` wrapper is a direct-stage test oracle. Shared refusal names are `no_feasible_proof_plan`, `proof_search_state_limit_exceeded`, `proof_candidate_limit_exceeded` and `fixed_source`; they contain no owner payload.

`outcome` is exactly `completed` for a typed result, `refused` for an expected typed refusal, or `failed` for an operational or internal failure. A negative report returned as a value is still `completed`; compiler makes no domain-verdict or evidence-standing claim. Refusals use the snake_case form of the payload-free `CompileError` variant name, including `proof_search_state_limit_exceeded`, `proof_candidate_limit_exceeded`, `placement_search_state_limit_exceeded`, or `placement_candidate_limit_exceeded`; a failure uses `error_kind` instead. Search summaries report counts already observed at exit and never publish a partial result; an unavailable count is omitted, not fabricated. Budget limits are span entry fields, while limit type is the refusal kind rather than a new outcome value.

An instrumented public boundary emits exactly one completion event on return, even when nested operations run; same-crate public-to-public calls use an uninstrumented core to avoid duplicate public completion events. Repeated search summaries are DEBUG, expected refusals are DEBUG even under INFO public spans, and returned failures retain their span's INFO or DEBUG tier. The library emits WARN only for a recoverable anomaly with a usable result and ERROR only for a hidden invariant break or unreturned data loss; the executable's `cli-common` terminal path owns one ERROR diagnostic for a returned command failure. Rejections counted inside searches stay aggregate, never per candidate; no event contains realization contents, witnesses, graph or plan values, paths, hashes, nonces, amounts, or free error text.

## Naming · `sec:tracing-compiler:naming`

Targets are static literals `compiler::input`, `compiler::analysis`, `compiler::proof`, `compiler::placement`, `compiler::target`, and `compiler::plan`, plus the static `compiler::engine_*` targets enumerated in the tables; span names use lower_snake_case without a package prefix, and event messages are short fixed lowercase English past-tense phrases. Field keys are static snake_case. `family: &'static str` is `compact`, `live`, or `state`; `operation_code: u16` is the architecture `OperationId` code. `outcome`, `refusal_kind`, and `error_kind` have the meanings in Events, with categorical variant names converted to snake_case by inserting an underscore before every capital and lowercasing; `phase: &'static str` is used only if a phase is needed and is never a duplicate span name. No `command`, `output_mode`, `exit_class`, or `result_published` field is copied from `cli-common` into library events.

Every emitted count is a `<noun>_count: u64`, every byte quantity is `<subject>_bytes: u64`, and configured search bounds are `state_limit: u64` and `candidate_limit: u64`; no bare plural, `_len`, `maximum`, or count-of-one field is introduced. Existing API and internal `usize` values remain unchanged; at emission, `u64::try_from(n).unwrap_or(u64::MAX)` saturates a collection length. Events carry only already-held, safe values and do not format errors or inputs with `Debug` or `Display`. The payload-free `stage` and `field` enums are exceptions: their `Debug` spelling is the declared enum variant name, without payload, rather than snake_case.

## Cost · `sec:tracing-compiler:cost`

Exact search, full validation, and formatting plan or graph values are expensive. Span fields use already available lengths, limits, and numeric operation codes; no field traverses a plan or formats a `BoundCompilerInput`, `CompileError`, witness, graph, or result. The `usize`-to-`u64` conversion occurs at emission and is cheap; any future derived field or lazy formatting is guarded by `tracing::enabled!`. Search summaries read existing counters once per search exit, never per state, and instrumentation creates no `Instant`, aggregate traversal, allocation, or measured-duration event when no subscriber is installed.

## Testing · `sec:tracing-compiler:testing`

Use a package-local `tracing-subscriber` dev dependency and an in-memory collector installed with `tracing::subscriber::with_default` around each call; serialize scoped collector installation and rebuild tracing callsite interest inside the scope before the subject call. Every capture test re-executes its own test binary as a child selecting exactly its full test path, marked by an environment variable; the parent requires successful exit and exactly one passed test, and includes the child's output in any failure. This keeps uncaptured tests in the parallel parent process from changing callsite interest during a capture, without installing a global subscriber. Assert targets, span names, levels, fixed messages, typed fields, exactly one public completion event, and search summaries on success and exhaustion. Each instrumented public boundary gets an accepted control and a one-coordinate expected refusal at its declared boundary, including state-limit exhaustion; assert `outcome = refused`, `refusal_kind`, DEBUG level, and no WARN or ERROR for that refusal. Compare aggregate event counts with returned reports where a report exists, and assert identical typed result bytes and values with and without capture; no partial result is inferred from an exhausted search. A two-thread regression test first touches a traced callsite without capture, then requires one span from each scoped capture.

The `CompileError::tracing_kind` table test checks every static name against the variant's underscore-before-every-capital conversion and the lower-snake-case field grammar; boundary capture tests pin representative returned refusal names.

The ignored oracle test asserts factor, product, global, and plan counts directly, emits no tracing event or stream text, and leaves wall measurement to the harness. Compiler has no production stdout contract to exercise in-process. Binary owners test JSON/NDJSON stdout, empty stdout before publication, control records under `RUST_LOG=off`, panic behavior, filter reload, and span close records in subprocesses of their actual executables; compiler tests never install a global subscriber or redirect process stdout.

## Implementation plan · `sec:tracing-compiler:plan`

1. Add a package-local scoped collector test utility and `tracing-subscriber` dev dependency, then capture typed results and one-coordinate refusal fixtures for binding, requirements, planners, and search limits; isolate each capture test in its own process.
2. Add `bind_input`, `analyze_target_requirements`, and shared `plan_target_operation` public spans and exactly one return event each; factor same-crate public-to-public calls through uninstrumented cores, preserve return values, and test the `completed`/`refused`/`failed` mapping.
3. Add DEBUG analysis and search spans and one aggregate event on every proof and placement search exit, including budget exhaustion; keep checked budgets and returned reports as semantic data, compare event counters with reports, and verify no duplicate completion.
4. Remove the ignored oracle's `Instant` and two `println!` calls, replace their measurements with assertions and harness timing, and test the absence of test stream writes.

The shared `cli-common` implementation supplies ADR-010's subscriber, control route, command runners, and result publication. Compiler's implementation sequence has no binary step and does not depend on the legacy initialization delegates or their eventual removal; executable owners own subprocess stdout/stderr tests.

## Open questions for the other packages · `sec:tracing-compiler:questions`

- Target naming is resolved to static `compiler::<domain>` literals and lower-snake-case span names; `cli-common` alone owns command fields.
- Public results use `completed`, expected typed refusals use `refused` at DEBUG, and returned operational failures use `failed` at the span tier; `refusal_kind` is a payload-free `CompileError` variant, while `error_kind` is for failures.
- Outer public analysis and planning spans are INFO, nested analysis and repeated searches are DEBUG, and search counts are emitted once per exit with `_count: u64` fields.
- `cli-common` owns pre-parse subscriber installation, control events, filter reload, and one terminal command record under ADR-010; the compiler library owns no subscriber or stream.
- The ignored oracle uses assertions and test-harness wall time, with no shared scoped collector crate and no measured-duration event; the subscriber's span close record supplies enabled span timings.
