# Planning Decisions

This directory records accepted cross-package implementation choices.

Planning decisions are subordinate to normative source and implemented ADRs.

## Ownership · `sec:decisions:ownership`

A decision owns one durable implementation choice and its consequences. It does
not own protocol semantics, package-local API details, roadmap sequencing,
current task status, or unresolved target research.

## Index · `tbl:decisions:index`

| Record | Status | Choice |
|---|---|---|
| [001](001-typed-rust-source.md) | Accepted | Typed Rust is the first-party semantic source. |
| [002](002-realization-layer.md) | Accepted | Introduce a target-independent realization layer. |
| [003](003-tapscript-first.md) | Accepted | Preserve multiple-backend boundaries; implement tapscript first. |
| [004](004-translation-validation.md) | Accepted | Validate each released bundle rather than initially trusting the compiler. |
| [005](005-value-representation.md) | Accepted | Permit value-representation latitude; keep closed asset identity rigid initially. |
| [006](006-transaction-abi.md) | Accepted | Generate one canonical transaction/witness ABI per target bundle. |
| [007](007-petgraph-graph-substrate.md) | Accepted | Use full-featured Petgraph directly for every first-party graph. |

## Record form · `rule:decisions:form`

Each record contains choice, fixed constraints, consequences, interpretations
not authorized, and supersession conditions. API sketches are illustrative
unless implementation and an ADR freeze them.

## Numbering · `rule:decisions:numbering`

Decision numbers are permanent and never reused. A replacement receives a new
number and names what it supersedes.

## Labels · `rule:decisions:labels`

Each decision mints one primary local label:

```text
dec:<area>:<name>
```

Other sections may mint local plan labels as needed. Decision labels remain
non-normative and non-identity-bearing, but their mints and citations are
mechanically checked under ADR-013.

## Machine use · `rem:decisions:machine-use`

Decision Markdown is not compiler or release configuration. Implemented policy
moves into typed source, tests, and, when repository-wide, an ADR.