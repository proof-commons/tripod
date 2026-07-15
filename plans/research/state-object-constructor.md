# Research Question: Metadata-Dependent Object and STATE Constructor Continuity

> **Status:** PROTOTYPE REQUIRED
> **Blocks:** production STATE constructor ABI; `announce-maturity`; every later
> STATE-spending target operation; metadata-dependent root succession; final
> object-constructor linking strategy; production constructor-continuity
> evidence
> **Affected packages:** `tapscript`, `linker`, `transaction`, `vectors`,
> `release`; findings may refine compiler fact-source and placement
> requirements and the typed Elements capability model
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-source.md),
> [D002](../decisions/002-realization-layer.md),
> [D003](../decisions/003-tapscript-first.md),
> [D004](../decisions/004-translation-validation.md),
> [D005](../decisions/005-value-representation.md),
> [D006](../decisions/006-transaction-abi.md)
> **Related normative constraints:** root identity and succession; every
> STATE-dependent operation consumes the canonical STATE root; fixed and
> authenticated predecessor/successor metadata; no alternate spend path;
> consensus-value authority; target transaction-shape obligations; model to
> script weld; metadata canonicality; permissionless construction; full
> root-history continuity in `docs/attestation/realization.md`
> **Expected decision output:** an accepted or rejected constructor-continuity
> design, likely recorded as a new implementation decision governing target
> object constructors, static code/program continuity, metadata commitment,
> internal-key policy, successor reconstruction, and production vectors
> **Machine-consumed by the toolchain:** no

---

## 1. Question

Can the initial Elements tapscript backend implement a deterministic,
metadata-dependent protocol object constructor—especially `STATE`—such that an
executing operation program can:

1. authenticate the consumed predecessor as an instance of the exact linked
   constructor family;
2. recover or verify the predecessor's canonical metadata;
3. compute the target-independent successor metadata required by the semantic
   operation;
4. verify that the successor output commits that metadata;
5. prove that predecessor and successor use the **same authenticated static
   code/program relation**;
6. prevent any alternate static code subtree, internal key, leaf version,
   metadata encoding, key-path spend, or spendable metadata path from
   bypassing the covenant;
7. remain publicly constructible where the operation is permissionless;
8. remain deterministic and canonically encodable;
9. fit the exact pinned Elements target's consensus, policy, stack, witness,
   crypto-budget, and transaction limits?

The primary candidate is a taproot-style constructor that combines:

- a fixed unspendable internal key;
- a linked static operation-code subtree;
- a dynamic, provably unspendable metadata leaf or metadata commitment;
- target tweak verification;
- one authenticated static-root witness reused for predecessor and successor
  verification.

The research must compare this candidate with credible alternatives before a
production constructor ABI is accepted.

---

## 2. Why the answer matters

### 2.1 STATE metadata changes on every state transition

The semantic STATE contains fields such as:

```text
Ω
Y_L
Y_T
Q
cycle
maturity
```

A valid STATE-spending operation must authenticate the predecessor fields and
commit the exact successor fields.

If metadata is committed into the output program, the successor program is not
generally byte-identical to the predecessor program because the semantic state
changed.

Simple scriptPubKey equality is therefore insufficient.

### 2.2 Static code identity must remain continuous

Correct successor metadata under the wrong code subtree is unsafe.

For example, a transaction could create an output committing the correct new
STATE fields but under a tree that:

- omits redemption;
- adds an operator escape;
- weakens a root weld;
- accepts confidential closed-asset identity;
- changes the cadence relation;
- introduces a spendable metadata path;
- uses different future operation scripts.

The successor must preserve both:

```text
new semantic metadata
```

and:

```text
the approved static constructor/program relation
```

### 2.3 Runtime metadata creates a recursive commitment problem

The static operation program must verify the constructor that contains the
operation program itself.

Embedding the final static subtree root directly into a leaf can create a
self-reference:

```text
leaf bytes contain root
root hashes leaf bytes
```

The implementation must avoid an undefined or manually patched fixed point.

A witnessed static code root authenticated against the predecessor is one
candidate solution.

### 2.4 Every STATE-dependent operation is blocked

The following target operations need a valid STATE constructor strategy:

```text
admit-deposits
cycle
redeem
receipt-relabel
clear
announce-maturity
```

The constructor is therefore a shared backend/linker/transaction seam, not a
local detail of one operation.

### 2.5 The constructor shapes the public ABI

The selected design affects:

- metadata schema;
- predecessor witness items;
- successor constructor instantiation;
- static-root symbol and relocation types;
- internal-key policy;
- parity/control data;
- taptree assembly;
- transaction layout;
- resource formulas;
- continuity vectors;
- bundle identity;
- release artifacts.

Freezing those APIs before the target construction works would make later
correction expensive.

### 2.6 A key-path escape would invalidate the whole covenant

A taproot constructor has both script-path and key-path semantics.

If any participant knows the internal key's discrete logarithm, or can derive
the tweaked output private key, the object may be spendable through the key
path without executing the covenant script.

The internal key must therefore be a deterministic, publicly auditable
nothing-up-my-sleeve point with no known private key, or another construction
must remove the key-path escape.

This requirement cannot be deferred to wallet convention.

---

## 3. Existing constraints

The prototype must satisfy all constraints below.

### 3.1 Semantic constraints

- `STATE` is identified by the closed amount-one `PID` root asset and canonical
  root history.
- Every operation reading STATE consumes the current canonical STATE input and
  creates its declared successor.
- STATE metadata has one canonical semantic interpretation.
- The successor fields are determined by the target-independent operation
  relation.
- An operation may not choose arbitrary successor fields.
- Root succession must be derivable from actual consumed and created objects.
- Once the pool is sealed, no later pool transition may revive it.
- The constructor must preserve full root-history replay semantics.

### 3.2 Architecture constraints

The constructor must support all STATE-mutating operations declared by the
architecture:

```text
admit-deposits
cycle
redeem
receipt-relabel
clear
announce-maturity
```

The linked static code set must be complete for the selected deployment
operation scope.

A pilot constructor with placeholder leaves must be labeled nonfinal.

### 3.3 Representation constraints

- STATE fields are public semantic facts.
- The constructor may use explicit or commitment-based program metadata, but
  predecessor and successor fields must be publicly and authentically
  available.
- No private owner opening may be required for permissionless STATE
  transitions.
- Closed root asset identity remains explicit under D005.
- Metadata must not impersonate consensus asset value.
- The constructor must not introduce confidential closed-asset identity.

### 3.4 Target constraints

The exact typed Elements target must define and verify every relied-upon
primitive, including where selected:

- input scriptPubKey/program introspection;
- output scriptPubKey/program introspection;
- current input index;
- byte concatenation;
- SHA-256 or streaming SHA-256;
- target tapleaf/tapbranch/taptweak hashing;
- x-only and compressed public-key forms;
- tweak verification;
- tapscript leaf version;
- stack element limits;
- initial stack policy;
- crypto budget;
- target control-path behavior;
- target transaction output program encoding.

No prototype result is accepted against an unspecified target.

### 3.5 Linker constraints

- the static code subtree is a typed linked symbol;
- every operation leaf is linked deterministically;
- the constructor recipe is deterministic;
- cyclic references use an explicit authenticated strategy;
- no manual post-link patch remains;
- predecessor and successor use one linked static code identity;
- constructor relation provenance survives linking.

### 3.6 Transaction constraints

- the transaction ABI provides canonical STATE input and successor positions;
- metadata is canonically encoded;
- predecessor metadata/static root/parity witness roles are explicit;
- successor metadata derives from the semantic relation;
- successor program is instantiated from the exact linked constructor recipe;
- all signing requests commit required outputs;
- no secret internal key or constructor key is needed.

### 3.7 Evidence constraints

The design requires:

- pattern-level vectors;
- linked-constructor vectors;
- complete transaction vectors;
- target-native execution;
- wrong-code-subtree vectors;
- metadata-leaf spend vectors;
- resource formulas and measurements;
- deterministic reproduction;
- bundle/ABI/report identity binding.

---

## 4. Definitions and terminology

### 4.1 Semantic metadata

Let:

```text
M
```

denote the canonical semantic metadata for one object instance.

For STATE:

```text
M_state = (Ω, Y_L, Y_T, Q, cycle, maturity)
```

The exact canonical byte encoding is:

```text
encode_metadata(object_kind, schema_version, M)
```

The encoding must be injective over valid typed metadata.

### 4.2 Static code subtree

Let:

```text
C_K
```

denote the linked static code/program subtree root for object kind `K`.

For STATE, the subtree contains the complete linked operation programs selected
for the deployment scope.

The root is independent of one concrete STATE value.

### 4.3 Metadata leaf

Let:

```text
L_K(M)
```

denote a dynamic metadata leaf or metadata commitment for object kind `K` and
metadata `M`.

The leading candidate uses a provably unspendable metadata leaf.

### 4.4 Constructor tree root

For the leading candidate:

```text
R_K(M) = TapBranch(C_K, L_K(M))
```

where branch ordering and hashing follow the exact pinned target.

The exact formula is target-specific and must be taken from typed target
semantics.

### 4.5 Internal key

Let:

```text
P_K
```

denote the constructor's x-only internal key.

For a covenant-only constructor, no participant may know a corresponding
private scalar.

### 4.6 Output key and constructor instance

Let:

```text
t_K(M) = TapTweak(P_K, R_K(M))
Q_K(M) = P_K + t_K(M)G
```

under the exact target tweak semantics.

The concrete object constructor instance is:

```text
Constructor_K(M) = target output program for Q_K(M)
```

These equations are illustrative of the leading candidate. The prototype must
use exact target definitions, including tagged hashing, branch ordering, scalar
domain, and parity handling.

### 4.7 Constructor continuity

A transition from `M_before` to `M_after` preserves constructor continuity when
it proves:

```text
input program  = Constructor_K(M_before)
output program = Constructor_K(M_after)
```

under the same authenticated:

```text
K
P_K
C_K
metadata schema
target constructor rule
```

### 4.8 Static-root witness

A static-root witness is a public 32-byte or target-defined value representing
`C_K`, supplied to the executing operation program.

It is not trusted merely because it is supplied.

The predecessor constructor must authenticate it, and the same authenticated
value must be reused for successor reconstruction.

### 4.9 NUMS internal key

A NUMS internal key is a deterministic public point intended to have no known
discrete logarithm.

The production key policy must define:

- exact point;
- derivation/provenance;
- validity;
- x-only encoding;
- absence of a retained secret;
- domain separation;
- target compatibility.

Calling a random-looking key “unspendable” is insufficient.

---

## 5. Required properties

A production constructor design must satisfy all of the following.

### 5.1 Canonical metadata

For every valid semantic metadata value:

- exactly one canonical encoding exists under the selected schema;
- alternate field order rejects;
- noncanonical integers reject;
- out-of-domain values reject;
- sentinel/tag collisions reject;
- unknown schema rejects.

If several concrete constructor representations are intentionally permitted,
the representation relation and canonical first-party builder policy must state
that explicitly.

### 5.2 Predecessor authentication

The executing operation program must verify that:

- the current input is the canonical target program class;
- the current input program equals the constructor derived from
  `M_before`;
- the supplied static root is the one committed by the predecessor;
- the supplied internal-key/constructor policy matches the linked bundle;
- the predecessor metadata schema is supported;
- current root asset and amount requirements are separately enforced.

### 5.3 Successor metadata correctness

The successor metadata must derive from the semantic operation.

The caller may not supply an arbitrary successor metadata blob and ask the
script to commit it without checking every semantic assignment.

### 5.4 Static code continuity

The successor must use the same authenticated static code subtree as the
predecessor.

The design must reject:

- successor under a different bundle root;
- successor with one operation leaf removed;
- successor with an extra escape leaf;
- successor with altered leaf version;
- successor under another internal key;
- predecessor and successor using independently supplied static roots.

### 5.5 Complete linked code scope

The static code root must bind exactly the operation programs in the selected
deployment scope.

A final production bundle must not contain undeclared leaves.

A pilot subtree with placeholder leaves cannot be release-final.

### 5.6 No key-path escape

No party may know the private key for the constructor's key path.

The design must:

- use an accepted NUMS/internal-key construction;
- document its exact provenance;
- contain no production key generation;
- contain no retained secret;
- bind the internal key in the linked constructor identity.

### 5.7 No metadata-leaf escape

If a metadata leaf is used, spending it must fail.

The target tree must not contain another metadata-related spend path.

### 5.8 Target-total constructor rule

For every valid semantic metadata value in the supported domain, the
constructor rule must either:

1. produce one valid target constructor deterministically; or
2. use an explicitly accepted representation/retry rule that is total and
   semantically safe.

A negligible-probability undefined constructor is still a formal
constructibility issue and must be resolved or named as an accepted residual
through a decision.

### 5.9 Public constructibility

Every public/permissionless STATE transition must be constructible from:

- public predecessor metadata;
- public linked static root/constructor recipe;
- public target facts;
- public successor formula;
- constructor-local sponsor material only.

No owner/operator secret may be required except where operation authorization
itself requires the operator.

The root's covenant-companion participation carries no independent owner
secret.

### 5.10 Determinism

Given identical:

- linked static code root;
- object kind;
- metadata schema;
- semantic metadata;
- internal-key policy;
- target identity;
- constructor configuration;

the constructor instance is identical.

### 5.11 Resource feasibility

The predecessor and successor verification pattern, complete operation leaf,
witness, control path, and representative complete transaction must fit the
selected target's:

- script/program limits;
- stack limits;
- stack element limits;
- witness/policy limits;
- crypto budget;
- transaction weight;
- standardness policy required by deployment.

### 5.12 Evidence and provenance

The linked constructor must expose:

- object kind;
- static root;
- internal key;
- metadata schema;
- constructor recipe;
- operation program/leaf;
- relation carriers;
- target capabilities;
- resource formula;
- source provenance;
- bundle identity;
- ABI identity.

---

## 6. Candidate approaches

No candidate is accepted by this note until the prototype and decision process
complete.

---

### Candidate A — Static code subtree plus dynamic unspendable metadata leaf

#### Construction

For object kind `K`:

```text
fixed NUMS internal key P_K
static linked code subtree root C_K
dynamic unspendable metadata leaf L_K(M)

R_K(M) = TapBranch(C_K, L_K(M))
Q_K(M) = P_K + TapTweak(P_K, R_K(M))G
```

The output program commits `Q_K(M)`.

#### Spend-time predecessor verification

The operation program receives or derives:

- predecessor metadata `M_before`;
- static code root `C_K`;
- required parity/compressed-point data;
- target schema/domain constants.

It:

1. encodes `M_before`;
2. derives the metadata leaf hash;
3. derives `R_K(M_before)`;
4. derives the tweak;
5. inspects the current input output program;
6. verifies it equals the expected tweaked key/program;
7. thereby authenticates `M_before` and `C_K`.

#### Successor verification

The operation program computes `M_after` from the semantic relation and then:

1. derives `L_K(M_after)`;
2. reuses the same authenticated `C_K`;
3. derives `R_K(M_after)`;
4. derives the successor tweak;
5. inspects the canonical successor output;
6. verifies it equals the expected constructor.

#### Advantages

- static operation programs remain stable across metadata values;
- dynamic metadata is committed into the output program;
- one constructor family supports many instances;
- no final static-root literal must be embedded recursively inside each leaf;
- linked code continuity can be witnessed and authenticated inductively;
- metadata commitment is reviewable;
- target script-path enforcement remains local.

#### Risks

- static-root witness authentication must be exact;
- metadata-leaf serialization/hash construction may be expensive;
- target tweak verification requires exact point/parity handling;
- key-path escape must be cryptographically closed;
- hash-to-scalar totality must be resolved;
- metadata leaf must be unspendable;
- witness size and crypto budget may be significant;
- dynamic control-path construction must be canonical;
- public metadata at spend time may reveal more than some future objects need.

#### Current status

Leading candidate. Requires complete target-native prototype.

---

### Candidate B — Fixed program with separately welded metadata output

#### Construction

Keep the protocol object's scriptPubKey/program fixed.

Store semantic metadata in a separate target output or data commitment welded
to the protocol object.

#### Advantages

- byte-identical program succession;
- simpler self-reference;
- simple static constructor;
- root program may use direct input/output program equality.

#### Risks

- introduces another object/output relation;
- must prove unique pairing between root and metadata;
- metadata output may be attacker-forgeable or replaceable;
- separate output adds transaction weight and contention;
- global state identity may split across objects;
- history replay and object recognition become more complex;
- may alter architecture object cardinality and therefore protocol denotation;
- metadata output must remain available for future operations;
- could reintroduce the same weld problem at another layer.

#### Current status

Research alternative only. Likely requires normative architecture review if it
adds a standing protocol object.

---

### Candidate C — Metadata embedded directly in every operation leaf

#### Construction

Generate operation leaves separately for every metadata value, with metadata
embedded in each script.

#### Advantages

- metadata available directly to the program;
- no separate metadata leaf;
- straightforward target script decoding.

#### Risks

- static code subtree changes with every state;
- successor requires regenerating the entire operation tree;
- operation programs depend recursively on their own resulting root;
- link-time fixed point may be undefined;
- script/witness sizes grow;
- every operation duplicates metadata;
- tree construction and ABI become state-dependent;
- difficult to preserve one static code identity.

#### Current status

Expected to be rejected unless a concrete fixed-point-free construction is
demonstrated.

---

### Candidate D — Fixed program with metadata supplied only in witness/history

#### Construction

Use a fixed program and provide STATE metadata in the spend witness without
committing it into the predecessor output program.

#### Advantages

- constant scriptPubKey;
- simple succession;
- small constructor.

#### Risks

- predecessor metadata is attacker-chosen;
- no binding to the consumed state;
- root history cannot authenticate semantic state;
- current transaction can select whichever predecessor fields make its formula
  pass;
- breaks STATE identity and consensus-value authority.

#### Current status

Rejected by existing semantic constraints unless an independent authenticated
commitment is added, in which case the candidate becomes another form of A or
B.

---

### Candidate E — Metadata commitment in another consensus field

#### Construction

Commit STATE metadata through another target field or protocol mechanism rather
than a dynamic taptree leaf.

Possible examples might involve:

- a target output nonce;
- an asset/value commitment relation;
- a dedicated commitment output;
- a target program commitment mechanism.

#### Advantages

- may reduce taptree recursion;
- could produce simpler target programs;
- may preserve a fixed script program.

#### Risks

- field may not be reliably introspectable;
- commitment may be attacker-selected;
- may interact with CT semantics;
- may be unavailable in the target UTXO database/sighash;
- may require additional outputs;
- may alter object recognition;
- target behavior may not support exact successor verification;
- could create a new lifecycle or availability dependency.

#### Current status

Open alternative only if exact target facts demonstrate a suitable authenticated
field.

---

## 7. Threat and failure model

The prototype assumes an attacker can choose:

- transaction inputs and outputs;
- input/output ordering where not constrained by ABI;
- witness items;
- predecessor metadata witness;
- successor metadata candidate;
- static-root witness;
- parity hints;
- target program/leaf;
- control path;
- target output programs;
- sponsor inputs;
- alternate linked-bundle roots;
- noncanonical metadata encodings;
- confidential/open outputs where target permits them.

The prototype assumes the attacker cannot break:

- target consensus;
- SHA-256 collision resistance or second-preimage resistance;
- secp256k1 discrete-log hardness;
- target signature security;
- native closed-asset conservation;
- target taproot/tweak semantics.

These cryptographic and substrate assumptions must be named in deployment
evidence.

### 7.1 Wrong predecessor metadata

Attacker supplies `M_fake` while spending a predecessor committed to
`M_actual`.

Required result:

```text
reject
```

### 7.2 Wrong successor metadata

Attacker creates successor for fields other than the semantic operation result.

Required result:

```text
reject
```

### 7.3 Wrong static code subtree

Attacker uses the correct successor metadata under another static root.

Required result:

```text
reject
```

### 7.4 Split-root substitution

Attacker authenticates predecessor under `C_before` but uses separately
supplied `C_after` for successor.

Required result:

```text
reject
```

The production pattern should reuse one authenticated static-root stack value
or prove exact equality.

### 7.5 Missing/extra operation leaf

Attacker creates successor under a tree omitting or adding operation programs.

Required result:

```text
reject
```

### 7.6 Wrong internal key

Attacker uses another internal key with the same metadata and static subtree.

Required result:

```text
reject
```

### 7.7 Key-path bypass

Attacker spends the object through a key path without executing a covenant
leaf.

Required property:

```text
no known valid private key exists for the selected internal/tweaked key path
```

This is supported by internal-key construction evidence rather than exhaustive
testing.

### 7.8 Metadata-leaf spend

Attacker selects the metadata leaf as the spending leaf.

Required result:

```text
target script failure
```

### 7.9 Alternate metadata encoding

Attacker uses:

- noncanonical integer;
- reordered field;
- alternate sentinel;
- unknown schema;
- duplicate/trailing data;
- alternate length encoding.

Required result:

```text
reject or fail constructor equality
```

### 7.10 Wrong parity or point form

Attacker supplies incorrect parity/compressed point for inspected x-only key.

Required result:

```text
reject
```

### 7.11 Unsupported tweak scalar

Constructor hash maps outside the valid scalar domain.

Required result:

```text
behavior defined by accepted totality policy
```

Undefined behavior is unacceptable.

### 7.12 Alternate witness version/program class

Attacker supplies a non-taproot or wrong-version output program with matching
bytes in another interpretation.

Required result:

```text
reject
```

### 7.13 Stale bundle constructor

Attacker uses a predecessor/successor constructor from another linked bundle or
backend configuration.

Required result:

```text
reject
```

### 7.14 Wrong successor output slot

Attacker places a valid successor constructor at a different output while the
canonical slot contains another output.

Required result:

```text
reject under canonical ABI
```

---

## 8. Leading candidate mechanics to prototype

This section describes the Candidate-A prototype target, not an accepted
production design.

### 8.1 Internal key

Select one deterministic NUMS internal key `P_STATE` with:

- exact derivation/provenance;
- valid x-only target encoding;
- no known discrete logarithm;
- no deployment secret;
- domain separation for attestation-contract constructor use;
- deterministic identity.

The prototype must document why the selected point does not create a known
key-path secret.

### 8.2 Static subtree

Construct one static STATE operation subtree root:

```text
C_STATE
```

For the synthetic prototype, the tree may contain:

- one active transition leaf;
- deterministic placeholder leaves sufficient to exercise realistic tree
  structure.

For the full prototype, use the expected STATE operation-leaf census or a
clearly identified representative structure.

Placeholder leaves make the prototype nonfinal.

### 8.3 Metadata encoding

Define a prototype STATE metadata schema containing at least:

```text
object/domain tag
metadata schema version
Ω
Y_L
Y_T
Q
cycle
maturity tag/payload
```

Requirements:

- fixed field order;
- exact widths/domains;
- no ambiguous sentinel;
- canonical encoding;
- total decode over valid states;
- deterministic leaf script length/encoding.

### 8.4 Metadata leaf

Construct a provably unspendable leaf, expected conceptually to contain:

```text
unspendable opcode
domain separator
object kind
schema version
metadata encoding or metadata digest
```

The prototype must determine whether the leaf commits:

- complete metadata bytes;
- a canonical metadata digest;
- another structured commitment.

The operation program must still authenticate the semantic fields it uses.

### 8.5 Target hash construction

Implement exact target:

- tapleaf hash;
- tapbranch hash and canonical child ordering;
- taptweak tagged hash;
- scalar validation;
- output-key relation.

Use typed target definitions from `target-elements`.

Do not rely on host-library helpers without cross-checking target semantics and
canonical vectors.

### 8.6 Predecessor verification program

The synthetic transition leaf should:

1. inspect current input program and version;
2. require the expected target program class;
3. receive predecessor metadata fields;
4. receive one static code-root witness;
5. encode/hash the metadata leaf;
6. combine metadata leaf with static root;
7. derive target tweak;
8. reconstruct or verify the current input key/program;
9. retain the authenticated static root for successor verification.

The prototype must record exact stack behavior.

### 8.7 Successor verification program

The program should:

1. derive successor metadata from predecessor fields and operation witness;
2. encode/hash successor metadata;
3. combine it with the **same authenticated static root**;
4. derive successor tweak;
5. inspect the canonical output program;
6. verify successor program/key;
7. enforce the operation's semantic state relation.

### 8.8 X-only/compressed bridge

If target output inspection yields an x-only key while tweak verification
requires a compressed point:

1. inspect x-only key;
2. validate target program version;
3. receive or derive one canonical parity bit;
4. validate parity encoding;
5. construct compressed point;
6. execute tweak verification;
7. reject wrong parity and malformed point.

The predecessor and successor may each require a parity witness unless target
semantics provide another route.

### 8.9 Root asset and amount

The constructor prototype must also enforce or integrate with separate
relations that the current STATE input and successor carry:

- explicit `PID` asset identity;
- amount one;
- canonical root input/output cardinality.

Constructor equality alone does not prove root identity.

### 8.10 Operation prototype

After the synthetic increment-style transition succeeds, integrate the
`announce-maturity` semantic relation because it is the smallest planned
STATE-changing operation:

- predecessor maturity unannounced;
- valid lead range;
- successor maturity announced;
- every other STATE field unchanged;
- operator authorization;
- no RESV use;
- optional sponsor flow separately enforced.

The constructor research can be considered mechanically successful before the
complete operation backend is release-ready, but the actual operation provides
the first realistic end-to-end test.

---

## 9. Hash-to-scalar totality

The target tweak relation may require a scalar strictly inside the secp256k1
order.

A 32-byte hash does not automatically satisfy that domain for every possible
value.

The prototype must evaluate the following policies.

### 9.1 Policy A — reject out-of-range tweak

#### Behavior

If the target tweak hash is outside the scalar domain, constructor creation
fails.

#### Advantages

- simple;
- matches strict target behavior;
- no representation nonce;
- canonical when defined.

#### Risks

- constructor is not total over semantic metadata;
- a valid abstract successor could become unconstructible;
- creates a negligible-probability liveness/semantic trap;
- requires an explicitly accepted residual if retained.

#### Current status

Not accepted without normative and assurance review.

### 9.2 Policy B — deterministic representation nonce

#### Behavior

Include a representation-only nonce/counter in the metadata leaf commitment and
search deterministically for a valid tweak.

#### Advantages

- can make construction practically total;
- deterministic first-party builder;
- keeps semantic metadata unchanged.

#### Risks

- several concrete constructor encodings may represent the same semantic state;
- script may need to authenticate or bound the nonce;
- enforcing “first valid nonce” on-chain may be expensive;
- noncanonical attacker-selected nonce may enlarge accepted representation;
- nonce becomes part of constructor/ABI identity.

#### Questions

- Is any valid nonce semantically safe?
- Is first-party canonicality sufficient even if the target accepts several?
- Must the target prove the nonce is the first valid one?
- What nonce domain guarantees practical or mathematical totality?

### 9.3 Policy C — internal-key retry/derivation

#### Behavior

Derive one of several NUMS internal keys deterministically until the tweak is
valid.

#### Advantages

- metadata leaf can remain unique;
- may preserve constructor recipe structure.

#### Risks

- internal key changes by metadata instance;
- static constructor identity becomes more complex;
- predecessor/successor must bind selected key;
- key-path safety/provenance must hold for every candidate;
- several concrete encodings may still exist.

### 9.4 Policy D — accepted cryptographic-negligibility residual

#### Behavior

Use fixed key and direct tweak; document the out-of-range event as a
cryptographically negligible constructibility residual.

#### Advantages

- simplest production construction;
- aligns with common taproot practice where invalid tweak probability is
  negligible.

#### Risks

- weakens total semantic constructibility;
- must be explicitly reflected in trust/residual documentation;
- may conflict with permissionless/lifecycle claims;
- cannot be hidden as “impossible.”

### 9.5 Required decision

The research cannot resolve until one policy is accepted or another total
construction is demonstrated.

The decision must state:

- semantic effect;
- canonical representation;
- target validation;
- constructor identity;
- witness/ABI impact;
- residual risk;
- release evidence.

---

## 10. Key-path escape policy

### 10.1 Required internal-key property

The internal key must have no known private scalar.

The project must not use:

- operator key;
- deployment signer key;
- random key generated and then “discarded” without auditable procedure;
- test key in production;
- target key whose secret is retained by build tooling.

### 10.2 Candidate NUMS construction

The research should compare:

1. one standard source-pinned NUMS point;
2. a Peer-Attestation-domain-separated NUMS derivation;
3. another target-supported unspendable internal-key mechanism.

### 10.3 Evidence limits

No finite test proves that nobody knows a discrete logarithm.

Evidence should instead establish:

- deterministic public derivation;
- no secret-generation step;
- source/procedure review;
- target point validity;
- exact public key identity;
- no private key field in build/deployment tooling.

### 10.4 Release binding

The final constructor/bundle identity must bind the exact internal key.

Changing the internal key changes the constructor and linked-bundle identity.

### 10.5 Test-only keys

Prototype tests may use a known test internal key only when exercising target
mechanics unavailable with a NUMS key.

Such tests cannot establish no-key-path production safety and must be clearly
separated.

The final target-native constructor vectors must use the selected production
NUMS policy.

---

## 11. Prototype design

### 11.1 Prototype location

Preferred locations, in order:

1. experimental/private module under the future `tapscript` package;
2. test-only integration fixture using typed target/linker components;
3. separate temporary prototype package if dependency isolation requires it.

Requirements:

- excluded from final release;
- clearly marked experimental;
- no stable public API;
- no production fallback;
- explicit target identity;
- explicit report output.

### 11.2 Prototype stages

#### Stage A — Off-chain constructor reference

Implement an independent Rust reference for:

- metadata encoding;
- metadata leaf script/hash;
- static subtree composition;
- taptree root;
- tweak;
- x-only/compressed output key;
- constructor output program.

Produce canonical vectors.

#### Stage B — Synthetic predecessor check

Create a synthetic metadata-bearing object and target program that validates
its own predecessor constructor.

Use a simple metadata field such as:

```text
counter
```

before full STATE.

#### Stage C — Synthetic successor check

Implement:

```text
counter_after = counter_before + 1
```

and verify the successor constructor under the same static root.

This isolates constructor mechanics from full protocol arithmetic.

#### Stage D — Wrong-code-root faults

Add the complete static-root substitution vector family.

This is the central safety milestone.

#### Stage E — Full STATE metadata schema

Replace synthetic metadata with the full typed STATE encoding.

Test boundary values and maturity variants.

#### Stage F — Representative static operation subtree

Use a deterministic representative STATE subtree with the intended leaf census
or a clearly documented approximation.

Measure control paths and witness requirements.

#### Stage G — `announce-maturity` integration

Implement predecessor/successor constructor checks around the actual semantic
STATE delta and authorization.

#### Stage H — Transaction ABI handoff

Generate a candidate linked constructor recipe and candidate transaction ABI.

Construct complete target-native transactions.

#### Stage I — Resource and reproducibility report

Measure and reproduce exact target results.

### 11.3 Independent reference implementation

Where practical, derive expected constructor bytes through two independent
paths:

- first-party explicit reference implementation;
- selected Elements/Rust library implementation.

Compare exact values.

Shared library use must be documented.

### 11.4 Prototype status type

Prototype artifacts should carry:

```text
Prototype
```

or equivalent status that the release package rejects.

---

## 12. Test and vector plan

### 12.1 Positive constructor vectors

Include:

- minimum valid metadata values;
- representative normal state;
- maximum valid amount fields below domain bound;
- each maturity variant;
- unchanged semantic fields;
- valid synthetic increment;
- valid announce-maturity transition;
- valid sponsorless operation;
- valid sponsored operation;
- deterministic reconstruction from canonical metadata;
- same static root across predecessor and successor.

### 12.2 Metadata negative vectors

- wrong `Ω`;
- wrong `Y_L`;
- wrong `Y_T`;
- wrong `Q`;
- wrong cycle;
- wrong maturity tag;
- wrong announced cycle;
- malformed sentinel;
- field reorder;
- duplicate field;
- trailing bytes;
- noncanonical integer;
- unknown schema;
- wrong object/domain tag;
- metadata digest mismatch.

### 12.3 Static-code continuity vectors

- successor under another static root;
- predecessor and successor use two independently supplied roots;
- one operation leaf removed;
- one escape leaf added;
- altered leaf bytes;
- altered leaf version;
- wrong subtree child order;
- static root from another linked bundle;
- correct code root but wrong object kind.

### 12.4 Internal-key and point vectors

- wrong internal key;
- wrong x-only key;
- wrong predecessor parity;
- wrong successor parity;
- invalid compressed point;
- malformed parity encoding;
- known test key rejected by production constructor policy;
- target program with wrong witness version.

### 12.5 Metadata-leaf vectors

- valid code-leaf spend;
- direct metadata-leaf spend;
- alternate spendable metadata script;
- omitted metadata leaf;
- metadata leaf under wrong version;
- metadata leaf with noncanonical script length.

### 12.6 ABI/layout vectors

- successor at canonical output;
- successor at wrong output;
- two STATE successors;
- no STATE successor;
- wrong coordinator/current input;
- sponsor output occupying STATE slot;
- wrong input count;
- wrong target program class.

### 12.7 Root identity vectors

- correct explicit PID amount one;
- wrong asset;
- correct PID wrong amount;
- confidential PID asset;
- duplicate PID/root output;
- wrong root predecessor outpoint where history fixture applies.

### 12.8 Key-path vectors

Finite vectors cannot prove no secret key exists.

Still include:

- arbitrary key-path signature rejection under the selected NUMS key;
- no production private key in constructor/deployment fixtures;
- exact NUMS derivation vector;
- source/procedure evidence.

### 12.9 Hash/scalar vectors

At the hash helper/pattern level:

- valid tweak zero/low/high scalar forms as target permits;
- scalar equal to or above curve order rejected;
- malformed scalar width rejected;
- deterministic metadata nonce/retry vectors if selected;
- canonical first-party representation;
- alternate nonce behavior according to accepted policy.

Finding a real metadata preimage producing an out-of-range hash is not required
if infeasible; test scalar-domain handling through lower-level pattern fixtures
and prove the constructor policy separately.

### 12.10 Mixed-program vectors

- STATE leaf from one operation combined with successor expectations from
  another;
- local companion program with wrong coordinator operation;
- operator-authorized leaf combined with permissionless branch assumptions;
- old-bundle predecessor with new-bundle successor.

### 12.11 History/replay vectors

Once integrated with model/linked history:

- valid STATE chain;
- intermediate wrong static root later restored at final cursor;
- final cursor correct but intermediate constructor wrong;
- successor under stale bundle then later valid bundle;
- post-sealing STATE revival attempt.

---

## 13. Measurement plan

### 13.1 Pattern-level measurements

Measure:

- predecessor constructor verification script bytes;
- successor constructor verification script bytes;
- metadata encoding/hash instructions;
- target tweak verification count;
- crypto-budget cost;
- witness bytes for metadata/static root/parity;
- initial stack item count;
- peak stack/altstack;
- maximum stack element;
- executed target opcodes/project cost.

### 13.2 Constructor-level measurements

Measure:

- static subtree leaf count;
- taptree depth per operation;
- control-path size;
- metadata-leaf size;
- constructor output program size;
- linked constructor manifest size.

### 13.3 Complete transaction measurements

At minimum measure a valid `announce-maturity` prototype transaction with:

- STATE predecessor;
- STATE successor;
- operator authorization;
- no sponsor;
- maximum permitted sponsor family under candidate bound;
- worst-case metadata witness fields;
- deepest relevant control path.

Later STATE operations may exceed this cost. The prototype acceptance applies
to the constructor mechanism, not every future operation's final resource fit.

### 13.4 Predicted versus observed

Compare:

- backend resource formulas;
- linker taptree/control formulas;
- transaction prediction;
- target-observed weight/policy/crypto behavior.

Any mismatch must be explained and fixed before production acceptance.

### 13.5 Target policy

Test both:

- consensus validity;
- policy/mempool validity required by deployment.

### 13.6 Reproducibility

Run the complete prototype twice from clean state and require identical:

- static subtree root;
- constructor recipe;
- metadata encodings;
- target program bytes;
- transaction bytes for deterministic test fixture;
- canonical report bytes.

---

## 14. Acceptance criteria

The leading or selected constructor design is accepted only when all mandatory
criteria pass.

### 14.1 Semantic safety

- [ ] predecessor metadata is authenticated against the actual consumed
      constructor;
- [ ] successor metadata equals the semantic operation result;
- [ ] successor constructor is verified at the canonical output;
- [ ] predecessor and successor use one authenticated static code identity;
- [ ] correct metadata under another static code subtree rejects;
- [ ] missing/extra/altered operation leaf changes constructor and rejects;
- [ ] wrong internal key rejects;
- [ ] wrong schema/object tag rejects;
- [ ] root asset and amount relations integrate correctly;
- [ ] history/replay can carry constructor continuity.

### 14.2 No escape path

- [ ] metadata leaf is unspendable;
- [ ] no alternate linked leaf/program exists outside approved scope;
- [ ] internal key has accepted NUMS/no-known-secret provenance;
- [ ] no production private key is generated or retained;
- [ ] arbitrary key-path spend cannot be produced in target tests;
- [ ] constructor output has exactly the selected target program class.

### 14.3 Totality and canonicality

- [ ] metadata encoding is injective and canonical;
- [ ] constructor rule is total over supported semantic metadata or an explicit
      accepted residual/retry policy exists;
- [ ] hash-to-scalar behavior is fully defined;
- [ ] first-party constructor output is deterministic;
- [ ] accepted alternate concrete encodings, if any, are represented
      explicitly and semantically safe.

### 14.4 Constructibility

- [ ] static root and constructor recipe are public bundle data;
- [ ] predecessor metadata is publicly available for STATE;
- [ ] successor construction requires no hidden root or internal-key secret;
- [ ] permissionless STATE paths require no owner/operator secret beyond their
      semantic authorization;
- [ ] transaction package can instantiate output and control data from the
      linked recipe.

### 14.5 Target evidence

- [ ] the reviewed target substrate is recorded as review provenance;
- [ ] every used primitive is typed in `target-elements`;
- [ ] positive target-native vectors pass;
- [ ] every required negative target-native vector rejects;
- [ ] wrong-code-subtree vector reaches target execution;
- [ ] target execution domain/leaf version is enforced.

### 14.6 Resource feasibility

- [ ] pattern fits target stack/element/script limits;
- [ ] crypto budget passes;
- [ ] representative complete transaction passes consensus limits;
- [ ] representative complete transaction passes required policy limits;
- [ ] predicted and observed resources agree;
- [ ] no resource assumption depends on unresolved target policy.

### 14.7 Toolchain integration

- [ ] tapscript backend can represent the pattern;
- [ ] linker can resolve the static subtree and constructor recipe without
      manual patching;
- [ ] transaction package can derive candidate ABI and construct a valid
      transaction;
- [ ] vector package can materialize all mandatory faults;
- [ ] release package can distinguish prototype from production artifact;
- [ ] all outputs are deterministic and identity-bound.

### 14.8 Decision handoff

- [ ] selected candidate and rejected alternatives are documented;
- [ ] implementation decision is created or updated;
- [ ] package plans reflect the accepted design;
- [ ] permanent vectors are assigned;
- [ ] target evidence requirements are updated;
- [ ] residual risks are named.

---

## 15. Rejection criteria

A candidate is rejected for the initial target if any of the following holds
and no approved modification resolves it.

### 15.1 Safety rejection

- accepts successor under a different static code subtree;
- trusts predecessor metadata without binding it to input constructor;
- permits independently selected predecessor/successor static roots;
- leaves a spendable metadata or key-path escape;
- cannot enforce canonical successor metadata;
- permits alternate schema/encoding with different meaning;
- fails root asset/amount integration;
- cannot preserve complete operation-leaf scope.

### 15.2 Constructibility rejection

- requires a private internal-key secret;
- requires another participant's secret on a permissionless path;
- static root/metadata needed for construction is unavailable;
- constructor is undefined for valid semantic metadata without accepted policy;
- transaction/control data cannot be independently constructed from public
  bundle/ABI.

### 15.3 Target rejection

- required primitive absent or inactive;
- target stack behavior differs from the pattern;
- wrong-code-root mutation cannot be made to reach enforcement;
- target accepts a mandatory negative vector;
- production-equivalent target cannot support the construction.

### 15.4 Resource rejection

- pattern exceeds hard target limit;
- representative complete transaction exceeds required policy/consensus limit;
- crypto budget cannot fit without artificial witness padding or another
  unaccepted workaround;
- resource formula cannot be reconciled with observed execution.

### 15.5 Determinism rejection

- identical typed inputs produce different constructor identities or bytes;
- canonical metadata has multiple uncontrolled encodings;
- linking requires manual iterative patching;
- constructor depends on host/environment state.

### 15.6 Assurance rejection

- prototype cannot be bound to exact target/bundle/ABI identities;
- reports include secrets;
- evidence relies only on local mock execution;
- production design differs materially from tested prototype.

A rejected candidate may remain viable for another backend.

---

## 16. Result

Pending.

When populated, this section must include:

- exact repository revision;
- exact target definition and deployment instance;
- candidate implementation revision;
- selected metadata schema;
- selected internal-key policy;
- static subtree structure;
- constructor recipe;
- witness schema;
- positive/negative vector result summary;
- predicted/observed resource table;
- deterministic artifact hashes;
- failed candidates and failure reasons;
- exact acceptance/rejection verdict.

Do not replace this section with a narrative claim unsupported by canonical
reports.

---

## 17. Decision and implementation handoff

Pending.

Expected handoff if Candidate A succeeds:

1. create a new accepted decision, tentatively:

   ```text
   D007: Use an Authenticated Static Code Root with Dynamic Unspendable
   Metadata Commitments for Elements Object Constructors
   ```

2. update `target-elements` with the approved complete constructor-verification
   capability;
3. stabilize tapscript constructor pattern APIs;
4. stabilize linker symbol/reference strategies;
5. define production metadata schema and constructor identity;
6. define transaction ABI witness/control roles;
7. promote permanent continuity vectors;
8. add release evidence requirements;
9. update roadmap Phase 6 from prototype-blocked to implementation-ready.

If no candidate succeeds:

- record the failure;
- identify whether another target approach remains;
- determine whether the issue is implementation-only or requires normative
  object-model review;
- do not begin STATE-spending backend operation implementation.

---

## 18. Residual risks

Even an accepted construction will retain residual assumptions.

### 18.1 Hash assumptions

Code/metadata continuity relies on target hash collision and second-preimage
resistance.

### 18.2 Discrete-log assumption

Key-path unspendability relies on no known discrete logarithm for the selected
internal key and ordinary curve assumptions.

### 18.3 Target implementation

Correctness relies on the reviewed target's:

- program introspection;
- hashing;
- tweak verification;
- script-path semantics;
- control-path validation.

### 18.4 Bundle upgrade/migration

Changing the static code subtree changes constructor identity.

Existing objects may require:

- explicit migration operation;
- old-bundle support;
- terminal settlement;
- versioned constructor coexistence.

This research does not define upgrade policy.

### 18.5 Public metadata

STATE metadata is public. The constructor does not provide STATE privacy.

### 18.6 Resource growth

The constructor may fit `announce-maturity` while larger STATE operations still
fail resource calibration.

### 18.7 Representation totality

Any nonce/retry or negligible-failure policy introduces representation and
constructibility complexity that must remain visible in release residuals.

### 18.8 Independent implementation complexity

External clients must implement exact constructor and metadata rules.

Canonical ABI vectors mitigate but do not eliminate implementation risk.

---

## 19. References

### Normative and current source

- [`../../docs/attestation/realization.md`](../../docs/attestation/realization.md)
- [`../../packages/architecture/src/spec.rs`](../../packages/architecture/src/spec.rs)
- [`../../packages/model/src/object.rs`](../../packages/model/src/object.rs)
- [`../../packages/model/src/pool.rs`](../../packages/model/src/pool.rs)
- [`../../packages/model/src/certify.rs`](../../packages/model/src/certify.rs)
- [`../../packages/model/src/invariant.rs`](../../packages/model/src/invariant.rs)
- [`../../packages/model/src/ops/maturity.rs`](../../packages/model/src/ops/maturity.rs)
- [`../../packages/model/src/tests/root_certificate_fault_tests.rs`](../../packages/model/src/tests/root_certificate_fault_tests.rs)

### Decisions

- [D001](../decisions/001-typed-rust-source.md)
- [D002](../decisions/002-realization-layer.md)
- [D003](../decisions/003-tapscript-first.md)
- [D004](../decisions/004-translation-validation.md)
- [D005](../decisions/005-value-representation.md)
- [D006](../decisions/006-transaction-abi.md)

### Package plans

- [`../packages/target-elements.md`](../packages/target-elements.md)
- [`../packages/tapscript.md`](../packages/tapscript.md)
- [`../packages/linker.md`](../packages/linker.md)
- [`../packages/transaction.md`](../packages/transaction.md)
- [`../packages/vectors.md`](../packages/vectors.md)
- [`../packages/release.md`](../packages/release.md)

### Target reference

- [`../reference/elements-tapscript.md`](../reference/elements-tapscript.md)

### Roadmap

- [`../roadmap.md`](../roadmap.md)

---

## 20. One-line research contract

> Determine, by exact typed-target construction and adversarial
> vectors, whether a metadata-dependent taproot object can authenticate its
> predecessor metadata and static code identity, derive its semantic successor,
> and verify that successor under the same code relation with no key-path or
> metadata escape, no hidden secret dependency, canonical total construction,
> deterministic linking, and acceptable complete-transaction resources—and do
> not freeze any STATE constructor, linker symbol, witness ABI, or
> STATE-spending operation until that result is accepted.
