# Tripod

Tripod is a contract closure compiler: it carries a typed, target-independent realization of a smart contract through analysis, emission, linking, and transaction construction to exact target bytes, with every seam machine-checked.

A compiler is proved by what it compiles. This tree therefore carries, beside the compiler, the contract that proves it: the attestation — a reserve-backed, two-class, conserved, burnable receipt, specified abstractly in the *Attestation* paper, realized as a typed architecture manifest and an executable reference model, and compiled toward an Elements/Liquid covenant. The attestation contract is the non-trivial exemplar the compiler's build-out is verified against; every capability the compiler claims must first hold on it, end to end.

The realization declares three operations: compact ASH, live-receipt transfer, and the maturity announcement, which updates the STATE object through its candidate constructor. Each is carried from typed declaration through analysis, backend construction, and linking to candidate transaction bytes, and is judged by the evidence in `packages/vectors/`. Everything past the realization is candidate: no crate produces a final deployment bundle, and no production target evidence exists in the repository.

The plan tree declares where the work stands: `plans/roadmap.md` in its status section and `plans/phases/README.md` in its phase index. The obligations still open are recorded in the phase cards under `plans/phases/` and in `plans/backlog.md`, and are read there.

## Layout

- `packages/` — the sixteen crates of the Cargo workspace, listed under *Packages* below.
- `papers/attestation/` — LaTeX source of the *Attestation* specification, built as the Meson subproject `attestation`.
- `docs/attestation/` — the realization document `realization.md`, the Elements/Liquid realization contract, and `human.md`, its plain-language companion; they live outside the paper tree so that editing them does not perturb the paper's derived identity.
- `adr/` — architecture decision records of implemented repository-wide engineering policy, indexed by `adr/README.md`. ADRs are subordinate to the specification, the realization contract, and the typed architecture, and take precedence over planning documents.
- `plans/` — the plan tree, indexed by `plans/README.md`: the roadmap, phase cards, package contracts, planning decisions, research notes, references, registers, generated label registers, archived guides, drafts and reviews, and the backlog. No plan claims semantic authority.
- `lint/hash-citation-families.tsv` — the families by which the hash-citation audit describes every hexadecimal value the tracked tree carries, each with what it measures and the program that writes it.
- `scripts/` — the gate shim `ci.sh` and its timing reader, `check-document-reproducibility.sh`, the target-native executor adapter and the capture drivers for native runs of record, and the helpers and contract tests the Meson graph calls.
- `Cargo.toml` — the workspace: its members, the package metadata every crate inherits, shared dependencies, and workspace lints.
- `meson.build` and `meson.options` — the build and the gate: document targets, generators, the ADR-014 lint census, and the gate's lanes.
- `AGENTS.md` — working rules for agents in this tree: the canonical build directory, routine and full-gate cadence, and the citation rules.
- `SECURITY.md`, `LICENSE-CODE`, `LICENSE-DOCS`, and `.gitignore`.

## Packages

The workspace lists sixteen members in `Cargo.toml`, in the order below. Every crate inherits version, MSRV, licence, and `publish = false` from the workspace package table and its lints from the workspace lint tables, and each crate's README states what it owns and what it does not. Three packages take their directory name as package name (`cli-common`, `execwrap`, `flatten-latex-main`); the rest carry the `tripod-` prefix.

| Directory | Package | Contract |
|---|---|---|
| `packages/architecture/` | `tripod-architecture` | Typed normative architecture manifest of the attestation realization: the finite identifier vocabulary, including the maturity announcement's operation, STATE data identifiers, and lead bounds, with validation, canonical semantic hashing, and the export behind `architecture.json` and `architecture.toml`. |
| `packages/artifacts/` | `tripod-artifacts` | Single generator/checker for the workspace's generated derivative artifacts under `packages/model/generated/`: one typed source, one writing generator (`generate-all`), one non-writing checker (`check-generated`). |
| `packages/cli-common/` | `cli-common` | Shared CLI scaffolding for ADR-010 command-line tools: argv parsing, JSON result and diagnostic records, output publication, and exit classes, with no repository knowledge. |
| `packages/compiler/` | `tripod-compiler` | Target-independent compilation analysis of a validated contract realization: relation, proof, disclosure, lifecycle, placement, layout, and coverage analysis over the compact-ASH and live-transfer pilot scope, the abstract target-requirement projection, and the validated maturity-announcement target-operation plan; it emits no target program. |
| `packages/document-stamps/` | `tripod-document-stamps` | Derive deterministic Attestation paper metadata from Git: the `attestation-stamps` binary derives the paper's date, timestamp, and two UUIDs from committed state and renders them for the TeX build. |
| `packages/execwrap/` | `execwrap` | Process wrapper that routes stdout/stderr to log files; the paper build runs the TeX toolchain through it, and it is not a sandbox. |
| `packages/flatten-latex-main/` | `flatten-latex-main` | Flattens a paper's `main.tex` into one self-contained `.tex`, inlining `\input` and `\subfile` references and embedding the `\addbibresource` bibliography, from an explicit list of files only. |
| `packages/labels/` | `tripod-labels` | Repository-wide documentation-label registries and conformance checks: label harvesting, citation and weld validation, deterministic label publications, the plan-tree checks, the ADR-014 census audit, and the forbidden-text and hash-citation audits; documentation tooling that no semantic package consumes. |
| `packages/linker/` | `tripod-linker` | Resolves typed relocatable backend bundles into deterministic candidate linked bundles: the compact-ASH, live-transfer, and maturity-announcement links, each with read-only candidate status and its outstanding obligations. |
| `packages/model/` | `tripod-model` | Executable state-machine model of the attestation contract: a pure transition function from a world to its successor or a guard, implementing the frozen architecture manifest and checked against it, over public data only. |
| `packages/realization/` | `tripod-realization` | Target-independent typed semantic realization of the attestation contract: typed declarations for compact ASH, live-receipt transfer, and the maturity announcement, and the canonical STATE metadata encoding, consumed by model conformance and compiler analysis. |
| `packages/tapscript/` | `tripod-tapscript` | Adapter from compiler-owned abstract target requirements to Elements target obligations, with typed instructions, an exact serializer and parser, an abstract stack validator, the compact-ASH and live-receipt constructors and bundles, and the candidate STATE constructor with its maturity-announcement records and programs; it runs nothing against a node. |
| `packages/target-elements/` | `tripod-target-elements` | Typed reviewed Elements tapscript target compatibility contract: the primitive, encoding, capability, resource, and evidence-requirement registries, the development deployment binding, the reviewed transaction forms including the maturity announcement's, and the 80-byte relay bound per tapscript witness stack item; it records no evidence and claims no production support. |
| `packages/target-elements-conformance/` | `tripod-target-elements-conformance` | Secretless target-native conformance harness for reviewed Elements primitives: it drives a caller-selected external executor over the revision-8 wire protocol and records typed primitive, prototype, declassification, and operation-step reports; the repository's development native evidence is produced through it. |
| `packages/transaction/` | `tripod-transaction` | Derives candidate transaction ABIs for compact ASH, live transfer, and the maturity announcement and constructs exact target transaction bytes; it holds no key and submits nothing, and signatures arrive through a capability adapter. |
| `packages/vectors/` | `tripod-vectors` | Carries canonical semantic and target fixtures for compact ASH, live transfer, and the maturity announcement, their relation-case evidence plans, the live-transfer and STATE runs of record, and five candidate reports for the maturity announcement (safety, continuity, root history, public recovery, resources). |

Package contracts for eight of these crates (`compiler`, `labels`, `linker`, `realization`, `tapscript`, `target-elements`, `transaction`, `vectors`) live under `plans/packages/`, beside contracts for `release` and `simplicity`, which describe packages the workspace does not contain.

## Requirements

- A Rust toolchain with Cargo. The workspace declares its MSRV once, `rust-version = "1.88"`, beside `edition = "2024"`, and every crate inherits both. The required CI Rust toolchains are the declared MSRV and current stable; nightly may be used for optional diagnostics or development, but no nightly feature may become load-bearing. The repository commits no `rust-toolchain.toml`: the build environment supplies the toolchains (ADR-011).
- No `Cargo.lock` is committed. Before v1 the dependency surface floats within the version ranges `Cargo.toml` declares, resolution is reproduced from the workspace manifests and the registry and is not byte-stable across time, and a dependency change lands as a reviewed edit to `Cargo.toml`. ADR-011 records the return of a committed lockfile as a deliberate step of the v1 approach.
- Meson 1.3.0 or later, with Ninja.
- `cargo` and `git`, both required at configuration, and a POSIX `sh` with the standard file, text, and hashing utilities the scripts under `scripts/` use.
- `python3`, required at configuration for the executor-adapter tests and used by `scripts/ci.sh` for its timing report.
- The TeX programs `latexmk`, `xelatex`, and `biber`, required unless `mock_mode` is set.
- `cargo-audit`, optional and externally provisioned: without it the advisory lane reports SKIP.

## Building

`build/` at the repository root is the canonical build directory, configured once and reused (`AGENTS.md`); ADR-015 and ADR-017 name it as the canonical build root. The document build, then the gate:

```sh
meson setup build
meson compile -C build attestation
meson test -C build --print-errorlogs
```

Configuration compiles nothing: every helper binary is a custom target that builds that one binary through `scripts/cargo-bin-sync.sh`, so Cargo remains the Rust dependency tracker. `meson.options` defines `mock_mode`, which configures the same graph with the TeX programs simulated by the test-only `execwrap-mock-tex` helper, for a machine without TeX; a mocked build renders no document:

```sh
meson setup build -Dmock_mode=true
```

The `attestation` target renders the specification and flattens it: the PDF is mirrored into `archive/rendered/` and the single-file `.tex` into `archive/flattened/`, both named from the paper subproject's version and both git-ignored. The mirrors wait on the label check, so no publication embeds an unlinted label graph.

### Generated artifacts

`packages/model/generated/` holds the workspace's generated derivative artifacts, owned by `tripod-artifacts`; they are publications, never semantic inputs. The first command regenerates them; the second checks the census, labels, generated artifacts, plans, forbidden text, and hash citations without writing; the third regenerates the planning label registers under `plans/labels/`:

```sh
meson compile -C build generate-artifacts
meson compile -C build lint
meson compile -C build generate-label-registers
```

Direct Cargo invocations of the generator and checker binaries are build-system internals: they take the full role-tagged ADR-014 census that Meson derives. A newly tracked file is listed in its directory's `meson.build`, or the census audit fails the build.

### Paper provenance

Before rendering, `attestation-stamps` derives four values from committed Git state and renders them into a generated `stamps.tex` and a `source-date-epoch` file:

- `date` — the UTC date of the latest commit touching an exact paper input, the visible frontmatter date.
- `timestamp` — the UTC time of the latest commit touching the paper subtree, the source date epoch every TeX step runs under and the source of every XMP date.
- `document_uuid` — the first 128 bits of a canonical digest over the exact input set, each input's path, Git file mode, and committed bytes (XMP `DocumentID`).
- `instance_uuid` — the first 128 bits of the Git tree object for `papers/attestation`, in UUID grouping (XMP `InstanceID`).

The derivation accepts only a revision resolving to checked-out `HEAD`, and a dirty or untracked file under the paper subtree fails it, so a dirty subtree blocks publication. The two UUIDs are source-provenance identities, not hashes of the final PDF bytes; byte reproducibility is checked separately by `scripts/check-document-reproducibility.sh`.

## Gate

`meson test` is the gate. Every lane is a Meson test, declared, timed, and statused by Meson's own harness; there is no second lane registry. The lanes, by kind:

- Rust formatting, lints, and API documentation: `cargo-fmt` (a formatting check), `cargo-clippy` (all targets, warnings denied), and `cargo-doc-workspace` (the workspace documentation, with links resolving and no warnings).
- Per-package tests: one lane per workspace member and profile, named `cargo-test-debug-` or `cargo-test-release-` followed by the package directory; configuration fails when this list and the `Cargo.toml` members disagree.
- Tracked-tree and text audits: `census-audit` (the ADR-014 census over the complete tracked set), `tracked-path-argv-audit`, `forbidden-text-check`, `hash-citations-check`, and `clean-tree`, scheduled after the parallel work, which fails on any modified tracked file or untracked, unignored leftover.
- Generated and documentation checks: `check-generated`, `labels-check`, and `plans-check`.
- Paper provenance: `attestation-stamps` (the paper subproject's probe of the derived stamps), `attestation-stamps-scopes` (the Git-derived identity scopes, with real Cargo and Git and no TeX), and `publication-mode-freshness` (the publication paths repair a wrong file mode).
- Build graph: `meson-mock-contract`, a nested Meson build under `build/mocks/` with the TeX toolchain mocked, covering the census wiring, stamp and report edges, generator and publication repair, restat, and render-failure propagation.
- Capture contracts: `live-native-v2-r7-capture-contract`, `live-native-v2-r8-capture-contract`, `live-native-maturity-capture-contract`, and `live-native-maturity-mutant-capture-contract`, which exercise the capture drivers' run boundaries in shell over a synthetic Git suite and a mock Cargo, with no node, target binary, or network.
- Executor adapter: `executor-classification` and `executor-boundaries`, node-free Python tests of the target-native executor adapter.
- Advisory: `cargo-audit`. When no `cargo-audit` binary is on `PATH` or under Cargo's home, `scripts/gate-cargo-audit.sh` exits 77 and Meson reports SKIP; a skipped lane is neither failure nor success, and a run that skipped one is not a green run (ADR-011).

These are fifty-four lanes: 21 single declarations in the root `meson.build`, 32 per-package lanes from one declaration over the 16 packages and the debug and release profiles, and 1 in `papers/attestation/meson.build`.

`scripts/ci.sh` is the runner-agnostic entry point, a thin shim over the same gate. It configures its own build directory (`target/ci-meson`, or the one `CI_BUILD_DIR` names) with `mock_mode=true` and with publication and generation outputs redirected into that directory, forwards its arguments to `meson test`, and prints a per-suite and per-test timing report from the run's logs, on failure as on success. A suite or a single lane is the same command narrowed:

```sh
scripts/ci.sh
scripts/ci.sh --suite lint
scripts/ci.sh cargo-clippy
scripts/ci.sh --list
```

The mocked gate renders no document. The complete repository gate of ADR-011 is `scripts/ci.sh`, plus the real document build and `meson test` in a non-mocked build directory, plus `scripts/check-document-reproducibility.sh`, which renders the document in two clean build directories, requires byte-identical PDFs, and refuses a dirty worktree. Target-native checks are outside the gate: the `target-elements-native-check` and `target-elements-prototype-check` targets are defined only when the `target_native_executor` option names an executor, and a run without one claims no target-native evidence.

### Command-line output contract

Every executable in the workspace follows ADR-010 ([adr/010-command-line-output-contract.md](adr/010-command-line-output-contract.md)) through `cli-common`: stdout carries only JSON result data, one object or NDJSON where a command documents streaming, apart from `execwrap`'s deliberate relay of child bytes, and a result command refuses a terminal; assets are written only to paths given by arguments such as `--output`; help, usage, diagnostics, and panic reports are JSON on stderr. The exit classes are 0 (success), 1 (failure), and 2 (usage, including terminal refusal); `execwrap` additionally relays child statuses 3 to 255 after successful wrapper startup, while codes 0 to 2 keep their shared meanings.

## Evidence

Evidence is kept apart from what it judges. `vectors` derives every expectation from the layer that defines the thing expected — relation identities from `realization`, coverage requirements from `compiler`, target bytes from `transaction` over a `linker` bundle — and none from the candidate being judged. Target acceptance and semantic acceptance are two verdicts, and the crate keeps them two.

Target-native evidence is produced through `target-elements-conformance`, which drives a caller-selected external executor over its secretless protocol and records what that executor answered. The executor is not authenticated, sandboxed, or shown independent, and a mock executor can never satisfy a target-native gate. The capture drivers under `scripts/` own the run boundary: from a clean worktree they run the ignored, environment-gated `tripod-vectors` tests against a development node in one serialized invocation and write the capture's file census and manifest. `target-elements` carries target evidence requirements and no evidence.

A run of record is a transcription. The corpora under `packages/vectors/fixtures/` hold what a real development node did in captured runs — among them the maturity announcement's whole-item, variable-schedule, predecessor-mutant, and successor-mutant runs — and a strict importer admits each one only after checking its fixed file census and manifest or its pinned addresses, before any transcript is parsed. Where a planner replays a corpus's recorded exchanges, replay establishes transcript consistency under the pinned schedule, not executor provenance, root freshness, or a promoted standing.

The live-transfer and maturity-announcement evidence plans each classify every row of their safety matrix into exactly one standing, and each plan's census is where the counts are read; for the maturity announcement that is `MaturityEvidenceCensus` in `packages/vectors/src/maturity_evidence.rs`, pinned by the test `the_census_figures_are_unmoved_by_the_binding_readers`. There a row is answered only by what the plan itself recomputed or observed: the exact owning validator driven twice, accepting the honest input and refusing the input that differs by the row's one change with a typed refusal naming the intended class; a validated report carrying the row's exact section and row; or a target observation bound to the row, either a refusal recorded against an accepted control at exactly the boundary the row declared or an acceptance with exact readback. A run's text alone answers nothing, a refusal at an unexpected boundary or an infrastructure failure answers nothing, and the census reports every required row answered only when none remains unanswered.

The rows waiting on a target run are registered with their named gaps in `packages/vectors/src/live_negative_half.rs` and `packages/vectors/src/maturity_negative_half.rs`, each held against its classifier in both directions by the test `every_outstanding_row_is_registered`; a row leaves a register only by being answered at its own site.

None of this is deployment evidence. Architecture finality and model success are not deployment readiness, the target contract claims no production support, no production target evidence exists in the repository, and at deployment-profile schema 2 `validate_production_deployment_release` refuses every profile, because that schema has nowhere to bind the transaction ABI and configuration its calibrations were measured under.

## Security and execution trust

First-party packages are public-data tools. They accept no private keys, passwords, tokens, signing nonces, blinding factors, private openings, wallet credentials, or production deployment authority, and the repository contains no production target backend, transaction signer, wallet, deployment release, or production key-management component. Disposable material generated for a test network is public fixture data under ADR-015, and a package introducing a legitimate secret input requires a reviewed security design before its interface is accepted.

Repository source, tests, Meson definitions, scripts, TeX, and `.latexmkrc` are executable, and the checkout does not sandbox itself. An untrusted contribution is run in an environment established outside the untrusted checkout, holding no credentials, production authority, repository write authority, or release authority. `execwrap` is a process launcher and byte router, not a sandbox, and the child output it relays is not sanitized; a target-native executor is caller-selected code, and selecting one grants it execution authority.

Path claims apply lexically beneath the repository root and the canonical build root `build/`. The tracked tree holds only ordinary Git blobs, with no symlinks, gitlinks, or submodules, and the host owns filesystem integrity, aliasing, and time-of-check/time-of-use races ([ADR-017](adr/017-path-scope-and-host-filesystem-trust.md)).

Architecture or model success is not deployment readiness: release builders produce unsigned candidates, and production signing or deployment authorization belongs to a separate operational boundary. See [SECURITY.md](SECURITY.md), which states what to report and how, and [ADR-015](adr/015-public-data-and-execution-trust.md).

## Version registry

The repository carries several intentionally distinct version numbers. Each has its own meaning and bump discipline; exactly one is a derivation of another (the realization binding).

| Version | Where | Meaning |
|---|---|---|
| Specification `v1.0.0` | `papers/attestation/sections/00_title.tex`, and the manifest's specification binding | The released abstract economic specification, pinned by the manifest's specification binding and anchor-set hash. |
| Realization version `0.6.0-dev` | the release envelope of `docs/attestation/realization.md`; `DOCUMENT` in `packages/architecture/src/spec.rs` | The tracked binding to the compiler line: `major.minor` copied from the workspace version, patch always zero, prerelease carried verbatim. It is patch-blind and content-blind, and envelope metadata outside both hashes; the behavioural hash alone witnesses denotation stability, and the binding gate derives the expected value from the workspace version at compile time. |
| Architecture schema `18` | `ARCHITECTURE_SCHEMA_VERSION` in `packages/architecture/src/spec.rs`; the manifest envelope | Shape of the exported manifest. Envelope metadata, never a hash input; artifact ingestion rejects any other value. |
| Attestation wire schema `13` | `ATTESTATION_SCHEMA_VERSION` in `packages/model/src/ledger.rs`, carried in the `schema_version` field of `AttestationContext` | Canonical attestation query encoding. `docs/attestation/realization.md` distinguishes it from the architecture and deployment-profile schemas under "Stable identifiers, the hash, the schemas, and the weld"; the three advance independently. |
| Deployment-profile schema `2` | `DEPLOYMENT_PROFILE_SCHEMA_VERSION` in `packages/architecture/src/deployment.rs` | Shape and census rules of the deployment-evidence profile. |
| Cargo workspace `0.6.0-dev` | `Cargo.toml` | The compiler line's version, inherited by every member crate through `version.workspace`; every crate inherits `publish = false` from the workspace package table and carries no repository URL. |
| Meson project `0.6.0-dev` | `meson.build` | Build-system definition version; mirrors the Cargo workspace version rather than diverging from it. The specification paper is a Meson subproject with its own version, `1.0.0` in `papers/attestation/meson.build`, and it is that subproject version that names the rendered PDF. |

A tag and its version are one fact: the tree declares the series of the highest name minted at its height, and a patch name names the commit it points at without moving a carrier. Patch tags take the form `0.6.N-dev` and leave `Cargo.toml` at the minor's `.0` until a minor release.

## Licensing

Code is licensed under the GNU Affero General Public License, version 3 only (`LICENSE-CODE`; `license = "AGPL-3.0-only"` in the workspace package table). The papers and documents are released under CC0 1.0 Universal (`LICENSE-DOCS`). Under ADR-011, third-party dependencies carry permissive licences compatible with that distribution, and a copyleft dependency enters the first-party build graph only through a separate repository policy decision.
