# Draft: Guide 12 Concept — End-to-End Compact ASH

> **Status:** Concept draft; not an execution guide
> **Phase:** Phase 4 — End-to-End Compact ASH
> **Entry:** Phase-3 exit; Guide-11 public-declassification decision recorded; Guide-11 evidence-boundary preflight complete
> **Primary semantic operation:** `compact-ash`
> **Affected packages:** `compiler`, `target-elements`, `tapscript`, future `linker`, future `transaction`, future `vectors`
> **Evidence support:** `target-elements-conformance`, subject to the operation-evidence ownership decision below
> **May affect after acceptance:** `release`, package contracts, Phase-4 card, backlog
> **Does not implement:** burn, clear, STATE, RESV, settlement, redemption, cycle, a production signer, wallet, calibrated deployment bound, production deployment, or release
> **Required result:** one complete candidate compiler-to-target path for `compact-ash`, with a typed candidate bundle, candidate transaction ABI, complete real target transactions, relation-indexed positive and negative evidence, and an explicit non-production status
> **Authority:** Attestation, Realization, typed architecture, implemented ADRs, accepted decisions, and the accepted Guide-11 representation policy take precedence over this concept
> **Trust boundary:** repository source and selected executors are executable; untrusted contributions run only in an externally established secretless environment under ADR-015

---

## Mission · `sec:guide12:mission`

Guide 12 proves the complete implementation pipeline on the smallest root-free, ownerless, permissionless attestation-contract operation:

```text
typed architecture
    ↓
target-independent realization
    ↓
validated compiler analysis
    ↓
target-selected operation plan
    ↓
typed tapscript proof patterns and programs
    ↓
relocatable target bundle
    ↓
linked candidate bundle
    ↓
candidate transaction/witness ABI
    ↓
complete valid Elements transactions
    ↓
real target execution
    ↓
relation-indexed semantic projection comparison
```

The semantic operation is compact ASH:

```text
two or more ASH inputs
    ↓
one ASH output

sum(input ASH values)
    =
output ASH value
```

The operation:

- is permissionless;
- is root-free;
- performs no issuance;
- performs no destruction;
- emits no burn, clear, or residue event;
- preserves the closed monetary asset `U`;
- may carry one optional fee-sponsor envelope;
- must remain independent of every individual sponsor amount;
- requires the resulting ASH to remain publicly usable under the accepted Guide-11 representation policy.

Compact ASH is deliberately selected first because it avoids:

- STATE constructor continuity;
- RESV succession;
- wide floor arithmetic;
- owner authorization over protocol value;
- issuance and destruction;
- event production;
- maturity and cadence;
- formula-bound payouts.

It does not avoid the hard compiler-to-target seams:

- relation preservation;
- proof-pattern selection;
- family cardinality and layout;
- closed-asset recognition;
- exact canonical partitioning;
- sponsor-region isolation;
- permissionless constructibility;
- output constructor binding;
- transaction ABI generation;
- target execution;
- resource accounting;
- relation-indexed evidence.

The guide succeeds only when every semantic relation reaches an identifiable target enforcement or evidence boundary. A transaction that happens to compact two values is insufficient if the compiler cannot account for why every required relation remains enforced.

---

## One-line thesis · `rem:guide12:thesis`

> Guide 12 must show that one exact compiler analysis of compact ASH becomes one typed, linked, ABI-driven Elements transaction relation whose every accepted transaction preserves the realization semantics and whose focused invalid variants fail at the evidence layer that owns the violated claim—without importing target details into compiler core, minting speculative identities, or claiming production readiness.

---

# 1. Executive rulings · `sec:guide12:rulings`

## 1.1 The first pipeline is complete or it is not a pipeline

Guide 12 must not stop at any of these intermediate states:

```text
compiler produced capabilities

tapscript emitted instructions

linker produced bytes

transaction builder produced a transaction

node accepted a transaction
```

The required result is the entire chain:

```text
semantic relation
    ↓
compiler-owned target requirement
    ↓
selected target proof
    ↓
reachable target carrier
    ↓
linked program
    ↓
ABI position and witness role
    ↓
complete target transaction
    ↓
real target verdict
    ↓
accepted semantic projection
```

A missing edge is a typed failure.

## 1.2 Compact ASH remains target-independent above the backend

The architecture and realization continue to own:

- operation identity;
- object families;
- cardinalities;
- authorization;
- exact `U` conservation;
- sponsor isolation;
- root absence;
- projection policy;
- lifecycle requirements;
- representation alternatives.

Compiler core continues to own:

- relation analysis;
- proof alternatives;
- disclosure;
- source requirements;
- constructibility;
- lifecycle;
- abstract carrier requirements;
- layout requirements;
- abstract target requirements;
- coverage requirements.

Only the backend and later packages may own:

- opcode selection;
- stack schedules;
- concrete input and output positions;
- tapleaves;
- taptrees;
- control blocks;
- target metadata encoding;
- witness item order;
- transaction serialization.

No Guide-12 convenience may move a target position into architecture, realization, or compiler relation identity.

## 1.3 No relation disappears at a package boundary

At each stage require exact equality between the in-scope relation census and the next stage’s subject census:

```text
realization relation census
=
compiler analyzed relation census
=
target operation-plan relation census
=
backend selected-proof relation census
=
linked reachable-carrier relation census
=
ABI relation census
=
operation vector coverage census
```

A package may add target-owned structural obligations. It may not remove a semantic relation.

## 1.4 Target acceptance and semantic acceptance are different verdicts

A real node accepting a transaction establishes a target verdict about exact transaction bytes.

It does not by itself establish:

- that the transaction was intended to represent compact ASH;
- that the semantic input and output objects were projected correctly;
- that no relation was omitted from compilation;
- that the accepted output denotes the intended successor;
- that the report’s claim census matches the compiler’s relation census.

Every accepted target transaction therefore receives a separate semantic projection comparison.

The successful condition is:

```text
target accepted
∧
accepted projection equals the expected compact-ASH projection
```

## 1.5 Construction rejection and target rejection remain distinct

A malformed operation request may fail before target execution because:

- the request is invalid;
- an input is duplicated;
- the ABI family counts are wrong;
- the constructor cannot instantiate;
- a signing witness is unavailable;
- a confidential proof cannot be built;
- the transaction is not representable.

Those are construction or ABI failures.

They count as target-negative evidence only when the claimed relation is itself a constructor or ABI relation.

A relation whose claim is “the target program rejects this transaction” requires the transaction to be materialized and executed.

## 1.6 Sponsor opacity survives concrete lowering

Guide 12 inherits the realization's sponsor-erasure law.

A target program, ABI, report, or canonical projection must not consume an individual sponsor amount as protocol data.

The concrete sponsor relation is established through:

- exact sponsor-region membership;
- sponsor/protocol disjointness;
- exact family and asset roles;
- sponsor-owner authorization;
- at most one sponsor envelope;
- complete transaction conservation under the target;
- independent enforcement of the `U` protocol relation;
- role-based rather than value-based anchor and fee-output recognition.

A sponsor amount is never required to be:

- explicit for protocol use;
- decoded;
- opened;
- compared with zero;
- proved positive;
- aggregated into a public integer;
- emitted in a public report.

The transaction constructor may use sponsor-private wallet data to balance and sign the transaction. That data remains sponsor-local and outside the protocol projection.

## 1.7 Public ASH representation is a Guide-11 input

Guide 12 does not reopen public declassification.

The accepted Guide-11 matrix determines which ASH value representations the backend may use.

The minimum Phase-4 requirement is:

```text
ASH is publicly and authentically usable
by an unrelated permissionless constructor
```

Possible accepted forms are:

```text
Explicit

PublicCommitted with durable authenticated opening
```

`PrivateCommitted` is not an admissible ASH representation for compact ASH because an unrelated permissionless actor must read or otherwise exactly prove the aggregate relation.

If Guide 11 selects an explicit-only public-maintenance policy, Guide 12 uses explicit ASH and does not keep a dormant confidential branch.

## 1.8 Prototype decisions do not become production patterns silently

Guide 10 accepted:

- a synthetic metadata-constructor continuity prototype;
- a wide-floor arithmetic prototype.

Compact ASH requires no wide-floor arithmetic.

The accepted constructor prototype may inform linker design, but it does not automatically become the ASH constructor. ASH has no semantic metadata field in the current model beyond its asset, amount, object family, and lineage.

Guide 12 must choose the simplest constructor sufficient for ASH and must not import the synthetic metadata schema merely because it exists.

## 1.9 Candidate and final states remain distinct

Guide 12 may produce:

```text
CandidateRelocatableBundle
CandidateLinkedBundle
CandidateTransactionAbi
CandidateOperationEvidence
```

It must not produce:

```text
FinalLinkedBundle
FinalTransactionAbi
ValidatedDeploymentRelease
ProductionTargetEvidence
```

Final calibrated bounds and production release belong to Phase 12.

## 1.10 No speculative digest is introduced

Guide 12 mints no digest merely because new cross-package objects appear.

Typed in-process values use exact typed comparison.

Complete bytes embedded in operation reports use exact byte comparison.

A digest is admitted only if a real cache, publication, distribution, deployment, or release consumer appears and ADR-016 admission is satisfied.

No field is reserved for a future hash.

---

# 2. Entry conditions · `sec:guide12:entry`

Guide 12 begins only when all applicable conditions hold.

## 2.1 Semantic entry

- architecture release validation passes;
- compact ASH remains in the validated realization scope;
- compiler analysis for compact ASH remains complete;
- the realization relation census equals the compiler relation census;
- Guide-11 representation policy is accepted;
- ASH has at least one publicly constructible representation;
- sponsor-value opacity remains enforced;
- no new target-specific field has entered realization or compiler core.

## 2.2 Target entry

- reviewed target contract validates;
- target contract version is unambiguous;
- every backend-used primitive has complete success and failure behavior;
- signature behavior is fully welded;
- selected sponsor signature/sighash behavior has target-native evidence;
- execution domain and leaf version are bound to the development environment;
- exact value/asset introspection used by the candidate is reviewed;
- required target resource interfaces are typed.

## 2.3 Evidence entry

Guide 11’s evidence-boundary repair is complete:

- canonical fixture and prototype subjects have unforgeable trust states;
- arbitrary fixtures cannot reach evidence gates;
- transcripts bind exact requests, target, and deployment;
- executor requests do not contain expected outcomes;
- environment identity is observed and rechecked;
- failed reports cannot pass a gate;
- executor classification is explicit;
- ADR-018 provenance is validated;
- protocol records are bounded and strict;
- executor process cleanup covers startup and timeout paths.

Guide 12 must not build operation release claims on an evidence layer that still allows caller-authored claims to certify unrelated scripts.

## 2.4 Repository entry

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

Record:

```text
starting revision
target contract revision
Guide-11 result revision
compiler pilot projection
clean-tree result
```

---

# 3. Required reading and authority · `sec:guide12:authority`

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

Accepted implementation decisions:

```text
plans/decisions/001-typed-rust-source.md
plans/decisions/002-realization-layer.md
plans/decisions/003-tapscript-first.md
plans/decisions/004-translation-validation.md
plans/decisions/005-value-representation.md
plans/decisions/006-transaction-abi.md
plans/decisions/007-petgraph-graph-substrate.md
plans/decisions/008-exact-certified-mathematics.md
```

Normative realization sections include:

```text
docs/attestation/realization.md
    architecture open/closed recognition
    compact ASH operation
    exact canonical partition
    exact open-flow partition
    sponsor-value opacity
    representation conformance
    translation discipline
    invariant accounting
    oracle obligations
    pins for flow, sponsor, transfer, account, and CT
```

Relevant imports include:

```text
(`[RZ-sec:operations:compact]`)
(`[RZ-trap:architecture:open-closed]`)
(`[RZ-trap:architecture:recognize-unforgeable]`)
(`[RZ-sec:kernel:canonical-partition]`)
(`[RZ-sec:kernel:open-flow]`)
(`[RZ-sec:kernel:sponsor-opacity]`)
(`[RZ-def:representation:sponsor-erasure]`)
(`[RZ-rule:translation:kernel-inline]`)
(`[RZ-rule:translation:cross-utxo]`)
(`[RZ-rule:translation:certificate-leaf]`)
(`[RZ-pin:pins:flow]`)
(`[RZ-pin:pins:sponsor]`)
(`[RZ-pin:pins:account]`)
(`[RZ-pin:pins:ct]`)
```

Package contracts:

```text
plans/packages/compiler.md
plans/packages/target-elements.md
plans/packages/tapscript.md
plans/packages/linker.md
plans/packages/transaction.md
plans/packages/vectors.md
```

Research inputs:

```text
plans/research/public-declassification.md
plans/research/state-constructor.md
plans/research/wide-arithmetic.md
plans/research/compiler-algorithms.md
plans/research/linker-algorithms.md
```

No package parses these Markdown files as semantic input.

---

# 4. Semantic operation contract · `sec:guide12:semantics`

## 4.1 Inputs and output

Let the operation consume \(n\) ASH objects:

\[
2 \le n \le \operatorname{ASH\_BATCH\_MAX}
\]

Each ASH input carries:

```text
object family: ASH
asset: U
value: x_i > 0
owner: none
publicly usable representation
```

The operation creates exactly one ASH output:

```text
object family: ASH
asset: U
value: X
owner: none
publicly usable representation
```

where:

\[
X=\sum_{i=0}^{n-1}x_i
\]

and \(X\) remains in the protocol amount domain:

\[
0<X<2^{51}
\]

## 4.2 Semantic effects

Compact ASH:

- consumes no root;
- creates no root;
- reads no STATE;
- changes no STATE quantity;
- issues nothing;
- destroys nothing;
- moves `U` ownerlessly from ASH inputs to one ASH output;
- emits no burn record;
- emits no burn projection;
- emits no clear projection;
- emits no distribution-residue projection;
- requires a transition-certificate projection at the model/semantic level;
- may use one optional fee-sponsor envelope.

## 4.3 Authorization

The protocol operation is permissionless.

No protocol owner, refund key, operator key, or cadence secret authorizes compact ASH.

A sponsor input remains separately sponsor-owner authorized. That authorization protects sponsor funds and does not authorize the compact-ASH semantic operation.

## 4.4 Canonical partition

The exact canonical `U` flow is:

\[
\sum \operatorname{value}(\text{ASH inputs})=\operatorname{value}(\text{ASH output})
\]

The movement kind is:

```text
ownerless-lateral
```

There is:

```text
no issuance
no destruction
no second U flow
no unclaimed U source
no unclaimed U destination
```

## 4.5 Open-flow partition

Compact ASH has no protocol L-BTC flow.

The only admitted open flow is the optional fee-sponsor envelope:

```text
ordinary sponsor L-BTC inputs
    →
optional sponsor change
+
target transaction fee
```

Protocol predicates retain:

```text
sponsor region present or absent
exact membership
owner authorization
role structure
balance verdict
```

They erase:

```text
individual sponsor values
sponsor denominations
sponsor openings
sponsor blinders
sponsor-local change allocation
```

## 4.6 Projection policy

Required:

```text
transition certificate
```

Forbidden:

```text
burn event
clear event
distribution residue
```

Target execution need not serialize a certificate. It must enforce a transaction relation from which the same certificate is derived.

## 4.7 Lifecycle

The compacted ASH output requires future exits:

```text
compact ASH
clear
```

Phase 4 implements only compact ASH.

Therefore the candidate bundle is explicitly lifecycle-incomplete. It is not a releasable ASH object profile until clear exists or another accepted lifecycle rule is adopted.

The candidate transaction fixtures may create synthetic test ASH under the candidate constructor. They must not describe that test constructor as a production ASH constructor.

---

# 5. Exact relation inventory · `sec:guide12:relations`

The compiler derives the actual relation census. Guide 12 must not replace it with a hand-maintained count.

The expected families include at least:

## 5.1 Cardinality

- ASH input minimum and maximum;
- ASH output exactly one;
- sponsor input minimum zero and bounded maximum;
- sponsor output zero or one.

## 5.2 Recognition

- every ASH input is recognized as `ASH` carrying `U`;
- the ASH output is recognized as `ASH` carrying `U`;
- sponsor inputs and output are recognized as ordinary L-BTC where present;
- the target fee output, if the ABI carries one, is recognized by its target role and not as `PLAIN_LBTC`.

## 5.3 Authorization

- operation is permissionless;
- sponsor inputs require their own owners;
- no hidden protocol signature exists.

## 5.4 Family closure

Input families are exactly:

```text
ASH
optional sponsor L-BTC
```

Output families are exactly:

```text
one ASH
optional sponsor change
target fee output where the selected target transaction form requires one
```

No other closed-asset-capable output is admitted.

## 5.5 Conservation and partition

- exact ownerless `U` conservation;
- exact canonical partition;
- complete source and destination membership;
- no issuance;
- no destruction;
- no overlap between canonical and open-flow references.

## 5.6 Sponsor relations

- exact fee-sponsor region;
- sponsor/protocol disjointness;
- at most one sponsor envelope;
- sponsor owners authorize;
- sponsor amounts remain erased;
- whole-transaction L-BTC conservation remains target evidence.

## 5.7 Root and projection policy

- every architecture root is forbidden;
- transition certificate required;
- every specialized projection forbidden.

## 5.8 Constructibility and representation

- public permissionless constructibility;
- accepted public ASH representation only;
- no owner-private dependency;
- no operator-private dependency;
- no sponsor amount enters protocol arithmetic.

## 5.9 Lifecycle

- compact exit represented;
- clear exit remains outstanding and explicit;
- candidate bundle cannot be marked lifecycle-complete.

---

# 6. Compiler public boundary · `sec:guide12:compiler-boundary`

## 6.1 Current boundary is insufficient for emission

The current compiler public target boundary exposes only:

```text
required capability census
external evidence-role census
```

That is sufficient for static target assessment and insufficient for backend emission.

A backend also needs typed, validated access to:

- relation identities;
- selected proof alternatives;
- relation activation cases;
- authenticatable fact-source requirements;
- constructibility requirements;
- representation selections;
- abstract carrier requirements;
- layout requirements;
- coverage requirements;
- unresolved external evidence.

Guide 12 must add the smallest public compiler-owned projection that a backend can consume without exposing mutable internals or graph handles.

## 6.2 Proposed operation-plan state

Illustrative shape:

```rust
pub struct ValidatedTargetOperationPlan {
    operation: architecture::OperationId,
    architecture_binding: realization::ArchitectureBinding,
    scope: compiler::CompilationScope,
    cases: Vec<TargetExecutionCase>,
    relations: Vec<TargetRelationRequirement>,
    carriers: Vec<AbstractCarrierRequirement>,
    layout: Vec<TargetLayoutRequirement>,
    coverage: Vec<TargetCoverageRequirement>,
    capabilities: TargetRequirementSet,
}
```

The exact fields are implementation-owned.

Required properties:

- private invariant-bearing fields;
- no public unchecked constructor;
- produced only from the complete validated analyzed program;
- exact compact-ASH operation scope;
- canonical ordering by stable typed key;
- no Petgraph indices;
- no target opcode;
- no concrete transaction index;
- no target byte;
- no filesystem or environment input;
- complete owner revalidation on construction;
- independent assembly validation.

## 6.3 No compiler identity yet

If `tapscript` consumes this value in-process as a typed Rust value, exact typed comparison is sufficient.

Do not mint a compiler-plan digest merely because the value becomes public.

Revisit identity only if the plan is:

- serialized for another process;
- cached independently;
- published;
- consumed by a separately versioned external backend.

## 6.4 Operation factorization remains intact

The compiler stores operation analysis per operation.

Guide 12 consumes only the compact-ASH factor.

It must not materialize a product with live transfer or another operation.

## 6.5 Backend admissibility gate

Before emission, the compiler plan must establish:

```text
operation scope is exactly compact-ash
relation census complete
proof selection complete for emitted relations
representation selection compatible with Guide 11
constructibility public
lifecycle incompleteness explicit
all abstract carriers present
all layout requirements present
coverage requirements complete
external evidence obligations retained
```

A plan exceeding explicit analysis limits fails with a typed error and no partial operation plan.

---

# 7. Target capability assessment · `sec:guide12:target-assessment`

## 7.1 Static assessment precedes pattern construction

The adapter assesses every compiler-required capability against the reviewed target.

Expected capability families include:

```text
authenticated object recognition
authenticated family cardinality
authenticated canonical partition
authenticated open-flow partition
authenticated root effects
authenticated projection set
exact public amount arithmetic or accepted public-commitment proof
whole-transaction value conservation
sponsor-owner authorization
public constructibility
```

Root effects and specialized projections may be discharged structurally as absences, but the target plan must still account for them.

## 7.2 Multi-state result remains visible

Every capability remains one of:

```text
unsupported

missing reviewed primitive

backend pattern required

backend structural obligation

external evidence required

complete backend pattern
```

No boolean support flag is introduced.

## 7.3 Missing capability is not a weakened operation

If no accepted proof remains for exact ASH value conservation under the selected representation, emission fails.

The backend must not:

- omit conservation;
- require sponsor values instead;
- fall back to an unauthenticated metadata amount;
- accept private ASH requiring unavailable owner openings;
- rely on transaction conservation to prove object-family closure.

## 7.4 Pattern admission is a separate step

A static target assessment saying the primitives exist does not say a complete backend pattern has been implemented.

Guide 12 must produce and validate the actual patterns before `CompleteBackendPattern` can acquire its first value.

A prototype pattern may remain visibly experimental until the complete operation evidence passes.

---

# 8. Candidate concrete transaction ABI · `candidate:guide12:canonical-layout`

## 8.1 Proposed input layout

The initial candidate uses canonical outpoint order within families.

### Sponsorless case

```text
input 0:
    coordinator ASH

inputs 1..N-1:
    member ASH
```

### Sponsored case

```text
input 0:
    coordinator ASH

inputs 1..N-1:
    member ASH

inputs N..M-1:
    sponsor L-BTC inputs
```

where:

```text
2 ≤ N ≤ candidate ASH bound
0 ≤ M-N ≤ candidate sponsor-input bound
```

The first ASH input is the canonical coordinator because it is the least ASH outpoint in the canonical ASH family order.

No caller selects another coordinator.

## 8.2 Proposed output layout

### Sponsorless case

```text
output 0:
    successor ASH
```

A zero-fee target transaction form may be used only if consensus permits it and the report says that policy relay is not claimed.

If the target requires an explicit fee output even for zero fee, that output receives its own target-only role and fixed position.

### Sponsored case

```text
output 0:
    successor ASH

output 1:
    optional sponsor change, present under the ABI’s exact condition

last output:
    target fee output, where required by the selected Elements transaction form
```

The target fee output is not a protocol object and never satisfies `PLAIN_LBTC`.

Its identity is structural:

```text
target fee-output role
+
canonical position
+
target program form
+
explicit reserve asset
```

not:

```text
ordinary output has value zero
```

## 8.3 Why separate sponsorless and sponsored coordinator leaves

The reviewed target has no admitted general conditional-branch primitive.

Rather than encode one attacker-selected branch inside one script, the candidate uses distinct operation leaves:

```text
compact-ash-sponsorless coordinator

compact-ash-sponsored coordinator
```

Both enforce the same compact-ASH semantic relation.

The sponsored leaf additionally enforces the sponsor-region shape.

Member inputs may use one shared member leaf if that leaf is valid in both execution cases and every case-specific global relation remains on the coordinator.

Otherwise, use distinct member leaves too. The choice is made by the placement proof, not by convenience.

## 8.4 Coordinator uniqueness

The coordinator leaf requires:

```text
current input index = first ASH input index
```

Every member leaf requires:

```text
current input lies inside the remaining ASH range
```

Therefore:

- a member leaf at input zero rejects;
- a coordinator leaf at any later input rejects;
- exactly one coordinator executes in every valid transaction.

The rule must be target-enforced, not merely emitted by the transaction builder.

## 8.5 Sponsor range

The sponsor range is the canonical input suffix after the ASH family.

The protocol proof authenticates:

- suffix start;
- suffix end;
- every member is ordinary L-BTC under an allowed sponsor input profile;
- no ASH input appears in the sponsor suffix;
- no sponsor input appears in the ASH range;
- all sponsor owners authorize the finalized target transaction.

It does not read individual sponsor values.

## 8.6 Optional sponsor change

The ABI defines one optional sponsor-change role.

A first-party builder omits a known explicit zero-valued change output.

That is construction policy, not protocol semantics.

The target relation admits the optional role based on exact layout and family rules, not on a positivity test.

For confidential sponsor values, output presence may leak whether change exists. The ABI records that shape leakage honestly.

## 8.7 Canonical ordering

Canonical ordering is:

```text
ASH inputs:
    ascending canonical outpoint order

sponsor inputs:
    ascending canonical outpoint order within the sponsor suffix

protocol outputs:
    fixed ABI role order

sponsor change:
    one fixed optional role

fee output:
    fixed target role
```

The builder rejects duplicates before sorting.

Caller order is not semantic and does not enter identity.

---

# 9. ASH constructor candidate · `candidate:guide12:ash-constructor`

## 9.1 Prefer a static constructor

ASH currently carries no changing semantic metadata beyond:

- explicit closed asset `U`;
- amount;
- ownerless object role;
- admitted operation programs.

The initial candidate therefore uses a deterministic static constructor binding:

```text
target contract
+
explicit U asset
+
static compact-ASH program set
+
public value representation policy
+
internal-key policy
+
leaf version
```

No synthetic counter or dynamic metadata leaf is introduced unless the backend proves a concrete need.

## 9.2 Candidate leaves

At minimum:

```text
compact-ash coordinator, sponsorless
compact-ash coordinator, sponsored
compact-ash member
```

The constructor may also carry test-only lifecycle placeholders only if they are provably unspendable and cannot be mistaken for implemented clear support.

Prefer omission plus explicit lifecycle incompleteness over a placeholder that resembles a future operation leaf.

## 9.3 Internal-key policy

Use the accepted public unspendable-key policy from constructor research only after checking that its exact scope applies to this static constructor.

No operator, release, wallet, or generated-and-discarded private key.

## 9.4 Key path and metadata path

The candidate constructor provides no accepted key-path escape.

If any metadata or commitment-only leaf exists, it is machine-checked unspendable for every witness.

## 9.5 Constructor scope

This constructor is candidate infrastructure for Phase 4.

It is not the final ASH constructor because the clear operation and its lifecycle are absent.

The candidate bundle must carry:

```text
implemented operation scope:
    compact-ash

outstanding lifecycle:
    clear
```

---

# 10. Tapscript proof-pattern set · `sec:guide12:patterns`

Each pattern states:

- owning semantic or structural relation;
- required target capabilities;
- typed instruction fragment;
- initial and final stack contract;
- every failure path;
- source facts;
- witness requirements;
- disclosure;
- constructibility;
- resource formula;
- positive and negative vectors.

## 10.1 ASH object recognition

For each ASH input and the successor:

```text
asset = exact linked U asset

program = exact linked ASH constructor

value representation = selected public ASH mode

amount satisfies protocol domain
```

A public script match alone is insufficient without the exact asset and linked constructor binding.

## 10.2 Family cardinality

The coordinator authenticates:

```text
ASH input range length
sponsor suffix length
transaction input count
output role count
```

Counts are derived from the transaction’s target introspection.

Caller-supplied counts are witnesses at most and must equal the derived values.

## 10.3 Closed-asset family closure

The coordinator checks every transaction input and output position that could carry the closed asset `U`.

Require:

```text
every U input is an ASH source in the authenticated ASH range

the only U output is the successor ASH at output 0

no sponsor or target fee role carries U

no output outside the protocol role census carries U
```

Foreign open assets are either forbidden by the candidate ABI or proved irrelevant under a reviewed rule. They must not become a route for hidden protocol value.

## 10.4 Exact `U` conservation

For publicly readable amounts, the coordinator computes:

\[
X=\sum_{i=0}^{N-1}x_i
\]

using checked arithmetic.

Every addition:

- uses exact target width;
- checks the success flag immediately;
- re-establishes the `<2^51` domain where required;
- fails closed on overflow.

The output amount must equal \(X\).

For PublicCommitted ASH, use the exact Guide-11 approved proof instead of assuming the amount is directly encoded.

## 10.5 Canonical partition

The coordinator proves:

```text
all ASH inputs belong to one U flow
the successor belongs to the same flow
movement kind is ownerless-lateral
there is no destruction
there is no issuance
there is no second U flow
```

The transaction need not serialize the model certificate. The relation must be sufficient for an independent report to derive the same certificate.

## 10.6 Member participation

Every non-coordinator ASH input proves:

- current input is inside the authenticated ASH member range;
- spent output carries exact `U`;
- spent program is the linked ASH constructor;
- selected compact-ASH member leaf is valid for the transaction;
- no owner or operator signature is required.

The local check must not attempt to re-prove the full global sum unless deliberate duplicate enforcement is selected and recorded.

## 10.7 Permissionless path

Every compact-ASH protocol leaf contains no:

```text
owner signature
operator signature
refund signature
cadence secret
private opening unavailable to the public constructor
```

The absence is checked in typed instructions and in emitted bytes.

Sponsor signatures remain in sponsor input programs and are not compact-ASH protocol authorization.

## 10.8 Sponsor isolation

The sponsored coordinator proves:

- exact sponsor suffix;
- exact optional sponsor-change role;
- target fee-output role;
- sponsor/protocol reference disjointness;
- no sponsor member satisfies an ASH role;
- no ASH member satisfies a sponsor role;
- no second sponsor envelope;
- no sponsor amount enters protocol predicates.

Whole-transaction L-BTC conservation remains external target evidence.

## 10.9 Root absence

The ABI admits no root input or root output.

The coordinator rejects a target program or asset matching a root family at any admitted protocol position.

The compiler and linker also establish that no root carrier or relocation enters the compact-ASH bundle.

## 10.10 Projection absence

The transaction emits no data-output family that could be recognized as:

```text
burn record
clear destruction
distribution residue
```

The accepted semantic projection contains only the transition certificate.

## 10.11 Final truth and failure behavior

Every target program:

- ends in the canonical true form on success;
- leaves no non-aborting failure state that can satisfy final truth;
- checks every arithmetic and verification result;
- has identical stack shape across all successful alternatives;
- remains within target stack and element bounds.

---

# 11. Relocatable bundle boundary · `sec:guide12:relocatable`

## 11.1 Backend output

`tapscript` emits a typed candidate relocatable bundle containing:

- compiler operation-plan binding;
- target projection;
- backend configuration;
- ASH constructor template;
- coordinator and member programs;
- selected proof patterns;
- concrete relation placements;
- concrete family-layout proposal;
- witness-role declarations;
- target symbols;
- relocations;
- resource formulas;
- relation-carrier provenance;
- explicit prototype/candidate status.

Illustrative shape:

```rust
pub struct CandidateRelocatableTapscriptBundle {
    operation: architecture::OperationId,
    target: target_elements::TargetProjection,
    programs: Vec<RelocatableProgram>,
    constructor: RelocatableConstructor,
    placements: Vec<ConcreteRelationPlacement>,
    layout: CandidateLayout,
    witnesses: Vec<WitnessRole>,
    resources: Vec<ResourceFormula>,
}
```

## 11.2 No arbitrary raw program

Every program is built from typed target instructions.

Raw bytes appear only as the deterministic encoding of a validated typed program.

A parser round-trip verifies:

```text
typed instructions
→ bytes
→ typed instructions
```

for every emitted program.

## 11.3 Symbols and relocations

Symbols are typed roles such as:

```text
U asset identifier
ASH constructor
coordinator program
member program
target leaf version
internal key
candidate ASH bound
candidate sponsor-input bound
```

Relocations state:

- source symbol;
- destination role;
- semantic field;
- encoding;
- width;
- multiplicity;
- expected placeholder where byte patching is unavoidable.

Prefer structured substitution before serialization.

No variable-width byte patch.

## 11.4 Backend pattern status

Patterns remain:

```text
experimental
candidate-operation-proven
production-approved
```

as distinct states.

Guide 12 may promote only the exact compact-ASH patterns to candidate-operation-proven after full operation evidence.

It does not promote unrelated generic patterns or Guide-10 prototypes.

---

# 12. Linker foundation · `sec:guide12:linker`

## 12.1 Package creation

Guide 12 may create:

```text
packages/linker
Cargo package: tripod-linker
Rust library: linker
```

Expected direct first-party dependencies:

```text
tapscript
target-elements
```

No dependency on:

```text
model
transaction
vectors
release
artifacts
```

## 12.2 First linker scope

The first linker supports only what compact ASH requires:

- typed definition/reference census;
- exact symbol resolution;
- fixed-width or structured relocation;
- deterministic static taptree assembly;
- constructor assembly;
- relation-carrier preservation;
- resource-formula resolution;
- candidate bundle state.

Do not build a universal linker framework before this operation demonstrates the required abstractions.

## 12.3 Two-pass symbol resolution

Pass 1:

```text
collect every definition
validate unique typed keys
validate type compatibility
sort by stable key
```

Pass 2:

```text
resolve every reference
validate expected type
reject missing or ambiguous targets
build frozen reference graph
```

String display names are not symbol identity.

## 12.4 Reference cycles

Compact ASH should not require a metadata-dependent constructor cycle.

If the candidate introduces a cycle, the linker must classify and resolve it under an explicit reviewed strategy.

Finding an SCC is not accepting the cycle.

Repeated hashing until bytes stabilize is prohibited.

## 12.5 Deterministic taptree

The initial tree contains the complete candidate ASH leaf set.

Inputs include:

- leaf version;
- program bytes;
- semantic program role;
- explicit positive integer weight;
- maximum depth;
- stable tie-break key.

If no execution-frequency data exists, use equal weights.

For the small compact-ASH leaf set, compare the selected tree with exhaustive enumeration under the declared objective.

## 12.6 Carrier closure

The linker compares:

```text
compiler-required carriers
backend-emitted carriers
linked reachable carriers
```

For every relation:

- one compatible carrier exists;
- it executes in every active case;
- all referenced facts resolve;
- the selected program remains reachable;
- deliberate duplicate carriers agree.

A uniquely carrying program cannot be removed as dead code.

## 12.7 Candidate bundle output

A `CandidateLinkedBundle` contains:

- exact operation scope;
- exact target projection;
- public deployment constants;
- candidate bounds;
- linked ASH constructor;
- linked programs;
- taptree and control recipes;
- concrete relation placements;
- layout/witness handoff;
- resource formulas;
- unresolved lifecycle and evidence obligations;
- explicit candidate status.

It is not final.

## 12.8 No linked-bundle digest by default

If `transaction` consumes `CandidateLinkedBundle` directly in-process, typed comparison is sufficient.

Do not mint `LinkedBundleId` merely to follow the future identity diagram.

If exact bundle bytes are written as an independently distributed artifact, revisit ADR-016 admission then.

---

# 13. Candidate transaction ABI · `sec:guide12:abi`

## 13.1 Package creation

Guide 12 may create:

```text
packages/transaction
Cargo package: tripod-transaction
Rust library: transaction
```

Expected direct dependencies:

```text
linker
target-elements
```

No RPC, wallet, or network access in the library.

## 13.2 ABI derivation

The ABI derives from:

```text
candidate linked bundle
+
typed target
+
candidate bound assignment
+
accepted Guide-11 representation policy
```

It does not parse target bytes or planning prose.

Illustrative boundary:

```rust
pub fn derive_candidate_abi(
    bundle: &linker::CandidateLinkedBundle,
    target: &target_elements::ReviewedElementsTapscriptDefinition,
) -> Result<CandidateTransactionAbi, TransactionError>;
```

## 13.3 ABI contents

The candidate compact-ASH ABI states:

- input family order;
- output role order;
- coordinator selection;
- ASH range;
- sponsor suffix;
- target fee-output role;
- optional sponsor-change condition;
- constructor recipes;
- tapleaf selection;
- control-path recipe;
- witness item order;
- transaction version;
- sequence requirements;
- selected representation;
- candidate bounds;
- target constraints;
- relation placements.

## 13.4 Typed operation request

A compact-ASH request may choose only:

```text
ASH input outpoints
optional sponsor input capabilities
optional sponsor-change destination under sponsor policy
explicit test construction randomness where required
```

It may not choose:

```text
successor ASH amount
successor ASH asset
successor ASH constructor
coordinator
program role
family range
witness order
specialized projection
```

Those derive from the ABI.

## 13.5 Public construction view

The permissionless constructor receives:

- ASH outpoints;
- exact target assets and programs;
- public ASH amounts or approved public openings;
- linked constructor data;
- candidate ABI;
- target transaction context;
- sponsor-local funds where selected.

It receives no owner or operator secret.

## 13.6 Sponsor capability

The sponsor interface supplies:

- sponsor outpoints;
- owner public identities;
- signing capability;
- private wallet amount/opening data as needed to construct balance;
- optional change destination.

Canonical reports retain only public sponsor role data and erase private values and openings.

## 13.7 Construction stages

The candidate transaction pipeline is:

1. validate request and public input view;
2. reject duplicate outpoints;
3. sort ASH inputs canonically;
4. derive coordinator;
5. validate ASH object and representation facts;
6. compute exact successor amount;
7. instantiate successor constructor;
8. derive sponsor range and optional change;
9. assemble target transaction roles;
10. finalize every protected output;
11. issue sponsor signing requests where needed;
12. collect and validate signatures;
13. assemble script witnesses and control paths;
14. perform ABI-local preflight;
15. return exact target bytes and a typed construction report.

No signing request is issued before protected outputs are final.

## 13.8 Synthetic test ASH

Because burn is not implemented, Guide 12 needs a test-only origin for ASH inputs.

The vector harness may create synthetic development ASH UTXOs under:

- the candidate ASH constructor;
- an explicitly issued disposable test asset representing `U`;
- deterministic public test values;
- a test-only funding ceremony.

These inputs are target fixture material, not protocol history.

Reports label them:

```text
synthetic target fixture
not produced by burn
not evidence of burn lineage
```

No burn event or attestation claim derives from them.

## 13.9 ABI status and identity

The output is `CandidateTransactionAbi`.

No public ABI digest is minted while all consumers receive the typed value directly and reports embed the complete subject.

A future external constructor or release package may activate an ABI identity under ADR-016.

---

# 14. Operation evidence package · `sec:guide12:evidence-package`

## 14.1 Ownership decision required

The operation evidence layer needs:

- architecture and realization semantics;
- executable model expectations;
- compiler relation and coverage scope;
- linked bundle;
- transaction ABI;
- target execution.

This is broader than `target-elements-conformance`, whose current purpose is target-generic primitive and prototype evidence.

The preferred ownership is the planned `vectors` package.

## 14.2 Package creation

Guide 12 may create:

```text
packages/vectors
Cargo package: tripod-vectors
Rust library: vectors
```

Expected direct dependencies for the compact-ASH tranche:

```text
architecture
realization
model
compiler
target-elements
linker
transaction
```

The exact use of `target-elements-conformance` must be decided explicitly.

## 14.3 Executor reuse candidates

### Candidate A — depend on `target-elements-conformance`

`vectors` consumes a target-generic public executor/supervision boundary from the conformance package.

Strength:

- one process supervision implementation;
- one strict protocol;
- one provenance model;
- one environment-binding mechanism.

Cost:

- updates the current package contract saying nothing depends on conformance;
- operation transactions require a more general request than primitive fixtures.

### Candidate B — extract a target-executor package

Create a shared package only after both native conformance and operation vectors demonstrate one stable common abstraction.

Strength:

- clean ownership;
- no operation semantics in primitive-conformance package.

Cost:

- another package and migration before the first operation.

### Candidate C — vectors owns a second executor

Not preferred.

It duplicates:

- process supervision;
- protocol framing;
- environment binding;
- provenance;
- timeout cleanup.

**Initial preference:** Candidate A for the first operation, with the conformance package exposing only target-generic execution and transcript mechanics. If the required interface cannot remain target-generic, take Candidate B. Do not duplicate the supervisor under Candidate C merely to avoid a package-contract update.

## 14.4 Canonical operation evidence plan

As in Guide 11, arbitrary vector fixtures cannot become gate-eligible evidence merely by carrying claim labels.

Introduce a validated operation plan:

```rust
pub struct CompactAshEvidencePlan {
    semantic_cases: Vec<CompactAshSemanticCase>,
    target_cases: Vec<CompactAshTargetCase>,
    relation_coverage: Vec<RelationCoverageRow>,
}
```

No public unchecked constructor.

The plan derives from:

- compiler coverage requirements;
- canonical mutation registry;
- exact linked bundle;
- exact candidate ABI.

An ad hoc vector may produce an experimental report only.

## 14.5 Report roles

Keep separate:

```text
semantic fixture result

construction result

target execution result

accepted semantic projection result

relation coverage result

resource result

external target dependency result
```

A broad “all compact-ASH tests passed” report is not sufficient.

## 14.6 No report digest

Guide 12 reports are compared by typed content and exact bytes.

No report digest is admitted unless a real persistent external consumer appears.

---

# 15. Semantic fixtures and expected results · `sec:guide12:fixtures`

## 15.1 Positive fixtures originate from valid model state

Where the executable model supplies a compact-ASH transition:

1. build a valid world through genesis and successful operations or an explicitly approved test fixture path;
2. identify a set of ASH objects;
3. execute compact ASH through the model’s invariant-wrapped path;
4. bind the execution;
5. project the realization observation;
6. evaluate every active realization relation;
7. record the expected semantic successor and certificate.

If synthetic model setup is used, it must be clearly typed as fixture construction and must still satisfy the global invariant before the operation.

## 15.2 Target materialization is separate

The semantic fixture contains no target positions, programs, control blocks, or bytes.

A target vector materializes it through:

```text
candidate linked bundle
+
candidate ABI
+
typed operation request
+
public target input view
```

## 15.3 Expected result is independent of target emission

Expected semantic values derive from the model and realization.

Expected target bytes derive from the linked bundle and ABI where bytes are the subject.

The candidate backend never generates the semantic expected result used to judge itself.

## 15.4 Accepted projection

For an accepted target transaction compare:

- input ASH family;
- output ASH family;
- exact explicit `U` identity;
- exact ASH aggregate amount;
- ownerless status;
- no roots;
- no issuance;
- no destruction;
- ownerless-lateral canonical flow;
- sponsor role structure;
- no specialized event;
- required transition-certificate derivation;
- public protocol observable where applicable.

---

# 16. Complete vector matrix · `sec:guide12:vectors`

## 16.1 Positive semantic cases

- minimum two ASH inputs;
- three ASH inputs;
- representative larger batch;
- values summing exactly to a limb or encoding boundary;
- values summing to \(2^{51}-1\);
- canonical input permutation normalized by ABI;
- sponsorless consensus transaction;
- sponsored transaction with one sponsor input;
- sponsored transaction with several sponsor inputs;
- sponsor change present;
- sponsor change absent;
- accepted public ASH representation selected by Guide 11.

## 16.2 Cardinality mutations

- zero ASH inputs;
- one ASH input;
- one above candidate ASH bound;
- zero ASH outputs;
- two ASH outputs;
- sponsor input count above candidate bound;
- two sponsor-change outputs;
- two target fee outputs;
- unexpected target-only output.

## 16.3 Input faults

- duplicate ASH outpoint;
- same outpoint claimed as ASH and sponsor;
- wrong asset at one ASH input;
- right asset under wrong constructor;
- right constructor with wrong representation;
- foreign open asset carrying ASH-shaped metadata;
- unrecognized closed-asset input;
- sponsor input inside ASH range;
- ASH input inside sponsor suffix;
- input order violating canonical order;
- coordinator not first ASH;
- two coordinator leaves;
- no coordinator leaf;
- member leaf at coordinator index;
- coordinator leaf at member index.

## 16.4 Output faults

- successor value one below sum;
- successor value one above sum;
- successor value zero;
- successor value outside amount domain;
- wrong output asset;
- confidential or unclassified closed asset;
- wrong ASH constructor;
- ordinary wallet `U` output;
- second `U` output;
- sponsor change carrying `U`;
- fee output carrying `U`;
- successor at wrong output role;
- successor and sponsor-change roles exchanged;
- malformed target fee output;
- unclaimed output.

## 16.5 Canonical partition faults

- omit one ASH source from the flow;
- cite one source twice;
- cite one destination twice;
- claim successor through two flows;
- add destruction leg;
- add issuance;
- change movement kind to lateral;
- change movement kind to destruction;
- leave one canonical object unwitnessed;
- route part of `U` into an undeclared object;
- use an aggregate that balances while one input is stolen from another role.

## 16.6 Authorization and constructibility faults

- hidden protocol owner signature;
- hidden operator signature;
- sponsor input without owner authorization;
- one sponsor owner omitted;
- output set changed after sponsor signing;
- public ASH opening missing;
- public ASH opening copied from another object;
- constructor needs creator-private data;
- permissionless constructor accesses owner fixture state;
- protocol path depends on sponsor amount.

## 16.7 Sponsor faults

- sponsor/protocol reference overlap;
- two sponsor envelopes;
- sponsor change outside canonical role;
- ordinary L-BTC output substituted for target fee output;
- target fee output substituted for sponsor change;
- sponsor asset is foreign;
- sponsor member left unclaimed;
- sponsor amount changed while protocol projection is fixed;
- sponsor amount set to zero with exact role structure;
- formula-valid protocol output plus sponsor-local denomination change;
- balanced theft attempt where successor ASH is short and sponsor change grows;
- confidential sponsor values under accepted target policy;
- report attempts to publish sponsor amount.

The zero-valued ordinary sponsor member is judged according to the standing semantic policy:

```text
role-exact semantic relation:
    may accept

first-party explicit builder:
    omits known zero change

deployment policy:
    may reject as policy
```

Those are separate results.

## 16.8 Root and projection faults

- STATE input added;
- RESV input added;
- PACE input added;
- authority input added;
- root-shaped output added;
- burn record added;
- `tag-burn` payload added;
- clear destruction output added;
- residue output added;
- burn projection claimed;
- clear projection claimed;
- transition certificate omitted from semantic report;
- false certificate edge introduced.

## 16.9 Representation faults

- private ASH under an explicit-only policy;
- PublicCommitted ASH without opening;
- opening bound to wrong commitment;
- wrong amount in opening;
- wrong asset generator;
- copied capsule;
- malformed capsule;
- public evidence available only in creator process;
- representation changed without compiler-plan update;
- output representation differs from ABI;
- confidential closed asset.

## 16.10 Constructor and linking faults

- missing coordinator program;
- missing member program;
- extra escape leaf;
- spendable key path;
- wrong internal key;
- wrong leaf version;
- wrong tree child ordering;
- stale constructor from another bundle;
- unresolved relocation;
- relocation applied twice;
- wrong U asset substituted at link time;
- candidate bound relocation inconsistent with ABI;
- unique relation carrier made unreachable;
- source-order-dependent taptree.

## 16.11 ABI and transaction faults

- family range overlap;
- family gap;
- wrong total input count;
- wrong total output count;
- wrong transaction version;
- wrong sequence;
- witness items reordered;
- control path from another program;
- request selects target program directly;
- request supplies successor amount;
- duplicate signing request;
- unexpected signature;
- target bytes changed after ABI validation;
- candidate bundle paired with another ABI;
- ad hoc transaction bypasses safe constructor.

## 16.12 Resource and infrastructure cases

- script at candidate maximum;
- witness at candidate maximum;
- maximum sponsor candidate;
- deepest selected control path;
- target consensus acceptance but policy rejection;
- policy acceptance where claimed;
- executor timeout;
- executor malformed response;
- executor wrong environment;
- executor wrong provenance;
- target infrastructure failure;
- resource prediction mismatch.

Infrastructure failures never count as expected negative target evidence.

---

# 17. Relation-indexed coverage · `sec:guide12:coverage`

For every compiler relation-case, require typed rows for:

```text
positive target case
negative target case
carrier reached
expected target verdict
observed target verdict
accepted semantic projection where accepted
mutation layer
collateral relations
```

## 17.1 Positive coverage

Positive coverage requires:

- relation active;
- selected proof present;
- carrier reachable;
- target transaction constructed;
- target accepted;
- accepted semantic projection matched.

## 17.2 Negative coverage

Negative coverage requires:

- intended relation identified;
- valid source transaction identified;
- focused mutation recorded;
- concrete mutated transaction materialized;
- carrier executed where target rejection is the claim;
- target rejected at the expected layer;
- collateral relations listed honestly.

## 17.3 Static relations

Relations discharged completely before target execution still receive evidence appropriate to their boundary.

Examples:

```text
no root declarations in bundle:
    compiler/linker structural evidence

no specialized projections:
    ABI/output-role structural evidence
    plus target mutation vectors where materializable

lifecycle clear remains outstanding:
    explicit incomplete status, not a pass
```

## 17.4 External evidence

Whole-transaction value conservation remains an explicit external evidence requirement.

The accepted target result must name that requirement and its exact development evidence.

A runtime target acceptance may satisfy it only if the target transaction actually reaches the relevant consensus relation. A mock or abstract stack result does not.

---

# 18. Resource analysis and candidate bounds · `sec:guide12:resources`

## 18.1 Candidate bounds are not calibration

Architecture declares:

```text
ASH_BATCH_MAX
FEE_SPONSOR_INPUT_MAX
```

as deployment-calibrated bounds.

Guide 12 may test candidate values.

It does not select final deployment values.

Every candidate is carried in a typed:

```text
CandidateBoundAssignment
```

and remains visibly non-final.

## 18.2 Candidate enumeration

Do not assume monotonicity if changing a bound changes:

- unrolled script size;
- layout;
- tree depth;
- proof pattern;
- transaction shape;
- policy result.

For the first implementation, enumerate a finite candidate list deterministically.

Suggested ASH candidates:

```text
2
4
8
16
32
64
```

Suggested sponsor-input candidates:

```text
0
1
2
4
8
16
```

These are research candidates, not accepted values.

## 18.3 Measure complete transactions

For each candidate measure:

- coordinator program bytes;
- member program bytes;
- ASH constructor bytes;
- taptree depth;
- control bytes;
- initial witness items;
- witness bytes;
- peak stack;
- maximum element;
- arithmetic and comparison operation count;
- crypto validation budget;
- transaction weight;
- policy result;
- package behavior where applicable.

## 18.4 Separate objectives

One fixture may maximize:

- transaction weight;
- witness bytes;
- stack;
- element width;
- validation budget;
- tree depth;
- policy pressure.

Do not claim one transaction maximizes all objectives unless proved.

## 18.5 Prediction and observation

Backend, linker, and transaction predictions are compared with real target observations.

A mismatch is a failed resource report even if the transaction itself is accepted.

## 18.6 Phase-4 bound result

Phase 4 may record:

```text
candidate bound N fits the tested bundle and ABI
```

It must not record:

```text
ASH_BATCH_MAX = N for production
```

Final calibration requires Phase-12 relinking and remeasurement of the exact final bundle and ABI.

---

# 19. Assurance boundaries · `sec:guide12:assurance`

## 19.1 Compiler

Establishes:

- complete operation analysis;
- proof/source/constructibility/lifecycle/placement/layout/coverage requirements;
- deterministic target operation plan.

Does not establish target behavior.

## 19.2 Target contract

Establishes:

- reviewed typed target facts;
- reviewed development binding.

Does not execute anything.

## 19.3 Tapscript

Establishes:

- typed deterministic emission;
- stack-valid patterns;
- complete relation placement;
- resource formulas.

Does not establish linking, ABI, or target acceptance.

## 19.4 Linker

Establishes:

- exact symbol and relocation resolution;
- deterministic constructor and tree;
- carrier reachability;
- linked resource formulas.

Does not construct complete transactions.

## 19.5 Transaction

Establishes:

- ABI-consistent construction;
- canonical layout;
- target transaction and witness materialization;
- sponsor signing requests.

Does not establish target acceptance.

## 19.6 Vectors

Establishes:

- finite bundle-specific translation evidence;
- exact expected/observed comparison;
- relation coverage;
- resource measurements.

Does not prove universal compiler correctness or production readiness.

## 19.7 Native executor

Reports:

- what one selected executable says one target environment did.

Does not establish:

- executor authenticity;
- independence;
- production activation;
- universal target correctness.

## 19.8 Phase-4 result

Establishes one candidate compact-ASH pipeline.

It does not establish:

- burn lineage;
- clear lifecycle;
- final ASH constructor;
- live transfer;
- STATE;
- redemption;
- settlement;
- cycle;
- final calibration;
- production target support;
- deployment release.

---

# 20. Suggested implementation waves · `sec:guide12:waves`

## Wave 0 — Revalidate Phase-3 handoff

Deliver:

- Guide-11 result imported into typed representation policy;
- canonical evidence-boundary regression suite green;
- exact compact-ASH compiler pilot projection recorded;
- current package and backlog status reconciled;
- no identity drift.

Suggested commit:

```text
plans: charter the compact-ash end-to-end tranche
```

## Wave 1 — Public compiler target-operation plan

Deliver:

- immutable validated compact-ASH target plan;
- relation, carrier, layout, and coverage projections;
- no graph handles;
- complete owner revalidation;
- focused corruption tests;
- public API test.

Suggested commit:

```text
compiler: expose the validated compact-ash target plan
```

## Wave 2 — Target capability closure

Deliver:

- exact static assessment for the operation plan;
- unsupported capability failures;
- public ASH representation decision carried;
- sponsor signature and target-conservation evidence retained;
- no complete-pattern overclaim.

Suggested commit:

```text
tapscript: assess the compact-ash target requirements
```

## Wave 3 — Proof patterns and stack schedules

Deliver:

- object recognition;
- family cardinality;
- canonical partition;
- exact aggregate arithmetic;
- member participation;
- sponsor isolation;
- root/projection absence;
- abstract stack validation;
- independent bounded stack oracle where needed.

Suggested commit:

```text
tapscript: implement compact-ash proof patterns
```

## Wave 4 — Relocatable bundle

Deliver:

- typed program roles;
- static ASH constructor candidate;
- symbols and relocations;
- concrete placements;
- preliminary layout;
- witness roles;
- resource formulas;
- candidate status.

Suggested commit:

```text
tapscript: emit a relocatable compact-ash bundle
```

## Wave 5 — Linker foundation

Deliver:

- linker crate;
- typed symbol census;
- two-pass resolution;
- structured relocation;
- deterministic small taptree;
- relation-carrier closure;
- candidate linked bundle;
- exhaustive small-tree oracle.

Suggested commit:

```text
linker: link the compact-ash candidate bundle
```

## Wave 6 — Candidate transaction ABI

Deliver:

- transaction crate;
- candidate ABI;
- sponsorless and sponsored layouts;
- coordinator rule;
- operation request;
- public input view;
- synthetic test-ASH origin;
- sponsor signing interface;
- exact transaction bytes.

Suggested commit:

```text
transaction: derive the compact-ash candidate ABI
```

## Wave 7 — Semantic and target fixtures

Deliver:

- canonical semantic fixtures;
- model/realization expectations;
- target materialization;
- accepted projection;
- canonical operation evidence plan;
- experimental versus evidence fixture separation.

Suggested commit:

```text
vectors: add compact-ash semantic and target fixtures
```

## Wave 8 — Real target execution

Deliver:

- sponsorless consensus cases;
- sponsored policy-valid cases;
- exact environment and provenance binding;
- target verdict reports;
- accepted semantic projection reports;
- zero infrastructure errors for required cases.

Suggested commit:

```text
vectors: execute compact-ash against the reviewed target
```

## Wave 9 — Negative relation coverage

Deliver:

- every focused mutation family;
- construction-versus-target classification;
- collateral relation declarations;
- exact relation coverage census;
- no uncovered active relation-case.

Suggested commit:

```text
vectors: complete compact-ash negative coverage
```

## Wave 10 — Candidate resource study

Deliver:

- finite candidate bound enumeration;
- complete transaction measurements;
- prediction/observation comparison;
- selected Phase-4 demonstration candidate;
- explicit non-calibration statement.

Suggested commit:

```text
vectors: measure compact-ash candidate bounds
```

## Wave 11 — Phase-4 gate and documentation

Deliver:

- package READMEs;
- package contracts;
- Phase-4 card;
- backlog gate record;
- identity/dependency impact;
- complete repository gate;
- clean final tree.

Suggested commit:

```text
plans: record the compact-ash candidate pipeline
```

Commit every coherent green wave before beginning the next.

---

# 21. Focused verification · `sec:guide12:verification`

## 21.1 Compiler

```sh
cargo test --locked -p tripod-compiler
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-compiler --no-deps
```

Focused areas:

```text
target-operation plan construction
relation census
carrier census
layout census
coverage census
owner revalidation
projection determinism
no graph-handle leakage
no identity field
```

## 21.2 Target contract

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

Focused areas:

```text
required primitive contracts
value/asset introspection
signature/sighash behavior
resource contracts
capability closure
evidence requirements
reviewed trust state
```

## 21.3 Tapscript

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

Focused areas:

```text
recognition pattern
family cardinality
canonical partition
checked aggregate arithmetic
member/coordinator schedules
sponsor opacity
root/projection absence
typed emission
parser round trip
resource formulas
candidate-status separation
```

## 21.4 Linker

```sh
cargo test --locked -p tripod-linker
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-linker --no-deps
```

Focused areas:

```text
symbol census
reference resolution
relocation
taptree determinism
constructor assembly
carrier closure
resource resolution
candidate/final type separation
```

## 21.5 Transaction

```sh
cargo test --locked -p tripod-transaction
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-transaction --no-deps
```

Focused areas:

```text
candidate ABI derivation
canonical input ordering
coordinator derivation
sponsor suffix
optional change
fee-output role
successor amount
constructor instantiation
witness order
signature commitment
public-data permissionless construction
```

## 21.6 Vectors and target execution

```sh
cargo test --locked -p tripod-vectors
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-vectors --no-deps
```

Focused areas:

```text
semantic fixtures
canonical evidence plan
target materialization
negative mutations
accepted projection
relation coverage
resource prediction
report determinism
ad hoc/evidence separation
```

If executor mechanics remain in target conformance:

```sh
cargo test --locked -p tripod-target-elements-conformance
```

## 21.7 Existing model and realization

```sh
cargo test --locked -p tripod-model
cargo test --locked -p tripod-realization
```

No backend change may alter model acceptance or realization semantics.

## 21.8 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new file enters its nearest Meson census in the same commit.

---

# 22. Full batch gate · `gate:guide12:batch`

After every coherent implementation wave:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run complete real target matrices separately:

```text
compact-ash sponsorless matrix
compact-ash sponsored matrix
compact-ash relation-mutation matrix
compact-ash constructor/linker mutation matrix
compact-ash ABI mutation matrix
compact-ash resource candidate matrix
```

Required real-run record:

```text
target contract revision
development binding
network and genesis
executor adapter and version
framework revision
binary-reported node revision
intended tip
upstream base
local topic census
bundle candidate
ABI candidate
bound candidates
case count
relation count
claim count
failures
infrastructure errors
report byte reproducibility
```

Run dependency checks if the graph changed:

```sh
cargo tree --locked -e features
cargo metadata --locked
cargo audit
```

A missing advisory tool is recorded as skipped.

Run document reproducibility if document inputs changed or repository batch policy requires it:

```sh
scripts/check-document-reproducibility.sh
```

A deferred run is recorded as deferred.

Final check:

```sh
git status --porcelain=v1 --untracked-files=all
```

The result must be empty.

---

# 23. Acceptance criteria · `sec:guide12:acceptance`

Accept the Phase-4 compact-ASH candidate only when:

- compiler target-operation scope is exactly compact ASH;
- relation census is complete;
- every target requirement is assessed;
- every selected proof is realization-approved;
- every relation has a reachable carrier;
- every active execution case reaches the required carriers;
- public ASH representation follows Guide 11;
- no owner/operator secret appears on the protocol path;
- exact `U` identity and constructor recognition hold;
- exact family counts and ranges hold;
- exactly one ASH output exists;
- exact aggregate `U` conservation holds;
- no issuance or destruction exists;
- sponsor region is exact and amount-opaque;
- sponsor owners authorize the finalized output set;
- no root participates;
- no specialized event is emitted;
- linked constructor and programs are deterministic;
- all mandatory symbols and relocations resolve;
- the candidate ABI is deterministic;
- complete valid target transactions accept;
- every accepted semantic projection matches;
- every required focused mutation fails at the owning layer;
- every relation-case has complete coverage;
- predicted and observed resources agree;
- at least one useful candidate bound is demonstrated;
- the bound remains explicitly non-final;
- the candidate bundle remains explicitly lifecycle-incomplete;
- no speculative digest is minted;
- reports reproduce;
- full repository gates pass;
- the final tree is clean.

---

# 24. Rejection criteria · `sec:guide12:rejection`

Reject the candidate if:

- one semantic relation disappears between packages;
- backend emission consumes compiler internals or graph handles;
- target-specific positions enter compiler core;
- a target capability is treated as a completed proof pattern;
- a required relation has no carrier;
- a carrier is reachable only in the wrong execution case;
- public ASH needs an owner-private opening;
- wrong asset or constructor passes;
- aggregate conservation can be balanced while routing `U` elsewhere;
- sponsor amount is read as protocol data;
- sponsor positivity substitutes for protocol output correctness;
- a hidden protocol signature appears;
- root or specialized event participation is possible;
- transaction builder is the only place enforcing a target-required relation;
- an ad hoc fixture can become evidence;
- construction rejection is counted as target rejection for a script claim;
- accepted target projection differs from the model;
- a real target report is missing environment or provenance;
- a mock satisfies the gate;
- resource prediction and observation disagree;
- the selected candidate exceeds hard target limits;
- candidate and final states are conflated;
- prototype bytes differ from the bytes actually tested;
- a report digest is introduced without an admitted consumer;
- any full gate fails or leaves the repository dirty.

---

# 25. Identity, schema, and dependency impact · `sec:guide12:impact`

Expected identity impact:

```text
Attestation version:
    unchanged

realization version:
    unchanged

architecture schema:
    unchanged

architecture semantic hash:
    unchanged

architecture behavioural hash:
    unchanged

anchor-set hash:
    unchanged

realization identity:
    none minted

compiler-plan identity:
    none minted unless a real external consumer appears

target-definition digest:
    none minted

relocatable-bundle digest:
    none minted

linked-bundle digest:
    none minted for in-process candidate use

transaction-ABI digest:
    none minted for in-process candidate use

vector-set/report digest:
    none minted

deployment-profile identity:
    remains dormant
```

Expected schema work:

```text
compiler public target-operation projection:
    new typed public boundary, not necessarily a serialized schema

relocatable bundle:
    candidate typed schema

linked bundle:
    candidate typed schema

transaction ABI:
    candidate typed schema

operation evidence:
    candidate typed report schemas
```

Expected dependency changes:

```text
new first-party linker package
new first-party transaction package
new first-party vectors package

possible vectors → target-elements-conformance edge,
subject to the executor-ownership decision

no new third-party dependency unless a concrete operation consumer requires one
```

Every dependency change receives ADR-011 review and a lockfile diff.

---

# 26. Final Phase-4 result matrix · `tab:guide12:result`

The completion record fills this table.

| Boundary | Required result | Actual result |
|---|---|---|
| realization | compact-ASH semantic relation complete | |
| compiler | validated target operation plan | |
| target assessment | every capability/evidence role classified | |
| tapscript | typed programs and proof patterns | |
| linker | deterministic candidate bundle | |
| transaction | deterministic candidate ABI and complete transactions | |
| target execution | positive cases accepted, negative cases rejected | |
| semantic projection | every accepted transaction matches | |
| coverage | every relation-case complete | |
| resources | prediction equals observation | |
| candidate bounds | useful candidate demonstrated, not calibrated | |
| lifecycle | clear explicitly outstanding | |
| identity | no speculative digest | |
| release | not claimed | |

---

# 27. Guide-12 exit checklist · `gate:guide12:exit`

## Compiler boundary

- [ ] target-operation plan has no unchecked constructor;
- [ ] complete analyzed validation runs on construction;
- [ ] compact-ASH scope is exact;
- [ ] relation census is exact;
- [ ] carrier census is exact;
- [ ] layout census is exact;
- [ ] coverage census is exact;
- [ ] no graph index is public;
- [ ] no target opcode or position enters compiler core;
- [ ] no compiler digest is minted without a consumer.

## Target and patterns

- [ ] reviewed target contract passes;
- [ ] every required capability is assessed;
- [ ] every selected pattern names its target prerequisites;
- [ ] every arithmetic success flag is checked;
- [ ] permissionless paths contain no protocol signature;
- [ ] sponsor signatures commit protected outputs;
- [ ] ASH representation follows Guide 11;
- [ ] confidential closed asset escapes reject;
- [ ] stack success/failure shapes validate;
- [ ] parser round-trip holds for every emitted program.

## Relocatable and linked bundle

- [ ] all symbols are typed;
- [ ] definitions are unique;
- [ ] references resolve in two passes;
- [ ] mandatory relocations resolve exactly once;
- [ ] no overlapping or variable-width byte relocation exists;
- [ ] constructor is deterministic;
- [ ] internal-key policy is explicit;
- [ ] taptree is deterministic;
- [ ] small-tree oracle agrees;
- [ ] every relation carrier remains reachable;
- [ ] bundle is explicitly candidate-only;
- [ ] clear lifecycle remains explicit.

## ABI and transactions

- [ ] input family order is canonical;
- [ ] coordinator derives from canonical ASH order;
- [ ] sponsor range is exact;
- [ ] output roles are exact;
- [ ] fee output cannot satisfy sponsor change;
- [ ] sponsor change cannot satisfy fee output;
- [ ] successor amount derives exactly;
- [ ] request cannot choose protected outputs or programs;
- [ ] signatures are requested after output finalization;
- [ ] witness and control ordering is canonical;
- [ ] synthetic ASH fixtures are labeled test-only;
- [ ] candidate ABI is not final.

## Evidence

- [ ] arbitrary fixtures cannot become gate-eligible;
- [ ] canonical evidence plan derives from compiler coverage;
- [ ] expected semantics derive independently of target emission;
- [ ] every positive transaction is complete and valid;
- [ ] every accepted projection matches;
- [ ] every required negative mutation reaches the owning evidence layer;
- [ ] construction and target rejection remain distinct;
- [ ] whole-transaction conservation remains explicit evidence;
- [ ] observed target environment matches binding;
- [ ] executable provenance satisfies ADR-018;
- [ ] no mock satisfies the gate;
- [ ] no required case has infrastructure error;
- [ ] report bytes reproduce;
- [ ] no report digest is minted.

## Resources

- [ ] coordinator program measured;
- [ ] member program measured;
- [ ] constructor measured;
- [ ] complete sponsorless transaction measured;
- [ ] complete sponsored transaction measured;
- [ ] candidate bound matrix measured;
- [ ] prediction equals observation;
- [ ] consensus and policy verdicts remain separate;
- [ ] selected bound is explicitly candidate-only;
- [ ] no Phase-12 calibration claim is made.

## Repository

- [ ] package READMEs are current;
- [ ] package contracts are current;
- [ ] Phase-4 card is current;
- [ ] backlog current state is current;
- [ ] every new file is in the Meson census;
- [ ] dependency review is recorded;
- [ ] identity impact is recorded;
- [ ] `cargo fmt --all` passes;
- [ ] Clippy passes with warnings denied;
- [ ] workspace tests pass;
- [ ] `scripts/ci.sh` passes;
- [ ] canonical Meson compile passes;
- [ ] canonical Meson tests pass;
- [ ] advisory result is passed or explicitly skipped;
- [ ] document reproducibility is passed or explicitly deferred under policy;
- [ ] `git diff --check` passes;
- [ ] final repository status is empty.

---

# 28. Completion report template · `sec:guide12:report-template`

```text
Guide 12 result
===============

Starting state:
    source revision:
    Guide-11 result:
    compiler compact-ASH projection:
    target contract:
    development binding:
    clean tree:

Compiler boundary:
    public target-operation type:
    scope:
    relations:
    execution cases:
    abstract carriers:
    layout requirements:
    coverage requirements:
    capabilities:
    external evidence:
    graph handles exposed:
    identity minted:

Target assessment:
    reviewed target revision:
    missing primitives:
    unsupported capabilities:
    backend patterns required:
    structural obligations:
    external evidence:
    selected representation:

Tapscript patterns:
    object recognition:
    family cardinality:
    canonical partition:
    aggregate arithmetic:
    member participation:
    coordinator:
    sponsor isolation:
    root absence:
    projection absence:
    program roles:
    stack result:
    candidate status:

Relocatable bundle:
    programs:
    constructor:
    symbols:
    relocations:
    placements:
    witness roles:
    resource formulas:
    unresolved obligations:

Linker:
    package revision:
    symbol census:
    reference graph:
    SCCs:
    relocation result:
    taptree policy:
    taptree depth:
    carrier census:
    candidate bundle:
    deterministic rebuild:

Transaction ABI:
    input layout:
    output layout:
    coordinator rule:
    sponsor range:
    sponsor change:
    fee output:
    transaction version:
    sequence:
    witness order:
    control-path roles:
    request fields:
    candidate bounds:
    candidate status:

Synthetic ASH fixture:
    test asset:
    constructor:
    funding method:
    values:
    explicit test-only statement:
    burn/event non-claim:

Semantic fixtures:
    model source:
    realization report:
    expected successor:
    expected certificate:
    fixture count:

Target vectors:
    positive cases:
    negative cases:
    construction failures:
    target rejections:
    infrastructure errors:
    accepted projection mismatches:
    relation coverage:
    unresolved coverage:

Target execution:
    executor:
    adapter version:
    framework revision:
    node version:
    binary-reported revision:
    intended tip:
    upstream base:
    local topics:
    network:
    genesis:
    consensus cases:
    policy cases:
    deterministic report bytes:

Resources:
    coordinator bytes:
    member bytes:
    constructor bytes:
    tree depth:
    control bytes:
    witness bytes:
    peak stack:
    largest item:
    arithmetic operations:
    validation budget:
    sponsorless weight:
    sponsored weight:
    policy result:
    candidate ASH bounds:
    candidate sponsor bounds:
    selected demonstration candidate:
    calibration claim:
        none

Lifecycle:
    compact:
        implemented candidate
    clear:
        outstanding
    release-complete:
        no

Identity impact:
    Attestation:
    realization:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    anchor-set hash:
    compiler identity:
        none
    bundle identity:
        none unless separately admitted
    ABI identity:
        none unless separately admitted
    vector/report identity:
        none
    deployment profile:
        dormant

Dependency impact:
    new first-party packages:
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
    linker:
    transaction:
    vectors:
    target-elements-conformance:
    Rustdoc:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    real target vectors:
    cargo audit:
    document reproducibility:
    git diff --check:
    final git status:

Phase-4 verdict:
    accepted candidate / rejected target path / blocked

Residuals:

Next phase:
```

---

# 29. What follows Guide 12 · `sec:guide12:next`

If Guide 12 succeeds, Phase 4 has one complete candidate compiler-to-target operation.

The next implementation guide should address live receipt transfer:

```text
Guide 13 — End-to-End Live Receipt Transfer
```

Guide 13 consumes rather than reopens:

- validated compiler target-operation boundary;
- static target assessment;
- typed tapscript pattern interface;
- relocatable bundle interface;
- linker symbol and relocation model;
- deterministic taptree policy;
- candidate transaction ABI framework;
- operation evidence framework;
- executor provenance and environment binding;
- canonical report trust states.

Guide 13 adds:

- protocol owner authorization;
- multiple protocol inputs and outputs;
- split and merge;
- output-committing signatures;
- explicit and private committed value alternatives;
- separate safety and minimality evidence;
- future burn and redemption lifecycle obligations.

Guide 12 itself adds none of those owner/value-representation claims beyond what compact ASH requires.

---

## Closing statement · `rem:guide12:closing`

> Compact ASH is the first complete pipeline test because its semantic relation is small and its implementation boundary is not. The guide succeeds only when exact typed semantics, compiler analysis, target requirements, emitted programs, linked carriers, transaction layout, real target execution, accepted semantic projection, and relation-indexed evidence all describe the same operation. A green script, a green node, or a green report alone is not that result; the welded chain is.
