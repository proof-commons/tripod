# `execwrap` tracing engineering design

## Purpose and shape · `sec:tracing-execwrap:purpose`

`execwrap` runs a caller-selected child and captures its two pipes for file routing. `src/lib.rs` owns routing identity preflight, child lifetime, reader threads, log notifications, byte movement, and deterministic mock TeX output; `src/writer.rs` owns byte-exact raw writing and line-oriented prefixed presentation. `src/bin/execwrap.rs` is the production CLI, and `src/bin/execwrap-mock-tex.rs` is a separate first-party mock command used by the Meson document graph. Unit and subprocess contract tests live in `src/tests/mod.rs` and `tests/subprocess_contract.rs`.

The public library surface includes `RoutingConfig`, `Stream`, `ExecError`, `ExecOutcome`, `Notification`, `preflight_routing`, `safe_open`, `run`, `MockChild`, `run_mock_child`, `Writer`, and `StreamProps`. A nonzero child status is returned in `ExecOutcome`, not treated as a setup error; `data_loss` reports a failed read, log write, reader join, or finalization even if the child exited zero. The binary retains the ADR-010 mapping: child 0 returns success 0, child 1 or 2 returns wrapper failure 1, and child statuses 3–255 are relayed unchanged (`[ADR010-rule:output:exit-codes]`).

Attestation and Lineage Meson callers supply `--redirect` to route both streams to explicit logs (`papers/attestation/meson.build`, `papers/lineage/meson.build`). The library's README example is a caller example, not a shipped command. `cli-common` owns command startup, the JSON subscriber, panic and control records, command spans, terminal records, and exit classes; neither this library nor its `Writer` installs a subscriber.

## Output inventory · `sec:tracing-execwrap:inventory`

This table inventories the design-time baseline before the implementation commits recorded below; source locations refer to that former code. Raw child-byte passthrough was an ADR-010 exception withdrawn for both streams; its presence here describes the former implementation. No current executable source calls `println!`, `eprintln!`, `print!`, or `eprint!`.

| Baseline location | Baseline output or observation | Class |
|---|---|---|
| `src/lib.rs:479,492-495,560-566` | Direct `io::stdout()` handle, raw child stdout passthrough, and parent flush. | structured result |
| `src/lib.rs:480,492-495,560-566` | Direct `io::stderr()` handle, raw child stderr passthrough, and parent flush. | structured result |
| `src/lib.rs:464-468` | `tracing::info!` child start with argument count and PID. | progress |
| `src/lib.rs:502-506` | `tracing::error!` parent passthrough write failure. | diagnostic |
| `src/lib.rs:537` | `tracing::error!` log writer failure. | diagnostic |
| `src/lib.rs:543` | `tracing::debug!` child stream closed. | progress |
| `src/lib.rs:547` | `tracing::error!` child stream read failure. | diagnostic |
| `src/lib.rs:556` | `tracing::error!` reader thread panic. | diagnostic |
| `src/lib.rs:566` | `tracing::error!` parent stream flush failure. | diagnostic |
| `src/lib.rs:573` | `tracing::error!` writer finalization failure. | diagnostic |
| `src/lib.rs:591` | `tracing::debug!` child exit status and data-loss outcome. | progress |
| `src/writer.rs:81` | `tracing::debug!` byte length of each raw write. | counter |
| `src/writer.rs:112` | `tracing::debug!` complete prefixed child line text, which can reveal child data. | diagnostic |
| `src/bin/execwrap.rs:71` | Shared JSON panic hook, then a direct `cli-common` stderr writer. | diagnostic |
| `src/bin/execwrap.rs:77-81,102-106` | Shared JSON help, version, and usage records, including ambiguous redirection; two direct control-record call sites. | diagnostic |
| `src/bin/execwrap.rs:112-116` | `tracing::error!` working-directory change failure with argument-derived directory. | diagnostic |
| `src/bin/execwrap.rs:125` | `tracing::error!` spawn failure without program path. | diagnostic |
| `src/bin/execwrap.rs:129` | `tracing::error!` other setup failure, potentially with free-form error text. | diagnostic |
| `src/bin/execwrap.rs:137` | `tracing::error!` captured-output loss after a successful child. | diagnostic |
| `src/bin/execwrap.rs:169-175` | `tracing::info!` immediate `WritingToLog` notice with path and stream. | progress |
| `src/bin/execwrap.rs:178-183` | `tracing::info!` deferred `WroteToLog` notice with path. | progress |
| `src/bin/execwrap-mock-tex.rs:51` | Shared JSON panic hook, then a direct `cli-common` stderr writer. | diagnostic |
| `src/bin/execwrap-mock-tex.rs:53-57` | Shared JSON help, version, and usage record through a direct control-record call. | diagnostic |
| `src/bin/execwrap-mock-tex.rs:67` | `tracing::error!` injected mock failure. | diagnostic |
| `src/bin/execwrap-mock-tex.rs:74` | `tracing::error!` mock output write failure. | diagnostic |
| `src/lib.rs:685-690` | Compare-if-changed mock files under caller-supplied output directory. | structured result |
| `README.md:84,87` | Two `println!` notification examples in a non-running library sample. | progress |
| `README.md:95` | One `println!` child-status example in that sample. | structured result |

`Writer::new` retains the public `_debug: bool` argument only for source compatibility; it has no effect after removal of per-write debug events.

`Notification::WroteToLog` remains public only for source compatibility; fully routed runs cannot emit it because the deferred output-only notification path is absent.

There is no package-local `Instant`, elapsed-time recorder, custom telemetry counter, or other timing sink. In the baseline, `first_write_stdout`, `first_write_stderr`, and `pending_paths` at `src/lib.rs:439-449,482,509-533,577-579` implemented notification semantics, not instrumentation; the unreachable deferred path is now removed. Baseline file writes at `src/writer.rs:83,115,149,152` and `src/lib.rs:690` produced assets rather than process-stream diagnostics, as file routing still does.

## Standard output and standard error · `sec:tracing-execwrap:streams`

The command is a side-effect command: child stdout and stderr bytes go only to explicit file routes (`--redirect`, `--redirect-output`, `--redirect-error`, or `--redirect-prefixed`), and wrapper stdout stays empty. `routing_preflight` refuses a configuration if either child stream has no file subscriber, using typed `unrouted_stream`; the binary emits an ADR-010 usage control record and exits 2 before spawning or opening output. Two routes resolving to one file still yield `ambiguous_redirection` before a side effect. Raw file routing remains byte-exact; prefixed routing remains a presentation asset. The child-status relay for 3–255 stays intact. This implements the stream contract (`[ADR010-rule:output:streams]`) and explicit-asset rule (`[ADR010-rule:output:assets]`); all current Meson callers provide a route for both streams.

The mock helper emits no stdout and writes its declared files under `--outdir`; its injected failure leaves those outputs unwritten. It is a first-party test-subject executable and follows the same stderr rule as the production wrapper. A library caller retains typed `ExecOutcome` and `Notification` values; no tracing event substitutes for a returned value or file asset. The README sample collects typed notifications and results without writing to a process stream.

Both `main` functions begin with `cli_common::install_command_subscriber(command_name)`, which installs the sole process subscriber and panic hook before clap parsing. Its unfiltered `cli_common::control` route renders help, version, usage, TTY refusal, and panic with the existing ADR-010 control JSON schema; its reloadable diagnostic route renders normal tracing JSON through the same non-interleaving locked stderr writer. Before parsing, diagnostics use nonempty `RUST_LOG` or `info`; after parsing, the shared reload applies nonempty `RUST_LOG`, then `--debug`, then the command default, which is WARN for production `--quiet` and INFO otherwise. An invalid `RUST_LOG` falls back to INFO and emits one WARN record without echoing the directive. Control records survive `RUST_LOG=off`, and a panic yields at most one best-effort control record through a nonblocking lock, with payload hidden until parsed `--debug`. The binaries hold no reload handle; the shared process cell owns it. The library and `Writer` never install or configure a subscriber.

The shared `cli_common::command` INFO span encloses each custom main's executable post-preflight work and has static `command` and `output_mode = "asset"` entry fields. Each invocation ends with one INFO `command completed` or ERROR `command failed` terminal event; usage refusals end with their WARN control record alone. A child status from 3 through 255, including a signal-derived status, emits `command completed` with `exit_class = "child_status"` and `child_exit_code: i32` while the process relays that status; it is not a wrapper failure class (`[ADR010-rule:output:exit-codes]`). Custom-main failures pass only a fixed kind (`spawn`, `change_directory`, `data_loss`, `child_exit`, `injected_failure`, or `io`) to the shared helper's `error` field, with no free-form text that might name a caller-supplied path. `exit_class`, `result_published`, and `output_mode` remain shared command fields, not fields copied into `execwrap` library events. Diagnostic JSON has the subscriber envelope with target, level, fields and span context; control JSON retains its distinct `schema`, `command`, `kind`, and `message` fields (`[ADR010-rule:output:control-plane]`).

ADR-010 requires subscriber-produced stderr and only structured JSON results or nothing on stdout; early records omit argv, environment values, and panic payloads, and the 0/1/2 exit classes remain. `cli-common` provides the early subscriber and unfiltered control route used by both binaries. Neither binary has a fallback direct stderr writer; subprocess tests assert the early records. An ordinary test harness remains outside the executable output contract, while both `execwrap` binaries are inside it.

## Spans · `sec:tracing-execwrap:spans`

| Operation | Span name | Literal target | Level | Entry fields (types) | Duration reason |
|---|---|---|---|---|---|
| Complete `run`, beneath the shared command span when called by the binary | `child_run` | `execwrap::run` | INFO | `argument_count: u64`, `route_count: u64` | Measures routing preparation, child lifetime, pipe draining, and writer finalization as one operation. |
| Public route identity and missing-route preflight | `routing_preflight` | `execwrap::routing` | DEBUG | `route_count: u64` | Separates filesystem identity work and usage rejection from execution. |
| One reader thread from its first read through EOF or error | `child_stream_read` | `execwrap::stream` | DEBUG | `stream: &'static str` | Shows each producer's read and drain time without one span per chunk. |
| Finalize a raw or prefixed writer | `writer_finalize` | `execwrap::writer` | DEBUG | `mode: &'static str` | Isolates trailing-line presentation and flush cost. |
| Generate and compare-if-changed mock outputs | `mock_child_run` | `execwrap::mock` | INFO | `child_kind: &'static str`, `output_count: u64` | Exposes mock filesystem work separately from the production child. |

Spans carry only entry context; `pid`, outcomes, counts learned during execution, and error classes belong to events. Reader threads capture the current `tracing::Dispatch` and parent span, then enter both in the spawned thread so scoped collectors and production span parentage agree. The subscriber emits one close record per enabled span by default, containing its name, target, level, entry fields, `busy_ns: u64`, and `idle_ns: u64`; there are no open or enter records. Nested durations are inclusive and not summed as independent work; a collector can derive exclusive time from parentage. No event carries measured wall time, and this package adds no telemetry `Instant` or correlation ID (`[ADR026-dec:tracing:core]`).

## Events · `sec:tracing-execwrap:events`

| Trigger | Literal target | Level | Fixed message | Fields (types) |
|---|---|---|---|---|
| Public preflight returns | `execwrap::routing` | DEBUG | `routing preflight completed` | `outcome: &'static str`, `route_count: u64`, `refusal_kind: &'static str` exactly on refusal (`ambiguous_redirection` or `unrouted_stream`), `error_kind: &'static str` exactly on operational failure. |
| Child spawned successfully | `execwrap::run` | INFO | `child started` | `argument_count: u64`, `pid: u32`. |
| One child's stream is fully accounted for after read, file routing, and finalization | `execwrap::stream` | DEBUG | `child stream completed` | `stream: &'static str`, `outcome: &'static str`, `read_bytes: u64`, `written_bytes: u64`, `error_kind: &'static str` exactly if that stream failed. |
| `run` returns a typed result or error | `execwrap::run` | INFO on `completed` or `failed`, DEBUG on `refused` | `child completed` | `outcome: &'static str`, `child_exit_code: i32` only on normal child exit, `data_loss: bool` when a child ran, `refusal_kind: &'static str` exactly on typed refusal, `error_kind: &'static str` exactly on operational failure. |
| First write to a routed file, only with `notify_on_write` | `execwrap::routing` | INFO | `log write started` | `route_kind: &'static str`, `stream: &'static str`; no path. |
| Writer finalization returns | `execwrap::writer` | DEBUG | `writer finalized` | `outcome: &'static str`, `error_kind: &'static str` exactly on operational failure. |
| Mock generation returns | `execwrap::mock` | INFO | `mock child completed` | `outcome: &'static str`, `child_kind: &'static str`, `output_count: u64`, `error_kind: &'static str` exactly on operational failure. |

`outcome` is `completed` when the operation returns its typed result, `refused` for an expected typed refusal, or `failed` for an operational error. A nonzero child exit, including a relayed status, leaves the wrapper run's outcome `completed`; it does not imply `data_loss`. A signal-derived `ExecOutcome.exit_code` remains `128 + signal`, but `child_exit_code` is omitted because no normal child exit code exists. Library refusals are DEBUG with `refusal_kind` and no WARN or ERROR; a returned operational failure takes the operation's span tier with `error_kind`; data loss is returned in `ExecOutcome` and the binary emits the one terminal ERROR. A command usage or TTY refusal uses only the unfiltered WARN control record. The shared `cli-common::command_failed` terminal helper records its second argument as `error`, so each binary passes only a fixed kind such as `injected_failure`, `spawn`, `io`, `data_loss`, or `child_exit`; this is the shared terminal schema, while package events retain `error_kind`.

The implementation uses these fixed fields instead of the baseline per-chunk `writer typesetter` and per-line `writer calligraphic` events, parent passthrough errors, raw child-line fields, output-path fields, and free-form I/O error display. The library owns the first-write event, and the binary's callback consumes the notification without duplicating that event. `child stream completed` is emitted once after all destinations have finalized; `written_bytes` counts bytes in file writes that returned success, while a failed write may have left an unobservable partial prefix in the destination. It can differ from `read_bytes` because prefixed presentation transforms bytes and multiple destinations each receive output. The library retains a first-failure class per stream for the aggregate while its returned result remains the authority on data loss. Public `preflight_routing` emits one completion event; `run` uses its uninstrumented internal routing core, so an internal call does not duplicate preflight completion for one public operation.

## Naming · `sec:tracing-execwrap:naming`

Every target is an explicit literal rooted at the Rust crate stem: `execwrap::routing`, `execwrap::run`, `execwrap::stream`, `execwrap::writer`, or `execwrap::mock`; binary-local package work uses the same domains, while `cli_common::command` and `cli_common::control` belong to the shared layer. Span names use lower snake case without a package prefix or dots, and messages are fixed short lowercase English phrases. Field keys are lower snake case; categorical values are closed `&'static str` vocabularies rather than formatted Rust values. `stream` is `stdout` or `stderr`; `mode` is `raw` or `prefixed`; `route_kind` is `combined_raw`, `combined_prefixed`, `stdout_only`, or `stderr_only`; `child_kind` is `xelatex`, `biber`, or `latexmk`.

Cardinalities such as `argument_count`, `route_count`, and `output_count` are `u64`, converted from in-memory lengths with saturation so observation cannot panic; byte volumes are `read_bytes` and `written_bytes`, both `u64`. `refusal_kind` is `ambiguous_redirection`, `unrouted_stream`, or the library-only `empty_command` when applicable; `error_kind` is a fixed operational class such as `io`, `spawn`, or `reader_panic`, with `phase` used only to disambiguate work within a failure. No generic `operation`, process `exit_class`, output mode, signal-derived pseudo-exit code, path, child byte or line, raw argv, working directory, or free-form error text is a package event field. The shared command span supplies command identity, and the binary's terminal event supplies process status (`[ADR010-rule:output:data-classification]`).

The custom-main terminal `phase` values are `routing`, `setup`, `child`, and `mock`; `cli-common` owns their `cli_common::command` target, while this package supplies these fixed phase words and failure kinds. The `tracing_name_vocabulary_is_snake_case` test pins every package categorical value emitted from an enum or fixed field to the lower snake case grammar.

## Cost · `sec:tracing-execwrap:cost`

Filesystem identity resolution, reading, writing, and prefixed-line conversion happen for correctness and output; tracing does not repeat them to form fields. Read and written byte totals use local scalar accumulators in the existing loops and are emitted once per stream after finalization, not on every 8 KiB read or line; counting only successful existing `write_all` calls preserves the sink's I/O call sequence on failure. No span or event performs an extra path resolution, string allocation, child-byte decode, error formatting, hash, or timer sample merely for telemetry. Optional derived fields are computed only behind `tracing::enabled!` at the literal target and level or with lazy formatting; static labels, booleans, and existing lengths remain cheap when no subscriber exists. The parent standard-stream write and flush paths are absent.

## Testing · `sec:tracing-execwrap:testing`

Library instrumentation tests use `tracing::subscriber::with_default` and a package-local capture subscriber built on the workspace `tracing-subscriber` dev-dependency; they record typed events and span-close metadata in memory without a global subscriber. Every scoped capture test re-executes its exact test path as the only selected test in a fresh child process, checks that the child succeeded and exactly one test passed, and includes child output on failure, preventing concurrent uncaptured tests from changing its process-wide callsite interest. Within that child, one process-wide test mutex serializes scoped capture installs, and each scope rebuilds callsite interest inside `with_default` before invoking the subject; a regression touches a traced call without capture, then requires one span from each of two capturing threads. Reader-thread tests clone the scoped dispatcher with `tracing::dispatcher::get_default(Dispatch::clone)`, install it in the thread with `with_default`, and enter the parent span there. For each selected boundary, use a routed control returning `completed` and a subject one coordinate away returning `refused` at DEBUG, assert the same typed result or asset bytes with and without capture, and assert no WARN or ERROR library event for the expected refusal. A failed writer and a spawn failure check fixed error kinds and the returned data-loss or error path; tests assert span presence and fields, never durations or ordering across threads.

Subprocess tests assert the two binaries' actual stdout, stderr, and exit classes in fresh processes. A fully routed child is the control; removing only its stdout route or only its stderr route gives `unrouted_stream`, exit 2, empty stdout, one usage control record, no spawned child, and no opened output. Since an output-only route refuses before spawning, its former deferred `WroteToLog` callback and completion event cannot occur; routed notifications use `WritingToLog` and `log write started`. The tests verify byte-exact raw log assets, prefixed presentation, child status mapping, `--quiet`, help, usage, and control records under `RUST_LOG=off`; an enabled INFO span yields exactly one close record with numeric `busy_ns` and `idle_ns`. A test-harness child installs the same shared subscriber, triggers a panic, and checks one redacted panic control record without adding a panic trigger to either product binary. File-route cases supersede the former parent-passthrough tests. Test code itself writes no process stream; any archived measurement stays an explicit result asset, while the harness reports test wall time. The process-global bootstrap and panic path are asserted only through subprocesses, never a global in-process test subscriber.

## Implementation plan · `sec:tracing-execwrap:plan`

1. The first library commit instrumented public routing preflight and `run` with the literal targets, entry-only spans, fixed refusal and error kinds, and one completion event per public call; it added scoped control/refusal tests. The library installs no subscriber.
2. The second library commit instrumented reader threads, writer finalization, and mock generation with dispatcher propagation, aggregate `read_bytes` and `written_bytes`, safe notifications, and scoped tests; it removed the per-chunk and child-line debug events while preserving return values and byte output.
3. The third commit required a file route for each child stream in preflight, returned `unrouted_stream` before spawn or output opening, removed both parent relay paths, and replaced passthrough subprocess tests with routed byte-exact and one-coordinate refusal cases. It preserved child-status mapping and Meson log assets under the ADR-010 stream contract.
4. The fourth commit migrated both binaries to `install_command_subscriber` as their first statement, the shared custom-main `command` span and one terminal event, and fixed text-free failure kinds; it retained `--quiet` through the post-parse reload. Its subprocess tests assert `RUST_LOG=off`, panic, help, usage, span close records, JSON-only stderr, empty stdout, and status relay.

## Open questions for the other packages · `sec:tracing-execwrap:questions`

- Stream policy is settled: ADR-010 requires explicit child file routes or pre-spawn usage refusal for stdout and stderr; the child-status relay remains unchanged.
- Names and observations are settled: literal crate-stem module targets, `read_bytes` and `written_bytes`, `completed/refused/failed`, static refusal and error kinds, and one aggregate event per stream after finalization.
- Subscriber ownership is settled: `cli-common` installs the sole early subscriber and panic hook, keeps control records unfiltered with the existing locked JSON writer, reloads diagnostics after parsing, and emits enabled span-close timing records; this library never installs a subscriber.
- Field safety is settled: no caller-supplied log path, executable, working directory, raw child content, or free-form I/O error is a tracing field; static route labels and numeric counts suffice.
