# Tripod

Tripod is a contract closure compiler: it carries a typed, target-independent realization of a smart contract through analysis, emission, linking, and transaction construction to exact target bytes, with every seam machine-checked.

A compiler is proved by what it compiles. This tree therefore carries, beside the compiler, the contract that proves it: the attestation — a reserve-backed, two-class, conserved, burnable receipt, specified abstractly in the *Attestation* paper, realized as a typed architecture manifest and an executable reference model, and compiled toward an Elements/Liquid covenant. The attestation contract is the non-trivial exemplar the compiler's build-out is verified against; every capability the compiler claims must first hold on it, end to end.

## Layout

- `papers/attestation/` - LaTeX source of the *Attestation* specification.
- `docs/attestation/` - the documents the specification is realized by.
  - `realization.md` - the Elements/Liquid realization document.
  - `human.md` - plain-language companion report.
- `packages/execwrap/` - Rust stdout/stderr routing wrapper used by the paper build.
- `packages/flatten-latex-main/` - Rust utility for flattening the paper entrypoint.
- `packages/cli-common/` - Shared Rust CLI scaffolding (implements ADR-010).
- `packages/model/` - Executable state-machine model of the attestation contract.
- `packages/architecture/` - Typed normative architecture manifest of the realization.
- `packages/realization/` - Target-independent typed semantic realization
  shared by executable-model conformance and future compiler analysis.
- `packages/artifacts/` - Generator/checker for the generated derivative artifacts.
- `packages/labels/` - Repository-wide documentation-label registries and checks.
- `adr/` - Architecture decision records for the repository tooling.

## Command-line output contract

Every executable in this workspace follows
[ADR-010](adr/010-command-line-output-contract.md): stdout carries only JSON
result data (single object or NDJSON) and refuses a terminal; assets are
written to paths given by arguments such as `--output`; everything else —
help, usage, diagnostics, panics — is JSON on stderr; exit codes are
0 (success), 1 (failure), 2 (usage).

## Generated artifacts

`packages/model/generated/` is owned by the `tripod-artifacts`
crate. Regenerate with:

```sh
cargo run -p tripod-artifacts --bin generate-all
```

Check without writing (tests and CI use the same non-writing path):

```sh
cargo run -p tripod-artifacts --bin check-generated | jq .
```

Regenerate planning label registers explicitly:

```sh
cargo run --locked \
  -p tripod-labels \
  --bin generate-label-registers \
  -- --repository-root . --output-root .
```

Check repository labels without writing:

```sh
cargo run --locked \
  -p tripod-labels \
  --bin check-labels \
  -- --repository-root . | jq .
```

## Requirements

- meson and ninja
- a Rust toolchain (cargo, edition 2024)
- TeX Live with `xelatex`, `biber`, and `latexmk`

## Building

```sh
meson setup build
meson compile -C build attestation
```

The rendered PDF lands in `archive/rendered/` and a flattened single-file
`.tex` (for arXiv submission or diffing) in `archive/flattened/`.

Rust checks:

```sh
meson test -C build
```

## CI

The runner-agnostic CI entry point is [scripts/ci.sh](scripts/ci.sh):
fmt, clippy (`-D warnings`), debug and release tests, the non-writing
generated-artifact gate, `cargo audit` (when installed), and a
clean-tree check — all Cargo invocations `--locked`. Toolchain and
dependency policy is [ADR-011](adr/011-toolchain-and-dependency-policy.md).

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
| Cargo workspace `0.1.0-dev` | `Cargo.toml` | The compiler line's version, inherited by every member crate through `version.workspace`; crates are `publish = false` and carry no repository URL. A tag and its version are one fact: the tree declares the series of the highest name minted at its height, and a patch name names the commit it points at without moving a carrier. |
| Meson project `0.1.0-dev` | `meson.build` | Build-system definition version; mirrors the cargo workspace version rather than diverging from it. The specification paper is a meson subproject with its own version (`1.0.0`), and it is that subproject version — not this one — that names the rendered PDF. |

## Licensing

Code is licensed under `LICENSE-CODE`; the papers and documents under
`LICENSE-DOCS`.
