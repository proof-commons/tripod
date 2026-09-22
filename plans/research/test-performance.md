# Research Question: Test Gate Performance · `q:test-performance:swiftness`

> **Status:** Open; scheduling prototype and decision required
> **Track:** `T11-110`
> **Affected packages:** Every workspace member and the Meson test harness
> **Expected handoff:** A measured first performance change with unchanged proof coverage

## Decisive question · `sec:test-performance:question`

How can the test gate become swift without weakening what a passing gate proves? The unit of diagnosis is both the Meson package lane and the individual libtest event: a lane wall includes compilation and waiting, whereas an individual test duration can include fixture initialization and contention. Neither is a CPU-work counter.

## Fixed constraints · `sec:test-performance:constraints`

Every retained package lane must prove everything its current `cargo test -p` invocation proves: unit, integration, and documentation tests, with the same features, assertions, refusals, and ignored-test policy. Per-test attribution and measured command walls remain mandatory. Scheduling changes must preserve failures and timeouts. A move of release verification to tags and releases requires an explicit toolchain-policy change; the current clean-tree gate still includes both profiles (`[ADR011-rule:toolchain:clean-tree]`). No cadence, test, harness, or budget changes are made by this record.

The threat model is a faster green obtained by dropping a binary, a doctest, a negative control, a property domain, or an optimized-profile failure; reusing mutable fixture state; hiding failed or untimed events; or comparing unlike caches and contention. Required vectors for any prototype are the exact before/after test census, unchanged positive and negative outcomes, intentional failure propagation, fixture isolation, deterministic canonical outputs, and retained per-test duration records. A shared fixture must leave its independent admission and mutation-refusal tests intact.

## Measured picture · `sec:test-performance:measurements`

The completed historical observation is `gate-pending106.log`, started on 2026-09-22 at 10:13:34 UTC and finished at 10:38:56 UTC on orel: all 52 lanes passed, the enclosing gate took 1,522 s, and `sh scripts/ci.sh` took 1,518.5 s before the final formatting check. Its lane walls sum to 12,533.60 s, giving 8.23 overlapping lane-equivalents against the enclosing wall. These numbers come from the log's Meson result lines, `RUN-REPORT` line, and completion footer; the sum was recounted with `awk` over all 52 result lines. The only matching completed log on this node was that file; earlier observations on other nodes remain unread.

The fresh inventory commands were `hostname`, `nproc`, `lscpu`, `rustc -Vv`, and `cargo -V`, each under `/workspace/run-report.sh nice -n 19 ionice -c3`. The node exposes 128 physical cores and 256 logical CPUs: two sockets, 64 cores per socket, and two threads per core; `nproc` returns 256. Rust is `1.100.0-nightly`, dated 2026-09-21, with LLVM 23.1.1; Cargo is `1.100.0-nightly`, dated 2026-09-16. The historical log records the same Rust and Cargo versions. The CPU inventory was measured at 10:49 UTC, not during the historical gate.

The fresh cold full run took 1,391.4 s; the warm full run took 568.0 s, with all 52 lanes passing in each. The subsequent warm debug-only command took 325.6 s and the release-only command 30.6 s, each passing all sixteen lanes. The warm full run's lane walls sum to 2,119.694528 s, or 3.73 overlapping lane-equivalents. These are the `RUN-REPORT` command walls and the sum of `duration` fields in each preserved `meson-logs/testlog.json`; the suite commands are `sh scripts/ci.sh --suite cargo-debug` and `sh scripts/ci.sh --suite cargo-release`. Historical and fresh observations remain separate. A missing measurement is unavailable, never zero.

Cold means first full-test compilation, not an empty target: the preceding documentation verification had already built checkers and label tests. The cold full run still compiled the workspace tests; its debug and release warm-up targets took 53.094 s and 178.278 s. The warm full run reused those builds: its warm-ups took 0.310 s and 0.256 s, and their Cargo logs contain no compilation. The isolated debug and release warm-ups took 0.258 s and 0.290 s respectively. These target walls are the last matching target records in each preserved `.ninja_log`, computed as `(end-start)/1000`; the other profile's old record in a suite-only snapshot is not a new measurement. The cold lane sum was 10,773.127129 s, giving 7.742653 overlapping lane-equivalents; isolated debug and release sums were 1,182.399273 s and 168.578795 s, giving 3.631447 and 5.509111 respectively.

The measured load averages, from `cat /proc/loadavg` immediately before and after each command, were: cold 13.09/12.39/14.29 to 11.32/15.73/19.15; warm 11.32/15.73/19.15 to 14.58/29.54/30.00; debug 14.58/29.54/30.00 to 29.64/41.68/37.54; release 29.64/41.68/37.54 to 31.72/41.13/37.50. Each triple is the one-, five-, and fifteen-minute average. Historical loads were unavailable. The warm isolated debug wall is 5:25.6, above the 5:00 reference budget and inside its 8:00 tolerance; release is 0:30.6, inside its 10:30 budget. This node differs from the budget's 14-core reference.

Warm Meson intervals show all 32 package lanes starting together and a peak of 37 concurrent lanes, obtained by sorting their start/end events and accumulating active lanes. The last release lane finished before debug vectors, so the observed release-only tail was zero. The full run's package-lane sums were 1,342.057175 s debug and 498.246001 s release. Comparing these with the isolated runs reveals changed contention, but does not isolate its cause. The isolated suite walls cannot simply be added or subtracted to predict a different full gate.

Summing sequential libtest suite-summary walls within each package gives debug vectors 342.992673 s of its 346.532810 s lane, conformance 292.380493 s of 297.046484 s, and compiler 184.273741 s of 187.218314 s. Their remaining 3.540137 s, 4.665991 s, and 2.944573 s jointly cover compilation, Cargo waiting, process launch, and other unreported work. Release vectors leaves 3.952936 s, conformance 10.641773 s, and execwrap 8.903859 s outside suite summaries. All 32 warm package logs contain Cargo lock notices, but no duration for each wait. Doctest compilation cannot be isolated from this residual. In contrast, cold release execwrap occupied 769.15 s while its suite summaries totalled only 0.037938 s: the cold queueing problem is real, but it does not explain the warm heavy lanes.

### Full lane walls · `tab:test-performance:lanes`

Durations are seconds. Historical values are the full Meson result-line census in `gate-pending106.log`; all historical results are `OK`. The warm values come from the preserved `warm/meson-logs/testlog.json` duration fields; all 52 results are `OK`. Both columns are rounded to two decimal places.

| Lane | Historical wall | Warm wall |
|---|---:|---:|
| cargo-test-debug-architecture | 53.98 | 42.65 |
| cargo-test-debug-artifacts | 56.16 | 9.26 |
| cargo-test-debug-cli-common | 60.51 | 2.32 |
| cargo-test-debug-document-stamps | 112.22 | 1.74 |
| cargo-test-debug-execwrap | 113.52 | 3.23 |
| cargo-test-debug-flatten-latex-main | 114.14 | 2.63 |
| cargo-test-debug-labels | 142.61 | 20.93 |
| cargo-test-debug-linker | 104.26 | 31.42 |
| cargo-test-debug-model | 204.55 | 153.25 |
| cargo-test-debug-realization | 155.08 | 83.48 |
| cargo-test-debug-tapscript | 88.98 | 123.20 |
| cargo-test-debug-target-elements | 85.67 | 4.79 |
| cargo-test-debug-target-elements-conformance | 411.32 | 297.05 |
| cargo-test-debug-transaction | 115.56 | 32.36 |
| cargo-test-debug-vectors | 487.79 | 346.53 |
| cargo-test-debug-compiler | 200.90 | 187.22 |
| cargo-test-release-architecture | 481.85 | 12.46 |
| cargo-test-release-artifacts | 622.73 | 15.15 |
| cargo-test-release-cli-common | 627.38 | 10.24 |
| cargo-test-release-document-stamps | 805.98 | 9.64 |
| cargo-test-release-execwrap | 889.16 | 9.03 |
| cargo-test-release-flatten-latex-main | 881.28 | 3.62 |
| cargo-test-release-labels | 832.15 | 13.22 |
| cargo-test-release-linker | 276.94 | 13.21 |
| cargo-test-release-model | 931.86 | 39.21 |
| cargo-test-release-realization | 879.89 | 28.40 |
| cargo-test-release-tapscript | 141.38 | 44.28 |
| cargo-test-release-target-elements | 276.74 | 9.18 |
| cargo-test-release-target-elements-conformance | 444.67 | 89.78 |
| cargo-test-release-transaction | 213.05 | 12.16 |
| cargo-test-release-vectors | 624.99 | 97.41 |
| cargo-test-release-compiler | 818.84 | 91.25 |
| attestation-stamps | 0.27 | 0.23 |
| cargo-clippy | 0.26 | 0.22 |
| census-audit | 0.02 | 0.02 |
| check-generated | 0.02 | 0.01 |
| cargo-fmt | 3.66 | 9.58 |
| cargo-doc-workspace | 18.22 | 0.30 |
| labels-check | 0.07 | 0.06 |
| plans-check | 0.06 | 0.05 |
| forbidden-text-check | 0.05 | 0.04 |
| hash-citations-check | 0.04 | 0.04 |
| publication-mode-freshness | 0.32 | 0.29 |
| live-native-v2-r7-capture-contract | 40.45 | 48.76 |
| live-native-v2-r8-capture-contract | 42.61 | 51.60 |
| attestation-stamps-scopes | 14.59 | 14.83 |
| tracked-path-argv-audit | 0.06 | 0.06 |
| executor-classification | 0.12 | 0.07 |
| cargo-audit | 4.68 | 1.75 |
| executor-boundaries | 140.53 | 140.18 |
| meson-mock-contract | 11.40 | 11.28 |
| clean-tree | 0.03 | 0.03 |

### Forty longest warm-run tests · `tab:test-performance:tests`

All forty are debug library tests, in suite ordinal 1. Binary identities come from the matching Cargo `Running` headings: `target_elements_conformance`, `model`, `vectors`, and `tapscript`, each built from its package's `src/lib.rs`. All completed successfully. Durations are seconds, rounded to six decimal places; ranking spans both profiles and every crate.

| Rank | Crate | Binary | Profile | Test | Seconds |
|---|---|---|---|---|---:|
| 1 | target-elements-conformance | target_elements_conformance | debug | `tests::constructor_tests::the_corpus_measures_no_retry_and_claims_nothing_about_the_tail` | 151.083792 |
| 2 | model | model | debug | `tests::property_trace_tests::arbitrary_action_traces_preserve_invariant` | 150.085849 |
| 3 | vectors | vectors | debug | `live_evidence::tests::every_fresh_native_refusal_matches_its_rows_declared_boundary` | 121.251727 |
| 4 | vectors | vectors | debug | `live_evidence::tests::both_discharging_vocabularies_are_really_driven` | 121.251574 |
| 5 | vectors | vectors | debug | `live_evidence::tests::the_computability_blocker_is_carried_by_no_row` | 121.246376 |
| 6 | vectors | vectors | debug | `live_evidence::tests::one_row_is_closed_by_the_operation_vocabulary_and_it_is_not_evidence` | 121.228569 |
| 7 | vectors | vectors | debug | `live_evidence::tests::every_typed_closure_routes_to_its_own_non_answer` | 121.218306 |
| 8 | vectors | vectors | debug | `live_evidence::tests::exactly_the_positive_target_observations_are_bound_to_the_fresh_corpus` | 121.197260 |
| 9 | vectors | vectors | debug | `live_evidence::tests::the_first_party_half_of_the_matrix_is_answered_in_full` | 121.194287 |
| 10 | vectors | vectors | debug | `live_evidence::tests::the_first_party_discharges_are_the_ones_the_census_staged` | 121.186333 |
| 11 | vectors | vectors | debug | `live_evidence::tests::the_four_leaf_arrangement_rows_have_two_observations_one_closure_and_one_requirement` | 121.184335 |
| 12 | vectors | vectors | debug | `live_evidence::tests::the_minimality_source_is_built_and_carries_no_verdict` | 121.180667 |
| 13 | vectors | vectors | debug | `live_evidence::tests::the_negative_half_awaits_a_run_rather_than_an_absent_control` | 121.169774 |
| 14 | vectors | vectors | debug | `live_evidence::tests::the_plan_classifies_every_row_exactly_once` | 121.166022 |
| 15 | vectors | vectors | debug | `live_evidence::tests::exactly_the_negative_refusals_are_bound_to_the_fresh_corpus` | 121.151580 |
| 16 | vectors | vectors | debug | `live_evidence::tests::the_unexpected_boundary_set_is_exact` | 121.141309 |
| 17 | vectors | vectors | debug | `live_evidence::tests::the_plan_refuses_to_call_itself_complete` | 121.117389 |
| 18 | vectors | vectors | debug | `live_evidence::tests::the_real_corpus_retypes_exactly_forty_two_rows_atomically` | 121.116763 |
| 19 | vectors | vectors | debug | `live_evidence::tests::the_seven_conservation_rows_declare_the_boundary_their_run_reached` | 121.110164 |
| 20 | vectors | vectors | debug | `live_evidence::tests::the_two_report_rows_are_requirements_not_plan_answers` | 121.059841 |
| 21 | vectors | vectors | debug | `live_evidence::tests::a_missing_ceremony_refuses_the_whole_overlay` | 120.982797 |
| 22 | vectors | vectors | debug | `live_evidence::tests::a_foreign_identity_cannot_replace_a_fresh_identity` | 120.979757 |
| 23 | vectors | vectors | debug | `live_evidence::tests::an_absent_overlay_moves_zero_rows_and_preserves_the_old_census` | 120.972046 |
| 24 | vectors | vectors | debug | `live_evidence::tests::a_non_exact_attribution_census_refuses_atomically` | 120.970266 |
| 25 | vectors | vectors | debug | `live_evidence::tests::a_cross_run_support_link_refuses_the_whole_overlay` | 120.965618 |
| 26 | vectors | vectors | debug | `live_evidence::tests::a_wrong_row_attribution_refuses_the_whole_overlay` | 120.964310 |
| 27 | vectors | vectors | debug | `live_evidence::tests::a_duplicated_primary_link_refuses_the_whole_overlay` | 120.960128 |
| 28 | vectors | vectors | debug | `live_evidence::tests::the_raw_partition_moves_six_native_rows_to_typed_closures` | 120.860477 |
| 29 | vectors | vectors | debug | `live_fault_discharge::tests::every_case_discharges_its_row_against_its_own_control` | 120.786988 |
| 30 | vectors | vectors | debug | `live_minimality_report::tests::the_two_reports_are_different_documents_at_their_first_line` | 112.983268 |
| 31 | target-elements-conformance | target_elements_conformance | debug | `tests::prototype_validate_tests::the_report_bytes_are_a_function_of_the_run` | 112.686885 |
| 32 | vectors | vectors | debug | `live_multi_shapes::byte_identity_tests::the_shapes_this_wave_added_carry_digests_of_their_own` | 111.460535 |
| 33 | target-elements-conformance | target_elements_conformance | debug | `tests::guide11_reproductions::g11_r03_the_canonical_wide_floor_matrix_is_the_evidence_subject` | 109.097388 |
| 34 | tapscript | tapscript | debug | `tests::state_announcement_tests::lead_cross_product_checks_both_endpoints_and_unsigned_overflow` | 107.861200 |
| 35 | target-elements-conformance | target_elements_conformance | debug | `tests::prototype_validate_tests::a_relabelled_case_status_is_refused` | 107.116131 |
| 36 | vectors | vectors | debug | `live_corpus_native_operator::tests::replay_refuses_changed_submission_signing_and_funding_subjects` | 106.834278 |
| 37 | target-elements-conformance | target_elements_conformance | debug | `tests::prototype_validate_tests::a_relabelled_claim_disposition_is_refused` | 106.107872 |
| 38 | target-elements-conformance | target_elements_conformance | debug | `tests::prototype_validate_tests::an_edited_fixture_projection_is_refused` | 105.710463 |
| 39 | target-elements-conformance | target_elements_conformance | debug | `tests::prototype_validate_tests::a_declared_mock_run_cannot_satisfy_the_prototype_gate` | 105.482762 |
| 40 | target-elements-conformance | target_elements_conformance | debug | `tests::prototype_validate_tests::a_deleted_case_row_is_refused` | 104.988560 |

### Executed counts and in-test sums · `tab:test-performance:crates`

Counts include completed unit, integration, and documentation tests; ignored tests are separate. Debug and release executed and ignored counts agree for every crate. Every completed test has a duration; none failed. Attribute counts are the separate source cross-check and need not equal expanded runtime tests. In-test sums are overlapping thread residence, in seconds, not additive gate savings.

| Crate | Source test attributes | Executed per profile | Ignored per profile | Debug in-test seconds | Release in-test seconds |
|---|---:|---:|---:|---:|---:|
| architecture | 165 | 166 | 0 | 41.448142 | 3.096100 |
| artifacts | 11 | 12 | 0 | 4.688629 | 6.787927 |
| cli-common | 74 | 76 | 0 | 0.201471 | 1.490019 |
| compiler | 814 | 812 | 3 | 6395.540664 | 2529.305041 |
| document-stamps | 44 | 45 | 0 | 1.239268 | 1.438733 |
| execwrap | 50 | 52 | 0 | 0.250178 | 0.175433 |
| flatten-latex-main | 40 | 41 | 0 | 0.162745 | 0.164175 |
| labels | 223 | 226 | 0 | 58.705355 | 15.713872 |
| linker | 239 | 244 | 0 | 488.802098 | 185.724694 |
| model | 376 | 358 | 0 | 201.153168 | 39.605617 |
| realization | 354 | 359 | 0 | 104.203852 | 24.320998 |
| tapscript | 613 | 617 | 0 | 1576.225373 | 185.657989 |
| target-elements | 252 | 253 | 0 | 1.563790 | 0.132734 |
| target-elements-conformance | 678 | 679 | 0 | 4620.234162 | 664.193825 |
| transaction | 453 | 466 | 0 | 1351.998087 | 324.413412 |
| vectors | 873 | 830 | 47 | 10245.431219 | 2353.870607 |

## Worst offenders and verified causes · `sec:test-performance:causes`

The three heaviest warm lanes are debug vectors (346.532810 s), debug target-elements-conformance (297.046484 s), and debug compiler (187.218314 s). Their suite-summary walls show that executed test harnesses dominate. Shared lazy initialization charges its wait to several concurrent test events; summed in-test duration is therefore not the saving from removing one initialization. The source review below follows every named top-forty test, grouping only tests with the same verified call chain.

Rank 1, `tests::constructor_tests::the_corpus_measures_no_retry_and_claims_nothing_about_the_tail`, took 151.083792 s. `packages/target-elements-conformance/src/tests/constructor_tests.rs` loops over 64 counters, three flags, and two object kinds: 384 real `construct_under_policy` calls with nonce zero, asserting the corpus count and absence of retries. Its curve-arithmetic construction is evidence; the test expressly makes no rare-tail claim. Rank 2, `tests::property_trace_tests::arbitrary_action_traces_preserve_invariant`, took 150.085849 s. `packages/model/src/tests/property_trace_tests.rs` configures 1,000 generated action traces and 100,000 maximum shrink iterations, then checks the invariant along each trace. This domain is not exhausted by a small finite census, so lowering its case count has no coverage argument here.

Ranks 3–28 are the individual assertions in `packages/vectors/src/live_evidence.rs`: their 120.860477–121.251727 s durations include `derive_live_evidence_plan` or its raw-plan path. Those paths consume the live transfer plan, demonstration bundle and ABI, first-party and fault discharges, and the admitted native corpus. `live_plan.rs`, `live_fault_discharge.rs`, and `live_corpus_native_v2_r7.rs` already memoize expensive common derivations with `OnceLock`. The clustered durations are consistent with shared initialization waits; there is no instrumentation that apportions initialization versus each assertion. Rank 29 explicitly validates each fault against its own honest control and mutant in `live_fault_discharge.rs`; rank 30 assembles and renders two distinct reports in `live_minimality_report.rs`; rank 32 constructs and settles every shape before checking its successor digest in `live_multi_shapes.rs`; rank 36 mutates six signing, funding, or submission subjects and replays each exchange in `live_corpus_native_operator.rs`. These repeated derivations and replays must not disappear behind a cache of the result being tested.

Ranks 31, 35, and 37–40 are in `packages/target-elements-conformance/src/tests/prototype_validate_tests.rs`. Its `report_over` helper constructs a target, the wide-floor matrix, an agreeing transcript, and an evaluated report each time. The determinism test at rank 31 constructs two independent reports; the other tests mutate status, disposition, fixture projection, mock trust, or a case census and require validation to refuse the result. Rank 33 in `guide11_reproductions.rs` independently builds and evaluates the same canonical matrix, then validates and gates it. Immutable shared inputs are a candidate; caching the independent evaluation or refusal would erase the assertion's subject.

Rank 34 in `packages/tapscript/src/tests/state_announcement_tests.rs` took 107.861200 s. Its explicit cross-product uses seven boundary cycle values and four lead values, retains ordered lead pairs, and calls `check_window` at endpoints and available neighboring values, including unsigned-overflow cases. The test is already a finite boundary census; any batching must retain every accepted and refused window. Compiler's longest warm tests also rebuild their proof subjects: `live_transfer_plan_tests::equal_inputs_and_scope_permutations_produce_equal_plans` took 73.637667 s for independent equivalent plan constructions, `coverage_oracle_tests::the_oracle_expects_sponsor_cardinality_in_both_pilots` took 68.096096 s for both pilots and both sides, and `analyzed_program_oracle_tests::repeated_clean_analysis_projects_equally` took 61.392878 s for repeated clean analyses. These functions were read in `packages/compiler/src/tests/`; their independence is part of the proof.

The long serial tail has a different cause. `scripts/test-executor-boundaries.py` repeatedly creates a signing fixture whose stand-in node lives for 2.0 s. The mock CLI acknowledges stop without terminating that process; the adapter's `Node.stop()` then waits for process exit. Serial signing cases pay the remaining lifetime repeatedly, contributing to the measured 140.183334 s lane. The two intentional boot-failure lifetime cases must remain. The root `meson.build` places this independent check after the serial `attestation-stamps-scopes` barrier, while the two pure-shell native capture checks run after the serial `cargo-doc-workspace` barrier. This ordering leaves otherwise independent work on the gate's final path.

## Remedies: less work and more parallel work · `sec:test-performance:remedies`

### Candidate matrix · `tab:test-performance:remedies`

Savings below are measured opportunities or explicitly conditional models, not timings of unimplemented changes. A zero fixed-duration saving can still hide contention relief; proving that relief requires a new comparable run. Costs that overlap cannot be added.

| Family and candidate | Measured basis and saving supported | Implementation cost | Proof retained | Proposed bite |
|---|---|---|---|---|
| More parallel: schedule independent checks before serial barriers | Warm independent tails are 51.645091 s after documentation and 140.200734 s after attestation; overlapping both with package tests conditionally saves 191.845825 s, projecting 376.154175 s | Audit working directories and shared outputs; move declarations or scheduling priority, preserve serial Cargo checks | All 52 lanes, argv, failures, timeouts, JSON streams, serial Cargo checks, and final clean-tree check | First bite: schedule executor and pure-shell capture checks alongside package tests, then remeasure |
| Less work: cooperative stand-in shutdown | Executor boundaries costs 140.183334 s; this is an upper bound on work available, not a measured shutdown saving; after the first bite its fixed-duration gate saving is zero | Give the fixture a bounded real stop handshake and test teardown failures | Keep all boundary/refusal assertions and deliberate lifetime/boot-failure cases | Separate fixture-only change with before/after lane walls |
| Less work: immutable conformance fixture reuse | Warm report tests cost 104.988560–112.686885 s each; exact shared construction cost and gate saving unavailable | Separate immutable setup from independent construction/evaluation; isolate mutations | Keep two independent determinism evaluations, corpus construction, every refusal, and admission checks | Instrument matrix setup first, then share only proven duplicate inputs |
| Less work: merge binaries that repeat common derivations | Vectors harnesses total 342.992673 s; common live-plan and corpus derivations already use per-process caches; duplicate-across-binary saving unavailable | Inventory fixture consumers and binary boundaries; merging can increase contention | Preserve every test, independent replays, isolation, and doctests | Measure initialization in each binary before proposing a merge |
| Less work: exhaust finite property orderings | Realization's boolean-order property took 20.405989 s debug, and the three-key ordering property 78.886289 s; two and six orderings respectively bound these ordering domains; no measured replacement saving, neither ends the warm package phase | Replace sampling only after proving the generated value affects ordering alone | Enumerate both boolean orders, all key orderings, and existing declaration permutations; keep the model's 1,000-trace property | Small realization-only exhaustive-order proposal with census and timings |
| Less work: admit an immutable corpus once | Vectors live-evidence tests cluster near 121 s, but admitted corpus and maturity fixtures are already cached; additional saving unavailable | Find uncached admission sites and distinguish setup from tested admission | Re-run negative admission, tampering, and mutation refusals independently | Instrument remaining admissions; do not add a redundant cache |
| Less work: release verification only for tags and releases | Warm isolated release costs 30.6 s; all release lanes finish before debug in the full run, so measured trailing-tail saving is 0 s and the fixed-duration wall remains 568.0 s | Change and enforce cadence/toolchain policy, plus release scheduling | Retain optimized-profile verification at tags/releases and explicitly account for its absence from merge proof | Defer cadence decision until contention and optimized-only failures are measured |
| Less work: avoid duplicate doctest compilation | Warm residuals are 3.540137 s vectors debug, 4.665991 s conformance debug, and 10.641773 s conformance release; these include more than doctests and are only cost ceilings | Separate compilation from execution without suppressing profile-specific coverage | Both profiles' doctest semantics, failure propagation, and per-test durations | Instrument rustdoc/lock intervals before designing deduplication |
| More parallel: Meson process count or longest-first order | Peak 37 lanes on 256 logical CPUs; all 32 package lanes already start together, so moving vectors first or raising a nonbinding limit has 0 s demonstrated saving | Record resource limits and tune with comparable repetitions | Same lanes, timeout handling, failure propagation, and output attribution | Measure a constrained-worker comparison only if a binding limit appears |
| More parallel: libtest thread tuning | Warm vectors 346.532810 s versus its 10,245.431219 s overlapping in-test sum indicates concurrency and waits; a faster thread setting is unmeasured | Controlled per-binary thread sweep with fixture initialization accounting | Exact executed and ignored census, independent fixtures, all assertions | Compare bounded thread counts on the heaviest binaries |
| More parallel: per-test process runner and partitioning | Longest individual event 151.083792 s; shared initialization near 121 s may recur in every process, so no saving established | Integrate process partitions, binary identity, doctests, JSON-equivalent timing, and failure handling | Everything `cargo test -p` proves, including doctests and ignored policy | Prototype one heavy binary before changing the gate runner |
| More parallel: split suites across nodes | Isolated profiles cost 325.6 s and 30.6 s; this node's full package phase is dominated by debug, and shared physical-host contention is unmeasured | Coordinate cache, artifacts, completion, and cross-node failure aggregation | Both profiles and all non-package checks complete before green | Defer: no measured independent capacity or two-node saving |

## Ranked recommendations · `sec:test-performance:recommendations`

First, overlap the independent checks with package tests while retaining the existing serial Cargo barriers. Holding measured durations fixed, moving the executor group removes 140.200734 s from the 568.0 s gate, projecting 427.799266 s; moving the capture group as well removes another 51.645091 s, projecting 376.154175 s (6:16.2). These are a scheduling model computed from Meson start/end intervals, not a promised observed wall. Additional CPU contention may lengthen the package phase. This is the recommended first bite because it preserves every test and has a measured exposed tail to remove.

Second, reduce repeated fixture construction only where instrumentation separates redundant setup from independent evidence. Prioritize the conformance report helper and executor teardown, while preserving determinism and refusal controls. The expected wall after this step is unavailable until that instrumentation and prototype exist; 376.154175 s remains the first step's conditional reference, with no additional saving credited. Exhausting realization's finite ordering properties can reduce work, but its lane is not currently critical. Lowering the model's trace count is unsupported.

Third, revisit release cadence and doctest compilation with a comparable warm series and optimized-only failure evidence. The current warm release tail is zero, so this record supports no additional fixed-duration wall reduction from dropping it. Keep both profiles until the policy and proof tradeoff are explicit. Cold release lock queueing does not establish a warm saving.

The prototype is the bounded scheduling change followed by the same measurement sequence. Acceptance requires preserved coverage and attribution, a lower comparable warm wall, and no new failure or skip. Reject a change that achieves speed by weakening any constraint above. A projection becomes a measured result only after that prototype supplies a before/after pair; overlapping costs cannot be subtracted twice.

## Measurement register, reproduction, and budgets · `sec:test-performance:tracking`

### Append-only observations · `tab:test-performance:register`

| Date | Node | Cores | Toolchain | Cache state | Gate wall | Debug-suite wall | Release-suite wall | Achieved parallelism | Top-3 lanes | Top-3 tests | Notes |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2026-09-22 10:13:34 UTC | orel | 128 physical / 256 logical, inventory at 10:49 | Rust 1.100.0-nightly, 2026-09-21; Cargo 1.100.0-nightly, 2026-09-16 | Cold; compilation observed; `cargo-target-gates` | 1,522 s enclosing; 1,518.5 s CI | Unavailable | Unavailable | 8.23 | release-model 931.86 s; release-execwrap 889.16 s; release-flatten-latex-main 881.28 s | conformance debug `a_declared_mock_run_cannot_satisfy_the_prototype_gate` 98.9 s; conformance debug `tests::constructor_tests::the_corpus_measures_no_retry_and_claims_nothing_about_the_tail` 84.7 s; vectors debug `shape_census::tests::every_cell_recomputes_its_verdict_against_the_live_registry` 80.1 s | `T11-110`; `gate-pending106.log`; 52 passed; loads unavailable; top tests rounded by the existing reader |
| 2026-09-22 10:49:10 UTC | orel | 128 physical / 256 logical | Rust 1.100.0-nightly, 2026-09-21; Cargo 1.100.0-nightly, 2026-09-16 | Cold full-test compilation; checkers and labels primed; `cargo-target-lanes` | 1,391.4 s | Unavailable for this cache state | Unavailable for this cache state | 7.742653 | release-labels 883.25 s; release-target-elements-conformance 879.68 s; release-execwrap 769.15 s | conformance debug `a_declared_mock_run_cannot_satisfy_the_prototype_gate` 99.6 s; conformance debug `tests::constructor_tests::the_corpus_measures_no_retry_and_claims_nothing_about_the_tail` 91.2 s; vectors debug `shape_census::tests::every_cell_recomputes_its_verdict_against_the_live_registry` 80.6 s | `T11-110`; measurement log opened 10:49:08 UTC, cold snapshot; 52 passed; loads in (`sec:test-performance:measurements`); top tests rounded by reader |
| 2026-09-22 11:12:23 UTC | orel | 128 physical / 256 logical | Rust 1.100.0-nightly, 2026-09-21; Cargo 1.100.0-nightly, 2026-09-16 | Warm; warm-up reuse observed; `cargo-target-lanes` | 568.0 s | 325.6 s, started 11:21:52 UTC | 30.6 s, started 11:27:19 UTC | 3.731857 | debug-vectors 346.532810 s; debug-target-elements-conformance 297.046484 s; debug-compiler 187.218314 s | conformance debug `tests::constructor_tests::the_corpus_measures_no_retry_and_claims_nothing_about_the_tail` 151.083792 s; model debug `tests::property_trace_tests::arbitrary_action_traces_preserve_invariant` 150.085849 s; vectors debug `live_evidence::tests::every_fresh_native_refusal_matches_its_rows_declared_boundary` 121.251727 s | `T11-110`; measurement log opened 10:49:08 UTC, warm snapshot; 52 passed, then 16 and 16; loads in (`sec:test-performance:measurements`); detailed attribution in (`tab:test-performance:tests`) and (`tab:test-performance:crates`) |

Append a dated row for each new full-gate measurement; do not replace an older row when performance changes. Cite the gate by date, node, toolchain, and `T11-110`, with its log as evidence. Separate suite runs belong to the full-run observation they follow. Record unknown historical metadata as unavailable. Cache classification follows actual compilation or reuse, not merely directory existence.

The command-wall denominator includes build preparation, tests, and the report reader. Achieved parallelism is the sum of Meson lane walls divided by that command wall; it measures overlapping lane residence, not CPU utilization. The report reader's in-suite column sums lane walls and is not suite elapsed time. In-test sums add the durations of completed individual tests and may exceed lane wall because tests overlap.

Run from the measured checkout on the compute node after sourcing `/workspace/toolchains/rust-env.sh`. `CI_BUILD_DIR` must be a relative path resolving into `/workspace/loops/tripod/tmp/`; `scripts/ci.sh` prefixes the checkout path when configuring publication outputs, so an absolute value is unsuitable. The path variables below use reusable descriptive names; the actual measurement's paths are retained in its execution log. The measured target is `/workspace/loops/tripod/cargo-target-lanes`. Do not clear it between the cold, warm, and isolated-suite runs, and do not treat an existing directory as proof of cache warmth.

```sh
export CARGO_TARGET_DIR=/workspace/loops/tripod/cargo-target-lanes
export TMPDIR=/workspace/loops/tripod/tmp
export CI_BUILD_DIR=../../tmp/test-performance-ci
export REPORT_DIR="$TMPDIR/test-performance-$(date -u +%Y%m%dT%H%M%SZ)"
mkdir "$REPORT_DIR"
rr() { time /workspace/run-report.sh nice -n 19 ionice -c3 "$@"; }
rr hostname
rr nproc
rr lscpu
rr rustc -Vv
rr cargo -V
rr ls -ld "$CARGO_TARGET_DIR" "$CARGO_TARGET_DIR/debug" "$CARGO_TARGET_DIR/release"
capture() {
  name=$1; shift
  date -u +%s.%N
  rr cat /proc/loadavg
  rr sh scripts/ci.sh "$@"
  result=$?
  rr cat /proc/loadavg
  date -u +%s.%N
  rr python3 scripts/ci-timing-report.py "$CI_BUILD_DIR"
  mkdir "$REPORT_DIR/$name"
  rr cp -a "$CI_BUILD_DIR/meson-logs" "$REPORT_DIR/$name/meson-logs"
  rr cp -a "$CI_BUILD_DIR/cargo-test-json" "$REPORT_DIR/$name/cargo-test-json"
  for file in "$CI_BUILD_DIR"/cargo-*.log "$CI_BUILD_DIR"/.ninja_log; do
    rr cp "$file" "$REPORT_DIR/$name/"
  done
  echo "run=$name exit=$result"
}
capture cold
capture warm
capture debug --suite cargo-debug
capture release --suite cargo-release
```

Capture the enclosing stdout and stderr whole. Each `RUN-REPORT` line supplies the command wall and exit status; the load samples immediately around the CI command supply the before/after one-, five-, and fifteen-minute averages. The cold snapshot preserves first-use compilation, and each later snapshot prevents the next `meson test` from overwriting the previous run's streams. Select the latest completed historical log from `ls -lt /workspace/loops/tripod/gate-pending*.log` by its completion footer, then read it whole and retain the producing `rebuild-gate.sh` command. A source-count cross-check is `rg -o '#\[test\]' packages/CRATE -g '*.rs' | wc -l`; source attributes are not the executed-test census.

The repository reader reports only ten global slow tests and does not preserve binary names or print executed counts. The top-forty extraction therefore reads every warm stream, increments a binary ordinal at each suite-start event, retains completed test events with numeric `exec_time`, and sorts durations numerically to six decimal places. The original stream retains any finer precision. The measurement used the following field extraction on the standard libtest event-line layout; non-test command output is excluded by the `TEST` and `TOTAL` prefixes.

```sh
awk '
  FNR==1 {suite=0; n=split(FILENAME, parts, "/"); package=parts[n]; sub(/\.json$/, "", package); profile=parts[n-1]; key=profile "\t" package}
  /"type": "suite", "event": "started"/ {suite++}
  /"type": "test"/ && !/"event": "started"/ {
    outcome=$0; sub(/^.*"event": "/, "", outcome); sub(/".*$/, "", outcome);
    test=$0; sub(/^.*"name": "/, "", test); sub(/".*$/, "", test);
    if (outcome=="ignored") {ignored[key]++; next}
    if (outcome!="ok" && outcome!="failed") {notifications[key]++; next}
    counts[key]++;
    if ($0 !~ /"exec_time":/) {missing[key]++; next}
    seconds=$0; sub(/^.*"exec_time": /, "", seconds); sub(/[, }].*$/, "", seconds);
    sums[key]+=seconds; timed[key]++;
    printf "TEST\t%.6f\t%s\t%s\t%d\t%s\t%s\n", seconds, profile, package, suite, outcome, test
  }
  END {for (key in counts) printf "TOTAL\t%s\t%d\t%d\t%d\t%d\t%.6f\n", key, counts[key], ignored[key], timed[key], missing[key], sums[key]}
' "$REPORT_DIR/warm/cargo-test-json"/*/*.json > "$REPORT_DIR/warm-attribution.tsv"
awk -F '\t' '$1=="TEST"' "$REPORT_DIR/warm-attribution.tsv" | sort -t "$(printf '\t')" -k2,2nr | head -40
awk -F '\t' '$1=="TOTAL"' "$REPORT_DIR/warm-attribution.tsv" | sort
```

Map suite ordinals to Cargo's ordered `Running` and `Doc-tests` headings in the same lane's `meson-logs/testlog.txt`, and confirm the ranked test names in the named source file. An event stream alone cannot recover the binary identity. Reconcile completion counts with the suite summaries and disclose ignored tests and events without duration; doctests remain part of the census. Exclude libtest's informational long-running-test notifications: counting every event other than `started` would count a long test twice. The sums use durations to six decimal places, with display rounding stated at the tables. If the stream format changes, validate the extractor before appending a row rather than silently accepting an empty ranking.

For lane walls and scheduling attribution, the measured extraction below reads Meson's one-record-per-line JSON. Run it for each snapshot, retaining its output. Sum the `LANE` durations and divide by that run's enclosing `RUN-REPORT` wall; the historical denominator is instead its explicitly identified completion footer. The scheduling model subtracts the interval from documentation end to attestation start and the interval from attestation end to mock-contract start. It assumes every independent check in those intervals moves, including the advisory and short stamp checks, while the serial checks keep their order and clean-tree remains last.

```sh
rr awk '
  function number(key, line) {sub("^.*\"" key "\"[[:space:]]*:[[:space:]]*", "", line); sub(/[,}].*$/, "", line); return line+0}
  {name=$0; sub(/^.*"name": "/, "", name); sub(/".*$/, "", name); sub(/^.*:/, "", name);
   duration=number("duration", $0); start=number("starttime", $0);
   printf "LANE\t%s\t%.6f\t%.6f\t%.6f\n", name, duration, start, start+duration;
   sum+=duration}
  END {printf "SUM\t%.6f\n", sum}
' "$REPORT_DIR/warm/meson-logs/testlog.json"
rr awk '$4 ~ /^cargo-test-warmup-(debug|release)\.ok$/ {elapsed[$4]=($2-$1)/1000} END {for (name in elapsed) printf "%s\t%.3f\n", name, elapsed[name]}' "$REPORT_DIR/warm/.ninja_log"
rr awk '/^test:/ {package=$0; ordinal=0} /Running (unittests|tests\/)|Doc-tests / {ordinal++; print package, ordinal, $0}' "$REPORT_DIR/warm/meson-logs/testlog.txt"
```

For the compilation residual, sum `exec_time` only from successful `type: suite` events within each package stream and subtract that serial harness total from its Meson wall. Do not subtract the overlapping individual-test sum. Retain Cargo's `Finished`, `Compiling`, and lock-notice lines alongside this calculation. The dated observations and source causes in this record are supported by the whole measurement log opened at 10:49:08 UTC and the reconciled attribution log opened at 11:29:55 UTC on 2026-09-22, under `/workspace/loops/tripod/tmp/`; their cold, warm, debug, and release snapshots remain separate.

The existing Meson comments state the suite budgets in minutes and seconds: warm debug 5:00 with tolerance 8:00; warm release 10:30 with tolerance 15:00; cold debug 15:45 and cold release 30:30, each with tolerance 1.5 times its budget. These are historical budgets on a 14-core reference host, read from `meson.build`, not measurements from this experiment. They remain unchanged. Judge isolated suite command walls against these units while disclosing the different node and build preparation; do not compare summed lane time with a wall budget.

## Residual uncertainty · `sec:test-performance:residuals`

One cold/warm pair does not estimate run-to-run variance. The gate does not measure sibling-node resource use, and load average cannot assign interference to a particular process. Warm-up build logs and Cargo lock messages expose compilation and waiting, but ordinary libtest events do not separate CPU execution, fixture waits, scheduler delays, and doctest compilation. The mocked gate does not prove real document rendering or byte reproducibility. The observed inventory is 128 physical cores and 256 logical CPUs; treating either as a count of independently available workers would ignore the shared host and container limits.

All requested fresh runs completed. Historical isolated suites and loads are unavailable because their log does not record them. No logs from the other nodes were accessible. The existing reader prints ten longest tests and aggregates suites; the forty-row and executed-count views required the explicit event extraction above. Informational `timeout` events announce long-running tests and are not failures or extra completions: the initial extraction was reconciled against suite summaries and corrected before recording counts. Every completed warm test had a duration. The source attribute census matched the supplied counts, but expansion, conditional compilation, ignored tests, and doctests make those counts different from execution.

The observed warm executor-boundaries wall of 140.183334 s exceeds its Meson comment's 0:05 budget and 0:20 tolerance. Its current source has many more signing cases than the comment's twelve-process description. The per-package Meson list contains sixteen members while an adjacent historical budget comment still refers to thirteen packages. These comments need a later measured update; this record changes neither comment nor budget. Warm release walls are much smaller than the earlier cold observations, so a release-cadence decision based only on those cold walls would misattribute compilation and lock queueing to test execution.

**Result:** Measurement complete; Pending scheduling prototype. **Decision handoff:** Select the first supported bite in (`sec:test-performance:recommendations`), preserve (`sec:test-performance:constraints`), append a comparable row to (`tab:test-performance:register`), and land the bite before closing `T11-110`.
