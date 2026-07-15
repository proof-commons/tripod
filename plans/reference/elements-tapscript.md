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

## Evidence levels · `tbl:elements-ref:evidence-levels`

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

## Capability groups · `tbl:elements-ref:groups`

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

## Known gaps · `tbl:elements-ref:gaps`

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
