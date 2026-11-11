# Linker tracing design

## Purpose and shape · `sec:tracing-linker:purpose`

`tripod-linker` is a library that turns typed, relocatable backend bundles into deterministic candidate links; it does not finalize a transaction or discharge the candidate's outstanding obligations. Its existing attestation entry points are `link_candidate` in `src/bundle.rs` for compact ASH, `link_live_candidate` in `src/live_bundle.rs` for live transfer, and `link_state_candidate` in `src/state_bundle.rs` for maturity announcements.

`src/lib.rs` exports the compact, live and state modules and their public types. Each flow resolves symbols, checks references and relocations, assembles a committed tree, closes relation carriers and projects resources; `src/error.rs` holds the common `LinkRefusal`, while `src/state_error.rs` holds the more detailed `StateLinkRefusal` nested in it. The live flow links an offered slice of bundles by owner and representation; the state flow additionally reapplies a constructor over linked bytes, checks an authenticated graph and measures linked resources. The crate has no binary; `transaction` and `vectors` are its principal workspace callers, including the latter's negative boundary cases.

`link_keyed_bundle` in `src/keyed_bundle.rs` adds the keyed boundary: it checks a recipe equality assertion and target-version offer and a complete instruction/reference census, retains sealed emission and encodes unchanged bytes. It neither commits a tree nor authenticates a predecessor. Its offer helper and inner checks are untraced; the outer boundary owns exactly one DEBUG completion event on every return.

## Output inventory · `sec:tracing-linker:inventory`

The inventory is the design-time baseline from before linker instrumentation was added; it covered every Rust source and test file beneath `packages/linker/src/`, and the absence of a match had no file or line to cite. Collection lengths and resource totals in the implementation are domain validation and result values, not observation counters.

| Existing item | Location | Class | Count |
|---|---|---|---:|
| `println!`, `print!`, direct `stdout()` or other standard-output write | No matches in `packages/linker/src/**/*.rs` | structured result | 0 |
| `eprintln!`, `eprint!`, direct `stderr()` or other standard-error write | No matches in `packages/linker/src/**/*.rs` | diagnostic | 0 |
| Printed or stored progress instrumentation | No matches in `packages/linker/src/**/*.rs` | progress | 0 |
| `Instant`, `SystemTime`, `Duration`, `elapsed()` or other local timing instrumentation | No matches in `packages/linker/src/**/*.rs` | timing | 0 |
| Bespoke observation counter or atomic tally | No matches in `packages/linker/src/**/*.rs` | counter | 0 |
| Existing `tracing` span or event | No matches in `packages/linker/src/**/*.rs`; dependency declared in `packages/linker/Cargo.toml:38` | diagnostic, progress, timing, counter | 0 |

## Standard output and standard error · `sec:tracing-linker:streams`

Linker returns typed `CandidateLinkedBundle`, `CandidateLinkedLiveTransferBundle` and `CandidateLinkedMaturityBundle` values or typed refusals; it publishes no standard-output result, writes no standard-error record and installs no subscriber. Candidate status and outstanding obligations remain returned values. An executable caller owns any documented JSON object plus newline or NDJSON result on stdout, while assets go only to explicit paths; failure before publication leaves stdout empty. ADR-010 requires all first-party standard-error records to come from one tracing subscriber and prohibits raw child-byte relay on both streams; `execwrap` writes child bytes only to explicit file routes (`[ADR010-rule:output:streams]`, `[ADR010-rule:output:json]`).

The first statement of an executable using linker is `cli_common::install_command_subscriber(command_name)`, which installs one JSON subscriber and the panic hook before parsing. Its unfiltered `cli_common::control` route keeps help, version, usage, TTY refusal and panic in the existing ADR-010 schema through a shared locked writer; its reloadable diagnostic route writes other tracing records. Before parsing, nonempty `RUST_LOG` selects the diagnostic filter, otherwise `info`; after parsing, the runner applies nonempty `RUST_LOG`, then `--debug`, then the command default. An invalid directive falls back to `info` with a warning that omits its value, and `RUST_LOG=off` cannot suppress control records. The panic route attempts at most one record through a nonblocking lock, omits the payload before parsed debug and exits 1. The command runner owns the parent span and sole terminal command event; a linker refusal stays a DEBUG observation rather than a second user-facing diagnostic (`[ADR010-rule:output:control-plane]`, `[ADR010-rule:output:early-startup]`, `[ADR010-rule:output:diagnostics]`).

The reload handle stays in a process-global cell inside `cli-common`, so a binary holds none. After parsing, its INFO `cli_common::command` span carries the static `command` and `output_mode`; success ends with one INFO `command completed` event carrying `exit_class = "success"` and `result_published`, while execution or publication failure ends with one ERROR `command failed` event and the shared redacted diagnostic. Usage and TTY refusal end with their control record alone; help, version and status control records are INFO, usage and TTY refusal WARN, and panic ERROR. There is no parse span, and an async runner instruments its future rather than holding a span guard across an await.

The subscriber's diagnostic JSON keeps timestamp, level, target, fields and span list, and emits one close record for each enabled span with `busy_ns: u64` and `idle_ns: u64` by default. Linker does not configure it. ADR-010 assigns the shared bootstrap and stream contract to `cli-common`; linker supplies library spans and events without installing a subscriber, and executable callers own their command integration. This preserves ADR-010's result/diagnostic split and gives ADR-026's spans observable duration without a local timer (`[ADR026-dec:tracing:core]`).

## Spans · `sec:tracing-linker:spans`

The file and line locations in this table are design-time insertion points recorded before instrumentation; span names, targets and fields describe the library contract.

| Operation and insertion point | Span name | Explicit target | Level | Entry fields and types | Duration value |
|---|---|---|---|---|---|
| `link_candidate` (`src/bundle.rs:464`) | `link_candidate` | `linker::bundle` | INFO | `family: &'static str = "compact"`, `leaf_count: u64` | Whole compact link, including candidate assembly. |
| `link_live_candidate` (`src/live_bundle.rs:677`) | `link_live_candidate` | `linker::live_bundle` | INFO | `family: &'static str = "live"`, `bundle_count: u64` | Whole offered-bundle accumulation and candidate assembly. |
| `link_keyed_bundle` (`src/keyed_bundle.rs`) | `link_keyed_bundle` | `linker::keyed_bundle` | DEBUG | `family: &'static str = "keyed"`, `offer_count: u64` | Checks constructor offers, typed occurrences and retained reference closure, then encodes unchanged material. |
| `link_one_bundle` (`src/live_bundle.rs:868`), once per offered bundle | `link_live_bundle` | `linker::live_bundle` | DEBUG | `family: &'static str = "live"`, `leaf_count: u64`, `representation: &'static str` | Locates a slow representation without logging owner material. |
| `link_state_candidate` (`src/state_bundle.rs:882`) | `link_state_candidate` | `linker::state_bundle` | INFO | `family: &'static str = "state"` | Whole state link, including the tapscript constructor application over linked bytes. |
| `resolve_references` / `state_graph_from_sources` (`src/graph.rs:281`, `src/state_graph.rs:494`) | `resolve_graph` | `linker::graph` / `linker::state_graph` | DEBUG | `family: &'static str`, `edge_count: u64` when known at entry | Separates graph, cycle and authenticated-cut work from relocation. |
| `substitute` / `substitute_live` / `substitute_state` (`src/relocate.rs:99`, `src/live_relocate.rs:112`, `src/state_relocate.rs:488`) | `substitute_relocations` | `linker::relocate` / `linker::live_relocate` / `linker::state_relocate` | DEBUG | `family: &'static str`, `site_count: u64` when known at entry | Measures rebuild, site checks and round-trip validation over program bytes. |
| `assemble` / `assemble_live` / `assemble_state_static` (`src/taptree.rs:503`, `src/live_taptree.rs:144`, `src/state_taptree.rs:204`) | `assemble_taptree` | `linker::taptree` / `linker::live_taptree` / `linker::state_taptree` | DEBUG | `family: &'static str`, `leaf_count: u64` | Measures tree optimization and control-path construction. |
| `close` / `close_live` / `close_state_carriers` (`src/carrier.rs:87`, `src/live_carrier.rs:282`, `src/state_carrier.rs:440`) | `close_carriers` | `linker::carrier` / `linker::live_carrier` / `linker::state_carrier` | DEBUG | `family: &'static str`, `relation_count: u64` when known at entry | Isolates relation and linked-byte comparisons. |
| `measure_state_resources` (`src/state_resource.rs:716`) | `measure_state_resources` | `linker::state_resource` | DEBUG | `family: &'static str = "state"` | Isolates linked-program measurement and admission checks. |

The outer span remains the caller of its phase spans; when vectors builds a fixture, its `vectors::bundle` parent encloses the linker span, and calls into tapscript keep their own child spans. Close durations are inclusive, so parent and child durations are never summed as independent work. Spans hold entry context only: no outcome, completed count, candidate value, raw symbol or key is recorded on a span. A count unavailable without doing the measured work is omitted from the entry span and, if needed, read from the returned candidate's outer completion event.

## Events · `sec:tracing-linker:events`

| Public boundary | Target | Level | Fixed message | Fields and types |
|---|---|---|---|---|
| `link_candidate`, `link_live_candidate` or `link_state_candidate` returns a candidate | Matching `linker::bundle`, `linker::live_bundle` or `linker::state_bundle` | INFO | `link completed` | `outcome: &'static str = "completed"`, `family: &'static str`, `leaf_count: u64`, `outstanding_obligation_count: u64` |
| One of those entry points returns an expected typed refusal | Same target as its outer span | DEBUG | `link refused` | `outcome: &'static str = "refused"`, `family: &'static str`, `refusal_kind: &'static str` |
| `link_keyed_bundle` returns success | `linker::keyed_bundle` | DEBUG | `keyed link completed` | `family = "keyed"`, `outcome = "completed"` (static strings); `constructor_count`, `literal_count`, `reference_count`, `relocation_count`, `leaf_count`, `outstanding_obligation_count` (all `u64`) |
| `link_keyed_bundle` returns a typed refusal | `linker::keyed_bundle` | DEBUG | `keyed link refused` | `family = "keyed"`, `outcome = "refused"`, `refusal_kind` (static strings) |
| A future entry point returns an operational fault rather than a typed link refusal | Same target as its outer span | Its span's tier | `link failed` | `outcome: &'static str = "failed"`, `family: &'static str`, `error_kind: &'static str` |

Each candidate entry point emits exactly one outer completion event while its span is open; none of the symbol, graph, relocation, tree, carrier or resource phases emits a second count event. `leaf_count` is the linked leaf total, and `outstanding_obligation_count` counts what the candidate still owes, never discharged work. A refusal uses the innermost `LinkRefusal`, `StateLinkRefusal` or `KeyedLinkRefusal` variant's fixed payload-free name, not `Display` or `Debug` output; no current linker path needs the reserved operational-fault row. Linker returns its typed result unchanged, and an executable caller's `cli-common` runner emits the one ERROR terminal record if that return fails the command.

## Naming · `sec:tracing-linker:naming`

Every span and event has an explicit literal `linker::<module>` target, never a binary crate target; names are unprefixed lower snake case and messages are fixed short English phrases. Static fields use lower_snake_case keys and closed `&'static str` values: `family` is `compact`, `live`, `state` or `keyed`; `outcome` is `completed`, `refused` or `failed`; `representation` is a static live representation label; `refusal_kind` is the snake_case name of the innermost typed refusal variant, including underscores around single-letter words, and `error_kind` names an operational fault. There is no generic `operation` field because the enclosing span names it, and no `standing` field because linker constructs candidates rather than observing target acceptance.

Counts such as `leaf_count` and `outstanding_obligation_count` are `u64` at emission, converted from a `usize` length with `u64::try_from(n).unwrap_or(u64::MAX)` so instrumentation cannot fail a link; byte totals, if later needed, use a subject-first `_bytes` suffix. A configured limit uses `_limit: u64` and a configured time bound may use `_ms: u64`; measured durations remain subscriber close data. A future operational fault may use `phase: &'static str` for its fixed failing step when the span name does not already supply it. `operation_code: u16` is used only where one architecture operation is in scope, and current linker spans need no such field. No event or span contains amounts, nonces, report contents, program bytes, witnesses, keys, raw symbols, caller-supplied paths, child output, error text or a correlation ID; parentage supplies correlation.

The keyed outer span and its completed event are DEBUG; the other outer candidate spans and completed events are INFO, phase spans are DEBUG, and a typed refusal event is DEBUG whatever its span tier. WARN means a recoverable anomaly after which the command still produces a result; ERROR belongs to the command's terminal diagnostic or a broken invariant the returned value cannot convey. Linker currently has neither kind of local exceptional event.

## Cost · `sec:tracing-linker:cost`

Entry fields read already-held lengths and static labels, and count conversion is cheap; no count traverses a map solely for tracing. Do not clone bytes, render a refusal, construct a string, hash a candidate or time a phase merely to populate a disabled field. If a later field needs derivation, compute it only under `tracing::enabled!` at its target and level or through lazy formatting; use a static variant-name match for `refusal_kind`. The diagnostic route emits close records only for enabled spans, and without a subscriber instrumentation pays the callsite check, not a local stopwatch or separate counter store.

## Testing · `sec:tracing-linker:testing`

Package-local recording tests use a workspace-inherited `tracing-subscriber` dev-dependency and `tracing::subscriber::with_default` around one call each to the compact, live and state entry points. They hold one process-wide mutex through each scoped capture and rebuild the callsite interest cache inside `with_default` before the subject runs. Each capture test re-executes as the sole filtered test in a child test process, with its baseline and scoped capture both inside that process, so unrelated parallel test threads cannot change callsite interest during the capture; the parent captures child output in memory and reports it only on failure. A regression first touches a traced entry without capture, then captures it from two threads and requires exactly one outer span in each. For each entry point, tests pair an accepted control with a one-coordinate subject refused at the declared linker boundary; they assert the returned candidate or exact refusal with and without capture, one outer event per call, `family`, `outcome`, `refusal_kind`, typed `u64` counts on success, and no WARN or ERROR record for the expected refusal. They capture target, level, span name, message, fields and one close record per enabled span in memory; they do not assert duration magnitudes or ordering across threads and never install a global subscriber. If a future linker path spawns a thread, its dispatcher must be carried explicitly with `tracing::dispatcher::get_default` and the parent span entered in that thread.

Linker tests write no standard stream and do not run a process-global subscriber. Subprocess assertions for the early control route, `RUST_LOG=off`, panic, span close records and stdout JSON belong to the executable owner and `cli-common`, whose own tests re-execute their test binary in a fresh process. The library's instrumentation must leave linked bytes, candidate status, outstanding obligations and refusal variants identical when collection is absent or present.

## Implementation plan · `sec:tracing-linker:plan`

1. Commit the three outer entry spans, static `family` and refusal-kind mappings, and exactly one completion/refusal event per candidate call; add scoped capture tests for compact, live and state controls and one-coordinate refusals. This library step uses the core tracing dependency and a local test-only subscriber dependency.
2. Commit compact and live child phase spans for graph, relocation, tree and carrier work, including the once-per-offered-bundle live span; assert parentage and unchanged candidate bytes in local tests. This is a library-only step.
3. Commit state child phase spans for relocation, authenticated graph, tree, resource and carrier work; the outer state span includes the tapscript constructor application, and local tests assert parentage and unchanged results. No stage count events or local timers are added. This completes linker's subscriber-free library instrumentation.

ADR-010 and the `cli-common` subscriber govern executable callers. Those callers own command-span integration and subprocess stream assertions for linker calls; linker has no binary or compatibility delegate.

## Open questions for the other packages · `sec:tracing-linker:questions`

- The cross-package family field is `family` with `compact`, `live`, `state` and `keyed`; it replaces linker's former `flow` and agrees with compiler, tapscript, transaction and vectors.
- Expected typed refusals are DEBUG with `outcome = "refused"` and `refusal_kind`; the command boundary alone emits the user-facing ERROR if the command fails.
- The shared candidate count is `outstanding_obligation_count: u64`, alongside `leaf_count: u64`; neither claims an outstanding obligation has been discharged.
- Enabled spans emit one default close record with `busy_ns` and `idle_ns`; nested durations are inclusive and no event or sidecar is created for linker's timing.
- The `cli-common` bootstrap under ADR-010 owns the early always-enabled control route, filter reload, locked writer and panic-safe dispatch; linker remains a subscriber-free library.

The keyed capture `keyed_link_has_one_completion_per_return` runs alone in a child test process, compares returned values with uninstrumented calls, and requires exactly one child test to run and checks the outer span, DEBUG event, family/outcome, success counts and refusal classification. The keyed relocation count is the constant zero justified by accepting the entire fixed instruction grammar, not the length of a relocation list. It captures the whole call so a nested fragment event cannot hide behind a target filter. No key, literal, program or local timer is recorded.
