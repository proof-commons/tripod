# Draft: Guide 11 Concept — Public Declassification and Confidential-to-Public Lifecycle

> **Status:** Concept draft; not an execution guide
> **Phase:** Phase 3 — Elements target and foundational prototypes
> **Entry:** Guide-10 constructor and wide-arithmetic decisions recorded; hardened native-evidence boundary in place
> **Primary research owner:** [public declassification](../research/public-declassification.md)
> **Affected packages:** `target-elements`, `tapscript`, `target-elements-conformance`
> **May affect after acceptance:** `realization`, `compiler`, `transaction`, `vectors`
> **Does not implement:** an attestation-contract operation, linked bundle, transaction ABI, calibrated bound, production signer, wallet, or release
> **Required result:** accepted initial declassification policy or explicit target rejection

---

## Mission · `sec:guide11:mission`

Guide 11 determines how a semantically private committed value may cross into a representation that is publicly authenticated and usable by unrelated future constructors.

The decisive lifecycle is:

```text
private committed value
    ↓
owner-authorized boundary or normalization
    ↓
publicly authenticated value
    ↓
fresh unrelated permissionless constructor
```

The guide must distinguish three questions that are easy to collapse incorrectly:

1. **Disclosure** — is the amount public?
2. **Authentication** — is that amount cryptographically bound to the target value commitment?
3. **Availability** — can an unrelated future constructor obtain every fact and witness needed to verify and use it?

A value is not publicly usable merely because its owner knows it. A value is not authenticated merely because it appears in metadata. A valid opening is not durable merely because it existed in the creator’s process. A commitment relation established by a host library is not target-native evidence that the emitted target program enforces it.

Guide 11 must produce an exact target-native answer for the initial Elements backend and then select one initial policy:

```text
direct public opening accepted

owner-authorized normalization accepted

explicit-only boundary accepted

or

target rejected the proposed confidential-to-public path
```

A mixed result is permitted. For example:

```text
private lateral transfer:
    accepted

normalization to public committed:
    accepted

direct private burn:
    deferred

direct private redemption:
    deferred

explicit boundary operations:
    accepted initial policy
```

The guide succeeds by establishing the honest matrix, not by maximizing the number of cells marked supported.

---

## One-line thesis · `rem:guide11:thesis`

> Guide 11 must prove that every public amount used after a confidential-value boundary is exactly bound to the target commitment, explicit closed-asset identity, canonical public evidence, and a permissionless future lifecycle—or else select normalization or an explicit-only policy without describing unsupported direct privacy as implemented.

---

# 1. Executive rulings · `sec:guide11:rulings`

## 1.1 Safety, disclosure, and constructibility remain separate axes

The guide must never infer one of these from another:

```text
target accepts the transaction
≠
the public amount is authenticated

public amount is authenticated
≠
an unrelated future constructor can obtain the proof

future constructor can obtain the proof
≠
the representation is disclosure-minimal

one representation is safe
≠
every representation is safe
```

Every candidate is evaluated on at least three independent axes:

| Axis | Question |
|---|---|
| safety | Can an invalid amount, asset, commitment, recipient, or transition be accepted? |
| disclosure | Which semantic facts become public, and why? |
| constructibility | Can the authorized or permissionless constructor obtain the required witness? |

A candidate passes only the claims it actually establishes.

## 1.2 The three value modes retain distinct meanings

Guide 11 uses the realization’s three-mode vocabulary:

```text
PrivateCommitted
    amount not publicly available
    value commitment remains target-enforced
    opening remains private

PublicCommitted
    amount and authenticated opening are public
    value commitment remains target-enforced
    future target relations may verify the opening

Explicit
    amount directly encoded
    no value-blinding term remains on that output
```

The modes denote the same semantic amount. They do not have the same:

- disclosure;
- transaction-balance construction;
- witness requirements;
- target resource cost;
- future lifecycle;
- residual blinding behavior.

`PublicCommitted` is not a synonym for “metadata says the amount.” It requires an approved exact relation binding the public amount to the exact target value commitment.

## 1.3 Closed protocol asset identity remains explicit

Value representation is parametric; closed-asset identity is not.

Every candidate must preserve explicit classification of closed protocol assets. In the future attestation-contract operations that consume the result, that includes:

```text
U
ENT
DIST_CTL
PID
PACE
ENT_AUTH
DIST_AUTH
```

The generic Guide-11 fixtures must not encode those protocol identities. Instead, they test the target-level rule:

```text
the value commitment is bound to the exact explicitly authenticated asset
or generator selected by the fixture
```

A confidential or unclassified asset commitment cannot satisfy a future closed-asset object relation.

## 1.4 Whole-transaction conservation and opening verification are different claims

Elements consensus may establish:

```text
the complete transaction balances
```

A target program may establish:

```text
this public amount opens this exact commitment under this exact asset generator
```

Neither substitutes for the other.

Whole-transaction balance does not prove that a public amount attached to one output corresponds to that output. A valid opening of one output does not prove that the transaction’s other confidential values balance.

Guide 11 therefore requires two separately typed evidence classes:

```text
opening-relation evidence

whole-transaction confidential-value-conservation evidence
```

Both must pass where the candidate relies on both.

## 1.5 Low-level curve and hash primitives are not an opening proof

The currently reviewed target contains low-level primitives such as:

- streaming SHA-256;
- scalar multiplication verification;
- tweak verification.

Their existence does not establish an authenticated public opening.

A public-opening candidate becomes supported only after a complete target program demonstrates:

1. exact amount domain;
2. exact asset/generator binding;
3. exact value-commitment binding;
4. scalar and point validity;
5. exact success/failure handling;
6. canonical witness form;
7. future public availability;
8. target-native positive and negative evidence;
9. resource feasibility.

No capability status is promoted merely because all apparent building blocks exist.

## 1.6 The capability and pattern ownership question is decided explicitly

The current target vocabulary includes claims such as:

```text
CommitmentEquality
AuthenticatedValueOpening
```

and currently marks them unsupported.

Guide 11 must decide whether an accepted construction is:

```text
a target primitive capability

a complete backend proof pattern over lower-level target primitives

or

an external target/consensus evidence requirement
```

The answer must not be implicit.

Recommended ownership:

```text
target-elements:
    primitive and consensus facts only

tapscript:
    complete opening and normalization proof patterns

target-elements-conformance:
    target-native execution and report mechanics
```

If the accepted result is a backend pattern, it must not silently upgrade the static target contract to claim that a primitive exists. If the target capability vocabulary currently conflates primitive availability with complete pattern availability, Guide 11 must repair that vocabulary before recording the result.

A research prototype does not automatically inhabit `BackendPatternId`. Promotion to a production backend pattern remains a separate reviewed act.

## 1.7 The Guide-10 evidence boundary is inherited, not weakened

Guide 11 begins only after the Guide-10 evidence hardening exists:

```text
complete fixture projection in every report row

exact case census

exact evidence-row census

typed claim-level coverage

validated report wrapper

observed network and genesis

exact target/deployment binding

bounded executor protocol

complete process-tree timeout cleanup
```

Guide 11 adds new claims to that framework. It does not reintroduce broad “one passing case means the row passed” aggregation.

## 1.8 No production secret enters the prototype

Guide-11 private openings and blinding factors are deterministic public test data.

They are secret-shaped but not secret under ADR-015 because they:

- exist only for disposable test transactions;
- authorize nothing of value;
- are clearly marked test-only;
- are never derived from production material;
- are discarded with the environment;
- never enter a production-secret interface.

No command accepts:

```text
private key
wallet seed
production blinding factor
production opening
RPC password
cookie path
bearer token
production endpoint
```

A future production transaction constructor requires its own secret-bearing design. Guide 11 does not establish one.

## 1.9 Public evidence must survive a fresh-process test

A public opening is not durable evidence merely because the process that created it can still remember it.

Every accepted public representation must pass:

```text
process A:
    construct and publish the target transaction

discard:
    all constructor-local private state
    all owner-local opening state
    all temporary files not part of the public fixture

process B:
    reconstruct the future spend using only:
        canonical public chain data
        reviewed target contract
        public prototype schema
        public constructor data
        constructor-local sponsor funds
```

If process B needs the original owner’s private opening, the representation is not permissionlessly constructible.

## 1.10 Capsule binding must avoid circular identity

A public opening capsule must bind to the intended target object without relying on a circular transaction-ID commitment.

Candidate binding fields include:

```text
domain separator
capsule schema
target contract version
constructor/prototype schema
output ordinal or typed output role
exact value commitment
exact explicit asset or generator
exact output program
metadata commitment
public amount
public opening data
```

A capsule must not bind to its own final transaction ID if the transaction ID commits to the capsule and thereby creates an unresolved self-reference.

The exact selected binding is a Guide-11 result.

## 1.11 Full private consumption must close residual blinding

A confidential input cannot be made explicit by prose.

For any private-to-public transition, the target transaction’s commitment algebra must close. The guide must distinguish:

```text
partial private consumption:
    a private change output may carry residual blinding

full private consumption:
    no private change remains
    residual blinding must still be routed or canceled exactly
```

Where the target requires a commitment-bearing public output to carry residual blinding, `Explicit` is not a valid substitute.

This is why the realization requires public-committed output where full confidential consumption cannot close into an explicit output.

## 1.12 Direct support and normalization are evaluated independently

The guide compares:

```text
direct:
    private input → public boundary object

normalization:
    private input → public representation
    public representation → boundary object

explicit-only:
    boundary accepts only already-explicit input
```

A direct candidate failing does not imply normalization fails.

A normalization candidate succeeding does not imply a direct path exists.

The final policy matrix must state both independently.

## 1.13 No attestation-contract operation is emitted

Guide-11 fixtures may model generic target transitions such as:

```text
private value → public committed value

private value → explicit value + private change

public committed value → future publicly constructed spend
```

They must not implement or name:

- burn;
- clear;
- redemption;
- settlement;
- cycle;
- STATE;
- receipt;
- ASH;
- pool;
- entitlement.

The guide tests the representation membrane, not an operation.

## 1.14 No speculative digest is introduced

Guide 11 mints no:

```text
OpeningPatternHash
CapsuleHash
NormalizationPatternHash
DeclassificationFixtureSetHash
DeclassificationReportHash
```

Typed values and exact fixture bytes are sufficient.

A future transaction ABI, linked bundle, or release report may activate identities under ADR-016. Guide 11 does not reserve fields for them.

---

# 2. Entry conditions · `sec:guide11:entry`

Guide 11 begins only when:

- Guide 10 has recorded its constructor and wide-arithmetic outcomes;
- the native evidence framework returns validated report wrappers;
- complete fixture projections are embedded or otherwise typed-bound;
- claim-level evidence coverage exists;
- the exact reviewed target is bound to the development binding;
- the executor reports the actual chain and genesis it runs;
- executor timeout and protocol-size limits are enforced;
- the starting tree is clean;
- the exact starting revision is recorded.

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

## 2.1 Constructor dependency

Public-committed representations may need metadata- or constructor-bound capsules.

If the accepted Guide-10 constructor can bind a capsule into an object constructor without introducing an escape path, Guide 11 may use it.

If Guide 10 rejected metadata-dependent constructor continuity, Guide 11 must prefer:

- separate public data output;
- witness-history publication;
- owner-authorized normalization;
- explicit-only boundary policy;

rather than silently assuming constructor-bound metadata is available.

## 2.2 Arithmetic dependency

The standalone opening verifier may need only bounded narrow arithmetic for:

```text
amount domain
scalar encoding
point relations
```

It must not depend on the Guide-10 wide-floor pattern unless the actual opening relation requires it.

Guide 11 should not turn an accepted wide-floor prototype into a universal arithmetic dependency.

## 2.3 Existing target status

At entry, the reviewed target is expected to state honestly:

```text
confidential-value conservation:
    external/incomplete target claim

commitment equality:
    unsupported

authenticated opening:
    unsupported
```

Those statuses remain until complete reviewed evidence supports a change.

The guide is not allowed to begin by editing them to the desired result.

---

# 3. Required reading and authority · `sec:guide11:authority`

Repository policy:

```text
AGENTS.md
adr/010-command-line-output-contract.md
adr/011-toolchain-and-dependency-policy.md
adr/014-meson-lint-census-and-stamps.md
adr/015-public-data-and-execution-trust.md
adr/016-semantic-identities-and-evidence-binding.md
adr/017-path-scope-and-host-filesystem-trust.md
adr/018-upstream-elements-workspace.md
```

Accepted decisions:

```text
plans/decisions/001-typed-rust-source.md
plans/decisions/003-tapscript-first.md
plans/decisions/004-translation-validation.md
plans/decisions/005-value-representation.md
plans/decisions/006-transaction-abi.md
plans/decisions/008-exact-certified-mathematics.md
```

Normative realization sections:

```text
docs/attestation/realization.md
    §5.8 sponsor-value opacity
    §10 representation conformance and opacity
    inspection-necessity rule
    inspection-burden rule
    sponsor erasure
    disclosure frontier
    O8 representation-parametric conformance
    O9 disclosure minimality
    P-asset-boundary
    P-opacity
    P-discharge
    P-declassify
    P-representation-liveness
    P-ct
```

Relevant imports include:

```text
(`[RZ-sec:realization:representation]`)
(`[RZ-rule:representation:inspection-necessity]`)
(`[RZ-rule:representation:inspection-burden]`)
(`[RZ-def:representation:sponsor-erasure]`)
(`[RZ-obl:oracle:representation]`)
(`[RZ-obl:oracle:disclosure]`)
(`[RZ-pin:pins:asset-boundary]`)
(`[RZ-pin:pins:opacity]`)
(`[RZ-pin:pins:declassify]`)
(`[RZ-pin:pins:repr-liveness]`)
(`[RZ-pin:pins:ct]`)
```

Research owner:

```text
plans/research/public-declassification.md
```

No package parses these documents as semantic input.

---

# 4. Scope · `sec:guide11:scope`

## 4.1 In scope

Guide 11 implements or decides:

- exact target review of value commitments and explicit/confidential value fields;
- independent public commitment construction;
- generic confidential transaction materialization in the native executor;
- deterministic public test blinders and nonces;
- whole-transaction CT balance fixtures;
- target-side commitment equality feasibility;
- target-side authenticated opening feasibility;
- exact public amount-domain checks;
- explicit asset/generator binding;
- public opening capsule candidates;
- canonical capsule encoding;
- output-instance binding;
- fresh-process public reconstruction;
- owner-authorized normalization candidates;
- direct private-to-public candidates;
- explicit-only initial policy;
- safety and minimality reports as separate outputs;
- accepted initial operation matrix or explicit target rejection;
- research, package, phase, and backlog handoff.

## 4.2 Out of scope

Guide 11 must not implement:

- direct attestation-contract burn;
- direct attestation-contract redemption;
- public ASH constructor;
- receipt normalization operation;
- final representation ABI;
- production confidential transaction construction;
- production randomness;
- production blinding-factor custody;
- wallet integration;
- signer integration;
- linked bundles;
- release evidence;
- deployment calibration;
- confidential closed protocol asset identity;
- private settlement or cycle arithmetic.

A successful generic normalization prototype is not a production normalization operation.

---

# 5. Package ownership · `sec:guide11:packages`

## 5.1 `target-elements`

`target-elements` owns only reviewed target facts:

- commitment and generator encodings;
- value-field forms;
- target introspection results;
- target consensus-conservation claim;
- low-level curve/hash primitives;
- newly reviewed primitive behavior if required;
- evidence requirements;
- capability prerequisites and statuses.

It remains dependency-free.

It does not own:

- public opening capsule schema;
- attestation-contract representation policy;
- normalization operation;
- private-to-public backend pattern;
- transaction layout.

## 5.2 `tapscript`

`tapscript` owns prototype proof patterns:

- exact commitment equality where expressible;
- exact authenticated opening where expressible;
- value-domain checks;
- explicit asset/generator checks;
- capsule validation;
- target stack schedule;
- proof-local witness schema;
- resource projection.

Prototype types must remain visibly non-production.

A suitable shape is:

```rust
pub struct PublicOpeningPrototype {
    program: TapscriptProgram,
    witness: PublicOpeningWitnessSchema,
    resources: PrototypeResourceProjection,
}
```

No protocol operation ID enters this type.

## 5.3 `target-elements-conformance`

The conformance package owns:

- generic CT fixtures;
- deterministic public test openings;
- target executor materialization;
- native target verdicts;
- CT conservation reports;
- opening-pattern reports;
- capsule persistence tests;
- fresh-process reconstruction;
- exact claim coverage;
- validated prototype reports.

## 5.4 Potential generic dependency

The independent host oracle and transaction materializer may require a reviewed Elements/secp256k1-zkp library.

Candidate roles include:

- asset-generator derivation;
- Pedersen commitment construction;
- commitment parsing;
- rangeproof construction;
- surjection-proof construction;
- transaction blinding;
- confidential transaction serialization.

Before adoption, record:

- exact crate and version;
- licence;
- Rust 1.88 compatibility;
- enabled features;
- transitive dependencies;
- unsafe and FFI boundary;
- deterministic construction behavior;
- platform requirements;
- advisory state;
- lockfile impact;
- why the upstream Python framework alone is insufficient.

`target-elements` remains dependency-free regardless.

---

# 6. Target and commitment review · `sec:guide11:target-review`

## 6.1 Questions to answer from source

The source review must establish exact target facts for:

1. explicit and confidential value encodings;
2. explicit asset and confidential asset encodings;
3. the value-commitment point encoding;
4. the asset-generator relation;
5. whether spent-input value commitments are introspectable;
6. whether output value commitments are introspectable;
7. whether a target program can compare commitment bytes exactly;
8. whether a target program can verify scalar multiplication;
9. whether a target program can verify point addition or an equivalent tweak relation;
10. parity and x-only/compressed conversion;
11. scalar canonicality and overflow behavior;
12. whole-transaction value-conservation rules;
13. full-consumption blinding-balance rules;
14. proof requirements for confidential outputs;
15. whether a public opening can be authenticated without private target state.

Every accepted fact enters typed source. Review provenance remains in the human reference.

## 6.2 Commitment relation

The conceptual relation is:

\[
C=rG+vH_A
\]

where:

- \(C\) is the exact value commitment;
- \(v\) is the public semantic amount;
- \(r\) is the value blinding factor;
- \(G\) is the target’s blinding generator;
- \(H_A\) is the value generator for asset \(A\).

The exact target convention may differ in notation, sign, generator derivation, or encoding. Guide 11 must use the reviewed target relation, not assume this formula without verification.

## 6.3 Candidate target proof

One candidate construction, subject to source review, is:

1. authenticate explicit asset \(A\);
2. derive or witness the exact asset generator \(H_A\);
3. range-check \(v<2^{51}\);
4. witness \(V=vH_A\);
5. verify the scalar-multiplication relation for \(V\);
6. verify \(C=V+rG\) through an approved target relation;
7. reject malformed scalar, point, parity, or commitment encodings;
8. retain authenticated \(v\).

This candidate is accepted only if the target’s low-level primitives establish the complete point relation.

If `TweakVerify` operates over x-only keys in a way that loses required parity or changes the accepted relation, it must not be repurposed by analogy.

## 6.4 Commitment equality

Commitment equality may be required independently of opening.

Candidate mechanisms include:

- exact byte equality after canonical encoding;
- a target commitment-equality primitive;
- normalization into one canonical point representation followed by equality;
- a complete backend relation built from reviewed point operations.

Hash equality is accepted only if the collision assumption and exact preimage framing are appropriate for the claim. A hash of a commitment does not prove two differently encoded semantic commitments are the same unless canonical encoding is independently established.

## 6.5 Capability disposition

For each claim, record one of:

```text
PrimitiveReviewed
BackendPatternPrototypeAccepted
ExternalConsensusEvidenceRequired
Unsupported
```

Do not force every claim into `StaticCapabilityStatus`.

The existing target capability registry may continue to state:

```text
CommitmentEquality:
    Unsupported

AuthenticatedValueOpening:
    Unsupported
```

while `tapscript` carries an experimental backend pattern. Promotion occurs only after the ownership model is reviewed and the pattern passes every Guide-11 gate.

---

# 7. Independent commitment oracle · `sec:guide11:oracle`

## 7.1 Purpose

The independent oracle computes expected commitments and opening relations without calling the production tapscript pattern or adopting the native executor’s result.

It produces exact public vectors for:

- asset generator;
- amount commitment;
- zero blinding;
- nonzero blinding;
- zero, one, and maximum amount;
- both point parities;
- malformed or out-of-group scalar;
- wrong asset generator.

## 7.2 Inputs

```rust
pub struct PublicCommitmentVector {
    pub asset_id: [u8; 32],
    pub amount: u64,
    pub blinding_factor: [u8; 32],
    pub expected_generator: Vec<u8>,
    pub expected_commitment: Vec<u8>,
}
```

The actual type may use reviewed library types rather than byte vectors internally. Canonical fixture projections retain exact bytes.

## 7.3 Independence

Acceptable oracle sources include:

- a reviewed generic Elements library implementation;
- published target vectors;
- a small separately implemented formula checked against both.

The production target pattern must not call the oracle to decide acceptance.

The native executor must not derive expected values from what the node returned.

## 7.4 Cross-check

Where practical, require three-way equality:

```text
independent host oracle commitment bytes
=
target/library transaction commitment bytes
=
commitment bytes observed by target introspection
```

A disagreement fails the prototype and is triaged before changing expected results.

---

# 8. Confidential transaction fixture support · `sec:guide11:ct-fixtures`

## 8.1 Executor extension

The native executor must materialize generic transactions containing:

- explicit asset + explicit value;
- explicit asset + confidential value;
- confidential asset + explicit value, for target-only encoding evidence where permitted;
- confidential asset + confidential value, for target-only encoding evidence where permitted;
- confidential input values;
- confidential output values;
- deterministic test-only blinders;
- valid rangeproofs;
- valid surjection proofs;
- intentionally invalid proofs;
- explicit and confidential change;
- issuance only if needed by the selected target evidence claim.

No fixture names an attestation-contract object or asset.

## 8.2 Deterministic test randomness

All test randomness is explicit fixture input:

```text
asset blinding factor
value blinding factor
nonce seed
rangeproof seed
surjection-proof seed
ephemeral test key where the target library requires one
```

The fixture labels every value test-only.

Given identical explicit randomness and inputs, the transaction bytes must be identical.

Production randomness policy is out of scope.

## 8.3 Validity layers

The executor distinguishes:

```text
fixture construction failure

consensus confidential-transaction rejection

script rejection

relay-policy rejection

infrastructure failure
```

A rangeproof-construction failure is not a target script rejection.

A value-balance failure rejected before script execution is CT consensus evidence, not proof that the opening script ran.

The report records which layer supplied the verdict.

## 8.4 Conservation fixture matrix

Required complete-transaction fixtures:

| Fixture | Expected |
|---|---|
| explicit input → explicit output, balanced | accept |
| confidential input → confidential output, balanced | accept |
| confidential input → public-committed output, balanced | accept if candidate supports |
| confidential input → explicit output + private change | accept if blinders close |
| several confidential inputs → public-committed output | accept if blinders close |
| one-unit semantic imbalance | consensus reject |
| correct amounts, wrong blinding balance | consensus reject |
| malformed rangeproof | consensus reject |
| malformed surjection proof | consensus reject |
| wrong explicit asset generator | reject |
| output commitment copied from another asset | reject |
| artificial undeclared confidential output | candidate closure reject where claimed |

Whole-transaction conservation and opening-pattern verdicts remain distinct report claims.

---

# 9. Candidate A — Explicit-only boundary · `candidate:guide11:explicit-only`

## 9.1 Policy

Boundary operations requiring public arithmetic or permissionless future construction accept only values already explicit.

Conceptually:

```text
PrivateCommitted:
    lateral transfer only

PublicCommitted:
    unsupported or normalization-only

Explicit:
    accepted at public boundary
```

## 9.2 Strengths

- simplest target program;
- no opening proof;
- no public capsule;
- easiest auditability;
- no future opening availability problem;
- exact target arithmetic directly available.

## 9.3 Limitations

- private live values do not directly retain burn/redemption lifecycle;
- users need a prior normalization path or cannot select the private representation;
- explicit boundary transaction reveals the amount before or at the boundary;
- normalization costs an extra transaction if supported;
- full representation-liveness claim may fail.

## 9.4 Acceptance condition

Explicit-only is acceptable as the initial policy only if one of these is true:

1. private committed representations are not enabled for objects requiring the boundary; or
2. an owner-authorized normalization path exists; or
3. lifecycle incompleteness remains explicit and blocks release claims for the private representation.

Guide 11 must not say:

```text
private transfer supported
```

and then silently omit all private exits required by that representation’s lifecycle.

---

# 10. Candidate B — Owner-authorized normalization · `candidate:guide11:normalization`

## 10.1 Policy

An owner first performs a value-preserving representation transition:

```text
PrivateCommitted
    ↓ owner-authorized normalization
PublicCommitted or Explicit
```

The later boundary operation consumes the public representation.

Normalization changes only representation. It must preserve:

- semantic amount;
- owner;
- class;
- closed asset;
- protocol object kind;
- lifecycle identity;
- public protocol projection apart from the representation fact.

## 10.2 Why normalization may be easier

The owner already has the private opening and may provide any required private witness.

Normalization can be separated from:

- boundary arithmetic;
- permissionless maintenance;
- event production;
- state changes.

The output becomes publicly usable before the permissionless or formula-bound transition begins.

## 10.3 Normalization variants

### Private to public committed

```text
private commitment input
    →
commitment output with public amount and authenticated public opening
```

Strength:

- commitment algebra retained;
- residual blinding can remain on the public output;
- future constructor can verify.

Cost:

- opening capsule;
- opening verification;
- public disclosure;
- rangeproof/proof construction.

### Private to explicit plus private change

```text
private input
    →
explicit public output
+
optional private owner change
```

Strength:

- public output simple to consume.

Risk:

- full consumption may leave no confidential output to carry residual blinding;
- split correctness and output closure must be exact;
- private change remains owner-dependent and must not be confused with the normalized public object.

### Private to explicit through several inputs

Several private inputs may allow blinders to cancel into explicit output under a deterministic construction, but this is a target transaction fact to demonstrate, not a general assumption.

## 10.4 Normalization authorization

Normalization is owner-authorized.

Every consumed object owner must authorize the finalized output set.

No operator key.

No permissionless trigger.

The output owner is preserved unless ordinary transfer semantics explicitly permit a caller-selected authorized destination. Guide 11’s generic prototype should use owner preservation to avoid importing protocol transfer policy.

## 10.5 Normalization threat matrix

| Mutation | Required result |
|---|---|
| amount changed | reject |
| owner changed | reject in prototype |
| asset changed | reject |
| confidential closed asset substituted | reject |
| public opening copied from another output | reject |
| public opening malformed | reject |
| output commitment mismatches amount/opening | reject |
| residual blinding does not close | consensus reject |
| hidden private output absorbs value | closure reject |
| extra output changes after signing | signature or fixture rejection |
| representation changed without owner authorization | reject |
| output has no future public opening | lifecycle rejection |
| normalization capsule exists only in private test memory | fresh-process failure |

---

# 11. Candidate C — Direct public-committed boundary · `candidate:guide11:direct-opening`

## 11.1 Policy

The owner-authorized boundary directly consumes private committed value and creates a public-committed result:

```text
PrivateCommitted input
    ↓
public amount-dependent boundary
    ↓
PublicCommitted output or public result
```

The target relation authenticates the input amount during the same transition.

## 11.2 Required facts

The target must establish:

- exact input object and explicit closed asset;
- exact input value commitment;
- exact public amount;
- exact opening relation;
- amount domain;
- boundary semantic equation;
- output or event amount;
- complete value conservation;
- residual blinding closure;
- owner authorization;
- public evidence durability.

## 11.3 Direct path versus result publication

There are two separate cases:

```text
input opening needed only during the boundary
    e.g. formula-bound destruction with no public committed successor

public committed successor needed later
    e.g. fresh public object consumed by unrelated maintenance
```

The first may publish the amount in a boundary record without creating a public-committed spendable output.

The second requires durable output-specific evidence.

Guide 11 must not infer the second from the first.

## 11.4 Direct-path risks

- target proof may be too large;
- point/parity handling may be incomplete;
- full private consumption may fail blinding closure;
- capsule may be circular or swappable;
- owner signatures may not commit all public evidence;
- future constructor may require private data;
- complete transaction may exceed policy even if the opening fragment fits.

Direct support is rejected if any one of these remains unresolved.

---

# 12. Public opening capsule · `sec:guide11:capsule`

## 12.1 Required contents

A candidate capsule should contain enough public data to reconstruct and verify the opening without hidden state:

```text
domain separator
capsule schema
target contract version
prototype constructor schema
asset or generator identity
public amount
opening scalar or approved proof data
exact value commitment
output role or ordinal
output program or constructor commitment where required
```

Not every field must be duplicated if another authenticated relation provides it. Every omission needs an explicit source.

## 12.2 Candidate locations

| Location | Strength | Main concern |
|---|---|---|
| output constructor metadata | strongly binds object and opening | depends on accepted constructor; larger program |
| separate nonspendable data output | durable and indexable | must bind capsule to output instance |
| creating transaction witness | already public chain data | future availability and output binding |
| separate ordinary output | easy publication | spendability and ownership ambiguity |
| external index only | simple implementation | not trust-minimized; rejected for protocol constructibility |

An external index may cache capsules. It cannot be the only authoritative source.

## 12.3 Binding rule

The capsule must be bound to the exact target object it opens.

At minimum, swapping any of these must reject:

- output ordinal;
- value commitment;
- asset/generator;
- output program;
- constructor schema;
- capsule schema;
- public amount;
- opening data.

If two outputs carry identical commitments and programs, the design must say whether their capsules are intentionally interchangeable. If output identity matters, include a noncircular role or ordinal binding.

## 12.4 Canonical encoding

The capsule defines:

- field order;
- widths;
- byte order;
- scalar encoding;
- point encoding;
- domain string;
- schema number;
- absent-field rules;
- trailing-byte rejection;
- duplicate-field rejection.

Unknown schema and noncanonical encoding fail closed.

## 12.5 Signature commitment

Where owner authorization is required, the signature must commit to:

- the public amount;
- the exact capsule bytes or their authenticated constructor commitment;
- the output commitment;
- output program;
- output position/role;
- every protected recipient and change output.

A capsule appended after signing is not acceptable.

The exact signature behavior remains separate target/deployment evidence.

---

# 13. Fresh-process lifecycle test · `sec:guide11:fresh-process`

## 13.1 Purpose

The fresh-process test is the decisive constructibility witness.

It proves that “public” means:

```text
available from canonical public data to a party
who did not participate in creation
```

rather than:

```text
still present in the creator’s heap
```

## 13.2 Stage A — construction

Process A receives:

- private test opening;
- deterministic test randomness;
- reviewed target;
- public prototype configuration.

It constructs and confirms the private-to-public transaction.

It publishes only canonical chain data and the selected public capsule location.

## 13.3 Destruction boundary

Before process B starts, destroy:

- input opening held by A;
- private blinding values not intentionally public;
- temporary construction plans;
- unpublished fixture side files;
- process memory;
- owner-local helper state.

The test harness may retain the expected semantic result separately as oracle data, but process B must not receive it through the constructor interface.

## 13.4 Stage B — unrelated construction

Process B receives only:

- chain transaction and witness data;
- UTXO/outpoint;
- reviewed target;
- public constructor/prototype schema;
- public capsule;
- its own sponsor-local funds where needed.

It must:

1. locate the public evidence;
2. parse it canonically;
3. bind it to the intended output;
4. verify the commitment opening;
5. recover the public semantic amount;
6. construct and execute a future generic spend;
7. require no owner or operator secret.

## 13.5 Failures

The fresh-process test must reject:

- missing capsule;
- capsule in uncommitted local file only;
- copied capsule;
- stale capsule from another output;
- wrong amount;
- wrong opening;
- wrong asset;
- wrong constructor;
- malformed public data;
- unavailable witness;
- owner-private dependency.

---

# 14. Safety and minimality reports · `sec:guide11:reports`

## 14.1 Safety report

The safety report asks whether invalid candidate transitions are rejected.

Required classes include:

- wrong amount;
- wrong asset;
- wrong generator;
- wrong commitment;
- wrong blinding factor;
- malformed proof;
- copied capsule;
- hidden value output;
- wrong owner;
- missing authorization;
- incomplete public data;
- invalid CT balance;
- confidential closed asset.

A safety report does not claim that disclosure is minimal.

## 14.2 Minimality report

The minimality report asks whether a less-disclosing supported representation remains accepted with the same semantic projection.

Compare, where supported:

```text
Explicit
PublicCommitted
PrivateCommitted followed by normalization
PrivateCommitted direct boundary
```

For semantically equivalent candidates, compare:

- accepted/rejected verdict;
- semantic amount;
- owner;
- asset;
- object role;
- public operation projection;
- lifecycle result;
- disclosures;
- resources.

A green explicit path does not prove a public-committed path is supported.

A green public-committed path does not prove private direct support.

## 14.3 Disclosure provenance

Every newly public fact records a typed reason:

```rust
pub enum DeclassificationReason {
    PublicState,
    PublicEvent,
    PermissionlessConstructibility,
    BoundaryArithmetic,
    DeploymentPolicy,
}
```

Equivalent reuse of existing realization types is preferred.

Guide 11 must distinguish:

```text
semantic disclosure required

target-safety disclosure added by the selected proof

deployment-policy disclosure selected for initial simplicity
```

An explicit-only policy may introduce deployment-policy disclosure without claiming the semantic relation required explicit encoding.

---

# 15. Required vectors · `sec:guide11:vectors`

## 15.1 Opening relation

Fixed vectors:

- amount zero;
- amount one;
- amount \(2^{51}-1\);
- zero blinding factor;
- nonzero blinding factor;
- both admitted point parities;
- wrong amount by one;
- wrong blinding factor;
- wrong asset generator;
- wrong commitment;
- malformed compressed point;
- malformed x-only point where applicable;
- scalar at group order;
- scalar above group order;
- wrong scalar width;
- wrong byte order;
- alternate commitment encoding;
- trailing bytes.

## 15.2 Commitment equality

- identical canonical commitments;
- one-bit difference;
- same amount, different blinding factor;
- same blinding factor, different amount;
- same value inputs under different asset generator;
- noncanonical encoding of same apparent point;
- malformed point;
- copied commitment from another output.

## 15.3 Capsule

- canonical capsule;
- wrong schema;
- wrong domain;
- wrong target version;
- wrong constructor/prototype schema;
- wrong output ordinal;
- wrong output program;
- wrong asset/generator;
- wrong commitment;
- wrong amount;
- wrong opening;
- missing field;
- duplicate field;
- reordered field where order is canonical;
- trailing bytes;
- copied capsule;
- capsule mutation after signing;
- capsule available only in private fixture state.

## 15.4 Confidential transaction balance

- explicit balanced;
- confidential balanced;
- mixed explicit/confidential balanced;
- full private consumption;
- partial consumption with private change;
- several confidential inputs;
- one-unit amount imbalance;
- correct values, wrong blinder sum;
- malformed rangeproof;
- malformed surjection proof;
- wrong nonce;
- hidden extra output;
- asset mismatch;
- output value commitment copied from another transaction.

## 15.5 Normalization

- private to public committed;
- private to explicit where algebra permits;
- private to explicit plus private change;
- full consumption;
- partial consumption;
- owner preserved;
- owner mutation;
- amount mutation;
- output commitment mutation;
- opening omitted;
- opening copied;
- private output escape;
- confidential closed asset;
- output mutation after authorization.

## 15.6 Direct boundary

Use one generic amount-dependent transition, not an attestation-contract operation.

Required cases:

- correct private opening and public result;
- wrong public result;
- wrong input commitment;
- opening belongs to another input;
- public result shortened while another output grows;
- full-consumption blinding closure;
- private residual output omitted;
- public evidence unavailable to process B.

## 15.7 Fresh-process lifecycle

- public evidence recovered;
- future spend accepted;
- owner opening deleted;
- missing evidence rejected;
- copied evidence rejected;
- wrong constructor rejected;
- wrong chain context rejected;
- cache rebuilt from chain bytes;
- no hidden process-local dependency.

---

# 16. Resource evidence · `sec:guide11:resources`

Measure each candidate as a complete target transaction, not only as an isolated fragment.

## 16.1 Opening verifier

Record:

- target program bytes;
- witness amount bytes;
- opening bytes;
- point/generator bytes;
- stack and altstack;
- largest item;
- hash operations;
- curve/tweak operations;
- validation budget;
- transaction weight;
- policy verdict.

## 16.2 Capsule

Record:

- capsule bytes;
- constructor increase where metadata-bound;
- data-output increase where separately published;
- witness-history bytes where used;
- parsing and verification cost;
- fresh-process retrieval cost as noncanonical diagnostics.

## 16.3 Normalization

Measure:

- full private consumption;
- partial consumption with private change;
- several private inputs;
- public-committed result;
- explicit result where permitted;
- owner authorization;
- proof bytes;
- complete transaction weight and policy result.

## 16.4 Direct boundary

Measure:

- one private input;
- multiple private inputs;
- public output/result;
- complete opening proof;
- no private change;
- private change;
- maximum prototype amount;
- maximum public capsule candidate.

## 16.5 Decision table

The research result should include:

| Path | Transactions | Public amount timing | Additional proof bytes | Future permissionless use | Initial decision |
|---|---:|---|---:|---:|---|
| explicit-only | 0 extra if already explicit | already public | low | yes | |
| normalize to explicit | +1 | normalization | | yes | |
| normalize to public committed | +1 | normalization | | yes | |
| direct public committed | 0 extra | boundary | | yes if capsule passes | |

No architectural batch bound is calibrated from these figures.

---

# 17. Acceptance criteria · `sec:guide11:acceptance`

## 17.1 Direct public opening

Accept direct public opening only when:

- public amount binds the exact input commitment;
- asset/generator identity is exact;
- amount domain is exact;
- scalar/point encodings are canonical;
- wrong amount, blinder, generator, commitment, or parity rejects;
- whole-transaction balance passes separately;
- full-consumption residual blinding closes;
- capsule is canonical and output-bound where a future object needs it;
- owner authorization commits every protected output and public datum;
- fresh-process future construction succeeds;
- complete target-native transactions pass;
- abstract stack and native behavior agree;
- resource predictions agree with observations.

## 17.2 Normalization

Accept normalization only when:

- it is owner-authorized;
- amount, owner, asset, and object role are preserved;
- public representation is exactly authenticated;
- hidden private output escape is impossible;
- residual blinding closes;
- future public lifecycle works from fresh-process public data;
- explicit and normalized semantic projections agree;
- complete target-native transactions pass.

## 17.3 Explicit-only policy

Accept explicit-only as the initial boundary policy only when:

- direct and normalization candidates are rejected or deliberately deferred with reasons;
- enabled private representations carry explicit lifecycle incompleteness;
- no package claims direct private boundary support;
- clients can determine before construction that explicit representation is required;
- the policy is typed as deployment/backend policy rather than semantic necessity;
- safety evidence for the explicit path passes.

## 17.4 Evidence acceptance

Every accepted result requires:

- exact fixture census;
- exact claim census;
- validated report wrapper;
- observed chain binding;
- complete executor provenance;
- zero required failures;
- zero required infrastructure errors;
- deterministic report bytes;
- no missing required subclaim hidden by a broad passing row.

---

# 18. Rejection criteria · `sec:guide11:rejection`

Reject a direct or normalized candidate if:

- a false opening passes;
- commitment bytes can be swapped across assets;
- capsule can be copied to another output;
- public amount is unauthenticated metadata;
- full consumption cannot close blinding;
- a future constructor needs owner-private state;
- confidential closed asset identity escapes;
- target stack behavior differs from its contract;
- an arithmetic or curve success flag is unchecked;
- accepted transaction relies on a mock;
- construction failure is counted as target rejection;
- target resources exceed a hard limit;
- tested and proposed production bytes differ;
- expected commitment bytes are generated by the same code under test;
- only an off-chain library checks the opening.

Reject the Guide-11 gate if:

- unsupported target capabilities are relabeled complete without a full pattern;
- a broad evidence row passes without every required claim;
- report rows omit fixture subjects;
- network/genesis are caller labels rather than observations;
- public data exists only in process-local memory;
- explicit-only policy is described as disclosure-minimal;
- normalization success is described as direct support;
- direct support is described as a production operation.

---

# 19. Suggested implementation waves · `sec:guide11:waves`

## Wave 0 — Fix any inherited evidence or target-boundary blockers

Deliver:

- remaining Guide-10 evidence fixes;
- exact reviewed target/development binding;
- observed chain identity;
- exact report subject and claim census;
- complete process cleanup;
- signature abstraction repair if authorization fixtures need it.

Suggested commit:

```text
target-conformance: close the declassification evidence preconditions
```

## Wave 1 — Review commitment and CT target facts

Deliver:

- value commitment format;
- asset-generator relation;
- explicit/confidential introspection forms;
- commitment equality feasibility;
- authenticated-opening feasibility;
- CT conservation evidence boundary;
- exact capability ownership ruling.

Suggested commit:

```text
target-elements: review the confidential value boundary
```

## Wave 2 — Extend the native executor to generic CT transactions

Deliver:

- deterministic public test blinders;
- confidential input/output materialization;
- rangeproof and surjection-proof fixtures;
- balance and proof failures;
- exact observed chain binding;
- no credential interface.

Suggested commit:

```text
target-conformance: materialize generic confidential transactions
```

## Wave 3 — Independent commitment oracle

Deliver:

- public commitment vectors;
- asset-generator vectors;
- amount/blinding boundaries;
- three-way host/library/native comparison;
- malformed point and scalar vectors.

Suggested commit:

```text
target-conformance: add the independent opening oracle
```

## Wave 4 — Commitment equality and opening prototypes

Deliver:

- readable target proof candidates;
- exact stack schedules;
- immediate success-flag checks;
- wrong amount/blinder/generator/commitment mutations;
- static/native equality;
- resource projections.

Suggested commit:

```text
tapscript: prototype authenticated public openings
```

## Wave 5 — Public capsule prototypes

Deliver:

- metadata, data-output, and witness-history candidates as applicable;
- canonical schema;
- noncircular output binding;
- swap/mutation vectors;
- signature-commitment review;
- resource comparison.

Suggested commit:

```text
tapscript: prototype durable public opening capsules
```

## Wave 6 — Normalization and direct-boundary transactions

Deliver:

- private-to-public-committed normalization;
- explicit normalization where algebra permits;
- direct private-to-public candidate;
- full/partial consumption;
- residual blinding closure;
- safety report;
- minimality report.

Suggested commit:

```text
target-conformance: compare declassification paths
```

## Wave 7 — Fresh-process lifecycle

Deliver:

- process A construction;
- destruction of private state;
- process B public reconstruction;
- future generic target spend;
- missing/copied/stale capsule failures.

Suggested commit:

```text
target-conformance: prove public lifecycle availability
```

## Wave 8 — Decision and Phase-3 handoff

Deliver:

- accepted initial matrix;
- updated public-declassification research;
- package documentation;
- Phase-3 card;
- backlog gate;
- identity and dependency impact;
- complete repository gate.

Suggested commit:

```text
plans: record the Guide-11 declassification decision
```

Commit each coherent green wave promptly.

---

# 20. Focused verification · `sec:guide11:verification`

## 20.1 Static target

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

Focused areas:

```text
commitment and value encodings
capability status
opening/equality ownership
confidential-value welds
evidence requirements
new primitive contracts
reviewed-target trust state
```

## 20.2 Tapscript prototypes

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

Focused areas:

```text
opening stack relation
commitment equality
amount-domain checks
generator binding
capsule parsing and binding
normalization prototype
direct boundary prototype
abstract execution oracle
resource projection
prototype/release type separation
```

## 20.3 Native conformance

```sh
cargo test --locked -p tripod-target-elements-conformance
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements-conformance --no-deps
```

Focused areas:

```text
confidential transaction materialization
deterministic test blinders
rangeproof and surjection-proof failures
whole-transaction CT balance
complete fixture projection
claim-level evidence coverage
fresh-process reconstruction
validated report wrapper
report determinism
executor process cleanup
```

## 20.4 Existing semantic owners

If the accepted result changes typed representation alternatives or lifecycle claims:

```sh
cargo test --locked -p tripod-realization
cargo test --locked -p tripod-compiler
```

No realization/compiler change should land until the target research result is accepted.

## 20.5 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new file enters the nearest Meson census in the same commit.

---

# 21. Full batch gate · `gate:guide11:batch`

After all coherent waves:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run the complete real native matrices separately:

```text
commitment and generator vectors

confidential transaction balance vectors

opening proof vectors

capsule binding vectors

normalization vectors

direct-boundary vectors

fresh-process lifecycle vectors
```

Run:

```sh
cargo audit
```

when available. If unavailable, record it as skipped.

Run document reproducibility when required by changed document inputs or repository batch policy. A deferred run is recorded as deferred, not passed.

Final check:

```sh
git status --porcelain=v1 --untracked-files=all
```

The result must be empty.

---

# 22. Identity and dependency impact · `sec:guide11:impact`

Expected identity impact:

```text
Attestation version:
    unchanged

realization version:
    unchanged

realization letter:
    unchanged unless normative realization text changes

architecture schema:
    unchanged

architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

generated architecture publications:
    unchanged

realization identity:
    none minted

compiler identity:
    none minted

target digest:
    none minted

opening-pattern digest:
    none minted

capsule digest:
    none minted

native report digest:
    none minted

deployment-profile identity:
    dormant and not release-valid
```

The target contract version may require a deliberate bump if the accepted target contract changes meaning. Merely adding research prototype code in `tapscript` does not move the static target contract.

Expected dependency impact:

```text
target-elements:
    none

tapscript:
    no new dependency expected

target-elements-conformance:
    likely reviewed Elements/secp256k1-zkp transaction and
    commitment-construction dependency, if the external framework
    cannot supply an adequate independent oracle
```

Any lockfile change receives explicit dependency review.

---

# 23. Final policy matrix · `tbl:guide11:result-matrix`

The Guide-11 result must fill this table with one of:

```text
supported directly
supported through normalization
explicit only
unsupported
deferred with named blocker
```

| Object/use | Explicit | Public committed | Private direct | Normalize first |
|---|---|---|---|---|
| lateral live-value transfer | | | | n/a |
| public ownerless maintenance object | required or accepted | | n/a | n/a |
| owner-authorized amount-dependent boundary | | | | |
| permissionless future maintenance | | | forbidden | n/a |
| formula-bound payout boundary | | | | |
| full private input consumption | | | | |
| partial private consumption with private change | | | | |

The operation-specific names remain owned by later phases. Guide 11 fills a representation-policy matrix, not an operation implementation matrix.

---

# 24. Guide-11 exit checklist · `gate:guide11:exit`

Guide 11 exits only when all applicable items hold.

## Target and evidence foundation

- [ ] exact commitment format reviewed;
- [ ] exact asset-generator relation reviewed;
- [ ] explicit and confidential value forms reviewed;
- [ ] whole-transaction CT conservation remains separately evidenced;
- [ ] commitment equality disposition is explicit;
- [ ] authenticated opening disposition is explicit;
- [ ] low-level primitives are not mistaken for a complete pattern;
- [ ] target capability and backend pattern ownership are distinct;
- [ ] validated reports bind complete fixtures and claim censuses;
- [ ] actual chain identity is observed and matches the binding;
- [ ] executor provenance and cleanup satisfy the inherited Guide-10 boundary.

## Independent oracle

- [ ] commitment oracle is independent of the production target pattern;
- [ ] published or independently computed vectors exist;
- [ ] asset-generator vectors exist;
- [ ] amount and blinder boundary vectors exist;
- [ ] host/library/native comparison passes;
- [ ] target observations never generate their own expectations.

## Confidential transaction execution

- [ ] explicit balanced transaction passes;
- [ ] confidential balanced transaction passes;
- [ ] mixed representation transaction passes where claimed;
- [ ] one-unit imbalance rejects at consensus;
- [ ] wrong blinding balance rejects;
- [ ] malformed rangeproof rejects;
- [ ] malformed surjection proof rejects;
- [ ] fixture construction failure is distinct from target rejection;
- [ ] test randomness is explicit and reproducible.

## Opening pattern

- [ ] public amount binds exact commitment;
- [ ] exact asset/generator binds the relation;
- [ ] amount domain is enforced;
- [ ] malformed scalar and point encodings reject;
- [ ] wrong amount rejects;
- [ ] wrong blinder rejects;
- [ ] wrong generator rejects;
- [ ] copied commitment/opening rejects;
- [ ] every target success flag is checked;
- [ ] exact authenticated amount remains available to the caller;
- [ ] abstract and native stack behavior agree.

## Public capsule

- [ ] canonical schema exists;
- [ ] no circular transaction-ID binding exists;
- [ ] capsule binds the exact output/object role;
- [ ] capsule swap rejects;
- [ ] malformed and unknown schema reject;
- [ ] trailing and duplicate fields reject;
- [ ] authorization commits public evidence where required;
- [ ] capsule is available from canonical public data;
- [ ] process B reconstructs the future spend without owner-private state.

## Normalization and direct paths

- [ ] normalization preserves amount, owner, asset, and object role;
- [ ] normalization is owner-authorized;
- [ ] full-consumption blinding closes;
- [ ] partial-consumption private change closes;
- [ ] hidden confidential output escape rejects;
- [ ] direct path is accepted only if complete;
- [ ] normalization success is not reported as direct support;
- [ ] explicit-only policy remains available as an honest result;
- [ ] unsupported private lifecycles remain explicit.

## Reports and decisions

- [ ] safety and minimality reports are separate;
- [ ] every newly public fact has a typed reason;
- [ ] exact fixture and claim censuses pass;
- [ ] no required case fails;
- [ ] no required case reports infrastructure trouble;
- [ ] report bytes reproduce;
- [ ] public-declassification research records the result;
- [ ] D005 is updated only if the accepted policy requires it;
- [ ] Phase-3 card records the accepted matrix;
- [ ] no operation, ABI, bundle, calibration, or release is overclaimed;
- [ ] no speculative identity is minted;
- [ ] full repository gates pass;
- [ ] final tree is clean.

---

# 25. Completion report template · `sec:guide11:report-template`

```text
Guide 11 result
===============

Starting state:
    source revision:
    Guide-10 gate:
    public-declassification research revision:
    clean tree:

Inherited evidence boundary:
    validated report wrapper:
    exact fixture binding:
    exact case census:
    exact evidence-row census:
    claim-level coverage:
    observed network/genesis:
    target/deployment exact binding:
    executor provenance:
    process cleanup:
    protocol limits:

Target review:
    value commitment encoding:
    asset generator:
    input commitment availability:
    output commitment availability:
    commitment equality:
    authenticated opening:
    whole-transaction CT conservation:
    required new primitives:
    target-contract version impact:

Independent oracle:
    implementation:
    dependency:
    published vectors:
    generator vectors:
    commitment vectors:
    host/library/native agreement:

CT executor:
    explicit transaction:
    confidential transaction:
    mixed transaction:
    deterministic test randomness:
    rangeproof:
    surjection proof:
    one-unit imbalance:
    wrong blinding balance:
    infrastructure/consensus distinction:

Opening prototype:
    candidate:
    amount domain:
    asset/generator binding:
    scalar relation:
    point relation:
    success-flag handling:
    exact stack contract:
    positive vectors:
    negative vectors:
    target-native result:
    resources:
    disposition:
        accepted / unsupported / deferred

Capsule:
    selected location:
    schema:
    domain separator:
    output binding:
    noncircular identity:
    signature commitment:
    swap mutations:
    fresh-process availability:
    resources:

Normalization:
    private → public committed:
    private → explicit:
    private → explicit + private change:
    full consumption:
    partial consumption:
    owner authorization:
    semantic projection equality:
    resources:
    disposition:

Direct boundary:
    private input opening:
    public result:
    residual blinding:
    owner authorization:
    future public use:
    target-native result:
    disposition:

Final representation policy:
    lateral transfer:
    public maintenance:
    owner-authorized boundary:
    permissionless future use:
    formula-bound payout:
    direct private support:
    normalization:
    explicit-only fallback:

Reports:
    safety cases:
    safety passed:
    safety failed:
    minimality cases:
    minimality passed:
    minimality failed:
    infrastructure errors:
    unresolved claims:
    deterministic bytes:

Identity impact:
    Attestation:
    realization:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    target contract version:
    opening/capsule/report identities:
        none
    deployment identity:
        none

Dependency impact:
    target-elements:
    tapscript:
    target-elements-conformance:
    Cargo.lock:
    licence:
    MSRV:
    unsafe/FFI:
    deterministic construction:
    advisories:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
    target-elements:
    tapscript:
    target-elements-conformance:
    realization, if changed:
    compiler, if changed:
    Rustdoc:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    real CT matrix:
    real opening matrix:
    real normalization matrix:
    fresh-process lifecycle:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Planning handoff:
    public-declassification research:
    D005:
    Phase 3:
    next guide:

Residuals:
```

---

# 26. What follows Guide 11 · `sec:guide11:next`

Guide 11 completes the last of the three Phase-3 prototype questions only if:

- Guide 10 accepted or explicitly rejected STATE constructor continuity;
- Guide 10 accepted or explicitly rejected exact wide-floor arithmetic;
- Guide 11 selects an initial public-declassification policy or records explicit target rejection.

If those conditions hold and the Phase-3 exit gate passes, the next guide should begin the first complete compiler-to-target protocol operation:

```text
Guide 12 — End-to-End Compact ASH
```

Guide 12 should consume, not reopen:

- the reviewed target contract;
- typed instruction core;
- exact native-evidence framework;
- constructor result where relevant;
- wide-arithmetic result where relevant;
- public-declassification policy.

Compact ASH remains the first operation because it needs neither STATE continuity nor wide floor arithmetic, but it does need the final public-value and sponsor-opacity policy.

Guide 12 may then introduce, for one operation only:

- compiler target-plan consumption;
- backend proof patterns;
- concrete relation placement;
- target layout;
- relocatable artifacts;
- linker candidate;
- transaction ABI;
- complete target transactions;
- relation-indexed positive and negative evidence.

Guide 11 itself introduces none of those.

---

## Closing statement · `rem:guide11:closing`

> The public side of a confidential-value boundary is valid only when its amount is exactly authenticated, its asset identity remains rigid, its commitment algebra closes, its evidence is canonically bound to the intended object, and an unrelated future constructor can recover and verify it from public data alone. If the target cannot establish all five, the correct result is normalization, explicit-only use, or explicit rejection—not an optimistic representation claim.
