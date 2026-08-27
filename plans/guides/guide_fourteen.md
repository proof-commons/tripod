# Draft: Guide 14 — End-to-End STATE Constructor and Maturity Announcement

> **Status:** Draft execution guide; not yet executed
> **Phase:** Phase 6 — STATE Constructor and Maturity Announcement
> **Entry:** the amended Phase-5 exit gate, (`gate:phase5:exit`), the accepted STATE-constructor prototype, (`sec:state-constructor:result`), and the Guide-14 preflight gate below
> **Primary typed operation:** `architecture::OperationId::AnnounceMaturity` or the exact existing architecture spelling discovered during Wave 0
> **Human operation name:** `announce-maturity`
> **Affected packages:** `architecture` only if an existing omission is confirmed; otherwise `realization`, `compiler`, `target-elements`, `tapscript`, `linker`, `transaction`, `vectors`, and `target-elements-conformance` only for target-generic executor, constructor-oracle, and public-test-signing mechanics
> **Supersedes as execution direction:** the Guide-14 concept draft
> **Does not implement:** burn, clear, redemption, request admission, settlement, cycle, RESV succession, PACE succession, authority succession, production wallets, production operator-key custody, production deployment, final calibration, or release
> **Required result:** one complete candidate compiler-to-target maturity-announcement pipeline that authenticates the current canonical STATE root, derives one announced successor with only maturity changed, preserves one exact static constructor relation, obtains exact operator authorization over finalized bytes, executes against a real Elements target, reconstructs the successor from public chain data, records one exact root-history edge, and keeps safety, constructor continuity, root history, public recovery, resources, lifecycle, and production non-claims distinct
> **Authority:** Attestation, Realization, typed architecture, implemented ADRs, accepted decisions, the accepted STATE-constructor research result, and the recorded Phase-5 result take precedence
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015
> **Initial review basis:** the supplied 51-file static-review selection, archived as review 10; every review row below is a hypothesis until reproduced against the complete working tree

---

## Mission · `sec:guide14-exec:mission`

Guide 14 takes the accepted metadata-dependent constructor prototype and promotes only the mechanisms required for the first real mutable-root operation:

```text
typed architecture
    ↓
target-independent maturity-announcement semantics
    ↓
validated compiler target-operation plan
    ↓
reviewed Elements capability assessment
    ↓
candidate STATE constructor
    ↓
maturity-announcement target programs
    ↓
deterministic linking
    ↓
candidate transaction and witness ABI
    ↓
operator authorization over finalized bytes
    ↓
real target execution
    ↓
independent STATE projection
    +
constructor-continuity projection
    +
root-history projection
    +
public successor reconstruction
```

The operation consumes one canonical STATE object and creates one canonical successor:

```text
STATE {
    maturity = Unannounced,
    other fields = F
}
    ↓ announce-maturity(a)
STATE {
    maturity = Announced { cycle = a },
    other fields = F
}
```

The implementation succeeds only when all of these agree:

\[\text{target accepted}\land\text{semantic STATE matched}\land\text{constructor continuity matched}\land\text{root edge matched}\land\text{public recovery matched}\]

A valid taproot output, a valid operator signature, a final root cursor, or an accepted target transaction is only one conjunct. None substitutes for the others.

---

## One-line thesis · `rem:guide14-exec:thesis`

> Guide 14 succeeds when one current, canonical, unannounced STATE root is authenticated and advanced to one publicly reconstructible announced successor under the same exact static constructor relation, with every unaffected semantic field preserved, one operator authorizing one finalized transaction, no RESV or economic flow introduced, and every evidence and production boundary stated no more strongly than the observations support.

---

# 1. Governing rulings · `sec:guide14-exec:rulings`

## 1.1 Typed Rust remains the semantic source · `rule:guide14-exec:typed-source`

The implementation follows D001.

The complete semantic STATE schema and maturity relation come from typed Rust values. No first-party semantic path parses:

- the Guide-14 concept or this guide;
- Realization Markdown;
- Attestation LaTeX;
- model source text;
- planning tables;
- target disassembly;
- generated JSON;
- a previous report.

Generated documentation and reports are one-way publications.

The backend may define a canonical byte encoding of typed STATE metadata. That encoding is a target representation of the typed source, not another semantic source.

## 1.2 Consume the Phase-5 pipeline, but revalidate it first · `rule:guide14-exec:consume-phase5`

Guide 14 inherits, rather than replaces:

- validated compiler target-operation plans;
- target-independent proof, placement, layout, and coverage analysis;
- typed target capability assessment;
- typed tapscript instruction and abstract-execution machinery;
- deterministic taptree construction;
- typed linker symbols, definitions, references, and relocations;
- candidate bundle and ABI status;
- finalized-transaction owner signing;
- target-generic sponsor signing;
- strict native-executor framing and response-shape validation;
- target and deployment binding;
- transcript-derived observations;
- separate target acceptance and semantic comparison;
- distinct safety, minimality, resource, and lifecycle evidence.

Inheritance is conditional on Wave 0 reproducing and resolving the preflight rows in §4. A Phase-5 record being historical evidence does not make every current helper safe for a new mutable-root gate.

A Phase-5 interface changes only where STATE presents a concrete fact it cannot represent. Every generalization records:

1. the missing fact;
2. the existing type that cannot carry it;
3. the smallest sufficient extension;
4. compact-ASH and live-transfer regression impact;
5. identity and schema impact;
6. evidence impact;
7. dependency impact;
8. focused positive and negative tests.

## 1.3 STATE is a canonical root, not an output class · `rule:guide14-exec:state-root`

An output carrying STATE-shaped metadata is not thereby the canonical STATE successor.

An accepted announcement establishes all of:

```text
predecessor:
    exactly one current canonical STATE root

successor:
    exactly one canonical STATE root

edge:
    STATE predecessor → STATE successor

other STATE-like inputs or outputs:
    absent

other root families:
    absent
```

The root-history evidence must reject:

- stale predecessors;
- duplicate predecessors;
- duplicate successors;
- missing successors;
- intermediate invalid edges hidden by a later cursor restoration;
- STATE termination;
- RESV, PACE, entitlement-authority, or distribution-authority participation;
- old-bundle to new-bundle continuation without an explicit migration relation.

## 1.4 One semantic field changes · `rule:guide14-exec:single-field-change`

Let \(S\) be predecessor STATE, \(S'\) successor STATE, and \(a\) the requested announcement cycle.

The operation requires:

\[\operatorname{maturity}(S)=\operatorname{Unannounced}\]

\[\operatorname{maturity}(S')=\operatorname{Announced}(a)\]

For every other semantic field \(f\):

\[f(S')=f(S)\]

The complete set of fields comes from the typed STATE definition. The backend does not maintain a second list of “fields to copy.”

A transaction that announces maturity correctly while changing any other semantic field is invalid.

## 1.5 Semantic metadata and constructor representation are separate · `rule:guide14-exec:metadata-separation`

STATE metadata may require a public representation nonce or equivalent constructor-only field.

The two classes remain distinct:

```text
semantic fields:
    protocol meaning

representation fields:
    deterministic target spelling
```

Semantic projection erases representation-only fields.

Constructor-continuity evidence retains them because target bytes depend on them.

Two constructor encodings differing only in an admitted representation nonce may project to the same semantic STATE, but only the first admissible nonce is canonical.

## 1.6 The accepted constructor is promoted narrowly · `rule:guide14-exec:prototype-promotion`

The accepted prototype selected:

```text
dynamic unspendable metadata leaf
+
linked static operation subtree
+
public deterministic internal key
+
canonical branch-side policy
+
deterministic representation-nonce retry
```

Guide 14 does not promote the prototype’s synthetic 48-byte counter schema as STATE metadata.

Each prototype mechanism receives an explicit promotion disposition during Wave 0:

| Prototype mechanism | Initial disposition |
|---|---|
| dynamic metadata leaf | candidate for promotion |
| metadata leaf that always aborts | candidate for promotion |
| static code subtree | candidate for promotion |
| fixed branch side | candidate for promotion |
| deterministic nonce retry | candidate for promotion |
| tweak-totality classification | candidate for promotion |
| public NUMS internal key | inherited with residual assumption |
| synthetic counter field | reject as STATE semantics |
| prototype schema widths and offsets | reject unless independently justified by typed STATE |
| prototype native report | historical evidence only |

No prototype helper silently becomes production-pattern authority.

## 1.7 Constructor continuity is bidirectional · `rule:guide14-exec:constructor-continuity`

The transaction establishes both:

```text
consumed input program
    =
constructor(predecessor semantic metadata,
            predecessor representation nonce,
            linked static subtree,
            target policy)

created output program
    =
constructor(successor semantic metadata,
            successor representation nonce,
            same linked static subtree,
            same target policy)
```

A caller-supplied output program is not authority.

Using the same internal key is insufficient.

Using a valid announcement leaf is insufficient.

Using an output program from another metadata value is insufficient.

Static-subtree continuity and metadata continuity are separate checks and separate evidence roles.

## 1.8 The successor is publicly recoverable from the announcement transaction · `rule:guide14-exec:public-recovery`

The canonical publication source for the initial candidate is the accepted announcement transaction’s public script-path witness and output set.

The announcement witness carries enough public data to recover:

- predecessor metadata;
- requested announcement cycle;
- successor semantic metadata by typed transition;
- successor representation nonce;
- metadata schema;
- static constructor recipe or exact linked static-root reference;
- successor output position;
- target leaf version and internal-key policy.

An unrelated process must be able to:

1. locate the accepted transaction;
2. verify its exact bytes and deployment binding;
3. decode the public witness;
4. derive successor semantic metadata;
5. reconstruct canonical successor constructor bytes;
6. compare them with the actual STATE successor output.

No additional OP_RETURN-like publication is added unless Wave 0 proves the witness cannot supply this information. A second publication would be duplicate authority and requires an explicit decision.

## 1.9 One operator authorizes one finalized transaction · `rule:guide14-exec:operator-authorization`

The operator signs only after all protected fields are fixed:

- predecessor STATE outpoint;
- predecessor spent-output fields;
- successor STATE metadata;
- successor representation nonce;
- successor output program;
- transaction input and output counts;
- optional sponsor inputs where covered;
- sponsor change;
- fee role;
- transaction version;
- lock time;
- issuance absence;
- output-witness fields covered by the selected message;
- executing leaf and script-path terms.

A signature over one candidate cannot authorize another.

Unknown nonempty key encodings that the target treats as forward-compatible success must not count as authorization. The operator key is committed in an approved encoding and the signature must actually verify.

## 1.10 Operator semantics and target-generic signing stay separate · `rule:guide14-exec:signer-ownership`

The vectors layer owns the statement:

```text
this public signer is the authorized maturity operator
```

The executor may own only target-generic mechanics:

```text
declare a public disposable test signer
fund an output at a caller-supplied program
authorize one input of exact finalized bytes
return the exact witness and byte binding
submit the complete transaction
```

The executor receives no maturity expectation and no semantic verdict.

No first-party interface accepts a production operator private key.

## 1.11 Announcement lead is exact · `rule:guide14-exec:announcement-window`

Let:

- \(c\) be predecessor STATE’s current cycle;
- \(a\) be the requested announcement cycle;
- \(L_{\min}\) be the architecture-owned minimum lead;
- \(L_{\max}\) be the architecture-owned maximum lead.

The request is valid exactly when:

\[c+L_{\min}\le a\le c+L_{\max}\]

Every addition is checked in the architecture-owned cycle domain.

Overflow is a semantic or construction refusal. It is not wrapping, saturation, clamping, or a target rejection attributed to a later clause.

The backend consumes typed bound references. It does not copy the numeric constants.

## 1.12 No economic flow is introduced · `rule:guide14-exec:no-economic-flow`

Maturity announcement advances STATE and moves no protocol value family.

It consumes or creates no:

- live receipt;
- time-locked receipt;
- ASH;
- entitlement;
- distribution control;
- distribution vault;
- request;
- burn record;
- closed-asset issuance;
- closed-asset destruction.

The singleton STATE asset is recreated through root succession. That is a root relation, not a general value-flow relation.

Whole-transaction target conservation remains external target evidence and cannot substitute for STATE succession.

## 1.13 Sponsor opacity survives STATE mutation · `rule:guide14-exec:sponsor-opacity`

Optional sponsorship remains isolated from STATE semantics.

No semantic relation, compiler plan, target program, canonical report, or future identity requires an individual sponsor amount to be:

- decoded for protocol use;
- explicit for protocol use;
- opened;
- compared with zero;
- proved positive;
- included in a public subtotal;
- emitted in a canonical report;
- emitted in ordinary diagnostics.

Sponsor safety derives from:

```text
exact sponsor-region membership
+
STATE/sponsor disjointness
+
sponsor-owner target authorization
+
at most one sponsor envelope
+
whole-transaction target conservation
+
independent STATE-succession enforcement
```

A balanced corruption that adjusts sponsor change while modifying STATE must still fail for the STATE relation.

## 1.14 Evidence classes remain separate · `rule:guide14-exec:evidence-separation`

Guide 14 maintains distinct results for:

1. semantic maturity safety;
2. operator authorization;
3. predecessor constructor authentication;
4. successor constructor reconstruction;
5. static-subtree continuity;
6. root-history continuity;
7. public successor recovery;
8. target execution;
9. sponsor isolation;
10. resource prediction and observation;
11. lifecycle incompleteness.

No one report silently satisfies another.

In particular:

```text
target accepted
≠
semantic STATE matched

semantic STATE matched
≠
constructor continuity matched

final root cursor matched
≠
every intermediate root edge valid

operator signature verified
≠
operator role was correct

constructor is reproducible
≠
constructor is release-final
```

## 1.15 Expected and observed boundaries must agree · `rule:guide14-exec:boundary-equality`

A negative row is answered by a target refusal only when:

- its control was accepted;
- its mutation was attributable;
- its intended carrier was reached;
- the observed target layer equals the row’s declared evidence boundary;
- the target’s result is bound to the exact submitted bytes.

Formally:

\[\operatorname{ObservedBoundary}(r)=\operatorname{DeclaredBoundary}(r)\]

A consensus-before-script refusal does not answer a script-path row.

A key-path refusal does not answer a script-path row.

An infrastructure failure answers no target row.

A refusal at an earlier unexpected layer is a finding and an unanswered row until the row is authoritatively retyped or a new candidate reaches the intended layer.

## 1.16 Candidate and final states remain distinct · `rule:guide14-exec:candidate-only`

Guide 14 may produce:

```text
ValidatedMaturityAnnouncementOperationPlan
CandidateStateMetadataEncoding
CandidateStateConstructor
CandidateMaturityTapscriptPlan
CandidateRelocatableMaturityBundle
CandidateLinkedMaturityBundle
CandidateMaturityAnnouncementAbi
CandidateMaturitySafetyReport
CandidateConstructorContinuityReport
CandidateRootHistoryReport
CandidatePublicRecoveryReport
CandidateMaturityResourceReport
```

It must not produce or imply:

```text
FinalStateConstructor
FinalLinkedBundle
FinalTransactionAbi
ProductionOperatorService
ProductionStateDeployment
ValidatedDeploymentRelease
```

The candidate remains lifecycle-incomplete because later STATE operations are absent.

## 1.17 No speculative identity · `rule:guide14-exec:no-speculative-identity`

Typed in-process values use exact typed equality.

Exact bytes use exact byte comparison where the bytes themselves are the subject.

No digest is minted for:

- STATE metadata;
- constructor;
- maturity plan;
- linked bundle;
- ABI;
- safety report;
- continuity report;
- root-history report;
- public-recovery report.

A digest enters only after the ADR-021 identity-admission procedure identifies a present consumer and a distinct decision.

No type reserves a future digest field.

## 1.18 Public disposable material only · `rule:guide14-exec:test-material`

Tests may use public cryptographic material under ADR-015’s disposable-test-material rule:

- published operator scalar;
- deterministic signing auxiliary input;
- public constructor nonce;
- public synthetic STATE fixture;
- disposable node cookies confined to disposable state.

Such values:

- authorize nothing on a production network;
- are committed source or deterministic disposable state;
- are clearly test-only;
- are never derived from production material;
- are destroyed or discardable with the test environment.

A legitimate production-secret input requires a separate reviewed security design. It does not enter through Guide 14.

---

# 2. Exact scope · `sec:guide14-exec:scope`

## 2.1 Implemented operation · `def:guide14-exec:operation`

The sole semantic operation is:

```text
announce-maturity
```

The exact architecture identifier is discovered from typed architecture during Wave 0. If the architecture already has an identifier, this guide reuses it. It does not mint a lookalike.

## 2.2 Predecessor · `def:guide14-exec:predecessor`

The operation consumes exactly one canonical STATE object with:

```text
asset:
    architecture-owned singleton STATE asset

amount:
    exact singleton amount

metadata:
    complete canonical STATE metadata

maturity:
    Unannounced

root role:
    current canonical STATE root
```

It consumes no RESV or other protocol root.

## 2.3 Request · `def:guide14-exec:request`

The semantic request selects:

```text
announced_cycle: Cycle
```

It may additionally select whether optional sponsorship is requested through the inherited typed sponsor interface.

It does not select:

- predecessor STATE;
- predecessor metadata;
- successor metadata;
- successor program;
- static subtree;
- internal key;
- operator identity;
- transaction positions;
- root cursor;
- control block;
- representation nonce result;
- fee role;
- target verdict.

## 2.4 Successor · `def:guide14-exec:successor`

The operation creates exactly one canonical STATE successor:

```text
maturity:
    Announced { cycle = announced_cycle }

all other semantic fields:
    copied exactly from predecessor

root role:
    new canonical STATE root
```

No STATE termination is permitted.

## 2.5 Root projection · `rule:guide14-exec:root-projection`

The exact root effect is:

```text
STATE:
    Succ {
        predecessor = consumed current STATE,
        successor = created announced STATE
    }

RESV:
    absent

PACE:
    absent

ENT_AUTH:
    absent

DIST_AUTH:
    absent
```

## 2.6 Required semantic projection · `rule:guide14-exec:projection`

The operation derives one transition certificate containing the exact STATE succession edge.

It emits no:

- burn projection;
- clear projection;
- distribution residue;
- specialized economic event;
- issuance event;
- destruction event.

The request does not author the certificate.

## 2.7 Lifecycle status · `rule:guide14-exec:lifecycle`

Phase 6 implements only maturity announcement.

The candidate records:

```text
implemented:
    announce-maturity

outstanding:
    burn-related STATE changes
    clear
    redemption
    request admission
    settlement
    cycle
    migration between STATE constructor generations

release-complete:
    false
```

No spendable placeholder leaf is emitted for an outstanding operation.

---

# 3. Repository ground truth to establish in Wave 0 · `sec:guide14-exec:ground-truth`

Every identifier below is classified as `EXISTS`, `NEW`, `RENAMED`, or `NOT NEEDED` before implementation begins.

## 3.1 Typed semantic sources · `tab:guide14-exec:semantic-ground`

Wave 0 records the exact existing owners of:

| Subject | Required owner |
|---|---|
| STATE object identifier | typed architecture |
| STATE singleton asset and amount | typed architecture |
| cycle domain | typed architecture or realization, according to current ownership |
| announcement lead bounds | typed architecture |
| maturity variants | typed architecture/realization |
| complete STATE semantic field census | typed first-party STATE source |
| announce-maturity relation | realization |
| root succession | realization |
| operator authorization class | architecture/realization |
| transition certificate | architecture/model/realization boundary |
| sponsor opacity | accepted realization and D005 |

If the complete metadata census exists only in model implementation or prose, that is a Wave-0 blocker. The backend does not guess it.

## 3.2 Existing constructor substrate · `tab:guide14-exec:constructor-ground`

Wave 0 inventories and revalidates:

- constructor metadata encoding helpers;
- metadata-leaf builder;
- metadata-leaf unspendability validator;
- tagged hashes;
- canonical tree construction;
- internal-key policy;
- curve and tweak oracle;
- branch ordering;
- representation-nonce retry;
- control-block construction;
- exact target program encoding;
- prototype matrix and run record.

The accepted prototype’s historical resource figures are not carried forward as current STATE measurements.

## 3.3 Existing Phase-5 substrate · `tab:guide14-exec:phase5-ground`

Wave 0 identifies exact existing entry points for:

- validated compiler operation plans;
- target requirement assessment;
- tapscript abstract validation;
- deterministic taptree linking;
- target transaction encoding and decoding;
- finalized signing requests;
- public test owner or operator signing;
- sponsor funding and signing;
- target submission and mined readback;
- environment and provenance binding;
- response-shape validation;
- transcript-derived evidence;
- resource comparison.

Any entry point whose actual contract differs from its current documentation is corrected before reuse.

## 3.4 Dependency direction · `rule:guide14-exec:dependency-direction`

The expected library direction is:

```text
architecture
    ↓
realization
    ↓
compiler
    ↓
target-elements
    ↓
tapscript
    ↓
linker
    ↓
transaction
    ↓
vectors
```

`target-elements-conformance` remains a target-generic conformance and executor boundary, reached from higher-level evidence code without introducing a reverse semantic dependency.

No package imports `vectors`.

No target-specific type enters `realization` or compiler relation identity.

No new third-party dependency is expected.

A dependency change requires the backlog’s full dependency-admission record.

---

# 4. Preflight review register · `tab:guide14-exec:preflight`

The following are review hypotheses against the selected tree. Every row is reproduced against the complete working tree and receives one finding status:

```text
CONFIRMED
REFUTED
RECLASSIFIED
```

and one execution status:

```text
DONE
OPEN with named blocker
```

| ID | Priority | Hypothesis | Required disposition before use |
|---|---:|---|---|
| `G14-R01` | P0 | Native refusals can be counted as answered even when the observed target layer differs from the row’s declared boundary. | Carry observed layer in the standing and require exact boundary equality; unexpected earlier refusal remains unanswered. |
| `G14-R02` | P0 | `OwnerSigningCensus` can pair candidate structure with independently supplied protected bytes and does not fully bind a production request to finalization’s exact selected leaf and code-separator position. | Derive or validate exact candidate bytes and exact finalized leaf data before reusing the census for operator signing. |
| `G14-R03` | P1 | A closeout can validate as completed over an empty or partial restart ledger, and a typed stop need not agree on the blocker. | Derive `Completed` from a full ledger and compare exact stop step and blocker. |
| `G14-R04` | P1 | “Wrong blinder” evidence names a blinder as the serialized changed field although the changed bytes are the value commitment. | Separate construction cause from serialized mutated field. |
| `G14-R05` | P1 | Report-layer “answerability” is counted as answered evidence without a bound validated report. | Split report-required from report-observed or bind structural evidence to an exact validator. |
| `G14-R06` | P1 | Proof-negative attribution accepts a caller-authored range broad enough to excuse arbitrary mutation. | Derive ranges from typed decoded fields or validate against a private field locator. |
| `G14-R07` | P1 | Recording another restart step after complete acceptance can panic despite the public API’s no-panic claim. | Return a typed `AlreadyComplete` refusal. |
| `G14-R08` | P1 | Crossing coordinator fragment accounting is plan-based rather than composition-based and cannot name the crossing destination-form fragment. | Make fragment census composition-aware and derive it from the same branch as emission. |
| `G14-R09` | P1 | Recorded-randomness fixture digests omit semantic amounts for opening-bearing outputs. | Bind every semantic amount under both contracts; make opening bytes contract-specific. |
| `G14-R10` | P1 | Closeout moved-row identities are free-form strings not validated as target transaction identities. | Use a typed parsed target identity. |
| `G14-R11` | P1 | Composed coordinator patterns under-report output, arithmetic, issuance, fee, or confidential-conservation dependencies present in their bytes. | Derive composed evidence and source sets from component patterns. |
| `G14-R12` | P1 | A rejected conservation response may carry openings whose documented provenance requires acceptance and mined outputs. | Add acceptance/refusal-specific response-shape validation. |
| `G14-R13` | P2 | Unavailable operation resource figures are serialized as numeric zero. | Preserve absence with presence-bearing wire fields and a coordinated protocol revision. |
| `G14-R14` | P2 | The native executor’s typed diagnostic stream can disclose caller-selected paths and can be line-injected through unescaped request text. | Use fixed diagnostics, quarantine arbitrary text, reject or escape line breaks. |
| `G14-R15` | P2 | The resource study still cites a cleared global owner-sighash blocker where its actual condition is lane-local unauthorizing witness use. | Replace stale blocker with a lane-specific non-claim or rebuild through real signing. |
| `G14-R16` | P2 | Current-state prose disagrees with current code on protocol revision, Phase-5 status, positive-row standing, and historical sponsor restrictions. | Correct active owners and explicitly time-scope historical records. |

## 4.1 Blocking rule · `gate:guide14-exec:preflight`

No maturity semantic implementation, compiler plan, constructor, linker, ABI, or target evidence begins while a confirmed P0 or P1 row remains open.

Confirmed P2 rows close before their affected boundary is reused:

- `G14-R13` and `G14-R14` close before real target execution;
- `G14-R15` closes before the maturity resource report;
- `G14-R16` closes before the Phase-6 gate record.

The preflight also revalidates the amended Phase-5 exit claim. Guide 14 does not silently inherit a historical count that no longer satisfies its row-boundary rules.

---

# 5. Decisions fixed by this guide · `sec:guide14-exec:decisions`

## 5.1 Public metadata carrier · `rule:guide14-exec:metadata-carrier`

The initial candidate publishes predecessor-construction data and the announced cycle in the STATE input’s script-path witness.

The successor semantic metadata is derived from those public values. Its canonical representation nonce is also public in the witness.

No separate metadata output is introduced in the initial candidate.

This decision is accepted only if Wave 0 proves that an unrelated process can reconstruct the successor from the accepted transaction alone. If that proof fails, implementation stops and files a focused carrier decision; it does not add an ad hoc second publication.

## 5.2 Static constructor relation · `rule:guide14-exec:static-relation`

Predecessor and successor share one exact linked static subtree.

The dynamic metadata leaf changes with metadata. The static operation subtree does not.

No migration between static subtrees is implemented.

## 5.3 Branch side · `rule:guide14-exec:branch-side`

Where the emitted program cannot perform byte-lexicographic child ordering, the constructor fixes:

```text
metadata leaf:
    left

static subtree:
    right
```

and admits only metadata encodings satisfying:

\[h_{\mathrm{metadata}}\le h_{\mathrm{static}}\]

A public representation nonce is ground deterministically from zero until this relation and the selected tweak-totality policy both hold.

The target program hashes in fixed order. It does not accept an order bit from the witness.

## 5.4 Tweak totality · `rule:guide14-exec:tweak-totality`

The candidate uses deterministic representation-nonce retry.

For each nonce \(r=0,1,\ldots,R_{\max}\):

1. encode metadata with nonce \(r\);
2. compute the metadata leaf;
3. check fixed branch-side admissibility;
4. compute the canonical tree root;
5. compute the target tweak;
6. accept the first nonce for which the target constructor is defined.

Zero tweak is admitted when the resulting point is valid.

The search refuses:

- tweak at or above the group order;
- identity result;
- malformed internal key;
- missing executing leaf;
- malformed metadata;
- branch-side failure after budget exhaustion.

Only retryable hash-derived totality failures advance the nonce.

No invalid semantic input is repaired by nonce search.

## 5.5 Internal key · `rule:guide14-exec:internal-key`

The candidate reuses the accepted public NUMS internal-key policy only after Wave 0 confirms that the exact constructor and target assumptions remain valid.

No operator, owner, wallet, release, deployment, or generated-and-discarded key is used as the internal key.

The residual discrete-log assumption remains explicit.

A refused key-path attempt establishes only that the attempt was observed and refused. It does not prove the residual.

## 5.6 Operator profile · `rule:guide14-exec:operator-profile`

The initial candidate uses the reviewed all-inputs/all-outputs script-path profile unless Wave 3 establishes a distinct operator profile with equal or stronger protected-data coverage.

The operator profile must commit to:

- every input and spent-output field required by the target;
- every output;
- successor metadata publication where serialized;
- successor output program;
- transaction version and lock time;
- relevant output witnesses;
- issuance absence;
- executing leaf and script-path terms.

No single-output or input-extending form is admitted.

The annex remains absent unless separately reviewed.

## 5.7 Synthetic target STATE · `rule:guide14-exec:synthetic-origin`

Because protocol genesis and earlier STATE-producing target operations are not implemented, native tests may fund one synthetic STATE predecessor.

Every record carrying it states:

```text
synthetic target STATE fixture
not protocol genesis
not evidence of trusted setup
not evidence of earlier root history
not production STATE
authorizes nothing of value
```

The tested root-history claim begins at that synthetic predecessor and covers exactly the announcement edge after it.

---

# 6. Typed STATE metadata · `sec:guide14-exec:metadata`

## 6.1 Semantic type · `def:guide14-exec:state-metadata`

Wave 1 introduces or confirms one typed semantic metadata value:

```rust
pub struct StateMetadata {
    // Exact fields derived from the authoritative typed STATE source.
}
```

The guide expects the semantic census to include the established STATE quantities and status:

```text
Ω
Y_L
Y_T
Q
cycle
maturity
```

That list is a research clue, not authority. Wave 1 obtains the exact list from the typed source and fails if it differs without a recorded semantic decision.

## 6.2 Maturity type · `def:guide14-exec:maturity`

The maturity field is a closed typed value equivalent to:

```rust
pub enum Maturity {
    Unannounced,
    Announced { cycle: Cycle },
    Complete,
}
```

The exact existing spelling and discriminants are reused where already owned.

Unknown tags, malformed payloads, alternate encodings, and trailing bytes reject.

## 6.3 Representation type · `def:guide14-exec:representation-metadata`

The target representation carries a separate field:

```rust
pub struct StateRepresentationNonce(u32);
```

or the exact width the accepted constructor policy and target bounds justify.

The nonce:

- begins at zero;
- increments deterministically;
- is not semantic STATE;
- is erased by semantic projection;
- is retained by constructor evidence;
- cannot be caller-selected as the final result;
- does not become an identity.

## 6.4 Canonical encoding · `rule:guide14-exec:canonical-metadata`

One encoding function and one strict decoder are defined:

```rust
fn encode_state_metadata(
    semantic: &StateMetadata,
    representation: StateRepresentationNonce,
) -> Vec<u8>;

fn decode_state_metadata(
    bytes: &[u8],
) -> Result<EncodedStateMetadata, StateMetadataRefusal>;
```

The encoding fixes:

- domain separator;
- schema revision;
- exact field order;
- exact field widths;
- integer byte order;
- enum discriminants;
- representation nonce position;
- reserved-field behavior;
- trailing-byte rejection.

A successfully decoded value re-encodes byte-identically:

\[\operatorname{encode}(\operatorname{decode}(b))=b\]

One semantic STATE and one representation nonce have exactly one encoding.

## 6.5 Transition function · `rule:guide14-exec:transition-function`

The successor is derived through one total typed function:

```rust
pub fn announce_maturity(
    predecessor: &StateMetadata,
    announced_cycle: Cycle,
    bounds: AnnouncementLeadBounds,
) -> Result<StateMetadata, MaturityTransitionRefusal>;
```

It:

1. requires `Unannounced`;
2. computes the lower and upper bounds with checked arithmetic;
3. validates the requested cycle;
4. copies every unaffected semantic field;
5. sets `Announced { cycle: announced_cycle }`;
6. returns no representation nonce;
7. returns no partial successor.

Representation grinding happens only after semantic transition succeeds.

## 6.6 Metadata refusals · `def:guide14-exec:metadata-refusals`

The initial closed vocabulary includes, subject to exact existing names:

```text
WrongDomain
UnsupportedSchema
WrongLength
UnknownMaturityDiscriminant
MaturityPayloadMalformed
ReservedFieldNonzero
TrailingBytes
PredecessorAlreadyAnnounced
PredecessorMaturityComplete
AnnouncementBelowMinimum
AnnouncementAboveMaximum
CycleArithmeticOverflow
UnexpectedSemanticFieldChange
NoncanonicalRepresentationNonce
RepresentationSearchExhausted
```

Every variant is reached by a focused test or explicitly classified as structurally unreachable with a proof of that fact.

---

# 7. Compiler target-operation plan · `sec:guide14-exec:compiler`

## 7.1 Plan type · `def:guide14-exec:operation-plan`

Wave 2 derives:

```rust
pub struct ValidatedMaturityAnnouncementOperationPlan {
    // private fields
}
```

There is no public unchecked constructor.

The plan retains:

- exact operation identity;
- architecture and realization binding;
- complete relation census;
- source and witness requirements;
- operator authorization;
- metadata preservation;
- root succession;
- sponsor isolation;
- constructibility;
- lifecycle obligations;
- representation requirements;
- abstract carrier requirements;
- layout requirements;
- coverage requirements;
- target capability requirements;
- external target evidence roles.

It contains no:

- target opcode;
- stack index;
- target transaction position;
- program byte;
- control block;
- representation nonce result;
- graph handle;
- filesystem path;
- environment value;
- digest.

## 7.2 Required relation families · `tab:guide14-exec:relations`

The exact census is derived from realization. Validation requires at least the following families to be represented where they exist in that source:

| Relation family | Required content |
|---|---|
| STATE input cardinality | exactly one |
| STATE output cardinality | exactly one |
| STATE predecessor recognition | exact root asset, amount, metadata, constructor, and current-root role |
| predecessor maturity | unannounced |
| successor maturity | announced at requested cycle |
| announcement lead | exact inclusive window |
| metadata preservation | every unaffected field equal |
| operator authorization | approved operator over finalized bytes |
| input closure | STATE plus optional sponsor only |
| output closure | STATE successor plus optional sponsor change and fee only |
| sponsor isolation | exact, disjoint, amount-opaque |
| root policy | STATE succession, every other root absent |
| issuance | absent |
| destruction | absent |
| projection | transition certificate present, specialized events absent |
| public construction | successor derivable from public data |
| public recovery | unrelated process can reconstruct successor |
| lifecycle | later STATE operations outstanding |

## 7.3 Carrier plan · `rule:guide14-exec:carrier-plan`

Expected carrier classes are:

```text
announcement leaf:
    predecessor metadata authentication
    maturity predecessor
    lead window
    successor metadata derivation
    successor constructor reconstruction
    operator authorization

coordinator structure:
    exact transaction counts
    STATE input/output closure
    root and event absence
    sponsor boundary

linked constructor:
    metadata-leaf commitment
    static-subtree continuity
    internal-key and leaf-version policy

target:
    selected signature semantics
    whole-transaction conservation
    taproot commitment and control-path validity

report:
    current-root freshness
    root-history edge sequence
    public reconstruction
```

A relation assigned to a report is not marked target-enforced.

A relation assigned to the target is not marked emitted merely because a target later accepts.

## 7.4 Compiler corruption tests · `rule:guide14-exec:compiler-corruption`

Required tests include:

- remove one relation;
- insert an extra relation;
- change root subject;
- insert RESV participation;
- remove operator authorization;
- alter the lead bounds;
- remove one unaffected-field dependency;
- remove public recoverability;
- remove lifecycle obligation;
- move a global relation to an optional inactive carrier;
- introduce sponsor amount as a source;
- introduce target type into compiler core;
- duplicate a relation;
- change a coverage boundary;
- insert graph handles into public projection.

Every corruption returns a typed plan refusal and no partial plan.

## 7.5 Determinism · `rule:guide14-exec:compiler-determinism`

Equal typed inputs produce equal plans and equal stable projections.

Declaration-order permutations do not change the projection.

Search exhaustion returns a typed complexity error and no best-so-far result.

---

# 8. Target assessment and operator closure · `sec:guide14-exec:target`

## 8.1 Required target capabilities · `tab:guide14-exec:target-capabilities`

Wave 3 classifies at least:

- current-input index;
- input count and output count;
- input asset and value inspection;
- input and output program inspection;
- transaction version and lock-time inspection where required;
- byte equality and verifying equality;
- checked fixed-width arithmetic and comparisons;
- byte slicing and concatenation;
- streaming SHA-256;
- tapleaf and tapbranch hashing support or exact compositional equivalents;
- tweak verification;
- x-only key encoding;
- signature verification;
- selected sighash semantics;
- script-path execution;
- target transaction conservation;
- fee-role recognition;
- resource limits.

Each requirement receives one explicit state:

```text
Unsupported
MissingTargetPrimitives
BackendPatternRequired
BackendStructural
ExternalEvidenceRequired
CompleteBackendPattern
```

No capability is inferred from a similarly named opcode.

## 8.2 Operator-key encoding closure · `rule:guide14-exec:operator-key`

The operator public key is:

- public;
- target-approved;
- exact-width;
- curve-valid;
- committed by deployment or typed STATE policy;
- not supplied by the request.

The target program pushes or reconstructs the approved encoding and verifies the signature.

Required negatives:

- empty key;
- wrong width;
- malformed approved key;
- unknown nonempty key type;
- another operator’s approved key;
- valid signature under another key;
- valid signature over another transaction.

## 8.3 Target-generic signer capability · `def:guide14-exec:signer-capability`

If the existing sponsor or owner signing interface cannot represent an operator without importing protocol meaning, add a target-generic capability:

```rust
ExecutorCapability::TestScriptPathAuthorization
```

with request and response types equivalent to:

```rust
pub struct TargetScriptPathSigningSubject {
    pub finalized_transaction: Vec<u8>,
    pub input_index: u16,
    pub spent_output: WireSpentOutput,
    pub executing_leaf: WireTapleaf,
    pub sighash_profile: WireSighashProfile,
    pub signer: PublicTestSignerHandle,
}

pub struct TargetScriptPathSigningResult {
    pub signature_bound_to: Vec<u8>,
    pub witness_prefix: Vec<Vec<u8>>,
    pub signer_public_key: Vec<u8>,
    pub sighash_profile: WireSighashProfile,
}
```

The exact record is minimized against existing protocol types during Wave 3.

The request contains no:

- maturity cycle meaning;
- expected verdict;
- expected message digest;
- production key;
- private scalar;
- semantic STATE.

The response is validated for:

- exact byte echo;
- exact signer identity;
- exact profile;
- exact input;
- nonempty witness;
- absence of success fields on refusal.

A protocol change increments the schema on Rust and Python together.

## 8.4 No profile by assertion · `rule:guide14-exec:profile-evidence`

The operator profile is established by:

1. source review;
2. independent first-party message recomputation;
3. one observed target acceptance;
4. focused controls moving protected terms.

A profile string returned by the executor is provenance, not evidence by itself.

---

# 9. Candidate STATE constructor · `sec:guide14-exec:constructor`

## 9.1 Constructor type · `def:guide14-exec:constructor-type`

Wave 4 introduces:

```rust
pub struct CandidateStateConstructor {
    metadata: EncodedStateMetadata,
    metadata_leaf: StateMetadataLeaf,
    static_subtree: LinkedStateStaticSubtree,
    tree: StateConstructorTree,
    internal_key: StateInternalKey,
    output_key: StateOutputKey,
    program: Vec<u8>,
    control_recipes: BTreeMap<StateLeafRole, StateControlRecipe>,
}
```

Fields remain private. Construction is fallible and validated.

No `Default` exists.

## 9.2 Tree shape · `rule:guide14-exec:tree-shape`

The initial constructor is:

```text
TapBranch
├── metadata commitment leaf
└── static maturity-operation subtree
```

The exact canonical child ordering follows §5.3.

The metadata leaf and static subtree are distinct typed roles. A generic vector of scripts is not sufficient constructor input.

## 9.3 Metadata leaf · `rule:guide14-exec:metadata-leaf`

The metadata leaf commits the exact canonical metadata bytes.

Its executable program always aborts, for every witness and initial stack.

Required evidence:

```text
abstract successful states:
    empty

abstract non-aborting successful states:
    empty

target-native accepted spend through metadata leaf:
    none

target-native attempted spend:
    rejected
```

An empty program is forbidden because an inherited true stack item could satisfy it.

## 9.4 Static subtree · `rule:guide14-exec:static-subtree`

The static subtree contains only:

- maturity-announcement operation leaves;
- exact support leaves required by the accepted constructor strategy.

It contains no placeholder for a later operation.

Duplicate leaf identities reject.

Conflicting program, role, version, or weight under one identity reject.

## 9.5 Predecessor authentication · `rule:guide14-exec:predecessor-authentication`

The announcement program authenticates the consumed program against:

- predecessor semantic metadata;
- predecessor representation nonce;
- exact metadata schema;
- authenticated static subtree;
- exact leaf version;
- exact internal key;
- target tweak relation;
- actual consumed input program.

A metadata witness that reconstructs another program rejects.

## 9.6 Successor reconstruction · `rule:guide14-exec:successor-reconstruction`

The program derives successor semantic metadata from the authenticated predecessor and requested cycle, then authenticates:

- successor representation nonce;
- successor metadata leaf;
- unchanged static subtree;
- exact leaf version;
- exact internal key;
- exact tweak relation;
- actual output-0 program.

No independent successor semantic metadata blob is accepted from the witness.

Where target scheduling requires successor byte components in the witness, the program verifies they equal the typed derivation before using them.

## 9.7 Constructor reference graph · `rule:guide14-exec:constructor-graph`

Every constructor dependency becomes a typed linker reference.

Wave 4 and Wave 6 compute the reference graph and its strongly connected components.

An SCC is not a strategy.

Any cyclic edge requires one named authenticated cut, such as:

- reading the actual consumed program from target introspection;
- authenticating a static-root witness against that consumed program;
- retaining the authenticated root for successor reconstruction.

Repeated hashing until stabilization is prohibited.

An externally supplied equality that is not independently checked remains an explicit outstanding obligation and blocks the candidate.

## 9.8 Totality refusals · `def:guide14-exec:constructor-refusals`

The constructor’s closed refusal vocabulary includes:

```text
MetadataEncodingRefused
MetadataLeafNotUnspendable
StaticSubtreeEmpty
StaticSubtreeIncomplete
DuplicateLeaf
ConflictingLeaf
WrongLeafVersion
WrongInternalKey
InternalKeyNotAPoint
CanonicalBranchSideNotSatisfied
RepresentationSearchExhausted
TweakAboveGroupOrder
TweakedPointIsIdentity
ControlPathTooDeep
ExecutingLeafAbsent
ExecutingLeafRepeated
ConstructorReferenceCycleUnresolved
ResourceBoundExceeded
```

Retryable and nonretryable refusals are explicitly distinguished.

---

# 10. Maturity-announcement tapscript · `sec:guide14-exec:patterns`

Every pattern record carries:

- identity;
- semantic owner;
- prerequisites;
- typed fragment;
- abstract precondition;
- success states;
- non-aborting failures;
- abort causes;
- source requirements;
- witness role;
- constructibility;
- disclosure;
- resource projection;
- target evidence;
- residuals.

All composed pattern metadata is derived from component records. No composed program manually under-reports evidence present in its instructions.

## 10.1 Proposed pattern census · `tab:guide14-exec:pattern-census`

Subject to exact naming in Wave 5:

```text
StateInputRecognitionV1
StateCoordinatorRoleV1
StateCardinalityV1
StateMetadataAuthenticationV1
MaturityPredecessorV1
AnnouncementWindowV1
StateMetadataCopyThroughV1
StateSuccessorConstructorV1
StateOperatorAuthorizationV1
StateSponsorIsolationV1
StateIssuanceAbsenceV1
StateRootAndEventClosureV1
StateAnnouncementProgramV1
```

An identity is minted only when the complete pattern record exists and its fragment has been abstractly walked.

## 10.2 Coordinator role · `rule:guide14-exec:coordinator`

Input 0 is the STATE coordinator.

Sponsor inputs, if any, occupy the suffix after it.

The coordinator authenticates:

- exact input count;
- exact output count;
- input 0 as the STATE input;
- output 0 as the STATE successor;
- sponsor suffix;
- optional sponsor-change role;
- fee role;
- no unclassified position.

## 10.3 Local STATE recognition · `rule:guide14-exec:state-recognition`

The executing input proves or carries through authenticated construction:

```text
exact STATE singleton asset
exact singleton amount
exact predecessor constructor
canonical predecessor metadata
current STATE root role
maturity = Unannounced
```

The program must not confuse:

- constructor commitment;
- current-root freshness;
- semantic metadata.

Current-root freshness may require report-layer chain context. If so, it remains a distinct external evidence role and is not claimed by the leaf.

## 10.4 Announcement-window fragment · `rule:guide14-exec:window-fragment`

The fragment obtains \(c\), \(a\), \(L_{\min}\), and \(L_{\max}\) from authenticated typed sources or linked literals and checks:

\[c+L_{\min}\le a\]

\[a\le c+L_{\max}\]

Every addition is checked before comparison.

Every Boolean is consumed immediately.

A known-false comparison followed by verification has no abstract success path.

## 10.5 Metadata copy-through · `rule:guide14-exec:copy-through-fragment`

The emitted program does not maintain a hand-authored field equality list independently of the typed metadata schema.

Preferred construction:

1. authenticate canonical predecessor bytes;
2. slice the bytes around the maturity field according to a schema-derived layout;
3. derive the successor maturity encoding;
4. concatenate unchanged prefix, changed maturity, unchanged suffix, and canonical representation fields;
5. authenticate the resulting successor constructor.

If fields cannot be safely represented by one prefix/suffix transformation, the generator emits one schema-derived comparison per unaffected field from the same typed field census.

## 10.6 Operator fragment · `rule:guide14-exec:operator-fragment`

The operator fragment:

- pushes or reconstructs the committed operator public key;
- never takes the key from an untrusted witness;
- consumes one signature witness item;
- uses the verifying signature primitive;
- leaves no signature Boolean;
- carries no unknown-key unverified success form.

## 10.7 Successor constructor fragment · `rule:guide14-exec:successor-fragment`

The successor fragment:

- uses output 0;
- authenticates exact STATE asset and amount;
- reconstructs canonical successor metadata;
- uses the authenticated static subtree;
- validates canonical representation nonce;
- verifies exact output program;
- drops no payload that another relation depends on;
- leaves no constructor equality as an unconsumed residual.

## 10.8 Root and absence closure · `rule:guide14-exec:absence-fragment`

The complete position census and explicit field checks establish:

- one STATE input;
- one STATE output;
- no second STATE position;
- no RESV or PACE position;
- no authority root;
- no receipt or ASH position;
- no entitlement, vault, or control output;
- no issuance;
- no destruction;
- no specialized event.

Issuance absence is checked on every input, including sponsor inputs.

Object-family absence derived structurally from a complete position census is recorded as structural evidence, not as an opcode that was never emitted.

## 10.9 Sponsor isolation · `rule:guide14-exec:sponsor-fragment`

The sponsor fragment reads no sponsor value field.

It authenticates:

- sponsor input suffix;
- reserve asset;
- sponsor-change program and role;
- target fee program and role;
- disjointness from STATE;
- exact counts.

## 10.10 Final stack · `rule:guide14-exec:final-stack`

Every complete program:

- ends on exactly one canonical true item;
- has no successful state with another main-stack shape;
- has no alternate-stack residue;
- has no non-aborting failure state surviving final truth;
- has no unknown-key unverified success;
- remains inside script, stack, element, operation, and validation-budget limits.

---

# 11. Linking · `sec:guide14-exec:linking`

## 11.1 Typed symbols · `rule:guide14-exec:symbols`

The candidate symbol census is derived from actual consumers and may include:

```text
StateSingletonAsset
StateSingletonAmount
StateMetadataSchema
StateMetadataDomain
UnannouncedDiscriminant
AnnouncedDiscriminant
MinimumAnnouncementLead
MaximumAnnouncementLead
StateStaticSubtree
OperatorPublicKey
OwnerSighashProfile
StateLeafVersion
UnspendableInternalKey
SponsorReserveAsset
SponsorChangeProgram
SponsorChangeVersion
FeeProgramDigest
```

A reserved symbol with no consumer is prohibited.

## 11.2 Two-pass resolution · `rule:guide14-exec:two-pass-link`

Pass one:

```text
collect all typed definitions
validate type and origin
reject duplicates
normalize by stable typed key
```

Pass two:

```text
resolve all references
validate expected type
build frozen reference graph
compute SCCs
apply cycle policy
```

Every mandatory reference resolves exactly once.

## 11.3 Relocations · `rule:guide14-exec:relocations`

Relocations are identified structurally by rebuilding a program with one symbol changed or by another typed method that demonstrates the exact consumer.

No raw byte scan alone assigns semantic relocation meaning.

Each relocation records:

- symbol;
- expected type;
- program role;
- exact instruction or typed field;
- pre-link value;
- linked value;
- resource effect.

Applying zero or two relocations for one mandatory reference rejects.

## 11.4 Deterministic tree · `rule:guide14-exec:taptree`

The static subtree and complete constructor tree:

- reject duplicate leaves;
- use exact checked cost arithmetic;
- use stable typed tie-breaks;
- are declaration-order independent;
- enforce target depth bounds;
- compare small instances against an independent exact oracle.

Saturating arithmetic cannot participate in an exact-optimum claim.

## 11.5 Carrier closure · `rule:guide14-exec:carrier-closure`

Wave 6 compares:

```text
compiler-required carriers
=
backend-emitted carriers
=
linked reachable carriers
=
ABI-carried roles
```

per execution case.

The comparison is duplicate-sensitive and bidirectional.

## 11.6 Linked candidate · `def:guide14-exec:linked-bundle`

The result is:

```rust
pub struct CandidateLinkedMaturityBundle {
    // exact plan, target, constructor, programs, tree, placements,
    // references, resources, ABI handoff, evidence obligations,
    // lifecycle, and candidate status
}
```

It carries no content digest.

---

# 12. Candidate transaction and witness ABI · `sec:guide14-exec:abi`

## 12.1 Input layout · `rule:guide14-exec:input-layout`

```text
input 0:
    canonical STATE predecessor and coordinator

inputs 1..s:
    optional sponsor suffix
```

There is exactly one STATE input.

A request cannot select another coordinator.

Sponsor inputs are canonicalized under the inherited policy.

Duplicates and STATE/sponsor overlap reject before sorting.

## 12.2 Output layout · `rule:guide14-exec:output-layout`

```text
output 0:
    canonical STATE successor

next optional output:
    sponsor change

final output where required:
    target fee role
```

No other protocol output is admitted.

STATE cannot satisfy sponsor change or fee.

Sponsor change and fee cannot satisfy STATE.

## 12.3 Typed request · `def:guide14-exec:typed-request`

The transaction layer introduces or reuses:

```rust
pub struct MaturityAnnouncementRequest {
    announced_cycle: Cycle,
    requested_form: MaturityRequestedForm,
    sponsor_change: SponsorChangeRequest,
}
```

The exact sponsor fields follow the inherited interface.

There is no request field for:

- predecessor outpoint independently of the current-root view;
- predecessor metadata;
- successor metadata;
- successor program;
- operator key;
- static subtree;
- internal key;
- representation nonce;
- transaction version;
- sequence;
- target fee role;
- control block.

## 12.4 Public current-STATE view · `def:guide14-exec:state-view`

Construction consumes one validated public view containing:

```text
current STATE outpoint
exact asset and amount
canonical predecessor semantic metadata
predecessor representation nonce
current-root binding
actual predecessor program
current cycle
accepted linked bundle
operator public identity
deployment symbols
```

Duplicate or contradictory views reject.

Later entries cannot overwrite earlier ones.

## 12.5 Constructor search · `rule:guide14-exec:successor-search`

The transaction layer:

1. validates the semantic request;
2. derives successor semantic metadata;
3. searches canonical representation nonce from zero;
4. derives the successor constructor;
5. compares the result with the linked constructor policy;
6. fixes output 0;
7. fixes optional sponsorship;
8. freezes protected bytes;
9. creates the operator signing request.

A caller cannot provide a nonce result or arbitrary output program.

## 12.6 Witness ABI · `rule:guide14-exec:witness`

The STATE input witness has a canonical typed order, expected to include roles equivalent to:

```text
operator signature
predecessor metadata publication
announced cycle or exact request encoding
successor representation nonce
announcement leaf
control block
constructor-specific authenticated public values
```

Only genuinely required items are retained.

The successor semantic metadata need not be independently witnessed if the program derives it.

Every witness role carries:

- encoding;
- minimum and maximum width;
- public/test-secret classification;
- source;
- consumer;
- target program;
- canonical position.

## 12.7 Finalization and signing states · `rule:guide14-exec:finalization`

Use type transitions equivalent to:

```text
MaturityConstruction
    ↓
FinalizedMaturityAnnouncement
    ↓
OperatorSigningStarted
    ↓
OperatorAuthorizedMaturityAnnouncement
    ↓
SubmitReadyMaturityAnnouncement
```

After finalization, protected mutation is unrepresentable or a typed refusal.

Protected regions include:

```text
inputs
spent-output census
outputs
successor metadata
successor representation nonce
successor program
output witnesses
version
lock time
sponsor region
fee region
executing leaf data
```

## 12.8 Canonical target bytes · `rule:guide14-exec:canonical-bytes`

External target bytes pass through a strict decoder.

Successful decoding requires:

\[\operatorname{decode}(b).\operatorname{encode}()=b\]

The decoder rejects:

- nonminimal compact sizes;
- unsupported asset or value forms;
- peg-ins;
- issuance-bearing bytes;
- unsupported proofs;
- inconsistent witness censuses;
- superfluous all-empty witness records;
- trailing bytes.

## 12.9 Candidate status · `rule:guide14-exec:abi-status`

The ABI result is:

```text
CandidateMaturityAnnouncementAbi
```

It is not final and carries no ABI digest.

---

# 13. Operator signing · `sec:guide14-exec:signing`

## 13.1 Signing request · `def:guide14-exec:signing-request`

The operator signing request binds:

- exact finalized transaction bytes;
- protected bytes or typed signing census;
- STATE input index;
- predecessor outpoint;
- exact spent-output fields;
- executing leaf and control data needed for recomputation;
- operator public identity;
- selected profile;
- deployment genesis.

It contains no opening, blinder, private key, or expected target verdict.

## 13.2 Exact census binding · `rule:guide14-exec:census-binding`

Before the existing signing census is reused, `G14-R02` must be closed.

For a production construction route:

```text
candidate structure
protected bytes
spent-output census
input index
leaf hash
leaf version
code-separator position
control block
deployment seed
```

must all derive from or be validated against one finalized candidate.

A caller may not substitute another committed leaf from the same tree.

The negative-evidence route over foreign bytes remains explicitly separate and cannot be called by production construction.

## 13.3 Signing response · `def:guide14-exec:signing-response`

A successful response carries:

- witness bytes;
- exact bytes signed;
- operator public identity;
- selected profile or returned type byte;
- input position.

The transaction layer validates every binding before insertion.

A refused response carries no witness or accepted artifact.

## 13.4 Operator signing faults · `rule:guide14-exec:signing-faults`

Required:

- missing response;
- duplicate response;
- unexpected response;
- wrong operator;
- wrong input;
- wrong leaf;
- wrong deployment;
- wrong profile;
- wrong type byte;
- empty signature;
- malformed signature;
- signature over another transaction;
- response bound to other bytes;
- metadata changed after signing;
- successor program changed after signing;
- sponsor input added after signing;
- fee role changed after signing.

---

# 14. Evidence architecture · `sec:guide14-exec:evidence`

## 14.1 Canonical evidence plan · `def:guide14-exec:evidence-plan`

Wave 8 derives:

```rust
pub struct MaturityAnnouncementEvidencePlan {
    // private fields
}
```

It is built only from:

- validated compiler operation plan;
- exact linked bundle;
- exact candidate ABI;
- canonical semantic fixture registry;
- canonical constructor mutation registry;
- canonical root-history mutation registry;
- exact target and deployment binding;
- expected executor provenance.

No public constructor accepts caller-authored verdict tuples.

## 14.2 Evidence standing · `rule:guide14-exec:standing`

The standing vocabulary distinguishes:

```text
FirstPartyDischarged
FirstPartyRequired
NativeAcceptanceObserved
NativeRefusalObserved
NativeRefusalAtUnexpectedBoundary
ConstructorContinuityObserved
RootHistoryObserved
PublicRecoveryObserved
ReportLayerObserved
InfrastructureBlocked
Experimental
```

An “answerable” capability is not an answered standing.

Every native refusal stores:

- observed layer;
- declared boundary;
- accepted control identity;
- submitted subject identity or exact bytes;
- refusal detail where available;
- mutation locator or structural separator;
- intended carrier;
- whether the carrier executed.

`NativeRefusalAtUnexpectedBoundary` is not answered.

## 14.3 Validated operation report · `def:guide14-exec:operation-report`

The operation report retains the complete generic execution transcript and binds:

- target projection;
- deployment binding;
- executor handshake;
- observed environment;
- provenance;
- exact sent requests;
- exact responses;
- funding steps;
- signing steps;
- submitted bytes;
- mined readback;
- target verdicts;
- semantic projection;
- constructor projection;
- root-history projection;
- public-recovery projection;
- resource comparison.

The validator constructs the report. It does not validate an arbitrary caller-authored summary while retaining the summary’s answers.

## 14.4 Safety report · `def:guide14-exec:safety-report`

The safety report answers:

```text
Did valid maturity announcements preserve the exact semantic operation,
and did every invalid semantic, operator, constructor, root, sponsor,
ABI, and protocol mutation fail at its declared boundary?
```

It does not answer constructor continuity or root history by implication.

## 14.5 Constructor-continuity report · `def:guide14-exec:continuity-report`

The continuity report carries, for predecessor and successor:

- semantic metadata;
- representation nonce;
- metadata bytes;
- metadata leaf script and hash;
- static subtree root;
- branch root;
- internal key;
- tweak;
- output key and parity;
- witness program;
- control recipe;
- exact equality results;
- residual assumptions.

It reports static-subtree continuity separately from semantic metadata continuity.

## 14.6 Root-history report · `def:guide14-exec:history-report`

The root-history report carries an ordered typed edge sequence:

```rust
pub struct StateRootEdge {
    predecessor: StateOutpoint,
    successor: StateOutpoint,
    operation: OperationId,
    certificate: TransitionCertificate,
}
```

The report validates every edge in order.

Final cursor equality is insufficient.

## 14.7 Public-recovery report · `def:guide14-exec:recovery-report`

The public-recovery report is produced by an unrelated process or a process constrained to receive only the public handoff.

It records:

- transaction locator;
- exact transaction bytes;
- STATE output index;
- public witness fields;
- decoded predecessor metadata;
- derived successor metadata;
- derived canonical nonce;
- reconstructed successor program;
- actual successor program;
- equality result;
- absence of creator-private dependencies.

## 14.8 Resource report · `def:guide14-exec:resource-report`

The resource report is separate and carries:

- metadata encoding bytes;
- metadata leaf bytes;
- static subtree bytes;
- announcement leaf bytes;
- constructor bytes;
- tree depth;
- control bytes;
- witness bytes by role;
- initial witness item count;
- peak main stack;
- peak alternate stack;
- largest element;
- hash operations;
- curve/tweak operations;
- validation budget;
- transaction weight;
- virtual size;
- target consensus verdict;
- relay-policy verdict;
- nonce-search attempts as noncanonical diagnostics;
- construction and execution time as noncanonical diagnostics.

Absence is never serialized as numeric zero.

## 14.9 Report canonicality · `rule:guide14-exec:report-canonicality`

Canonical report bytes exclude:

- wall-clock time;
- elapsed time;
- hostname;
- username;
- process ID;
- temporary path;
- executor path;
- caller-selected filesystem path;
- ambient environment;
- raw child stderr;
- production or test private scalars.

Timing is emitted separately.

If a report becomes an externally consumed interchange document, ADR-022 activates before that consumer merges.

---

# 15. Semantic and target fixtures · `sec:guide14-exec:fixtures`

## 15.1 Positive semantic fixture · `rule:guide14-exec:positive-fixture`

A positive fixture begins from a model-valid world containing:

- one operational STATE;
- maturity `Unannounced`;
- nontrivial values in every semantic field;
- a current cycle;
- an authorized operator;
- canonical root history;
- no RESV participation in the operation;
- optional sponsor context where selected.

The transition runs through the invariant-wrapped model path and retains:

- predecessor world;
- request;
- canonical order;
- successor world;
- appended transition certificate.

## 15.2 Synthetic target materialization · `rule:guide14-exec:synthetic-materialization`

The target fixture funds a synthetic predecessor at the exact candidate STATE constructor for the semantic predecessor.

Funding evidence proves only:

- the target can hold an output at that program;
- the output can later be spent by the candidate path.

It does not prove genesis, prior root history, or production authority.

## 15.3 Expected semantic result · `rule:guide14-exec:expected-semantics`

Expected STATE is produced by:

- realization transition;
- executable model;
- typed projection adapter.

The backend constructor does not generate expected semantics.

## 15.4 Target materialization · `rule:guide14-exec:target-materialization`

Target bytes derive from:

```text
candidate linked bundle
+
candidate ABI
+
typed maturity request
+
public current-STATE view
+
public disposable operator signer
+
optional sponsor capability
```

Semantic fixtures contain no target program, control block, transaction byte, or signature.

## 15.5 Positive class witnesses · `rule:guide14-exec:class-witnesses`

A positive class includes an executable typed predicate.

Examples:

```text
minimum lead:
    announced_cycle = current_cycle + L_min

maximum lead:
    announced_cycle = current_cycle + L_max

single-field transition:
    all semantic fields but maturity compare equal

nonzero representation nonce:
    every earlier nonce is checked and refused for a retryable reason

sponsored:
    exact sponsor suffix and fee role are present

public recovery:
    independent reconstruction equals output 0’s program
```

A class name alone cannot make a fixture canonical.

---

# 16. Required vector matrix · `sec:guide14-exec:vectors`

Every row carries:

- polarity;
- mutation layer;
- declared evidence boundary;
- intended relation;
- intended carrier;
- collateral closure;
- canonical control;
- mutation locator;
- expected projection.

## 16.1 Positive cases · `tab:guide14-exec:positive`

Required:

1. minimum valid lead;
2. maximum valid lead;
3. representative interior lead;
4. smallest current cycle;
5. current cycle near checked upper domain;
6. nontrivial unaffected fields;
7. representation nonce zero;
8. representation nonce nonzero;
9. sponsorless;
10. sponsored without change;
11. sponsored with change where supported;
12. public successor recovery;
13. repeated equal construction producing equal candidate bytes;
14. accepted target transaction with semantic, constructor, root, and recovery projections all matching.

## 16.2 Maturity-window faults · `tab:guide14-exec:window-faults`

Required:

- predecessor already announced;
- predecessor complete;
- one below minimum;
- one above maximum;
- equal to current cycle where outside the lead;
- lower-bound addition overflow;
- upper-bound addition overflow;
- malformed cycle encoding;
- request cycle differs from successor;
- successor remains unannounced;
- successor becomes complete.

## 16.3 Metadata faults · `tab:guide14-exec:metadata-faults`

For every unaffected semantic field:

- change that field alone;
- keep the maturity transition correct;
- keep constructor shape otherwise valid;
- require refusal or report-layer semantic mismatch at the declared boundary.

Additionally:

- omit one field;
- duplicate one field;
- reorder fields;
- unknown schema;
- wrong domain separator;
- noncanonical enum tag;
- nonzero reserved field;
- trailing bytes;
- metadata from another STATE object;
- correct semantic metadata with noncanonical representation nonce;
- semantic field changes compensated by another field.

## 16.4 Operator faults · `tab:guide14-exec:operator-faults`

Required:

- missing operator signature;
- empty signature;
- malformed signature;
- wrong operator;
- stale operator;
- unknown key encoding;
- malformed approved key;
- valid signature under another key;
- valid signature over another candidate;
- wrong profile;
- wrong input;
- wrong leaf;
- duplicate response;
- unexpected response;
- successor metadata changed after signing;
- successor program changed after signing;
- sponsor input added after signing;
- fee output changed after signing.

## 16.5 Predecessor-constructor faults · `tab:guide14-exec:predecessor-faults`

Required:

- wrong STATE asset;
- wrong singleton amount;
- wrong predecessor program;
- predecessor metadata reconstructs another program;
- wrong static subtree;
- wrong leaf version;
- wrong internal key;
- wrong control block;
- stale constructor from another bundle;
- metadata leaf missing;
- metadata leaf duplicated;
- operation leaf missing;
- extra escape leaf;
- metadata leaf selected for execution;
- key-path spend attempt.

## 16.6 Successor-constructor faults · `tab:guide14-exec:successor-faults`

Required:

- successor from wrong semantic metadata;
- successor from predecessor metadata unchanged;
- wrong announcement cycle;
- wrong representation nonce;
- later admissible nonce instead of first;
- successor under another static subtree;
- wrong internal key;
- wrong parity;
- wrong leaf version;
- wrong control recipe;
- arbitrary caller-supplied output program;
- no STATE successor;
- two STATE successors;
- extra STATE-like output;
- spendable metadata leaf.

## 16.7 Branch-order and totality faults · `tab:guide14-exec:totality-faults`

Required:

- metadata child on wrong side;
- caller-supplied branch order;
- source-order-dependent tree;
- nonce starts at one;
- nonce skips an admissible value;
- nonce search exceeds bound;
- invalid internal key retried;
- malformed metadata retried;
- missing leaf retried;
- zero tweak rejected merely for zero;
- tweak at or above group order accepted;
- identity result accepted;
- target and oracle parity disagree;
- repeated hashing used as fixed-point search.

## 16.8 Root-history faults · `tab:guide14-exec:history-faults`

Required:

- stale predecessor;
- wrong current-root view;
- two predecessors;
- two successors;
- missing edge;
- duplicate edge;
- successor cursor restored after invalid intermediate edge;
- STATE termination;
- old static subtree to new subtree without migration;
- unrelated STATE-shaped input;
- root cursor points to sponsor change;
- RESV edge;
- PACE edge;
- authority edge;
- transition certificate names another predecessor;
- transition certificate names another successor.

## 16.9 Absence and economic faults · `tab:guide14-exec:absence-faults`

Required:

- add live receipt input or output;
- add time-locked receipt;
- add ASH;
- add entitlement;
- add request;
- add distribution control or vault;
- add RESV input or output;
- add PACE;
- add authority object;
- add issuance;
- add destruction;
- add burn record;
- add clear event;
- add residue projection;
- introduce canonical `U`, `ENT`, or `DIST_CTL` flow.

## 16.10 Sponsor faults · `tab:guide14-exec:sponsor-faults`

Required:

- STATE/sponsor overlap;
- two sponsor envelopes;
- foreign sponsor asset;
- missing sponsor authorization;
- sponsor change at STATE output 0;
- STATE successor in sponsor range;
- fee/change substitution;
- sponsor member unclassified;
- empty sponsor offer for sponsored request;
- report publishes sponsor amount;
- report publishes sponsor opening;
- balanced STATE corruption compensated by sponsor change;
- zero-valued sponsor member under exact role structure;
- confidential sponsor value where the selected target policy claims support.

## 16.11 ABI and linker faults · `tab:guide14-exec:abi-faults`

Required:

- wrong coordinator;
- duplicate coordinator;
- no coordinator;
- STATE and sponsor positions exchanged;
- wrong input count;
- wrong output count;
- wrong transaction version;
- wrong sequence;
- witness item reorder;
- predecessor and successor metadata witnesses exchanged;
- leaf from another bundle;
- control block from another program;
- unresolved metadata-schema symbol;
- unresolved lead-bound symbol;
- unresolved operator symbol;
- relocation omitted;
- relocation applied twice;
- duplicate tree leaf;
- conflicting leaf weight;
- conflicting leaf role;
- candidate outside bounds;
- raw transaction bypassing safe construction;
- target bytes changed after ABI validation.

## 16.12 Protocol and report faults · `tab:guide14-exec:protocol-faults`

Required:

- blank request;
- oversized request;
- unterminated request;
- malformed JSON;
- unknown request field;
- wrong schema;
- wrong environment;
- wrong provenance;
- signing response bound to other bytes;
- accepted submission without identity;
- accepted submission without mined readback where required;
- rejected submission carrying accepted identity;
- rejected signing response carrying witness;
- infrastructure response carrying target observation;
- conservation rejection carrying accepted-only openings;
- caller-authored evidence standing;
- report summary edited;
- failed row removed;
- duplicate passing row;
- observed target layer differs from declared row boundary;
- canonical report includes wall time;
- typed diagnostic contains a caller path or injected line.

## 16.13 Public-recovery faults · `tab:guide14-exec:recovery-faults`

Required:

- missing transaction;
- copied transaction bytes;
- wrong output index;
- stale successor;
- malformed public witness;
- metadata from another successor;
- missing representation nonce;
- wrong static subtree;
- creator-process memory required;
- temporary file required;
- wallet descriptor required;
- unknown publication field;
- reconstructed program differs from chain output.

---

# 17. Root-history evidence · `sec:guide14-exec:history`

## 17.1 Edge sequence · `rule:guide14-exec:edge-sequence`

The root-history validator consumes an ordered sequence of typed edges.

For a single announcement the expected sequence has exactly one member.

It validates:

- predecessor equals the current cursor before the edge;
- successor is unique;
- certificate operation is maturity announcement;
- certificate predecessor and successor agree with the target transaction;
- no unrelated root effect appears;
- final cursor equals the successor only after the edge validates.

## 17.2 Checkpoint binding · `rule:guide14-exec:checkpoint`

Native root-history evidence binds:

- network identity;
- genesis identity;
- block hash;
- block height;
- transaction identity;
- predecessor outpoint;
- successor outpoint;
- target contract;
- linked candidate;
- candidate ABI.

A reorg that removes or changes the transaction stales the observation.

Evidence from another chain prefix is not silently reused.

## 17.3 Bundle continuity · `rule:guide14-exec:bundle-continuity`

Guide 14 implements no constructor migration.

Therefore:

\[\text{predecessor static subtree}\ne\text{successor static subtree}\Rightarrow\text{reject}\]

even when semantic metadata would otherwise match.

---

# 18. Resource study · `sec:guide14-exec:resources`

## 18.1 Candidate parameters · `rule:guide14-exec:candidate-parameters`

Candidate-only parameters include:

- metadata schema size;
- representation-nonce search bound;
- static leaf set;
- tree depth policy;
- sponsor input bound;
- witness limits;
- target validation budget.

No candidate parameter is final calibration.

## 18.2 Complete measurements · `rule:guide14-exec:measurements`

Measure complete transactions for:

- minimum lead;
- maximum lead;
- representative interior lead;
- nontrivial metadata;
- nonce zero;
- nonce nonzero;
- greatest nonce attempt found in the finite corpus;
- sponsorless;
- sponsored without change;
- sponsored with change where supported;
- deepest control path;
- largest metadata witness;
- operator signature;
- candidate maximum sponsor shape.

## 18.3 Prediction and observation · `rule:guide14-exec:resource-comparison`

Predictions derive from exact linked programs and exact candidate bytes.

Observations come from the target.

Both are bound to the same serialization.

A mismatch:

- records both figures;
- returns a typed report failure;
- stops later dependent execution;
- cannot coexist with an overall successful resource result.

Absent observations remain absent.

## 18.4 Separate objectives · `rule:guide14-exec:resource-objectives`

No one transaction is presumed to maximize all dimensions.

The report separately identifies maxima for:

- metadata bytes;
- script bytes;
- control depth;
- witness bytes;
- peak stack;
- hashing;
- tweak checks;
- validation budget;
- transaction weight;
- nonce attempts.

---

# 19. Implementation waves · `sec:guide14-exec:waves`

Each wave ends formatted, focused-test green, documented, and committed before the next begins. A wave that cannot satisfy its exit records a typed stopped result and does not allow dependent waves to proceed.

## Wave 0 — Revalidate the Phase-5 handoff · `task:guide14-exec:wave0`

**Entry**

- clean working tree;
- exact source revision recorded;
- Phase-5 assessment located;
- accepted constructor research located.

**Deliverables**

- reproduce `G14-R01` through `G14-R16`;
- close every confirmed P0/P1;
- record exact protocol revision and active phase consistently;
- identify the complete typed STATE source;
- inventory existing maturity semantics;
- classify every prototype mechanism as promoted, rejected, or still experimental;
- classify every existing signing, evidence, and resource helper intended for reuse;
- record dependency graph;
- answer every repository-ground question in §3;
- prove that the witness carrier of §5.1 is sufficient, or stop for a carrier decision.

**Exit**

- preflight gate passes;
- no confirmed P0/P1 remains;
- complete metadata ownership is known;
- no inherited evidence count depends on wrong-boundary refusals;
- tree clean.

**Suggested commit**

```text
plans: charter the maturity-announcement execution
```

## Wave 1 — Typed STATE metadata and semantic transition · `task:guide14-exec:wave1`

**Deliverables**

- exact semantic `StateMetadata`;
- exact maturity type;
- architecture-owned lead-bound adapter;
- canonical metadata encoding and strict decoder;
- representation nonce separated from semantics;
- `announce_maturity` typed transition;
- complete per-field copy-through tests;
- public witness recovery model;
- model and realization conformance.

**Exit**

- every valid semantic transition derives one successor;
- every invalid maturity or lead rejects;
- every unaffected field is mechanically preserved;
- no target type enters realization.

**Suggested commit**

```text
realization: define maturity-announcement STATE metadata
```

## Wave 2 — Validated compiler operation plan · `task:guide14-exec:wave2`

**Deliverables**

- `ValidatedMaturityAnnouncementOperationPlan`;
- exact relation census;
- source, constructibility, lifecycle, carrier, layout, and coverage projections;
- root-history and public-recovery requirements;
- sponsor-opacity check;
- corruption-resistant validator;
- independent relation/case census oracle;
- public API tests.

**Exit**

- one validated plan derives from equal typed inputs deterministically;
- no relation is missing or duplicated;
- no target field or digest leaks upward.

**Suggested commit**

```text
compiler: expose the validated maturity-announcement plan
```

## Wave 3 — Target and operator-signing closure · `task:guide14-exec:wave3`

**Deliverables**

- exact target capability assessment;
- operator-key encoding closure;
- selected operator profile;
- independent message recomputation;
- target-generic public-test signer capability where needed;
- strict response shape;
- target-native operator-signature control;
- unknown-key negatives;
- protocol revision update on both sides if required.

**Exit**

- one finalized synthetic candidate can be signed by the public test operator;
- returned witness binds exact bytes;
- no production key interface exists;
- no caller-authored profile is accepted as evidence.

**Suggested commit**

```text
target-elements: close maturity operator requirements
```

## Wave 4 — Candidate STATE constructor · `task:guide14-exec:wave4`

**Deliverables**

- metadata commitment leaf;
- always-aborting metadata program;
- exact static subtree type;
- predecessor and successor constructor recipes;
- canonical branch side;
- deterministic nonce search;
- tweak-totality policy;
- internal-key policy;
- control recipes;
- constructor reference graph;
- abstract and oracle tests.

**Exit**

- constructor derives deterministically;
- wrong metadata, root, key, version, parity, and control path reject;
- metadata leaf has no success path;
- no key-path escape is claimed;
- synthetic prototype schema does not leak into STATE.

**Suggested commit**

```text
tapscript: implement the candidate STATE constructor
```

## Wave 5 — Maturity-announcement patterns · `task:guide14-exec:wave5`

**Deliverables**

- coordinator role;
- exact cardinality;
- predecessor recognition;
- metadata authentication;
- maturity predecessor;
- lead-window checks;
- copy-through;
- successor reconstruction;
- operator authorization;
- sponsor isolation;
- issuance and absence closure;
- complete composed program;
- component-derived evidence/source/disclosure metadata;
- final-stack and resource checks.

**Exit**

- every program schedules from its declared witness;
- every relation has a carrier or a named external role;
- no composed pattern under-reports dependencies;
- no unchecked Boolean or signature form survives.

**Suggested commit**

```text
tapscript: implement maturity announcement
```

## Wave 6 — Linker extension · `task:guide14-exec:wave6`

**Deliverables**

- typed STATE symbols;
- two-pass definition and reference resolution;
- explicit SCC strategy;
- structured relocations;
- duplicate-sensitive tree input;
- deterministic exact-cost tree;
- carrier closure;
- candidate linked maturity bundle.

**Exit**

- every mandatory reference resolves exactly once;
- every cycle has an authenticated strategy;
- declaration order does not change bytes;
- no manual post-link patch exists.

**Suggested commit**

```text
linker: link the maturity-announcement constructor
```

## Wave 7 — Candidate maturity ABI · `task:guide14-exec:wave7`

**Deliverables**

- current-STATE view;
- typed request;
- input and output layouts;
- successor metadata derivation;
- constructor search;
- sponsor consistency;
- output finalization;
- signing states;
- witness roles;
- canonical target bytes and strict decoder;
- post-finalization mutation refusals;
- candidate-only status.

**Exit**

- one sponsorless candidate becomes submit-ready;
- sponsored forms become submit-ready where inherited policy claims them;
- every protected mutation after signing rejects;
- the production route is exactly bound to finalization’s leaf and protected bytes.

**Suggested commit**

```text
transaction: derive the maturity-announcement ABI
```

## Wave 8 — Canonical safety evidence · `task:guide14-exec:wave8`

**Deliverables**

- canonical semantic fixture registry;
- complete safety matrix;
- relation and boundary links;
- positive class witnesses;
- first-party refusal evidence;
- target controls;
- accepted semantic projections;
- validated safety report;
- explicit unexpected-boundary standing.

**Exit**

- every safety row is answered or explicitly outstanding;
- no wrong-boundary refusal is counted as answered;
- no infrastructure result is treated as a target verdict;
- controls and mutants are exact-subject bound.

**Suggested commit**

```text
vectors: complete maturity-announcement safety evidence
```

## Wave 9 — Constructor continuity · `task:guide14-exec:wave9`

**Deliverables**

- predecessor constructor projection;
- successor constructor projection;
- static-subtree equality;
- canonical nonce evidence;
- tweak and parity evidence;
- control-path comparison;
- validated continuity report;
- stale and cross-bundle negatives.

**Exit**

- accepted target bytes reconstruct both constructors;
- static continuity and metadata continuity both hold;
- neither is inferred from the other.

**Suggested commit**

```text
vectors: verify STATE constructor continuity
```

## Wave 10 — Root history and public recovery · `task:guide14-exec:wave10`

**Deliverables**

- typed STATE edge sequence;
- current-root and stale-root checks;
- invalid-intermediate-edge cases;
- checkpoint and reorg binding;
- unrelated-process public handoff;
- successor metadata recovery;
- successor program reconstruction;
- synthetic-origin non-claims;
- validated root-history and recovery reports.

**Exit**

- exactly one valid STATE edge is recorded;
- final cursor cannot hide an invalid edge;
- an unrelated process reconstructs output 0;
- no creator-private state is used.

**Suggested commit**

```text
vectors: verify STATE history and public recovery
```

## Wave 11 — Sponsored maturity forms · `task:guide14-exec:wave11`

**Deliverables**

- sponsorless control;
- sponsored control;
- sponsor change absent;
- sponsor change present where supported;
- missing sponsor authorization;
- STATE/sponsor overlap;
- balanced STATE-corruption attempt;
- sponsor-report exclusion tests;
- exact target and policy verdicts.

**Exit**

- sponsorship changes no STATE semantic result;
- no sponsor amount enters protocol evidence;
- every sponsor form is accounted for by acceptance, refusal, or explicit unsupported status.

**Suggested commit**

```text
vectors: exercise sponsored maturity announcement
```

## Wave 12 — Resource study · `task:guide14-exec:wave12`

**Deliverables**

- complete measurements from §18;
- exact linked predictions;
- target observations;
- nonce-search corpus;
- prediction mismatch negative;
- presence-bearing unavailable observations;
- noncanonical timing report;
- explicit non-calibration result.

**Exit**

- prediction equals observation for every claimed dimension;
- no stale Phase-5 blocker is used to explain a lane-local omission;
- no missing measurement is encoded as zero.

**Suggested commit**

```text
vectors: measure the maturity-announcement candidate
```

## Wave 13 — Phase-6 gate and handoff · `task:guide14-exec:wave13`

**Deliverables**

- package READMEs;
- package contracts;
- Phase-6 card;
- roadmap and backlog state;
- complete result matrix;
- identity, schema, security, and dependency impact;
- full repository gate;
- clean final tree.

**Exit**

- §23 passes;
- no stronger claim than the reports support;
- Phase 6 exits only by owner action after the gate record is reviewed.

**Suggested commit**

```text
plans: record the STATE and maturity candidate
```

---

# 20. Verification · `sec:guide14-exec:verification`

## 20.1 Working cadence · `rule:guide14-exec:cadence`

For Rust changes:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Run focused package tests during each wave.

Run the full workspace and Meson gate once per coherent batch, not after every edit.

Documentation-only work still runs plan, label, census, link, and diff checks.

Every run reports wall time separately from canonical output.

## 20.2 Focused package commands · `tab:guide14-exec:focused-tests`

```sh
cargo test --locked -p tripod-architecture
cargo test --locked -p tripod-model maturity
cargo test --locked -p tripod-model root
cargo test --locked -p tripod-model realization_conformance

cargo test --locked -p tripod-realization
cargo test --locked -p tripod-compiler
cargo test --locked -p tripod-target-elements
cargo test --locked -p tripod-tapscript
cargo test --locked -p tripod-linker
cargo test --locked -p tripod-transaction
cargo test --locked -p tripod-vectors
cargo test --locked -p tripod-target-elements-conformance
```

For every changed public package:

```sh
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p <package> --no-deps
```

## 20.3 Required preflight regressions · `tab:guide14-exec:preflight-regressions`

At minimum:

```text
a native refusal answers a row only at its declared boundary
a consensus-before-script refusal cannot answer a script-path row
a key-path refusal cannot answer a script-path row
a report-layer row is not answered merely because it is answerable

a foreign-byte census rejects candidate/protected-byte disagreement
a production census rejects another committed leaf from the same tree
a production census rejects a wrong code-separator position

an empty restart ledger cannot validate a completed closeout
a partial accepted ledger cannot validate a completed closeout
a typed stop must match the ledger’s exact blocker
recording after completion returns a typed refusal

a wrong-blinder case distinguishes cause from serialized field
an arbitrary whole-transaction declared mutation range is refused

crossing fragment accounting follows composition
a composed pattern carries every component evidence dependency

recorded-randomness digest changes when a semantic amount changes
byte-identity and recorded-randomness differ only on opening material

a rejected conservation response cannot carry accepted-only openings
operation resource absence remains absent on the wire
typed diagnostics cannot contain caller paths or injected lines
```

## 20.4 Required maturity regressions · `tab:guide14-exec:maturity-regressions`

At minimum:

```text
minimum and maximum announcement leads accept
one below and one above reject
checked cycle overflow rejects
predecessor already announced rejects
successor changes only maturity
every unaffected field mutation rejects

metadata decode and re-encode are byte-identical
unknown schema and trailing bytes reject
representation nonce is erased semantically
first admissible nonce is selected
nonce search exhaustion is typed
zero tweak is not rejected merely for zero

metadata leaf always aborts
wrong static subtree rejects
wrong predecessor metadata rejects
wrong successor metadata rejects
wrong internal key and parity reject
key-path attempt is observed under its own layer

operator signs exact finalized bytes
wrong operator rejects
unknown nonempty key cannot count as authorization
post-signing metadata and output mutations reject

one STATE predecessor and successor are exact
stale predecessor rejects
invalid intermediate root edge cannot be hidden
RESV participation rejects
public recovery reconstructs output 0
```

## 20.5 Real target matrices · `tab:guide14-exec:target-runs`

Run separately:

```text
maturity semantic safety
operator authorization
predecessor constructor
successor constructor
branch-order and totality
root history
public recovery
sponsor forms
resources
```

Each run records:

- source revision;
- target contract revision;
- node version and binary-reported revision;
- intended integration tip;
- upstream base and local topics;
- protocol revision;
- network and genesis;
- block hash and height;
- linked candidate;
- candidate ABI;
- metadata schema;
- constructor policy;
- nonce-search bound;
- exact requests;
- exact responses;
- exact submitted bytes;
- target verdicts;
- independent projections;
- case and relation counts;
- infrastructure errors;
- wall time outside canonical report bytes.

## 20.6 Full batch gate · `gate:guide14-exec:batch`

After a coherent batch:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Documentation and census:

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

When dependencies changed:

```sh
cargo tree --locked -e features
cargo metadata --locked
cargo audit
```

A missing advisory tool is recorded as skipped, never passed.

When document policy requires it:

```sh
scripts/check-document-reproducibility.sh
```

Final check:

```sh
git status --porcelain=v1 --untracked-files=all
```

It must be empty.

---

# 21. Acceptance criteria · `sec:guide14-exec:acceptance`

Guide 14 is accepted only when:

## Entry and inheritance

- the complete working tree, not only the original 51-file selection, was reviewed;
- all `G14-R01`–`G14-R16` rows were dispositioned;
- every confirmed P0/P1 row is closed;
- Phase-5 inheritance is revalidated under exact boundary equality;
- the accepted constructor prototype was re-read against the current target;
- no historical report is treated as current execution evidence;
- the tree was clean at entry.

## Semantic STATE

- complete STATE metadata has one typed owner;
- predecessor STATE is current, canonical, singleton, and unannounced;
- requested cycle lies inside the inclusive lead window;
- checked arithmetic does not wrap or clamp;
- successor maturity is exactly `Announced { cycle = requested }`;
- every unaffected semantic field is identical;
- no representation-only field changes semantic projection;
- no RESV or other root participates;
- no economic value family moves;
- one exact transition certificate is derived.

## Constructor

- metadata encoding is canonical;
- metadata leaf commits exact bytes;
- metadata leaf always aborts;
- static subtree is exact and unchanged;
- predecessor constructor authenticates;
- successor constructor reconstructs;
- branch side is canonical;
- nonce search starts at zero and selects the first admissible result;
- search exhaustion is typed;
- zero tweak follows the target rule;
- out-of-range tweak and identity result reject;
- internal-key policy is exact;
- no accepted key-path escape exists;
- every constructor cycle has an authenticated strategy;
- no repeated-hash fixed-point search exists.

## Compiler and target

- realization and compiler relation censuses agree;
- every target requirement is classified;
- every selected proof is realization-approved;
- every active relation has a reachable carrier;
- operator source requirements are exact;
- root-history and public-recovery requirements are explicit;
- no target type enters compiler core;
- no capability is counted as evidence.

## Linking and ABI

- every symbol is typed;
- every mandatory reference resolves exactly once;
- duplicate and conflicting leaves reject;
- tree construction is deterministic;
- exact or checked tree-cost arithmetic is used;
- carrier closure holds;
- input 0 is the sole STATE coordinator;
- output 0 is the sole STATE successor;
- sponsor suffix and fee roles are exact;
- successor program is derived, not caller-supplied;
- output set is finalized before signing;
- canonical decode/re-encode equality holds;
- candidate bundle and ABI remain non-final.

## Authorization

- operator key encoding is approved and curve-valid;
- successful announcement carries an actually verified operator signature;
- selected profile commits every protected field;
- signing request is bound to exact finalization, leaf, deployment, and bytes;
- wrong operator, profile, input, leaf, or candidate rejects;
- post-signing metadata or program mutation rejects;
- no production private key enters any interface.

## Evidence

- canonical evidence plans have no unchecked constructors;
- canonical coverage consumes validated reports only;
- native refusals answer rows only at their declared boundary;
- unexpected earlier refusals remain visible and unanswered;
- every positive class has a typed witness;
- every accepted target transaction receives independent semantic comparison;
- constructor continuity is independently checked;
- root history is edge-based;
- public recovery succeeds without creator state;
- report-layer evidence is observed or structurally discharged, never merely answerable;
- infrastructure failure carries no target observation;
- no mock is gate-eligible.

## Resources and repository

- complete candidate transactions are measured;
- prediction equals target observation where claimed;
- missing observations remain absent;
- nonce attempts and wall time stay noncanonical;
- no final calibration is claimed;
- no speculative digest is minted;
- no new dependency entered without review;
- report bytes reproduce;
- full repository gates pass;
- final tree is clean.

---

# 22. Rejection criteria · `sec:guide14-exec:rejection`

Reject or typed-stop the candidate if:

- no authoritative typed STATE metadata census exists;
- model or prose is parsed as semantic source;
- a prototype field silently becomes STATE;
- one semantic relation disappears between packages;
- predecessor is stale, noncanonical, or already announced;
- announcement lead is outside bounds;
- cycle arithmetic wraps, saturates, or clamps;
- an unrelated semantic field changes;
- successor metadata is independently caller-authored;
- constructor representation affects semantic STATE;
- a later nonce is accepted while an earlier one works;
- nonce search repairs semantic invalidity;
- metadata leaf is spendable;
- predecessor or successor static subtree differs;
- constructor cycle has no explicit strategy;
- repeated hashing is used as fixed-point search;
- zero tweak is rejected merely for zero;
- wrong key, parity, leaf version, or control path passes;
- key-path refusal is filed as script-path evidence;
- operator signature is missing, unverified, or bound to other bytes;
- another committed leaf from the same tree can be substituted into production signing;
- output changes after signing;
- sponsor amount becomes protocol data;
- a sponsored request silently becomes sponsorless;
- target acceptance substitutes for semantic comparison;
- semantic comparison substitutes for constructor continuity;
- final cursor substitutes for root history;
- report answerability substitutes for observed report evidence;
- a wrong-boundary refusal is counted as answered;
- caller-authored mutation bounds establish attribution;
- infrastructure failure counts as target rejection;
- target rejection carries accepted-only artifacts;
- absent resource observations become zero;
- typed diagnostics contain paths or unescaped request text;
- candidate and final status are conflated;
- a digest is added without an admitted consumer;
- any required gate fails or leaves a dirty tree.

---

# 23. Phase-6 exit gate · `gate:guide14-exec:exit`

Phase 6 exits when all of the following hold:

1. the Phase-5 handoff has been revalidated and every confirmed preflight blocker is closed;
2. the exact typed maturity-announcement semantics are implemented;
3. canonical STATE metadata and representation separation are complete;
4. predecessor and successor authenticate one exact static subtree;
5. metadata leaf, key path, wrong-key, wrong-root, wrong-parity, and wrong-control escapes reject;
6. operator authorization binds one exact finalized transaction;
7. one sponsorless positive control is accepted by a real target;
8. every claimed sponsored form has an explicit disposition;
9. every accepted transaction matches the expected semantic STATE;
10. every accepted transaction matches constructor continuity;
11. exactly one STATE root edge is independently recovered;
12. no invalid intermediate root edge can be hidden by final cursor equality;
13. an unrelated process reconstructs the successor from public chain data;
14. no RESV, issuance, destruction, economic flow, or specialized event appears;
15. safety, continuity, history, recovery, and resources remain separate reports;
16. every negative row is answered at its declared boundary or remains explicitly outstanding;
17. resource prediction equals target observation;
18. lifecycle incompleteness remains explicit;
19. no production key custody, deployment, calibration, release, or universal security claim is made;
20. no speculative identity is minted;
21. canonical report bytes reproduce;
22. the full repository gate passes;
23. the final working tree is clean.

A typed stopped result is valid and does not satisfy this exit gate.

The phase card moves to `Exited` only through a separate owner-reviewed closeout after this gate’s evidence is recorded.

---

# 24. Identity, schema, security, and dependency impact · `sec:guide14-exec:impact`

## 24.1 Identity

Expected:

```text
Attestation version:
    unchanged

realization version:
    unchanged unless a semantic correction independently requires movement

architecture schema and semantic hashes:
    unchanged if maturity semantics and metadata are already represented

compiler identity:
    none minted

STATE constructor identity:
    none minted

bundle identity:
    none minted for in-process candidate use

ABI identity:
    none minted

report identities:
    none minted

deployment profile:
    dormant
```

Exact transaction identities from observed native runs are evidence locators, not newly designed content-identity schemes.

## 24.2 Schema

Expected typed schema additions:

```text
canonical STATE metadata representation
validated maturity-announcement operation plan
candidate STATE constructor
candidate maturity tapscript plan
candidate linked bundle
candidate maturity ABI
target-generic public-test signing step where needed
maturity safety report
constructor-continuity report
root-history report
public-recovery report
resource report
```

Any native-protocol change increments the protocol revision in Rust and Python together.

Historical protocol revisions are not widened silently.

## 24.3 Security

Permitted:

- public canonical STATE metadata;
- public representation nonce;
- public NUMS internal key;
- public disposable test operator scalar;
- public disposable test signatures;
- disposable development node state.

Not permitted:

- production operator private keys;
- production signer credentials;
- production wallet seeds;
- production node credentials;
- production deployment authority;
- secret input through command-line arguments;
- diagnostic publication of arbitrary child or request text.

## 24.4 Dependencies

No new third-party dependency is expected.

Any proposal records:

- concrete consumer;
- exact source and version;
- features;
- transitive graph;
- licence;
- MSRV;
- unsafe and FFI boundary;
- build scripts;
- determinism implications;
- advisory status;
- lockfile impact;
- public API leakage.

No new library edge between semantic core and target conformance is admitted merely to reuse an oracle.

## 24.5 Interchange

In-process typed values are not interchange documents.

Native executor records remain under the existing protocol contract.

Rendered Guide-14 reports are terminal audit closures until a real external consumer exists.

A parser that consumes one back into a decision activates ADR-022 before merging.

---

# 25. Completion report template · `sec:guide14-exec:report-template`

```text
Guide 14 result
===============

Starting state:
    source revision:
    working tree:
    Phase-5 result revision:
    Phase-5 revalidation:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    realization version:
    target contract revision:
    executor protocol revision:
    constructor-research revision:

Preflight:
    G14-R01:
    G14-R02:
    G14-R03:
    G14-R04:
    G14-R05:
    G14-R06:
    G14-R07:
    G14-R08:
    G14-R09:
    G14-R10:
    G14-R11:
    G14-R12:
    G14-R13:
    G14-R14:
    G14-R15:
    G14-R16:
    metadata authority:
    prototype promotion:
    metadata carrier:
    constructor cycle:
    operator profile:
    clean tree:

Semantic STATE:
    predecessor:
    predecessor maturity:
    current cycle:
    announced cycle:
    minimum lead:
    maximum lead:
    successor:
    successor maturity:
    semantic field census:
    unaffected fields preserved:
    STATE root effect:
    RESV effect:
    other root effects:
    economic flows:
    issuance:
    destruction:
    transition certificate:

Metadata:
    typed source:
    schema:
    domain:
    canonical field order:
    semantic fields:
    representation fields:
    reserved fields:
    canonical encoding:
    strict decoder:
    decode/re-encode equality:
    public witness carrier:

Compiler:
    validated plan:
    operation:
    relation census:
    source requirements:
    operator requirements:
    root requirements:
    constructibility:
    lifecycle:
    carriers:
    layout:
    coverage:
    external evidence:
    target types leaked:
    graph handles leaked:
    identity minted:

Target:
    reviewed definition:
    deployment:
    execution domain:
    leaf version:
    hashing:
    byte operations:
    tweak verification:
    signature semantics:
    selected sighash:
    input/output introspection:
    transaction conservation:
    fee role:
    missing capabilities:
    external requirements:

Constructor:
    metadata leaf:
    metadata leaf success set:
    static subtree:
    branch side:
    representation nonce:
    search bound:
    first admissible nonce:
    tweak:
    internal key:
    output key:
    predecessor program:
    successor program:
    control recipes:
    reference graph:
    SCCs:
    cycle strategies:
    outstanding assumptions:

Tapscript:
    coordinator:
    cardinality:
    STATE recognition:
    metadata authentication:
    maturity predecessor:
    announcement window:
    copy-through:
    successor reconstruction:
    operator authorization:
    sponsor isolation:
    issuance absence:
    root/event closure:
    final stack:
    evidence dependencies:
    resource projection:

Linker:
    symbols:
    definitions:
    references:
    relocations:
    duplicate-leaf result:
    exact cost:
    taptree:
    carrier closure:
    deterministic rebuild:
    candidate bundle:

ABI:
    input layout:
    output layout:
    current-STATE view:
    request:
    successor derivation:
    constructor search:
    sponsor form:
    output finalization:
    signing request:
    witness roles:
    canonical bytes:
    strict decoder:
    post-signing mutation result:
    candidate status:

Operator:
    public test identity:
    exact bytes signed:
    selected profile:
    returned witness:
    exact binding:
    wrong operator:
    wrong candidate:
    wrong leaf:
    empty signature:
    malformed signature:
    unknown key:
    production key claim:
        none

Synthetic STATE:
    funding method:
    target outpoint:
    metadata:
    constructor:
    protocol genesis claim:
        none
    prior root-history claim:
        none
    trusted-setup claim:
        none
    production STATE claim:
        none

Safety:
    row count:
    answered:
    outstanding:
    first-party:
    native acceptances:
    native refusals:
    unexpected-boundary refusals:
    infrastructure errors:
    accepted semantic projections:
    canonical bytes:

Constructor continuity:
    predecessor reconstructed:
    successor reconstructed:
    static subtree equal:
    metadata continuity:
    representation nonce canonical:
    tweak/parity:
    control paths:
    residual assumptions:
    canonical bytes:

Root history:
    checkpoint:
    predecessor cursor:
    successor cursor:
    edge count:
    edge sequence:
    stale predecessor:
    duplicate successor:
    invalid intermediate edge:
    STATE termination:
    RESV edge:
    other root edges:
    canonical bytes:

Public recovery:
    independent process:
    transaction located:
    exact bytes matched:
    witness decoded:
    predecessor metadata recovered:
    successor metadata derived:
    representation nonce recovered:
    successor constructor reconstructed:
    output program matched:
    creator-private dependency:
        none
    canonical bytes:

Sponsor:
    sponsorless:
    sponsored:
    sponsor change absent:
    sponsor change present:
    missing authorization:
    STATE overlap:
    balanced corruption:
    amount disclosure:
        none

Resources:
    metadata bytes:
    metadata leaf bytes:
    announcement leaf bytes:
    static subtree bytes:
    constructor bytes:
    tree depth:
    control bytes:
    witness bytes:
    operator signature bytes:
    sponsor witness bytes:
    peak main stack:
    peak alternate stack:
    largest element:
    hash work:
    tweak work:
    validation budget:
    transaction weight:
    virtual size:
    nonce attempts:
    prediction/observation:
    absent observations encoded as zero:
        no
    final calibration claim:
        none

Lifecycle:
    announce-maturity:
        candidate implemented
    burn:
        outstanding
    clear:
        outstanding
    redemption:
        outstanding
    admission:
        outstanding
    settlement:
        outstanding
    cycle:
        outstanding
    migration:
        outstanding
    release-complete:
        false

Security:
    public disposable operator:
    production operator keys accepted:
        no
    production wallets accepted:
        no
    diagnostics path-safe:
    arbitrary child output quarantined:
    future secret-bearing review required:

Identity:
    Attestation:
    realization:
    architecture:
    compiler:
        none
    constructor:
        none
    bundle:
        none
    ABI:
        none
    reports:
        none
    deployment profile:
        dormant

Dependencies:
    first-party changes:
    third-party additions:
    Cargo.lock:
    licences:
    MSRV:
    unsafe/FFI:
    advisories:

Verification:
    cargo fmt:
    workspace clippy:
    workspace tests:
    architecture:
    model:
    realization:
    compiler:
    target-elements:
    tapscript:
    constructor oracle:
    linker:
    transaction:
    vectors:
    target-elements-conformance:
    Rustdoc:
    protocol cross-language:
    maturity native matrix:
    operator native matrix:
    constructor native matrix:
    root-history native matrix:
    public-recovery native matrix:
    sponsor native matrix:
    resource native matrix:
    scripts/check-plans.sh:
    meson lint:
    scripts/ci.sh:
    meson compile:
    meson test:
    document reproducibility:
    cargo audit:
    git diff --check:
    final git status:

Phase-6 verdict:
    accepted candidate
    / typed stopped
    / constructor deferred
    / target path rejected
    / blocked

Residuals:

Handoff:
```

---

# 26. Handoff after Guide 14 · `sec:guide14-exec:handoff`

If Guide 14 succeeds, Phase 6 has:

- one canonical typed STATE metadata representation;
- one candidate metadata-dependent STATE constructor;
- one operator-authorized maturity-announcement path;
- one target-accepted mutable-root transaction;
- one independently checked STATE succession edge;
- one unrelated-process successor reconstruction;
- one complete candidate-specific safety and resource disposition.

The next phase consumes these boundaries rather than reopening them:

```text
Phase 7 — Burn, ASH, and Clear
```

Phase 7 adds:

- owner-authorized burn;
- fresh ASH construction;
- public ASH maintenance;
- permissionless clear;
- STATE mutation by a permissionless operation;
- `tag-recon` destruction;
- residual ASH;
- burn and clear projections;
- interaction between mutable STATE and public amount-dependent semantics.

Guide 14 makes none of those claims.

---

## Closing statement · `rem:guide14-exec:closing`

> Maturity announcement changes one semantic field and therefore tests nearly every implementation boundary around it. The guide is complete only when the current canonical STATE root is authenticated, one exact announced successor is derived from typed metadata, every unaffected field is preserved, one operator authorizes one finalized transaction, predecessor and successor share one exact static constructor relation, branch order and tweak totality are deterministic, the target accepts the exact bytes, one root-history edge is independently recovered, an unrelated process reconstructs the successor from public chain data, every negative result is credited only at its declared boundary, and no candidate artifact is allowed to masquerade as production STATE, final calibration, or release.
