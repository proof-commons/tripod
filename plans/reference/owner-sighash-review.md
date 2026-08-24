# Owner Sighash Source Review · `ref:sighash-review:owner-message`

> **Status:** Human review reference
> **Upstream:** <https://github.com/ElementsProject/elements>
> **Reviewed tip:** `b7fc5d080a`
> **Target owner:** [`target-elements`](../packages/target-elements.md)
> **Policy:** (`[ADR011-rule:toolchain:target-compatibility]`)
> **Machine-consumed:** no

## Scope · `sec:sighash-review:scope`

This is the dimension-by-dimension reading of the target's taproot message that the owner-sighash concept's Wave 1 owes, at the tip that concept pinned. It establishes what the source says and it establishes nothing else.

It answers four questions the concept left open: which message term carries each required dimension of the selected owner profile; whether the spent-scripts term commits the taproot internal key or the profile's requirement is mis-stated; whether the two refused dimensions are refusals with content when read against the branches that would have carried them; and whether the issuance-rangeproof term is the recorded output-witness hazard's twin.

It does not populate the reviewed sighash capability, does not move any dimension to established, does not compute a digest, and does not claim that any digest is authoritative. A source review alone populates nothing, and the section on the capability's state says so in the target's own words.

It does not review confidential funding, the transaction-wide materializer, or blinding. That boundary runs in both directions and is recorded rather than negotiated.

## Provenance · `rule:sighash-review:provenance`

Every source claim below was read from the Elements working tree at `b7fc5d080a`, clean, with nothing built and nothing executed from that tree by this review. Paths and line numbers are at that tip and are worthless at any other.

Everything executed for this review ran on the shared instance under the standard lane wrapper, never on the orchestrator host. The two runnable diagnoses are the evidence: the recorded output-witness diagnosis at `scripts/diagnose-taproot-output-witness-digest.py` and the twin this review adds at `scripts/diagnose-taproot-issuance-rangeproof-digest.py`. Both booted a disposable regtest chain, and every value either printed is test material from a chain the process created and destroyed.

The reviewed tip's behaviour is the behaviour of that tip. It is not the behaviour of another tip, another target, or any deployment this review did not run against.

## The message, term by term · `tab:sighash-review:terms`

`SignatureHashSchnorr` at `src/script/interpreter.cpp:2688-2802` is the whole construction, and the result is one SHA256 over the stream at `:2801`. The hasher is not empty when the first term arrives: `PrecomputedTransactionData` seeds it at `:2671-2673` with the tag from `:552` and then the deployment's genesis block hash twice. BIP-341's epoch byte is deliberately absent and the source keeps the comment saying so at `:2714-2716`.

The terms below are in the order the source writes them, for a script-path spend under the selected profile's default type byte, whose output type resolves to all-outputs and whose input type is not the permitted-extension one.

| # | Term | Width | Written at | Computed at | Commits to |
|---|---|---|---|---|---|
| 0 | tagged-hash seed | — | `:2671-2673` | `:552` | the tag, then the genesis block hash twice |
| 1 | hash type | 1 | `:2722` | — | the exact type byte, `0x00` under this profile |
| 2 | version | 4 | `:2725` | — | the transaction version field |
| 3 | locktime | 4 | `:2726` | — | the transaction locktime field |
| 4 | outpoint flags hash | 32 | `:2728` | `:2368-2375` | the issuance and pegin flags of every input |
| 5 | prevouts hash | 32 | `:2729` | `:2379-2386` | every input's outpoint |
| 6 | spent asset-and-amount hash | 32 | `:2730` | `:2454-2462` | every spent output's asset and value fields |
| 7 | spent scripts hash | 32 | `:2736` | `:2465-2472` | every spent output's script |
| 8 | sequences hash | 32 | `:2737` | `:2390-2397` | every input's sequence |
| 9 | issuances hash | 32 | `:2738` | `:2402-2412` | every input's issuance, or one zero byte where null |
| 10 | issuance rangeproofs hash | 32 | `:2739` | `:2431-2440` | the two issuance rangeproofs of every entry the input-witness vector happens to hold |
| 11 | outputs hash | 32 | `:2742` | `:2443-2450` | every output, serialized |
| 12 | output witnesses hash | 32 | `:2743` | `:2418-2425` | every entry the output-witness vector happens to hold |
| 13 | spend type | 1 | `:2749` | `:2748` | the script-path flag and whether an annex is present |
| 14 | input index | 4 | `:2768` | — | which input is being authorized |
| 15 | annex hash | 32 | `:2770-2772` | `:3268` | present only when an annex is |
| 16 | tapleaf hash | 32 | `:2795` | `:3287` | the executing leaf, script path only |
| 17 | key version | 1 | `:2796` | `:2702` | the fixed key version, script path only |
| 18 | codeseparator position | 4 | `:2798` | `:581`, `:1472` | where in the leaf's execution the check occurred, script path only |

Terms 10 and 12 are stated deliberately as "every entry the vector happens to hold" rather than as one entry per input or per output. Neither helper takes an index and neither consults the transaction's own cardinality; both iterate whatever the vector contains. That is the whole subject of the length-dependence section below.

Term 15 is absent under the selected profile, which refuses the annex, so the spend-type byte at term 13 is the constant `0x02` for every candidate this arc produces.

## The required dimensions, one by one · `tab:sighash-review:dimensions`

The dimension vocabulary is `SighashDimension` at `packages/target-elements/src/authorization.rs:20-41`, and the selected profile at `packages/tapscript/src/authorization.rs:294-370` requires eight of its ten members and refuses two. Each required dimension is read below against the term that carries it.

| Dimension | Carried by | Source | Established from source |
|---|---|---|---|
| `AllOutputs` | terms 11 and 12 | `:2741-2743` | Yes. Both are written only when the output type is all-outputs, and the pair covers the output list and the output witnesses together. |
| `AllInputs` | terms 4, 5 and 8 | `:2727-2729`, `:2737` | Yes. Three whole-transaction hashes over the input list, all three gated on the input type not being the permitted-extension one. |
| `Issuance` | terms 9 and 10 | `:2738-2739` | Yes, with the caveat that term 10's value depends on a vector length rather than on the input count. |
| `Version` | term 2 | `:2725` | Yes, written directly and unconditionally. |
| `LockTime` | term 3 | `:2726` | Yes, written directly and unconditionally. |
| `TapleafHash` | term 16 | `:2793-2795` | Yes for a script-path spend, which is the only spend path this profile admits. The leaf hash written is the one the control-block check already computed at `:3287`. |
| `SpentOutputs` | terms 6 and 7 | `:2730`, `:2736` | Yes. The two hashes are taken over the precomputed spent-output set rather than over the transaction, which is why a component holding only the transaction cannot form the message. |
| `InternalKey` | no term | — | **No.** The message construction contains no internal-key term at all. The next section settles what that means. |

Seven of the eight are established from source in the sense the review can establish anything: the term exists, its position in the stream is fixed, and its content is the dimension's own subject. The eighth is not, and it is not a matter of the review having failed to look.

## The internal-key question, settled · `rule:sighash-review:internal-key`

The concept asked whether the spent-scripts term commits the taproot internal key or the profile's requirement is mis-stated. The answer is that the spent-scripts term does **not** commit the internal key, and the requirement as written is not establishable from the message.

What the spent-scripts term commits is each spent output's `scriptPubKey`, hashed at `src/script/interpreter.cpp:2465-2472` and written at `:2736`. For a taproot output that script is the witness program, and the 32 bytes inside it are the **tweaked output key**, not the internal key. The two are different values related by a tweak the message never mentions.

The internal key is bound to that output key, but by a check outside the message. `VerifyTaprootCommitment` at `:3217-3229` reads the internal key from the control block at `:3222`, reads the output key from the witness program at `:3224`, computes the merkle root from the executing leaf and the supplied path at `:3226`, and requires that the output key be the internal key tweaked by that root at `:3228`. The script-path branch runs that check at `:3288-3290` and refuses with a witness-program mismatch before the leaf executes at all.

So the relation between the message and the internal key is a composition rather than a commitment: the message fixes the output key, and consensus separately requires any witness to present an internal key and a path that reproduce it. That composition is strong enough for the property an owner cares about, and it is not the same claim as "the message commits the internal key", for two reasons that a review must not blur.

First, the binding lives in the witness check, and the witness is not covered by the message. Nothing in the stream at `:2712-2801` reads the control block; the only witness data the message ever touches is the annex hash and the two length-dependent vector terms.

Second, the tweak is not injective in the message's view. The message is identical for any two internal-key-and-path pairs that tweak to the same output key with the same executing leaf, because the message carries the output key and the tapleaf hash and nothing about the path. That no such pair is easy to find is a hardness argument, not a commitment.

The consequence is a first-party one and it is stated here rather than repaired here. Under the accepted evidence ruling a dimension is established or it is not, and a dimension the review reached but could not exercise stays unreviewed with a recorded reason. `InternalKey` is exactly that dimension: no message term carries it, so no reading of the message construction and no recomputation of the message can ever move it. Left as it stands, the profile's disposition can never become established, because `assess` at `packages/tapscript/src/authorization.rs:262-274` recomputes the disposition from the required set and one permanently unreviewable member keeps it at review-incomplete forever.

Two repairs are admissible and the choice belongs to the owner rather than to this review. The dimension can be re-typed so that the internal key's protection is recorded as carried by `SpentOutputs`, which is where the output key actually is, leaving the required set at seven message-carried dimensions. Or it can stay required and be established by the composition above, in which case what exercises it is a control-block check and not a message term, and the record must say so in those words.

One reading supports the first repair and is worth stating because it is checkable rather than argued. The coverage map at `packages/tapscript/src/authorization.rs:322-368` assigns every protected datum to a dimension, and it assigns none to `InternalKey`. The requirement therefore protects nothing the coverage argument itself names, which is the signature of a requirement that was stated by analogy to BIP-341 rather than derived from this target's message.

## The two refused dimensions, checked · `tab:sighash-review:refusals`

A refusal is only a claim if the branch it refuses does what the refusal says. Both do.

| Refused dimension | Recorded ground | The branch | What the branch actually removes |
|---|---|---|---|
| `SingleOutput` | leaves other outputs free | `:2741`, `:2774-2790` | Terms 11 and 12 are not written at all, because they are gated on the output type being all-outputs at `:2741`. In their place the branch writes the hash of one output at `:2781` and the hash of one output witness at `:2789`, both at the signing input's own position. Every other output and every other output witness is uncommitted, which is the recorded ground exactly. |
| `InputExtensionPermitted` | leaves the input set open | `:2727`, `:2750-2766` | Seven terms — 4, 5, 6, 7, 8, 9 and 10 — are not written at all, because all seven are gated at `:2727` on the input type not being the permitted-extension one. In their place the branch writes the signing input's own outpoint flag, outpoint, spent asset, spent value, spent script, sequence and issuance at `:2751-2765`. The recorded ground understates it: what the branch opens is not only the input set but the whole spent-output census for every input but one. |

The second row is worth carrying forward. Refusing the permitted-extension dimension is what makes `SpentOutputs` a whole-transaction dimension at all; under the other input type, terms 6 and 7 cover one spent output rather than the set, and a profile that admitted it would be claiming a census-wide commitment the message does not make.

## The single-output branch reads a vector it did not measure · `rem:sighash-review:single-output-index`

The refused single-output branch contains a read this review is obliged to record, because a refusal with a second ground is a stronger refusal than one with a first.

At `src/script/interpreter.cpp:2775` the branch checks the signing position against the **output** list, and refuses when the position is not inside it. At `:2786` it then indexes the **output-witness** vector at that same position. The two containers are not the same object and the bounds check covers one of them.

On the consensus path the two lengths agree, because a transaction that reaches script verification with a taproot witness was deserialized from a wire form whose witness flag was set, and `UnserializeTransaction` at `src/primitives/transaction.h:394-398` resizes both vectors together whenever that flag is present. On the signing path they do not agree: the wallet resizes the input-witness vector at `src/wallet/wallet.cpp:2315` and nothing resizes the output-witness vector until serialization, so a transaction being signed carries an empty output-witness vector and a non-empty output list.

That path is reachable through an ordinary wallet interface, and it was reached. The twin diagnosis's last row asks the node to sign the explicit row's own bytes with the single-output hash type; on the shared instance at this tip the node's process ended on signal 11 while serving the request, the client reported the connection closed mid-call, and the node wrote no shutdown record because it did not shut down. The run is `scripts/diagnose-taproot-issuance-rangeproof-digest.py`, the probe row is `single_output_probe`, and the recorded process status is negative eleven.

This arc never reaches the branch, because the selected profile refuses the dimension and the accepted rulings fix the type byte at the default. The observation is recorded because a reader who narrowed the profile later would be reaching a branch that ends the process, and because the behaviour is the target's rather than this repository's — it carries a friction label (`obs:upstream:taproot-single-output-witness-index`).

Two boundaries keep the observation honest. It is not a consensus defect: no wire form reaches that read with mismatched lengths. And it is not this review's to repair; what this review owes is the record and the label.

## The two length-dependent terms · `sec:sighash-review:length-dependence`

Terms 10 and 12 are the two places the message's value depends on how many entries a witness vector holds rather than on the transaction's own cardinality. The source symmetry is exact, and so the concept's hypothesis — that the recorded output-witness hazard has an input-side twin — is well posed.

Both helpers have the same shape. `GetOutputWitnessesSHA256` at `:2418-2425` iterates the output-witness vector and hashes each entry; `GetIssuanceRangeproofsSHA256` at `:2431-2440` iterates the input-witness vector and hashes each entry's two issuance rangeproofs. Neither takes a position and neither reads `vout` or `vin`.

Both vectors are handled together everywhere the serialization touches them. `UnserializeTransaction` clears the witness at `src/primitives/transaction.h:385` and resizes the input-witness vector at `:397` and the output-witness vector at `:398`, both only when the witness flag is set at `:394`. `SerializeTransaction` sets that flag when the transaction has any witness at `:460` and resizes both vectors at `:473-474` before writing them. A default entry on either side is a run of zero-length prefixes: two for an output witness, whose serialization at `src/primitives/txwitness.h:51` is the surjection proof then the range proof, and two for an input witness's issuance proofs, whose serialization at `:20` starts with the two rangeproof fields.

So on the source alone the two terms are the same hazard. What separates them is who grows which vector before the message is formed, and that is a property of the signer rather than of the message.

### The reproduction, and what it settled · `tab:sighash-review:twin-verdict`

The twin was run by the recorded diagnosis's own method, extended so the two vectors vary independently. For each produced signature the run computes four candidate messages over the same transaction — both vectors as the wire form carries them, the output side emptied, the input side emptied, and both emptied — and reports which one the signature actually verifies against. Four candidates rather than two is the whole point: two would have confounded the terms.

| Row | Wallet said | Target said | Signature verifies against |
|---|---|---|---|
| fully explicit | complete | mandatory script verify flag failed, invalid Schnorr signature | the output-side-emptied candidate |
| two confidential outputs | complete | accepted | the candidate with both vectors grown |

The four candidates were distinct in both rows, so no verdict rests on a coincidence, and the accepted row is the control that establishes the candidates are being modelled correctly rather than merely differing.

The verdict has two halves and they point in opposite directions, which is why the concept was right to charter a reproduction instead of asserting a finding.

**The input-side term is length-dependent.** Emptying the input-witness vector alone changed the message in both rows. The term therefore carries the same hazard shape as its output-side sibling for any signer that forms the message from witnessless bytes — which is exactly what a first-party signer working from the protected preimage would be.

**The hazard is not observed in the target's own signer.** The explicit row's signature verifies against the output-side-emptied candidate and against nothing else, so the wallet hashed the input-witness vector at its consensus length and the output-witness vector at zero. The source says why: `CWallet::SignTransaction` resizes the input-witness vector to the input count at `src/wallet/wallet.cpp:2315` before any signing begins, and the precompute that fixes both terms is taken at `src/script/sign.cpp:790` and `:805` from a transaction that has already been through that resize. The output-witness vector has no such resize on any signing path. The PSBT path grows the same vector at `src/psbt.cpp:189`, so the asymmetry is not an accident of one interface.

The honest statement is therefore: the twin is **confirmed as a structural property of the message and refuted as an observed defect of this target's signer**. The recorded output-witness hazard stands exactly as recorded, and it stands alone in that signer.

That is why this finding earns no new entry at the upstream register. The register's own bar is that a friction cost an investigation, forced an adaptation, or blocks a capability; the input-side length dependence costs this repository nothing that the output-side one does not already cost, and the output-side one is filed and labelled (`obs:upstream:eg-019`).

### What it costs the census · `rule:sighash-review:census-consequence`

The accepted result carries the output-witness vector at its consensus length and does not carry the input-witness vector. That asymmetry is correct for the shapes this arc produces and it rests on a condition that must be checked rather than assumed.

The condition is that no input bears an issuance. When that holds, every input-witness entry is default-constructed, its two issuance rangeproofs serialize to one zero byte each — the run measured two bytes per entry, `0000` — and the whole term is a function of the input count alone. The input count is in the protected bytes, so the term is recoverable and the census needs no field for it.

When it does not hold, the entries carry real rangeproof bytes that the preimage does not contain in any encoded form, and the term becomes unrecoverable in exactly the way the proof-bearing lane's output-witness term is. The selected profile requires the `Issuance` dimension, so an issuance-bearing candidate is admissible under the profile even though no shape this arc builds today has one.

So the census's silence about the input side is a claim with a precondition, and the precondition belongs in the wave that builds the census: an issuance-bearing candidate needs either a census field for the input-witness issuance proofs or a typed refusal saying the profile does not admit that shape.

## Message terms no dimension names · `tab:sighash-review:unnamed-terms`

Seven of the nineteen terms are named by no dimension of the vocabulary. Naming them here is the point of a census: what is not a dimension is either a constant a test asserts or an input the census must carry, and the difference decides whether a later wave is choosing a value or checking one. Term 16 is listed with them only so the count can be checked.

| Term | Disposition under the accepted rulings |
|---|---|
| 0, tagged-hash seed | A census input. The deployment's genesis block hash is not derivable from any candidate, and two identical candidates on two chains have different messages. |
| 1, hash type | A constant, `0x00`, fixed by the profile ruling. A returned byte that disagrees is a refusal and not a variant. |
| 13, spend type | A constant, `0x02`, fixed by the script-path ruling and the refused annex together. |
| 14, input index | A census input, one per signing request, which the finalized form already builds one of per receipt input. |
| 15, annex hash | Absent by the profile's refusal of the annex, which is what makes term 13 a constant. |
| 17, key version | A constant, `0x00`, fixed by the target rather than by the profile: `:2702` sets it for every tapscript spend and the comment there says an upgraded key version would arrive as a new signature version. |
| 18, codeseparator position | A census field whose value is `0xffffffff` for every leaf this arc emits, because `:581` sets that value at the head of evaluation and only `:1472` moves it. No leaf in this workspace emits the opcode that moves it, so the constancy is a condition on the leaf vocabulary rather than a property of the target, and the census carries the field rather than assuming the condition. |
| 16, tapleaf hash | Named by a dimension, and listed here only to keep the count honest: it is the one script-path term the vocabulary does name. |

The arithmetic is therefore: two of the seven are values nothing derives and the census must carry, the genesis hash and the input index; three become constants a test asserts, the hash-type byte, the spend-type byte and the key version; one is absent by the annex refusal; and one is a census field whose value is constant while the leaf vocabulary stays as it is. That is what the profile ruling meant by turning message terms into constants, counted rather than claimed.

## What this review establishes about the capability, which is nothing · `rem:sighash-review:capability-state`

`SighashCapability` is unreviewed at this document's completion and this document does not change that.

The capability is constructed at `packages/target-elements/src/authorization.rs:419-423` with an empty reviewed set, every dimension in the unreviewed set, and the sighash-semantics evidence requirement. The type's own doctrine at `:44-60` says why the split exists: the earlier review covered the signature primitives and not the message construction, and a set of booleans would have forced answers to questions nobody asked.

Under the accepted evidence ruling, a source reading is one of three things a dimension needs and not the whole of it. The reviewed set moves when a dimension carries both its citation and the observation that exercised it, and no observation is recorded here. So `OwnerSighashProfile::assess` at `packages/tapscript/src/authorization.rs:262-274` still returns review-incomplete naming all eight required dimensions, and the `SighashProfileUnreviewed` residual stands in every vocabulary that carries it.

What this document supplies is the first of the three, for seven of the eight required dimensions, plus a settled answer about the eighth that a later wave must act on before an established disposition is reachable at all.

## Non-claims · `rem:sighash-review:nonclaims`

This review does not claim that a digest was computed, that any digest is authoritative, or that any candidate is signable today.

It does not claim that the target accepts anything, that a recomputation matching a target's digest would establish acceptance, or that an acceptance whose message nobody recomputed would establish which dimensions a signature committed to.

It does not claim that an explicit-lane observation says anything about the proof-bearing lane, and both diagnosis rows it rests on are explicit-lane rows.

It does not claim that confidential funding, materialization, or blinding is implemented, reviewed, or affected, and it leaves the confidential-predecessor blocker untouched.

It does not claim that the single-output observation is a consensus defect, and it does not claim the single-output branch is reachable from any wire form.

It does not claim that any key, signature, or chain it touched is anything but test material on a chain nobody settles on.
