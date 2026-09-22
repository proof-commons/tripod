# Adopted-Source Drafts

This directory archives externally authored normative drafts the project has accepted for integration, kept verbatim as the record of what was supplied. A draft is planning input only: it becomes binding through the ADR or package change that integrates it, and until then the existing ADRs and owners remain authoritative.

Archived drafts are excluded from the load-bearing documentation weight budget and accounted against the separate archive budget instead, under the weight rule in [the plans README](../README.md).

The directory stands even when empty: it is the template for the next adopted draft, so the census, the weight class, and the rules below do not have to be rebuilt when one arrives. The four founding drafts — the label calculus, the environment-kind registry, the identity-adjudication procedure, and the interchange conventions — were retired into their adopting records and are now the normative bodies of [ADR-019](../../adr/019-label-calculus.md) through [ADR-022](../../adr/022-interchange-conventions.md); their draft bytes remain in Git history.

## Index · `tab:drafts:index`

| Draft | Subject | Integration owner | Deliberate divergences |
|---|---|---|---|
| [Relay-witness split study](relay-witness-split-study.md) | Relay admission rule, witness consumers, transport alternatives, counted effects and evidence obligations. | [ADR-025](../../adr/025-relay-admissible-announcement-witness.md) and route track `T11-100` through `T11-109`. | Recorded node revision strings replaced by "the binary revision string recorded at `packages/vectors/fixtures/maturity-run-of-record/RUN-REPORT`"; absolute Elements citation prefixes replaced by `elements: `; the bare checkout sentence and source-version sentence name the Elements source tree at version 28.99.0; the absolute study-worktree path replaced by "the study worktree"; backtick delimiters removed from seven bracketed byte/width arrays on supplied lines 49, 207 and 248 because the labels gate parsed them as imported citations; otherwise verbatim. |
| [Constant-elision proposal](relay-witness-constant-elision-proposal.md) | Constant-elision witness transport, emission policy, historical replay and accepted-evidence route. | [ADR-025](../../adr/025-relay-admissible-announcement-witness.md) and route track `T11-100` through `T11-109`. | The opening status sentence identifies the document as the design proposal ratified by ADR-025 without naming a role or review; otherwise verbatim, including illustrative Rust sketches. |

A new draft joins this index with its subject and its integration owner, and joins the census in `meson.build` in the same commit.

## Draft rule · `rule:drafts:authority`

A draft never overrides an ADR, the backlog, a package contract, or an accepted decision. Where a draft and a present-day owner disagree, the owner is right until the integrating change lands; the integration batch records every deliberate divergence.
