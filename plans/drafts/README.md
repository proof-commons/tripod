# Adopted-Source Drafts

This directory archives externally authored normative drafts the project has accepted for integration, kept verbatim as the record of what was supplied. A draft is planning input only: it becomes binding through the ADR or package change that integrates it, and until then the existing ADRs and owners remain authoritative.

Archived drafts are excluded from the load-bearing documentation weight budget and accounted against the separate archive budget instead, under the weight rule in [the plans README](../README.md).

The directory stands even when empty: it is the template for the next adopted draft, so the census, the weight class, and the rules below do not have to be rebuilt when one arrives. The four founding drafts — the label calculus, the environment-kind registry, the identity-adjudication procedure, and the interchange conventions — were retired into their adopting records and are now the normative bodies of [ADR-019](../../adr/019-label-calculus.md) through [ADR-022](../../adr/022-interchange-conventions.md); their draft bytes remain in Git history.

## Index · `tab:drafts:index`

No draft is currently archived. A new draft joins this index with its subject and its integration owner, and joins the census in `meson.build` in the same commit.

## Draft rule · `rule:drafts:authority`

A draft never overrides an ADR, the backlog, a package contract, or an accepted decision. Where a draft and a present-day owner disagree, the owner is right until the integrating change lands; the integration batch records every deliberate divergence.
