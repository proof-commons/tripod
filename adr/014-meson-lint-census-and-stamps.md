# ADR-014: Meson Lint Census and Stamp-File Dependency Graph

**Status:** Decided and implemented
**Scope:** Every first-party checker and generator command, their Meson
wiring, the file census they consume, and the unit-test fixture policy
**Amends:** the census-discovery mechanism of
(`[ADR013-rule:labels:census]`); the checker command-line surface under
ADR-010

---

## Context · `sec:build:context`

Checker and generator binaries resolve their subjects from compile-time
paths and runtime directory walks. The build system therefore cannot see
the true input set: every `meson test` reruns every lint in full
regardless of what changed, no build target can depend on "lints passed"
as a graph fact, and the binaries are not relocatable — a compiled
checker is welded to the checkout that built it.

The paper subproject already models the target discipline: explicit
input lists, custom targets with declared outputs, and cargo-built
helper binaries synced copy-if-changed so a no-op rebuild does not
cascade. This record extends that discipline to the lint and generation
lanes.

## Inputs arrive by argument · `rule:build:arguments`

A first-party command receives the repository root, every subject file
or directory, and every output path by command-line argument.

No command resolves a repository path from its own compiled location.
`env!("CARGO_MANIFEST_DIR")` is forbidden outside test fixtures, in
library and binary code alike, including transitively: a library entry
point that needs repository paths takes them as parameters.

Arguments are role-tagged (for example `--adr`, `--plan`, `--doc`,
`--crate`), so the build system states membership while shape and
classification policy remain in Rust. A subject that violates the
declared role is a diagnostic, not a silent reclassification.

## Output or stamp · `rule:build:output-or-stamp`

Every Meson-invoked command declares its effect as exactly one of:

1. real output files, written only to argument-supplied paths under
   (`[ADR010-rule:output:assets]`);
2. a `--stamp <file>` argument: on success the command creates the file
   empty if absent, or updates its modification time if present, and
   writes nothing else to it.

On failure the stamp is left untouched, so the build graph keeps the
target dirty and reruns it. The stamp carries no content; stdout remains
the JSON report under (`[ADR010-rule:output:json]`). Stamp handling is
implemented once, in the shared command-line crate, so every checker
behaves identically.

## The build system owns the census · `rule:build:census`

Every directory of lint subjects carries a `meson.build` that declares
its files in explicit, hand-managed lists, grouped semantically — ADR
records, planning documents, register publications, per-crate Rust
sources, Attestation TeX sources. No globs, no configure-time discovery: a
file joins the census by being written into its own directory's list,
and the top-level build assembles the role groups from those lists.

Exclusions are equally explicit. A file of a subject type that is
deliberately not a lint subject — an imported TeX macro file, an
integration-test source — is declared in its directory's exclusion
list; tracked paths that are categorically never subjects (build
definitions, licences, archives, scripts, data blobs) are matched by
one exclusion pattern the build supplies to the audit.

Only tracked files are lint subjects, which aligns the lint gate with
the clean-tree law (`[ADR011-rule:toolchain:clean-tree]`): what CI
validates is exactly what the repository records.

## Census staleness is a hard failure · `rule:build:census-verification`

The hand-managed lists are audited, never trusted, from two
independent directions:

1. A cheap always-stale audit target runs a first-party binary that
   invokes `git ls-files` itself — the build passes the git program,
   the declared census, and the exclusions — so the weld is fresh on
   every build. A tracked subject missing from its directory's list,
   or a declared file no longer tracked, fails the build naming the
   paths.
2. The discovery walk of (`[ADR013-rule:labels:census]`) survives
   inside each checker as a verifier, not a source: the checker
   re-discovers its subjects on disk and hard-fails when the argument
   census and reality disagree.

Declared lists, tracked files, and the on-disk tree are thereby welded
pairwise. The ADR-013 completeness guarantee is preserved in a
stronger form: a newly added file cannot silently sit outside the
label graph — it can only fail the build until its directory's list
names it.

## Checks are stamp targets · `rule:build:stamp-targets`

Each check suite is one custom target: its inputs are the suite's census
slice plus the checker binary, its output is a build-directory stamp,
and its command ends with the stamp argument of
(`rule:build:output-or-stamp`).

The checker binaries are themselves custom targets built by the copy-if-changed cargo sync idiom, so the tools carry real dependency edges while cargo remains the tracker of Rust sources and resolves dependencies from the workspace manifests (`[ADR011-rule:toolchain:locked]`).

`meson test` entries become thin wrappers that depend on the stamp
targets. A rerun on an unchanged tree executes nothing.

## Generators emit stamps for committed publications · `rule:build:generator-stamps`

Generators keep writing committed in-tree publications under the
generation/check split (`[ADR012-rule:labels:generation]`); a committed
publication cannot be a declared build-directory output, so the
generator target's declared output is a stamp, and regeneration is
incremental over the same census slices.

Generated files remain publications, never semantic inputs
(`[ADR011-rule:toolchain:generated]`), and remain nonparticipating in
the label graph (`[ADR013-rule:labels:generated-nonparticipation]`).

## Publications gate on lints · `rule:build:publication-gating`

The export targets — the archive PDF mirror and the flattened source —
depend on the label-check stamp: a publication must not embed an
unlinted label graph. The plain in-build PDF compile does not, so draft
iteration stays fast.

## Cargo-global lanes stay cargo-tracked · `rule:build:cargo-lanes`

Workspace-wide `cargo fmt`, `cargo clippy`, and `cargo test` remain
Meson tests without stamp targets. Cargo is their dependency tracker;
duplicating its input model in the build system would add a second
source of truth without removing work.

## Unit tests are hermetic · `rule:build:hermetic-tests`

`cargo test` never discovers repository paths at runtime. Test subjects
are embedded at compile time with `include_str!` — cargo tracks the
fixture file, so the test rebuilds and reruns when it changes — or are
constructed synthetically in memory.

The question "does the committed repository pass" belongs exclusively to
the Meson-driven checker targets. No unit test re-implements it, and the
compile-time workspace-root constructors are deleted rather than kept as
a test convenience.

## Consequences · `sec:build:consequences`

- A no-op rebuild is a no-op: unchanged inputs rerun no lint, and
  downstream paper targets stay clean through restat.
- The true input set of every lint is stated in reviewed `meson.build`
  files: adding a subject is a visible diff in its own directory, and
  the build regenerates automatically when a list changes.
- Forgetting to list a new file cannot pass silently: the census audit
  of (`rule:build:census-verification`) fails the build until the
  directory's list names it.
- Checker binaries become relocatable and testable in isolation: their
  behaviour is a function of their arguments.
- Unit tests lose live-repository assertions; equivalent coverage moves
  behind the checker binaries where it is census-complete.

## Rejected alternatives · `sec:build:alternatives`

**Tool-emitted depfiles.** Discovery stays in Rust and the tool emits
Make-style dependency lines for the first build to consume later. The
first run has no edges, the build system never learns the census, and
the explicit-arguments boundary of (`rule:build:arguments`) never
materialises.

**Configure-time discovery from git.** Partitioning `git ls-files` at
`meson setup` time makes the census a wholesale-regenerated snapshot:
invisible in review, semantically flat, silently stale the moment a
file lands, and repaired only by a manual reconfigure. The allowlist
concern of (`[ADR013-rule:labels:census]`) is answered not by
discovery but by the audit of
(`rule:build:census-verification`): the lists are explicit, yet a
missed file is a build failure, never a silent omission.

**One generated manifest file.** A single checked-in census file has no
directory ownership and would be regenerated by tooling rather than
maintained where the files live; review degenerates to rubber-stamping
a machine diff.

**Git metadata as a regeneration dependency.** Registering the git index
as a configure dependency would reconfigure automatically on every
staging operation, but couples the build to git internals and fires far
more often than the census changes.

**Per-file stamps.** The label graph is a global analysis: one changed
Markdown file can invalidate resolution anywhere. Suite-granularity
stamps match the semantics; file-granularity stamps would claim a
precision the checker cannot honour.

**Status quo Meson tests.** Tests always rerun and declare no inputs or
outputs. They remain the right shape only for the cargo-global lanes of
(`rule:build:cargo-lanes`), whose tracker is cargo itself.

## Verification · `gate:build:implementation`

ADR-014 is implemented when:

- no first-party library or binary crate compiles a repository path
  into itself, and grep finds no `env!("CARGO_MANIFEST_DIR")` outside
  test fixtures;
- every checker accepts `--stamp` with touch semantics from the shared
  command-line crate, and every generator writes only argument-supplied
  outputs;
- every subject directory's `meson.build` declares its files in
  explicit hand-managed lists, and the top-level build assembles the
  role groups from those lists into every checker and generator
  target's arguments;
- a file added, removed, or renamed without updating its directory's
  list fails the census audit with a diagnostic naming the paths;
- a second `ninja` invocation on an unchanged tree runs no lint or
  generator command;
- editing one lint subject reruns exactly the suites whose census
  contains it;
- the archive mirror and flatten targets rebuild only after a current
  label-check stamp;
- `cargo test` passes with the network and the repository checkout
  outside the workspace crates absent, proving fixture hermeticity;
- `scripts/ci.sh` and the Meson document lane pass end to end.
