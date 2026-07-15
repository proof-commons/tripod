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
- `adr/` - Architecture decision records for the repository tooling.

## Command-line output contract

Every executable in this workspace follows
[ADR-010](adr/010-command-line-output-contract.md): stdout carries only JSON
result data (single object or NDJSON) and refuses a terminal; assets are
written to paths given by arguments such as `--output`; everything else —
help, usage, diagnostics, panics — is JSON on stderr; exit codes are
0 (success), 1 (failure), 2 (usage).

## Requirements

- a Rust toolchain (cargo, edition 2024)

## Licensing

Code is licensed under `LICENSE-CODE`; the papers and documents under
`LICENSE-DOCS`.
