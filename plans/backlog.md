# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 1 — typed realization foundation
> **Current condition:** every Phase-1 correctness, build-graph, CLI-contract, and documentation remediation finding is closed; only the completion-evidence gate remains — the planning-status reconciliation (F1-016), the evidence-tag ceremony (F1-033), and the recorded gate run itself (F1-017 / R1-013)
> **Next gate:** Phase 2 — target-independent compiler analysis
> **Authority:** Current execution queue only; normative specifications, typed architecture, implemented ADRs, accepted decisions, package contracts, research results, phase cards, and the roadmap take precedence

This backlog contains current release-blocking work and the immediately following
compiler/linker preparation lane.

It does not duplicate:

- normative protocol or realization semantics;
- complete package contracts;
- phase deliverables already owned by [`phases/`](phases/README.md);
- research arguments already owned by [`research/`](research/README.md);
- historical baseline identities already owned by
  [`phases/00-baseline.md`](phases/00-baseline.md);
- generated identities from authoritative artifacts;
- implementation diaries preserved by Git history.

Long-term sequencing is owned by [`roadmap.md`](roadmap.md). Cross-package
choices are owned by [`decisions/`](decisions/README.md), package boundaries by
[`packages/`](packages/README.md), and unresolved prototypes by
[`research/`](research/README.md).

---

## 1. Backlog contract · `sec:backlog:contract`

### 1.1 Status vocabulary · `tbl:backlog:status`

| Status | Meaning |
|---|---|
| **TODO** | Ready when named dependencies are complete. |
| **IN PROGRESS** | Actively being implemented, reviewed, or verified. |
| **BLOCKED** | A named dependency prevents safe progress. |
| **DONE** | Implementation, focused tests, documentation, and required evidence are complete. |
| **DROPPED** | Deliberately not implemented; rationale and replacement are recorded. |
| **SUPERSEDED** | Replaced by a named task, decision, ADR, or contract. |

Code resembling an intended result is not sufficient for `DONE`.

A gate is complete only when its required commands have run in the required
environments, every required lane passed, and the repository is clean under the
complete status check.

### 1.2 Priority vocabulary · `tbl:backlog:priority`

| Priority | Meaning |
|---|---|
| **P0** | Safety or assurance-boundary defect capable of manufacturing false semantic evidence. |
| **P1** | Phase-gate blocker: model/realization disagreement, invalid theorem, or broken build/output contract. |
| **P2** | Required correctness, determinism, policy, or documentation closure before Phase-1 exit. |
| **P3** | Maintainability or planning defect that must close before the gate but follows correctness work. |
| **POST** | Work explicitly after Phase 1; it does not block the Phase-1 gate. |

### 1.3 Task families · `tbl:backlog:families`

| Prefix | Owner |
|---|---|
| `F1` | Phase-1 audit and remediation |
| `R1` | Typed realization foundation |
| `C1` | Post-Phase-1 compiler/linker algorithm preparation |
| `Q` | Prototype or research dependency |

### 1.4 Definition of done · `rule:backlog:done`

An implementation task is `DONE` only when it records:

1. implementing source files;
2. positive tests;
3. focused negative, mutation, property, or integration tests;
4. affected documentation, contracts, decisions, or ADRs;
5. exact verification commands and results;
6. generated-artifact and label-register impact;
7. semantic, identity, schema, and versioning impact;
8. dependency and feature impact;
9. confirmation that checks created no staged, unstaged, or untracked
   nonignored change;
10. any residual limitation deliberately retained.

A research task is `DONE` only when it records:

1. the precise question;
2. exact dependency, tool, and target versions;
3. positive and negative prototype evidence;
4. complexity and resource measurements;
5. accepted and rejected candidates;
6. result and decision handoff;
7. permanent production tests;
8. assurance class and remaining trust boundary.

A finding may close through:

- implementation plus a focused regression;
- a typed proof that the reported state is unconstructible;
- an approved trust-boundary change in the owning ADR or decision.

“Existing tests pass” is not closure unless a named test reaches the reported
path.

### 1.5 Label and authority rule · `rule:backlog:authority`

Planning labels are non-normative and non-identity-bearing.

Under ADR-013:

- each PLAN mint is unique;
- each same-owner citation resolves;
- cross-owner citations use explicit prefixes;
- generated registers do not participate in the source graph.

Planning labels never become compiler, linker, ABI, deployment, evidence, or
release identity.

---

## 2. Current repository state · `sec:backlog:state`

### 2.1 Implemented foundation · `tbl:backlog:implemented`

| Package or policy | Current state |
|---|---|
| Layer 0 | Published specification |
| Realization | Published and architecture-welded |
| `architecture` | Final typed architecture, canonical hashes, deployment-profile validation |
| `model` | Executable state machine, invariant checker, property/corruption suites, indexer and audit projections |
| `realization` | Target-independent typed semantics for compact ASH and live receipt transfer |
| `artifacts` | Generated-publication writer/checker and realization-document weld |
| `labels` | Owner-aware Markdown/Rust label graph and deterministic registers |
| `cli-common` | Shared ADR-010 infrastructure and current stamp support |
| `document-stamps` | Git-derived paper metadata |
| `execwrap` | Byte-preserving process wrapper; TeX simulation isolated in a separate `execwrap-mock-tex` binary (F1-027) |
| `flatten-latex-main` | Deterministic atomic LaTeX flattener |
| ADR-010 | Implemented in Rust commands; boundary corrections landed (F1-015 output classes, F1-024 subprocess/TTY coverage, F1-031/F1-032 caller-controlled-text omission) |
| ADR-011 | Implemented; clean-tree gate correction landed (F1-010) |
| ADR-013 | Implemented; malformed Realization import correction landed (F1-008) |
| ADR-014 | Implemented; stamp/restat repair landed (F1-005), shell-checker no-op correction landed (F1-026) |
| D007 | Accepted; active planning cleanup landed (F1-013) |
| D008 | Accepted |

### 2.2 Not implemented · `tbl:backlog:not-implemented`

```text
compiler
target-elements
tapscript
linker
transaction
vectors
release
independent deployment observers
production deployment
```

Architecture finality does not imply deployment readiness.

A green model does not prove target correctness. A self-consistent event cache
does not prove chain provenance. A generated artifact does not become semantic
source.

### 2.3 Readiness statement · `rem:backlog:readiness`

```text
Attestation specification:             published
Realization contract:                 published
Typed architecture:                   final and pinned
Executable model:                     implemented
Typed realization pilots:             implemented; remediation landed
Phase-1 implementation foundation:    present
Phase-1 correctness findings:         closed
Phase-1 evidence gate:                awaiting its recorded run
Post-Phase-1 compiler preparation:     partially prepared
Target/backend/linker:                 not implemented
Independent deployment evidence:      absent
Deployment release:                   absent
```

---

## 3. Phase-1 dependency and priority tree · `fig:backlog:phase1-tree`

The Phase-1 gate and post-Phase-1 work are separate. No `C1` task is allowed to
masquerade as a prerequisite for fixing current Phase-1 correctness.

```text
PHASE-1 SAFETY ROOT
│
├── P0 provenance
│   └── F1-001 checkpoint/indexer provenance boundary
│
├── P1 semantic conformance
│   ├── F1-002 exact mixed canonical flows
│   │   ├── F1-003 exact compact-ASH weld
│   │   ├── F1-004 one sponsor envelope
│   │   └── F1-018 positive ordinary L-BTC rule
│   ├── F1-006 cap-aware quiescence
│   ├── F1-011 architecture-derived constructibility
│   ├── F1-021 checked exact-rational reduction
│   ├── F1-023 checkpoint/reorg monotonicity decision
│   └── F1-025 exact relation-identity vocabulary
│
├── P1/P2 output and build contracts
│   ├── F1-020 checker report/stamp transaction design
│   │   └── F1-007 checker stamp implementation
│   ├── F1-005 paper-stamp restat repair
│   │   ├── F1-019 all-or-nothing render arguments
│   │   └── F1-026 no-op/stamp semantics for shell checkers
│   ├── F1-009 coherent Git revision stamping
│   ├── F1-010 complete clean-tree verification
│   ├── F1-014 remove or declare jq
│   ├── F1-015 classify shell/Python checker output
│   ├── F1-022 resolve execwrap exit semantics
│   ├── F1-027 isolate mock TeX execution from the production wrapper
│   └── F1-024 real subprocess coverage for every binary
│
├── P2 output/diagnostic contracts
│   ├── F1-031 caller-controlled execwrap diagnostics
│   ├── F1-032 external Git stderr diagnostics
│   └── F1-034 staged multi-output document stamps
│
├── P2 documentation and graph hygiene
│   ├── F1-008 exhaustive malformed-import diagnostics
│   ├── F1-012 normative and companion corrections
│   ├── F1-013 remove D007-superseded adapter planning
│   └── F1-016 reconcile status and completion evidence
│
└── GATE
    └── F1-033 immutable evidence-tag ceremony
        └── F1-017 complete Phase-1 evidence
            └── R1-013 Phase-1 exit
```

Every branch above `F1-017` must close. A task may proceed in parallel when it
does not depend on an unresolved semantic decision.

---

## 4. Phase-1 remediation register · `sec:backlog:phase1-register`

### 4.1 Summary · `tbl:backlog:phase1-findings`

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `F1-001` | P0 | **DONE** | Public checkpoint reconstruction can create credited events without chain provenance. |
| `F1-002` | P1 | **DONE** | Realization cannot represent exact mixed movement-plus-destruction flows. |
| `F1-003` | P1 | **DONE** | Compact-ASH architecture/realization weld is incomplete. |
| `F1-004` | P1 | **DONE** | Realization accepts multiple generic sponsor envelopes. |
| `F1-005` | P1 | **DONE** | Paper stamp success output defeats claimed Ninja restat behavior. |
| `F1-006` | P1 | **DONE** | Sponsored quiescence ignores active-backing-cap-blocked requests. |
| `F1-007` | P2 | **DONE** | Checker stamps can be updated despite command failure. |
| `F1-008` | P2 | **DONE** | Malformed non-PA Realization imports may disappear silently. |
| `F1-009` | P2 | **DONE** | Arbitrary `tree_ref` can mix selected-ref metadata with worktree bytes. |
| `F1-010` | P2 | **DONE** | CI cleanliness misses staged and untracked nonignored files. |
| `F1-011` | P2 | **DONE** | Constructibility authorization is selected by operation name rather than architecture semantics. |
| `F1-012` | P2 | **DONE** | Normative and companion prose contain arithmetic, weld, and valuation inaccuracies. |
| `F1-013` | P2 | **DONE** | Active planning still prescribes graph adapters prohibited by D007. |
| `F1-014` | P2 | **DONE** | Full Meson tests depend on undeclared `jq`. |
| `F1-015` | P3 | **DONE** | Shell/Python checker streams conflict with broad ADR-010/014 wording. |
| `F1-016` | P3 | **TODO** | Planning statuses and completion evidence disagree. |
| `F1-017` | Gate | **BLOCKED** | Complete Phase-1 gate has not been run and recorded. |
| `F1-018` | P1 | **DONE** | Realization accepts zero-valued ordinary L-BTC that the model rejects. |
| `F1-019` | P2 | **DONE** | Partial `attestation-stamps` render arguments silently select JSON mode. |
| `F1-020` | P1 | **DONE** | One command cannot atomically commit stdout success and a filesystem stamp. |
| `F1-021` | P2 | **DONE** | Public query reduction can produce an invalid zero-denominator rational. |
| `F1-022` | P2 | **DONE** | `execwrap` child-status propagation contradicts the documented global exit classes. |
| `F1-023` | P1 | **DONE** | Reorg reprojection may increase valuation despite downward-only Layer-0 wording. |
| `F1-024` | P2 | **DONE** | ADR-010 subprocess coverage is incomplete across shipped binaries. |
| `F1-025` | P2 | **DONE** | Relation IDs reuse unrelated relation kinds and weaken semantic identity clarity. |
| `F1-026` | P2 | **DONE** | Shell checker stamps and always-stale wiring violate no-op and touch-only claims. |
| `F1-027` | P2 | **DONE** | Production `execwrap` exposes hidden mock flags that fabricate TeX outputs. |
| `F1-028` | P3 | **DONE** | Maintenance-potential report counts used unchecked `as u64` narrowing. |
| `F1-029` | P3 | **DONE** | Positive scenario tests use bare `apply` + manual invariant check, blurring evidence class. |
| `F1-030` | P2 | **DONE** | Document stamp inputs are not fully canonical or committed-blob-bound. |
| `F1-031` | P2 | **DONE** | `execwrap` logs caller-controlled child program text despite the raw-argv prohibition. |
| `F1-032` | P2 | **DONE** | `census-audit` logs raw stderr from an argument-supplied external Git program. |
| `F1-033` | P3 | **TODO** | Phase completion evidence lacks a non-self-referential commit/tag ceremony. |
| `F1-034` | P2 | **DONE** | Document-stamp render can partially publish its two real outputs after a late failure. |

---

## 5. Provenance and semantic-conformance work · `sec:backlog:semantic-work`

### F1-001 — Close the checkpoint provenance boundary

**Priority:** P0
**Owners:** `model`, future `vectors`/`release`
**Primary files:** `packages/model/src/ledger.rs`, indexer tests
**Imports:** (`[RZ-sec:ledger:authentication]`),
(`[RZ-obl:oracle:ledger]`)

#### Problem

`IndexerCheckpoint` is publicly constructible. Its conversion to
`ReferenceIndexer` validates event-table consistency, context shape, ordering,
and payload domains, but not canonical-chain provenance.

A caller can construct:

```text
valid context
+ positive genesis clear
+ invented burn row
+ matching event row
```

and obtain credited attestation terms.

The current public conversion establishes an internally consistent recognized
event cache, not an authenticated index.

#### Required boundary

Choose one:

**A. Provenance-validating reconstruction**

- receive validated canonical chain material;
- derive block membership, transaction identity, operation/event type, burn
  input/output shape, fresh ASH value, clear STATE transition, and checkpoint
  membership;
- compare stored rows against the independently derived event sequence.

**B. Explicit trusted cache**

- rename the type to expose that it is an unverified/trusted cache snapshot;
- make arbitrary construction or conversion crate-private;
- prevent caller-authored rows from producing any type named a validated or
  reference indexer;
- bind persisted cache reuse to a prior validated event report.

The deployment-facing path ultimately requires option A or equivalent
independent event evidence.

#### Required regressions

- invented burn;
- invented clear;
- invented event with plausible block hash;
- event absent from canonical chain;
- free-standing burn payload;
- compacted ASH presented as burn;
- valid provenance-derived reconstruction;
- valid trusted-cache round trip under the explicitly named trust class.

#### Exit

- [x] no public untrusted row-only path manufactures attestation credit;
- [x] consistency and provenance are separate typed claims;
- [x] event and query evidence remain separate;
- [x] public names and documentation expose the trust boundary;
- [x] schema and report identity impact are recorded.

#### Evidence · DONE

Adopted boundary **B** (explicit trusted cache), the Phase-1 decision.

- Commit `ed6b33e`. `IndexerCheckpoint` becomes `ModelIndexerCheckpoint`
  with private fields, obtainable only from an existing `ReferenceIndexer`.
  The public `TryFrom<IndexerCheckpoint> for ReferenceIndexer` that promoted
  arbitrary rows is removed; the opaque cache exposes read-only
  `context`/`query`/`event_snapshot` over a private infallible `restore`. A
  `compile_fail` doctest pins that arbitrary rows cannot be promoted.
  `IndexerSnapshot` becomes `IndexerDiagnosticSnapshot`, documented as an
  untrusted diagnostic projection with no path back to a query-capable index.
  A test-only `UntrustedIndexerFixture` (whose `check` returns a consistency
  result, not an indexer) replaces the checkpoint field-mutation tests.
- Commit `7a6b1a0`. `from_model_history` becomes crate-private
  `from_assumed_kernel_history`, named for the source and the assumption; a
  `compile_fail` doctest proves external code cannot reach it. The projection
  path and its private payload validators carry
  `cfg_attr(not(test), allow(dead_code))` pending a future safe adapter.
- Docs. Realization §12.2 now separates three roles explicitly: the trusted
  reference-model projection (expected result), the independent deployment
  event derivation required by O6 (the only provenance authenticator), and the
  hash-bound checkpoint (a consistency cache, never event evidence).
- Residual: provenance-validating reconstruction (boundary A) and independent
  target-chain recognition remain future target/vector work, as designed.
- Verified: full model suite (277), both `compile_fail` doctests, model clippy
  `-D warnings`, and the Meson labels/generated lanes green via the flatpak SDK.

---

### F1-002 — Preserve exact mixed canonical flows

**Priority:** P1
**Owners:** `model`, `realization`
**Primary files:** model certificate/conformance projection; realization
observation/evaluation

#### Problem

The model’s exact flow permits:

\[
\sum \text{sources}=\sum \text{destinations}+\sum \text{destruction legs}.
\]

Partial clear and terminal settlement can contain both movement and destruction
in one flow.

Certificate derivation flattens the flow into delta-family rows that share
sources. Realization rejects repeated source references and also expects a
destruction row’s complete source value to equal only its destruction amount.

#### Required design

Prefer one first-class observed exact-flow value:

```text
asset
source references
destination references
optional movement kind
destruction legs
```

Validate the exact equation once per flow.

Enforce source/destination uniqueness between exact flows.

Derive the manifest’s active delta-family set separately from exact accounting.

If flattened rows remain, add a stable flow-group identity and regroup before
partition and arithmetic validation.

#### Required regressions

- partial clear with residual ASH;
- full clear;
- continuing settlement;
- terminal settlement with receipts and residue;
- terminal residue-only settlement;
- one source used by two independent flows;
- one destination funded by two flows;
- active delta-family set still equals the manifest.

#### Exit

- [x] every model-valid mixed flow projects into realization;
- [x] no exact accounting constraint is weakened;
- [x] family projection and flow partition are distinct;
- [x] both existing pilots retain their current denotation;
- [x] identity/schema impact is recorded.

#### Evidence · DONE

Adopted the first-class exact-flow value on both sides; no flattened
representation is stored.

- Commit `67ae17b` (model). The certificate stores
  `CertifiedCanonicalPartition` (issuances + flows) instead of
  `Vec<CanonicalDelta>`. Each flow keeps the kernel's grouping and its
  summed source/destination amounts; derivation re-checks the exact
  per-flow equation against the actual consumed source objects.
  `active_families()` projects the delta-family set; a compatibility
  `canonical_deltas()` derives the flat view for unmigrated consumers.
  `issuance_delta` becomes `issuance_projection` over the stored partition.
- Commit `d09eacd` (model). Manifest delta-family conformance derives its
  actual set from `active_families()`, keeping family projection distinct
  from exact accounting.
- Commit `2d6b8c3` (realization). `OperationObservation` carries an
  `ObservedCanonicalPartition`. The normalizer enforces between-flow
  source/destination uniqueness while a flow's movement and destruction
  legs share its one source set; the evaluator's `canonical_flow_holds`
  checks the source/destination/destruction equation, the movement-kind
  rule, and positive destruction legs; issuances check that the issued
  amount equals their destination sum. The model→realization adapter
  projects the exact partition and drops the old issuance-authority
  rejection.
- Tests: the pilots retain their denotation (compact ASH, live transfer);
  fault cases break flow arithmetic, movement kind, and cross-flow source
  reuse; a mixed movement-plus-destruction flow (partial-clear shape) is
  now structurally representable, and reusing a source across flows is a
  partition overlap. The model's existing clear/settlement transitions
  exercise mixed flows through the new certificate partition.
- Identity: no typed-architecture change, so the architecture semantic and
  behavioural hashes are unchanged; only the model/realization public API
  and the model label register (new `certified-canonical-partition` label)
  move. Phase 1 has no public realization hash.
- Verified via the flatpak SDK: full model suite (280) and realization
  suite (90), clippy `-D warnings` on both, and the generated/labels Meson
  lanes.

---

### F1-003 — Complete the compact-ASH weld

**Priority:** P1
**Depends on:** `F1-002` where common exact-flow types change
**Owners:** `realization`

The compact-ASH realization must bind exactly:

- ASH input minimum two;
- ASH maximum `ASH_BATCH_MAX`;
- exactly one ASH output;
- optional ordinary L-BTC sponsor inputs bounded by
  `FEE_SPONSOR_INPUT_MAX`;
- optional ordinary L-BTC sponsor change bounded by one;
- sponsor-owner authorization;
- permissionless operation authorization;
- exact ownerless-lateral `U` flow;
- at most one sponsor envelope;
- exact input/output families;
- exact bound set;
- exact open-flow set;
- exact value-flow set;
- exact root policy;
- exact projection policy;
- empty data-output set;
- exact witness set;
- public constructibility;
- representation and lifecycle support.

`validate_compact_ash_architecture` must compare both directions. Extra
declarations fail; presence-only checks are insufficient.

Required mutations include sponsor authorization/cardinality, bound
replacement, missing/extra value flow, missing/extra witness, unexpected data
output, changed root/projection, changed ASH minimum, and changed output count.

#### Evidence · DONE

- Commit `a5fd729`. `packages/realization/src/validate.rs`:
  `validate_compact_ash_architecture` now also checks the exact sponsor input
  (PlainLbtc, min 0, `FeeSponsorInputMax`, `SponsorOwner`) and output (min 0,
  exactly one), the exact bound set `{AshBatchMax, FeeSponsorInputMax}`, exact
  open-flow and value-flow sets (set equality, not membership), an empty
  data-output set, and the exact four-witness set (CanonicalDelta, AshLineage,
  ValueFlowClosure, UtxoLifecycle) verified against the published architecture.
  Ownership is explicit through `compact_error`/`mismatch_compact`.
- A positive test (`compact_ash_weld_accepts_the_published_architecture`) pins
  the hardened weld; the `derive(&ARCHITECTURE, …)` path already exercises it.
  Verified: realization + model suites, fmt, clippy `-D warnings` green.
- Residual: per-field mutation tests need an `Architecture`-mutation harness
  that neither weld has (the spec is built from `&'static` slices); deferred.

---

### F1-004 — Enforce one sponsor envelope

**Priority:** P1
**Owners:** `realization`, `model`

Add a distinct typed relation:

\[
\#\text{FeeSponsor flows}\in\{0,1\}.
\]

Keep separate claims for:

- sponsor family cardinality;
- sponsor-owner authorization;
- balance;
- source/destination uniqueness;
- envelope multiplicity;
- protocol/sponsor separation.

For both pilots, test zero, one, and two valid disjoint envelopes, plus duplicate
source, duplicate destination, missing owner, and family-count overflow.

#### Evidence · DONE

- Commit `4ebb826`. Added a typed `SponsorEnvelopeMultiplicity` relation (its
  own `RelationKind`, `Relation`, and `RelationFailure`) that counts declared
  `FeeSponsor` open flows and fails when the count exceeds one. Both pilots
  declare it with maximum one, ordered after open-flow policy and before
  sponsor isolation. Sponsor cardinality, owner authorization, balance, and
  isolation remain distinct relations.
- A test in each pilot adds a second disjoint balanced envelope and confirms it
  fails multiplicity (and only multiplicity). Realization now agrees with the
  model's one-envelope rule (`open_flow_tests`). Verified: realization + model
  suites, fmt, clippy `-D warnings` green.

---

### F1-006 — Make sponsored quiescence active-backing aware

**Priority:** P1
**Owners:** `model`

#### Problem

A locally valid request may not fit current active-backing headroom:

\[
\Omega+Q+\delta>\texttt{ACTIVE\_BACKING\_MAX}.
\]

The current scheduler may classify such a world as eligible and then fail with
`ActiveBackingCapExceeded`.

#### Required semantics

Distinguish:

- malformed request;
- cross-pool request;
- sealed-pool request;
- cap-blocked request;
- individually fitting request;
- aggregate batch too large although a subset fits.

Choose a deterministic nonempty fitting subset where one exists.

If none fits and permissionless maintenance cannot restore headroom, return a
typed cap-blocked residual rather than claiming full discharge.

#### Exit theorem

```text
Eligible ⇒ deterministic sponsored driver succeeds
```

Required tests cover exact headroom, one above, fitting subsets, individually
oversized requests, mixed malformed/cap-blocked sets, sealed pools, progress,
and near-cap property traces.

#### Evidence · DONE

- Commit lands a typed `AdmissionCapacityPlan` in
  `packages/model/src/maintenance.rs` that separates three notions the old
  single selector conflated: locally valid requests, the canonical bounded
  batch that fits current active-backing headroom, and whether the entire
  locally valid set could eventually be admitted without an intervening
  redemption. Principals are subtracted from headroom one at a time in
  canonical outpoint order, so an attacker-controlled open request set cannot
  overflow `Sat`; two independent headroom counters keep the full-set proof
  from being coupled to the finite current batch.
- The ambiguous `admissible_requests` selector is replaced by
  `capacity_admissible_request_batch`; the deterministic scheduler, the
  shared-state sweepability precondition, and the property admission driver all
  consume the capacity-aware helpers.
- `packages/model/src/quiescence.rs` gains a typed
  `ActiveBackingCapacityBlocked` residual. `classify_quiescence_eligibility`
  now returns it for a live pool whose locally valid requests exceed headroom,
  keeps the sealed-pool residual keyed on locally valid presence, and treats
  malformed/underfunded requests as non-blocking. `residuals_match_report`
  checks the new residual against the world.
- Focused tests in `active_backing_cap_tests.rs` cover exact headroom, one
  above, a fitting subset when the full set exceeds headroom, canonical skip of
  an oversized request, the `admission_batch_max` bound, and a mixed
  malformed/cap-blocked set that does not collapse. The non-vacuous
  `property_maintenance` theorem still holds: eligible worlds fully discharge;
  blocked worlds match their classified residuals.
- Verified: full `tripod-model` suite (277 tests) and clippy
  `-D warnings` green via the flatpak SDK.

#### Refinement · progress before residual

- Correcting the earlier "eligible ⇒ full discharge" framing, the driver now
  returns a typed `QuiescenceOutcome { world, report, residuals }` with
  `is_fully_discharged()`, and a shared terminal classifier
  `quiescence_residuals` names every residual class present at the scheduler
  fixpoint (not only the two blocking classes that a precondition test reports).
  `classify_quiescence_eligibility` delegates to it, so the precondition and
  terminal classifiers cannot drift apart.
- `drive_sponsored_quiescence` returns the outcome instead of a bare
  `(World, report)` pair, making explicit that the driver admits the canonical
  capacity-fitting subset, processes each admitted request's cycle and
  distribution to completion, and only then names what remains — a
  capacity-blocked request never stalls unrelated permissionless work.
- The `property_maintenance` theorem now drives *every* generated world (not
  just the eligible ones) to a fixpoint and asserts the driver can neither
  create nor clear a blocking residual: a world is eligible exactly when the
  terminal outcome carries no blocking residual. New focused tests prove
  progress-before-residual (admit 60 of {60,50} at headroom 100, cycle and
  settle it fully, leave the 50 as a named residual) and full discharge when
  the whole set fits.
- Verified: full `tripod-model` suite (282 tests) and clippy
  `-D warnings` green via the flatpak SDK.

---

### F1-011 — Derive constructibility authorization from architecture

**Priority:** P2
**Owners:** `realization`

Remove operation-name policy such as:

```text
operation == compact-ash
```

Derive execution authorization from architecture and explicit execution case.

Support:

- permissionless;
- owner-authorized;
- operator-authorized;
- refund-key;
- client-authorized;
- cadence operator case;
- cadence delayed-permissionless case.

A one-bit permissionless flag is insufficient for cadence.

Test synthetic authorization mutations, owner/operator witness availability,
cadence cases, sponsor-local isolation, and cross-operation laundering.

#### Evidence · DONE

- Commit replaces the `operation == OperationId::CompactAsh` boolean passed to
  `validate_constructibility` with a typed `ConstructibilityAuthorization` case
  set. `constructibility_authorizations` in
  `packages/realization/src/validate.rs` derives those cases from the typed
  operation row: `PermissionClass` selects the case shape, owner and
  client-authorized cases read the operation's `InputOwner` input objects, the
  refund-key case reads the `RefundKey` input, and a cadence-band operation
  yields both an operator window and a delayed-permissionless window. The
  operation name is never inspected.
- `packages/realization/src/constructibility.rs` gains
  `ConstructibilityAuthorization`, broadens `AvailabilityClass` with
  `RefundKey`/`ClientOwners`, and validates each dependency's availability
  against the case with `availability_allowed`. Permissionless and
  delayed-permissionless windows keep the focused
  `PermissionlessPrivateDependency` error; every other unmet authorization
  reports the new typed `ConstructibilityWitnessUnavailable`. Sponsor-local
  isolation and cross-operation laundering rejections are unchanged.
- `validate_scoped_realization` now validates every derived authorization case
  per scoped operation, so both pilots weld against real architecture with no
  name special-case.
- Tests: derivation of each `PermissionClass` against the real architecture,
  an operator dependency unavailable to receipt owners, a mutation test that
  flips `TransferLive` to permissionless and observes the owner witness become
  a private-dependency failure, and the refund-key case naming its input. The
  existing permissionless/sponsor/cross-operation rejections still hold.
- Verified: full `tripod-realization` suite and clippy `-D warnings`
  green via the flatpak SDK.

---

### F1-018 — Reject zero-valued ordinary L-BTC in realization

**Priority:** P1
**Owners:** `realization`, model-conformance adapter

#### Problem

The model rejects every zero-valued L-BTC output except `CPFP_ANCHOR`.

Realization currently recognizes ordinary `PLAIN_LBTC` solely by owner
presence, and sponsor-isolation checks ignore unclaimed zero-valued ordinary
L-BTC.

A pilot observation can therefore contain an owner-bearing zero-value
`PLAIN_LBTC` output that passes realization but cannot be accepted by the model.

#### Required rule

```text
PLAIN_LBTC:
    value > 0
    owner present

CPFP_ANCHOR:
    value = 0
    owner absent
    family permitted by operation
```

Test zero-valued sponsor inputs/outputs, unclaimed zero-valued ordinary L-BTC,
and valid zero-valued CPFP only in the exact declared condition.

#### Evidence · DONE

- Commit `6f1efd5`. `packages/realization/src/evaluate.rs`: the `PlainLbtc`
  recognition shape now requires `!observed.value.is_zero() && owner.is_some()`,
  folded into the positive-owned arm; `CpfpAnchor` remains the sole zero-value
  shape.
- Negative tests added in `compact_ash_tests.rs` and `live_transfer_tests.rs`:
  zero-value sponsor input, zero-value sponsor change, and zero-value unclaimed
  `PLAIN_LBTC` each fail recognition; positive owner-bearing sponsor
  input/output remains accepted.
- Verified via the Flatpak SDK: the `tripod-realization` tests and the
  model `realization_conformance` suite pass; `cargo fmt` and clippy with
  `-D warnings` are clean. No generated-artifact or label-register impact.
- Residual: the scope of the positivity rule is disputed — see the Open concern
  below, tracked for future research.

#### Open concern — potential representation-layer over-assertion (future research)

The F1-018 rule was implemented, but its scope is disputed and should be
revisited before it is treated as settled. The realization positivity rule is
applied to **every** recognized value-bearing L-BTC object, including the
open, sponsor-owned pass-through coin. For a sponsor coin that merely funds
the chain fee, this asserts a property (`value > 0`) that:

- reads more state than the operation needs — the fee-sponsor role is defined
  by a balance relation (`Σ sources = Σ destinations + fee`), which a backend
  may discharge by consensus Confidential-Transaction conservation without ever
  opening an individual sponsor value;
- constrains implementation choices — `P-explicit` is deliberately scoped to
  values consumed by covenant arithmetic (receipt, entitlement, vault, ASH,
  reserve, payout, issuance); sponsor L-BTC is **not** in that list, and the
  representation model treats value as a leakage axis (private/public/explicit
  denote the same amount) under a stated minimality obligation;
- is not backed by a well-specified attack. The stated justification is
  partition cleanliness (keep exactly one zero-value carve-out, the ownerless
  `CPFP_ANCHOR`, so the positive-value open-flow partition stays total), not a
  demonstrated exploit against a positive-but-confidential or zero pass-through
  sponsor coin.

Research question: determine whether "value-bearing ⇒ positive" should be
**narrowed** to protocol-accounted objects and lifted from open pass-through
sponsor L-BTC; whether enforcing it at the sponsor seam costs a minimality
violation (an emitted covenant cannot read a blinded sponsor value and should
not be required to); and whether the realization document's §5.1 uniform rule
should itself be re-scoped, since a real fee coin is positive anyway and a
semantically-zero one is inert. Resolving this may reverse or narrow F1-018;
until then the implemented rule stands as a conservative reference-model
normalization, not a proven safety requirement.

---

### F1-021 — Make exact-rational reduction checked

**Priority:** P2
**Owners:** `model`

#### Problem

`AttestationQueryResult` is publicly constructible, and `reduce()` does not
validate its terms. A term with `y = 0` can produce an `ExactRational` with a
zero denominator.

`ExactRational` itself has public fields and does not enforce reduced,
positive-denominator form.

#### Required API

Prefer:

```text
try_reduce()
    validate query
    reduce exactly
    return canonical rational
```

Require:

- denominator \(>0\);
- reduced numerator/denominator;
- canonical zero \(0/1\);
- no unchecked public constructor capable of invalid state.

Keep any unchecked reducer crate-private and call it only after semantic
validation.

Test zero denominator, zero numerator, reducible terms, several terms, and
canonical round trips.

#### Evidence · DONE

- Commit `0f890b5`. `packages/model/src/ledger.rs`: public `reduce()` replaced
  by `try_reduce()` (validates via `validate_query` then reduces); the unchecked
  reducer is now crate-private `reduce_validated()`; `ExactRational` fields are
  private with `numerator()`/`denominator()` accessors.
- Tests in `serialization_tests.rs`, `canonical_rejection_tests.rs`, and
  `attestation_reorg_tests.rs`: zero-denominator and zero-aggregate terms return
  `QueryValidationError`; reducible, canonical-zero (`0/1`), and multi-term round
  trips pass; all `reduce()` callers migrated to `try_reduce().unwrap()`.
- Verified via the Flatpak SDK: `cargo fmt --all --check`, the
  `tripod-model` test suite, and clippy with `-D warnings` are all
  green. No generated-artifact or label-register impact. No residual.

---

### F1-023 — Resolve reorg monotonicity semantics

**Priority:** P1
**Owners:** Layer 0, realization document, model indexer
**Blocks:** `F1-012`

#### Problem

Checkpoint-relative reprojection can move a burn across a clearing boundary.

One direction lowers the sampled value; the reverse direction raises it.
Layer-0 prose currently says settlement-order revision may revise valuation
only downward.

Pure reprojection across independently selected valid checkpoint contexts does
not naturally enforce that one-way claim.

#### Required decision

Choose and state one law:

**Context-relative law**

- each query is exact relative to its checkpoint context;
- monotonicity is claimed only within one retained canonical history and reader
  sampling policy;
- no monotonic ordering is claimed across distinct checkpoint contexts.

**Stateful downward-only law**

- the indexer preserves a prior observation and clamps or otherwise governs
  reorg revisions;
- stateful behavior, cache identity, and auditability are specified explicitly.

The first is more compatible with exact stateless recomputation.

#### Required tests

- clear-before-burn to clear-after-burn;
- reverse direction;
- chained clear→burn;
- earlier-history reorg preserving pair order;
- unchanged raw record under both contexts.

Record whether the correction is presentation-only or changes Layer-0 or
realization denotation.

#### Evidence · DONE

Adopted the **context-relative law**: the valuation is specified over one
consistent chronology, and a reorganization is a change of chronology under
which the same immutable record is revalued upward or downward; no monotone
order is asserted across chronologies. Treated as a **Layer-0 patch release
(v0.5.0 → v0.5.1)**, per the maintainer decision.

- Layer-0 paper: the Monotonicity postulate (Layer-0 §1 interface section) is
  rewritten to the
  consistent-chronology form — within one chronology the record adds only
  nonnegative terms (non-decreasing); a reorg selects a different canonical
  context and may revalue the same raw record up or down; the raw record is
  invariant. The Conservative-Valuation bound is restated without the
  downward-only phrasing. `\setversion` and the master/section headers,
  changelog, `\pdfmetaversion`, and the paper `meson.build` are bumped to
  v0.5.1. The label set is unchanged, so the Layer-0 anchor-set hash is stable.
- Architecture binding: `SpecificationBinding::version` moves to `"0.5.1"` in
  `spec.rs`. This moves the **architecture semantic hash**
  (`003bca0f…` → `237846f3…`); the **behavioural hash is unchanged**
  (`04b0a11b…`), confirming no behavioural-array change. The generated
  `architecture.{json,toml}` are regenerated, and the realization masthead +
  verbatim appendix + §17 index line are updated to the new hash/version — the
  `tripod-artifacts` weld tests enforce byte-identity and pass.
- Realization + companion: the realization reorg sections were already
  checkpoint-relative (no downward-only wording); the human companion gains a
  paragraph stating a reorg re-values the immutable record up or down against
  the selected history, and to wait for settlement depth before treating a
  valuation as final.
- Tests: `attestation_reorg_tests.rs` adds explicit downward and upward
  reprojection of the identical record and a raw-record-invariant check across
  both chronologies, alongside the existing cross-boundary reprojection test.
- Decision: this is a **denotation-preserving clarification** — the implemented
  stateless recomputation already produced context-relative values; the
  downward-only sentence was the inconsistency. It is shipped as a Layer-0
  letter/patch under the versioning policy, not a semantic widening.
- Residual: `plans/phases/00-baseline.md` is a frozen phase-0 snapshot (its
  behavioural fields already predate the v3 behavioural hash) and is
  intentionally left unsynced rather than half-updated.
- Verified via the flatpak SDK: architecture suite incl. the versioning gate
  and weld tests; full model suite incl. the reorg tests; generated-check and
  labels-check lanes; workspace clippy `-D warnings`; and the mocked Meson
  contract (after committing the paper subtree clean).

---

### F1-025 — Give every semantic relation an exact relation kind

**Priority:** P2
**Owners:** `realization`

#### Problem

Current stable relation IDs reuse broader or unrelated `RelationKind` values:

- open-flow policy under sponsor isolation;
- canonical-delta policy under conservation;
- input-family closure under output closure.

Typed subjects keep keys distinct, but the relation family no longer states
what the relation is. This weakens future coverage, diagnostics, proof
selection, and publication clarity.

#### Required change

Add exact relation kinds where semantically distinct, including as needed:

```text
AllowedObjectFamilies
CanonicalDeltaPolicy
OpenFlowPolicy
SponsorEnvelopeMultiplicity
```

Review `ClassClosure`, lifecycle, representation, and authorization naming for
the same property.

Because Phase 1 publishes no realization hash, this is the correct time to
repair the vocabulary.

Test ID stability under declaration order, no collisions, exact report
classification, and unchanged pilot behavior.

#### Evidence · DONE

- Commit `fb9f884`. Added the `AllowedObjectFamilies`, `CanonicalDeltaPolicy`,
  and `OpenFlowPolicy` relation kinds and retargeted each relation ID in both
  pilots to the kind matching its `Relation` value; the genuine amount-
  conservation (Asset) and sponsor-isolation (Sponsor) relations keep their
  kinds. `OutputClosure` had no remaining meaning and was removed.
- Test ID builders in both pilot suites, the model realization-conformance
  fixture, and the realization public API moved in lockstep; the full
  realization and model suites pass, so every stable relation key is accounted
  for. Verified: fmt, clippy `-D warnings` green.

---

## 6. Build, CLI, and repository-contract work · `sec:backlog:tooling-work`

### F1-020 — Redesign checker report/stamp commitment

**Priority:** P1
**Owners:** `cli-common`, checker binaries, Meson
**Blocks:** `F1-007`

#### Problem

No general atomic transaction exists between an arbitrary stdout sink and a
filesystem stamp.

Either order fails one contract:

```text
touch stamp, then stdout:
    stdout failure can leave a false success stamp

stdout, then touch stamp:
    stamp failure can leave success JSON on stdout despite exit 1
```

Simple call reordering cannot simultaneously satisfy ADR-010’s empty-stdout
failure rule and ADR-014’s untouched-stamp failure rule.

#### Required design

Separate direct result mode from Meson build mode.

**Direct mode**

```text
JSON report on stdout
no stamp
```

**Build mode**

```text
--report <file>
--stamp <file>
no stdout result
```

Build mode should:

1. derive and serialize in memory;
2. stage the report uniquely;
3. atomically publish the report;
4. touch the stamp last;
5. emit no stdout.

A stamp failure may leave a report publication but no false success stamp; the
target remains dirty and reruns.

An acceptable alternative is a trusted Meson wrapper that stages child stdout
and touches the stamp only after successful child exit.

Update ADR-014 so the contract is mechanically satisfiable.

#### Evidence · DONE

- Commit adds the two-mode contract to `cli-common`. `CheckOutputArgs`
  carries reciprocal `--report`/`--stamp` clap options (a lone member is a
  usage error). `finish_check_command` publishes the result under the active
  mode: direct mode writes JSON to stdout (refused on a terminal, exit class
  2), build mode writes the report compare-if-changed via an atomic
  sibling-temp rename and touches the success stamp only afterward.
- The impossible ordering is gone because build mode never writes stdout: it
  derives in memory, publishes the report, then touches the stamp last. A
  failed report write leaves no success stamp; the target stays dirty.
- ADR-014's output-or-stamp rule is rewritten to describe the two modes,
  report-before-stamp, and the all-or-nothing pair, so the contract is now
  mechanically satisfiable.
- Tests: report-then-stamp success, compare-if-changed on rerun, half-specified
  mode rejected, and a failed report write leaving the stamp untouched.
- Verified: `cli-common` suite, clippy `-D warnings`, and all 10 Meson lanes
  green via the flatpak SDK.

---

### F1-007 — Implement failure-safe checker completion

**Priority:** P2
**Depends on:** `F1-020`
**Owners:** `cli-common`, `check-generated`, `check-labels`, `census-audit`

After the build-mode contract is selected, centralize it in shared
infrastructure.

Test:

- successful report and stamp;
- serialization failure;
- broken report destination;
- full destination;
- rename failure;
- stamp failure;
- TTY refusal in direct mode;
- usage failure;
- unchanged prior stamp on every pre-stamp failure.

No binary may carry a private copy of the transaction ordering.

#### Evidence · DONE

- Commit centralizes the build-mode contract in `cli-common::run_check_command`,
  which installs the panic hook and tracing, runs the check, and publishes
  through `finish_check_command`. `check-generated`, `check-labels`, and
  `census-audit` all migrate to it: each drops its `--stamp`-only argument and
  private `touch_stamp`/`emit`/TTY-refusal handling and flattens
  `CheckOutputArgs`. No binary carries a private copy of the ordering.
- The Meson `checker_wrap` stdout-redirection shell shim is removed; the three
  Rust checker targets consume `--report @OUTPUT1@ --stamp @OUTPUT0@` directly,
  taking shell redirection off the correctness boundary.
- Failure safety: a failed check returns `Err` from the closure (mapping to the
  failure exit class) after emitting per-item diagnostics, so neither the report
  nor the stamp is written; the shared `finish_check_command` tests cover the
  pre-stamp failure leaving the stamp untouched.
- Residual: broken-destination / full-destination / rename-failure fault
  injection beyond the parent-is-a-file case is not separately simulated; the
  atomic-rename path is shared and covered by the failed-write test.
- Verified: checker crate suites, clippy `-D warnings`, all 10 Meson lanes, and
  the mocked Meson contract green via the flatpak SDK.

---

### F1-005 — Restore paper-stamp restat behavior

**Priority:** P1
**Owners:** `document-stamps`, paper Meson, mock contract

Remove the always-retouched paper success stamp.

The paper derivation already has two real outputs:

```text
stamps.tex
source-date-epoch
```

Retain compare-if-changed writes. Probe the real outputs where an existence test
is needed.

The mock contract must prove command non-execution on the second unchanged
build through invocation counts, trace files, Ninja log inspection, or another
deterministic mechanism. Equal bytes alone are insufficient.

Exit requires:

- unchanged real-output mtimes;
- no downstream TeX/mock/flatten command rerun;
- changed paper Git state reruns the path;
- dirty paper state still fails;
- output-or-stamp classification is unambiguous.

#### Evidence · DONE

- Commit `3153f6c` removed the always-retouched `attestation-stamps.ok`
  success output: `RenderRequest` and the CLI drop `--stamp`, the render mode
  is the three-value `--template`/`--stamps-output`/`--epoch-output` set, and
  the Meson target and probe use only the two real outputs.
- Commit `6c7253a` proves command non-execution on a no-op build through
  Ninja's build log: `scripts/test-meson-mock.sh` discovers the render
  (`main.pdf`) and flattener outputs and asserts neither re-executes, while the
  always-stale stamp derivation may rerun with unchanged outputs. Verified via
  the mocked Meson contract (green end-to-end).
- Residual: the changed-paper-Git-state rerun direction needs a cloned source
  fixture (§11.3) and is deferred; the no-op count landed.

---

### F1-019 — Make render arguments all-or-nothing

**Priority:** P2
**Owners:** `document-stamps` CLI

The current clap requirement runs only from `--template` toward the three
outputs. A caller may pass one output argument without `--template`; the command
then silently enters JSON mode and ignores the output argument.

Define valid modes as exactly:

```text
zero render arguments:
    JSON derivation mode

all render arguments:
    render mode
```

Every partial subset exits 2 with empty stdout and one JSON usage record.

Implement through reciprocal requirements or one exact clap argument group.

#### Evidence · DONE

- Commit `550ae56`. `packages/document-stamps/src/bin/attestation-stamps.rs`:
  each render output (`--stamps-output`, `--epoch-output`, `--stamp`) now
  carries `requires = "template"`, making every partial render argument set a
  clap usage error (reciprocal-requirements repair).
- New integration test `packages/document-stamps/tests/subprocess_contract.rs`
  drives every singleton and partial subset and asserts exit 2, empty stdout,
  one JSON `usage_error` record on stderr, and no files written; `serde_json`
  added as a dev-dependency; the `tests/` file is censused under
  `document_stamps_excluded_files` (ADR-014), not the crate-source list.
- Verified: `cargo test --locked -p tripod-document-stamps`, fmt, and
  clippy `-D warnings` green; `meson test` `attestation-stamps` lanes pass.
- Residual: the render success `--stamp` output is retained; whether it is
  removed is governed by `F1-005`, not this finding.

---

### F1-009 — Make revision selection coherent

**Priority:** P2
**Owners:** `document-stamps`

Choose one contract:

**Full selected-ref support**

- mode, bytes, history, tree identity, and timestamp all come from `tree_ref`;
- render sources are staged from or verified against the same ref.

**HEAD-only support**

- resolve `tree_ref` and `HEAD`;
- reject inequality;
- document the restriction.

Do not retain hybrid metadata from a selected ref with bytes from another
worktree revision.

Test older ref from newer checkout, dirty HEAD, and unchanged scope behavior.

#### Evidence · DONE

- Commit adopts the HEAD-only contract. `packages/document-stamps/src/lib.rs`
  gains `verify_selected_revision_is_head`, which peels both the selected
  revision and `HEAD` to their commit objects
  (`peel_to_commit` builds the `^{commit}` suffix at runtime) and rejects
  inequality with the new typed `SelectedRevisionIsNotHead`. It runs in
  `derive` after the object-format guard and before the dirty-subtree check and
  any input read, so a non-HEAD revision aborts before Git and filesystem state
  are ever mixed. Commit-object equality means a branch or tag pointing at
  exactly HEAD is accepted; literal string equality is not required.
- The CLI help and the `StampRequest::tree_ref` doc state the restriction, and
  the README provenance section documents that selected-ref publication from an
  un-checked-out revision is unsupported.
- Tests: HEAD accepted, a branch alias resolving to HEAD accepted, an older
  commit rejected, and a derive-level test proving the rejection happens before
  digesting (the fake stubs only the pre-guard queries, so any later Git call
  would panic). The dirty-subtree rejection is unchanged.
- Verified: `tripod-document-stamps` suite, clippy `-D warnings`, and
  `scripts/test-attestation-stamps.sh .` (all identity-scope assertions pass)
  via the flatpak SDK.

---

### F1-010 — Strengthen clean-tree verification

**Priority:** P2
**Owners:** `scripts/ci.sh`, ADR-011 gate

Replace the final `git diff --exit-code` check with a complete status test such
as:

```sh
git status --porcelain=v1 --untracked-files=all
```

Fail on:

- unstaged tracked changes;
- staged changes;
- untracked nonignored files.

Permit ignored build products.

Tests must cover all four cases and name offending paths.

#### Evidence · DONE

- Commit `a645f93`. `scripts/ci.sh` lane 11 replaces `git diff --exit-code`
  with `git status --porcelain=v1 --untracked-files=all`, failing on any
  non-empty status and printing offending paths.
  `scripts/check-document-reproducibility.sh` strengthens its dirty-worktree
  probe the same way.
- Behavior verified in a disposable repository across all four cases: clean tree
  passes; unstaged tracked change, staged change, and untracked nonignored file
  each fail; an ignored build product passes.
- No generated-artifact or label-register impact. No residual.

---

### F1-014 — Remove or declare `jq`

**Priority:** P2
**Owners:** stamp integration test, Meson, README

Preferred repair: use `python3`, already resolved by Meson, for JSON extraction.

Alternative: resolve and pass `jq` explicitly and document it as a requirement.

The full Meson test surface must not depend on an undeclared ambient PATH
program.

#### Evidence · DONE

- Commit `a645f93`. `scripts/test-attestation-stamps.sh` replaces the `jq`
  field extractor with an inline `python3` reader (python3 is already resolved
  by top-level Meson); `README.md` requirements now list Python 3 and the POSIX
  shell utilities.
- Verified: `sh scripts/test-attestation-stamps.sh .` passes every
  identity-scope assertion, and a `jq` word-boundary grep over `scripts`,
  `meson.build`, and `README.md` finds no build or test dependency on it.
- No generated-artifact or label-register impact. No residual.

---

### F1-015 — Resolve shell/Python checker output policy

**Priority:** P3
**Owners:** ADR-010, ADR-014, build tooling

Classify every first-party executable checker.

Preferred:

- move build-facing command boundaries under Rust/`cli-common`;
- emit typed JSON reports and diagnostics;
- use shell/Python only behind a JSON-speaking boundary.

Alternative:

- amend ADR-010 with a precise exclusion for orchestration scripts;
- state stream, exit, and secret-handling rules;
- test the boundary.

Policy must not remain broader than implementation.

#### Evidence · DONE

- Commit takes the preferred path: the last shell checker on a build-facing
  command boundary, `scripts/check-forbidden-text.sh`, is deleted and replaced
  by a Rust `check-forbidden-text` binary that runs under the shared
  `cli-common` two-mode contract. It emits a typed JSON report and JSON
  diagnostics on stderr; no first-party checker now conflicts with the
  ADR-010/014 stream wording. The remaining scripts (`check-plans.sh` wraps a
  Python checker; `ci.sh`, `test-meson-mock.sh` are orchestration) do not own a
  public diagnostic contract.
- Verified: labels suite, workspace clippy `-D warnings`, and all 10 Meson
  lanes green via the flatpak SDK.

---

### F1-022 — Resolve `execwrap` exit semantics

**Priority:** P2
**Owners:** ADR-010, root README, `execwrap`

`execwrap` propagates child statuses, including arbitrary values and signals.
The repository documentation currently says every executable uses only:

```text
0 success
1 failure
2 usage
```

A child exiting 2 makes wrapper runtime failure indistinguishable from wrapper
usage.

Choose one:

- map all nonzero child results to wrapper failure 1 and report child status
  structurally; or
- document a precise `execwrap` exception and define how child statuses avoid
  collision with reserved wrapper statuses.

Update ADR, README, tests, and Meson expectations together.

#### Evidence · DONE

- Commit `2a6e0c9`. `wrapper_exit_from_child_status` maps a relayed child
  status onto the wrapper exit code so the reserved classes stay intact: child
  0 → success 0, child 1 or 2 → wrapper failure 1 (code 2 stays reserved for
  wrapper usage), child 3..=255 (including `128 + signal`) relay unchanged, and
  an unspawnable child stays failure 1.
- ADR-010 documents the child-status-relay exception, the root README no longer
  claims every executable uses only 0/1/2, and subprocess tests cover the
  0/1/2/3/42 mapping, the unspawnable-child case, and signal relay. Verified:
  execwrap suites, fmt, clippy `-D warnings` green.

---

### F1-024 — Complete subprocess coverage

**Priority:** P2
**Owners:** all command packages

ADR-010 requires subprocess coverage for each shipped executable, not only
shared-helper tests.

Cover:

```text
attestation-stamps
check-generated
generate-all
check-labels
generate-label-registers
census-audit
execwrap
flatten-latex-main
```

For each applicable binary test:

- help;
- version;
- unknown argument;
- missing argument;
- success;
- runtime failure;
- empty stdout on failure;
- JSON-only diagnostics;
- exit class;
- TTY refusal;
- explicit asset writes;
- direct versus build-report mode;
- panic behavior where practical.

The all-or-nothing render regression belongs here.

#### Progress — control-plane coverage landed first

Commit `91b5ce8` added per-package subprocess tests for the six previously
uncovered binaries (`check-generated`, `generate-all`, `check-labels`,
`census-audit`, `generate-label-registers`, `flatten-latex-main`), pinning the
uniform ADR-010 control-plane contract: `--help`/`--version` exit 0 with empty
stdout, missing/unknown arguments are usage class 2, and every control-plane
record is one JSON object on stderr. `attestation-stamps` and `execwrap`
(+`execwrap-mock-tex`) already had subprocess coverage. The one lane that
remained after this step was real PTY-backed TTY refusal, closed below.

#### Evidence · DONE

- The last open lane — real PTY-backed TTY refusal — now runs. A reusable
  `openpty` harness (`nix` with the `term` feature) attaches a child's stdout to
  a pseudo-terminal, drains the master on a detached helper thread through a
  channel (this sandbox does not reliably deliver EIO on slave close, so the
  reader is never joined), captures stderr to a temp file, and returns the exit
  status, terminal-stdout bytes, and stderr.
- Coverage is by output-mode across both refusal code paths, since every binary
  funnels through `cli-common`: `census-audit` (the `run_check_command` path)
  refuses a terminal stdout in **direct** mode with exit 2 and one `tty_refusal`
  record (`fields.stream = stdout`) and, crucially, does so *before* any
  semantic work — `run_check_command` now performs the direct-mode refusal ahead
  of the check closure. In **build** mode (`--report`/`--stamp`) the same binary
  publishes its report and stamp and exits 0 on a terminal, proving refusal
  depends on output mode, not binary identity. `attestation-stamps` covers the
  `run_stdout_json_command` path (JSON mode refuses a terminal; render mode, a
  `run_no_stdout_command` side effect, never refuses).
- No silent cap: the refusal decision lives entirely in `cli-common`, so these
  representative binaries exercise each distinct code path rather than
  duplicating an identical assertion per binary; the checker success paths are
  exercised for real by the Meson `generated-check`/`labels-check`/`census-audit`
  lanes every build.
- Verified: the `tripod-labels` and `tripod-document-stamps`
  subprocess suites (including the PTY tests), `cli-common` lib suite (38), and
  clippy `-D warnings` across the CLI crates, green via the flatpak SDK.

---

### F1-027 — Isolate mocked TeX execution from the production wrapper

**Priority:** P2
**Owners:** `execwrap`, Meson

#### Problem

The production `execwrap` binary carried hidden `--mock-child`, `--mock-outdir`,
and `--mock-fail` flags that skipped child execution and fabricated TeX outputs.
Test simulation must not be reachable through the ordinary production wrapper
interface: "hidden from help" is not "unavailable".

#### Evidence · DONE

- Commit `fa09ccf`. The simulation moved into a dedicated `execwrap-mock-tex`
  binary (`--child`/`--outdir`/`--fail`, calling the existing `run_mock_child`),
  classified under ADR-010 as a build/test-internal side-effect command. The
  production wrapper now has no mock flags and always executes its child.
- The mock Meson graph invokes `execwrap-mock-tex` directly (built only in
  mock_mode). Subprocess tests confirm the production wrapper rejects every
  former mock flag as a usage error, the mock helper writes each child's
  outputs, and `--fail` writes nothing. Verified: execwrap suites and the
  end-to-end mocked Meson contract green.

---

### F1-028 — Use checked report-count conversions

**Priority:** P3
**Owners:** `model`

The maintenance phase potential widened platform-local `usize` report counts
into protocol-analysis `u64` values with unchecked `as u64` casts. On realistic
platforms the widening is harmless, but the project's exactness policy avoids
unreviewed casts in proof-relevant values.

#### Evidence · DONE

- Commit replaces the `as u64` casts on the report count fields in
  `maintenance_phase_potential` (`admissible_requests`, `entitlements`,
  `live_distributions`, `time_locked_receipts`, `ash_outputs`) with
  `u64::try_from(..).map_err(|_| Guard::Overflow)?`. The function already
  returns `Result`, so the checked conversion adds no new failure surface at
  the call sites. Casts that follow an established bound elsewhere were left
  unchanged; only the proof-relevant potential counts were converted.
- Verified: full `tripod-model` suite and clippy `-D warnings` green
  via the flatpak SDK.

---

### F1-030 — Bind canonical inputs to committed paper blobs

**Priority:** P2
**Owners:** `document-stamps`

Three related input-identity defects: digest bytes were read from the worktree
after a Git metadata lookup; inputs were not required beneath the dirty-checked
paper subtree; and lexical aliases (`./`, `/./`) could bypass duplicate
detection or canonical framing. The Part-III HEAD-only restriction narrows but
does not close these.

#### Evidence · DONE

- Commit adds `canonical_relative_path`, which rejects absolute paths and every
  lexical alias (leading `.`/`..`, root, prefix) and requires a nonempty
  `Normal`-component path. The paper subtree is canonicalized the same way, and
  every input must be strictly beneath it or fail the new
  `InputOutsidePaperTree`, so the subtree dirty-check covers every rendered
  input.
- Digest authority moves to the committed blob: `tracked_blob` returns the mode
  and object id via a `:(literal)` pathspec with exact-path verification, and
  `cat-file blob <oid>` (a new binary-safe `run_bytes`) supplies the digest
  bytes. Those bytes are compared to the worktree bytes the build renders; any
  divergence is a dirty-subtree failure. History and status queries use literal
  pathspecs so a metacharacter filename cannot change the selected set, and a
  second subtree cleanliness check runs after all inputs are read (narrowing,
  not closing, the live-worktree race — documented honestly).
- Tests: leading-`./` alias rejected, normalizing `/./` alias caught as a
  duplicate, input outside the subtree rejected, worktree/blob byte
  disagreement flagged dirty, exact-path mismatch rejected, and a
  pathspec-metacharacter filename resolved literally.
- Identity: the digest recipe (domain, canonical path, mode, committed bytes)
  is unchanged for clean canonical inputs, so the real-git integration test
  `scripts/test-attestation-stamps.sh .` still passes every identity-scope
  assertion; the accepted input domain is merely stricter.
- Verified: `tripod-document-stamps` suite, clippy `-D warnings`, and
  the real-git integration script green via the flatpak SDK.

---

### F1-029 — Distinguish checked transitions from branch-local apply

**Priority:** P3
**Owners:** `model` tests

Many positive scenario tests call `Transition::apply(..).unwrap()` and then a
separate `check_invariant`, which proves "the branch guard accepted and the
invariant held afterward" but reads like "the invariant-wrapped public path
enforced it." The evidence class should be mechanically obvious in the test
source. Fault, authorization-failure, corruption, and kernel/branch-local tests
must keep raw `apply`, so this is a per-module discipline change, not a blind
sweep.

#### Progress — helper landed, first module converted

Commit adds a shared `apply_checked` test helper (in `scenario_fixtures`) that
runs a transition through the normative `execute` path — the invariant-wrapped
public entry point — and returns the successor world. The receipt-relabel
module is converted as the pilot: its two public positive-scenario tests now
advance through `apply_checked` instead of `apply().unwrap()` plus a separate
`check_invariant`, while its authorization-failure and kernel-structural
fault-injection tests deliberately keep raw `apply`.

#### Evidence · DONE

- The adoption sweep is complete across the positive-scenario modules: `cycle`,
  `ash_clear`, `redemption`, `admission`, `transfer`, `burn_attestation`,
  `request_lifecycle`, `stronger_fee_auction`, `realization_conformance`, and
  `checkpoint_semantics` now advance their success scenarios through
  `apply_checked`, dropping the paired `apply().unwrap()` + separate
  `check_invariant` where the invariant was the sole remaining assertion.
- The discipline is per-module, not a blind sweep: raw `apply` is deliberately
  retained wherever the evidence class differs — negative `Err` assertions
  (authorization/cadence/ceiling/cap failures), fault-injection and
  history-corruption robustness tests, kernel-structural `TxBuilder` cases,
  forged-certificate/reorg-tamper tests, and stale-candidate `if let Ok(..)`
  rebuilds. `fee_auction` is left untouched because its one apply is matched
  against an expected `Err(CadenceTooEarly)`; `settlement` positives already
  flow through the `settle_batch` fixture with no direct apply to convert.
- Verified: full `tripod-model` suite (282 tests) and clippy
  `-D warnings` green via the flatpak SDK.

---

### F1-026 — Repair shell checker stamp and no-op behavior

**Priority:** P2
**Owners:** forbidden-text checker, Meson, ADR-014

The forbidden-text checker:

- truncates the stamp with `: >`;
- is always stale;
- reruns on every build;
- receives no explicit tracked-file census;
- contradicts the broad no-op lint claim.

Repair by moving the check under the selected checker/report contract or by
giving it explicit inputs and touch-only semantics.

A successful second unchanged Ninja invocation must execute no ordinary lint
command whose inputs are fully represented by the graph.

If an always-stale Git-derived audit remains necessary, isolate and document it
as such without claiming all lint commands are clean on no-op.

#### Progress — stamp truncation replaced; core remains TODO

Commit `478ccf9` replaced `: > "$stamp"` with `touch "$stamp"` in the
forbidden-text checker, the publication mirror, and the generator/cargo-quiet
Meson wrappers, removing the accidental-truncation hazard. This is stamp-write
hygiene only: the always-stale execution, the missing explicit tracked-file
census, and the no-op lint claim are unaddressed, so this finding stays
**TODO**.

#### Evidence · DONE

- Commit replaces the shell checker with a Rust `check-forbidden-text` binary
  under the two-mode contract: build mode writes a compare-if-changed JSON
  report and touches the success stamp only after the report is written, so
  the truncation-and-touch stamp handling is now the shared, tested path.
- The audit is now an explicitly declared always-fresh repository audit: its
  own source files (`src/bin/check-forbidden-text.rs`, `src/forbidden.rs`) join
  the labels crate-source census, and ADR-014 documents the two stamp-target
  classes — incremental lint suites (no-op on an unchanged tree) versus the
  `census-audit`/`forbidden-text-check` repository audits that ask git for the
  complete tracked set every build and cascade nothing through their
  compare-if-changed reports. The broad no-op claim is scoped to the
  incremental suites, so implementation and policy agree.
- Verified: labels suite, workspace clippy `-D warnings`, all 10 Meson lanes,
  and the mocked Meson contract green via the flatpak SDK.

---

### F1-031 — Omit caller-controlled program text from `execwrap` diagnostics

**Priority:** P2
**Owners:** execwrap, ADR-010

#### Problem

`execwrap` logged the child program path on both the normal "executing command"
info record and the spawn-failure error record. That path is caller-controlled
`argv[0]` — raw child argv under ADR-010 — so a caller could surface arbitrary
text (for example `/nonexistent/SHOULD_NOT_APPEAR_api-token`) in the JSON
diagnostics. Hidden redaction is not an adequate boundary because raw child argv
is prohibited by field class.

#### Evidence · DONE

- The `program` field is removed from the normal info record (now
  `argument_count` and `pid` only, "executing child process") and the field is
  dropped from `ExecError::Spawn` entirely, so its `Display`
  ("failed to spawn child process: {source}") and the binary's spawn-failure
  record can no longer disclose it. Argument count and pid remain as known-safe
  metadata.
- Two subprocess regressions execute the real binary: a spawn failure whose
  program path embeds a secret, and one whose path is a credential-bearing URL.
  Each asserts exit 1, empty stdout, the secret absent from stderr, and that
  every non-blank stderr line is a JSON object.
- ADR-010 is clarified: program identity may be logged only from trusted typed
  configuration or an explicit safe identifier; a caller-supplied executable
  path is raw child argv and is omitted.
- Verified: `execwrap` clippy `-D warnings` and the full `execwrap` subprocess
  suite (21 tests) green via the flatpak SDK.

---

### F1-032 — Omit untrusted git stderr from `census-audit` diagnostics

**Priority:** P2
**Owners:** labels, ADR-010

#### Problem

`census-audit` takes the git program as an argument and, on a non-zero
`git ls-files`, logged the child's raw stderr via
`String::from_utf8_lossy(&listing.stderr)`. A supplied executable can print
arbitrary secret-bearing text there, bypassing the field-based diagnostic
policy — the external program's stderr is neither typed git status nor a safe
field.

#### Evidence · DONE

- The failure branch now logs only the process status and the fixed
  "git ls-files failed" message; the raw child stderr is omitted rather than
  relayed or heuristically redacted. ADR-010 is clarified: the raw stderr of an
  argument-supplied external program is arbitrary child output and is omitted,
  with only the exit status reported.
- A new subprocess regression runs the real `census-audit` binary against a
  fake git script that writes a secret to stderr and exits 17. It asserts
  exit 1, empty stdout, JSON-only stderr, the secret absent, and the generic
  failure message present.
- Verified: `tripod-labels` clippy `-D warnings` and the labels
  subprocess suite green via the flatpak SDK.

---

### F1-034 — Stage both render outputs before publication

**Priority:** P2
**Owners:** document-stamps

#### Problem

Render mode wrote `stamps.tex` and `source-date-epoch` with two independent
compare-if-changed atomic writes. If the first succeeded and the second failed,
the command exited failure but left a partially updated pair. The failed Meson
edge reruns and repairs the pair, so this is not a false success, but the
multi-output effect was not transaction-like.

#### Evidence · DONE

- `write_if_changed` is split into two primitives: `stage_if_changed` performs
  the compare, temp-file create, write, and fsync (returning `None` for an
  unchanged output), and `publish` performs the single rename that mutates a
  final path. `render_outputs` stages *both* outputs before publishing either,
  so any read/create/write/fsync failure for either destination aborts before
  any final output changes.
- Publication order is dependency-first: `source-date-epoch` is renamed before
  `stamps.tex`, so a failure during the second rename tends to leave the more
  visible TeX metadata old rather than pairing new TeX metadata with an old
  epoch. This is a documented recovery preference, not a proof of atomicity —
  two independent paths cannot be renamed as one transaction, and the code and
  tests say so honestly.
- Failure-injection and coherence tests cover: epoch staging failure leaves
  stamps untouched; stamps staging failure leaves epoch untouched; an epoch
  publish (rename-onto-directory) failure leaves stamps unpublished; a
  successful rerun repairs a partial prior state; and an identical rerun leaves
  both mtimes unchanged.
- Verified: `tripod-document-stamps` clippy `-D warnings` and the
  full crate suite (39 lib + subprocess tests) green via the flatpak SDK.

---

## 7. Label, documentation, and planning work · `sec:backlog:documentation-work`

### F1-008 — Reject malformed Realization imports exhaustively

**Priority:** P2
**Owners:** `labels`

For every participating single-backtick square token in Realization:

1. parse a known imported owner;
2. otherwise parse the intentional legacy model-label form;
3. otherwise emit exactly one typed diagnostic.

Required classes:

- unknown prefix → `UnknownOwner`;
- malformed known-owner label → `InvalidLabel`;
- missing parentheses → form error;
- self-qualified R13 import → owner-boundary form error;
- valid legacy model token → synthetic MODEL import.

Fenced and double-backtick examples remain nonparticipating.

No imported-looking participating token may disappear silently.

#### Evidence · DONE

- Commit `e23fbc2` added the initial import classifier; commit `6cf6b53`
  reworked the harvester to audit every square-bracketed token in its own
  grammar class (the document's status-tag family): enforced-pin and invariant
  tags are audited in place — enforced pins resolve to their pin mints,
  invariant ordinals range-check against the architecture clause set — template
  examples (placeholder glyphs) are explicitly exempted, and only genuine
  citation-shaped tokens reach the import classifier. A dedicated
  InvalidStatusTag diagnostic types every status-tag failure; unknown owner,
  malformed label, and bare or asymmetric imported form each get their typed
  diagnostic.
- The realization document legend was corrected to "always backticked", one
  bare model cite was repaired, and a test covers valid tags, both example
  forms, and each malformed case. labels-check went from 101 false positives to
  clean; fmt, clippy `-D warnings`, and the labels suite pass.

---

### F1-012 — Correct normative and companion prose

**Priority:** P2
**Depends on:** `F1-023`
**Owners:** Realization document, human companion, generated registers if labels move

Correct at least:

#### Floor arithmetic

Replace the false claim that \(\Omega/Y\) is irrational.

The floor is rational when \(\Omega\) and \(Y\) are integers, but generally
nonintegral and not necessarily representable exactly in a selected fixed-width
target encoding.

#### STATE/RESV use

Use the typed manifest census:

```text
STATE + RESV:
    admit-deposits
    cycle
    redeem

STATE only:
    clear
    receipt-relabel
    announce-maturity

RESV termination:
    sealing redemption
```

Do not claim every pool-touching transition spends both roots.

#### Conservative valuation

A last-clearing floor may under-credit the attestation relative to the
burn-instant redeemable value. It does not exaggerate sacrifice cost.

#### Reorg semantics

Apply the decision from `F1-023` consistently across Layer 0, realization, human
companion, and model documentation.

Record:

- Layer-0 version impact;
- realization letter/major impact;
- architecture semantic-hash impact;
- behavioral-hash impact;
- anchor-set impact;
- label-register impact.

#### Progress — presentation corrections landed

Commit `dc25a74` landed the three presentation-only corrections that do not
depend on the reorg decision: the floor-arithmetic wording
(`docs/attestation/realization.md`), the STATE/RESV operation census and the
conservative-valuation wording (`docs/attestation/human.md`), and the stale
`§16 → §17` anchor-index comment (`packages/architecture/src/spec.rs`). These
touched no label token, generated artifact, or architecture hash (`labels-check`
and the `artifacts` weld tests pass unchanged).

#### Evidence · DONE

- The remaining **Reorg semantics** correction was applied by `F1-023` (commit
  `d7b0747`) consistently across every surface this finding names, and a
  repository-wide search now finds no residual downward-only wording in
  `papers`, `docs`, `packages`, or `plans` (outside these descriptive backlog
  rows):
  - **Layer 0** — the Monotonicity postulate in
    `papers/attestation/sections/01_interface.tex` is stated over one consistent
    chronology; a change of chronology may revalue a record *either upward or
    downward*, and no monotone order is asserted between valuations drawn from
    different chronologies.
  - **Realization** — `docs/attestation/realization.md` (goals G4 and SP2,
    and the reorg-sensitivity section 12.4) describes the derived valuation as
    checkpoint-relative reprojection, monotone only relative to a checkpoint
    policy.
  - **Human companion** — `docs/attestation/human.md` re-values an immutable
    record under a new history "possibly up, possibly down."
  - **Model** — `packages/model` prose (`ledger.rs`, `lib.rs`) describes a reorg
    as producing a fresh reference index / checkpoint reprojection, never a
    downward-only revision.
- Recorded impacts: Layer-0 version `0.5.0 → 0.5.1` (patch); architecture
  semantic hash moved (`003bca0f… →…`); behavioural hash **unchanged**
  (`04b0a11b…`), so no Realization letter/major revision was required; no label
  anchor was minted or moved, so the anchor set and label registers are
  unchanged (`labels-check` green throughout).
- Verified: `labels-check`, `generated-check`, `plans-check`, and the `artifacts`
  weld/versioning gate green via the flatpak SDK.

---

### F1-013 — Reconcile graph planning with D007

**Priority:** P2
**Owners:** compiler/linker research notes, package contracts, backlog

Remove active tasks requesting first-party graph wrapper types or adapter
layers that reproduce or hide Petgraph storage/traversal.

Replace them with:

```text
direct Petgraph graph construction
+ typed stable keys
+ key/index lookup metadata
+ canonical insertion
+ canonical result normalization
+ first-party typed publication projection
```

Package-local helper functions are allowed. A wrapper reproducing or hiding
Petgraph storage/traversal is not.

No shared graph crate is introduced.

#### Evidence · DONE

- Commit `03434c6`. `plans/research/compiler-algorithms.md` and
  `plans/research/linker-algorithms.md` now prescribe direct package-owned
  Petgraph construction (graph-boundary prose, the Stage-1 heading, and the
  handoff line); `plans/backlog.md` rewrites the adapter task around forbidding
  wrapper/adapter layers.
- Verified: a grep for the prohibited `private graph adapter`,
  `frozen graph adapter`, and `typed graph adapters` wording across the
  research, package, phase, and backlog plans returns only text that enumerates
  the forbidden designs; `check-plans.sh` and `labels-check` pass.
- Residual: the rejected wrapper names survive only where D007 lists them as
  forbidden.

---

### F1-016 — Reconcile planning status and evidence

**Priority:** P3
**Owners:** backlog, phase cards, research notes, package contracts

Required closure:

- R1-013 has one status everywhere;
- C1 dependency review reflects the already adopted Petgraph dependency;
- completed tasks carry completed evidence;
- D007-superseded tasks are rewritten;
- “implementation present” remains distinct from “phase gate passed”;
- all detailed statuses agree with summary tables;
- current phase, roadmap, and phase card agree;
- every `DONE` task satisfies (`rule:backlog:done`).

---

## 8. Phase-1 implementation lane · `sec:backlog:r1`

### 8.1 Status · `tbl:backlog:r1`

| ID | Status | Deliverable |
|---|---|---|
| `R1-001` | **DONE** | Realization crate and package contract |
| `R1-002` | **DONE** | Typed stable IDs and local-handle separation |
| `R1-003` | **DONE** | Typed domains, expression graph, evaluator |
| `R1-004` | **DONE** | Semantic relation vocabulary (vocabulary remediation F1-025 landed) |
| `R1-005` | **DONE** | Constructibility, lifecycle, representation (constructibility remediation F1-011 landed) |
| `R1-006` | **DONE** | Deterministic scoped derivation |
| `R1-007` | **DONE** | Compact-ASH pilot (exact weld F1-003 landed) |
| `R1-008` | **DONE** | Live-transfer pilot (one-sponsor-envelope F1-004 landed) |
| `R1-009` | **DONE** | Dependency-derived pilot declassification |
| `R1-010` | **DONE** | Architecture/realization validation (exact weld F1-003 landed) |
| `R1-011` | **DONE** | Model-conformance projection/tests (exact flows F1-002 landed) |
| `R1-012` | **DROPPED** | No Phase-1 realization publication; typed values are direct inputs |
| `R1-013` | **BLOCKED** | Complete Phase-1 evidence and exit |

Every implementation deliverable is complete and its named F1 remediation has
landed; only `R1-013` (the recorded exit gate) is outstanding, blocked solely on
running and recording `F1-017`.

### 8.2 Phase-1 semantic closure

Before `R1-013`:

- exact mixed flows are representable;
- compact ASH is exactly architecture-welded;
- one sponsor envelope is enforced;
- ordinary L-BTC positivity agrees with the model;
- constructibility derives from architecture authorization;
- relation IDs name exact semantic families;
- model execution remains independent of realization acceptance;
- partial scope cannot be mistaken for complete scope.

### 8.3 No Phase-1 publication

R1-012 remains dropped because current consumers use typed Rust values
directly. A later publication requires:

- real consumer;
- explicit schema;
- canonical ordering;
- deterministic bytes;
- unknown-field rejection;
- one writer;
- one non-writing checker;
- explicit partial/complete scope;
- no reverse semantic dependency.

---

## 9. Phase-1 gate · `gate:backlog:phase1`

### 9.1 Preconditions

Phase 1 may exit only when:

```text
F1-001 through F1-016 are closed
F1-018 through F1-034 are closed
F1-017 is DONE
R1-013 is DONE
```

`F1-017` is the evidence task; it does not substitute for remediation.

### 9.2 Required environments

- declared Rust MSRV;
- current stable Rust;
- canonical Meson build at `build/`;
- required TeX environment for the document lane;
- environment without accidental undeclared helpers where practical.

A skipped required lane is not a pass.

### 9.3 Required commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
scripts/check-document-reproducibility.sh
git status --porcelain=v1 --untracked-files=all
```

`cargo audit` follows ADR-011. If unavailable, it is reported as skipped, never
silently passed. The phase evidence states whether release policy requires it
to be installed.

### 9.4 Evidence record

Record:

- Rust, Cargo, Meson, Ninja, Python, Git, and TeX tool versions;
- command results;
- optional skipped tools;
- generated-artifact freshness;
- label and census status;
- subprocess-contract status;
- no-op Meson command counts;
- stamp output mtimes;
- document reproducibility result;
- final clean-tree output;
- identity/schema/version changes;
- accepted residuals;
- unresolved non-gate work.

### 9.5 Exit statement

The Phase-1 evidence must establish:

- checkpoint/event provenance cannot be forged through a public cache API;
- model-valid exact flows survive realization projection;
- both pilots exactly agree with architecture;
- sponsor multiplicity and ordinary L-BTC rules agree across layers;
- constructibility is architecture-derived;
- semantic relation identity is exact;
- cap-aware quiescence claims are true;
- query reduction cannot produce an invalid rational;
- reorg semantics are explicit and tested;
- checker/report/stamp behavior is mechanically coherent;
- unchanged builds do not rerun represented lint/document work;
- Git revision stamping is coherent;
- every shipped binary has tested output behavior;
- normative and companion prose match typed source;
- every required lane passes;
- the repository is genuinely clean.

Only then does (`gate:phase1:exit`) pass.

---

## 10. Post-Phase-1 boundary · `sec:backlog:post-phase1`

The following work begins **after** Phase 1. It is not part of `F1-017` and must
not delay correctness fixes by expanding current scope.

Allowed before Phase-1 exit:

- dependency review;
- throwaway prototypes;
- exact small-instance oracles;
- synthetic complexity measurements;
- planning corrections.

Not allowed before Phase-1 exit:

- production compiler APIs;
- target fields in realization;
- stable compiler/linker ABI;
- production-marked backend patterns;
- final calibration;
- release capability claims.

---

## 11. Post-Phase-1 dependency tree · `fig:backlog:post-phase1-tree`

```text
PHASE 1 EXIT
│
├── dependency and graph foundation
│   └── C1-004 dependency review
│       ├── C1-005 direct-Petgraph canonical construction
│       │   └── C1-010 typed symbol/SCC prototype
│       │       ├── C1-011 structured relocation
│       │       └── C1-012 bounded-depth taptree
│       └── C1-006 exact keyed linear systems
│           └── C1-007 optional certified numerical prototype
│
├── Phase-2 semantic vocabulary
│   ├── C1-008 exact proof-plan search
│   └── C1-009 execution-case-aware placement
│
└── integration
    └── C1-013 complete small-instance oracle suite
        └── C1-014 preparation review and Phase-2 handoff
```

The critical post-Phase-1 path is:

```text
C1-004 → C1-005 → C1-010 → C1-011/C1-012
```

Proof and placement work waits for Phase-2 relation and carrier types.

Numerical work remains absent unless a concrete consumer exists.

---

## 12. Post-Phase-1 task register · `sec:backlog:c1`

### 12.1 Summary · `tbl:backlog:c1`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `C1-001` | POST | **DONE** | Compiler/linker/mathematics/solver research notes |
| `C1-002` | POST | **DONE** | D007 direct Petgraph decision |
| `C1-003` | POST | **DONE** | D008 exact/certified mathematics decision |
| `C1-004` | POST-P0 | **IN PROGRESS** | Concrete dependency review |
| `C1-005` | POST-P0 | **TODO** | Canonical construction over direct Petgraph graphs |
| `C1-006` | POST-P1 | **TODO** | Exact keyed linear systems and Bareiss elimination |
| `C1-007` | POST-P2 | **BLOCKED** | Certified numerical prototype, only with a real consumer |
| `C1-008` | POST-P0 | **BLOCKED** | Exact proof-plan search |
| `C1-009` | POST-P0 | **BLOCKED** | Execution-case-aware placement |
| `C1-010` | POST-P1 | **TODO** | Typed symbol resolution and SCC policy |
| `C1-011` | POST-P1 | **TODO** | Structured or simultaneous fixed-width relocation |
| `C1-012` | POST-P1 | **TODO** | Deterministic bounded-depth taptree |
| `C1-013` | POST-P1 | **BLOCKED** | Complete algorithm-oracle suite |
| `C1-014` | POST-Gate | **BLOCKED** | Preparation review and Phase-2 handoff |

### C1-004 — Dependency review

Close exact release, feature, MSRV, license, transitive, unsafe, determinism,
parallelism, and advisory review for dependencies actually used.

Current adopted graph dependency:

```text
petgraph 0.8.3
```

Review:

- exact release and upstream source;
- selected features;
- default-feature behavior;
- Rust 1.88 support;
- license;
- transitive graph and duplicates;
- dependency-internal unsafe/SIMD;
- parallel determinism;
- advisory status;
- replacement boundary.

Commands:

```sh
cargo tree --locked -p petgraph -e features
cargo tree --locked -i petgraph
```

`num-rational` and `faer` remain unadopted until a concrete implementation
consumes them.

### C1-005 — Direct-Petgraph canonical construction

Do not create a graph wrapper.

Required shape:

```text
package-owned concrete Petgraph graph
+ typed node/edge weights
+ stable keys
+ canonical insertion
+ key/index lookup metadata
+ canonical stable-key result projection
```

Implement or demonstrate:

- canonical Kahn ordering where canonical topological output is required;
- Petgraph SCC, reachability, and traversal;
- deterministic SCC normalization;
- deterministic cycle diagnostics;
- reverse dependency closure.

Test insertion permutations, unrelated nodes, duplicates, missing endpoints,
self-loops, large SCCs, deep chains, disconnected graphs, and repeated
construction equality.

### C1-006 — Exact keyed linear systems

Implement:

- stable row/column keys;
- exact rational coefficients;
- canonical coefficient DTO;
- exact RHS;
- fraction-free Bareiss elimination;
- exact rank and consistency;
- reduced exact result;
- exact substitution.

Retain a simpler exact-rational Gaussian-elimination oracle.

Test permutations, singular/inconsistent systems, redundant equations,
nonzero equation scaling, exact rational solutions, and coefficient-growth
boundaries.

### C1-007 — Certified numerical analysis

Activate only when a real consumer exists.

If activated:

- select explicit LU/QR/SVD/Cholesky methods by typed problem class;
- record scaling, rank, condition, residual, and backward error;
- keep raw floating values out of semantic identity;
- convert release-sensitive results to exact values or conservative exact
  bounds;
- compare against exact or high-precision oracles.

For \(Ax=b\), record at least:

\[
\eta=\frac{\lVert b-Ax\rVert_\infty}{\lVert A\rVert_\infty\lVert x\rVert_\infty+\lVert b\rVert_\infty}.
\]

A small \(\eta\) is diagnostic evidence, not exact semantic proof.

### C1-008 — Exact proof-plan search

After Phase-2 relation types exist:

- enumerate realization-approved alternatives;
- enforce capability, source, witness, constructibility, representation,
  disclosure, lifecycle, resource, and coverage constraints;
- compute a Pareto frontier;
- select canonically under explicit policy;
- fail with a typed complexity error when search budget is exhausted.

Initial implementation is exact deterministic enumeration or branch-and-bound,
checked against exhaustive small-instance enumeration.

### C1-009 — Execution-case-aware placement

Placement must cover every active execution case:

```text
sponsorless/sponsored
explicit/confidential
continuing/terminal
empty/nonempty
pre-maturity/conversion/post-maturity
```

For each relation \(r\):

\[
\operatorname{requiredCases}(r)\subseteq\bigcup_{c\text{ carries }r}\operatorname{executedCases}(c).
\]

A carrier being present somewhere is insufficient.

Compare exact search with exhaustive carrier-subset oracles.

### C1-010 — Typed symbol resolution and SCC policy

Implement two-pass typed resolution:

1. complete definition census;
2. complete reference resolution.

Use direct Petgraph graphs with typed edge roles.

Normalize SCCs by stable key and require an explicit strategy for every cyclic
edge. SCC membership alone does not authorize a cycle.

### C1-011 — Structured relocation

Prefer structured linking before byte serialization.

If byte relocation is unavoidable:

- fixed width;
- typed source and target;
- exact encoding;
- exact expected placeholder;
- no overlap;
- one resolution per mandatory relocation;
- replacements computed from pristine bytes;
- simultaneous application;
- full post-link reparse and validation.

Compare with an independently constructed expected program.

### C1-012 — Deterministic bounded-depth taptree

Initial objective:

\[
\min\sum_i w_i d_i\qquad\text{subject to}\qquad d_i\le L.
\]

Use deterministic length-limited Huffman/package-merge with stable-key
tie-breaking and exact target branch ordering.

Compare small instances with exhaustive full-binary-tree enumeration.

### C1-013 — Algorithm-oracle suite

Required independent oracles:

| Production analysis | Oracle |
|---|---|
| canonical topological order | valid-order enumeration plus least-key rule |
| SCC | mutual-reachability equivalence |
| interning | non-interned evaluator |
| dependency closure | repeated complete scan |
| exact linear solve | second rational elimination |
| numerical solve | exact/high-precision comparison where applicable |
| proof selection | exhaustive candidate enumeration |
| placement | exhaustive carrier subsets |
| relocation | independently constructed structured output |
| taptree | exhaustive small-tree enumeration |
| calibration | exhaustive finite candidates and monotonicity tests |

### C1-014 — Phase-2 handoff

The preparation lane completes when:

- direct Petgraph policy is demonstrated;
- exact arithmetic policy is demonstrated;
- numerical work is absent or certified;
- proof and placement searches agree with exhaustive oracles;
- symbol/SCC, relocation, and taptree prototypes pass;
- complexity limits fail closed;
- dependency review is complete;
- no local graph, matrix, or solver handle enters semantic identity;
- Phase 1 passed independently.

---

## 13. Research handoff register · `sec:backlog:research`

### 13.1 Compiler-era research · `tbl:backlog:research-compiler`

| ID | Note | Current handoff |
|---|---|---|
| `Q-COMPILER-ALG` | [`compiler-algorithms.md`](research/compiler-algorithms.md) | Rewrite adapter language under D007; then prototype |
| `Q-LINKER-ALG` | [`linker-algorithms.md`](research/linker-algorithms.md) | Rewrite adapter language under D007; then prototype |
| `Q-NUMERICAL` | [`numerical-linear-algebra.md`](research/numerical-linear-algebra.md) | Concrete-consumer and dependency-review gate |
| `Q-OPTIMIZATION` | [`optimization-solvers.md`](research/optimization-solvers.md) | Exact prototype before external solver |

### 13.2 Target research · `tbl:backlog:research-target`

| ID | Note | Blocks |
|---|---|---|
| `Q-STATE` | [`state-constructor.md`](research/state-constructor.md) | STATE ABI and Phase 6 |
| `Q-ARITH` | [`wide-arithmetic.md`](research/wide-arithmetic.md) | Redemption, settlement, cycle |
| `Q-DECLASS` | [`public-declassification.md`](research/public-declassification.md) | Direct private burn/redemption |
| `Q-SETTLE` | [`settlement-layout.md`](research/settlement-layout.md) | Settlement ABI and calibrated bound |

Research may begin early but must not:

- add target fields to realization;
- publish a stable production ABI;
- become release evidence automatically;
- use draft defaults as calibration;
- call model wrappers independent observers;
- put solver output into semantic identity;
- contradict an accepted decision while remaining active.

Accepted results move into typed source, permanent tests, package contracts, and
a decision or ADR where required.

---

## 14. Algorithm and mathematical laws · `sec:backlog:algorithm-laws`

### 14.1 Problem classes · `rule:backlog:problem-classes`

```text
typed semantic AST:
    first-party typed source

graph storage/traversal:
    direct Petgraph graphs

exact semantic arithmetic:
    checked integers, BigInt, reduced exact rationals

numerical linear algebra:
    private working values only when a real consumer exists

proof and placement:
    exact finite search initially

SAT/LP/MILP:
    separate dependency and certificate review after measured need

target tree:
    deterministic bounded-depth coding algorithm

target arithmetic:
    exact target relation plus independent host reference

cryptographic/consensus mathematics:
    reviewed target libraries plus target-native evidence
```

A numerical solve is not proof selection. A graph library is not a semantic
AST. A small floating residual is not exact equality.

### 14.2 Exactness · `rule:backlog:exactness`

Semantic, conservation, authorization, identity, calibration, and release
claims use:

- checked bounded integers;
- arbitrary-precision integers;
- reduced exact rationals;
- exact finite search;
- independently checked certificates;
- target-native execution where the claim concerns the target.

Numerical analysis may produce candidates, diagnostics, sensitivities, and
conservative estimates. A release-sensitive result becomes and is checked as an
exact integer, rational, interval, bound, or certificate.

### 14.3 Identity · `rule:backlog:identity`

Always distinguish:

```text
local handle:
    process-local graph, arena, matrix, or solver position

stable key:
    complete typed semantic identity

digest:
    optional domain-separated commitment
```

Never use as semantic identity:

- Petgraph indices;
- matrix positions;
- solver variable numbers;
- hash-map or traversal order;
- source path or line;
- pivot order;
- floating-point bits;
- iteration count;
- thread schedule;
- temporary path.

### 14.4 Complexity failure · `rule:backlog:complexity`

An analysis exceeding its explicit budget returns a typed complexity error.

It must not:

- drop a relation;
- weaken authorization;
- expose more information silently;
- remove a lifecycle exit;
- switch to a hidden greedy fallback;
- claim optimality from incomplete search;
- accept the best partial result seen so far.

---

## 15. Dependency register · `sec:backlog:dependencies`

### 15.1 Current and candidate dependencies · `tbl:backlog:dependencies`

| Dependency | Status | Role |
|---|---|---|
| `petgraph = 0.8.3` | Adopted; review closure pending | Graph storage and algorithms |
| `num-bigint` | Existing | Exact arbitrary-size integers |
| `num-integer` | Existing | Exact integer helpers |
| `num-traits` | Existing | Numeric traits |
| `num-rational` | Not adopted | Future exact rational analysis |
| `faer` | Not adopted | Future certified numerical diagnostics |
| `fixedbitset` | Deferred | Dense local coverage sets if measured |
| `elements` | Phase-3 review | Target transaction and consensus types |
| `elements-miniscript` | Conditional | Standard target program support |
| `secp256k1-zkp` | Prototype-gated | CT commitments and proofs |
| SAT/LP/MILP solver | Deferred | Large exact planning problems |
| `salsa` | Deferred | Incremental compiler queries |
| `egg` | Deferred | Equality saturation |
| `rayon` | No semantic authorization | Parallel independent work only |

### 15.2 Dependency-entry rule · `rule:backlog:dependency-entry`

A deferred dependency enters only when:

1. a concrete consumer exists;
2. current code demonstrates the missing functionality;
3. simpler exact first-party code is insufficient;
4. purpose, maintenance, source, version, license, MSRV, unsafe boundary,
   transitive graph, determinism, and advisories are reviewed;
5. public API leakage is considered;
6. focused tests and an independent oracle exist;
7. lockfile changes are reviewed;
8. all gates remain green and clean.

Unused dependencies are not added to advertise intent.

---

## 16. Verification matrix · `sec:backlog:verification`

### 16.1 Focused Phase-1 lanes · `tbl:backlog:focused-tests`

| Area | Command |
|---|---|
| architecture | `cargo test --locked -p tripod-architecture` |
| model | `cargo test --locked -p tripod-model` |
| realization | `cargo test --locked -p tripod-realization` |
| labels | `cargo test --locked -p tripod-labels` |
| document stamps | `cargo test --locked -p tripod-document-stamps` |
| CLI common | `cargo test --locked -p cli-common` |
| execwrap | `cargo test --locked -p execwrap` |
| artifacts | `cargo test --locked -p tripod-artifacts` |
| flattener | `cargo test --locked -p flatten-latex-main` |

### 16.2 Required behavioral regressions · `tbl:backlog:regressions`

| Finding | Required named behavior |
|---|---|
| F1-001 | invented row cannot create authenticated index credit |
| F1-002 | partial clear and terminal settlement preserve exact flows |
| F1-003 | every compact-ASH architecture mutation fails |
| F1-004 | two valid sponsor envelopes fail |
| F1-005 | second unchanged build executes no document command |
| F1-006 | driver reaches a progress-maximal typed outcome: an eligible world fully discharges; a blocked world admits its fitting subset and names its residual |
| F1-007/F1-020 | no false stamp and no failure stdout contract breach |
| F1-008 | every malformed import emits one diagnostic |
| F1-009 | selected revision is coherent |
| F1-010 | staged/untracked files fail cleanliness |
| F1-011 | authorization mutation changes constructibility result |
| F1-018 | zero-valued ordinary L-BTC fails |
| F1-019 | every partial render argument set is usage error |
| F1-021 | zero denominator cannot produce `ExactRational` |
| F1-022 | child status and wrapper status are unambiguous |
| F1-023 | both reorg directions have explicit expected semantics |
| F1-024 | every binary’s real CLI contract is executed, TTY refusal PTY-tested and mode-dependent |
| F1-025 | each relation family has exact typed identity |
| F1-026 | represented no-op checks do not rerun or rewrite stamps |
| F1-027 | production `execwrap` rejects the mock-TeX flags |
| F1-028 | oversized report counts fail with a typed overflow, not silent narrowing |
| F1-029 | positive scenarios advance through the invariant-wrapped `apply_checked` path |
| F1-030 | stamp inputs are canonical and committed-blob-bound; a dirty subtree fails |
| F1-031 | a spawn failure never echoes caller-controlled program text |
| F1-032 | a failing external git never relays raw child stderr |
| F1-034 | a staging failure leaves neither render output changed |

### 16.3 Meson and document evidence

```sh
meson compile -C build
meson test -C build --print-errorlogs
scripts/check-document-reproducibility.sh
```

The Meson contract separately proves:

- byte reproducibility;
- command non-execution on no-op;
- output repair;
- failure propagation;
- no undeclared helper dependency;
- compare-if-changed output behavior.

### 16.4 Documentation and census

```sh
scripts/check-plans.sh
git diff --check
git diff --cached --check
```

Every new tracked Rust or documentation subject joins its nearest
`meson.build` census.

### 16.5 Clean repository

The final check covers:

```text
unstaged tracked changes
staged tracked changes
untracked nonignored files
```

Ignored build products are permitted.

---

## 17. Backlog hygiene · `sec:backlog:hygiene`

### 17.1 Adding work · `rule:backlog:add`

A new task states:

- phase or lane;
- priority and status;
- dependencies;
- concrete output;
- assurance class;
- affected files/packages;
- focused exit test;
- verification command;
- identity/schema impact;
- dependency impact.

### 17.2 Splitting work · `rule:backlog:split`

Split a task when it:

- crosses semantic, compiler, linker, target, transaction, evidence, or release
  boundaries;
- has an independently reviewable security consequence;
- mixes exact correctness with numerical diagnostics;
- mixes dependency adoption with algorithm acceptance;
- contains one part that can complete while another remains research-blocked.

### 17.3 Dropping work · `rule:backlog:drop`

A dropped task records:

- why it is unnecessary;
- supporting evidence;
- replacement;
- affected documentation;
- identity and release consequences.

### 17.4 Retention · `rule:backlog:retention`

After a phase baseline:

- compact completed prose into the phase card or release record;
- retain immutable evidence in Git history;
- keep only current and immediately preparatory work here;
- do not create another historical archive under `plans/`.

---

## 18. Execution order · `sec:backlog:order`

Execute in this order unless new evidence changes dependencies:

```text
1. Close F1-001.
2. Close semantic conformance:
       F1-002, F1-003, F1-004, F1-006,
       F1-011, F1-018, F1-021, F1-023, F1-025.
3. Select and implement the report/stamp contract:
       F1-020, then F1-007.
4. Repair document/build contracts:
       F1-005, F1-019, F1-009, F1-026, F1-030, F1-034.
5. Repair repository, CLI, and diagnostic policy:
       F1-008, F1-010, F1-014, F1-015, F1-022, F1-024,
       F1-027, F1-028, F1-029, F1-031, F1-032.
6. Correct documents and planning:
       F1-012, F1-013, F1-016.
7. Prepare the non-self-referential evidence-tag ceremony (F1-033);
       then run and record F1-017.
8. Mark R1-013 and Phase 1 complete only if every required gate passes.
9. Begin post-Phase-1 work with C1-004 and C1-005.
10. Begin production compiler analysis only after the Phase-1 exit.
```

No post-Phase-1 prototype may be used to defer a current correctness fix.

---

## 19. Current completion gate · `gate:backlog:current`

Phase 1 is complete only when:

```text
R1-001 through R1-011 are implementation-complete
R1-012 remains explicitly DROPPED with rationale
F1-001 through F1-016 are closed
F1-018 through F1-034 are closed
F1-017 is DONE
R1-013 is DONE
```

The post-Phase-1 preparation lane is ready for production Phase-2 work only
when:

```text
C1-004 through C1-014 are DONE
```

Allowed deferrals after Phase 1:

- no external optimizer if exact search is sufficient;
- no `faer` without a concrete numerical consumer;
- no sparse or parallel numerical path without measured need;
- no final target/linker interface before real relocatable programs;
- no release identity from prototype artifacts.

Until the Phase-1 gate passes:

- production compiler analysis remains blocked;
- target-specific fields remain forbidden in realization;
- no stable linker or transaction ABI is published;
- no floating-point result becomes semantic identity;
- no draft bound becomes deployment calibration;
- no target prototype becomes release evidence;
- no self-consistent cache is described as independent event provenance.

---

## 20. One-line backlog · `rem:backlog:one-line`

> Close provenance first; preserve exact model flows through realization; make sponsor, constructibility, quiescence, query, CLI, stamp, Git, and reorg contracts mechanically true; correct the documents; record a genuinely clean Phase-1 gate; then begin exact, deterministic, direct-Petgraph compiler and linker work with independent small-instance oracles.