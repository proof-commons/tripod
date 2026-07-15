# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 1 — typed realization foundation
> **Next gate:** Phase 2 — compiler analysis preparation
> **Authority:** Current execution queue only; normative source, accepted
> decisions, package plans, and the roadmap take precedence
> **Last status review:** 2026-07-15

This file contains the work that is ready or nearly ready to execute.

It does not contain:

- the complete long-term roadmap;
- package architecture;
- accepted-decision rationale;
- unresolved design essays;
- normative protocol requirements;
- copied source identities;
- conversational history.

Long-term sequencing is owned by [`roadmap.md`](roadmap.md). Cross-package
architecture is owned by
[`toolchain-architecture.md`](toolchain-architecture.md). Package-specific
requirements are owned by [`packages/`](packages/), and unresolved research is
owned by [`research/`](research/).

---

## 1. Using this backlog

### 1.1 Task statuses

| Status | Meaning |
|---|---|
| **TODO** | Ready to start once named dependencies are complete. |
| **IN PROGRESS** | Actively being implemented or drafted. |
| **BLOCKED** | Cannot proceed until the named dependency or research result is available. |
| **DONE** | Implemented, tested, documented, and linked to evidence. |
| **DROPPED** | No longer intended; rationale and replacement are recorded. |

A task is not `DONE` merely because source code appears to contain the intended
change. Completion requires the task's exit check to pass.

### 1.2 Task identifiers

Task prefixes indicate the owning gate:

| Prefix | Gate |
|---|---|
| `P0` | Planning-folder rewrite |
| `B0` | Baseline correctness, security, and reproducibility |
| `R1` | Typed realization foundation |
| `C1` | Compiler analysis preparation |
| `Q` | Research question or prototype dependency |

Only `P0`, `B0`, and the immediately following `R1` tasks are active in this
backlog. Later compiler/backend work belongs in the roadmap until Phase 1 is
complete.

### 1.3 Definition of done

Every completed implementation task must record:

1. implementing source files;
2. positive tests;
3. negative or mutation tests where applicable;
4. documentation or ADR updates;
5. verification command;
6. confirmation that tests/checkers did not modify tracked files.

Every completed planning task must record:

1. new authoritative document;
2. replaced or deleted documents;
3. link/index validation;
4. confirmation that no normative or generated identity changed.

---

# 2. Completed gate: P0 — planning-folder rewrite

> **Gate status:** DONE (2026-07-15)
> **Blocks:** Phase-0 baseline completion and the start of substantial
> `realization` implementation

The planning rewrite is a blank-page information-architecture rewrite. Existing
long-form plan files are not edited into place; valid conclusions are rewritten
into focused documents, after which superseded files are removed.

## 2.1 Planning task summary

| ID | Status | Task | Depends on | Output |
|---|---|---|---|---|
| `P0-001` | **DONE** | Rewrite the plan index and governance rules. | none | `plans/README.md` |
| `P0-002` | **DONE** | Write the cross-package toolchain architecture. | `P0-001` | `plans/toolchain-architecture.md` |
| `P0-003` | **DONE** | Write the gate-driven roadmap. | `P0-001`, `P0-002` | `plans/roadmap.md` |
| `P0-004` | **DONE** | Rewrite the current execution backlog. | `P0-003` | `plans/backlog.md` |
| `P0-005` | **DONE** | Write accepted implementation decision records. | `P0-002` | `plans/decisions/*.md` |
| `P0-006` | **DONE** | Write all future package plans. | `P0-002`, `P0-005` | `plans/packages/*.md` |
| `P0-007` | **DONE** | Write prototype-driven research notes. | `P0-002`, `P0-005` | `plans/research/*.md` |
| `P0-008` | **DONE** | Replace the copied opcode plan with a compatibility reference. | none | `plans/reference/elements-tapscript.md` |
| `P0-009` | **DONE** | Remove superseded conversational plans. | `P0-005`–`P0-008` | old files deleted |
| `P0-010` | **DONE** | Validate the rewritten plan tree. | `P0-009` | link/census/style checks |

---

## P0-001 — Rewrite the plan index

> **Status:** DONE
> **Evidence:** commit db9a930; plans/README.md is the concise index; scripts/check-plans.sh census green

### Scope

Create a concise `plans/README.md` that owns:

- planning authority and non-normative status;
- current phase;
- plan document classes;
- exhaustive active-document index;
- status vocabulary;
- conflict and supersession rules;
- update procedure;
- immediate navigation.

### Exit check

- [ ] `plans/README.md` is a concise index rather than a monolithic plan.
- [ ] Every intended document in the new tree appears in the index.
- [ ] Plans are explicitly excluded as protocol/compiler inputs.
- [ ] The next phase and current gate are clear.
- [ ] No copied architecture hashes or stale source identities appear.

### Evidence

Record the final document path and Markdown-link check when complete.

---

## P0-002 — Write the toolchain architecture

> **Status:** DONE
> **Evidence:** commit db9a930; plans/toolchain-architecture.md
> **Depends on:** `P0-001`

### Scope

Create `plans/toolchain-architecture.md` as the single planning home for:

- typed data flow;
- package responsibilities;
- dependency direction;
- calibration orchestration;
- identity/hash ownership;
- generated-artifact direction;
- assurance boundaries;
- model/compiler separation;
- target/backend boundary;
- representation policy;
- permissionless constructibility;
- canonical layouts;
- object constructors;
- deterministic builds;
- fail-closed package behavior.

### Exit check

- [ ] Every planned package has one cross-package responsibility.
- [ ] Compiler semantics never depend on generated files or source scraping.
- [ ] Model and compiler roles remain independent.
- [ ] The linker/transaction/calibration data flow is acyclic.
- [ ] Architecture, realization, compiler, target, bundle, ABI, evidence, and
      deployment-profile identities remain distinct.
- [ ] Target-specific assumptions do not appear above the backend boundary.

---

## P0-003 — Write the roadmap

> **Status:** DONE
> **Evidence:** commit db9a930; plans/roadmap.md
> **Depends on:** `P0-001`, `P0-002`

### Scope

Create `plans/roadmap.md` with:

- Phase 0 baseline hardening;
- Phase 1 realization pilots;
- Phase 2 compiler analysis;
- Phase 3 target and foundational prototypes;
- Phase 4 end-to-end `compact-ash`;
- Phase 5 live receipt transfer;
- Phase 6 STATE and maturity announcement;
- Phase 7 burn/ASH/clear;
- Phase 8 arithmetic and redemption;
- Phase 9 requests/admission;
- Phase 10 settlement;
- Phase 11 cycle;
- Phase 12 evidence/release.

### Exit check

- [ ] Every phase has purpose, deliverables, dependencies, and an exit gate.
- [ ] Research prototypes do not silently freeze package APIs.
- [ ] `compact-ash` is the first complete backend operation.
- [ ] settlement requires a batch-size-2 prototype.
- [ ] cycle is implemented after its dependent seams.
- [ ] independent deployment evidence is kept separate from model evidence.

---

## P0-004 — Rewrite the current backlog

> **Status:** DONE
> **Evidence:** commits a7e99fc, db9a930; this file
> **Depends on:** `P0-003`

### Scope

Replace the historical maintenance report with this dependency-ordered task
queue.

### Exit check

- [ ] Every current review finding has a task.
- [ ] Every task has an output and objective exit check.
- [ ] Long-term package design remains outside the backlog.
- [ ] The start of Phase 1 is represented without prematurely adding backend
      tasks.
- [ ] Completed tasks can link to implementation and verification evidence.

---

## P0-005 — Write accepted decision records

> **Status:** DONE
> **Evidence:** commit db9a930; plans/decisions/001..006 + README
> **Depends on:** `P0-002`

Create:

```text
plans/decisions/
├── README.md
├── 001-typed-rust-source.md
├── 002-realization-layer.md
├── 003-tapscript-first.md
├── 004-translation-validation.md
├── 005-value-representation.md
└── 006-transaction-abi.md
```

Each record must state:

- status;
- context;
- accepted implementation decision;
- consequences;
- alternatives;
- evidence or prototype dependency;
- supersession conditions;
- relevant normative constraints.

### Exit check

- [ ] All six records use one template.
- [ ] They describe implementation direction, not protocol authority.
- [ ] Package plans can cite them instead of repeating their arguments.
- [ ] Unresolved technical content remains in research notes rather than being
      misclassified as accepted.

---

## P0-006 — Write package plans

> **Status:** DONE
> **Evidence:** commit db9a930; nine plans under plans/packages/
> **Depends on:** `P0-002`, `P0-005`

Create:

```text
plans/packages/
├── realization.md
├── compiler.md
├── target-elements.md
├── tapscript.md
├── simplicity.md
├── linker.md
├── transaction.md
├── vectors.md
└── release.md
```

Every package plan must state:

- purpose;
- normative typed inputs;
- forbidden inputs;
- typed outputs;
- dependency direction;
- public API/trust boundary;
- determinism and identity;
- responsibilities;
- evidence obligations;
- generated artifacts;
- non-goals;
- milestones;
- exit criteria;
- open research dependencies.

### Exit check

- [ ] All nine package plans exist.
- [ ] API sketches are explicitly provisional.
- [ ] No package consumes a generated publication as semantic input.
- [ ] `simplicity.md` is clearly parked.
- [ ] `realization.md` is marked as the next substantial package.
- [ ] Package responsibilities do not overlap ambiguously.

---

## P0-007 — Write research notes

> **Status:** DONE
> **Evidence:** commit db9a930; four notes + register under plans/research/
> **Depends on:** `P0-002`, `P0-005`

Create:

```text
plans/research/
├── README.md
├── state-constructor.md
├── wide-arithmetic.md
├── public-declassification.md
└── settlement-layout.md
```

Each research note must name:

- the unresolved question;
- blocked packages/phases;
- existing constraints;
- candidate approaches;
- required prototype;
- measurements;
- acceptance/rejection criteria;
- expected decision output.

### Exit check

- [ ] No unresolved constructor/arithmetic/declassification/settlement design is
      hidden inside an accepted package plan.
- [ ] Each question has an executable decision path.
- [ ] Prototype results cannot silently become publication contracts.

---

## P0-008 — Replace the Elements opcode reference

> **Status:** DONE — completed as documentation work; the original
> source-pinning requirement was dropped (2026-07-15)
> **Evidence:** commit db9a930; plans/reference/elements-tapscript.md supersedes the copied survey

Replace:

```text
plans/doc/tapscript_opcodes.md
```

with:

```text
plans/reference/elements-tapscript.md
```

**Scope revision (2026-07-15):** this task originally required an exact
upstream source revision. That requirement was dropped: an exact
Elements implementation revision is not a protocol or release identity.
the attestation realization targets the tapscript capability set deployed on
Liquid mainnet and trusts consensus rather than re-auditing it; see
ADR-011, "Target substrate compatibility". The replacement reference
documents the stable Liquid tapscript interface used by the backend and
remains non-authoritative.

The reference must state:

- reference-only status;
- upstream repository (explanatory reference only);
- upstream license/provenance expectations for any copied excerpt;
- relevant opcode semantics;
- production and regtest activation status;
- resource-limit facts;
- byte-order/prefix facts;
- known test gaps;
- distinction from the future typed target compatibility contract;
- explicit statement that the compiler does not consume the Markdown.

### Exit check

- [ ] No broken repository-relative upstream links remain.
- [ ] Copied text is attributed and license-compatible.
- [ ] The reference does not present itself as deployment evidence.
- [ ] The reference does not present an implementation source pin as a
      protocol or release requirement.
- [ ] `plans/README.md` indexes the replacement.

---

## P0-009 — Remove superseded plan files

> **Status:** DONE
> **Evidence:** commit db9a930; arch.md, plan.md, compiler-linker.md, doc/tapscript_opcodes.md deleted
> **Depends on:** `P0-005`–`P0-008`

Delete after valid content has been rewritten:

```text
plans/arch.md
plans/plan.md
plans/compiler-linker.md
plans/doc/tapscript_opcodes.md
```

Remove `plans/doc/` if it becomes empty.

Do not create duplicate historical copies under `plans/history/` unless a
specific legal or review requirement is identified. Git history is the default
historical record.

### Exit check

- [ ] No conversational transcript remains under `plans/`.
- [ ] No active plan depends on a deleted document.
- [ ] No accepted decision exists only in deleted prose.
- [ ] No stale schema/hash/package-name claim remains because of copied text.

---

## P0-010 — Validate the planning tree

> **Status:** DONE
> **Evidence:** commit db9a930 + follow-up; scripts/check-plans.sh green (26 documents, census/links/hygiene), git diff --check green
> **Depends on:** `P0-009`

### Required checks

- every Markdown file under `plans/` appears in `plans/README.md`;
- every relative Markdown link resolves;
- no deleted plan is referenced;
- no placeholder URL remains;
- no old package path/name remains;
- no confidence percentage remains;
- no chat-style assistant language remains;
- no generated file is described as a first-party semantic input;
- no plan describes itself as protocol-normative;
- `git diff --check` passes.

A small plan-census/link-check script may be added if useful, but it must remain
a documentation checker rather than a semantic compiler input.

### Exit check

```sh
git diff --check
```

plus the repository's chosen Markdown link/census checker.

---

# 3. Completed gate: B0 — baseline hardening

> **Gate status:** DONE (2026-07-15)
> **Depends on:** planning tasks may proceed in parallel
> **Blocks:** recording the compiler-era baseline and beginning Phase 1

The tasks below come from static source review. Before implementation, reproduce
the behavior with focused tests where practical. If a finding proves invalid,
mark the task `DROPPED` with evidence rather than silently deleting it.

## 3.1 Baseline task summary

| ID | Status | Task | Depends on | Primary output |
|---|---|---|---|---|
| `B0-001` | **DONE** | Remove raw child-argv logging from `execwrap`. | none | safe diagnostic metadata |
| `B0-002` | **DONE** | Preserve unredirected child streams and fail on relay loss. | none | corrected stream routing |
| `B0-003` | **DONE** | Correct `execwrap` help/version/usage behavior. | none | ADR-010 subprocess contract |
| `B0-004` | **DONE** | Disable panic payload reporting by default. | none | fail-closed panic redaction |
| `B0-005` | **DONE** | Reject indexer checkpoints without exactly one genesis clear. | none | stronger event-index validation |
| `B0-006` | **DONE** | Make PDF builds reproducible. | none | source-date policy and reproducibility test |
| `B0-007` | **DONE** | Split hash verification from supported/release envelope validation. | none | explicit envelope APIs |
| `B0-008` | **DONE** | Validate architecture document metadata at release. | `B0-007` recommended | `DocumentSpec` validation |
| `B0-009` | **DONE** | Reject duplicate set-like architecture declarations. | none | duplicate validation/mutations |
| `B0-010` | **DONE** | Make artifact writes collision-safe. | none | unique temporary writes |
| `B0-011` | **DONE** | Correct independent-indexer claims in human-facing docs. | none | accurate assurance wording |
| `B0-012` | **DONE** | Reject unmatched label delimiters. | none | strict label harvesting |
| `B0-013` | **DONE** | Add the full baseline verification lane. | `B0-001`–`B0-012` | green clean checkout |
| `B0-014` | **DONE** | Record the compiler-era baseline. | `P0-010`, `B0-013` | baseline commit record |

---

## B0-001 — Remove raw child-argv logging

> **Status:** DONE
> **Evidence:** commit f09cc1e (+29a5b54 credential-URL test); `cargo test --locked -p execwrap` green
> **Primary files:**
> `packages/execwrap/src/lib.rs`,
> `packages/execwrap/tests/subprocess_contract.rs`
> **Contract:** ADR-010 redaction requirements

### Problem

`execwrap::run` currently logs the complete child command vector:

```rust
tracing::info!(
    command = ?command,
    pid = child.id(),
    "executing command",
);
```

Arguments may contain credentials, URLs with userinfo, tokens, passwords,
private keys, or other secrets. Existing redaction helpers are not applied to
this field.

### Required change

At the default log level, log only safe metadata, for example:

- executable/program name;
- argument count;
- process ID.

Do not log raw argv under `--debug` unless a future centrally enforced safe
renderer is separately designed and tested.

### Required tests

Add a subprocess test using a recognizable secret, such as:

```text
--api-token SHOULD_NOT_APPEAR
```

Require:

- process behavior remains correct;
- stdout does not contain the secret;
- no JSON stderr record contains the secret;
- the program name and argument count may still appear.

Also test a credential-bearing URL.

### Exit check

- [ ] no raw command vector is logged;
- [ ] default and debug modes do not leak test secrets;
- [ ] ADR-010 documentation and code agree;
- [ ] all `execwrap` tests pass.

Verification:

```sh
cargo test --locked -p execwrap
```

---

## B0-002 — Preserve unredirected streams and fail on relay loss

> **Status:** DONE
> **Evidence:** commits f09cc1e, 29a5b54; subprocess tests cover bare/partial redirection, pipes, /dev/full data loss
> **Primary files:**
> `packages/execwrap/src/lib.rs`,
> `packages/execwrap/tests/subprocess_contract.rs`

### Problem

A child stream with no file subscriber is currently relayed only if the parent
stream was a TTY. Under `Command::output()`, CI capture, or a shell pipeline,
unsubscribed output may be discarded while the wrapper reports success.

Parent relay errors and reader-thread panics are also not fully reflected in
`data_loss`.

### Required change

Unless the user explicitly requests discard behavior:

- every unsubscribed child stdout stream is forwarded to parent stdout;
- every unsubscribed child stderr stream is forwarded to parent stderr;
- relay write/flush failure marks data loss;
- reader-thread panic marks wrapper failure/data loss;
- a successful child cannot produce a successful wrapper result after output
  loss.

Do not introduce silent discard as an implicit TTY policy. If discard is ever
needed, make it explicit and separately documented.

### Required tests

End-to-end subprocess tests for:

1. no redirection under `Command::output()` preserves stdout;
2. no redirection preserves stderr;
3. redirect stdout only while stderr remains forwarded;
4. redirect stderr only while stdout remains forwarded;
5. pipeline/non-TTY forwarding;
6. parent write failure, if a hermetic fixture can simulate it;
7. reader failure/panic policy;
8. `/dev/full` or equivalent data-loss failure.

### Exit check

```sh
cargo test --locked -p execwrap
```

All pass-through behavior must be documented in the binary contract.

---

## B0-003 — Correct `execwrap` help, version, and usage behavior

> **Status:** DONE
> **Evidence:** commits f09cc1e, 29a5b54; full help/version/usage/exit-class subprocess matrix green
> **Primary file:** `packages/execwrap/src/bin/execwrap.rs`
> **Contract:** ADR-010

### Problem

The binary checks for the `--` child-command separator before clap handles help
or version. As a result:

```text
execwrap --help
execwrap --version
```

can be reported as missing-separator usage failures.

Ambiguous routing is also an invalid-argument combination but currently follows
a runtime failure path.

### Required change

- install the panic hook at the earliest safe point;
- allow clap help/version control paths before separator validation;
- preserve the required `--` rule for real child execution;
- return usage code 2 for invalid argument combinations;
- perform routing semantic preflight before opening files or spawning a child;
- keep stdout empty for help/version/usage diagnostics;
- emit one or more valid JSON stderr records only.

### Required subprocess matrix

```text
execwrap --help
    exit 0
    JSON help on stderr
    empty stdout

execwrap --version
    exit 0
    JSON version on stderr
    empty stdout

execwrap --no-such-flag
    exit 2
    JSON usage error
    empty stdout

execwrap true
    exit 2
    missing-separator usage error

execwrap -- true
    exit 0

execwrap --redirect x --redirect-output x -- true
    exit 2
    invalid-argument diagnostic
```

### Exit check

```sh
cargo test --locked -p execwrap
```

---

## B0-004 — Disable panic payload reporting by default

> **Status:** DONE
> **Evidence:** commit f09cc1e; gate defaults disabled, `cargo test --locked -p cli-common` green
> **Primary files:**
> `packages/cli-common/src/lib.rs`,
> `packages/cli-common/src/tests/mod.rs`
> **Contract:** ADR-010

### Problem

The global panic-payload gate currently starts enabled, so a panic before
successful argument parsing may expose a secret payload without `--debug`.

### Required change

Default the global gate to disabled.

After successful argument parsing, set it from the parsed debug flag.

A panic before debug status is known must omit its payload.

If early debug-payload support is later desired, it requires a separately
reviewed pre-parse mechanism that cannot mistake a child argument or arbitrary
value for the wrapper's debug flag.

### Required tests

- default state omits payload;
- non-debug parsed state omits payload;
- debug state includes string payload;
- non-string payload remains omitted;
- JSON panic record remains valid;
- no Rust default panic text appears in binary subprocess tests.

### Exit check

```sh
cargo test --locked -p cli-common
```

and relevant binary subprocess tests.

---

## B0-005 — Require exactly one genesis clear in indexer checkpoints

> **Status:** DONE
> **Evidence:** commits 2284df0, 29a5b54; all nine rejection mutations plus round-trip green
> **Primary files:**
> `packages/model/src/ledger.rs`,
> `packages/model/src/tests/indexer_order_tests.rs`

### Problem

An empty burn map, clear map, and event list can satisfy the current census and
be accepted as a `ReferenceIndexer` checkpoint even though the event model
requires genesis clearing first.

### Required change

`validate_event_index` must require:

- a nonempty event list;
- the first event is `GenesisClear`;
- the clear ID is the genesis variant;
- exactly one genesis clear exists in the clear map;
- exactly one genesis-clear event exists;
- its payload order matches the event order;
- every other event follows it in strict canonical order.

### Required mutation tests

Reject:

1. empty maps and events;
2. clear map without genesis;
3. event list without genesis;
4. ordinary clear as first event;
5. burn as first event;
6. two genesis-clear events;
7. two distinct genesis-clear IDs;
8. genesis clear present in map but absent from events;
9. genesis event referencing a missing clear.

Retain positive round-trip coverage for a valid checkpoint.

### Exit check

```sh
cargo test --locked -p tripod-model
```

---

## B0-006 — Make PDF builds reproducible

> **Status:** DONE
> **Evidence:** commits 00e7d68, 29a5b54; `scripts/check-document-reproducibility.sh` green (sha256-equal PDFs)
> **Primary files:**
> `papers/attestation/main.tex`,
> `papers/attestation/sections/00_title.tex`,
> `papers/attestation/meson.build`,
> repository build documentation
> **Contract:** ADR-011

### Problem

The paper uses ambient date material:

```tex
\date{\today}
pdfdate={\today}
```

The build does not currently expose one explicit source-date input for all TeX
tools. Identical source built on different days may produce different visible
content and PDF metadata.

### Required decision

Adopt one reproducible date policy:

1. release/source date passed as an explicit typed/build input; or
2. source date derived from a pinned `SOURCE_DATE_EPOCH`.

The chosen date must govern:

- visible title date;
- PDF metadata date;
- XeLaTeX;
- Biber;
- latexmk;
- any document ID/creation timestamp behavior supported by the installed
  toolchain.

### Required implementation

- remove direct ambient-clock dependence;
- document the source-date input;
- set reproducibility environment consistently in Meson;
- preserve local developer ergonomics without weakening release builds;
- add a reproducibility script or test.

### Required test

Build the release PDF twice in separate clean build directories with the same
source-date input and compare exact bytes/hashes.

### Exit check

The exact command should be recorded after implementation, conceptually:

```sh
scripts/check-document-reproducibility.sh
```

It must succeed in the documented TeX environment.

---

## B0-007 — Split publication hash verification from envelope support validation

> **Status:** DONE
> **Evidence:** commits cd49938, 29a5b54; verify_hashes/validate_envelope/validate_release_envelope split, malformed version rejected
> **Primary files:**
> `packages/architecture/src/export.rs`,
> `packages/architecture/src/tests/export_hash_tests.rs`

### Problem

`PublishedArchitecture::validate_envelope` verifies hash algorithms and digests
but accepts arbitrary schema version, realization version, and publication
status because those fields are correctly excluded from the hashes.

Hash invisibility is not the same as support or release validity.

### Required API distinction

Provide clearly named operations for at least:

1. **hash verification**
   - supported declared hash algorithms;
   - body hash;
   - behavioural hash.

2. **supported-envelope validation**
   - supported architecture schema;
   - recognized publication status;
   - valid realization-version syntax or supported identity;
   - hash verification.

3. **release-envelope validation**
   - supported envelope;
   - final publication status;
   - expected architecture/release identity where applicable;
   - release pins.

Exact names may differ, but callers must not mistake hash verification for
complete release validation.

### Required tests

- arbitrary envelope metadata can leave hashes valid;
- unsupported schema is rejected by supported-envelope validation;
- malformed realization version is rejected;
- unknown publication status is rejected at parse or validation;
- draft is rejected by release-envelope validation;
- final expected envelope passes;
- generated JSON/TOML still equal the typed expected value exactly.

### Exit check

```sh
cargo test --locked -p tripod-architecture
cargo test --locked -p tripod-artifacts
```

Update callers and documentation to use the correct validation level.

---

## B0-008 — Validate architecture document metadata

> **Status:** DONE
> **Evidence:** commit cd49938; per-field DocumentSpec mutations incl. zero anchor pin
> **Primary files:**
> `packages/architecture/src/validate.rs`,
> architecture validation tests
> **Depends on:** `B0-007` recommended

### Problem

Architecture release validation checks final status and presence of an anchor
hash but does not comprehensively validate `DocumentSpec`.

### Required validation

Define explicit rules for:

- supported nonzero architecture schema;
- nonblank realization version;
- valid realization-version form;
- nonblank Layer-0 version;
- nonblank target network;
- release anchor-set hash policy;
- all-zero anchor hash;
- expected target-network constraints, if this crate is specifically bound to
  Liquid;
- publication status.

Distinguish:

- draft-valid metadata;
- supported publication metadata;
- final release metadata.

### Required mutation tests

Mutate each field independently and require the intended error:

- schema 0;
- unsupported schema;
- blank realization version;
- malformed realization version;
- blank Layer-0 version;
- blank target network;
- absent release pin;
- zero release pin;
- draft release status.

### Exit check

```sh
cargo test --locked -p tripod-architecture
```

---

## B0-009 — Reject duplicate set-like declarations

> **Status:** DONE
> **Evidence:** commits cd49938, 29a5b54; per-collection-class duplicate mutations, artifacts/hashes unchanged
> **Primary files:**
> `packages/architecture/src/validate.rs`,
> `packages/architecture/src/export.rs`,
> mutation tests

### Problem

Several export fields are sorted and deduplicated. Validation does not
consistently reject duplicate declarations before that normalization.

A duplicate in normative typed source may therefore disappear from publication
and hashes.

### Required change

Reject duplicates in every normative collection treated semantically as a set.

At minimum inspect:

- operation open flows;
- operation reads;
- operation writes;
- operation witnesses;
- operation value-flow classes;
- operation bounds;
- object allocators;
- object mutators;
- object deallocators;
- object witnesses;
- quantity reads;
- quantity readers;
- quantity writers;
- asset destruction operations;
- decision rationale, if declared set-like;
- root uses;
- projections;
- issuances;
- canonical-delta signatures;
- data-output-family signatures.

Do not deduplicate silently unless duplicate-insensitivity is itself an explicit
normative rule.

### Required tests

Add one mutation per collection class proving duplicate rejection.

Where ordering is presentation-only, continue canonical sorting after
validation.

### Exit check

```sh
cargo test --locked -p tripod-architecture
cargo run --locked -p tripod-artifacts --bin check-generated > /dev/null
git diff --exit-code
```

Any intentional change to canonical artifact bytes or architecture hashes must
undergo separate normative review rather than being hidden in this task.

---

## B0-010 — Make generated-artifact writes collision-safe

> **Status:** DONE
> **Evidence:** commits 312a563, 29a5b54; unique staged tempfiles, concurrency/failure/no-leftover tests green
> **Primary file:** `packages/artifacts/src/lib.rs`

### Problem

Temporary output paths are derived with:

```rust
path.with_extension("tmp")
```

Files such as:

```text
architecture.json
architecture.toml
```

both map to:

```text
architecture.tmp
```

Concurrent generators can race or exchange bytes.

### Required change

Use unique temporary files in the destination directory, then atomically
persist/rename.

Preferred implementation:

- `tempfile::NamedTempFile` or equivalent;
- destination-directory staging;
- cleanup on failure;
- no predictable shared temporary path;
- target replacement only after complete write and flush.

If the generated directory is treated as one release unit, consider staging the
complete set and validating it before replacement. That larger transaction is
optional for this task; per-file collision safety is mandatory.

### Required tests

- concurrent writes to different same-stem artifacts do not collide;
- failure leaves the original target intact;
- no temporary file remains after success;
- expected bytes remain exact;
- check path remains non-writing.

### Exit check

```sh
cargo test --locked -p tripod-artifacts
```

---

## B0-011 — Correct independent-indexer claims

> **Status:** DONE
> **Evidence:** commit 312a563; human.md wording matches the realization deployment requirement
> **Primary files:**
> `papers/attestation/human.md`,
> any related README/plan wording

### Problem

Human-facing prose currently implies the repository already runs two separately
implemented indexers and compares them. The source contains a strong
differential API and mismatch-detection tests, but deployment-grade independent
implementations remain future evidence.

### Required change

State precisely:

- the model provides separate event and query comparison boundaries;
- current tests prove those comparisons detect representative mismatches;
- deployment release requires a separately implemented candidate indexer;
- another `ReferenceIndexer` invocation is not independent evidence;
- receipt-accounting audit remains a third separate claim.

Do not weaken the intended deployment requirement.

### Exit check

- [ ] no document claims independent deployment implementation already exists
      unless one is actually present;
- [ ] realization and human-facing wording agree;
- [ ] generated/paper builds remain green.

---

## B0-012 — Reject unmatched label delimiters

> **Status:** DONE
> **Evidence:** commits 312a563, 29a5b54; scan_labels + document harvester strictness tests green
> **Primary files:**
> `packages/model/src/artifacts.rs`,
> model label tests,
> architecture document label harvesters as applicable

### Problem

Label harvesters split lines on acute accents or backticks and may ignore an
unclosed token instead of reporting malformed source.

### Required change

For each controlled label delimiter:

- require balanced delimiters on every relevant line;
- reject an unmatched opening or closing delimiter;
- preserve existing label shape/type checks;
- avoid mistaking unrelated Markdown structures for model labels;
- report file and line where possible.

### Required tests

- unmatched acute opening;
- unmatched acute closing or odd count;
- malformed label inside balanced delimiters;
- valid multiple labels on one line;
- unrelated text remains ignored as intended;
- relevant document backtick structures remain correctly classified.

### Exit check

```sh
cargo test --locked -p tripod-model
cargo test --locked -p tripod-architecture
```

---

## B0-013 — Run and preserve the complete baseline gate

> **Status:** DONE
> **Evidence:** see evidence record below
> **Depends on:** `B0-001`–`B0-012`

### Required Rust lanes

```sh
scripts/ci.sh
```

Run on:

- declared Rust MSRV;
- current stable Rust.

### Required document lanes

```sh
meson setup build
meson compile -C build attestation
meson test -C build --print-errorlogs
git diff --exit-code
```

Also run the new PDF reproducibility check from `B0-006`.

### Required evidence

Record:

- toolchain versions;
- whether `cargo audit` ran or was skipped;
- Meson/Ninja/TeX versions;
- successful generated-artifact report;
- successful clean-tree check;
- successful reproducibility hashes.

A missing optional advisory tool remains a loudly recorded skip under ADR-011,
not a silent pass. Release policy may later require the lane to be installed.

### Evidence record (2026-07-15, commit 29a5b54)

- Rust lanes green on three toolchains: nightly 1.99.0 (2026-07-07),
  MSRV rustc 1.88.0, stable rustc 1.97.0 — fmt, clippy `-D warnings`,
  and all 19 workspace test suites each (`claude-rust` toolbox,
  per-toolchain `CARGO_TARGET_DIR`).
- `cargo audit`: SKIPPED (not installed) — recorded loudly per ADR-011.
- Document lane green: meson 1.9.2, ninja 1.13.2, XeTeX
  3.141592653-2.6-0.999998 (TeX Live 2026 Flatpak), biber 2.21;
  4/4 meson tests including the `check-generated` gate.
- `check-generated` report: all four artifacts current, no strays.
- Reproducibility: `scripts/check-document-reproducibility.sh` green —
  both clean builds render sha256
 .
- Clean tree: `git diff --exit-code -- . ':(exclude)plans'` green
  (plans/ carried the in-progress P0 rewrite).

### Exit check

Every required lane passes without modifying tracked files.

---

## B0-014 — Record the compiler-era baseline

> **Status:** DONE
> **Depends on:** `P0-010`, `B0-013`

### Required output

Record one clean commit as the baseline for Phase 1.

The baseline record must contain or link to:

- commit ID;
- architecture schema;
- realization version;
- architecture semantic hash;
- architecture behavioural hash;
- Layer-0 version and anchor-set hash;
- target-independent package graph status;
- verification matrix result;
- document reproducibility result.

Do not copy these identities into every plan. Keep one baseline record and link
to canonical generated artifacts.

### Baseline record (2026-07-15)

- **Identities:** architecture schema 17; realization version tracked compiler-line binding
  (letter 13a); publication status final; semantic hash
 
  (`sha256-canonical-json-v2`); behavioural hash
 
  (`sha256-canonical-json-behavioural-v2`); Layer-0 v0.5.0, anchor-set
  hash
 .
  Canonical artifacts: `packages/model/generated/architecture.{json,toml}`.
- **Package graph:** architecture, model, artifacts, cli-common,
  execwrap, flatten-latex-main. No realization/compiler/backend crates
  exist yet (Phase-1 scope).
- **Verification matrix:** see the B0-013 evidence record (nightly
  1.99.0 / MSRV 1.88.0 / stable 1.97.0 Rust lanes, meson document
  lane, check-generated, plan-tree checker) — re-run green at the
  baseline commit.
- **Document reproducibility:**
  `scripts/check-document-reproducibility.sh` green; both clean builds
  render sha256
 .

### Exit check

- [ ] repository is clean;
- [ ] all Phase-0 gates pass;
- [ ] baseline commit is immutable and identifiable;
- [ ] `roadmap.md` marks Phase 0 complete;
- [ ] `backlog.md` promotes the Phase-1 tasks to current work.

---

# 4. Current gate: R1 — typed realization foundation

> **Gate status:** CURRENT (Phase-0 baseline recorded)
> **Depends on:** `B0-014` (DONE)
> **Primary plan:** [`packages/realization.md`](packages/realization.md)

The Phase-0 baseline is recorded: these tasks are current work.

## 4.1 Phase-1 task summary

| ID | Status | Task | Depends on | Output |
|---|---|---|---|---|
| `R1-001` | **TODO** | Create the realization crate skeleton and package contract. | `B0-014`, package plan | workspace crate |
| `R1-002` | **TODO** | Define identity ownership and deterministic semantic IDs. | `R1-001` | fact/expression/relation IDs |
| `R1-003` | **TODO** | Define typed domains and the expression arena. | `R1-002` | typed expression graph |
| `R1-004` | **TODO** | Define the minimum semantic relation vocabulary. | `R1-003` | relation types |
| `R1-005` | **TODO** | Define operation, constructibility, lifecycle, and representation declarations. | `R1-004` | operation schema |
| `R1-006` | **TODO** | Implement `derive(&Architecture)`. | `R1-005` | deterministic `RealizationSpec` |
| `R1-007` | **TODO** | Declare `compact-ash`. | `R1-006` | first pilot |
| `R1-008` | **TODO** | Declare `transfer-live-receipts`. | `R1-006` | second pilot |
| `R1-009` | **TODO** | Derive declassification for both pilots. | `R1-007`, `R1-008` | typed disclosure result |
| `R1-010` | **TODO** | Add architecture/realization bidirectional validation. | `R1-007`, `R1-008` | validation suite |
| `R1-011` | **TODO** | Add model-conformance tests for both pilots. | `R1-009`, `R1-010` | conformance evidence |
| `R1-012` | **TODO** | Add deterministic derivative publication only if needed. | `R1-009` | optional generated artifact |
| `R1-013` | **TODO** | Run and record the Phase-1 exit gate. | `R1-001`–`R1-012` | green Phase 1 |

`TODO` here follows the status vocabulary: ready to start once the named
preceding `R1` dependencies are complete.

---

## R1-001 — Create the realization crate skeleton

> **Status:** TODO
> **Depends on:** `B0-014`,
> [`packages/realization.md`](packages/realization.md)

### Required package metadata

```text
directory: packages/realization
Cargo package: tripod-realization
Rust library name: realization
```

Inherit:

- authors;
- edition;
- license;
- publish status;
- Rust version;
- workspace version;
- workspace lints.

Initial dependency:

```text
architecture
```

Avoid dependencies on:

- model;
- compiler;
- target-elements;
- tapscript;
- linker;
- generated publication parsers.

### Required crate-level documentation

State:

- purpose;
- normative typed input;
- forbidden inputs;
- target independence;
- deterministic derivation;
- relation to model conformance;
- relation to compiler analysis;
- current provisional status.

### Exit check

```sh
cargo check --locked -p tripod-realization
cargo clippy --locked -p tripod-realization --all-targets -- -D warnings
```

---

## R1-002 — Define semantic identity ownership

> **Status:** TODO
> **Depends on:** `R1-001`

### Required decisions

Define which identifiers remain owned by `architecture` and which are introduced
by `realization`.

Likely ownership:

```text
architecture:
    AssetId
    RootId
    ObjectId
    OperationId
    QuantityId
    WitnessId
    ClauseId
    BoundId
    TagId
    stable architecture discriminants

realization:
    FactId
    ExprId
    RelationId
    ObservableId
    LifecycleRequirementId
    ProofAlternativeId
```

Compiler-lowered node/program IDs remain future compiler ownership.

### Requirements

- IDs are deterministic;
- IDs do not depend on declaration insertion order;
- structural identifiers use explicit domain separation;
- collisions are handled fail-closed;
- IDs can carry source provenance;
- no target opcode participates in semantic identity.

### Exit check

Focused tests prove:

- same typed declaration → same IDs;
- source-order permutation → same structural IDs where order is semantic-set
  order;
- semantic mutation → changed affected IDs;
- unrelated presentation mutation → unchanged semantic IDs where intended.

---

## R1-003 — Define typed domains and expression arena

> **Status:** TODO
> **Depends on:** `R1-002`

### Required domains

At minimum distinguish:

- protocol amount;
- count/cardinality;
- cycle;
- block age;
- boolean;
- ratio/configuration constant;
- asset identity;
- object/root identity;
- owner/address identity;
- output/input reference.

Do not use one untyped integer domain for all semantic values.

### Required expression behavior

- checked addition/subtraction;
- multiplication with explicit bounds;
- exact floor operation;
- equality and ordering;
- conditional activation;
- bounded family sums;
- typed constants;
- fact references;
- deterministic structural identity;
- dependency traversal;
- reference evaluation in tests.

### Exit check

- ill-typed expressions are unrepresentable or rejected;
- overflow/domain semantics are explicit;
- dependency traversal is deterministic;
- reference evaluation matches current model helper arithmetic on boundary
  fixtures.

---

## R1-004 — Define the minimum relation vocabulary

> **Status:** TODO
> **Depends on:** `R1-003`

The initial vocabulary must be sufficient for the two pilots without encoding
all future target details.

Expected relation families include:

- domain;
- cardinality;
- object recognition;
- authorization;
- equality;
- conservation;
- recipient pin;
- object/class closure;
- open-flow isolation;
- state assignment;
- root succession;
- event projection;
- constructibility;
- lifecycle.

The final enum structure is intentionally not predetermined by this backlog.

### Exit check

- both pilot operations can be expressed without generic string predicates;
- every relation has typed operands and provenance;
- no relation embeds tapscript/Elements details;
- relation dependency traversal is complete.

---

## R1-005 — Define operation and representation declarations

> **Status:** TODO
> **Depends on:** `R1-004`

Define typed structures for:

- input families;
- output families;
- semantic preconditions;
- authorization;
- state effects;
- projections;
- public observables;
- constructibility;
- witness availability;
- lifecycle effects;
- representation capabilities;
- proof alternatives.

### Exit check

- permissionless versus owner-authorized construction is explicit;
- public and private witness availability cannot be conflated;
- representation latitude does not alter semantic value;
- lifecycle exits are declared;
- architecture operation ownership is exact.

---

## R1-006 — Implement deterministic architecture derivation

> **Status:** TODO
> **Depends on:** `R1-005`

Implement the planned derivation:

```rust
pub fn derive(
    architecture: &architecture::Architecture,
) -> Result<RealizationSpec, RealizationError>;
```

> Illustrative signature until the package plan is accepted.

### Required behavior

- pure;
- deterministic;
- no filesystem reads;
- no environment reads;
- no generated artifact reads;
- no model source scraping;
- complete validation;
- explicit errors for missing declarations or unresolved references.

### Exit check

Repeated derivation produces equal typed values and stable canonical debug/test
representations.

---

## R1-007 — Declare `compact-ash`

> **Status:** TODO
> **Depends on:** `R1-006`

Declare the complete semantic operation described by the roadmap and package
plan.

### Required model comparisons

- minimum ASH cardinality;
- maximum bound reference;
- ownerless `U` conservation;
- one ASH output;
- no burn projection;
- no root use;
- permissionless constructibility;
- sponsor flow separation.

### Exit check

Architecture, realization, and model agree on every declared family and
semantic relation.

---

## R1-008 — Declare `transfer-live-receipts`

> **Status:** TODO
> **Depends on:** `R1-006`

Declare:

- live-only receipt inputs and outputs;
- owner authorization;
- exact semantic value conservation;
- destination-owner freedom;
- sponsor separation;
- representation alternatives;
- no root use.

### Exit check

Architecture, realization, and model agree on every declared family and
semantic relation.

---

## R1-009 — Derive pilot declassification

> **Status:** TODO
> **Depends on:** `R1-007`, `R1-008`

Derive disclosure facts from operation expression/relation dependencies.

Do not author a parallel disclosure list.

### Exit check

- declassification is deterministic;
- every disclosed fact has dependency provenance;
- no undisclosed required fact exists;
- no target encoding decision is treated as an abstract disclosure;
- compiler-facing code consumes the typed result, not a JSON file.

---

## R1-010 — Add architecture/realization validation

> **Status:** TODO
> **Depends on:** `R1-007`, `R1-008`

Validation must be bidirectional:

- every architecture pilot operation has one realization;
- every realization operation maps to one architecture operation;
- every family, bound, flow, authorization, and projection resolves;
- no undeclared semantic object appears;
- no required architecture fact is omitted;
- no duplicate semantic relation ID exists.

### Exit check

Focused mutation tests prove omissions, duplicates, transpositions, and wrong
bound references fail.

---

## R1-011 — Add model-conformance tests

> **Status:** TODO
> **Depends on:** `R1-009`, `R1-010`

For each pilot:

- construct valid worlds/transitions through the model;
- project relevant semantic facts;
- evaluate declared relations;
- compare declared event/certificate effects;
- mutate one relation at a time and require failure;
- preserve the distinction between model evidence and future backend evidence.

### Exit check

Positive and negative conformance tests pass in debug and release profiles.

---

## R1-012 — Add derivative publication only if required

> **Status:** TODO
> **Depends on:** `R1-009`

A realization publication is optional during Phase 1.

If added, it must have:

- typed source;
- canonical schema;
- deterministic bytes;
- explicit generator;
- non-writing checker;
- unknown-field rejection;
- no reverse semantic dependency.

Do not add a generated artifact merely because one might be useful later.

### Exit check

If no publication is needed, mark this task `DROPPED` with rationale.

---

## R1-013 — Run the Phase-1 gate

> **Status:** TODO
> **Depends on:** all required `R1` tasks

Run:

```sh
scripts/ci.sh
```

and the Phase-1 focused tests documented in
[`roadmap.md`](roadmap.md).

### Exit check

All Phase-1 roadmap criteria pass and the checkout remains clean.

---

# 5. Blocked research register

These are not current implementation tasks. They are listed so Phase-1 work
does not accidentally absorb target-specific questions.

| ID | Status | Research note | Blocks |
|---|---|---|---|
| `Q-STATE` | **PROTOTYPE REQUIRED** | `research/state-constructor.md` | STATE backend ABI and Phase 6 |
| `Q-ARITH` | **PROTOTYPE REQUIRED** | `research/wide-arithmetic.md` | redemption, admission, settlement, cycle |
| `Q-DECLASS` | **OPEN / PROTOTYPE REQUIRED** | `research/public-declassification.md` | confidential-to-public backend paths |
| `Q-SETTLE` | **OPEN / PROTOTYPE REQUIRED** | `research/settlement-layout.md` | settlement ABI and calibrated batch size |

Research prototypes may begin before their consuming phases, but they must not
add target-specific fields to `RealizationSpec`.

---

# 6. Recently established foundation

The source snapshot already contains substantial maintenance infrastructure
that the Phase-0 gate must preserve and reverify:

- behavioural-hash v2 with calibrated draft defaults projected out;
- explicit retirement record for behavioural-hash v1;
- generated-artifact writer/checker split;
- non-writing generated checks;
- complete architecture JSON/TOML equality checks;
- realization appendix and masthead welds;
- workspace ADRs for CLI and dependency policy;
- runner-agnostic CI script;
- Cargo `--locked` build paths;
- byte-exact raw `execwrap` writer mode;
- deterministic/atomic LaTeX flattener;
- model public-boundary documentation;
- deployment-profile census tests;
- separate event/query/accounting differential APIs.

These are not marked as newly completed backlog tasks because they predate this
rewritten queue. They remain part of the baseline and must continue passing.

If the Phase-0 gate reveals that one of these claims is incomplete, add a
specific `B0` task rather than weakening the gate.

---

# 7. Backlog hygiene

## 7.1 Adding a task

A new task must state:

- one owning phase;
- one status;
- dependencies;
- concrete output;
- focused exit check;
- affected files or package;
- verification command where known.

## 7.2 Splitting a task

Split a task when:

- it has independently reviewable security consequences;
- it changes more than one assurance boundary;
- one part can complete while another remains blocked;
- it would otherwise mix normative and implementation changes.

## 7.3 Dropping a task

A dropped task must record:

- why it is no longer needed;
- evidence supporting the conclusion;
- replacement task or decision, if any;
- whether documentation was corrected.

## 7.4 Completed-task retention

Keep recently completed tasks long enough to support phase review. After a
phase baseline is recorded, compact older completed tasks into a phase record
or release note and remove them from the active queue.

The active backlog should remain reviewable in one sitting.

---

# 8. Current execution order

Unless dependencies change, execute in this order:

```text
2. Execute R1-001 through R1-013 in dependency order.
3. Do not begin production backend operation emission before Phase 2.
```

Within the baseline tasks, use this security-first order:

```text
B0-001  secret logging
B0-002  stream/data-loss behavior
B0-003  CLI control paths
B0-004  panic redaction
B0-005  indexer genesis
B0-006  PDF reproducibility
B0-007  envelope API
B0-008  document metadata
B0-009  duplicate declarations
B0-010  artifact temporary writes
B0-011  documentation accuracy
B0-012  delimiter strictness
B0-013  complete verification
B0-014  baseline record
```

---

# 9. Current gate completion

Phase 0 is complete only when:

```text
P0-001 … P0-010  DONE
B0-001 … B0-014  DONE
```

and the complete Phase-0 exit gate in [`roadmap.md`](roadmap.md) passes.

Until then:

- the repository remains in baseline-hardening mode;
- `realization` production implementation is blocked;
- compiler/backend package proliferation is deferred;
- target prototypes must remain isolated and non-normative.

---

# 10. One-line backlog

> Rewrite the planning tree; close the outstanding security, stream-integrity,
> indexer, envelope, manifest, artifact, documentation, label, and PDF
> reproducibility findings; record one clean compiler-era baseline; then build
> the typed realization vocabulary and prove it on `compact-ash` and live
> receipt transfer before any production backend work begins.
