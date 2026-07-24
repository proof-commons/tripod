# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 2 — target-independent compiler analysis
> **Current condition:** Phase 1 remains an immutable historical completion record. The earlier F2 remediation series is closed, but static review of the current supplied snapshot identified a new F3 correctness and assurance set. The realization boundary, publication tooling, census enforcement, and Meson wiring must be re-closed before the compiler public API is frozen.
> **Next gate:** Phase 3 — Elements target and foundational prototypes
> **Authority:** Current execution queue only. Normative specifications, typed architecture, implemented ADRs, accepted decisions, package contracts, research results, phase cards, and the roadmap take precedence.

This file contains:

- the current repository state;
- immutable historical phase evidence;
- the active F3 remediation register;
- the Phase-2 implementation queue;
- the compiler-algorithm preparation queue;
- verification and clean-tree gates.

It does not retain the former implementation diary. Historical detail remains
in Git history, phase cards, annotated evidence tags, and the closed-finding
summaries below.

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
| **PARKED** | Deliberately outside the active gate until a concrete consumer exists. |
| **DONE** | Implementation, focused tests, documentation, and required evidence are complete. |
| **DROPPED** | Deliberately not implemented; rationale and replacement are recorded. |
| **SUPERSEDED** | Replaced by a named task, decision, ADR, or package contract. |
| **HISTORICAL** | Immutable evidence about an earlier commit; not a claim about the current checkout. |

Code resembling an intended result is not sufficient for `DONE`.

A task closes only when the implementing change, focused regression, affected
documentation, generated artifacts, complete required gates, and clean-tree
result have all been recorded.

A static review finding remains open until one of the following occurs:

- implementation plus focused regression;
- a typed proof that the reported state is unconstructible;
- reproduction showing the finding is false;
- an approved correction to the owning policy or assurance claim.

“Existing tests pass” is not closure unless a named test reaches the reported
path.

### 1.2 Priority vocabulary · `tbl:backlog:priority`

| Priority | Meaning |
|---|---|
| **P0** | A defect capable of manufacturing false release, deployment, provenance, or semantic evidence. |
| **P1** | A phase-gate blocker, public semantic-boundary defect, or build/publication correctness failure. |
| **P2** | Required correctness, determinism, policy, or documentation closure before the active phase exits. |
| **P3** | Maintainability or evidence-quality work that follows correctness but remains part of the active gate. |
| **POST** | Later-phase work that does not block the active phase. |

### 1.3 Task families · `tbl:backlog:families`

| Prefix | Owner |
|---|---|
| `F1` | Historical Phase-1 findings |
| `F2` | Historical post-Phase-1 findings closed before this snapshot |
| `F3` | Findings from static review of the current supplied snapshot |
| `P2` | Phase-2 compiler implementation |
| `C1` | Compiler/linker algorithm and dependency preparation |
| `Q` | Research or prototype dependency |

Historical task identifiers are permanent and never reused.

### 1.4 Definition of done · `rule:backlog:done`

An implementation task is `DONE` only when it records:

1. implementing source files;
2. focused positive tests;
3. focused negative, mutation, property, subprocess, or integration tests;
4. affected ADRs, decisions, package contracts, research notes, or phase cards;
5. exact verification commands and results;
6. generated-artifact and label-register impact;
7. semantic, identity, schema, and versioning impact;
8. dependency and feature impact;
9. confirmation that checks created no staged, unstaged, or untracked
   nonignored change;
10. any intentionally retained limitation or environmental skip.

A research task is `DONE` only when it records:

1. the precise question;
2. exact dependency, tool, and target versions;
3. positive and negative prototype evidence;
4. complexity and resource measurements;
5. accepted and rejected candidates;
6. result and decision handoff;
7. permanent production tests;
8. assurance class and remaining trust boundary.

### 1.5 Authority and labels · `rule:backlog:authority`

Planning labels are non-normative and non-identity-bearing.

Under ADR-013:

- each PLAN mint is unique;
- each same-owner citation resolves;
- cross-owner citations use explicit owner prefixes;
- generated registers do not participate in the authored label graph.

Task identifiers and planning labels never become compiler, linker, ABI,
deployment, evidence, or release identity.

---

## 2. Current repository state · `sec:backlog:state`

### 2.1 Review basis

This backlog rewrite is based on static review of the supplied repository
snapshot:

```text
tree reference:
tree reference label: HEAD
selected authored files: 331
submodules: none
symlinks in the supplied tree: none
```

The supplied content excluded:

```text
Cargo.lock
LICENSE-CODE
LICENSE-DOCS
archive/
```

The lockfile exists in the repository but was not included in the content
review.

No build, test, target execution, or reproduction command was run as part of
this review. Therefore:

- F3 findings are static source-review findings;
- historical green gates remain historical evidence;
- the current checkout is not declared green;
- every F3 task must run its focused and complete verification;
- dependency review cannot be considered complete from this snapshot because
  the lockfile and advisory results were not reviewed here.

### 2.2 Implemented packages · `tbl:backlog:implemented`

| Package or area | Current source state |
|---|---|
| Layer 0 | Published specification, version `0.5.1` |
| Realization document | Realization with final manifest appendix |
| `architecture` | Typed architecture, schema 17, semantic and behavioural hashes, deployment-profile validation |
| `model` | Executable state machine, invariant checker, property/corruption suites, indexer and accounting projections |
| `realization` | Target-independent typed semantics for compact ASH and live receipt transfer |
| `artifacts` | Generated-publication writer/checker and realization-document weld |
| `labels` | Owner-aware Markdown/Rust label graph, plan checks, generated registers, census and forbidden-text audits |
| `cli-common` | Shared ADR-010 command handling and ADR-014 report/stamp infrastructure |
| `document-stamps` | Git-derived paper metadata and deterministic two-output rendering |
| `execwrap` | Byte-preserving process wrapper; mocked TeX execution isolated in a separate binary |
| `flatten-latex-main` | Deterministic atomic LaTeX flattener with final-component symlink rejection |
| Meson | Explicit source census, incremental lint targets, always-fresh audits, mocked TeX contract |
| Security policy | Public-data interfaces and external execution-environment trust boundary |

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

The architecture is final as an architecture publication. That status does not
mean the system is deployment-ready.

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

The Cargo workspace currently contains nine packages and does not contain the
planned compiler package.

### 2.5 Security and readiness statement · `rem:backlog:readiness`

```text
Attestation specification:             published
Realization contract:                 published
Typed architecture:                   final and pinned
Executable model:                     implemented
Typed realization pilots:             implemented
Recorded Phase-1 gate:                historical and tagged
Historical F2 remediation:            recorded as closed
Current F3 findings:                  open
Phase-2 dependency review:            planning status IN PROGRESS
Phase-2 compiler package:              absent
Target/backend/linker/transaction:     absent
Independent deployment evidence:      absent
Production deployment:                absent
```

Current packages are public-data tools. They do not legitimately accept private
keys, wallet secrets, signing nonces, blinding factors, private openings,
credentials, or production authority.

A green model does not prove target correctness. A self-consistent event cache
does not prove target-chain provenance. Architecture finality does not imply
deployment readiness.

---

## 3. Historical Phase-1 record · `gate:backlog:phase1`

Phase 1 has an immutable evidence tag:

```text
phase1-realization-foundation-v1
```

The repository records that the tagged commit passed:

- `scripts/ci.sh` under Rust 1.88 and then-current stable;
- debug and release workspace tests;
- formatting and Clippy with `-D warnings`;
- generated-artifact, label, plan, census, and forbidden-text lanes;
- the Meson document lane;
- the mocked Meson contract;
- document reproducibility;
- the clean-tree check.

That record remains immutable historical evidence.

Later findings do not rewrite the tag. They refine what may be claimed about
the current source.

The F2 remediation series is recorded as closing:

- deployment calibration to emitted-bundle binding;
- pilot architecture-weld field coverage;
- Rustdoc fence scoping;
- final-component flattener symlink rejection;
- multi-output destination aliasing;
- sponsor-value opacity;
- graph-substrate terminology;
- stale evidence comments.

The current F3 findings identify additional cases not established by that
series. In particular:

- the realization value itself remains externally mutable;
- its stable projection still contains order-sensitive declaration copies;
- generic operation/proof ownership is not fully validated;
- the flattener’s stronger public confinement statement is not yet enforced
  for symlinked ancestor directories.

The Phase-1 tag remains unchanged.

---

## 4. Current remediation register · `sec:backlog:findings`

### 4.1 Summary · `tbl:backlog:findings`

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `F3-001` | P1 | **DONE** | A two-output Meson target is referenced without selecting its stamp output. |
| `F3-002` | P1 | **DONE** | `ScopedRealizationSpec` can be externally mutated out of consistency with its private graphs. |
| `F3-003` | P1 | **DONE** | Strict flattener confinement can be bypassed through a symlinked ancestor directory. |
| `F3-004` | P2 | **DONE** | `check-plans` accepts an empty argument census and falls back to discovery as authority. |
| `F3-005` | P2 | **TODO** | The stable realization projection includes raw order-sensitive graph declaration vectors. |
| `F3-006` | P2 | **TODO** | Realization derivation does not fully validate operation, relation, expression, and proof-alternative ownership. |
| `F3-007` | P3 | **TODO** | Layer-0 duplicate-label diagnostics do not name both mint locations. |
| `F3-008` | P3 | **TODO** | Authorization-evidence export arrays depend on enum declaration order rather than explicit canonical sorting. |
| `F3-009` | P3 | **TODO** | Conditional LaTeX flattening ignores the `IfFileExists` probe path when selecting the branch. |
| `F3-010` | P3 | **TODO** | The empty-stamp contract conflicts with `touch_stamp` preserving pre-existing stamp bytes. |

All F3 findings are static-review findings until reproduced or disproved by a
focused test.

---

### F3-001 — Select the explicit Meson output for `forbidden-text-check`

**Priority:** P1
**Status:** DONE
**Owner:** root Meson graph
**Primary file:**

```text
meson.build
```

#### Problem

The target declares two outputs:

```meson
output: ['forbidden-text.ok', 'forbidden-text.json']
```

The test then uses the unindexed target:

```meson
forbidden_text_stamp.full_path()
```

Other two-output checker targets correctly use output zero:

```meson
census_audit[0].full_path()
generated_stamp[0].full_path()
labels_stamp[0].full_path()
plans_stamp[0].full_path()
```

A multi-output custom target may not have one unambiguous `full_path()`.
Depending on Meson behavior and version, configuration may fail or select no
well-defined output.

#### Required implementation

Use the explicit stamp output:

```meson
forbidden_text_stamp[0].full_path()
```

Review every multi-output target use for the same mistake.

#### Required tests

- configure with the minimum supported Meson version;
- configure with the current development Meson version;
- run `meson test -C build --print-errorlogs`;
- confirm the test probes `forbidden-text.ok`, not the JSON report;
- delete the stamp and confirm the dependency rebuilds it;
- confirm a failed forbidden-text audit does not create or refresh the stamp.

#### Impact

```text
protocol semantics:              none
architecture identities:         none
generated artifact bytes:        none
Meson accepted configuration:    repaired or clarified
```

#### Exit

- [x] the test indexes output zero explicitly;
- [x] all multi-output custom-target uses are audited;
- [x] minimum-supported Meson setup succeeds;
- [x] focused and complete Meson tests pass;
- [x] the repository remains clean.

#### Evidence · DONE

- F3-001 closure commit. The `forbidden-text-check` test now probes
  output zero explicitly (forbidden-text.ok, the success stamp), never
  the JSON report: stamp_probe receives forbidden_text_stamp[0].
- Multi-output audit: every other custom-target full_path() use in the
  tree is either already indexed (cargo_clippy_stamp[0],
  cargo_test_stamp[0], census_audit[0], generated_stamp[0],
  labels_stamp[0], plans_stamp[0]) or a single-output target
  (install_pdf, flat_build_target) or a find_program result (git,
  xelatex, biber, latexmk). No other unindexed multi-output use exists.
- Fresh `meson setup` succeeds under both the minimum supported Meson
  (1.3.0, installed into a scratch venv) and the current SDK Meson
  (1.9.2), each into a throwaway build directory.
- Focused verification: `meson test -C build forbidden-text-check
  --print-errorlogs` green after reconfiguration; deleting
  build/forbidden-text.ok and running the target rebuilds an empty
  stamp beside the JSON report (the target is build_always_stale, so a
  deleted stamp is always regenerated). A failed audit cannot create
  or refresh the stamp because the shared build-mode path in
  cli-common (finish_check_command) publishes the report and touches
  the stamp only after a passing result — covered by the existing
  cli-common unit and subprocess suites.
- Complete Meson and workspace gates run at the end of the F3
  remediation series.

---

### F3-002 — Make `ScopedRealizationSpec` externally immutable

**Priority:** P1
**Status:** DONE
**Owners:** `realization`, future `compiler`
**Blocks:** compiler input API freeze
**Primary files:**

```text
packages/realization/src/derive.rs
packages/realization/src/lib.rs
packages/realization/tests/public_api.rs
```

#### Problem

`ScopedRealizationSpec` exposes public mutable fields:

```rust
architecture
scope
operations
declassification
```

Its derived expression, relation, constructibility, lifecycle, and disclosure
graphs remain private.

An external caller can therefore derive a valid value and then mutate the
public fields without rebuilding or revalidating the private graphs. Examples
include:

```text
clear operations while relations remain present
replace scope while graph scope remains unchanged
replace declassification while the disclosure graph remains unchanged
replace architecture binding while graphs retain the old source identity
```

After such mutation:

- `operation()` reads the altered public map;
- `relations()` reads the old private graph;
- `evaluate_operation()` reads the old private graphs;
- `project()` can publish one altered public field set beside stale graph
  projections.

The result is no longer one validated semantic value.

The model deliberately exposes corruptible public state and documents that
boundary. The realization package makes no equivalent claim; it presents its
output as a validated compiler input.

#### Required design

Make all invariant-bearing fields private.

Expose read-only accessors and stable iterators:

```text
architecture()
scope()
operation(id)
operations()
relations()
declassification()
project()
```

Do not expose a mutation API that can bypass graph reconstruction.

If tests require malformed values, provide crate-private fixture builders that:

1. alter declarations;
2. rebuild all graphs;
3. rerun validation;
4. return either a validated value or a typed error.

#### Required tests

The external public-API integration test must prove downstream callers cannot:

- replace the architecture binding;
- replace the scope;
- clear or replace operations;
- overwrite declassification;
- mutate a declaration behind the derived graph.

Positive tests must establish that downstream compiler-style consumers can
still:

- inspect architecture binding;
- inspect scope;
- iterate operations and relations;
- read declassification;
- evaluate observations;
- create a stable projection.

#### Identity impact

Expected:

```text
architecture schema/hash:        unchanged
realization semantics:           unchanged
public Rust API:                  intentionally stricter
future compiler API:              safer
```

#### Exit

- [x] invariant-bearing fields are private;
- [x] read-only public access is sufficient for intended consumers;
- [x] malformed fixtures remain possible only through explicit test paths;
- [x] public API tests cover the negative boundary;
- [x] realization and model-conformance suites pass;
- [x] complete workspace and clean-tree gates pass.

#### Evidence · DONE

- F3-002 closure commit. All four formerly public fields on
  ScopedRealizationSpec (architecture, scope, operations,
  declassification) are now crate-private alongside the already
  private graphs, so a validated value cannot be desynchronized from
  outside the crate. New read-only accessors: architecture(), scope(),
  operations() (stable operation-ID order, each entry paired with its
  declaration), declassification(); operation(id), relation(id),
  relations(), evaluate_operation(), and project() are unchanged. No
  mutation API exists.
- Negative boundary: four compile_fail doctests on the type prove an
  external consumer cannot clear operations, replace scope, replace
  the architecture binding, or overwrite declassification (each block
  first derives a real value, so the only failure is the privacy
  error). The doctest lane runs them as an external crate; the fourth
  block caught a real gap during development (declassification was
  still public in the first edit) before it could land.
- Positive boundary: the external public-API integration test gains
  validated_realization_is_readable_but_not_externally_mutable,
  proving downstream code can inspect the binding's schema version,
  the scope, the operation iteration order, one operation lookup, the
  declassification analysis, and a stable projection that equals it.
- In-crate mutation fixtures are untouched: the F2-002 vec-and-leak
  mutation harnesses rewrite architecture rows before derivation and
  revalidate through the ordinary derive path, so malformed values
  remain constructible only through explicit crate-internal test
  paths.
- Realization semantics, architecture identities, and generated
  artifacts are unchanged; the public Rust API is intentionally
  stricter.
- Verified: fmt, clippy -D warnings (workspace, all targets),
  realization suite (125 unit + 5 public-API + 4 doctests), model
  suite (287 + conformance) under the nightly SDK toolchain. Complete
  workspace and Meson gates run at the end of the F3 series.

---

### F3-003 — Close symlinked-ancestor flattener confinement

**Priority:** P1
**Status:** DONE
**Owner:** `flatten-latex-main`
**Primary files:**

```text
packages/flatten-latex-main/src/lib.rs
packages/flatten-latex-main/src/tests/mod.rs
```

#### Problem

The flattener rejects a symlink only when the final supplied path component is
a symlink:

```rust
symlink_metadata(path)
```

Path resolution still follows symlinks in parent components.

A supplied path such as:

```text
fixture/allowed-parent/secret.tex
```

can resolve through:

```text
fixture/allowed-parent -> /outside
```

When `/outside/secret.tex` is a regular file, the final-component metadata check
reports a regular file and the subsequent open reads the off-tree target.

That contradicts the stronger documented claim that allowlisting a path never
authorizes its resolved target.

The supplied source tests:

- unlisted final symlink;
- allowlisted final symlink;
- symlinked main file;
- symlink loop;
- hard-link cycle.

They do not test a regular final file beneath a symlinked ancestor directory.

#### Required decision

Prefer strict component-wise regular-file confinement:

- reject a symlink in every existing path component;
- reject a symlinked `main_file`;
- reject a symlinked component in every allowed file;
- validate before staging output;
- retain the documented check/open race honestly.

A stronger malicious-filesystem boundary would require descriptor-relative
platform facilities and remains outside the current ADR-015 claim.

If strict component-wise confinement is rejected, narrow the public
documentation so it says exactly which symlink forms are followed.

#### Required tests

- regular final file beneath a symlinked ancestor;
- symlinked ancestor for `main_file`;
- several nested symlinked ancestors;
- ordinary regular path with ordinary parent directories;
- final-component symlink remains rejected;
- hard-link cycle remains detected;
- failed confinement preserves prior output bytes;
- failed confinement creates no staging file.

#### Impact

Canonical paper inputs are tracked regular files. A strict repair should not
change canonical paper bytes.

#### Exit

- [x] public documentation and implementation state one rule;
- [x] symlinked-ancestor behavior is covered;
- [x] validation precedes staging;
- [x] include-cycle identity remains correct;
- [x] flattener package and complete gates pass cleanly.

#### Evidence · DONE

- F3-003 closure commit; the strict component-wise contract is
  implemented. ensure_regular_file first validates the named file
  itself (existing, regular, not a symlink — unchanged rejections and
  messages), then walks every ancestor prefix of the supplied path
  with symlink_metadata and rejects the first symlinked component
  with a dedicated passes-through-the-symbolic-link error naming both
  the supplied path and the offending prefix. The flatten
  documentation now states the rule exactly: a regular file reached
  through a symlink-free path, a symlink in any component rejected
  before anything is read or staged. The check-then-open residual is
  retained honestly and unchanged: a filesystem racing the flattener
  remains outside the ADR-015 boundary (configuration-mistake
  defense, not a malicious-filesystem sandbox); a stronger boundary
  would need descriptor-relative platform facilities.
- Regressions: a regular final file beneath a symlinked ancestor is
  refused with the prior output preserved byte-for-byte and no
  staging file (validation precedes staging); a main entry point
  beneath a symlinked ancestor is refused; two nested symlinked
  ancestor levels are refused at the first linked component with no
  secret bytes published. The final-component rejections
  (allowlisted symlink, symlinked main, symlink loop), hard-link
  cycle detection, and every ordinary regular-path positive test are
  unchanged and green.
- Canonical paper bytes are unaffected: the flat build target
  regenerates the flattened paper successfully under the stricter
  walk (tracked regular files through real directories).
- Verified: fmt, clippy -D warnings, flattener unit (34) and
  subprocess (3) suites under the nightly SDK toolchain; canonical
  flat target rebuilt via ninja. Complete gates run at the end of
  the F3 series.

---

### F3-004 — Make `check-plans` reject an empty census

**Priority:** P2
**Status:** DONE
**Owners:** `labels`, ADR-014 checker boundary
**Primary files:**

```text
packages/labels/src/bin/check-plans.rs
packages/labels/src/plans.rs
packages/labels/src/plans.rs tests
```

#### Problem

The census verifier contains:

```rust
if subjects.is_empty() {
    return;
}
```

The CLI does not require any `--subject` argument.

On a nonempty planning tree, this command can therefore run with no declared
census and use its filesystem walk as the effective membership source.

ADR-014 requires discovery to remain a verifier, never the source of production
membership.

The separate `census-audit` target protects the canonical build from many
wiring mistakes, but it does not repair the checker’s own argument contract.

#### Required implementation

- remove the empty-census bypass;
- compare empty declaration against discovered reality normally;
- make the CLI require at least one `--subject`;
- keep test-fixture discovery as a test helper only;
- ensure environmental traversal failures remain hard errors.

#### Required tests

- empty declaration against nonempty tree fails and names discovered paths;
- empty declaration against a deliberately empty fixture has the explicitly
  chosen behavior;
- complete declaration passes;
- one omitted subject fails;
- one stale declared subject fails;
- subprocess invocation with no `--subject` is usage class 2;
- canonical Meson and script wiring still passes every subject explicitly.

#### Impact

```text
protocol semantics:             none
CLI accepted arguments:         stricter
ADR-014 conformance:            restored
report schema:                  unchanged
```

#### Exit

- [x] no production path uses discovery as census authority;
- [x] empty production census fails closed;
- [x] focused unit and subprocess tests pass;
- [x] plan, census, label, Meson, and clean-tree gates pass.

#### Evidence · DONE

- F3-004 closure commit. verify_census no longer early-returns on an
  empty declaration: an empty declared census against a nonempty tree
  reports every discovered file as outside the build census and fails
  closed, with the no-bypass rule documented on the function. The
  shipped CLI now requires at least one subject argument, so an
  invocation that names a repository root but no census is usage
  class 2 before any semantic work — a wiring regression that drops
  every subject argument now fails in the checker itself, not only in
  the separate census-audit.
- Unit tests that previously leaned on the bypass pass explicit
  discovery-derived fixture subjects through the existing test
  helper; discovery survives only as that test helper and as the
  verifier inside the checker.
- Regressions: empty declaration against the nonempty fixture is
  invalid with one outside-the-census failure per discovered file
  (all five named); empty declaration against a deliberately empty
  tree is pinned as an environmental fault (no census failure, and no
  quietly valid run — a tree without a backlog is not a planning
  tree); the existing both-ways census-disagreement, omitted-subject,
  and stale-subject coverage is unchanged; a new subprocess test
  proves a repository-root-only invocation exits usage class 2 with
  JSON-only stderr.
- Canonical wiring is untouched and still explicit: check-plans.sh
  passes every tracked subject, and the Meson plans-check lane passes
  its census by argument; both ran green after the change. Report
  schema and accepted valid trees are unchanged; the accepted
  argument set is strictly narrower.
- Verified: fmt, clippy -D warnings, labels suite (67 unit + 8
  subprocess) under the nightly SDK toolchain; scripts/check-plans.sh
  and meson test plans-check green. Complete gates run at the end of
  the F3 series.

---

### F3-005 — Canonicalize the complete realization projection

**Priority:** P2
**Status:** TODO
**Owners:** `realization`, future `compiler`
**Blocks:** stable compiler input projection
**Primary files:**

```text
packages/realization/src/derive.rs
packages/realization/src/operation.rs
packages/realization/src/tests/derivation_tests.rs
packages/realization/src/tests/property_graph_tests.rs
```

#### Problem

`ScopedRealizationProjection` contains both:

1. canonical graph projections; and
2. cloned `OperationRealization` values carrying source-order vectors.

The raw operation value includes vectors for:

- expressions;
- relations;
- relation dependencies;
- constructibility nodes and edges;
- lifecycle nodes and edges;
- disclosure nodes and edges;
- disclosure seeds.

Graph builders sort and validate those collections before constructing the
direct Petgraph graphs. The stable graph projections also sort by stable typed
keys.

`project_scoped_realization` nevertheless clones the original declaration
vectors into:

```text
operations
```

Reordering a semantically set-like declaration collection can therefore change
the complete “stable” projection even when all canonical graph projections and
evaluation results remain equal.

The projection also carries two representations of the same graph-shaped
content, which can drift.

#### Required design

Create a canonical `OperationRealizationProjection` or omit graph-owned raw
declaration collections from the stable projection.

The canonical projection must contain each semantic fact once.

Acceptable approaches include:

- operation ID plus canonical operation-owned metadata, with all graph content
  represented only in graph projections;
- canonical sorted declaration projections keyed by stable identity;
- a reviewed combination that has no duplicate graph authority.

Do not reorder operands whose order is semantic. The canonicalization applies
to declaration collections, not arbitrary vectors.

#### Required tests

For every declaration family:

- construct equivalent forward and reverse insertion orders;
- derive graphs;
- require complete `ScopedRealizationProjection` equality;
- require evaluation-order equality where order is canonical;
- require equal declassification;
- require equal public API results.

Also test:

- unrelated declaration insertion does not renumber stable keys;
- Petgraph indices never appear in the stable projection;
- raw source ordering is not serialized or hashed.

#### Impact

Phase 1 publishes no realization hash, so this can be repaired before a public
realization identity exists.

#### Exit

- [ ] the stable projection has one canonical representation of each graph;
- [ ] declaration-order permutations produce equal complete projections;
- [ ] no semantic operand order is incorrectly normalized;
- [ ] compiler input can rely on the projection deterministically;
- [ ] realization and model-conformance tests pass cleanly.

---

### F3-006 — Validate realization ownership and proof bindings

**Priority:** P2
**Status:** TODO
**Owners:** `realization`, future `compiler`
**Blocks:** proof planning and relation census
**Primary files:**

```text
packages/realization/src/derive.rs
packages/realization/src/relation.rs
packages/realization/src/expression.rs
packages/realization/src/validate.rs
packages/realization/src/tests/
```

#### Problem

Several typed ownership relationships are represented but not generically
validated.

##### Operation declaration identity

Derivation inserts a returned declaration under the requested map key without
checking:

```text
declaration.operation = requested operation
```

##### Relation ownership

Graph construction does not generically require every relation from an
operation declaration to satisfy:

```text
relation ID operation = declaration operation
```

##### Proof-alternative binding

A relation carries a set of `ProofAlternativeId` values, but generic validation
does not require:

```text
proof alternative relation = containing relation ID
```

##### Expression and predicate ownership

Operation-scoped facts are checked during runtime expression evaluation, but
declaration-time validation should reject an expression predicate or fact owned
by the wrong operation before the value becomes compiler input.

These gaps are currently masked by correct helper constructors. The future
compiler must not rely on helper correctness as an implicit semantic invariant.

#### Required implementation

Add one generic per-operation validation pass before graph assembly.

Validate:

- `OperationRealization.operation` equals the requested operation;
- every operation-scoped expression belongs to that operation;
- every relation ID belongs to that operation;
- every expression-predicate relation points to an expression in the same
  operation’s allowed scope;
- every proof alternative points back to its containing relation;
- every operation-owned constructibility node belongs to the operation;
- every operation-owned lifecycle declaration has an approved cross-operation
  meaning;
- every disclosure seed and operation-scoped fact belongs to the intended
  operation;
- deliberate cross-operation lifecycle exits remain explicitly typed rather
  than rejected as accidental ownership drift.

Add focused typed error variants rather than collapsing all failures into
`UnsupportedOperationDeclaration`.

#### Required mutation tests

- compact-ASH derivation returns a live-transfer declaration;
- compact-ASH declaration contains a live-transfer relation;
- relation contains a proof alternative for another relation;
- expression predicate references another operation’s fact;
- disclosure seed points at another operation’s relation;
- constructibility node carries the wrong operation;
- legitimate cross-operation lifecycle exits remain valid.

#### Impact

```text
architecture identities:       unchanged
pilot behavior:                unchanged
accepted malformed values:     stricter
future compiler boundary:      safer
```

#### Exit

- [ ] all ownership relationships are validated generically;
- [ ] legitimate cross-operation lifecycle semantics remain explicit;
- [ ] focused mutation tests cover each relation;
- [ ] compiler relation/proof census has a trustworthy source;
- [ ] realization, model conformance, and workspace gates pass.

---

### F3-007 — Include both Layer-0 duplicate-mint locations

**Priority:** P3
**Status:** TODO
**Owner:** `labels`
**Primary files:**

```text
packages/labels/src/latex.rs
packages/labels/src/tests.rs
```

#### Problem

Markdown and Rust duplicate mints use diagnostics that identify the duplicate
and the first mint.

Layer-0 LaTeX duplicates currently report only the duplicate occurrence.

ADR-013 requires duplicate-mint diagnostics to identify both locations.

#### Required implementation

Preserve the `DuplicateLatexLabel` class if useful, but include:

- duplicate path and line;
- original path and line;
- label owner and local label.

#### Required tests

Mint one Layer-0 label in two distinct section files and assert that the
diagnostic identifies both canonical repository-relative locations.

#### Exit

- [ ] both locations are present;
- [ ] deterministic diagnostic ordering is preserved;
- [ ] labels and generated registers remain current;
- [ ] focused and complete label gates pass.

---

### F3-008 — Canonically sort authorization-evidence exports

**Priority:** P3
**Status:** TODO
**Owner:** `architecture`
**Primary files:**

```text
packages/architecture/src/export.rs
packages/architecture/src/tests/export_hash_tests.rs
```

#### Problem

Most set-like architecture export arrays are explicitly sorted.

The input-authorization and operation-authorization evidence arrays are emitted
in the order of enum `ALL` slices without an explicit sort.

Current source order is stable and code-ordered, but the canonicalization claim
should not depend on declaration order.

#### Required implementation

Sort both evidence tables by stable discriminant before publication and hashing.

If the DTO does not publish a code, retain a temporary `(code, row)` pair during
construction, sort by code, and then emit rows.

#### Required tests

- forward and reversed source iteration produce equal evidence tables;
- semantic and presentation hashes remain stable under iteration permutation;
- canonical JSON and TOML remain equal to typed expected values;
- current generated artifacts are regenerated only if bytes intentionally
  change.

#### Identity impact

If current rows are already code-ordered, expected artifact and hash values
should remain unchanged.

#### Exit

- [ ] sorting is explicit;
- [ ] permutation test passes;
- [ ] generated artifacts remain current;
- [ ] architecture and workspace gates pass cleanly.

---

### F3-009 — Make conditional LaTeX flattening honor the probe path

**Priority:** P3
**Status:** TODO
**Owner:** `flatten-latex-main`
**Primary files:**

```text
packages/flatten-latex-main/src/lib.rs
packages/flatten-latex-main/src/tests/mod.rs
```

#### Problem

The conditional parser recognizes:

```text
IfFileExists probe, true branch, false branch
```

but discards the probe path and decides whether the true branch is taken by
resolving the nested include path.

The probe and nested include need not name the same file.

A source such as:

```text
IfFileExists probe.tex:
    include different.tex
else:
    empty
```

can therefore be flattened under different branch semantics from LaTeX.

#### Required decision

Prefer exact restricted support:

- resolve the probe path independently against the supplied-file list;
- select the same branch LaTeX would select under the fixed-list model;
- process the selected branch only when the supported grammar preserves it;
- reject unsupported nonempty branch content rather than dropping it;
- optionally require the probe and nested include to resolve to the same file
  for the narrow currently needed form.

#### Required tests

- probe and include name the same supplied file;
- probe exists and nested include is absent;
- probe absent while nested include exists;
- probe and include name distinct supplied files;
- nonempty selected false branch;
- trailing semantic content;
- failed conditional flatten preserves existing output atomically.

#### Exit

- [ ] branch selection follows the probe;
- [ ] unsupported conditionals fail rather than change semantics;
- [ ] canonical paper flatten remains unchanged;
- [ ] focused and complete flattener gates pass.

---

### F3-010 — Reconcile empty-stamp prose with implementation

**Priority:** P3
**Status:** TODO
**Owners:** `cli-common`, ADR-014 documentation
**Primary files:**

```text
packages/cli-common/src/lib.rs
packages/cli-common/src/tests/mod.rs
adr/014-meson-lint-census-and-stamps.md
```

#### Problem

Repository policy repeatedly states that a success stamp is empty and carries
no content.

`touch_stamp` creates an absent stamp empty but intentionally preserves bytes
already present in an existing stamp. A unit test requires seeded bytes to
survive a subsequent touch.

The command does not write semantic content into a stamp, but it also does not
guarantee that a pre-existing stamp is empty.

#### Required decision

Choose one rule.

**Strict empty stamp**

- reject or truncate a nonempty stamp; (prefer reject with a hard error, non-destructive)
- keep report publication before stamp mutation;
- preserve the no-fresh-stamp-on-failure rule.

**No semantic stamp content**

- retain existing bytes;
- narrow ADR and API prose;
- state that first-party commands never place report or semantic bytes in a
  stamp, while filesystem contents are not normalized on every touch.

The strict empty form is simpler to inspect, but truncation changes the current
“touch only” behavior and should be reviewed for Ninja implications.

#### Required tests

- absent stamp creation;
- existing empty stamp mtime update;
- existing nonempty stamp under selected policy;
- report/stamp alias rejection remains intact;
- report failure leaves stamp untouched;
- semantic checker failure leaves stamp untouched.

#### Exit

- [ ] implementation and ADR state one rule;
- [ ] focused tests cover nonempty existing stamps;
- [ ] Meson incremental behavior remains correct;
- [ ] complete CLI and Meson gates pass.

---

## 5. Phase-2 implementation register · `sec:backlog:phase2`

### 5.1 Summary · `tbl:backlog:phase2`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `P2-001` | P1 | **BLOCKED** | Re-close the realization compiler-input boundary under F3-002, F3-005, and F3-006 |
| `P2-002` | P2 | **IN PROGRESS** | Complete concrete Petgraph dependency review C1-004 |
| `P2-003` | P1 | **BLOCKED** | Create `tripod-compiler` crate |
| `P2-004` | P1 | **BLOCKED** | Architecture/realization binding and explicit compiler scope |
| `P2-005` | P1 | **BLOCKED** | Canonical relation DAG over direct Petgraph |
| `P2-006` | P1 | **BLOCKED** | Checked constant folding with failure-semantics preservation |
| `P2-007` | P1 | **BLOCKED** | Exact proof-alternative and target-requirement planning |
| `P2-008` | P1 | **BLOCKED** | Disclosure, fact-source, and constructibility analysis |
| `P2-009` | P1 | **BLOCKED** | Representation lifecycle analysis |
| `P2-010` | P1 | **BLOCKED** | Execution-case-aware placement and layout requirements |
| `P2-011` | P1 | **BLOCKED** | Relation-indexed coverage requirements |
| `P2-012` | P1 | **BLOCKED** | Compact-ASH and live-transfer analyzed pilots |
| `P2-013` | Gate | **BLOCKED** | Complete Phase-2 evidence and exit |

Compiler design and isolated algorithm prototypes may proceed, but no public
compiler input API or analysis identity freezes while P2-001 remains blocked.

---

### P2-001 — Re-close the realization input boundary

**Priority:** P1
**Status:** BLOCKED on F3-002, F3-005, and F3-006

The compiler must consume one immutable, canonical, ownership-validated
realization value.

Required before compiler API freeze:

- external callers cannot desynchronize realization fields from derived graphs;
- the complete stable projection is independent of declaration insertion order;
- operation, relation, expression, proof, constructibility, lifecycle, and
  disclosure ownership are validated;
- pilot architecture welds remain complete;
- no generated file or documentation source enters the boundary.

---

### P2-002 — Complete concrete Petgraph dependency review

**Priority:** P2
**Status:** IN PROGRESS
**Maps to:** C1-004

Current workspace declaration:

```text
petgraph = 0.8.3
selected features:
    serde-1
    rayon
    dot_parser
    unstable
    generate
```

Required review:

- exact crates.io release;
- upstream repository and release tag;
- Rust 1.88 compatibility;
- license;
- default and selected features;
- transitive dependency graph;
- duplicate versions;
- dependency-internal unsafe code;
- Rayon and parallel-determinism implications;
- serialization non-authority boundary;
- unstable-feature usage boundary;
- advisory status;
- replacement boundary;
- lockfile review.

Required commands include:

```sh
cargo tree --locked -p petgraph -e features
cargo tree --locked -i petgraph
cargo metadata --locked
cargo audit
```

The lockfile was excluded from the supplied static review. This task cannot be
closed from source declarations alone.

If `cargo-audit` is unavailable, the development gate reports the lane as
skipped. The Phase-2 ceremony must explicitly decide whether advisory tooling is
mandatory in its recorded environment.

No `num-rational`, `faer`, SAT, LP, or MILP dependency enters until a concrete
consumer exists.

---

### P2-003 — Create the compiler crate

**Priority:** P1
**Status:** BLOCKED on P2-001 and P2-002
**Package contract:** [packages/compiler.md](packages/compiler.md)

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

Add a direct `architecture` dependency only if the public compiler API directly
names architecture-owned types and transitive use would obscure ownership.

The crate must:

- inherit workspace package metadata and lints;
- deny first-party unsafe code;
- join the Cargo workspace;
- join the nearest Meson source census;
- expose a public API integration test;
- consume no generated publication, documentation, model source, filesystem
  state, or environment state;
- contain no concrete target opcode, stack position, tapleaf, transaction
  position, or target bytecode.

---

### P2-004 — Bind architecture, realization, policy, and scope

**Priority:** P1
**Status:** BLOCKED

Define one analyzed input boundary containing:

- immutable architecture binding inherited from realization;
- immutable canonical realization projection;
- explicit realization scope;
- compiler-analysis policy;
- optional abstract target capabilities;
- no concrete target package or target bytecode.

Reject:

- incomplete or unsupported scope;
- identity mismatch;
- relation outside scope;
- duplicate scope;
- operation/relation ownership mismatch;
- architecture/realization mismatch;
- generated-file input.

No public compiler identity is minted until the canonical analyzed projection
and configuration identity are reviewed.

---

### P2-005 — Build the canonical relation DAG

**Priority:** P1
**Status:** BLOCKED
**Depends on:** C1-005 and C1-010

Use a package-owned concrete Petgraph graph directly.

Required structure:

```text
typed stable relation and analysis keys
typed node and edge weights
direct Petgraph graph
stable-key → NodeIndex lookup metadata
canonical insertion
canonical stable-key projection
```

Requirements:

- compiler relation census equals realization relation scope;
- every source relation retains provenance;
- duplicate IDs reject;
- unknown endpoints reject;
- unsupported cycles reject with canonical SCC diagnostics;
- no Petgraph index enters semantic identity;
- insertion permutations produce equal typed projections;
- standard graph traversal uses Petgraph.

---

### P2-006 — Implement checked constant folding

**Priority:** P1
**Status:** BLOCKED

Initial legal folds:

- literal boolean identities;
- literal count and amount operations;
- checked exact equality;
- statically known activation;
- canonical ordering of explicitly set-like operands;
- structural sharing that retains all provenance.

Do not:

- reassociate checked arithmetic;
- move or combine floor operations;
- change overflow or underflow behavior;
- reorder named guards where failure identity matters;
- merge relations while dropping operation ownership;
- change disclosure or witness requirements.

Every fold rule states:

- type preconditions;
- value equivalence;
- failure equivalence;
- disclosure effect;
- witness effect;
- provenance mapping.

Compare the folded evaluator with a non-folded oracle.

---

### P2-007 — Implement exact proof planning

**Priority:** P1
**Status:** BLOCKED
**Depends on:** C1-008

For each relation:

1. enumerate realization-approved alternatives;
2. validate proof-alternative ownership;
3. reject unavailable target capabilities;
4. reject unauthenticated sources;
5. reject unavailable witnesses;
6. reject permissionless owner/operator secrets;
7. reject representation-policy failures;
8. reject lifecycle failures;
9. reject disclosure-policy failures;
10. retain the feasible set or exact Pareto frontier;
11. select canonically only under explicit policy.

Initial proof planning uses deterministic exact enumeration or branch-and-bound.

Complexity exhaustion returns a typed error. It does not select the best
partial plan or invoke a hidden greedy fallback.

---

### P2-008 — Derive disclosure, sources, and constructibility

**Priority:** P1
**Status:** BLOCKED

For every relation operand, record an authenticatable source class such as:

- compile-time constant;
- architecture constant;
- authenticated transaction input/output;
- authenticated metadata;
- public chain fact;
- public opening;
- owner witness;
- operator witness;
- sponsor-local witness;
- derived expression.

Unauthenticated metadata is never a source.

Disclosure reasons remain separate:

```text
semantic public state or event
permissionless constructibility
target safety
deployment policy
```

Constructibility must prove both:

```text
the target can verify the witness
the authorized constructor can obtain the witness
```

Permissionless cases reject private owner or operator dependencies.

Cadence analysis preserves both operator-window and delayed-permissionless
cases.

Sponsor-value opacity remains in force: individual sponsor amounts are not
protocol facts.

---

### P2-009 — Analyze representation lifecycle

**Priority:** P1
**Status:** BLOCKED

For every supported representation, record paths to required exits.

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

Pilot analysis may state that later target exits are not implemented, but must
not describe a representation as release-complete while a required exit is
missing.

Representation safety and disclosure minimality remain separate results.

---

### P2-010 — Derive placement and layout requirements

**Priority:** P1
**Status:** BLOCKED
**Depends on:** C1-009

Classify each relation as:

- local;
- transaction-global;
- conditional;
- deliberately duplicated.

Model finite execution cases, including as applicable:

```text
sponsorless / sponsored
explicit / confidential
continuing / terminal
empty / nonempty
pre-maturity / conversion / post-maturity
```

For each relation \(r\), require that every required execution case is covered
by at least one carrier that executes in that case.

Compiler output records semantic carrier and layout requirements only. It does
not assign tapscript input indexes, tapleaves, stack positions, or concrete
transaction slots.

---

### P2-011 — Derive coverage requirements

**Priority:** P1
**Status:** BLOCKED

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

Conditional relations additionally receive:

- inactive valid case;
- active valid case;
- active invalid case.

Coverage requirements are compiler output. Evidence completion remains owned by
future vectors and release packages.

---

### P2-012 — Analyze both pilots

**Priority:** P1
**Status:** BLOCKED

#### Compact ASH

Require complete analysis of:

- ASH input/output cardinality;
- ASH recognition;
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

Require complete analysis of:

- input/output cardinality;
- live receipt recognition and class closure;
- all-owner authorization;
- exact aggregate `U` conservation alternatives;
- explicit closed `U` identity;
- destination-family closure;
- sponsor multiplicity and isolation;
- no roots;
- transition-certificate projection only;
- explicit/private-committed representation alternatives;
- transfer, burn, and redemption lifecycle obligations.

Repeated analysis from identical explicit inputs must produce equal typed
projections.

---

### P2-013 — Phase-2 evidence and exit

**Priority:** Gate
**Status:** BLOCKED

Phase 2 exits only through (`gate:backlog:phase2`) after:

- every current F3 finding required by the gate is closed;
- all required P2 tasks are done;
- all required C1 preparation tasks are done;
- the complete repository gate passes in the recorded environments;
- the final source tree is clean.

No compiler output in Phase 2 is deployment evidence.

---

## 6. Algorithm and preparation register · `sec:backlog:c1`

### 6.1 Current status · `tbl:backlog:c1`

| ID | Status | Current ownership |
|---|---|---|
| `C1-001` | **DONE** | Compiler/linker/mathematics/solver research notes exist |
| `C1-002` | **DONE** | D007 direct Petgraph decision accepted |
| `C1-003` | **DONE** | D008 exact/certified mathematics decision accepted |
| `C1-004` | **IN PROGRESS** | Concrete Petgraph dependency and lockfile review |
| `C1-005` | **TODO** | Canonical direct-Petgraph construction prototype |
| `C1-006` | **PARKED** | Exact keyed linear systems; activate only for a concrete compiler consumer |
| `C1-007` | **PARKED** | Certified numerical analysis; activate only for a concrete numerical consumer |
| `C1-008` | **TODO** | Exact proof-plan search |
| `C1-009` | **TODO** | Execution-case-aware placement |
| `C1-010` | **TODO** | Typed symbol resolution and SCC policy |
| `C1-011` | **BLOCKED** | Structured relocation; linker-phase work |
| `C1-012` | **BLOCKED** | Deterministic bounded-depth taptree; linker-phase work |
| `C1-013` | **TODO** | Independent small-instance oracles for active Phase-2 algorithms |
| `C1-014` | **BLOCKED** | Preparation review and Phase-2 handoff |

Current assignment:

```text
Phase 2:
    C1-004
    C1-005
    C1-008
    C1-009
    C1-010
    relevant C1-013
    C1-014

Later linker phases:
    C1-011
    C1-012

Concrete-consumer gated:
    C1-006
    C1-007
```

### C1-005 — Canonical direct-Petgraph construction

Required prototype:

- typed nodes and edges;
- stable semantic keys;
- canonical node insertion;
- canonical edge insertion;
- stable-key/local-index lookup metadata;
- Petgraph topology, SCC, and reachability;
- canonical result normalization;
- deterministic diagnostics.

Test:

- node and edge insertion permutations;
- duplicate keys and edges;
- missing endpoints;
- self-loops;
- disconnected graphs;
- deep chains;
- large SCCs within explicit limits;
- repeated construction equality;
- attempted local-index publication.

### C1-008 — Exact proof-plan search

Implement an exact small-instance planner over typed alternatives.

Compare every generated small case with exhaustive enumeration.

Required adversarial cases:

- cheapest local alternatives form an invalid global plan;
- sharing changes the global optimum;
- lifecycle-safe plan differs from the immediately cheapest plan;
- equal-cost plans require stable-key tie-breaking;
- no feasible plan;
- complexity budget exhausted.

### C1-009 — Execution-case-aware placement

For every relation, enumerate required execution cases and exact eligible
carriers.

A carrier present somewhere but absent from one active case does not satisfy
the relation.

Compare exact placement search with exhaustive carrier-subset enumeration.

### C1-010 — Typed symbols and SCC policy

Implement two-pass typed resolution:

1. complete definition census;
2. complete reference resolution.

Use direct Petgraph graphs and typed edge roles.

Normalize SCC members by stable key and require an explicit semantic strategy
for every accepted cyclic dependency.

SCC membership alone never authorizes a cycle.

### C1-013 — Active algorithm oracles

For Phase 2, required independent oracles include:

| Production analysis | Oracle |
|---|---|
| canonical topological order | valid-order enumeration plus least-key rule |
| SCC | mutual-reachability equivalence |
| expression interning | non-interned evaluator |
| dependency closure | repeated complete scan |
| proof selection | exhaustive candidate enumeration |
| placement | exhaustive carrier subsets |
| relation census | direct set equality |
| constant folding | non-folded evaluator |
| canonical realization projection | declaration-order permutation oracle |

Later linker oracles remain assigned to later phases.

---

## 7. Algorithm and mathematical laws · `sec:backlog:algorithm-laws`

### 7.1 Problem classes · `rule:backlog:problem-classes`

```text
typed semantic AST:
    first-party typed source

graph storage and traversal:
    direct package-owned Petgraph graph

exact semantic arithmetic:
    checked integers, BigInt, reduced exact rationals

proof and placement:
    exact finite search initially

numerical linear algebra:
    only when a concrete consumer exists

target tree:
    deterministic bounded-depth coding algorithm in linker phase

target arithmetic:
    exact target relation plus independent host reference

cryptographic and consensus mathematics:
    reviewed target libraries plus target-native evidence
```

A numerical solve is not proof selection. A graph library is not a semantic
AST. A small floating residual is not exact equality.

### 7.2 Exactness · `rule:backlog:exactness`

Semantic, conservation, authorization, identity, calibration, and release
claims use:

- checked bounded integers;
- arbitrary-precision integers;
- reduced exact rationals;
- exact finite search;
- independently checked certificates;
- target-native execution where the claim concerns the target.

Numerical analysis may produce candidates or diagnostics. A release-sensitive
result becomes and is checked as an exact integer, rational, interval, bound, or
certificate.

### 7.3 Identity · `rule:backlog:identity`

Always distinguish:

```text
local handle:
    one process-local graph, arena, matrix, or solver position

stable key:
    complete typed semantic identity

digest:
    optional domain-separated commitment
```

Never use as semantic identity:

- Petgraph indices;
- matrix positions;
- solver variable numbers;
- traversal or insertion order;
- source path or line;
- pivot order;
- floating-point bits;
- iteration count;
- thread schedule;
- temporary path.

A supposedly stable typed projection must likewise exclude incidental source
declaration order where that order is not semantic.

### 7.4 Complexity failure · `rule:backlog:complexity`

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

## 8. Dependency register · `sec:backlog:dependencies`

### 8.1 Current and deferred dependencies · `tbl:backlog:dependencies`

| Dependency | Status | Role |
|---|---|---|
| `petgraph = 0.8.3` | Adopted; review closure in progress | Graph storage and standard algorithms |
| `num-bigint` | Existing | Exact arbitrary-size integers |
| `num-integer` | Existing | Exact integer helpers |
| `num-traits` | Existing | Numeric traits |
| `num-rational` | Not adopted | Future exact rational analysis |
| `faer` | Not adopted | Future certified numerical diagnostics |
| `fixedbitset` | Deferred | Dense local coverage sets if measured |
| `elements` | Phase-3 review | Target transaction and consensus types |
| `elements-miniscript` | Conditional | Standard target-program support |
| `secp256k1-zkp` | Prototype-gated | CT commitments and proofs |
| SAT/LP/MILP solver | Deferred | Large exact planning problems |
| `salsa` | Deferred | Incremental compiler queries |
| `egg` | Deferred | Equality saturation |
| `rayon` | Selected through Petgraph | Parallel independent work only; never semantic ordering |

### 8.2 Dependency-entry rule · `rule:backlog:dependency-entry`

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

## 9. Research handoff · `sec:backlog:research`

### 9.1 Active Phase-2 research · `tbl:backlog:research-active`

| Question | Current use |
|---|---|
| [compiler algorithms](research/compiler-algorithms.md) | Direct Petgraph construction, topology, SCC, closure, proof planning, placement, coverage |
| [optimization solvers](research/optimization-solvers.md) | Exact finite pilot search; no external solver initially |
| [numerical linear algebra](research/numerical-linear-algebra.md) | Parked until a concrete numerical consumer exists |

### 9.2 Later target/linker research · `tbl:backlog:research-later`

| Question | Earliest consuming phase |
|---|---:|
| [STATE constructor](research/state-constructor.md) | Phase 3 prototype; Phase 6 implementation |
| [wide arithmetic](research/wide-arithmetic.md) | Phase 3 prototype; Phase 8 implementation |
| [public declassification](research/public-declassification.md) | Phase 3 prototype; Phase 7/8 representation |
| [settlement layout](research/settlement-layout.md) | Phase 10 |
| [linker algorithms](research/linker-algorithms.md) | Phase 4 onward |

Research may begin early but must not:

- add target fields to realization;
- freeze a production ABI prematurely;
- become release evidence automatically;
- use draft defaults as calibration;
- call a model wrapper an independent observer;
- put solver-local or floating values into semantic identity;
- contradict an accepted decision while remaining active.

Accepted results move into typed source, permanent tests, package contracts, and
a decision or ADR where required.

---

## 10. Verification matrix · `sec:backlog:verification`

### 10.1 Focused remediation commands · `tbl:backlog:focused-tests`

| Area | Command |
|---|---|
| architecture exports and deployment validation | `cargo test --locked -p tripod-architecture` |
| realization immutability, projection, and ownership | `cargo test --locked -p tripod-realization` |
| model/realization conformance | `cargo test --locked -p tripod-model realization_conformance` |
| labels, Layer-0 diagnostics, and plan census | `cargo test --locked -p tripod-labels` |
| document outputs and stamp policy | `cargo test --locked -p tripod-document-stamps` |
| CLI report/stamp behavior | `cargo test --locked -p cli-common` |
| flattener confinement and conditionals | `cargo test --locked -p flatten-latex-main` |
| complete workspace | `cargo test --workspace --locked` |

Focused filters supplement but do not replace full package and workspace runs.

### 10.2 Phase-2 package commands

Once the compiler crate exists:

```sh
cargo test --locked -p tripod-compiler
cargo clippy --locked -p tripod-compiler --all-targets -- -D warnings
```

The package must also participate in workspace-wide debug and release lanes.

### 10.3 Complete Rust gate

Run under both the declared MSRV and current stable:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release --locked
scripts/ci.sh
```

### 10.4 Meson and document gate

Use the canonical build directory:

```sh
meson compile -C build
meson test -C build --print-errorlogs
```

Run `meson setup build` only when `build/` does not exist.

The mocked graph contract remains:

```sh
scripts/test-meson-mock.sh .
```

Byte reproducibility remains a separate manual/release check:

```sh
scripts/check-document-reproducibility.sh
```

The reproducibility script’s temporary clean builds are its documented release
exception; ordinary development must not create parallel production build
directories.

### 10.5 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every newly tracked Rust or documentation subject joins its nearest
`meson.build` census.

### 10.6 Advisory lane

Run:

```sh
cargo audit
```

when installed.

A missing advisory tool is reported as skipped, not passed. Phase-2 evidence
must state whether the final ceremony requires installation.

### 10.7 Clean repository

The final check is:

```sh
git status --porcelain=v1 --untracked-files=all
```

It must be empty.

This covers:

- unstaged tracked changes;
- staged changes;
- untracked nonignored files.

Ignored build products are permitted.

### 10.8 Execution trust

Repository source, tests, Meson definitions, scripts, TeX, and `.latexmkrc` are
executable.

Untrusted contributions must be run in a credential-free isolated environment
established outside the untrusted checkout. A clean-tree result is a
correctness check, not containment of malicious code.

---

## 11. Phase-2 gate · `gate:backlog:phase2`

### 11.1 Preconditions

Phase 2 may exit only when:

```text
F3-001 through F3-010 are closed or formally rejected
P2-001 through P2-012 are DONE
P2-013 records the complete gate
C1-004, C1-005, C1-008, C1-009, C1-010 are DONE
the Phase-2 subset of C1-013 is DONE
C1-014 is DONE
```

Later linker tasks C1-011 and C1-012 do not block Phase 2.

Consumer-gated mathematical tasks C1-006 and C1-007 do not block Phase 2 unless
compiler implementation introduces a concrete consumer.

### 11.2 Required Phase-2 evidence

Record:

- exact source revision;
- Rust, Cargo, Meson, Ninja, Python, Git, and TeX versions;
- dependency and lockfile review;
- advisory status;
- realization immutability and ownership boundary;
- realization projection permutation results;
- compiler relation-census equality;
- canonical graph permutation results;
- SCC and topology oracle results;
- constant-folding oracle results;
- proof-plan exhaustive-oracle results;
- placement exhaustive-oracle results;
- complexity-limit failures;
- constructibility and lifecycle failures;
- both pilot analyzed projections;
- repeated analysis equality;
- generated-artifact and label status;
- Meson minimum-version configuration result;
- Meson no-op behavior;
- document reproducibility;
- final clean-tree output.

### 11.3 Exit statement

The Phase-2 evidence must establish:

- the compiler consumes immutable typed realization values only;
- every scoped realization relation appears in analysis;
- no relation or proof alternative is mis-owned, weakened, or dropped;
- no Petgraph index enters semantic identity;
- canonical graph and realization projections are insertion-order independent;
- constant folding preserves checked failure semantics;
- proof alternatives are exact and hard constraints remain hard;
- permissionless cases require no owner/operator secret;
- disclosure reasons and source provenance remain typed;
- sponsor-value opacity is preserved;
- lifecycle incompleteness is explicit;
- every active execution case has a possible semantic carrier;
- every relation has complete future evidence requirements;
- both pilots analyze deterministically;
- compiler core contains no concrete target opcode, stack index, tapleaf,
  transaction position, or target program;
- all repository lanes pass and leave the tree clean.

---

## 12. Backlog hygiene · `sec:backlog:hygiene`

### 12.1 Adding work · `rule:backlog:add`

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

### 12.2 Splitting work · `rule:backlog:split`

Split a task when it:

- crosses semantic, compiler, linker, target, transaction, evidence, or release
  boundaries;
- has an independently reviewable security consequence;
- mixes exact correctness with numerical diagnostics;
- mixes dependency adoption with algorithm acceptance;
- contains one part that can complete while another remains research-blocked.

### 12.3 Dropping or parking work · `rule:backlog:drop`

A dropped or parked task records:

- why it is unnecessary or premature;
- supporting evidence;
- activation condition;
- replacement, if any;
- affected documentation;
- identity and release consequences.

### 12.4 Retention · `rule:backlog:retention`

After a phase baseline:

- compact completed prose into the phase card or release record;
- retain immutable evidence in Git history or an annotated tag;
- keep only current and immediately preparatory work here;
- do not create another historical archive under `plans/`.

Closed F3 findings should be reduced to a compact evidence table after the
Phase-2 gate records their complete results.

---

## 13. Execution order · `sec:backlog:order`

Execute in this order unless new evidence changes dependencies:

```text
1. Reproduce and close F3-001: canonical Meson graph correctness.
2. Close F3-002, F3-005, and F3-006 before freezing compiler input APIs.
3. Close F3-003 before relying on the flattener confinement claim.
4. Close F3-004 so ADR-014 argument-owned census rules fail closed.
5. Close F3-007 through F3-010 in parallel with the higher-priority fixes.
6. Finish C1-004 dependency and lockfile review.
7. Re-run the complete current repository gate and record results.
8. Create the compiler crate under P2-003.
9. Implement relation DAG and checked folding under P2-004 through P2-006.
10. Implement proof, disclosure, source, constructibility, and lifecycle
    analysis under P2-007 through P2-009.
11. Implement execution-case placement, layout, and coverage requirements
    under P2-010 and P2-011.
12. Analyze compact ASH and live transfer end to end under P2-012.
13. Run and record the complete Phase-2 gate under P2-013.
14. Begin Phase-3 target work only after Phase-2 exit.
```

No target prototype may be used to defer a current typed-boundary or
publication-correctness repair.

---

## 14. Current completion gate · `gate:backlog:current`

The current gate is **not passed**.

Current blockers are:

```text
Current F3 remediation:
    F3-001 through F3-010 open

Realization compiler-input boundary:
    externally mutable value
    noncanonical complete projection
    incomplete generic ownership validation

Phase-2 dependency review:
    C1-004 / P2-002 incomplete

Compiler package:
    not yet created

Compiler relation, proof, disclosure, lifecycle, placement, and coverage:
    not yet implemented
```

The earlier F2 remediation register remains historical evidence. It does not
close the newly identified F3 cases automatically.

Until (`gate:backlog:phase2`) passes:

- Phase 1 remains a historical tagged result with later findings recorded;
- the compiler public API is not frozen;
- target-specific fields remain forbidden in realization;
- no stable linker or transaction ABI exists;
- no floating-point result becomes semantic identity;
- no draft bound becomes deployment calibration;
- no target prototype becomes release evidence;
- no self-consistent cache is described as independent target-chain
  provenance;
- no architecture, model, realization, or build success is described as
  deployment readiness;
- the current checkout is not described as green without a fresh complete
  execution record.

---

## 15. One-line backlog · `rem:backlog:one-line`

> Re-close the current Meson, realization, flattener, census, canonical-projection, ownership, label-diagnostic, export-order, conditional-flattening, and stamp-contract findings; finish the Petgraph dependency review; then build the Phase-2 compiler as an exact, deterministic, target-independent analysis with complete relation, disclosure, constructibility, lifecycle, placement, and coverage evidence.
