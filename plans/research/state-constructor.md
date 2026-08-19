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
discipline warns against (`case:identity:evidence`),
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
