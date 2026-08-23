# Draft: Guide 14 Concept — End-to-End STATE and Maturity Announcement

> **Status:** Concept draft; not an execution guide
> **Phase:** Phase 6 — STATE Constructor and Maturity Announcement
> **Entry:** (`gate:phase5:exit`) and the accepted STATE-constructor research result
> **Primary semantic operation:** `announce-maturity`
> **Affected packages:** `realization`, `compiler`, `target-elements`, `tapscript`, `linker`, `transaction`, `vectors`
> **Evidence support:** the target-executor, signer-handoff, validated-operation-report, safety-evidence, and resource-comparison boundaries established by Phases 4 and 5; `target-elements-conformance` only where the inherited executor interface remains target-generic
> **May affect after acceptance:** package contracts, Phase-6 card, backlog, STATE-constructor research, signer boundary, root-history evidence, Meson graph, dependency graph
> **Supersedes as concept direction:** none
> **Does not implement:** cycle, clear, redemption, admission, settlement, RESV succession, PACE succession, entitlement or distribution authority succession, production wallets, production private-key custody, production deployment, final calibration, or release
> **Required result:** one complete candidate compiler-to-target implementation of maturity announcement, with authenticated mutable STATE continuity, exact operator authorization, exact successor metadata, canonical STATE root succession, no RESV participation, public successor reconstruction, real target execution, complete relation-indexed safety evidence, root-history evidence, resource measurements, and explicit non-production lifecycle status
> **Authority:** Attestation, Realization, typed architecture, implemented ADRs, accepted decisions, accepted STATE-constructor research, the accepted Guide-11 representation policy, and the Phase-4 and Phase-5 candidate results take precedence over this concept
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015

---

## Mission · `sec:guide14:mission`

Guide 14 extends the compiler-to-target pipeline from root-free operations to the first operation that authenticates and advances mutable protocol STATE:

```text
typed architecture
    ↓
target-independent maturity-announcement realization
    ↓
validated compiler analysis
    ↓
validated maturity-announcement target-operation plan
    ↓
reviewed Elements target assessment
    ↓
STATE-constructor and announcement patterns
    ↓
candidate relocatable bundle
    ↓
deterministic linking
    ↓
candidate STATE transaction and witness ABI
    ↓
operator-authorized complete Elements transaction
    ↓
real target execution
    ↓
accepted semantic and root-history projection
    ↓
constructor-continuity and relation-indexed evidence
```

The operation consumes the current canonical STATE object and creates its canonical successor:

```text
STATE(Unannounced)
    ↓
STATE(Announced { cycle = a })
```

It changes one semantic field:

```text
maturity:
    Unannounced
        →
    Announced { cycle = a }
```

It preserves every other STATE field exactly.

It additionally requires:

- the announced cycle lies inside the architecture-owned lead window;
- one authorized operator approves the finalized transaction;
- the predecessor is the current canonical STATE root;
- exactly one canonical STATE successor is created;
- predecessor and successor share one authenticated static constructor identity;
- the successor metadata is canonical and publicly recoverable;
- no RESV, PACE, entitlement authority, or distribution authority participates;
- no issuance, destruction, receipt movement, ASH movement, or specialized event occurs;
- the transition certificate derives one exact STATE succession edge;
- optional fee sponsorship remains isolated and amount-opaque.

Guide 14 is not complete merely because an output program is a valid taproot program or because a node accepts the transaction. The successful result is the conjunction:

\[\text{target accepted}\land\text{STATE projection matched}\land\text{root-history edge matched}\land\text{constructor continuity matched}\]

---

## One-line thesis · `rem:guide14:thesis`

> Guide 14 succeeds when one unannounced canonical STATE root becomes one announced canonical STATE successor under an exact public constructor, with only the maturity field changed, an authorized operator signing one finalized transaction, every constructor and root-history seam checked independently, no RESV or economic value relation disturbed, and all artifacts remaining visibly candidate-only and lifecycle-incomplete.

---

# 1. Governing rulings · `sec:guide14:rulings`

## 1.1 Guide 14 consumes the earlier pipeline · `rule:guide14:consume-pipeline`

Guide 14 consumes rather than redesigns without cause:

- the validated compiler target-operation boundary;
- representation-specific compiler planning;
- typed target capability assessment;
- typed tapscript pattern interface;
- relocatable bundle interface;
- typed linker symbols and references;
- deterministic taptree policy;
- candidate linked-bundle state;
- candidate transaction ABI framework;
- finalized-transaction signing requests;
- target-generic executor supervision;
- strict bounded protocol framing;
- exact target and deployment transcript binding;
- validated operation-report trust states;
- canonical versus experimental evidence subjects;
- exact accepted-projection comparison;
- relation-indexed coverage;
- resource prediction and observation comparison;
- candidate-versus-final type separation.

An inherited interface changes only when STATE presents a concrete fact it cannot express. The change records:

- the missing fact;
- why the inherited boundary cannot represent it;
- the smallest generalization;
- compact-ASH and live-transfer regression impact;
- identity, schema, evidence, and dependency impact.

Guide 14 must not introduce a second compiler plan, linker, transaction model, executor, or report validator merely because STATE is the first mutable root.

## 1.2 STATE is a semantic root, not a convenient output · `rule:guide14:state-root`

The STATE object is the architecture-owned canonical root of mutable economic state.

A transaction carrying STATE-shaped bytes is not thereby the STATE transition.

An accepted announcement must establish:

```text
consumed STATE:
    the current canonical STATE root

created STATE:
    exactly one canonical successor

root edge:
    Succ { input = current STATE, output = successor STATE }

other STATE-like objects:
    absent
```

The following do not substitute for root succession:

- equal metadata values under another constructor;
- another output carrying the STATE asset;
- a transaction consuming an old STATE root;
- a transaction creating two possible successors;
- a final root cursor that happens to equal the expected one after an invalid intermediate edge.

## 1.3 Every relation survives the pipeline · `rule:guide14:relation-census`

For the maturity-announcement scope, require exact equality among:

```text
realization relation census
=
compiler analyzed relation census
=
compiler target-operation-plan relation census
=
backend selected-proof relation census
=
backend emitted-placement relation census
=
linked reachable-carrier relation census
=
ABI relation census
=
operation safety evidence relation census
=
constructor-continuity evidence relation census
=
root-history evidence relation census
```

A package may add target-owned structural obligations. It may not:

- drop a semantic relation;
- merge STATE identity with constructor identity;
- merge operator authorization with root succession;
- merge metadata preservation with transaction acceptance;
- omit an outstanding future STATE operation;
- treat the final root cursor as proof of every intermediate edge;
- turn target capability into evidence completion.

## 1.4 Semantic state and representation metadata stay distinct · `rule:guide14:state-representation`

STATE metadata contains semantic fields and may contain a canonical representation nonce required by the target constructor.

Those two classes remain distinct:

```text
semantic STATE:
    protocol meaning

representation nonce:
    public deterministic spelling of that meaning
```

Changing only the representation nonce must not change semantic STATE.

The successor operation changes only the maturity field. A constructor nonce chosen to satisfy target branch ordering or tweak totality is not a second state transition.

Semantic projection erases the nonce.

Constructor evidence retains it because target bytes depend on it.

## 1.5 The constructor is authenticated in both directions · `rule:guide14:constructor-continuity`

The transaction proves both:

```text
predecessor program
    commits to predecessor metadata and the selected static subtree

successor program
    commits to successor metadata and the same selected static subtree
```

The successor is not accepted merely because:

- it is a valid taproot program;
- it uses the same internal key;
- it contains an announcement leaf;
- it hashes to some well-formed output key;
- the caller says it came from the predecessor.

Static code continuity is exact typed and byte-level continuity over the linked constructor recipe.

Metadata continuity is semantic equality over every unaffected field plus the exact permitted maturity change.

## 1.6 One operator authorizes one finalized transaction · `rule:guide14:operator-authorization`

The maturity announcement is operator-authorized.

The operator signs only after:

- the predecessor STATE input is fixed;
- the successor STATE output is fixed;
- the announced cycle is fixed;
- predecessor and successor metadata encodings are fixed;
- constructor nonces are fixed;
- successor output program is fixed;
- sponsor inputs are fixed where the selected sighash commits them;
- sponsor change is fixed;
- fee role is fixed;
- transaction version and locktime are fixed;
- every target proof or output witness covered by the selected sighash is fixed.

An operator signature over one output set cannot authorize another.

No target forward-compatibility key form may satisfy authorization without actual verification.

## 1.7 No hidden semantic edit · `rule:guide14:single-field-change`

Let \(S\) be predecessor STATE and \(S'\) the successor.

Guide 14 requires:

\[\operatorname{maturity}(S)=\operatorname{Unannounced}\]

\[\operatorname{maturity}(S')=\operatorname{Announced}(a)\]

For every other semantic STATE field \(f\):

\[f(S')=f(S)\]

The complete field census comes from the typed STATE source. It is never restated as a hand-maintained list in the backend.

A transaction changing maturity correctly while also changing one economic field is invalid.

## 1.8 Announcement lead is exact and checked · `rule:guide14:announcement-window`

Let:

- \(c\) be the current cycle committed by predecessor STATE;
- \(a\) be the announced maturity cycle;
- \(L_{\min}\) be the architecture-owned minimum announcement lead;
- \(L_{\max}\) be the architecture-owned maximum announcement lead.

The accepted relation is:

\[c+L_{\min}\le a\le c+L_{\max}\]

All additions are checked in the architecture-owned cycle domain.

Overflow is a semantic or construction failure, not a wrapped cycle and not a target rejection attributed to another relation.

The backend does not copy the lead constants. It consumes typed bound references and linked values from their owning layer.

## 1.9 Root and value relations remain separate · `rule:guide14:root-value-separation`

Maturity announcement advances STATE and moves no protocol value object.

It must not:

- consume or create `U`;
- consume or create receipts;
- consume or create ASH;
- consume or create entitlements;
- consume or create distribution control or vault objects;
- consume RESV;
- issue or destroy any closed asset.

The singleton STATE asset is preserved through root succession. That is a root relation, not a value-flow relation to be merged with transfer conservation.

## 1.10 Sponsor opacity survives STATE mutation · `rule:guide14:sponsor-opacity`

Optional fee sponsorship remains separate from STATE semantics.

A STATE program, ABI, canonical report, or future identity must not require an individual sponsor amount to be:

- explicit for protocol use;
- decoded;
- opened;
- compared with zero;
- proved positive;
- publicly aggregated;
- emitted in diagnostics;
- emitted in canonical reports.

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
independent enforcement of exact STATE succession
```

A balanced-theft or balanced-state-corruption attempt remains mandatory: changing sponsor change to preserve whole-transaction balance cannot excuse changing STATE metadata or omitting the canonical successor.

## 1.11 Construction failure and target rejection remain distinct · `rule:guide14:failure-layers`

Guide 14 distinguishes:

```text
semantic-request rejection

compiler-plan rejection

constructor derivation rejection

linker rejection

ABI/construction rejection

signer refusal

executor infrastructure failure

consensus rejection before script

script-path rejection

relay-policy rejection

accepted target transaction

report-layer STATE-projection rejection

report-layer root-history rejection

report-layer constructor-continuity rejection
```

Examples:

- an announcement outside the architecture window is semantic invalidity;
- nonce-search exhaustion is constructor failure;
- a missing symbol is linker failure;
- a missing operator signature is signing or script-path failure depending on whether complete bytes were produced;
- an invalid control block is target rejection;
- an accepted transaction with changed unrelated STATE is report-layer semantic rejection.

No layer is inferred from what a test expected.

## 1.12 Candidate and final states remain distinct · `rule:guide14:candidate-state`

Guide 14 may produce:

```text
ValidatedMaturityAnnouncementOperationPlan

CandidateStateConstructor

CandidateRelocatableMaturityBundle

CandidateLinkedMaturityBundle

CandidateMaturityAnnouncementAbi

CandidateMaturitySafetyReport

CandidateStateContinuityReport

CandidateRootHistoryReport
```

It must not produce:

```text
FinalStateConstructor

FinalLinkedBundle

FinalTransactionAbi

ProductionOperatorInterface

ProductionStateDeployment

ValidatedDeploymentRelease
```

The candidate constructor is lifecycle-incomplete because clear, redemption, admission, and cycle remain unimplemented.

## 1.13 No speculative identity · `rule:guide14:no-speculative-identity`

Typed in-process values use exact typed comparison.

Exact metadata, program, transaction, control-block, and report bytes use exact byte comparison where those bytes are the subject.

Guide 14 mints no:

```text
StateConstructorHash
MaturityPlanHash
MaturityBundleHash
MaturityAbiHash
StateContinuityReportHash
RootHistoryReportHash
```

A digest enters only when a real cache, process, publication, distribution, deployment, or signing consumer appears and the identity-adjudication discipline adopted by ADR-021 admits it.

No candidate type reserves a future digest field.

## 1.14 Exactness and deterministic failure · `rule:guide14:exactness`

Guide 14 uses:

- checked bounded integers;
- exact field and metadata equality;
- exact target bytes;
- exact finite sets and maps;
- exact symbol and relation censuses;
- exact deterministic nonce search;
- exact target verdicts;
- independently checked constructor and root-history projections.

A search exceeding its explicit budget returns a typed failure and no partial constructor.

It must not:

- wrap a cycle;
- clamp an invalid lead;
- keep the best nonce found before exhaustion;
- fall back to a caller-supplied output program;
- accept a shorter metadata field census;
- drop an unmatched root edge;
- publish incomplete evidence as complete.

---

# 2. Entry conditions · `sec:guide14:entry`

## 2.1 Phase-5 entry · `gate:guide14:phase5-entry`

Required:

- Phase 5 has passed on the current tree;
- live transfer has one complete owner-authorized candidate path;
- finalized-output signing is implemented and target-evidenced;
- owner-key encoding constraints close the unknown-key success path;
- the compiler, linker, transaction, executor, and validated-report boundaries remain green;
- candidate and final states remain distinct;
- the tree is clean.

Guide 14 must not begin against a Phase-5 concept or a report that claims multi-owner authorization without validated target execution.

## 2.2 Semantic entry · `gate:guide14:semantic-entry`

Required:

- architecture release validation passes;
- `announce-maturity` remains in architecture and realization scope;
- maturity-announcement realization validates against architecture;
- compiler analysis remains complete;
- predecessor maturity is typed `Unannounced`;
- successor maturity is typed `Announced { cycle }`;
- the announcement lead bounds remain architecture-owned;
- STATE root succession is explicit;
- RESV and every other root are forbidden;
- optional sponsorship remains amount-opaque;
- no target-specific type has entered realization or compiler core.

## 2.3 Constructor-research entry · `gate:guide14:constructor-entry`

Required:

- the accepted Guide-10 STATE-constructor result is re-read against the current target contract;
- its synthetic metadata is not silently reused as production STATE metadata;
- branch-order policy is exact;
- metadata-leaf unspendability is exact;
- internal-key policy is exact;
- tweak-totality policy is exact;
- canonical representation nonce semantics are exact;
- the linker can represent every constructor reference;
- every prototype assumption is either promoted explicitly or rejected.

Prototype acceptance is not production-pattern admission.

## 2.4 Target entry · `gate:guide14:target-entry`

Required:

- reviewed target definition validates;
- development binding is welded to that exact target;
- streaming hash, byte-string, stack, curve, and tweak prerequisites are reviewed where the constructor uses them;
- selected signature and sighash profile is reviewed and target-evidenced;
- input and output asset/program introspection is reviewed;
- target transaction-form requirements are known;
- execution domain and leaf version are observed;
- policy and consensus verdicts remain separate;
- no mock is gate-eligible.

## 2.5 Evidence entry · `gate:guide14:evidence-entry`

Required:

- canonical evidence plans cannot be caller-forged;
- canonical coverage changes only from validated transcript-derived evidence;
- executor requests contain no expected result;
- transcripts bind exact target, deployment, request, and response subjects;
- operation reports retain the generic execution transcript;
- failed canonical cases fail the gate;
- infrastructure failure never counts as target rejection;
- target rejection carries no success artifact;
- response roles and verdicts are exact;
- strict bounded framing holds in both directions;
- report bytes reproduce;
- safety, constructor continuity, root history, and resources remain distinct report roles.

## 2.6 Repository entry · `rule:guide14:repository-entry`

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

Record:

```text
starting revision
Phase-5 result revision
architecture schema and semantic hash
realization version
target contract revision
executor protocol revision
accepted STATE-constructor research revision
current compiler maturity-announcement projection
clean-tree result
```

If the tree is not clean, stop and classify every change before proceeding.

---

# 3. Preflight questions · `sec:guide14:preflight`

## 3.1 STATE metadata census · `q:guide14:metadata-census`

What is the complete typed STATE field census?

The execution guide must identify:

- the owning typed source;
- canonical field order;
- exact field widths and domains;
- semantic fields;
- representation-only fields;
- reserved fields;
- unknown-field behavior;
- successor copy-through rule;
- target witness or chain-publication location.

The backend must not derive the census from model source text or realization prose.

## 3.2 Constructor promotion · `q:guide14:constructor-promotion`

Which exact Guide-10 prototype mechanisms are promoted?

For each mechanism, record:

```text
prototype mechanism
STATE requirement it now serves
prototype assumptions still valid
new STATE-specific inputs
target-native evidence required
status after promotion
```

At minimum adjudicate:

- dynamic metadata leaf;
- static code subtree;
- metadata-leaf unspendability;
- fixed branch-side policy;
- canonical representation nonce;
- tweak-totality retry;
- internal key;
- control-block parity;
- successor reconstruction.

## 3.3 Static-subtree self-reference · `q:guide14:constructor-cycle`

Does the STATE operation program depend on the static subtree root that commits to that same program?

If so:

- compute the reference graph;
- identify the SCC;
- classify every cyclic edge;
- require one authenticated strategy that cuts the cycle;
- reject repeated hashing until stabilization;
- reject externally asserted equality without an outstanding obligation.

The compact-ASH identity-introspection solution is reused only if it proves the STATE constructor’s different relation.

## 3.4 Metadata publication · `q:guide14:metadata-publication`

How does a future unrelated process recover successor STATE metadata?

Candidate sources include:

```text
the announcement transaction witness
a canonical unspendable data output
a constructor-specific public record
another typed chain-publication role
```

The choice must provide:

- exact canonical bytes;
- unambiguous association with the successor;
- no creator-local dependency;
- deterministic recovery;
- no external report silently standing in for chain data.

## 3.5 Branch ordering · `q:guide14:branch-order`

Does the constructor require:

\[h_{\mathrm{meta}}\le h_{\mathrm{static}}\]

so the program can hash:

```text
metadata leaf hash
then static subtree root
```

without a target primitive that lexicographically orders byte strings?

If yes:

- the representation nonce is ground deterministically;
- the nonce is semantically erased;
- the search bound is explicit;
- search exhaustion is typed;
- target-native tests cover both accepted and rejected nonces;
- the program proves the fixed side rather than assuming it.

If no, the replacement must identify the reviewed target mechanism that performs canonical branch ordering.

## 3.6 Tweak totality · `q:guide14:tweak-totality`

What happens when the taproot tweak is not below the group order or the tweaked key is the identity?

The exact target rule is:

```text
tweak < group order:
    admitted, including tweak = 0

tweaked key = identity:
    refused
```

Zero tweak must not be rejected merely for being zero.

The execution guide chooses and records one policy:

```text
reject instance

canonical public nonce retry with explicit bound

named negligible residual
```

No policy is selected silently by a helper default.

## 3.7 Operator signing boundary · `q:guide14:operator-signing`

How does the target-generic executor expose a public test operator identity and authorize one finalized script-path input without learning maturity semantics?

The interface must distinguish:

```text
target-generic test signing capability
from
protocol operator role
```

The vectors package owns the operator meaning.

The executor may own:

- a public disposable test scalar;
- the corresponding public key;
- target-specific sighash construction;
- signature production over exact finalized bytes.

No production key enters.

## 3.8 Root-history evidence · `q:guide14:root-history`

Can the existing model and vector projections prove:

- exactly one predecessor STATE edge;
- exactly one successor STATE edge;
- no intermediate invalid STATE transition;
- no stale predecessor;
- no root cursor restoration after an invalid edge;
- no STATE termination;
- no RESV edge?

If the final cursor alone is insufficient, introduce a typed edge sequence rather than another cursor check.

## 3.9 Synthetic STATE origin · `q:guide14:synthetic-state`

Because target genesis and earlier STATE-producing operations are not implemented, how is the initial target STATE funded?

The synthetic ceremony must state:

```text
synthetic target STATE fixture
not protocol genesis
not evidence of trusted setup
not evidence of prior root history
not production authority
authorizes nothing of value
```

The ceremony must not silently become a genesis implementation.

## 3.10 Evidence-role separation · `q:guide14:evidence-roles`

Can the evidence architecture keep separate:

- semantic maturity-window safety;
- operator authorization;
- metadata copy-through;
- predecessor constructor authentication;
- successor constructor reconstruction;
- root-history continuity;
- target execution;
- public metadata recoverability;
- resource evidence;
- lifecycle incompleteness?

If not, use separate report types rather than one report whose optional fields change its meaning.

## 3.11 Preflight gate · `gate:guide14:preflight`

Implementation begins only when:

- every question above has an explicit disposition;
- complete STATE metadata ownership is identified;
- prototype promotion is itemized;
- every constructor cycle is classified;
- metadata recovery is public and deterministic;
- branch-order and tweak-totality policies are explicit;
- selected operator sighash is exact;
- test signer material is explicitly public and disposable;
- root-history evidence is edge-based rather than cursor-only;
- safety, continuity, history, and resource reports remain separate;
- no production secret interface is introduced;
- focused tests and full repository checks pass;
- the tree is clean.

---

# 4. Required authority · `sec:guide14:authority`

Guide 14 is governed by:

```text
Attestation
Realization
typed architecture
implemented ADRs
accepted decisions
accepted STATE-constructor research
accepted Phase-4 and Phase-5 pipeline boundaries
package contracts
Phase-6 card
this concept's eventual execution guide
```

Relevant decisions include:

- D001 typed Rust source;
- D002 target-independent realization;
- D003 tapscript first;
- D004 translation validation;
- D005 value-parametric representation and explicit closed assets;
- D006 canonical transaction and witness ABI;
- D007 Petgraph substrate;
- D008 exact semantic mathematics.

Relevant realization obligations include:

- STATE is canonical and singleton;
- maturity begins unannounced;
- announcement changes only maturity;
- announced cycle lies inside the exact lead window;
- operator authorization is required;
- no RESV edge occurs;
- unaffected state fields are preserved exactly;
- optional sponsor value remains opaque;
- transition-certificate root edges are derived rather than authored by branch code.

No package parses these documents as semantic input.

---

# 5. Semantic maturity-announcement contract · `sec:guide14:semantics`

## 5.1 Predecessor · `def:guide14:predecessor`

The operation consumes exactly one canonical STATE predecessor.

It carries:

```text
asset:
    exact architecture-owned STATE singleton asset

amount:
    exact singleton amount

metadata:
    complete canonical STATE

maturity:
    Unannounced

root role:
    current canonical STATE cursor
```

It consumes no other protocol root.

## 5.2 Request · `def:guide14:request`

The semantic request selects one value:

```text
announced maturity cycle a
```

It may additionally select optional sponsor capability under the inherited sponsor policy.

It does not select:

- predecessor STATE;
- successor STATE program;
- successor constructor bytes;
- unaffected STATE fields;
- operator public identity;
- target input or output position;
- root cursor;
- control block;
- transaction version;
- fee role.

Those derive from the current world, deployment policy, linked bundle, and ABI.

## 5.3 Announcement window · `rule:guide14:window`

Let \(c\) be predecessor STATE’s current cycle.

The request is valid exactly when:

\[c+L_{\min}\le a\le c+L_{\max}\]

and all arithmetic is representable in the typed cycle domain.

Required boundary cases include:

```text
a = c + L_min
a = c + L_max
a = c + L_min - 1
a = c + L_max + 1
```

The two outside cases reject.

## 5.4 Successor · `def:guide14:successor`

The operation creates exactly one canonical STATE successor.

Its maturity is:

```text
Announced { cycle = a }
```

Every other semantic STATE field equals the predecessor.

Its root role is:

```text
new canonical STATE cursor
```

No STATE termination is permitted.

## 5.5 Authorization · `rule:guide14:authorization`

The architecture-authorized operator approves the finalized transaction.

A missing, malformed, unrelated, stale, or differently bound signature rejects.

No receipt owner, sponsor owner, refund key, or cadence-delayed permissionless branch can substitute for operator authorization.

Sponsor signatures protect sponsor inputs only.

## 5.6 Root effects · `rule:guide14:root-effects`

The exact root projection is:

```text
STATE:
    Succ { predecessor, successor }

RESV:
    absent

PACE:
    absent

ENT_AUTH:
    absent

DIST_AUTH:
    absent
```

No transaction may consume or create an inactive decoy root-shaped object and still satisfy the relation.

## 5.7 Canonical flow and issuance · `rule:guide14:no-economic-flow`

Maturity announcement declares no canonical value flow and no issuance.

The singleton STATE root asset is consumed and recreated through root succession.

There is:

```text
no U flow
no ENT flow
no DIST_CTL flow
no destruction
no issuance
no receipt output
no ASH output
no entitlement output
no vault or control output
```

## 5.8 Projection policy · `rule:guide14:projection`

Required:

```text
transition certificate
```

Forbidden specialized projections:

```text
burn
clear
distribution residue
```

The certificate derives the STATE root edge from actual consumed and created objects. The request does not author it.

## 5.9 Sponsor flow · `rule:guide14:sponsor-flow`

The only open flow is the optional fee-sponsor region.

It may contain:

- ordinary L-BTC sponsor inputs;
- optional sponsor change;
- target fee role.

It must remain disjoint from STATE and every protocol object.

---

# 6. Typed STATE metadata · `sec:guide14:metadata`

## 6.1 One typed source · `rule:guide14:metadata-source`

The complete STATE metadata schema derives from one typed first-party source.

It is not reconstructed from:

- model source text;
- realization Markdown;
- Attestation LaTeX;
- planning tables;
- target scripts;
- generated JSON;
- a report from an earlier run.

Every backend field maps to one typed semantic field or one explicitly representation-only field.

## 6.2 Canonical encoding · `rule:guide14:metadata-encoding`

The encoding states:

- domain separator;
- schema version;
- complete field order;
- exact widths;
- byte order per field;
- canonical enum discriminants;
- reserved-field rule;
- representation nonce;
- unknown-field rejection.

A decoder accepts exactly one byte string for one metadata value.

Reserved fields are zero and remain zero across announcement.

A tolerant decoder is not admitted for constructor identity.

## 6.3 Semantic projection · `rule:guide14:metadata-projection`

The semantic projection excludes:

- representation nonce;
- tree position;
- control-path parity;
- local graph handles;
- target byte offsets;
- linker indices;
- search-attempt count.

It retains every protocol field.

Two encodings differing only in an admitted representation nonce project to the same semantic STATE.

## 6.4 Exact copy-through · `rule:guide14:metadata-copy-through`

The successor metadata is derived by a total typed function:

```rust
fn announce_maturity(
    predecessor: &StateMetadata,
    announced_cycle: Cycle,
) -> Result<StateMetadata, StateTransitionRefusal>;
```

The function:

- requires `Unannounced`;
- validates the lead window;
- sets `Announced { cycle }`;
- copies every other semantic field;
- resets the representation nonce before canonical grinding;
- rejects overflow;
- returns no partial metadata.

No field is copied by byte offset without typed decoding.

## 6.5 Public recoverability · `rule:guide14:metadata-recoverability`

The exact successor metadata bytes required for future constructor use are recoverable from canonical public chain data.

An unrelated process must be able to obtain:

- successor outpoint;
- successor metadata;
- constructor schema;
- static code identity or recipe;
- representation nonce;
- leaf version;
- internal-key policy;
- required control recipe.

It must not require:

- operator private state;
- creator process memory;
- temporary files;
- a private database;
- a hidden wallet descriptor.

## 6.6 Synthetic fields are forbidden by default · `rule:guide14:no-synthetic-state`

The Guide-10 prototype’s counter, object kind, flags, and nonce do not enter STATE merely because the prototype had them.

A prototype field is promoted only if:

- the typed STATE schema has a semantic owner for it; or
- it is an explicitly representation-only constructor field.

Every promoted field is named in the preflight record.

---

# 7. Candidate STATE constructor · `sec:guide14:constructor`

## 7.1 Tree shape · `candidate:guide14:state-constructor`

The initial candidate follows the accepted constructor shape:

```text
TapBranch
├── metadata commitment leaf
└── static operation-program subtree
```

The metadata leaf commits the complete STATE metadata encoding.

The static subtree commits the complete candidate operation leaf set.

The branch hash follows the target’s canonical child-order rule.

## 7.2 Metadata leaf is unspendable · `rule:guide14:metadata-leaf`

The metadata leaf exists to commit data, not to authorize a spend.

Its program aborts for every witness and every initial stack.

An empty script is prohibited because an initial true item could satisfy it.

The abstract validator and target-native vectors prove:

```text
success set:
    empty

non-aborting successful path:
    absent

abort:
    guaranteed
```

## 7.3 Static code identity · `rule:guide14:static-code`

Predecessor and successor use one exact linked static subtree.

The successor must not:

- add a leaf;
- remove a leaf;
- reorder a leaf under a source-order-dependent policy;
- substitute another operation program;
- change leaf version;
- change internal-key policy;
- retain a stale subtree from another candidate.

Static code continuity is checked independently of metadata continuity.

## 7.4 Predecessor authentication · `rule:guide14:predecessor-authentication`

The announcement leaf authenticates the predecessor constructor from:

- exact predecessor metadata;
- exact static subtree;
- exact leaf version;
- exact internal key;
- exact target tweak relation;
- actual consumed input program.

A caller-supplied metadata object that does not reconstruct the consumed input’s program rejects.

## 7.5 Successor reconstruction · `rule:guide14:successor-reconstruction`

The announcement leaf reconstructs or verifies the successor constructor from:

- canonical successor metadata;
- unchanged static subtree;
- exact leaf version;
- exact internal key;
- exact target tweak relation;
- actual successor output program.

The output program is not caller authority.

A valid target program derived from another metadata value rejects.

## 7.6 Canonical branch side · `rule:guide14:canonical-side`

Where the target language cannot order branch children, the candidate fixes:

```text
metadata leaf first
static subtree second
```

and constructs only metadata encodings satisfying:

\[h_{\mathrm{meta}}\le h_{\mathrm{static}}\]

The target’s canonical branch hash and the program’s fixed-order hash are then equal.

The representation nonce is incremented deterministically from zero until:

- branch-side ordering holds; and
- the tweak-totality condition holds.

The search has an explicit maximum-attempt count.

Failure returns no constructor.

## 7.7 Tweak totality · `rule:guide14:tweak-totality`

Let \(P\) be the internal point, \(r\) the merkle root, and:

\[t=H_{\mathrm{TapTweak/elements}}(P_x\parallel r)\]

The output point is:

\[Q=P+tG\]

The target admits \(t=0\).

It rejects:

- \(t\ge n\), where \(n\) is the group order;
- \(Q\) equal to the identity.

A retry policy changes only the representation nonce, which changes metadata leaf, root, and tweak.

It cannot repair:

- invalid internal key;
- missing executing leaf;
- repeated executing leaf;
- excessive control depth;
- malformed metadata;
- wrong static subtree.

Retryability is defined once and shared by every constructor search.

## 7.8 Internal key · `rule:guide14:internal-key`

Use the inherited published unspendable internal-key policy only if its assumptions remain valid for this constructor.

No:

- operator key;
- owner key;
- release key;
- wallet key;
- generated-and-discarded key;
- caller-selected internal key.

The residual discrete-log assumption remains explicit.

## 7.9 No key-path escape · `rule:guide14:no-keypath`

The candidate STATE constructor provides no accepted key-path spend.

Vectors cover:

- attempted key-path spend;
- wrong internal key;
- operator key substituted as internal key;
- valid script path under wrong parity;
- valid leaf under stale control block.

## 7.10 Candidate leaf set · `rule:guide14:leaf-set`

The Phase-6 static subtree contains only implemented maturity-announcement programs and support leaves required by the accepted constructor.

It contains no spendable placeholder for:

- admission;
- clear;
- redemption;
- cycle;
- settlement;
- another future STATE operation.

Missing future operations remain typed lifecycle obligations.

## 7.11 Constructor reference graph · `rule:guide14:reference-graph`

Every constructor reference is typed and enters the linker graph.

A cycle is accepted only under an explicit authenticated strategy.

An SCC is not itself a strategy.

Repeated hashing until bytes stabilize is prohibited.

External authentication that cuts a cycle leaves an explicit outstanding equality obligation unless the equality is independently verified.

---

# 8. Compiler target-operation plan · `sec:guide14:compiler`

## 8.1 Plan boundary · `rule:guide14:operation-plan`

Introduce or derive a validated maturity-announcement operation plan from the complete analyzed program.

It retains:

- exact architecture and realization binding;
- exact `announce-maturity` scope;
- relation census;
- predecessor and successor source requirements;
- operator authorization;
- root succession requirements;
- metadata-preservation requirements;
- constructibility requirements;
- lifecycle obligations;
- abstract carriers;
- layout requirements;
- coverage requirements;
- target capabilities;
- external evidence.

It contains no target opcode, tree node, byte offset, transaction position, or digest.

## 8.2 Required relation families · `tab:guide14:relations`

The exact census is derived from realization. The review checklist includes:

| Family | Requirement |
|---|---|
| STATE input cardinality | exactly one |
| STATE output cardinality | exactly one |
| STATE recognition | exact root asset, singleton amount, constructor |
| maturity predecessor | unannounced |
| maturity successor | announced at requested cycle |
| announcement lead | inside exact architecture window |
| metadata preservation | every unaffected field equal |
| operator authorization | selected authorized operator |
| input closure | STATE plus optional sponsor only |
| output closure | STATE successor plus optional sponsor change and fee only |
| sponsor isolation | exact, disjoint, amount-opaque |
| root policy | STATE succession; every other root forbidden |
| issuance policy | none |
| destruction policy | none |
| projection policy | transition certificate required; specialized events forbidden |
| constructibility | public construction plus operator signing capability |
| lifecycle | future STATE operations explicitly outstanding |

## 8.3 Carrier placement · `rule:guide14:carriers`

Expected carriers include:

```text
announcement leaf:
    predecessor metadata
    successor metadata
    operator authorization
    announcement window
    constructor continuity
    STATE succession

bundle and ABI structure:
    leaf-set closure
    no key-path escape
    root and event absence
    witness roles

external target:
    exact transaction conservation
    selected sighash semantics
```

Every active relation has a reachable carrier.

A structural relation is not marked target-executed merely because the complete transaction later accepts.

## 8.4 Compiler validation · `rule:guide14:compiler-validation`

The operation plan is re-derived and checked in both directions.

Required corruption tests include:

- removed STATE relation;
- extra relation;
- wrong root subject;
- RESV relation inserted;
- lead bound changed;
- operator requirement removed;
- unaffected-field requirement removed;
- carrier moved to an inactive case;
- lifecycle obligation removed;
- sponsor amount introduced;
- target type inserted;
- coverage row changed;
- graph handle inserted into a public projection.

## 8.5 Public API discipline · `gate:guide14:compiler-api`

External callers cannot:

- construct a validated operation plan;
- mutate its relation census;
- claim evidence completion;
- choose graph handles;
- supply target positions;
- supply constructor bytes;
- mint a plan digest.

Equal typed inputs produce equal projections.

Declaration permutations produce equal projections.

---

# 9. Target assessment and proof patterns · `sec:guide14:target-patterns`

## 9.1 Static target assessment · `rule:guide14:target-assessment`

Assess every required compiler capability against the exact reviewed target.

Expected capability families include:

```text
authenticated STATE recognition
authenticated singleton cardinality
authenticated current-root use
authenticated successor-root effect
canonical metadata encoding
streaming hashing
byte-string composition
taproot tweak verification
operator authorization
output-committing sighash
public constructor data
sponsor isolation
whole-transaction conservation
```

Every result remains multi-state:

```text
Unsupported

MissingTargetPrimitives

BackendPatternRequired

BackendStructural

ExternalEvidenceRequired

CompleteBackendPattern
```

## 9.2 No capability inflation · `rule:guide14:no-capability-inflation`

Low-level primitives do not imply a complete constructor proof.

In particular:

```text
streaming hash exists
    ⇏ predecessor constructor authenticated

tweak verification exists
    ⇏ successor continuity proved

signature primitive exists
    ⇏ operator key encoding safe

target conservation exists
    ⇏ STATE metadata preserved

node accepted
    ⇏ current root consumed
```

## 9.3 STATE recognition pattern · `rule:guide14:state-recognition`

The predecessor pattern authenticates:

```text
exact STATE root asset
exact singleton amount
actual consumed program
canonical predecessor metadata
selected static subtree
current-root role
```

A copied constructor under another asset rejects.

A stale STATE root rejects at the root-history or target input boundary.

## 9.4 Metadata transition pattern · `rule:guide14:metadata-transition`

The announcement pattern:

1. parses canonical predecessor metadata;
2. requires `Unannounced`;
3. reads current cycle;
4. validates the requested announced cycle;
5. derives successor metadata;
6. proves all unaffected fields equal by construction or exact comparison;
7. validates canonical successor representation;
8. reconstructs the successor output program.

The request does not supply an arbitrary successor metadata blob.

## 9.5 Operator pattern · `rule:guide14:operator-pattern`

The operator pattern:

- authenticates recognized key encoding;
- obtains the expected public operator identity from typed deployment or STATE policy;
- verifies a nonempty signature;
- rejects invalid signatures;
- rejects unknown nonempty key types that the primitive would otherwise accept without verification;
- binds the finalized transaction under the selected sighash profile.

The target primitive’s success result alone is insufficient.

## 9.6 Root succession pattern · `rule:guide14:root-pattern`

The coordinator authenticates:

- one STATE input;
- one STATE output;
- predecessor is current root;
- successor program matches derived constructor;
- no second STATE object;
- no other root;
- exact role order.

The root edge is independently projected from accepted bytes.

## 9.7 Sponsor isolation pattern · `rule:guide14:sponsor-pattern`

Where sponsorship is active, authenticate:

- exact suffix start and length;
- reserve asset;
- sponsor program class;
- optional sponsor-change role;
- target fee role;
- no STATE or closed protocol asset in sponsor positions;
- no sponsor amount among protocol operands.

## 9.8 Final stack · `rule:guide14:final-stack`

Every emitted operation program:

- reaches one canonical true item on success;
- checks every comparison and arithmetic result;
- leaves no alternate-stack residue;
- has no non-aborting failure state satisfying final truth;
- retains exact computed truth where the abstract state can settle it;
- remains within target stack and element limits.

---

# 10. Candidate transaction and witness ABI · `sec:guide14:abi`

## 10.1 Input layout · `rule:guide14:input-layout`

The candidate layout is:

```text
input 0:
    canonical STATE predecessor and coordinator

inputs 1..s:
    optional sponsor suffix
```

No caller chooses another coordinator.

Sponsor inputs are ordered canonically within the suffix.

Duplicates and overlap reject before sorting.

## 10.2 Output layout · `rule:guide14:output-layout`

The candidate output layout is:

```text
output 0:
    canonical STATE successor

next optional role:
    sponsor change

final target role where required:
    fee output
```

No other protocol output is admitted.

The fee role and sponsor change cannot satisfy STATE.

STATE cannot satisfy either open-asset role.

## 10.3 Typed request · `rule:guide14:typed-request`

A maturity-announcement request may select:

- announced cycle;
- optional sponsor capability;
- optional sponsor-change destination under sponsor policy;
- explicit public test-only construction parameters where the constructor policy requires them.

It may not select:

- predecessor STATE outpoint independently of the current root view;
- predecessor metadata;
- successor metadata except through the announced cycle;
- successor program;
- static subtree;
- internal key;
- operator identity;
- root cursor;
- transaction positions;
- target fee role;
- representation nonce result;
- control block.

## 10.4 Public construction view · `rule:guide14:public-view`

The transaction constructor receives public typed facts:

- current STATE outpoint;
- exact STATE asset and amount;
- canonical predecessor metadata;
- current root binding;
- current cycle;
- linked static subtree;
- target constructor contract;
- operator public identity;
- candidate ABI;
- sponsor-local capability where selected.

Duplicate or contradictory views of one outpoint reject.

A later entry must not overwrite an earlier contradictory view.

## 10.5 Constructor search · `rule:guide14:constructor-search`

The builder derives:

- predecessor constructor recipe;
- successor semantic metadata;
- canonical successor metadata encoding;
- representation nonce;
- metadata leaf;
- successor tree root;
- successor output program;
- control data.

The search order is deterministic.

Search diagnostics may report attempt count.

Attempt count does not enter semantic projection or identity.

## 10.6 Witness roles · `rule:guide14:witness-roles`

The STATE input witness may carry typed roles such as:

```text
operator signature
predecessor metadata
successor metadata
announcement leaf
control block
constructor-specific public witnesses
```

The exact order is ABI-owned and canonical.

No witness item is inferred by parsing arbitrary script bytes.

The metadata items are public.

The operator signature is test-only evidence in Phase 6 and production-secret handling is not claimed.

## 10.7 Output finalization · `rule:guide14:output-finalization`

Before the operator signing request exists:

- announced cycle is final;
- successor metadata is final;
- representation nonce is final;
- successor output program is final;
- sponsor inputs are final where committed;
- sponsor change is final;
- fee role is final;
- transaction version is final;
- locktime is final;
- protected output witnesses are final where the selected sighash commits them.

A post-signing metadata or program edit invalidates the signature.

## 10.8 Sponsored-form consistency · `rule:guide14:sponsor-consistency`

Require exact equivalence:

```text
request says sponsored
⇔ sponsor capability supplied
⇔ sponsor offer contains at least one input
⇔ selected shape is sponsored
⇔ fee role is present
```

An empty sponsor offer does not silently select the sponsorless form.

Duplicate sponsor outpoints reject.

## 10.9 Canonical transaction decoding · `rule:guide14:canonical-transaction`

External target bytes enter through a strict parser.

A successfully decoded transaction satisfies:

```text
decode(bytes).encode() = bytes
```

The parser rejects:

- nonminimal compact sizes;
- unknown field prefixes;
- unsupported issuance or peg-in fields;
- trailing bytes;
- superfluous all-empty witness sections;
- inconsistent input/witness census;
- unsupported proof fields.

## 10.10 Candidate status · `rule:guide14:abi-status`

The result is:

```text
CandidateMaturityAnnouncementAbi
```

It is not final and carries no ABI digest by default.

---

# 11. Operator signing boundary · `sec:guide14:signing`

## 11.1 No production key custody · `rule:guide14:no-production-key`

No first-party Phase-6 library or command accepts a production operator private key.

The target executor may use one published disposable test scalar tied only to a disposable development chain.

That material:

- authorizes nothing of value;
- is committed source or deterministic test state;
- is labeled test-only;
- is never derived from production material;
- is destroyed or discardable with the test environment.

## 11.2 Target-generic signing capability · `rule:guide14:target-signing`

The executor interface exposes target-generic work, not maturity semantics.

Illustratively:

```text
declare public test signer
fund object at caller-supplied constructor
authorize one finalized script-path input
submit complete transaction
```

The vectors package owns the interpretation:

```text
this signer is the maturity operator
```

The executor owns only target mechanics.

## 11.3 Signing request · `def:guide14:signing-request`

The signing request binds:

- exact finalized transaction bytes;
- input index;
- predecessor outpoint;
- operator public identity;
- selected target sighash profile;
- executing tapleaf where applicable;
- expected witness role.

It carries no expected target verdict.

## 11.4 Signing response · `def:guide14:signing-response`

A successful response carries:

- exact bytes signed;
- signature witness;
- public signer identity or exact binding needed to compare it;
- observed sighash profile where available.

It carries no private scalar.

The builder compares the returned binding against the exact request bytes.

A mismatch rejects before submission.

## 11.5 Signer faults · `rule:guide14:signer-faults`

Required faults include:

- no signature;
- wrong operator;
- empty signature;
- malformed signature;
- signature under another profile;
- signature over another transaction;
- signature for another input;
- duplicate response;
- unexpected signer;
- output changed after signing;
- metadata changed after signing;
- sponsor input added after signing;
- fee role changed after signing.

---

# 12. Linking · `sec:guide14:linking`

## 12.1 Linker reuse · `rule:guide14:linker-reuse`

Guide 14 extends the inherited linker only where mutable STATE requires:

- dynamic metadata-leaf recipe;
- static-subtree commitment;
- predecessor/successor constructor references;
- metadata schema symbol;
- representation-nonce policy;
- operator-key or signer-profile symbol;
- root-carrier closure;
- constructor resource formulas.

No general dynamic linker is introduced beyond the demonstrated need.

## 12.2 Typed symbols · `rule:guide14:symbols`

Expected symbols include:

```text
STATE singleton asset
STATE singleton amount
STATE metadata schema
STATE metadata domain
maturity discriminants
minimum announcement lead
maximum announcement lead
static STATE subtree
maturity-announcement program
operator public identity
selected sighash profile
target leaf version
unspendable internal key
candidate sponsor bound
target fee role
```

A symbol is a typed role, not a display string.

## 12.3 Two-pass resolution · `rule:guide14:two-pass`

Pass one:

```text
collect every typed definition
validate uniqueness
validate type
normalize by stable key
```

Pass two:

```text
resolve every reference
validate expected type
build frozen reference graph
compute SCCs
apply cycle policy
```

Every mandatory reference resolves exactly once.

## 12.4 Duplicate tree leaves reject · `rule:guide14:tree-census`

The taptree input rejects duplicate leaf identities.

It does not use “last declaration wins.”

A leaf’s program role derives from or is validated against its leaf identity.

Conflicting weights, roles, or programs under one leaf key are typed failures.

## 12.5 Deterministic taptree · `rule:guide14:taptree`

The candidate tree input states:

- exact leaf set;
- exact leaf version;
- exact positive weights;
- stable tie-break;
- maximum depth;
- static/dynamic constructor composition.

Tree construction is independent of declaration order.

For small instances, compare against an exact bounded-depth oracle where a depth bound applies.

If the implementation computes an unrestricted optimum and then rejects excess depth, it says so and does not claim bounded-depth optimality.

## 12.6 Exact tree cost · `rule:guide14:tree-cost`

Tree objective arithmetic is exact or checked.

Saturating arithmetic must not participate in an “exact optimum” claim.

Overflow returns a typed failure or uses an exact wider domain.

## 12.7 Carrier closure · `rule:guide14:carrier-closure`

Compare:

```text
compiler-required carriers
backend-emitted carriers
linked reachable carriers
ABI-carried roles
```

Every STATE relation remains carried through the announcement leaf or exact structural evidence.

A constructor-continuity relation cannot disappear because the target program bytes remain reachable.

## 12.8 Candidate linked bundle · `rule:guide14:linked-bundle`

The linked bundle retains:

- exact maturity-announcement scope;
- exact target projection;
- candidate STATE constructor;
- linked announcement program;
- metadata schema and constructor recipe;
- operator signing profile;
- taptree and control recipes;
- relation placements;
- root-carrier census;
- ABI handoff;
- resource formulas;
- unresolved target evidence;
- outstanding future STATE lifecycle;
- explicit non-production status.

No bundle digest is minted by default.

---

# 13. Evidence architecture · `sec:guide14:evidence`

## 13.1 Canonical evidence plan · `rule:guide14:evidence-plan`

Introduce a canonical validated maturity-announcement evidence plan with private fields and no unchecked constructor.

It derives from:

- compiler coverage;
- linked constructor;
- candidate ABI;
- canonical semantic fixtures;
- canonical constructor mutations;
- canonical root-history mutations;
- exact target and deployment binding.

Ad hoc STATE vectors produce experimental reports only.

## 13.2 Transcript-derived state changes · `rule:guide14:transcript-evidence`

Canonical coverage changes only through validated wrappers built from the exact generic execution transcript.

A public API must not accept caller-authored tuples such as:

```text
vector identity
+
Accepted
+
ProjectionMatched
```

as evidence.

The wrapper binds:

- exact target;
- exact deployment;
- exact executor provenance;
- exact sent requests;
- exact responses;
- exact transaction bytes;
- exact target observations;
- independently recomputed STATE projection;
- independently recomputed root-history projection;
- independently recomputed constructor comparison.

## 13.3 Maturity safety report · `def:guide14:safety-report`

The safety report answers:

```text
Did every valid announcement preserve the exact semantic STATE
transition, and did every invalid maturity, operator, root, metadata,
constructor, sponsor, and ABI mutation fail at its owning boundary?
```

It carries:

- complete case census;
- relation census;
- mutation census;
- target observations;
- semantic projections;
- exact failure layers;
- executor provenance;
- lifecycle status.

## 13.4 Constructor continuity report · `def:guide14:constructor-report`

The continuity report answers:

```text
Did predecessor and successor bind the exact expected metadata,
static subtree, internal-key policy, branch order, and target tweak
relation?
```

It carries:

- predecessor metadata and constructor recipe;
- successor metadata and constructor recipe;
- metadata leaf hashes;
- static subtree root;
- branch root;
- tweak inputs;
- pinned or observed output programs;
- control paths;
- comparison results;
- explicit outstanding assumptions.

It does not become the semantic safety report.

## 13.5 Root-history report · `def:guide14:history-report`

The root-history report answers:

```text
Did one canonical STATE predecessor advance to one canonical STATE
successor with no invalid intermediate edge and no unrelated root
participation?
```

It carries an ordered typed edge sequence, not only final cursors.

It rejects:

- missing edge;
- duplicate edge;
- stale predecessor;
- two successors;
- invalid intermediate edge hidden by final cursor restoration;
- unexpected termination;
- RESV or another root edge.

## 13.6 Public recoverability report · `def:guide14:recoverability-report`

Where successor metadata is published through chain witness or another public role, an unrelated process:

1. locates the accepted transaction;
2. verifies exact transaction bytes;
3. recovers successor metadata;
4. decodes it canonically;
5. reconstructs the successor constructor;
6. compares the reconstruction with the accepted output program.

The process receives no creator memory, temporary path, operator private key, or private database.

## 13.7 Resource report · `def:guide14:resource-report`

The resource report remains separate and carries:

- metadata size;
- announcement program bytes;
- constructor leaf bytes;
- static subtree size;
- control depth and bytes;
- nonce-search attempts as diagnostic data;
- operator signature bytes;
- sponsor witness bytes;
- transaction weight;
- target observations;
- policy verdict.

A resource pass does not satisfy safety, continuity, or root history.

## 13.8 Validated operation report · `rule:guide14:validated-report`

The operation report retains the exact generic execution transcript.

It recomputes:

- target and deployment binding;
- request/response census;
- funding and signing steps;
- submitted bytes;
- target verdicts;
- accepted STATE projections;
- root-history edges;
- constructor comparisons;
- resource comparisons;
- summary counts.

Canonical bytes exclude wall time, host identity, process ID, temporary paths, and ambient environment values.

Timing is emitted separately as diagnostics.

## 13.9 Interchange boundary · `rule:guide14:interchange`

If a Guide-14 report becomes an externally consumed document, ADR-022 activates for that document.

Until a real external consumer exists:

- no interchange namespace is allocated;
- no theory coordinate is assigned;
- no CBOR/CDDL encoder or decoder is built merely for anticipation;
- internal command streams remain under ADR-010.

---

# 14. Semantic fixtures and accepted projections · `sec:guide14:fixtures`

## 14.1 Model-valid fixture · `rule:guide14:model-fixture`

A positive fixture starts from a model-valid world with:

- operational STATE;
- maturity unannounced;
- exact current cycle;
- authorized operator;
- canonical root history;
- no conflicting root mutation;
- optional sponsor inputs under the public-data test policy.

The transition runs through the invariant-wrapped model path and retains:

- predecessor world;
- exact request;
- canonical order;
- successor world;
- appended transition certificate.

## 14.2 Synthetic target STATE · `rule:guide14:synthetic-target-state`

Because target genesis and prior STATE-producing operations are not implemented, target vectors may fund a synthetic STATE output.

Every report carries all applicable disclaimers:

```text
synthetic target STATE fixture
not protocol genesis
not evidence of trusted setup
not evidence of earlier root history
not production STATE
authorizes nothing of value
```

Synthetic funding is not root-history evidence before the tested edge.

## 14.3 Expected semantic result · `rule:guide14:expected-result`

Expected semantics derive from:

- realization relations;
- executable model;
- typed projection adapters.

The constructor oracle does not generate the expected semantic STATE.

The target backend does not decide which fields are permitted to change.

## 14.4 Target materialization · `rule:guide14:materialization`

Target materialization derives from:

```text
candidate linked bundle
+
candidate maturity ABI
+
typed announcement request
+
public current-STATE view
+
test-only operator signing capability
+
optional sponsor capability
```

The semantic fixture contains no target position, program, control block, transaction byte, signature, or constructor nonce.

## 14.5 Accepted STATE projection · `rule:guide14:accepted-state-projection`

For every accepted transaction compare:

- exact predecessor STATE outpoint;
- exact predecessor semantic metadata;
- predecessor maturity unannounced;
- exact requested announced cycle;
- exact successor semantic metadata;
- every unaffected field equal;
- exact singleton STATE asset and amount;
- one STATE successor;
- operator authorization;
- STATE succession edge;
- no STATE termination;
- no RESV, PACE, entitlement-authority, or distribution-authority edge;
- no issuance;
- no destruction;
- no canonical `U`, `ENT`, or `DIST_CTL` flow;
- sponsor-region membership and change presence;
- no specialized event;
- required transition certificate;
- exact operation identity.

## 14.6 Accepted constructor projection · `rule:guide14:accepted-constructor-projection`

For every accepted successor compare:

- metadata schema;
- semantic metadata;
- representation nonce;
- metadata leaf script;
- metadata leaf hash;
- static subtree root;
- branch root;
- leaf version;
- internal key;
- output-key parity;
- output program;
- control recipe.

The semantic projection may erase the nonce; constructor evidence may not.

## 14.7 Positive class witnesses · `rule:guide14:positive-class-witnesses`

A positive class name does not prove its own predicate.

Before canonical admission, each class evaluates a typed witness.

Examples:

```text
minimum lead:
    announced cycle = current + L_min

maximum lead:
    announced cycle = current + L_max

all unaffected fields preserved:
    complete typed field equality except maturity

public recoverability:
    unrelated process reconstructs successor program

canonical nonce retry:
    first admitted nonce is selected by deterministic search

sponsored:
    sponsor region present and exact

sponsor-change present:
    exactly one sponsor-change role exists
```

---

# 15. Required vector matrix · `sec:guide14:vectors`

## 15.1 Positive semantic cases · `tab:guide14:positive`

Required:

- announcement at minimum lead;
- announcement at maximum lead;
- representative interior lead;
- smallest current cycle;
- current cycle near its checked upper domain;
- all unaffected economic fields nontrivial;
- sponsorless transaction;
- sponsored transaction;
- sponsor change present;
- sponsor change absent;
- canonical representation nonce zero where admitted;
- canonical representation nonce nonzero where required;
- public successor metadata recovery;
- repeated execution producing byte-identical canonical report bytes.

## 15.2 Announcement-window faults · `tab:guide14:window-faults`

Required:

- predecessor already announced;
- predecessor maturity complete;
- announced cycle below minimum lead;
- announced cycle above maximum lead;
- announced cycle equal to current cycle;
- checked addition overflow;
- malformed cycle encoding;
- request cycle differs from successor metadata;
- successor maturity remains unannounced;
- successor maturity marked complete.

## 15.3 Metadata preservation faults · `tab:guide14:metadata-faults`

For every STATE semantic field other than maturity:

- mutate that field alone;
- preserve maturity change correctly;
- preserve constructor shape;
- require rejection or report-layer semantic mismatch at the owning boundary.

Additionally:

- omit one field;
- duplicate one field;
- reorder fields;
- set reserved field;
- use unknown schema;
- use noncanonical enum discriminant;
- change only representation nonce and require semantic equality;
- change semantic field while compensating another and require rejection;
- copy metadata from another STATE instance.

## 15.4 Operator faults · `tab:guide14:operator-faults`

Required:

- missing signature;
- empty signature;
- malformed signature;
- unrelated signer;
- stale operator identity;
- unknown key encoding;
- valid signature over another transaction;
- valid signature under another sighash profile;
- signature over predecessor but not successor outputs;
- successor metadata changed after signing;
- successor program changed after signing;
- sponsor input added after signing;
- fee output changed after signing;
- duplicate signing response;
- unexpected signer response.

## 15.5 Predecessor-constructor faults · `tab:guide14:predecessor-faults`

Required:

- wrong STATE asset;
- wrong singleton amount;
- wrong predecessor program;
- predecessor metadata does not reconstruct consumed program;
- wrong static subtree;
- wrong leaf version;
- wrong internal key;
- wrong control path;
- stale constructor from another bundle;
- repeated metadata leaf;
- metadata leaf absent;
- executing metadata leaf instead of announcement leaf;
- key-path spend attempt.

## 15.6 Successor-constructor faults · `tab:guide14:successor-faults`

Required:

- successor program derived from wrong metadata;
- successor program derived from predecessor metadata;
- successor under another static subtree;
- successor under another internal key;
- successor with wrong parity;
- successor with wrong leaf version;
- successor control recipe from another tree;
- successor metadata leaf spendable;
- caller-supplied arbitrary output program;
- two STATE successors;
- no STATE successor;
- extra STATE-like output;
- output constructor nonce not canonical;
- output constructor search result not the first admissible nonce.

## 15.7 Branch-order and totality faults · `tab:guide14:constructor-totality-faults`

Required:

- metadata hash on wrong side;
- children hashed in caller order;
- source-order-dependent tree;
- nonce search starts at one instead of zero;
- nonce search skips an admissible nonce;
- search budget exhausted;
- invalid internal key retried instead of refused;
- missing executing leaf retried instead of refused;
- zero tweak accepted where the sum remains a valid point;
- tweak at or above group order rejected;
- identity result rejected;
- target and oracle parity disagree;
- repeated hashing used as a fixed-point search.

## 15.8 Root-history faults · `tab:guide14:history-faults`

Required:

- stale STATE predecessor;
- two predecessors;
- two successors;
- successor cursor restored after an invalid intermediate edge;
- missing STATE edge;
- STATE termination;
- old-bundle predecessor to new-bundle successor without migration;
- unrelated STATE-shaped input;
- STATE output not recorded as root;
- root cursor moved to sponsor output;
- RESV edge added;
- PACE edge added;
- authority edge added;
- certificate names another predecessor or successor.

## 15.9 Root and absence faults · `tab:guide14:absence-faults`

Required:

- add RESV input;
- add RESV output;
- add PACE input;
- add PACE output;
- add entitlement-authority input;
- add distribution-authority input;
- add receipt input or output;
- add ASH input or output;
- add entitlement;
- add distribution control or vault;
- add issuance;
- add destruction;
- emit burn record;
- emit clear event;
- emit residue event.

## 15.10 Sponsor faults · `tab:guide14:sponsor-faults`

Required:

- sponsor/STATE overlap;
- two sponsor envelopes;
- foreign sponsor asset;
- missing sponsor authorization;
- sponsor change in STATE role;
- STATE successor in sponsor range;
- target fee role substituted for sponsor change;
- sponsor change substituted for fee role;
- sponsor member unclassified;
- report contains sponsor amount;
- report contains sponsor opening;
- balanced STATE-corruption attempt hidden by sponsor change;
- zero-valued sponsor member under exact role structure;
- confidential sponsor values where claimed.

## 15.11 ABI and linker faults · `tab:guide14:abi-faults`

Required:

- wrong coordinator;
- duplicate coordinator;
- no coordinator;
- STATE and sponsor positions exchanged;
- wrong total input count;
- wrong total output count;
- wrong transaction version;
- wrong sequence;
- witness item reorder;
- predecessor metadata and successor metadata exchanged;
- announcement leaf from another bundle;
- control block from another program;
- unresolved metadata-schema symbol;
- unresolved lead-bound symbol;
- relocation omitted;
- relocation applied twice;
- duplicate tree leaf;
- conflicting leaf weight;
- candidate shape outside declared bounds;
- raw transaction bypasses safe constructor.

## 15.12 Protocol and report faults · `tab:guide14:protocol-faults`

Required:

- blank protocol record;
- oversized request;
- unterminated request;
- unknown handshake field;
- wrong environment;
- wrong provenance;
- signing response bound to other bytes;
- accepted submission with no transaction identity;
- rejected submission with accepted identity;
- rejected funding step with funded outputs;
- rejected signing step with signature witness;
- constructor response carrying another role’s fields;
- caller-authored coverage verdict;
- operation summary omits generic transcript binding;
- report summary edited;
- failed row removed;
- duplicate passing row inserted;
- canonical report includes wall time.

## 15.13 Public-recoverability faults · `tab:guide14:recoverability-faults`

Required:

- missing transaction;
- copied transaction bytes;
- wrong successor index;
- stale successor;
- malformed metadata publication;
- metadata publication from another successor;
- missing representation nonce;
- wrong static subtree identity;
- creator-local-only metadata;
- unknown publication field;
- external path or wallet dependency;
- reconstructed program differs from chain output.

---

# 16. Root-history evidence · `sec:guide14:history`

## 16.1 Edge sequence, not cursor equality · `rule:guide14:edge-sequence`

The root-history report retains an ordered sequence of typed root edges.

Final cursor equality is insufficient.

A trace such as:

```text
STATE A → invalid STATE B → STATE C
```

does not become valid because the final cursor is `C`.

Every intermediate edge is validated.

## 16.2 One appended certificate · `rule:guide14:one-certificate`

The accepted semantic transition extends predecessor history by exactly one certificate.

The certificate:

- names the maturity-announcement branch;
- carries exact canonical order;
- carries exact STATE succession;
- carries no RESV or other root edge;
- carries no issuance, destruction, or specialized event.

## 16.3 Reorg and checkpoint boundary · `rule:guide14:reorg-boundary`

Where target evidence is tied to a development chain checkpoint, a reorg may stale the root-history observation.

The report binds:

- network;
- genesis;
- block hash;
- block height;
- transaction identity;
- predecessor outpoint;
- successor outpoint.

A report from another canonical chain prefix is not silently reused.

## 16.4 Bundle continuity · `rule:guide14:bundle-continuity`

A predecessor under one static constructor and successor under another requires an explicit migration relation.

Guide 14 implements no migration.

Therefore:

```text
predecessor bundle ≠ successor bundle
    ⇒ reject
```

even where the semantic metadata otherwise agrees.

---

# 17. Resource evidence · `sec:guide14:resources`

## 17.1 Candidate dimensions · `rule:guide14:candidate-dimensions`

Phase 6 evaluates candidate parameters such as:

```text
metadata schema size:
    fixed by typed STATE

static leaf set:
    maturity-announcement candidate only

sponsor input maximum:
    inherited candidate assignments

constructor nonce-search maximum:
    explicit research candidates

tree objective and maximum depth:
    explicit candidate policy
```

These are candidate values, not final deployment calibration.

## 17.2 Complete transaction measurements · `rule:guide14:measurements`

Measure:

- minimum valid announcement;
- maximum valid announcement;
- representative interior announcement;
- predecessor metadata with nontrivial values in every field;
- nonce zero;
- nonce nonzero;
- largest observed nonce-search attempt within the finite corpus;
- sponsorless transaction;
- sponsored transaction;
- sponsor change present;
- sponsor change absent;
- operator signature;
- deepest candidate control path;
- largest metadata witness;
- candidate maximum sponsor shape.

## 17.3 Resource dimensions · `tab:guide14:resource-dimensions`

Record separately:

- announcement program bytes;
- metadata leaf bytes;
- static subtree bytes;
- constructor tree depth;
- control bytes;
- predecessor metadata witness bytes;
- successor metadata witness bytes;
- operator signature bytes;
- sponsor witness bytes;
- initial witness items;
- peak main stack;
- peak alternate stack;
- maximum element bytes;
- hashing operations;
- tweak-verification cost;
- validation budget;
- transaction weight;
- consensus verdict;
- relay-policy verdict;
- nonce-search attempts as diagnostics;
- construction and execution time as diagnostics.

Timing and search-attempt count do not enter semantic identity.

## 17.4 Prediction and observation · `rule:guide14:resource-comparison`

Backend, linker, and ABI resource predictions are compared with real target observations.

A mismatch fails the resource report and operation plan even if the target accepted.

Once a fatal mismatch is observed:

- no later operation step is emitted;
- the generic executor returns operation-plan refusal;
- the report cannot simultaneously state completion.

## 17.5 Separate objectives · `rule:guide14:separate-objectives`

One fixture may maximize:

- program bytes;
- metadata leaf bytes;
- control depth;
- witness bytes;
- stack;
- hash work;
- validation budget;
- transaction weight;
- nonce-search attempts.

No one fixture is claimed to maximize all dimensions unless proved.

## 17.6 Phase-6 result · `rule:guide14:resource-result`

Phase 6 may record:

```text
the tested candidate STATE constructor and maturity ABI fit the
reviewed target under the measured candidate policy
```

It must not record final production STATE bounds or constructor policy.

Final calibration waits for the exact final multi-operation STATE constructor and ABI.

---

# 18. Relation-indexed coverage · `sec:guide14:coverage`

For every relation-case, require:

```text
relation identity
execution case
activation
selected proof
carrier
positive requirement
negative requirement
expected evidence boundary
target verdict where applicable
accepted STATE projection where applicable
constructor projection where applicable
root-history projection where applicable
mutation layer
collateral dependency closure
```

## 18.1 Positive runtime coverage · `rule:guide14:positive-coverage`

Positive runtime coverage requires:

- relation active;
- complete target transaction;
- intended announcement leaf executed;
- target accepted;
- semantic STATE projection matched;
- constructor projection matched where applicable;
- root-history projection matched where applicable.

Target acceptance alone discharges nothing.

## 18.2 Constructor coverage · `rule:guide14:constructor-coverage`

Constructor relations receive independent coverage for:

- predecessor reconstruction;
- successor reconstruction;
- metadata leaf unspendability;
- branch ordering;
- nonce canonicality;
- tweak totality;
- internal key;
- parity;
- control path;
- static subtree continuity.

A semantic maturity match does not satisfy constructor coverage.

## 18.3 Authorization coverage · `rule:guide14:authorization-coverage`

Required:

- recognized operator key and valid signature;
- missing signature;
- empty signature;
- malformed signature;
- wrong operator;
- unknown key form;
- wrong transaction binding;
- wrong sighash;
- post-signing metadata mutation;
- post-signing successor-program mutation.

## 18.4 Root-history coverage · `rule:guide14:history-coverage`

Required:

- valid STATE succession;
- stale predecessor;
- duplicate successor;
- missing edge;
- invalid intermediate edge;
- unexpected termination;
- unexpected RESV edge;
- bundle mismatch.

The final cursor alone cannot satisfy these rows.

## 18.5 Conditional sponsor coverage · `rule:guide14:sponsor-coverage`

Require:

```text
sponsor absent:
    sponsor relation inactive and transaction valid

sponsor present:
    sponsor relation active and valid

sponsor present with overlap:
    sponsor relation active and invalid
```

Sponsor-change presence is its own typed case dimension and not inferred from the class name.

## 18.6 First-party and external evidence · `rule:guide14:boundary-coverage`

A compiler-static or backend-structural negative relation requires a separately stated first-party discharge condition.

It is not target-negative evidence.

External target claims, including selected sighash semantics and whole-transaction conservation, remain explicit report roles bound to exact target, deployment, bundle, ABI, and transaction bytes.

## 18.7 Negative vector ownership · `rule:guide14:negative-ownership`

Every canonical negative vector names:

- intended relation;
- semantic mutation class;
- exact changed fields;
- expected evidence boundary;
- intended carrier;
- collateral closure.

A target rejection discharges a row only when all six agree with the validated operation report.

---

# 19. Assurance boundaries · `sec:guide14:assurance`

## 19.1 Realization · `rem:guide14:realization-assurance`

Establishes target-independent maturity-announcement semantics.

Does not establish metadata bytes, taproot continuity, signature bytes, or target behavior.

## 19.2 Compiler · `rem:guide14:compiler-assurance`

Establishes complete relation, source, constructibility, lifecycle, placement, layout, and coverage analysis.

Does not establish constructor or target behavior.

## 19.3 Target contract · `rem:guide14:target-assurance`

Establishes reviewed static target facts and evidence requirements.

Does not construct or execute STATE.

## 19.4 Tapscript · `rem:guide14:tapscript-assurance`

Establishes typed constructor and announcement patterns, stack validity, and concrete relation placement.

Does not establish final linking, target acceptance, or root history.

## 19.5 Linker · `rem:guide14:linker-assurance`

Establishes exact symbol resolution, reference-cycle policy, deterministic tree assembly, and carrier closure.

Does not establish semantic metadata or operator signing.

## 19.6 Transaction · `rem:guide14:transaction-assurance`

Establishes candidate-ABI-consistent metadata derivation, constructor instantiation, output finalization, and signing requests.

Does not establish production key custody or target acceptance.

## 19.7 Safety evidence · `rem:guide14:safety-assurance`

Establishes finite candidate-specific acceptance and rejection evidence.

Does not prove universal compiler, constructor, or target correctness.

## 19.8 Continuity evidence · `rem:guide14:continuity-assurance`

Establishes predecessor/successor constructor agreement for the tested candidate.

Does not establish every future STATE operation or migration.

## 19.9 Root-history evidence · `rem:guide14:history-assurance`

Establishes the tested STATE succession edge against one exact chain and checkpoint.

Does not establish trusted setup, genesis, production finality, or universal reorg safety.

## 19.10 Phase-6 result · `rem:guide14:phase-assurance`

Establishes one candidate STATE mutation pipeline.

It does not establish:

- clear;
- redemption;
- admission;
- cycle;
- final STATE constructor;
- production operator service;
- production root deployment;
- final calibration;
- release.

---

# 20. Suggested implementation waves · `sec:guide14:waves`

Each wave ends formatted, focused-test green, documented, and committed before the next begins.

## Wave 0 — Revalidate the Phase-5 handoff · `task:guide14:wave0`

**Deliverables**

- Phase-5 gate confirmed on the working tree;
- inherited compiler, signer, linker, ABI, executor, report, and resource boundaries reviewed;
- inherited evidence residuals dispositioned;
- preflight questions answered;
- accepted STATE-constructor research re-read;
- no unexplained identity drift.

**Suggested commit**

```text
plans: charter the STATE and maturity tranche
```

## Wave 1 — Typed STATE metadata and semantic projection · `task:guide14:wave1`

**Deliverables**

- complete typed STATE metadata source;
- canonical encoding;
- semantic projection;
- representation nonce separation;
- exact maturity successor function;
- field-by-field corruption tests;
- public recovery model.

**Suggested commit**

```text
realization: define canonical STATE metadata projection
```

## Wave 2 — Maturity compiler operation plan · `task:guide14:wave2`

**Deliverables**

- validated maturity-announcement operation plan;
- exact relation, carrier, layout, lifecycle, and coverage censuses;
- exact operator source requirements;
- exact root-succession requirements;
- corruption-resistant validator;
- public API tests.

**Suggested commit**

```text
compiler: expose the validated maturity-announcement plan
```

## Wave 3 — Target constructor and signer closure · `task:guide14:wave3`

**Deliverables**

- selected operator sighash profile;
- target-native valid operator signature;
- unknown-key closure;
- streaming-hash and tweak prerequisites;
- target-generic test signer capability;
- branch-order and tweak-totality dispositions;
- exact target evidence roles.

**Suggested commit**

```text
target-elements: close STATE constructor and operator requirements
```

## Wave 4 — Candidate STATE constructor · `task:guide14:wave4`

**Deliverables**

- canonical metadata leaf;
- always-aborting metadata program;
- static maturity subtree;
- predecessor authentication;
- successor reconstruction;
- canonical nonce grinding;
- internal-key policy;
- no key-path escape;
- constructor mutation vectors.

**Suggested commit**

```text
tapscript: implement the candidate STATE constructor
```

## Wave 5 — Maturity-announcement patterns · `task:guide14:wave5`

**Deliverables**

- current STATE recognition;
- maturity predecessor check;
- exact lead-window check;
- exact metadata copy-through;
- operator authorization;
- root succession;
- sponsor isolation;
- absence relations;
- complete abstract schedules.

**Suggested commit**

```text
tapscript: implement maturity announcement
```

## Wave 6 — Linker extension · `task:guide14:wave6`

**Deliverables**

- metadata and static-subtree symbols;
- constructor reference graph;
- SCC disposition;
- structured relocations;
- duplicate-sensitive tree census;
- deterministic candidate tree;
- exact cost arithmetic;
- root-carrier closure;
- candidate linked bundle.

**Suggested commit**

```text
linker: link the maturity-announcement constructor
```

## Wave 7 — Candidate maturity ABI · `task:guide14:wave7`

**Deliverables**

- STATE input/output roles;
- canonical metadata witnesses;
- constructor search;
- successor program derivation;
- sponsor suffix;
- output finalization;
- operator signing request;
- canonical target bytes;
- strict decoder;
- candidate-only status.

**Suggested commit**

```text
transaction: derive the maturity-announcement ABI
```

## Wave 8 — Canonical safety and continuity evidence · `task:guide14:wave8`

**Deliverables**

- canonical semantic fixtures;
- target materialization;
- maturity-window vectors;
- metadata-preservation vectors;
- operator vectors;
- constructor vectors;
- accepted STATE projections;
- validated safety and continuity reports.

**Suggested commit**

```text
vectors: complete maturity and constructor evidence
```

## Wave 9 — Root-history and public-recovery evidence · `task:guide14:wave9`

**Deliverables**

- ordered root-edge report;
- stale and intermediate-edge faults;
- unrelated-process metadata recovery;
- successor reconstruction from public chain data;
- checkpoint and reorg binding;
- explicit synthetic-origin non-claims.

**Suggested commit**

```text
vectors: verify STATE history and public recovery
```

## Wave 10 — Resource study · `task:guide14:wave10`

**Deliverables**

- metadata and constructor measurements;
- nonce-search corpus;
- sponsorless and sponsored transactions;
- operator signature measurements;
- prediction/observation equality;
- explicit non-calibration result.

**Suggested commit**

```text
vectors: measure the maturity-announcement candidate
```

## Wave 11 — Phase-6 gate and handoff · `task:guide14:wave11`

**Deliverables**

- package READMEs;
- package contracts;
- Phase-6 card;
- backlog gate record;
- identity, schema, security, and dependency impact;
- complete repository gate;
- clean final tree.

**Suggested commit**

```text
plans: record the STATE and maturity candidate
```

---

# 21. Focused verification · `sec:guide14:verification`

## 21.1 Working cadence · `rule:guide14:rust-cadence`

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Focused runs supplement but do not replace complete workspace execution.

## 21.2 Architecture and model · `tab:guide14:model-tests`

```sh
cargo test --locked -p tripod-architecture
cargo test --locked -p tripod-model maturity
cargo test --locked -p tripod-model root
cargo test --locked -p tripod-model realization_conformance
```

Focused areas:

```text
maturity field
announcement lead bounds
STATE singleton
operator authorization
unchanged state fields
root certificate
root history
no RESV use
```

## 21.3 Realization · `tab:guide14:realization-tests`

```sh
cargo test --locked -p tripod-realization
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-realization --no-deps
```

Focused areas:

```text
maturity relation census
STATE metadata projection
announcement window
operator constructibility
root succession
sponsor opacity
lifecycle obligations
architecture weld
```

## 21.4 Compiler · `tab:guide14:compiler-tests`

```sh
cargo test --locked -p tripod-compiler
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-compiler --no-deps
```

Focused areas:

```text
maturity target-operation plan
STATE relation census
operator source requirements
root carrier
layout census
coverage census
lifecycle
determinism
corruption validation
no graph handles
no target positions
no digest
```

## 21.5 Target · `tab:guide14:target-tests`

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

Focused areas:

```text
streaming hash
byte-string operations
tweak verification
zero tweak
group-order rejection
operator-key encoding
signature behavior
selected sighash
target resources
evidence registry
```

## 21.6 Tapscript · `tab:guide14:tapscript-tests`

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

Focused areas:

```text
metadata leaf always aborts
canonical metadata encoding
predecessor reconstruction
successor reconstruction
announcement window
operator authorization
unknown-key closure
root succession
sponsor isolation
absence relations
known computed truth
final stack
resource formulas
```

## 21.7 Constructor oracle · `tab:guide14:constructor-tests`

```sh
cargo test --locked -p tripod-target-elements-conformance constructor
```

Focused areas:

```text
tagged hashes
branch ordering
canonical nonce
tweak zero
tweak group-order boundary
identity result
internal key
parity
control block
retry classification
public metadata recovery
```

## 21.8 Linker · `tab:guide14:linker-tests`

```sh
cargo test --locked -p tripod-linker
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-linker --no-deps
```

Focused areas:

```text
metadata/static symbols
two-pass resolution
constructor SCC
duplicate leaf refusal
conflicting weight refusal
structured relocation
taptree determinism
bounded-depth policy
exact cost arithmetic
root carrier closure
candidate status
```

## 21.9 Transaction · `tab:guide14:transaction-tests`

```sh
cargo test --locked -p tripod-transaction
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-transaction --no-deps
```

Focused areas:

```text
maturity ABI
current STATE view
metadata derivation
canonical nonce search
successor program
sponsor consistency
operator signing request
post-signing mutation
witness roles
superfluous witness refusal
decode/encode equality
candidate status
```

## 21.10 Vectors · `tab:guide14:vectors-tests`

```sh
cargo test --locked -p tripod-vectors
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-vectors --no-deps
```

Focused areas:

```text
canonical maturity plan
positive class witnesses
maturity-window faults
metadata copy-through faults
operator faults
constructor faults
root-history faults
accepted projections
transcript-derived coverage
report role separation
resource prediction
report determinism
```

## 21.11 Executor and protocol · `tab:guide14:executor-tests`

Run the inherited executor package’s complete suite.

Focused areas:

```text
public test operator
valid script-path signature
wrong operator
wrong transaction binding
strict bounded requests
blank request refusal
unknown-field refusal
role-correct responses
verdict-correct responses
exact transcript subjects
environment binding
provenance
timeout cleanup
no expected answer in request
```

## 21.12 Documentation · `tab:guide14:documentation-tests`

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new tracked file enters its nearest Meson census in the same commit.

---

# 22. Full batch gate · `gate:guide14:batch`

After every coherent implementation series:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run real target matrices separately:

```text
maturity semantic safety matrix
operator authorization matrix
STATE constructor-continuity matrix
root-history matrix
public metadata recovery matrix
sponsor matrix
candidate resource matrix
```

Record:

```text
target contract revision
deployment binding
network and genesis
executor declaration
adapter version
framework revision
binary-reported revision
intended tip
upstream base
local topic census
candidate linked bundle
candidate ABI
STATE metadata schema
constructor policy
candidate nonce-search bound
case count
relation count
claim count
root-edge count
failures
infrastructure errors
report byte reproducibility
```

If dependencies changed:

```sh
cargo tree --locked -e features
cargo metadata --locked
cargo audit
```

A missing advisory tool is recorded as skipped.

Run document reproducibility when policy requires it:

```sh
scripts/check-document-reproducibility.sh
```

Final check:

```sh
git status --porcelain=v1 --untracked-files=all
```

The result must be empty.

---

# 23. Acceptance criteria · `sec:guide14:acceptance`

Accept the Phase-6 candidate only when:

- Phase 5 remains green;
- STATE semantic scope is exact;
- the complete metadata census has one typed owner;
- realization and compiler relation censuses agree;
- every target requirement is assessed;
- every selected proof is realization-approved;
- every active relation-case has a reachable carrier;
- predecessor STATE is current, canonical, singleton, and unannounced;
- announced cycle lies inside the exact lead window;
- successor maturity is exactly announced at the requested cycle;
- every unaffected STATE field is identical;
- one authorized operator signs the finalized protected transaction;
- unknown key forms cannot satisfy authorization without verification;
- predecessor constructor authenticates;
- successor constructor reconstructs exactly;
- metadata leaf is unspendable;
- static subtree continuity holds;
- branch ordering is canonical;
- nonce selection is deterministic and first-admissible;
- tweak zero is handled according to the exact target rule;
- invalid tweak and identity result reject;
- no key-path escape exists;
- exactly one STATE succession edge exists;
- no invalid intermediate root edge is hidden;
- no RESV or other root participates;
- no issuance, destruction, receipt, ASH, entitlement, control, or vault appears;
- sponsor membership and sponsor-change presence are exact;
- sponsor value remains opaque;
- accepted STATE projections match;
- accepted constructor projections match;
- root-history projections match;
- successor metadata is recoverable from public chain data;
- canonical coverage derives from validated execution;
- resource prediction equals observation;
- a resource mismatch refuses the operation plan;
- candidate constructor and ABI remain lifecycle-incomplete;
- no production operator-key interface is claimed;
- no speculative digest is minted;
- canonical report bytes reproduce;
- full repository gates pass;
- final tree is clean.

---

# 24. Rejection criteria · `sec:guide14:rejection`

Reject the candidate if:

- one semantic relation disappears between packages;
- target details enter compiler core;
- a target capability is treated as a complete constructor proof;
- the STATE metadata census is hand-maintained in two places;
- a synthetic prototype field silently becomes STATE;
- predecessor maturity is not unannounced;
- announced cycle lies outside the exact window;
- arithmetic wraps or clamps;
- one unrelated STATE field changes;
- operator signature is missing or bound to other bytes;
- unknown key encoding succeeds without verification;
- output changes after signing;
- predecessor constructor is not authenticated;
- successor constructor is caller-supplied;
- metadata leaf is spendable;
- branch ordering depends on declaration order;
- nonce search skips an earlier admissible value;
- invalid internal key is retried;
- zero tweak is rejected merely for being zero;
- tree-cost saturation is described as exact;
- duplicate leaves are resolved by last declaration;
- final cursor equality hides an invalid root edge;
- RESV or another root participates;
- issuance or destruction appears;
- sponsor amount becomes protocol data;
- a sponsored request silently becomes sponsorless;
- construction failure counts as target rejection;
- caller-authored status discharges canonical coverage;
- target acceptance substitutes for STATE projection;
- semantic projection substitutes for constructor continuity;
- constructor continuity substitutes for root history;
- operation report discards the subject-bound execution transcript;
- target rejection carries a success artifact;
- protocol framing is unbounded or blank-record tolerant;
- public metadata recovery uses creator memory;
- lifecycle incompleteness is hidden;
- synthetic STATE is described as genesis;
- candidate and final types are conflated;
- a report digest is added without an admitted consumer;
- any full gate fails or leaves the tree dirty.

---

# 25. Identity, schema, security, and dependency impact · `sec:guide14:impact`

Expected identity impact:

```text
Attestation version:
    unchanged

realization version:
    unchanged unless a semantic correction independently requires movement

architecture schema:
    unchanged unless STATE metadata is not already completely represented

architecture semantic hash:
    unchanged unless architecture meaning changes

architecture behavioural hash:
    unchanged unless behaviour changes

anchor-set hash:
    unchanged

compiler identity:
    none minted

STATE constructor identity:
    none minted for in-process candidate use

bundle identity:
    none minted for in-process candidate use

ABI identity:
    none minted for in-process candidate use

safety report identity:
    none minted

continuity report identity:
    none minted

root-history report identity:
    none minted

deployment-profile identity:
    remains dormant
```

Expected schema work:

```text
canonical STATE metadata projection
maturity-announcement compiler operation projection
candidate STATE constructor
candidate linked maturity bundle
candidate maturity ABI
target-generic test signer step
maturity safety evidence plan/report
constructor-continuity report
root-history report
public metadata-recovery report
```

Expected security impact:

```text
public disposable test operator scalar:
    admitted

public constructor metadata:
    admitted

production operator private key:
    not admitted

production signer service:
    not established

production STATE deployment:
    not established
```

Expected dependency impact:

```text
reuse Phase-5 compiler, tapscript, linker, transaction, vectors,
and executor packages

possible extension of target-generic signing support

no third-party dependency without a concrete consumer and ADR-011 review
```

Every schema migration is explicit.

No historical report or protocol revision is silently widened.

---

# 26. Final Phase-6 result matrix · `tab:guide14:result`

The completion report fills the final column.

| Boundary | Required result | Actual result |
|---|---|---|
| architecture/realization | maturity semantics complete | |
| compiler | validated maturity-announcement target plan | |
| target assessment | every capability and evidence role classified | |
| metadata | canonical complete STATE encoding | |
| predecessor constructor | exact authentication | |
| successor constructor | exact reconstruction | |
| operator authorization | finalized transaction signed | |
| linker | deterministic candidate bundle | |
| transaction | deterministic candidate ABI | |
| target execution | positive cases accepted, negative cases rejected | |
| semantic projection | every accepted STATE transition matches | |
| constructor continuity | predecessor/successor recipe matches | |
| root history | one exact STATE succession edge | |
| public recovery | unrelated process reconstructs successor | |
| sponsor isolation | exact and amount-opaque | |
| coverage | every relation-case complete | |
| resources | prediction equals observation | |
| lifecycle | future STATE operations outstanding | |
| secret boundary | public test operator only | |
| identity | no speculative digest | |
| release | not claimed | |

---

# 27. Guide-14 exit checklist · `gate:guide14:exit`

## Entry and inheritance

- [ ] Phase-5 gate passes on the current tree;
- [ ] inherited compiler, signer, linker, ABI, executor, report, and resource interfaces are reviewed;
- [ ] inherited evidence residuals are dispositioned;
- [ ] every preflight question has an explicit answer;
- [ ] accepted constructor research is revalidated;
- [ ] no identity drift is unexplained.

## Metadata and semantics

- [ ] complete STATE metadata census has one typed owner;
- [ ] canonical encoding is exact;
- [ ] semantic and representation fields are separate;
- [ ] predecessor maturity is unannounced;
- [ ] successor maturity is announced at the requested cycle;
- [ ] announcement window is exact;
- [ ] checked arithmetic does not wrap;
- [ ] every unaffected field is equal;
- [ ] reserved fields remain canonical;
- [ ] metadata is publicly recoverable;
- [ ] synthetic prototype fields do not enter silently.

## Compiler

- [ ] maturity plan has no unchecked constructor;
- [ ] operation scope is exact;
- [ ] relation census is exact;
- [ ] operator requirements are complete;
- [ ] STATE root requirements are complete;
- [ ] constructibility is complete;
- [ ] lifecycle is explicit;
- [ ] carrier, layout, and coverage censuses are exact;
- [ ] no graph handles or target positions leak;
- [ ] no compiler digest is minted.

## Constructor and programs

- [ ] metadata leaf always aborts;
- [ ] predecessor constructor authenticates;
- [ ] successor constructor reconstructs;
- [ ] static subtree continuity holds;
- [ ] branch order is canonical;
- [ ] representation nonce is deterministic;
- [ ] first admissible nonce is selected;
- [ ] search exhaustion is typed;
- [ ] zero tweak follows the exact target rule;
- [ ] invalid tweak rejects;
- [ ] identity result rejects;
- [ ] internal-key policy is explicit;
- [ ] no key-path escape exists;
- [ ] final stack and failure states validate.

## Linking and ABI

- [ ] every symbol is typed;
- [ ] every mandatory reference resolves;
- [ ] every constructor cycle has an explicit strategy;
- [ ] duplicate leaves reject;
- [ ] conflicting weights and roles reject;
- [ ] taptree is deterministic;
- [ ] tree-cost arithmetic is exact or checked;
- [ ] root carrier remains reachable;
- [ ] input and output roles are exact;
- [ ] request, sponsor offer, shape, and transaction form agree;
- [ ] successor program is derived, not caller-supplied;
- [ ] output set is finalized before signing;
- [ ] canonical decode/encode equality holds;
- [ ] superfluous witness sections reject;
- [ ] bundle and ABI remain candidate-only.

## Authorization

- [ ] public test operator identity is exact;
- [ ] every successful announcement carries valid operator authorization;
- [ ] selected sighash profile is observed;
- [ ] missing signature rejects;
- [ ] wrong operator rejects;
- [ ] unknown key success is closed;
- [ ] transaction-binding mismatch rejects;
- [ ] post-signing metadata mutation rejects;
- [ ] post-signing output-program mutation rejects;
- [ ] no production private key is accepted.

## Evidence

- [ ] canonical evidence plan has no unchecked constructor;
- [ ] canonical coverage consumes validated execution only;
- [ ] every positive class has a typed witness;
- [ ] valid announcements accept;
- [ ] every required invalid announcement rejects;
- [ ] accepted STATE projections match;
- [ ] constructor projections match;
- [ ] root-history projections match;
- [ ] public recovery succeeds;
- [ ] construction and target failures remain separate;
- [ ] safety, continuity, history, recovery, and resource reports remain distinct;
- [ ] operation report retains the generic execution transcript;
- [ ] target and deployment binding is exact;
- [ ] executor provenance validates;
- [ ] zero required infrastructure errors;
- [ ] no mock satisfies the gate.

## Protocol and reports

- [ ] request and response framing is strict and bounded in both directions;
- [ ] blank records reject;
- [ ] unknown fields reject;
- [ ] response fields match their operation role;
- [ ] target rejection carries no success artifact;
- [ ] accepted steps carry required observations;
- [ ] report validator recomputes case, relation, edge, and summary censuses;
- [ ] canonical report bytes exclude timing and host data;
- [ ] report bytes reproduce.

## Root history

- [ ] exactly one STATE predecessor is consumed;
- [ ] exactly one STATE successor is created;
- [ ] one succession edge is derived;
- [ ] no STATE termination occurs;
- [ ] no stale predecessor passes;
- [ ] no duplicate successor passes;
- [ ] invalid intermediate edges reject;
- [ ] final cursor restoration cannot hide an invalid trace;
- [ ] no RESV, PACE, or authority edge occurs;
- [ ] bundle mismatch rejects without migration.

## Resources

- [ ] metadata leaf measured;
- [ ] announcement program measured;
- [ ] static subtree measured;
- [ ] operator witness measured;
- [ ] sponsorless transaction measured;
- [ ] sponsored transaction measured;
- [ ] control depth and bytes measured;
- [ ] nonce-search corpus reported;
- [ ] predicted and observed resources agree;
- [ ] mismatch refuses the plan and stops later work;
- [ ] consensus and policy remain separate;
- [ ] candidate parameters remain non-final.

## Repository

- [ ] package READMEs are current;
- [ ] package contracts are current;
- [ ] phase index, roadmap, active card, and backlog agree;
- [ ] Phase-6 card is current;
- [ ] backlog is current;
- [ ] every new file is in the Meson census;
- [ ] dependency review is recorded;
- [ ] security impact is recorded;
- [ ] identity and schema impact is recorded;
- [ ] `cargo fmt --all` passes;
- [ ] Clippy passes with warnings denied;
- [ ] workspace tests pass;
- [ ] `scripts/ci.sh` passes;
- [ ] Meson compile passes;
- [ ] Meson tests pass;
- [ ] advisory status is passed or explicitly skipped;
- [ ] document reproducibility is passed or explicitly deferred under policy;
- [ ] `git diff --check` passes;
- [ ] final repository status is empty.

---

# 28. Completion report template · `sec:guide14:report-template`

```text
Guide 14 result
===============

Starting state:
    source revision:
    working tree:
    Phase-5 result:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    realization version:
    target contract revision:
    executor protocol revision:
    STATE-constructor research revision:
    maturity compiler projection:

Preflight:
    Phase-5 interface reuse:
    metadata census:
    constructor promotion:
    constructor reference cycle:
    metadata publication:
    branch ordering:
    tweak totality:
    operator signing:
    root-history evidence:
    synthetic STATE:
    report roles:
    clean tree:

Semantic STATE:
    predecessor maturity:
    current cycle:
    announced cycle:
    minimum lead:
    maximum lead:
    successor maturity:
    unaffected field census:
    unaffected fields preserved:
    root effect:
    RESV effect:
    issuance:
    destruction:
    specialized events:

Compiler:
    target-operation type:
    scope:
    relation census:
    operator requirements:
    metadata requirements:
    root requirements:
    constructibility:
    lifecycle:
    carriers:
    layout:
    coverage:
    external evidence:
    graph handles exposed:
    identity minted:

Target assessment:
    reviewed target:
    deployment binding:
    streaming hash:
    branch hashing:
    tweak verification:
    operator-key encoding:
    signature semantics:
    sighash profile:
    input/output introspection:
    missing primitives:
    unsupported capabilities:
    structural obligations:
    external evidence:

STATE metadata:
    typed source:
    schema:
    domain:
    fields:
    semantic fields:
    representation fields:
    reserved fields:
    canonical bytes:
    decoder:
    public recovery:
    synthetic fields promoted:

Constructor:
    tree shape:
    metadata leaf:
    static subtree:
    predecessor authentication:
    successor reconstruction:
    canonical branch side:
    representation nonce:
    search bound:
    attempts:
    tweak result:
    internal key:
    key-path result:
    leaf version:
    control depth:
    outstanding assumptions:

Tapscript:
    STATE recognition:
    maturity predecessor:
    announcement window:
    metadata copy-through:
    operator authorization:
    root succession:
    sponsor isolation:
    absence relations:
    final stack:
    resource formula:

Linker:
    symbols:
    definitions:
    references:
    SCCs:
    cycle strategies:
    relocations:
    duplicate-leaf result:
    taptree:
    tree-cost arithmetic:
    carrier closure:
    candidate bundle:
    deterministic rebuild:

Transaction ABI:
    input layout:
    output layout:
    coordinator:
    current-STATE view:
    successor metadata:
    successor program:
    sponsor suffix:
    sponsor change:
    fee role:
    request fields:
    constructor search:
    output finalization:
    operator signing request:
    witness roles:
    canonical decode:
    candidate parameters:
    candidate status:

Operator signing:
    public test identity:
    selected sighash:
    exact bytes signed:
    signature result:
    wrong-operator result:
    missing-signature result:
    unknown-key result:
    post-signing mutation result:
    production-key claim:
        none

Synthetic STATE:
    funding method:
    STATE asset:
    predecessor metadata:
    constructor:
    target outpoint:
    protocol-genesis claim:
        none
    prior-root-history claim:
        none
    production-STATE claim:
        none

Safety evidence:
    canonical evidence mutation boundary:
    positive class witnesses:
    positive cases:
    semantic failures:
    metadata failures:
    operator failures:
    predecessor failures:
    successor failures:
    sponsor failures:
    ABI failures:
    target verdicts:
    accepted STATE projections:
    relation coverage:
    infrastructure errors:
    report bytes:

Constructor continuity:
    predecessor metadata leaf:
    successor metadata leaf:
    static subtree:
    predecessor root:
    successor root:
    predecessor program:
    successor program:
    parity:
    control path:
    oracle comparison:
    outstanding equality:

Root history:
    predecessor cursor:
    successor cursor:
    edge count:
    edge sequence:
    stale-predecessor result:
    duplicate-successor result:
    intermediate-edge result:
    RESV edge:
    other root edges:
    checkpoint:
    reorg binding:
    report bytes:

Public recovery:
    independent process:
    transaction located:
    bytes matched:
    metadata recovered:
    metadata decoded:
    constructor reconstructed:
    successor program matched:
    creator-private dependency:
        none

Resources:
    announcement program bytes:
    metadata leaf bytes:
    static subtree bytes:
    constructor bytes:
    control bytes:
    predecessor metadata witness:
    successor metadata witness:
    operator signature bytes:
    sponsor witness bytes:
    initial witness items:
    peak stack:
    peak altstack:
    largest item:
    hash work:
    tweak cost:
    validation budget:
    sponsorless weight:
    sponsored weight:
    nonce-search attempts:
    prediction mismatch result:
    candidate nonce bound:
    final calibration claim:
        none

Lifecycle:
    announce maturity:
        candidate implemented
    admission:
        outstanding
    clear:
        outstanding
    redemption:
        outstanding
    cycle:
        outstanding
    release-complete:
        false

Security:
    production operator keys accepted:
        no
    public disposable test operator:
    public STATE metadata:
    future secret-bearing design required:

Identity impact:
    Attestation:
    realization:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    anchor-set hash:
    compiler identity:
        none
    STATE constructor identity:
        none
    bundle identity:
        none unless separately admitted
    ABI identity:
        none unless separately admitted
    report identities:
        none
    deployment profile:
        dormant

Dependency impact:
    first-party changes:
    third-party additions:
    Cargo.lock:
    licences:
    MSRV:
    unsafe/FFI:
    advisories:

Verification:
    cargo fmt --all:
    cargo clippy --workspace --all-targets --locked -- -D warnings:
    cargo test --workspace --locked:
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
    target executor:
    protocol cross-language:
    Rustdoc:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    real maturity safety matrix:
    real operator matrix:
    real constructor matrix:
    real root-history matrix:
    real public-recovery matrix:
    real resource matrix:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Phase-6 verdict:
    accepted candidate / constructor deferred / rejected target path /
    blocked

Residuals:

Next phase:
```

---

# 29. Handoff after Guide 14 · `sec:guide14:handoff`

If Guide 14 succeeds, Phase 6 has one complete mutable-STATE candidate and one target-evidenced STATE succession.

The next phase should implement burn, ASH, and clear:

```text
Guide 15 — Burn, ASH, and Clear
```

Guide 15 consumes rather than reopens:

- canonical STATE metadata;
- authenticated predecessor and successor constructors;
- public metadata recovery;
- static-subtree continuity;
- representation-nonce policy;
- internal-key and key-path policy;
- operator-authorized finalized signing flow;
- STATE root-history evidence;
- validated operation reports;
- target/deployment/provenance binding;
- relation-indexed coverage;
- candidate resource measurement.

It adds:

- owner-authorized burn;
- fresh ASH construction;
- burn-event type and value anchors;
- public ASH maintenance;
- STATE-changing permissionless clear;
- `tag-recon` destruction;
- residual ASH;
- clear-event projection;
- the first use of the STATE constructor by a permissionless operation;
- interaction between mutable STATE and public amount-dependent semantics.

Guide 14 itself makes none of those burn, ASH, clear, destruction, or attestation claims.

---

## Closing statement · `rem:guide14:closing`

> Maturity announcement is the first operation whose semantic effect is one field and whose implementation burden is an entire mutable-root boundary. Guide 14 succeeds only when the current canonical STATE is authenticated, one announced successor is derived from complete typed metadata, every unaffected field is preserved, the operator signs one finalized transaction, predecessor and successor share one exact static constructor, branch ordering and tweak totality are handled deterministically, one root-history edge is independently recovered, successor metadata remains public to an unrelated process, and no target acceptance, final cursor, constructor hash, or report summary is allowed to stand in for that welded chain.