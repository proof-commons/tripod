# Tripod Compiler Roadmap

> **Status:** ACTIVE
> **Scope:** Ordered implementation phases, dependencies, deliverables, and
> phase-exit gates
> **Current phase:** Phase 1 — typed realization foundation
> **Next substantial phase:** Phase 2 — target-independent compiler analysis
> **Authority:** Implementation sequencing only; normative source and accepted
> decisions take precedence

This document defines **when** planned work occurs and **what evidence is
required before the repository advances to the next phase**.

It does not define:

- protocol semantics;
- detailed package APIs;
- target opcode semantics;
- current task assignments;
- accepted architectural rationale.

Those concerns live in normative source,
[`toolchain-architecture.md`](toolchain-architecture.md), package plans,
decision records, research notes, and [`backlog.md`](backlog.md).

---

## 1. Roadmap principles

### 1.1 Progress is gate-driven

A phase is complete only when its exit gate passes.

A package existing in the workspace does not by itself complete a phase.
Likewise, a green unit test does not upgrade model evidence into compiler,
backend, substrate, or deployment evidence.

Each phase gate requires:

1. the declared typed deliverables;
2. focused positive and negative tests;
3. deterministic output where applicable;
4. generated-artifact freshness where applicable;
5. documentation updated to match implementation;
6. a clean working tree after checks;
7. no unresolved release-blocking findings in the completed scope.

### 1.2 Research does not silently become architecture

Prototype work may occur before its consuming implementation phase, but it
must remain isolated under the corresponding research note until its result is
accepted.

A prototype does not freeze:

- a package API;
- a stable identifier;
- a publication schema;
- a target capability;
- a transaction ABI;
- a release promise.

A successful prototype should produce either:

- an accepted decision record;
- a rejected alternative with documented evidence;
- or a revised research question.

### 1.3 The target-independent boundary comes first

Substantial backend work must not begin before the typed realization boundary
exists.

The implementation order is intentionally:

```text
architecture
    ↓
realization
    ↓
compiler analysis
    ↓
exact target and backend prototypes
    ↓
linked target programs
```

Beginning with tapscript emission would force target assumptions into semantic
types before the target-independent relation is stable.

### 1.4 The first operations are design probes

The first two typed operation declarations are:

1. `compact-ash`;
2. `transfer-live-receipts`.

They are chosen because they test different semantic and representation
requirements.

| Operation | Design pressure |
|---|---|
| `compact-ash` | Public/openable ownerless value, bounded aggregation, permissionless construction, no root state, no owner signature. |
| `transfer-live-receipts` | Owner authorization, lateral value conservation, value-representation latitude, class closure, sponsor isolation. |

The pilot declarations test the realization vocabulary before it is expanded to
all operations.

### 1.5 Hard operations arrive after their seams

The roadmap intentionally delays operations that combine several unresolved
seams.

In particular:

- STATE-spending operations wait for the state-constructor prototype;
- wide-floor operations wait for the arithmetic prototype;
- confidential-to-public transitions wait for the declassification prototype;
- settlement waits for a batch-2 layout prototype;
- cycle is implemented last because it combines nearly every major seam.

### 1.6 Evidence is developed with implementation

Relation coverage, rejecting mutations, representation metamorphisms, resource
formulas, and reproducibility checks are phase deliverables—not cleanup work
deferred until the end.

The project follows translation validation rather than assuming compiler
correctness from implementation review alone.

---

## 2. Phase overview

| Phase | Status | Primary result |
|---|---|---|
| **0** | **COMPLETE (2026-07-15)** | Rewritten planning tree and trustworthy compiler-era baseline. |
| **1** | **ACTIVE** | Typed target-independent realization with two pilot operations. |
| **2** | **PLANNED** | Target-independent compiler analysis for the pilots. |
| **3** | **PLANNED / PROTOTYPE-DRIVEN** | Typed Elements target compatibility contract and foundational backend prototypes. |
| **4** | **PLANNED** | End-to-end `compact-ash` on the selected Elements target. |
| **5** | **PLANNED** | Live-receipt transfer with value-representation evidence. |
| **6** | **PLANNED** | STATE constructor integrated through `announce-maturity`. |
| **7** | **PLANNED** | Burn, ASH compaction, and clear pipeline. |
| **8** | **PLANNED** | Wide arithmetic integrated through redemption. |
| **9** | **PLANNED** | Deposit request and admission pipeline. |
| **10** | **PLANNED / PROTOTYPE-DRIVEN** | Settlement layout validated and implemented. |
| **11** | **PLANNED** | Cycle implementation after all major seams are established. |
| **12** | **PLANNED** | Complete evidence, calibration, deployment profile, and release gate. |

Phases may contain several commits and internal milestones. Phase order may be
changed only through an explicit roadmap update that names:

- the dependency being reordered;
- the evidence showing the new order is safe;
- affected package plans;
- affected research notes;
- any decision record that must change.

---

# Phase 0 — Planning reset and baseline hardening

> **Primary documents:** [`backlog.md`](backlog.md),
> [`README.md`](README.md)
> **Substantial new semantic packages permitted:** no

## 0.1 Purpose

Establish a clean, reproducible, accurately documented baseline before adding
the realization/compiler package graph.

This phase is maintenance and planning work. It must not redesign protocol
semantics or move the architecture behavioural hash.

## 0.2 Planning-tree deliverables

The rewritten planning tree must contain:

```text
plans/
├── README.md
├── toolchain-architecture.md
├── roadmap.md
├── backlog.md
├── packages/
├── decisions/
├── research/
└── reference/
```

Required outcomes:

- every active planning file has one purpose;
- every file is listed in the index;
- accepted decisions are separated from research;
- package plans are separated from roadmap sequencing;
- the backlog contains current tasks only;
- conversational transcripts are removed;
- superseded confidence percentages are removed;
- the Elements reference is clearly a compatibility survey, not a deployment
  target or consensus-implementation pin (ADR-011);
- no plan is presented as compiler input or normative protocol source.

The old planning documents are removed only after their valid conclusions have
been rewritten into the focused structure.

## 0.3 Baseline-hardening deliverables

The current baseline findings are tracked individually in
[`backlog.md`](backlog.md). Phase 0 includes at least the following work.

### Command-line and process integrity

- stop `execwrap` from logging raw child argv;
- preserve unredirected child streams under non-TTY execution;
- treat relay/read/finalization data loss as wrapper failure;
- correct `execwrap --help`, `--version`, and usage-class behavior;
- default panic-payload reporting to disabled;
- add subprocess tests for the complete ADR-010 contract.

### Model/indexer integrity

- reject an indexer checkpoint with no genesis clear;
- reject duplicate or malformed genesis-clear structure;
- add focused event-index mutation tests.

### Publication and release validation

- separate hash verification from supported-envelope and release-envelope
  validation;
- validate `DocumentSpec` fields explicitly;
- reject duplicate normative set-like declarations rather than silently
  normalizing them;
- make generated-artifact temporary writes collision-safe.

### Reproducibility

- make the PDF/document build independent of ambient wall-clock time;
- pin or explicitly supply the source/release date;
- verify byte-identical PDF output in clean build directories;
- preserve the existing deterministic flattened-LaTeX behavior.

### Documentation accuracy

- correct any claim that separately implemented deployment indexers already
  exist when only the differential harness exists;
- reject unmatched model-label/document-label delimiters;
- update planning status to reflect completed maintenance accurately.

## 0.4 Baseline verification matrix

The canonical Rust verification command is:

```sh
scripts/ci.sh
```

It must cover:

```text
rustfmt
clippy with -D warnings
debug tests
release tests
non-writing generated-artifact check
dependency advisory lane
clean working tree
```

The document/Meson verification lane is:

```sh
meson setup build
meson compile -C build attestation
meson test -C build --print-errorlogs
git diff --exit-code
```

When reusing an existing build directory, setup may use the appropriate Meson
reconfigure command. The clean-checkout CI lane should still exercise a fresh
build directory.

The reproducibility check must build the same document in two clean build
directories with the same explicit source-date input and require identical
hashes for the release PDF.

The exact reproducibility command may be implemented as a script or test during
this phase. Once implemented, it must be documented here and in the repository
README.

## 0.5 Phase-0 non-goals

Phase 0 does not include:

- creating the complete future package graph;
- changing protocol operation semantics;
- changing stable architecture identifiers;
- changing Layer-0 semantics;
- adding target opcodes to model or architecture code;
- implementing tapscript;
- implementing a compiler IR;
- recalibrating deployment bounds;
- declaring a deployment release.

## 0.6 Phase-0 exit gate

Phase 0 is complete only when:

- [x] the new planning tree is complete and exhaustively indexed;
- [x] old conversational plan files are removed;
- [x] all high-priority baseline findings are closed;
- [x] all medium-priority release-integrity findings selected for the gate are
      closed or explicitly moved to a later phase with rationale;
- [x] `scripts/ci.sh` passes on the declared MSRV;
- [x] `scripts/ci.sh` passes on current stable Rust;
- [x] the Meson/document lane passes in a clean build directory;
- [x] generated checks do not modify tracked files;
- [x] document builds do not modify tracked files except explicit requested
      release mirrors;
- [x] two clean document builds with the same source-date input are
      byte-identical;
- [x] all generated artifacts are current;
- [x] `git diff --exit-code` passes after checks;
- [x] current architecture identities remain unchanged unless an intentional,
      separately reviewed normative change was required;
- [x] the resulting baseline commit is recorded as the starting point for
      Phase 1.

The baseline commit should be recorded in one dedicated roadmap/baseline entry
or release note, not copied into every plan document.

---

# Phase 1 — Typed realization foundation

> **Status:** ACTIVE
> **Primary package plan:**
> [`packages/realization.md`](packages/realization.md)
> **Depends on:** Phase 0
> **Target/backend code permitted:** only isolated research prototypes

## 1.1 Purpose

Introduce the typed, target-independent semantic declaration shared by model
conformance and compiler analysis.

The realization layer bridges the finite architecture and the relation detail
required by a compiler without turning the executable model into a source
language.

## 1.2 Core deliverables

Create:

```text
packages/realization/
```

with a package named:

```text
tripod-realization
```

The package must provide typed representations for at least:

- semantic fact identities;
- expression identities and typed expression arena;
- arithmetic domains;
- semantic relations;
- operation realization;
- state assignments;
- authorization relations;
- recipient relations;
- object recognition and closure;
- event projections;
- public observables;
- representation capabilities;
- constructibility requirements;
- witness availability;
- lifecycle requirements;
- proof alternatives;
- derived declassification.

The exact internal enum vocabulary remains provisional until the pilot
operations are complete.

## 1.3 Initial public entry point

The intended public derivation is:

```rust
pub fn derive(
    architecture: &architecture::Architecture,
) -> Result<RealizationSpec, RealizationError>;
```

> Illustrative API; not frozen until the Phase-1 exit gate passes.

The derivation must be:

- pure;
- deterministic;
- typed;
- independent of filesystem publications;
- independent of target opcodes;
- validated bidirectionally against architecture coverage.

## 1.4 Pilot operation A: `compact-ash`

The declaration must express:

- nonempty ASH input family;
- manifest-calibrated maximum;
- permissionless authorization;
- explicit/semantic `U` conservation;
- ownerless input and output objects;
- one ASH output;
- optional sponsor envelope as a separate open flow;
- no root state;
- no burn projection;
- transition-certificate projection;
- lifecycle reduction in ASH output count;
- public constructibility requirements;
- no private witness dependency.

The declaration must be checked against current model behavior.

## 1.5 Pilot operation B: `transfer-live-receipts`

The declaration must express:

- nonempty live-receipt input and output families;
- manifest-calibrated maxima;
- every consumed receipt owner authorizes;
- same-class closure;
- exact semantic `U` conservation;
- destination-owner freedom inside the authorized output set;
- optional sponsor envelope as a separate open flow;
- no root state;
- transition-certificate projection;
- representation latitude for value-preserving movement;
- owner-secret availability for the authorized operation.

The declaration must be checked against current model behavior.

## 1.6 Derived declassification

For both pilots, derive declassification from the typed dependency graph.

Expected semantic result:

```text
compact-ash:
    no public value disclosure beyond already-public ASH facts

transfer-live-receipts:
    no economy-determining value disclosure required by the abstract relation
```

The exact target representation remains a backend concern. The realization
declares semantic dependencies and allowed proof alternatives.

If a derivative `declassification.json` remains published, it must be rendered
from the typed result through the existing generation/check discipline. The
compiler must consume the typed in-memory value, not the JSON.

## 1.7 Model-conformance integration

Add tests demonstrating:

- every pilot operation maps to exactly one architecture operation;
- architecture and realization identifiers agree;
- declared input/output families equal the manifest families;
- relation dependencies cover the model's relevant semantic facts;
- successful model transitions satisfy declared relations;
- representative model rejections violate a declared relation;
- derived projections agree with model certificates;
- no target detail enters the realization.

The migration should be incremental. Existing operation code remains while
conformance tests are added.

## 1.8 Phase-1 exit gate

Phase 1 is complete only when:

- [ ] the `realization` crate is in the workspace;
- [ ] all crate metadata and lints follow ADR-011;
- [ ] the derivation API is pure and deterministic;
- [ ] fact, expression, and relation IDs are deterministic;
- [ ] expression domains and checked arithmetic semantics are typed;
- [ ] `compact-ash` is completely declared;
- [ ] `transfer-live-receipts` is completely declared;
- [ ] both declarations are bidirectionally welded to architecture;
- [ ] both declarations are checked against current model behavior;
- [ ] declassification is derived from typed dependencies;
- [ ] no generated file is consumed as semantic input;
- [ ] no model source file is scraped;
- [ ] no target opcode or stack concept appears in target-independent types;
- [ ] repeated derivation produces equal typed values and equal canonical bytes
      for any derivative publication;
- [ ] debug and release workspace tests pass;
- [ ] the checkout remains clean.

---

# Phase 2 — Target-independent compiler analysis

> **Status:** PLANNED
> **Primary package plan:** [`packages/compiler.md`](packages/compiler.md)
> **Depends on:** Phase 1

## 2.1 Purpose

Lower `RealizationSpec` into deterministic target-independent analysis suitable
for multiple backends.

## 2.2 Deliverables

Create:

```text
packages/compiler/
```

with a package named:

```text
tripod-compiler
```

Implement for the pilot operations:

- relation graph construction;
- structural hashing/hash-consing;
- canonical relation ordering;
- constant folding;
- proof-plan alternatives;
- disclosure dependency analysis;
- witness-availability analysis;
- permissionless constructibility check;
- representation lifecycle analysis;
- fact-source requirements;
- obligation-placement requirements;
- canonical layout requirements;
- target-capability requirements;
- source provenance for every analyzed relation.

## 2.3 Required relation contract

Every analyzed relation must carry:

- stable deterministic identifier;
- relation kind;
- typed operands;
- owning operation;
- source architecture/realization provenance;
- expression dependencies;
- proof alternatives;
- disclosure dependencies;
- witness-availability requirements;
- candidate carrying location;
- target-capability requirements;
- vector obligations.

No relation may exist only as prose or an untyped backend convention.

## 2.4 Initial analysis outputs

The exact output type is provisional, but it should conceptually contain:

```rust
pub struct AnalyzedProgram {
    pub architecture_identity: ArchitectureIdentity,
    pub realization_identity: RealizationIdentity,
    pub relation_graph: RelationGraph,
    pub proof_plan: ProofPlan,
    pub disclosure_plan: DisclosurePlan,
    pub lifecycle_paths: Vec<NormalizationPath>,
    pub placement_requirements: Vec<PlacementRequirement>,
    pub layout_requirements: Vec<OperationLayoutRequirement>,
    pub target_requirements: TargetRequirementSet,
}
```

> Illustrative API; not frozen until the Phase-2 exit gate passes.

## 2.5 Pilot proof alternatives

The compiler should represent, without yet selecting concrete opcodes:

### `compact-ash`

- explicit arithmetic over public/openable ASH;
- closed-asset identity check;
- exact ownerless conservation;
- manifest-derived shape;
- permissionless public construction.

### `transfer-live-receipts`

- explicit-value conservation as one proof alternative;
- confidential-value conservation as another proof alternative;
- explicit closed-asset identity;
- owner authorization;
- same-class constructor closure;
- sponsor isolation.

## 2.6 Phase-2 exit gate

Phase 2 is complete only when:

- [ ] the compiler crate is in the workspace;
- [ ] the compiler consumes only typed realization and typed policy/target
      abstractions;
- [ ] both pilot operations lower deterministically;
- [ ] all relations have stable IDs and provenance;
- [ ] every relation has at least one abstract proof alternative;
- [ ] every disclosure has a reason;
- [ ] every permissionless required witness is publicly available;
- [ ] every supported representation has a lifecycle path;
- [ ] every obligation has a placement requirement;
- [ ] every layout requirement derives from typed operation declarations;
- [ ] no tapscript opcode or stack index appears in target-independent IR;
- [ ] repeated analysis produces byte-identical derivative reports;
- [ ] relation-coverage skeletons contain no unowned relation;
- [ ] debug/release workspace checks remain green and clean.

---

# Phase 3 — Exact Elements target and foundational prototypes

> **Status:** PLANNED / PROTOTYPE-DRIVEN
> **Primary package plans:**
> [`packages/target-elements.md`](packages/target-elements.md),
> [`packages/tapscript.md`](packages/tapscript.md)
> **Primary research notes:**
> [`research/state-object-constructor.md`](research/state-object-constructor.md),
> [`research/wide-arithmetic.md`](research/wide-arithmetic.md),
> [`research/public-declassification.md`](research/public-declassification.md)
> **Depends on:** Phase 2 for production interfaces
> **May begin earlier:** isolated target research and throwaway prototypes

## 3.1 Purpose

Replace target assumptions with a typed Elements target compatibility
contract and validate the backend's hardest primitive constructions before
package interfaces freeze around them.

## 3.2 Target compatibility contract

Create the typed `target-elements` package and define one initial target,
expected to be a regtest target matching the intended Liquid semantics.

Per ADR-011 ("Target substrate compatibility"), no Elements
consensus-implementation source revision is part of the protocol identity
or a release pin; the node version used by target-native tests is recorded
as ordinary test provenance.

The compatibility contract must include:

- network kind;
- genesis/network identity strategy;
- tapscript activation;
- leaf version;
- opcode assignments used by the backend;
- exact semantics relevant to those opcodes;
- asset/value commitment prefixes;
- sighash behavior;
- transaction and policy resource limits;
- source references and integration-test provenance names.

The Markdown reference is not machine-consumed.

## 3.3 Minimal tapscript foundation

Implement enough typed backend infrastructure to support isolated prototypes:

- script builder;
- opcode representation;
- stack-effect model;
- deterministic encoding;
- explicit/confidential prefix checks;
- basic input/output introspection patterns;
- signature pattern interface;
- resource accounting interface;
- target error reporting.

This is foundation work, not complete operation emission.

## 3.4 State-constructor prototype

The prototype must test:

- canonical state metadata encoding;
- static code-subtree commitment;
- dynamic metadata commitment;
- predecessor constructor authentication;
- successor constructor reconstruction;
- one authenticated code root reused across predecessor and successor;
- x-only/compressed-point conversion;
- tweak verification;
- wrong internal key rejection;
- wrong parity rejection;
- wrong metadata rejection;
- wrong code-subtree rejection;
- metadata-leaf spend rejection;
- deterministic encoding;
- measured target cost.

The prototype's acceptance criteria are defined in
`research/state-object-constructor.md`.

A successful result produces an accepted implementation decision before the
STATE constructor API freezes.

## 3.5 Wide-arithmetic prototype

Implement and measure the exact floor relation required by
`floor_mul_div`.

The prototype must include:

- quotient/remainder or other exact witness relation;
- operand domain checks;
- zero-divisor rejection;
- wide multiplication;
- carry/borrow correctness;
- exact equality;
- remainder bound;
- boundary vectors;
- random differential tests against Rust arithmetic;
- stack and resource measurements;
- optional bounded SMT experiment if adopted.

The prototype's acceptance criteria are defined in
`research/wide-arithmetic.md`.

## 3.6 Public-declassification prototype

Test the concrete target handling of a confidential semantic input whose
result becomes publicly usable.

The prototype must compare:

- explicit output;
- public commitment plus authenticated public opening;
- residual blinding routing;
- permissionless successor construction;
- target transaction balancing;
- canonical representation.

It should focus on the burn-to-ASH or a synthetic equivalent before the burn
operation is fully implemented.

## 3.7 Phase-3 exit gate

Phase 3 is complete only when:

- [ ] the initial Elements target is encoded as typed Rust;
- [ ] every target claim used by the backend is typed and carries review
      provenance;
- [ ] target capability tests pass on the selected regtest environment;
- [ ] the minimal typed tapscript builder is deterministic;
- [ ] stack-effect validation rejects malformed patterns;
- [ ] explicit/confidential guards are tested;
- [ ] the state-constructor prototype reaches a documented decision;
- [ ] the wide-arithmetic prototype reaches a documented decision;
- [ ] the public-declassification prototype reaches a documented decision or is
      explicitly deferred behind a supported initial representation policy;
- [ ] measured resource reports exist for all accepted prototypes;
- [ ] package plans and decision records are updated from prototype results;
- [ ] no provisional research design is silently treated as a release contract.

---

# Phase 4 — End-to-end `compact-ash`

> **Status:** PLANNED
> **Depends on:** Phases 2 and 3
> **Primary packages:** compiler, target-elements, tapscript, linker,
> transaction, vectors

## 4.1 Purpose

Exercise the entire planned pipeline on the smallest meaningful
permissionless protocol operation.

## 4.2 Deliverables

Implement:

```text
typed architecture
→ realization compact-ash relation
→ compiler analysis
→ Elements proof selection
→ tapscript emission
→ relocatable program
→ linked ASH constructor/taptree
→ canonical transaction/witness ABI
→ concrete regtest transaction
→ relation-indexed evidence
```

The operation must support its calibrated finite input bound as a typed
parameter, although the first end-to-end vector may use a small batch.

## 4.3 Required evidence

Positive vectors:

- minimum valid ASH batch;
- larger valid bounded batch;
- optional valid sponsor envelope;
- deterministic output;
- exact ASH value conservation;
- no burn projection.

Negative vectors:

- one ASH input when the minimum is two;
- too many ASH inputs;
- duplicate input;
- wrong asset;
- wrong object family;
- wrong output count;
- wrong output value;
- non-ASH protocol output;
- closed-asset exfiltration;
- hidden private witness requirement;
- hidden signature requirement;
- malformed sponsor region;
- incorrect transaction layout.

## 4.4 Phase-4 exit gate

- [ ] `compact-ash` compiles from typed realization without handwritten backend
      operation policy;
- [ ] every relation has a carrying predicate;
- [ ] the linked bundle is deterministic;
- [ ] the transaction ABI is typed and documented;
- [ ] valid transactions execute on the pinned target;
- [ ] relation-indexed rejecting mutations all fail;
- [ ] permissionless construction uses public data plus sponsor-local data only;
- [ ] resource measurements fit target limits;
- [ ] the relation-coverage report is complete for the operation;
- [ ] generation/check paths remain separate and the checkout remains clean.

---

# Phase 5 — Live receipt transfer

> **Status:** PLANNED
> **Depends on:** Phase 4
> **Primary decision:** value-parametric, closed-asset-identity-rigid

## 5.1 Purpose

Validate owner authorization, metadata-parameterized receipt constructors,
closed-asset output closure, and value-representation latitude.

## 5.2 Deliverables

Implement live receipt transfer with:

- explicit `U` asset identity;
- owner-authorized input family;
- exact same-class output closure;
- owner and denomination redistribution within authorized conservation;
- optional sponsor envelope;
- explicit-value proof plan;
- confidential-value-conservation proof plan where supported;
- typed receipt constructor instances;
- representation metamorphisms.

## 5.3 Required evidence

Safety vectors include:

- missing owner signature;
- one omitted owner in a multi-owner transfer;
- time-locked input in live transfer;
- wrong asset;
- ASH output;
- bare/unclassified `U` output;
- hidden closed `U` exfiltration;
- value mismatch;
- duplicate destination reference;
- sponsor value altering receipt conservation;
- wrong constructor metadata.

Minimality vectors include:

- valid explicit transfer;
- semantically equivalent confidential-value transfer;
- split and merge under commitments;
- confidential sponsor value where supported;
- equal public semantic projection under representation change.

## 5.4 Phase-5 exit gate

- [ ] live transfer supports every selected representation plan;
- [ ] explicit closed-asset identity is enforced at every protocol seam;
- [ ] all owners authorize the complete economic output set;
- [ ] confidential-value acceptance is demonstrated where claimed;
- [ ] safety and minimality reports are separate;
- [ ] receipt constructor continuity vectors pass;
- [ ] no unsupported confidential asset identity is accepted;
- [ ] resource limits and deterministic bundle checks pass.

---

# Phase 6 — STATE constructor and announce maturity

> **Status:** PLANNED
> **Depends on:** successful state-constructor research decision and Phase 5

## 6.1 Purpose

Integrate authenticated metadata-dependent STATE succession in the smallest
state-changing operation.

## 6.2 Deliverables

Implement:

- canonical STATE metadata encoding;
- predecessor STATE authentication;
- successor STATE reconstruction;
- maturity-state encoding;
- unannounced-to-announced transition;
- minimum and maximum lead checks;
- operator authorization;
- optional sponsor envelope;
- exact preservation of all unaffected state fields;
- transition-certificate state edge;
- no RESV edge.

## 6.3 Required evidence

Positive vectors:

- minimum valid lead;
- maximum valid lead;
- valid operator authorization;
- valid sponsor envelope;
- exact successor-state reconstruction.

Negative vectors:

- lead below minimum;
- lead above maximum;
- second announcement;
- sealed predecessor;
- wrong operator signature;
- wrong predecessor constructor;
- wrong successor code subtree;
- wrong metadata field;
- changed economic field;
- hidden RESV use;
- missing STATE succession;
- metadata-leaf escape.

## 6.4 Phase-6 exit gate

- [ ] STATE constructor research decision is implemented;
- [ ] canonical state vectors are published and checked;
- [ ] predecessor and successor use one authenticated constructor lineage;
- [ ] all wrong-code-subtree vectors reject;
- [ ] `announce-maturity` matches the model relation;
- [ ] STATE succession resource use fits target limits;
- [ ] transaction/witness ABI is deterministic;
- [ ] relation coverage is complete.

---

# Phase 7 — Burn, ASH, and clear

> **Status:** PLANNED
> **Depends on:** Phases 4–6 and public-declassification decision

## 7.1 Purpose

Implement the share-nothing burn hot path and permissionless public clearing
path while preserving event provenance and accounting.

## 7.2 Burn deliverables

Implement:

- live-only receipt inputs;
- every-owner authorization;
- exactly one fresh ASH output;
- optional live receipt change;
- canonical burn-record ordinal layout;
- record payload family;
- exact value relation;
- burn event projection;
- optional sponsor envelope;
- no root use;
- no ASH input;
- no time-locked output;
- no closed-asset escape.

## 7.3 ASH deliverables

Retain and extend `compact-ash` with:

- public/openable ASH constructor policy;
- no private opening required for future compaction or clear;
- ownerless lifecycle closure;
- attestation silence.

## 7.4 Clear deliverables

Implement:

- STATE succession;
- nonempty bounded ASH batch;
- no RESV consumption;
- public/openable ASH values;
- clear clamp:

  ```text
  X = min(B, Y_L, Y - 1)
  ```

- positive progress;
- exact `tag-recon` destruction;
- optional residual ASH;
- clear projection;
- exact preservation of unaffected state;
- optional sponsor envelope;
- permissionless public construction.

## 7.5 Required evidence

Include:

- genuine burn event;
- ASH-input false burn;
- time-locked-input false burn;
- over-claiming records;
- under-claiming records;
- duplicate/skipped record ordinals;
- compacted ASH never creating a burn event;
- oversized ASH partial clear;
- residual ASH re-emission;
- no clear to zero total supply;
- zero-progress rejection;
- wrong state decrement;
- RESV substitution;
- hidden owner-recovery route;
- representation/declassification vectors.

## 7.6 Phase-7 exit gate

- [ ] burn and clear share the intended authenticated ASH lineage;
- [ ] event-type and value anchors are separately enforced;
- [ ] record acceptance remains distinct from burn provenance;
- [ ] ASH is permissionlessly usable under the selected public representation;
- [ ] clear preserves the `Y - 1` floor;
- [ ] no owner-recovery or closed-asset escape exists;
- [ ] event projection matches the model;
- [ ] full relation coverage and resource reports pass.

---

# Phase 8 — Wide arithmetic and redemption

> **Status:** PLANNED
> **Depends on:** accepted wide-arithmetic design and Phase 7

## 8.1 Purpose

Integrate exact wide floor arithmetic into a formula-bound reserve payout and
RESV succession/termination relation.

## 8.2 Deliverables

Implement:

```text
p = floor(x * Ω / Y)
```

with:

- exact authenticated receipt amount;
- exact public state values;
- owner authorization;
- formula-bound L-BTC payout;
- receipt destruction under `tag-redeem`;
- STATE succession;
- RESV succession for nonterminal redemption;
- RESV termination for exact sealing redemption;
- sponsor isolation;
- no payout reduction by fees;
- optional representation normalization/declassification path.

## 8.3 Required evidence

Include:

- partial redemption;
- floor-rounding boundary cases;
- wrong quotient;
- wrong remainder;
- zero divisor;
- arithmetic boundary values;
- time-locked receipt rejection;
- wrong owner;
- payout redirection;
- payout shortening with balancing output growth;
- pending-`Q` sealing rejection;
- exact sealing success;
- false RESV termination;
- sponsor interference;
- wrong state/reserve relation.

## 8.4 Phase-8 exit gate

- [ ] wide floor proof matches reference arithmetic;
- [ ] boundary and randomized differential tests pass;
- [ ] optional SMT evidence is included if adopted;
- [ ] payout and reserve-state deltas match the model;
- [ ] sealing is accepted only in the exact terminal case;
- [ ] nonterminal and terminal root edges are correctly derived;
- [ ] resource measurements fit the target;
- [ ] relation coverage is complete.

---

# Phase 9 — Deposit request and admission

> **Status:** PLANNED
> **Depends on:** Phases 6 and 8

## 9.1 Purpose

Implement the open request pipeline into scarce entitlements and active backing.

## 9.2 Client request construction

`create-request` remains client-policy construction over ordinary L-BTC.

The transaction package should provide a canonical request constructor, but
malformed open request-shaped outputs remain inert until an admission attempts
to consume them.

## 9.3 Cancellation

Implement:

- refund-key authorization;
- full-value refund;
- no receipt-owner substitution;
- sponsor-funded fee isolation;
- confidential/public request variants only where exact refund remains
  constructible and verifiable.

## 9.4 Admission

Implement:

- nonempty bounded request batch;
- request validation at consumption;
- target-pool binding;
- exact principal/service-budget partition;
- one entitlement per request;
- `ENT_AUTH` succession;
- exact `ENT` issuance;
- STATE and RESV succession;
- exact `Q` and reserve updates;
- active-backing cap;
- bounded admission reward;
- permissionless public construction.

## 9.5 Phase-9 exit gate

- [ ] malformed requests remain inert before consumption;
- [ ] cancellation returns full request value to the refund key;
- [ ] admission is permissionless and publicly constructible;
- [ ] every request creates exactly one entitlement;
- [ ] entitlement owner and target are pinned;
- [ ] principal and service budget exhaust request value;
- [ ] active-backing cap is enforced;
- [ ] over-cap admission leaves requests unspent;
- [ ] issuance and root succession match the model;
- [ ] resource and relation-coverage reports pass.

---

# Phase 10 — Settlement prototype and implementation

> **Status:** PLANNED / PROTOTYPE-DRIVEN
> **Depends on:** Phases 5, 8, and 9
> **Primary research note:**
> [`research/settlement-layout.md`](research/settlement-layout.md)

## 10.1 Purpose

Resolve and implement the hardest variable-batch relation without freezing an
untested draft layout.

## 10.2 Prototype-first rule

Before selecting the general ABI:

1. build a complete batch-size-2 prototype;
2. implement at least the candidate layouts required by the research note;
3. compare correctness, witness availability, stack/resource cost, and
   obligation placement;
4. run accepting and rejecting vectors;
5. select or reject the designs through an explicit decision record.

No larger default is assumed deployable before measurement.

## 10.3 Semantic deliverables

The final settlement implementation must enforce:

- one control;
- vault presence iff remaining class value is positive;
- nonempty bounded entitlement batch;
- matching target cycle;
- per-entitlement floor before aggregation;
- owner/class/value routing;
- entitlement destruction;
- continuing control/vault succession;
- terminal control closure;
- exact residue destruction and projection;
- permissionless construction;
- sponsor isolation;
- no maturity-dependent class rewrite.

## 10.4 Phase-10 exit gate

- [ ] batch-2 prototype evidence exists;
- [ ] one layout strategy is accepted by decision record;
- [ ] every batch obligation has a carrying predicate;
- [ ] owner routing and uniqueness are enforced;
- [ ] per-entitlement floors are exact;
- [ ] continuing and terminal cases match the model;
- [ ] offsetting or omitted relations are caught by independent mutations;
- [ ] worst-case transactions are measured;
- [ ] calibrated bound does not exceed measured feasibility;
- [ ] permissionless construction requires no owner private witness;
- [ ] relation coverage is complete.

---

# Phase 11 — Cycle

> **Status:** PLANNED
> **Depends on:** Phases 6, 8, 9, and 10

## 11.1 Purpose

Implement the operation that combines state succession, reserve carry,
cadence, issuance, class accounting, distribution creation, maturity
conversion, and fee sponsorship.

## 11.2 Deliverables

Implement:

- STATE succession;
- RESV succession;
- PACE succession and cadence reset;
- distribution-authority succession;
- cadence band:

  ```text
  before MIN: invalid
  MIN <= age < MAX: operator-authorized
  age >= MAX: permissionless
  ```

- exact cycle issuance:

  ```text
  ΔY = floor(Q * Y / Ω)
  ```

- bootstrapping class split;
- normal-phase all-live issuance;
- operator fee split;
- exact `U` issuance;
- optional control and vault creation;
- exact `DIST_CTL` issuance;
- empty-cycle behavior;
- atomic maturity conversion;
- maturity-cycle CPFP anchor;
- reserve-carry open flow;
- sponsor isolation;
- transition and event projections.

## 11.3 Required evidence

Include:

- too-early cycle;
- operator-only-band rejection;
- valid operator-band cycle;
- valid delayed permissionless cycle;
- missing authority;
- wrong issuance amount;
- wrong issuance destination;
- empty cycle;
- empty maturity cycle;
- post-maturity all-live issuance;
- wrong control/vault shape;
- wrong cadence reset;
- hidden operator gate on delayed path;
- wrong anchor condition;
- active-backing cap preservation;
- target CSV semantics;
- package-relay evidence for CPFP usage.

## 11.4 Phase-11 exit gate

- [ ] all root relations are enforced;
- [ ] cadence semantics match the three-regime model;
- [ ] empty and nonempty cycles match the model;
- [ ] issuance amounts and destinations are exact;
- [ ] maturity conversion is atomic;
- [ ] delayed cycle remains permissionless;
- [ ] all transaction layouts and witnesses are canonical;
- [ ] worst-case cycle transactions fit calibrated limits;
- [ ] relation coverage is complete;
- [ ] target dependency evidence exists for cadence and package behavior.

---

# Phase 12 — Complete evidence and deployment release

> **Status:** PLANNED
> **Depends on:** Phases 4–11
> **Primary plans:**
> [`packages/vectors.md`](packages/vectors.md),
> [`packages/release.md`](packages/release.md)

## 12.1 Purpose

Assemble complete deployment evidence and validate one final deployment
profile without conflating architecture, model, compiler, backend, substrate,
and independent-indexer claims.

## 12.2 Required package outputs

The completed toolchain must produce typed, deterministic:

- realization identity;
- compiler configuration identity;
- exact target identity;
- linked bundle;
- bundle hash;
- transaction layout and witness ABI;
- canonical wire vectors;
- resource calibration report;
- relation-coverage report;
- backend differential report;
- model unit-test report;
- property-test report;
- substrate dependency reports;
- script-integration report.

## 12.3 Independent evidence

Deployment release additionally requires separately implemented reports for:

1. raw attestation event projection;
2. canonical attestation query;
3. receipt-accounting audit.

These reports must remain separately hashed.

A candidate is not independent merely because it:

- runs in another process;
- reconstructs a `ReferenceIndexer` checkpoint;
- wraps the same model implementation;
- invokes the same query code through another API.

The independent implementation boundary must be documented in the deployment
evidence.

## 12.4 Calibration

Final calibration must:

1. enumerate every calibrated bound;
2. construct worst-case valid transactions for every affected operation;
3. measure complete transactions;
4. account for shared-bound users;
5. select deterministic values;
6. rerun all affected measurements after selection;
7. bind evidence and script-bundle hashes;
8. validate runtime constants against profile values.

Draft defaults are not release calibration.

## 12.5 Release validation

The release package must validate:

- architecture release;
- behavioural versioning gate;
- realization identity;
- compiler configuration identity;
- typed target compatibility-contract identity;
- linked-bundle identity;
- transaction ABI identity;
- calibrated-bound census;
- dependency-evidence census;
- artifact hashes;
- separate model/property/differential/integration reports;
- independent event/query/accounting reports;
- final deployment-profile status;
- domain-separated deployment-profile hash;
- clean and reproducible publication output.

Missing evidence is a release failure, never a warning.

## 12.6 Phase-12 exit gate

- [ ] every declared operation is implemented or explicitly excluded by a
      different deployment profile approved through normative review;
- [ ] every semantic relation has complete coverage;
- [ ] every target dependency has verified evidence where required;
- [ ] every calibrated bound has one unambiguous measured value;
- [ ] runtime bounds equal profile calibration;
- [ ] all bundle and report hashes are nonzero and verified;
- [ ] independent event, query, and accounting reports are present and
      separately committed;
- [ ] architecture and deployment release validators pass;
- [ ] generated release artifacts are byte-reproducible;
- [ ] release checks leave the checkout clean;
- [ ] the deployment profile is final;
- [ ] the final bundle/profile identities are recorded in the release record.

---

## 3. Parallel work policy

Some work may proceed in parallel without violating phase ordering.

### Allowed in parallel

During Phases 1–2:

- source review of the Elements target;
- throwaway regtest experiments;
- state-constructor spike;
- wide-arithmetic spike;
- transaction resource measurements;
- independent-indexer planning.

During operation phases:

- vector mutation design;
- independent indexer/auditor implementation;
- documentation updates;
- substrate test development;
- release-report schema design.

### Not allowed to freeze early

Before their gates:

- no target-specific type enters `RealizationSpec`;
- no provisional state-constructor encoding becomes a stable ABI;
- no draft arithmetic schedule becomes release evidence;
- no unmeasured settlement batch default becomes calibrated;
- no copied Markdown target claim becomes machine-consumed;
- no publication artifact becomes a compiler input;
- no test double is labeled an independent deployment implementation.

Parallel research is encouraged. Premature interface commitment is not.

---

## 4. Cross-phase invariants

Every phase must preserve these repository properties.

### 4.1 Normative identity

No implementation-only phase may accidentally move:

- Layer-0 identity;
- architecture semantic hash;
- architecture behavioural hash;
- stable architecture identifiers;
- realization-version major.

If a phase requires such a move, stop and perform normative version review.

### 4.2 One-way generation

No phase may introduce a semantic dependency from typed code onto a generated
publication artifact.

### 4.3 Determinism

All newly introduced typed outputs, generators, and reports must be
deterministic from their explicit inputs.

### 4.4 Clean tests

No test or check command may rewrite tracked files.

### 4.5 Locked builds

All Cargo CI/build/release paths use `--locked`.

### 4.6 Separate evidence classes

A model test never becomes:

- compiler proof;
- backend proof;
- substrate proof;
- independent-indexer proof;
- deployment release evidence

without the separately named report and boundary required for that class.

### 4.7 Fail-closed behavior

Unresolved relation, capability, constructor, layout, calibration, identity, or
evidence is an error.

---

## 5. Deferred work

The following work is outside the active critical path.

### Parked until after the first tapscript deployment

- production Simplicity emission;
- confidential closed protocol asset identity;
- generalized optimizer infrastructure;
- global instruction-search optimization;
- complete machine-checked Elements script semantics;
- generalized private entitlement arithmetic;
- arbitrary foreign sponsor sidecars;
- broad model API redesign;
- wallet-balance activation in the model;
- protocol operation changes not required by implementation evidence.

### Reconsideration rule

Deferred work may re-enter the roadmap only when:

1. its motivating problem is measured or demonstrated;
2. the relevant package boundary exists;
3. a decision or research note defines its scope;
4. the active roadmap identifies what it displaces;
5. normative versioning is reviewed if semantics may change.

---

## 6. Roadmap change control

A roadmap change should answer:

- Which phase changes?
- Why is the existing order insufficient?
- Which dependency has changed?
- What evidence supports the new order?
- Which package plans change?
- Which research notes change?
- Which accepted decisions change?
- Does any normative identity move?
- Does the current backlog need reordering?

Large phase changes should be reviewed separately from implementation commits.

---

## 7. Current entry point

While Phase 1 is active:

1. use [`backlog.md`](backlog.md) as the current task queue;
2. implement the typed realization foundation per
   [`packages/realization.md`](packages/realization.md)
   (`R1-001`–`R1-013`);
4. run the Phase-1 exit gate before beginning compiler analysis.

Do not begin by implementing tapscript operation bodies.

---

## 8. One-line roadmap

> First make the existing architecture, model, artifacts, tools, documents, and
> plans reproducibly trustworthy; then define the target-independent realization
> and compiler relation graph; next pin and prototype the target's hard
> constructions; prove the complete pipeline on `compact-ash` and live transfer;
> add STATE, burn/clear, arithmetic, admission, settlement, and cycle in
> dependency order; and finally release only a calibrated, reproducible bundle
> carrying relation-indexed backend evidence and separately implemented
> event/query/accounting reports.
