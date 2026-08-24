# Draft: Owner Sighash Profile, Review, and Accepted-Result Concept

> **Status:** Concept draft; not an execution guide; EXTERNAL PARALLEL WORK — the separately reviewed component the confidential-funding guide's Wave 4 waits on, chartered here and implemented by nobody until the owner accepts this charter
> **Phase:** Phase 5 — Live Receipt Transfer; this work supplies the one missing component the phase card names
> **Entry:** the reproduced `OwnerSighashNotComputable` blocker, the `SighashProfileUnreviewed` residual, and the recorded `G11-W11-06` diagnosis
> **Primary semantic operation:** establish what an owner's signature commits to, and expose one accepted result a candidate handoff can bind to
> **Affected packages:** `target-elements` and `tapscript` for the reviewed contract and the selected profile; `transaction` for the finalized form and its signing requests; `vectors` for the blockers and their evidence
> **Evidence support:** the Elements consensus source at reviewed tip `b7fc5d080a`, the recorded `G11-W11-06` diagnosis, the first-party finalization boundary, and the executor's advertised test authorization capability
> **May affect after acceptance:** the reviewed sighash capability, the selected profile and its coverage map, the protected preimage, the signing-request record, the unauthorizing-signature width, evidence plans, package contracts, and the dependency graph
> **Supersedes as concept direction:** the assumption that an owner's message can be formed from the protected preimage alone
> **Does not implement:** confidential funding, the transaction-wide materializer, production signing, wallets, key custody, production randomness, privacy guarantees, or release
> **Required result:** one reviewed owner sighash profile whose every required dimension is established from target source and observed from a target-accepted witness, one typed signing-input census sufficient to form the target's message, and one accepted result the confidential-funding guide's Wave-4 handoff binds to without importing any digest
> **Authority:** Attestation, the typed architecture, implemented ADRs, accepted decisions, and the confidential-funding concept's accepted rulings take precedence; the project owner resolves every charter decision below
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015; every key this work touches is test-scoped material on a chain nobody settles on

---

## Mission · `sec:guide-sighash:mission`

This concept charters the work that makes an owner's authorization computable, reviewed, and bindable — so that the confidential-funding guide's Wave-4 handoff has something to hand to and something to take back, and so that no positive row of the Guide-13 matrix is blocked any longer on a component nobody owns.

The missing component is exact. Section 10.2 of Guide 13 checks an owner signature with the target's own verifying primitive over the target's own taproot sighash, and section 1.7 forbids a builder asserting that digest. Nothing here computes it: `FinalizedLiveTransfer::protected_bytes` hands an owner a preimage instead, and `OwnerSighashNotComputable` records that a preimage is not a digest.

The concept's central finding, established below from target source, is that the gap is wider than "nobody hashed the preimage". The preimage is not a sufficient input to the target's message under any hash type: the message mixes in seven whole-transaction hashes, two of which cover data the preimage omits entirely, and it is seeded with a per-deployment constant the preimage does not carry. A component that hashed `protected_bytes` would produce a number, and not the one the target verifies against.

So the work is three things: establish the message from source, expose the census of inputs a message needs, and produce an accepted result saying which of those the review actually established. The last is the answer to the confidential-funding execution guide's pending decision §4.1, deepened here so the ruling there can cite this entry rather than re-argue it.

---

## One-line thesis · `rem:guide-sighash:thesis`

> The owner sighash becomes computable when the target's message construction is read from source rather than named, when the handoff carries the census of inputs that message actually needs rather than a preimage that omits two of them, when the selected profile's every required dimension is established by a review and observed from a target-accepted witness, and when the accepted result the confidential-funding guide binds to is a typed census plus opaque authorization bytes — never a digest, and never a signed candidate.

---

## Scope and ownership · `sec:guide-sighash:scope`

This work owns the message, the profile, the review, and the accepted result. It owns no funding, no materializer, no blinding, and no candidate construction.

Ownership divides along the boundaries that already exist:

- `target-elements` owns the reviewed contract of the target's authorization behaviour, including `SighashCapability`, whose reviewed set is empty today and whose unreviewed set is every dimension the target offers;
- `tapscript` owns `selected_owner_profile`, `ProtectedDatum`, and the coverage map that argues one dimension carries one protected datum;
- `transaction` owns `FinalizedLiveTransfer` and `LiveSigningRequest`, and therefore what a signing request carries — the surface this work needs widened, and the one place a widening can happen without forking the signing path;
- `vectors` owns the blockers, the matrix classification, and the evidence that moves a standing; a capability existing never moves one.

The confidential-funding guide owns proof finalization, the protected byte set, and the mandatory handoff order. This concept never reopens any of that guide's accepted rulings, nor the three the concept of record accepted — custody, determinism, and canonical-request evidence.

Two things this work does not own are worth naming because a reader will expect them here. The dimension-mapping repair that sends `ProtectedDatum::ProofFields` to the created outputs' dimension as well as the spent outputs' belongs to that guide's Wave 3; this work consumes the amended map and reviews whether the dimensions it names are established. And `NoConfidentialPredecessorCanBeFunded` is not this work's blocker at any wave, under any result, for any row.

---

## Verified target boundary · `sec:guide-sighash:target-boundary`

Every claim in this section was read from the Elements source at reviewed tip `b7fc5d080a` with the checkout clean on `master`. Nothing here was inferred from an RPC name, a function name, or a build; nothing here was executed. Paths and line numbers are at that tip.

### The message, term by term · `def:guide-sighash:message-terms`

`SignatureHashSchnorr` at `src/script/interpreter.cpp:2688-2803` is the whole construction. It writes into a hasher pre-seeded at `src/script/interpreter.cpp:2673` with the tagged hash `TapSighash/elements` — the tag itself at `src/script/interpreter.cpp:552` — followed by the deployment's genesis block hash **twice**. BIP-341's epoch byte is deliberately absent, and the source says so in a retained comment at `src/script/interpreter.cpp:2714-2716`.

The terms, in the order the source writes them, for a non-`ANYONECANPAY` spend whose output type resolves to `SIGHASH_ALL`:

| Term | Width | Source | What it commits to |
|---|---|---|---|
| tagged-hash seed | — | `interpreter.cpp:552`, `:2673` | the tag, then the genesis block hash twice |
| hash type | 1 | `:2722` | the exact type byte, `0x00` for default |
| version | 4 | `:2725` | the transaction version field |
| locktime | 4 | `:2726` | the transaction locktime field |
| outpoint flags hash | 32 | `:2728` | issuance and pegin flags of every input |
| prevouts hash | 32 | `:2729`, `:2380-2386` | every input's outpoint |
| spent asset-and-amount hash | 32 | `:2730`, `:2454-2462` | every spent output's asset and value fields |
| spent scripts hash | 32 | `:2736`, `:2465-2472` | every spent output's script |
| sequences hash | 32 | `:2737`, `:2390-2397` | every input's sequence |
| issuances hash | 32 | `:2738`, `:2402-2412` | every input's issuance, or a zero byte where null |
| issuance rangeproofs hash | 32 | `:2739`, `:2431-2439` | every input witness's two issuance rangeproofs |
| outputs hash | 32 | `:2742`, `:2443-2450` | every output, serialized |
| **output witnesses hash** | 32 | `:2743`, `:2418-2425` | **every output witness, at the vector's own length** |
| spend type | 1 | `:2748-2749` | the script-path flag and whether an annex is present |
| input index | 4 | `:2768` | which input is being authorized |
| annex hash | 32 | `:2770-2772` | present only when an annex is |
| tapleaf hash | 32 | `:2795` | script path only |
| key version | 1 | `:2796` | script path only |
| codeseparator position | 4 | `:2798` | script path only |

The result is one SHA256 over that stream at `src/script/interpreter.cpp:2801`.

Three consequences follow, and each is load-bearing below.

**The message is not a function of the transaction alone.** Four of its terms come from outside the candidate's own serialization: the genesis block hash, the spent outputs' asset, value, and script fields, the executing leaf's hash and codeseparator position, and the annex's presence. A component handed only a transaction cannot form the message for any input.

**The output-witness term is length-dependent.** `GetOutputWitnessesSHA256` at `src/script/interpreter.cpp:2417-2425` iterates `txTo.witness.vtxoutwit` and hashes whatever entries are there; it does not index by output. The vector's length is a property of how the transaction was last deserialized, which the next subsection makes exact.

**The issuance-rangeproof term is length-dependent in the same way.** `GetIssuanceRangeproofsSHA256` at `src/script/interpreter.cpp:2431-2439` iterates `txTo.witness.vtxinwit` with the same shape and the same absence of a per-input index, and its result enters the message at `:2739`. The recorded `G11-W11-06` diagnosis established the output-side hazard; the input-side twin is a hypothesis this concept charters a reproduction for rather than a finding it asserts.

### Where the output-witness vector's length comes from · `rule:guide-sighash:witness-length`

`DeserializeTransaction` at `src/primitives/transaction.h:381-404` clears the witness at `:385` and resizes `vtxoutwit` to the output count at `:398` — **only** when the witness flag is set. A transaction serialized without its witness therefore deserializes with an empty output-witness vector, and a signer working from those bytes hashes the empty string in the message's output-witness position.

`SerializeTransaction` at `src/primitives/transaction.h:448-476` sets that flag when `HasWitness()` holds, and then resizes `vtxoutwit` to the output count at `:474` before writing it. So the moment a transaction carries any witness at all — including the input witness a taproot script-path spend must carry — the vector on the wire has one entry per output, and consensus hashes one entry per output.

`CTxOutWitness` at `src/primitives/txwitness.h:45-63` serializes the surjection proof and then the rangeproof, each length-prefixed. A default-constructed entry is therefore the two bytes `00 00`.

This is the established `G11-W11-06` finding, and `scripts/diagnose-taproot-output-witness-digest.py` is its recorded, runnable diagnosis: it boots a disposable node, spends one taproot output three ways, and for each records which of two candidate digests the produced signature actually verifies against — the one computed with the vector empty, or the one computed with it grown. Its case A is fully explicit, and it is expected to sign the empty-vector digest and be refused. That is the part a reader is most likely to get wrong, so it is stated plainly here: **the hazard is not confined to the confidential lane.**

### What the current preimage omits · `rem:guide-sighash:preimage-gap`

`FinalizedLiveTransfer` computes its protected bytes as `parts.protected.encode_without_witness()` (`packages/transaction/src/live_finalize.rs:281`) and hands exactly those bytes to every signing request (`:393-421`). The module says so in its own words at `:32-42`: what the owner is handed is the message's preimage and the census of what the profile must commit to, and no digest is minted.

Measured against the term table, the preimage supplies the version, the locktime, the inputs, and the outputs. It does not supply, for either lane:

- the deployment's genesis block hash, which seeds the hasher twice;
- the spent outputs' asset, value, and script fields, which three separate terms cover;
- the output-witness vector at its consensus length, which one term covers;
- the input-witness vector's issuance rangeproofs, which one term covers;
- the executing leaf's hash, the key version, and the codeseparator position, which the script path adds;
- whether an annex is present, which the spend-type byte carries.

The distinction between the two lanes is sharp and it decides the second charter decision. For the **explicit** lane the omitted output-witness content is *recoverable* from the preimage: every entry is default-constructed, so the vector's serialization is the two bytes `00 00` repeated once per output, and a component that knows the output count can reconstruct it. For the **proof-bearing** lane it is *not* recoverable: the entries are the rangeproofs themselves, and the preimage does not contain them.

So an accepted result that promises authorization bytes "bound to those exact protected bytes" is promising something the explicit lane can just barely deliver by reconstruction and the private lane cannot deliver at all. That is the deepening the owner ordered.

### Admissible hash types, and what each one changes · `tab:guide-sighash:hash-types`

`src/script/interpreter.cpp:2721` admits `hash_type <= 0x03` or `0x81 <= hash_type <= 0x83`, and refuses everything else by returning false — which `CheckSchnorrSignature` at `:2969-2971` reports as a hash-type script error. `src/script/interpreter.cpp:2719-2720` splits the byte into an output type and an input type.

| Byte | Signature length | Output terms | Input terms | Note |
|---|---|---|---|---|
| `0x00` default | 64 | outputs hash and output-witnesses hash | whole-transaction, seven terms | the type byte itself is still hashed, as `0x00` |
| `0x01` all | 65 | same as default | same as default | a different message from `0x00`, because the byte differs |
| `0x02` none | 65 | none | whole-transaction | every output free after signing |
| `0x03` single | 65 | this output and this output's witness, `:2774-2790` | whole-transaction | other outputs free |
| `0x81`, `0x82`, `0x83` | 65 | as above | this input only, `:2750-2766` | input set open after signing |

Two details of that table are not decoration. A 65-byte signature is one whose trailing byte is the hash type, and `:2958-2966` refuses a trailing `0x00` — so `0x00` and `0x01` are two different profiles, not two spellings of one. And `SIGHASH_RANGEPROOF` is not among the admissible taproot bytes at all: it is a segwit-v0 flag, gated at `:277-279` and consumed at `:2270` and `:2817`, so a profile naming it would be naming a flag the taproot path cannot carry.

The single-output branch at `:2784-2789` indexes `tx_to.witness.vtxoutwit[in_pos]` after checking only `in_pos` against `vout.size()` at `:2775`, and under the length behaviour above the two sizes can differ. This concept claims no defect; it records the read and charters a bounded reproduction, because a profile that refuses `SIGHASH_SINGLE` — as the selected profile already does — never reaches the branch.

### Key path, script path, and one dimension carried indirectly · `rule:guide-sighash:spend-path`

`src/script/interpreter.cpp:2691-2706` sets the extension flag to 0 for a key-path spend and 1 for a script-path spend, and the script path alone appends the tapleaf hash, the key version, and the codeseparator position at `:2793-2798`. `:3265-3300` chooses between them: the annex is stripped and hashed first, a remaining stack of one is a key-path spend, and a stack of more than one is a script-path spend whose control block is checked against the taproot commitment before the leaf executes.

This arc's shapes are script-path spends of tapscript leaves, so three terms exist here that do not exist for a key-path spend — and one of them, the codeseparator position, is a property of *where in the leaf's execution* the check occurred rather than of the transaction.

One consequence deserves its own statement. `selected_owner_profile` at `packages/tapscript/src/authorization.rs:294-370` requires eight of the ten dimensions the target names, and `SighashDimension::InternalKey` is among the eight. The message construction contains no internal-key term. What it contains is the spent scripts hash at `:2736`, and a taproot output's script *is* the tweaked output key — so the internal key is committed indirectly. That is very likely the right answer, and it is exactly the kind of claim a review must establish rather than assume: the coverage map assigns no protected datum to `InternalKey`, so nothing today forces the question. Establishing it is a Wave-1 deliverable.

**Ruling (post-review): `InternalKey` is re-typed onto `SpentOutputs`.** Wave 1's review settled the question against the indirect-commitment reading above, in the internal-key section of plans/reference/owner-sighash-review.md: the spent-scripts term commits the tweaked output key and not the internal key, the binding between the two lives in the control-block check outside the message, and the tweak is not injective in the message's view. Of the review's two admissible repairs the owner selects the first: the internal key's protection is recorded as carried by `SpentOutputs`, where the output key actually is, and `InternalKey` leaves the required set, which becomes seven message-carried dimensions. The coverage map already assigns `InternalKey` no protected datum, so no protected datum moves and the profile's coverage argument is unchanged in content. Wave 2 lands the re-typing; the recorded reason travels with the dimension so a later reader finds the composition argument, not a silent deletion.

### What the review has established so far, which is nothing · `rem:guide-sighash:review-state`

`packages/target-elements/src/authorization.rs:419-423` constructs the reviewed sighash capability with an empty reviewed set and every dimension unreviewed, against the evidence requirement `SighashSemantics`. `OwnerSighashProfile::assess` at `packages/tapscript/src/authorization.rs:262-274` therefore returns `ReviewIncomplete` naming all eight required dimensions, and `SighashProfileUnreviewed` is carried in four vocabularies — the linker's obligations, the tapscript recognition residuals, the transaction crate's ABI obligations as `SelectedSighashProfileUnreviewed`, and the vectors residual set.

The design is deliberate and the source says why at `packages/target-elements/src/authorization.rs:44-60`: the source review covered the signature primitives and not the message construction, and recording the dimensions as booleans would have forced yes-or-no answers to questions the review never asked. This concept's Wave 1 is the review that was never done.

---

## First charter decision: the profile · `rule:guide-sighash:profile-decision`

The project owner selects the sighash profile before any message is formed, because the profile decides which terms are in the message and therefore what every later type carries.

The selected profile is already an all-inputs, all-outputs profile with two refusals, and this concept does not reopen that. What is open is the type byte, the lane discipline, and the annex disposition — three choices the existing profile does not express, because the dimension vocabulary has no term for any of them.

| Option | Type byte | Signature width | Consequence |
|---|---|---|---|
| Default only | `0x00` | 64 | The narrowest message that still covers both output terms. Matches the 64-byte `UNAUTHORIZING_SIGNATURE` every weight in the workspace is already measured with, so no measured figure moves. The type byte cannot be re-used to mean anything else later. |
| Explicit all only | `0x01` | 65 | The same coverage, one byte wider per signature, and every recorded weight in `live_measurements` and `live_pairs` becomes a weight of something the profile no longer produces. Its only advantage is that the type is stated on the wire rather than implied by the signature's length. |
| Both admitted, selected per ceremony | `0x00` or `0x01` | 64 or 65 | Two messages per candidate, two widths, and a report that names one could be read as the other. Buys nothing this arc needs. |
| Anything narrower | `0x02`, `0x03`, or the `0x8x` family | 65 | Refused by the existing profile's own argument, and the refusals have grounds already recorded: a single-output commitment leaves every other output free, and a permitted input extension leaves the consumed receipt set open. |

Two sub-choices ride on the same ruling.

**Lane discipline.** One profile for both lanes, or one per lane. One profile is recommended: the message terms are identical across the lanes — the output-witness term is present in both and only its *content* differs — so a per-lane profile would encode a difference the target does not make, and would invite an explicit-lane success to be read as evidence about the private lane, which the arc's discipline forbids anyway.

**Annex disposition.** The spend-type byte at `src/script/interpreter.cpp:2748` carries annex presence, decided by the stack-shape rule at `:3265-3270`. The recommendation is that the profile *refuses* the annex: this arc's witnesses carry none, an annex is non-standard for relay, and a profile permitting one would have to say what it may contain. A refused annex fixes the spend-type byte at `0x02` for every candidate this arc produces — a checkable constant rather than a variable.

**Recommendation.** Default only, one profile for both lanes, annex refused, script path only. It is the narrowest complete commitment the target offers, it leaves every already-measured weight true, and it turns three message terms into constants a test can assert rather than inputs a builder supplies.

**Ruling: ACCEPTED — default-only type byte, one profile for both lanes, annex refused, script path only.** The recommendation is adopted whole: the three open message terms become constants later waves assert rather than inputs a builder supplies, and every already-measured 64-byte weight stays true. The other three options are closed as profile directions for this work.

---

## Second charter decision: the accepted-result type · `rule:guide-sighash:result-decision`

This is the confidential-funding execution guide's pending decision §4.1, deepened as ordered. That entry asked what the parallel owner-sighash work's accepted result *is*, as a value the guide's Wave-4 handoff can bind to, and offered three options with a recommendation of the first. This entry supersedes those three with four, because the target boundary above shows the first option is not implementable as written.

### What §4.1 assumed, and what the source says · `rem:guide-sighash:41-correction`

§4.1's recommended option was an accepted-profile token plus opaque authorization bytes, on the stated ground that the handoff hands out protected bytes and target spent-output data and takes back opaque authorization bytes bound to those exact protected bytes — the boundary it says `LiveSigningRequest` and `FinalizedLiveTransfer::check_offered` already implement for the explicit lane.

The direction is right: nothing about the digest should cross, and `check_offered` is the right pattern for binding an answer to one candidate. The claim that the existing boundary already implements it is not. `LiveSigningRequest` carries the input index, the owner, the representation plan, the protected bytes, the leaf role, the required dimensions, and the protected data (`packages/transaction/src/live_finalize.rs:493-501`). It carries no spent-output data, no genesis hash, no output-witness vector, no leaf hash, no codeseparator position, and no annex disposition. §4.1's own prose names spent-output data as something the handoff supplies; the type it points at does not supply it.

So the correction is not a change of direction. It is that the boundary §4.1 pointed at is a *smaller* boundary than the message needs, in five named places, for both lanes.

### Byte-level worked examples · `def:guide-sighash:worked-examples`

Every preimage below is written exactly. No digest value is printed: this document establishes what is hashed, and the wave that implements the work computes the hashes — one computation is never both observation and expectation.

**Example E — the explicit lane, three outputs, one taproot script-path input.**

Two destination outputs and one fee output, no proofs. The finalized form must still carry an input witness, so `HasWitness()` holds and the wire form's output-witness vector is grown to three entries.

```text
what consensus hashes in the output-witness position
    entry 0    00 00        empty surjection proof, empty rangeproof
    entry 1    00 00
    entry 2    00 00
    preimage   00 00 00 00 00 00        six bytes

what a signer working from encode_without_witness() hashes there
    preimage   <empty>                  zero bytes
```

The two preimages differ, so the two messages differ, so the signature is complete and invalid — precisely the target's recorded answer in the phase card's native run of record.

The recovery is arithmetic: three outputs means three entries means six zero bytes, and a component told the output count can rebuild it. That is why option B below is viable at all for this lane, and why an explicit-lane success would prove nothing about the private one.

**Example P — the proof-bearing lane, two confidential destinations and one fee output.**

Under the confidential-funding concept's required form each protocol output carries a valid rangeproof and an empty surjection proof; the fee output carries neither.

```text
what consensus hashes in the output-witness position
    entry 0    00                       empty surjection proof
               <compact size> <rangeproof bytes>
    entry 1    00
               <compact size> <rangeproof bytes>
    entry 2    00 00

what a signer working from encode_without_witness() hashes there
    preimage   <empty>
```

Nothing about entry 0's or entry 1's rangeproof bytes is present in the preimage, in any encoded form, at any length, and no arithmetic recovers them. A result type binding only to the protected bytes is, for this lane, a result type binding to a message the target never forms.

**Example D — the deployment seed.**

```text
hasher state before any transaction data
    tag        SHA256("TapSighash/elements") twice, per TaggedHash
    then       <32-byte genesis block hash>
    then       <32-byte genesis block hash>            again
```

Two candidates identical to the last byte, signed against two different regtest chains, have different messages. A result bound to protected bytes alone is bound to a value that does not determine the message, so a signature produced against one deployment would pass as valid material for another and be refused on chain.

### The options · `tab:guide-sighash:result-options`

| Option | Out | Back | Consequence |
|---|---|---|---|
| **A. Protected bytes and opaque authorization** — §4.1's first option, verbatim | protected bytes, an accepted-profile token | opaque bytes per owner, bound to those protected bytes | Not implementable. Five message terms are absent from the preimage, and for the private lane one is unrecoverable. It promises a binding to a value that does not determine the message. |
| **B. Protected bytes, a signing-input census, and opaque authorization** | protected bytes, the output-witness vector at consensus length, the spent-output census per input, the deployment's genesis hash, and per input the index, leaf hash, leaf version, codeseparator position, and annex disposition | opaque bytes per owner, plus the hash-type byte used, bound to one candidate identity | The message becomes formable. Nothing about the digest crosses either way, and the census records what the target reads, not what anyone computed. It is wider than `LiveSigningRequest` by five fields, and widening that type is an edit this arc already owns. |
| **C. A digest-producing capability** — §4.1's second option | a call | a 32-byte message per input | Imports the digest design into the consuming guide, which its §1.5 forbids, and inverts the honest direction: §1.7 requires an observed or recomputed digest, not an asserted one. |
| **D. A fully authorized candidate** — §4.1's third option | the candidate | a signed candidate | Moves finalization across the boundary the concept of record fixed, and makes the protected-bytes refusal unenforceable: the party that could mutate a protected byte is the party returning the result. |
| **E. A witness-completed candidate** | the candidate | the input witness stacks only | Softer than D and still wrong: the returning party decides the annex and the control block, both message terms, so the census the consuming guide validated is not the census that was signed. |

**Recommendation.** Option B, with three constraints that make it a boundary rather than a channel.

The census is *observed target data plus candidate structure*, never an opening, blinder, nonce input, key, or proof input. The confidential-funding concept already fixes that the handoff supplies target spent-output data without openings and imposes no digest need for openings; option B keeps that exactly.

The census is *bound*, not accompanying. The candidate identity a returned authorization names is the identity of the census-and-protected-bytes pair, so an authorization for a candidate whose output-witness vector changed is a `WrongCandidate` refusal in the pattern `check_offered` already sets — exact bytes compared, rather than trusting that the signer looked.

The returned hash-type byte is *checked against the profile*, not recorded. A returned `0x01` under a default-only profile is a refusal and not a variant: the two produce different messages, and a report accepting both would be naming a profile it did not hold to.

**What changed against §4.1's three options.** The direction survives — option B is option A's shape. What changed is that option A's binding target is insufficient, by five named fields and not as a matter of taste; that the accepted result must be a *type* the consuming guide validates rather than a token it trusts; and that the answer carries one field option A did not have, the hash-type byte, because the profile decision above makes that byte load-bearing. Options C and D are re-recorded with their original grounds and closed for the same reasons; option E is new and closed as a softer form of D.

**Ruling: ACCEPTED — option B, with its three constraints.** The accepted result is protected bytes, the signing-input census, and opaque authorization bytes plus the returned hash-type byte, bound to one candidate identity. The census is observed target data plus candidate structure and never an opening, blinder, nonce input, key, or proof input; the binding is exact-byte in the pattern `check_offered` already sets; the returned hash-type byte is checked against the profile, never merely recorded. Options A, C, D, and E are closed. This ruling resolves the confidential-funding execution guide's pending decision on the accepted-result type, whose entry now cites this one.

### The failure matrix · `tab:guide-sighash:failure-matrix`

Every row is a way the handoff can be wrong. Each has a distinct *observed* symptom and a distinct owner, so that a run refusing for one reason is never filed as evidence about another. Construction and protocol refusals are never target verdicts, and the target column says what the target would actually say — not what a caller would conclude.

| # | Failure | Target's answer | Detected by | Owner |
|---|---|---|---|---|
| 1 | Message formed from the preimage alone, output-witness term empty | invalid Schnorr signature | recomputation against both candidate digests, as the recorded diagnosis does | this work |
| 2 | Output-witness vector at the wrong length | invalid Schnorr signature | census length compared against the output count before signing | this work |
| 3 | Issuance-rangeproof term at the wrong input-witness length | invalid Schnorr signature | the Wave-1 reproduction; unestablished today | this work |
| 4 | Genesis hash of another deployment | invalid Schnorr signature | census bound to the run's deployment record | this work |
| 5 | Returned hash-type byte disagrees with the profile | invalid signature, or a hash-type script error outside the admissible set | profile check on the returned answer | this work |
| 6 | Annex present but undeclared, or declared and absent | invalid Schnorr signature | spend-type byte recomputed from the finalized witness | this work |
| 7 | Codeseparator position wrong | invalid Schnorr signature | recomputation from the executing leaf | this work |
| 8 | Leaf hash wrong, or the control block does not commit | witness program mismatch, before any signature check | the target's own control-block check | this work |
| 9 | Spent-output census wrong or short | missing-data failure, or an invalid signature | census cardinality checked against the input count | this work |
| 10 | Signature 64 bytes where 65 is required, or the reverse | signature size error, or a hash-type error for a trailing zero | width checked against the profile | this work |
| 11 | Wrong input index | invalid Schnorr signature | one request per receipt input, as the finalized form already builds | consuming guide |
| 12 | Authorization bound to another candidate | never submitted | `WrongCandidate`, by exact byte comparison | consuming guide |
| 13 | A protected byte mutated after signing started | never submitted | the finalized form's three post-boundary rejections | consuming guide |
| 14 | A rangeproof byte mutated after finalization | invalid signature if submitted | the protected-bytes repair's own test: protected bytes that did not change when a proof changed are refused | consuming guide |
| 15 | Key encoding unrecognized and non-empty | the check reports success **without verifying** | the owner-key encoding gate the workspace already requires | the existing owner-key obligation |

Row 15 is not this work's to repair, and it is listed because a review of the message that ignored it would let an unverified success be read as an authorization. The reviewed contract already names the behaviour and already requires the pattern to authenticate the encoding itself.

---

## Third charter decision: evidence and review · `rule:guide-sighash:evidence-decision`

The owner selects what establishes the profile, and what "reviewed" means when the review completes.

The arc's constraint on the answer is that this work's evidence is *its own*. It may not consume funding evidence, and it may not be consumed as transfer, safety, minimality, or resource evidence. Those are distinct roles, and one ceremony recording several of them still keeps them apart.

| Option | What would move `SighashProfileUnreviewed` to established | Consequence |
|---|---|---|
| Source review alone | a dimension-by-dimension reading of the message construction at the pinned tip, with citations | Establishes what the source says and nothing about what a node does. The contract's own doctrine is that an unreviewed dimension is unavailable; this would move one to reviewed with nothing executed. |
| Source review plus recomputation | the reading, plus an independently written message construction reproducing the target's digest for a known witness | Two origins, which is the standing rule. Still no target verdict: a matching recomputation says the model is right about the message, not that a node accepts a spend built from it. |
| Source review, recomputation, and one observed acceptance | the above, plus one target-accepted spend of a first-party candidate whose signature verifies against the recomputed message | The full chain, and it is reachable on the explicit lane without any confidential funding. |
| Observed acceptance alone | a node accepted something | An acceptance whose message nobody recomputed establishes that some signature was valid, not which dimensions it committed to. This is the option that would let a narrower profile pass as the selected one. |

**Recommendation.** The third. The review's verdict is a `SighashCapability` whose reviewed set was populated dimension by dimension, each carrying both its source citation and the observation that exercised it, under the `SighashSemantics` evidence requirement the contract already names.

Two boundaries ride with it.

**The review boundary is this work's own.** The confidential-funding concept's sighash-boundary rule puts the digest work under its own review and its own typed blocker, and this concept accepts that placement rather than negotiating it. The consequence is symmetric: no wave of that guide reviews this profile, and no wave of this work reviews funding, a materializer, or a blinder.

**A dimension is established or it is not.** The contract offers no third state and this work adds none. A dimension the review reached but could not exercise stays unreviewed with a recorded reason, and the disposition stays `ReviewIncomplete` naming it. An `Established` disposition with one dimension exercised by nothing would be the exact overstatement the type exists to prevent.

**Ruling: ACCEPTED — source review, recomputation, and one observed acceptance.** The reviewed set is populated dimension by dimension, each carrying both its source citation and the observation that exercised it; the review boundary stands in both directions; a dimension the review could not exercise stays unreviewed with a recorded reason and the disposition stays `ReviewIncomplete` naming it. The other three options are closed.

---

## The two typed blockers, quantified · `sec:guide-sighash:blockers`

The counts below are the ones the source asserts at tree `0.5.1-dev`. Nothing here was executed; where a number is arithmetic over published counts rather than a read constant, it says so.

### What `OwnerSighashNotComputable` blocks today · `tab:guide-sighash:blocked-rows`

The Guide-13 safety matrix has 108 rows (`packages/vectors/src/live_safety.rs:1637`), in seven sections: 16 positive explicit, 10 positive private, 14 owner-signature fault, 16 object fault, 22 value fault, 13 sponsor fault, 17 structural fault (`packages/vectors/src/live_safety.rs:1630-1636`).

| Standing | Rows | Source |
|---|---|---|
| First-party discharged | 25 | `live_evidence.rs:854` |
| First-party undischarged | 0 | `live_evidence.rs:855` |
| Native run required | 0 | `live_evidence.rs:1036` |
| Report-layer answerable | 2 | `live_evidence.rs:1063` |
| Operation-vocabulary closed | 1 | `live_evidence.rs:893` |
| Experimental | 0 | census cross-foot, `live_evidence.rs:866-875` |
| Infrastructure blocked | 80 | arithmetic against the 108 cross-foot |

Inside the 80:

| Blocker | Rows | Source |
|---|---|---|
| `OwnerSighashNotComputable` | 26 | every positive row, `live_evidence.rs:603-604` and `:1026`; the constant at `:549-550` |
| `PredecessorConstructorAbsent` | 1 | the time-locked-input row, `live_evidence.rs:560` |
| `SponsorEnvelopeSignerAbsent` | 1 | the missing-sponsor-authorization row, `live_evidence.rs:561-563` |
| `RawSurgeryPathAbsent` | 1 | the raw-surgery row, `live_evidence.rs:564-566` |
| `NoAcceptingControlExists` | 51 | arithmetic; the branch at `live_evidence.rs:609-613`, asserted nonzero at `:1041-1047` |

The 51 are the sharpest part of the count, and they are why this work is worth more than its 26 rows. `NoAcceptingControlExists` is derived rather than declared: `a_positive_control_exists()` at `packages/vectors/src/live_evidence.rs:538-540` returns false, and its recorded reason is that no owner signature can be produced over the digest the target's verifying primitive forms. So 77 of the 108 rows stand behind this one component — 26 by name, 51 by derivation.

Beyond the matrix, the same blocker is carried in five further places:

- every member of every minimality pair has the verdict `NotSubmitted(OwnerSighashNotComputable)` — 10 members across 5 pairs (`live_pairs.rs:1159-1161`, `:1667-1671`);
- the minimality condition `BothTargetTransactionsAccept` is blocked by it for all 5 pairs, and `BothProjectionsEqualTheExpectedTransfer` awaits the verdicts that blocking prevents (`live_pairs.rs:1356-1364`);
- the minimality failure mode `PrivateMaterializationRejects` awaits a target run for it (`live_minimality_report.rs:659-662`);
- every resource case's `ConsensusVerdict` is `NotClaimable(NoTargetVerdictExists(OwnerSighashNotComputable))`, and `RelayPolicyVerdict` is not claimable without a consensus one (`live_measurements.rs:1297-1300`, `:1575-1584`);
- the safety report's own run standing is `NoRunRequested(OwnerSighashNotComputable)` (`live_report.rs:550`).

One detail deserves its own line because the first charter decision moves it. `UNAUTHORIZING_SIGNATURE` at `packages/vectors/src/live_evidence.rs:822` is 64 bytes of `0x5c`, filling every signature position so that a candidate can be serialized and weighed at all. Sixty-four bytes is the default-type width. Every measured weight in the workspace is therefore already a weight of a candidate signed under the recommended profile, and a ruling for the explicit-all option would make every one of those figures a weight of something the profile no longer produces.

### What clearing it does, and what it does not · `rule:guide-sighash:clearing`

Clearing `OwnerSighashNotComputable` answers no row. It converts standings, and rows move only on observed acceptance.

- The 26 positive rows stop being infrastructure-blocked and become rows a run could answer. The 16 explicit ones are reachable by this work alone. The 10 private ones additionally need the confidential predecessor, which is the other guide's to fund, and this work never touches that blocker.
- The 51 derived rows stop being blocked the moment a positive control exists, and become rows awaiting a run. None of them is answered by the conversion.
- The three specifically blocked rows do not move at all.
- The minimality pairs gain one of their two missing conditions; the other is the confidential predecessor's.
- The resource dimensions become claimable in principle, and stay unclaimed until a verdict exists.

`SighashProfileUnreviewed` is a separate residual and is cleared separately, by the review verdict rather than by a run. The source already states why the two are apart: a digest could be computed tomorrow and the semantic claim about what it commits to would still be candidate-scoped until the review completes. This work clears both, in that order, and never simultaneously.

`NoConfidentialPredecessorCanBeFunded` is untouched by every wave below, under every ruling, for every row.

---

## Implementation waves · `sec:guide-sighash:waves`

Six waves. The ordering constraint the confidential-funding concept fixed is mandatory and is carried unchanged: digest review proceeds in parallel with that guide's Waves 0 through 3, and integration waits until proof finalization and this work's accepted result are both independently accepted. Neither alone is entry.

```text
this work        W0 ─ W1 ─ W2 ─ W3 ─ W4 ──────────────┐
                                                       ├─ W5 ─→ ct-funding W4
ct-funding       W0 ─ W1 ─ W2 ─ W3 ────────────────────┘
```

### Wave 0 — Record the rulings and fix the boundary · `task:guide-sighash:wave0`

**Deliverables**

- the three charter rulings carried forward as recorded, not reopened;
- the profile's type byte, lane discipline, annex disposition, and spend path stated as the constants later waves assert;
- the signing-input census named field by field, with its exclusions — no opening, blinder, nonce input, key, or proof input — as a rule and not an omission;
- the review boundary recorded in both directions, and the census's ADR-015 disposition recorded as public test material under the existing test-material rule;
- the package and dependency boundary recorded without implementation by implication.

**Suggested commit**

```text
plans: decide the owner sighash profile and accepted result
```

### Wave 1 — Review the message from source · `task:guide-sighash:wave1`

**Entry condition** — Wave 0's rulings are recorded.

**Deliverables**

- one dimension-by-dimension review at the pinned tip, each required dimension carrying its citation and the term that carries it;
- the internal-key question settled: whether the spent-scripts term commits it, or the profile's requirement is mis-stated;
- the two refused dimensions checked against the branches that would have carried them, so a refusal is a claim with content;
- the issuance-rangeproof length hazard reproduced or refuted as the output-witness hazard's twin, by the diagnosis's own method;
- the single-output witness indexing recorded as an observation, with the profile's refusal noted as why this arc never reaches it;
- an upstream-friction label for anything the review finds that is the target's behaviour rather than this workspace's;
- `SighashCapability` still unreviewed at wave end: a source review alone populates nothing.

**Suggested commit**

```text
target-elements: review the taproot message construction
```

### Wave 2 — The census and an independent construction · `task:guide-sighash:wave2`

**Entry condition** — Wave 1's review is complete.

**Deliverables**

- the signing-input census as a typed value, with every field option B names;
- the census constructible only from a proof-finalized candidate, on the finalized form's own pattern — no public constructor, no route skipping finalization;
- an independently written message construction consuming the census, whose expectation comes from the source review rather than from the same code path;
- the profile's constants asserted as constants: spend-type byte, type byte, signature width;
- typed refusals distinguishing at least census cardinality mismatch, output-witness length mismatch, annex disagreement, deployment mismatch, a leaf hash not committing under the control block, and a type byte outside the profile;
- no digest asserted as authoritative: the wave produces a candidate recomputation, and the residual stays.

**Suggested commit**

```text
transaction: census the owner signing inputs
```

### Wave 3 — Observe the profile on the explicit lane · `task:guide-sighash:wave3`

**Entry condition** — Wave 2's construction reproduces the recorded diagnosis's two candidate digests.

**Deliverables**

- one finalized explicit candidate whose owner authorization is produced against the Wave-2 message and submitted;
- the target's verdict recorded at the observed outcome layer, with construction and infrastructure failures kept distinct from it;
- the accepted witness read back and the signature re-verified against the recomputed message, so acceptance and recomputation are two origins;
- negative controls: a signature over the empty-vector message; over another deployment's message; carrying the non-selected type byte; and the owner-key encoding negatives the existing obligation enumerates;
- `OwnerSighashNotComputable` cleared at its owning boundary, on the observed acceptance and never on a capability existing;
- an explicit statement that this establishes nothing about the proof-bearing lane, because the recoverability argument does not hold for a rangeproof.

**Suggested commit**

```text
vectors: record an observed owner authorization
```

### Wave 4 — The review verdict and the accepted result · `task:guide-sighash:wave4`

**Entry condition** — Wave 3's observation is validated.

**Deliverables**

- `SighashCapability` populated dimension by dimension, each carrying its citation and the observation that exercised it, and anything unexercised left unreviewed with its reason;
- the profile's disposition recomputed rather than declared, `Established` only if every required dimension is reviewed;
- the accepted result as a typed value under the accepted option, with the returned hash-type byte checked against the profile;
- `SighashProfileUnreviewed` cleared in all four vocabularies, or explicitly not cleared with the dimension that stopped it named;
- the accepted result documented in the shape the consuming guide's handoff takes, as the thing its §4.1 ruling cites;
- a typed stopped result if a required dimension cannot be established: stopped is valid, overstated is not.

**Suggested commit**

```text
tapscript: establish the selected owner sighash profile
```

### Wave 5 — Hand the accepted result across · `task:guide-sighash:wave5`

**Entry condition** — this work's Wave 4 has reached its accepted result, **and** the confidential-funding guide's proof finalization is independently accepted. Neither alone is entry.

**Deliverables**

- the proof-finalized candidate's census built from the other guide's finalized form, with the output-witness vector at its real proof-bearing length;
- one proof-bearing owner authorization produced, submitted, accepted, and re-verified against the recomputed message;
- the protected-bytes repair exercised from this side: a candidate whose rangeproof changed and whose protected bytes did not is refused before any message is formed;
- the private lane's own negative controls, distinct from the explicit lane's;
- the row conversions recorded honestly: what became answerable, what became a run away, and what did not move;
- an explicit statement that the funding, materialization, and blinding this candidate rests on belong to the other guide and are not evidenced here.

**Suggested commit**

```text
vectors: record a proof-bearing owner authorization
```

---

## Non-claims · `sec:guide-sighash:nonclaims`

Carried by the result rather than left to implication, and checked rather than written down.

This work never claims:

- production signing, production key custody, production randomness, production nonce generation, erasure, or side-channel resistance;
- a production multi-owner signing protocol, or any statement about how real owners would coordinate;
- wallet compatibility, wallet correctness, or that any wallet computes the message this review establishes;
- that a key it touches is anything but test material on a chain nobody settles on;
- owner anonymity, graph privacy, count privacy, timing privacy, or any privacy property;
- that an explicit-lane acceptance establishes proof-bearing signing;
- that a recomputation matching a target's digest establishes that a target accepts a spend;
- that an acceptance whose message nobody recomputed establishes which dimensions the signature committed to;
- that confidential funding, materialization, or blinding is implemented, reviewed, or accepted by this work;
- that `NoConfidentialPredecessorCanBeFunded` is affected in any way;
- that clearing a blocker answers a row, or that a capability existing moves one;
- that the reviewed tip's behaviour is the behaviour of any other tip, any other target, or any deployment this work did not run against;
- that a candidate type is final, stable, production-capable, or released.

The result is candidate-only. A secret-bearing selection stops at ADR-015 rather than weakening the boundary, and the census's exclusion rule is what keeps the question from arising: the handoff carries observed target data and candidate structure, never an opening.

---

## Acceptance and rejection · `sec:guide-sighash:acceptance`

The concept is acceptable for translation into an execution guide only when:

- the owner has recorded the profile, accepted-result, and evidence rulings;
- the verified target boundary is established from source at the pinned tip with citations, and the two length-dependent terms are named as such;
- the accepted-result entry is written so that the confidential-funding guide's §4.1 ruling can cite it, with the correction to that entry's first option stated rather than implied;
- the failure matrix distinguishes construction refusals, protocol refusals, and target verdicts, and assigns each row an owner;
- the blocker quantification separates what this work clears from what it converts and from what it does not touch;
- the six waves preserve the mandatory ordering, and Wave 5's two-part entry condition is stated as a conjunction;
- the review boundary is recorded in both directions;
- production, privacy, and cross-lane non-claims are carried by the result.

Implementation acceptance requires one observed explicit-lane acceptance whose signature re-verifies against an independently recomputed message, one populated reviewed capability whose disposition is recomputed rather than declared, and one accepted result the consuming guide validates rather than trusts. No stronger claim.

Reject or stop the concept when:

- a digest is asserted by a builder rather than observed or recomputed;
- the accepted result binds to the protected bytes alone, or crosses a digest in either direction;
- a dimension is moved to reviewed by a source reading with nothing exercising it, or by an acceptance nobody recomputed;
- an explicit-lane result is read as evidence about the proof-bearing lane;
- a blocker is cleared on a capability existing rather than on an observed result;
- this work reviews confidential funding, or the confidential-funding guide reviews this profile;
- the census acquires an opening, a blinder, a nonce input, a key, or a proof input;
- a narrower profile is admitted without a separate argument that it preserves every protected datum.

---

## Identity, schema, security, and handoff · `sec:guide-sighash:impact`

**Identity.** This concept adds no architecture operation, phase, release identity, or digest. It is external parallel work under its own charter and its own review, as the concept of record's sighash-boundary rule places it, and it is not a numbered guide. Every type it charters is candidate-only and subordinate to existing identities.

**Schema.** The signing-input census is a plain typed value in process and carries no interchange obligation in that form; a serialized or archival form is bound by ADR-022 under that ADR's own scope rules, on the same form-not-crate-boundary criterion the concept of record accepted. No schema identity is minted by this concept.

**Security.** Every key, signature, and chain this work touches is test material under the existing test-material rule: disposable, regtest-scoped, and authorizing nothing anywhere else. The census excludes openings by rule, so no secret-bearing interface is selected and ADR-015's design gate is not reached. The unrecognized-key behaviour the reviewed contract already records stays an upstream friction and an independent obligation on the owner check, not a repair this work performs.

**Dependencies.** The execution guide names where the independent message construction lives, what it depends on, and its graph effect. This concept approves no dependency by naming a need.

**Handoff.** The accepted result is the value the confidential-funding execution guide's §4.1 ruling cites and its Wave-4 entry condition names. That guide's Wave 4 does not start until the ruling is recorded and this work's Wave 4 has reached its accepted result. The closeout report records the selected profile, the reviewed dimensions with what exercised each, the observed acceptances, the blockers cleared, the standings converted, the rows moved — which may honestly be none — and the non-claims that survive. A typed stopped result is valid; an overstated one is not.
