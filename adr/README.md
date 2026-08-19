# Architecture Decision Records

This directory records implemented or repository-wide engineering policy.

ADRs govern current first-party code. They are subordinate to normative
protocol artifacts but take precedence over planning documents.

## Authority · `sec:adr:authority`

The relevant authority order is:

1. Attestation economic specification;
2. realization conformance contract;
3. typed architecture;
4. implemented ADRs;
5. planning decisions;
6. package, phase, research, and backlog documents.

Plans may propose future policy. An ADR records policy accepted for the
repository itself.

## Record index · `tab:adr:index`

| Record | Status | Policy |
|---|---|---|
| [ADR-010](010-command-line-output-contract.md) | Decided and implemented | First-party command-line streams, diagnostics, and exit classes. |
| [ADR-011](011-toolchain-and-dependency-policy.md) | Decided and implemented | Rust, dependency, locking, unsafe-code, target-compatibility, and reproducibility policy. |
| [ADR-014](014-meson-lint-census-and-stamps.md) | Decided and implemented | Build-system-owned lint census, argument-passed inputs, and stamp-file dependency edges. |
| [ADR-015](015-public-data-and-execution-trust.md) | Decided and implemented | Public-data interfaces, untrusted execution, crash artifacts, and future secret boundaries. |
| [ADR-017](017-path-scope-and-host-filesystem-trust.md) | Decided and implemented | Repository/build path scope, central prohibition of symlinks and submodules, source-derived reference confinement, lexical output roles, and the host filesystem/TOCTOU boundary. |
| [ADR-018](018-upstream-elements-workspace.md) | Decided; adoption proceeds in the Elements checkout | Upstream Elements workspace shape: pristine-mirror default branch, fix and notes branches, derived merged worktree, the gripe register, and executed-revision provenance. |
| [ADR-019](019-label-calculus.md) | Decided and implemented; checker re-engineering tracked as DI-003 | Adoption of the archived label calculus as normative, with this repository's adoption parameters and the hyphenated-area amendment. |
| [ADR-020](020-environment-kinds.md) | Decided and implemented | Adoption of the archived environment-kind registry as the kind vocabulary, with the migration adjudication and the recorded extension set. |
| [ADR-021](021-identity-adjudication.md) | Decided and implemented for current identity policy; the evidence and release surfaces activate with their consumers | Adoption of the archived identity adjudication procedure as normative, with this repository's local recipe convention, current identities, recorded separation migration, and recorded divergences. |
| [ADR-022](022-interchange-conventions.md) | Decided; normative now and unimplemented by design until an externally consumed document exists | Adoption of the archived interchange conventions as the wire-format discipline for externally consumed documents, with the boundary against ADR-010, the executor-protocol stop, the alignment records, and the no-implementation standing. |

## Numbering · `rule:adr:numbering`

ADR numbers are permanent and never reused.

A replacement receives a new number and names the records it supersedes.
Implemented ADR identities remain stable even when their prose is rewritten
for density.

## Labels · `rule:adr:labels`

Each numbered ADR is an independent label owner under
(`[PLAN-sig:labels:owners]`); mint discipline follows
(`[PLAN-judg:labels:minting]`).

The calculus itself is adopted by (`[ADR019-dec:labels:adoption]`), which
makes the archived draft under `plans/drafts/` its normative text. A
clause of the calculus is therefore cited with the `PLAN` prefix, at the
draft; ADR-019 is cited for an adoption parameter or an amendment.

The identity adjudication procedure is adopted the same way by
(`[ADR021-dec:identity:adoption]`). A clause of the procedure is cited
with the `PLAN` prefix, at the draft; ADR-021 is cited for this
repository's recipe convention, its current identities, or the recorded
separation migration.

The interchange conventions are adopted the same way by
(`[ADR022-dec:interchange:adoption]`). A clause of the conventions is
cited with the `PLAN` prefix, at the draft; ADR-022 is cited for the
boundary against ADR-010, the executor-protocol stop, the alignment
records, or the no-implementation standing. The namespace labels that
draft governs are not documentation labels and are never cited as any.

A plan citing an ADR label uses the ADR owner prefix, for example:

```text
(`[ADR010-rule:output:streams]`)
(`[ADR011-rule:toolchain:locked]`)
(`[ADR019-rule:labels:areas]`)
```

ADR labels are policy-owned upstream labels and are linted when imported.

## Writing rules · `rule:adr:writing`

Each ADR states:

- context;
- decision;
- consequences;
- rejected alternatives;
- implementation or verification gate.

ADRs do not repeat package plans, roadmap sequencing, or long implementation
catalogues.

## Machine use · `rem:adr:machine-use`

ADR Markdown is not compiler, linker, transaction, or release input.

Implemented policy is represented in code, configuration, and tests. ADRs
explain and govern that implementation.

