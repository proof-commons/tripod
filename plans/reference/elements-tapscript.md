# Elements Tapscript Capability Reference · `ref:elements:tapscript`

> **Status:** Human review reference
> **Upstream:** <https://github.com/ElementsProject/elements>
> **Target owner:** [`target-elements`](../packages/target-elements.md)
> **Policy:** (`[ADR011-rule:toolchain:target-compatibility]`)
> **Machine-consumed:** no

## Scope · `sec:elements-ref:scope`

This reference summarizes Liquid/Elements tapscript capability families
relevant to the attestation contract's implementation.

It helps reviewers locate:

- execution-domain rules;
- opcode families;
- transaction introspection;
- fixed-width arithmetic;
- hashing and byte operations;
- signatures and timelocks;
- confidential transaction behavior;
- issuance;
- target resource and policy limits;
- known evidence gaps.

It does not:

- define protocol semantics;
- define backend patterns;
- pin an Elements consensus implementation as protocol identity;
- prove production activation;
- select a sighash profile;
- approve authenticated opening or constructor proofs;
- provide deployment evidence;
- become compiler or backend input.

Machine-consumed target facts live in typed Rust.

## Evidence levels · `tab:elements-ref:evidence-levels`

| Level | Meaning |
|---|---|
| surveyed | This document identifies a relevant behavior |
| reviewed | Behavior was checked against deployed capability and, where useful, upstream implementation |
| typed | `target-elements` encodes the exact contract |
| pattern-complete | Backend has a complete tested proof pattern |
| deployment-evidenced | Exact target/deployment report verifies the claim |

A surveyed primitive is not automatically an approved attestation-contract proof.

## Provenance policy · `rule:elements-ref:provenance`

The target review records, as appropriate:

- upstream repository and source location;
- revision consulted during review;
- node/library version used by tests;
- network and genesis;
- activation/configuration;
- upstream license for copied excerpts;
- first-party integration test name.

Implementation revision is review or test provenance, not protocol or target
identity.

Relevant upstream areas commonly include equivalents of:

```text
script opcode declarations
script interpreter semantics
tapscript OP_SUCCESS treatment
network activation configuration
taproot sighash documentation/implementation
functional tapscript opcode tests
transaction and CT serialization
policy/resource limits
```

Paths may move. The review records the actual locations consulted.

## Capability groups · `tab:elements-ref:groups`

| Group | Provisional target use |
|---|---|
| streaming SHA-256 | structured constructor and large preimage hashing |
| input introspection | outpoint, asset, value, program, sequence, issuance |
| current input index | local range membership and coordinator selection |
| output introspection | asset, value, nonce, program, family closure |
| transaction introspection | version, locktime, input/output counts, weight |
| signed fixed-width arithmetic | narrow arithmetic and wide-limb components |
| numeric conversion | script-number and fixed-width interoperability |
| elliptic-curve verification | constructor/tweak and possible opening patterns |
| signatures | owner/operator/sponsor authorization |
| relative timelocks | cycle cadence |
| byte operations | structured encodings, point construction, masks |
| CT consensus | confidential value conservation |
| issuance/reissuance | `U`, `ENT`, and `DIST_CTL` substrate support |

Exact opcode numbers and stack behavior belong in the typed target registry.

## Surveyed opcode registry · `tab:elements-ref:opcodes`

> Provisional survey facts; verify before encoding them in `target-elements`.

| Code | Surveyed name | Group |
|---:|---|---|
| 196 | `OP_SHA256INITIALIZE` | streaming SHA-256 |
| 197 | `OP_SHA256UPDATE` | streaming SHA-256 |
| 198 | `OP_SHA256FINALIZE` | streaming SHA-256 |
| 199 | `OP_INSPECTINPUTOUTPOINT` | input introspection |
| 200 | `OP_INSPECTINPUTASSET` | input introspection |
| 201 | `OP_INSPECTINPUTVALUE` | input introspection |
| 202 | `OP_INSPECTINPUTSCRIPTPUBKEY` | input introspection |
| 203 | `OP_INSPECTINPUTSEQUENCE` | input introspection |
| 204 | `OP_INSPECTINPUTISSUANCE` | input introspection |
| 205 | `OP_PUSHCURRENTINPUTINDEX` | current input |
| 206 | `OP_INSPECTOUTPUTASSET` | output introspection |
| 207 | `OP_INSPECTOUTPUTVALUE` | output introspection |
| 208 | `OP_INSPECTOUTPUTNONCE` | output introspection |
| 209 | `OP_INSPECTOUTPUTSCRIPTPUBKEY` | output introspection |
| 210–214 | `OP_INSPECTVERSION` through `OP_TXWEIGHT` | transaction introspection |
| 215–219 | `OP_ADD64` through `OP_NEG64` | signed fixed-width arithmetic |
| 220–223 | signed 64-bit comparisons | signed comparison |
| 224–226 | fixed-width conversion operations | conversion |
| 227 | `OP_ECMULSCALARVERIFY` | elliptic-curve verification |
| 228 | `OP_TWEAKVERIFY` | elliptic-curve verification |

## Surveyed encodings · `tab:elements-ref:encoding-values`

> Provisional survey facts; verify before encoding them in `target-elements`.

| Class | Surveyed prefix or form |
|---|---|
| explicit asset | `0x01` |
| explicit value | `0x01` |
| confidential value | `0x08`, `0x09` |
| confidential asset | `0x0a`, `0x0b` |
| explicit inspected value | 8-byte little-endian target amount |
| arithmetic operands | exact 8-byte signed little-endian |

## Surveyed sharp edges · `tab:elements-ref:sharp-edges`

> Provisional survey facts; verify before encoding them in `target-elements`.

| Surface | Surveyed behavior requiring exact verification |
|---|---|
| arithmetic overflow | operands may remain while a false success flag is pushed |
| division result | remainder, quotient, success in target stack order |
| indexed inspection | negative or out-of-range index aborts |
| stack plus altstack | surveyed combined limit 1000 |
| stack element | surveyed limit 520 bytes |
| initial push policy | surveyed 80-byte policy claim; exact scope unresolved |
| crypto budget | surveyed base `50 + serialized input witness bytes` |
| crypto operation cost | surveyed cost 50 for selected signature/EC operations |
| input nonce | may not remain available in the spend-time introspection path |
| unknown key lengths | forward-compatibility behavior must not be accepted accidentally |

## Execution domain · `sec:elements-ref:execution-domain`

Before backend use, verify:

- tapscript activation on the selected deployment instance;
- required leaf version;
- opcode availability in tapscript;
- failure outside the accepted execution domain;
- treatment of relevant opcode bytes under OP_SUCCESS rules;
- script-path and control-path behavior.

Development regtest support does not prove production activation or policy
equivalence.

## Introspection · `sec:elements-ref:introspection`

The backend expects typed access to selected input/output/transaction facts.

Required review dimensions include:

- index domain and out-of-range behavior;
- exact operand/result stack order;
- explicit versus confidential payload forms;
- representation prefix;
- field width;
- field-specific byte order;
- native witness-program versus non-native program result;
- null issuance/value/nonce forms;
- malformed or unknown encoding behavior.

Counts propose ABI ranges but do not authenticate object families.

Program, asset, and constructor checks still enforce family membership.

## Asset and value encodings · `sec:elements-ref:encodings`

Asset and value representation are separate axes.

The review must identify exact target classes for:

- explicit asset;
- confidential asset;
- explicit value;
- confidential value;
- null value;
- nonce forms.

The attestation contract's initial Elements profile requires explicit closed
protocol asset identity.

A confidential value commitment does not authorize a confidential protocol
asset.

Unknown prefixes fail closed.

Byte order is recorded per field rather than through one global convention.

## Arithmetic · `sec:elements-ref:arithmetic`

The surveyed target provides fixed-width signed arithmetic and comparison
operations suitable for:

- checked narrow amount arithmetic;
- count/index arithmetic;
- limb-based wide proofs;
- quotient/remainder relations.

Before use, the typed target must record for every selected operation:

- exact operand width;
- signed interpretation;
- operand order;
- result order;
- overflow/failure behavior;
- success flag;
- failure-path stack effect;
- malformed-width behavior;
- resource cost.

In particular, a failure that leaves operands on stack differs materially from
an aborting instruction.

No production wide-floor claim follows merely from 64-bit primitives. See
[wide arithmetic research](../research/wide-arithmetic.md).

## Hashing and constructors · `sec:elements-ref:constructors`

Potential constructor primitives include:

- SHA-256;
- streaming SHA-256;
- byte concatenation;
- target tapleaf/tapbranch/tweak hashing;
- output program introspection;
- elliptic-curve/tweak verification.

Before a constructor pattern is approved, it must establish:

- canonical metadata encoding;
- predecessor binding;
- successor reconstruction;
- static code continuity;
- internal-key policy;
- parity/point handling;
- metadata path unspendability;
- totality policy;
- resource feasibility.

Low-level tweak verification is not itself a complete constructor.

See [STATE constructor research](../research/state-constructor.md).

## Signatures · `sec:elements-ref:signatures`

The typed target must describe signature behavior by dimensions, including:

- output commitment;
- current/all input commitment;
- permitted input-set extension;
- issuance commitment;
- version/locktime commitment;
- script-path semantics;
- malformed and empty signature behavior;
- public-key encoding behavior;
- crypto budget.

The backend selects one profile compatible with semantic authorization.

Output mutation tests are required for every protected recipient or record
family.

## Timelocks · `sec:elements-ref:timelocks`

Cycle cadence uses relative timelock behavior.

Review:

- transaction-version prerequisites;
- sequence flags;
- block versus time mode;
- minimum-age comparison;
- disabled forms;
- exact boundary behavior;
- target activation;
- policy behavior.

Maturity uses committed cycle arithmetic, not target timelocks.

## Confidential transactions · `sec:elements-ref:ct`

The target review distinguishes:

- explicit asset identity;
- confidential asset commitment;
- explicit value;
- confidential value commitment;
- CT value conservation;
- rangeproof and surjection-proof behavior;
- output nonce;
- issuance/reissuance interaction;
- commitment equality;
- input data retained at spend time.

CT conservation can support lateral value preservation only when object and
closed-asset output closure are separately enforced.

Do not advertise authenticated public opening until a complete pattern and
permissionless lifecycle pass
[public declassification research](../research/public-declassification.md).

## Issuance and reissuance · `sec:elements-ref:issuance`

Review exact target behavior for:

- issuance fields;
- null issuance;
- asset entropy and asset-ID derivation;
- issuance amount;
- reissuance/inflation token;
- input issuance introspection;
- transaction commitment;
- explicit/confidential forms.

The target package describes substrate behavior.

Architecture and backend packages decide how protocol authorities map onto it.

## Resources · `sec:elements-ref:resources`

Maintain separate:

- consensus limits;
- policy/standardness limits;
- deployment-selected stricter limits.

Review at least:

- transaction weight;
- witness bytes;
- script bytes;
- initial stack item count;
- stack plus altstack;
- element size;
- crypto budget;
- target operation cost where defined;
- mempool policy;
- package relay.

Exact complete transactions determine deployment batch bounds. Isolated script
size is insufficient.

## Known gaps · `tab:elements-ref:gaps`

| Gap | Required resolution |
|---|---|
| production activation | deployment-specific evidence |
| regtest/production equivalence | typed and tested claim |
| initial witness-push policy scope | exact source/target review |
| crypto-budget treatment of custom operations | target-native tests |
| input nonce/opening availability | public-declassification research |
| metadata-dependent constructor | STATE constructor research |
| wide floor arithmetic | wide-arithmetic research |
| settlement feasibility | batch-size-2 settlement research |
| package relay | deployment policy report |
| destruction-output exclusion | target/indexer report |
| Rust library/node compatibility | canonical transaction cross-check |

A gap cannot be promoted into a typed capability merely because the backend
would benefit from it.

## Typed handoff · `rule:elements-ref:typed-handoff`

`target-elements` converts accepted reviewed facts into deterministic typed
values.

For every backend-used primitive, the target contract should expose:

- capability identity;
- execution domain;
- encoding;
- stack contract;
- failure modes;
- resource interface;
- evidence requirement;
- review provenance.

The backend consumes those typed values, not this document.

## Review provenance record · `tab:elements-ref:review-record`

This section records the source review that produced the typed contract in
`tripod-target-elements`. Everything in it is review provenance: it
supports a human reader checking the transcription, and it is deliberately
absent from the typed contract and from that contract's stable projection. A
node at a different revision implementing the same reviewed semantics satisfies
the same typed contract.

No package parses this file.

| Field | Value |
|---|---|
| upstream repository | Elements (Liquid) node implementation |
| revision consulted | `6f43e3ffe7308589f3cbaaec9115ce7456b1bf99` |
| upstream licence | The MIT License (MIT), compatible with this workspace |
| review date | 2026-08-13 |
| typed by | `packages/target-elements/src/opcode.rs` |

### Source locations consulted · `tab:elements-ref:review-sources`

| Location | Facts taken from it |
|---|---|
| `src/script/script.h` | opcode byte declarations; script-number width and minimality rules; script element, operation, stack, and script size bounds; the per-check validation weight constant and offset |
| `src/script/script.cpp` | the opcode-success classification, which excludes every reviewed extension byte; witness-program recognition |
| `src/script/interpreter.h` | the tapscript leaf version and leaf mask; control-block sizes; the script verification flags |
| `src/script/interpreter.cpp` | the evaluation case blocks for every reviewed primitive; the domain gate; the introspection push helpers; the signature and curve check helpers; the validation weight accounting; the relative-timelock check |
| `src/primitives/confidential.h` | asset, value, and nonce prefix bytes and serialized widths; the big-endian storage of an explicit amount |
| `src/primitives/transaction.h` | the outpoint issuance and peg-in flag bits; the sequence disable flag, type flag, mask, and granularity |
| `src/consensus/consensus.h`, `src/policy/policy.h` | block weight and witness scale factor; the standard transaction weight bound |
| `src/crypto/sha256.cpp` | the streaming hash state serialization and its maximum message length |
| `src/serialize.h` | the size bound the current-input-index primitive checks against |

### Upstream tests consulted · `tab:elements-ref:review-tests`

| Test | Use |
|---|---|
| `test/functional/feature_tapscript_opcodes.py` | the behavioural reference for the introspection, arithmetic, and conversion primitives; confirms the outpoint flag bytes, the split payload and prefix pushes, the little-endian explicit amount, and the rejection of a negative widening input |
| `test/functional/test_framework/script.py` | the opcode constants used by the functional tests |

No upstream C++ unit test in `src/test/` exercises the reviewed extension
primitives. The functional test above is the only upstream behavioural source,
which is itself a reason the typed contract states evidence requirements rather
than claiming verification.

### Claims accepted into the typed contract · `tab:elements-ref:review-accepted`

- the tapscript leaf version is target-specific and is not the corresponding
  upstream Bitcoin value;
- every reviewed extension primitive is gated to the tapscript domain by a
  negative test against the pre-tapscript signature versions, with no separate
  feature flag and no discouragement flag;
- the reviewed extension bytes are deliberately excluded from opcode-success
  treatment;
- the reviewed domain enforces no per-script operation budget and no script
  size bound, so both are stated as zero cost rather than omitted;
- asset and value introspection push the payload and the prefix as two separate
  stack items, payload first; nonce introspection pushes one item with its
  prefix inline;
- an explicit amount is stored big-endian in the transaction field and reaches
  the stack little-endian;
- fixed-width arithmetic that overflows, and division by zero, leave both
  operands in place and push a false above them rather than aborting;
- fixed-width comparison always consumes both operands and pushes exactly one
  item, so its false is an answer rather than a failure flag;
- the narrowing conversion aborts when its result exceeds the script-number
  range, while the widening conversion reads its operand unsigned;
- signature verification distinguishes an empty signature, which consumes the
  operands and pushes a false, from an invalid one, which aborts;
- the curve-check primitives consume three operands and push nothing;
- the relative-timelock primitive inspects its operand in place, pushing and
  popping nothing, and requires a minimum transaction version;
- the per-check validation budget is charged only by the signature and curve
  primitives.

### Claims left unresolved · `tab:elements-ref:review-unresolved`

| Claim | Why it is unresolved |
|---|---|
| transaction version and locktime introspection context | these two primitives carry no context-availability guard, and the behaviour of a context-less checker was not established from source |
| null asset introspection | the push helper asserts rather than raising a script error, so the source defines no consensus behaviour for the case |
| message length constraints on stack-message signature verification | the interpreter imposes none; whether the verification routine does was not established |
| whole-transaction confidential value conservation | reviewed as an external consensus claim, not as a script primitive; no typed primitive establishes it |
| commitment equality | no reviewed script primitive establishes it |
| authenticated opening | low-level curve and hash primitives exist, which does not constitute an opening proof |

### Contract revision 2 review · `tab:elements-ref:review-v2`

The Guide-10 prototypes needed primitives the first review had not reached, so
the contract moved to revision 2. The census below is review provenance for
that revision; the typed registry remains the authority.

| Field | Value |
|---|---|
| revision consulted | merged tip `0b3bffd`, upstream base `b7fc5d0` |
| workspace | ADR-018, topics `fix/tapscript-opcodes` and notes |
| census after revision | 55 opcodes |
| newly reviewed | 17 compound-proof primitives |
| review date | 2026-08-16 |

The newly reviewed primitives are the stack-rearrangement, byte-string
equality, boolean-verification, concatenation, width, slicing, and
bitwise-logic groups. They are ordinary script primitives the first review had
no need for; admitting them is a scope change, not a semantic discovery.

An eighth compound-proof capability, canonical byte ordering, is typed
`Unsupported` and names no primitive. The reviewed target has no
byte-lexicographic comparison: its ordering primitives read fixed-width signed
integers and its script-number ordering reads a number, and neither orders a
thirty-two byte digest. A construction needing canonical ordering must build it
from the primitives that do exist and prove the construction, which is a
different claim from having the capability.

### Corrections and additions from that review · `tab:elements-ref:review-v2-repairs`

| Correction | Detail |
|---|---|
| tweak-verify operand | the tweak operand is width-only. The interpreter checks its length is thirty-two and passes the bytes to the pay-to-contract check without interpreting them, so a stricter typed operand claimed a check the target does not perform (`src/script/interpreter.cpp:2206-2220`) |
| reach property | every reviewed prototype schedule reads no lower than the third stack item and uses no pick, roll, or altstack. This is a property of the emitted programs, machine-checked, not a target rule |

Five failure classes were added to the executor vocabulary, each naming a
refusal no existing class stated. Three are refusals on the script's own shape
rather than on a value it computed: script size, operation count, and combined
stack-item count. Two are refusals of the spend's authentication data before
any script runs: a sighash type declined from the signature's trailing byte
before verification is attempted, and a control block whose width is not one
the format defines, which the target refuses before reaching a version byte to
judge. A rejection carrying none of these would previously have been read as a
malformed response rather than as the verdict it is.

### Transient stack occupancy of the verifying forms · `tab:elements-ref:verifying-transient`

The Guide-11 stack-resource weld found the two verifying signature primitives
declaring their branching counterpart's maximum stack growth rather than the
depth they settle at. Source review confirmed the declared rows and located the
transient the success algebra does not represent.

| Confirmation | Detail |
|---|---|
| verifying forms push before they verify | the interpreter shares one case block between a branching primitive and its verifying counterpart. It pops the operands, pushes the truth value, and only then, if the opcode is the verifying one, pops that value again on success and aborts on failure. The verifying form therefore transiently occupies its branching counterpart's depth before the implicit verification consumes the result (`src/script/interpreter.cpp:1476-1499` for the transaction-signature pair, `src/script/interpreter.cpp:1689-1734` for the stack-message pair) |

The typed consequence is stated once, over the verifying primitives the
signature weld already names, as a transient term above the surviving depth.
Neither reviewed row moved.

### Target-native tests still required · `rule:elements-ref:review-required-tests`

The Guide-9 development conformance run resolved the required primitive
evidence rows; the executor provenance is recorded below. Sighash semantics
remain unresolved by design, commitment equality and authenticated opening
remain unsupported by the static contract, and confidential-value
conservation remains deferred to complete transaction evidence. No
production deployment evidence has been produced and production target
support is not claimed. A development report speaks only for its exact
executor, revision, and disposable chain.

### Native execution provenance · `tab:elements-ref:native-provenance`

| Fact | Value |
|---|---|
| Executor adapter | `scripts/elements-native-executor.py` via its launcher, protocol schema 1 |
| Implementation | Elements Core daemon v28.99.0-6f43e3ffe730 |
| Upstream revision executed | `6f43e3ffe7308589f3cbaaec9115ce7456b1bf99` (rebuilt from a clean tree; an earlier stale binary two commits behind was refused for evidence because it predated the EC opcode stack-size fix) |
| Chain | disposable elementsregtest, wallet-free genesis-free-coins funding |
| Development identifiers | synthetic nonzero network and genesis IDs recorded in the gate record |
| Fixture census | 398 cases, 18 of 18 required evidence rows, zero infrastructure errors |
| Determinism | report bytes identical across two fresh-node runs and the build lane |

The Guide-10 prototype matrices ran later, against a daemon at the merged tip
recorded in (`tab:elements-ref:review-v2`): 36 constructor rows and 39
wide-floor rows, each twice and byte-identically. Those runs are prototype
evidence and are recorded in the research files, not here. A report still
speaks only for its exact executor, revision, and disposable chain, and no
production deployment evidence exists for either revision.

### Confidential-value review · `tab:elements-ref:review-ct`

The Guide-11 Wave-5 review read the confidential-value machinery directly,
because every later declassification candidate depends on the exact relation
rather than on a conceptual one. The census below is review provenance; the
typed contract remains the authority.

| Field | Value |
|---|---|
| revision consulted | merged tip `78499c2064751b7662357ff93660c0dcc0f2f1b4`, upstream base `b7fc5d080a7e9ccc0ef48c3ba11db243e794bdb0` |
| workspace | ADR-018, branch `merged`, both trees read-only |
| upstream base subject | merge of upstream pull request 1580, dated 2026-08-14 |
| review date | 2026-08-19 |
| curve library | the zero-knowledge fork vendored in-tree at `src/secp256k1`, not a separate checkout |

The vendored curve library carries no independent revision: it is committed
inside the node tree and reports the node's own tip.

#### Encodings · `tab:elements-ref:ct-encodings`

| Fact | Mechanism | Location |
|---|---|---|
| one commitment template serves asset, value, and nonce | a template parameterised by explicit width and by two confidential prefix bytes; the null form is the empty buffer and serialises as a single zero byte | `src/primitives/confidential.h:16-97` |
| explicit value is nine bytes | prefix `0x01` then eight bytes of amount stored big-endian; confidential value is thirty-three bytes under prefix `0x08` or `0x09` | `src/primitives/confidential.h:130-154`, `src/primitives/confidential.cpp:14-19` |
| explicit asset is thirty-three bytes | prefix `0x01` then the thirty-two byte asset identifier; confidential asset is thirty-three bytes under prefix `0x0a` or `0x0b` | `src/primitives/confidential.h:100-127`, `src/primitives/confidential.cpp:6-12` |
| the nonce shares the shape | thirty-three bytes explicit, and confidential prefixes `0x02` and `0x03` | `src/primitives/confidential.h:160-164` |
| an unrecognised prefix is a deserialisation failure | the unserialise switch admits zero, one, and the two confidential prefixes, and throws on anything else | `src/primitives/confidential.h:42-59` |
| a value commitment is a point in a quadratic-residue encoding | the leading byte is `9` exclusive-or the squareness of the y coordinate, so `0x08` marks a square y and `0x09` a non-square one; the remaining thirty-two bytes are the normalised x coordinate | `src/secp256k1/src/modules/generator/main_impl.h:275-279` |
| parsing a commitment checks the prefix pair, the field limit, and curve membership | it then copies the input verbatim, so the supplied prefix selects which of the two y values the point takes | `src/secp256k1/src/modules/generator/main_impl.h:281-297`, `:266-273` |
| a generator uses the same convention shifted | the leading byte is `11` exclusive-or the squareness of y, so `0x0a` marks a square y and `0x0b` a non-square one | `src/secp256k1/src/modules/generator/main_impl.h:79-92`, `:59-77` |

The parity convention is quadratic residuosity, not the oddness of the y
coordinate that a compressed public key encodes. This is the single most
load-bearing encoding fact in the review.

#### Asset generator derivation · `tab:elements-ref:ct-generator`

| Fact | Mechanism | Location |
|---|---|---|
| the generator is a sum of two hashed curve points | two tagged hashes over the same thirty-two byte asset identifier, under the sixteen-byte prefixes for the first and second generation, each mapped to the curve and the two results added | `src/secp256k1/src/modules/generator/main_impl.h:204-248` |
| the curve map is a documented indifferentiable construction | the Fouque and Tibouchi map, with the sign of y taken from the oddness of the field element rather than from its Jacobi symbol | `src/secp256k1/src/modules/generator/main_impl.h:94-202` |
| the blinded form adds a scalar multiple of the conventional base point | used only to check that a reissuance input carries the blinded reissuance token | `src/secp256k1/src/modules/generator/main_impl.h:257-264`, `src/confidential_validation.cpp:244-259` |
| a field element out of range degrades the return value but still saves a generator | the two field conversions are folded into the return code, while the point is written regardless; both consensus call sites assert on that code | `src/secp256k1/src/modules/generator/main_impl.h:228-247`, `src/confidential_validation.cpp:115-116`, `:181-182` |

#### Commitment algebra · `tab:elements-ref:ct-algebra`

| Fact | Mechanism | Location |
|---|---|---|
| the relation is the conceptual one, with the stated sign | the commitment is the blinding scalar times the conventional base point plus the amount times the asset generator; both terms are added, neither negated | `src/secp256k1/src/modules/generator/pedersen_impl.h:42-49`, `src/secp256k1/src/modules/generator/main_impl.h:308-335` |
| the opening scalar is thirty-two bytes big-endian | read by the standard scalar conversion, which reports overflow against the group order | `src/secp256k1/src/modules/generator/main_impl.h:322-323` |
| a scalar at or above the group order is refused | the commit routine returns failure without computing | `src/secp256k1/src/modules/generator/main_impl.h:323-334` |
| the zero scalar is admitted | consensus commits every explicit amount under an all-zero blinding buffer | `src/confidential_validation.cpp:111-112`, `:160`, `:199` |
| the amount is an unsigned sixty-four bit quantity at this layer | it is multiplied into the generator by a small-scalar routine, with range enforced by the caller rather than here | `src/secp256k1/src/modules/generator/pedersen_impl.h:34-45` |
| an identity result is refused | a commitment that lands on the point at infinity returns failure, which is how a zero amount under a zero blinder is rejected | `src/secp256k1/src/modules/generator/main_impl.h:325-329` |

#### Transaction conservation and proofs · `tab:elements-ref:ct-conservation`

| Fact | Mechanism | Location |
|---|---|---|
| conservation is an exact tally, not an excess | the outputs are summed and negated, the inputs are added, and the result must be the point at infinity | `src/secp256k1/src/modules/generator/main_impl.h:371-396`, `src/confidential_validation.cpp:73-81` |
| explicit values join the tally as zero-blinded commitments | so one relation covers both modes, over the same asset generator the entry carries | `src/confidential_validation.cpp:194-200`, `:333-350` |
| the value set spans inputs, issuance pseudo-inputs, and every output | an issuance contributes the issued amount and, on a first issuance, the reissuance token, both on the input side | `src/confidential_validation.cpp:173-308`, `:310-361` |
| a zero-value explicit output is admitted only when unspendable, and then leaves the tally | it is skipped rather than committed, so it contributes nothing to the balance | `src/confidential_validation.cpp:337-345` |
| a range proof is required exactly when the value is confidential | an explicit value carrying a range proof is refused, and a confidential value without an output witness is refused | `src/confidential_validation.cpp:369-391` |
| the range proof is verified against the serialized generator, not the asset identifier | for an explicit asset the generator is derived and serialized into the commitment buffer before the check | `src/confidential_validation.cpp:380-384` |
| a range proof over a spendable output must exclude zero | a proven minimum of zero is refused unless the script is unspendable, which is what stops value being conjured through reissuance tokens | `src/script/sigcache.cpp:157-164` |
| the range proof binds the output script | the script is passed as the proof's extra commitment, so a proof does not transfer between outputs | `src/script/sigcache.cpp:153`, `src/confidential_validation.cpp:388` |
| a surjection proof is required exactly when the asset is confidential | an explicit asset carrying one is refused, and the target list is the input generators in order followed by each issuance pseudo-input | `src/confidential_validation.cpp:165-170`, `:393-420` |
| a coinbase is explicit in both fields and is checked against the fee map | it may carry no output witness at all | `src/confidential_validation.cpp:425-451` |
| closing the blinding balance is a construction obligation, not a consensus check | the constructing side solves for the final blinding factor so the tally closes over both the value and the asset blinders, and refuses the degenerate cases where that factor would be zero or would unblind its own output; consensus checks only the resulting tally | `src/blind.cpp:575-602` |

The distinction in the last row matters for what may be claimed. The target's
consensus statement is the exact tally and nothing more. That the blinders sum
correctly is a property the constructing side arranges, so a deployment
relying on it is relying on its own construction rather than on a rule the
target enforces.

#### Introspection of commitments · `tab:elements-ref:ct-introspection`

| Fact | Mechanism | Location |
|---|---|---|
| value introspection pushes the payload then the prefix | the prefix is the top item; the payload is eight bytes for an explicit value and thirty-two for a commitment | `src/script/interpreter.cpp:105-122` |
| an explicit amount is reversed on the way to the stack | stored big-endian in the transaction, pushed little-endian by a reverse iterator | `src/script/interpreter.cpp:113-115` |
| a confidential amount reaches the stack as the bare x coordinate | the prefix byte is separated, so the canonical thirty-three byte form is not pushed as one item | `src/script/interpreter.cpp:116-118` |
| a null value is presented as an explicit zero | prefix `0x01` over eight zero bytes, so a program cannot distinguish an absent value from a zero one at this primitive | `src/script/interpreter.cpp:108-111` |
| asset introspection splits the same way without reversal | the thirty-two byte payload keeps its stored order, and the prefix follows it | `src/script/interpreter.cpp:98-103` |
| spent-input and output introspection read the same helpers | the spent outputs come from the precomputed cache, and both families are refused outside tapscript and when the cache is not ready | `src/script/interpreter.cpp:1803-1884`, `:1901-1956` |
| the target exposes commitments and compares nothing | no reviewed primitive takes two commitments and judges them; equality would be byte equality over the exposed items, which is a composition rather than a primitive | `src/script/interpreter.cpp:1846-1854`, `:1929-1938` |

#### Output-committing signature profile · `tab:elements-ref:ct-sighash`

| Fact | Mechanism | Location |
|---|---|---|
| the reviewed digest carries no epoch byte | the upstream Bitcoin epoch prefix is deliberately absent, so the digest is not the Bitcoin one | `src/script/interpreter.cpp:2718-2722` |
| only six signature-hash bytes are accepted | the three output modes and their three anyone-can-pay counterparts | `src/script/interpreter.cpp:2725-2728` |
| the all-outputs mode commits every output and every output witness | two separate hashes, the second covering the range and surjection proofs | `src/script/interpreter.cpp:2747-2750`, `:2424-2431`, `:2449-2456` |
| an output hash covers the whole output | the full serialization, so asset, value commitment, nonce, and script are all committed | `src/script/interpreter.cpp:2449-2456` |
| the non-anyone-can-pay path commits the spent asset and amount commitments | so input commitments are bound as well, while output nonces are excluded by a recorded upstream decision | `src/script/interpreter.cpp:2733-2746` |
| the single-output mode commits only the positional output and its witness | insufficient on its own for a profile that must protect change and every other protected output | `src/script/interpreter.cpp:2780-2796` |

The profile that satisfies the Guide-11 normalization prerequisite is therefore
the default or all-outputs mode without anyone-can-pay: it is the only reviewed
mode committing every output, every output value commitment, every output
script, and every output witness at once.

#### Curve primitives available to a program · `tab:elements-ref:ct-curve-ops`

| Fact | Mechanism | Location |
|---|---|---|
| scalar multiplication is verified, not computed | three operands, a result and a base both thirty-three bytes and a thirty-two byte scalar, all consumed and nothing pushed | `src/script/interpreter.cpp:2169-2195` |
| its width check does not check the prefix | the compression predicate tests length alone, so prefix rejection happens inside the parse the check delegates to, which admits only the two compressed public-key prefixes | `src/pubkey.h:204-207`, `src/pubkey.cpp:267-277` |
| the tweak check is the pay-to-contract relation | a thirty-three byte tweaked key with a compressed prefix, a thirty-two byte tweak, and a thirty-two byte x-only base, verified as base plus tweak times the conventional base point | `src/script/interpreter.cpp:2198-2225`, `src/pubkey.cpp:239-244` |
| the base of the tweak check is x-only | it is lifted to its even-y representative, so the relation is narrower than a general point addition | `src/script/interpreter.cpp:2216-2219` |
| both curve primitives are tapscript-only and charge the validation budget | each is refused under the two pre-tapscript signature versions and each consumes a budget unit before verifying | `src/script/interpreter.cpp:2172`, `:2186`, `:2201`, `:2214` |
| byte concatenation is available | so a program can assemble a thirty-three byte operand from a prefix and a payload | `src/script/interpreter.cpp:1023-1041` |

#### Why an on-script public opening is not reachable · `tab:elements-ref:ct-opening-gap`

The review answered the openability question negatively, and names three
independent blockers rather than one.

| Blocker | Statement |
|---|---|
| the generator cannot be derived on-script | the derivation needs two curve maps and a point addition, and no reviewed primitive performs either; a program can read an asset identifier but cannot turn it into the generator the commitment uses (`src/secp256k1/src/modules/generator/main_impl.h:204-248`) |
| the encoding domains do not meet | commitments and generators are encoded by the squareness of y under prefixes `0x08`, `0x09`, `0x0a`, and `0x0b`, while both curve primitives accept only the compressed public-key prefixes, which encode the oddness of y; no reviewed primitive converts between the two conventions (`src/secp256k1/src/modules/generator/main_impl.h:88`, `:278`, `src/pubkey.cpp:271`) |
| a supplied parity is unbound | a witness may assemble a compressed operand from an exposed x coordinate, but nothing ties the parity byte it chooses to the squareness prefix of the same point. The two conventions select different y values in general, so the relation a program could verify holds for the point or its negation, and the amount it appears to open is therefore signed only by convention |

The second blocker is what the guide's warning against reasoning by analogy
with the tweak relation protects against. The tweak check does verify a
relation of the same shape as the commitment relation, base plus scalar times
the conventional base point. It is nonetheless not the commitment relation: its
base is x-only rather than a general point, and its encoding is the compressed
public-key one rather than the squareness one. The shapes agree and the
domains do not.

The consequence is recorded rather than softened. Under the reviewed revision
an authenticated public opening has no complete on-script form, and the
capability stays unsupported in the typed contract for that reason and not for
want of looking.

## Updating · `rule:elements-ref:update`

Update this reference when:

- the typed target contract is implemented;
- a target-used capability is added or removed;
- a known gap is resolved;
- test provenance changes materially;
- deployment instance or activation evidence changes.

An update states which typed target facts, backend patterns, reports, and
release identities are affected.

## Completion · `gate:elements-ref:completion`

This reference is complete for target implementation when:

- every release-used target claim has typed ownership;
- every backend-used opcode has exact stack and failure semantics;
- encoding and byte-order rules are explicit;
- consensus and policy limits are distinguished;
- target-native tests exist;
- known gaps are mapped to evidence or research;
- no source implementation revision is treated as protocol identity;
- no package parses this Markdown.
