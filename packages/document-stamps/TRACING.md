# Tracing design for `tripod-document-stamps`

## Purpose and shape · `sec:tracing-document-stamps:purpose`

`document-stamps` derives the Attestation paper's input-set date, paper-subtree timestamp, document UUID and instance UUID from committed Git state. The public `run(&StampRequest)` returns `AttestationStampValues`; `render(&RenderRequest)` fills a template and publishes `stamps.tex` and `source-date-epoch` with compare-if-changed staging. Neither path changes its derivation rules or result bytes for instrumentation.

`src/lib.rs` is a flat library containing validation, Git access, canonical input loading, digesting, timestamps, rendering and publication. `src/bin/attestation-stamps.rs` is the sole binary, with one-object JSON mode and file-only render mode. `src/tests/mod.rs` uses a fake `GitRunner`; `tests/subprocess_contract.rs` checks command streams and exit classes. `papers/attestation/meson.build` invokes render mode for the TeX build. The only workspace crate this package calls is `cli-common`, which owns the executable's stream and subscriber setup.

## Output inventory · `sec:tracing-document-stamps:inventory`

The table distinguishes runtime standard-stream routes from assets and illustrative README output; its locations and write descriptions are the design-time baseline, and its dispositions state the implemented behavior. Only the first row is a runtime stdout result, and the two former library error events were the only package-owned stderr diagnostics in that baseline.

| Baseline location | Design-time write or route | Class | Implemented disposition |
|---|---|---|---|
| `src/bin/attestation-stamps.rs:92-94` | `run_stdout_json_command` delegates one `AttestationStampValues` object and newline to `cli-common::emit` (`packages/cli-common/src/lib.rs:861-864`). | structured result | Keep byte-for-byte JSON stdout in inspection mode. |
| `src/bin/attestation-stamps.rs:83-90`; `src/lib.rs:192-229` | `run_no_stdout_command` leaves stdout empty while `render_outputs` writes two named files. | structured result asset | Keep `stamps.tex` and `source-date-epoch` at argument-named paths, never on stdout. |
| `src/lib.rs:326` | `tracing::error!` on Git spawn failure formatted `std::io::Error`, which could contain the caller-selected executable path. | diagnostic | The event is removed; the public completion carries fixed `error_kind`, and `cli-common` owns the terminal ERROR. |
| `src/lib.rs:340` | `tracing::error!` on nonzero Git exit in `run_text`; `run_bytes` returned the same failure class without this event. | diagnostic | The event is removed; the public completion carries fixed `error_kind`, and `cli-common` owns the terminal ERROR. |
| `src/bin/attestation-stamps.rs:67-68` | The former panic hook and parser delegated help, version, usage and panic JSON records to `cli-common`'s direct locked stderr writer. | diagnostic | The first action installs the shared subscriber before parsing; the control-record wire shape is preserved. |
| `src/bin/attestation-stamps.rs:83-94` | The former startup path initialized JSON tracing after parsing; the runners issued command-failure diagnostics and serialized the optional stdout result. | diagnostic and structured result | The runners reload the filter, open the command span, emit the terminal event and handle output mode without installing a second subscriber. |
| `README.md:62`, `README.md:63`, `README.md:64` | Three `println!` calls appear only in an illustrative library example. | illustrative structured result | They are caller-owned example output, not shipped binary writes or instrumentation. |

There is no live `println!`, `eprintln!`, `print!`, `eprint!`, direct `stdout()` or `stderr()` call, bespoke progress reporter, telemetry counter or local timing code in the package. `write!` at `src/lib.rs:799` formats an in-memory hex string. `SystemGit` at `src/lib.rs:320-330` captures child stdout for derivation and does not relay child stderr. The integration tests' pipe and PTY handles observe the binary; they are not package output sites.

## Standard output and standard error · `sec:tracing-document-stamps:streams`

JSON mode retains exactly one newline-terminated JSON object with `date: string`, `timestamp: { epoch: integer, iso_8601: string, pdf: string }`, `document_uuid: string` and `instance_uuid: string`. Its stdout TTY refusal occurs before derivation and exits 2 with empty stdout. Render mode writes only its two explicit assets, leaves stdout empty on every branch and does not refuse terminal stdout. A failure before JSON publication leaves stdout empty; a broken pipe during publication can leave a partial record while the command exits 1, as ADR-010 specifies (`[ADR010-rule:output:json]`).

The binary's first action is `cli_common::install_command_subscriber(COMMAND_NAME)`, which installs the one process-global JSON subscriber and panic hook before clap or any other panic-capable work. The library never installs a subscriber, and neither binary mode owns a direct stderr writer. `cli-common` has an unfiltered control route and a reloadable diagnostic route sharing one locked writer: help, version, usage, TTY refusal and panic keep the ADR-010 `schema`, `command`, `kind`, `message` and optional typed fields, while runtime events keep the tracing JSON `timestamp`, `level`, `target`, `fields`, current `span` and `spans` list. Help, version and status control records are INFO; usage and TTY refusal are WARN; panic and diagnostic control records are ERROR. The control route remains enabled even with `RUST_LOG=off` (`[ADR010-rule:output:control-plane]`).

Before parsing, the diagnostic filter uses nonempty `RUST_LOG` or otherwise INFO. After parsing, the runner reloads it using nonempty `RUST_LOG`, then `--debug`, then this command's INFO default; the reload handle stays inside `cli-common`. An invalid directive falls back to INFO and emits one WARN `filter directive rejected` event without echoing the environment value. The panic hook sends at most one best-effort control event through a nonblocking lock, drops it on contention or re-entry, and exits 1; its payload stays hidden unless parsed `--debug` enabled disclosure. The runner emits one terminal INFO `command completed` with `exit_class = "success"`, `output_mode` and `result_published`, or one ERROR `command failed` with `exit_class = "failure"`, a fixed `phase` and centrally redacted `error` text; usage and TTY refusal end with their control record alone. The implemented `cli-common` subscriber and amended ADR-010 require every first-party stderr byte to come through the subscriber, while the shared control schema, stdout bytes, exit classes and redaction policy remain intact (`[ADR010-rule:output:streams]`, `[ADR010-rule:output:exit-codes]`, `[ADR010-rule:output:data-classification]`). The legacy hook and tracing initialization helpers remain delegates for binaries still using them; they do not form a direct stderr route.

## Spans · `sec:tracing-document-stamps:spans`

Targets are explicit string literals; span names are lowercase snake case without a package prefix. Spans carry entry context only, not an outcome or count learned at return. The `render` entry uses an uninstrumented derivation core instead of calling the instrumented public `run`, so one external render call yields one package completion event.

| Operation and placement | Span name | Target | Level | Entry fields (type) | Duration measured |
|---|---|---|---|---|---|
| Public `run`, enclosing `derive` at `src/lib.rs:148-154,363-400` | `derive` | `document_stamps::derive` | INFO | `input_count: u64` | Whole Git-backed derivation in JSON mode. |
| Root, format, HEAD and first cleanliness checks at `src/lib.rs:368-374` | `validate_repository` | `document_stamps::derive` | DEBUG | none | Git-backed admission cost before reading inputs. |
| Canonical input loading at `src/lib.rs:511-534` | `canonical_inputs` | `document_stamps::derive` | DEBUG | `input_count: u64` | Per-input tree lookup, blob read and worktree-byte comparison. |
| Each `SystemGit::run` call at `src/lib.rs:318-334` | `git_command` | `document_stamps::git` | DEBUG | `git_operation: &'static str` | Process startup and Git I/O; one span per call makes a separate Git-call counter unnecessary. |
| Input hashing at `src/lib.rs:640-651` | `document_digest` | `document_stamps::derive` | DEBUG | `input_count: u64` | SHA-256 work over committed input bytes. |
| Public `render` at `src/lib.rs:180-190` | `render` | `document_stamps::render` | INFO | `input_count: u64` | Derivation, template substitution and both output publications as one operation. |
| Compare, stage, fsync and rename at `src/lib.rs:192-229` | `publish_outputs` | `document_stamps::publication` | DEBUG | `output_count: u64` (`2`) | Incremental file comparison and durable publication cost. |

`git_operation` is one fixed label from `rev_parse`, `status`, `ls_tree`, `cat_file` or `log`, selected by the call site rather than parsed from argv. No span records the Git executable, repository root, revision, input path, template, destination, Git output or derived identities. The shared runners add an enclosing INFO `command` span at `cli_common::command` after parsing, with `command` and `output_mode` (`stdout` for JSON mode or `none` for the no-stdout render runner); there is no parse span. The subscriber emits one close record for each enabled span with its name, target, level, entry fields and `busy_ns: u64` and `idle_ns: u64`. Close records are on by default, so the INFO command and public-operation spans report duration; DEBUG Git and phase spans appear when that filter tier is enabled. Nested durations are inclusive and a collector derives exclusive time from parentage rather than summing them.

## Events · `sec:tracing-document-stamps:events`

Exactly one library completion event is emitted within the span of each public `run` or `render` call, including an early return. The fixed message identifies the boundary; `outcome` carries `completed`, `refused` or `failed`. A completed result is not a claim about target acceptance. A typed expected refusal is DEBUG even though its enclosing span is INFO; a returned operational failure emits at its outer span's INFO tier, and only `cli-common` emits the terminal ERROR. Neither the Git helper nor a publication phase emits a second error event.

| Public boundary | Target | Message | Level | Fields (type and condition) |
|---|---|---|---|---|
| `run` | `document_stamps::derive` | `stamp values derived` | INFO for `completed` or `failed`; DEBUG for `refused` | `outcome: &'static str`, `input_count: u64`; `input_bytes: u64` only when already-held lengths are summed under an enabled event; `refusal_kind: &'static str` and `phase: &'static str` exactly on refusal; `error_kind: &'static str` and `phase: &'static str` exactly on failure. |
| `render` | `document_stamps::render` | `stamps rendered` | INFO for `completed` or `failed`; DEBUG for `refused` | `outcome: &'static str`, `input_count: u64`, `output_count: u64` (`2`); `changed_count: u64` only after both publications succeed; `refusal_kind: &'static str` and `phase: &'static str` exactly on refusal; `error_kind: &'static str` and `phase: &'static str` exactly on failure. |

The fixed `phase` values are `validate`, `git`, `inputs`, `digest`, `template`, `stage` and `publish`; a span does not repeat its own phase as a field. `input_count`, `output_count` and `changed_count` count items, while `input_bytes` is a byte quantity. `changed_count` means successfully published outputs whose bytes differed; it is omitted on failure because the two renames are not atomic and a partial publication must not be reported as full success. There is no `git_command_count` or `output_bytes` event field. The baseline ERROR events at `src/lib.rs:326,340` were removed, avoiding duplicate ERROR records and omission failures from their free-form Git spawn text.

## Naming · `sec:tracing-document-stamps:naming`

Targets are explicit `document_stamps::derive`, `document_stamps::git`, `document_stamps::render` and `document_stamps::publication` literals, and names and field keys use lowercase snake case. Count fields end in `_count` and use `u64`, with in-memory lengths converted at emission by saturation (`u64::try_from(n).unwrap_or(u64::MAX)`) so telemetry cannot panic or alter a result. Byte quantities end in `_bytes`; no measured duration is an event field. A Git child result has only `status_success: bool`, so this package cannot emit `child_exit_code`. The shared `command`, `output_mode`, `exit_class` and `result_published` fields belong to `cli-common`, not to library events; `schema` belongs to control records, not all tracing records.

Every current `StampError` variant has one payload-free category. A refusal means the supplied repository, revision, input set or template fails this operation's declared contract; a failure means Git, decoding, reading or publication could not complete. The table fixes the `refusal_kind` or `error_kind` string used by the completion event, without formatting the variant's payload.

| `StampError` variant | Outcome | Kind field value |
|---|---|---|
| `AliasedOutputs` | `refused` | `refusal_kind = "aliased_outputs"` |
| `RepositoryRootMismatch` | `refused` | `refusal_kind = "repository_root_mismatch"` |
| `UnsupportedGitObjectFormat` | `refused` | `refusal_kind = "unsupported_git_object_format"` |
| `GitCommandFailed` | `failed` | `error_kind = "git_command_failed"` |
| `InvalidGitOutput` | `failed` | `error_kind = "invalid_git_output"` |
| `SelectedRevisionIsNotHead` | `refused` | `refusal_kind = "selected_revision_is_not_head"` |
| `DirtyPaperSubtree` | `refused` | `refusal_kind = "dirty_paper_subtree"` |
| `InvalidInputPath` | `refused` | `refusal_kind = "invalid_input_path"` |
| `InputOutsidePaperTree` | `refused` | `refusal_kind = "input_outside_paper_tree"` |
| `DuplicateInput` | `refused` | `refusal_kind = "duplicate_input"` |
| `InvalidTrackedInput` | `refused` | `refusal_kind = "invalid_tracked_input"` |
| `InputReadFailed` | `failed` | `error_kind = "input_read_failed"` |
| `MissingPathHistory` | `refused` | `refusal_kind = "missing_path_history"` |
| `InvalidTreeObjectId` | `failed` | `error_kind = "invalid_tree_object_id"` |
| `TreePrefixResolutionFailure` | `refused` | `refusal_kind = "tree_prefix_resolution_failure"` |
| `TemplateReadFailed` | `failed` | `error_kind = "template_read_failed"` |
| `UnresolvedPlaceholder` | `refused` | `refusal_kind = "unresolved_placeholder"` |
| `OutputWriteFailed` | `failed` | `error_kind = "output_write_failed"` |

## Cost · `sec:tracing-document-stamps:cost`

Git calls, worktree reads, hashing and fsync remain required domain work. INFO spans use only cheap request lengths, and DEBUG spans use fixed operation labels and already-held lengths. An optional `input_bytes` total is computed only if its completion event is enabled and the loaded lengths are available; no extra Git read, input traversal, clone, digest, path formatting or error formatting occurs solely for disabled tracing. No subscriber means only a callsite check; tracing has no local `Instant`, counter registry or manual progress clock (`[ADR026-dec:tracing:core]`). The subscriber, rather than this library, times entered spans and emits close records.

## Testing · `sec:tracing-document-stamps:testing`

Library tests wrap the fake `GitRunner` call in `tracing::subscriber::with_default` with a package-local in-memory capture subscriber. Each scoped capture test re-executes itself as the sole `--exact` test in a fresh child process marked by an environment variable; the parent requires a successful exit and a summary reporting exactly one passed test, including the child's output on failure. The child keeps the capture-install lock and interest-cache rebuild, and all instrumentation assertions run there. A clean one-input control must return unchanged values and one `completed` event; the same fixture with only a dirty-subtree status changed must return `DirtyPaperSubtree` and one DEBUG `refused` event with `refusal_kind = "dirty_paper_subtree"`, with no WARN or ERROR library event. A Git spawn-failure fixture must return `GitCommandFailed`, produce one INFO `failed` event, and contain no executable path, argv or Git output in any captured field. The same domain values and rendered bytes are asserted with and without capture; tests check names, targets, levels, fields and span-close presence, never duration magnitude or cross-thread event order. If an instrumented operation later starts a thread, it must explicitly carry the dispatcher and parent span into that thread.

Subprocess tests of the real binary, not a global in-process subscriber, assert one JSON stdout object and newline in inspection mode, empty stdout and exact named assets in render mode, the existing PTY distinction, JSON-only stderr, help, version, usage and TTY-refusal control records under `RUST_LOG=off`, one numeric `busy_ns`/`idle_ns` close record for an enabled INFO span, the correct exit classes and no early argument echo. A separate exact-selected test-harness child installs the same shared bootstrap and panics before parsing to assert the unfiltered panic control record without adding an intentional panic trigger to the shipped binary. No test prints its own progress or uses a telemetry stopwatch; a package-local capture helper needs only a workspace-inherited `tracing-subscriber` dev dependency.

## Implementation plan · `sec:tracing-document-stamps:implementation`

1. **Library commit 1, independent of `cli-common`:** add fixed `StampError` refusal/failure classification and a package-local capture helper, then instrument public `run` with the INFO `derive` span and its single completion event; make `render` call an uninstrumented derivation core so public calls do not double-report, and assert the clean control and one-coordinate dirty-subtree refusal without changing returned values.
2. **Library commit 2, independent of `cli-common`:** add DEBUG `validate_repository`, `canonical_inputs`, `document_digest` and `git_command` spans with explicit domain targets and static `git_operation`; remove the two existing Git ERROR events, assert safe spawn-failure fields and unchanged derivation bytes, and keep all counters as emitted summaries rather than new state.
3. **Library commit 3, independent of `cli-common`:** add the INFO `render` span, DEBUG `publish_outputs` span and single render completion event with `output_count` and completed-only `changed_count`; preserve template substitution, staging, rename order, compare-if-changed behavior and both asset bytes, with local-capture tests.
4. **Binary commit 4, using the amended ADR-010 contract and implemented `cli-common` subscriber:** replace the first panic-hook statement with `install_command_subscriber(COMMAND_NAME)`, retain the shared runners for filter reload, command span, terminal record and JSON emission, and add subprocess coverage for both modes, PTY refusal, help, usage, panic, `RUST_LOG=off` and enabled span-close records. No other package waits on a new `document-stamps` API; Meson's Attestation build continues to consume the same two files.

## Open questions for the other packages · `sec:tracing-document-stamps:questions`

The shared vocabulary is settled for this package: explicit Rust crate-stem domain targets replace a workspace prefix; enabled spans emit close records by default with `busy_ns` and `idle_ns`, while high-frequency Git spans remain DEBUG; counts use `_count: u64` and bytes use `_bytes: u64` from completed work; outcomes are exactly `completed`, `refused` and `failed` with separate `refusal_kind` and `error_kind`; and `cli-common` supplies command context through its enclosing span while `schema` stays on control records. The binary uses the implemented pre-parse bootstrap and amended ADR-010 contract; library instrumentation does not depend on command setup. No unresolved cross-package naming or stream question remains here.
