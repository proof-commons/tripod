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
  figure. Seven negative rows in the two constructor tables are answered
  by target refusals at their declared boundaries, each bound to its row
  through the subject, carrier, control and boundary rule. The remaining
  target rows wait on a run, with the plan saying which.
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

The runs of record, including the two mutant runs, transcribe what one
real node did once in each run, so a reader without a node can see the
target's own words. A run answers a row only through an exact refusal
bound to that row's declared facts; its text alone answers none.

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

## Keyed birth capture (Guide-15 Wave 1)

`keyed_birth_native` constructs the A0–A3 experiment without admitting a run.
Each placement requests two sponsor outputs of 100,000 satoshis. Output zero
pays a locally constructed parent with equal 10,000-satoshi covenant outputs
at positions zero and one, 79,000 satoshis change and an explicit 1,000-satoshi
fee. Output one funds both children of that pair: each retains a 10,000-satoshi
successor, 99,000 satoshis sponsor change and a 1,000-satoshi fee. The second
placement gets fresh funding. Amounts are fixture choices, not funding floors;
these rows claim no issuance, peg-in or flags coverage. Fourteen requests run
in order: fund/sign-parent/submit-parent/sign-negative/submit-negative/sign-control/
submit-control, repeated for the second placement. Parent readback is retained
before a child is constructed. The birth leaf appends true to the typed fragment,
under the reviewed leaf version and the constructor's fixed unspendable key.

The following is the exact capture command, run from the clean repository root
**inside `localhost/tripod-native:f44`**, never on the `CoreOS` host or in the
document image. The orchestrator mounts the pinned toolchain read-only at
`/elements` (including its `PROVENANCE.txt` and shared library), the checkout
at `/src`, and supplies a fresh writable `$BIRTH_REPORT_DIR`, writable `$TMPDIR`
and `$HOME`, and the usual isolated Cargo toolchain/cache/target mounts.
`$ELEMENTS_INTENDED_TIP` is the full source-provenance tip from that toolchain's
`PROVENANCE.txt`; `$BIRTH_NETWORK_ID` and `$BIRTH_GENESIS_ID` are the reviewed
64-character lowercase hexadecimal deployment identifiers, in display order.
The directory must not contain a previous capture, timing file, report or manifest.

```sh
env \
  TRIPOD_LIVE_EXECUTOR="$PWD/scripts/elements-native-executor.sh" \
  TRIPOD_LIVE_REPORT_DIR="$BIRTH_REPORT_DIR" \
  TRIPOD_LIVE_SUITE_SHORT_SHA="$(git rev-parse --short=12 HEAD)" \
  TRIPOD_LIVE_SUITE_COMMIT="$(git rev-parse HEAD)" \
  TRIPOD_LIVE_SUITE_TREE="$(git rev-parse HEAD^{tree})" \
  TRIPOD_LIVE_NETWORK_ID="$BIRTH_NETWORK_ID" \
  TRIPOD_LIVE_GENESIS_ID="$BIRTH_GENESIS_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD=/elements/bin/elementsd \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI=/elements/bin/elements-cli \
  ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK=/elements/src/test/functional \
  ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP="$ELEMENTS_INTENDED_TIP" \
  ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID="$BIRTH_NETWORK_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET=1 \
  ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY=/elements/lib/libsecp256k1.so \
  ELEMENTS_NATIVE_EXECUTOR_FEE_POLICY=node-defaults \
  ELEMENTS_NATIVE_EXECUTOR_MATURITY_OBSERVATION=1 \
  ELEMENTS_NATIVE_EXECUTOR_PYTHON=python3 \
  ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX= \
  ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_BASE= \
  ELEMENTS_NATIVE_EXECUTOR_LOCAL_TOPICS= \
  cargo test -p tripod-vectors --profile gate --test maturity_native \
    keyed_birth::keyed_birth_campaign_runs_against_a_real_target -- --ignored --exact --nocapture
```

These are all application-specific environment inputs. `PATH` resolves Cargo
and Python; `HOME`, `TMPDIR`, `CARGO_HOME`, `CARGO_TARGET_DIR` and `RUSTUP_HOME`
belong to the wrapper's toolchain and writable-directory setup. The observation
switch uses the existing executor's policy/readback route; it does not change
the protocol. The executor deadline is 900 seconds. The driver hashes the actual
`elementsd` and retains the sibling toolchain provenance file inside the capture.
The birth tests are included from `tests/keyed_birth/mod.rs` in the existing
`maturity_native` integration target. This submodule preserves Cargo's automatic
test discovery and the root Meson target census.

The file set follows the existing report-capture carrier:
`<short>.report.capture`, `<short>.report.timing`, `RUN-REPORT`, and
`MANIFEST.sha256` covering those three files. `diagnostics/` additionally retains
executor diagnostics. The enhanced capture binds target, deployment, handshake,
observed environment, source identities, and every request/response link. Its
birth payload retains frozen parent and child bytes, sponsor signing echoes,
funding facts, policy and startup arguments, relay result and exact reason,
consensus route and evidence, accepted txid/wtxid, block hash/height and raw mined
readback. The timing and run report record elapsed wall and eligibility; a
failed run retains the transcript before asserting failure. A preflight or
artifact-write failure may leave only partial files and is always ineligible.

A run of record must report that file set and its digests, suite commit/tree,
toolchain provenance and executable digest, image, deployment, protocol/adapter/
node versions, policy origins, startup arguments, fourteen-operation census,
all four classifications, exact refusal texts, accepted readbacks, wall time
and any timeout or infrastructure failure. Relay and mempool floors are observed;
the block floor remains the pinned source's declaration. A negative's refused
relay route is script-verdict evidence, not a claim of direct block rejection.
Attribution requires the exact `OP_EQUALVERIFY` error, the birth leaf's sole
equality at byte offset eight, a matching mined control, and exclusion of the
sponsor's equality and signature sites. The public-key hash check excludes its
P2WPKH equality; CHECKSIG cannot emit that equality error. This is not independent
ECDSA verification or a claim that the negative executed its sponsor signature.
Ambiguous errors, earlier-layer refusals, altered pairs, signing mismatches,
missing readbacks, open classifications, incomplete transcripts or provenance
disagreements make a run ineligible. Capture eligibility is not admission:
the later admission step binds `Wave1TargetPacket` and `BirthCampaignManifest`
to the reviewed archive and measurement home.

## Key-free continuation capture (Guide-15 Wave 1)

`keyed_continuation_native` stages eleven exchanges: two-output sponsor funding,
parent signing/submission, then signing/submission for wrong program, wrong
version, displaced successor and the control, in that order. Sponsor output one
is reserved for all four children. The mined parent has a 10,000-satoshi primary
output at birth position zero, a 10,000-satoshi alternative-program output at
one, 79,000 satoshis sponsor change and a 1,000-satoshi fee. Each child spends
parent output zero plus the 100,000-satoshi reserve, with 10,000 satoshis successor,
99,000 satoshis alternative-program change and a 1,000-satoshi fee. Displacement
exchanges the two complete spendable outputs. These amounts are fixtures, not
floors; the alternative-program change is disposable experiment funding.

The primary leaf is born first, continue first and final true. The alternative
recipe inserts an empty push/drop immediately before final true. Both commitments
are reconstructed under the reviewed leaf version and the same fixed unspendable
internal key, never supplied as programs. No semantic key literal or key-separation
claim is made; the manifest's public-key slot is the all-zero placeholder for this
key-free campaign. Wrong-key, mixed-key and coexistence rows remain deferred.

Run the following exact command from the clean repository root **inside
`localhost/tripod-native:f44`**, with the same read-only `/elements`, `/src` and
isolated writable toolchain/cache mounts described for birth above. Supply fresh
`$CONTINUATION_REPORT_DIR`, deployment identifiers `$CONTINUATION_NETWORK_ID` and
`$CONTINUATION_GENESIS_ID`, and `$ELEMENTS_INTENDED_TIP` from `PROVENANCE.txt`.
Neither the `CoreOS` host nor the document image can run this capture.

```sh
env \
  TRIPOD_LIVE_EXECUTOR="$PWD/scripts/elements-native-executor.sh" \
  TRIPOD_LIVE_REPORT_DIR="$CONTINUATION_REPORT_DIR" \
  TRIPOD_LIVE_SUITE_SHORT_SHA="$(git rev-parse --short=12 HEAD)" \
  TRIPOD_LIVE_SUITE_COMMIT="$(git rev-parse HEAD)" \
  TRIPOD_LIVE_SUITE_TREE="$(git rev-parse HEAD^{tree})" \
  TRIPOD_LIVE_NETWORK_ID="$CONTINUATION_NETWORK_ID" \
  TRIPOD_LIVE_GENESIS_ID="$CONTINUATION_GENESIS_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD=/elements/bin/elementsd \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI=/elements/bin/elements-cli \
  ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK=/elements/src/test/functional \
  ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP="$ELEMENTS_INTENDED_TIP" \
  ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID="$CONTINUATION_NETWORK_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET=1 \
  ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY=/elements/lib/libsecp256k1.so \
  ELEMENTS_NATIVE_EXECUTOR_FEE_POLICY=node-defaults \
  ELEMENTS_NATIVE_EXECUTOR_MATURITY_OBSERVATION=1 \
  ELEMENTS_NATIVE_EXECUTOR_PYTHON=python3 \
  ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX= \
  ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_BASE= \
  ELEMENTS_NATIVE_EXECUTOR_LOCAL_TOPICS= \
  cargo test -p tripod-vectors --profile gate --test maturity_native \
    keyed_continuation::native::keyed_continuation_campaign_runs_against_a_real_target -- --ignored --exact --nocapture
```

These are all application-specific environment inputs. The wrapper supplies
`PATH`, `HOME`, `TMPDIR`, `CARGO_HOME`, `CARGO_TARGET_DIR` and `RUSTUP_HOME` for the
toolchain and writable directories. The existing executor protocol and adapter
are unchanged; its deadline is 900 seconds. This test is a submodule of the
existing `maturity_native` integration target, not a new Cargo target.

The five files are `<short>.report.capture`, `<short>.report.timing`,
`<short>.report.capture.timing`, `RUN-REPORT` and `MANIFEST.sha256`, which covers
all four other files. `diagnostics/` retains executor diagnostics separately.
The enhanced capture's payload includes `wire_journal`: every complete request
and response serialized with the protocol's JSON serializer, with journal-local
operation/request/response links, handshake and environment. This includes full
funding facts, signing echoes, frozen submission bytes, policy and startup
arguments, relay result/reason, consensus route/evidence, accepted txid/wtxid,
block hash/height and raw mined readback. Admission replays that JSON, never
debug text. Incomplete runs retain available responses and null missing responses
before asserting ineligibility. A preflight or write failure may leave partial
files and is ineligible.

A run of record reports those files and digests, suite commit/tree, image,
deployment, executable digest and source provenance, protocol/adapter/node
versions, policy origins and startup arguments, eleven-exchange census, four
classifications, exact refusal texts/readbacks, wall and infrastructure failures.
Relay/mempool policy is observed; the block floor remains a source declaration.
The control requires exact mined readback. Negatives require the exact verifying
equality error, a mined same-coin control and replayed one-coordinate pairs:
birth stays zero, sponsor hash equality is checked, wrong version preserves the
payload, and program/displacement rows preserve version one. The three equality
offsets are 8, 15 and 16; an undifferentiated error alone identifies none of them.
The classification is coordinate-based isolation, not a native opcode trace or
independent ECDSA verification. Refused relay is not direct-block rejection.
Ambiguous or sponsor-site errors, earlier-layer refusals (including version-zero
policy refusal), missing readbacks, malformed links, incomplete responses or open
rows make the run ineligible. Repair such a subject or stop its obligation;
capture eligibility never constitutes admission. No issuance, peg-in, funding
floor, native stack metrics or key-dependent measurement is claimed.


## Keyed constructor and continuation capture (Guide-15 Wave 1)

`keyed_native` stages measurement (c)'s native path subjects and measurement (b)'s keyed rows in one history. All keyed programs come from `construct_keyed` and descriptor reconstruction. The two fixed deployment handles are defined only in the ignored test. No constructor home decision or measurement admission is made here.

The 25-exchange census is one seven-output `fund_sponsor`, four parent sign/submit pairs, and eight subject sign/submit pairs. Sponsor outputs 0–3 create four distinct mined parents in this order: single k, mixed first k, mixed second k, l. Each parent reuses the birth layout: equal 10,000-satoshi keyed outputs at positions 0 and 1, 79,000 sponsor change and a 1,000 fee. Only position zero is spent. Output 4 funds the single-k subjects, output 5 the mixed pair, and output 6 the separate l control. Each reserve is 100,000 satoshis. These are fixture amounts, not floors. Every child returns 99,000 sponsor change and pays a 1,000 fee; output zero carries 10,000 for a single handle or 20,000 for two handles. Assets, fees and unrelated outputs stay fixed within each pair.

Submission order is wrong control parity, uncommitted leaf, key-path attempt, wrong key, canonical k control, mixed keys, same-key two-input control, separate l control. Every negative is constructed, signed and submitted before its consuming control. The canonical k control is shared by the constructor negatives and wrong key, and is also the k half of independent coexistence.

| Row | Coordinate and boundary |
|---|---|
| Canonical k | Exact mined script-path control on the first parent |
| Wrong control parity | Flip only bit zero of the original control; commitment refusal before clauses |
| Uncommitted leaf | Flip one byte of the offered key literal, retaining the original control; commitment refusal before clauses |
| Key-path attempt | Replace only the keyed witness by a single 64-byte zero signature; exact Schnorr refusal, no absent-discrete-log proof |
| Wrong key | Replace only output zero's program by the reconstructed l recipe; payload equality refusal |
| Mixed keys | Spend mixed first k plus l, output zero under k; compare with mixed first k plus mixed second k |
| Same-key control | Two distinct birth-zero k handles, output zero under k; exact mined control |
| Independent coexistence | Separate canonical k and l transactions accepted on this one history; no mixed-key acceptance claim |

The mixed row changes the second handle's semantic key. Since a funded coin's program is immutable, this necessarily uses an equivalent different second predecessor, with its outpoint, leaf/control, transaction identities and sponsor signature as declared collateral. The first k coin, sponsor reserve, values/assets, fee and output programs are fixed. This is a same-first-coin pair, not a claim that one outpoint can change keys. Both second predecessors are mined and their controls authenticated; both birth indexes are zero. All rows are expressible with the existing executor; none is approximated or omitted.

Witness-only constructor mutations keep the complete non-witness transaction fixed; their collateral is witness bytes, wtxid, weight and observation. Wrong-key collateral is the successor commitment, sponsor signature and transaction identities/weight. Signing echoes and sponsor hash preimages are checked, not independently ECDSA-verified. The keyed equality offsets are 42 (birth), 49 (version), 50 (payload). Equality attribution uses reconstructed one-coordinate pairs and a mined control, never an undifferentiated error alone. Mixed attribution additionally uses the authenticated l predecessor and same-key control.

The executor's `key_path_rejection` detector requires every input to use the key path. A P2WPKH sponsor therefore leaves this row's raw layer as `script_path_rejection`. That raw response is retained unchanged. The campaign separately classifies `KeyPathSignatureRefusal` only for exact `Invalid Schnorr signature` text, the reconstructed one-item keyed witness and the mined same-coin script control; the sponsor uses ECDSA and neither keyed leaf contains a signature opcode. A signature-size error is not accepted as that boundary. Commitment rows require exact `Witness program hash mismatch`; equality rows require exact `Script failed an OP_EQUALVERIFY operation`. Both recognized script wrappers are accepted; ambiguous, wrong-site or earlier-layer responses remain open. A relay refusal is not a direct-block rejection.

Run this exact command from the clean repository root **inside `localhost/tripod-native:f44`**, with the read-only `/elements` toolchain and `/src` repository plus isolated writable toolchain/cache mounts described for birth. Supply a fresh `$KEYED_REPORT_DIR`, deployment identifiers `$KEYED_NETWORK_ID` and `$KEYED_GENESIS_ID`, and `$ELEMENTS_INTENDED_TIP` from `PROVENANCE.txt`. Never run in the document image or on the `CoreOS` host.

```sh
env \
  TRIPOD_LIVE_EXECUTOR="$PWD/scripts/elements-native-executor.sh" \
  TRIPOD_LIVE_REPORT_DIR="$KEYED_REPORT_DIR" \
  TRIPOD_LIVE_SUITE_SHORT_SHA="$(git rev-parse --short=12 HEAD)" \
  TRIPOD_LIVE_SUITE_COMMIT="$(git rev-parse HEAD)" \
  TRIPOD_LIVE_SUITE_TREE="$(git rev-parse HEAD^{tree})" \
  TRIPOD_LIVE_NETWORK_ID="$KEYED_NETWORK_ID" \
  TRIPOD_LIVE_GENESIS_ID="$KEYED_GENESIS_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD=/elements/bin/elementsd \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI=/elements/bin/elements-cli \
  ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK=/elements/src/test/functional \
  ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP="$ELEMENTS_INTENDED_TIP" \
  ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID="$KEYED_NETWORK_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET=1 \
  ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY=/elements/lib/libsecp256k1.so \
  ELEMENTS_NATIVE_EXECUTOR_FEE_POLICY=node-defaults \
  ELEMENTS_NATIVE_EXECUTOR_MATURITY_OBSERVATION=1 \
  ELEMENTS_NATIVE_EXECUTOR_PYTHON=python3 \
  ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX= \
  ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_BASE= \
  ELEMENTS_NATIVE_EXECUTOR_LOCAL_TOPICS= \
  cargo test -p tripod-vectors --profile gate --test maturity_native \
    keyed_native::keyed_campaign_runs_against_a_real_target -- --ignored --exact --nocapture
```

These are all application-specific environment inputs. The wrapper supplies `PATH`, `HOME`, `TMPDIR`, `CARGO_HOME`, `CARGO_TARGET_DIR` and `RUSTUP_HOME`; executor deadline is 900 seconds. The test remains a submodule of `maturity_native`, not a new Cargo target. No adapter or protocol change is needed.

Expected files: `<short>.report.capture`, `<short>.report.timing`, `<short>.report.capture.timing`, `RUN-REPORT`, `MANIFEST.sha256` (covering the other four), plus separate `diagnostics/`. The capture includes public descriptors and every complete executor request/response in `wire_journal`, with links, handshake, environment, funding facts, signing echoes, frozen bytes, policy/startup arguments, relay result/reason, consensus route/evidence, accepted txid/wtxid and block hash/height/raw readback. Replay uses wire JSON alone, never debug text. Partial runs are retained before ineligibility is asserted; preflight/write failures can leave incomplete files and are ineligible.

A run of record reports file digests, suite commit/tree, image, deployment, executable/source provenance, protocol/adapter/node versions, policy origins/startup arguments, 25 exchanges, all eight classes, exact errors and mined readbacks/heights, wall and infrastructure failures. Every parent and control requires exact mined readback. Coexistence additionally requires the l control at a later height than the k control. Missing/malformed responses or links, recipe drift, incomplete execution, ambiguous or earlier-layer refusals, or any open row makes the run ineligible. `admitted` stays false; measurement records follow a separate admission step. No native stack metrics, issuance/peg-in coverage or universal key-path impossibility is claimed.


## Keyed cost comparison and focused capture

`keyed_cost::cost_packet` reads the three admitted archives and projects exact isolated birth (with final truth), isolated continuation (with final truth), key-free composed and complete keyed programs through the typed stack engine. Instruction count, operation charges and validation charge are static predictions. Exact leaf/output/control bytes, `CompactSize` witness framing, control depth and complete transaction serialization are separate quantities. Standard transaction weight is four times stripped bytes plus witness bytes, not discounted virtual size. Every archived subject retains its node-read weight beside that serialization; differences are findings, never silently normalized. The key-free continuation archive contains birth as well as continuation, so its weight is not an isolated-continuation observation. Sponsor signatures and output shapes differ between archives; subtracting their transaction weights would not measure a clause.

The descriptor's public semantic key, internal-key derivation, root and output commitment are fixed construction data, not an amortized transaction-weight deduction. Every script-path input repeats the complete leaf (including the 34-byte key push/drop) and its control. The packet keeps positional sponsor witnesses and output proof framing separate. Native stack peaks, maximum element, validation budget and opcode timing remain explicitly absent. Submission resource reports do not independently measure script length or initial stack count; those fields are null in these archives. The birth archive uses its already-admitted journal bridge because its original capture predates complete wire JSON; no new debug decoder is introduced here.

The focused campaign has twelve exchanges: funding, parent signing/submission, isolated-continuation signing/submission; fresh funding, resource-parent signing/submission, oversized signing/submission, same-coin resource-control signing/submission. Both fundings have two sponsor outputs; output zero funds two equal parent outputs and output one reserves the child sponsor coin. Each negative is submitted before its control. The standalone continuation supplies missing exact native weight for its eight-byte leaf. The resource leaf is `Drop; true`, with no covenant clauses: only the initial argument length changes from one to 521 bytes, with witness bytes, wtxid, weight and response as collateral. This observes target relay/resource admission; it does not claim a minimal policy floor. The expected size error is `mandatory-script-verify-flag-failed (Push value size limit exceeded)` beside exact mined control. The exact `bad-witness-nonstandard` relay reason beside the mined control is instead classified only as a witness-policy refusal; it makes no consensus size-site claim. Other refusals stay open, and relay and consensus outcomes are never conflated. An altered report changes only a reported resource dimension beside the unchanged admitted source and is refused offline by `validate_resource_report`; it requires no fabricated executor result or native submission.

Run the following only inside `localhost/tripod-native:f44`, with read-only `/elements` toolchain and `/src` repository and the isolated writable mounts described for birth. Supply fresh `$KEYED_COST_REPORT_DIR`, deployment `$KEYED_COST_NETWORK_ID` and `$KEYED_COST_GENESIS_ID`, and `$ELEMENTS_INTENDED_TIP` from `PROVENANCE.txt`. This staging step does not run it.

```sh
env \
  TRIPOD_LIVE_EXECUTOR="$PWD/scripts/elements-native-executor.sh" \
  TRIPOD_LIVE_REPORT_DIR="$KEYED_COST_REPORT_DIR" \
  TRIPOD_LIVE_SUITE_SHORT_SHA="$(git rev-parse --short=12 HEAD)" \
  TRIPOD_LIVE_SUITE_COMMIT="$(git rev-parse HEAD)" \
  TRIPOD_LIVE_SUITE_TREE="$(git rev-parse HEAD^{tree})" \
  TRIPOD_LIVE_NETWORK_ID="$KEYED_COST_NETWORK_ID" \
  TRIPOD_LIVE_GENESIS_ID="$KEYED_COST_GENESIS_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD=/elements/bin/elementsd \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI=/elements/bin/elements-cli \
  ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK=/elements/src/test/functional \
  ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP="$ELEMENTS_INTENDED_TIP" \
  ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID="$KEYED_COST_NETWORK_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET=1 \
  ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY=/elements/lib/libsecp256k1.so \
  ELEMENTS_NATIVE_EXECUTOR_FEE_POLICY=node-defaults \
  ELEMENTS_NATIVE_EXECUTOR_MATURITY_OBSERVATION=1 \
  ELEMENTS_NATIVE_EXECUTOR_PYTHON=python3 \
  ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX= \
  ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_BASE= \
  ELEMENTS_NATIVE_EXECUTOR_LOCAL_TOPICS= \
  cargo test -p tripod-vectors --profile gate --test maturity_native \
    keyed_cost::keyed_cost_campaign_runs_against_a_real_target -- --ignored --exact --nocapture
```

These are all application-specific environment inputs. The wrapper supplies `PATH`, `HOME`, `TMPDIR`, `CARGO_HOME`, `CARGO_TARGET_DIR` and `RUSTUP_HOME`. Executor deadline is 900 seconds. The ignored test lives under the existing `maturity_native` target; no executor or protocol change is needed.

Expected file set: `<short>.report.capture`, `<short>.report.timing`, `<short>.report.capture.timing`, `RUN-REPORT`, `MANIFEST.sha256`, plus separate `diagnostics/`. The capture retains complete wire-JSON responses, request/response links, handshake, environment, executable provenance, funding, policy/startup arguments, exact transactions and raw observations. A run of record must report all five digests, suite commit/tree, image and deployment, protocol/adapter/node versions, twelve exchanges, control mined readbacks/heights, serialized versus node-read weights, any mismatch, oversized relay reason and consensus route separately, wall and infrastructure failures. Missing weight on either control, incomplete replay, wrong signing echo, missing exact mined control, or an ambiguous size refusal makes the capture ineligible; mismatches remain findings for review rather than being erased. Files are written before asserting eligibility, while preflight/write failures can leave incomplete files. `admitted` remains false. No measurement record, native stack metric, funding floor or scaling claim follows from staging.

## Keyed funding and standardness staging (measurement e)

`keyed_funding` stages measurement (e) in `plans/guides/guide_fifteen.md` and the funding subjects in `plans/drafts/wave1-route-study.md` without running or admitting them. The policy asset comes from the existing `fund_sponsor` reserve, not generic `pay` or a newly issued asset. Programs come from `keyed_constructor::construct_keyed` under its fixed NUMS recipe; the deployment key is defined only in the ignored test. SOURCE has one sponsor input, keyed output zero, ordinary P2WPKH change and an explicit empty-program fee. MERGE has a mined SOURCE's exact output-zero outpoint plus one sponsor input; its keyed successor remains output zero. Setup requires ordinary default relay, exact signed-byte readback and unchanged policy. These fixture amounts are not adopted funding floors.

The immutable schedule is SOURCE control then MERGE control, SOURCE/MERGE pairs for each handle point, SOURCE/MERGE pairs for each fee offset, then two MERGE open-output controls. It has 36 subjects: 17 SOURCE and 19 MERGE. Every SOURCE point uses three exchanges (`fund_sponsor`, `sign_sponsor`, `submit`); every MERGE point uses five (`fund_sponsor` with two outputs, SOURCE sign/submit, MERGE sign/submit). Total: 146 exchanges, 36 funding requests, 55 independent sponsor coins, 55 signing requests and 55 submissions. Each coin holds 100,000 policy-asset satoshis. Fresh funding prevents an accepted point or direct-block retry from spending a later point's inputs. Coin identities, signatures and consequent transaction identities/weights are declared collateral, not an assertion of same-coin comparison. Asset, sponsor program, constructor, target, policy/startup arguments and per-role input/output shape are fixed; the two open-output rows explicitly change output count.

Both controls use a 10,000-satoshi handle and 1,000-satoshi explicit fee. Every MERGE setup SOURCE also uses these amounts and requires relay/template admission. The handle grid, fixed before capture, is `0, 1, 2, 10, 100, 299, 300, 301, 999, 1000, 10000` satoshis for both roles. The fee stays at 1,000 and change balances exactly. Zero is a separate consensus-positivity control, excluded from the positive threshold reading. MERGE always consumes a 10,000-satoshi handle; this grid varies its successor amount, not its predecessor's value or key.

For each role, let `F = ceil(max(reported relay rate, reported mempool rate) * ceil(control node-read weight / 4) / 1000)` in satoshis, with rates parsed exactly from BTC/kvB. Fee points are the fixed list `max(0,F-2), max(0,F-1), F, F+1, F+2`; repeated values after clamping remain separate observations. The constructor and 10,000-satoshi successor stay fixed, and change balances the fee. These are finite symbolic points decided before capture, resolved from the initial accepted controls; no response-dependent search or added points occur. The frozen control must pay strictly more than its reported floor. Each point also records its own exact node-read weight and computed floor: DER signature length can shift weight, so the control anchor is not silently asserted to be every point's exact floor. The declared block-assembly default remains separately retained in the policy; it is not relabeled as a node-reported relay rate.

The boundary reader requires a complete monotone sequence under the frozen policy. A handle refusal contributes only when relay says exactly `dust` and direct-block inclusion succeeds; a fee refusal contributes only with `min relay fee not met` or `mempool min fee not met` and direct-block inclusion. It reports last refused / first admitted sampled values and whether they differ by one satoshi; it never interpolates a gap or extrapolates beyond the grid. If all positive points relay, it reports the tested range 1–10,000 and no discovered positive threshold beyond consensus positivity. Unknown reasons, nonmonotone outcomes, missing policy/weight/readback or ambiguous observations leave the reading open. A reported fee floor is an arithmetic reference, not an observed threshold. Changed policy, constructor size, signature-weight distribution or input count reopens the measurement.

The ordinary-output control adds a distinct P2WPKH output of 1,000, keeps the successor at 10,000, change at 98,000 and fee at 1,000. Its redistribution neighbor moves 9,000 from successor to that ordinary output: successor 1,000, ordinary output 10,000, unchanged change and fee. The keyed program is unchanged and both inputs total 110,000. Both require ordinary relay/mined controls; they test lack of output closure and lack of value preservation in the two clauses, not a different covenant. No confidential output is proposed: `maturity_native/confidential.rs` supports its typed maturity sponsor construction, not arbitrary keyed carrier outputs with independently balanced range/surjection proofs. No proof mutation is therefore staged here.

Responses retain complete wire request/response objects, policy readings and startup arguments, signing echoes, raw bytes, relay allowed/reason, route, block evidence, txid/wtxid, height and exact mined readback. The existing maturity-observation adapter attempts ordinary relay/template inclusion first; after non-script relay refusal it separately attempts direct-block inclusion. A script refusal is not retried by that executor and remains open here; a timeout or unavailable direct-block path is never a consensus refusal. `ConsensusRefused` requires the adapter's typed pre-script refusal and its block-refusal marker, not just a relay error string. Successful ordinary relay yields template-mined evidence, not a second forced-block experiment. Raw responses are never rewritten to fit the expected boundary. Replay reconstructs every request and descriptor from wire JSON, including all setup SOURCE readbacks; no debug rendering is evidence.

The offline tests are registered under the existing `maturity_native` target; there is exactly one ignored funding capture test. It reuses the existing sponsor signing/readback, deterministic replay, capture guard, wire carrier and run-file helpers. The new module's `keyed_funding_new`, `keyed_funding_build`, `keyed_funding_step` and `replay_keyed_funding` spans expose construction coordinates and progress counts; completion/refusal events preserve typed outcomes without introducing measured durations in library code.

For a separately authorized capture, run this exact command from the clean repository root inside `localhost/tripod-native:f44`, with the read-only `/elements` installation and isolated writable toolchain/cache mounts described above. Supply a fresh `$KEYED_FUNDING_REPORT_DIR`, `$KEYED_NETWORK_ID`, `$KEYED_GENESIS_ID` and `$ELEMENTS_INTENDED_TIP` from the installation's `PROVENANCE.txt`. This staging step runs none of it.

```sh
env \
  TRIPOD_LIVE_EXECUTOR="$PWD/scripts/elements-native-executor.sh" \
  TRIPOD_LIVE_REPORT_DIR="$KEYED_FUNDING_REPORT_DIR" \
  TRIPOD_LIVE_SUITE_SHORT_SHA="$(git rev-parse --short=12 HEAD)" \
  TRIPOD_LIVE_SUITE_COMMIT="$(git rev-parse HEAD)" \
  TRIPOD_LIVE_SUITE_TREE="$(git rev-parse HEAD^{tree})" \
  TRIPOD_LIVE_NETWORK_ID="$KEYED_NETWORK_ID" \
  TRIPOD_LIVE_GENESIS_ID="$KEYED_GENESIS_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD=/elements/bin/elementsd \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI=/elements/bin/elements-cli \
  ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK=/elements/src/test/functional \
  ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP="$ELEMENTS_INTENDED_TIP" \
  ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID="$KEYED_NETWORK_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET=1 \
  ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY=/elements/lib/libsecp256k1.so \
  ELEMENTS_NATIVE_EXECUTOR_FEE_POLICY=node-defaults \
  ELEMENTS_NATIVE_EXECUTOR_MATURITY_OBSERVATION=1 \
  ELEMENTS_NATIVE_EXECUTOR_PYTHON=python3 \
  ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX= \
  ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_BASE= \
  ELEMENTS_NATIVE_EXECUTOR_LOCAL_TOPICS= \
  cargo test -p tripod-vectors --profile gate --test maturity_native \
    keyed_funding::keyed_funding_campaign_runs_against_a_real_target -- --ignored --exact --nocapture
```

Executor deadline is 3,600 seconds. The environment wrapper supplies `PATH`, `HOME`, `TMPDIR`, `CARGO_HOME`, `CARGO_TARGET_DIR` and `RUSTUP_HOME`. Expected files are `<short>.report.capture`, `<short>.report.timing`, `<short>.report.capture.timing`, `RUN-REPORT`, `MANIFEST.sha256` covering the other four, and separate `diagnostics/`. Complete and partial journals are retained before eligibility is asserted; failed preflight or writes may leave incomplete files. `eligible_for_review` requires completed replay, classified points and ordinary-relay controls (including both open-output rows), but does not itself settle a threshold; `admitted` is always false. No funding number, policy law or measurement record is accepted by staging.


## Keyed scaling staging (measurement f)

`keyed_scaling` stages measurement (f) in `plans/guides/guide_fifteen.md` in the route study's explicit policy-asset envelope. Every MERGE input consumes output zero of a distinct, exactly mined SOURCE transaction under the reconstructed public keyed descriptor. No sibling output counts as a handle. SOURCE preparation is separately tagged and costed. Each SOURCE uses one admitted P2WPKH sponsor, keyed output zero of 10,000 satoshis, ordinary change and a 1,000-satoshi explicit fee. MERGE uses one such fee sponsor, a 10,000-satoshi keyed successor, ordinary change and a 20,000-satoshi explicit fee; these are fixture values, not funding floors. Every response's policy and exact weight must establish that its own fee covers the reported floor. The explicit funding route is reused from measurement (e); other assets, confidential witnesses/outputs, issuance and peg-in are outside this fixed envelope.

Frozen count arithmetic starts from the admitted complete keyed weights 1331 and 1586 (the keyed cost record in `plans/research/state-constructor.md`): a marginal input is 255 units (41 stripped bytes times four plus 91 witness bytes). The one-input carrier has 1076 fixed units; reserving one more signature byte gives 1077 for a 72-byte low-S DER signature including sighash, with a 33-byte compressed public key. The larger input count adds eight units of `CompactSize` framing, yielding `1085 + 255*n`. Against the reviewed 400,000 standard-weight envelope (`packages/target-elements/src/resource.rs`), the proposed maximum is 1564 (worst-envelope weight 399905), its neighbour 1565 (400160), and the representative interior is 782. Actual signature sizes and node weights are retained, never forced to the upper envelope; any mismatch remains evidence. This is the nearest input-count neighbour in the fixed output/witness family, not a one-weight-unit pair. An observed failure at another boundary leaves the weight-boundary claim open. The maximum is provisional until capture and admission.

Submission order is standalone SOURCE (zero covenant inputs), MERGE 1, MERGE 2, MERGE 782, overweight MERGE 1565, then MERGE 1564. Each MERGE has disjoint fresh preparation coins: the executor may directly mine the policy-refused neighbour, so sharing its coins with the control would invalidate the experiment. There are 3915 SOURCE creations, 21 fundings (batches no larger than 255 outputs), 3920 signings and 3920 submissions: 7861 exchanges. The sole standalone SOURCE remains unspent. The largest complete request and response carriers are checked offline against the existing 4 MiB limits, with 64 KiB reserved for native metadata; no protocol or executor change and no count approximation is made. There is no exchange-count cap in the existing planner loop. The 3600-second deadline is a capture configuration, not a promised measured runtime; a timeout or oversize response retains a partial, ineligible capture rather than selecting fewer inputs. If that occurs, split into independently bound small/interior and endpoint captures, each with its own distinct SOURCE preparation; no split is asserted necessary without that evidence.

Default-relay controls require exact mined bytes, unchanged policy, exact node/serialized weight agreement and sufficient fee. The neighbour requires exact `tx-size` relay text; direct-block acceptance is classified separately from relay admission, while an exact `bad-txns-oversize` verdict is separately labelled consensus. Ambiguous errors, insufficient fee, mismatched readback or earlier refusals remain open. The keyed descriptor constructor has no transaction input-count limit; `transaction::construct::check_weight` checks the 4,000,000 consensus envelope and accepts both staged boundary carriers offline. There is therefore no existing typed N+1 construction refusal to stage at this policy bound. No covenant count law or new construction policy is invented.

For an independently authorized capture, run the following from the clean repository root inside `localhost/tripod-native:f44`, with the read-only `/elements` installation and isolated writable toolchain/cache mounts described above. Supply a fresh `$KEYED_SCALING_REPORT_DIR`, `$KEYED_NETWORK_ID`, `$KEYED_GENESIS_ID` and `$ELEMENTS_INTENDED_TIP` from the installation provenance. This staging step runs no native test.

```sh
env \
  TRIPOD_LIVE_EXECUTOR="$PWD/scripts/elements-native-executor.sh" \
  TRIPOD_LIVE_REPORT_DIR="$KEYED_SCALING_REPORT_DIR" \
  TRIPOD_LIVE_SUITE_SHORT_SHA="$(git rev-parse --short=12 HEAD)" \
  TRIPOD_LIVE_SUITE_COMMIT="$(git rev-parse HEAD)" \
  TRIPOD_LIVE_SUITE_TREE="$(git rev-parse HEAD^{tree})" \
  TRIPOD_LIVE_NETWORK_ID="$KEYED_NETWORK_ID" \
  TRIPOD_LIVE_GENESIS_ID="$KEYED_GENESIS_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTSD=/elements/bin/elementsd \
  ELEMENTS_NATIVE_EXECUTOR_ELEMENTS_CLI=/elements/bin/elements-cli \
  ELEMENTS_NATIVE_EXECUTOR_FRAMEWORK=/elements/src/test/functional \
  ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP="$ELEMENTS_INTENDED_TIP" \
  ELEMENTS_NATIVE_EXECUTOR_NETWORK_ID="$KEYED_NETWORK_ID" \
  ELEMENTS_NATIVE_EXECUTOR_ENABLE_WALLET=1 \
  ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY=/elements/lib/libsecp256k1.so \
  ELEMENTS_NATIVE_EXECUTOR_FEE_POLICY=node-defaults \
  ELEMENTS_NATIVE_EXECUTOR_MATURITY_OBSERVATION=1 \
  ELEMENTS_NATIVE_EXECUTOR_PYTHON=python3 \
  ELEMENTS_NATIVE_EXECUTOR_LAUNCH_PREFIX= \
  ELEMENTS_NATIVE_EXECUTOR_UPSTREAM_BASE= \
  ELEMENTS_NATIVE_EXECUTOR_LOCAL_TOPICS= \
  cargo test -p tripod-vectors --profile gate --test maturity_native \
    keyed_scaling::keyed_scaling_campaign_runs_against_a_real_target -- --ignored --exact --nocapture
```

The wrapper supplies `PATH`, `HOME`, `TMPDIR`, `CARGO_HOME`, `CARGO_TARGET_DIR` and `RUSTUP_HOME`. The command lists every campaign/executor environment variable; none introduces a policy override. Expected files are `<short>.report.capture`, `<short>.report.timing`, `<short>.report.capture.timing`, `RUN-REPORT`, `MANIFEST.sha256` (covering the other four), plus separate `diagnostics/`. Each complete executor response is retained in the capture's wire JSON. Replay reconstructs requests from the descriptor and schedule without debug text. Eligibility requires the full census, all default-relay controls and a classified overweight neighbour; `admitted` remains false. Report the run identity, all counts, SOURCE preparation and MERGE costs separately, actual signature lengths, exact refusal/route, node weights and mined readbacks, walls, resource mismatches and absent native stack/validation/opcode-timing dimensions. Tracing spans expose construction counts, exchange progress and replay outcome; the library prints nothing.
