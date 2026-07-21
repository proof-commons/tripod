# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 1 — typed realization foundation
> **Current condition:** Phase-1 implementation is substantially present; audit remediation and complete gate evidence remain open
> **Parallel preparation lane:** compiler/linker algorithm and dependency foundations
> **Next gate:** Phase 2 — target-independent compiler analysis
> **Authority:** Current execution queue only; normative source, implemented ADRs, accepted planning decisions, package contracts, research results, and the roadmap take precedence

This backlog contains current work and immediately preparatory work.

It does not contain:

- protocol or realization semantics;
- a second copy of package contracts;
- the complete long-term roadmap;
- generated identities copied from authoritative artifacts;
- historical implementation diaries;
- unresolved research essays duplicated from research notes;
- target implementation revisions treated as protocol identity;
- evidence claims unsupported by recorded command output.

Long-term sequencing is owned by [`roadmap.md`](roadmap.md). Cross-package choices
are owned by [`decisions/`](decisions/README.md), package boundaries by
[`packages/`](packages/README.md), phase gates by
[`phases/`](phases/README.md), and unresolved prototypes by
[`research/`](research/README.md).

---

## 1. Using this backlog · `sec:backlog:usage`

### 1.1 Status vocabulary · `tbl:backlog:status`

| Status | Meaning |
|---|---|
| **TODO** | Ready once named dependencies are complete. |
| **IN PROGRESS** | Actively being implemented, reviewed, or verified. |
| **BLOCKED** | A named dependency or decision prevents safe progress. |
| **DONE** | Implementation, tests, documentation, and required evidence are complete. |
| **DROPPED** | Deliberately not implemented; rationale and replacement are recorded. |
| **SUPERSEDED** | Replaced by a named task, decision, ADR, or package contract. |

Source code resembling an intended result is not enough for `DONE`.

A gate task is complete only when its required commands have run successfully
in the required environments and the resulting repository state is clean.

### 1.2 Task prefixes · `tbl:backlog:prefixes`

| Prefix | Owner |
|---|---|
| `P0` | Completed planning reset |
| `B0` | Historical baseline hardening |
| `F1` | Current audit/remediation findings required before Phase-1 exit |
| `C1` | Compiler/linker algorithm and dependency preparation |
| `R1` | Typed realization foundation |
| `Q` | Prototype or research dependency |

`C1` is a preparation lane. It does not authorize production compiler work
before (`gate:phase1:exit`).

### 1.3 Definition of done · `rule:backlog:done`

Every completed implementation task records:

1. implementing source files;
2. positive tests;
3. focused negative, mutation, property, or integration tests;
4. applicable documentation, package-contract, decision, or ADR updates;
5. exact verification commands;
6. confirmation that checks modified no tracked or untracked nonignored file;
7. semantic, generated-artifact, and identity impact;
8. dependency and feature impact;
9. any residual limitation deliberately retained.

Every completed research task records:

1. the question answered;
2. exact dependency, tool, and target versions used;
3. positive and negative prototype evidence;
4. complexity and resource measurements;
5. accepted and rejected candidates;
6. result and decision handoff;
7. permanent production tests;
8. assurance class and remaining trust boundary.

A task may be marked `DONE` only when its detailed status, summary table, and
exit checklist agree.

### 1.4 Planning-label rule · `rule:backlog:labels`

Planning labels are non-normative and non-identity-bearing.

Under ADR-013:

- every PLAN mint is unique;
- every same-owner citation resolves;
- cross-owner citations use explicit owner prefixes;
- generated registers do not participate in the source graph.

Planning labels never become compiler, linker, ABI, deployment, evidence, or
release identity.

---

# 2. Current execution summary · `sec:backlog:current`

## 2.1 Completed foundation · `tbl:backlog:completed`

| Gate | Status | Result |
|---|---|---|
| P0 — planning reset | **DONE** | Focused decisions, package contracts, phase cards, research notes, registers, and current backlog |
| B0 — historical baseline | **DONE** | Reproducible compiler-era baseline recorded under an immutable tag |
| ADR-010 | Implemented, with current script-boundary review open | JSON command streams, exit classes, panic and TTY policy |
| ADR-011 | Implemented | Toolchain, locking, dependency, unsafe-code, target, and reproducibility policy |
| ADR-013 | Implemented | Owner-aware Markdown/Rust documentation graph |
| ADR-014 | Implemented, with current stamp/restat findings open | Build-owned census, explicit arguments, and stamp dependency graph |
| D007 | Accepted and implemented in current graph-owning crates | Direct Petgraph graph substrate with typed stable keys |
| D008 | Accepted | Exact semantic claims and certified numerical-analysis policy |

## 2.2 Current implementation · `tbl:backlog:packages`

| Package | Current role |
|---|---|
| `architecture` | Typed normative architecture, canonical hashes, and deployment-profile validation |
| `model` | Executable reference behavior, invariant checking, property tests, corruption tests, and indexer/audit projections |
| `realization` | Target-independent typed semantics for compact ASH and live receipt transfer |
| `artifacts` | Generated-publication writer, checker, and realization-document weld |
| `labels` | Repository-wide documentation/source label graph and generated registers |
| `cli-common` | Shared ADR-010 command infrastructure and ADR-014 stamp support |
| `document-stamps` | Deterministic paper metadata derived from Git state |
| `execwrap` | Byte-preserving child-process wrapper and mocked TeX children |
| `flatten-latex-main` | Deterministic atomic LaTeX flattener |

Not yet implemented:

```text
compiler
target-elements
tapscript
linker
transaction
vectors
release
```

## 2.3 Readiness capsule · `rem:backlog:readiness`

```text
Attestation specification:        published
Realization contract:         published and architecture-welded
Typed architecture release:       final and pinned
Executable reference model:       implemented
Typed realization package:        implemented for two Phase-1 pilots
Phase-1 implementation tasks:     substantially complete
Phase-1 audit remediation:        open
Phase-1 full gate evidence:        not yet recorded
Compiler analysis:                not implemented
Target/backend/linker:             not implemented
Independent deployment observers: not implemented
Deployment release:               none
```

Architecture finality does not imply deployment readiness.

A green model does not imply target correctness. A self-consistent checkpoint
does not imply event provenance. A generated artifact does not become semantic
source.

---

# 3. Immediate audit and remediation lane · `sec:backlog:remediation`

> **Lane:** F1
> **Status:** ACTIVE
> **Gate effect:** Every release-relevant F1 task blocks (`gate:phase1:exit`)
> **Evidence source:** Static repository review; each finding must be reproduced or explicitly disproved before closure
> **Rule:** A rejected finding is closed only by a focused test or typed argument demonstrating why the reported path is impossible

## 3.1 Remediation summary · `tbl:backlog:remediation`

| ID | Priority | Status | Finding | Primary owners |
|---|---:|---|---|---|
| `F1-001` | P0 | **TODO** | Public checkpoint reconstruction can accept provenance-free invented burn/clear rows. | model |
| `F1-002` | P1 | **TODO** | Realization observations cannot represent exact mixed movement-plus-destruction flows. | realization, model |
| `F1-003` | P1 | **TODO** | Compact-ASH realization omits sponsor cardinalities and an exact architecture weld. | realization |
| `F1-004` | P1 | **TODO** | Realization accepts multiple generic sponsor envelopes while the model rejects them. | realization, model |
| `F1-005` | P1 | **TODO** | Always-stale paper-stamp target retouches a success stamp and defeats claimed restat behavior. | document-stamps, Meson |
| `F1-006` | P1 | **TODO** | Sponsored-quiescence eligibility ignores active-backing-cap-blocked requests. | model |
| `F1-007` | P2 | **TODO** | Checker stamps are updated before stdout success is established. | cli-common, checker binaries |
| `F1-008` | P2 | **TODO** | Malformed non-PA imports in Realization Markdown may disappear without diagnostics. | labels |
| `F1-009` | P2 | **TODO** | Document-stamp `tree_ref` may combine selected-ref metadata with current worktree bytes. | document-stamps |
| `F1-010` | P2 | **TODO** | CI clean-tree check misses staged and untracked nonignored files. | scripts, ADR-011 gate |
| `F1-011` | P2 | **TODO** | Permissionless constructibility is hardcoded to compact ASH instead of derived from architecture and execution case. | realization |
| `F1-012` | P2 | **TODO** | Normative and companion prose contain floor/weld/conservatism inaccuracies. | realization document, companion docs |
| `F1-013` | P2 | **TODO** | Active graph planning still prescribes adapters superseded by D007. | planning |
| `F1-014` | P2 | **TODO** | Full Meson tests have an undeclared `jq` dependency. | Meson, integration script |
| `F1-015` | P3 | **TODO** | Build-facing shell/Python checkers do not satisfy or explicitly fall outside ADR-010 JSON diagnostics. | ADR/CLI/build tooling |
| `F1-016` | P3 | **TODO** | Planning statuses and completion checklists disagree with implementation state. | planning |
| `F1-017` | Gate | **BLOCKED** | Run and record the complete Phase-1 gate after F1 remediation. | repository-wide |

---

## F1-001 — Close the indexer-checkpoint provenance boundary

> **Status:** TODO
> **Priority:** P0
> **Primary files:** `packages/model/src/ledger.rs`,
> model indexer/checkpoint tests
> **Imports:** (`[RZ-sec:ledger:authentication]`),
> (`[RZ-obl:oracle:ledger]`)

### Finding

`IndexerCheckpoint` is publicly constructible and reconstructs a
`ReferenceIndexer` after checking internal event-table consistency, context
shape, ordering, and payload domains.

It does not independently establish that a checkpoint burn or clear row was
derived from an authenticated canonical transition.

The current public conversion can therefore treat a self-consistent invented
burn row as recognized history.

### Required decision

Choose one explicit boundary.

#### Option A — provenance-validating reconstruction

Checkpoint reconstruction receives enough authenticated chain material to
derive or verify:

- canonical block membership;
- transaction identity;
- operation/event type;
- live-receipt burn inputs;
- absence of ASH burn inputs;
- one fresh ASH output;
- consensus-enforced ASH value;
- clear STATE transition and destruction;
- block hash and checkpoint membership.

Checkpoint rows are compared against the independently derived event sequence.

#### Option B — trusted cache snapshot

If the checkpoint is an internal cache only:

- rename it to expose the trust class;
- make arbitrary construction or conversion crate-private;
- document that it is not event-recognition validation;
- bind persisted snapshots to a previously validated event report;
- prevent any public constructor from turning caller-authored recognized rows
  into a value called a validated reference indexer.

### Required tests

- invented burn with valid context and genesis clear;
- invented clear;
- block-hash mismatch;
- event not present in canonical chain;
- burn tag/payload without burn transaction;
- compacted ASH presented as a burn;
- valid trusted snapshot round trip;
- valid provenance-derived reconstruction.

### Exit check

- [ ] no public untrusted path can manufacture attestation credit from recognized rows alone;
- [ ] checkpoint trust class is explicit in types and documentation;
- [ ] event and query evidence remain separate;
- [ ] model tests prove the invented-burn regression;
- [ ] realization/document claims match the implemented boundary.

Identity impact must be recorded. A cache schema change need not move protocol
identity, but report or artifact schemas may change.

---

## F1-002 — Preserve exact mixed canonical flows in realization observations

> **Status:** TODO
> **Priority:** P1
> **Primary files:** `packages/realization/src/observation.rs`,
> `packages/realization/src/evaluate.rs`,
> `packages/model/src/certify.rs`,
> `packages/model/src/history.rs`,
> model-to-realization projection tests

### Finding

The model partitions canonical value by exact flow. One flow may contain both:

- current-state destinations; and
- one or more destruction legs.

Examples:

```text
partial clear:
    ASH source
    → residual ASH
    + tag-recon destruction

terminal settlement:
    vault source
    → receipt outputs
    + distribution-residue destruction
```

Certificate derivation currently flattens such a flow into separate delta rows
sharing the same source set.

The realization observation then rejects repeated source references and also
requires a destruction row’s source total to equal the destruction amount. That
cannot represent:

\[
\text{source}=\text{movement}+\text{destruction}
\]

when both terms are positive.

### Required design

Prefer one observed exact-flow type:

```text
asset
source references
destination references
optional movement kind
destruction legs
```

Validate:

\[
\sum \text{sources}
=
\sum \text{destinations}
+
\sum \text{destruction legs}
\]

Source and destination uniqueness is enforced between exact flows, not between
flattened family summaries.

The active architecture delta-family set remains a separately derived
projection.

If flattened deltas are retained, they require a stable flow-group identity and
must be regrouped before partition and arithmetic validation.

### Required tests

- partial clear with residual ASH;
- full clear without residual;
- terminal settlement with receipts and positive residue;
- terminal settlement with residue only;
- continuing settlement;
- one source incorrectly used by two separate flows;
- one destination funded by two flows;
- active delta-family projection remains equal to the manifest.

### Exit check

- [ ] model-valid mixed flows project into realization without weakening;
- [ ] exact source/destination uniqueness remains enforced;
- [ ] destruction arithmetic is checked at flow level;
- [ ] compact ASH and live transfer remain unchanged semantically;
- [ ] accepted transition relation and architecture denotation impact are reviewed.

---

## F1-003 — Complete the compact-ASH architecture/realization weld

> **Status:** TODO
> **Priority:** P1
> **Primary files:** `packages/realization/src/declarations/compact_ash.rs`,
> `packages/realization/src/validate.rs`,
> compact-ASH realization tests

### Required compact-ASH relation census

The realization must cover exactly:

- ASH input minimum two;
- ASH input maximum `ASH_BATCH_MAX`;
- exactly one ASH output;
- optional `PLAIN_LBTC` sponsor inputs bounded by
  `FEE_SPONSOR_INPUT_MAX`;
- optional `PLAIN_LBTC` sponsor change bounded by exactly one;
- permissionless operation authorization;
- sponsor-owner authorization on sponsor inputs;
- exact ownerless-lateral `U` flow;
- one generic sponsor envelope at most;
- exact allowed input/output families;
- exact open-flow set;
- exact value-flow set;
- exact bound set;
- exact root policy;
- exact projection policy;
- empty data-output set;
- exact witness set;
- public constructibility;
- supported representation and lifecycle paths.

### Architecture weld

`validate_compact_ash_architecture` must compare every relevant architecture row
bidirectionally, matching the rigor already used for live transfer.

Presence-only checks are insufficient. Extra declarations must fail.

### Required mutation tests

- sponsor input authorization changed;
- sponsor input maximum changed;
- sponsor output maximum changed;
- sponsor bound removed or replaced;
- extra value-flow class;
- missing value-flow class;
- missing witness;
- extra witness where exact equality is required;
- unexpected data-output family;
- changed open-flow set;
- changed projection;
- changed root use;
- changed ASH minimum or output count.

### Exit check

- [ ] every compact-ASH architecture field used by realization is exact;
- [ ] every realization relation has one architecture or realization owner;
- [ ] sponsor cardinality mutations fail the intended relation;
- [ ] architecture and realization coverage are bidirectional;
- [ ] no stringly comparison or generated-publication input is added.

---

## F1-004 — Enforce one generic sponsor envelope in realization

> **Status:** TODO
> **Priority:** P1
> **Primary files:** `packages/realization/src/relation.rs`,
> `packages/realization/src/evaluate.rs`,
> pilot declaration and tests

### Finding

The model rejects more than one `FeeSponsor` flow.

Realization currently checks that sponsor flows are isolated and balanced but
can accept several disjoint valid sponsor flows.

### Required relation

Add an explicit typed constraint equivalent to:

```text
FeeSponsor flow count ∈ {0,1}
```

The relation should have:

- stable typed identity;
- focused failure;
- source provenance;
- architecture/ABI handoff where later relevant.

Do not bury the count rule in an unrelated conservation failure.

### Required tests

For both Phase-1 pilots:

- zero sponsor flow passes;
- one valid sponsor flow passes;
- two disjoint valid sponsor flows fail;
- one flow with duplicated source fails separately;
- one flow with duplicated destination fails separately;
- sponsor input count above the architecture bound fails cardinality.

### Exit check

- [ ] model and realization agree on envelope multiplicity;
- [ ] sponsor balance, authorization, cardinality, and multiplicity remain distinct claims;
- [ ] future ABI work can identify the exact sponsor-envelope relation.

---

## F1-005 — Restore real Ninja restat behavior for paper stamps

> **Status:** TODO
> **Priority:** P1
> **Primary files:** `packages/document-stamps`,
> `papers/attestation/meson.build`,
> `scripts/test-meson-mock.sh`
> **Imports:** (`[ADR014-rule:build:output-or-stamp]`),
> (`[ADR014-rule:build:mock-contract]`)

### Finding

The paper-stamp generator has real outputs:

```text
stamps.tex
source-date-epoch
```

but also touches a success stamp on every run. The target is always stale, so
the success stamp changes on every no-op build and can dirty downstream edges
despite compare-if-changed writes on the real outputs.

The command also mixes both ADR-014 effect classes: real outputs and stamp.

### Required change

Preferred solution:

- remove the generator success stamp;
- retain only `stamps.tex` and `source-date-epoch` as outputs;
- keep compare-if-changed writes;
- probe the real outputs in any Meson existence test;
- preserve always-stale execution only where Git state cannot be represented as
  an ordinary dependency.

### Mock-contract improvement

The mock contract must verify command non-execution on the second unchanged
build, not merely byte equality.

Use one deterministic mechanism such as:

- command invocation counters;
- unchanged command-log mtimes;
- Ninja log inspection;
- a dedicated mock trace;
- `ninja -d explain` assertions.

### Exit check

- [ ] unchanged stamp derivation leaves both real-output mtimes unchanged;
- [ ] downstream TeX/flatten commands do not rerun after an unchanged derivation;
- [ ] no-op mirror bytes and command counts remain unchanged;
- [ ] changed paper Git state reruns the required path;
- [ ] dirty paper state still fails;
- [ ] ADR-014 output-or-stamp discipline is restored.

---

## F1-006 — Make sponsored quiescence cap-aware

> **Status:** TODO
> **Priority:** P1
> **Primary files:** `packages/model/src/quiescence.rs`,
> `packages/model/src/maintenance.rs`,
> model quiescence/property tests

### Finding

A valid open request may be locally well-formed but exceed available active
backing:

\[
\Omega+Q+\delta>\texttt{ACTIVE\_BACKING\_MAX}
\]

Such a request is inert and cancelable but cannot currently be admitted.

The deterministic sponsored scheduler treats locally valid requests as
admissible, selects them, and can fail with `ActiveBackingCapExceeded`.
Eligibility nevertheless classifies a live pool as eligible.

### Required semantics

Introduce a typed residual such as:

```text
request blocked by active-backing capacity
```

Distinguish:

- malformed request;
- cross-pool request;
- sealed-pool request;
- cap-blocked request;
- request that fits individually;
- batch that fails only because its aggregate exceeds headroom.

The scheduler should choose a deterministic nonempty fitting subset when one
exists.

If no request fits and no permissionless maintenance operation can restore
headroom, return a residual classification rather than claiming full sponsored
discharge.

### Required tests

- exact headroom;
- one unit above headroom;
- several requests whose full batch exceeds headroom but a subset fits;
- one individually oversized request;
- cap-blocked plus malformed request;
- cap-blocked on sealed pool;
- scheduler makes progress and then returns a named residual;
- near-cap property traces.

### Exit check

- [ ] `Eligible` implies the deterministic sponsored driver succeeds;
- [ ] every ineligible valid world has a named residual;
- [ ] subset selection is canonical;
- [ ] request cancellation remains owner-controlled and is not falsely called permissionless maintenance;
- [ ] the theorem and human documentation state the exact preconditions.

---

## F1-007 — Commit checker stamps only after stdout succeeds

> **Status:** TODO
> **Priority:** P2
> **Primary files:** `packages/cli-common`,
> `check-generated`, `check-labels`, `census-audit`

### Finding

The checkers can update their success stamp before JSON result serialization,
stdout writing, and flushing are known to have succeeded.

ADR-014 requires failed commands to leave the stamp untouched.

### Required shared API

Provide one shared command path with this order:

```text
derive report
serialize report
write and flush stdout
touch stamp
return success
```

The stamp operation must remain absent from:

- runtime failure;
- serialization failure;
- stdout write failure;
- stdout flush failure;
- TTY refusal;
- usage failure.

Avoid per-binary copies of the ordering logic.

### Required tests

Use an injectable writer or subprocess harness to cover:

- successful report and stamp update;
- serialization failure;
- broken pipe;
- full output file;
- stamp I/O failure after stdout success;
- unchanged stamp bytes and mtime on every pre-stamp failure.

### Exit check

- [ ] every Rust checker uses the shared path;
- [ ] failed stdout emission leaves the prior stamp untouched;
- [ ] JSON report and stamp semantics remain documented;
- [ ] existing exit classes remain unchanged.

---

## F1-008 — Reject malformed Realization imports exhaustively

> **Status:** TODO
> **Priority:** P2
> **Primary files:** `packages/labels/src/repository.rs`,
> labels tests

### Finding

A square-bracketed Realization token that is neither a valid imported owner
token nor a valid legacy model label may be ignored without a diagnostic.

### Required parsing policy

For every participating single-backtick square token in Realization:

1. parse known imported owner and local label;
2. otherwise parse the valid legacy model form if that compatibility path is
   intentionally retained;
3. otherwise emit exactly one typed diagnostic.

Expected diagnostic classes:

- unknown prefix → `UnknownOwner`;
- known owner with malformed local label → `InvalidLabel`;
- imported token without required parentheses → form error;
- self-qualified R13 import → owner-boundary form error;
- valid model legacy token → synthetic MODEL import.

### Required tests

- unknown owner;
- malformed ADR owner number;
- malformed known-owner local label;
- valid ADR import;
- valid MODEL compatibility form;
- nonparenthesized import;
- self-qualified R13 import;
- double-backtick and fenced examples remain nonparticipating.

### Exit check

- [ ] no participating imported-looking token disappears silently;
- [ ] diagnostics are traversal-order independent;
- [ ] generated registers remain scoped and deterministic.

---

## F1-009 — Make document-stamp revision selection coherent

> **Status:** TODO
> **Priority:** P2
> **Primary files:** `packages/document-stamps`,
> stamp CLI and integration tests

### Finding

An arbitrary selected `tree_ref` currently supplies Git mode, history, and tree
identity, while input bytes are read from the current worktree.

A non-HEAD ref can therefore produce hybrid metadata.

### Required decision

Choose one contract.

#### Full selected-ref support

- read publication input bytes from Git objects at `tree_ref`;
- derive mode, bytes, history, tree identity, and timestamps from the same ref;
- verify render-mode source bytes equal the selected ref or stage the selected
  source explicitly.

#### HEAD-only support

- resolve `tree_ref` and `HEAD`;
- reject inequality;
- document that the command operates only on checked-out committed HEAD.

Do not retain an API that appears to support arbitrary refs while combining
revisions.

### Required tests

Create two commits with different input bytes and:

- request the older commit from a newer checkout;
- verify coherent older-ref derivation or explicit rejection;
- verify dirty HEAD rejection;
- verify non-input paper and docs scope behavior remains unchanged.

### Exit check

- [ ] all four stamp values derive from one coherent revision;
- [ ] render-mode source and metadata agree;
- [ ] CLI documentation states the exact contract;
- [ ] no raw Git argv or secret data enters diagnostics.

---

## F1-010 — Strengthen repository cleanliness verification

> **Status:** TODO
> **Priority:** P2
> **Primary files:** `scripts/ci.sh`,
> clean-tree tests, ADR-011 documentation

### Finding

`git diff --exit-code` detects unstaged tracked changes only.

It misses:

- staged changes;
- untracked nonignored files.

### Required check

Use a complete status check such as:

```sh
git status --porcelain=v1 --untracked-files=all
```

The check must fail on every nonignored path or index/worktree change.

Ignored build artifacts remain permitted.

### Required tests

- clean tree passes;
- unstaged tracked modification fails;
- staged tracked modification fails;
- untracked nonignored file fails;
- ignored build artifact passes;
- generated checker leaving a stray file fails.

### Exit check

- [ ] CI’s final lane matches its “clean working tree” name;
- [ ] staged and untracked paths are named in diagnostics;
- [ ] release/document scripts use consistent cleanliness semantics where required.

---

## F1-011 — Derive constructibility authorization from architecture

> **Status:** TODO
> **Priority:** P2
> **Primary files:** `packages/realization/src/validate.rs`,
> constructibility policy and tests

### Finding

Permissionless constructibility validation is currently selected by:

```text
operation == compact-ash
```

rather than by the supplied architecture’s authorization class.

This becomes fail-open when later permissionless operations are added.

### Required policy

Derive constructibility authorization from architecture.

Support at least:

- permissionless;
- owner-authorized;
- operator-authorized;
- refund-key;
- client-authorized;
- cadence band with distinct operator and delayed-permissionless execution
  cases.

A one-bit permissionless flag is insufficient for cadence-band analysis.

### Required tests

- synthetic permissionless operation with owner-private dependency;
- owner-authorized operation with the same dependency;
- operator-authorized operation;
- cadence operator case;
- cadence delayed permissionless case;
- architecture authorization mutation changes constructibility validation;
- cross-operation laundering remains rejected.

### Exit check

- [ ] no operation-name allowlist determines authorization semantics;
- [ ] every execution case has explicit witness availability;
- [ ] permissionless cases reject owner/operator-private dependencies;
- [ ] sponsor-local data remains isolated.

---

## F1-012 — Correct normative and companion-document inaccuracies

> **Status:** TODO
> **Priority:** P2
> **Primary files:** `docs/attestation/realization.md`,
> `docs/attestation/human.md`,
> generated realization register if labels move

### Required corrections

#### Floor arithmetic

Replace the false statement that \(\Omega/Y\) is irrational.

The correct statement is that the floor is generally nonintegral and may not
have an exact finite representation in the target’s selected fixed-width
encoding.

#### STATE/RESV weld

Replace prose saying `clear` is the only STATE-without-RESV operation.

The typed manifest declares:

```text
STATE and RESV:
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

The human companion must not claim every pool-touching transaction spends both
roots.

#### Conservative attestation valuation

The last-clearing floor may under-credit the recorded sacrifice relative to the
burn-instant redeemable value. It does not overstate sacrifice cost.

### Versioning review

Treat these as presentation corrections unless a semantic or label value
changes.

Record:

- realization letter impact;
- architecture semantic-hash impact;
- behavioural-hash impact;
- Layer-0 anchor-set impact;
- generated register impact.

### Exit check

- [ ] corrected prose matches typed architecture and model;
- [ ] no normative equation or manifest fact changes unintentionally;
- [ ] document welds and label checks pass;
- [ ] generated registers remain current;
- [ ] revision history records the presentation correction.

---

## F1-013 — Reconcile graph planning with D007

> **Status:** TODO
> **Priority:** P2
> **Primary files:** `plans/backlog.md`,
> `plans/research/compiler-algorithms.md`,
> `plans/research/linker-algorithms.md`,
> package contracts

### Finding

D007 prohibits first-party graph wrappers and generic graph adapters.

Active planning still requests:

- a canonical graph adapter;
- a frozen graph adapter;
- a private graph adapter.

### Required rewrite

Replace adapter work with:

```text
direct Petgraph graph construction
+
typed stable keys
+
key/index lookup metadata
+
canonical insertion
+
canonical result normalization
+
first-party typed publication projection
```

Package-local helper functions are permitted.

A wrapper type that reproduces or hides Petgraph storage/traversal is not.

### Exit check

- [ ] no active task contradicts D007;
- [ ] current labels and realization implementations remain direct Petgraph users;
- [ ] C1-005 is reframed around canonical construction/oracles;
- [ ] no shared graph crate is introduced;
- [ ] graph local handles remain absent from semantic identity.

---

## F1-014 — Remove or declare the Meson `jq` dependency

> **Status:** TODO
> **Priority:** P2
> **Primary files:** `scripts/test-attestation-stamps.sh`,
> `meson.build`, `README.md`

### Finding

The full Meson test surface invokes `jq`, but Meson does not resolve it and the
requirements do not declare it.

### Preferred fix

Use `python3` for JSON extraction because Python is already a required,
Meson-resolved test dependency.

Alternative: resolve and pass an explicit `jq` program and document it.

### Exit check

- [ ] full Meson tests run in an environment without `jq`, or `jq` is explicitly required and passed by path;
- [ ] subprocesses do not depend on ambient undeclared PATH tools;
- [ ] README requirements match the full test surface.

---

## F1-015 — Resolve the shell/Python checker output contract

> **Status:** TODO
> **Priority:** P3
> **Primary files:** ADR-010, ADR-014,
> `scripts/check_plans.py`, `scripts/check-forbidden-text.sh`,
> Meson checker wiring

### Finding

Build-facing first-party shell/Python checkers emit plain text, while ADR-010
and ADR-014 describe JSON-only command streams for first-party executable
checkers.

### Required decision

Preferred:

- move build-facing checker command boundaries under Rust/`cli-common`;
- emit typed JSON reports and JSON diagnostics;
- retain shell/Python only as internal implementation where the external
  command contract remains JSON.

Alternative:

- adopt a new ADR explicitly narrowing ADR-010;
- define which orchestration scripts are excluded and why;
- keep the boundary mechanically testable.

Do not silently leave policy broader than implementation.

### Exit check

- [ ] every shipped checker is classified;
- [ ] stream and exit behavior are tested;
- [ ] Meson report files have a documented format;
- [ ] no plain diagnostic path remains accidentally covered by ADR-010.

---

## F1-016 — Reconcile planning status and completion evidence

> **Status:** TODO
> **Priority:** P3
> **Primary files:** this backlog, phase cards, D007/D008 handoff records

### Required corrections

- R1-013 has one status everywhere;
- C1-004 reflects the already adopted Petgraph dependency and identifies the
  exact review evidence still missing;
- completed C1 tasks have completed evidence rather than unchecked exit lists;
- D007-superseded graph-adapter tasks are rewritten;
- Phase 1 is described as implementation-complete only if every required
  implementation task is actually complete;
- gate completion remains blocked until F1 remediation and command evidence
  pass.

### Exit check

- [ ] summary and detailed statuses agree;
- [ ] every `DONE` task satisfies (`rule:backlog:done`);
- [ ] current phase card and roadmap agree;
- [ ] no accepted decision is contradicted by active planning.

---

## F1-017 — Run and record the complete Phase-1 gate

> **Status:** BLOCKED
> **Depends on:** `F1-001` through `F1-016`, required R1 tasks
> **Output:** Phase-1 evidence record and status transition

### Required environments

- declared Rust MSRV;
- current stable Rust;
- canonical Meson build;
- required TeX/document environment for the document lane;
- environment without accidental undeclared helper dependencies where
  practical.

### Required command surface

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

A skipped required lane is not a pass.

`cargo audit` remains governed by ADR-011: absence is reported as skipped, not
silently green. Release policy decides whether the tool is mandatory at the
Phase-1 gate.

### Evidence record

Record:

- tool versions;
- command results;
- skipped optional tools;
- test counts where practical;
- generated-artifact status;
- label/census status;
- no-op Meson graph result;
- document reproducibility result;
- final clean-tree output;
- identity changes caused by remediation;
- residual open findings.

### Exit check

Phase 1 moves to complete only when (`gate:phase1:exit`) and
(`gate:backlog:current`) both pass.

---

# 4. Historical baseline record · `sec:backlog:baseline`

> **Recorded:** 2026-07-15
> **Status:** Historical and immutable

The baseline identities remain recorded in the annotated tag and
[`phases/00-baseline.md`](phases/00-baseline.md).

Current source identities may differ through reviewed presentation or
hash-algorithm migrations. Historical records are not rewritten when later
review finds additional work.

## 4.1 Historical identities · `tbl:backlog:baseline-identities`

| Field | Baseline value |
|---|---|
| architecture schema | 17 |
| realization version | tracked compiler-line binding |
| denotation-preserving change | 13a |
| publication status | final |
| semantic algorithm | `sha256-canonical-json-v2` |
| semantic hash | |
| behavioural algorithm | `sha256-canonical-json-behavioural-v2` |
| behavioural hash | |
| Layer-0 version | `0.5.0` |
| Layer-0 anchor-set hash | |
| reproducible PDF SHA-256 | |

The current behavioural-hash algorithm is v3. Its migration record is owned by
the architecture versioning gate and Realization revision history.

## 4.2 Historical evidence · `tbl:backlog:baseline-evidence`

| Lane | Recorded result |
|---|---|
| Rust MSRV | 1.88.0 green |
| stable Rust | green at baseline |
| formatting | green |
| Clippy `-D warnings` | green |
| debug/release tests | green |
| generated-artifact check | green |
| documentation/label checks | green |
| Meson document build | green |
| PDF reproducibility | byte-identical clean builds |
| clean tree | green under the then-implemented check |
| `cargo audit` | skipped loudly when unavailable |

The F1 lane refines current assurance. It does not alter the historical record.

---

# 5. Algorithm and dependency laws · `sec:backlog:algorithm-laws`

## 5.1 Problem-class separation · `rule:backlog:problem-classes`

```text
typed semantic AST:
    first-party typed source

graph storage, reachability, SCC, traversal:
    direct Petgraph graphs

exact semantic arithmetic:
    checked integers, BigInt, exact rationals

numerical dense/sparse linear algebra:
    private faer working values when a concrete consumer exists

proof and placement selection:
    exact finite search initially

LP, MILP, SAT, SMT:
    separate dependency and certificate review after measured need

target tree optimization:
    explicit bounded-depth coding algorithm

target arithmetic:
    exact target relation plus independent host reference

cryptographic and consensus mathematics:
    reviewed target/cryptographic libraries and target-native evidence
```

A numerical linear solve is not proof selection. An optimizer is not a graph
algorithm. A graph library is not a semantic AST. A small floating residual is
not exact semantic equality.

## 5.2 Exactness · `rule:backlog:exactness`

Exact semantic, conservation, authorization, identity, calibration, and release
claims use:

- checked bounded integers;
- arbitrary-precision integers;
- reduced exact rationals;
- exact finite search;
- independently checked certificates;
- target-native execution where the claim concerns a concrete target.

Numerical analysis may produce:

- candidates;
- diagnostics;
- sensitivity information;
- least-squares fits;
- conditioning and rank diagnostics;
- conservative estimates.

A release-sensitive numerical result must become and be checked as:

- an exact integer;
- a reduced rational;
- a conservative exact interval or bound;
- an independently checked primal/dual or other certificate.

## 5.3 Identity · `rule:backlog:algorithm-identity`

Every analysis distinguishes:

```text
local handle:
    process-local arena, graph, matrix, or solver position

stable key:
    complete typed semantic identity

digest:
    optional domain-separated commitment
```

The following are never semantic identity:

- Petgraph `NodeIndex` or `EdgeIndex`;
- matrix row/column position;
- solver variable number;
- hash-map iteration order;
- source traversal order;
- pivot order;
- raw floating-point bits;
- solver iteration count;
- thread schedule;
- temporary path.

## 5.4 Canonical ordering · `rule:backlog:canonical-order`

Every canonical graph result, matrix, plan, bundle, ABI, vector set, and report
uses explicit stable-key ordering.

Third-party iteration order is never assumed canonical.

Direct Petgraph graph values are permitted. Direct Petgraph serialization is
not the project’s semantic publication schema.

## 5.5 Independent oracles · `rule:backlog:oracles`

Every nontrivial production algorithm has an independent small-instance oracle.

| Production analysis | Oracle |
|---|---|
| topological order | valid-order enumeration and least-key policy |
| SCC | mutual-reachability equivalence |
| interning | non-interned evaluator |
| dependency closure | repeated complete scan |
| exact linear solve | second rational elimination path |
| numerical solve | exact or high-precision comparison where applicable |
| proof selection | exhaustive candidate enumeration |
| placement | exhaustive carrier-subset enumeration |
| taptree construction | exhaustive small binary-tree enumeration |
| relocation | independently constructed structured expected program |
| calibration search | exhaustive finite candidate range |
| resource formula | complete target transaction measurement |

## 5.6 Complexity failure · `rule:backlog:complexity`

A search or analysis exceeding its explicit budget fails with a typed
complexity error.

It must not:

- drop a relation;
- weaken authorization;
- expose additional information silently;
- remove a lifecycle exit;
- select a hidden greedy fallback;
- claim optimality from incomplete search;
- accept the best partial result seen so far.

---

# 6. Compiler/linker preparation lane · `sec:backlog:c1`

> **Lane status:** ACTIVE PREPARATION
> **Production compiler entry:** blocked on (`gate:phase1:exit`)
> **Allowed now:** dependency review, typed contracts, exact oracles,
> synthetic prototypes, and complexity measurements
> **Not allowed now:** stable compiler/linker ABI, target fields in realization,
> production-marked backend patterns, or final calibration claims

## 6.1 C1 task summary · `tbl:backlog:c1-tasks`

| ID | Status | Task | Depends on | Output |
|---|---|---|---|---|
| `C1-001` | **DONE** | Land compiler/linker/mathematics/solver research notes and census updates. | none | research notes |
| `C1-002` | **DONE** | Record direct Petgraph substrate decision. | `C1-001` | D007 |
| `C1-003` | **DONE** | Record exact/certified mathematics policy. | `C1-001` | D008 |
| `C1-004` | **IN PROGRESS** | Close concrete dependency release, feature, MSRV, license, transitive, unsafe, determinism, and advisory review. | `C1-002`, `C1-003` | dependency review record |
| `C1-005` | **TODO** | Prototype canonical construction and normalization over direct Petgraph graphs. | `C1-004`, `F1-013` | direct-graph oracle prototype |
| `C1-006` | **TODO** | Prototype exact keyed linear systems and Bareiss elimination. | `C1-004` | exact matrix prototype |
| `C1-007` | **TODO** | Prototype certified numerical analysis only after a concrete consumer exists. | `C1-004`, `C1-006` | optional numerical prototype |
| `C1-008` | **BLOCKED** | Implement exact proof-plan search prototype. | Phase-2 relation vocabulary | exhaustive/branch-and-bound prototype |
| `C1-009` | **BLOCKED** | Implement execution-case-aware placement prototype. | compiler carrier/case types | exact placement prototype |
| `C1-010` | **TODO** | Prototype typed symbol resolution and SCC policy on direct Petgraph graphs. | `C1-005` | synthetic linker graph prototype |
| `C1-011` | **TODO** | Prototype structured or simultaneous fixed-width relocation. | `C1-010` | relocation prototype |
| `C1-012` | **TODO** | Prototype deterministic bounded-depth taptree construction. | `C1-010` | package-merge prototype |
| `C1-013` | **BLOCKED** | Build the complete algorithm-oracle suite. | `C1-005`–`C1-012` | property/exhaustive oracle tests |
| `C1-014` | **BLOCKED** | Run preparation review and hand off to Phase 2. | `C1-001`–`C1-013`, Phase 1 | accepted algorithm policy |

## 6.2 C1-004 dependency closure

### Petgraph

Current selected dependency:

```text
petgraph 0.8.3
```

Current workspace features:

```text
serde-1
rayon
dot_parser
unstable
generate
```

The dependency is already used by graph-owning packages.

C1-004 closes only when the repository records:

- exact upstream release;
- license;
- Rust 1.88 support;
- selected feature rationale;
- default-feature behavior;
- transitive graph;
- duplicate versions;
- dependency-internal unsafe/SIMD boundary;
- parallel determinism implications;
- advisory result;
- replacement boundary.

### Numerical dependencies

`num-rational` and `faer` are not adopted merely because research names them.

Adopt only when a concrete implementation task consumes them.

Phase 1 requires no numerical dependency under D008.

### Exit commands

```sh
cargo tree --locked -p petgraph -e features
cargo tree --locked -i petgraph
```

When future candidates are actually added:

```sh
cargo tree --locked -p num-rational -e features
cargo tree --locked -p faer -e features
cargo tree --locked -i num-rational
cargo tree --locked -i faer
```

## 6.3 C1-005 direct-Petgraph construction prototype

This task does not create a graph wrapper.

Required properties:

- concrete Petgraph graph type owned by each package;
- first-party typed node and edge weights;
- stable-key collection;
- canonical node insertion;
- canonical edge insertion;
- stable key ↔ local index maps;
- explicit self-loop and parallel-edge policies;
- deterministic diagnostics;
- canonical stable-key result projection;
- no graph index in public output;
- no direct Petgraph serialization as semantic publication.

Required algorithms:

- canonical Kahn topological ordering where canonical order is required;
- Petgraph SCC/reachability/traversal;
- deterministic SCC normalization;
- deterministic representative-cycle diagnostics;
- reverse dependency closure.

Required tests:

- node/edge insertion permutations;
- unrelated-node insertion;
- duplicate keys;
- missing endpoints;
- self-loop;
- one large SCC;
- deep chain;
- disconnected graph;
- dense small graph;
- repeated construction equality.

## 6.4 C1-006 exact keyed linear-system prototype

Required typed values:

- stable row keys;
- stable column keys;
- exact rational coefficients;
- canonical exact coefficient DTO;
- exact right-hand side;
- exact result or certificate;
- no matrix position as semantic identity.

Implement fraction-free Bareiss elimination for small systems.

Retain a simpler independent exact-rational Gaussian-elimination oracle.

Tests include:

- identity and diagonal matrices;
- full-rank integer systems;
- exact rational solutions;
- singular and inconsistent systems;
- row and column permutations;
- redundant equations;
- nonzero equation scaling;
- coefficient mutation;
- coefficient-growth boundaries.

## 6.5 C1-007 certified numerical prototype

This task remains optional until an implemented numerical consumer exists.

If activated, require explicit methods for:

- pivoted LU;
- QR;
- SVD;
- Cholesky after SPD validation;
- multiple right-hand sides.

For \(Ax=b\), record at least:

\[
\eta=
\frac{\lVert b-Ax\rVert_\infty}
{\lVert A\rVert_\infty\lVert x\rVert_\infty+\lVert b\rVert_\infty}
\]

Also record decomposition, rank, condition estimate, residual, backward error,
tolerance policy, and exact or conservative certification mode.

Raw floating-point output remains nonsemantic.

## 6.6 C1-008/C1-009 exact planning prototypes

Proof and placement planning use hard constraints before objective cost.

Required hard constraints include:

- semantic relation preservation;
- target capability;
- authenticatable facts;
- witness availability;
- permissionless constructibility;
- representation;
- disclosure;
- lifecycle;
- relation and execution-case coverage.

Initial solver:

```text
deterministic exact enumeration or branch-and-bound
+
exhaustive small-instance oracle
```

No external optimizer becomes production-load-bearing without measured need and
a separately reviewed certificate boundary.

## 6.7 C1-010 through C1-012 linker prototypes

### Symbol/SCC prototype

- complete typed definition census;
- complete reference resolution;
- typed reference graph;
- canonical SCCs;
- typed condensation DAG;
- explicit strategy on every cyclic edge.

### Relocation prototype

Prefer structured target programs before byte serialization.

If byte relocation is unavoidable:

- fixed width;
- typed source and destination;
- exact encoding;
- exact expected placeholder;
- no overlap;
- one resolution per mandatory relocation;
- replacements computed against pristine bytes;
- simultaneous application;
- full post-link reparse and validation.

### Taptree prototype

Initial objective:

\[
\min\sum_i w_i d_i
\qquad\text{subject to}\qquad
d_i\le L
\]

Use deterministic length-limited Huffman/package-merge with stable-key
tie-breaking and exact target branch ordering.

Compare small instances with exhaustive full-binary-tree enumeration.

## 6.8 C1-013/C1-014 completion

The algorithm lane is ready for Phase 2 when:

- direct Petgraph construction policy is demonstrated;
- exact matrix policy is demonstrated where required;
- numerical work remains absent or certified;
- exact proof/placement search agrees with exhaustive oracles;
- SCC/cycle, relocation, and taptree prototypes pass;
- complexity limits fail closed;
- dependency versions/features are reviewed;
- no third-party local handle enters semantic identity;
- Phase 1 has passed independently.

---

# 7. Typed realization lane · `sec:backlog:r1`

> **Gate status:** CURRENT
> **Phase card:** [`phases/01-realization.md`](phases/01-realization.md)
> **Package contract:** [`packages/realization.md`](packages/realization.md)
> **Allowed first-party direct dependency:** `architecture`
> **Graph substrate:** direct Petgraph graphs under D007
> **Numerical dependency:** none for Phase 1

## 7.1 R1 task summary · `tbl:backlog:r1-tasks`

| ID | Status | Task | Output |
|---|---|---|---|
| `R1-001` | **DONE** | Create realization crate and package contract. | workspace crate |
| `R1-002` | **DONE** | Define local handles, stable keys, and typed semantic IDs. | identity types |
| `R1-003` | **DONE** | Define typed domains, expression graph, and deterministic evaluator. | typed expression graph |
| `R1-004` | **DONE** | Define minimum semantic relation vocabulary. | relation types |
| `R1-005` | **DONE** | Define constructibility, lifecycle, representation, and operation declarations. | operation adjuncts |
| `R1-006` | **DONE** | Implement deterministic scoped derivation. | scoped realization |
| `R1-007` | **DONE**, remediation open | Declare compact ASH. | first pilot |
| `R1-008` | **DONE**, remediation open | Declare live receipt transfer. | second pilot |
| `R1-009` | **DONE** | Derive pilot declassification. | typed disclosure result |
| `R1-010` | **DONE**, remediation open | Add architecture/realization bidirectional validation. | validation suite |
| `R1-011` | **DONE**, remediation open | Add model-conformance tests. | conformance evidence |
| `R1-012` | **DROPPED** | Add a realization publication only if a real consumer exists. | no Phase-1 publication |
| `R1-013` | **BLOCKED** | Run and record Phase-1 exit gate. | complete Phase-1 evidence |

“Remediation open” means the existing implementation is present but F1 findings
must close before the corresponding task contributes to the phase exit.

## 7.2 Completed Phase-1 foundation

Current realization implementation provides:

- architecture binding;
- explicit partial scope;
- checked amount and count domains;
- typed stable fact/expression/relation IDs;
- direct Petgraph expression and relation graphs;
- constructibility graph;
- lifecycle graph;
- disclosure graph;
- deterministic projections;
- compact-ASH declaration;
- live-transfer declaration;
- dependency-derived pilot declassification;
- architecture conformance checks;
- post-execution model observation and conformance evaluation;
- focused mutation tests;
- public API integration tests.

## 7.3 Remaining realization closure

Before `R1-013` can complete:

- `F1-002` must make mixed exact flows representable;
- `F1-003` must complete the compact-ASH architecture weld;
- `F1-004` must enforce one sponsor envelope;
- `F1-011` must derive constructibility authorization from architecture;
- all realization tests must pass in debug and release under MSRV and stable.

## 7.4 R1-012 rationale

No Phase-1 realization publication is required.

The current real consumers are Rust packages and tests consuming typed values
directly. Adding a publication now would create schema and migration work
without an external or review consumer.

If a later consumer appears, publication work requires:

- one typed source;
- explicit schema;
- canonical ordering;
- deterministic bytes;
- unknown-field rejection;
- explicit writer;
- non-writing checker;
- no reverse semantic dependency;
- explicit partial/complete scope.

## 7.5 R1-013 gate

R1-013 completes only through `F1-017`.

Required Phase-1 properties include:

- architecture remains the only first-party direct dependency;
- Petgraph remains private implementation substrate;
- local handles and semantic keys remain distinct;
- both pilots are architecture-welded;
- mixed flows have an extension-safe representation;
- declassification is dependency-derived;
- constructibility and lifecycle are explicit;
- model execution remains independent of realization acceptance;
- pilot scope cannot be mistaken for complete scope;
- no generated file, documentation, target type, or planning file is consumed;
- all repository lanes pass;
- the final tree is actually clean.

---

# 8. Research and prototype register · `sec:backlog:research`

## 8.1 Active preparation questions · `tbl:backlog:research-active`

| ID | Research note | Status | Blocks |
|---|---|---|---|
| `Q-COMPILER-ALG` | [`compiler-algorithms.md`](research/compiler-algorithms.md) | Rewrite required under D007, then prototype | Compiler identity, proof, placement, and coverage |
| `Q-LINKER-ALG` | [`linker-algorithms.md`](research/linker-algorithms.md) | Rewrite required under D007, then prototype | SCC, relocation, taptree, and carrier closure |
| `Q-NUMERICAL` | [`numerical-linear-algebra.md`](research/numerical-linear-algebra.md) | Dependency review and concrete-consumer gate | Certified numerical analysis |
| `Q-OPTIMIZATION` | [`optimization-solvers.md`](research/optimization-solvers.md) | Exact prototype required | Proof, placement, and calibration optimization |

## 8.2 Target-dependent questions · `tbl:backlog:research-target`

| ID | Research note | Status | Blocks |
|---|---|---|---|
| `Q-STATE` | [`state-constructor.md`](research/state-constructor.md) | Prototype required | STATE backend ABI and Phase 6 |
| `Q-ARITH` | [`wide-arithmetic.md`](research/wide-arithmetic.md) | Prototype and measurement required | Redemption, settlement, cycle |
| `Q-DECLASS` | [`public-declassification.md`](research/public-declassification.md) | Open; prototype required | Direct private burn/redemption |
| `Q-SETTLE` | [`settlement-layout.md`](research/settlement-layout.md) | Open; batch-size-2 prototype required | Settlement ABI and calibrated bound |

## 8.3 Research isolation · `rule:backlog:research-isolation`

Research may begin before the consuming phase, but it must not:

- add target-specific fields to realization;
- publish stable production ABI;
- become release evidence automatically;
- claim production capability;
- use draft defaults as calibration;
- label model wrappers as independent observers;
- introduce solver output as semantic identity;
- bypass package dependency direction;
- contradict an accepted decision while remaining marked active.

Accepted results move into typed source, permanent tests, package contracts, and
a decision or ADR where appropriate.

---

# 9. Dependency map · `sec:backlog:dependencies`

## 9.1 Current and candidate dependencies · `tbl:backlog:dependency-map`

| Dependency | Status | Intended role | Consumers |
|---|---|---|---|
| `petgraph = 0.8.3` | adopted; review record closure pending | graph storage and standard graph algorithms | labels, realization |
| `num-bigint` | existing | exact arbitrary-size integers | model; future exact analysis |
| `num-integer` | existing | exact integer helpers | model; future exact analysis |
| `num-traits` | existing | numeric traits | model; future exact analysis |
| `num-rational` | not adopted | exact rational coefficients/certificates | future concrete analysis |
| `faer` | not adopted | numerical diagnostics and certified analysis | future concrete numerical consumer |
| `fixedbitset` | deferred | dense local coverage/case sets | compiler/linker if measured |
| `elements` | Phase-3 review | target transaction and consensus types | target-elements, transaction |
| `elements-miniscript` | conditional | standard target program/control support | tapscript |
| `secp256k1-zkp` | prototype-gated | CT commitments and proofs | transaction/target research |
| SAT/LP/MILP solver | deferred | large exact planning problems | compiler/linker |
| `salsa` | deferred | incremental compiler queries | compiler |
| `egg` | deferred | equality saturation | compiler |
| `rayon` | no direct semantic authorization | parallel independent work only | future measured use |

## 9.2 Existing dependencies to reuse · `tbl:backlog:dependency-existing`

| Dependency | Use |
|---|---|
| `serde` | first-party DTOs and external envelopes |
| `serde_json` | deterministic reports/publications |
| `sha2` | project semantic and artifact identities |
| `thiserror` | typed library failures |
| `anyhow` | binary/orchestration boundaries |
| `proptest` | generated algorithm/model tests |
| `tempfile` | collision-safe staging and hermetic fixtures |
| `toml` | architecture publication encoding |
| `clap` | typed command interfaces |
| `tracing` | JSON runtime diagnostics |
| `python3` | planning structure and portable integration helpers |

## 9.3 Deferred-dependency rule · `rule:backlog:deferred-dependencies`

A deferred dependency enters only after:

1. a concrete consumer exists;
2. current code demonstrates the missing functionality;
3. simpler exact first-party code is insufficient;
4. purpose, license, maintenance, MSRV, unsafe boundary, transitive graph,
   determinism, and advisories are reviewed;
5. public API leakage is considered;
6. focused tests and an independent oracle exist;
7. the lockfile update is reviewed;
8. all gates remain green and clean.

Unused dependencies are not added to advertise intent.

---

# 10. Verification matrix · `sec:backlog:verification`

## 10.1 Rust lanes

Run under declared MSRV and current stable:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release --locked
scripts/ci.sh
```

## 10.2 Focused remediation lanes

```sh
cargo test --locked -p tripod-model
cargo test --locked -p tripod-realization
cargo test --locked -p tripod-architecture
cargo test --locked -p tripod-labels
cargo test --locked -p tripod-document-stamps
cargo test --locked -p cli-common
```

## 10.3 Dependency lanes

For each new or changed dependency:

```sh
cargo tree --locked -p <package>
cargo tree --locked -p <package> -e features
cargo tree --locked -i <package>
git diff -- Cargo.toml Cargo.lock packages/*/Cargo.toml
```

When installed:

```sh
cargo audit
```

A missing advisory tool is reported as skipped, not passed.

## 10.4 Meson and documents

```sh
meson compile -C build
meson test -C build --print-errorlogs
scripts/check-document-reproducibility.sh
```

The Meson contract must separately establish:

- byte reproducibility;
- no-op non-execution;
- output repair;
- failure propagation;
- no undeclared helper dependency.

## 10.5 Documentation and census

```sh
scripts/check-plans.sh
git diff --check
```

Every new tracked Rust or documentation subject joins its nearest
`meson.build` census.

## 10.6 Clean tree

The final gate checks all three classes:

```text
unstaged tracked changes
staged tracked changes
untracked nonignored files
```

Ignored build products are permitted.

## 10.7 Mathematical and graph algorithm lane

The future algorithm gate includes:

```text
canonical direct-Petgraph insertion permutations
topological oracle
SCC mutual-reachability oracle
declassification closure oracle
Bareiss versus rational elimination
numerical versus exact well-conditioned solutions
rank-deficient and ill-conditioned rejection
proof search versus exhaustive oracle
placement versus exhaustive carrier oracle
relocation versus independently constructed structured output
package-merge versus exhaustive small-tree oracle
calibration enumeration and monotonicity checks
no graph, matrix, or solver local handle in semantic identity
```

---

# 11. Backlog hygiene · `sec:backlog:hygiene`

## 11.1 Adding a task · `rule:backlog:add`

A new task states:

- owner phase or preparation lane;
- status;
- dependencies;
- concrete output;
- assurance class;
- focused exit check;
- affected package/files;
- verification command;
- identity impact;
- dependency impact.

## 11.2 Splitting a task · `rule:backlog:split`

Split a task when it:

- crosses semantic, compiler, linker, target, transaction, evidence, or release
  boundaries;
- has an independently reviewable security consequence;
- mixes exact correctness with numerical diagnostics;
- mixes dependency adoption with algorithm acceptance;
- contains one part that can complete while another remains research-blocked.

## 11.3 Dropping a task · `rule:backlog:drop`

A dropped task records:

- why it is unnecessary;
- supporting evidence;
- replacement;
- affected documentation;
- identity and release consequences.

## 11.4 Completed-task retention · `rule:backlog:retention`

After a phase baseline:

- compact completed prose into the phase card or release record;
- retain immutable evidence in Git history;
- keep current and immediately preparatory work here;
- do not create a second historical archive under `plans/`.

## 11.5 Review-finding closure · `rule:backlog:finding-closure`

A static-review finding closes through one of:

1. implementation and a focused regression test;
2. a typed proof that the reported state is unconstructible;
3. a documented trust-boundary change approved by the owning ADR or decision.

“Existing tests pass” is not sufficient unless a named test reaches the
reported path.

---

# 12. Current execution order · `sec:backlog:order`

Execute in this order unless new evidence changes dependencies:

```text
1. Reproduce and close F1-001 through F1-011.
2. Correct documentation and planning drift in F1-012 through F1-016.
3. Run F1-017 and record complete Phase-1 evidence.
4. Mark Phase 1 complete only if every required gate passes.
5. Complete C1 dependency and direct-Petgraph construction review.
6. Finish exact algorithm prototypes and small-instance oracles.
7. Begin production compiler analysis only after Phase 1.
8. Keep target/backend production work behind Phase 3.
```

Dependency outline:

```text
F1-001 ───────────────┐
F1-002 ───────────────┤
F1-003 ───────────────┤
F1-004 ───────────────┤
F1-005 ───────────────┤
F1-006 ───────────────┤
F1-007 ───────────────┤
F1-008 ───────────────┤
F1-009 ───────────────┤
F1-010 ───────────────┤
F1-011 ───────────────┤
F1-012 ───────────────┤
F1-013 ───────────────┤
F1-014 ───────────────┤
F1-015 ───────────────┤
F1-016 ───────────────┤
                       └── F1-017 → R1-013 → Phase 1 exit

C1-001
  ├── C1-002 ──┐
  └── C1-003 ──┴── C1-004
                         ├── C1-005 → C1-010 → C1-011/C1-012
                         └── C1-006 → optional C1-007

Phase-2 relation and carrier vocabulary
    → C1-008/C1-009
    → C1-013
    → C1-014
    → production compiler analysis
```

---

# 13. Current gate completion · `gate:backlog:current`

Phase 1 is complete only when:

```text
R1-001 through R1-011 are complete
R1-012 is DONE or DROPPED with rationale
F1-001 through F1-016 are closed
F1-017 is DONE
R1-013 is DONE
```

The Phase-1 evidence must establish:

- exact architecture/realization pilot agreement;
- faithful representation of exact mixed canonical flows;
- one-envelope sponsor policy;
- provenance-safe indexer construction;
- cap-aware quiescence claims;
- correct stamp/restat behavior;
- coherent Git revision stamping;
- exhaustive label diagnostics;
- architecture-derived constructibility;
- accurate normative/companion prose;
- no undeclared Meson helper;
- a resolved checker-output contract;
- MSRV and stable success;
- debug and release success;
- generated and label freshness;
- document reproducibility;
- actual repository cleanliness.

The C1 preparation lane is ready for Phase 2 only when:

```text
C1-001 through C1-014 are DONE
```

Allowed deferrals:

- no external optimizer if exact pilot search is sufficient;
- no `faer` if no concrete numerical consumer exists;
- no sparse or parallel numerical path without measured need;
- synthetic linker/taptree prototypes may precede real backend artifacts;
- final linker production interfaces remain blocked on actual relocatable
  target programs.

Until the current gate passes:

- production compiler analysis remains blocked;
- target-specific fields remain forbidden in realization;
- no stable linker or transaction ABI is published;
- no floating-point result becomes semantic identity;
- no draft bound becomes deployment calibration;
- no target prototype becomes release evidence;
- no self-consistent cache is described as independent event provenance.

---

# 14. One-line backlog · `rem:backlog:one-line`

> Close the provenance, exact-flow, sponsor, restat, quiescence, census, and documentation findings; record a genuinely clean Phase-1 gate; then advance through exact, deterministic, direct-Petgraph compiler and linker algorithms with independent small-instance oracles before any production target or ABI freezes.
