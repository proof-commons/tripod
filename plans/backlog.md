# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 3 — Elements target and foundational prototypes
> **Current condition:** Phases 1 and 2 are complete. The target-independent compiler constructs one deterministic, validated scoped analyzed program for the complete pilot scope, factorized per operation, with relation-owned requirements, a corruption-resistant assembly validator, and independent assembly oracles; the Phase-2 exit gate record is in §2.11, including passed document reproducibility. The next work is the Guide-11 preflight register in §5.6. This condition line is otherwise unreconciled against the recorded Guide-8 through Guide-10 completions; `G11-R12` owns that reconciliation.
> **Next gate:** Phase 4 — compact-ash end-to-end pipeline
> **Authority:** Current execution queue only. The specification, the realization document, typed architecture, implemented ADRs, accepted decisions, package contracts, phase cards, and accepted research results take precedence.

This file contains only:

- the current repository and readiness state;
- the latest static-review basis and open findings;
- compact historical milestones and gate records;
- the active Phase-2 compiler queue;
- immediate algorithm preparation;
- dependency posture;
- verification and clean-tree requirements.

Detailed historical implementation narratives belong to Git history, annotated
tags, phase cards, ADRs, accepted decisions, and focused evidence records—not
to the active backlog.

---

## 1. Backlog contract · `sec:backlog:contract`

### 1.1 Authority · `rule:backlog:authority`

This backlog sequences work. It is not protocol, realization, compiler, target,
ABI, evidence, deployment, or release input.

When artifacts disagree:

| Subject | Owner |
|---|---|
| abstract economic interface | the Attestation specification |
| realization meaning, invariants, obligations, and residuals | the realization document |
| finite registries and stable architecture IDs | typed architecture |
| current executable reference behavior | executable model |
| repository engineering policy | implemented ADRs |
| accepted cross-package implementation direction | planning decisions |
| package boundary | package contract |
| phase entry, deliverables, and exit | phase card |
| unresolved design question | research note |
| current task ordering | this backlog |

A lower owner never overrides an upper owner on the upper owner’s subject.

### 1.2 Status vocabulary · `tab:backlog:status`

| Status | Meaning |
|---|---|
| **TODO** | Ready when its named dependencies are complete. |
| **IN PROGRESS** | Actively being implemented, reviewed, or verified. |
| **BLOCKED** | A named dependency or correctness defect prevents safe progress. |
| **PARKED** | Deliberately inactive until a concrete consumer or measured need exists. |
| **DONE** | Implementation, focused evidence, required gates, documentation, and clean-tree evidence are recorded. |
| **DROPPED** | Deliberately not implemented; rationale and replacement are recorded. |
| **HISTORICAL** | Immutable evidence about an earlier revision; not a claim about the current checkout. |

Code resembling an intended result is not sufficient for `DONE`.

A static-review finding remains open until one of these occurs:

1. the finding is reproduced and fixed with focused coverage;
2. a typed proof shows the reported state is unconstructible;
3. reproduction shows the finding is false;
4. the owning policy or assurance claim is deliberately corrected.

“Existing tests pass” does not close a finding unless a named test reaches the
reported path.

### 1.3 Priority vocabulary · `tab:backlog:priority`

| Priority | Meaning |
|---|---|
| **P0** | Can manufacture false release, deployment, provenance, or semantic evidence. |
| **P1** | Trusted semantic/evidence boundary defect or active phase blocker. |
| **P2** | Required correctness, determinism, publication, identity, or documentation work before phase exit. |
| **P3** | Maintainability or evidence-quality work required by the active gate. |
| **POST** | Later-phase work that does not block the current gate. |

### 1.4 Definition of done · `rule:backlog:done`

An implementation task is `DONE` only when it records:

1. implementing source files;
2. focused positive and negative tests;
3. affected ADRs, decisions, package contracts, phase cards, or research notes;
4. exact verification commands and outcomes;
5. generated-publication and label impact;
6. semantic, identity, schema, and migration impact;
7. dependency impact;
8. final clean-tree output.

A review finding additionally records whether reproduction confirmed,
refuted, or reclassified it.

A dependency task additionally records:

- selected source and exact version;
- enabled features;
- transitive graph;
- licence;
- MSRV;
- unsafe boundary;
- determinism and parallelism implications;
- advisory status;
- lockfile impact.

A research task additionally records:

- exact question and constraints;
- prototype and tool versions;
- accepted and rejected candidates;
- positive and negative evidence;
- measurements;
- permanent implementation handoff.

### 1.5 Identity admission · `rule:backlog:identity-admission`

No new digest or digest-bearing field enters merely because an object is
important.

A proposed identity must satisfy
(`[ADR016-rule:identity:admission]`) and name:

- the complete typed object or exact bytes;
- owner and producer;
- a present consumer;
- the exact decision the consumer makes;
- assurance class;
- canonical projection and encoding;
- domain separator and recipe identifier;
- stale conditions;
- migration behavior;
- explicit non-claims.

No named consumer means no digest. No distinct decision means no digest.

---

## 2. Review basis and current evidence · `sec:backlog:review-basis`

### 2.1 Latest static review · `tab:backlog:review-basis`

The current review is the sixth static review, performed in two independent
passes over the supplied concatenation of the tree:

```text
tree:
   

selected files:
    220
```

The supplied filter excluded 352 files, among them:

```text
Cargo.lock
the paper sources
most unit and integration tests
most compiler implementation files
most executable-model implementation files
the label-tool implementation
several build/tooling packages and scripts
```

No Cargo, Meson, TeX, the native executor, advisory, or reproducibility
command was run as part of this review.

Therefore:

- the review makes no current green-build claim;
- the excluded packages and test directories received no content review;
- lockfile checksums and resolved features were not independently verified;
- advisory status was not checked;
- licence compatibility was not independently checked;
- historical gate records remain historical evidence only.

Its findings are recorded in §5.6. Its central verdict is that the native
evidence layer still does not bind evidence to its subject: an execution
transcript retains neither the requested subjects nor the exact fixtures
sent to the child, so it can be rebound to a census, matrix, target, or
deployment that was never executed; and the claim a fixture files under is
caller-authored, so claim-bearing metadata can be attached to an unrelated
or trivially true script. Both passes rank those as release-blocking for
that subsystem, and no public-declassification prototype begins while they
remain open. The review text is archived at
[plans/reviews/review-6-0.3.4-dev.md](reviews/review-6-0.3.4-dev.md).

### 2.2 Earlier review basis

The repository retains seven earlier reviewed trees as historical context:

```text
initial reviewed tree:
   

follow-up reviewed tree:
   

proof-planning reviewed tree:
   

second-review tree:
   

third-review tree:
   

fourth-review tree:
   

fifth-review tree:
   
```

Those reviews and their findings are evidence about their exact trees. They are
not current-checkout execution evidence. The second through sixth review
texts are archived under [plans/reviews/](reviews/README.md).

### 2.3 Historical Phase-1 gate · `gate:backlog:phase1`

The repository records the Phase-1 evidence tag:

```text
phase1-realization-foundation-v1
```

The tag is the immutable evidence record for its exact commit. Its existence
does not establish that the current checkout passes.

### 2.4 Historical T1–T5 remediation gate · `gate:backlog:t1-t4-remediation`

The repository records a completed remediation batch for T1–T5. Its recorded
result was:

- `scripts/ci.sh`: every available lane passed;
- `cargo-audit`: skipped because unavailable, so the run was partial-green;
- real Meson compile: passed;
- Meson tests: passed;
- document byte reproducibility: deferred;
- final clean-tree check: empty.

This is historical evidence for the remediation tree, not a current-tree gate.

### 2.5 Historical compiler-input gate · `gate:backlog:p2-004`

The repository records a completed P2-004 compiler-input batch:

- typed owner revalidation;
- explicit canonical compiler scope;
- strict analysis policy;
- immutable bound compiler input;
- focused public-API tests;
- available CI and Meson lanes passed;
- advisory lane skipped where unavailable;
- document byte reproducibility deferred because paper inputs were unchanged;
- final clean-tree check empty.

### 2.6 Historical Guide-2 gate · `gate:backlog:guide2`

The repository records a completed Guide-2 batch for:

```text
C1-005
P2-005
P2-006
```

The batch covered:

- scoped relation DAG;
- scoped expression DAG;
- predicate binding;
- checked constant folding;
- independent folding oracle;
- declaration-permutation determinism;
- dependency review for the compiler’s Petgraph edge.

The latest static review did not find a defect in these components.

### 2.7 Historical Guide-3 gate · `gate:backlog:guide3`

The repository records a completed Guide-3 batch for:

```text
C1-008
P2-007
P2-008
P2-009
```

The batch covered:

- relation-obligation classification;
- exact proof-plan enumeration;
- source requirements;
- constructibility;
- disclosure;
- representation lifecycle;
- independent exhaustive planning oracle.

The latest static review found T6 and T7 in this area. The historical gate
remains evidence that its recorded commands passed; it is not evidence that the
tested assertions were complete. P2-007 and C1-008 were therefore reopened as
blocked correctness work and closed again by the Guide-4 batch in §2.8.

### 2.8 Guide-4 reacceptance gate · `gate:backlog:guide4`

The repository records a completed Guide-4 batch for:

```text
T6
T7
T8
T9
C1-008
P2-007
```

The batch covered:

- external-evidence capability retention and fixed-requirement filtering;
- representation relations as static mode constraints;
- an oracle that derives fixed evidence requirements independently;
- the current-phase declaration weld across all four planning entry points;
- accurate compiler implementation-status documentation.

Its recorded result, on the tree ending at the crate-state reconciliation
commit:

- `scripts/ci.sh`: every available lane passed; `cargo-audit` skipped because
  unavailable, so the run was partial-green;
- real Meson compile and test: 10/10 passed, including check-generated
  (architecture publications byte-identical), labels, plans, census, and the
  reproducibility stamps;
- document byte reproducibility: deferred because paper inputs were unchanged;
- final clean-tree check: empty.

Identity impact of the batch: the architecture identity, generated
architecture publications, public schema, and dependency graph are unchanged;
no realization public identity or compiler-plan identity exists.

### 2.9 Guide-5 placement gate · `gate:backlog:guide5`

The repository records a completed Guide-5 batch for:

```text
C1-009
P2-010
```

The batch delivered, all crate-private:

- typed execution cases derived per feasible proof plan (representation
  fixed by the plan; sponsor absent/present the only expansion; four
  semantic cases per pilot across the plan set);
- relation discharge classification on independent axes (boundary, scope,
  activation, multiplicity) with an exhaustive per-variant match;
- abstract carrier roles with input-family coordinator anchors, quantified
  every-member roles, and availability-based eligibility;
- target-independent layout requirements citing relation IDs as semantic
  owners;
- exact feasible placement enumeration with explicit typed limits, no
  partial results, and inclusion-minimal candidate retention;
- an independent exhaustive placement oracle restating every hard
  predicate, agreeing with production on every pilot scope and fifteen
  adversarial synthetic cases;
- end-to-end pilot analysis: two feasible plans per pilot, four cases per
  pilot, 36 feasible placements per compact-ASH plan and 216 per
  live-transfer plan, with the combined scope proven to be the exact
  product of its operations.

Findings recorded for later work:

- multi-operation placement factorizes as a product across operations;
  the analyzed-program work should consider per-operation storage;
- compact ASH declares sponsor recognition without a sponsor cardinality
  relation, so its sponsor family is bounded by no cardinality census;
  live transfer declares both. This is a realization-level asymmetry, not
  a compiler defect.

Its recorded result, on the tree ending at the placement documentation
commit:

- `scripts/ci.sh`: every available lane passed; `cargo-audit` skipped
  because unavailable, so the run was partial-green;
- real Meson compile and test: 10/10 passed, including check-generated,
  labels, plans, census, and the reproducibility stamps;
- document byte reproducibility: deferred because paper inputs were
  unchanged;
- final clean-tree check: empty.

Identity impact of the batch: the architecture identity, generated
architecture publications, public schema, and dependency graph are
unchanged; no compiler-plan identity was minted; placement limits are an
explicit parameter, not public policy.

### 2.10 Guide-6 coverage gate · `gate:backlog:guide6`

The repository records a completed Guide-6 batch for:

```text
P2-011
C1-010
C1-013
```

Sponsor-cardinality ruling: the Guide-5 finding that compact ASH declared
sponsor recognition without sponsor cardinality was confirmed as a
realization omission and repaired. The realization now declares PLAIN_LBTC
input cardinality (minimum zero, maximum the fee-sponsor-input-max bound)
and output cardinality (minimum zero, maximum exactly one) for compact ASH,
with recognition-before-cardinality dependencies, and typed validation
derives the expected family-relation census from the architecture operation
rows so a missing cardinality or recognition relation now fails derivation.
The architecture identity and generated publications are unchanged; both
pilots now have identical placement shape.

The batch delivered, all crate-private:

- relation-indexed coverage requirements: stable boundary-carrying
  requirement identities, an exhaustive typed mutation catalogue with no
  wildcard arm, positive/negative/inactive-valid derivation per boundary,
  per-operation aggregation with no cross-operation product;
- carrier coverage compressed from the feasible placement product
  (216 placements per plan compress to at most three assignment
  alternatives per relation-case) with multiplicity preserved;
- accepted semantic-projection requirements answered by the evidence role
  of their own boundary;
- typed coverage dependencies: two-pass definition census and reference
  resolution, a direct Petgraph graph, strict active-descendant collateral
  closure, canonical SCC diagnostics with cycles forbidden;
- an independent coverage oracle restating the acceptance matrices,
  collateral closure by repeated scan, and SCC by mutual reachability,
  plus property-generated bounded instances;
- end-to-end pilot coverage: two plans and four cases per pilot,
  46 and 48 relation-cases per plan, graphs of 344/365 and 362/489
  nodes/edges, combined scope factorizing per operation, sponsor opacity
  over the complete aggregate projection, determinism under declaration,
  candidate, placement, and relation-case permutations.

Defects surfaced and repaired inside the batch:

- the independent oracle caught the projection binder assigning the
  target-execution evidence role to every boundary's accepted projection;
  each boundary's projection now carries its answering role, guarded in
  the owning module;
- pilot integration removed a quadratic carrier-compression rescan over
  the combined placement product; identical verdicts, polynomial cost.

Its recorded result, on the tree ending at the coverage documentation
commit:

- `scripts/ci.sh`: every available lane passed; `cargo-audit` skipped
  because unavailable, so the run was partial-green;
- real Meson compile and test: 10/10 passed, including check-generated
  (architecture publications byte-identical), labels, plans, census, and
  the reproducibility stamps;
- document byte reproducibility: deferred because paper inputs were
  unchanged;
- final clean-tree check: empty.

Identity impact of the batch: the architecture identity, generated
architecture publications, public schema, and dependency graph are
unchanged; the realization relation census expanded internally to match
architecture cardinalities; no realization or compiler identity exists or
was minted; no new dependency entered.

### 2.11 Guide-7 Phase-2 exit gate · `gate:backlog:guide7`

The repository records the completed Guide-7 batch for:

```text
P2-012
P2-013
C1-014
```

together with the second-review preflight repairs recorded in §5.2.

The batch delivered, all crate-private:

- relation-indexed requirement bundles whose capability, source, and
  evidence unions equal the candidate aggregates exactly in both
  directions;
- factorized operation analysis: cases, relation-case requirement
  bundles, carrier eligibility, placements, layout, coverage, and the
  coverage dependency graph derived per operation, with the existing
  exact search reused unchanged on the operation-restricted census;
- one scoped analyzed program per scope, binding its exact typed source,
  reporting partial architecture scope with the eleven out-of-scope
  operations derived from the architecture census, explicit lifecycle
  incompleteness, and unresolved external evidence, with the execution
  report excluded from the stable semantic projection;
- a corruption-resistant assembly validator rechecking all thirteen
  closure families against fresh component re-derivation, with
  sixty-four corruption mutations each rejected for a focused typed
  reason; coverage closure is exact in both directions, resolving SR2-09;
- an independent assembly census oracle restating every expected census
  without the production expectation derivation, a requirement-union
  oracle, and a two-level factorization oracle;
- end-to-end pilot acceptance: the acceptance relation lists are the
  censuses, rejected proof/mode pairings are absent, and the absence of
  a compiler identity is enforced by two compile-time locks;
- the combined-scope product regressions demoted to explicit phase-exit
  lanes after replacement signal landed, taking the ordinary compiler
  suite from minutes to seconds.

Measured pilot results: 23 and 24 relations, 46 and 48 relation-cases,
216 placements and 69 layout requirements per operation factor, coverage
graphs of 344/365 and 362/489 nodes/edges, 4 combined plans storing 432
placements while denoting the never-materialized 46,656-member product,
2 unresolved substrate-conservation obligations, 3 outstanding lifecycle
exits, 11 of 13 architecture operations outside scope.

Phase-2 exit evidence, on the final batch tree:

- declared MSRV lane, rustc 1.88.0: `scripts/ci.sh` — every available
  lane passed; advisories and the Meson contract skipped in that
  environment;
- current stable lane, rustc 1.97.1: same result, after repairing four
  new stable-toolchain pedantic findings;
- nightly lane with `CI_REQUIRE_MESON=1`: every available lane passed
  including the mocked Meson contract and its census-audit edge;
  advisories skipped;
- real Meson compile and test: 10/10 passed;
- the three explicit product regressions: factor sizes 216 × 216,
  product 46,656, exact set equality against the retired global
  enumeration;
- document byte reproducibility: PASSED — fresh and reused builds
  produce identical PDF bytes, and the reused build refreshes the
  source epoch;
- `cargo-audit`: SKIPPED, tool unavailable; the run is partial in the
  advisory dimension only;
- dependency evidence: `Cargo.toml` and `Cargo.lock` unchanged;
- final clean-tree check: empty.

One transient test failure during a deliberately concurrent triple-lane
run did not reproduce in isolation and the affected lane was rerun
uncontended and passed; the recorded results are the uncontended runs.

Identity impact of the batch: Layer-0 version, architecture schema and
hashes, and generated publications unchanged; no realization or compiler
identity exists or was minted; no compiler publication was added; the
deployment-profile identity remains dormant behind validation; no new
dependency entered.

### 2.12 Guide-8 target foundation gate · `gate:backlog:guide8`

The Guide-8 batch delivered the third-review preflight (§5.3, all fifteen
rows), the `tripod-target-elements` crate, the minimal public
compiler target-requirement boundary, and the `tripod-tapscript`
capability adapter. Starting revision 085ef25; batch branch merged
fast-forward after this record.

Preflight: sponsor erasure is fee-sponsor-flow-role based with two-point
overlap rejection and a kernel balanced-theft regression; the compiler
refuses protocol-claimed ordinary-L-BTC scopes with a typed error and both
pilot analyzed projections were verified byte-identical; the complete
analyzed-program validator runs on the construction path with the execution
report validated; projections compare exactly and duplicate-sensitively;
the coverage graph uses one enabler-toward-dependent orientation with
endpoint classes checked on insertion; exact-search counters are checked
before increment.

Target foundation: 38 opcodes reviewed against upstream Elements source
with review provenance recorded in the target reference and excluded from
the semantic projection; the tapscript leaf version is the reviewed 0xc4;
arithmetic failure retains operands and pushes false; asset and value
introspection push payload and prefix separately; 27 encoding classes,
37 capabilities, and 20 evidence requirements validate with mutation and
permutation coverage; sighash dimensions are recorded unreviewed and
commitment equality and authenticated opening remain unsupported; the
development binding rejects production, zero identifiers, and version
mismatch and carries no credential field. The compiler exposes
RequiredCapability with a complete census and a read-only requirement set
constructible only from fully validated analysis; the adapter classifies
all thirteen capabilities exhaustively with an independent oracle, the
authorization rows block on the unreviewed sighash dimensions, and the
complete-backend-pattern claim is unconstructible by type.

Gate evidence, all on the merged batch tree: ci.sh eleven lanes green
under the pinned SDK toolchain with the advisory lane RUNNING through an
isolated cargo-audit shim (149 crates scanned, no advisories) and the
mocked Meson contract required; canonical meson compile and test 10/10;
the workspace suite green across 46 test binaries; document byte
reproducibility not required this batch because no paper input changed
(the one-line realization-document correction is covered by the label and
weld lanes). Toolchain note: the SDK extension bundles clippy 0.1.88
against a 1.99-nightly cargo, and the current upstream clippy 0.1.99 adds
an assert-is-empty lint; its machine-applicable rewrites were adopted, and
roughly ninety non-machine-applicable test assertion sites remain OPEN as
drift work for when the lint reaches a supported gate toolchain.

Identity impact: Layer-0 version, architecture schema and hashes, and
generated publications unchanged; no target, deployment, compiler, or
report digest minted; Cargo.lock gained exactly the two first-party
package stanzas; no third-party dependency entered.

### 2.13 Guide-9 target-native gate · `gate:backlog:guide9`

The Guide-9 batch closed the fourth-review register (§5.4, all eleven
rows), made the target contract sound, and delivered the typed instruction
core with development target-native primitive evidence. Starting revision
88d0661; batch branch merged fast-forward after this record.

Contract soundness: a reviewed-Elements trust state distinct from generic
validation, constructible only by the first-party derivation or exact
typed equality; a typed success algebra with retained operands and
condition-discriminated alternatives re-reviewed against upstream with
line provenance; six cross-subcontract welds; transitive capability-status
closure under an explicit strength order; package-owned encoding
interpretation; a reviewed literal-push contract whose minimality cascade
is ordered data and whose relay-versus-consensus enforcement is typed; a
five-byte timelock operand class making the sequence disable flag
stateable. Identity and publication repairs: the draft and release
validators return the only wrappers accepted by the public architecture
identity functions with published values proven unmoved; publication
freshness now includes the required mode with in-place mode-only repair;
ADR-016 records the bounded grandfathered-recipe exception.

Instruction core and adapter: typed instructions, checked stack items,
canonical minimal-push serialization, a strict supported-subset parser
with round-trip equality, and an abstract stack validator carrying every
success alternative and all three failure shapes under checked work
limits, agreeing with an independent bounded oracle; the static adapter
consumes only the reviewed contract and answers both compiler censuses
with exhaustive matches and both-direction exact equality.

Native evidence: a secretless conformance harness with a lock-step
protocol that fails closed on every malformation and types mock runs as
unable to satisfy the gate; a 398-case fixture census whose expectations
come from the typed contract and independent published vectors; a
first-party executor adapter driving a disposable wallet-free
elementsregtest node, judging consensus fixtures by block validation and
relay fixtures by mempool acceptance, with an explicit reason-string
mapping that leaves target-collapsed causes coarse rather than guessed.
The run: 398 of 398 cases passed, 18 of 18 required evidence rows, zero
infrastructure errors, report bytes identical across fresh-node runs and
the build lane; triage attributed and repaired every first-contact
divergence in its owning module, including a transcription error in a
published signature vector that only real verification exposed. Executor
provenance: Elements Core v28.99.0-6f43e3ffe730, rebuilt from a clean
tree at the reviewed revision after an earlier stale binary predating the
elliptic-curve stack-size fix was refused for evidence; synthetic
development identifiers 32x09 and 32x07.

Gate evidence on the merged tree: ci.sh eleven lanes green under the
pinned SDK toolchain with the advisory lane running through the isolated
shim and the mocked Meson contract required; canonical meson compile and
test eleven of eleven; the native lane on the batch tip with the stamp
produced; document byte reproducibility not required because no paper
input changed. Honest bounds: sighash semantics remain unresolved by
design, commitment equality and authenticated opening remain unsupported,
confidential-value conservation remains transaction-level evidence, the
report completeness is partial-unresolved-remains, and the Guide-9
concept text stays in the session archive because carrying it breached
the plan-tree hard byte cap.

Identity impact: Layer-0 version, architecture schema and hashes, and
generated publications unchanged; no digest of any kind minted; the
conformance package is the only Cargo.lock addition; no new third-party
dependency beyond workspace-existing crates plus a test-only PTY helper.

### 2.14 Guide-10 prototype gate · `gate:backlog:guide10`

The Guide-10 batch closed the fifth-review register (§5.5, all thirteen
rows), made the native evidence layer self-validating, and delivered
both Phase-3 constructor and arithmetic prototypes with accepted
research decisions. Starting revision bdb67da; batch branch merged
fast-forward after this record.

Evidence-boundary repairs: protocol and report schema 2 with strict
bounded NDJSON; a validated report wrapper that recomputes every case
status, evidence disposition, and summary with duplicate-sensitive
censuses in both directions; complete fixture projections in every
report row; a typed claim census beneath the broad evidence rows with
deliberately unresolved claims stated; reviewed development binding
retaining the exact target projection; observed executor environment
compared before any case runs; separated adapter, node, and integration
provenance with the checkout-fallback misattribution removed;
process-group timeout supervision with a descendant-survival regression;
the signature operand model made representable for empty, invalid, and
unknown-key paths; the relation-identity weld; and the pre-release
deployment-profile trust-state split.

Substrate: target contract V2 expands the reviewed census from
thirty-eight to fifty-five opcodes with seventeen compound-proof
primitives reviewed at the ADR-018 merged tip; canonical byte ordering
is recorded unsupported; the reach bound, that no reviewed primitive
reads below the third stack item, is machine-checked and governed every
compound layout; operand-conditioned result widths and width-only tweak
operands were review-based contract corrections.

Prototype decisions, recorded in the research owners: the
metadata-dependent constructor is an accepted prototype under the
dynamic-metadata-leaf candidate, with the successor derived on-stack
from the predecessor's authenticated bytes, one authenticated static
root, fixed-order branch hashing with creator-side nonce grinding, and
a machine-checked unspendable metadata leaf; exact wide floor
arithmetic is an accepted prototype under derived limbs at base two to
the twenty-six through the reviewed Euclidean sixty-four-bit division,
witnessing only the five semantic amounts. Public declassification
remains the open third prototype.

Native evidence on the merged tree: both complete matrices ran against
the reviewed executor, Elements Core v28.99.0-0b3bffd93138 at the
ADR-018 intended tip 0b3bffd with upstream base b7fc5d0 and local
topics fix/tapscript-opcodes and notes, chain elementsregtest, through
the check-target-elements-prototypes command: constructor continuity
thirty-six of thirty-six cases with nine of nine required claims, and
wide floor thirty-nine of thirty-nine cases with eleven of eleven
required claims, each report byte-identical over two consecutive gated
runs. First native contact surfaced two constructor fixture defects and
zero program defects; both fixtures were repaired with review
justification. Measured resources: constructor 699 script and 924
witness bytes with the 520-byte element bound binding at a fifth of
maximum; wide floor 523 script and 606 witness bytes. Report digests
are recorded in the research files; the artifacts themselves are run
output and are not checked in.

Gate evidence on the merged tree: the working debug lane green through
every wave with the workspace suite growing from 1913 to 2151 tests;
ci.sh lanes green under the pinned SDK toolchain with the advisory lane
through the isolated shim; canonical meson compile and test green; the
native primitive lane and both native prototype lanes run on the batch
tip. Honest bounds: the five newly typed failure classes are unexercised
by any native row; compound fixtures state no admitted failure-class
set, so observed classes are recorded but not compared; the request
still carries the fixture expectation under the typed boundary; mock
runs remain typed as unable to satisfy any gate; nothing here is an
operation, linked bundle, transaction ABI, calibration, or production
claim.

Identity impact: Layer-0 version, architecture schema and hashes, and
generated publications unchanged; the target contract version moved to
V2 as an explicit reviewed revision; no digest of any kind minted; the
nix crate moved from development to production dependency of the
conformance package for process-group supervision; no new third-party
package entered the graph.

---

### The verification harness's two standing hazards · `rem:backlog:verification-harness`

A verdict is read from the report wrapper's own report line, never from a pipeline's shell status: the wrapper propagates its exit code faithfully, and a pipe to a filter truncates the status to the last stage's. And a single shared build-target directory is poisoned when checkouts of two different commits build the same crates into it, so a lane that moves between commits pins a target directory of its own.

### Publication metadata · `rem:backlog:publication-metadata`

The published documents carry their attribution and their terms: the attribution position names Proof Commons, a commons rather than a company, and the document set is dedicated to the public domain. A specification exists to be reproduced, cited and implemented, and an attribution condition taxes exactly that use, so the attribution that matters is the one the title page carries. The rights line states the dedication, the licence-URL field names the canonical deed for machine readers, and the document licence file holds the legal code verbatim.

### The crates carry the product's name · `rem:backlog:crate-naming`

Each crate manifest and the build system's project name the product, so the product the repository describes is the product its artifacts are named after. The name feeds nothing that hashes, exports or gates: the versioning derivation reads the version alone, the binary and library target names are their own, and the generated model artifacts never carried it.

### The realization's fixed body states its law · `rem:backlog:law-delimitation`

The realization's fixed body states the law without restating the era. The versioning law leaves its worked example to the masthead, which prints the live binding; the hash pins defer to the algorithm the attached envelope names; and the schema discriminants are read at their carriers — the envelope for the architecture manifest, the canonical query context for the wire, the bound profile for deployment. The workspace authors field names the publishing organization.

### The shared crate's acknowledgement · `rem:backlog:cli-common-thanks`

The shared command-line crate's README thanks the developers whose work inspired the tooling. Gratitude for inspiration, stated once, naming no repository and claiming no lineage: the crate's own manual remains the authority on what it does.

### The tip's tests assert their own consistency · `rem:backlog:tip-convergence`

The release profile's identity test derives its expected value from the profile itself rather than comparing against a hand-pinned constant, so the tree asserts its own internal consistency instead of a value a reader would have to take on trust. The paper is Attestation, the product is Tripod, and the protocol object is the attestation contract.

### The licence terms and their citation families · `rem:backlog:licence-convergence`

The code licence is stated once, in the licence file, and the manifest key, the toolchain licence rule and the dependency comments agree with it rather than restating it a second way. The dependency-review clauses name what a dropped feature pulled in and which rule refuses it, which holds under any workspace licence, and the toolchain record says why a copyleft workspace strengthens rather than relaxes the permissive-dependency rule. Documents stay under the public-domain dedication.

### The hash-citation audit the lint suite runs · `rem:backlog:hash-citation-audit`

The audit holds every published hexadecimal value to the identity rule, as a labels binary in the lint suite wired exactly as the forbidden-text check is: it asks git for the complete tracked set and requires every value to be described by a rule in the citation-families table, which names what each kind of value measures and the program that writes it. The audit places rather than recomputes — whether a digest is right belongs to the test that owns it — and what it establishes is that every value has an owner on the record.

### The cut the settled records carry · `rem:backlog:record-cut`

A planning record earns its place by what it establishes, so the passages that spent themselves on a document's own corrections are gone and what survives argues the result. A citation is a promise that a reader can fetch what it names, so the sites that cited tags no hub carries name the phase cards and the archived record as the durable record, with nothing hash-shaped substituted. The archive index states its rule for when a settled record may be edited at all, and states it there rather than here: a wave record that paraphrases a rule it does not own carries a second copy of it, and the copy stops being true the moment the rule is sharpened.

### The discipline every published value follows · `rem:backlog:hash-citation-discipline`

Every hexadecimal value the tree publishes declares how a reader establishes it: recomputable from bytes the tree carries under a named algorithm, resolvable to a referent a reader can fetch, or defined by something other than its author. A value that nothing regenerates, nothing resolves and nothing defines stands nowhere, including inside an encoded rendering, which a decode-and-assert check reads. The cascade above the captures recomputes from the bytes it is taken over, and each chosen constant is a derivation from a string published beside it.

### The separators spell the product's name · `rem:backlog:separator-convergence`

A domain separator is hashed input that identifies a recipe, and the product's name is part of that identity. Each separator spells it at the version it carries, because no projection, encoding or algorithm moved. The identities that follow those bytes are re-taken by the tree's own generator rather than edited, so the typed pin, the generated artifacts, the realization's envelope and the versioning gate carry one result.

## 3. Current repository state · `sec:backlog:state`

### 3.1 Implemented areas · `tab:backlog:implemented`

| Area | Current source state |
|---|---|
| Specification | Released *Attestation* paper, version owned by the paper source |
| Realization document | Attestation Realization with final architecture appendix and tracked version binding |
| `architecture` | Typed architecture, validation, semantic/behavioural hashes, deployment-profile scaffolding |
| `model` | Executable state machine, invariants, property/corruption suites, indexer and accounting projections |
| `realization` | Target-independent typed semantics for compact ASH and live receipt transfer |
| `compiler` input | Owner-revalidated architecture/realization binding, explicit scope, strict policy |
| `compiler` graphs | Scoped relation and expression DAGs with stable typed projections |
| `compiler` folding | Conservative checked constant folding with independent oracle |
| `compiler` source analysis | Typed operands, authenticatable source requirements, sponsor-value read rejection |
| `compiler` constructibility | Authorization-case analysis and source/constructibility weld |
| `compiler` disclosure | Inherited and added disclosure with typed reasons |
| `compiler` lifecycle | Representation choices and required-exit analysis |
| `compiler` proof search | Exact deterministic feasible-plan enumeration; fixed external-evidence requirements retained, representation modes static; reaccepted in §2.8 |
| `compiler` placement | Typed execution cases, discharge classification, carrier eligibility, exact feasible placement sets, layout requirements, independent placement oracle; gate record §2.9 |
| `compiler` coverage | Relation-indexed coverage requirements, typed mutation catalogue, coverage dependency graph with forbidden-cycle SCC policy, independent coverage oracle; gate record §2.10 |
| `compiler` analyzed program | Relation-indexed requirement bundles, factorized operation analysis, scoped analyzed programs with a corruption-resistant validator and independent assembly oracles; gate record §2.11 |
| `artifacts` | Generated-publication derivation, writer/checker, realization-document weld |
| `labels` | Owner-aware Markdown/Rust label graph, census, plan checks, register rendering |
| `cli-common` | ADR-010 streams, diagnostics, checker report/stamp publication, batch publication |
| Meson | Explicit source census, stamp-backed checks, mocked document graph |
| Security policy | Public-data interfaces and external execution-environment boundary |
| Path policy | Central tracked-mode audit, lexical output roles, explicit host-filesystem non-claims |

### 3.2 Current published identities · `tab:backlog:identities`

Exact hash and version values are deliberately not duplicated here. Read each
identity from its authority:

| Identity | Authority |
|---|---|
| Specification version | `papers/attestation/main.tex` and `sections/00_title.tex` |
| Architecture schema, semantic/behavioural algorithms and hashes, anchor-set hash | `packages/model/generated/architecture.json`, checked against typed architecture |
| Realization version (tracked binding) | `docs/attestation/realization.md` masthead |
| Attestation wire schema | typed model/architecture constants |
| Deployment-profile schema | typed architecture constants |

The generated architecture publication is authoritative. Planning prose must
not become a competing identity source.

Architecture finality does not imply:

- complete realization scope;
- compiler completeness;
- target support;
- linked bundle or ABI existence;
- deployment evidence;
- production readiness.

### 3.3 Not implemented · `tab:backlog:not-implemented`

```text
public complete compiler-analysis API beyond the target-requirement boundary
compiler-plan identity
backend proof patterns
STATE-constructor, wide-arithmetic, and declassification prototypes
production target-native evidence

tripod-linker
tripod-transaction
tripod-vectors
tripod-release

independent deployment observers
production deployment
production signer or wallet
production key management
```

### 3.4 Readiness statement · `rem:backlog:readiness`

```text
Attestation specification:          published
Realization contract:              published
Typed architecture:                final and pinned
Executable model:                  implemented
Typed realization pilots:          implemented
Compiler input/graph/folding:       implemented internally
Compiler source/disclosure/lifecycle:
                                    implemented internally
Compiler exact planning:            implemented and reaccepted
Compiler placement/layout:          implemented internally, section 2.9
Compiler coverage:                  implemented internally, section 2.10
Complete analyzed pilots:           implemented internally, section 2.11
Compiler target-requirement boundary:
                                    public and validated, section 2.12
Typed Elements target contract:     reviewed and welded, section 2.13
Capability and evidence adapter:    implemented, section 2.13
Typed instruction core:             implemented, section 2.13
Development target-native evidence: recorded, section 2.13
Production target evidence:        absent
Backend patterns/linker/ABI:       absent
Independent deployment evidence:   absent
Production deployment:             absent
Phase-2 exit gate:                 passed and recorded, section 2.11
```

Current packages are public-data tools. They do not legitimately accept private
keys, seed material, signing nonces, blinding factors, private openings,
credentials, or production authority.

---

## 4. Compact historical record · `sec:backlog:history`

### 4.1 Completed phases · `tab:backlog:completed-phases`

| Phase | Status | Durable record |
|---|---|---|
| Phase 0 | HISTORICAL | the recorded baseline and identities on [the Phase-0 card](phases/00-baseline.md) |
| Phase 1 | HISTORICAL | the completion evidence on [the Phase-1 card](phases/01-realization.md), and the gate record in [the backlog archive](history/backlog-history.md) §2.3 |
| Phase 2 | HISTORICAL | Guide-4 through Guide-7 gate records, §2.8–§2.11 |
| Phase 3 | Active | current backlog and Phase-3 card |

### 4.2 Historical finding families · `tab:backlog:historical-findings`

| Family | Status | Scope |
|---|---|---|
| `F1` | HISTORICAL | Phase-1 remediation |
| `F2` | HISTORICAL | Post-Phase-1 boundary remediation |
| `F3-001`–`F3-010` | HISTORICAL | Meson, publication, projection, ownership, and stamp remediation |
| `F4-001`–`F4-005` | DONE or DROPPED | Predicate, lifecycle, and labels findings |
| `A17-001`–`A17-005` | DONE | ADR-017 implementation |
| `R1`–`R6` | DONE | Path, ADR status, dependency, CI, recipe, and compiler-status review |
| `S1`–`S7` | DONE | Semantic-boundary review and status weld |
| `T1` | DONE | Bound conformance observations to executed requests |
| `T2` | DONE | Typed substrate-conservation evidence requirement |
| `T3` | DONE | Batch-staged multi-output publication |
| `T4` | DONE | Runtime-bound conformance in the global invariant |
| `T5` | DONE | Removed duplicated volatile identities from planning prose |

### 4.3 Identity work · `tab:backlog:identity-work`

| ID | Status | Result |
|---|---|---|
| `I1-001` | DONE | ADR-016 adopted |
| `I1-002` | DONE | Current identity inventory |
| `I1-003` | DONE | Future immediate-edge identity DAG |
| `I1-004` | PARKED | Typed evidence envelopes; waits for a persistent report consumer |
| `I1-005` | BLOCKED | Deployment-profile migration; waits for bundle, ABI, and evidence types |
| `I1-006` | BLOCKED | Release root; waits for the release package |

The identity freeze is lifted only under ADR-016 admission. Phase 2 still mints
no public realization or compiler digest without a real consumer.

---

## 5. Current static-review findings · `sec:backlog:findings`

### 5.1 Summary · `tab:backlog:findings-current`

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `T6` | P1 | DONE | External-evidence relations bypassed abstract capability validation during proof planning. |
| `T7` | P1 | DONE | Representation-relation proof choices were not constrained by the selected representation mode. |
| `T8` | P2 | DONE | Current-phase declarations disagreed and the plan checker did not detect all copies. |
| `T9` | P2 | DONE | Compiler status documentation materially understated implemented internal analysis. |

All four findings were reproduced or directly corrected, repaired, and
verified in the Guide-4 batch; the gate record is in §2.8. Their evidence
records follow. Implementation detail beyond these records belongs to Git
history. The later two-pass review recorded in §2.1 opened the SR2 findings
in §5.2.

### 5.2 Second-review findings · `tab:backlog:findings-sr2`

| ID | Severity | Status | Finding |
|---|---|---|---|
| `SR2-01` | High | DONE | CI could report green without the tracked-entry mode audit; lint alias and mocked contract omitted census-audit. |
| `SR2-02` | Medium | DONE | Root-use policy was reused as the observed root effect, rejecting valid succession under succession-or-termination. |
| `SR2-03` | Medium | DONE | Lifecycle relations and lifecycle graph declarations were not bidirectionally welded. |
| `SR2-04` | Medium | DONE | Placement validation accepted noncanonical, duplicate, and surplus-layout assignments. |
| `SR2-05` | Medium | DONE | Placed proof-plan validation did not compare the exact offered and placed sets. |
| `SR2-06` | Medium | DONE | Public deployment-profile hashing accepted unvalidated profiles; hashing now requires the validated wrapper and the identity stays dormant. |
| `SR2-07` | Medium | DONE | Open-flow observation normalization did not enforce reference sides, cross-flow uniqueness, or anchor exclusion. |
| `SR2-08` | Low | DONE | Representation-conditional activation was keyed by mode only; it is now keyed by object and mode. |
| `SR2-09` | Low | DONE | Coverage shape validation was weaker than derivation; the analyzed-program validator compares exact independently re-derived coverage projections in both directions, gate record §2.11. |
| `SR2-10` | Low | DONE | Compiler package-index status and review provenance had drifted; both reconciled. |
| `SR2-H1` | Hardening | DONE | Tempfile publication can replace generated files with owner-only permissions; closed by the SR3-04 repair in the Guide-8 preflight wave. |
| `SR2-H2` | Hardening | DONE | Dirty-tree document-reproducibility probe skips with exit 0; closed by the SR3-06 repair in the Guide-8 preflight wave. |

The DONE rows were repaired in the Guide-7 preflight wave with focused tests
and the working debug lane; the batch full gate is recorded with the Guide-7
exit record. Open-flow complete partitioning deliberately remains a
sponsor-isolation relation verdict rather than observation parsing, so an
unclaimed member stays an evaluable semantic failure. The two hardening rows
were parked with explicit activation conditions; the third review re-raised
both, which satisfies those conditions, so they are active again and tracked
with the §5.3 register.

### 5.3 Third-review findings · `tab:backlog:findings-sr3`

The third static review reported two passes over the tree recorded in §2.1.
First-pass findings carry SR3 identifiers; second-pass findings retain the
review's S2 identifiers.

| ID | Severity | Status | Finding |
|---|---|---|---|
| `SR3-01` | High | DONE | A malformed same-owner citation whose one-sided parenthesis test is suppressed by an unrelated parenthesis elsewhere on the line falls back to a bare occurrence and silently mints the label. Citations now classify by their immediate parenthesized group with a focused malformed-group diagnostic in both parsers; the previously test-locked vulnerable assertion was inverted; a corpus scan showed zero live classification changes. |
| `SR3-02` | Medium | DONE | Census filesystem traversal suppresses read-dir and entry errors, so an unreadable tree with an empty declared group can verify as an empty census. Traversal failures are now typed census-unreadable diagnostics consumed by verification, with unreadable-directory, failed-entry, and scoped-derivation regressions; container-nested fences are additionally rejected. |
| `SR3-03` | Medium | DONE | The complete analyzed-program validator is not on the production construction path; the constructor runs only the narrow assembly-closure check. The constructor now runs the complete fresh re-derivation validator, and the execution report is validated rather than excluded; the measured pilot construction cost stays seconds-scale. |
| `SR3-04` | Medium | DONE | Atomic publication staging leaves published public files with owner-only tempfile permissions; supersedes the parked SR2-H1. A typed publication-mode helper now sets 0644 or 0755 on the staged file before rename in batch publication, report writing, and the shell sync path, with absent, changed, and unchanged destination mode tests. |
| `SR3-05` | Medium | DONE | Git-derived checker census arguments are expanded unquoted, so an unusual tracked path can split or alter checker argv. A shared census-argument library now emits shell-quoted role-tagged argv re-read as quoted words, fails closed on an empty census, and a loud preflight audit rejects tracked paths outside a safe grammar; derived argv verified byte-identical on the live tree. |
| `SR3-06` | Medium | DONE | The document-reproducibility gate exits 0 after skipping its reused-build epoch probe on a dirty tree; supersedes the parked SR2-H2. The gate now fails a dirty tree by default, reserves exit 0 for both advertised checks passing, and reports an explicit partial status under an opt-in flag; fixing it exposed and repaired a signing-config defect that had prevented the epoch probe from ever running here. |
| `SR3-07` | Low | DONE | Proof-search rejection statistics count source and constructibility failures as capability rejections. Local feasibility now returns a typed rejection reason with a matching counter per cause; repairing it exposed a miscounted source-derivation branch and a counter with no increment site, both corrected. |
| `SR3-08` | Low | DONE | The package index still says complete analyzed pilots are open, contradicting the recorded Guide-7 state. The index row now states the implemented-internally claim with the public boundary and target adapter absent. |
| `S2-01` | High | DONE | Sponsor erasure is keyed to the ordinary L-BTC object family instead of fee-sponsor flow membership, erasing protocol-role amounts that future operations must read. The model and realization layers derive the sponsor region from exact fee-sponsor flow membership with two-point overlap rejection, protocol-role readability, and a kernel balanced-theft regression; the compiler derives a three-valued flow role from the declared open flows and refuses protocol-claimed ordinary L-BTC scopes with a typed error, so the remaining family-shaped guards rest on an enforced gate whose lifting condition is documented at each use; both pilot analyzed projections were verified byte-identical. |
| `S2-03` | Medium | DONE | The coverage dependency graph mixes opposite edge orientations against its documented prerequisite-to-dependent convention. All edges are normalized to enabler-toward-dependent with declared endpoint classes checked on insertion; a reversed prerequisite edge between same-class endpoints remains caught by census comparison, and that limit is stated. |
| `S2-04` | Medium | DONE | Analyzed-program coverage-graph validation compares sets rather than exact projections, silently accepting duplicated nodes and edges. Stored and required projections now compare exactly with duplicate, order, and length-preserving corruption regressions. |
| `S2-05` | Low | DONE | Exact-search counters use unchecked addition and can overflow instead of returning the promised typed complexity failure. A checked counter module tests the limit before incrementing under an inclusive root-counted rule, diagnostic counters saturate, and boundary regressions pin a budget of one, exact-limit completion, and limit-plus-one. |
| `S2-06` | Low | DONE | The paper subproject declares C as a project language with no C target, requiring an undocumented compiler. The language declaration is removed; setup no longer probes a C toolchain and the mocked contract passes. |
| `S2-07` | Low | DONE | Compiler planning status remains stale and a resolved factorization question is still listed as open, although the drift finding was marked closed. The resolved per-operation-factor decision is now recorded as a rule and removed from the open questions; the genuinely open boundary, ownership, objective, and identity questions remain. |

The Guide-8 preflight wave owned SR3-01 through SR3-08 and S2-01 through
S2-07; all were closed before the compiler target-requirement boundary
became public, gate record §2.12.

### 5.4 Fourth-review findings · `tab:backlog:findings-r2`

The fourth static review reported two passes over the tree recorded in §2.1.
Reconfirmed first-pass findings carry R2-C identifiers; second-pass additions
carry R2-N identifiers. The Guide-9 preflight owns the register; the review
ranks the target-contract rows as Phase-3 blockers before the typed
instruction core consumes the contracts.

| ID | Severity | Status | Finding |
|---|---|---|---|
| `R2-C01` | High | DONE | The opcode stack contract carries one success result sequence, so primitives with alternative valid success shapes, retained operands, or discriminated encodings cannot be represented. A typed success algebra now carries fixed, retained-operand, and condition-discriminated cases re-reviewed against the upstream interpreter with recorded line provenance; the retained timelock operand, issuance push order, and independently confidential issuance amounts are all representable, and the timelock contradiction with its resource row is repaired. |
| `R2-C02` | High | DONE | Redundant target subcontracts are only locally validated; signature, timelock, issuance, confidential-value, resource, and evidence views can contradict one another inside one validated definition. Six weld validators now require cross-view agreement with focused globally-contradictory mutation coverage and a welded-reviewed-contract control. |
| `R2-C03` | High | DONE | Capability assessment reads only direct primitive statuses, so a reviewed capability whose transitive prerequisite is unsupported still assesses as usable. Statuses now compose under an explicit strength order and a capability may not exceed its weakest transitive prerequisite; the reviewed registry verified closure-coherent with no downgrades, and the target validator rejects violations with the offending edge named. |
| `R2-C04` | Medium | DONE | Encoding numericity is derived from byte-order presence, making the byte-order validation circular and letting a numeric class validate without any order. Interpretation is now an exhaustive package-owned match over the encoding class with independent V1 shape expectations, making the missing and spurious byte-order rejections genuinely reachable. |
| `R2-C05` | Medium | DONE | The adapter consumes only the capability census and drops the compiler's external-evidence-role census, so a new role can disappear without adapter work. The assessment set now carries both censuses with exhaustive role matching, both-direction exact equality, typed duplicate and mismatch rejections, and an independent evidence-role oracle. |
| `R2-C06` | Medium | DONE | The active architecture semantic hash predates the domain-separated identity recipe; the exception or migration must be explicit, never a silent prefix change. ADR-016 now records a bounded grandfathered-recipe rule for the architecture semantic and anchor-set hashes, welded into the identity table; future identities keep the domain-separated form. |
| `R2-C07` | Low | DONE | The package index still marks the implemented target packages as planned, and the root README layout omits the current package set; paper end-marker comment drift is noted but deferred with the paper-edit obligations. The index rows and the root layout now state the active honest claims; the paper end-marker fixes remain deferred to the next deliberate paper edit. |
| `R2-N01` | High | DONE | A caller-assembled target definition can acquire the same validated wrapper as the built-in reviewed Elements definition, so the reviewed trust state is caller-assertable. A distinct reviewed-Elements wrapper is now constructible only by the first-party derivation or by exact typed equality with it; mutated-but-coherent definitions stay generic under public-API tests. |
| `R2-N02` | Medium | DONE | The adapter accepts a combined target-and-binding value but ignores the deployment half entirely, so assessments are invariant under activation, network, and resource-override differences. The assessment API now consumes the reviewed static definition and is named static; the combined value is no longer named anywhere in the adapter crate, and deployment-aware assessment remains deliberately deferred until a real consumer exists. |
| `R2-N03` | Medium | DONE | Public architecture identity functions accept unvalidated architecture values, so an invalid architecture can bear the active semantic hash recipe. The draft and release validators now return borrowing validated wrappers that are the only public path to the identity and publication functions; unchecked projections are crate-private for mutation tests, and an equivalence test plus the pinned gates prove no published identity moved. |
| `R2-N04` | Medium | DONE | Compare-if-changed publication checks bytes only, so mode-only corruption survives indefinitely, including a synced helper binary that lost its executable bit. Destination freshness now means bytes and required mode; mode-only mismatches are repaired in place without rewriting bytes, the repair is visible in the typed result, both sync scripts repair equal-byte destinations, and a mock-toolchain shell test runs the full matrix. |

### 5.5 Fifth-review findings · `tab:backlog:findings-sr5`

The fifth static review reported two passes over the tree recorded in §2.1.
The second pass consolidates the first: first-pass identifiers R5-01 through
R5-07 map into the SR5 register as SR5-01, SR5-02, SR5-03, SR5-06, SR5-08,
SR5-12, and SR5-13 respectively; the remaining SR5 rows are second-pass
additions. The Guide-10 preflight waves owned the register and closed
every row before prototype evidence was recorded; the gate record is
§2.14. Each row states the reviewed defect and the repair the batch
delivered.

| ID | Severity | Status | Finding |
|---|---|---|---|
| `SR5-01` | High | DONE | The public native-report gate validates only required evidence rows that happen to be present; an empty, row-deleted, or relabeled report census passes. Repair delivered: complete owner validation returning a validated report wrapper with exact duplicate-sensitive case and evidence-row censuses and recomputed statuses, dispositions, and summary. |
| `SR5-02` | High | DONE | Broad evidence rows pass when only a subset of their semantic claim has cases: issuance-absent cases complete issuance introspection, rejection-only signature cases complete signature semantics, explicit-form cases complete confidential encodings, and consensus resource cases complete relay-policy resource evidence. Repair delivered: typed claim-level evidence census beneath the broad requirement identifiers with exact required-claim coverage. |
| `SR5-03` | High | DONE | Native reports retain only the case ordinal, expected outcome, observed outcome, and status, omitting the exact script, stack, context, enforcement layer, leaf version, and expected resources; two fixture sets can produce indistinguishable reports. Repair delivered: complete canonical fixture projection embedded in each report row. |
| `SR5-04` | High | DONE | A development binding validated against one target definition can later combine with a different definition of the same contract version, because the binding retains only the version. Repair delivered: retain the validated target projection in the binding or introduce a reviewed-development-binding wrapper constructible only against the reviewed Elements definition. |
| `SR5-05` | High | DONE | The static signature model cannot represent the target's documented empty-signature and unknown-public-key-type behavior: exact-width operand types reject the empty form before the failure contract applies and exclude the succeeds-without-verification path. Repair delivered: operand alternatives with conditional failure semantics, plus native unknown-key vectors before signature evidence is complete. |
| `SR5-06` | Medium | DONE | Reported network and genesis identifiers are caller declarations copied through the pipeline; the executor never reports what chain it actually ran, and the recorded synthetic identifiers confirm the fields are run labels. Repair delivered: a typed executor environment observation compared against the validated binding before any case executes. |
| `SR5-07` | Medium | DONE | Relation identifiers are not generically welded to relation bodies; the kind vocabulary contains an unused member and lacks a member for expression predicates. Repair delivered: an exhaustive relation-identity validator deriving expected kind and subject from every body variant, with the kind vocabulary corrected. |
| `SR5-08` | Medium | DONE | Native executor provenance cannot express the executed tip, upstream base, and local-topic census that ADR-018 requires, and a checkout-HEAD fallback can misattribute a binary's revision. Repair delivered: separated adapter, node, and integration-provenance fields with no checkout fallback into the binary-reported revision. |
| `SR5-09` | Medium | DONE | Executor timeout kills only the immediate child, so a real adapter's node, temporary datadir, cookie, and inherited pipes can outlive the run. Repair delivered: process-group supervision with graceful-then-forced group termination and a descendant-retaining regression test. |
| `SR5-10` | Medium | DONE | Executor protocol lines are read into an unbounded buffer, so one unterminated line can exhaust memory before typed rejection. Repair delivered: explicit per-phase protocol record limits enforced with bounded reads. |
| `SR5-11` | Medium | DONE | Schema-2 deployment profiles can acquire a type named validated deployment release despite the documented production-blocking ABI gap. Repair delivered: split structural profile validity from production-release validity, with the latter unconstructible under schema 2. |
| `SR5-12` | Low | DONE | Blank protocol lines and trailing blank data are silently accepted despite the documented fail-closed protocol. Repair delivered: strict NDJSON framing with blank and trailing records rejected. |
| `SR5-13` | Low | DONE | Public status documentation is stale: the root README denies the instruction core and omits the conformance package, and the target package still claims no native evidence exists. Repair delivered: reconcile the statements with the package-boundary claim that the static crate owns requirements while development native evidence lives in the conformance package. |

The review also recorded three lower-severity observations to close alongside
the owning repairs: accepted responses are not protocol-shape validated
against the advertised handshake capabilities, public fixture construction
does little semantic context validation, and the Python adapter tolerates an
empty context script path where the fixture script is nonempty.

### 5.6 Sixth-review findings · `tab:backlog:findings-sr6`

The sixth static review reported two passes over the tree recorded in §2.1.
First-pass and second-pass findings are consolidated into one SR6 register
whose identifiers are the Guide-11 preflight identifiers `G11-R01` through
`G11-R14`, with the two hardening rows carried as `G11-H01` and `G11-H02`.
The Guide-11 preflight waves own the register; the summaries below are the
required dispositions recorded in that guide's own preflight table.

The review basis is the archived review's own tree, not the working tree: under the
guide's own rule each finding is a hypothesis until it is reproduced
against the working tree, and is then either fixed, disproved with a
typed argument, or reclassified with a narrower assurance claim. The
Wave-0 column records that adjudication. Wave 0 repaired nothing, so
every row opened TODO; Wave 1 closed the two claim-laundering rows.

| ID | Priority | Status | Wave 0 | Finding |
|---|---:|---|---|---|
| `G11-R01` | P0 | TODO | CONFIRMED | A transcript can be rebound to another fixture census, prototype matrix, target, or deployment binding; bind the transcript to its exact subjects and requests. |
| `G11-R02` | P0 | DONE | CONFIRMED | Primitive claims can be manufactured by attaching claim-bearing case metadata to an unrelated script; gate only a canonical validated primitive plan. |
| `G11-R03` | P0 | DONE | CONFIRMED | Prototype claims are caller-authored and can certify a trivial true script as constructor or wide-floor evidence; gate only relation-specific canonical matrices. |
| `G11-R04` | P1 | TODO | CONFIRMED | Consensus resource cases are credited to policy-resource evidence because evidence ownership derives from case ID without the enforcement layer; derive evidence from the complete fixture and correct the plan class. |
| `G11-R05` | P1 | TODO | CONFIRMED | The primitive native gate can accept a report whose summary is failed; gate every canonical case and reject failed completeness. |
| `G11-R06` | P1 | TODO | CONFIRMED | ADR-018 execution provenance is recorded but not enforced by evidence gates; validate executable provenance before gate eligibility. |
| `G11-R07` | P1 | TODO | CONFIRMED | Meson defaults an executor to reviewed-non-mock; require explicit caller selection and fail closed. |
| `G11-R08` | P1/P2 | TODO | CONFIRMED | Relation bodies can admit several semantic relation subjects; derive exactly one canonical subject from each body. |
| `G11-R09` | P1/P2 | TODO | CONFIRMED | Target V1 is advertised as supported while current validation applies the V2 census and algebra; remove V1 support or implement genuine version dispatch. |
| `G11-R10` | P1/P2 | TODO | CONFIRMED | The signature weld omits unknown-key behavior and much of the success algebra; weld the complete signature relation. |
| `G11-R11` | P2 | TODO | CONFIRMED | Bare prototype report digests persist despite the recorded no-report-identity decision; remove them or admit typed retained report references. |
| `G11-R12` | P2 | TODO | RECLASSIFIED | Current backlog state contradicts recorded Guide-8 through Guide-10 completion and duplicates finding IDs; reconcile current state and enforce unique IDs. |
| `G11-R13` | P2/P3 | TODO | CONFIRMED | Process-group establishment failure can leave the direct child unreaped; kill and reap on every pre-supervisor failure. |
| `G11-R14` | P3 | TODO | CONFIRMED | Constructor retry retries internal-key defects no metadata nonce can repair; share a typed retryability predicate. |
| `G11-H01` | Hardening | TODO | CONFIRMED | Opcode resource stack-growth rows are not generically welded to success and non-aborting failure effects; derive and compare exact maximum stack growth. |
| `G11-H02` | Future blocker | TODO | CONFIRMED | Issuance observations carry authority fields the realization evaluator does not yet enforce; add no issuance realization scope until enforced or externally evidenced. |

Wave-1 evidence for the two closed rows. `G11-R02`: the evidence path
takes only `CanonicalPrimitiveFixtureSet`, whose one constructor is
`canonical_fixture_set`, and `evaluate` regenerates the census and
compares complete projections — asserted by
`g11_r02_a_trivial_true_script_cannot_bear_signature_evidence`,
`g11_r02_an_arbitrary_census_is_refused_on_provenance_not_completeness`,
`g11_r02_changing_a_canonical_fixture_member_removes_gate_eligibility`,
`g11_r02_a_canonical_case_whose_claims_change_is_refused`, and
`g11_r02_permuting_canonical_declaration_order_is_harmless`. `G11-R03`:
`prototype_gate` reads only a report validated against a
`ConstructorPrototypeMatrix` or `WideFloorPrototypeMatrix`, whose rows are
private and whose one constructor each is the canonical generator —
asserted by `g11_r03_a_trivial_leaf_cannot_certify_the_wide_floor_relation`,
`g11_r03_the_canonical_wide_floor_matrix_is_the_evidence_subject`,
`g11_r03_a_canonical_case_with_one_added_claim_fails`,
`g11_r03_a_canonical_case_with_one_removed_claim_fails`,
`g11_r03_replacing_the_canonical_program_under_the_same_case_name_fails`,
`g11_r03_replacing_the_constructor_successor_program_fails`, and
`g11_r03_raw_bytes_are_not_reported_as_a_typed_program`.

Wave 1 also closed the arbitrary-census route that two open rows were
reproduced through. `G11-R01`'s script-substitution reproduction now runs
on the experimental path and reproduces the same defect. `G11-R05`'s
construction added a fixture to the canonical census and is no longer
expressible; its test records the closed route and the still-open gate
defect, and Wave 3 owns finding a route that reaches the gate itself.

No public-declassification prototype begins while `G11-R01` through
`G11-R10` remain open. `G11-R12` owns the full reconciliation of this
backlog's current state against the recorded Guide-8 through Guide-10
completions; that reconciliation is deliberately not performed by the
chartering import that opened this register.

Wave-0 evidence, one pointer per row. The Rust pointers name the
crate-internal `guide11_reproductions` suites, which pass by asserting the
defective behaviour and which the repairing waves invert.

```text
G11-R01  conformance g11_r01_a_transcript_rebinds_to_a_deployment_it_never_ran_on
G11-R02  conformance g11_r02_a_trivial_true_script_bears_signature_claims
         (Wave 1 flipped: g11_r02_a_trivial_true_script_cannot_bear_signature_evidence)
G11-R03  conformance g11_r03_a_trivial_leaf_certifies_the_whole_wide_floor_relation
         (Wave 1 flipped: g11_r03_a_trivial_leaf_cannot_certify_the_wide_floor_relation)
G11-R04  conformance g11_r04_consensus_resource_cases_pass_the_policy_resource_row
G11-R05  conformance g11_r05_the_gate_accepts_a_failed_report
G11-R06  conformance g11_r06_the_gate_accepts_a_run_with_no_workspace_provenance
G11-R07  meson.options target_native_executor_class, value reviewed-non-mock
G11-R08  realization g11_r08_one_closure_body_admits_two_subjects
G11-R09  target-elements g11_r09_a_v2_body_stamped_v1_validates
G11-R10  target-elements g11_r10_a_contradictory_unknown_key_rule_still_validates
G11-R11  plans/research/state-constructor.md and wide-arithmetic.md, report digest rows
G11-R12  duplicate DI-F02 rows, DI-F03 active under a DONE parent, and section 3.3
         still listing both accepted prototypes as not implemented; the opening
         condition line is already reconciled, so the finding narrows to those three
G11-R13  executor.rs ExecutorSupervisor::adopt takes the child by value and drops it
         on the establish failure path, and the standard child destructor neither
         kills nor waits
G11-R14  conformance g11_r14_an_invalid_internal_key_is_retried_to_exhaustion
G11-H01  no weld reads OpcodeResourceCost::maximum_stack_growth at all, the timelock
         weld included, so the field is welded to nothing
G11-H02  ObservedIssuance::authority is never read in realization, and authority_input
         only for referential existence in observation.rs
```

### T6 — Validate capabilities for external-evidence obligations · `task:review:external-evidence-capability`

**Priority:** P1
**Status:** DONE

Reproduction: with every capability available except whole-transaction value
conservation, production planning returned feasible plans for both pilots
(two compact-ASH and four live-transfer candidates) that carried the
substrate-conservation evidence requirement while omitting the conservation
capability entirely.

Repair: the external-evidence obligation class now retains its approved proof
alternative, required capabilities, and source requirements. Classification
validates that the realization approves exactly one substrate-conservation
alternative, with a typed error for any other shape. Enumeration tests the
fixed capability set against the capability view before the search begins and
seeds every candidate from the fixed requirements. The evidence requirement
remains unresolved, and no sponsor amount enters any fixed requirement.

Focused tests, in the compiler proof suite:

- `a_missing_whole_transaction_capability_blocks_substrate_conservation`
- `adding_the_whole_transaction_capability_restores_feasibility`
- `external_evidence_carries_its_capability_and_source_into_every_candidate`
- `fixed_external_requirements_name_no_sponsor_amount`
- `an_unexpected_external_evidence_alternative_is_rejected`

### T7 — Constrain every representation-sensitive proof choice · `task:review:representation-proof-compatibility`

**Priority:** P1
**Status:** DONE

Reproduction: unconstrained live-transfer planning returned four candidates,
two of which paired a selected mode with a contradictory arithmetic proof on
the representation relation itself (private committed with public arithmetic,
and explicit with confidential conservation).

Repair: representation relations are declared relation-only in both pilot
declarations, with the approved mode sets unchanged, and the compiler
classifies them as statically validated alongside lifecycle exits. The
conservation relation keeps exact proof/mode compatibility. Live transfer now
yields exactly one candidate per approved mode, and the selected mode remains
in each candidate's representations for later placement and backend work.

Focused tests, in the compiler proof suite:

- `representation_relations_carry_no_proof_variable`
- `live_transfer_conservation_agrees_with_every_selected_mode`
- `compact_ash_keeps_both_modes_under_public_arithmetic`
- `every_pilot_relation_is_classified_exactly_once`, extended

The independent oracle no longer calls production obligation classification.
It derives the substrate-conservation capability and its typed external
source row itself, creates no proof variable for representation relations,
and validates fixed requirements before enumeration. New oracle regressions
cover the missing whole-transaction capability across all scopes, one
candidate per live mode, and random capability views over all thirteen
capabilities, while still distinguishing infeasibility from unexpected
errors.

### T8 — Weld every current-phase declaration · `task:review:phase-declaration-drift`

**Priority:** P2
**Status:** DONE

Repair: the plans README declares Phase 2 in the fixed declaration form. The
plan checker now compares the backlog current gate, the roadmap current
phase, the plans README current phase, and exactly one active numbered phase
card, normalized to the numeric phase. Diagnostics are order-stable and name
the stale file, the observed phase, and the expected phase; malformed or
missing declarations fail explicitly.

Focused tests: ten phase-weld tests in the labels plans module covering
agreement, each stale copy, zero and two active cards, malformed and missing
declarations, a gate without a phase card, and traversal-order independence.

### T9 — Refresh compiler status documentation · `task:review:compiler-status-drift`

**Priority:** P2
**Status:** DONE

Repair: the compiler crate Rustdoc lists the implemented internal stages and
the remaining absent work, and keeps the boundary statement that no public
value can be mistaken for a completed analysis. The compiler package
contract and the packages index carry an accurate status and milestone
states, and the phase card separates delivered-analysis requirements from
the open placement, coverage, and pilot deliverables. The stale-phrase sweep
found two current-state matches; both were corrected.

Verification: compiler package tests, a warning-free Rustdoc build, and the
plans check all passed.

---

## 6. Phase-2 implementation queue · `sec:backlog:phase2`

### 6.1 Summary · `tab:backlog:phase2`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `P2-001` | P1 | DONE | Immutable, canonical, ownership-validated realization boundary |
| `P2-002` | P2 | DONE | Petgraph dependency and lockfile review |
| `P2-003` | P1 | DONE | Compiler crate boundary and typed error root |
| `P2-004` | P1 | DONE | Bind architecture, realization, policy, and explicit scope |
| `P2-005` | P1 | DONE | Canonical compiler relation and expression DAGs |
| `P2-006` | P1 | DONE | Checked constant folding |
| `P2-007` | P1 | DONE | Exact proof planning; reaccepted after T6 and T7 closed, gate record §2.8 |
| `P2-008` | P1 | DONE | Disclosure, source, and constructibility analysis |
| `P2-009` | P1 | DONE | Representation lifecycle analysis |
| `P2-010` | P1 | DONE | Execution-case placement and layout requirements; gate record §2.9 |
| `P2-011` | P1 | DONE | Relation-indexed coverage requirements; gate record §2.10 |
| `P2-012` | P1 | DONE | Compact-ASH and live-transfer analyzed pilots; gate record §2.11 |
| `P2-013` | Gate | DONE | Complete Phase-2 evidence and exit; gate record §2.11 |

### 6.2 Completed foundation

The following are implemented internally and remain subject to their package
tests and owner validation:

```text
P2-001  realization boundary
P2-002  Petgraph review
P2-003  compiler crate and error root
P2-004  compiler input binding
P2-005  relation/expression DAGs
P2-006  checked constant folding
P2-007  exact proof planning, reaccepted
P2-008  source, constructibility, and disclosure analysis
P2-009  representation lifecycle
P2-010  execution-case placement and layout requirements
P2-011  relation-indexed coverage requirements
```

No complete analyzed-program public API is exposed yet. This is deliberate:
partial analyses must not be mistaken for a completed compiler result.

### P2-007 — Repair and reaccept exact proof planning · `task:phase2:proof-planning`

**Priority:** P1
**Status:** DONE
**Depends on:** T6, T7, P2-005, P2-006
**Blocks:** P2-010 through P2-013

Reaccepted after T6 and T7 closed; the gate record is in §2.8.

Reacceptance required, and the recorded evidence establishes:

1. every in-scope relation classified exactly once;
2. every approved proof alternative retained or rejected for a typed reason;
3. external-evidence capabilities validated;
4. proof/representation compatibility applied completely;
5. capability and source requirements consistent;
6. hard constraints applied before objective or candidate retention;
7. exact feasible set compared with an independently derived exhaustive oracle;
8. no partial result on complexity exhaustion;
9. deterministic candidate ordering and equality under declaration
   permutations.

The search retains the exact feasible set; it does not silently select one
candidate under an unstated target policy.

### P2-010 — Derive execution-case placement and layout requirements · `task:phase2:placement`

**Priority:** P1
**Status:** DONE
**Depends on:** accepted P2-007, P2-008, P2-009, C1-009
**Blocks:** P2-011 and P2-012

Delivered in the Guide-5 batch; the gate record and evidence are in §2.9.

Relations are classified per execution case on independent axes — discharge
boundary, semantic scope, activation, and carrier multiplicity — rather than
one local/global/conditional/duplicated category. Execution cases expand only
the sponsor dimension; the representation is fixed by each feasible proof
plan. Every active runtime relation case has an eligible abstract carrier;
carrier eligibility, exact feasible placement sets, and target-independent
layout requirements are validated against an independent exhaustive oracle.
Complexity limits are explicit typed parameters and exhaustion returns no
partial result. No target bytes or concrete target positions enter this
stage.

### P2-011 — Derive relation-indexed coverage requirements · `task:phase2:coverage`

**Priority:** P1
**Status:** DONE
**Depends on:** P2-010, C1-010, C1-013
**Blocks:** P2-012

Delivered in the Guide-6 batch; the gate record and evidence are in §2.10.

Exact equality holds among the realization relation scope, the compiler
relation scope, the relation-case plan scope, and the coverage scope. Every
active relation-case carries positive and negative requirements at each of
its boundaries, a carrier requirement when runtime-carried, and an accepted
semantic-projection requirement; inactive relation-cases carry an explicit
inactive-valid requirement; conditional relations satisfy the
inactive-valid, active-valid, active-invalid triplet across the case set,
and representation modes are covered across the complete feasible plan
set. A broad operation test does not substitute for relation coverage.

### P2-012 — Analyze both pilots end to end · `task:phase2:pilots`

**Priority:** P1
**Status:** DONE
**Depends on:** T6, T7, P2-005 through P2-011
**Blocks:** P2-013

Delivered in the Guide-7 batch; the gate record and evidence are in §2.11.

Both pilots analyze end to end into one validated scoped analyzed program
per scope: the acceptance relation lists are the exact relation censuses,
every requirement in the acceptance matrices holds, rejected proof and
representation pairings are absent from the plan set, repeated analysis
from equal typed inputs produces equal stable projections, and the
combined two-pilot scope stores per-operation factors whose product
equals the retired global enumeration exactly. The analyzed value claims
no deployment lifecycle completeness: the pilots retain their future
target and operation obligations explicitly.

### P2-013 — Phase-2 evidence and exit · `gate:backlog:phase2`

**Priority:** Gate
**Status:** DONE
**Depends on:** T6–T9, P2-004 through P2-012, C1-005, C1-008,
C1-009, C1-010, C1-013

Passed; the complete exit evidence is the Guide-7 gate record in §2.11,
including the declared-MSRV and current-stable lanes, the meson-required
CI run, the explicit product regressions, and passed document byte
reproducibility. Every exit condition held: findings closed, censuses
exact, capabilities failing closed, deterministic pilot analysis,
independent oracles agreeing, gates recorded, current-phase declarations
moving together, and a clean final tree.

Phase completion does not itself justify a persistent compiler digest;
none was minted.

---

## 7. Immediate algorithm preparation · `sec:backlog:algorithms`

### 7.1 Current status · `tab:backlog:algorithms`

| ID | Status | Deliverable |
|---|---|---|
| `C1-001` | DONE | Compiler/linker/mathematics/solver research notes |
| `C1-002` | DONE | Direct Petgraph decision |
| `C1-003` | DONE | Exact/certified mathematics decision |
| `C1-004` | DONE | Petgraph dependency review |
| `C1-005` | DONE | Canonical direct-Petgraph compiler graph prototype |
| `C1-006` | PARKED | Exact keyed linear systems until a consumer exists |
| `C1-007` | PARKED | Certified numerical analysis until a consumer exists |
| `C1-008` | DONE | Exact proof-plan search; reaccepted after T6/T7, gate record §2.8 |
| `C1-009` | DONE | Execution-case-aware placement; gate record §2.9 |
| `C1-010` | DONE | Typed symbol resolution and SCC policy in coverage scope; gate record §2.10 |
| `C1-011` | BLOCKED | Structured relocation; linker phase |
| `C1-012` | BLOCKED | Deterministic bounded-depth target tree; linker phase |
| `C1-013` | DONE | Independent small-instance oracles; placement oracle §2.9, coverage/collateral/SCC oracles §2.10 |
| `C1-014` | DONE | Preparation review and Phase-2 handoff; gate record §2.11 |

### 7.2 C1-008 — Proof-plan search reacceptance

Reaccepted in the Guide-4 batch, gate record §2.8. The recorded evidence
establishes each former condition:

- external-evidence capability requirements are retained;
- representation-sensitive proofs are checked against selected modes;
- candidate capabilities and sources agree;
- the independent oracle derives those rules independently;
- infeasibility and complexity exhaustion remain distinct;
- no partial result is returned.

### 7.3 C1-009 — Execution-case-aware placement

Delivered in the Guide-5 batch with P2-010; the gate record is in §2.9.
Every active relation case is covered by an eligible carrier, production
search is compared with an independent exhaustive assignment oracle on
small instances and both pilot scopes, and the adversarial case list —
optional-carrier defects, global-on-local defects, source unavailability,
duplicate enforcement, permutation equality, and both budget exhaustions —
is retained as focused regressions.

### 7.4 C1-010 — Typed symbols and SCC policy

Delivered in the Guide-6 batch within the coverage scope; the gate record is
in §2.10. Two-pass typed resolution runs a complete definition census before
complete reference resolution; duplicate keys, unknown references, and
cross-operation references fail; SCC members and diagnostics are normalized
by stable key and every current cycle is rejected. SCC membership does not
authorize a semantic cycle — an accepted cycle would require a new typed
resolution strategy and policy review. This work prepares linker policy but
pulled no linker or target types into compiler core.

### 7.5 C1-013 — Independent algorithm oracles

| Production analysis | Independent oracle |
|---|---|
| canonical topology | valid-order enumeration plus least-key rule |
| SCC | mutual-reachability equivalence |
| expression interning | non-interned evaluator |
| dependency closure | repeated complete scan |
| proof selection | exhaustive candidate enumeration with independently derived hard constraints |
| placement | exhaustive carrier subsets |
| relation census | direct set equality |
| constant folding | non-folded evaluator |
| stable projection | declaration-order permutation |
| coverage | direct relation × case matrix equality |

The proof-selection oracle must not call the same obligation-classification or
compatibility helper as production for the property it is meant to verify.

Delivered across the Guide-5 and Guide-6 batches: the placement oracle in
§2.9; the relation-by-case coverage oracle, the repeated-scan collateral
oracle, and the mutual-reachability SCC oracle in §2.10. Each restates its
hard predicates independently of the production helper it verifies.

---

## 8. Dependency posture · `sec:backlog:dependencies`

### 8.1 Current and deferred dependencies · `tab:backlog:dependencies`

| Dependency | Status | Role |
|---|---|---|
| `petgraph = 0.8.3` | Adopted and reviewed; `serde-1` only | Graph storage and standard algorithms |
| `num-bigint` | Existing | Exact arbitrary-size integers |
| `num-integer` | Existing | Exact integer helpers |
| `num-traits` | Existing | Numeric traits |
| `proptest` | Existing | Property and independent-oracle testing |
| `num-rational` | Deferred | Future exact-rational consumer |
| `faer` | Deferred | Future certified numerical diagnostics |
| `fixedbitset` | Deferred | Dense local coverage sets if measured |
| Elements libraries | Phase-3 review | Target transaction and consensus types |
| SAT/LP/MILP solver | Deferred | Larger exact planning only after measured need |
| `salsa` | Deferred | Incremental compiler queries |
| `egg` | Deferred | Equality saturation |
| `rayon` | Removed and deferred | Requires measured need and schedule-independent results |

The latest static review excluded `Cargo.lock`; no lockfile claim is made by
that review.

### 8.2 Dependency-entry rule · `rule:backlog:dependency-entry`

A deferred dependency enters only when:

1. a concrete consumer exists;
2. current implementation demonstrates missing functionality;
3. simpler exact first-party code is insufficient;
4. source, version, licence, MSRV, unsafe boundary, transitive graph,
   determinism, and advisories are reviewed;
5. public API leakage is considered;
6. focused tests and an independent oracle exist;
7. lockfile changes are reviewed;
8. required gates remain green and clean.

Unused dependencies are not added to advertise intent.

---

## 9. Mathematical and algorithmic laws · `sec:backlog:laws`

### 9.1 Exactness · `rule:backlog:exactness`

Semantic, conservation, authorization, identity, calibration, and release
claims use:

- checked bounded integers;
- arbitrary-precision integers;
- reduced exact rationals;
- exact finite search;
- independently checked certificates;
- target-native execution where the claim concerns the target.

A floating residual is diagnostic evidence, not exact equality.

### 9.2 Handles and identity · `rule:backlog:handles`

Always distinguish:

```text
local handle:
    process-local graph, arena, matrix, or solver position

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

### 9.3 Complexity failure · `rule:backlog:complexity`

An analysis exceeding its explicit budget returns a typed complexity error.

It must not:

- drop a relation;
- weaken authorization;
- increase disclosure silently;
- remove a lifecycle exit;
- switch to an undocumented greedy fallback;
- claim optimality from incomplete search;
- accept the best partial result seen before exhaustion.

### 9.4 Evidence separation · `rule:backlog:evidence-separation`

Keep separate:

- typed validation;
- capability availability;
- witness/source availability;
- model execution;
- realization conformance;
- compiler completeness;
- backend pattern evidence;
- linked-bundle execution;
- target consensus/policy results;
- event projection;
- attestation query;
- receipt accounting;
- calibration;
- release validation.

A report or status from one class never silently satisfies another.

In particular:

```text
target capability exists
≠
required evidence was produced

sponsor role structure is valid
≠
substrate conservation was evidenced

model relation is conformant
≠
deployment is ready
```

---

## 10. Verification matrix · `sec:backlog:verification`

### 10.1 Working cadence

For Rust changes:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Do not run the full Meson/release surface after every small edit.

Documentation-only changes do not require the Rust lane, but they still require
the relevant documentation and census checks.

### 10.2 Focused commands · `tab:backlog:focused-tests`

| Area | Command |
|---|---|
| architecture/profile | `cargo test -p tripod-architecture` |
| realization | `cargo test -p tripod-realization` |
| compiler | `cargo test -p tripod-compiler` |
| compiler planning | `cargo test -p tripod-compiler proof` |
| compiler oracle | `cargo test -p tripod-compiler oracle` |
| model conformance | `cargo test -p tripod-model realization_conformance` |
| model bounds | `cargo test -p tripod-model bound_conformance` |
| artifacts | `cargo test -p tripod-artifacts` |
| labels/plans | `cargo test -p tripod-labels` |
| checker report/stamps | `cargo test -p cli-common` |
| document stamps | `cargo test -p tripod-document-stamps` |
| flattener | `cargo test -p flatten-latex-main` |
| process wrapper | `cargo test -p execwrap` |
| mocked Meson graph | `scripts/test-meson-mock.sh .` |

Focused filters supplement but never replace complete package and workspace
runs.

### 10.3 Full gate

After a completed batch:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

The canonical build directory is `build/`.

Use:

```sh
meson setup build
```

only if `build/` does not exist.

For a complete release-oriented result, additionally run the required MSRV and
stable lanes and the separate byte-reproducibility check:

```sh
scripts/check-document-reproducibility.sh
```

### 10.4 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new tracked subject must join its nearest `meson.build` census.

### 10.5 Dependency and advisory evidence

```sh
cargo tree -e features
cargo metadata
cargo audit
```

A missing `cargo-audit` is recorded as skipped, never passed.

### 10.6 Clean repository

The final check is:

```sh
git status --porcelain=v1 --untracked-files=all
```

It must be empty.

### 10.7 Execution trust

Repository source, tests, Cargo manifests, Meson definitions, scripts, TeX,
and `.latexmkrc` are executable.

Untrusted contributions run only in an externally established,
credential-free isolated environment under ADR-015. A clean-tree result is a
correctness check, not malicious-code containment.

---

## 11. Current gate · `gate:backlog:current`

The Phase-2 gate is **passed**; the record is §2.11. The current gate is
Phase 3 — Elements target and foundational prototypes.

The Guide-8 target foundation is complete; the gate record is §2.12. The
Phase-3 task state is:

| ID | Status | Task |
|---|---|---|
| `T3-001` | DONE | Typed target package boundary |
| `T3-002` | DONE | Reviewed initial target definition |
| `T3-003` | DONE | Encoding and opcode contracts |
| `T3-004` | DONE | Development deployment binding |
| `T3-005` | DONE | Compiler capability adapter |
| `T3-006` | DONE | Target-native primitive conformance, gate record §2.13 |
| `T3-007` | DONE | Typed tapscript instruction foundation, gate record §2.13 |
| `T3-008` | DONE | STATE-constructor prototype accepted, gate record §2.14 |
| `T3-009` | DONE | Exact wide floor-arithmetic prototype accepted, gate record §2.14 |

Current blockers are:

```text
Phase-3 work:
    public declassification prototype
```

Standing rules that survive the phase exit:

- Phases 1 and 2 remain evidence about their recorded trees;
- no compiler public complete-analysis API is frozen;
- no public realization/compiler digest is minted without a real consumer;
- target-specific fields remain forbidden in realization and compiler core;
- no stable linker or transaction ABI exists;
- no floating value or local graph handle enters semantic identity;
- no draft bound becomes deployment calibration;
- no raw report digest is treated as evidence identity without typed role and
  subject binding;
- no self-consistent model checkpoint is described as independent
  target-chain evidence;
- no architecture, model, realization, hash, build, or test success is
  described as deployment readiness;
- no current checkout is described as green without a fresh complete execution
  record.

---

## 12. Execution order · `sec:backlog:order`

Execute in this order unless reproduction changes dependencies:

```text
1. Charter the public-declassification prototype, the remaining
   Phase-3 foundational question; the constructor and wide-floor
   prototypes are accepted with gate record 2.14.
2. Keep target types out of compiler core; the target packages consume
   the analyzed boundary, never the reverse.
3. Mint no target hash and claim no production activation without a real
   consumer and reviewed evidence.
```

No new hash, target prototype, report field, or publication may defer a current
typed-boundary or correctness repair.

---

## 13. Adopted-draft integration queue · `sec:backlog:drafts`

Four externally authored normative drafts were accepted 2026-08-16, audited (28 findings), corrected upstream by the author, and re-adopted and
archived verbatim under [plans/drafts/](drafts/README.md): the label
calculus, the environment-kind registry, the identity-adjudication
procedure, and the interchange conventions. Integration is chartered as
its own batch; the drafts bind nothing until the integrating changes
land. The user has ruled that a superseded ADR is deleted, not kept
marked superseded: retired records belong to Git history, not the
active tree.

### 13.1 Integration tasks · `tab:backlog:draft-tasks`

| ID | Status | Task |
|---|---|---|
| `DI-001` | ACTIVE | Gap census: map each draft clause onto the present ADR-012/ADR-013 text, the ADR-016 identity rules, and the implemented labels package; classify every clause as already-implemented, divergent, or new; record the checker-engineering findings register. |
| `DI-002` | DONE | Adopt the corrected label calculus and kind registry as ADRs; delete the ADR text they retire; migrate the eight conflicting kind tokens to the registry forms corpus-wide per the user's ruling, with every citation updated in the same commit; record the hyphenated-area amendment and the local-extension register. |
| `DI-002b` | DONE | Replace the second-edition adopted drafts with the author's third-edition texts and repair every citation that dangled as a result. The third edition was verified against the audit register before the swap: all 26 findings fixed, the 7 defects among them included, and a mechanical re-audit clean — every per-document citation resolves, every mint is unique, and the kind registry's headline counts of 333 names, 349 rows, 208 kinds, 3 declared hybrids and 4 device classes derive exactly from its tables. Fourteen citations across three plan files were retargeted in the same commit as the swap, per the calculus's same-commit rule. Both adopting records were then refreshed to the editions they adopt: ADR-019 restates all seven adoption parameters and records that the checker implements the authorship warrant species only, ADR-020 names this repository as the registry's acceptee and recasts its extension register as the recorded extension set with located first-hand evidence, and a second kind-migration round settled two further tokens. |
| `DI-003` | DONE | Re-engineer the labels checker to the calculus: a single participation scanner shared by every check (mints, citations, links, hygiene, inline-code discipline), owner signatures with registered prefixes, imported and synthetic citations, anchor harvests, and the kind registry as the checker's kind vocabulary. W1 landed the participation scanner and the DI-F02 repair, W2 the seven adoption parameters as typed data with the kind vocabulary and warrant totality, and W3 the near-miss warnings, the companion attestation register, and the gate reconciliation across both records. |
| `DI-004` | DONE | Adopt the identity-adjudication procedure against ADR-016: classify every existing digest through the benefit criterion with admission records, and MIGRATE the two grandfathered recipes to domain-separated forms per the user's ruling — the architecture semantic and anchor-set hashes change under a recorded recipe migration, superseding the ADR-016 grandfather clause. Both halves delivered: the migration, and a census of the whole tree read from the owning code, recorded as six admission records and eleven stop records in the identities register. |
| `DI-005` | TODO | Interchange conventions: record adoption as the standing wire-format discipline for future externally consumed documents; no implementation until a consumer exists. Draft refreshed 2026-08-18 to the forward-compatibility edition: ceiling-based acceptance with downward-closed holding, open-companion tolerant validation, never-assigned stamps rejected as checkably false claims; the fourth-edition audit's two defects and one editorial finding all fixed on resupply, delta purely additive, nothing dangling. |
| `DI-006` | DONE | Three-part labels: the paper's 16 two-segment labels take the area `attestation`, a division's home being the document itself; `abs` takes the registry's `abst` per the user's ruling. 16 mints and 59 sites moved in one commit; the anchor-set pin moved……, retired value reproduced; the semantic hash followed, the behavioural did not. Layer-0 kinds enforced. W-B holds the arity in one rule over every entry point: a label-intended occurrence that is not three-part fails as `malformed_label_shape`, and the realization's 20 two-segment divisions are frozen by name, not exempted as a surface. |

The user's integration rulings, 2026-08-16: kind-token conflicts migrate
to the registry forms; the two grandfathered identity recipes migrate
now rather than persisting as a recorded divergence; the area grammar is
adopted with a recorded amendment admitting hyphens; per-package owner
prefixes are registered as checker data in DI-003. DI-002 landed
ADR-019 and ADR-020, deleted the retired records, migrated six of the
eight conflicts (task and res adjudicated as genuine local extensions),
and re-pinned the three identities the label rename moved: the Layer-0
anchor-set hash, the architecture semantic hash, and the
deployment-profile identity, each with the retired pin reproduced
before the new one was taken. The Layer-0 LaTeX surface keeps three
unadjudicated tokens (motto, a true collision; invest; abs), recorded
in ADR-020.

DI-002b's swap carried three consequences beyond the citation repairs.
The calculus's third edition adds a derivation authority the checker has
no notion of — a profile signature, a reserved-kind set, two warrant
rules, warrant totality and inventory discipline — which is now the
largest unmet item of its implementation gate and belongs to DI-003. The
gap census's postcondition checklist was reframed as a gate checklist,
both the calculus and the registry having moved to Gates. And this
record's own label-calculus gate was renamed from the implementation
name to the adoption name: the third edition mints the implementation
text for its own gate, and although the owner-keyed registries keep two
owners minting one text from being a duplicate, one text naming two
gates is a reader hazard the record yields on.

A second kind-migration round followed the swap. The Layer-0
verification appendix minted three walkthrough anchors under the token
ver, which the adopted registry assigns to Version and Revision;
verification is verif. The three mints in
`papers/attestation/sections/A1_verification.tex` moved to the verif
token, and the generated Layer-0 register followed in the same commit,
six sites in two files and no other occurrence anywhere in the tree.

The identity consequence was measured rather than assumed. The Layer-0
anchor-set recipe was reproduced from the pre-change tree over the
realization contract's thirty-eight cited anchors and returned the
pinned value exactly, which is what validates the reproduction; none of the three
renamed labels is in that set, because the contract cites the
walkthrough section but never these three sub-anchors. The same
recomputation after the rename returns the same value, so nothing was
re-pinned: the ceremony ran and its answer was that the anchor set did
not move. No welded witness semantic tag carries a walkthrough anchor
either, so the architecture semantic hash and the deployment-profile
identity are likewise untouched.

DI-002b closes with the refresh of the two adopting records. ADR-019
now fixes all seven adoption parameters the third edition asks for: the
owner signature, the owner partition by tree location, an empty profile
signature, an empty reserved-kind set, the designated typed-data
classes, the two citation-index designations, and the scanned-region
recognition for Markdown, Rust, and LaTeX. The profile signature and the
reserved-kind set are empty together on purpose — a reserved kind no
profile governs admits neither warrant rule, so a nonempty reserved set
under an empty profile signature would reserve kinds nobody could use —
and the record states plainly that the checker implements the authorship
warrant species alone. ADR-020 names this repository as the registry's
acceptee, recasts its register as the recorded extension set in the
edition's own terms with a located occurrence behind each of the
thirteen entries, and adjudicates the six keeps by genre: task against
the registry's exercise and code-asset readings of Task, res against
Result, trap against Pitfall, obl against Requirement, err against
Erratum, and pin against Version. Four tokens left the set because the
third edition carries them as rows of its own — pkg, q, req, and test —
with no mint moved. Postcondition and Requirement remained registry
rows, so the round-one decisions on post and req stand as taken.

Four residual items outlive the close, none of them DI-002b's to
discharge. The derivation-warrant machinery — a profile signature, a
census, standard places, inventory discipline, and the half of warrant
totality that governs reserved kinds — is unbuilt, as are the near-miss
warnings the calculus asks for, the general form of synthetic-citation
totality, and the registry's companion attestation register with its
homonym view; all of these are DI-003, and ADR-019 and ADR-020 record
them as unmet in their own terms. The identity census with admission and
stop records is DI-004. The interchange conventions stay dormant until a
consumer exists, which is DI-005. And the Layer-0 LaTeX surface keeps
three unadjudicated tokens under ADR-020 — motto, a true collision;
invest; and abs — which enter scope with that surface and not before.

DI-004 opened with the recipe migration the user ruled on, folded into
one recorded migration covering both identities. The retired values were
reproduced before anything moved, and reproduced independently of the
repository's own Rust: the architecture semantic recipe recomputed from
the committed manifest body under sorted-key compact JSON returned the pinned value, and
the Layer-0 recipe recomputed over the thirty-eight anchors harvested
straight from the realization document returned the pinned value. Both
matched the pins exactly, which is what licensed the migration to
proceed.

Each recipe then took a domain-separation prefix folding in its own
identifier, the idiom the behavioural and deployment-profile hashes
already use. The semantic hash moved to `sha256-canonical-json-v3`; the
anchor set moved to `sha256-anchor-set-v2`. Both
new values were predicted from the recipe before the code was run and
then confirmed by the generator, so the implementation was checked
against an independent computation rather than trusted. No projection or
encoding changed: the meaning identified is the same and only the
measurement moved.

The welds moved atomically with the values — the typed pin, both
generated manifests, the realization document's masthead and attached
appendix, the identities register, and the synthetic release-profile
identity, which moved because the deployment profile binds the
architecture semantic hash as a hashed input while its own recipe stayed
at version 1. The behavioural hash did not move and no version was
bumped: its recipe and body are untouched, the versioning gate keys on
it alone, and ADR-016 holds that a recipe migration does not by itself
imply a semantic version change. The grandfather clause is superseded by
the migration record in the same ADR.

DI-004 then closed with the census. It was taken from the code outward
rather than from the ADR's table — every digest, identity and hash the
tree computes was found by sweeping packages and scripts and then
reading the owning modules — and each was put to the benefit criterion:
which named consumer decides what, and in which assurance class. The
answer is six admitted identity classes and eleven recorded stops, all
in `plans/registers/identities.md`, which now opens with a census table
of one line per entry and carries admission records and stop records as
its two register bodies. Nothing new was admitted and no unclassified
digest was found, so the migration remains DI-004's only change to what
the tree computes.

The reading paid for itself twice. Eight of the eleven refusals were
practice, or enforced in code, without ever having been written down as
decided outcomes: the native conformance reports that deliberately carry
no digest and no field reserved for one, the compiler's analyzed
program whose refusal is held by a compile-time probe and an exhaustive
destructuring rather than by convention, the Phase-1 realization that
publishes an architecture binding instead of an identity, the target's
own tagged hashes and sighashes, which are the modelled protocol's
values and not identities of this corpus, the chain identifiers received
as validated typed fields, the executor's wire frames, the CI lane
records, and field-level digests in general. And the profile's raw hash
fields were undercounted: the register named the report and artifact
rows, while the schema also carries one evidence hash per substrate
dependency and the calibration's own evidence and script-bundle hashes.
All four families are recorded together, with the one binding profile
validation actually enforces — the calibration's bundle hash must equal
the released emitted-script-bundle row — separated from the presence
checks that decide nothing.

Two admission records gained precision the ADR's table does not carry.
The architecture semantic hash is the one admitted identity that crosses
a process boundary today, because the ledger's schema-13 attestation
context carries it, and each of its consumers decides by rejection: the
indexer refuses a context whose manifest hash is not the build's
expected typed hash on every construction path, profile validation
refuses a profile bound to another architecture, and the weld fails the
build on a disagreeing masthead. The document and instance identities
are locator-grade provenance, and their 128-bit truncation is admissible
only on that condition — a release-bound provenance identity could not
carry it — which the records now state rather than leave implied.

The cross-check against ADR-016 found no row the census contradicts: all
nine rows of its current-identities table hold against the code, and the
ADR was not amended. One imprecision is recorded rather than corrected —
the table calls the two publication identities UUIDs, which the
producing functions do too, though neither value sets a version or
variant field — because the register already carries that non-claim and
the ADR remains the owner of its own wording. The gap census was
repaired where the migration and the census falsified it: domain
separation and recipe identifiers are implemented rather than divergent,
the no-identity clause is implemented rather than divergent, the
implementation gate's evidence names the three items DI-004 discharged,
and the second open ruling is settled.

What outlives DI-004 belongs elsewhere. The deployment-profile hash and
the profile's raw hash fields stay pre-admission with their revisit
conditions bound to the release surface, not to this batch; the benefit
walk as a standing requirement on new proposals, rather than as the
one-time census just taken, is a change to ADR-016's admission rule that
no task yet owns; and the executor protocol's stop records the reasoning
DI-005 will restate when the interchange conventions are adopted as the
discipline for externally consumed documents.

DI-003 opened with W1, the participation scanner. The survey found the
judgment re-derived in six places: the Markdown scanner's own fence
loop, a second fence loop in the plan-tree link check, and the
double-backtick skip restated inline at each of the four span consumers
in `packages/labels/src/repository.rs`. All six now consume one module,
`packages/labels/src/participation.rs`, whose module documentation
carries the survey as a table — decision site, the logic it used to
carry, where it lives now — so a seventh recognizer has somewhere to be
refused. The Rust comment-and-literal segmentation and the LaTeX comment
strip moved into it as well, making it the single home of the
scanned-region recognition ADR-019 fixes for all three concrete
syntaxes.

One genuine disagreement was found and resolved rather than preserved.
The Realization harvest read its span list from the fence-aware scanner
but computed the boundaries of the generated upward-citation index by
walking the raw source, so a section heading displayed inside a fenced
block could open that index region — and, because the region stayed open
across the fence close, swallow the real body citation below it. The
anchor set derives from body citations, so the effect was a body
citation lost and the genuine index reported stale. The region walk now
reads participating lines only. The resolution is what the calculus
requires, since a displayed heading is not authored text and cannot
partition a document; it is a deliberate behavior change, recorded here
and reproduced by a test that fails with the fix reverted and passes
with it in place. No occurrence of the pattern exists in the tree today,
so no diagnostic moved.

Behavior preservation was measured, not reviewed. A capture script,
`scripts/capture-label-goldens.sh`, runs all four labels binaries over
the whole tree with the argv the CI lanes use, writing each stream and
exit status; the before and after captures are byte-identical once the
tracing wall-clock field is normalized, which includes both generated
registers reproduced whole at 17376 and 35384 bytes. The committed
registers under `plans/labels/` are unchanged. The label census is
identical across the refactor at 136 Layer-0, 329 realization, 129 ADR,
307 model, 1206 planning and 6 documentation mints with 211 imported
citations.

Two classes of site were surveyed and deliberately left alone, both
recorded in the scanner's module documentation. The forbidden-text audit
is participation-blind by design — a banned token is banned inside a
fence and inside a string literal too — so it keeps its whole-tree grep.
And several plan-tree hygiene checks read raw text: the scaffolding,
placeholder, confidence, deleted-path and machine-input markers, and the
backlog's task-status agreement walk. Their subjects are not label
occurrences, and rewiring them would change what they report, so they
are recorded as participation-blind rather than quietly converted.

W2 made the adoption parameters typed data. All seven now sit in
`packages/labels/src/adoption.rs` for comparison against the ADR-019
table, and the signature drives prefix resolution rather than restating
it: an imported token splits at its first hyphen, exact because a prefix
carries none, and becomes an owner only through the signature. The
package owners are registered there, settling the DI-003 ruling, their
prefixes derived by rule from the directory name and checked against the
census, so a new package is a reviewed registration, never a silent
owner.

The kind vocabulary is now the registry's: the 208 distinct kinds of the
draft's Convention tables and the 13 recorded extensions, committed
rather than parsed at check time so an edition swap shows every moved
token in review, and checked against their documents in each direction
so the tables cannot go stale unseen. The census found 2113 mints over
64 distinct kinds, every one governed except `motto` and `abs`, single
occurrences on the Layer-0 LaTeX surface that ADR-020 already records as
unadjudicated. That record puts the surface outside its scope, so those
two report rather than fail, and enforcement over the owners it does
govern lands with the tree already conformant.

Warrant totality is enforced ahead of its subject. The reserved set and
the profile signature are empty, so its reserved and inventory arms are
vacuous, and both are implemented and fixture-tested all the same, so
the first registered profile finds the enforcement live. Place detection
is unbuilt and reports no place, which fails closed. Two-pass staging is
structural: the adoption data load before any source is read, and
warrant totality derives from completed registries before any citation
resolves.

One defect surfaced in passing: diagnostics were emitted only when the
check failed, so no passing run could ever show a warning. Warnings now
emit on a passing check, which is what makes the two Layer-0 tokens
visible.

W3 closed the batch with the near-miss warnings, in four families and
warnings only: an interior differing from a label in case, in spacing,
or in bracketing, and a label-shaped backtick span in scanned comment
text where the acute carries the label syntax. Each family repairs the
span before testing it and every repair changes the text, so a form the
grammar accepts can never reach a warning; displayed spans stay silent
and no existing error was demoted. The live tree raises 198, all of them
the comment family — one finding rather than 198, since Rust comments
cite labels in backticks throughout.

The companion attestation register the acceptee owes is generated at
plans/labels/attestation.md, 6176 bytes, by its own target: separate
from the upstream generator because its derivation is corpus-wide where
theirs is scoped, its evidence rows carrying a mint census that answers
to every owner. Three views of one base under the recorded ordering —
the 13 first-hand extension rows with spelling, locator, sense, and
census; the status map, 349 base rows by reference and the three the
edition daggers; and Hom(C_A), 32 pairs over 15 names. The derived base
relation is welded to the registry's published headline counts, so a
parse that drifts fails rather than re-deriving quietly. Task rows in
Hom only because this corpus extends it, which is the register's point.

Both records were then walked bullet by bullet. The calculus's
implementation gate has one unmet bullet left, inventory discipline,
waiting on the decision that registers the first profile rather than on
code; two ADR-019 claims had gone stale the other way, package prefixes
and the profile note, and both now read as built. The registry's
adoption gate lost its recorded unmet item with the register and gained
one this walk found: head validation is unimplemented, the checker
validating a kind token but never the pair a head declares. The 146
environment heads under adr/ and plans/ were read by hand and all carry
catalogued pairs, so nothing rests on the gap, and ADR-020 names it.

Three residuals outlive DI-003, none its own to discharge: the
derivation-warrant machinery waits on a first inventory profile, Π and K
empty by decision; Layer-0 kind scope waits on the motto, invest, and
abs adjudication, entering with that surface; and the interchange
conventions stay dormant until a consumer exists, which is DI-005.

One of those three tokens is now settled. On the user's ruling this
corpus keeps its opening motto apart from the registry's slogan genre,
so Motto and `motto` enter X_A as its fourteenth entry, on first-hand
evidence at the title section. ADR-020 records the deviation, the
checker's kind and pair tables carry it, and the regenerated companion
register puts Hom(C_A) at 34 pairs over 16 names. Layer-0 stays
reported: `invest` and `abs` are open.

The acute code syntax now classifies as the calculus fixes it. The
harvester had paired acutes blindly, one comment line at a time, so
every acute opened: a lone one used as an apostrophe failed the file
and a pair swallowed the prose between into a bogus label. Pairing is
now settled over the logical comment region, an acute opens only where
label-shaped text follows, and one that opens nothing stays text, while
an opener whose region ends first still fails hard. No rule anywhere
banned the acute in Rust sources — the forbidden-text audit knows one
token and it is not this — so nothing needed repealing. Both syntaxes
are readable in comments during the migration: the acute participates,
the backtick spelling only warns, and the sweep of the 198 warned sites
follows. Goldens over the tree are byte-identical.

That sweep is done: all 198 warned spans are gone. 196 became live
imported citations, each carrying the registered prefix of the owner
that mints the label — PLAN for the Guide-10 rules, R13, PA, and
numbered ADRs elsewhere — and imported citations rose by exactly 196,
from 211 to 407. Conformance held 132, tapscript 23, architecture 12,
target-elements 11. Nothing was minted to make a citation resolve: no
crate mints a label, so every reference was owed elsewhere. Two sites
quote a label's spelling rather than cite it, in the guard-listing
weld that parses the document for it; both are double-backtick spans,
shown rather than meant. Groups that straddled a line were brought
onto one, a group being read within its own line. The generated
registers regenerate unchanged, so no anchor set moved.

W3 closed the batch with head validation: each head's name is now held
to the kind its label declares, over the decision records and the
authored planning tree. The recognizer finds exactly the 146 heads the
hand reading counted — 4 authored here, 142 in the archive it does not
judge — and all validate by an exact pair, so no head or label moved.
Reduction implements the sub- prefix and the catalogued modifiers only;
the other devices are unimplemented and untriggered, so a numbered or
lettered head fails until they are. The three waves: acute
classification, the 196-site sweep, head validation.

### 13.2 Checker findings so far · `tab:backlog:draft-findings`

| ID | Status | Finding |
|---|---|---|
| `DI-F01` | DONE | The plans-tree link scanner read bracketed patterns inside fenced blocks as Markdown links, so a CDDL regex in an archived draft failed as a broken link. Fenced interiors are now blanked before link scanning; the systematic single-scanner repair landed as DI-003 W1, which folded this blanking into the shared scanner. |
| `DI-F02` | DONE | The Realization harvest computed the boundaries of the generated upward-citation index from the raw source while reading its spans from the fence-aware scanner. A section heading displayed inside a fenced block therefore opened the index region, which stayed open across the fence close and swallowed the body citation below it, losing an anchor and reporting the genuine index stale. The region walk now reads participating lines only. Found and resolved by DI-003 W1; no occurrence existed in the tree, so no diagnostic moved. |
| `DI-F02` | ACTIVE | Participation is enforced inconsistently across checks: the label scanner honors fences, the link scanner did not, and the scaffolding, placeholder, and confidence hygiene checks still scan fenced material. One participation model must feed every check. |
| `DI-F03` | ACTIVE | The calculus's owner signatures, imported-citation prefixes, synthetic citations, anchor harvests, and acute-delimiter hard failure are only partially realized in the present checker; the gap census of DI-001 owns the exact delta. |

### 13.3 Toolchain engineering · `tab:backlog:toolchain-tasks`

| ID | Status | Task |
|---|---|---|
| `CI-001` | DONE | Rewrite the shell CI driver in Python with a typed lane tracker: every lane declared with status and skip reason, every lane and the whole run wall-timed, and a timing report emitted as the success output and on failure alike. Delivered as scripts/ci.py with an eleven-lane registry and the ci.sh shim; the first dataset shows the two test lanes at eighty-four percent of a twenty-minute gate. |
| `CI-002` | ACTIVE | Move test execution to the meson layer per the user's ruling: no workspace-level cargo test in the gate; each package's test groups run as individual meson-driven lanes, and per-test timing uses the nightly libtest JSON output, which the user has admitted as not affecting what the tests prove. The timing report gains per-package and per-test figures; attribution replaces the aggregate block. Contention on the shared cargo target directory is measured and the chosen serialization or partitioning recorded honestly. |
| `CI-003` | DONE | User ruling: the generated registers are archive-budget, by role not directory; labels/README.md stays prose. Combined 783755 to 724577 bytes; archive 1168518 to 1228130. |

## 14. Backlog hygiene · `sec:backlog:hygiene`

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

A new digest additionally satisfies ADR-016 admission.

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

After a phase or remediation series:

- move durable evidence into the owning phase card, ADR, decision, research
  result, release record, or annotated tag;
- retain permanent task IDs in compact tables;
- delete implementation diaries from the active backlog;
- rely on Git history rather than creating a second archive under `plans/`;
- keep only current and immediately preparatory work detailed here.

---

## 15. One-line backlog · `rem:backlog:one-line`

> Settle the STATE-constructor, wide-arithmetic, and declassification prototypes on the evidenced primitive substrate, without emitting operations, completing backend patterns, minting speculative identities, or claiming production activation.
