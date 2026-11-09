# Research Question: Metadata-Dependent Constructor Continuity · `q:constructor:state`

> **Status:** Accepted prototype — see (`sec:state-constructor:result`)
> **Blocks:** STATE ABI; `announce-maturity`; all later STATE-spending target operations
> **Affected packages:** tapscript, linker, transaction, vectors, release
> **Decisions:** D003, D004, D006
> **Imports:** (`[RZ-rule:translation:bind]`),
> (`[RZ-rule:translation:struct]`),
> (`[RZ-rule:translation:certificate-leaf]`),
> (`[RZ-inv:invariant:succession]`)
> **Expected handoff:** accepted constructor-continuity decision or documented target rejection

## Question · `sec:state-constructor:question`

Can the initial Elements backend authenticate a metadata-dependent predecessor
object, derive its semantic successor, and verify that successor under the same
linked static code relation?

For STATE, the construction must bind:

```text
semantic metadata
+
closed PID root identity
+
linked operation-program set
+
target constructor policy
```

while preventing:

- metadata substitution;
- static code-subtree substitution;
- internal-key substitution;
- key-path bypass;
- spendable metadata path;
- alternate schema or encoding;
- independently chosen predecessor and successor roots;
- stale constructor use across bundles.

The result must be publicly constructible for permissionless STATE operations
and fit the selected target.

## Fixed constraints · `sec:state-constructor:constraints`

### Semantic

- STATE carries public canonical metadata:
  ```text
  Ω, Y_L, Y_T, Q, cycle, maturity
  ```
- every STATE read is a real succession edge;
- predecessor metadata is authenticated;
- successor metadata is operation-derived;
- successor preserves the approved static operation relation;
- closed PID identity and amount one are checked separately;
- root history must detect intermediate constructor substitution.

### Target

The prototype may use only capabilities present in the typed Elements target,
including where selected:

- input/output program introspection;
- SHA-256 or streaming SHA-256;
- byte concatenation;
- target tapleaf/tapbranch/tweak rules;
- x-only and compressed point handling;
- tweak verification;
- target leaf version;
- stack, witness, crypto, transaction, and policy limits.

Node implementation version is test provenance under
(`[ADR011-rule:toolchain:target-compatibility]`), not constructor identity.

### Toolchain

- no manual post-link patch;
- no production private internal key;
- no source or plan parsing;
- exact linked bundle and ABI binding;
- deterministic constructor bytes from explicit inputs;
- prototype artifacts cannot enter a final release silently.

## Terminology · `sec:state-constructor:terms`

Let:

```text
M_K       canonical metadata for object kind K
C_K       linked static operation-code root for K
L_K(M)    dynamic metadata commitment/leaf
P_K       internal key
R_K(M)    target program-tree root
Q_K(M)    target output key/program
```

A constructor instance is:

```text
Constructor_K(M)
```

under one exact target, schema, internal key, and linked static code root.

Constructor continuity means:

```text
input  = Constructor_K(M_before)
output = Constructor_K(M_after)
```

with the same authenticated:

```text
K
C_K
P_K
metadata schema
target construction rule
```

## Candidate matrix · `tab:state-constructor:candidates`

| Mint | Candidate | Strength | Main risk |
|---|---|---|---|
| (`candidate:constructor:dynamic-metadata-leaf`) | Static code root plus dynamic unspendable metadata leaf | Preserves one static code identity while metadata changes | Tweak verification, root authentication, key-path and totality |
| `candidate:constructor:separate-metadata-output` | Fixed program plus separately welded metadata object | Simple fixed program | Adds pairing/object state and may require normative change |
| `candidate:constructor:metadata-in-every-leaf` | Metadata embedded into all operation leaves | Direct metadata visibility | Entire code tree changes; recursive commitment |
| `candidate:constructor:witness-only-metadata` | Fixed program with witness metadata only | Small constructor | Metadata is unauthenticated; rejected absent another commitment |
| `candidate:constructor:alternate-consensus-field` | Metadata committed through another target field | May avoid dynamic tree | Field availability and successor authentication uncertain |

The leading prototype is
(`candidate:constructor:dynamic-metadata-leaf`).

## Leading construction · `candidate:constructor:dynamic-metadata-leaf`

The candidate conceptually uses:

```text
fixed publicly derived internal key P_K
linked static code root C_K
dynamic unspendable metadata leaf L_K(M)

R_K(M) = branch(C_K, L_K(M))
Q_K(M) = target_tweak(P_K, R_K(M))
```

The exact formulas, tagged hashes, child ordering, scalar handling, parity, and
program encoding come from the typed target.

### Predecessor check

The executing program:

1. authenticates the current target program class;
2. receives or derives canonical predecessor metadata;
3. receives one static code-root witness;
4. reconstructs the metadata commitment;
5. combines it with the static root;
6. reconstructs or verifies the predecessor target program;
7. retains the authenticated static root.

### Successor check

The same program:

1. derives successor metadata from the semantic operation;
2. constructs the successor metadata commitment;
3. reuses the authenticated static root;
4. derives the successor target program;
5. verifies the canonical successor output.

Two independently supplied roots are not sufficient.

## Internal-key policy · `rule:state-constructor:internal-key`

The internal key must be a deterministic, publicly auditable point with no
known private scalar.

Production must not use:

- operator key;
- release key;
- generated-then-discarded secret;
- test key;
- mutable deployment key.

Evidence can establish deterministic derivation and absence of a secret
generation step. It cannot prove discrete-log impossibility.

The final bundle binds the exact internal key.

## Constructor totality · `q:constructor:tweak-totality`

The prototype must define behavior when target tweak/scalar rules reject a
hash-derived scalar.

Candidates:

| Policy | Consequence |
|---|---|
| reject the rare constructor | simplest, but semantic construction is not total |
| deterministic metadata representation nonce | practical totality; introduces representation multiplicity |
| deterministic internal-key retry | changes key selection per instance |
| explicit negligible residual | requires named trust/constructibility residual |

No policy is accepted by silence. The deterministic representation nonce was
selected; see (`sec:state-constructor:result`).

The selected policy must define:

- canonical first-party construction;
- target validation;
- ABI fields;
- identity effects;
- release residuals.

## Threat model · `sec:state-constructor:threats`

The attacker may choose:

- predecessor metadata witness;
- successor metadata;
- static-root witness;
- internal key or parity hint;
- operation program;
- control path;
- output position;
- metadata schema/encoding;
- constructor from another bundle;
- alternate target program class.

Required failures:

| Mutation | Required result |
|---|---|
| wrong predecessor metadata | reject |
| wrong successor metadata | reject |
| correct metadata under wrong static root | reject |
| distinct predecessor/successor roots | reject |
| operation leaf removed or escape leaf added | reject |
| wrong internal key or parity | reject |
| metadata leaf selected for spend | reject |
| unknown/noncanonical schema | reject |
| successor at wrong output | reject |
| wrong PID asset or amount | reject |
| old-bundle predecessor with unauthorized new-bundle successor | reject |
| intermediate bad constructor later restoring final cursor | history replay rejects |

## Prototype · `sec:state-constructor:prototype`

### Stage 1 — off-chain reference

Implement independent typed construction for:

- metadata encoding;
- metadata leaf hash;
- static/dynamic tree root;
- target tweak;
- output program.

Cross-check exact bytes with a reviewed target library where practical.

### Stage 2 — synthetic metadata transition

Use one synthetic public counter:

```text
counter_after = counter_before + 1
```

Verify predecessor and successor constructors under one static root.

### Stage 3 — adversarial continuity

Run every wrong-root, wrong-key, wrong-schema, metadata-leaf, and output-slot
mutation.

This stage is mandatory before full STATE integration.

### Stage 4 — full STATE schema

Replace the counter with typed STATE metadata and test every maturity variant
and amount boundary.

### Stage 5 — maturity announcement

Integrate the real `announce-maturity` state relation:

- unannounced predecessor;
- valid lead;
- announced successor;
- unchanged economic fields;
- operator authorization;
- no RESV.

### Stage 6 — linker and ABI

Produce:

- typed static-root symbol;
- constructor recipe;
- target program;
- metadata schema;
- witness roles;
- control-path recipe;
- resource formula;
- prototype bundle and ABI identities.

## Vectors · `sec:state-constructor:vectors`

Required positive cases:

- minimum metadata;
- representative state;
- near-domain amount values;
- each maturity variant;
- valid synthetic transition;
- valid maturity announcement;
- sponsorless and sponsored construction.

Required negative families:

- every metadata field changed independently;
- malformed maturity tag/payload;
- field reorder and trailing bytes;
- wrong object/domain tag;
- wrong static subtree;
- missing/extra/changed operation leaf;
- wrong internal key;
- wrong parity/compressed point;
- metadata leaf spend;
- wrong target version;
- wrong output slot/count;
- wrong PID asset or amount;
- stale constructor/bundle;
- mixed operation programs;
- intermediate history corruption.

## Measurements · `sec:state-constructor:measurements`

Measure:

- predecessor verification bytes;
- successor verification bytes;
- metadata hash cost;
- tweak/crypto operations;
- witness bytes;
- initial and peak stack;
- element size;
- static tree leaf count and depth;
- control-path bytes;
- complete maturity-announcement transaction weight and policy result.

Compare backend/linker/transaction predictions with target-native observations.

## Acceptance · `gate:state-constructor:accept`

Accept a constructor only when:

- predecessor metadata binds to the consumed constructor;
- successor metadata is semantically exact;
- one authenticated static code identity is reused;
- wrong code/key/schema vectors reject;
- metadata and key paths provide no accepted escape;
- internal-key policy has no retained secret;
- tweak totality policy is explicit;
- permissionless construction needs no hidden secret;
- linker resolves the recipe without manual patching;
- transaction ABI is canonical;
- complete target-native transactions pass;
- predicted and observed resources agree;
- output reproduces deterministically.

## Rejection · `gate:state-constructor:reject`

Reject a candidate if it:

- accepts a wrong static subtree;
- trusts unbound metadata;
- permits independent predecessor/successor roots;
- leaves a key or metadata escape;
- is undefined over valid metadata without accepted policy;
- requires a secret on a permissionless path;
- cannot be linked deterministically;
- exceeds hard target limits;
- passes only in a mock interpreter;
- differs materially between prototype and production bytes.

## Result · `sec:state-constructor:result`

Accepted as a prototype. The selected candidate is
(`candidate:constructor:dynamic-metadata-leaf`): a dynamic metadata leaf beside
a static code subtree, under one authenticated static root.

### Decisive design results · `sec:state-constructor:decisive`

**Successor metadata is derived, not witnessed.** The program builds the
successor's bytes on the stack out of the predecessor's already-authenticated
bytes: constant slices either side of the changed field, a checked counter
increment, the witnessed nonce, and a literal zero reserved tail. No
independent successor witness exists, so the mutation and adjacency threats
against it are not defended — they are unrepresentable.

**Canonical branch order is a creator-side property.** The program hashes the
two children in one fixed order and compares the derived tweak against the
introspected input program. It never computes an ordering, carries an order
bit, or hashes in caller-supplied order. Instances whose fixed-order hash is
not the canonical one are excluded at creation by grinding the schema's nonce
field, the policy named `CanonicalNonceRetry`. This matters because the
reviewed target has no byte-lexicographic comparison at all: its ordering
primitives read fixed-width integers, neither of which orders a digest.

**One authenticated static root.** The independently-chosen-roots threat is
answered by there being one root to reuse, rather than by a check that two
supplied roots agree. The static-subtree requirement is satisfied
structurally.

**The reach bound governs every layout.** No reviewed primitive in the emitted
schedule reads below the third stack item, and the schedule uses no `OP_PICK`,
no `OP_ROLL`, and no altstack. The property is machine-checked over the emitted
program rather than asserted in prose.

Pinned domains: the predecessor counter is `0..=2^63-2`, excluding the value
whose increment would set the target's signed flag; the metadata schema is 48
bytes with the nonce at bytes `36..40` and a zero reserved tail at `40..48`;
domain and schema are pinned to the recipe's own constants rather than to a
caller argument.

The metadata leaf is machine-checked unspendable. The internal key is the NUMS
point

```text
50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0
```

derived as SHA-256 of the generator's uncompressed encoding. Determinism and
the absence of a generation step are evidenced; discrete-log hardness is not,
and remains a named residual.

### Resources · `tab:state-constructor:resources`

Measured on the emitted program, not predicted.

| Quantity | Value | Bound |
|---|---:|---|
| script bytes | 699 | — |
| witness bytes | 924 | 400000 weight |
| peak main stack | 9 | 1000 |
| largest element | 103 | 520 |
| hash operations | 16 | — |
| curve checks | 2 | — |
| validation budget | 10.3% | per-check allowance |
| control path | 1 | 128 |

The largest element sits at 19.8% of its limit, which is the binding one.

### Native evidence · `sec:state-constructor:evidence`

| Fact | Value |
|---|---|
| matrix | 36 of 36 rows agreed |
| claims | 9 of 9 covered |
| completeness | `complete_for_constructor_prototype` |
| node | Elements Core daemon v28.99.0-0b3bffd93138 |
| workspace | ADR-018 merged tip `0b3bffd`, upstream base `b7fc5d0`, topics `fix/tapscript-opcodes` and notes |
| determinism | two gated runs byte-identical |

This table is a historical record of a run, not a locator for one. The report
itself is not retained in the repository, and no digest of it is kept: a bare
hash with no retained bytes, no typed reference, and no consumer able to
revalidate it from the repository is the weakest form the adopted identity
discipline warns against (`[ADR021-case:identity:evidence]`),
and it would contradict the stop record in the identity register, §3.5, which
declines to mint a native conformance report identity. Reports are compared by
typed content and exact bytes in process, where they exist.

First contact with the node found two defects, both in fixtures and neither in
the program: a nonce-grinding discard that the fixture failed to apply, and a
sorted-children row that described the same tree twice and so could not be the
refusal it claimed. Both were repaired as fixtures. Zero program defects were
found by native execution.

### Residuals · `sec:state-constructor:residuals`

- no proof that the NUMS point has no scalar, only that no step produced one;
- tweak totality is implemented by nonce retry and was never exercised — zero
  retries across 384 instances — so the policy is carried deliberately unleant
  on, the triggering event being roughly `2^-128`;
- scalar validity for the tweak is decided by the target during its own check,
  not established beforehand by the program.

## Handoff · `sec:state-constructor:handoff`

A successful result creates an accepted constructor decision and updates:

- target capability status;
- tapscript constructor pattern;
- linker symbols and reference strategies;
- transaction metadata/witness ABI;
- permanent vectors;
- release evidence requirements;
- [Phase 6](../phases/06-state-and-maturity.md).

A failed result records target infeasibility. It does not silently weaken STATE
succession.

### What this acceptance is · `sec:state-constructor:scope`

The accepted object is a prototype and nothing else. The work emitted no
attestation-contract operation, froze no ABI, produced no bundle, minted no
identity of any kind, and calibrated no bound. The 48-byte schema is the
prototype's, not STATE's: it carries a synthetic counter where STATE carries
its metadata, which is what (`sec:state-constructor:prototype`) Stage 2 asks
for and is not Stage 4.

Promoting this construction to a production backend pattern is a separate
reviewed act, with its own decision, and cannot happen by a prototype being
reused. What the acceptance licenses now is the STATE-side design work that
was blocked on knowing whether the target could do this at all.

Evidence lives in `target-elements-conformance` and in the Guide-10 gate record
in [the backlog](../backlog.md) (§2.14). Guide 11 public declassification
remains the open third foundational prototype.

## Keyed-covenant evidence · `sec:state-constructor:keyed-evidence`

This is the distinct Wave-1 keyed-covenant evidence set required by (`obl:guide15-exec:measurement-home`). It does not inherit the STATE prototype's acceptance or discharge later keyed measurements.

### Measurement (a): born-first · `rec:state-constructor:keyed-birth`

The archive is `packages/vectors/fixtures/keyed-birth-run-of-record/`: the capture, outer timing, ceremony timing, `RUN-REPORT` and original `MANIFEST.sha256` were copied from run `w20c1-birth-20261108T151743Z`, with all five node-side SHA-256 digests verified before and after transfer into the fixture directory. The manifest covers the capture, outer timing and report; the reader independently pins all five files, including the ceremony timing omitted by that manifest. `packages/vectors/src/keyed_birth_corpus.rs` decodes the capture and the complete fourteen-response Debug carrier retained in `RUN-REPORT`, checks their links and projections, replays `replay_birth_capture`, and rebuilds `Wave1TargetPacket` and `BirthCampaignManifest`. The immutable capture still says `admitted false`: admission is the reader's result, not an edit to the observation.

This capture omitted complete executor responses from its exchange records, so admission uses an archive-specific decoder for the run report's Debug transcript. Later keyed captures retain each complete response in the capture itself in its wire JSON form, so their admission readers read no debug text.

Bindings: reviewed `elements-tapscript-v2`, development `elementsregtest`, leaf version 196, protocol 8, adapter `elements-native-executor` 2.1.0, node Elements Core daemon v28.99.0. The exact node revision, functional-framework revision, intended source tip, executable digest and retained `PROVENANCE.txt` content are bound in the capture's handshake and `executable_provenance`; these are source provenance, not protocol identities. The capture binds the observed network and genesis to the development declaration and retains the active domain/leaf and capability census. Execution used `localhost/tripod-native:f44` and `/elements/bin/elementsd`. Policy is separate: explicit policy asset, `node-defaults` are declared selections; exact startup arguments are retained observations, and relay and mempool floors are observed at 100 sat/kvB each; block floor 100 sat/kvB is declared by pinned-source `DEFAULT_BLOCK_MIN_TX_FEE`, with no override, not a node observation. The reader checks that every response reports the same policy and startup arguments.

| Subject | Spent output index | Current input index | Admission result |
| --- | --- | --- | --- |
| A0 | 0 | 0 | Mined control, height 4 |
| A1 | 1 | 0 | Born-first `EqualVerify` refusal, paired with A0 |
| A2 | 0 | 1 | Mined control, height 7 |
| A3 | 1 | 1 | Born-first `EqualVerify` refusal, paired with A2 |

Both refusals retain exactly `mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)` as their relay reason and a script-path rejection class. Attribution requires the exact birth leaf's sole equality at byte offset 8, the accepted pair, and exclusion of the sponsor's own equality/signature sites. A different or ambiguous site remains open. The archive retains the consensus route and evidence, txid/wtxid, block hashes, heights, and complete raw mined bytes; controls use the `relay-and-template` route. Four mined readbacks occur at heights 3 and 4 for the first parent/control and 6 and 7 for the second. There are fourteen exchanges: two fundings, six signings, two parents, two controls accepted (twelve acceptances), and the two refusals. Negatives were constructed and signed before their controls. Each pair fixes its parent, programs, assets, values, successors and funding outpoint; the reserved funding output is index 1 in both placements, with fresh funding for the second pair. Only the spent output index changes per negative; transaction identities, sponsor authorization and observation results are declared collateral. Sponsor outputs are 100,000 sat each, equal parent covenant outputs 10,000 sat each, and explicit parent/child fees 1,000 sat: fixture choices, not funding floors.

Retained walls are 0.984 s in `RUN-REPORT` and 0.985 s in the outer timing; the ceremony sidecar records passed without a numeric wall. The orchestrator's capture report gives 0.99 s for the ignored test and 0.2 s node startup, with one transient RPC connection retry before startup. Those infrastructure details are not subject verdicts or additional measured covenant dimensions.

**Pinned-source reading (not a native observation):** paths below are relative to the Elements source root identified by the retained executable provenance, read on the node at `/var/home/tripod/loops/toolchains/elements/b7fc5d08/src`. `src/script/interpreter.cpp:1836` pushes the selected input's txid, calls `push4_le(stack, inp.prevout.n)` at line 1840, then pushes a separate one-byte flag item at line 1843. `src/script/interpreter.cpp:86` defines `push4_le` using little-endian conversion and the four-byte `uint32_t` width, not script-number encoding. `src/script/interpreter.cpp:137` derives the flag item separately from asset issuance and peg-in state. `src/primitives/transaction.h:41` and `:45` declare the two high flag bits; `src/primitives/transaction.h:50` gives the index mask `0x3fffffff`. For non-coinbase Elements inputs, deserialization extracts those flags at lines 217 and 220, and stores the masked index in `prevout.n` at line 226. Thus inspection reports the index without issuance or peg-in flag bits, followed by the separate flags item. The coinbase exception at lines 209–213 is outside these subjects.

Absent dimensions: no issuance subjects, no peg-in subjects, and no native stack-depth/altstack/element/budget measurements. Static fragment stack checks are separate evidence. No funding floor, authenticated current-program equality, sole-leaf census, whole keyed realization, or measurements (b) onward are admitted here. Current-input outpoint inspection is accepted as the born-first realization mechanism because the four admitted subjects and the pinned index/flag reading meet the acceptance condition named by ruling 160 (`rec:phase7:wave1-birth-admission`). The bite-1.0 gate is not closed.

### Measurement (b), key-free continuation · `rec:state-constructor:keyfree-continuation`

The second entry of the distinct keyed-covenant evidence set is the key-free part of measurement (b), retained at `packages/vectors/fixtures/keyed-continuation-run-of-record/` from run `w20c1-cont-20261108T175223Z` on `0.7.80-dev`. All five node-side SHA-256 digests were verified before and after moving the capture, outer timing, ceremony timing, `RUN-REPORT` and `MANIFEST.sha256` into the archive. The manifest covers all four other files; the reader independently pins all five. Original framing bytes and the capture's `admitted false` standing are unchanged. Admission is the result of `packages/vectors/src/keyed_continuation_corpus.rs`: it shares the birth reader's manifest and capture-record parsing, decodes complete responses from the capture's wire JSON journal, checks their links and outer projections, and replays `replay_continuation_capture`. No debug transcript supplies any response.

Bindings: reviewed `elements-tapscript-v2`, development `elementsregtest`, leaf version 196, protocol 8, adapter `elements-native-executor` 2.1.0, and Elements Core daemon v28.99.0. The exact node/framework revisions, intended source tip, executable digest, `PROVENANCE.txt` content, network/genesis identifiers and active domain/leaf/capability census are bound by the retained handshake, environment, executable provenance and outer header. The reader rebuilds `Wave1TargetPacket` and `ContinuationCampaignManifest` from these facts. Execution used `localhost/tripod-native:f44` and `/elements/bin/elementsd`. Policy remains separate: explicit policy asset and `node-defaults` are declared selections; startup arguments are observed. Relay and mempool floors are observed at 100 sat/kvB each; the 100 sat/kvB block floor is declared by pinned-source `DEFAULT_BLOCK_MIN_TX_FEE`, without override. All eleven responses must agree on policy and startup arguments, and all six policy facts retain their origins.

| Subject | Changed coordinate beside control | Admission result |
| --- | --- | --- |
| Control | None | Mined control, height 4 |
| Wrong program | Output 0 program payload | Program equality refusal |
| Wrong version | Output 0 witness version, payload unchanged | Version equality refusal |
| Displaced successor | Permutation of the two complete spendable outputs | Program equality refusal |

There are eleven exchanges, terminal `complete`: sponsor funding, parent signing and submission, then signing/submission of wrong program, wrong version, displaced successor and finally the same-coin control. Eight exchanges are accepted; all three negatives are script-path rejections with route `refused` and exact relay reason `mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)`. The version-zero output therefore reached script, not an earlier policy refusal. The parent mined at height 3 and the control at height 4, both by `relay-and-template`, with exact raw readback, txid/wtxid and block identity retained. Refused relay is not a direct-block rejection observation.

Every child spends parent output 0 with the same reserve at sponsor output 1; birth remains valid. The primary leaf is born first, continue first and final true under the reviewed leaf version and fixed unspendable internal key. The alternative leaf is reconstructed by inserting empty push/drop before final true, never supplied as a program. The manifest fixes the parent, predecessor, funding, assets, amounts, fee and leaf recipes within each pair; transaction identities, sponsor authorization and observations are declared collateral, and the displaced row exchanges complete outputs. Sponsor outputs are 100,000 sat; parent programs receive 10,000 sat each with 79,000 sat change and 1,000 sat fee; children have a 10,000 sat successor, 99,000 sat change and 1,000 sat fee. These are fixture choices, not funding floors.

The shared-site limit applies to every row: the exact error does not identify one of equality offsets 8, 15 and 16. Reconstructed recipes, valid birth, checked sponsor hash equality and the mined one-coordinate control independently isolate version versus program comparison. This is neither a native opcode trace nor independent ECDSA verification. Ambiguous, earlier-layer or unmatched outcomes remain open; this archive has none.

**Pinned-source reading (not an observation):** the Elements source root is `/var/home/tripod/loops/toolchains/elements/b7fc5d08/src`, at the exact intended tip bound by the retained provenance. `src/script/interpreter.cpp:1856`–`:1859` calls `pushspk` on the spent output's script, and `src/script/interpreter.cpp:1948`–`:1951` calls it on the selected transaction output's script. The shared helper at `src/script/interpreter.cpp:124`–`:135` tests `IsWitnessProgram`; lines 129–130 push the program bytes first, then the witness version as a script number in a separate top item. For a non-witness script, lines 132–133 instead push the cached script hash and script-number `-1`. Thus the wrong-version subject changes the version item while preserving the payload; wrong-program and displacement change the payload item while preserving version one. This reading supports the coordinate isolation, not a claim that the error text reveals the executing equality site.

The report and outer timing both retain 0.873 s; the ignored test was reported as 0.87 s, node startup as 0.2 s, with one transient RPC connect retry before readiness. The ceremony sidecar records passed without a numeric wall. Startup and retry are infrastructure facts, not subject verdicts.

Absent dimensions: wrong-key, mixed-key and coexistence rows, measurement (c), the full leaf/key-path census and its constructor premises, and native stack metrics. No issuance, peg-in or funding-floor coverage is claimed. Ruling 161's acceptance condition is not yet met: it requires measurement (b)'s key rows and measurement (c), which wait for the constructor (`rec:phase7:wave1-continuation-keyfree-admission`). The bite-1.0 gate remains open.

### Measurement (c): keyed constructor and path census · `rec:state-constructor:keyed-constructor`

This third entry of the keyed-covenant evidence set admits measurement (c)'s native subjects from `packages/vectors/fixtures/keyed-run-of-record/`, run `w20c1-keyed-20261109T101211Z` on `0.7.84-dev`. Measurement (b)'s keyed rows from the same archive are a separate entry below because continuation equality and path closure answer different obligations. The five files are the capture, outer timing, ceremony timing, `RUN-REPORT` and `MANIFEST.sha256`; all five node-side SHA-256 digests were verified before and after their move into the fixture directory. The manifest covers the other four files, and `packages/vectors/src/keyed_corpus.rs` independently pins all five. Its admission shares the birth and continuation readers' manifest, framing and packet binding, reconstructs both public descriptors, decodes complete responses from wire JSON, checks outer projections and replays `replay_keyed_capture`. No debug transcript supplies a response. Original framing bytes and `admitted false` remain unchanged; admission is the reader's result.

Bindings: reviewed `elements-tapscript-v2`, development `elementsregtest`, leaf version 196, protocol 8, adapter `elements-native-executor` 2.1.0, and Elements Core daemon v28.99.0. The retained header, handshake, environment and executable provenance bind the exact node/framework revisions, intended source tip, binary SHA-256, `PROVENANCE.txt`, network/genesis identifiers and activation/capability census; both reconstructed descriptors are bound to the capture. Execution used `localhost/tripod-native:f44` and `/elements/bin/elementsd`. `Wave1TargetPacket` is rebuilt from these facts. Policy is separately attributed: explicit policy asset and `node-defaults` are declared choices; startup arguments are observed; relay and mempool floors are observed at 100 sat/kvB; the 100 sat/kvB block floor is declared by pinned-source `DEFAULT_BLOCK_MIN_TX_FEE` without override. All 25 responses agree on policy and startup arguments, with all six policy facts retaining their origins.

| Constructor subject | Changed coordinate beside canonical k control | Admission result |
| --- | --- | --- |
| Canonical k | None | Mined control, height 7 |
| Wrong control parity | Only the original control's parity bit | Commitment refusal |
| Uncommitted leaf | Only one offered key-literal byte; original control retained | Commitment refusal |
| Key-path attempt | Only keyed witness path selection, one 64-byte zero signature | Key-path signature refusal |

The two commitment refusals report exactly `mandatory-script-verify-flag-failed (Witness program hash mismatch)`; the key-path attempt reports exactly `mandatory-script-verify-flag-failed (Invalid Schnorr signature)`. All three precede their same-coin control and retain identical non-witness transaction bytes. Witness bytes, wtxid, weight and observation are declared collateral. The raw adapter layer for all three is `script_path_rejection`, route `refused`: its key-path detector requires every input to take the key path, so the P2WPKH sponsor prevents that aggregate label. The keyed row's separate `KeyPathSignatureRefusal` classification uses its reconstructed one-item witness, exact Schnorr error and mined control; the sponsor uses ECDSA and the keyed leaf has no signature opcode. No covenant clause is claimed executed by the key-path attempt, and a relay refusal is not a direct-block rejection observation.

The complete public descriptor and reconstruction account for exactly one script leaf and the single conditional taproot key path under each key. The leaf is the canonical 32-byte public semantic-key push and drop, then born first, continue first and final true, under the reviewed version and fixed NUMS policy derived from SHA-256 of the uncompressed secp256k1 generator. No metadata or alternate leaf exists. Every control is authenticated against its actual mined predecessor program. The offline evidence at `packages/vectors/tests/keyed_constructor/mod.rs` retains repeated exact-byte construction, descriptor-only reconstruction in a fresh process for both keys, distinct output programs when only the semantic key changes, typed foreign-internal-key refusal and typed extra-leaf census refusal; the native archive adds the commitment and signature boundaries above. Fixed test handles remain test deployment data, not plan constants or additional spending predicates.

**Pinned-source reading (not an observation):** the source root is `/var/home/tripod/loops/toolchains/elements/b7fc5d08/src`, at the exact intended tip bound by the retained provenance and checked by `git rev-parse HEAD`. `src/script/interpreter.cpp:3186`–`:3188` hashes the leaf version, compact script length and script with the Elements tapleaf tag. `src/script/interpreter.cpp:3217`–`:3228` extracts the internal key from the control, takes the output key from the witness program, reconstructs the Merkle root and calls `CheckTapTweak` with `control[0] & 1`; `src/pubkey.cpp:259`–`:264` passes the output key, parity, internal key and tweak to `secp256k1_xonly_pubkey_tweak_add_check`. `src/script/interpreter.cpp:3287`–`:3289` computes the offered leaf hash and returns `SCRIPT_ERR_WITNESS_PROGRAM_MISMATCH` if that commitment fails, before `ExecuteWitnessScript` at line 3297. `src/script/script_error.cpp:94` renders that error as `Witness program hash mismatch`. Thus parity and uncommitted-leaf coordinates reach a commitment boundary, not either covenant equality.

**Pinned-source reading (not an observation), key path:** `src/script/interpreter.cpp:3274`–`:3279` sends a one-item witness after annex handling to `CheckSchnorrSignature` against the output program with `SigVersion::TAPROOT`. The checker at `src/script/interpreter.cpp:2949`–`:2973` checks signature size/hash type, computes the taproot signature hash and, at line 2972, returns `SCRIPT_ERR_SCHNORR_SIG` when signature verification fails; `src/script/script_error.cpp:108` renders `Invalid Schnorr signature`. The frozen witness coordinate and exact error distinguish this site from the sponsor's ECDSA path. This source reading supports attribution; it is neither a native instruction trace nor independent sponsor-signature verification.

Conditional premises remain exactly those of (`cav:w1route:constructor-assumptions`): the NUMS argument retains discrete-log hardness and SHA-256 preimage resistance under the guide's conditional standing; finite failed key-path attempts do not prove impossibility (`judg:guide15-exec:nums-standing`). Key separation additionally assumes second-preimage resistance of the target's tagged SHA-256 chain for separation from a generated key, and collision resistance where both keys are adversarial; these are proposed premises for acceptance alongside, not consequences of, the guide's preimage premise (`assum:guide15-exec:sha256-preimage`). Reader membership must authenticate the registered public recipe, not infer it from a key-shaped literal in an arbitrary script. If a key change leaves the output program equal, or a permitted path bypasses either clause, the proposal fails rather than acquiring an extra authorization condition (`rule:guide15-exec:wave1-failure`).

The complete campaign has 25 exchanges, terminal `complete`: one seven-output funding, twelve signings and twelve submissions, with twenty accepted exchanges and five refusals. Four parents mined at heights 3–6; controls mined at 7–9, all by `relay-and-template`, with exact raw bytes, txid/wtxid and block identities retained. Report and outer timing retain 2.436 s; the ignored test was reported as 2.44 s. The ceremony sidecar records passed without a numeric wall. Diagnostics retain two transient RPC connect retries before readiness; these are infrastructure detail, not verdicts. Authenticated current-program equality with a sole key-committing public leaf is accepted as the continue-first realization mechanism because the admitted measurements (b) and (c), including the full leaf/key-path census and the stated conditional premises, meet ruling 161's acceptance condition; those premises retain their conditional standing. No native stack metrics, issuance or peg-in coverage, funding floor, cost, scaling or later measurement, or unconditional key-path impossibility is admitted, and the bite-1.0 gate is not closed. See (`rec:phase7:wave1-keyed-admission`).

### Measurement (b), keyed continuation · `rec:state-constructor:keyed-continuation`

This fourth entry admits the keyed continuation rows from the same five-file archive, packet, descriptors, policy origins and 2.436 s run described at (`rec:state-constructor:keyed-constructor`). Together with (`rec:state-constructor:keyfree-continuation`), it supplies measurement (b)'s keyed and key-free subject evidence; it makes no acceptance disposition for ruling 161.

| Continuation subject | Coordinate and control | Admission result |
| --- | --- | --- |
| Canonical k | One birth-zero k handle, output zero under k | Mined control, height 7 |
| Wrong key | Only output-zero program changes to the same reconstructed recipe under l | Program equality refusal |
| Mixed keys | k+l to output zero under k, beside the k+k control | Mixed-key equality refusal |
| Same-key control | Two distinct birth-zero k handles, output zero under k | Mined control, height 8 |
| Independent coexistence | Separate k and l transactions on one history | Both mined, heights 7 and 9 |

Wrong key and mixed keys each report exactly `mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)`, raw layer `script_path_rejection`, route `refused`, before their consuming controls. The mixed pair fixes the first k coin, sponsor reserve, values/assets, fee and output programs. Changing the second funded handle's semantic key necessarily changes its outpoint, authenticated leaf/control and predecessor program; those, transaction identities and sponsor signature are declared collateral. Both candidate second predecessors are mined and birth-position-zero; this is a same-first-coin pair, not a claim that one outpoint changes keys. Independent coexistence consumes separate k and l handles and makes no mixed-key acceptance claim.

Submission order is wrong parity, uncommitted leaf, key-path attempt, wrong key, canonical k, mixed keys, same-key control, independent l. Every negative is constructed, signed and submitted before its control. Sponsor outputs 0–3 create the four parents; reserves 4, 5 and 6 fund the single-k group, mixed pair and independent l. Each reserve is 100,000 sat; each parent has two equal 10,000 sat keyed outputs, 79,000 sat sponsor change and 1,000 sat fee. Children pay 1,000 sat fee and return 99,000 sat sponsor change, with 10,000 sat successor for a single handle or 20,000 sat for two. These are fixture choices, not floors.

The shared-site limit remains: equality offsets 42, 49 and 50 are birth, version and payload, and the error text alone selects none. Reconstructed descriptors, authenticated mined predecessors, valid birth in both input positions, fixed witness version, checked sponsor hash equality and the mined one-coordinate controls independently isolate program equality; the mixed row isolates the l handle beside its same-key control. The separate-item source reading at (`rec:state-constructor:keyfree-continuation`) still applies. This is not a native opcode trace or independent ECDSA verification. All eight campaign subjects have their expected classes with nothing open; conditional constructor premises remain as stated above. Authenticated current-program equality with a sole key-committing public leaf is accepted as the continue-first realization mechanism because the admitted measurements (b) and (c), including the full leaf/key-path census and the stated conditional premises, meet ruling 161's acceptance condition; those premises retain their conditional standing. No native stack metrics, issuance or peg-in coverage, funding floor, cost, scaling or later measurement, or unconditional key-path impossibility is admitted, and the bite-1.0 gate is not closed.
