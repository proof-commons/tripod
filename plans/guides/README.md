# Implementation Guides

This directory contains concept guides for upcoming implementation batches.

A concept guide sketches the mission, scope, and tranche structure of a
future batch before that batch is chartered. It is planning input only: the
backlog remains the execution queue, and the owners named in the backlog
authority table remain authoritative on their subjects.

## Index · `tbl:guides:index`

| Guide | Status |
|---|---|
| [guide_seven_concept.md](guide_seven_concept.md) | Concept — P2-012/P2-013 analyzed pilots and Phase-2 exit |
| [guide_eight_concept.md](guide_eight_concept.md) | Concept — Phase-3 typed Elements target contract |

The Guide-9 concept is not carried here: adding it breached the plan-tree
hard byte cap, so it stays in the batch's session archive and the executed
guide's obligations are recorded in the backlog gate record instead.

## Guide rule · `rule:guides:authority`

A concept guide never overrides the backlog, a package contract, a phase
card, an ADR, or an accepted decision. When a chartered batch completes, its
durable evidence lives in the backlog gate record and Git history, not in
the guide.
