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

## The maturity closure (Guide-14)

The maturity link is closed here over the bytes a spend would run, not over the structures that produced them. Two fixture deployments are linked through the public entries — the compiler's validated announcement plan, the composed announcement record, a candidate constructor over that record's own static subtree, the bound deployment bridge, the issued singleton and the architecture's declaration of it — and the announcement leaf's linked program is encoded once and then parsed back through the target's own decoder. From that point every question is asked of the parse, and the parse is required to re-encode to the bytes it came from. A test that read the structure it is checking would pass when the structure was wrong; this one fails when the bytes are wrong, which is the only failure a spend can have.

Four things are checked over those bytes. The census of removed and kept checks reads the decoded instructions directly: the three primitives the reduction removed are absent, every positional introspection is preceded by a literal zero, no push carries any of the four byte strings a predecessor-program literal could have been, and each of the thirteen kept checks is found as its own instruction pattern — the self-position pin, input zero's asset, explicit amount and script version, the two script-version equalities inside the authentications, the two tweak relations over the internal key the leaf itself pushes, the two lead comparisons, output zero's asset and explicit amount, and the committed operator key with its verification. The same census is driven the other way: a program rebuilt from the decoded instructions with one kept check's instruction removed, or with one removed primitive inserted, is refused by name. The discharge table is located rather than restated — each emitted component is re-emitted through the public builders from values recovered out of the decoded pushes, and found at the range the published closure claims, while the deployment row, the model-scope rows and constructibility locate no bytes at all and the operator authorization's in-script half does. The golden evidence is recomputed from the linked bytes with public-point arithmetic supplied by the conformance package rather than by the linker: the leaf hash of those bytes, the outer branch with the metadata leaf, the tagged tweak over the internal key read from the leaf's own four pushes, and the tweaked point, each compared with what the bundle carries. The second deployment's bytes are then required to differ from the first's only inside the encoded spans of the instructions that moved.

Four adoption vectors are built as real target transactions, each with the census of the outputs its inputs spend. One carries the announcement beside further inputs and outputs of another asset at positions the leaf never names, and is accepted. Three are refused, and each names the field that refuses it: the singleton spent at input one, the successor placed at output one, and an output zero left short of the amount the leaf pins. Every verdict is derived by comparing the observation with the literals decoded out of the leaf, so the leaf's own bytes decide it and nothing is written down for them to agree with.

Half of that gate is node-free and half is not, and the record says which. What the bytes decide, they decide here. Output zero's program is authenticated by a curve relation over witness-supplied metadata rather than by a byte equality, and the operator's signature is a witness fact, so neither is settled by any of this. Each vector therefore retains the exact submission subject a run would send, and an ignored, environment-gated test names the run that would send them. That run is outstanding: no pod carries the target toolchain today, and until one does the native half of the adoption gate stays unanswered rather than approximated.
