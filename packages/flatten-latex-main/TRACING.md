# Tracing engineering record for `flatten-latex-main`

## Purpose and shape · `sec:tracing-flatten-latex-main:purpose`

`flatten-latex-main` turns a caller-supplied main TeX file and fixed allowlist into one reproducible `.tex` asset at `--output`; it recursively inlines `\input` and `\subfile`, embeds `\addbibresource`, and refuses ambiguous, unsafe, cyclic, or unsupported references.

`src/lib.rs` owns the public `flatten` function and `FlattenOptions`, regular-file validation, suffix matching, line dispatch, recursive processing, and atomic staging and rename; `src/bin/flatten-latex-main.rs` is the sole binary and calls the library through `cli-common`; `src/tests/mod.rs` covers semantics and `tests/subprocess_contract.rs` covers CLI streams and exit classes.

The Attestation and Lineage Meson document targets call the binary at `papers/attestation/meson.build:378` and `papers/lineage/meson.build:172`; the workspace has no other Rust call site for `flatten` outside this package. Its public return is `Result<()>`; the generated TeX is an explicit asset, never a stdout result.

## Output inventory · `sec:tracing-flatten-latex-main:inventory`

This table records the design-time baseline before package instrumentation; its site references describe that baseline, while its last column states the resulting behavior.

| Baseline site | Design-time write or observation | Class | Resulting behavior |
|---|---|---|---|
| `src/lib.rs:146-159,336-382` | `writeln!` writes the fixed header and rewritten source through the staged file's `BufWriter`. | structured result asset | The byte-identical TeX stays at `--output`; tracing never carries document bytes. |
| `src/lib.rs:574-595,626,645-688` | `writeln!` writes commented directives, embedded bibliography, conditional and include markers, and surviving source through the same writer. | structured result asset | The TeX asset stays atomic; one completion event carries aggregate counts after persistence. |
| `src/lib.rs:605-608` | Lenient missing bibliography writes a `% WARNING` comment to the TeX asset. | diagnostic embedded in result asset | The comment remains and one WARN `bibliography omitted` event is emitted; strict mode returns an error. |
| `src/bin/flatten-latex-main.rs:59` | `tracing::info!` reports the caller-supplied output path and `flattened paper written`. | diagnostic and outcome | This event is absent; the library emits one completion event after persistence without the path. |
| `src/bin/flatten-latex-main.rs:48`; `packages/cli-common/src/lib.rs:635-642` | Shared argument parsing emits help, version, or usage JSON through the direct `emit_control_plane_record` writer. | diagnostic | The pre-parse subscriber renders the same control-record bytes and preserves exit classes. |
| `src/bin/flatten-latex-main.rs:50`; `packages/cli-common/src/lib.rs:916-938` | The shared runner installs JSON tracing late, emits `command failed` on `flatten` failure, and maps exit 0/1. | diagnostic and outcome | The runner reloads the installed subscriber and emits one terminal command record; library outcome metadata stays separate. |
| `src/bin/flatten-latex-main.rs:47`; `packages/cli-common/src/lib.rs:738-757` | The shared panic hook writes one JSON control record and exits 1. | diagnostic | The bootstrap installs the hook, which dispatches through the subscriber's nonblocking control route. |

There is no `println!`, `eprintln!`, `print!`, `eprint!`, direct `stdout()` or `stderr()` call, or direct standard-stream write in this package; no bespoke telemetry counter, progress, or timing code exists. At the design-time baseline, the binary's path-bearing success event was the only local tracing call; the library now emits the spans and events below, and `Cargo.toml` inherits `tracing`. Test helpers write to strings or files, not standard streams.

## Standard output and standard error · `sec:tracing-flatten-latex-main:streams`

This side-effect command keeps stdout empty on success, failure, help, and version; its result is the deterministic TeX asset named by `--output`. It does not refuse terminal stdout, because it publishes no stdout result. This preserves ADR-010's asset and stream split (`[ADR010-rule:output:streams]`, `[ADR010-rule:output:assets]`).

The binary's first action is `cli_common::install_command_subscriber(COMMAND_NAME)`, which installs its one process-global JSON subscriber and panic hook before clap can emit a record. `parse_args_or_exit` follows, and `run_no_stdout_command` reloads the diagnostic filter from a process-global handle using nonempty `RUST_LOG`, then parsed `--debug`, then the INFO command default; before parsing, the filter uses nonempty `RUST_LOG` or INFO. An invalid directive falls back to INFO and produces one WARN `filter directive rejected` event without echoing the environment value. The library installs no subscriber, and the binary installs no second one.

The subscriber's unfiltered `cli_common::control` route preserves the existing `schema`, `command`, `kind`, `message`, optional fields, and locked JSON line bytes for help, version, usage, TTY refusal, and panic even when `RUST_LOG=off`; its filtered diagnostic route uses the existing JSON tracing envelope. Both share one locked stderr writer. The panic hook emits at most one best-effort control event through a nonblocking lock, drops it on contention or re-entry, and exits 1; payload disclosure remains disabled until parsed debug permits it. Usage and TTY refusal end with their control record alone. Runtime failure ends with `cli-common`'s one ERROR `command failed` terminal event and exit 1; success ends with its INFO `command completed` event and exit 0.

ADR-010 requires the one subscriber to render every first-party stderr byte: early control records retain their wire shape, enabled span closes carry numeric durations, and raw child-byte relay is excluded from both standard streams (`[ADR010-rule:output:control-plane]`, `[ADR010-rule:output:early-startup]`). The child-stream file-route rule belongs to `execwrap` and does not change this flattener's empty stdout or TeX asset. `cli-common` owns the bootstrap, writer, filter, terminal records and exit mapping; this package never writes stderr directly.

## Spans · `sec:tracing-flatten-latex-main:spans`

The site references in this and the Events table identify design-time boundaries before instrumentation; the span and event names, targets, levels and fields describe the implemented records.

| Operation and site | Span name | Literal target | Level | Entry fields (type) | Duration worth measuring |
|---|---|---|---|---|---|
| `flatten` at `src/lib.rs:59` | `flatten` | `flatten_latex_main::flatten` | INFO | `allowed_file_count: u64`, `strict_bibliography: bool` | Whole validation, recursive I/O, staging and rename; nested under `cli_common::command` for a binary call. |
| Input role validation at `src/lib.rs:67-70` | `validate_inputs` | `flatten_latex_main::input` | DEBUG | none | Separates metadata checks from generation on a slow filesystem. |
| `write_flattened` at `src/lib.rs:136` | `write_flattened` | `flatten_latex_main::output` | DEBUG | none | Isolates document generation and flush from validation and rename. |
| `FlattenContext::process_file` at `src/lib.rs:281` | `process_file` | `flatten_latex_main::input` | DEBUG | `depth: u64`, `is_main: bool` | Shows recursive read cost and include-chain depth without source paths. |

Each target is an explicit Rust crate-stem and domain literal, and span names are unprefixed lower snake case. Spans carry entry context only: no outcome or completed count is recorded on a span. The JSON subscriber emits one close record for each enabled span, with its name, target, level, entry fields, and `busy_ns: u64` and `idle_ns: u64`; the durations are inclusive and must not be summed as independent work. There is no per-reference span or local `Instant`, and no separate parse or result-publication span in this package.

## Events · `sec:tracing-flatten-latex-main:events`

| Trigger | Literal target | Level | Fixed message | Fields (name: type) |
|---|---|---|---|---|
| `flatten` returns after successful rename at `src/lib.rs:98-102` | `flatten_latex_main::flatten` | INFO | `flatten completed` | `outcome: &'static str` (`completed`), `include_count: u64`, `omitted_include_count: u64`, `bibliography_count: u64`, `file_count: u64`, `max_include_depth: u64` |
| `flatten` returns an expected typed input or syntax refusal | `flatten_latex_main::flatten` | DEBUG | `flatten completed` | `outcome: &'static str` (`refused`), `refusal_kind: &'static str`, `depth: u64` |
| `flatten` returns an operational failure | `flatten_latex_main::flatten` | INFO | `flatten completed` | `outcome: &'static str` (`failed`), `error_kind: &'static str`, `phase: &'static str` (`validate`, `write`, or `rename`) |
| A missing bibliography is tolerated at `src/lib.rs:604-608` | `flatten_latex_main::resolve` | WARN | `bibliography omitted` | `strict_bibliography: bool` (`false`) |

The first three rows are mutually exclusive forms of exactly one completion event inside the root span, emitted on every public `flatten` return; a failure before persistence never claims `completed`. `include_count` counts successfully inlined ordinary and true-branch conditional includes; `omitted_include_count` counts `flatten-ignore` includes and absent conditional probes; `bibliography_count` counts embedded bibliographies; `file_count` counts successfully processed files including the main file; `max_include_depth` is zero for the main file and one for its direct includes. These are observations, not output bytes or result fields.

`refusal_kind` is one of the fixed, payload-free categories `invalid_input`, `invalid_output`, `invalid_reference`, `missing_include`, `ambiguous_reference`, `include_cycle`, `unsupported_syntax`, or `missing_bibliography`; it is present exactly for `refused`. `error_kind` is `io` or `publication` and appears exactly for `failed`; `phase` identifies the fixed step. The returned `anyhow::Error` remains the caller's detailed result. No library refusal is WARN or ERROR, no per-include event remains, and the binary's path-bearing success event is removed. The lenient bibliography warning is recoverable, so it keeps its TeX comment and one WARN event without a counter field; `cli-common` separately owns the user-facing terminal command record.

## Naming · `sec:tracing-flatten-latex-main:naming`

Targets are explicit literals under `flatten_latex_main::<domain>` (`flatten`, `input`, `output`, `resolve`); event messages are fixed short lowercase English phrases, and field keys are lower snake case. `outcome` has exactly `completed`, `refused`, or `failed`; `refusal_kind`, `error_kind`, and `phase` use the closed values above; counts and depth are `u64`, with saturating conversion from collection lengths at emission. The output path, source text, reference names, include stack, raw error text, and result bytes never become fields. Command-only fields such as `command`, `output_mode`, `exit_class`, and `result_published` belong to `cli-common`, not the library.

## Cost · `sec:tracing-flatten-latex-main:cost`

The four spans use already-held booleans and lengths; `allowed_file_count` is converted with saturation and appears only on the root, since its value is available through the JSON span list on child records. The primitive include, omission, bibliography, file and maximum-depth tallies live in `FlattenContext`, update only when the INFO completion event is enabled, and are read once after successful persistence; no additional walk, path formatting, string join, hash, or clock read is performed for tracing. A disabled subscriber leaves semantic validation, staging and output bytes unchanged. Error classification carries a static category from the decision branch to the owned return boundary rather than parsing or formatting an `anyhow::Error`.

Error classification wraps returned errors on the error path regardless of subscriber interest, so callers receive the same chain and downcast surface with or without capture; `ClassifiedError::source` skips the forwarded outer message, and only completion-event emission is gated by `tracing::enabled!`.

## Testing · `sec:tracing-flatten-latex-main:testing`

Library tests use a package-local capture subscriber through `tracing::subscriber::with_default`, never a global subscriber, to assert the INFO root span, its close record and one `completed` event with the five expected tallies for a small control. A one-coordinate-away missing include returns the same error with a DEBUG `refused` completion and `refusal_kind = missing_include`, with no WARN or ERROR event; a lenient missing bibliography produces exactly one WARN while preserving its `% WARNING` asset comment. Compare return values and byte-identical output with and without capture; assert fields, not elapsed durations or cross-thread order. If a test spawns a thread, carry the dispatcher and parent span into it explicitly.

The capture helper holds a package-local static mutex for the whole scoped subscriber installation and capture, and rebuilds callsite interest inside `with_default` before the subject runs; a regression first calls the traced boundary without capture, then captures it on two threads and requires exactly one root span in each. The mutex handles poisoning without making later tests depend on an earlier test's result.

Each scoped capture test re-executes itself as the only selected test in a fresh child test process before running its assertions; the parent requires a successful exit and a test summary with exactly one pass, and includes the child's output on failure. This isolates process-wide callsite interest from unrelated parallel tests while retaining the mutex, interest rebuild and strict record assertions inside the child.

Tests compare refused and operational error chains, alternate renderings and classification downcasts between captured and uncaptured calls, require one copy of the outer message, and check all declared categorical names against lower snake case; this package has no category containing a single-letter word.

Subprocess tests exercise the actual binary's process-global bootstrap: success and pre-publication failure leave stdout empty, stderr is JSON-only, exit classes stay 0/1/2, and an enabled INFO span yields one close record with numeric `busy_ns` and `idle_ns`. Help and usage records remain present with `RUST_LOG=off` and keep the control-record schema; the shared `cli-common` subprocess suite covers its panic and contended-writer paths. The TeX asset is still byte-identical, and failure leaves no partial output. No test writes its own standard stream; any test-only progress is a scoped tracing event.

## Implementation plan · `sec:tracing-flatten-latex-main:plan`

1. The library commit added the four spans, primitive `FlattenContext` tallies, one post-persist or refusal/failure completion event, the lenient bibliography WARN, and scoped capture tests for a completed control and a one-coordinate expected refusal. The library neither installs a subscriber nor owns stderr.
2. With ADR-010 and the `cli-common` bootstrap in place, the binary commit installed `install_command_subscriber(COMMAND_NAME)` first, retained parsed `--debug` through the runner's reload, removed the binary success event, and extended subprocess tests for `RUST_LOG=off`, JSON control records, span closes, empty stdout, exit classes and unchanged asset bytes.
3. Compatibility-delegate removal in `cli-common` and the workspace ban on direct `stderr()` calls belong to the shared owner; this package has no local commit for them.

## Open questions for the other packages · `sec:tracing-flatten-latex-main:questions`

- The early control route keeps the existing ADR-010 wire record and remains unfiltered; diagnostics and span closes use the tracing JSON envelope, and both routes share one locked writer.
- Counts are the five named aggregate fields on the single completion event; no generic `count = 1` or per-include event remains.
- Caller-supplied output paths never enter this package's tracing fields; the TeX file remains the explicit result asset.
- The library's post-persist `flatten completed` event reports the package operation, while `cli-common` emits the distinct terminal command record and measures its enclosing command span.
