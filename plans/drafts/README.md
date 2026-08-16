# Adopted-Source Drafts

This directory archives externally authored normative drafts the project
has accepted for integration, kept verbatim as the record of what was
supplied. A draft is planning input only: it becomes binding through the
ADR or package change that integrates it, and until then the existing
ADRs and owners remain authoritative.

Archived drafts are excluded from the load-bearing documentation weight
budget and accounted against the separate archive budget instead, under
the weight rule in [the plans README](../README.md).

## Index · `tbl:drafts:index`

| Draft | Subject | Integration owner |
|---|---|---|
| [label-calculus.md](label-calculus.md) | The documentation/source label calculus as owners, judgments, and inference rules | backlog §13 (supersedes the ADR-012/ADR-013 statement) |
| [environment-kinds.md](environment-kinds.md) | The environment kind registry: 248 names into 148 kinds | backlog §13 |
| [identity-adjudication.md](identity-adjudication.md) | The digest/identity adjudication procedure | backlog §13 (refines ADR-016) |
| [interchange-conventions.md](interchange-conventions.md) | Deterministic CBOR/CDDL interchange: envelope, registry, satisfaction | backlog §13 |

## Draft rule · `rule:drafts:authority`

A draft never overrides an ADR, the backlog, a package contract, or an
accepted decision. Where a draft and a present-day owner disagree, the
owner is right until the integrating change lands; the integration batch
records every deliberate divergence.
