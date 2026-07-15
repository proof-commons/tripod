# Elements Tapscript Capability Survey

> **Status:** REFERENCE — NOT A TARGET OR CONSENSUS-IMPLEMENTATION PIN
> **Target:** Liquid mainnet tapscript capability set
> **Scope:** Human-readable survey of Elements tapscript capabilities relevant
> to the planned attestation-contract backend
> **Upstream repository:** `https://github.com/ElementsProject/elements`
> (explanatory reference only)
> **Implementation revision:** intentionally not pinned as protocol identity —
> see ADR-011, "Target substrate compatibility"
> **Upstream license:** expected to be MIT; verify before retaining any copied
> fixture or source excerpt
> **Machine-consumed by the toolchain:** no
> **Machine-consumed target authority:** future
> [`tripod-target-elements`](../packages/target-elements.md) typed Rust
> declarations
> **Supersedes:** `plans/doc/tapscript_opcodes.md`

---

## 1. Purpose

This document summarizes Elements tapscript functionality relevant to the
planned attestation compiler and Elements backend.

It exists to help reviewers and implementers answer:

- which target primitive families appear relevant;
- where their upstream definitions and implementations are expected to live;
- which execution domains and activation conditions matter;
- which encoding and resource rules require exact source verification;
- which project relations may use each capability;
- which source, test, policy, and deployment gaps remain open.

This document does **not**:

- define attestation-contract semantics;
- define compiler proof plans;
- define backend opcode patterns;
- pin a deployable Elements target;
- prove production activation;
- prove target behavior;
- provide deployment evidence;
- select a sighash profile;
- select transaction layouts;
- establish authenticated commitment opening;
- establish wide-arithmetic feasibility;
- establish package relay behavior;
- become input to any first-party package.

The future `target-elements` package will encode the exact typed backend
compatibility contract as typed Rust values. Backend and release tooling
consume those typed values, never this Markdown.

---

## 2. Authority and evidence levels

A target capability passes through several distinct levels.

| Level | Meaning |
|---|---|
| **Surveyed** | This reference identifies a possibly relevant upstream primitive or behavior. |
| **Source-supported** | Exact behavior is verified against reviewed upstream source (review provenance recorded; ADR-011). |
| **Activated** | The behavior is verified as active in the selected execution/network context. |
| **Typed target capability** | `target-elements` encodes the exact source-supported behavior as a validated typed declaration. |
| **Approved proof pattern** | The backend implements a complete target pattern using the capability, with stack, failure, resource, and constructibility contracts. |
| **Deployment-evidenced** | The exact selected target/deployment report verifies the claim required by the final profile. |

This document records only a survey.

A source-supported opcode is not automatically:

- active on every network;
- available in every script version;
- sufficient to prove an attestation-contract relation;
- constructible by a permissionless actor;
- within target resource limits;
- deployment-evidenced.

---

## 3. Provenance expectations for target implementation

Per ADR-011 ("Target substrate compatibility"), no Elements
consensus-implementation source revision becomes protocol identity or a
release pin. Source links in this document are explanatory review aids;
verifying an interface claim against upstream source is a review-depth
choice, not a release requirement.

The Phase-3 target review must record:

```text
network flavor
network/genesis binding strategy
activation state
tapscript leaf version
opcode numbers and stack contracts the backend emits against
sighash and transaction conventions selected
Rust Elements library version, if used (pinned via Cargo.lock)
node binary/build identity used by target-native tests
    (ordinary test provenance for reproducibility and diagnosis,
    not part of the protocol or target identity)
upstream license, before retaining any copied fixture or excerpt
```

The source review should cover, at minimum, the exact revision's equivalents
of:

```text
src/script/script.h
    opcode assignments and names

src/script/script.cpp
    OP_SUCCESS classification/carve-out

src/script/interpreter.cpp
    execution semantics and failure behavior

src/kernel/chainparams.cpp
    network deployment and activation configuration

doc/taproot-sighash.mediawiki or exact successor
    Elements taproot sighash semantics

test/functional/feature_tapscript_opcodes.py
    upstream functional coverage
```

Paths may change upstream. Record the actual paths and symbols at the selected
revision rather than relying on the paths above indefinitely.

The future typed target package should preserve source provenance for every
capability it advertises.

---

## 4. Provisional implementation and activation summary

The previous repository survey reported that Elements assigns semantics to the
tapscript opcode range:

```text
OP_SUCCESS196 through OP_SUCCESS228
```

and updates tapscript behavior for:

```text
OP_CHECKSIGFROMSTACK
OP_CHECKSIGFROMSTACKVERIFY
```

The survey further reported that:

- the relevant opcode range is excluded from Elements' ordinary `OP_SUCCESSx`
  behavior;
- these opcodes are valid only in tapscript;
- executing them in legacy or segwit-v0 script fails;
- the expected tapscript leaf version is `0xc0`;
- the opcode family activated on Liquid with the Elements taproot deployment;
- Elements-style regtest chains make taproot available from chain setup;
- later Simplicity deployment uses a distinct execution mechanism and is
  outside this survey.

These statements are **provisional reference claims** until rechecked
during the target review against a supported node and network
configuration.

Do not copy them into target code as constants from this file.

---

## 5. Capability-group overview

The surveyed target surface is organized into these groups.

| Codes | Group | Planned project relevance |
|---:|---|---|
| 196–198 | Streaming SHA-256 | Structured constructor/hash preimages exceeding one stack element or where incremental hashing reduces stack pressure. |
| 199–204 | Input introspection | Authenticate outpoints, assets, values, programs, sequence, and issuance facts. |
| 205 | Current input index | Authenticate local family membership and coordinator position. |
| 206–209 | Output introspection | Enforce asset/value/program/nonce properties of canonical output families. |
| 210–214 | Transaction introspection | Enforce version, locktime, family counts, and resource-related constraints. |
| 215–223 | Signed fixed-width arithmetic | Checked narrow arithmetic, comparisons, decomposition, limb operations, and floor-proof components. |
| 224–226 | Numeric conversion | Convert script numbers and fixed-width transaction fields for target arithmetic. |
| 227–228 | Elliptic-curve verification | Candidate constructor/tweak and commitment-opening mechanics. |
| Existing opcode updates | BIP340-style signature from stack | Candidate target proof/signature constructions and message verification. |

The table identifies possible relevance, not an accepted backend design.

---

# 6. Streaming SHA-256

## 6.1 Surveyed opcodes

| Code | Surveyed name | Provisional behavior |
|---:|---|---|
| 196 | `OP_SHA256INITIALIZE` | Initialize a serialized SHA-256 context after consuming one byte string. |
| 197 | `OP_SHA256UPDATE` | Consume a byte string and serialized context, then produce the updated context. |
| 198 | `OP_SHA256FINALIZE` | Consume the final byte string and context, then produce the completed SHA-256 digest. |

The exact operand order and serialized context format must be verified from the
pinned interpreter source.

The prior survey described the context as including:

- 32-byte SHA-256 state;
- 8-byte little-endian bit counter;
- unprocessed remainder buffer.

It also reported fail-closed behavior for:

- malformed context;
- context serialization failure;
- total message length beyond the implementation's supported SHA-256 domain.

Those details must become typed target stack/failure contracts before backend
use.

## 6.2 Project use

Potential use includes:

- metadata-leaf hashes;
- constructor/static-root reconstruction;
- target tagged hashes;
- preimages exceeding the 520-byte element limit;
- hash constructions where repeated concatenation causes excessive stack
  pressure.

When a complete preimage fits safely in one element and the target retains
`OP_CAT`, ordinary concatenation plus `OP_SHA256` may be simpler.

Backend policy should select deterministically between:

```text
concatenation path
streaming path
```

under exact target limits and resource formulas.

## 6.3 Required evidence

Before release use:

- exact context format;
- exact stack order;
- malformed-context vectors;
- empty/small/large preimages;
- multi-update path;
- boundary message sizes;
- target-native execution;
- resource formula;
- equality with an independent SHA-256 reference.

---

# 7. Input introspection

## 7.1 Surveyed opcodes

| Code | Surveyed name | Provisional result |
|---:|---|---|
| 199 | `OP_INSPECTINPUTOUTPOINT` | Push input outpoint components and issuance/pegin flags. |
| 200 | `OP_INSPECTINPUTASSET` | Push inspected input asset data and representation prefix. |
| 201 | `OP_INSPECTINPUTVALUE` | Push inspected input value/commitment data and representation prefix. |
| 202 | `OP_INSPECTINPUTSCRIPTPUBKEY` | Push native witness-program data and version, or a script hash plus a non-native sentinel. |
| 203 | `OP_INSPECTINPUTSEQUENCE` | Push input sequence as a target fixed-width value. |
| 204 | `OP_INSPECTINPUTISSUANCE` | Push input issuance/reissuance fields or a null representation. |

Indexed introspection reportedly aborts when an index is:

- negative under target script-number interpretation;
- beyond the last input.

Exact target error behavior must be source-verified.

## 7.2 Outpoint result

The prior survey described the outpoint result as including:

```text
transaction ID
4-byte output index
1-byte flags
```

with flag bits indicating:

- issuance;
- pegin;
- both;
- neither.

Verify:

- stack order;
- transaction-ID byte order;
- output-index byte order;
- flag values;
- pegin behavior;
- issuance behavior.

## 7.3 Asset result

The prior survey described asset inspection as a tuple containing:

```text
32-byte asset payload
1-byte representation prefix
```

The payload denotes either:

- explicit asset ID;
- confidential asset commitment body.

The target package must model the two axes explicitly:

```text
asset representation class
asset payload bytes
```

the attestation contract's initial Elements backend requires explicit closed protocol
asset identity.

## 7.4 Value result

The prior survey described value inspection as:

```text
8-byte little-endian value for explicit representation
or
32-byte commitment body for confidential representation
plus
1-byte representation prefix
```

Verify exact behavior for:

- explicit values;
- confidential values;
- null values;
- malformed/unknown prefixes;
- byte order;
- stack order.

## 7.5 Input scriptPubKey/program result

The prior survey described:

### Native witness program

```text
witness program
witness version
```

### Non-native program

```text
SHA-256(scriptPubKey)
script-number sentinel -1
```

This distinction is central to:

- canonical object recognition;
- current input constructor authentication;
- STATE predecessor reconstruction;
- target execution-domain checks.

The typed target must model it precisely.

## 7.6 Input sequence

Input sequence supports:

- cadence relative age;
- transaction-version/sequence validation;
- ABI constraints.

Verify whether the inspected value is:

- unsigned 4-byte little-endian;
- target fixed-width;
- script number after conversion.

## 7.7 Input issuance

The surveyed result includes fields related to:

- inflation/reissuance tokens;
- issued amount;
- asset entropy;
- asset blinding nonce.

Exact push order and null encoding must be verified.

attestation uses issuance/reissuance for closed protocol assets, so this
capability requires dedicated source and target-native evidence.

## 7.8 Missing input nonce

The prior survey reported that an input's original output nonce is not
consistently retained/introspectable from the UTXO database and target sighash.

This is important for public-opening and confidential-value research.

Do not design a production proof requiring input nonce introspection unless the
exact pinned target demonstrates the field is available and authenticated.

---

# 8. Current input index

## 8.1 Surveyed opcode

| Code | Surveyed name | Provisional behavior |
|---:|---|---|
| 205 | `OP_PUSHCURRENTINPUTINDEX` | Push the index of the currently executing input. |

## 8.2 Project use

Potential use includes:

- local object-family membership;
- coordinator selection;
- canonical range/rank verification;
- local input introspection without witness-selected indexes;
- mixed-program exclusion;
- local relation-carrier binding.

The backend must still authenticate the expected family range and operation
layout. Knowing the current input index does not prove what family occupies
that index.

## 8.3 Required vectors

- first input;
- middle input;
- final input;
- local program under wrong ABI range;
- wrong coordinator rank;
- family count mutation;
- mixed operation program at a valid index.

---

# 9. Output introspection

## 9.1 Surveyed opcodes

| Code | Surveyed name | Provisional result |
|---:|---|---|
| 206 | `OP_INSPECTOUTPUTASSET` | Push output asset payload and representation prefix. |
| 207 | `OP_INSPECTOUTPUTVALUE` | Push output value/commitment payload and representation prefix. |
| 208 | `OP_INSPECTOUTPUTNONCE` | Push output nonce or empty vector for null nonce. |
| 209 | `OP_INSPECTOUTPUTSCRIPTPUBKEY` | Push native witness-program data and version, or hash plus non-native sentinel. |

Indexed output introspection reportedly aborts for negative or out-of-bounds
indexes.

## 9.2 Project use

Output introspection is load-bearing for:

- exact closed-asset output closure;
- formula-bound payouts;
- reserve successor;
- receipt output class/owner/value;
- STATE successor;
- ASH aggregate/residual;
- control/vault successors;
- sponsor change;
- data-output/program classification;
- constructor continuity.

## 9.3 Output nonce

Output nonce is relevant to confidential transaction construction.

The target package must distinguish:

- null nonce;
- public-key nonce form;
- malformed/unknown forms.

The presence of output nonce inspection does not imply the corresponding input
nonce is available in a later spend.

## 9.4 Output program inspection

The x-only native witness program returned for taproot outputs may need to be
converted to or checked against:

- compressed point form;
- linked constructor recipe;
- target tweak relation.

This is part of
[`../research/state-object-constructor.md`](../research/state-object-constructor.md).

---

# 10. Transaction introspection

## 10.1 Surveyed opcodes

| Code | Surveyed name | Provisional result |
|---:|---|---|
| 210 | `OP_INSPECTVERSION` | Push transaction version. |
| 211 | `OP_INSPECTLOCKTIME` | Push transaction locktime. |
| 212 | `OP_INSPECTNUMINPUTS` | Push input count. |
| 213 | `OP_INSPECTNUMOUTPUTS` | Push output count. |
| 214 | `OP_TXWEIGHT` | Push transaction weight. |

## 10.2 Project use

Potential use includes:

- canonical transaction ABI;
- exact family-range authentication;
- cadence transaction prerequisites;
- locktime constraints;
- output-family completeness;
- resource-limit enforcement;
- sponsor suffix boundary.

Counts propose ranges; object-constructor and target-program checks authenticate
those ranges.

## 10.3 Encoding

Verify for each result:

- byte width;
- signed/unsigned interpretation;
- script-number versus fixed-width representation;
- conversion required before arithmetic;
- target failure behavior.

## 10.4 Transaction weight

Determine whether `OP_TXWEIGHT` returns:

- exact consensus transaction weight;
- one fixed-width target value;
- a value including final witness under the executing context;
- a policy-relevant or consensus-only metric.

The backend should not rely on in-script weight checks until exact semantics and
resource usefulness are understood.

Complete resource calibration remains off-chain even when weight is
introspectable.

---

# 11. Signed fixed-width arithmetic

## 11.1 Surveyed opcodes

| Code | Surveyed name | Operation |
|---:|---|---|
| 215 | `OP_ADD64` | Signed 64-bit addition with success indication. |
| 216 | `OP_SUB64` | Signed 64-bit subtraction with success indication. |
| 217 | `OP_MUL64` | Signed 64-bit multiplication with success indication. |
| 218 | `OP_DIV64` | Signed division producing quotient/remainder with success indication. |
| 219 | `OP_NEG64` | Signed negation with success indication. |
| 220 | `OP_LESSTHAN64` | Signed comparison. |
| 221 | `OP_LESSTHANOREQUAL64` | Signed comparison. |
| 222 | `OP_GREATERTHAN64` | Signed comparison. |
| 223 | `OP_GREATERTHANOREQUAL64` | Signed comparison. |

## 11.2 Operand width

The prior survey reported that arithmetic and comparison opcodes require
operands of exactly:

```text
8 bytes
```

and abort on any other width.

Verify this for every opcode, including comparisons.

## 11.3 Overflow behavior

The prior survey reported this behavior for arithmetic operations that can
overflow:

### Success

```text
operands consumed
result pushed
success flag true pushed
```

### Failure

```text
operands remain
success flag false pushed
```

This nonuniform stack behavior is critical.

The typed target must encode exact success and failure stack contracts. The
backend must verify every success flag and must not schedule later instructions
under an incorrect failure-path stack assumption.

## 11.4 Division

The prior survey described division as producing:

```text
remainder
quotient
success flag
```

under a particular push/stack ordering.

The exact resulting stack order must be confirmed from interpreter source and
target-native tests before wide arithmetic work.

Other reported rules include:

- division by zero fails;
- signed minimum divided by `-1` fails;
- remainder is nonnegative and less than the absolute divisor.

Verify exact signed quotient and remainder semantics.

## 11.5 Comparisons

Comparisons reportedly have no arithmetic overflow flag but still require exact
8-byte operands.

They push a script boolean.

The initial protocol arithmetic uses nonnegative values below `2^51`, so signed
comparison can be used only after those domain constraints are established.

## 11.6 Project use

### Narrow arithmetic

Expected uses:

- bounded additions/subtractions;
- ratio arithmetic where products remain below signed 64-bit maximum;
- count/index arithmetic;
- metadata/state comparisons.

### Wide arithmetic

The same opcodes may implement multi-limb products and comparisons for values
whose full product exceeds 64 bits.

See:

[`../research/wide-arithmetic.md`](../research/wide-arithmetic.md).

## 11.7 Required evidence

- exact-width failures;
- signed boundaries;
- every overflow case;
- success-flag checking;
- division result ordering;
- divide-by-zero;
- signed-minimum edge;
- comparison truth table;
- limb-bound vectors;
- target-native execution;
- stack-effect validation.

---

# 12. Numeric conversions

## 12.1 Surveyed opcodes

| Code | Surveyed name | Provisional behavior |
|---:|---|---|
| 224 | `OP_SCRIPTNUMTOLE64` | Convert a minimally encoded script number to signed 8-byte little-endian form. |
| 225 | `OP_LE64TOSCRIPTNUM` | Convert signed 8-byte little-endian to script number, subject to script-number range. |
| 226 | `OP_LE32TOLE64` | Convert unsigned 4-byte little-endian to signed 8-byte little-endian. |

## 12.2 Project use

Conversions may be needed for:

- transaction counts;
- input index;
- sequence;
- version;
- locktime;
- transaction weight;
- fixed-width amount arithmetic;
- limb decomposition constants.

## 12.3 Failure behavior

Verify:

- exact operand width;
- minimal script-number requirement;
- target script-number range;
- negative handling;
- conversion overflow/failure class;
- result encoding.

Do not assume every target count fits a legacy 4-byte script-number conversion
without checking the selected bound.

---

# 13. Elliptic-curve verification

## 13.1 Surveyed opcodes

| Code | Surveyed name | Provisional relation |
|---:|---|---|
| 227 | `OP_ECMULSCALARVERIFY` | Verify `Q = kP` for typed scalar and compressed points. |
| 228 | `OP_TWEAKVERIFY` | Verify `Q = P + kG` for x-only internal key, scalar, and compressed output point. |

The exact operand pop order, scalar interpretation, point validation, and
failure behavior must be confirmed.

## 13.2 Scalar constraints

The prior survey reported:

- scalar is exactly 32 bytes;
- scalar is big-endian unsigned;
- scalar must be within secp256k1 scalar range;
- malformed scalar or point aborts.

This is relevant to constructor hash-to-scalar totality.

## 13.3 Point forms

The survey described:

- `OP_ECMULSCALARVERIFY` using compressed points;
- `OP_TWEAKVERIFY` using:
  - x-only internal key;
  - scalar;
  - compressed output point.

Output program introspection may return only an x-only program. The backend may
need a validated parity witness plus `OP_CAT` to form the compressed point.

## 13.4 Project use

Potential use includes:

- metadata-dependent constructor verification;
- target tweak verification;
- authenticated public value opening;
- commitment relations.

A low-level EC opcode does not itself establish a complete high-level proof
pattern.

The target package should advertise:

```text
source primitive available
```

separately from:

```text
approved object constructor pattern available
approved value-opening pattern available
```

## 13.5 Crypto budget

The previous survey reported that each custom EC verification opcode consumes
a target crypto/sigops budget unit equivalent to 50 budget points.

This must be verified from source and target-native tests.

The existing upstream functional test may not fully cover budget accounting.

## 13.6 Required vectors

- valid relation;
- invalid relation;
- malformed point;
- wrong point;
- scalar zero;
- scalar one;
- scalar near order;
- scalar equal/above order;
- wrong scalar width;
- wrong parity/compressed point;
- insufficient crypto budget;
- stack underflow.

---

# 14. Signature-from-stack behavior

## 14.1 Surveyed behavior

The previous survey reported BIP340-style tapscript behavior for:

```text
OP_CHECKSIGFROMSTACK
OP_CHECKSIGFROMSTACKVERIFY
```

with operands conceptually including:

```text
x-only public key
message
Schnorr signature
```

The exact stack order must be source-verified.

## 14.2 Empty signature

The previous survey reported:

- empty signature produces false for non-VERIFY form;
- empty signature does not consume crypto budget.

Verify exact behavior.

## 14.3 Invalid nonempty signature

The prior survey reported fail-closed abort behavior for invalid nonempty
signatures.

Verify exact target semantics.

## 14.4 Unknown public-key lengths

The survey reported:

- empty public key aborts;
- nonempty non-32-byte public keys may use forward-compatible behavior.

This surface requires careful target modeling.

The backend must not accept an upgradeable/unknown public-key form for a
current protocol signature unless the selected target proof pattern explicitly
permits and tests it.

## 14.5 Project use

Potential uses include:

- proving a structured message;
- constructor or commitment proof helpers;
- target-specific authorization mechanisms.

Ordinary operation authorization may instead use tapscript signature opcodes and
a selected transaction sighash.

The exact sighash profile remains a separate target/backend/deployment choice.

---

# 15. Existing byte and hash operations

The prior survey reported that Elements retains several historically disabled
Bitcoin opcodes as ordinary operations rather than treating them as
`OP_SUCCESSx`, including:

- `OP_CAT`;
- bitwise operations such as `OP_AND`, `OP_OR`, `OP_XOR`, and `OP_INVERT`.

Before the backend emits against these opcodes, the target review must
verify (against a supported node and, where useful, upstream source):

- availability in tapscript;
- operand limits;
- byte behavior;
- execution-domain restrictions;
- resource cost;
- policy status.

Potential project use includes:

- structured hash preimages;
- x-only/compressed point construction;
- fixed-width masks;
- metadata encoding checks;
- limb operations where appropriate.

Do not make a production pattern depend on these operations merely because
this reference says they exist.

---

# 16. Provisional encoding summary

This section summarizes the previous survey and must be replaced or confirmed
by the typed target definition.

## 16.1 Explicit asset/value prefix

The survey reported:

```text
0x01
```

for explicit asset and explicit value encoding classes.

## 16.2 Confidential value prefixes

The survey reported:

```text
0x08
0x09
```

for value commitment forms.

## 16.3 Confidential asset prefixes

The survey reported:

```text
0x0a
0x0b
```

for asset commitment forms.

## 16.4 Explicit value byte order

The survey reported that value introspection pushes explicit values as:

```text
8-byte little-endian signed-compatible amount
```

rather than the big-endian representation used in serialized Elements
commitment fields.

Verify exact semantics and domain.

## 16.5 Asset and transaction byte order

The survey reported that asset IDs, transaction IDs, issuance entropy, and
blinding nonces use transaction-internal byte order, which may be reversed
relative to RPC display hex.

Every typed target field needs its own byte-order rule.

## 16.6 Null issuance values

The survey reported absent issuance values represented by:

```text
explicit zero value bytes
+
explicit prefix
```

Verify exact stack behavior.

## 16.7 Unknown prefixes

The initial backend must fail closed on unknown/future prefix classes unless an
explicit target policy says otherwise.

Do not treat an unknown asset prefix as harmless open data.

---

# 17. Provisional tapscript resource model

All resource facts in this section require exact source verification.

## 17.1 Script-size and opcode-count behavior

The prior survey reported BIP342-style tapscript behavior in which legacy:

```text
10,000-byte script-size limit
201 non-push-opcode limit
```

do not apply in the same way to tapscript, leaving block/transaction weight as
the principal aggregate size bound.

Verify exact Elements behavior and any additional Elements-specific limits.

## 17.2 Stack element count

The prior survey reported:

```text
maximum stack + altstack elements = 1000
```

after every executed opcode and for the initial stack.

Verify exact counting behavior.

## 17.3 Stack element size

The prior survey reported:

```text
maximum stack element size = 520 bytes
```

during execution.

This motivates streaming hashes for larger preimages.

Verify exceptions, script/control handling, and exact policy.

## 17.4 Initial push standardness

The prior survey reported an additional policy limit:

```text
80 bytes per initial push
```

The exact scope is currently a known review gap.

Resolve:

- whether it applies to all initial witness stack items;
- whether tapleaf script is excluded;
- whether control data is excluded;
- whether annex or other fields are treated separately;
- policy versus consensus status;
- exact target failure behavior.

This uncertainty must not be promoted into `target-elements` as a fact before
source verification.

## 17.5 Crypto budget

The prior survey described a per-input tapscript crypto budget:

```text
initial budget =
    50
    +
    total serialized input witness size in bytes
```

with each nonempty signature or selected custom crypto operation consuming:

```text
50
```

budget units.

Verify:

- whether the CompactSize prefix is included;
- which witness components are counted;
- which opcodes consume budget;
- empty-signature behavior;
- failure threshold;
- custom EC opcode accounting;
- interaction with target policy.

## 17.6 Block and transaction limits

The target package must type, with review provenance:

- block weight;
- transaction weight/policy limits;
- witness rules;
- standardness constraints;
- package policy.

Do not copy mutable numeric policy values into this reference without exact
revision provenance.

## 17.7 Resource evidence gap

The previous survey noted that upstream functional tests may not explicitly
exercise crypto-budget accounting for every custom EC opcode.

First-party target evidence should cover the exact selected claim.

---

# 18. Sighash and introspection safety

## 18.1 Introspection does not replace authorization

Transaction introspection can verify transaction fields.

Owner/operator/sponsor signatures must still use a target sighash profile
matching the realization's authorization requirements.

## 18.2 Output commitment

The initial backend requires signatures to commit the complete required
economic output set.

This is particularly important for:

- multi-owner transfer;
- burn records;
- formula-bound payout;
- sponsor change;
- normalization/declassification outputs.

## 18.3 Input-set extension

Any ANYONECANPAY-like or input-extension policy is a separate dimension.

It must not weaken output commitment.

The exact selected profile remains a backend/deployment decision and requires
target-native mutation tests.

## 18.4 Witness mutation and revalidation

Changing witness data can alter transaction witness identity and trigger target
revalidation.

Do not infer from that fact that every introspected field is committed by the
selected signature.

The backend must model signature commitment and script introspection
separately.

---

# 19. Project capability mapping

This table maps project needs to surveyed target families. It does not approve
the proof patterns.

| Project relation or seam | Surveyed target capabilities |
|---|---|
| Canonical input/output asset closure | Input/output asset introspection; transaction counts; current input index |
| Explicit amount arithmetic | Input/output value introspection; fixed-width arithmetic and conversion |
| Sponsor isolation | Input/output asset/value/program introspection; owner signatures; family counts |
| Canonical transaction ABI | Current input index; input/output counts; program introspection |
| Cadence band | Input sequence; transaction version; target relative-timelock semantics; signatures |
| Issuance authority and amount | Input issuance introspection; asset/value inspection; closed output family checks |
| Metadata-dependent object constructor | Program introspection; SHA-256/`OP_CAT` or streaming hashes; tweak verification |
| Wide floor arithmetic | Signed 64-bit arithmetic/comparisons; conversions; stack operations |
| Confidential live transfer | Explicit asset checks; CT value conservation; output closure; signatures |
| Commitment-preserving relabel | Commitment inspection/equality strategy; explicit asset checks; constructor checks |
| Public value opening | Commitment inspection; EC/hash operations; exact opening pattern not yet approved |
| Burn-to-ASH | Receipt/ASH asset and value checks; output closure; public value synchronization; signatures |
| Clear | Public ASH values/openings; STATE constructor; arithmetic; output introspection |
| Settlement | Bounded introspection; wide floors; constructor checks; control/vault/receipt closure |
| CPFP/package behavior | Zero-value anchor construction; package-relay policy evidence outside script alone |

---

# 20. Known gaps and unresolved claims

The following items must be resolved before the corresponding target capability
is release-ready.

## 20.1 Implementation source revision

Intentionally not pinned: an Elements implementation revision is not a
protocol or release identity (ADR-011). Node versions used by
target-native integration tests are recorded as test provenance when
those tests exist.

## 20.2 Production activation

The prior survey reports production activation, but final release needs exact
network/source/deployment evidence.

## 20.3 Regtest-production equivalence

Regtest availability does not prove production activation or policy
equivalence.

## 20.4 Initial-push policy scope

Resolve source behavior for witness items, scripts, control data, and any
target-specific exceptions.

## 20.5 Custom crypto-budget coverage

Confirm exact accounting for opcodes 227–228 and signature-from-stack behavior.

## 20.6 Input program inspection coverage

The previous survey noted upstream functional coverage was stronger for some
native witness-program cases than non-native input cases.

First-party tests must cover the exact project use.

## 20.7 Input nonce availability

Determine exactly which confidential output data survives into spend-time
target introspection and transaction construction.

This affects public-declassification design.

## 20.8 Authenticated value opening

No complete attestation-contract value-opening proof pattern is approved merely
from low-level EC support.

See:

[`../research/public-declassification.md`](../research/public-declassification.md).

## 20.9 Metadata-dependent object constructor

No production constructor is approved merely from tweak verification support.

See:

[`../research/state-object-constructor.md`](../research/state-object-constructor.md).

## 20.10 Wide arithmetic

No production `floor_mul_div` pattern is approved merely from 64-bit arithmetic
support.

See:

[`../research/wide-arithmetic.md`](../research/wide-arithmetic.md).

## 20.11 Settlement feasibility

No general settlement bound/layout is approved merely from introspection and
arithmetic availability.

See:

[`../research/settlement-layout.md`](../research/settlement-layout.md).

## 20.12 Package relay

Package-relay behavior is target/deployment policy evidence, not an opcode
fact.

## 20.13 Unspendable-output exclusion

The exact target semantics and indexer/UTXO behavior for protocol destruction
outputs require separate deployment evidence.

## 20.14 Rust library compatibility

Any selected Rust Elements library must be checked against the supported
test node and canonical transaction bytes (library versions pinned via
`Cargo.lock`).

## 20.15 Standardness and deployment policy

Consensus support does not imply ordinary relay/mining acceptance.

---

# 21. Upstream test coverage

The previous survey identified an upstream functional test with a path
equivalent to:

```text
test/functional/feature_tapscript_opcodes.py
```

at the then-reviewed source.

The survey reported coverage for:

- streaming SHA-256;
- transaction introspection;
- fixed-width arithmetic;
- numeric conversions;
- custom EC operations;
- signature-from-stack behavior;
- malformed inputs and target failures;
- chain execution of the custom opcodes.

Before relying on that report:

1. verify the test exists at the selected revision;
2. record exact path and test name;
3. run it against the exact selected build where practical;
4. record required optional build features;
5. identify which attestation-contract claims it does not cover;
6. add first-party tests for those gaps.

Upstream tests are useful provenance and regression evidence.

They do not replace project-specific target patterns and bundle-level vectors.

---

# 22. Planned first-party target evidence

The future target/vector/release path should produce reports for at least the
following claims.

## 22.1 Opcode assignment and execution domain

- codes match source;
- opcodes are not treated as unconditional success;
- tapscript-only behavior;
- wrong script version fails.

## 22.2 Introspection

- correct tuple order;
- explicit/confidential forms;
- index bounds;
- byte order;
- scriptPubKey native/non-native behavior;
- issuance fields.

## 22.3 Arithmetic

- exact width;
- success/failure stack behavior;
- overflow;
- division/remainder;
- comparisons;
- conversions.

## 22.4 Crypto

- valid/invalid scalar and points;
- tweak relation;
- parity handling;
- crypto budget;
- malformed input.

## 22.5 Sighash

- selected profile commits required outputs;
- protected-output mutation invalidates signature;
- input-extension behavior matches policy;
- malformed/empty signature behavior.

## 22.6 Timelocks

- all cadence boundaries;
- transaction-version and sequence prerequisites;
- wrong mode fails.

## 22.7 Confidential transactions

- explicit/confidential value forms;
- CT conservation;
- wrong balance;
- range/surjection proof behavior;
- explicit closed asset policy;
- commitment equality where used.

## 22.8 Policy/resources

- stack limits;
- element size;
- initial push policy;
- transaction weight;
- crypto budget;
- mempool/standardness;
- package relay where applicable.

Each report must bind the exact target and tool identity.

---

# 23. Relationship to `target-elements`

The future target package is authoritative for first-party machine-consumed
target facts.

It should encode:

```text
target schema
review provenance (upstream repository/revision reviewed)
network flavor
activation rules
opcode registry
stack contracts
failure modes
encoding prefixes
byte orders
sighash capabilities
timelock capabilities
CT capabilities
issuance semantics
consensus limits
policy limits
evidence requirements
```

The target package must not merely copy this document into Rust.

It must:

- review the exact selected source;
- encode typed facts;
- validate internal closure;
- run source-conformance and target-native tests;
- produce deterministic identities.

When `target-elements` is implemented, this reference should link its exact
target identity and remain a human survey.

---

# 24. Relationship to the tapscript backend

The backend consumes typed target facts and implements complete proof patterns.

It must not hardcode target stack semantics independently.

For each pattern, the backend records:

- source semantic relation;
- selected proof alternative;
- required target capabilities;
- exact opcodes;
- exact stack contract;
- witness ABI;
- failure behavior;
- resource formula;
- target-native vectors.

This reference may help reviewers understand the implementation but does not
authorize a pattern.

---

# 25. Relationship to release

The release package validates:

- exact target identity;
- exact deployment instance;
- activation/network/genesis binding;
- required capability evidence;
- backend/bundle target binding;
- transaction ABI target binding;
- observer context target binding.

A target source commit with passing unit tests is necessary but insufficient
for deployment release.

The release also distinguishes:

- development regtest evidence;
- production activation evidence;
- production policy/package evidence;
- functionary/L-BTC settlement assumptions.

---

# 26. Updating this reference

Update this document when:

- a supported node/test target is selected;
- source paths or symbols move;
- activation facts change;
- target package facts are implemented;
- a relevant capability is added or rejected;
- a known gap is resolved;
- upstream tests change materially;
- a production target is selected.

An update must state:

- old revision;
- new revision;
- changed claims;
- target package impact;
- backend impact;
- report regeneration requirements;
- release identity impact.

Do not update source claims by pointing at a mutable upstream branch.

---

# 27. Replacement procedure for the old copied document

When this file is added:

1. retain attribution and license provenance required for any copied material;
2. remove:
   ```text
   plans/doc/tapscript_opcodes.md
   ```
3. remove `plans/doc/` if empty;
4. update all internal references to:
   ```text
   plans/reference/elements-tapscript.md
   ```
5. verify no broken `../src/...` links remain;
6. verify `plans/README.md` indexes this file;
7. verify this file is described as reference-only;
8. run Markdown link and plan-census checks;
9. ensure no compiler/package code reads either document.

Git history preserves the old source copy for historical review.

---

# 28. Definition of done

This reference is complete for Phase 0 when:

- [ ] the old copied opcode plan is removed;
- [ ] this document is indexed;
- [ ] upstream repository is identified;
- [ ] the intentionally-unpinned implementation-revision policy is stated
      (ADR-011);
- [ ] upstream license/provenance status is recorded;
- [ ] relevant source paths are listed without broken repository-relative
      links;
- [ ] capability groups are summarized without pretending to be typed target
      authority;
- [ ] source support, activation, proof-pattern availability, and deployment
      evidence are distinguished;
- [ ] encoding/resource facts are marked provisional until exact verification;
- [ ] known gaps are explicit;
- [ ] research notes and package plans are linked;
- [ ] machine-consumed authority is handed to `target-elements`;
- [ ] no plan or target package parses the document;
- [ ] all relative repository links resolve;
- [ ] `git diff --check` passes.

This reference is complete for Phase 3 only when:

- [ ] the supported node/test target and its recorded test provenance are
      identified;
- [ ] upstream license is verified for any retained copied material;
- [ ] production and development target profiles are distinguished;
- [ ] activation claims are source- and target-verified;
- [ ] every backend-used opcode has exact stack and failure semantics;
- [ ] all encoding/byte-order claims are verified;
- [ ] all resource-limit claims are verified;
- [ ] initial-push policy scope is resolved;
- [ ] crypto-budget behavior is verified;
- [ ] target package identity is linked;
- [ ] first-party evidence gaps are mapped to reports;
- [ ] no provisional wording remains for a release-used claim.

---

# 29. One-line reference contract

> This file is a human review map from reviewed Elements tapscript
> primitives to possible attestation-contract target uses: it records review
> provenance,
> capability groups, provisional encoding/resource facts, activation and test
> boundaries, and known gaps, while leaving all machine-consumed target
> authority, proof-pattern approval, bundle generation, and deployment evidence
> to typed packages and target reports.
