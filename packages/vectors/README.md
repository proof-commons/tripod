# `tripod-vectors`

Owns the canonical compact-ASH evidence substrate: semantic fixtures,
their independently derived expectations, the mutation registry, the
relation-indexed coverage matrix, and the plan that binds all of them to
one exact linked bundle and one exact transaction ABI (Guide-12 §16–§19).

The package owns evidence, not semantics. Every expectation it states is
derived from the layer that defines the thing being expected — relation
identities come from `realization`, coverage requirements come from
`compiler`, target bytes come from `transaction` over a `linker` bundle —
and none is derived from the candidate being judged.

Nothing here executes a target. A semantic fixture deliberately contains
no target index, program, tapleaf, control block, transaction byte, or
sponsor wallet state (§17.2); materialization is a separate, later step
that reads the bundle and the ABI. Target acceptance and semantic
acceptance are two verdicts, and this package keeps them two (§1.4).

The distinction between a canonical evidence subject and an experimental
one is a type, not a convention. Only a subject admitted into the
`CompactAshEvidencePlan` can carry an evidence-grade claim; anything
constructed ad hoc is typed as experimental and cannot be promoted
without going through the plan's checked constructor (§16.4).

## The live-transfer half (Guide-13)

Three studies, deliberately separate documents with separate schemas,
because Guide-13 §13.6 forbids either satisfying the other:

- **Safety.** The complete §15 matrix transcribed as typed rows — one
  hundred and eight of them across seven tables, each carrying polarity,
  mutation layer, expected boundary, intended relation, and collateral.
  The evidence plan classifies every row into exactly one standing, and
  the counts are recomputed from that classification rather than
  tabulated anywhere. The whole first-party half is discharged: each row
  had its owning validator driven twice, once on an honest input and
  once with one stated change, and the refusal had to name the row's own
  class. Of the target half, most rows are answered by an acceptance of
  the row's own shape; one is answered by a determinism observation,
  which is counted in its own bucket and never added to the acceptance
  figure; and the rest are waiting on a run, with the plan saying which.
- **Disclosure minimality.** Five pairs, each grown from one semantic
  fixture with both members built by the same function, scored one §16.2
  condition at a time. Its verdict is per pair rather than one token for
  the matrix: three pairs satisfy all ten conjuncts on recorded
  acceptances of both members' shapes, and the two that do not name the
  conjunct they fail and the lane whose run is missing. No member of any
  pair was itself submitted, and the report says so at every identity it
  cites.
- **Resources.** Prediction and observation as separate figures over the
  same exact bytes, never one constant used twice.

The run of record is a transcription and not evidence. It records what
one real node did once, so that a reader without a node can see the
target's own words, and it discharges no row.

Three rows of §15 are answered by neither a validator nor a target. The
two sponsor-report rows ask what this workspace's own canonical bytes
publish, and one row asks for a program mixing operations — an input the
architecture admits no value of, recorded as a closure of the operation
vocabulary rather than counted as an unanswered refusal. The third is
the deterministic-public-fixture-openings row, whose own gate is the
byte-identity contract: it is answered by a first-party recomputation
that reproduced a fixture's openings byte for byte, filed under a
standing minted for determinism observations so that it can never be
read as an acceptance.
