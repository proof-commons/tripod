# Static Reviews

This directory archives the external static reviews commissioned against
named trees of this repository, kept verbatim as the record of what was
found and when.

Each review opened a findings register in the backlog, and a later batch
remediated those findings. The review is the evidence that produced the
register; the register and the backlog remain the owners of what was done
about it.

Archived reviews are excluded from the load-bearing documentation weight
budget and accounted against the separate archive budget instead, under the
weight rule in [the plans README](../README.md).

## Index · `tab:reviews:index`

| Review | Tree reviewed | Findings register | Remediating batch |
|---|---|---|---|
| [review-2-0.2.3-dev.md](review-2-0.2.3-dev.md) | 0.2.3-dev | backlog §5.2 (SR2) | Guide 7 |
| [review-3-0.3.1-dev.md](review-3-0.3.1-dev.md) | 0.3.1-dev | backlog §5.3 (SR3/S2) | Guide 8 |
| [review-4-0.3.2-dev.md](review-4-0.3.2-dev.md) | 0.3.2-dev | backlog §5.4 (R2) | Guide 9 |
| [review-5-0.3.3-dev.md](review-5-0.3.3-dev.md) | 0.3.3-dev | backlog §5.5 (SR5) | Guide 10 |
| [review-6-0.3.4-dev.md](review-6-0.3.4-dev.md) | 0.3.4-dev | backlog §5.6 (SR6) | Guide 11 |
| [review-7-0.3.6-dev.md](review-7-0.3.6-dev.md) | 0.3.6-dev | Guide-12 preflight register (guide §3) | Guide 12 |

## Review rule · `rule:reviews:authority`

A review never overrides the backlog, a package contract, a phase card, an
ADR, or an accepted decision.

An archived review is evidence about the exact tree it names, not a current
claim about the repository. Its findings were remediated in the batch named
above, so a defect it reports has very likely already been repaired: read it
as a record of that tree's state, and read the backlog for the current one.
Where a review and a present-day owner disagree, the owner is right and the
review is history.
