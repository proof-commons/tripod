# Implementation Guides

This directory archives the implementation guides written for this
repository: the executed guides that chartered a completed batch, and the
concept guides that sketched a batch before it was chartered.

An executed guide is the guidance a batch was actually run against, kept
verbatim as the historical record of what that batch was asked to do. A
concept guide sketches the mission, scope, and tranche structure of a future
batch before that batch is chartered. Both are planning input only: the
backlog remains the execution queue, and the owners named in the backlog
authority table remain authoritative on their subjects.

Archived guides are excluded from the load-bearing documentation weight
budget and accounted against the separate archive budget instead, under the
weight rule in [the plans README](../README.md).

## Executed guides · `tab:guides:executed`

| Guide | Batch |
|---|---|
| [guide_four.md](guide_four.md) | T6–T9 repair, C1-008/P2-007 proof-plan reacceptance |
| [guide_five.md](guide_five.md) | C1-009/P2-010 placement, carriers, and layout |
| [guide_six.md](guide_six.md) | P2-011/C1-010/C1-013 relation-indexed coverage closure |
| [guide_seven.md](guide_seven.md) | P2-012/P2-013 scoped analyzed programs and Phase-2 exit |
| [guide_eight.md](guide_eight.md) | Typed Elements target contract and capability adapters |
| [guide_nine.md](guide_nine.md) | Target-native primitive conformance and tapscript instruction core |
| [guide_ten.md](guide_ten.md) | STATE constructor and exact wide-arithmetic prototypes |
| [guide_eleven.md](guide_eleven.md) | Guide-11 preflight register and public declassification |
| [guide_twelve.md](guide_twelve.md) | Guide-12 preflight register and end-to-end compact ASH |
| [guide_thirteen.md](guide_thirteen.md) | Guide-13 preflight register and end-to-end live receipt transfer |

## Concept guides · `tab:guides:concepts`

| Guide | Sketch |
|---|---|
| [guide_seven_concept.md](guide_seven_concept.md) | P2-012/P2-013 analyzed pilots and Phase-2 exit |
| [guide_eight_concept.md](guide_eight_concept.md) | Phase-3 typed Elements target contract |
| [guide_nine_concept.md](guide_nine_concept.md) | Target-native primitive conformance |
| [guide_ten_concept.md](guide_ten_concept.md) | STATE constructor and wide-arithmetic prototypes |
| [guide_eleven_concept.md](guide_eleven_concept.md) | Public declassification and confidential-to-public lifecycle |
| [guide_twelve_concept.md](guide_twelve_concept.md) | Phase-4 end-to-end compact ASH |
| [guide_thirteen_concept.md](guide_thirteen_concept.md) | Phase-5 end-to-end live receipt transfer |
| [guide_fourteen_concept.md](guide_fourteen_concept.md) | Phase-6 end-to-end STATE and maturity announcement |
| [guide_confidential_funding_concept.md](guide_confidential_funding_concept.md) | Phase-5 confidential test materialization and funding protocol |

## Feature requests · `tab:guides:feature-requests`

| Document | Request |
|---|---|
| [guide_thirteen_feature_requests.md](guide_thirteen_feature_requests.md) | The three Guide-12 gaps blocking the negative evidence half, addressed to the numbered Guide 13 |

A feature-request document is correspondence to the guide author: it
records what a completed batch established the guide itself must change
before some class of evidence can be discharged. Like the guides, it is
planning input only.

A concept guide and the executed guide of the same number are different
documents: the concept is the earlier sketch, the executed guide is what the
batch was run against, and they may disagree. The executed guide is the
record of the charter; neither is a record of the outcome.

## Guide rule · `rule:guides:authority`

A guide never overrides the backlog, a package contract, a phase card, an
ADR, or an accepted decision. When a chartered batch completes, its durable
evidence lives in the backlog gate record and Git history, not in the guide.

An archived guide describes what a batch was asked to do at the time it was
written. It is not a current claim about the repository: where a guide and a
present-day owner disagree, the owner is right and the guide is history.
