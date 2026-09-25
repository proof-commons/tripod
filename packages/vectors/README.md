# `tripod-vectors`

Owns the canonical evidence substrate for compact ASH, live transfer, and the maturity announcement: semantic fixtures, independently derived expectations, the mutation registry, the relation-indexed coverage matrix, and the plan binding compact ASH to one exact linked bundle and transaction ABI (Guide-12 §16–§19). Its twenty-two `maturity_` modules include the run-of-record corpora, the five candidate-scope safety, continuity, root-history, public-recovery, and resource reports, and `maturity_refinement` (Guide-14).

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

The maturity link is closed here over the bytes a spend would run, not over the structures that produced them. Three fixture deployments are linked through the public entries — the compiler's validated announcement plan, the composed announcement record, a candidate constructor over that record's own static subtree, the bound deployment bridge, the issued singleton and the architecture's declaration of it — and the announcement leaf's linked program is encoded once and then parsed back through the target's own decoder. From that point every question is asked of the parse, and the parse is required to re-encode to the bytes it came from. A test that read the structure it is checking would pass when the structure was wrong; this one fails when the bytes are wrong, which is the only failure a spend can have.

Four things are checked over those bytes. The census of removed and kept checks reads the decoded instructions directly: the three primitives the reduction removed are absent, every positional introspection is preceded by a literal zero, no push carries any of the four byte strings a predecessor-program literal could have been, and each of the thirteen kept checks is found as its own instruction pattern — the self-position pin, input zero's asset, explicit amount and script version, the two script-version equalities inside the authentications, the two tweak relations over the internal key the leaf itself pushes, the two lead comparisons, output zero's asset and explicit amount, and the committed operator key with its verification. The same census is driven the other way: a program rebuilt from the decoded instructions with one kept check's instruction removed, or with one removed primitive inserted, is refused by name. The discharge table is located rather than restated — each emitted component is re-emitted through the public builders from values recovered out of the decoded pushes, and found at the range the published closure claims, while the deployment row, the model-scope rows and constructibility locate no bytes at all and the operator authorization's in-script half does. The golden evidence is recomputed from the linked bytes with public-point arithmetic supplied by the conformance package rather than by the linker: the leaf hash of those bytes, the outer branch with the metadata leaf, the tagged tweak over the internal key read from the leaf's own four pushes, and the tweaked point, each compared with what the bundle carries. The second deployment's bytes are then required to differ from the first's only inside the encoded spans of the instructions that moved. The third deployment commits the x-only key of a published test signer instead of a fill, and its linked bytes are read back at the site the leaf's first instruction pair consumes: a key of meaningless material is one nobody holds a scalar for, so a signature that verifies under it cannot be produced at all, and committing a key somebody can sign under is what makes that verification reachable for the wave that supplies the transaction ABI.

Four adoption vectors are built as real target transactions, each with the census of the outputs its inputs spend. One carries the announcement beside further inputs and outputs of another asset at positions the leaf never names, and is accepted. Three are refused, and each names the field that refuses it: the singleton spent at input one, the successor placed at output one, and an output zero left short of the amount the leaf pins. Every verdict is derived by comparing the observation with the literals decoded out of the leaf, so the leaf's own bytes decide it and nothing is written down for them to agree with.

Half of that gate is node-free and half is not, and the record says which. What the bytes decide, they decide here: output zero's program is authenticated by a curve relation over witness-supplied metadata rather than by a byte equality, and the operator's signature is a witness fact, so neither is settled by any comparison over the leaf's literals. The other half is a run, and an ignored, environment-gated test performs it. That test issues a singleton asset on a target node, funds a coin of it at the constructor's own output program, funds the shapes' foreign coins in the target's own reserve asset, and rebuilds each of the four shapes over that funding — the positions, amounts and programs are the vector's, and the outpoints, the two assets and the witnesses are the run's, because those are what a node requires to be real — before submitting them. What the node establishes is the commitment chain: its interpreter accepts the revealed leaf against the funded output's key, which is the whole path from the linked bytes through the static subtree and the output key to the control block that the node-free half only recomputed, and it then refuses at the committed operator key's verification, which is the leaf's first instruction pair. The archived whole-item run cannot establish an accepted case, whereas the pinned variable-schedule run of record establishes one accepted submission; three facts of the archived deployment explain its limit: the committed operator key is public, meaningless material no scalar belongs to, and the leaf verifies a signature under it; the pinned singleton asset is a fixture constant, while a node derives an asset identity from its own issuing outpoint; and the announcement's witness roles are populated and its successor's representation nonce searched by nothing in that archived exercise, which the linked bundle's own outstanding set records rather than leaves to be discovered. So every case is refused before any compared field is read. Two rules of the target's own decide where each case stops, and the run records both: its relay policy admits no tapscript stack item as wide as the announcement's predecessor-metadata role declares, so a spend carrying that role at its declared width is refused before any script runs and the placeholder is capped at the limit; and its amount verification refuses an output of explicit value zero before it examines any witness, so the one retained shape that carries such an output — an output zero short of the amount the leaf pins, which below a pinned amount of one can only be zero — is refused before the leaf while the other three are refused inside it. The variable-schedule run of record supplies the accepted control, while the linked bundle retains the relay-width rule as a construction obligation.

The STATE constructor-continuity projector reconstructs both constructors from submitted bytes under the retained witness schedule, and its validated continuity report keeps the schedule and byte identity with each comparison (`0.6.243-dev`). The archived whole-item corpus preserves the relay-refused run; the pinned second corpus at `packages/vectors/fixtures/maturity-variable-run-of-record/` admits an accepted variable-schedule submission with exact mined readback on `0.6.245-dev`. That acceptance was observed with `-minrelaytxfee=0` and `-blockmintxfee=0`; the accepted-byte row's classification stands in the Phase-6 card's accepted-byte classification record.
