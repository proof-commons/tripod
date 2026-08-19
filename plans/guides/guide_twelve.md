# Guide 12 — End-to-End Compact ASH

> **Status:** Execution guide; not yet executed
> **Phase:** Phase 4 — End-to-End Compact ASH
> **Entry:** Phase-3 exit gate passed and recorded; Guide-11 declassification policy accepted; Guide-12 preflight register below closed
> **Primary semantic operation:** `compact-ash`
> **Affected packages:** `compiler`, `target-elements`, `tapscript`, new `linker`, new `transaction`, new `vectors`, and `target-elements-conformance` where executor mechanics remain shared
> **May affect after acceptance:** Phase-4 card, package contracts, backlog, Meson graph, dependency graph
> **Supersedes as execution direction:** `guide_twelve_concept.md`; the concept remains an archived design input
> **Does not implement:** burn, clear, STATE, RESV, issuance, settlement, redemption, cycle, production signing, production wallet custody, final calibration, production activation, deployment release, or release authentication
> **Required result:** one complete candidate compiler-to-target implementation of compact ASH, including a validated compiler operation plan, typed target programs, deterministic candidate linking, a candidate transaction ABI, complete real target transactions, relation-indexed positive and negative evidence, accepted semantic-projection comparison, resource measurements, and an explicit non-production status
> **Review basis:** static review of the supplied selected files at tree `0.3.6-dev`; every review item below remains a hypothesis until reproduced against the working tree
> **Trust boundary:** repository code and selected executors are executable; untrusted contributions run only in an externally established, credential-free environment under ADR-015

---

## Mission · `sec:guide12-exec:mission`

Guide 12 proves the complete implementation pipeline on the smallest root-free, ownerless, permissionless attestation-contract operation:

```text
typed architecture
    ↓
target-independent realization
    ↓
validated compiler analysis
    ↓
validated target-operation plan
    ↓
reviewed target assessment
    ↓
typed tapscript proof patterns
    ↓
candidate relocatable bundle
    ↓
deterministic linker
    ↓
candidate linked bundle
    ↓
candidate transaction/witness ABI
    ↓
complete Elements transactions
    ↓
real target execution
    ↓
accepted semantic projection
    ↓
relation-indexed evidence
```

The semantic operation is compact ASH:

```text
two or more ASH inputs
    ↓
exactly one ASH output

sum(input ASH values)
    =
output ASH value
```

Compact ASH is selected first because it avoids:

- STATE and RESV succession;
- constructor continuity over mutable state;
- wide floor arithmetic;
- owner authorization over protocol value;
- issuance and destruction;
- burn, clear, and residue events;
- maturity and cadence;
- formula-bound payouts.

It does not avoid the hard implementation seams:

- exact relation preservation;
- target proof selection;
- concrete carrier placement;
- family cardinality and closure;
- explicit closed-asset recognition;
- exact canonical partitioning;
- sponsor-region isolation;
- permissionless constructibility;
- public value availability;
- target constructor binding;
- deterministic linking;
- transaction ABI generation;
- target execution;
- resource accounting;
- relation-indexed evidence.

A node accepting a transaction is not the Guide-12 result. The guide succeeds only when every semantic relation reaches one explicit enforcement or evidence boundary and the resulting accepted transaction projects back to the exact compact-ASH semantic transition.

---

## One-line thesis · `rem:guide12-exec:thesis`

> Guide 12 succeeds when one validated compact-ASH compiler analysis becomes one typed, linked, ABI-driven Elements transaction relation whose every accepted transaction preserves the realization semantics and whose focused invalid variants fail at the boundary owning the violated claim—without importing target detail into compiler core, inspecting sponsor amounts, minting speculative identities, or claiming production readiness.

---

# 1. Governing rulings · `sec:guide12-exec:rulings`

## 1.1 The first pipeline is complete or it is not a pipeline · `rule:guide12-exec:complete-chain`

Guide 12 must not stop at any of these intermediate statements:

```text
the compiler emitted capabilities

the target adapter found reviewed primitives

tapscript emitted instructions

the linker produced bytes

the transaction builder produced a transaction

the node accepted the transaction
```

The required chain is:

```text
semantic relation
    ↓
compiler-owned requirement
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

A missing edge is a typed failure. No package may replace the missing edge with a comment, a boolean support flag, or an undocumented convention.

## 1.2 Target details remain below compiler core · `rule:guide12-exec:target-boundary`

Architecture and realization continue to own:

- operation identity;
- object and asset families;
- cardinalities and bound references;
- authorization;
- exact value relations;
- canonical deltas;
- open-flow roles;
- projection policy;
- constructibility;
- representation latitude;
- lifecycle obligations.

Compiler core continues to own:

- relation analysis;
- proof alternatives;
- fact-source requirements;
- disclosure;
- constructibility;
- lifecycle;
- abstract target requirements;
- abstract carriers;
- layout requirements;
- coverage requirements.

Only target and later packages may own:

- opcode selection;
- stack scheduling;
- tapleaves and taptrees;
- concrete transaction positions;
- control paths;
- target metadata encodings;
- target witness ordering;
- target transaction serialization;
- concrete resource formulas.

No Guide-12 convenience may introduce a target opcode, byte, tapleaf, stack index, transaction index, control path, or Elements-specific encoding into architecture, realization, or compiler semantic identity.

## 1.3 No relation disappears at a package boundary · `rule:guide12-exec:relation-census`

For the compact-ASH scope, require exact equality among:

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
operation evidence relation census
```

A package may add target-owned structural obligations. It may not remove, merge away, weaken, or silently reclassify an upstream semantic relation.

Every equality is checked in both directions:

- a missing relation is a completeness defect;
- an unexpected relation is an unowned requirement;
- a duplicate relation is two answers to one question;
- a reordered canonical vector is a projection defect where order is part of the schema.

## 1.4 Target acceptance and semantic acceptance are separate · `rule:guide12-exec:two-verdicts`

A real target accepting exact transaction bytes establishes a target verdict.

It does not establish:

- that those bytes were intended to represent compact ASH;
- that the compiler analyzed every compact-ASH relation;
- that the ABI classified every input and output;
- that the accepted output is the intended semantic successor;
- that no target relation was omitted;
- that the report’s claim census is correct.

Every accepted target transaction therefore receives a separate semantic-projection comparison.

The successful condition is:

\[\text{target accepted} \land \text{accepted projection} = \text{expected compact-ASH projection}\]

Verdict equality without projection equality is insufficient.

## 1.5 Construction failure and target rejection remain distinct · `rule:guide12-exec:failure-layers`

A malformed or impossible candidate may fail before target execution because:

- the semantic request is invalid;
- an input is duplicated;
- the public input view is incomplete;
- the candidate ABI rejects the shape;
- a constructor cannot instantiate;
- a signature is unavailable;
- a confidential proof cannot be built;
- linking is incomplete;
- the transaction is not serializable;
- executor infrastructure fails.

Those are semantic, construction, linker, ABI, or infrastructure failures.

They count as target-negative evidence only when the claimed relation is itself about that earlier boundary. A claim that “the target program rejects this transaction” requires a complete target transaction to be materialized and executed.

Every report distinguishes at least:

```text
semantic-request rejection

compiler-plan rejection

backend-emission rejection

linker rejection

ABI/construction rejection

executor infrastructure failure

consensus rejection before script

script-path rejection

relay-policy rejection

accepted transaction

report-layer semantic-projection rejection
```

No layer is inferred from what a test expected.

## 1.6 Sponsor opacity survives concrete lowering · `rule:guide12-exec:sponsor-opacity`

Guide 12 inherits the realization's sponsor-erasure law.

A compiler plan, target program, ABI, canonical report, or future release identity must not require an individual sponsor amount to be:

- explicit for protocol use;
- decoded;
- opened;
- compared with zero;
- proved positive;
- aggregated as a public integer;
- emitted in a diagnostic;
- emitted in a canonical report;
- used to distinguish sponsor change from another role.

The concrete sponsor relation is established through:

- exact sponsor-region membership;
- exact sponsor/protocol disjointness;
- exact asset and family roles;
- authorization of every sponsor input by its own target spending condition;
- at most one generic sponsor envelope;
- complete target transaction conservation;
- independent enforcement of every compact-ASH protocol output;
- role-based fee and change recognition.

The transaction constructor may use sponsor-private wallet state to construct and sign a balanced target transaction. Those values remain sponsor-local and are erased by the protocol projection.

A zero-valued ordinary sponsor member remains a semantic member when its role structure is exact. The first-party builder omits known explicit zero-valued sponsor change as construction policy. A deployment may separately reject such an output as policy. None of those facts licenses sponsor-value inspection in the protocol relation.

## 1.7 Guide 11 fixes the Phase-4 representation · `rule:guide12-exec:representation`

Guide 12 does not reopen public declassification.

The accepted Guide-11 policy selects an explicit public boundary for ownerless public maintenance. Therefore the Phase-4 compact-ASH candidate uses:

```text
ASH asset:
    explicit U identity

ASH amount:
    explicit target amount

ASH opening:
    not applicable

ASH owner:
    none
```

`PrivateCommitted` ASH is not supported in this candidate.

`PublicCommitted` ASH remains deferred against the three Guide-11 blockers and is not carried as a dormant backend branch.

The compiler may retain target-independent representation alternatives internally. The selected Phase-4 target-operation plan chooses `Explicit` under an explicit typed policy and records that the alternative set was narrowed by deployment/backend policy, not by semantic necessity.

## 1.8 Prototype decisions do not become operation patterns silently · `rule:guide12-exec:prototype-promotion`

Guide 10 accepted:

- a synthetic metadata-constructor continuity prototype;
- a wide-floor arithmetic prototype.

Compact ASH needs neither mutable STATE metadata nor wide-floor arithmetic.

The accepted constructor prototype may inform implementation technique. It does not automatically become the ASH constructor.

A Guide-10 prototype is promoted into a compact-ASH pattern only when:

- its exact relation is required by compact ASH;
- its exact source and target assumptions still hold;
- its resource behavior is measured in the complete compact-ASH transaction;
- its relation-indexed target evidence passes;
- the promotion is recorded explicitly.

Otherwise the prototype remains prototype-only.

## 1.9 Candidate and final states remain distinct · `rule:guide12-exec:candidate-state`

Guide 12 may construct:

```text
ValidatedTargetOperationPlan

CandidateTapscriptPlan

CandidateRelocatableTapscriptBundle

CandidateLinkedBundle

CandidateTransactionAbi

CandidateCompactAshEvidencePlan

ValidatedCandidateCompactAshReport
```

It must not construct or claim:

```text
FinalCompilerPlan

FinalLinkedBundle

FinalTransactionAbi

FinalCalibratedBounds

ProductionTargetEvidence

ValidatedDeploymentRelease

ProductionRelease
```

The candidate bundle is lifecycle-incomplete because `clear` is not implemented.

## 1.10 No speculative identity · `rule:guide12-exec:no-speculative-identity`

Typed in-process values use exact typed comparison.

Exact program and transaction bytes embedded in reports use exact byte comparison.

Guide 12 mints no:

```text
CompilerPlanHash
TapscriptPlanHash
RelocatableBundleHash
LinkedBundleHash
TransactionAbiHash
VectorSetHash
CompactAshReportHash
```

A digest enters only when a real package, process, cache, publication, distribution, deployment, or signature consumer appears and ADR-016 admission is satisfied.

No candidate type reserves a future digest field.

## 1.11 Exactness and deterministic failure · `rule:guide12-exec:exactness`

Semantic, target, ABI, linking, and evidence decisions use:

- checked bounded integers;
- exact finite sets and maps;
- exact byte comparison;
- exact target verdicts;
- exact finite search;
- independently checked small-instance oracles.

A search or analysis exceeding an explicit budget returns a typed complexity failure and no partial result.

It must not:

- drop a relation;
- select the best partial candidate;
- weaken a target proof;
- truncate a vector matrix;
- publish an incomplete report as complete;
- convert timeout into target rejection.

## 1.12 One authored source per semantic object · `rule:guide12-exec:typed-source`

First-party semantic paths consume validated typed Rust values.

Generated JSON, report files, plans, Markdown, model source text, target reference prose, and script disassembly remain publications or review inputs. They are never reverse-parsed into compiler, backend, linker, transaction, or evidence semantics.

External target bytes enter only through explicit target parsers and are converted immediately into validated typed values.

---

# 2. Entry conditions · `sec:guide12-exec:entry`

Core compact-ASH implementation begins only when every entry condition below holds.

## 2.1 Phase-3 entry · `gate:guide12-exec:phase3-entry`

Before Phase 4 becomes active:

- the Phase-3 exit gate is evaluated against the current tree;
- the target foundation is complete for every primitive Guide 12 will use;
- the Guide-10 constructor and wide-floor decisions remain accepted or explicitly superseded;
- the Guide-11 initial declassification policy remains accepted;
- package contracts and backlog state agree;
- the current tree is green and clean.

The backlog’s statement that the Phase-3 gate is ready to be checked is not itself the gate passing.

## 2.2 Semantic entry · `gate:guide12-exec:semantic-entry`

Required:

- architecture release validation passes;
- compact ASH remains in architecture and realization scope;
- the compact-ASH realization validates against architecture;
- the compiler analyzes compact ASH completely;
- realization and compiler relation censuses agree;
- sponsor-value opacity remains enforced;
- explicit ASH is an admitted representation;
- compact and clear lifecycle obligations remain explicit;
- no target-specific type has entered realization or compiler core.

## 2.3 Target entry · `gate:guide12-exec:target-entry`

Required:

- the reviewed target definition validates;
- the development deployment binding is welded to that exact target projection;
- the target-contract revision is unambiguous;
- every used primitive has complete success, failure, encoding, resource, and evidence contracts;
- signature behavior is fully welded;
- value and asset introspection used by the candidate is reviewed;
- execution-domain and leaf-version activation are observed;
- every required target evidence class is either available or explicitly unresolved;
- no mock is eligible for operation evidence.

## 2.4 Evidence entry · `gate:guide12-exec:evidence-entry`

Required:

- canonical evidence subjects cannot be forged by caller-authored claim metadata;
- transcripts bind exact target, deployment, and requests;
- executor requests contain no expected outcome;
- environment identity is observed and rechecked;
- required executor provenance is validated;
- failed reports and failed canonical cases cannot pass;
- protocol records are bounded and strict;
- executor startup, timeout, and shutdown paths reap the direct child;
- experimental conservation, normalization, and lifecycle protocol status is explicit;
- the Guide-12 review register below is closed.

## 2.5 Repository entry · `rule:guide12-exec:repository-entry`

Before editing:

```sh
git status --porcelain=v1 --untracked-files=all
git log -1 --oneline
```

Record:

```text
starting revision
working-tree status
architecture schema and semantic hash
realization version
target contract revision
native protocol revision
Guide-11 result revision
compact-ASH compiler stable projection
```

If the tree is not clean, stop and classify every change before proceeding.

---

# 3. Guide-12 preflight review register · `tab:guide12-exec:preflight`

The following findings come from two static passes over the supplied selected files at tree `0.3.6-dev…`. They are hypotheses until reproduced against the complete working tree. Each row is closed only by:

1. reproducing and fixing it with focused coverage;
2. disproving it with a typed argument;
3. showing it is unreachable through the complete public API;
4. deliberately narrowing the owning assurance claim.

| ID | Priority | Finding | Required disposition |
|---|---:|---|---|
| `G12-R01` | P1 | `ExpectedExecutorProvenance` has public fields of type `RevisionId`, while `RevisionId::new` admits abbreviated values; external code can bypass the constructor requiring full expected IDs. | Introduce a full-width revision type or private invariant-bearing fields; revalidate expected width at the gate. |
| `G12-R02` | P1 | `anchor_set_hash` accepts arbitrary strings under newline-join framing, so `{"a\nb"}` and `{"a","b"}` have one preimage and invalid anchor names can acquire an identity. | Hash only a validated anchor-set type; preserve the current recipe without migration unless framing itself changes. |
| `G12-R03` | P1 | Several shipped `emit-*` binaries bypass ADR-010: no shared panic hook, no TTY refusal, direct output, plain-text diagnostics, and ad hoc argv handling. | Move every shipped binary onto `cli-common` and add subprocess-contract coverage. |
| `G12-R04` | P1 | The Python executor captures raw child stderr and propagates it into first-party diagnostics or report detail; it also logs argv-derived paths. | Omit child stderr and caller paths; report fixed typed phase, method, and status only. |
| `G12-R05` | P1/P2 | Normalization report ingestion silently overwrites duplicate responses and ignores unexpected rows. | Require exact duplicate-sensitive response census equality. |
| `G12-R06` | P1/P2 | Lifecycle matrix completeness is checked only against the first pass; later passes may omit or duplicate rows. | Require every pass to carry the exact row census, with at least two complete distinct-process passes for cache independence. |
| `G12-R07` | P2 | Consensus script-error parsing stops at the first `)`, truncating mapped messages containing parentheses. | Parse the exact outer wrapper or use structured target error codes; test every mapped message. |
| `G12-R08` | P1/P2 | Guide-11, Phase-3, and backlog status/count prose disagree about whether Guide 11 ran and whether the lifecycle result is 8, 9, 16, or 18 rows/observations. | Reconcile active planning documents and enforce current status/census agreement. |
| `G12-R09` | P0/P1 | Rust and Python both claim native protocol revision 3 while carrying incompatible capability and response schemas; lifecycle has no Rust protocol type and `observed_openings` is rejected by Rust’s strict conservation response. | Unify under a new protocol revision or assign distinct experimental protocol schemas. |
| `G12-R10` | P1 | Confidential nonces with prefixes `0x02/0x03` are typed as quadratic-residue parity, although they use compressed-point oddness. | Correct the typed target fact or introduce an explicit opaque/no-parity-claim state. |
| `G12-R11` | P1 | Tapscript `resource_projection` counts opcode bytes and omits all push opcode, length-prefix, and payload bytes. | Derive script bytes from exact encoded program length. |
| `G12-R12` | P1 | Abstract stack validation retains pushed-literal width but not exact bytes, so `push 0x00; VERIFY` may be reported as having a successful path. | Track exact known bytes or narrow the API claim and prevent success-set liveness conclusions. |
| `G12-R13` | P2 | `TapscriptProgram::decode` enforces its 10,000-instruction limit only after parsing and allocating the entire input. | Stop parsing at the work bound and return typed failure immediately. |
| `G12-R14` | P1/P2 | Infrastructure-error responses can carry interpreter resource observations because shape validation returns before checking resources. | Reject every target observation on a non-target outcome; apply the rule to every workload. |
| `G12-R15` | P1/P2 | Experimental Python runners lack bounded records, total deadlines, process-group supervision, and cleanup on every post-spawn failure. | Reuse the Rust supervisor or implement equivalent bounded cleanup and regression coverage. |
| `G12-R16` | P2 | Evidence-registry documentation says no node evidence exists, contradicting current development conformance records; the intended claim is only that status is external to the immutable registry. | Correct the package documentation without introducing mutable evidence status. |

## 3.1 Preflight disposition rule · `rule:guide12-exec:preflight-disposition`

Each row receives:

```text
CONFIRMED
REFUTED
RECLASSIFIED
```

followed by:

```text
DONE
or
OPEN with named blocker
```

A broad “tests pass” result does not close a row unless a focused test reaches the reported path.

## 3.2 Preflight blocking rule · `gate:guide12-exec:preflight`

Core compiler-to-target work does not begin while any confirmed P0 or P1 row remains open.

P2 rows may remain only if:

- they cannot affect the candidate pipeline;
- the narrower assurance claim is recorded;
- Phase-4 exit explicitly excludes the affected lane;
- no report or gate overclaims the unresolved behavior.

`G12-R09`, `G12-R11`, `G12-R12`, and `G12-R14` are direct Phase-4 blockers because Guide 12 depends on protocol compatibility, exact resource prediction, abstract program validation, and strict evidence-layer separation.

---

# 4. Required authority · `sec:guide12-exec:authority`

Guide 12 is governed by:

```text
Attestation
Realization
typed architecture
implemented ADRs
accepted decisions
accepted Guide-11 result
package contracts
Phase-4 card
this guide's sequencing
```

When they disagree on an owner’s subject, the higher owner wins.

Required repository policy includes:

```text
ADR-010 command-line output
ADR-011 toolchain and dependency policy
ADR-015 execution trust
ADR-016 identities and evidence binding
ADR-017 path trust
ADR-018 upstream Elements workspace
ADR-019 label calculus
ADR-020 environment kinds
```

Required implementation decisions include:

```text
D001 typed Rust source
D002 realization layer
D003 tapscript first
D004 translation validation
D005 value representation
D006 transaction ABI
D007 Petgraph substrate
D008 exact and certified mathematics
```

Primary realization imports include:

```text
compact-ASH operation
open/closed recognition
exact canonical partition
exact open-flow partition
sponsor-value opacity
sponsor erasure
translation kernel inlining
cross-UTXO obligations
certificate relation
accounting and CT pins
```

No package parses these documents as semantic input.

---

# 5. Semantic compact-ASH contract · `sec:guide12-exec:semantics`

## 5.1 Inputs and output · `def:guide12-exec:operation`

Let the operation consume \(n\) ASH objects:

\[2 \le n \le N_{\mathrm{ASH}}\]

where \(N_{\mathrm{ASH}}\) is the candidate Phase-4 assignment for the architecture-owned `ASH_BATCH_MAX`, not a final calibration.

Each input carries:

```text
object family:
    ASH

asset:
    explicit U

value:
    xᵢ, explicit and positive

owner:
    none

representation:
    explicit public amount
```

The operation creates exactly one ASH output carrying:

\[X=\sum_{i=0}^{n-1}x_i\]

with:

\[0 < X < 2^{51}\]

## 5.2 Semantic effects · `rule:guide12-exec:effects`

Compact ASH:

- consumes no root;
- creates no root;
- reads no STATE;
- writes no STATE quantity;
- issues nothing;
- destroys nothing;
- consumes only ASH among protocol objects;
- creates exactly one ASH;
- moves `U` ownerlessly;
- emits no burn record;
- emits no burn event;
- emits no clear event;
- emits no residue event;
- requires a transition-certificate projection at the semantic/model boundary;
- may carry one optional fee-sponsor region.

## 5.3 Authorization · `rule:guide12-exec:authorization`

The protocol operation is permissionless.

No compact-ASH protocol leaf may require:

```text
receipt owner
ASH owner
operator
refund key
cadence key
private opening
private wallet database
```

Sponsor inputs remain authorized by their own target spending conditions. That authorization protects sponsor funds and does not authorize the compact-ASH semantic operation.

## 5.4 Exact canonical `U` flow · `rule:guide12-exec:u-flow`

The exact canonical flow is:

\[\sum \operatorname{value}(\text{ASH inputs})=\operatorname{value}(\text{successor ASH})\]

Its kind is:

```text
ownerless-lateral
```

There is:

```text
no issuance
no destruction
no second U flow
no omitted U source
no omitted U destination
no non-ASH U source
no non-ASH U destination
```

An aggregate balance alone is insufficient. Every source and destination belongs to the one declared flow exactly once.

## 5.5 Open-flow relation · `rule:guide12-exec:open-flow`

Compact ASH has no protocol L-BTC flow.

Its optional fee-sponsor region may contain:

```text
ordinary L-BTC sponsor inputs
optional sponsor change
target fee role
```

The protocol proof authenticates:

- exact sponsor-region membership;
- exact sponsor/protocol reference disjointness;
- at most one sponsor region;
- target roles and positions;
- absence of `U` and every other closed protocol asset from sponsor roles.

The target’s complete transaction validation establishes the residual conservation relation.

No protocol predicate reads an individual sponsor amount.

## 5.6 Root and projection policy · `rule:guide12-exec:roots-projections`

Every architecture root is forbidden:

```text
STATE
RESV
PACE
ENT_AUTH
DIST_AUTH
```

Required semantic projection:

```text
transition certificate
```

Forbidden specialized projections:

```text
burn
clear
distribution residue
```

Target execution need not serialize a certificate. It must enforce a transaction relation from which an independent observer derives the same compact-ASH certificate.

## 5.7 Lifecycle · `rule:guide12-exec:lifecycle`

The successor ASH requires two exits:

```text
compact-ash
clear
```

Guide 12 implements only compact ASH.

Therefore every candidate bundle and ABI records:

```text
implemented lifecycle:
    compact-ash

outstanding lifecycle:
    clear

release-complete:
    false
```

No report may describe the successor ASH profile as release-complete.

---

# 6. Exact relation inventory · `sec:guide12-exec:relations`

The actual relation census is derived from validated realization data. The list below is a review checklist, not a second registry.

## 6.1 Cardinality · `tab:guide12-exec:cardinality`

Required relation families include:

| Family | Requirement |
|---|---|
| ASH inputs | minimum 2, maximum candidate `ASH_BATCH_MAX` |
| ASH outputs | exactly 1 |
| sponsor inputs | minimum 0, maximum candidate `FEE_SPONSOR_INPUT_MAX` |
| sponsor outputs | 0 or 1 ordinary sponsor change |
| target fee role | exact target-dependent cardinality |
| protocol outputs | complete, disjoint, and position-bound |

## 6.2 Recognition · `tab:guide12-exec:recognition`

Required:

- each ASH input carries exact linked `U`;
- each ASH input is locked to the exact linked candidate ASH constructor;
- the successor carries exact linked `U`;
- the successor uses the exact linked ASH constructor;
- every ASH amount is in the exact explicit target encoding;
- sponsor inputs and change use admitted ordinary L-BTC roles;
- the target fee role cannot satisfy sponsor change;
- sponsor change cannot satisfy the target fee role;
- no confidential or unclassified output carries `U`;
- no open-asset look-alike satisfies ASH recognition.

## 6.3 Authorization and constructibility · `tab:guide12-exec:constructibility`

Required:

- compact ASH is permissionless;
- sponsor inputs satisfy their own target authorization;
- no protocol secret exists;
- the public constructor obtains every ASH fact from chain data and linked public configuration;
- no creator-local ASH state is required;
- no sponsor amount is promoted into protocol data.

## 6.4 Closure and partition · `tab:guide12-exec:closure`

Required:

- input families are exactly ASH plus optional sponsor members;
- output roles are exactly successor ASH, optional sponsor change, and target fee role;
- every closed-asset-capable position is classified;
- every canonical `U` source and destination appears exactly once;
- no reference appears in both protocol and sponsor regions;
- no output remains unclassified where it could hide `U`.

## 6.5 Target-independent policy · `tab:guide12-exec:policy-relations`

Required:

- every root forbidden;
- transition certificate required;
- every specialized projection forbidden;
- explicit ASH representation selected;
- compact and clear lifecycle obligations retained;
- whole-transaction conservation retained as external target evidence.

---

# 7. Compiler target-operation boundary · `sec:guide12-exec:compiler-boundary`

## 7.1 Current public boundary is insufficient · `rem:guide12-exec:compiler-gap`

The current public compiler target boundary exposes:

```text
abstract capability census
external evidence-role census
```

That is sufficient for static target assessment and insufficient for operation emission.

A backend also needs validated typed access to:

- exact operation scope;
- relation identities;
- activation cases;
- selected representation policy;
- approved proof alternatives;
- source requirements;
- constructibility requirements;
- lifecycle obligations;
- abstract carriers;
- layout requirements;
- coverage requirements;
- unresolved external evidence.

Guide 12 adds the smallest compiler-owned public projection that carries those facts without exposing mutable internals or graph handles.

## 7.2 Validated operation-plan type · `rule:guide12-exec:operation-plan`

Introduce a compiler-owned type with private fields and no unchecked constructor, illustratively:

```rust
pub struct ValidatedTargetOperationPlan {
    operation: architecture::OperationId,
    source: TargetOperationSource,
    cases: BTreeMap<TargetExecutionCaseId, TargetExecutionCase>,
    relations: BTreeMap<realization::RelationId, TargetRelationRequirement>,
    carriers: BTreeSet<AbstractCarrierRequirement>,
    layout: BTreeSet<TargetLayoutRequirement>,
    coverage: BTreeMap<CoverageRequirementId, TargetCoverageRequirement>,
    capabilities: TargetRequirementSet,
    external_evidence: BTreeSet<ExternalEvidenceRole>,
    lifecycle: TargetLifecycleStatus,
}
```

Exact naming remains implementation-owned.

Required properties:

- constructed only from a fully validated analyzed program;
- source operation is exactly `compact-ash`;
- architecture and realization bindings are retained;
- explicit Phase-4 representation policy is retained;
- relation, carrier, layout, and coverage censuses are exact;
- canonical containers use stable typed keys;
- no Petgraph index appears;
- no search count appears;
- no target byte or opcode appears;
- no filesystem path or environment value appears;
- no digest appears;
- complete re-derivation validation runs before publication.

## 7.3 Alternative retention and selection · `rule:guide12-exec:plan-selection`

The compiler retains every feasible target-independent plan.

The Phase-4 policy then filters:

```text
operation:
    compact-ash only

ASH representation:
    Explicit

constructibility:
    public permissionless

lifecycle:
    compact implemented, clear outstanding

target:
    no target-specific choice yet
```

No feasible plan is removed because it is inconvenient.

After target assessment, the backend selects one concrete candidate only under a typed `CompactAshBackendPolicy` that states:

- exact target projection;
- explicit representation;
- candidate cardinality assignment;
- layout family;
- sponsor profile;
- proof-pattern preference;
- deterministic tie-break.

A lexicographic least-key selection is permitted only when the policy states it and every remaining candidate is semantically equivalent under the accepted objective.

## 7.4 Public API discipline · `gate:guide12-exec:compiler-api`

Required tests include:

- external callers cannot construct the validated operation-plan state;
- no internal analyzed-program type leaks;
- no graph handle leaks;
- equal inputs produce equal projections;
- declaration permutations produce equal projections;
- a removed relation rejects;
- an added relation rejects;
- a changed carrier rejects;
- a changed layout requirement rejects;
- a changed coverage row rejects;
- sponsor amount cannot enter any public plan field;
- no digest field exists.

---

# 8. Target assessment · `sec:guide12-exec:target-assessment`

## 8.1 Static assessment comes first · `rule:guide12-exec:assessment-order`

The target adapter assesses every compiler-required capability against the exact reviewed target.

Expected capability families include:

```text
authenticated object recognition
authenticated family cardinality
authenticated canonical partition
authenticated open-flow partition
authenticated root effects
authenticated projection set
exact public amount arithmetic
owner authorization for sponsor inputs
public constructibility
whole-transaction value conservation
```

Root effects and specialized projections may be discharged as structural absences, but each remains represented in the assessment census.

## 8.2 Support remains multi-state · `rule:guide12-exec:assessment-states`

Every capability remains one of:

```text
Unsupported

MissingTargetPrimitives

BackendPatternRequired

BackendStructural

ExternalEvidenceRequired

CompleteBackendPattern
```

A reviewed primitive is not a completed proof pattern.

An external consensus claim is not a primitive.

A structural ABI obligation is not a target execution result.

## 8.3 Missing capability fails closed · `rule:guide12-exec:no-weakening`

If no accepted target proof remains for any compact-ASH relation, emission fails.

The backend must not:

- omit conservation;
- substitute whole-transaction balance for object closure;
- substitute sponsor positivity for protocol correctness;
- use unauthenticated metadata amount;
- accept confidential `U`;
- require owner-private ASH openings;
- add hidden authorization;
- drop lifecycle obligations.

## 8.4 First complete backend-pattern identifiers · `rule:guide12-exec:pattern-admission`

The uninhabited `BackendPatternId` gains its first variants only for exact patterns the complete compact-ASH operation evidence establishes.

Illustrative candidates:

```rust
pub enum BackendPatternId {
    CompactAshObjectRecognitionV1,
    CompactAshShapeV1,
    CompactAshCanonicalPartitionV1,
    CompactAshExplicitSumV1,
    CompactAshSponsorIsolationV1,
}
```

A pattern enters only with:

- semantic owner;
- target prerequisites;
- typed instruction fragment;
- stack contract;
- failure behavior;
- ABI assumptions;
- source requirements;
- resource formula;
- positive vectors;
- negative vectors;
- operation evidence.

Patterns not needed by compact ASH remain uninhabited.

---

# 9. Concrete shape policy · `sec:guide12-exec:shape-policy`

## 9.1 No hidden loop or branch assumption · `rule:guide12-exec:finite-shapes`

The reviewed target has no general loop primitive and Guide 12 assumes no general conditional dispatch.

Therefore a finite candidate bound is represented by statically specialized transaction shapes.

Define a typed key:

```rust
pub struct CompactAshShape {
    pub ash_inputs: NonZeroCount,
    pub sponsor_inputs: Count,
    pub sponsor_change: SponsorChangePresence,
}
```

A valid candidate shape satisfies:

```text
2 ≤ ash_inputs ≤ candidate ASH bound
0 ≤ sponsor_inputs ≤ candidate sponsor bound
sponsor_change ⇒ sponsor_inputs > 0
```

The backend emits shape-specific coordinator and member relations, or another exact finite construction proven equivalent.

No attacker-selected in-script dispatch determines which semantic operation runs.

## 9.2 Why specialization is explicit · `rem:guide12-exec:specialization`

A generic “inspect the first \(n\) inputs” description is not an emitted program unless the target can:

- authenticate \(n\);
- inspect exactly those positions;
- enforce the range;
- reject every omitted or extra member;
- do so within its actual instruction set.

Guide 12 does not hide finite unrolling behind target-independent prose.

The candidate-bound study measures how shape specialization affects:

- leaf count;
- program bytes;
- tree depth;
- control bytes;
- witness size;
- transaction weight;
- compile and link time.

## 9.3 Initial useful candidate · `rule:guide12-exec:useful-candidate`

The Phase-4 demonstration candidate must support:

- at least one batch larger than the minimum;
- at least one sponsored shape;
- sponsor change present and absent where the target transaction form permits both;
- every count from 2 through its stated ASH candidate bound, unless the candidate explicitly uses a sparse supported-count set and reports that limitation.

A candidate supporting only two ASH inputs is useful as a first wave and insufficient for Phase-4 exit.

---

# 10. Candidate transaction layout · `candidate:guide12-exec:canonical-layout`

## 10.1 Input order · `rule:guide12-exec:input-layout`

Within one shape:

```text
inputs 0..n-1:
    ASH family

inputs n..n+s-1:
    sponsor suffix
```

ASH inputs are sorted by canonical outpoint order.

Sponsor inputs are sorted by canonical outpoint order within the suffix.

Duplicates are rejected before sorting.

Input 0 is the canonical coordinator.

No caller selects a different coordinator.

## 10.2 Output roles · `rule:guide12-exec:output-layout`

The candidate output order is:

```text
output 0:
    successor ASH

next optional role:
    sponsor change

final target role where the selected Elements form requires it:
    fee output
```

The exact fee-output representation is established by a focused target-source review before the ABI freezes.

The review must answer separately for:

- sponsorless zero-fee consensus transactions;
- sponsored positive-fee transactions;
- relay policy;
- explicit fee-output cardinality;
- canonical fee position;
- zero-fee representation.

If sponsorless compact ASH is not constructible under target consensus, Phase 4 blocks. The backend must not silently make sponsorship mandatory where architecture declares it optional.

## 10.3 Coordinator uniqueness · `rule:guide12-exec:coordinator`

The coordinator program requires:

```text
current input index = 0
```

Each member program requires:

```text
1 ≤ current input index < n
```

and authenticates the exact shape key or an equivalent target relation.

Therefore:

- a member leaf at input 0 rejects;
- a coordinator leaf at a later input rejects;
- two coordinator leaves cannot satisfy one valid transaction;
- no valid transaction lacks a coordinator;
- every ASH member lies in the authenticated ASH range.

## 10.4 Sponsor suffix · `rule:guide12-exec:sponsor-suffix`

The sponsor suffix is every input after the ASH range.

The coordinator authenticates:

- exact suffix start;
- exact suffix length;
- each sponsor member carries the reserve asset L-BTC;
- no sponsor member carries `U`;
- no ASH member lies in the sponsor suffix;
- no sponsor member lies in the ASH range;
- the sponsor region appears at most once.

Sponsor authorization is supplied by each sponsor input’s own target spending condition.

The Phase-4 candidate records the admitted sponsor input profile. It must not claim support for arbitrary wallet programs unless the exact target relation for arbitrary owner authorization is actually established.

## 10.5 Sponsor change · `rule:guide12-exec:sponsor-change`

The candidate ABI admits at most one sponsor-change role.

The protocol relation recognizes it by:

```text
declared output role
+
canonical position
+
reserve asset
+
admitted sponsor output program class
```

It does not recognize change by positivity or by comparing the amount with zero.

A first-party explicit builder omits known zero-valued change.

Confidential sponsor-change presence may reveal whether the private residual is zero. Any builder claiming sponsor-value opacity records that shape leakage.

## 10.6 Target fee role · `rule:guide12-exec:fee-role`

The target fee output is not a protocol object.

It cannot satisfy:

```text
ASH
sponsor change
CPFP anchor
any closed protocol object
```

Its identity is target-structural:

```text
target fee role
+
canonical position
+
target-defined program form
+
reserve asset
```

not “an ordinary output whose amount is zero.”

---

# 11. ASH constructor · `candidate:guide12-exec:ash-constructor`

## 11.1 Static constructor first · `rule:guide12-exec:static-constructor`

ASH carries no mutable semantic metadata beyond:

- explicit `U`;
- amount;
- ownerless object role;
- linked operation-program set.

The initial candidate therefore uses a deterministic static constructor binding:

```text
target contract
+
explicit U asset
+
static ASH program set
+
explicit value policy
+
internal-key policy
+
leaf version
+
candidate shape set
```

No synthetic counter, representation nonce, or mutable metadata leaf enters merely because the Guide-10 prototype used one.

## 11.2 Candidate leaf set · `rule:guide12-exec:leaf-set`

For each admitted shape, the constructor carries at least:

```text
compact-ash coordinator leaf
compact-ash member leaf
```

Shape sharing is permitted only where one typed proof establishes that a shared leaf enforces the exact same relation for every shape using it.

No placeholder clear leaf is added.

Omission plus explicit lifecycle incompleteness is preferred over an apparently usable future leaf.

## 11.3 Internal key · `rule:guide12-exec:internal-key`

Use the accepted public unspendable internal-key policy only after confirming its exact assumptions for this constructor.

There is no:

- operator key;
- release key;
- wallet key;
- generated-and-discarded private key;
- caller-selected internal key.

The residual discrete-log assumption remains explicit.

## 11.4 No key-path escape · `rule:guide12-exec:no-keypath`

The candidate constructor provides no accepted key-path escape.

Every target vector includes an attempted key-path spend and requires rejection or unconstructibility under the selected internal-key policy.

## 11.5 Constructor status · `rule:guide12-exec:constructor-status`

The constructor is:

```text
candidate compact-ASH constructor
```

It is not:

```text
final ASH constructor
```

because the clear lifecycle is absent.

---

# 12. Tapscript proof patterns · `sec:guide12-exec:patterns`

Every pattern states:

- owning semantic or target-structural relation;
- required target primitives and evidence;
- typed instruction fragment;
- initial and successful stack state;
- every non-aborting failure state;
- every aborting cause;
- witness requirements;
- constructibility;
- disclosure;
- resource formula;
- positive and negative vectors.

## 12.1 ASH input recognition · `rule:guide12-exec:input-recognition`

For every ASH input, establish:

```text
spent asset = exact linked U asset
spent program = exact linked ASH constructor
value encoding = exact explicit amount form
value domain = positive and below 2^51
current input belongs to the authenticated ASH range
selected leaf role matches coordinator/member position
```

A public program match without exact `U` is insufficient.

A copied ASH program carrying another asset is inert and cannot satisfy the relation.

## 12.2 Successor recognition · `rule:guide12-exec:output-recognition`

At output 0, establish:

```text
asset = exact linked U
program = exact linked ASH constructor
value encoding = explicit
value = exact aggregate X
value domain = positive and below 2^51
```

No other output may carry `U`.

## 12.3 Family cardinality · `rule:guide12-exec:family-cardinality`

The coordinator authenticates:

- total input count;
- exact ASH count \(n\);
- exact sponsor count \(s\);
- total output count;
- optional sponsor-change presence;
- target fee-role presence;
- complete role order.

Caller-proposed counts are witnesses at most. They must equal target-introspected counts.

## 12.4 Exact aggregate arithmetic · `rule:guide12-exec:aggregate`

For explicit ASH amounts, the coordinator computes:

\[X=\sum_{i=0}^{n-1}x_i\]

Each addition:

- consumes exact fixed-width operands;
- checks the success flag immediately;
- rejects overflow;
- re-establishes the semantic amount domain;
- leaves no unchecked arithmetic flag.

The output amount must equal \(X\).

The backend must not use a sponsor amount to close or validate the `U` equation.

## 12.5 Closed-asset closure · `rule:guide12-exec:closed-asset-closure`

The coordinator inspects every admitted input and output position that could carry a protocol closed asset.

Require:

```text
every U input:
    one ASH source in the ASH range

the only U output:
    successor ASH at output 0

every sponsor role:
    L-BTC, never U

every target fee role:
    L-BTC, never U

every unexpected output:
    rejected
```

No confidential asset commitment can carry a closed protocol asset.

## 12.6 Canonical partition · `rule:guide12-exec:canonical-partition`

The coordinator proves:

```text
all ASH sources belong to one U flow
successor belongs to that flow
movement kind is ownerless-lateral
source total equals successor value
no source appears twice
no source is omitted
no destination appears twice
no destination is omitted
no issuance exists
no destruction exists
```

A whole-transaction aggregate cannot substitute for this partition.

## 12.7 Member participation · `rule:guide12-exec:member-participation`

Every non-coordinator ASH input proves:

- its current input index is in the shape’s member range;
- its spent asset is exact `U`;
- its spent program is the linked ASH constructor;
- it selected the correct member leaf;
- the operation is permissionless.

Member leaves do not independently re-prove the whole aggregate unless deliberate duplicate enforcement is selected and recorded.

## 12.8 Permissionless path · `rule:guide12-exec:permissionless`

The emitted compact-ASH protocol leaves contain no signature check.

The byte-level audit must show no hidden:

```text
owner signature
operator signature
refund signature
cadence gate
private opening
```

Sponsor input signatures remain in sponsor input programs and are not compact-ASH authorization.

## 12.9 Sponsor isolation · `rule:guide12-exec:sponsor-isolation`

The sponsored coordinator proves:

- exact sponsor suffix;
- exact optional change role;
- exact target fee role;
- protocol/sponsor disjointness;
- no sponsor reference satisfies a protocol role;
- no protocol reference satisfies a sponsor role;
- no second sponsor region;
- no sponsor amount enters a protocol predicate.

Whole-transaction L-BTC conservation remains an external target claim.

## 12.10 Root and event absence · `rule:guide12-exec:absence`

The exact family census makes root and specialized-event absence structural.

The candidate transaction contains no:

```text
STATE
RESV
PACE
ENT_AUTH
DIST_AUTH
burn record
tag-burn output
tag-recon output
distribution-residue output
```

The compiler, linker, ABI, and target vector layers each independently check their own view of this absence.

## 12.11 Final truth and failure states · `rule:guide12-exec:final-truth`

Every program:

- reaches one canonical true item on success;
- leaves no unchecked arithmetic success flag;
- leaves no alternate-stack residue;
- has no non-aborting failure state satisfying final truth;
- remains within target element and stack limits.

This requirement depends on closing `G12-R12`: exact known false literals must not acquire impossible abstract success paths.

---

# 13. Relocatable backend bundle · `sec:guide12-exec:relocatable`

## 13.1 Backend output · `rule:guide12-exec:relocatable-output`

`tapscript` emits a typed candidate relocatable bundle containing:

- validated compiler operation plan;
- exact target projection;
- backend policy;
- candidate shape set;
- static ASH constructor template;
- typed coordinator and member programs;
- selected proof-pattern IDs;
- concrete relation placements;
- concrete candidate layout;
- witness-role declarations;
- typed symbols;
- typed relocations;
- resource formulas;
- relation-carrier provenance;
- explicit candidate status;
- outstanding clear lifecycle.

## 13.2 Typed programs only · `rule:guide12-exec:typed-programs`

Every program is constructed from typed target instructions.

Raw script bytes appear only as deterministic serialization of a validated typed program.

For every emitted program:

```text
typed instructions
→ canonical bytes
→ supported-subset parser
→ equal typed instructions
```

must hold.

The parser’s instruction work bound is enforced during parsing, not after unbounded allocation.

## 13.3 Exact resource projection · `rule:guide12-exec:program-resources`

Program script bytes are:

```text
exact canonical encoded byte length
```

not the sum of opcode costs alone.

Pushes contribute:

- push opcode;
- width prefix;
- payload bytes.

Other resource dimensions remain separately typed:

- operation cost;
- validation budget;
- peak main stack;
- peak alternate stack;
- maximum item;
- target policy dependencies.

## 13.4 Symbols and relocations · `rule:guide12-exec:symbols`

Symbols identify typed roles such as:

```text
U asset identifier
ASH constructor
coordinator program for one shape
member program for one shape
target leaf version
unspendable internal key
candidate ASH bound
candidate sponsor bound
target fee role
```

A relocation states:

- source symbol;
- target role;
- semantic location;
- width;
- encoding;
- multiplicity;
- expected placeholder where byte patching is unavoidable.

Prefer structured substitution before serialization.

Variable-width byte patching is prohibited.

## 13.5 Candidate status · `rule:guide12-exec:backend-status`

Backend artifacts distinguish:

```text
prototype
candidate-operation-proven
production-approved
```

Guide 12 may promote only the exact compact-ASH patterns and bundle to `candidate-operation-proven`, and only after full operation evidence.

---

# 14. Linker foundation · `sec:guide12-exec:linker`

## 14.1 Package creation · `rule:guide12-exec:linker-package`

Create:

```text
packages/linker
Cargo package: tripod-linker
Rust library: linker
```

Allowed direct first-party dependencies:

```text
tapscript
target-elements
```

Forbidden direct dependencies:

```text
model
transaction
vectors
release
artifacts
```

## 14.2 First scope only · `rule:guide12-exec:linker-scope`

The first linker implements only what compact ASH requires:

- typed definition census;
- typed reference census;
- exact two-pass symbol resolution;
- structured relocation;
- deterministic static taptree assembly;
- constructor assembly;
- relation-carrier closure;
- resource-formula resolution;
- candidate linked-bundle state.

No universal linker abstraction is introduced before the first operation demonstrates its need.

## 14.3 Two-pass resolution · `rule:guide12-exec:two-pass-linking`

Pass 1:

```text
collect definitions
validate unique typed keys
validate definition types
sort by stable key
```

Pass 2:

```text
resolve every reference
validate expected target type
reject missing targets
reject ambiguous targets
build frozen reference graph
```

Display strings are not symbol identity.

## 14.4 Cycle policy · `rule:guide12-exec:linker-cycles`

Compact ASH is expected to require no metadata-dependent constructor cycle.

If a cycle appears:

- compute SCCs through Petgraph;
- normalize members by stable key;
- classify every cyclic edge;
- require an explicit authenticated resolution strategy;
- reject unclassified cycles.

Finding an SCC is not accepting it.

Repeated hashing until bytes stabilize is prohibited.

## 14.5 Deterministic taptree · `rule:guide12-exec:taptree`

The candidate tree input contains:

- complete leaf set;
- exact leaf version;
- program role;
- exact positive integer weight;
- maximum depth;
- stable tie-break key.

If no execution-frequency data exists, use equal weights.

For the small compact-ASH leaf set, compare the selected deterministic tree against exhaustive enumeration under the declared objective.

Tree construction is independent of declaration order.

## 14.6 Carrier closure · `rule:guide12-exec:carrier-closure`

The linker compares:

```text
compiler-required carriers
backend-emitted carriers
linked reachable carriers
```

For every relation-case:

- at least one compatible carrier exists;
- the carrier is reachable in every active case;
- required facts resolve;
- the selected program remains reachable;
- deliberate duplicate carriers agree;
- no uniquely carrying program is removed.

## 14.7 Candidate linked bundle · `rule:guide12-exec:linked-bundle`

`CandidateLinkedBundle` contains:

- exact compact-ASH scope;
- exact target projection;
- candidate shape and bound assignment;
- linked constructor;
- linked programs;
- taptree and control recipes;
- concrete relation placements;
- layout and witness handoff;
- linked resource formulas;
- unresolved target evidence;
- unresolved clear lifecycle;
- explicit candidate status.

It has no digest unless a separately admitted external consumer appears.

---

# 15. Candidate transaction ABI · `sec:guide12-exec:abi`

## 15.1 Package creation · `rule:guide12-exec:transaction-package`

Create:

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

The library performs no RPC, wallet lookup, network submission, or production key custody.

## 15.2 Third-party transaction substrate · `rule:guide12-exec:transaction-dependency`

Before implementation selects a Rust Elements transaction library, record:

- exact crate and version;
- upstream source and tag;
- enabled features;
- licence;
- Rust 1.88 compatibility;
- transitive graph;
- build scripts;
- dependency-internal unsafe or FFI;
- deterministic serialization behavior;
- sighash and CT coverage;
- advisory status;
- lockfile change;
- why first-party minimal structures are insufficient.

No dependency enters merely because the package is planned.

The selected library supplies target transaction structures. It does not become protocol authority.

## 15.3 ABI derivation · `rule:guide12-exec:derive-abi`

The candidate ABI derives from:

```text
candidate linked bundle
+
exact reviewed target
+
candidate shape assignment
+
Guide-11 explicit ASH policy
+
typed target transaction-form decision
```

It does not parse script bytes, reports, plans, or reference prose.

## 15.4 ABI contents · `rule:guide12-exec:abi-contents`

The candidate compact-ASH ABI states:

- input family order;
- output role order;
- exact shape key;
- coordinator rule;
- ASH range;
- sponsor suffix;
- sponsor-change role;
- target fee role;
- constructor recipe;
- selected tapleaf per input role;
- control-path recipe;
- witness item order;
- target transaction version;
- sequence constraints;
- explicit ASH representation;
- candidate bounds;
- target policy status;
- concrete relation placements.

## 15.5 Typed operation request · `rule:guide12-exec:request`

A compact-ASH request may select only:

- ASH input outpoints;
- optional sponsor input capabilities;
- optional sponsor-change destination under sponsor policy;
- explicit test-only construction randomness where required.

It may not select:

- successor amount;
- successor asset;
- successor program;
- coordinator;
- family range;
- target program role;
- witness order;
- specialized projection;
- target fee role.

Those derive from the ABI.

## 15.6 Public construction view · `rule:guide12-exec:public-view`

The permissionless constructor receives:

- ASH outpoints;
- exact target asset and program data;
- public explicit ASH amounts;
- linked constructor data;
- candidate ABI;
- target transaction context;
- sponsor-local capabilities where selected.

It receives no protocol owner or operator secret.

## 15.7 Sponsor capability · `rule:guide12-exec:sponsor-capability`

A sponsor adapter may supply:

- sponsor outpoints;
- public owner/program identities;
- signing capability;
- private wallet amount/opening data needed for transaction construction;
- optional change destination.

Canonical reports retain only public role data and erase private sponsor values and openings.

Signing requests bind:

- exact finalized transaction;
- exact input;
- public signer role;
- selected sighash profile;
- protected output set.

## 15.8 Construction stages · `rule:guide12-exec:construction-stages`

The candidate transaction pipeline is:

1. validate request and public input view;
2. reject duplicate and overlapping outpoints;
3. sort ASH inputs canonically;
4. select exact supported shape;
5. derive coordinator;
6. validate public ASH object facts;
7. compute exact successor amount;
8. instantiate successor constructor;
9. derive sponsor suffix and optional change role;
10. assemble target transaction roles;
11. finalize all protected outputs;
12. issue sponsor signing requests;
13. collect and validate signatures;
14. assemble script-path witnesses and control data;
15. perform ABI-local preflight;
16. return exact target bytes plus a typed construction report.

No signing request is issued before protected outputs are final.

## 15.9 Synthetic test ASH · `rule:guide12-exec:synthetic-ash`

Because burn is not implemented, target vectors create synthetic development ASH using:

- an explicitly issued disposable test asset standing for `U`;
- the candidate ASH constructor;
- deterministic public test values;
- a test-only funding ceremony.

Every report states:

```text
synthetic target fixture
not produced by burn
not evidence of burn lineage
not an attestation event
authorizes nothing of value
```

No burn or attestation claim derives from synthetic ASH.

## 15.10 ABI status · `rule:guide12-exec:abi-status`

The output is:

```text
CandidateTransactionAbi
```

It is not final and carries no identity digest by default.

---

# 16. Operation evidence ownership · `sec:guide12-exec:evidence-ownership`

## 16.1 Package creation · `rule:guide12-exec:vectors-package`

Create:

```text
packages/vectors
Cargo package: tripod-vectors
Rust library: vectors
```

Expected direct dependencies:

```text
architecture
realization
model
compiler
target-elements
linker
transaction
```

The exact executor dependency is decided in the preflight implementation series.

## 16.2 Executor ownership decision · `rule:guide12-exec:executor-ownership`

The preferred first-operation direction is:

```text
target-elements-conformance
    owns target-generic process supervision,
    strict protocol framing,
    environment binding,
    and executor provenance

vectors
    owns compact-ASH semantic subjects,
    operation claims,
    relation coverage,
    and accepted projections
```

`vectors` may depend on a narrow public target-generic executor boundary from `target-elements-conformance`.

That boundary must not expose primitive or prototype claim semantics as operation semantics.

If a clean target-generic boundary cannot be exposed without making `target-elements-conformance` own compact-ASH meaning, extract a shared executor package before proceeding.

Do not duplicate the process supervisor in `vectors`.

## 16.3 Protocol revision · `rule:guide12-exec:protocol-revision`

If conservation, normalization, lifecycle, and compact-ASH operation workloads remain under the native executor protocol, introduce a new revision containing typed records for all admitted workloads.

The revision must include:

- complete request and response types;
- exact capability vocabulary;
- bounded record sizes;
- request-subject-only rule;
- typed workload discrimination;
- target/deployment transcript binding;
- exact observed-value shapes;
- no unknown-field tolerance;
- explicit migration refusal for earlier revisions.

Rust and Python implementations must round-trip every record in both directions.

If the experimental lanes remain separate, assign them distinct schemas and stop calling them native protocol revision 3.

## 16.4 Canonical operation evidence plan · `rule:guide12-exec:evidence-plan`

Introduce:

```rust
pub struct CompactAshEvidencePlan {
    semantic_cases: Vec<CompactAshSemanticCase>,
    target_cases: Vec<CompactAshTargetCase>,
    relation_coverage: BTreeMap<CoverageRequirementId, RelationCoverageRow>,
}
```

with private fields and no unchecked constructor.

It derives from:

- compiler coverage requirements;
- canonical mutation registry;
- exact candidate linked bundle;
- exact candidate ABI;
- exact target and deployment binding.

Ad hoc vectors produce experimental reports only.

## 16.5 Report roles · `rule:guide12-exec:report-roles`

Keep distinct:

```text
semantic fixture result
construction result
linker result
ABI validation result
target execution result
accepted semantic projection result
relation coverage result
resource result
external target dependency result
```

A broad “all compact-ASH tests passed” record is insufficient.

No report carries a digest in this phase.

---

# 17. Semantic fixtures · `sec:guide12-exec:fixtures`

## 17.1 Positive fixture origin · `rule:guide12-exec:model-fixtures`

A positive semantic fixture begins from a model-valid world.

Where the ordinary model path cannot yet produce ASH without burn, use an explicitly approved test-only model fixture that:

- creates synthetic ASH;
- preserves the global invariant;
- is labeled fault/test construction rather than protocol history;
- is inaccessible to production callers.

Then:

1. execute compact ASH through the invariant-wrapped model path;
2. retain the predecessor/request/successor binding;
3. project realization observations;
4. evaluate every active realization relation;
5. record the exact semantic successor and certificate.

## 17.2 Target materialization is separate · `rule:guide12-exec:materialization`

The semantic fixture contains no:

- target input index;
- target output index;
- target program;
- tapleaf;
- control block;
- transaction byte;
- sponsor wallet state.

Target materialization occurs through:

```text
candidate linked bundle
+
candidate ABI
+
typed operation request
+
public target input view
```

## 17.3 Independent expected result · `rule:guide12-exec:expected-result`

Expected semantic behavior derives from:

- realization relations;
- executable model;
- typed projection adapters.

The candidate backend does not generate the semantic expected result used to judge itself.

Expected target bytes may derive from the linked bundle and ABI when exact bytes are themselves the subject. That does not make them semantic expectations.

## 17.4 Accepted projection · `rule:guide12-exec:accepted-projection`

For an accepted target transaction, compare:

- exact ASH input family;
- exact successor ASH family;
- exact explicit `U`;
- exact aggregate amount;
- ownerless status;
- no roots;
- no issuance;
- no destruction;
- ownerless-lateral canonical flow;
- sponsor-region existence and membership;
- no specialized event;
- transition-certificate derivation;
- exact operation identity.

Individual sponsor amounts and openings are absent.

---

# 18. Complete vector matrix · `sec:guide12-exec:vectors`

## 18.1 Positive semantic cases · `tab:guide12-exec:positive-vectors`

Required:

- minimum two ASH inputs;
- three ASH inputs;
- candidate maximum;
- values summing at fixed-width boundaries;
- values summing to \(2^{51}-1\);
- canonical input-order normalization;
- sponsorless consensus transaction;
- sponsored transaction;
- one sponsor input;
- multiple sponsor inputs where the candidate supports them;
- sponsor change present;
- sponsor change absent;
- exact successor amount;
- repeated execution producing equal report bytes.

## 18.2 Cardinality mutations · `tab:guide12-exec:cardinality-vectors`

Required:

- zero ASH inputs;
- one ASH input;
- one above candidate ASH maximum;
- zero ASH outputs;
- two ASH outputs;
- sponsor count above candidate maximum;
- two sponsor-change outputs;
- duplicate target fee roles;
- missing required target fee role;
- unexpected target-only output.

## 18.3 Input mutations · `tab:guide12-exec:input-vectors`

Required:

- duplicate ASH outpoint;
- one outpoint claimed as ASH and sponsor;
- wrong asset at one ASH input;
- correct asset under wrong constructor;
- correct constructor under wrong representation;
- foreign open asset carrying ASH-shaped metadata;
- unrecognized closed-asset input;
- sponsor member in ASH range;
- ASH member in sponsor suffix;
- noncanonical ASH ordering;
- wrong coordinator;
- two coordinator leaves;
- no coordinator;
- member leaf at input zero;
- coordinator leaf at member index.

## 18.4 Output mutations · `tab:guide12-exec:output-vectors`

Required:

- successor one below the sum;
- successor one above the sum;
- successor zero;
- successor at or above \(2^{51}\);
- wrong output asset;
- confidential closed asset;
- wrong constructor;
- ordinary wallet `U` output;
- second `U` output;
- sponsor change carrying `U`;
- fee role carrying `U`;
- successor at wrong role;
- sponsor change and fee exchanged;
- malformed fee role;
- unclaimed output.

## 18.5 Canonical-partition mutations · `tab:guide12-exec:partition-vectors`

Required:

- omit one ASH source;
- cite one source twice;
- cite one destination twice;
- claim successor through two flows;
- add a destruction;
- add issuance;
- change movement kind to owner-controlled lateral;
- leave one canonical object unwitnessed;
- route part of `U` into an undeclared object;
- preserve aggregate totals while shortening successor and growing another output.

## 18.6 Authorization and constructibility mutations · `tab:guide12-exec:constructibility-vectors`

Required:

- hidden owner signature;
- hidden operator signature;
- sponsor input without valid authorization;
- one sponsor authorization omitted;
- output set changed after sponsor signing;
- public ASH amount unavailable;
- public amount copied from another object;
- constructor uses creator-private state;
- permissionless constructor accesses owner fixture state;
- protocol path depends on sponsor amount.

## 18.7 Sponsor mutations · `tab:guide12-exec:sponsor-vectors`

Required:

- sponsor/protocol reference overlap;
- two sponsor regions;
- sponsor change outside its role;
- ordinary L-BTC substituted for fee role;
- fee role substituted for sponsor change;
- foreign sponsor asset;
- sponsor member left unclassified;
- individual sponsor denomination changed while protocol projection is fixed;
- zero-valued sponsor member with exact role structure;
- balanced theft attempt;
- confidential sponsor values where candidate policy permits them;
- report attempts to publish sponsor amount or opening.

The expected result for a zero-valued ordinary sponsor member remains layered:

```text
semantic relation:
    may accept exact role structure

first-party explicit builder:
    omits known zero change

deployment policy:
    may reject as nonstandard
```

## 18.8 Root and projection mutations · `tab:guide12-exec:absence-vectors`

Required:

- add STATE input;
- add RESV input;
- add PACE input;
- add authority input;
- add root-shaped output;
- add burn record;
- add `tag-burn`;
- add clear destruction;
- add residue output;
- claim burn projection;
- claim clear projection;
- omit semantic transition certificate;
- introduce false certificate membership.

## 18.9 Representation mutations · `tab:guide12-exec:representation-vectors`

Required:

- private ASH under explicit-only policy;
- PublicCommitted ASH without authenticated opening;
- wrong amount encoding;
- noncanonical explicit amount;
- confidential `U`;
- representation changed without plan update;
- output representation differs from ABI;
- public fact available only in creator memory.

## 18.10 Constructor and linker mutations · `tab:guide12-exec:linker-vectors`

Required:

- missing coordinator program;
- missing member program;
- extra escape leaf;
- key-path escape;
- wrong internal key;
- wrong leaf version;
- wrong tree order;
- stale constructor from another candidate;
- unresolved symbol;
- ambiguous symbol;
- relocation omitted;
- relocation applied twice;
- wrong `U` asset substituted;
- candidate-bound relocation disagrees with ABI;
- unique relation carrier unreachable;
- source-order-dependent tree.

## 18.11 ABI mutations · `tab:guide12-exec:abi-vectors`

Required:

- family overlap;
- family gap;
- wrong total input count;
- wrong total output count;
- wrong transaction version;
- wrong sequence;
- witness item reorder;
- control path from another program;
- caller chooses target program;
- caller supplies successor amount;
- duplicate signing request;
- unexpected signature;
- target bytes changed after ABI validation;
- bundle paired with another ABI;
- raw transaction bypasses safe constructor.

## 18.12 Resource and infrastructure cases · `tab:guide12-exec:resource-vectors`

Required:

- program at candidate maximum;
- witness at candidate maximum;
- deepest candidate control path;
- maximum candidate sponsor shape;
- consensus acceptance with policy rejection;
- policy acceptance where claimed;
- executor timeout;
- malformed response;
- oversized record;
- unterminated record;
- wrong environment;
- wrong provenance;
- target infrastructure failure;
- resource prediction mismatch;
- infrastructure response carrying a target observation.

Infrastructure failure never counts as expected target rejection.

---

# 19. Relation-indexed coverage · `sec:guide12-exec:coverage`

For every compiler relation-case, require:

```text
relation identity
execution case
activation
selected proof
carrier
positive requirement
negative requirement
expected evidence boundary
expected target verdict where applicable
observed target verdict
accepted semantic projection where applicable
mutation layer
collateral relation closure
```

## 19.1 Positive coverage · `rule:guide12-exec:positive-coverage`

Positive runtime coverage requires:

- relation active;
- selected proof present;
- compatible carrier reachable;
- complete target transaction constructed;
- target accepted;
- semantic projection matched.

For compiler-static or backend-structural relations, positive coverage uses the appropriate typed structural evidence instead of inventing target execution.

## 19.2 Negative coverage · `rule:guide12-exec:negative-coverage`

Negative runtime coverage requires:

- valid source transaction;
- intended violated relation;
- exact changed fields;
- complete mutated target transaction;
- intended carrier executed;
- expected target rejection;
- observed target rejection;
- collateral relations recorded honestly.

A mutation affecting dependent relations is reported as:

```text
focused mutation with dependency collateral
```

not falsely described as isolated.

## 19.3 Inactive and structural relations · `rule:guide12-exec:inactive-structural`

Inactive relations retain explicit inactive-valid coverage.

Structural examples include:

```text
no roots in bundle
no specialized event role in ABI
no protocol signature in emitted leaves
clear lifecycle outstanding
```

Lifecycle incompleteness is a status, not a passing target case.

## 19.4 External evidence · `rule:guide12-exec:external-evidence`

Whole-transaction value conservation remains external target evidence.

The operation report names:

- exact target;
- exact development binding;
- exact transaction bytes;
- exact evidence role;
- target verdict;
- executor provenance.

A mock, abstract stack result, or model transaction does not satisfy the external target claim.

---

# 20. Resource analysis and candidate bounds · `sec:guide12-exec:resources`

## 20.1 Candidate bounds are not deployment calibration · `rule:guide12-exec:candidate-bounds`

Architecture owns:

```text
ASH_BATCH_MAX
FEE_SPONSOR_INPUT_MAX
```

Guide 12 evaluates candidate assignments and does not select final deployment values.

Each assignment is typed:

```rust
pub struct CandidateBoundAssignment {
    pub ash_batch_max: NonZeroU64,
    pub sponsor_input_max: u64,
}
```

and explicitly marked candidate-only.

## 20.2 Deterministic candidate enumeration · `rule:guide12-exec:bound-enumeration`

Initial research candidates may include:

```text
ASH:
    2, 4, 8, 16, 32, 64

sponsor:
    0, 1, 2, 4, 8, 16
```

These figures are research inputs, not accepted values.

Do not assume monotonicity when changing a bound changes:

- specialized leaf count;
- script size;
- tree depth;
- control path;
- ABI layout;
- transaction shape;
- policy result.

Enumerate the finite candidate set deterministically.

## 20.3 Complete-transaction measurement · `rule:guide12-exec:complete-measurement`

For each candidate measure:

- coordinator program bytes;
- member program bytes;
- push bytes;
- constructor bytes;
- leaf count;
- tree depth;
- control bytes;
- initial witness items;
- total witness bytes;
- peak main stack;
- peak alternate stack;
- maximum item;
- arithmetic operations;
- comparison operations;
- validation budget;
- complete transaction weight;
- consensus verdict;
- relay-policy verdict.

Script bytes must equal exact canonical program encoding length.

## 20.4 Separate objectives · `rule:guide12-exec:separate-objectives`

One fixture may maximize:

- transaction weight;
- witness bytes;
- peak stack;
- item width;
- validation budget;
- tree depth;
- policy pressure.

Do not claim one fixture maximizes every objective unless proved.

## 20.5 Prediction and observation · `rule:guide12-exec:resource-comparison`

Backend, linker, and ABI predictions are compared with real target observations.

A resource mismatch fails the resource report even if the transaction is accepted.

## 20.6 Phase-4 bound statement · `rule:guide12-exec:bound-result`

Phase 4 may record:

```text
candidate assignment N fits the tested candidate bundle and ABI
```

It must not record:

```text
production ASH_BATCH_MAX = N
```

Final calibration waits for the exact final bundle and ABI in the release phase.

---

# 21. Assurance boundaries · `sec:guide12-exec:assurance`

## 21.1 Compiler · `rem:guide12-exec:compiler-assurance`

Establishes:

- complete compact-ASH analysis;
- proof/source/constructibility/lifecycle/placement/layout/coverage requirements;
- deterministic target-operation projection.

Does not establish target behavior.

## 21.2 Target contract · `rem:guide12-exec:target-assurance`

Establishes:

- reviewed typed target facts;
- exact development binding;
- immutable evidence requirements.

Does not execute anything.

## 21.3 Tapscript · `rem:guide12-exec:tapscript-assurance`

Establishes:

- typed deterministic instruction construction;
- stack-valid candidate patterns;
- concrete relation placement;
- candidate resource formulas.

Does not establish linking, ABI correctness, or target acceptance.

## 21.4 Linker · `rem:guide12-exec:linker-assurance`

Establishes:

- exact symbol and relocation resolution;
- deterministic constructor and taptree;
- linked carrier reachability;
- linked resource formulas.

Does not construct complete transactions.

## 21.5 Transaction · `rem:guide12-exec:transaction-assurance`

Establishes:

- ABI-consistent target transaction construction;
- canonical role layout;
- target witness materialization;
- sponsor signing requests.

Does not establish target acceptance.

## 21.6 Vectors · `rem:guide12-exec:vectors-assurance`

Establishes:

- finite candidate-specific translation evidence;
- target result comparison;
- semantic projection comparison;
- relation coverage;
- resource measurements.

Does not prove universal compiler correctness or production readiness.

## 21.7 Executor · `rem:guide12-exec:executor-assurance`

Reports what one selected executable says one target environment did.

It does not establish:

- executor authenticity;
- target implementation correctness;
- production activation;
- evidence independence merely from different bytes;
- universal target behavior.

## 21.8 Phase-4 result · `rem:guide12-exec:phase4-assurance`

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

# 22. Implementation waves · `sec:guide12-exec:waves`

Each wave ends formatted, focused-test green, documented, and committed before the next begins.

## Wave 0 — Reproduce the Guide-12 review register · `task:guide12-exec:wave0`

**Deliverables**

- focused reproduction for `G12-R01` through `G12-R16`;
- CONFIRMED, REFUTED, or RECLASSIFIED disposition;
- no broad refactor;
- exact affected trust boundary per row;
- current Phase-3 and Guide-11 status reconciled.

**Suggested commit**

```text
plans: record the Guide-12 preflight findings
```

## Wave 1 — Close command, identity, and protocol foundations · `task:guide12-exec:wave1`

**Deliverables**

- invariant-bearing full revision type;
- validated anchor-set identity input;
- ADR-010 migration of shipped `emit-*` binaries;
- raw child stderr and argv-path omission;
- exact normalization and lifecycle row censuses;
- corrected script-error parsing;
- unified native protocol revision or explicit separate experimental protocols;
- corrected confidential nonce parity fact;
- evidence-registry documentation correction;
- supervised experimental runners or their retirement.

**Suggested commits**

```text
architecture: validate the attestation anchor identity input
target-conformance: unify the executor protocol and report censuses
target-elements: correct confidential nonce encoding facts
cli: bring target conformance emitters under ADR-010
plans: reconcile Guide-11 and Phase-3 status
```

## Wave 2 — Correct tapscript validation and resources · `task:guide12-exec:wave2`

**Deliverables**

- exact push-inclusive script-byte accounting;
- exact known-literal truth/equality reasoning or a narrowed abstract-execution claim;
- in-loop decoder work bound;
- infrastructure response observation closure;
- focused abstract/native oracle regressions;
- warning-free Rustdoc.

**Suggested commit**

```text
tapscript: close abstract execution and resource accounting
```

## Wave 3 — Revalidate and close Phase 3 · `task:guide12-exec:wave3`

**Deliverables**

- Phase-3 exit gate evaluated on current tree;
- exact target/native evidence state recorded;
- package contracts current;
- Phase-3 card current;
- backlog current;
- clean repository.

**Suggested commit**

```text
plans: record the Phase-3 exit
```

No later wave begins before this wave passes.

## Wave 4 — Public compiler target-operation plan · `task:guide12-exec:wave4`

**Deliverables**

- immutable validated compact-ASH target plan;
- exact relation/case/carrier/layout/coverage projections;
- explicit Guide-11 representation selection;
- no graph handles;
- independent assembly validator;
- public API tests.

**Suggested commit**

```text
compiler: expose the validated compact-ash target plan
```

## Wave 5 — Target transaction-form and dependency review · `task:guide12-exec:wave5`

**Deliverables**

- source review of fee-output, sponsorless, and sponsored transaction forms;
- selected Rust Elements transaction substrate or explicit first-party alternative;
- dependency and lockfile review;
- sponsor input authorization profile;
- exact target evidence requirements;
- no production activation claim.

**Suggested commit**

```text
target-elements: review the compact-ash transaction substrate
```

## Wave 6 — Static target assessment and proof patterns · `task:guide12-exec:wave6`

**Deliverables**

- exact target assessment of every operation requirement;
- shape-specialization policy;
- recognition pattern;
- cardinality pattern;
- canonical partition pattern;
- explicit aggregate pattern;
- coordinator/member patterns;
- sponsor isolation;
- root and projection absence;
- complete stack schedules;
- first operation-proven pattern IDs only where earned.

**Suggested commit**

```text
tapscript: implement compact-ash proof patterns
```

## Wave 7 — Candidate relocatable bundle · `task:guide12-exec:wave7`

**Deliverables**

- static ASH constructor;
- typed program roles;
- typed symbols and relocations;
- concrete placements;
- candidate shape set;
- witness roles;
- resource formulas;
- candidate-only status.

**Suggested commit**

```text
tapscript: emit the compact-ash candidate bundle
```

## Wave 8 — Linker foundation · `task:guide12-exec:wave8`

**Deliverables**

- linker package;
- two-pass symbol resolution;
- typed reference graph;
- cycle policy;
- structured relocation;
- deterministic taptree;
- exhaustive small-tree oracle;
- relation-carrier closure;
- `CandidateLinkedBundle`.

**Suggested commit**

```text
linker: link the compact-ash candidate bundle
```

## Wave 9 — Candidate transaction ABI · `task:guide12-exec:wave9`

**Deliverables**

- transaction package;
- candidate ABI;
- canonical input order;
- coordinator derivation;
- sponsor suffix;
- optional change;
- target fee role;
- synthetic test-ASH origin;
- public construction view;
- sponsor signing capability;
- exact target transaction bytes.

**Suggested commit**

```text
transaction: derive the compact-ash candidate ABI
```

## Wave 10 — Canonical semantic and target fixtures · `task:guide12-exec:wave10`

**Deliverables**

- vectors package;
- model/realization expectations;
- canonical operation evidence plan;
- target materialization;
- accepted projection;
- experimental versus evidence subject separation;
- exact case and relation censuses.

**Suggested commit**

```text
vectors: add compact-ash semantic and target fixtures
```

## Wave 11 — Real target execution · `task:guide12-exec:wave11`

**Deliverables**

- sponsorless consensus cases;
- sponsored policy-valid cases;
- strict typed operation protocol;
- exact environment and provenance binding;
- validated target reports;
- accepted semantic projection reports;
- deterministic bytes;
- zero required infrastructure errors.

**Suggested commit**

```text
vectors: execute compact-ash against the reviewed target
```

## Wave 12 — Negative relation coverage · `task:guide12-exec:wave12`

**Deliverables**

- every required focused mutation;
- construction-versus-target classification;
- collateral relation closure;
- exact relation-case coverage;
- no uncovered active relation-case;
- no expected value sent to executor.

**Suggested commit**

```text
vectors: complete compact-ash relation coverage
```

## Wave 13 — Candidate resource study · `task:guide12-exec:wave13`

**Deliverables**

- finite candidate-bound enumeration;
- complete transaction measurements;
- prediction/observation comparison;
- one useful Phase-4 demonstration candidate;
- explicit non-calibration result.

**Suggested commit**

```text
vectors: measure compact-ash candidate bounds
```

## Wave 14 — Phase-4 gate and handoff · `task:guide12-exec:wave14`

**Deliverables**

- package READMEs;
- package contracts;
- Phase-4 card;
- backlog gate record;
- identity and dependency impact;
- complete repository gate;
- clean final tree.

**Suggested commit**

```text
plans: record the compact-ash candidate pipeline
```

---

# 23. Focused verification · `sec:guide12-exec:verification`

## 23.1 Working Rust cadence · `rule:guide12-exec:rust-cadence`

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Focused runs supplement but do not replace the complete workspace lane.

## 23.2 Architecture · `tab:guide12-exec:architecture-tests`

```sh
cargo test --locked -p tripod-architecture
```

Focused filters:

```text
validated anchor-set input
anchor framing ambiguity
semantic hash unchanged where recipe unchanged
draft/release validation
deployment profile remains pre-release
```

## 23.3 Compiler · `tab:guide12-exec:compiler-tests`

```sh
cargo test --locked -p tripod-compiler
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-compiler --no-deps
```

Focused filters:

```text
target operation plan
relation census
carrier census
layout census
coverage census
representation policy
owner revalidation
projection determinism
declaration permutation
no graph handles
no digest fields
```

## 23.4 Target contract · `tab:guide12-exec:target-tests`

```sh
cargo test --locked -p tripod-target-elements
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-target-elements --no-deps
```

Focused filters:

```text
confidential nonce parity
value and asset introspection
signature and sighash profile
fee-output target facts
capability closure
evidence requirements
reviewed trust state
```

## 23.5 Tapscript · `tab:guide12-exec:tapscript-tests`

```sh
cargo test --locked -p tripod-tapscript
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-tapscript --no-deps
```

Focused filters:

```text
exact pushed literal truth
exact pushed literal equality
decoder work bound
push-inclusive script bytes
recognition pattern
family cardinality
canonical partition
checked aggregate arithmetic
coordinator/member schedules
sponsor opacity
root/projection absence
typed emission
parser round trip
resource formulas
candidate status
```

Required exact-literal regressions:

```text
push [] ; VERIFY
push 00 ; VERIFY
push 80 ; VERIFY
push 01 ; VERIFY
push 01 ; push 02 ; EQUALVERIFY
push 01 ; push 01 ; EQUALVERIFY
```

## 23.6 Linker · `tab:guide12-exec:linker-tests`

```sh
cargo test --locked -p tripod-linker
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-linker --no-deps
```

Focused filters:

```text
symbol census
two-pass resolution
unknown and ambiguous references
relocation
taptree determinism
constructor assembly
carrier closure
resource resolution
candidate/final separation
```

## 23.7 Transaction · `tab:guide12-exec:transaction-tests`

```sh
cargo test --locked -p tripod-transaction
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-transaction --no-deps
```

Focused filters:

```text
candidate ABI
canonical input ordering
coordinator derivation
shape selection
sponsor suffix
optional change
fee-output role
successor amount
constructor instantiation
witness order
signature commitment
public permissionless construction
synthetic ASH non-claims
```

## 23.8 Vectors · `tab:guide12-exec:vectors-tests`

```sh
cargo test --locked -p tripod-vectors
RUSTDOCFLAGS='-D warnings' \
  cargo doc --locked -p tripod-vectors --no-deps
```

Focused filters:

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

## 23.9 Executor and protocol · `tab:guide12-exec:executor-tests`

```sh
cargo test --locked -p tripod-target-elements-conformance
```

Focused filters:

```text
protocol cross-language round trip
bounded records
blank record refusal
unknown field refusal
request expectation exclusion
target/deployment transcript binding
environment recheck
full-width expected provenance
infrastructure observation exclusion
process-group startup cleanup
timeout cleanup
experimental-runner cleanup
child stderr omission
```

## 23.10 Existing semantics · `tab:guide12-exec:semantic-tests`

```sh
cargo test --locked -p tripod-model
cargo test --locked -p tripod-realization
```

No backend change may alter model acceptance or realization semantics.

## 23.11 Documentation and census · `tab:guide12-exec:documentation-tests`

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new tracked file enters its nearest Meson census in the same commit.

---

# 24. Full batch gate · `gate:guide12-exec:batch`

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
compact-ASH sponsorless matrix
compact-ASH sponsored matrix
compact-ASH relation-mutation matrix
compact-ASH constructor/linker matrix
compact-ASH ABI matrix
compact-ASH resource-candidate matrix
```

Record for every real run:

```text
target contract revision
development binding
network and genesis
executor declaration
adapter version
framework revision
binary-reported node revision
intended tip
upstream base
local topic census
candidate bundle subject
candidate ABI subject
candidate bound assignment
case count
relation count
claim count
failures
infrastructure errors
report byte reproducibility
```

If the dependency graph changed:

```sh
cargo tree --locked -e features
cargo metadata --locked
cargo audit
```

A missing advisory tool is recorded as skipped, never passed.

Run document reproducibility when document inputs changed or batch policy requires it:

```sh
scripts/check-document-reproducibility.sh
```

Final check:

```sh
git status --porcelain=v1 --untracked-files=all
```

The result must be empty.

---

# 25. Acceptance criteria · `sec:guide12-exec:acceptance`

Accept the Phase-4 compact-ASH candidate only when:

- Phase 3 is closed on the current tree;
- every confirmed Guide-12 P0/P1 finding is closed;
- compiler target-operation scope is exactly compact ASH;
- relation census is complete;
- every target requirement is assessed;
- every selected proof is realization-approved;
- every active relation-case has a reachable carrier;
- explicit ASH follows Guide 11;
- no protocol owner or operator secret exists;
- exact `U` and constructor recognition hold;
- exact family counts and ranges hold;
- exactly one successor ASH exists;
- exact aggregate `U` conservation holds;
- canonical source/destination partition holds;
- no issuance or destruction exists;
- sponsor membership is exact and amount-opaque;
- sponsor input spending conditions authorize the finalized transaction under the admitted profile;
- no root participates;
- no specialized event is emitted;
- linked constructor and programs are deterministic;
- every symbol and mandatory relocation resolves exactly once;
- candidate ABI is deterministic;
- complete valid target transactions accept;
- every accepted semantic projection matches;
- every required mutation fails at its owning boundary;
- relation-case coverage is exact;
- predicted and observed resources agree;
- script-byte prediction includes pushes exactly;
- at least one useful candidate bound is demonstrated;
- candidate bounds remain explicitly non-final;
- clear remains an explicit outstanding lifecycle;
- no speculative digest is minted;
- reports reproduce byte-for-byte;
- full repository gates pass;
- final tree is clean.

---

# 26. Rejection criteria · `sec:guide12-exec:rejection`

Reject the candidate if:

- one semantic relation disappears between packages;
- compiler public output exposes graph handles or target positions;
- a target capability is treated as a complete proof pattern;
- a required relation has no carrier;
- a carrier is reachable only in the wrong execution case;
- public ASH needs owner-private state;
- wrong asset or constructor passes;
- aggregate balance can hide a wrong `U` flow;
- sponsor amount is read as protocol data;
- sponsor positivity substitutes for protocol correctness;
- hidden protocol authorization appears;
- root or specialized event participation is possible;
- the transaction builder is the only place enforcing a target-required relation;
- an ad hoc fixture can become evidence;
- construction failure is counted as target rejection for a script claim;
- accepted target projection differs from model semantics;
- a target report lacks exact environment or provenance;
- a mock satisfies the gate;
- protocol implementations disagree under one revision;
- an infrastructure result carries target observations;
- resource prediction omits push bytes;
- abstract validation reports impossible target liveness;
- target bytes differ from the bytes actually tested;
- candidate and final states are conflated;
- a report digest is introduced without an admitted consumer;
- any full gate fails or leaves the repository dirty.

---

# 27. Identity, schema, and dependency impact · `sec:guide12-exec:impact`

Expected identity impact:

```text
Attestation version:
    unchanged

realization version:
    unchanged

architecture schema:
    unchanged

architecture semantic hash:
    unchanged, unless a semantic correction independently requires movement

architecture behavioural hash:
    unchanged

anchor-set hash:
    unchanged

realization identity:
    none minted

compiler identity:
    none minted

target digest:
    none minted

relocatable-bundle digest:
    none minted

linked-bundle digest:
    none minted for in-process candidate use

transaction-ABI digest:
    none minted for in-process candidate use

vector/report digest:
    none minted

deployment-profile identity:
    remains dormant and not production-release-valid
```

Expected typed schema work:

```text
compiler target-operation projection
candidate relocatable bundle
candidate linked bundle
candidate transaction ABI
canonical compact-ASH evidence plan
operation report types
executor protocol revision, if workloads are unified
```

Expected dependency work:

```text
new first-party linker package
new first-party transaction package
new first-party vectors package

possible vectors → target-elements-conformance edge
or one extracted target-executor package

possible reviewed Rust Elements transaction dependency
only after explicit ADR-011 review
```

Every schema migration is explicit. Earlier protocol/report revisions remain historical and are not silently widened.

---

# 28. Final Phase-4 result matrix · `tab:guide12-exec:result`

The completion record fills every final column.

| Boundary | Required result | Actual result |
|---|---|---|
| preflight | Guide-12 review register closed | |
| Phase 3 | exit gate passed on current tree | |
| realization | compact-ASH semantic relation complete | |
| compiler | validated target-operation plan | |
| target assessment | every capability and evidence role classified | |
| tapscript | typed operation patterns and programs | |
| constructor | deterministic static candidate ASH constructor | |
| linker | deterministic candidate linked bundle | |
| transaction | deterministic candidate ABI and complete transactions | |
| target execution | positive cases accepted, negative cases rejected | |
| semantic projection | every accepted target transaction matches | |
| coverage | every relation-case complete | |
| resources | prediction equals observation | |
| candidate bounds | useful candidate demonstrated, not calibrated | |
| lifecycle | clear explicitly outstanding | |
| identity | no speculative digest | |
| deployment release | not claimed | |

---

# 29. Guide-12 exit checklist · `gate:guide12-exec:exit`

## Preflight

- [ ] `G12-R01` through `G12-R16` reproduced and dispositioned;
- [ ] every confirmed P0/P1 finding is closed;
- [ ] protocol revisioning is coherent;
- [ ] shipped commands comply with ADR-010;
- [ ] target review facts are correct;
- [ ] exact resource accounting includes pushes;
- [ ] exact-literal abstract execution is sound for the claims consumers make;
- [ ] executor and runner cleanup is bounded;
- [ ] Guide-11 and Phase-3 status is reconciled;
- [ ] Phase-3 exit gate passes.

## Compiler boundary

- [ ] validated target-operation plan has no unchecked constructor;
- [ ] complete analyzed validation runs before publication;
- [ ] compact-ASH scope is exact;
- [ ] relation census is exact;
- [ ] carrier census is exact;
- [ ] layout census is exact;
- [ ] coverage census is exact;
- [ ] explicit representation policy is retained;
- [ ] no graph index is public;
- [ ] no target opcode or position enters compiler core;
- [ ] no compiler digest is minted without a consumer.

## Target and patterns

- [ ] reviewed target validates;
- [ ] nonce and point-encoding conventions are correct;
- [ ] every required capability is assessed;
- [ ] every selected pattern names its prerequisites;
- [ ] every arithmetic success flag is checked;
- [ ] permissionless leaves contain no protocol signature;
- [ ] sponsor authorization profile is explicit;
- [ ] explicit ASH follows Guide 11;
- [ ] confidential `U` rejects;
- [ ] abstract success/failure states validate;
- [ ] exact pushed-literal truth and equality cases pass;
- [ ] parser work limit is enforced during parsing;
- [ ] script-byte accounting equals encoded length;
- [ ] parser round-trip holds.

## Relocatable and linked bundle

- [ ] every symbol is typed;
- [ ] definitions are unique;
- [ ] references resolve in two passes;
- [ ] mandatory relocations resolve exactly once;
- [ ] no overlapping or variable-width byte relocation exists;
- [ ] constructor is deterministic;
- [ ] internal-key policy is explicit;
- [ ] taptree is deterministic;
- [ ] exhaustive small-tree oracle agrees;
- [ ] every required carrier remains reachable;
- [ ] bundle is explicitly candidate-only;
- [ ] clear lifecycle remains explicit.

## ABI and transactions

- [ ] input family order is canonical;
- [ ] coordinator derives from canonical ASH order;
- [ ] finite shape specialization is explicit;
- [ ] sponsor suffix is exact;
- [ ] output roles are exact;
- [ ] fee role cannot satisfy sponsor change;
- [ ] sponsor change cannot satisfy fee role;
- [ ] successor amount derives exactly;
- [ ] caller cannot choose protected outputs or programs;
- [ ] signatures are requested after output finalization;
- [ ] witness and control ordering is canonical;
- [ ] sponsor amounts remain outside protocol predicates and reports;
- [ ] synthetic ASH is labeled test-only;
- [ ] candidate ABI is not final.

## Evidence

- [ ] arbitrary fixtures cannot become gate-eligible;
- [ ] canonical evidence plan derives from compiler coverage;
- [ ] expected semantics derive independently of target emission;
- [ ] every positive target transaction is complete;
- [ ] every accepted projection matches;
- [ ] every negative relation reaches its owning evidence layer;
- [ ] construction and target rejection remain distinct;
- [ ] whole-transaction conservation remains explicit evidence;
- [ ] target environment matches binding;
- [ ] executor provenance satisfies ADR-018;
- [ ] no mock satisfies the gate;
- [ ] no required case has infrastructure failure;
- [ ] non-target outcomes carry no target observations;
- [ ] protocol records are strict and bounded;
- [ ] report bytes reproduce;
- [ ] no report digest is minted.

## Resources

- [ ] coordinator program measured;
- [ ] member program measured;
- [ ] every push byte counted;
- [ ] constructor measured;
- [ ] complete sponsorless transaction measured;
- [ ] complete sponsored transaction measured;
- [ ] candidate shape matrix measured;
- [ ] prediction equals observation;
- [ ] consensus and policy verdicts remain separate;
- [ ] selected bound is candidate-only;
- [ ] no final calibration claim is made.

## Repository

- [ ] package READMEs are current;
- [ ] package contracts are current;
- [ ] Phase-4 card is current;
- [ ] backlog state is current;
- [ ] every new file is in the Meson census;
- [ ] dependency review is recorded;
- [ ] identity and schema impact is recorded;
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

# 30. Completion report template · `sec:guide12-exec:completion-report`

```text
Guide 12 result
===============

Starting state:
    source revision:
    working tree:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    realization version:
    target contract revision:
    native protocol revision:
    Guide-11 result:
    Phase-3 gate:
    compiler compact-ASH projection:

Preflight:
    G12-R01 provenance invariant:
    G12-R02 anchor-set identity:
    G12-R03 CLI contract:
    G12-R04 child diagnostic boundary:
    G12-R05 normalization census:
    G12-R06 lifecycle pass census:
    G12-R07 script-error parser:
    G12-R08 planning status:
    G12-R09 protocol schema:
    G12-R10 nonce parity:
    G12-R11 script-byte resources:
    G12-R12 exact-literal abstraction:
    G12-R13 parser work bound:
    G12-R14 infrastructure observations:
    G12-R15 runner supervision:
    G12-R16 evidence documentation:

Compiler boundary:
    public target-operation type:
    scope:
    representation policy:
    relations:
    execution cases:
    abstract carriers:
    layout requirements:
    coverage requirements:
    capabilities:
    external evidence:
    lifecycle:
    graph handles exposed:
    identity minted:

Target assessment:
    reviewed target:
    deployment binding:
    missing primitives:
    unsupported capabilities:
    backend patterns required:
    structural obligations:
    external evidence:
    sponsor authorization profile:
    transaction-form review:
    selected representation:

Tapscript patterns:
    object recognition:
    family cardinality:
    closed-asset closure:
    canonical partition:
    aggregate arithmetic:
    member participation:
    coordinator:
    sponsor isolation:
    root absence:
    projection absence:
    exact-literal result:
    parser result:
    program roles:
    stack result:
    candidate status:

Relocatable bundle:
    programs:
    shapes:
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

Transaction substrate:
    Rust dependency:
    version:
    features:
    licence:
    MSRV:
    unsafe/FFI:
    advisories:
    lockfile impact:

Transaction ABI:
    input layout:
    output layout:
    coordinator rule:
    shape specialization:
    sponsor suffix:
    sponsor change:
    fee role:
    transaction version:
    sequence:
    witness order:
    control-path roles:
    request fields:
    candidate bounds:
    candidate status:

Synthetic ASH:
    test asset:
    issuance/funding method:
    constructor:
    values:
    test-only statement:
    burn-lineage non-claim:
    attestation non-claim:

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
    protocol schema:
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
    push bytes included:
    constructor bytes:
    leaf count:
    tree depth:
    control bytes:
    witness bytes:
    peak stack:
    peak altstack:
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
        candidate implemented
    clear:
        outstanding
    release-complete:
        false

Identity impact:
    Attestation:
    realization:
    architecture schema:
    architecture semantic hash:
    architecture behavioural hash:
    anchor-set hash:
    compiler identity:
        none
    target-plan identity:
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
    protocol cross-language:
    Rustdoc:
    scripts/check-plans.sh:
    meson compile -C build lint:
    scripts/ci.sh:
    meson compile -C build:
    meson test -C build --print-errorlogs:
    real compact-ASH matrices:
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

# 31. Handoff after Guide 12 · `sec:guide12-exec:handoff`

If Guide 12 succeeds, Phase 4 has one complete candidate compiler-to-target operation.

The next execution guide should implement live receipt transfer:

```text
Guide 13 — End-to-End Live Receipt Transfer
```

Guide 13 consumes rather than reopens:

- validated compiler target-operation boundary;
- exact target assessment;
- typed tapscript pattern interface;
- relocatable bundle interface;
- linker symbol and relocation model;
- deterministic taptree policy;
- candidate transaction ABI framework;
- operation evidence framework;
- strict executor protocol;
- target environment and provenance binding;
- canonical report trust states;
- Guide-11 representation policy.

Guide 13 adds:

- protocol owner authorization;
- multiple protocol outputs;
- split and merge;
- output-committing signatures;
- explicit and private committed value alternatives;
- separate safety and minimality evidence;
- future burn and redemption lifecycle obligations.

Guide 12 itself makes none of those owner/value-representation claims beyond what compact ASH requires.

---

## Closing statement · `rem:guide12-exec:closing`

> Compact ASH is the first complete pipeline test because its semantic relation is small and its implementation boundary is not. Guide 12 succeeds only when exact typed semantics, compiler analysis, target requirements, emitted programs, linked carriers, transaction layout, real target execution, accepted semantic projection, relation-indexed evidence, and resource measurements all describe the same operation. A green script, a green node, or a green report alone is not that result; the welded chain is.
