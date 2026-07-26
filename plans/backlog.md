# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 2 — target-independent compiler analysis
> **Current condition:** Adopt the identity-and-digest architecture, complete the current dependency review, reproduce and close the realization-boundary findings, then create the compiler package. No new semantic, report, bundle, ABI, deployment, or release digest may be introduced before its producer, consumer, decision, assurance class, stale condition, and migration rule are explicit.
> **Next gate:** Phase 3 — Elements target and foundational prototypes
> **Authority:** Current execution queue only. Normative specifications, typed architecture, implemented ADRs, accepted decisions, package contracts, research results, phase cards, and the roadmap take precedence.

This file contains:

- the current repository and readiness state;
- compact historical phase and remediation records;
- the active identity-architecture work;
- current static-review findings;
- the Phase-2 implementation queue;
- compiler-algorithm preparation;
- verification and clean-tree gates.

It does not retain implementation diaries. Git history, annotated tags, phase
cards, and accepted ADRs preserve historical detail.

Long-term sequencing is owned by [roadmap.md](roadmap.md). Package boundaries
are owned by [packages/](packages/README.md). Accepted implementation choices
are owned by [decisions/](decisions/README.md). Unresolved prototypes are owned
by [research/](research/README.md).

---

## 1. Backlog contract · `sec:backlog:contract`

### 1.1 Status vocabulary · `tbl:backlog:status`

| Status | Meaning |
|---|---|
| **TODO** | Ready when its named dependencies are complete. |
| **IN PROGRESS** | Actively being implemented, reviewed, or verified. |
| **BLOCKED** | A named dependency prevents safe progress. |
| **PARKED** | Deliberately inactive until a concrete consumer exists. |
| **DONE** | Implementation, focused tests, complete required gates, documentation, and clean-tree evidence are recorded. |
| **DROPPED** | Deliberately not implemented; rationale and replacement are recorded. |
| **SUPERSEDED** | Replaced by a named task, ADR, decision, or package contract. |
| **HISTORICAL** | Immutable evidence about an earlier commit; not a claim about the current checkout. |

Code resembling the intended result is not sufficient for `DONE`.

A static-review finding remains open until one of the following occurs:

- implementation plus a focused regression;
- a typed proof that the reported state is unconstructible;
- reproduction showing the finding is false;
- an approved correction to the owning policy or assurance claim.

“Existing tests pass” is not closure unless a named test reaches the reported
path.

### 1.2 Priority vocabulary · `tbl:backlog:priority`

| Priority | Meaning |
|---|---|
| **P0** | Can manufacture false release, deployment, provenance, or semantic evidence. |
| **P1** | Phase-gate blocker or trusted semantic/build boundary defect. |
| **P2** | Required correctness, identity, dependency, or determinism work before the active phase exits. |
| **P3** | Maintainability or evidence-quality work required by the active gate. |
| **POST** | Later-phase work that does not block the active phase. |

### 1.3 Task families · `tbl:backlog:families`

| Prefix | Owner |
|---|---|
| `F1` | Historical Phase-1 findings |
| `F2` | Historical post-Phase-1 remediation |
| `F3` | Historical static-review remediation closed before this rewrite |
| `F4` | Current static-review findings |
| `I1` | Identity, digest, evidence-binding, and release-root architecture |
| `P2` | Phase-2 compiler implementation |
| `C1` | Compiler/linker algorithm and dependency preparation |
| `A17` | ADR-017 path-scope and host-filesystem-trust implementation |
| `R` | Path-scope-revision static review findings |

Task identifiers are permanent and never reused.

### 1.4 Definition of done · `rule:backlog:done`

An implementation task is `DONE` only when it records:

1. implementing source files;
2. focused positive and negative coverage;
3. affected ADRs, decisions, package contracts, or phase cards;
4. exact verification commands and results;
5. generated-publication and label impact;
6. semantic, identity, schema, and migration impact;
7. final clean-tree output.

A dependency-review task additionally records:

- selected version and source;
- features;
- transitive graph;
- license;
- MSRV;
- unsafe boundary;
- determinism and parallelism implications;
- advisory status;
- lockfile impact.

A research task additionally records:

- exact question and constraints;
- prototype target and tool versions;
- accepted and rejected candidates;
- positive and negative evidence;
- measurements;
- permanent implementation handoff.

### 1.5 Authority and machine use · `rule:backlog:authority`

Planning labels and task identifiers are non-normative and
non-identity-bearing.

This file is not compiler, linker, target, ABI, deployment, evidence, or
release input. Implemented policy moves into typed source, tests, configuration,
and ADRs.

---

## 2. Current repository state · `sec:backlog:state`

### 2.1 Review provenance · `tbl:backlog:review-provenance`

This rewrite is based on static review of the supplied repository snapshot:

```text
reviewed tree:
   

selected authored files:
    331

submodules:
    none

symlinks in supplied tree:
    none
```

The supplied content excluded:

```text
Cargo.lock
LICENSE-CODE
LICENSE-DOCS
archive/
```

The lockfile exists in the repository but was excluded from the supplied
content review.

No build, test, target execution, advisory scan, or reproduction command was
run as part of this static review. Therefore:

- F4 findings are source-review findings until reproduced;
- historical green gates remain historical evidence;
- this tree is not declared green by this review;
- P2 dependency review cannot close from this snapshot;
- every implementation series must record its own complete execution result.

The earlier F3 review basis remains historical and must not be described as the
current tree.

### 2.2 Implemented packages · `tbl:backlog:implemented`

| Package or area | Current source state |
|---|---|
| Layer 0 | Published specification, version `0.5.1` |
| Realization document | Realization with final architecture appendix |
| `architecture` | Typed architecture, validation, semantic/behavioural identities, deployment-profile scaffolding |
| `model` | Executable state machine, invariant checker, property/corruption suites, indexer and accounting projections |
| `realization` | Target-independent typed semantics for compact ASH and live receipt transfer |
| `artifacts` | Generated-publication writer/checker and document weld |
| `labels` | Owner-aware Markdown/Rust label graph, census, plan, and publication checks |
| `cli-common` | ADR-010 command handling and checker report/stamp infrastructure |
| `document-stamps` | Git-derived paper metadata and deterministic publication inputs |
| `execwrap` | Byte-preserving process wrapper; mocked TeX isolated separately |
| `flatten-latex-main` | Deterministic atomic allowlist-based LaTeX flattener |
| Meson | Explicit source census, stamp-backed checks, mocked document graph |
| Security policy | Public-data interfaces and external execution-environment boundary |

### 2.3 Current published identities · `tbl:backlog:identities`

| Identity | Current value |
|---|---|
| Layer-0 version | `0.5.1` |
| Realization version | tracked compiler-line binding |
| Architecture schema | `17` |
| Architecture semantic algorithm | `sha256-canonical-json-v2` |
| Architecture semantic hash | |
| Architecture behavioural algorithm | `sha256-canonical-json-behavioural-v3` |
| Architecture behavioural hash | |
| Layer-0 anchor-set hash | |
| Attestation wire schema | `13` |
| Deployment-profile schema | `2` |
| Cargo workspace version | `0.1.0` |
| Meson project version | `0.0.1` |

The architecture is final as an architecture publication. That does not imply
compiler completeness, target support, deployment evidence, or deployment
readiness.

### 2.4 Not implemented · `tbl:backlog:not-implemented`

```text
tripod-compiler
tripod-target-elements
tripod-tapscript
tripod-linker
tripod-transaction
tripod-vectors
tripod-release

independent deployment observers
production deployment
production signer or wallet
production key management
```

### 2.5 Readiness statement · `rem:backlog:readiness`

```text
Attestation specification:             published
Realization contract:                 published
Typed architecture:                   final and pinned
Executable model:                     implemented
Typed realization pilots:             implemented
Phase-1 gate:                          historical tagged evidence
F3 remediation:                       recorded closed historically
Identity/digest architecture:         adopted; inventory and future DAG recorded
Static-review findings:               all resolved (F4 register closed)
Phase-2 dependency review:            complete (Petgraph; P2-002/C1-004)
Phase-2 compiler package:             created; analysis absent
Target/backend/linker/transaction:     absent
Independent deployment evidence:      absent
Production deployment:                absent
```

Current packages are public-data tools. They do not legitimately accept private
keys, wallet secrets, signing nonces, blinding factors, private openings,
credentials, or production authority.

A green model does not prove target correctness. A self-consistent report does
not prove independent target-chain provenance. Architecture finality does not
imply deployment readiness.

---

## 3. Historical record · `gate:backlog:phase1`

### 3.1 Phase evidence

The repository records these immutable historical milestones:

```text

Phase-1 realization foundation:
    phase1-realization-foundation-v1
```

The phase cards own their exact evidence and scope:

- [Phase 0](phases/00-baseline.md);
- [Phase 1](phases/01-realization.md).

Later findings do not rewrite those tags. They refine what may be claimed about
later source.

### 3.2 Historical remediation

The permanent historical finding families remain:

| Family | Status | Record |
|---|---|---|
| `F1` | HISTORICAL | Phase-1 remediation and evidence |
| `F2` | HISTORICAL | Post-Phase-1 boundary and policy remediation |
| `F3-001` through `F3-010` | DONE in recorded history | Meson output selection, realization immutability, flattener confinement, plan census, canonical projection, ownership validation, diagnostics, export ordering, conditional flattening, and strict checker stamps |

These records are retained by Git history and the previous backlog revisions.
This static review did not rerun their evidence.

---

## 4. Identity and digest architecture · `sec:backlog:identity`

The immediate policy owner is
[ADR-016](../adr/016-semantic-identities-and-evidence-binding.md).

The governing rule is:

> No named consumer, no digest. No distinct decision, no digest. No independent lifecycle, no child identity.

Types establish representable shape. Validators establish object validity.
Tests, proofs, and target execution provide scoped correctness evidence.
Semantic identities compare canonical meanings. Artifact digests compare exact
bytes. Report identities bind evidence roles to exact subjects. None of these
mechanisms substitutes for another.

I1-001 through I1-003 are closed, so the identity freeze is lifted. New
identity-bearing work is admitted only under
(`[ADR016-rule:identity:admission]`), against the activation conditions the
register records. I1-004 through I1-006 remain parked or blocked on their own
consumers.

### 4.1 Current digest inventory · `tbl:backlog:digest-inventory`

| Current identity or digest | Producer | Consumer | Decision and assurance |
|---|---|---|---|
| Git commit/tree IDs | Git | document stamps and publication tooling | Source provenance only |
| Document UUID | `document-stamps` | PDF XMP | Exact paper-input provenance |
| Instance UUID | Git tree derivation | PDF XMP | Paper-subtree instance provenance |
| Layer-0 anchor-set hash | labels/architecture | label and architecture weld | Exact imported Layer-0 dependency set |
| Architecture semantic hash | architecture | realization binding, query context, profile, document/artifact weld | Canonical complete architecture meaning |
| Architecture behavioural hash | architecture | versioning gate | Realization-major stability only |
| Generated-file exact comparison | artifact/label checkers | CI and Meson | Publication freshness; no additional digest required |
| Deployment-profile hash | architecture | tests; future release consumer | Dormant aggregate profile identity |
| Profile artifact/report hash fields | future release producer | profile validation currently checks presence/binding selectively | Provisional pre-production references; recipes and typed roles incomplete |

Document provenance identities must not enter protocol, realization, compiler,
target, bundle, ABI, or deployment semantics.

The behavioural hash remains a narrow versioning witness. It must not become a
second general architecture identity repeated through every future artifact.

### I1-001 — Adopt ADR-016 · `task:identity:adopt-policy`

**Priority:** P1
**Status:** DONE
**Blocks:** new persistent compiler/report/bundle/ABI identities

Required:

- add ADR-016 to the ADR census and index;
- classify semantic identity, artifact digest, provenance identity, report
  identity, deployment identity, and release identity;
- require the digest-admission record of
  (`[ADR016-rule:identity:admission]`);
- prohibit field-level digest proliferation;
- prohibit hash matching as a replacement for validation;
- prohibit hash inequality as evidence of independence;
- establish immediate dependency edges;
- establish one future release-manifest root.

Exit:

- [x] ADR status and implementation scope are explicit;
- [x] ADR census and labels pass;
- [x] planning identity policy cites rather than restates the ADR;
- [x] no current digest is silently reinterpreted.

**Evidence (2026-07-24):** ADR-016 added at
[016-semantic-identities-and-evidence-binding.md](../adr/016-semantic-identities-and-evidence-binding.md),
wired into `adr/meson.build` and `adr/README.md`. Verified via the SDK build:
census-audit `valid:true` (declared 274 = subjects 274, no `missing_from_census`
or `not_tracked`); check-labels `valid:true` (adr_labels 101, all imported
citations resolve); check-plans reports a valid documentation tree. Planning
identity policy (section 4) cites ADR-016 rather than restating it, and the
section 4.1 inventory reinterprets no existing digest.

### I1-002 — Complete the current identity inventory · `task:identity:inventory`

**Priority:** P1
**Status:** DONE
**Depends on:** I1-001

Update [the identity register](registers/identities.md) so every current digest
records:

```text
typed object or exact bytes
owner
producer
consumer
decision
assurance class
stale condition
recipe
migration
non-claims
status: active, publication-only, dormant, provisional, or historical
```

Required decisions:

- keep active identities with real consumers;
- keep publication-only identities out of semantic flow;
- mark deployment-profile identity dormant until release consumes it;
- keep exact generated-byte checks without adding redundant hashes;
- identify raw profile hash fields as provisional pre-production references;
- remove or defer any identity with no present consumer.

Exit:

- [x] every current digest has one classified purpose;
- [x] no digest has two incompatible meanings;
- [x] dormant and provisional identities are visibly non-release-ready;
- [x] the register remains planning-only.

#### Resolution (2026-07-25)

The register now inventories nine identity-bearing mechanisms, each with the
eleven required fields, against the sources rather than against this backlog:
Git object IDs; document identity; paper instance identity; the Layer-0
anchor-set hash; the architecture semantic hash; the architecture behavioural
hash; generated-file exact comparison; the deployment-profile hash; and the raw
profile artifact and report hash fields.

Classification outcome: three active (anchor-set, semantic, behavioural), three
publication-only (Git object IDs, document identity, instance identity), one
active but deliberately digest-free (generated-file comparison), one dormant
(deployment-profile hash, consumed only by architecture tests), and one
provisional (raw profile hash fields, whose recipes and typed roles are
undefined). Nothing was removed: every entry has a present or explicitly
deferred consumer.

Two boundaries are recorded that the previous table left implicit. The document
and instance identities answer different provenance questions — declared input
set against paper-subtree state — so neither substitutes for the other. The
generated-file check is byte equality, strictly stronger than a digest over the
same bytes, so adding a hash beside it would mint a redundant identity with no
consumer.

The forward-looking ownership rows are retained but separated from the current
inventory and marked unminted, with activation deferred to I1-003. Petgraph
indices moved out of the identity table into an explicit local-handle rule.

Source: `plans/registers/identities.md`. The register remains planning-only and
is not toolchain input.

### I1-003 — Define the future immediate-edge identity DAG · `task:identity:future-dag`

**Priority:** P1
**Status:** DONE
**Depends on:** I1-001 and I1-002
**Blocks:** public compiler identity and downstream identity fields

Define activation points and immediate consumers for:

```text
ArchitectureSemanticId
    → RealizationId
    → CompilerPlanId
    → TargetPlanId
    → LinkedBundleId
    → TransactionAbiId
    → DeploymentProfileId
    → ReleaseManifestId
```

Rules:

- no future identity is minted before a real consumer exists;
- a parent binds only immediate dependencies;
- transitive dependencies are not repeated as an all-to-all hash mesh;
- canonical projections contain no local graph indices, source order, paths,
  line numbers, thread schedules, temporary paths, or floating working values;
- a child receives its own identity only when separately consumed, transported,
  cached, signed, versioned, or published.

Phase-2 may proceed without minting a public realization or compiler hash. If
no persistent cross-process consumer exists, typed comparison remains the
boundary.

Exit:

- [x] every proposed identity has a named activation phase and consumer;
- [x] every edge states its assurance and non-claims;
- [x] no speculative hash field enters compiler core;
- [x] migration rules are defined before publication.

#### Resolution (2026-07-25)

The identity register now carries the DAG. Each of the eight links records an
activation phase, the immediate consumer whose existence is the activation
condition, the immediate edges it binds, its assurance, its non-claims, and
whether any child receives its own identity.

The activation conditions are boundary conditions, not dates: an activation
phase is permission, and reaching it without the named consumer does not
activate the identity. On that reading the realization identity does not
activate in Phase 2, because the compiler consumes realization as an in-process
typed value rather than as external bytes, and the compiler plan identity
activates only on a real cross-process cache, published artifact, or separately
versioned backend consumer.

The projection exclusions are stated once as an activation rule rather than
repeated per edge, and cover graph indices, source order, paths, line numbers,
solver variable numbers, matrix positions, traversal order, thread schedules,
temporary paths, and floating working values.

Two consequences are recorded explicitly. A field reserved in compiler core for
a future digest is itself a speculative identity, so Phase 2 ships none and
typed comparison remains the boundary. The linked bundle is where semantic
identity and artifact digest meet, and the two remain separate entries with
neither substituting for the other.

Migration is fixed before any publication: every identity carries a recipe
identifier from its first publication, and any change of projection, encoding,
domain separator, algorithm, included fields, or exclusion rules mints a new
recipe identifier rather than redefining the published one.

Source: `plans/registers/identities.md`.

### I1-004 — Define typed evidence envelopes · `task:identity:evidence-envelopes`

**Priority:** P2
**Status:** PARKED until a persistent report consumer is implemented
**Must complete before:** release-used compiler, target, calibration, or observer report identities

Replace ambiguous bare report digests with typed references carrying at least:

```text
evidence role
report schema
subject identities
producer or implementation identity
configuration identity where relevant
result status
canonical payload or payload digest
```

Independence remains a reviewed provenance claim. Different report hashes do
not prove independent implementation.

If Phase 2 emits only ephemeral local diagnostics, no report identity is
required.

### I1-005 — Migrate deployment-profile evidence before production · `task:identity:profile-migration`

**Priority:** POST
**Status:** BLOCKED on implemented bundle, ABI, and evidence types

Before production:

- bind calibration to the exact final linked bundle and transaction ABI;
- replace raw artifact/report hash arrays with typed identities or artifact
  references;
- define every artifact digest recipe;
- bind report roles and exact subjects;
- retain separate event, query, accounting, and script-integration claims;
- revise deployment-profile schema rather than appending ambiguous raw fields.

The documented schema-2 ABI-binding limitation remains release-blocking.

### I1-006 — Define one release root · `task:identity:release-root`

**Priority:** POST
**Status:** BLOCKED on the release package

The release manifest is the sole aggregate release root. It binds:

- deployment profile;
- required evidence references;
- distributed artifact roles, canonical paths, schemas, and byte digests;
- release policy;
- explicit source revision;
- explicit release date.

If release signing is introduced, sign the release-manifest identity. Do not
sign every internal field or intermediate object separately without a distinct
authority boundary.

---

## 5. Current static-review findings · `sec:backlog:findings`

These findings were identified by static review of the supplied tree. They are
not reproduced execution results.

### 5.1 Summary · `tbl:backlog:findings`

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `F4-001` | P1 | DONE | Expression-predicate relations are not cross-validated against the expression graph during realization derivation. |
| `F4-002` | P1 | DONE | Lifecycle edges and paths lack generic semantic-shape validation. |
| `F4-003` | P2 | DROPPED | The strict empty-stamp rule is bypassed by shell-produced generator and publication stamps. |
| `F4-004` | P2 | DONE | Unknown or malformed bracket-free owner-qualified PLAN/DOC labels may be silently ignored. |
| `F4-005` | P2 | DONE | The ADR owner shares the bracket-free owner-token hole and lacks the known-owner check entirely. |

### F4-001 — Validate relation-to-expression binding · `task:findings:predicate-binding`

**Priority:** P1
**Status:** DONE
**Owners:** `realization`, future compiler
**Blocks:** trusted compiler input boundary

#### Static basis

`validate_operation_ownership` validates relation IDs and proof-alternative
bindings, but does not inspect:

```rust
Relation::ExpressionPredicate { expression }
```

`build_relation_graph` has no expression registry, and scoped validation does
not require that the predicate expression:

- exists;
- belongs to the same operation;
- has semantic type `Bool`.

A malformed realization may therefore derive and fail only when evaluated.

#### Required implementation

Before returning a validated realization:

- every expression-predicate relation resolves to exactly one expression;
- predicate expression ownership matches the relation operation;
- predicate expression type is `Bool`;
- missing, foreign, or non-boolean predicates return focused typed errors.

#### Required tests

- undeclared predicate expression;
- predicate expression from another operation;
- non-boolean predicate;
- valid same-operation boolean predicate;
- declaration-order permutation;
- model conformance remains unchanged.

#### Exit

- [x] finding reproduced or disproved;
- [x] validation occurs during derivation, not first evaluation;
- [x] focused mutation tests pass;
- [x] realization, model conformance, and complete gates pass cleanly.

#### Resolution (2026-07-24)

Reproduced CONFIRMED, then fixed. Reproduction: `validate_operation_ownership`
never matched `Relation::ExpressionPredicate`, `build_relation_graph` took no
expression registry, and `validate_scoped_realization` did not inspect the
variant, so existence, same-operation ownership, and `Bool` type were all
deferred to `evaluate_operation` — a malformed predicate derived `Ok`.

Fix: a new `validate_predicate_bindings` pass, invoked from
`validate_operation_ownership` (per operation, before graph assembly), checks
each predicate against the operation's own declared expressions: foreign
targets report `ForeignExpressionOwnership`; undeclared targets report the new
`UnknownPredicateExpression`; non-boolean targets report the new
`NonBooleanPredicateExpression`. Same-operation ownership makes the operation's
declared set the complete registry.

Source: `packages/realization/src/validate.rs` (new pass and errors),
`packages/realization/src/error.rs` (two focused variants). Tests
(`packages/realization/src/tests/expression_tests.rs`): undeclared, foreign,
non-boolean, valid same-operation boolean, and expression-permutation cases.
Model conformance and existing pilots are unchanged.

### F4-002 — Validate lifecycle graph semantics · `task:findings:lifecycle-graph`

**Priority:** P1
**Status:** DONE
**Owners:** `realization`, future compiler
**Blocks:** lifecycle claims used by proof planning

#### Static basis

The lifecycle graph validates node/edge uniqueness, endpoint existence,
acyclicity, and reachability. It does not generically require a
`RequiresExit` edge to have the semantic shape:

```text
Representation { object: O, mode: M }
    →
RequiredExit { object: O, operation: E }
```

General reachability can therefore pass through reversed, cross-object, or
otherwise unrelated lifecycle nodes.

Lifecycle declarations are also absent from the generic per-operation
ownership pass.

#### Required implementation

For the current graph vocabulary:

- source must be a representation node;
- target must be a required-exit node;
- source and target object IDs must match;
- reversed edges reject;
- cross-object edges reject;
- required-exit-to-required-exit edges reject;
- approved cross-operation exit operations remain typed semantic payloads;
- operation declaration ownership is validated before graph assembly.

If intermediate lifecycle reasoning is later required, introduce explicit node
and edge kinds rather than weakening the current edge.

#### Required tests

- reversed edge;
- cross-object edge;
- required-exit-to-required-exit edge;
- unrelated intermediate path;
- valid ASH compact/clear paths;
- valid live transfer/burn/redeem paths;
- declaration-order permutation.

#### Exit

- [x] finding reproduced or disproved;
- [x] lifecycle reachability has semantic edge validation;
- [x] legitimate cross-operation exits remain representable;
- [x] realization and complete gates pass cleanly.

#### Resolution (2026-07-24)

Reproduced CONFIRMED, then fixed. Reproduction: `LifecycleDependencyDeclaration`
endpoints are the untyped `LifecycleNodeId` enum, so reversed
(`RequiredExit -> Representation`), representation-to-representation,
exit-to-exit, and cross-object edges were all representable and passed
`build_lifecycle_graph`, which checked only uniqueness, endpoint existence, and
acyclicity. General reachability could then satisfy a required exit through a
spurious edge.

Fix: a new `validate_lifecycle_edge_shape` pass in `build_lifecycle_graph`
requires each `RequiresExit` edge to run `Representation -> RequiredExit` over
one object; reversed, representation-to-representation, and exit-to-exit edges
report the new `MalformedLifecycleEdge`, and object mismatch reports the new
`CrossObjectLifecycleEdge`. Legitimate cross-operation exits stay representable
because the exit operation remains a typed payload of `RequiredExit` with a
matching object. With the shape enforced the graph is bipartite and cannot
cycle, so the retained cycle guard is now defensive for any future edge kind;
lifecycle stays outside the per-operation ownership pass by design, since a
`RequiredExit` names its exit operation as content, not an owner.

Source: `packages/realization/src/lifecycle.rs` (shape pass),
`packages/realization/src/error.rs` (two focused variants). Tests
(`packages/realization/src/tests/lifecycle_tests.rs`): reversed,
representation-to-representation, exit-to-exit, cross-object, and well-shaped
acceptance; the derived-pilot path test continues to pass for ASH
compact/clear and live transfer/burn/redeem.

### F4-003 — Apply strict empty-stamp policy to every stamp · `task:findings:stamp-contract`

**Priority:** P2
**Status:** DROPPED
**Owners:** `cli-common`, Meson, publication scripts
**Policy:** ADR-014

#### Resolution (2026-07-24) — REFUTED by reproduction

Dropped: reproduction showed the finding is false. The shell-`touch` sites do
exist (`meson.build` `generator_wrap` and `cargo_quiet_stamp_wrap`,
`scripts/sync-publication.sh`) and would re-date a nonempty file, but none of
them is an ADR-014 `--stamp`-argument stamp. Every stamp the empty-byte
contract governs — the checker `--stamp` outputs (`labels.ok`, `generated.ok`,
`plans.ok`, `forbidden-text.ok`) — routes through `cli_common::touch_stamp`.
The shell-`touch` outputs are `build_always_stale: true` build-dir markers
whose bytes and mtime are never consulted as a freshness oracle, and ADR-014
restates the empty-byte refusal only for the checker `--stamp` argument; its
generator-stamp rule deliberately keeps a committed publication out of the
declared build-dir outputs. The two sets are disjoint, so there is no policy
bypass to fix. The finding conflated "produced by shell `touch`" with "subject
to the strict empty-stamp policy".

#### Original static basis (retained for the record)

`cli_common::touch_stamp` rejects a nonempty stamp without truncating it.

Generator and publication paths still use shell `touch`, including:

```text
root Meson generator wrapper
scripts/sync-publication.sh
```

Those paths accept and re-date a nonempty existing stamp, contradicting the
strict empty-stamp policy.

#### Original required implementation (not pursued)

Route every first-party stamp mutation through one shared implementation or one
equivalent strict rule:

- absent stamp creates an empty regular file;
- existing empty stamp is re-dated;
- existing nonempty stamp fails without truncation;
- foreign bytes and mtime survive refusal;
- alias and type checks precede side effects;
- report/publication/generation failure never produces a fresh stamp.

For publication mirrors, validate the stamp before changing the destination.

#### Required tests

Cover checker, generator, and mirror stamps under:

- absent;
- existing empty;
- existing nonempty;
- non-regular destination;
- aliased role;
- operation failure;
- no-op Ninja restat behavior.

#### Exit

- [ ] all stamp producers implement one rule;
- [ ] nonempty refusal precedes side effects;
- [ ] mocked Meson repair/restat/failure tests pass;
- [ ] complete Meson and clean-tree gates pass.

### F4-004 — Reject malformed owner-qualified PLAN/DOC tokens · `task:findings:owner-token`

**Priority:** P2
**Status:** DONE
**Owner:** `labels`
**Policy:** ADR-013

#### Static basis

PLAN and DOC harvesting recognizes known valid imported owners without square
brackets, but a malformed or unknown owner-qualified token can fail both
imported and local parsing and then be silently ignored as ordinary inline
code.

Examples include malformed ADR widths, mistyped prefixes, and unknown uppercase
owner prefixes.

#### Required implementation

Outside fenced and double-backtick examples:

- known imported owner without square brackets:
  `InvalidImportedCitationForm`;
- unknown or malformed owner-qualified label-like token:
  `UnknownOwner` or a focused equivalent;
- valid local labels remain local;
- ordinary inline code remains nonparticipating.

#### Required tests

- valid local label;
- valid imported label;
- known owner without brackets;
- malformed ADR owner width;
- mistyped known owner;
- unknown uppercase owner;
- fenced example;
- double-backtick example;
- deterministic diagnostic ordering.

#### Exit

- [x] finding reproduced or disproved;
- [x] malformed cross-owner forms fail closed;
- [x] registers remain current;
- [x] label, plan, census, and complete gates pass.

#### Resolution (2026-07-25)

Reproduced CONFIRMED, then fixed. Reproduction: with the fix reverted, a plan
file carrying a short ADR owner width, a mistyped known owner, an unknown
uppercase owner, and a known owner with a malformed local label produced one
diagnostic — the bracket-free known-owner case already caught by
`looks_imported`. The other four fell through the planning-shape parse and were
discarded as ordinary inline code.

Fix: `harvest_markdown_owner` now classifies every bracket-free token through a
shared `diagnose_bracket_free_owner_token` before local parsing is attempted. A
token naming a known owner reports `InvalidImportedCitationForm`; a token whose
uppercase or numeric owner prefix precedes a colon-bearing label-like remainder,
but which names no known owner or carries a malformed local label, reports
`UnknownOwner`. The label-like test is what keeps ordinary hyphenated inline
code out: a token with no colon in its remainder never participates. No valid
local label can reach either arm, because every label segment is lowercase and
the segment before a token's first hyphen therefore fails the owner-prefix test.

Source: `packages/labels/src/repository.rs` (shared classifier plus
`looks_owner_qualified_label`). Tests (`packages/labels/src/tests.rs`): one
exhaustive PLAN fixture pinning the diagnostic set and its order across a valid
local mint, a valid import, a bracket-free known owner, a short ADR width, a
mistyped owner, an unknown owner, a malformed local label, ordinary inline code,
a double-backtick example, and a fenced example; plus a DOC fixture proving the
shared harvest behaves identically for both Markdown owners.

### F4-005 — Reject malformed owner-qualified ADR tokens · `task:findings:adr-owner-token`

**Priority:** P2
**Status:** DONE
**Lane:** current static-review remediation
**Owner:** `labels`
**Policy:** ADR-013
**Depends on:** F4-004
**Assurance:** focused unit tests plus the complete label and census gates
**Identity and schema impact:** none; diagnostics only, no digest or register
schema change
**Dependency impact:** none

#### Basis

Found while reproducing F4-004, not by the supplied static review. The ADR
harvest shares the finding and is strictly weaker: it recognizes bracketed
imports and local ADR-shaped labels, but has no bracket-free known-owner check
at all, so both a known owner written without brackets and an unknown or
malformed owner-qualified token were silently discarded.

Entered as its own identifier under the split rule rather than widened into
F4-004, because F4-004's static basis, required tests, and exit name the PLAN
and DOC owners only.

#### Required implementation

The ADR owner applies the same bracket-free classifier as PLAN and DOC, with
its own ADR local-label shape unchanged.

#### Required tests

- known owner without brackets in an ADR;
- mistyped known owner in an ADR;
- valid local ADR label unaffected;
- ordinary inline code nonparticipating;
- deterministic diagnostic ordering.

#### Verification

```text
cargo test -p tripod-labels
meson test -C build
```

#### Exit

- [x] finding reproduced;
- [x] malformed cross-owner forms fail closed for the ADR owner;
- [x] registers remain current;
- [x] label, plan, census, and complete gates pass.

#### Resolution (2026-07-25)

Reproduced CONFIRMED by fixture probe: an ADR containing a bracket-free
`A-` citation and a mistyped `PLN-` owner produced no diagnostic at all.

Fix: `harvest_adrs` calls the same `diagnose_bracket_free_owner_token` on the
same terms as the Markdown owners. One classifier now serves every Markdown
owner, so a future owner cannot reintroduce the hole by omission.

Source: `packages/labels/src/repository.rs`. Test
(`packages/labels/src/tests.rs`): an ADR fixture pinning both diagnostics and
their order alongside an unaffected local mint and ordinary inline code.

### 5.2 Path-scope-revision review · `tbl:backlog:findings-r`

A second static review of the tree at the compiler-crate commit, taken before
the ADR-017 series landed. Its identifiers are the reviewer's own and are
retained verbatim.

The review excluded the lockfile and licence files, so it verifies no locked
resolution, advisory status, checksum, provenance, licence compatibility, or
execution result. It reports no clear static defect permitting an invalid
protocol transition in the implemented model, and reclassifies its predecessor's
output-path finding as over-scoped.

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `R1` | P1 | DONE | Filesystem checks exceed the intended trust boundary while the central Git-mode invariant is unenforced. |
| `R2` | P1 | DONE | ADR-016 is marked Proposed while the repository treats it as adopted policy. |
| `R3` | P1 | TODO | D007 requires a full Petgraph feature surface, contradicting Cargo and the completed dependency review. |
| `R4` | P2 | TODO | `scripts/ci.sh` reports `CI green` after skipping the mocked Meson contract lane. |
| `R5` | P2 | TODO | The documented document-identity recipe says declared order; the implementation sorts by canonical path. |
| `R6` | P2 | TODO | Several planning documents still describe the compiler package as merely planned. |

### R1 — Path handling exceeds the assurance boundary · `task:review:path-scope`

**Priority:** P1
**Status:** DONE
**Owner:** `labels`, `cli-common`, `flatten-latex-main`, `execwrap`, ADR-014,
ADR-015
**Policy:** ADR-017

#### Resolution — closed before the review was received

This finding was already remediated by the A17 series, which landed after the
reviewed commit. Every recommendation is satisfied, and the mapping is exact:

- central mode audit consuming a mode-bearing listing and rejecting any mode
  other than `100644` or `100755`, over the complete tracked set including
  lint-excluded paths — A17-001;
- generic destination identity reduced to lexical normalization, with
  device/inode and hard-link comparison removed — A17-002;
- flattener ancestor walk removed while allowlist resolution, absolute and
  parent-traversal rejection, ambiguity rejection, cycle detection, strict
  bibliography handling, and atomic staging all remain — A17-003;
- `execwrap` retained as an operation-specific check with its hazard and
  residual host race documented, explicitly not repository-wide policy —
  A17-004;
- ADR-014 and ADR-015 edited directly to own tracked modes and the
  repository/build-root boundary, with ADR-017 coordinating rather than
  superseding — the ADR-017 landing commit.

The review's independent finding that the supplied tree contains no symlink,
gitlink, or submodule agrees with the audit's own evidence on the current tree:
342 tracked entries, every mode `100644` or `100755`.

One point is recorded as a difference rather than a gap. The review suggests
removing the tests that asserted the generic alias guarantees. They were
instead rewritten to assert the new behaviour and state why, because a deleted
test leaves the next reader free to restore the check as a supposed fix.

### R2 — ADR-016 status contradicted its use · `task:review:adr016-status`

**Priority:** P1
**Status:** DONE
**Owner:** ADR-016, `adr/README.md`

#### Basis

The record carried `Proposed` while the backlog recorded the identity
architecture as adopted, closed I1-001, lifted the identity freeze, and
governed new digests by the record's own admission rule; the identity register
named it the active policy owner. Under the repository's authority order an
implemented ADR outranks planning prose, so a planning document cannot make a
proposed record current policy. The contradiction mattered because this record
governs whether future compiler, target, report, bundle, ABI, deployment, and
release identities may be introduced at all.

#### Resolution (2026-07-26)

The content was already adopted, so the status was stale rather than the plans
overclaiming. ADR-016 now reads: decided and implemented for current identity
policy, with the evidence-envelope, profile-migration, and release-root
portions activating with their named consumers. That phrasing keeps the
distinction the original `Proposed` was reaching for — parts of the
deployment-profile and report design are not implemented — without leaving the
authoritative record weaker than the policy it governs.

`adr/README.md` carries the same status in the same change.

---

## 6. Phase-2 implementation register · `sec:backlog:phase2`

### 6.1 Summary · `tbl:backlog:phase2`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `P2-001` | P1 | DONE | Immutable, canonical, ownership-validated realization boundary |
| `P2-002` | P2 | DONE | Concrete Petgraph dependency and lockfile review |
| `P2-003` | P1 | DONE | Create `tripod-compiler` |
| `P2-004` | P1 | BLOCKED | Bind architecture, realization, policy, and explicit scope |
| `P2-005` | P1 | BLOCKED | Canonical relation DAG over direct Petgraph |
| `P2-006` | P1 | BLOCKED | Checked constant folding preserving failure semantics |
| `P2-007` | P1 | BLOCKED | Exact proof-alternative and target-requirement planning |
| `P2-008` | P1 | BLOCKED | Disclosure, source, and constructibility analysis |
| `P2-009` | P1 | BLOCKED | Representation lifecycle analysis |
| `P2-010` | P1 | BLOCKED | Execution-case-aware placement and layout requirements |
| `P2-011` | P1 | BLOCKED | Relation-indexed coverage requirements |
| `P2-012` | P1 | BLOCKED | Compact-ASH and live-transfer analyzed pilots |
| `P2-013` | Gate | BLOCKED | Complete Phase-2 evidence and exit |

### P2-001 — Realization compiler-input boundary · `task:phase2:realization-boundary`

**Status:** DONE; the F4-001 and F4-002 compiler-trust prerequisites are now
closed

Implemented source records:

- external immutability of invariant-bearing realization fields;
- read-only accessors;
- canonical graph projections;
- declaration-order permutation checks;
- generic operation/relation/expression/proof/constructibility/disclosure
  ownership validation;
- no target types or generated publications as inputs.

F4-001 and F4-002 refined two remaining validation relationships and are now
closed: predicate bindings and lifecycle-edge shape are validated during
derivation, so the realization boundary is sufficient for compiler API freeze.

### P2-002 — Complete Petgraph dependency review · `task:phase2:dependency-review`

**Priority:** P2
**Status:** DONE
**Blocks:** P2-003

Reviewed declaration (before this task):

```text
petgraph = "=0.8.3"

selected features:
    serde-1
    rayon
    dot_parser
    unstable
    generate
```

Required review:

- exact crates.io release and upstream tag;
- license;
- Rust 1.88 compatibility;
- default and selected features;
- transitive dependency graph and duplicate versions;
- dependency-internal unsafe code;
- Rayon and deterministic-output implications;
- serialization non-authority boundary;
- unstable API boundary;
- advisory status;
- lockfile changes and provenance;
- replacement boundary.

Required commands include:

```sh
cargo tree --locked -p petgraph -e features
cargo tree --locked -i petgraph
cargo metadata --locked
cargo audit
```

If `cargo-audit` is unavailable, record the lane as skipped. The Phase-2 gate
must explicitly decide whether advisory tooling is mandatory in its final
environment.

No new numerical or solver dependency enters during this task.

#### Review result (2026-07-24)

**Selected version and source:** `petgraph = "=0.8.3"`, exact-pinned, from
`registry+https://github.com/rust-lang/crates.io-index`
(checksum `8701b58e…0b27455`). License `MIT OR Apache-2.0`; MSRV `1.64`,
compatible with the workspace MSRV `1.88`.

**First-party consumers:** `tripod-labels` and
`tripod-realization` only. Both use core petgraph exclusively —
`graph::DiGraph`, `graph::NodeIndex`, `Direction`, and `visit::EdgeRef`. No
first-party code uses any non-default petgraph feature.

**Feature decision:** four of the five enabled features had no first-party
consumer and are removed under the backlog dependency-entry rule and
(`[ADR016-rule:identity:admission]`) (a feature is not carried to advertise
intent):

```text
dot_parser  removed — no consumer; additionally pulled dot-parser and
                      dot-parser-macros at GPL-2.0-or-later into an
                      MIT/Apache workspace
rayon       removed — no consumer; added nondeterministic parallelism
                      the algorithm-laws rule keeps out of semantic ordering
unstable    removed — no consumer; unstable API surface
generate    removed — no consumer; random-graph generators
serde-1     retained — no product consumer yet, but raw label-graph
                      serialization for noncanonical diagnostics is
                      anticipated; kept deliberately with a guard test in
                      packages/labels and a manifest note
```

**Transitive graph after trim:** petgraph depends only on `fixedbitset`
(0.5.7), `hashbrown` (0.15.5), `indexmap` (2.14.0), `serde`, and
`serde_derive` — all `MIT OR Apache-2.0`, all MSRV ≤ 1.88. A pre-existing
duplicate `hashbrown` (0.15.5 and 0.17.1) is unchanged by this task. No new
crate version is added.

**Lockfile impact:** the trim removes 14 crates from `Cargo.lock` —
`dot-parser`, `dot-parser-macros`, `pest`, `pest_derive`, `pest_generator`,
`pest_meta`, `litrs`, `ucd-trie`, `rayon`, `rayon-core`, `crossbeam-deque`,
`crossbeam-epoch`, `crossbeam-utils`, and `either`. `serde_derive` was already
present through the workspace `serde` derive feature.

**Unsafe boundary:** petgraph contains internal `unsafe` (≈17 sites in
`graph_impl`, `stable_graph`, `matrix_graph`, and `unionfind`), confined to its
own data structures. First-party crates deny `unsafe` (ADR-011) and rely only
on petgraph's safe API; no petgraph unsafe invariant is exposed across a
first-party boundary.

**Determinism and parallelism:** with `rayon` removed, no parallel iterator
enters the graph tree. The algorithms used (topological order, SCC,
reachability) are deterministic, and canonical projections exclude `NodeIndex`
per (`[ADR016-rule:identity:immediate-edges]`).

**Advisory status:** `cargo audit` — advisory tooling is **not installed** in
the review environment; recorded as skipped, never passed (per
(`tbl:backlog:dependencies`) and the verification matrix). The Phase-2 gate
must decide whether advisory tooling is mandatory in its final environment.

**Serialization non-authority:** petgraph serde output is diagnostic only and
is never protocol, semantic, or release identity.

**Replacement boundary:** the graph substrate remains replaceable; only the
safe core API is used, so a future substitution would touch construction and
traversal call sites, not identity.

**Verification (SDK build, tree `6dac0b5` + this change):** `cargo fmt --check`
clean; `cargo clippy --workspace --all-targets --locked -D warnings` clean;
`cargo test --workspace --locked` and `--release --locked` all pass, 0 failed;
`meson test -C build` 10/10 OK (including census-audit, labels-check,
check-generated, plans-check). Clean tree after commit.

### P2-003 — Create the compiler crate · `task:phase2:create-compiler`

**Priority:** P1
**Status:** DONE
**Package contract:** [compiler.md](packages/compiler.md)

Create:

```text
packages/compiler
Cargo package: tripod-compiler
library: compiler
```

Initial first-party dependency:

```text
realization
```

A direct architecture dependency is added only if the compiler’s public types
directly name architecture-owned IDs and ownership would otherwise be obscured.

The crate must:

- inherit workspace metadata and lints;
- deny first-party unsafe code;
- join Cargo and Meson censuses;
- expose a public API integration test;
- consume typed values only;
- contain no generated-publication, filesystem, environment, model-source,
  target-opcode, stack-index, tapleaf, transaction-position, or target-bytecode
  input;
- mint no public compiler digest before a real consumer exists and I1-003
  permits it.

#### Resolution (2026-07-26)

`packages/compiler` exists as `tripod-compiler`, library `compiler`,
inheriting workspace metadata and lints, forbidding unsafe code, and joined to
both the Cargo workspace and the Meson census. The census weld refused the
first run until the new files were tracked, which is the ADR-014 behaviour
working rather than an obstacle.

The crate carries its boundary and its error root only. Input binding and every
analysis stage belong to P2-004 and later, and nothing partial is exposed in
the meantime, so no value this crate produces today can be mistaken for a
completed analysis. It mints no public compiler digest: under I1-003 an
analysis identity activates only on a real cross-process, cached, or published
consumer, and a field reserved for a future digest would itself be speculative.

`CompileError` is `non_exhaustive` and carries only the five input-boundary
failures, each exercised by the public-API integration test. Later stages
extend the vocabulary without a breaking change and without this crate guessing
their shapes now.

The package contract admits a direct `architecture` dependency only where the
compiler's public types name architecture-owned IDs and ownership would
otherwise be obscured. That condition holds: the error vocabulary names
operations, realization does not re-export `OperationId`, and a compiler-local
operation identifier would duplicate an architecture-owned ID rather than cite
it. The dependency is taken on that stated ground and recorded in the manifest.

Source: `packages/compiler`. Verified under both toolchains, plus
`meson test -C build` 10/10.

### P2-004 — Bind input and scope · `task:phase2:bind-input`

Define one analyzed input boundary containing:

- immutable architecture binding inherited from realization;
- immutable canonical realization projection;
- explicit operation scope;
- typed analysis policy;
- optional abstract target capabilities;
- no concrete target package or target bytes.

Reject:

- unsupported or incomplete scope;
- duplicate scope;
- architecture/realization mismatch;
- operation/relation ownership mismatch;
- generated-file input.

### P2-005 — Build the canonical relation DAG · `task:phase2:relation-dag`

Use a package-owned direct Petgraph graph with:

```text
typed stable keys
typed node and edge weights
stable-key → NodeIndex metadata
canonical insertion
canonical stable-key projection
```

Requirements:

- compiler relation census equals realization scope;
- every source relation retains provenance;
- duplicate IDs reject;
- unknown endpoints reject;
- unsupported cycles reject with canonical SCC diagnostics;
- Petgraph indices remain local;
- insertion permutations produce equal typed projections;
- standard graph algorithms use Petgraph.

### P2-006 — Implement checked constant folding · `task:phase2:constant-folding`

Initial legal folds:

- literals;
- boolean identities;
- exact count/amount operations;
- statically known activation;
- explicitly set-like canonical ordering;
- structural sharing retaining provenance.

Do not:

- reassociate checked arithmetic;
- move or combine floors;
- change overflow or underflow behavior;
- reorder named failure conditions;
- drop relation ownership;
- change disclosure or witness requirements.

Compare folded evaluation with a non-folded oracle.

### P2-007 — Implement exact proof planning · `task:phase2:proof-planning`

For each relation:

1. enumerate realization-approved alternatives;
2. reject unavailable target capabilities;
3. reject unauthenticated sources;
4. reject unavailable witnesses;
5. reject permissionless owner/operator secrets;
6. reject representation failures;
7. reject lifecycle failures;
8. reject disclosure failures;
9. retain the exact feasible set or Pareto frontier;
10. select canonically only under explicit policy.

Initial search uses deterministic exact enumeration or branch-and-bound.

Complexity exhaustion returns a typed error and never selects a partial or
hidden greedy result.

### P2-008 — Derive disclosure, sources, and constructibility · `task:phase2:constructibility`

Every relation operand records an authenticatable source class.

Permissionless cases require public or constructor-local sponsor facts only.

Disclosure reasons remain separate:

```text
semantic public state or event
permissionless constructibility
target safety
deployment policy
```

Sponsor-value opacity remains in force: individual sponsor amounts are not
protocol facts.

### P2-009 — Analyze representation lifecycle · `task:phase2:lifecycle`

For each supported representation, record paths to required exits.

At minimum:

```text
live receipt:
    transfer
    burn
    redeem

ASH:
    compact
    clear
```

A pilot may be valid within its present scope while lifecycle-incomplete for a
deployment. That distinction must remain typed and explicit.

### P2-010 — Derive placement and layout requirements · `task:phase2:placement`

Classify relations as:

- local;
- transaction-global;
- conditional;
- deliberately duplicated.

Model finite execution cases where applicable:

```text
sponsorless / sponsored
explicit / confidential
continuing / terminal
empty / nonempty
pre-maturity / conversion / post-maturity
```

Every active required case must have at least one possible semantic carrier.

Compiler core does not assign tapscript input indexes, stack positions,
tapleaves, or concrete transaction slots.

### P2-011 — Derive coverage requirements · `task:phase2:coverage`

Require exact equality among:

```text
realization relation scope
compiler relation scope
placement requirement scope
coverage requirement scope
```

Every relation receives:

- active accepting case;
- focused rejecting mutation;
- activation requirements;
- representation cases;
- carrier requirement;
- accepted semantic projection checks.

Conditional relations additionally receive inactive-valid, active-valid, and
active-invalid cases.

### P2-012 — Analyze both pilots · `task:phase2:pilots`

#### Compact ASH

Analyze:

- cardinality and recognition;
- permissionless authorization;
- ownerless `U` conservation;
- exact canonical delta;
- sponsor multiplicity and isolation;
- no roots;
- transition-certificate projection only;
- public constructibility;
- explicit/public representation;
- compact and clear lifecycle.

#### Live transfer

Analyze:

- input/output cardinality;
- live receipt recognition and closure;
- all-owner authorization;
- exact aggregate `U` conservation alternatives;
- explicit closed `U`;
- destination-family closure;
- sponsor multiplicity and isolation;
- no roots;
- transition-certificate projection only;
- explicit/private-committed alternatives;
- transfer, burn, and redemption lifecycle.

Repeated analysis from identical typed inputs must produce equal projections.

### P2-013 — Phase-2 evidence and exit · `gate:backlog:phase2`

Phase 2 exits only when:

- I1-001 through I1-003 are done;
- F4-001 and F4-002 are closed;
- F4-004 is closed, and F4-003 is dropped as reproduced-false, or both are
  formally shown not to block the gate;
- P2-002 through P2-012 are done;
- C1-004, C1-005, C1-008, C1-009, and C1-010 are done;
- active Phase-2 algorithm oracles are complete;
- all required repository gates pass in recorded environments;
- the source tree is clean.

Phase 2 may publish no persistent identity merely to demonstrate completion.
If a persistent compiler report or analyzed-plan identity is introduced, its
consumer and ADR-016 admission record must land in the same implementation
series.

---

## 7. Algorithm and dependency preparation · `sec:backlog:c1`

### 7.1 Current status · `tbl:backlog:c1`

| ID | Status | Deliverable |
|---|---|---|
| `C1-001` | DONE | Compiler/linker/mathematics/solver research notes |
| `C1-002` | DONE | Direct Petgraph decision |
| `C1-003` | DONE | Exact/certified mathematics decision |
| `C1-004` | DONE | Concrete Petgraph dependency and lockfile review (see P2-002) |
| `C1-005` | TODO | Canonical direct-Petgraph construction prototype |
| `C1-006` | PARKED | Exact keyed linear systems until a concrete consumer exists |
| `C1-007` | PARKED | Certified numerical analysis until a concrete consumer exists |
| `C1-008` | TODO | Exact proof-plan search |
| `C1-009` | TODO | Execution-case-aware placement |
| `C1-010` | TODO | Typed symbol resolution and SCC policy |
| `C1-011` | BLOCKED | Structured relocation; linker phase |
| `C1-012` | BLOCKED | Deterministic bounded-depth target tree; linker phase |
| `C1-013` | TODO | Independent small-instance oracles for active Phase-2 algorithms |
| `C1-014` | BLOCKED | Preparation review and Phase-2 handoff |

### C1-005 — Canonical direct-Petgraph construction

Required:

- typed nodes and edges;
- stable semantic keys;
- canonical node and edge insertion;
- stable-key/local-index metadata;
- Petgraph topology, SCC, and reachability;
- canonical stable-key projection;
- deterministic diagnostics.

Test permutations, duplicate keys and edges, missing endpoints, self-loops,
disconnected graphs, deep chains, SCCs, and attempted local-index publication.

### C1-008 — Exact proof-plan search

Implement an exact pilot planner and compare every generated small case with
exhaustive enumeration.

Required adversarial cases:

- cheapest local alternatives form an invalid global plan;
- sharing changes the optimum;
- lifecycle-safe plan differs from the cheapest immediate plan;
- equal-cost plans require stable-key tie-breaking;
- no feasible plan;
- complexity budget exhausted.

### C1-009 — Execution-case-aware placement

Enumerate required execution cases and eligible carriers.

A carrier present somewhere but absent from one active case does not satisfy the
relation.

Compare production search with exhaustive carrier-subset enumeration.

### C1-010 — Typed symbols and SCC policy

Use two-pass typed resolution:

1. complete definition census;
2. complete reference resolution.

Normalize SCC members by stable key. Every accepted cyclic dependency requires
an explicit semantic resolution strategy. SCC membership alone never
authorizes a cycle.

### C1-013 — Active algorithm oracles

Required Phase-2 oracles:

| Production analysis | Independent oracle |
|---|---|
| canonical topology | valid-order enumeration plus least-key rule |
| SCC | mutual-reachability equivalence |
| expression interning | non-interned evaluator |
| dependency closure | repeated complete scan |
| proof selection | exhaustive candidate enumeration |
| placement | exhaustive carrier subsets |
| relation census | direct set equality |
| constant folding | non-folded evaluator |
| canonical realization/compiler projection | declaration-order permutation |

---

## 8. Dependency policy · `sec:backlog:dependencies`

### 8.1 Current and deferred dependencies · `tbl:backlog:dependencies`

| Dependency | Status | Role |
|---|---|---|
| `petgraph = 0.8.3` | Adopted; reviewed (P2-002); features trimmed to `serde-1` | Graph storage and standard algorithms |
| `num-bigint` | Existing | Exact arbitrary-size integers |
| `num-integer` | Existing | Exact integer helpers |
| `num-traits` | Existing | Numeric traits |
| `num-rational` | Not adopted | Future exact-rational consumer only |
| `faer` | Not adopted | Future certified numerical diagnostics only |
| `fixedbitset` | Deferred | Dense local coverage sets if measured |
| Elements libraries | Phase-3 review | Target transaction and consensus types |
| SAT/LP/MILP solver | Deferred | Large exact planning problems only after measured need |
| `salsa` | Deferred | Incremental compiler queries |
| `egg` | Deferred | Equality saturation |
| `rayon` | Selected through Petgraph | Independent work only; never semantic ordering |

### 8.2 Dependency-entry rule · `rule:backlog:dependency-entry`

A deferred dependency enters only when:

1. a concrete consumer exists;
2. current implementation demonstrates the missing functionality;
3. simpler exact first-party code is insufficient;
4. purpose, maintenance, source, version, license, MSRV, unsafe boundary,
   transitive graph, determinism, and advisories are reviewed;
5. public API leakage is considered;
6. focused tests and an independent oracle exist;
7. lockfile changes are reviewed;
8. all gates remain green and clean.

Unused dependencies are not added to advertise intent.

---

## 9. Path scope and host filesystem trust · `sec:backlog:path-scope`

The immediate policy owner is
[ADR-017](../adr/017-path-scope-and-host-filesystem-trust.md), which is
Decided with implementation required in the same series. ADR-014 and ADR-015
were edited directly so the three records agree; ADR-017 supersedes neither.

The governing boundary is:

> The repository validates repository shape, semantic path derivation, lexical
> role separation, and publication bytes. The host owns what paths resolve to
> and whether they are swapped.

This series simplifies rather than extends. Each task removes a check that
claimed a boundary the repository cannot hold, or moves that check to its
single central owner. Every task is a net reduction in first-party filesystem
logic; none weakens a constraint on source-derived paths.

### 9.1 Summary · `tbl:backlog:path-scope`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `A17-001` | P1 | DONE | Census audit consumes Git modes and rejects symlinks and gitlinks |
| `A17-002` | P1 | DONE | Shared destination identity becomes lexical |
| `A17-003` | P1 | DONE | Flattener drops ancestor alias analysis, keeps source-derived confinement |
| `A17-004` | P2 | DONE | `execwrap` role uniqueness is simplified or documented as the sole exception |
| `A17-005` | P2 | DONE | Removal of preflight race claims from prose and diagnostics |

### A17-001 — Central tracked-entry mode audit · `task:path:census-modes`

**Lane:** ADR-017 implementation
**Priority:** P1
**Status:** TODO
**Depends on:** nothing
**Owner:** `labels` (`census-audit`), Meson
**Policy:** (`[ADR014-rule:build:tracked-entry-modes]`)
**Concrete output:** the audit reads a mode-bearing tracked-file listing and
fails on any entry whose mode is not `100644` or `100755`, including paths
excluded from lint subjects
**Assurance:** focused unit tests plus the census-audit lane on the real tree
**Affected files:** `packages/labels/src/census.rs`,
`packages/labels/src/bin/census-audit.rs`, `meson.build`
**Focused exit test:** a fixture repository carrying a tracked symlink fails
the audit naming the path and its mode
**Verification:** `cargo test -p tripod-labels` and
`meson test -C build`
**Identity and schema impact:** the audit report gains a mode-defect class; no
digest changes
**Dependency impact:** none

The mode listing is the single owner of repository shape. It replaces, and is
not added alongside, per-tool alias analysis.

#### Resolution (2026-07-26)

The audit now invokes `git ls-files --stage -z` and parses each record into a
mode and a path. `CensusAuditReport` gains a `disallowed_modes` defect list and
its schema moves to 2; the audit is invalid when that list is nonempty, and the
binary reports each defect with both the path and the rejected mode.

The mode check runs over the complete tracked set before exclusions are
applied, so a lint-excluded path is still bound by the repository-shape rule.
A malformed listing record is an error rather than a skipped entry: silently
dropping one would discard exactly the tracked symlink the audit exists to
catch.

Evidence on the real tree: 342 tracked entries, every mode `100644` or
`100755`, no defects — the invariant the record states already holds, and is
now enforced rather than assumed.

Source: `packages/labels/src/census.rs`,
`packages/labels/src/bin/census-audit.rs`. Tests: listing-parse and
mode-rejection unit tests in `packages/labels/src/tests.rs`, plus an
end-to-end wiring test in `packages/labels/tests/subprocess_contract.rs` that
drives the binary with a symlink-bearing listing and asserts the failure names
the path and mode, with no success stamp published.

### A17-002 — Lexical destination identity · `task:path:lexical-destinations`

**Lane:** ADR-017 implementation
**Priority:** P1
**Status:** TODO
**Depends on:** A17-001
**Owner:** `cli-common`
**Policy:** (`[ADR017-rule:path:output-roles]`)
**Concrete output:** shared output-role validation compares lexically
normalized absolute paths only — no canonicalization, symlink resolution,
device/inode comparison, hard-link detection, or mount identity
**Assurance:** focused unit tests over the normalization rules
**Affected files:** `packages/cli-common/src/lib.rs` and its tests
**Focused exit test:** two roles naming one path through `.` and `..`
components collide; two roles aliased only by a hard link do not
**Verification:** `cargo test -p cli-common` and `meson test -C build`
**Identity and schema impact:** none
**Dependency impact:** none

Normalization resolves a relative path against its documented base, makes it
absolute, and removes `.` and `..` components without touching the filesystem.
Generic hard-link and symlink-parent tests are removed rather than relaxed: a
hard link is not semantic identity, and first-party outputs are identified by
role, path, schema, and bytes.

#### Resolution (2026-07-26)

`DestinationIdentity` is now one lexically normalized absolute path.
`destination_identity` resolves a relative path against the process working
directory, then removes `.` and resolves `..` without consulting the
filesystem; `..` cannot escape a root because popping a root leaves it in
place. `ensure_distinct_outputs` compares those paths for equality.

Removed: `entry_identity`'s deepest-existing-ancestor canonicalization and
`existing_file_identity`'s device/inode probe, along with the `aliases` helper
that combined them.

The two alias tests were not deleted. They now assert the opposite outcome and
say why: hard-linked destinations and symlinked parent directories are
lexically distinct and pass, because detecting some aliases while the host may
replace or remount a path at any moment establishes no boundary. Recording the
decision as a passing test keeps a future reader from restoring the check as a
supposed fix. A third test pins the case that still matters — one destination
named both relatively and absolutely.

Note on a check that stays: `document-stamps` continues to read Git modes for
its declared publication-input set. That is not filesystem alias analysis; the
mode is an input to its own digest recipe.

Source: `packages/cli-common/src/lib.rs`,
`packages/cli-common/src/tests/mod.rs`.

### A17-003 — Flattener source-derived confinement only · `task:path:flattener`

**Lane:** ADR-017 implementation
**Priority:** P1
**Status:** TODO
**Depends on:** A17-002
**Owner:** `flatten-latex-main`
**Policy:** (`[ADR017-rule:path:derived-references]`)
**Concrete output:** the ancestor-by-ancestor symlink walk is removed; allowlist
resolution, absolute-path rejection, parent-traversal rejection, ambiguity
rejection, cycle detection, and atomic output all remain
**Assurance:** the existing focused suite, less the removed alias cases
**Affected files:** `packages/flatten-latex-main/src/lib.rs` and its tests
**Focused exit test:** an include naming an absolute path, a `..` segment, an
undeclared member, or a cycle still fails closed; a supplied allowlist entry
whose ancestor is a symlink no longer fails
**Verification:** `cargo test -p flatten-latex-main` and `meson test -C build`
**Identity and schema impact:** none
**Dependency impact:** none

An allowlist constrains what source text may select. It does not authenticate
the host filesystem beneath a supplied entry, and the code must no longer imply
that it does.

#### Resolution (2026-07-26)

Removed: the ancestor-by-ancestor `symlink_metadata` walk in
`ensure_regular_file`, and the device/inode `IncludeFrame` identity used for
cycle detection.

Retained unchanged: allowlist resolution by trailing-component match with no
filesystem lookup, absolute-reference rejection, `..` rejection, ambiguity
rejection, cycle detection, atomic staged output, and the regular-file check on
the final component — which stays as a file-type role check, since inlining a
directory or device is a caller error worth naming.

Cycle detection is now by allowlist path, and that is not a weakening. Every
frame below the entry point is an entry of a finite allowlist, so an unbounded
chain must repeat a path; the hard-link alias case closes one frame later than
an inode comparison did, and its test still passes unchanged.

The three ancestor-rejection tests were replaced rather than deleted. Two now
assert acceptance and say why — a supplied path's ancestors are the host's
business — and a third pins what still fails closed: a reference naming no
allowlist entry stays unreachable even when a symlinked directory sits inside
the paper tree, because resolution never touches the filesystem.

Source: `packages/flatten-latex-main/src/lib.rs` and its tests.

### A17-004 — `execwrap` role uniqueness · `task:path:execwrap-roles`

**Lane:** ADR-017 implementation
**Priority:** P2
**Status:** TODO
**Depends on:** A17-002
**Owner:** `execwrap`
**Policy:** (`[ADR017-rule:path:local-checks]`)
**Concrete output:** either the wrapper reduces to lexical role uniqueness, or
its concurrent-writer alias check is retained as the sole documented local
exception
**Assurance:** the existing subprocess-contract suite
**Affected files:** `packages/execwrap/src/lib.rs`,
`packages/execwrap/src/writer.rs`, and their tests
**Focused exit test:** the chosen behaviour is pinned by a test that names the
concurrent-writer hazard rather than filesystem aliasing in general
**Verification:** `cargo test -p execwrap` and `meson test -C build`
**Identity and schema impact:** none
**Dependency impact:** none

Retention requires documenting the exact hazard, the additional check, the
remaining host race, and why lexical role uniqueness is insufficient. A
retained exception is a package-local correctness measure and never a
repository-wide filesystem security claim.

#### Resolution (2026-07-26) — retained as the documented exception

Retained, not simplified. ADR-017 names this exact hazard when it admits a
package-local check, and the hazard is real: two redirection routes resolving
to one file are opened independently, each truncating, then written
concurrently as the child produces output, so the log ends up interleaved and
partially overwritten while every I/O call succeeds. The damage is to the
operation's own output, not to repository state.

Lexical comparison is insufficient here in a way it is not for ordinary output
roles: it catches one name spelled two ways, but not two names hard-linked to
one file, two paths beneath a symlinked directory, or a dangling link and the
name it points at. Redirection paths are routinely assembled by build glue,
where those aliases arise by accident rather than by a caller's choice.

The four required disclosures — exact hazard, additional check, why lexical is
insufficient, and the remaining host race — are documented on `FileIdentity`
itself, where a reader meets the code. No behaviour changed; the exception is
now stated instead of implied.

Source: `packages/execwrap/src/lib.rs`. Test:
`two_routes_to_one_log_are_refused_because_they_would_interleave` in
`packages/execwrap/src/tests/mod.rs` names the concurrent-writer hazard rather
than filesystem aliasing in general, and asserts that the refusal precedes
truncation while claiming nothing about the host race.

### A17-005 — Remove preflight race claims · `task:path:race-claims`

**Lane:** ADR-017 implementation
**Priority:** P2
**Status:** TODO
**Depends on:** A17-001 through A17-004
**Owner:** every first-party package, plans, and ADR prose
**Policy:** (`[ADR017-rule:path:toctou]`)
**Concrete output:** no comment, diagnostic, README, or plan describes a
preflight path check as closing a host-level race
**Assurance:** review sweep plus the forbidden-text lane where a phrase is
mechanically detectable
**Affected files:** wherever the sweep finds a claim
**Focused exit test:** the sweep is recorded with the phrases it changed
**Verification:** `meson test -C build` and the complete gate
**Identity and schema impact:** none
**Dependency impact:** none

Atomic staging and compare-if-changed remain required and remain honestly
described: they are correctness properties of an honest writer, not protection
against a host replacing a path before, during, or after publication.

#### Resolution (2026-07-26)

The sweep found the claims concentrated in the two loci this series already
touched, and both were rewritten as part of their own tasks rather than left
for a separate pass. The flattener's module doc no longer claims symlink-free
path confinement and now states the boundary and the race explicitly;
`cli-common` no longer describes canonicalized-entry identity.

Remaining occurrences of the vocabulary were checked and are correct as they
stand: ADR-015 and ADR-017 state the non-claims deliberately, `execwrap`
documents its retained exception including the residual window, and the
flattener's symlink diagnostic still describes behaviour it really has —
a symlinked allowlist entry is refused as a file-type role check.

No first-party comment, diagnostic, README, or plan now describes a preflight
path check as closing a host-level race.

---

## 10. Mathematical and algorithmic laws · `sec:backlog:algorithm-laws`

### 10.1 Exactness · `rule:backlog:exactness`

Semantic, conservation, authorization, identity, calibration, and release
claims use:

- checked bounded integers;
- arbitrary-precision integers;
- reduced exact rationals;
- exact finite search;
- independently checked certificates;
- target-native execution where the claim concerns the target.

A floating residual is diagnostic evidence, not exact equality.

### 10.2 Identity · `rule:backlog:identity`

Always distinguish:

```text
local handle:
    one process-local graph, arena, matrix, or solver position

stable key:
    complete typed semantic identity

digest:
    optional domain-separated commitment admitted under ADR-016
```

Never use as semantic identity:

- Petgraph indices;
- matrix positions;
- solver variable numbers;
- insertion or traversal order;
- source path or line;
- pivot order;
- floating-point bits;
- thread schedule;
- temporary path.

A canonical projection excludes incidental declaration order where order is not
semantic.

### 10.3 Complexity failure · `rule:backlog:complexity`

An analysis exceeding its explicit budget returns a typed complexity error.

It must not:

- drop a relation;
- weaken authorization;
- expose more information silently;
- remove a lifecycle exit;
- switch to a hidden greedy fallback;
- claim optimality from incomplete search;
- accept the best partial result seen before exhaustion.

---

## 11. Verification matrix · `sec:backlog:verification`

### 11.1 Focused commands · `tbl:backlog:focused-tests`

| Area | Command |
|---|---|
| architecture and profile validation | `cargo test --locked -p tripod-architecture` |
| realization validation and projection | `cargo test --locked -p tripod-realization` |
| model/realization conformance | `cargo test --locked -p tripod-model realization_conformance` |
| labels and plan census | `cargo test --locked -p tripod-labels` |
| checker report/stamp behavior | `cargo test --locked -p cli-common` |
| document provenance outputs | `cargo test --locked -p tripod-document-stamps` |
| Meson stamp/publication behavior | `scripts/test-meson-mock.sh .` |
| complete workspace | `cargo test --workspace --locked` |

Focused filters supplement but never replace full package and workspace runs.

### 11.2 Complete Rust gate

Run under the declared MSRV and current stable:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release --locked
scripts/ci.sh
```

### 11.3 Meson and document gate

Use the canonical build directory:

```sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run `meson setup build` only when `build/` does not exist.

Byte reproducibility remains a separate release/manual check:

```sh
scripts/check-document-reproducibility.sh
```

### 11.4 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every newly tracked subject joins its nearest `meson.build` census.

### 11.5 Dependency and advisory evidence

```sh
cargo tree -e features
cargo metadata
cargo audit
```

A missing advisory tool is recorded as skipped, never passed.

### 11.6 Clean repository

The final check is:

```sh
git status --porcelain=v1 --untracked-files=all
```

It must be empty.

### 11.7 Execution trust

Repository source, tests, Meson definitions, scripts, TeX, and `.latexmkrc` are
executable.

Untrusted contributions run only in an externally established, credential-free
isolated environment. A clean-tree result is a correctness check, not a
malicious-code containment boundary.

---

## 12. Phase-2 gate · `gate:backlog:current`

The current gate is **not passed**.

Current blockers are:

```text
Identity architecture:
    I1-001 through I1-003 closed; inventory and future DAG recorded
    I1-004 through I1-006 parked or blocked on their own consumers

Realization validation:
    F4-001 and F4-002 closed; predicate and lifecycle-edge shape validated

Build/documentation correctness:
    F4-003 dropped (reproduced false); F4-004 and F4-005 closed
    every current static-review finding is resolved

Dependency review:
    P2-002 / C1-004 complete (Petgraph reviewed; features trimmed)

Compiler:
    package created (P2-003); boundary and error root only
    relation/proof/disclosure/lifecycle/placement/coverage analysis absent
```

Every preparatory identity, finding, and dependency task is closed, and the
complete gate has been run and recorded below. The sole remaining blocker is
the compiler itself.

### 12.1 Recorded complete gate run · `rem:backlog:gate-run`

Run on 2026-07-26 against `db03dc2`, on a tree reporting no staged, unstaged,
or untracked nonignored paths. This records one execution; it does not make any
later tree green.

| Lane | Result |
|---|---|
| `cargo fmt --all --check`, MSRV 1.88.0 and stable 1.97.1 | clean |
| `cargo clippy --workspace --all-targets --locked -D warnings`, both toolchains | clean |
| `cargo test --workspace --locked`, both toolchains | 37 suites, 865 tests, 0 failed |
| `cargo test --workspace --release --locked`, both toolchains | 37 suites, 865 tests, 0 failed |
| `scripts/ci.sh` | 11 of 11 lanes pass |
| `meson compile -C build` and `meson test -C build --print-errorlogs` | 10 of 10 pass |
| `scripts/check-plans.sh` and `meson compile -C build lint` | pass |
| `git diff --check`, staged and unstaged | clean |
| `cargo tree --locked -e features`, `cargo metadata --locked` | resolve against the committed lockfile; 148 locked packages |
| `cargo audit` | SKIPPED — not installed in this environment |
| `git status --porcelain=v1 --untracked-files=all` | empty |

The advisory lane is recorded as skipped, never as passed. No advisory
statement may be made from this run.

Two defects were found by running the gate rather than by review, and both were
fixed before the recorded run. The Meson lane exposed nothing new; the Rust
lanes did. Current-stable clippy rejected a nested conditional in the labels
crate that both nightly and the declared MSRV accept, which means the stable
lane had not been exercised recently and the repository's clippy cleanliness
was toolchain-dependent. Separately, the clean-tree lane correctly refused the
run while that fix was still uncommitted.

Byte reproducibility remains a separate release check rather than a gate lane,
but it was run alongside this one: two independent clean build directories
rendered a byte-identical PDF. Its reused-build epoch probe skipped itself
because the worktree carried this uncommitted record, so that sub-check is
recorded as skipped, not passed.

Until (`gate:backlog:phase2`) passes:

- Phase 1 remains historical tagged evidence;
- no compiler public API is frozen;
- no public realization/compiler digest is minted without a real consumer;
- target-specific fields remain forbidden in realization and compiler core;
- no stable linker or transaction ABI exists;
- no floating value or local graph handle enters semantic identity;
- no draft bound becomes deployment calibration;
- no raw report digest is treated as evidence identity without typed role and
  subject binding;
- no self-consistent cache is described as independent target-chain evidence;
- no architecture, model, realization, hash, build, or test success is
  described as deployment readiness;
- no current tree is described as green without a fresh complete execution
  record.

---

## 13. Backlog hygiene · `sec:backlog:hygiene`

### 13.1 Adding work · `rule:backlog:add`

A new task states:

- phase or lane;
- priority and status;
- dependencies;
- concrete output;
- assurance class;
- affected files and packages;
- focused exit test;
- verification command;
- identity and schema impact;
- dependency impact.

A new digest additionally satisfies
(`[ADR016-rule:identity:admission]`).

### 13.2 Splitting work · `rule:backlog:split`

Split a task when it:

- crosses semantic, compiler, linker, target, transaction, evidence, or release
  ownership;
- mixes exact correctness with numerical diagnostics;
- mixes dependency adoption with algorithm acceptance;
- combines independent security consequences;
- contains one part that can complete while another remains research-blocked.

### 13.3 Dropping or parking work · `rule:backlog:drop`

A dropped or parked task records:

- why it is unnecessary or premature;
- supporting evidence;
- activation condition;
- replacement, if any;
- identity and release consequences.

“No present consumer” is sufficient reason to park an identity, report digest,
dependency, or publication.

### 13.4 Retention · `rule:backlog:retention`

After a phase baseline:

- move durable evidence into the phase card or release record;
- retain immutable evidence in Git history or an annotated tag;
- keep only current and immediately preparatory work here;
- preserve permanent task IDs in compact tables;
- do not create another historical archive under `plans/`.

---

## 14. Execution order · `sec:backlog:order`

Execute in this order unless new evidence changes dependencies:

```text
1. Land ADR-016 and this backlog rewrite together.
2. Complete the current identity inventory.
3. Define future immediate identity edges and activation points.
4. Reproduce F4-001 through F4-004.
5. Close F4-001 and F4-002 before freezing compiler input APIs.
6. Close F4-004 and F4-005 (F4-003 dropped as reproduced-false).
7. Complete the Petgraph dependency and lockfile review.
8. Run and record the complete current repository gate. DONE; see the recorded
   gate run in (`gate:backlog:current`).
9. Create tripod-compiler.
10. Implement relation DAG and checked folding.
11. Implement exact proof, disclosure, source, constructibility, and lifecycle analysis.
12. Implement execution-case placement, layout, and coverage requirements.
13. Analyze compact ASH and live transfer end to end.
14. Run and record the complete Phase-2 gate.
15. Begin Phase-3 target work only after Phase-2 exit.
```

No new hash, target prototype, report field, or publication may defer a current
typed-boundary or correctness repair.

---

## 15. One-line backlog · `rem:backlog:one-line`

> Adopt one consumer-driven identity architecture; close the remaining realization, lifecycle, stamp, and label-boundary findings; finish the Petgraph review; then build the Phase-2 compiler as an exact, deterministic, target-independent analysis without speculative hashes, target details, weakened relations, or ambiguous evidence.
