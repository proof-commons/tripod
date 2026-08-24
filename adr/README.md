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
| [ADR-019](019-label-calculus.md) | Decided and adopted; implemented for the authorship warrant species | The normative label calculus, with this repository's adoption parameters, hyphenated-area amendment, and implementation standing. |
| [ADR-020](020-environment-kinds.md) | Decided and adopted; implemented | The normative environment-kind registry, with this repository's migration adjudication, recorded extension set, and attestation standing. |
| [ADR-021](021-identity-adjudication.md) | Decided and adopted; implemented for current identity policy, with evidence and release surfaces activating with their consumers | The normative identity adjudication procedure, with this repository's local recipe convention, current identities, separation migration, and implementation standing. |
| [ADR-022](022-interchange-conventions.md) | Decided and adopted; normative now and unimplemented by design until an externally consumed document exists | The normative wire-format discipline for externally consumed documents, with the ADR-010 boundary, executor-protocol stop, alignment records, and consumer-triggered implementation rule. |
| [ADR-023](023-script-tree-label-carrier.md) | Decided and implemented | The script tree joins the label carrier: two amended ADR-019 rows putting `scripts/*.py` under `DOC` and scanning Python comments, strings excluded. |

## Numbering · `rule:adr:numbering`

ADR numbers are permanent and never reused.

A replacement receives a new number and names the records it supersedes.
Implemented ADR identities remain stable even when their prose is rewritten
for density.

## Labels · `rule:adr:labels`

Each numbered ADR is an independent label owner under
(`[ADR019-sig:labels:owners]`); mint discipline follows
(`[ADR019-judg:labels:minting]`).

The calculus is the normative body of (`[ADR019-dec:labels:adoption]`). A
clause of the calculus, an adoption parameter, or an amendment is cited
with the `ADR019` prefix.

The identity adjudication procedure is the normative body of
(`[ADR021-dec:identity:adoption]`). Its clauses and this repository's
recipe convention, current identities, and separation migration are
cited with the `ADR021` prefix.

The interchange conventions are the normative body of
(`[ADR022-dec:interchange:adoption]`). Their clauses, the boundary against
ADR-010, the executor-protocol stop, the alignment records, and the
consumer-triggered implementation rule are cited with the `ADR022`
prefix. The namespace labels they govern are not documentation labels.

A plan citing an ADR label uses the ADR owner prefix, for example:

```text
(`[ADR010-rule:output:streams]`)
(`[ADR011-rule:toolchain:locked]`)
(`[ADR019-rule:labels:areas]`)
```

ADR labels are policy-owned upstream labels and are linted when imported.

## Writing rules · `rule:adr:writing`

An ADR written directly for this repository states:

- context;
- decision;
- consequences;
- rejected alternatives;
- implementation or verification gate.

ADRs do not repeat package plans, roadmap sequencing, or long implementation
catalogues.

An ADR carrying an externally authored normative text instead records its status, edition, and provenance; states repository-specific amendments and implementation standing as an overlay; and carries the full normative text as its body.

Every repository amendment to such a text is one entry of that overlay, under its own label, stating exactly what it changes. Nothing reaches the adopted body transparently: where the body diverges from the adopted bytes at all — the heading depth the folding requires among it — an entry names the divergence, so the difference between the body and the text as adopted is exactly the set of listed entries. Several amendments never share one entry, and an entry never stands without a label.

## Machine use · `rem:adr:machine-use`

ADR Markdown is not compiler, linker, transaction, or release input.

Implemented policy is represented in code, configuration, and tests. ADRs
explain and govern that implementation.
