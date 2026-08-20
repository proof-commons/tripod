# Tripod

Tripod is a contract closure compiler: it carries a typed, target-independent realization of a smart contract through analysis, emission, linking, and transaction construction to exact target bytes, with every seam machine-checked.

A compiler is proved by what it compiles. This tree therefore carries, beside the compiler, the contract that proves it: the attestation — a reserve-backed, two-class, conserved, burnable receipt, specified abstractly in the *Attestation* paper, realized as a typed architecture manifest and an executable reference model, and compiled toward an Elements/Liquid covenant. The attestation contract is the non-trivial exemplar the compiler's build-out is verified against; every capability the compiler claims must first hold on it, end to end.

## Layout

- `papers/attestation/` - LaTeX source of the *Attestation* specification.
- `docs/attestation/` - companion documentation outside the paper source tree.
  - `human.md` - plain-language companion report.
  - `realization.md` - the Elements/Liquid realization document.
- `packages/execwrap/` - Rust stdout/stderr routing wrapper used by the paper build.
- `packages/flatten-latex-main/` - Rust utility for flattening the paper entrypoint.
- `packages/cli-common/` - Shared Rust CLI scaffolding (implements ADR-010).
- `packages/model/` - Executable state-machine model of the attestation contract.
- `packages/architecture/` - Typed normative architecture manifest of the realization.
- `packages/realization/` - Target-independent typed semantic realization,
  currently scoped to compact ASH and live receipt transfer.
- `packages/compiler/` - Target-independent relation, proof, disclosure,
  lifecycle, placement, and layout analysis over the realization.
- `packages/target-elements/` - Typed static Liquid/Elements target contract
  and its development deployment binding. It owns target requirements, not
  evidence: no evidence is produced or recorded here, and no production
  target evidence exists anywhere in this repository.
- `packages/tapscript/` - Elements tapscript backend: the compiler-to-target
  capability adapter, together with typed instructions, checked stack items,
  a serializer and parser, and an abstract stack validator over the reviewed
  primitive contracts. Nothing here executes anything.
- `packages/target-elements-conformance/` - Target-native conformance harness.
  It runs a caller-selected external executor against the reviewed primitive
  fixtures and records what that executor answered. Development native
  evidence is produced and recorded here, separately from the static
  contract; a mock executor can never satisfy its gate.
- `packages/linker/` - Resolves the backend's relocatable bundle into a
  deterministic candidate linked bundle: symbol resolution, the typed
  reference graph and its cycle policy, structured relocation, taptree
  assembly, and relation-carrier closure.
- `packages/transaction/` - Derives the candidate compact-ASH transaction ABI
  from a linked bundle and constructs exact target transaction bytes from it.
  It holds no key, signs nothing, and performs no network submission: a
  sponsor signs through an external capability adapter.
- `packages/artifacts/` - Generator/checker for the generated derivative artifacts.
- `packages/labels/` - Repository-wide documentation-label registries and checks.
- `adr/` - Architecture decision records for the repository tooling.

## Command-line output contract

Every executable in this workspace follows
[ADR-010](adr/010-command-line-output-contract.md): stdout carries only JSON
result data (single object or NDJSON) and refuses a terminal; assets are
written to paths given by arguments such as `--output`; everything else —
help, usage, diagnostics, panics — is JSON on stderr. Most commands use only
exit classes 0 (success), 1 (failure), and 2 (usage); `execwrap` additionally
relays child statuses 3-255 after successful wrapper startup, while codes 0-2
retain the shared meanings.

## Generated artifacts

`packages/model/generated/` is owned by the `tripod-artifacts`
crate. Regenerate with:

```sh
meson compile -C build generate-artifacts
```

Check generated artifacts and repository labels without writing:

```sh
meson compile -C build lint
```

Regenerate planning label registers explicitly:

```sh
meson compile -C build generate-label-registers
```

Direct Cargo invocations of the generator/checker binaries are build-system
internals: they require the full role-tagged ADR-014 census that Meson derives.

## Requirements

- Meson and Ninja
- Python 3
- a Rust toolchain with Cargo, supporting Rust 1.88 and edition 2024
- TeX Live with `xelatex`, `biber`, and `latexmk`
- Git, for the lint census and build-time paper provenance stamps
- a POSIX shell environment with the standard file, comparison, hashing, and text utilities used by `scripts/`

## Building

```sh
meson setup build
meson compile -C build attestation
```

The rendered PDF lands in `archive/rendered/` and a flattened single-file
`.tex` (for arXiv submission or diffing) in `archive/flattened/`.

### Paper provenance

Before rendering, Meson derives four values from committed Git state
(`tripod-document-stamps`) and renders them into a generated
`stamps.tex` and a `source-date-epoch` file:

- **DocumentUUID** — first 128 bits of a canonical SHA-256 digest over the
  exact declared paper inputs (XMP `DocumentID`). Each input is a canonical
  repository-relative path strictly beneath `papers/attestation`, and its
  digest bytes are read from the committed Git blob and verified equal to the
  rendered worktree bytes — a divergence is a dirty-subtree failure.
- **InstanceUUID** — first 128 bits of the Git SHA-1 tree object ID for
  `papers/attestation`, formatted with UUID grouping and no bit rewriting
  (XMP `InstanceID`). Removing the hyphens recovers the tree-object prefix,
  which Git must resolve uniquely or publication aborts.
- **date** — UTC date of the latest commit touching an exact paper input
  (the visible frontmatter date).
- **timestamp** — UTC time of the latest commit touching the paper subtree,
  used as `SOURCE_DATE_EPOCH` and all XMP dates.

The publication command derives and renders checked-out committed `HEAD`. A
different `--tree-ref` is rejected unless it resolves to the same commit as
`HEAD`; selected-ref publication from an un-checked-out revision is not
currently supported.

A dirty paper subtree blocks canonical publication. The two UUIDs are XMP
identities; the PDF trailer `/ID` is left toolchain-derived (no PDF-rewrite
step). The values are source-provenance identities, not hashes of the final
PDF bytes; byte reproducibility is established separately by
[scripts/check-document-reproducibility.sh](scripts/check-document-reproducibility.sh).

Rust checks:

```sh
meson test -C build --print-errorlogs
```

## CI

Every lane of the gate is a Meson test, so `meson test` is the gate and `meson test --list` is the authoritative lane list: fmt, clippy (`-D warnings`), per-package debug and release tests, the ADR-014 census audit, the non-writing generated-artifact and label gates, plan-tree, forbidden-text and hash-citation checks, the mocked Meson contract, the node-free executor classification tests, `cargo audit` (skipped, not passed, when it is not installed), and a clean-tree check.

The runner-agnostic entry point is [scripts/ci.sh](scripts/ci.sh), a shim
that configures a build directory with the TeX toolchain mocked, runs
`meson test`, and prints the timing report; arguments are forwarded, so
`scripts/ci.sh --suite lint` and `scripts/ci.sh cargo-clippy` are
content-scoped runs of the same gate. Toolchain and dependency policy is
[ADR-011](adr/011-toolchain-and-dependency-policy.md).

## Security and execution trust

Current first-party packages are public-data tools and do not accept production
secret material. Repository source and build definitions are executable;
untrusted contributions must be run in a secretless isolated environment
established outside the untrusted checkout. `execwrap` is not a sandbox, and
architecture/model success is not deployment readiness.

See [SECURITY.md](SECURITY.md) and
[ADR-015](adr/015-public-data-and-execution-trust.md).

## Version registry

The repository carries several intentionally distinct version numbers.
Each has its own meaning and bump discipline; exactly one is a
derivation of another (the realization binding):

| Version | Where | Meaning |
|---|---|---|
| Specification `v1.0.0` | `papers/attestation/main.tex` (owned by `sections/00_title.tex`) | The released abstract economic specification. Pinned by the manifest's specification binding and anchor-set hash. |
| Realization version `0.6.0-dev` | `docs/attestation/realization.md` masthead, manifest envelope | The tracked binding to the compiler line: `major.minor` copied from the workspace version, patch always zero, prerelease carried verbatim. Patch-blind and content-blind — the behavioural hash alone witnesses denotation stability, and the versioning gates enforce both halves. |
| Architecture schema `17` | manifest envelope | Shape of the exported manifest DTO. Envelope metadata, never a hash input. |
| Attestation wire schema | (`[RZ-rem:manifest:discriminants]`) | Canonical attestation query encoding. |
| Deployment-profile schema `2` | `packages/architecture/src/deployment.rs` | Shape and census rules of the deployment-evidence profile. |
| Cargo workspace `0.4.0-dev` | `Cargo.toml` | The compiler line's version, inherited by every member crate through `version.workspace`; crates are `publish = false` and carry no repository URL. A tag and its version are one fact: the tree declares the series of the highest name minted at its height, and a patch name names the commit it points at without moving a carrier. |
| Meson project `0.4.0-dev` | `meson.build` | Build-system definition version; mirrors the cargo workspace version rather than diverging from it. The specification paper is a meson subproject with its own version (`1.0.0`), and it is that subproject version — not this one — that names the rendered PDF. |

## Licensing

Code is licensed under `LICENSE-CODE`; the papers and documents under
`LICENSE-DOCS`.
