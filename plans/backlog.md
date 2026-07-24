# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 2 — target-independent compiler analysis
> **Current condition:** Phase 1 has an immutable recorded completion tag, but a later static review of commit identified new correctness and assurance gaps. Those findings do not rewrite the historical tag; they are current remediation work and must close before Phase 2 exits. Compiler implementation may begin only behind the dependency and realization-boundary prerequisites named below.
> **Next gate:** Phase 3 — Elements target and foundational prototypes
> **Authority:** Current execution queue only. Normative specifications, typed architecture, implemented ADRs, accepted decisions, package contracts, research results, phase cards, and the roadmap take precedence.

This file contains current remediation, active Phase-2 implementation work, and
the immediately following research/prototype dependencies.

It deliberately does not retain the former long-form Phase-1 implementation
diary. Phase-1 history is preserved by Git, the Phase-1 card, and the annotated
evidence tag `phase1-realization-foundation-v1`.

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

Code resembling an intended result is not sufficient for `DONE`.

A gate is complete only when its required commands have run in the required
environments, every required lane passed, and the complete repository status
check is clean.

### 1.2 Priority vocabulary · `tbl:backlog:priority`

| Priority | Meaning |
|---|---|
| **P0** | A defect capable of manufacturing false release, deployment, provenance, or semantic evidence. |
| **P1** | A phase-gate correctness blocker or cross-layer semantic mismatch. |
| **P2** | Required correctness, determinism, policy, or documentation closure before the active phase exits. |
| **P3** | Maintainability or evidence-quality work that follows correctness but remains part of the active gate. |
| **POST** | Later-phase work that does not block the active phase. |

### 1.3 Task families · `tbl:backlog:families`

| Prefix | Owner |
|---|---|
| `F2` | Findings discovered after the recorded Phase-1 gate |
| `P2` | Phase-2 compiler implementation |
| `C1` | Compiler/linker algorithm and dependency preparation |
| `Q` | Research or prototype dependency |

Historical `F1` and `R1` identifiers remain immutable references to Phase-1
work. They are not reused.

### 1.4 Definition of done · `rule:backlog:done`

An implementation task is `DONE` only when it records:

1. implementing source files;
2. focused positive tests;
3. focused negative, mutation, property, or integration tests;
4. affected documentation, ADRs, decisions, or package contracts;
5. exact verification commands and results;
6. generated-artifact and label-register impact;
7. semantic, identity, schema, and versioning impact;
8. dependency and feature impact;
9. confirmation that no staged, unstaged, or untracked nonignored change was
   created by checks;
10. any intentionally retained limitation.

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
- an approved correction to the owning policy or assurance claim.

“Existing tests pass” is not closure unless a named test reaches the reported
path.

### 1.5 Authority and labels · `rule:backlog:authority`

Planning labels are non-normative and non-identity-bearing.

Under ADR-013:

- each PLAN mint is unique;
- each same-owner citation resolves;
- cross-owner citations use explicit prefixes;
- generated registers do not participate in the source graph.

Task identifiers and planning labels never become compiler, linker, ABI,
deployment, evidence, or release identity.

---

## 2. Current repository state · `sec:backlog:state`

### 2.1 Review basis

The current remediation register is based on static review of:

```text
commit:
tree reference supplied to review: HEAD
selected authored files: 331
```

The review did not execute the build or test suite. `Cargo.lock`, licenses, and
archive outputs were excluded from the supplied content.

Therefore:

- findings below are source-review findings;
- prior recorded green gates remain historical evidence;
- the current checkout is not declared green merely because the earlier tag was
  green;
- each remediation task must run its own focused and complete verification.

### 2.2 Implemented packages · `tbl:backlog:implemented`

| Package or area | Current source state |
|---|---|
| Layer 0 | Published specification, version `0.5.1` |
| Realization document | Realization, final manifest appendix |
| `architecture` | Typed architecture, schema 17, semantic and behavioural hashes, deployment-profile validation |
| `model` | Executable state machine, invariant checker, property and corruption tests, indexer and accounting projections |
| `realization` | Target-independent typed semantics for compact ASH and live receipt transfer |
| `artifacts` | Generated-publication writer/checker and realization-document weld |
| `labels` | Owner-aware Markdown/Rust label graph, plan checks, registers, census and forbidden-text audits |
| `cli-common` | Shared ADR-010 command and ADR-014 report/stamp infrastructure |
| `document-stamps` | Git-derived paper metadata and two-output rendering |
| `execwrap` | Byte-preserving process wrapper; mock TeX execution isolated in a separate binary |
| `flatten-latex-main` | Deterministic atomic LaTeX flattener |
| Meson | Explicit source census, incremental lint targets, always-fresh repository audits, mocked TeX contract |
| Security policy | Public-data tools and external execution-environment trust boundary |

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
```

Architecture finality does not imply deployment readiness.

A green model does not prove target correctness. A self-consistent event cache
does not prove target-chain provenance. A generated publication does not become
semantic source.

### 2.5 Readiness statement · `rem:backlog:readiness`

```text
Attestation specification:             published
Realization contract:                 published
Typed architecture:                   final and pinned
Executable model:                     implemented
Typed realization pilots:             implemented
Recorded Phase-1 gate:                historically green and tagged
Post-gate static findings:             open
Phase-2 compiler package:              absent
Target/backend/linker/transaction:     absent
Independent deployment evidence:      absent
Production deployment:                absent
```

---

## 3. Historical Phase-1 record · `gate:backlog:phase1`

Phase 1 has a recorded immutable evidence tag:

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

The findings in this backlog were discovered after that ceremony. They do not
alter the tagged commit or retroactively edit the tag. They refine the current
assurance claim in the same way that later Phase-0 findings refined the baseline
without rewriting its historical record.

The current source must not claim that the earlier evidence covered paths that
were not actually exercised. In particular, exact pilot-weld mutation coverage
must be corrected before the Phase-2 gate relies on that claim.

---

## 4. Current remediation register · `sec:backlog:findings`

### 4.1 Summary · `tbl:backlog:findings`

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `F2-001` | P0 | **DONE** | Bound calibrations are not bound to the final emitted script bundle. |
| `F2-002` | P1 | **DONE** | Pilot architecture welds omit operation fields that can change semantics. |
| `F2-003` | P2 | **DONE** | Ordinary or unrelated Rust comments can suppress label harvesting through cross-comment fence state. |
| `F2-004` | P2 | **DONE** | The LaTeX flattener’s off-list symlink-target confinement claim is not enforced. |
| `F2-005` | P2 | **DONE** | Multi-output commands accept aliased destinations and may succeed without producing distinct assets. |
| `F2-006` | P2 | **TODO** | Ordinary sponsor L-BTC positivity may over-constrain confidential-value minimality. |
| `F2-007` | P3 | **DONE** | Active graph-planning prose retains adapter terminology prohibited by D007. |
| `F2-008` | P3 | **DONE** | Subprocess and Phase-1 evidence comments overstate or misstate current coverage. |

---

### F2-001 — Bind every calibration to the released script bundle

**Priority:** P0
**Status:** DONE
**Owners:** `architecture`, future `release`
**Primary files:**

```text
packages/architecture/src/deployment.rs
packages/architecture/src/tests/deployment_tests.rs
```

#### Problem

Each `BoundCalibration` carries:

```rust
script_bundle_hash
```

and the profile independently carries:

```rust
artifacts.emitted_script_bundle
```

Release validation currently requires both hashes to be nonzero but does not
require equality.

The synthetic profile accepted by
`fully_populated_final_profile_validates` currently uses different values for
the calibration bundle and final emitted bundle.

The missing relation is:

\[
\forall c\in\text{calibrated bounds},\quad c.\text{script bundle hash}=\text{profile final emitted script bundle hash}
\]

#### Risk

A deployment profile can validate when calibration measurements apply to
different script bytes from the bundle being released.

A stale calibration may therefore survive:

- script changes;
- constructor changes;
- taptree changes;
- linker changes;
- backend-configuration changes;
- resource increases.

This is a false release-evidence path even though no production release
currently exists.

#### Required implementation

Add a typed error such as:

```text
BoundCalibrationBundleMismatch(BoundId)
```

For every calibrated bound require:

```text
calibration.script_bundle_hash
    ==
profile.artifacts.emitted_script_bundle
```

Do not merge this error with missing evidence. Nonzero-but-wrong evidence is an
identity mismatch, not absence.

Review whether deployment-profile schema 2 is sufficient to bind calibration to
the final transaction ABI. If the current type cannot bind ABI identity, record
that as a future schema requirement rather than implying script-bundle equality
alone proves complete transaction calibration.

#### Required tests

- valid profile with every calibration bound to the final bundle;
- one mismatched calibration;
- all calibrations bound to one stale bundle;
- final bundle changed without recalibration;
- zero calibration hash remains a missing-evidence error;
- profile hash changes when the calibration bundle binding changes;
- the previous mismatching synthetic “valid” fixture is rejected.

#### Identity and schema impact

Expected immediate impact:

```text
architecture schema:          unchanged
deployment-profile schema:    unchanged unless ABI binding is added now
architecture semantic hash:   unchanged
architecture behavioural hash: unchanged
accepted deployment profiles: stricter
```

#### Exit

- [x] every calibration names the final emitted bundle;
- [x] mismatch has a focused typed error;
- [x] the valid fixture uses matching identities;
- [x] stale-bundle mutations fail;
- [x] ABI-binding residual is stated honestly;
- [x] architecture tests and complete repository gates pass cleanly.

#### Evidence · DONE

- Commits `47295dd` (validator, fixture, core regressions) and the F2-001
  closure commit (absence/mismatch separation, ABI residual). In
  `packages/architecture/src/deployment.rs`, validate_bound_calibrations
  rejects any calibration whose nonzero bundle hash differs from
  artifacts.emitted_script_bundle with the dedicated error
  BoundCalibrationBundleMismatch(BoundId); a zero hash still reports
  MissingBoundEvidence, so absence and identity mismatch never merge.
- The synthetic release fixture binds every calibration and the artifact
  to one RELEASED_BUNDLE constant; the former mismatching fixture shape is
  now itself a regression (all-calibrations-stale case).
- Regressions: one mismatched calibration, every calibration stale, final
  bundle changed without recalibration, matching bindings produce no
  mismatch error, zero bundle hash stays missing-evidence, and the
  profile hash moves when a calibration's bundle binding changes.
- ABI residual recorded on the BoundCalibration type: schema 2 cannot
  bind the transaction ABI/configuration of the measurement, so bundle
  equality alone does not prove the measured shape used the final ABI; a
  future profile schema must add that binding before production release.
  Deployment-profile schema is unchanged; accepted profiles are stricter.
- Verified: fmt, clippy -D warnings, architecture suite green under the
  nightly SDK toolchain; full workspace gates run at the end of the F2
  remediation series.

---

### F2-002 — Make pilot architecture welds complete

**Priority:** P1
**Status:** DONE
**Owners:** `realization`, `architecture`
**Blocks:** Phase-2 compiler semantic API
**Primary files:**

```text
packages/realization/src/validate.rs
packages/realization/src/tests/
packages/architecture/src/spec.rs
packages/architecture/src/validate.rs
```

#### Problem

The compact-ASH and live-transfer validators compare many architecture fields
but omit at least:

```text
operation kind
quantity reads
quantity writes
issuance set as an explicit field
```

The most direct missing field is `OperationSpec.kind`.

A mutation such as:

```text
compact-ash:
    covenant-branch → client-protocol
```

can remain structurally valid architecture while the pilot realization accepts
the old semantic declaration.

A quantity read or write can likewise be added together with the reciprocal
quantity declaration while the pilot realization keeps its former dependency
and disclosure graph.

#### Required design

Create one complete normalized pilot operation projection or otherwise check
every realization-relevant `OperationSpec` field exactly.

The weld must cover:

- operation ID;
- operation kind;
- primary authorization;
- root policy;
- issuance set;
- input families, cardinalities, and input authorization;
- output families and cardinalities;
- canonical deltas;
- data-output families;
- open-flow set;
- read set;
- write set;
- witness set;
- value-flow set;
- bound set;
- projection policy.

Absence is part of the contract. An empty read, write, issuance, or data-output
set must be checked explicitly where expected.

#### Required mutation harness

Add per-field mutations for both pilots, including:

- `CovenantBranch → ClientProtocol`;
- a valid extra quantity read plus reciprocal quantity-reader declaration;
- a valid extra quantity write plus reciprocal quantity-writer declaration;
- changed primary authorization;
- changed input authorization;
- changed minimum or maximum;
- added or removed family;
- changed bound;
- added or removed issuance;
- changed delta family, condition, or tag;
- added data output;
- added or removed open flow;
- added or removed value-flow class;
- added or removed witness;
- changed root use;
- changed projection rule.

The harness must demonstrate that architecture draft validation can succeed
while the realization weld rejects the semantic drift where that distinction is
the purpose of the test.

#### Completion-evidence correction

The former Phase-1 F1-003 entry required per-field mutation coverage but marked
the task complete while explicitly deferring that harness.

Closure of F2-002 must update the active planning record so the evidence claim
states exactly what was tested.

The immutable Phase-1 tag is not changed.

#### Identity impact

A test-only or validation-only repair should not move:

```text
architecture schema
architecture semantic hash
architecture behavioural hash
Layer-0 anchor set
Realization version
```

A change to the actual architecture declaration remains subject to the
behavioural-version gate.

#### Exit

- [x] all realization-relevant operation fields are welded;
- [x] every field family has at least one focused mutation;
- [x] the actual published architecture still derives both pilots;
- [x] pilot behavior remains unchanged;
- [x] Phase-1 completion prose no longer overclaims deferred coverage;
- [x] realization, model-conformance, architecture, and complete workspace gates pass cleanly.

#### Evidence · DONE

- Commits `bc43b74` (field welds, initial mutations) and the F2-002
  closure commit (full per-field harness). Both pilot welds in
  `packages/realization/src/validate.rs` now also check operation kind
  (CovenantBranch), the empty issuance set, the empty quantity read set,
  and the empty quantity write set, with dedicated mismatch classes
  (OperationKind, Issuances, Reads, Writes). With the existing checks
  this covers every realization-relevant OperationSpec field, absence
  included; the operation ID is fixed by the lookup itself.
- Mutation harness: each pilot test module rewrites the published
  operation row (vec-and-leak) and runs a table of one focused mutation
  per field family — primary authorization, input authorization, input
  minimum, output maximum, sponsor input/output cardinality, added and
  removed input family, added output family, changed bound set, delta
  kind/condition/destruction-tag/extra-delta, added data output, removed
  open flow, added or removed value-flow class, removed witness, changed
  root use, changed projection rule — plus the kind reclassification,
  issuance, read, and write cases and both weld-accept tests.
- The draft-versus-weld distinction is demonstrated end to end twice: a
  reciprocal quantity read (compact ASH) and a reciprocal quantity write
  (live transfer) each pass validate_draft and are rejected only by the
  weld.
- Architecture declarations are untouched: schema, semantic hash,
  behavioural hash, and pilot derivations are unchanged; the published
  architecture still welds and derives both pilots.
- The Phase-1 overclaim correction is carried by this register (F2-002
  reopened the deferred F1-003 harness and closes it here); the immutable
  Phase-1 tag is unchanged.
- Verified: fmt, clippy -D warnings, realization suite green under the
  nightly SDK toolchain; full workspace gates run at the end of the F2
  remediation series.

---

### F2-003 — Scope Rustdoc fences to actual documentation blocks

**Priority:** P2
**Status:** DONE
**Owner:** `labels`
**Primary files:**

```text
packages/labels/src/rust_source.rs
packages/labels/src/tests.rs
```

#### Problem

The Rust label scanner extracts all comments into one sequence without
retaining:

- ordinary versus documentation comment kind;
- line versus block documentation;
- contiguous documentation-block identity.

One fence state then spans every comment segment in the file.

An ordinary comment can therefore hide a real label:

```rust
// ```text
fn unrelated() {}
// ´def:area:hidden´
// ```
```

A fence opened in one Rustdoc block can also suppress labels in another block
after intervening code.

ADR-013 excludes fenced Rustdoc examples, not arbitrary comments and unrelated
comment blocks.

#### Required implementation

Retain typed comment provenance, for example:

```text
ordinary line comment
outer line documentation
inner line documentation
ordinary block comment
outer block documentation
inner block documentation
contiguous block identity
```

Fence handling must:

- apply only to documentation comments;
- remain inside one contiguous Rustdoc block;
- stop or diagnose at the end of that block;
- never allow an ordinary comment to open a Rustdoc fence;
- never allow one item’s fence to suppress another item’s documentation.

Acute-label parsing in ordinary comments remains active.

#### Required tests

- plain comment containing a fence marker around a label;
- valid fenced outer Rustdoc example;
- valid fenced inner Rustdoc example;
- fenced block documentation;
- an open fence followed by ordinary code;
- an open fence followed by another documentation block;
- ordinary comment between fenced documentation lines;
- labels before and after a fenced example;
- unclosed-fence diagnostic located at the opening documentation block.

#### Impact

```text
protocol semantics:           none
architecture identities:      none
label publication bytes:      may change if previously hidden labels exist
model_labels.json:            may change
planning registers:           unchanged unless source labels change
```

#### Exit

- [x] ordinary comments cannot suppress labels;
- [x] Rustdoc fences remain nonparticipating;
- [x] fence state cannot cross documentation blocks;
- [x] diagnostics retain correct line and column;
- [x] generated label publications are regenerated if needed;
- [x] labels, generated-artifact, census, and complete gates pass cleanly.

#### Evidence · DONE

- F2-003 closure commit. `packages/labels/src/rust_source.rs` now
  retains typed comment provenance: a CommentKind (ordinary/outer-doc/
  inner-doc, line and block forms, with `////` and `/***` classified
  ordinary exactly as rustdoc does) and a contiguous block identity
  (consecutive same-kind line comments with no intervening code share a
  block; every block comment is its own block).
- Fence handling opens only in documentation comments, lives inside one
  contiguous documentation block, and a block ending with its fence
  open is diagnosed (UnclosedMarkdownFence at the opening line) instead
  of silently swallowing the rest of the file. Acute-label parsing in
  ordinary comments is unchanged.
- Regressions: plain-comment fence markers around a label still mint;
  outer-doc, inner-doc (existing), and block-doc fenced examples stay
  nonparticipating; an open fence followed by code or by another
  documentation block suppresses nothing outside its block and is
  diagnosed at line 1 of the fence; an ordinary comment between fenced
  doc lines participates; labels before and after a fenced example
  mint.
- Repository impact: the full check-labels census run over the current
  tree is green with no register or model_labels.json change, so no
  previously hidden participating occurrence existed.
- Verified: fmt, clippy -D warnings, labels suite (71 tests), and the
  check-labels lane (ci.sh lane-6 argv) under the nightly SDK
  toolchain.

---

### F2-004 — Enforce or narrow the flattener symlink contract

**Priority:** P2
**Status:** DONE
**Owner:** `flatten-latex-main`
**Primary files:**

```text
packages/flatten-latex-main/src/lib.rs
packages/flatten-latex-main/src/tests/mod.rs
```

#### Problem

The flattener says that a symlink whose target is outside the allowed-file list
cannot be read.

The implementation resolves a reference to an allowed path and then opens that
path with `File::open`, which follows symlinks.

An allowlisted symlink can therefore point to a target not present in the
allowlist and publish that target’s bytes.

The existing test covers only an unlisted symlink, which fails before the open.

#### Required decision

Choose one explicit contract.

**Preferred: strict regular-file confinement**

- reject symlink `main_file` and allowed files;
- accept only regular files;
- validate before opening;
- document the remaining filesystem race honestly;
- use filesystem identity for include-cycle detection where necessary.

**Alternative: path authorization**

- state that allowlisting a path authorizes its filesystem-resolved target;
- remove the claim that off-list symlink targets are unreachable;
- document the trust consequence.

The preferred contract matches the repository’s current publication-security
posture and the document-stamp regular-blob rule.

#### Required tests

- allowlisted symlink to a file outside the fixture tree;
- symlinked main entrypoint;
- symlink cycle;
- two aliases of one included file;
- ordinary regular allowlisted file;
- unlisted symlink remains rejected;
- failed confinement preserves an existing output and leaves no staged file.

#### Impact

The canonical paper build already receives tracked regular inputs through the
document-stamp boundary, so a strict rejection should not change canonical
paper output.

#### Exit

- [x] public documentation states the implemented rule exactly;
- [x] allowlisted off-tree symlink behavior is tested;
- [x] include-cycle identity matches the selected rule;
- [x] failed flatten remains atomic;
- [x] flattener and complete workspace gates pass cleanly.

#### Evidence · DONE

- F2-004 closure commit; the preferred strict regular-file confinement
  is implemented. `flatten` validates `main_file` and every supplied
  file with `symlink_metadata` before anything is read or staged: a
  symlink (allowlisted, dangling, or looping) is rejected by its own
  file type, and only existing regular files are accepted. The library
  documentation now states this rule exactly and documents the
  check-then-open residual honestly: a filesystem racing the flattener
  between validation and open is outside the ADR-015 boundary
  (configuration-mistake defense, not a malicious-filesystem sandbox).
- Include-cycle identity is filesystem identity where available
  (device/inode of the validated regular file, unix) with
  component-path equality as fallback, so two allowlist hard-link
  aliases of one file close a cycle instead of recursing.
- Regressions: allowlisted symlink to an off-tree target refused with
  nothing published; symlinked main entry point refused; a symlink loop
  refused without being chased; a hard-link alias include cycle
  detected; confinement failure preserves an existing output
  byte-for-byte and leaves no staging file (validation precedes
  staging); the unlisted-symlink, absolute-path, and traversal
  rejections are unchanged.
- Canonical paper output is unaffected: the document-stamp boundary
  already supplies tracked regular files, and the meson wiring passes
  existing regular files only.
- Verified: fmt, clippy -D warnings, flattener unit and subprocess
  suites (34 tests) under the nightly SDK toolchain.

---

### F2-005 — Reject aliased multi-output destinations

**Priority:** P2
**Status:** DONE
**Owners:** `document-stamps`, `cli-common`, `labels`, shared filesystem helper if introduced
**Primary files:**

```text
packages/document-stamps/src/bin/attestation-stamps.rs
packages/document-stamps/src/lib.rs
packages/cli-common/src/lib.rs
packages/labels/src/bin/generate-label-registers.rs
```

#### Problem

Multi-output commands validate argument completeness but not output-role
uniqueness.

For document stamps, the same path may be supplied for:

```text
stamps output
epoch output
```

Both files stage successfully. The epoch is published first and the stamps file
then overwrites it. The command exits success even though no distinct epoch
asset remains.

Related role-alias cases exist for:

- checker report and success stamp;
- Layer-0 and realization register outputs.

#### Required design

Validate that output roles name distinct destinations before semantic work or
publication.

At minimum reject exact path equality.

Prefer a reusable destination-identity helper capable of recognizing:

- lexical aliases;
- existing hard-link aliases;
- symlinked parent-directory aliases;
- pending destinations under an existing aliased ancestor.

This helper must remain a correctness guard, not a claimed malicious-filesystem
sandbox.

#### Required tests

- identical output paths;
- lexical aliases;
- existing hard-link aliases;
- parent-directory symlink aliases;
- two distinct valid destinations;
- failed validation produces no output and no stamp;
- direct checker mode remains unaffected;
- build checker mode rejects report/stamp aliasing.

#### Impact

```text
canonical Meson wiring:       unchanged
CLI accepted argument set:    stricter
artifact bytes:               unchanged
schemas and hashes:           unchanged
```

#### Exit

- [x] every multi-output command validates role uniqueness;
- [x] alias failures occur before publication;
- [x] no command exits success with one role overwriting another;
- [x] focused subprocess and filesystem tests pass;
- [x] complete CLI and Meson gates pass cleanly.

#### Evidence · DONE

- F2-005 closure commit. The reusable destination-identity helper lives
  in `cli-common` (not `execwrap`): destination_identity canonicalizes
  the deepest existing ancestor (so lexical dot spellings and symlinked
  parent directories agree), folds the necessarily nonexistent pending
  components lexically, and additionally carries the device/inode of an
  existing destination file so hard-link aliases are recognized;
  ensure_distinct_outputs rejects the first aliased role pair. The
  helper documents itself as a correctness guard, not a
  malicious-filesystem sandbox.
- Enforced before any semantic work or mutation at all three
  multi-output boundaries: document-stamps render_outputs
  (stamps/epoch), cli-common finish_check_command build mode
  (report/stamp, with a dedicated CheckResultError variant so a stamp
  can never hold report bytes), and labels generate_registers
  (Layer-0/realization registers, with a GenerateError variant).
- Regressions: identical paths, lexical dot-dot aliases, existing
  hard-link aliases, symlinked-parent aliases, and distinct-paths
  acceptance for the helper; aliased report/stamp fails before
  publication with nothing written; aliased render outputs preserve
  prior destination bytes; aliased register outputs write nothing.
  Direct checker mode takes the (None, None) arm and is untouched.
- Canonical Meson wiring already supplies distinct paths, so artifact
  bytes, schemas, and hashes are unchanged; the accepted argument set
  is strictly narrower.
- Verified: fmt, clippy -D warnings, cli-common (50), document-stamps
  (44), and labels (72) suites under the nightly SDK toolchain.

---

### F2-006 — Decide sponsor L-BTC positivity versus representation minimality

**Priority:** P2
**Status:** TODO
**Owners:** `realization`, future `compiler`, D005 representation policy
**Primary files:**

```text
packages/realization/src/evaluate.rs
packages/realization/src/tests/compact_ash_tests.rs
packages/realization/src/tests/live_transfer_tests.rs
plans/decisions/005-value-representation.md
plans/research/public-declassification.md
```

#### Current state

Realization recognizes ordinary `PLAIN_LBTC` only when:

```text
value > 0
owner present
```

This was introduced to agree with the model’s positive-value open-flow
partition and to preserve `CPFP_ANCHOR` as the sole zero-value L-BTC family.

#### Open question

For sponsor L-BTC, the semantic requirement may be only the role-local balance:

\[
\sum \text{sponsor sources}=\sum \text{sponsor destinations}+\text{fee}
\]

A backend may be able to discharge that relation by confidential-transaction
conservation without opening every individual sponsor amount.

A blanket positivity check may therefore:

- require disclosure not needed by the operation;
- prevent a private sponsor-value proof alternative;
- conflict with D005’s minimality analysis;
- mistake a reference-model normalization for a target safety requirement.

No current backend exists, so this must be decided before target proof planning
freezes the relation.

#### Required research result

Choose one:

**Keep positivity as a semantic relation**

- identify the concrete safety or closure failure prevented;
- state how a confidential sponsor amount proves positivity;
- preserve permissionless and owner-authorized constructibility;
- add target requirements and evidence obligations.

**Narrow positivity to protocol-accounted values**

- keep positivity for receipt, entitlement, vault, ASH, reserve, payout,
  issuance, and other arithmetic-bearing values;
- allow sponsor pass-through value to be discharged through exact sponsor-flow
  conservation;
- preserve the sole semantic zero-valued protocol object rule separately.

**Treat zero sponsor value as inert and nonparticipating**

- define canonical normalization and ABI behavior;
- prove no extra zero-value ordinary output can impersonate `CPFP_ANCHOR`.

#### Required tests

- explicit positive sponsor input/change;
- confidential-value sponsor plan;
- zero-valued sponsor input;
- zero-valued sponsor output;
- zero-valued ownerless ordinary L-BTC;
- true `CPFP_ANCHOR`;
- sponsor-flow exact balance;
- sponsor/protocol overlap;
- disclosure comparison between supported plans.

#### Exit

- [ ] D005 and realization state one rule;
- [ ] compiler proof alternatives can represent the selected rule;
- [ ] safety and minimality claims remain separate;
- [ ] model/realization difference is intentional and documented if retained;
- [ ] focused positive and negative tests pass;
- [ ] no unsupported target capability is assumed.

---

### F2-007 — Remove prohibited graph-adapter terminology

**Priority:** P3
**Status:** DONE
**Owners:** compiler/linker research and package plans
**Primary files:**

```text
plans/research/compiler-algorithms.md
plans/research/linker-algorithms.md
plans/packages/compiler.md
plans/packages/linker.md
```

#### Problem

D007 requires:

```text
package-owned direct Petgraph graph
+ typed stable keys
+ key/index lookup metadata
+ canonical typed projections
```

and prohibits first-party graph wrappers or adapters.

Active research prose still includes phrases such as “typed graph adapters” or
“private adapter,” which can be read as reintroducing the prohibited
abstraction.

#### Required correction

Use consistent language:

```text
direct package-owned Petgraph graph
typed node and edge weights
stable-key ↔ local-index lookup metadata
package-local helper functions
canonical stable-key projection
```

A lookup table or helper function is not a graph abstraction layer.

#### Exit

- [x] no active plan requests a graph wrapper or adapter;
- [x] D007 terminology is consistent across compiler and linker notes;
- [x] no shared graph crate is proposed;
- [x] plan and label checks pass.

#### Evidence · DONE

- F2-007 closure commit. In compiler-algorithms.md the expected handoff
  now names package-owned direct Petgraph graphs with typed key/index
  lookup metadata (formerly typed graph adapters), and the key/index
  retention paragraph names the package-local lookup metadata (formerly
  the private adapter). The linker notes and package plans contain no
  adapter or wrapper requests; the remaining adapter mention in
  compiler.md describes the narrow target-matching capability boundary,
  not a graph abstraction, and D007's own prohibition text is untouched.
  No shared graph crate is proposed anywhere. check-plans and
  check-labels pass.

---

### F2-008 — Reconcile evidence comments with actual coverage

**Priority:** P3
**Status:** DONE
**Owners:** CLI subprocess tests, Phase-1 planning record
**Primary files:**

```text
packages/artifacts/tests/subprocess_contract.rs
packages/flatten-latex-main/tests/subprocess_contract.rs
plans/phases/01-realization.md
plans/backlog.md
```

#### Problem

Some subprocess-test headers still say TTY refusal is incomplete under F1-024,
while the repository’s current evidence model says representative PTY tests
exercise each shared refusal path.

The Phase-1 completion record also needs to distinguish:

- requirements actually tested;
- requirements covered only indirectly;
- mutation coverage deferred and now reopened under F2-002.

#### Required correction

- update stale subprocess-test module comments;
- state that TTY refusal is tested by shared code path and output mode, not
  redundantly for every binary;
- do not claim per-binary success, runtime-failure, panic, or TTY coverage where
  only shared-path representative coverage exists;
- link the reopened exact-weld mutation work to F2-002;
- leave the immutable evidence tag unchanged.

#### Exit

- [x] comments match actual tests;
- [x] no obsolete “TTY lane incomplete” statement remains;
- [x] Phase-1 prose distinguishes historical pass from later findings;
- [x] documentation, labels, and complete gates pass.

#### Evidence · DONE

- F2-008 closure commit. The artifacts and flattener subprocess-test
  headers no longer claim the TTY lane is incomplete under F1-024; both
  now state the actual model: TTY refusal is implemented by the shared
  cli-common output path and exercised representatively by the PTY
  harnesses in the labels and document-stamps subprocess suites — one
  test per distinct shared refusal path and output mode, never one
  redundant test per binary — with success-path coverage owned by the
  unit suites and the Meson-driven build.
- The Phase-1 card's evidence section now carries a later-review status
  note distinguishing the historical gate pass from the subsequently
  found weld-field and deferred-mutation gaps, linking the reopened
  work to F2-002 and naming the register's evidence records as
  authoritative where the card and register disagree. The immutable
  Phase-1 tag is unchanged.

---

## 5. Phase-2 implementation register · `sec:backlog:phase2`

### 5.1 Summary · `tbl:backlog:phase2`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `P2-001` | P1 | **DONE** | Close realization-boundary prerequisite F2-002 |
| `P2-002` | P2 | **IN PROGRESS** | Complete dependency review C1-004 |
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

`P2-003` through `P2-012` are blocked until F2-002 and the dependency review
establish a safe compiler input boundary. Design work and isolated prototypes
may proceed without freezing public APIs.

---

### P2-001 — Close the realization input boundary

**Priority:** P1
**Status:** DONE with F2-002

The compiler must not begin from a pilot realization whose architecture weld
accepts unrepresented operation fields.

Exit is exactly F2-002’s exit.

---

### P2-002 — Complete concrete dependency review

**Priority:** P2
**Status:** IN PROGRESS
**Maps to:** C1-004

Current adopted graph dependency:

```text
petgraph = 0.8.3
features:
    serde-1
    rayon
    dot_parser
    unstable
    generate
```

Required review:

- exact crates.io release;
- upstream source and release tag;
- Rust 1.88 compatibility;
- license;
- default and selected features;
- transitive graph;
- duplicate versions;
- dependency-internal unsafe code;
- Rayon and parallel-determinism implications;
- serialization non-authority boundary;
- unstable-feature usage boundary;
- advisory status;
- replacement boundary.

Required commands include:

```sh
cargo tree --locked -p petgraph -e features
cargo tree --locked -i petgraph
cargo metadata --locked
cargo audit
```

If `cargo-audit` is unavailable, the active development gate reports the lane
as skipped. Phase-2 completion policy must decide whether the tool is mandatory
for the final ceremony.

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

Add a direct `architecture` dependency only if the public API directly names
architecture-owned types and transitive use would obscure ownership.

The crate must:

- inherit workspace package metadata and lints;
- deny first-party unsafe code;
- join the Cargo workspace;
- join the Meson source census;
- expose a public API integration test;
- consume no generated publication or filesystem state.

---

### P2-004 — Bind architecture, realization, policy, and scope

**Priority:** P1
**Status:** BLOCKED

Define one analyzed input boundary containing:

- architecture binding inherited from realization;
- explicit realization scope;
- compiler-analysis policy;
- optional abstract target capabilities;
- no concrete target package or target bytecode.

Reject:

- incomplete or unsupported scope;
- identity mismatch;
- relation outside scope;
- duplicate scope;
- architecture/realization mismatch;
- generated-file input.

No public compiler identity is minted until the canonical analyzed projection is
reviewed.

---

### P2-005 — Build the canonical relation DAG

**Priority:** P1
**Status:** BLOCKED
**Depends on:** C1-005, C1-010

Use a package-owned concrete Petgraph graph directly.

Required structure:

```text
typed stable relation/analysis keys
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
- reorder named guards when failure identity matters;
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
2. reject unavailable target capabilities;
3. reject unauthenticated sources;
4. reject unavailable witnesses;
5. reject permissionless owner/operator secrets;
6. reject representation-policy failures;
7. reject lifecycle failures;
8. reject disclosure-policy failures;
9. retain the feasible set or Pareto frontier;
10. select canonically only under explicit policy.

Initial proof planning uses exact deterministic enumeration or branch-and-bound.

Complexity exhaustion returns a typed error. It does not choose the best partial
candidate or a hidden greedy fallback.

---

### P2-008 — Derive disclosure, sources, and constructibility

**Priority:** P1
**Status:** BLOCKED
**Depends on:** F2-006

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

Pilot analysis may state that later target exits are not yet implemented, but
must not describe a representation as release-complete while a required exit is
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

For each active relation require coverage of every required execution case.

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

Phase 2 exits only through (`gate:phase2:exit`) after every F2 finding and every
P2 task required by this register is closed.

No Phase-2 compiler output is deployment evidence.

---

## 6. Algorithm and preparation register · `sec:backlog:c1`

### 6.1 Current status · `tbl:backlog:c1`

| ID | Status | Current ownership |
|---|---|---|
| `C1-001` | **DONE** | Compiler/linker/mathematics/solver research notes exist |
| `C1-002` | **DONE** | D007 direct Petgraph decision accepted |
| `C1-003` | **DONE** | D008 exact/certified mathematics decision accepted |
| `C1-004` | **IN PROGRESS** | Concrete Petgraph dependency review |
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

The previous umbrella requirement that every C1 task through relocation and
taptree complete before compiler analysis was overbroad.

Current assignment is:

```text
Phase 2:
    C1-004, C1-005, C1-008, C1-009, C1-010, relevant C1-013, C1-014

Later linker phases:
    C1-011, C1-012

Concrete-consumer gated:
    C1-006, C1-007
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
- lifecycle-safe plan differs from immediate cheapest plan;
- equal-cost plans require stable-key tie-breaking;
- no feasible plan;
- complexity budget exhausted.

### C1-009 — Execution-case-aware placement

For each relation \(r\), enforce:

\[
\operatorname{requiredCases}(r)\subseteq\bigcup_{c\text{ carries }r}\operatorname{executedCases}(c)
\]

Compare exact placement search with exhaustive carrier-subset enumeration.

A carrier present somewhere but absent in one active case does not satisfy the
relation.

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
| architecture deployment validation | `cargo test --locked -p tripod-architecture` |
| realization welds and sponsor policy | `cargo test --locked -p tripod-realization` |
| model/realization conformance | `cargo test --locked -p tripod-model realization_conformance` |
| labels and Rust scanning | `cargo test --locked -p tripod-labels` |
| document outputs | `cargo test --locked -p tripod-document-stamps` |
| CLI report/stamp behavior | `cargo test --locked -p cli-common` |
| flattener confinement | `cargo test --locked -p flatten-latex-main` |
| complete workspace | `cargo test --workspace --locked` |

Focused filters supplement but do not replace full package and workspace runs.

### 10.2 Phase-2 package commands

Once the compiler crate exists:

```sh
cargo test --locked -p tripod-compiler
cargo clippy --locked -p tripod-compiler --all-targets -- -D warnings
```

The package must also participate in the workspace-wide debug and release
lanes.

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
scripts/check-document-reproducibility.sh
```

Run `meson setup build` only when `build/` does not exist.

The mocked contract remains:

```sh
scripts/test-meson-mock.sh .
```

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

A missing advisory tool is reported as skipped, not passed. The Phase-2 evidence
record must state whether the final gate requires installation.

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

---

## 11. Phase-2 gate · `gate:backlog:phase2`

### 11.1 Preconditions

Phase 2 may exit only when:

```text
F2-001 through F2-008 are closed
P2-001 through P2-012 are DONE
P2-013 records the complete gate
C1-004, C1-005, C1-008, C1-009, C1-010 are DONE
the Phase-2 subset of C1-013 is DONE
C1-014 is DONE
```

The later linker tasks C1-011 and C1-012 do not block Phase 2.

The consumer-gated mathematical tasks C1-006 and C1-007 do not block Phase 2
unless compiler implementation introduces a concrete consumer.

### 11.2 Required Phase-2 evidence

Record:

- Rust, Cargo, Meson, Ninja, Python, Git, and TeX versions;
- dependency review and lockfile state;
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
- Meson no-op behavior;
- document reproducibility;
- advisory status;
- final clean-tree output.

### 11.3 Exit statement

The Phase-2 evidence must establish:

- the compiler consumes typed realization values only;
- every scoped realization relation appears in analysis;
- no relation is weakened or dropped;
- no Petgraph index enters semantic identity;
- canonical graph results are insertion-order independent;
- constant folding preserves checked failure semantics;
- proof alternatives are exact and hard constraints remain hard;
- permissionless cases require no owner/operator secret;
- disclosure reasons and source provenance remain typed;
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

---

## 13. Execution order · `sec:backlog:order`

Execute in this order unless new evidence changes dependencies:

```text
1. Close F2-001 immediately: false deployment-evidence acceptance.
2. Close F2-002 before freezing any compiler semantic API.
3. Run C1-004 dependency review in parallel.
4. Resolve F2-006 before proof and disclosure planning freeze.
5. Close repository/tooling findings F2-003 through F2-005.
6. Correct planning and evidence wording under F2-007 and F2-008.
7. Create the compiler crate (P2-003).
8. Implement relation DAG and checked folding (P2-004 through P2-006).
9. Implement proof, disclosure, source, constructibility, and lifecycle analysis
   (P2-007 through P2-009).
10. Implement execution-case placement, layout, and coverage requirements
    (P2-010 and P2-011).
11. Analyze compact ASH and live transfer end to end (P2-012).
12. Run and record the complete Phase-2 gate (P2-013).
13. Begin Phase-3 target work only after the Phase-2 exit.
```

No target prototype may be used to defer a current typed-boundary correction.

---

## 14. Current completion gate · `gate:backlog:current`

The current gate is **not passed**.

Current blockers are:

```text
P0 deployment evidence binding:
    F2-001

P1 realization/compiler boundary:
    F2-002

Phase-2 dependency review:
    C1-004 / P2-002

Compiler package:
    not yet created

Compiler relation, proof, disclosure, lifecycle, placement, and coverage:
    not yet implemented
```

Phase 2 is complete only when (`gate:backlog:phase2`) passes.

Until then:

- Phase 1 remains a historical tagged result with later findings recorded;
- the compiler public API is not frozen;
- target-specific fields remain forbidden in realization;
- no stable linker or transaction ABI exists;
- no floating-point result becomes semantic identity;
- no draft bound becomes deployment calibration;
- no target prototype becomes release evidence;
- no self-consistent cache is described as independent target-chain provenance;
- no architecture/model/realization success is described as deployment
  readiness.

---

## 15. One-line backlog · `rem:backlog:one-line`

> Bind calibration to the exact final bundle; complete the pilot architecture weld; repair source-label, flattener, and multi-output boundaries; settle sponsor-value semantics; finish the Petgraph dependency review; then build the Phase-2 compiler as an exact, deterministic, target-independent analysis with complete relation, disclosure, constructibility, lifecycle, placement, and coverage evidence.
