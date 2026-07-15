# Release and Deployment-Profile Package Plan

> **Status:** PLANNED
> **Planned source directory:** `packages/release`
> **Planned Cargo package:** `tripod-release`
> **Planned Rust library name:** `release`
> **Implementation phases:** Initial types and orchestration may begin before
> Phase 12; final release gate belongs to Phase 12
> **Depends on packages:** `architecture`, `realization`, `compiler`,
> `target-elements`, `tapscript`, `linker`, `transaction`, and `vectors`
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-is-normative.md),
> [D002](../decisions/002-target-independent-realization-layer.md),
> [D003](../decisions/003-multiple-backends-tapscript-first.md),
> [D004](../decisions/004-translation-validation-over-compiler-trust.md),
> [D005](../decisions/005-value-parametric-asset-rigid.md),
> [D006](../decisions/006-canonical-transaction-layout-abi.md)
> **Open research dependencies:** all research questions selected by the final
> backend scope must be resolved before final release
> **Authority:** Final first-party assembly and validation of typed deployment
> identities, artifacts, evidence, and deployment profile; no authority to
> redefine protocol semantics or waive normative obligations
> **Machine-consumed planning document:** no

---

## 1. Purpose

The `release` package will provide the final first-party assembly and validation
gate for one attestation-contract deployment candidate.

It consumes typed:

- architecture identity and release validation result;
- target-independent realization and its completeness/identity;
- compiler analyzed-program and configuration identities;
- exact target definition and deployment instance;
- backend configuration and relocatable artifact identities;
- final linked bundle;
- final transaction and witness ABI;
- calibrated finite bounds and complete measurement reports;
- generated publication artifacts and their expected typed derivations;
- model unit-test and property-test reports;
- relation-indexed target execution reports;
- representation-safety and minimality reports;
- target substrate reports;
- script-integration reports;
- separately implemented event-projection report;
- separately implemented attestation-query report;
- separately implemented receipt-accounting report;
- typed deployment-profile data.

It owns:

- release-candidate assembly;
- cross-package identity validation;
- architecture-release invocation;
- realization-completeness validation;
- compiler-analysis completeness validation;
- exact target/deployment binding;
- final linked-bundle and ABI binding;
- calibration orchestration and final calibration verification;
- artifact hashing under documented canonical recipes;
- evidence-report census and identity validation;
- independent-observer report validation;
- deployment-profile construction;
- deployment-release validation;
- deployment-profile hash verification;
- deterministic release-manifest construction;
- deterministic release-directory/archive construction;
- release publication only after every gate passes;
- non-writing release checking;
- release command output under ADR-010;
- final release failure classification.

It does not own:

- anchor-set semantics;
- architecture declarations;
- realization formulas or relations;
- compiler proof planning;
- target capabilities;
- backend program emission;
- linker reference resolution;
- transaction construction;
- evidence generation;
- independent observer implementation;
- production key custody;
- network broadcast;
- mining or confirmation;
- discretionary release waivers.

The planned direction is:

```text
typed architecture release
        +
complete typed realization
        +
complete compiler analysis
        +
exact target/deployment identity
        +
final linked bundle
        +
final transaction/witness ABI
        +
calibrated bounds and measurements
        +
model/backend/substrate/observer evidence
        ↓
assemble ReleaseCandidate
        ↓
validate every identity and evidence boundary
        ↓
construct final DeploymentProfile
        ↓
architecture::validate_deployment_release
        ↓
verify domain-separated profile identity
        ↓
construct ValidatedRelease
        ↓
optional explicit publication to caller-selected output
```

A release candidate is not a release.

Only a successfully validated `ValidatedRelease` may be published as final.

---

## 2. Why a separate release package is required

The existing repository already distinguishes:

```text
architecture release
```

from:

```text
deployment release
```

Architecture release proves that the typed architecture is:

- internally valid;
- final;
- anchor-set pinned;
- canonically published.

It does not prove that one concrete target deployment has:

- selected an exact network and genesis;
- calibrated every finite bound;
- verified target dependencies;
- emitted the intended target programs;
- linked constructors correctly;
- published one canonical transaction ABI;
- passed relation-indexed target execution;
- produced independent event/query/accounting reports;
- assembled a complete final profile.

As the compiler-era package graph grows, those claims will be produced by
different packages and tools.

Without one release owner, final assembly can drift through:

- shell scripts;
- CI YAML;
- ad hoc archive commands;
- manual hash copy/paste;
- Markdown release checklists;
- profile fields supplied independently of report identities;
- stale evidence attached to changed bundle bytes;
- generated files accepted without typed equality checks;
- candidate calibration mistaken for final calibration;
- regtest evidence attached to another target;
- one observer report standing in for several claims.

The release package closes the graph by validating every typed identity,
artifact, and evidence binding in one fail-closed process.

---

## 3. Assurance boundary

The release package establishes:

- the architecture itself is releasable;
- the realization scope is complete for the deployment's approved operation
  set;
- the compiler analysis covers exactly that realization scope;
- the target identity and deployment instance are exact and compatible;
- the final linked bundle matches compiler, backend, target, deployment
  constants, and calibrated bounds;
- the final transaction ABI matches the linked bundle;
- calibration measurements apply to the exact final bundle and ABI;
- all required target dependencies have verified evidence;
- relation-indexed backend reports apply to the exact final artifacts;
- representation claims are supported by exact reports;
- model and property reports are present;
- independent event/query/accounting reports are separately present and valid;
- generated publications equal independently derived typed expected values;
- the final deployment profile has one unambiguous interpretation;
- architecture deployment-release validation passes;
- the deployment-profile hash verifies;
- the release manifest and archive are deterministic.

It does not establish:

- that normative documents are economically correct;
- that cryptographic assumptions are true;
- that source-reviewed target software contains no unknown defect;
- that finite vectors prove every possible target transaction;
- that independent implementations share no conceptual error;
- that the deployment will remain live in a fee market;
- that a released transaction will relay or confirm;
- that a final deployment cannot later require an incident response.

The release manifest must state its exact assurance scope and named residuals.

---

## 4. Dependency direction

### 4.1 Broad final-layer dependencies are intentional

The release package may depend broadly because it assembles typed output from
all earlier assurance boundaries.

Expected dependencies include:

```text
architecture
realization
compiler
target-elements
tapscript
linker
transaction
vectors
```

It may also depend on:

```text
artifacts
```

if the existing artifact crate provides expected publication bytes and
document-weld checks required by release.

### 4.2 No reverse dependency

None of the packages above may depend on `release`.

The direction is:

```text
release
    → architecture
    → realization
    → compiler
    → target/backend/linker/transaction/vectors
```

according to their actual package graph.

There must be no:

```text
vectors → release → vectors
```

or:

```text
linker → release → linker
```

cycle.

### 4.3 Report type ownership

Initially, report types produced by `vectors` may be consumed directly by
`release`.

If separately implemented external tools need stable report types without
depending on the vectors implementation, extract a narrow evidence-schema
package through a later decision.

Do not create that package before concrete consumers require it.

### 4.4 Calibration orchestration

The release package is the initial planned owner of **calibration
orchestration**, because it can depend on:

- linker candidate construction;
- transaction worst-case construction;
- vector/target measurement;
- architecture deployment-profile types.

The pure search implementation may later be extracted if it becomes a
substantial independent package.

The calibration dependency direction is:

```text
release calibration runner
    ├── calls linker::link_candidate
    ├── calls transaction::derive_candidate_abi
    ├── calls transaction::worst_case_transactions
    ├── calls vectors/target measurement
    └── calls linker::finalize
```

The linker and transaction packages remain unaware of release.

---

## 5. Normative typed inputs

### 5.1 Architecture

The package consumes:

```rust
&architecture::Architecture
```

and calls typed architecture validation.

It does not parse `architecture.json` or `architecture.toml` as semantic input.

Generated files may be release assets only after exact comparison with typed
expected values.

### 5.2 Realization

The package consumes:

```rust
&realization::RealizationSpec
```

plus:

- realization scope;
- completeness result;
- realization schema/identity;
- architecture binding;
- declassification identity;
- lifecycle and constructibility completion status.

A pilot realization cannot be assembled into a full final deployment.

### 5.3 Compiler analysis

The package consumes:

```rust
&compiler::AnalyzedProgram
```

and, where target proof selection is separate:

```rust
&compiler::TargetCompilationPlan
```

It validates:

- relation census;
- realization binding;
- target binding;
- proof-plan identity;
- disclosure plan;
- lifecycle plan;
- placement and layout requirements;
- coverage policy.

### 5.4 Target

The package consumes:

```rust
&target_elements::ElementsTarget
```

including:

- target-definition identity;
- deployment-instance identity;
- network ID;
- genesis ID;
- activation binding;
- target evidence requirements;
- consensus/policy limits.

A regtest target and production target remain distinct.

### 5.5 Backend configuration and relocatable identity

The release package records and validates:

- backend configuration identity;
- pattern-library identity;
- relocatable bundle identity;
- source/compiler version;
- selected proof/representation identity.

It does not use relocatable output as the final deployment artifact.

### 5.6 Final linked bundle

The release package consumes:

```rust
&linker::LinkedBundle
```

The bundle must be final, not candidate status.

It binds:

- exact target;
- deployment constants;
- calibrated bounds;
- constructors/programs;
- taptrees/control recipes;
- relation carriers;
- resource formulas;
- linked-bundle identity.

### 5.7 Final transaction ABI

The release package consumes:

```rust
&transaction::TransactionAbi
```

The ABI must bind:

- final linked bundle;
- exact target;
- calibrated bounds;
- operation scope;
- layouts;
- witness schemas;
- constructor/metadata schemas;
- supported representations;
- ABI identity.

### 5.8 Evidence reports

The release consumes typed validated reports for:

- architecture publication;
- realization/model conformance;
- model unit tests;
- model property tests;
- compiler analysis;
- backend patterns;
- linked-bundle target execution;
- relation coverage;
- representation safety;
- representation minimality where claimed;
- resource calibration;
- target dependencies;
- script integration;
- event projection;
- attestation query;
- receipt accounting.

### 5.9 Publication assets

Publication assets enter as typed descriptors plus bytes.

Examples:

```text
architecture.json
architecture.toml
declassification.json
model_labels.json
linked script bundle
transaction ABI
canonical vectors
resource reports
evidence reports
release manifest
deployment profile publication
```

Each asset descriptor identifies:

- logical asset kind;
- canonical path;
- schema/encoding;
- exact bytes or deterministic byte producer;
- hash recipe;
- source typed value;
- release relevance.

The release package does not infer asset kind from filename alone.

### 5.10 Explicit release metadata

Nonsemantic publication metadata may enter as typed explicit input:

- release date;
- release label;
- release notes reference;
- source commit;
- operator/maintainer public identity;
- output directory;
- archive format.

Such metadata must be excluded from semantic identities where appropriate and
must never come from ambient wall-clock time.

---

## 6. Forbidden inputs and behavior

The release package must not consume as semantic source:

- architecture JSON/TOML;
- declassification JSON;
- realization Markdown;
- anchor-set LaTeX;
- planning files;
- model source text;
- target opcode Markdown;
- backend disassembly;
- untyped report JSON;
- CI status text;
- shell command output without typed parsing and validation.

The package must not:

- redefine architecture or realization semantics;
- fix a missing compiler relation;
- emit missing backend programs;
- resolve unresolved linker symbols manually;
- generate production private keys;
- sign production transactions;
- broadcast transactions;
- silently choose target/network defaults;
- substitute draft calibration defaults;
- attach evidence from another bundle or ABI;
- merge independent evidence claims into one opaque status;
- mark test doubles as independent implementations;
- accept zero evidence hashes;
- downgrade required failures to warnings;
- repair stale generated assets during checking;
- write release artifacts before validation;
- use ambient current time;
- include host paths or secrets in canonical release identity;
- publish a candidate bundle/profile as final;
- create a waiver entry that has no typed policy.

---

## 7. Release-state model

The package should encode release progression through distinct types or
validated states.

### 7.1 Release inputs

Conceptually:

```rust
pub struct ReleaseInputs<'a> {
    pub architecture: &'a architecture::Architecture,
    pub realization: &'a realization::RealizationSpec,
    pub analyzed_program: &'a compiler::AnalyzedProgram,
    pub target_plan: &'a compiler::TargetCompilationPlan,
    pub target: &'a target_elements::ElementsTarget,
    pub linked_bundle: &'a linker::LinkedBundle,
    pub transaction_abi: &'a transaction::TransactionAbi,
    pub evidence: &'a EvidenceSet,
    pub publications: &'a PublicationAssetSet,
    pub metadata: &'a ReleaseMetadata,
}
```

> Illustrative API; exact fields and names are not frozen.

### 7.2 Release candidate

A `ReleaseCandidate` is assembled after input parsing and basic identity
normalization but before final validation.

It may contain failures/incomplete status.

It must not expose APIs that imply finality.

### 7.3 Validated release

A `ValidatedRelease` can be constructed only after every mandatory check
passes.

Conceptually:

```rust
pub struct ValidatedRelease {
    pub manifest: ReleaseManifest,
    pub deployment_profile: architecture::DeploymentProfile,
    pub deployment_profile_hash: [u8; 32],
    pub assets: ValidatedPublicationAssetSet,
    pub identity: ReleaseIdentity,
}
```

> Illustrative API; not frozen.

Prefer private fields and validated constructors so invalid final release
values cannot be assembled freely by downstream code.

### 7.4 Published release

Publication is an explicit side effect:

```text
ValidatedRelease
    +
caller-selected output destination
    ↓
PublishedReleaseReceipt
```

The published receipt records:

- release identity;
- output paths;
- file hashes;
- archive hash if applicable;
- publication status.

A published receipt does not alter the validated release's semantic identity.

---

## 8. Public API boundary

### 8.1 Candidate assembly

Conceptually:

```rust
pub fn assemble_candidate(
    inputs: ReleaseInputs<'_>,
) -> Result<ReleaseCandidate, ReleaseError>;
```

> Illustrative API; not frozen.

Assembly is non-writing and deterministic.

### 8.2 Candidate validation

Conceptually:

```rust
pub fn validate_candidate(
    candidate: ReleaseCandidate,
    policy: &ReleasePolicy,
) -> Result<ValidatedRelease, ReleaseError>;
```

Validation must be fail-closed.

### 8.3 Combined validation

A convenience API may combine both:

```rust
pub fn validate_release(
    inputs: ReleaseInputs<'_>,
    policy: &ReleasePolicy,
) -> Result<ValidatedRelease, ReleaseError>;
```

### 8.4 Non-writing check API

The release package should expose a pure/non-writing checker for an existing
release directory or archive.

Conceptually:

```rust
pub fn check_release(
    expected: &ValidatedRelease,
    actual: &Path,
) -> Result<ReleaseCheckReport, ReleaseError>;
```

The checker:

- reads;
- parses;
- hashes;
- compares;
- never repairs or rewrites.

### 8.5 Publication API

Conceptually:

```rust
pub fn publish_release(
    release: &ValidatedRelease,
    output: &Path,
    options: &PublicationOptions,
) -> Result<PublishedReleaseReceipt, ReleaseError>;
```

Publication must:

- stage outputs;
- verify staged bytes;
- atomically publish where possible;
- never overwrite an unrelated release silently;
- use caller-selected output;
- avoid ambient timestamps and filesystem metadata.

### 8.6 Calibration API

Conceptually:

```rust
pub fn calibrate(
    request: CalibrationRequest<'_>,
    policy: &CalibrationPolicy,
) -> Result<CalibrationResult, ReleaseError>;
```

> Illustrative API; not frozen.

Calibration returns typed evidence and final bound assignment. It does not
publish final release output automatically.

### 8.7 No implicit process behavior in library APIs

The library does not:

- execute Git;
- read process argv;
- discover target binaries from `PATH`;
- spawn target nodes;
- inspect current working tree;
- choose output directories;
- read current time.

CLI/orchestration adapters supply those facts explicitly.

---

## 9. Release policy

### 9.1 Typed policy

A `ReleasePolicy` may specify:

- required operation scope;
- required evidence classes;
- accepted report schemas;
- accepted target definition;
- production network requirements;
- required representation claims;
- reproducibility requirements;
- publication asset census;
- archive format;
- source-cleanliness requirement;
- tool-version rules.

The policy must not redefine protocol semantics.

### 9.2 No waiver policy

The initial release policy has **no general waiver mechanism**.

A required failure or missing report blocks release.

If a waiver system is ever proposed, it requires a separate accepted decision
covering:

- typed waiver identity;
- scope;
- severity;
- expiration;
- consumer visibility;
- cryptographic/identity binding;
- release semantics;
- normative compatibility.

Until then:

```text
missing required evidence = release failure
failed required evidence = release failure
incomplete required evidence = release failure
unsupported required evidence = release failure
```

### 9.3 Development versus production policies

The package may support distinct typed policies:

```text
development candidate
regtest integration release
production deployment release
```

A development policy must not be labeled final production release.

Differences may include:

- operation scope;
- target network;
- report requirements;
- source cleanliness;
- archive publication;
- independent observer requirements.

Production policy remains the strictest and must satisfy architecture
deployment release.

### 9.4 Policy identity

Every release binds one exact release-policy identity.

Changing policy requires evidence reassessment and moves release identity.

---

## 10. Identity graph validation

### 10.1 Required identity chain

The release package validates a chain resembling:

```text
anchor-set identity
    ↓
architecture identity
    ↓
realization identity
    ↓
analyzed-program identity
    ↓
target-plan identity
    ↓
target definition + deployment instance
    ↓
backend configuration
    ↓
relocatable bundle
    ↓
linked bundle
    ↓
transaction ABI
    ↓
vector set
    ↓
evidence reports
    ↓
deployment profile
    ↓
release identity
```

Not every layer necessarily has a published hash in the earliest milestone,
but every release-relevant typed identity must be explicit.

### 10.2 Cross-binding matrix

The release package should validate every applicable pairwise binding.

Examples:

| Subject | Must bind |
|---|---|
| Realization | architecture |
| Analyzed program | architecture + realization + scope |
| Target plan | analyzed program + target capability identity |
| Relocatable bundle | target plan + backend configuration |
| Linked bundle | relocatable bundle + target deployment + bounds |
| Transaction ABI | linked bundle + target + bounds |
| Vector set | realization/compiler scope + target/bundle/ABI as applicable |
| Execution report | target + bundle + ABI + vector set |
| Resource report | target + bundle + ABI + candidate/final bounds |
| Observer reports | network + genesis + architecture hash + checkpoint/schema |
| Deployment profile | architecture + network/genesis + artifacts + reports |
| Release manifest | all final assets and identities |

### 10.3 Identity mismatch

Any mismatch is a release failure even when report contents otherwise say
“passed.”

### 10.4 No identity inference from filenames

Do not infer:

```text
this is bundle X
```

from:

```text
bundle-X.json
```

Parse and validate the typed internal identity and hash.

### 10.5 Zero and placeholder identities

Reject:

- all-zero hash;
- placeholder URL;
- “unknown” source revision in recorded provenance;
- empty tool version;
- blank report test name;
- candidate identity where final required;
- mutable target branch name as review provenance.

---

## 11. Architecture release validation

### 11.1 Typed validation

Call the architecture crate's supported release validation.

Require:

- draft validation;
- supported document/envelope metadata;
- final publication status;
- anchor-set anchor-set pin;
- semantic hash validity;
- behavioural hash validity;
- versioning gate.

### 11.2 Generated architecture publications

Derive expected:

```text
architecture.json
architecture.toml
```

from typed architecture.

For each committed/release asset:

1. parse;
2. reject unknown fields;
3. validate supported envelope;
4. verify hashes;
5. compare full typed value;
6. compare exact canonical bytes.

Do not trust embedded hash fields alone.

### 11.3 Realization appendix/document weld

The release should consume a machine-produced weld report proving:

- appendix TOML equals generated architecture TOML;
- masthead identities match;
- realization document labels/welds pass;
- anchor-set pin matches anchor-set citations.

The release package need not reimplement Markdown extraction if the artifacts
crate owns the tested logic. It must bind the report or rerun the typed check.

### 11.4 Architecture identities

Use typed architecture identities as the source for every downstream binding.

Do not derive architecture identity by hashing a release archive file with a
different recipe.

---

## 12. Realization completeness validation

### 12.1 Required operation scope

Production release requires complete realization coverage for every operation
in the approved architecture/deployment scope.

The default production scope is:

```text
all architecture operations
```

A reduced scope requires:

- a separately approved deployment profile or normative decision;
- clear indication that the bundle is not a complete realization;
- no reuse of full-release language.

### 12.2 Completeness checks

Require:

- every operation realized;
- every relevant object realized;
- every relation validated;
- every public observable defined;
- every constructibility requirement complete;
- every lifecycle exit represented;
- every supported representation has complete lifecycle;
- declassification derived;
- no unresolved semantic placeholder;
- no target detail in realization identity.

### 12.3 Model conformance report

Require a passed report bound to the exact realization identity and scope.

Pilot conformance reports do not satisfy full release.

---

## 13. Compiler-analysis validation

Require:

- analyzed-program schema supported;
- realization binding exact;
- relation census equal;
- no dropped relation;
- proof alternatives complete;
- selected target plan supported;
- disclosure plan complete;
- minimality claims accurately scoped;
- fact sources complete;
- permissionless witness availability valid;
- lifecycle plan complete;
- placement requirements complete;
- layout requirements complete;
- target requirements complete;
- relation coverage requirements complete;
- compiler configuration identity present.

A compiler report for another target plan or operation scope is stale.

---

## 14. Target and deployment validation

### 14.1 Exact target

Require:

- exact target-definition identity;
- exact upstream source revision;
- exact deployment-instance identity;
- network ID;
- genesis ID;
- production/development policy match;
- activation evidence;
- supported target schema.

### 14.2 Target evidence census

For every target capability selected by the compiler/backend, require a passed
evidence report if policy marks it required.

Architecture dependency requirements and target evidence requirements must map
completely.

### 14.3 Network/genesis consistency

Require equality among:

- target deployment instance;
- deployment profile;
- observer report contexts;
- canonical wire vectors;
- transaction ABI;
- linked bundle;
- release manifest.

### 14.4 Regtest versus production

A production release must not attach regtest-only evidence as production
activation/policy evidence unless an accepted equivalence claim explicitly
covers the tested property.

Source-level semantic equivalence and network-level deployment evidence remain
distinct.

---

## 15. Linked bundle and ABI validation

### 15.1 Final status

Require:

- final `LinkedBundle`;
- no candidate flag;
- no unresolved mandatory relocation;
- complete operation scope;
- final calibrated bounds;
- valid linked-bundle identity;
- deterministic bundle manifest.

### 15.2 Relation carriers

Require exact equality among:

- realization relation scope;
- compiler relation scope;
- linked carrier scope;
- coverage report scope.

No relation may be carried only by an unreachable program.

### 15.3 Constructor closure

Require:

- every protocol object family has a linked constructor;
- static code/program continuity complete;
- metadata schema complete;
- no spendable metadata escape;
- internal keys and target commitments valid;
- reference-resolution report passed.

### 15.4 ABI binding

Require:

- ABI target equals bundle target;
- ABI bundle ID equals final bundle ID;
- ABI bounds equal final calibrated bounds;
- operation scope equal;
- every target program reference resolves;
- witness schemas complete;
- representation support matches selected target plan;
- ABI identity verifies.

### 15.5 Canonical ABI publication

Derive expected ABI publication from the typed ABI and compare exact bytes.

---

## 16. Calibration orchestration

### 16.1 Inputs

Calibration consumes:

- relocatable backend bundle;
- exact target;
- deployment constants;
- manifest minima;
- target limits;
- calibration policy;
- transaction fixture inputs;
- measurement executor.

### 16.2 Required bound census

Every architecture bound marked for deployment calibration appears exactly
once.

Reject:

- missing bound;
- duplicate bound;
- unexpected bound;
- zero value;
- below semantic minimum;
- target-unrepresentable value.

### 16.3 Search policy

The initial calibration runner may use:

- deterministic binary search for proven monotone dimensions;
- deterministic bounded enumeration;
- another explicit deterministic search.

The policy records:

- search bounds;
- monotonicity justification;
- tested candidates;
- failure dimensions;
- selected value;
- tie-break.

### 16.4 Complete affected-operation set

For every bound, measure every operation family that references it.

Examples:

```text
FEE_SPONSOR_INPUT_MAX:
    many operation families

ASH_BATCH_MAX:
    compact-ash
    clear

SETTLEMENT_BATCH_MAX:
    settle-distribution input and output families
```

The exact affected set derives from typed architecture/compiler data.

### 16.5 Worst-case fixtures

Use transaction-package valid worst-case fixtures.

For every candidate:

- link candidate bundle;
- derive candidate ABI;
- construct objective-specific worst-case transactions;
- target-execute them;
- measure resources;
- compare predicted/observed values.

### 16.6 Finalization

After selecting all values:

1. relink exact final bundle;
2. derive exact final ABI;
3. regenerate every worst-case fixture;
4. rerun every measurement;
5. verify all selected limits;
6. verify predicted/observed resource agreement;
7. bind final bundle and ABI hashes;
8. construct typed `BoundCalibration` entries;
9. reject any stale candidate report.

### 16.7 Calibration outputs

Produce:

- final `BoundAssignment`;
- per-bound calibration evidence;
- per-operation resource reports;
- final target-limit report;
- final linked bundle;
- final transaction ABI;
- calibration identity.

### 16.8 No automatic release

Calibration success produces release inputs.

It does not publish a final release or flip profile status automatically.

---

## 17. Evidence-report census

### 17.1 Required model reports

At minimum:

- unit-test report;
- property-test report;
- model/realization conformance report.

The current deployment profile directly names unit and property reports. Later
profile evolution may separately bind the realization-conformance report or
include it in a typed script-integration/evidence index.

Until schema changes are accepted, the release manifest should bind all
additional reports even when the deployment profile has no dedicated field.

### 17.2 Required backend reports

At minimum:

- compiler-analysis report;
- backend-pattern report;
- linked-bundle relation execution report;
- relation coverage report;
- script-integration report;
- resource calibration report.

### 17.3 Required representation reports

At minimum:

- representation-safety report;
- representation-minimality report when minimality is claimed;
- public-declassification report where used;
- lifecycle/constructibility report.

A deployment that does not claim confidential/minimal representation at one
seam may omit that minimality claim only if the selected explicit policy is
stated accurately and all required safety evidence passes.

### 17.4 Required target reports

One verified report per required target/dependency claim.

### 17.5 Required independent reports

Separately:

- event projection;
- canonical query;
- receipt accounting.

These correspond to distinct deployment-profile fields where already modeled.

### 17.6 Required document/publication reports

- generated artifact freshness;
- realization appendix weld;
- document label/anchor conformance;
- reproducible PDF/document build where release includes documents;
- canonical flattened source where included.

### 17.7 Report status

Every required report must be:

```text
passed
```

Reject:

```text
failed
incomplete
unsupported
infrastructure_error
skipped
```

for a final production release.

---

## 18. Independent observer validation

### 18.1 Event report

Validate:

- exact context;
- exact canonical event order;
- genesis clear;
- burn/clear event identity;
- block context;
- ASH values;
- burn records;
- clear values;
- exact event count;
- report identity.

### 18.2 Query report

Validate:

- exact context;
- supported wire schema;
- expected architecture hash;
- every tested address;
- semantic query validation;
- canonical bytes;
- zero-result cases;
- report identity.

### 18.3 Accounting report

Validate:

- exact context;
- residue event sequence;
- historical class totals;
- current state fields;
- current receipt/ASH/distribution terms;
- report identity.

### 18.4 Independence declaration

Require a typed or canonical declaration identifying:

- candidate implementation;
- source/version;
- shared dependencies;
- whether model/reference code is linked;
- test fixture source;
- tool identity.

A declaration does not prove independence automatically, but its absence
blocks an evidence claim labeled independent.

### 18.5 No substitution

One report cannot be used as another because:

- event equality does not prove query computation;
- query equality does not prove event recognition;
- aggregate accounting equality does not prove residue event equality.

---

## 19. Deployment-profile construction

### 19.1 Architecture-owned profile type

Construct:

```rust
architecture::DeploymentProfile
```

from validated release inputs.

The release package does not define a competing deployment-profile type unless
a later profile schema revision requires a typed migration.

### 19.2 Required fields

Populate:

- supported profile schema;
- final status;
- architecture semantic hash;
- network ID;
- genesis ID;
- script limits;
- calibrated bounds;
- dependency evidence;
- artifact hashes;
- test evidence.

### 19.3 Artifact hashes

Populate current architecture-owned fields:

```text
normative_rust
compiler_configuration
emitted_script_bundle
reference_indexer
architecture_json
architecture_toml
canonical_wire_vectors
```

Each field requires one documented canonical hash recipe.

Do not hash an arbitrary directory using host-dependent metadata.

### 19.4 Test evidence

Populate:

```text
unit_test_report_hash
property_test_report_hash
independent_event_projection_report_hash
independent_attestation_query_report_hash
independent_receipt_accounting_report_hash
script_integration_report_hash
```

Additional report hashes not represented directly in profile schema 2 must be
bound by the release manifest or require a future profile-schema revision.

Do not overload one field with a semantically different report.

### 19.5 Dependency evidence

For every architecture dependency:

- required dependency appears exactly once;
- optional evidence appears at most once;
- status follows policy;
- nonzero evidence hash;
- nonblank tool version;
- nonblank test name;
- target/deployment context compatible.

### 19.6 Bound calibration entries

For every calibrated architecture bound:

- exactly one entry;
- final value;
- evidence hash;
- exact final script-bundle hash;
- measured weight;
- measured witness bytes;
- measured opcode/project cost;
- value dominates manifest minimum;
- measurements fit declared script limits.

### 19.7 Deployment validation

Call:

```rust
architecture::validate_deployment_release(
    architecture,
    &profile,
)
```

Any returned error blocks release.

The release package may add stricter checks. It must not weaken architecture
validation.

### 19.8 Deployment-profile hash

Compute:

```rust
architecture::deployment_profile_hash(...)
```

or its supported successor.

Verify:

- algorithm identifier;
- canonical typed value;
- domain separation;
- deterministic result;
- profile bytes/publication bind the same typed profile.

---

## 20. Artifact hashing

### 20.1 One typed artifact registry

Represent every release asset as a typed entry.

Conceptually:

```rust
pub struct PublicationAsset {
    pub kind: PublicationAssetKind,
    pub path: CanonicalReleasePath,
    pub media_type: PublicationMediaType,
    pub schema: Option<PublicationSchema>,
    pub bytes: Vec<u8>,
    pub digest: ArtifactDigest,
    pub source: PublicationSourceBinding,
}
```

> Illustrative API; not frozen.

### 20.2 Canonical path rules

Release paths must be:

- relative;
- normalized;
- separator-stable;
- free of `..`;
- free of host absolute paths;
- unique;
- deterministically ordered.

### 20.3 File hash recipe

For a single file:

```text
sha256(file bytes)
```

or another explicitly versioned algorithm.

Do not include:

- mtime;
- uid/gid;
- host path;
- filesystem inode;
- ambient permissions

unless a canonical archive schema explicitly requires normalized mode bits.

### 20.4 Source-tree artifact hash

A hash such as `normative_rust` requires an explicit canonical recipe.

Possible recipe:

```text
domain separator
+
for every included canonical relative path in sorted order:
    path length
    path bytes
    file length
    file bytes
```

The included file census must be typed or explicitly generated from package
ownership—not inferred from current Git status without a stable policy.

Before use, define:

- included packages/files;
- exclusions;
- symlink policy;
- line-ending policy;
- executable-bit policy;
- submodule policy;
- generated-file policy;
- algorithm identifier.

### 20.5 Archive hash

If a release archive is published, its byte hash may differ from the typed
release identity.

To make archive bytes reproducible:

- sort paths;
- normalize timestamps;
- normalize owner/group;
- normalize modes;
- choose one compression implementation/version or uncompressed canonical
  archive;
- exclude host-specific metadata;
- verify extracted files against the release manifest.

### 20.6 Hash validation

After staging publication:

- read back bytes;
- recompute file hashes;
- recompute archive hash;
- compare asset census;
- compare release manifest;
- only then publish atomically.

---

## 21. Release manifest

### 21.1 Purpose

The release manifest is the reviewable index binding every final identity,
artifact, and report.

It supplements the architecture deployment profile where profile schema 2 has
no dedicated field for newer compiler-era artifacts.

### 21.2 Conceptual structure

```rust
pub struct ReleaseManifest {
    pub schema_version: ReleaseManifestSchema,

    pub release_status: ReleaseStatus,
    pub release_label: String,
    pub source_revision: SourceRevision,
    pub release_date: ReproducibleDate,

    pub attestation: AttestationReleaseBinding,
    pub architecture: ArchitectureReleaseBinding,
    pub realization: RealizationReleaseBinding,
    pub compiler: CompilerReleaseBinding,
    pub target: TargetReleaseBinding,
    pub backend: BackendReleaseBinding,
    pub bundle: LinkedBundleReleaseBinding,
    pub transaction_abi: TransactionAbiReleaseBinding,
    pub calibration: CalibrationReleaseBinding,

    pub evidence: EvidenceIndex,
    pub assets: PublicationAssetIndex,

    pub deployment_profile: DeploymentProfileBinding,
}
```

> Illustrative API; not frozen.

### 21.3 Status

Use typed status:

```text
candidate
validated
final
```

Only the final validated value may be published under a final release label.

### 21.4 Current profile relationship

The release manifest is not a competing deployment profile.

It:

- binds the profile;
- indexes additional compiler-era artifacts;
- records publication assets;
- records exact evidence identities;
- records source/release provenance.

The deployment profile remains the architecture-owned deployment release
object.

### 21.5 Canonical publication

Potential artifact:

```text
release-manifest.json
```

The name is illustrative.

It requires:

- typed source;
- schema;
- canonical ordering;
- deterministic bytes;
- unknown-field rejection;
- domain-separated identity;
- generator/checker split.

---

## 22. Reproducible release metadata

### 22.1 Release date

The release date is an explicit typed input.

It must not default to the current local date inside canonical release APIs.

For document builds, the same release/source date is passed through the
reproducible build environment.

### 22.2 Source revision

The release binds an exact source revision.

A dirty working tree cannot be represented by the same source revision without
an explicit source-snapshot identity.

Production policy should require a clean source tree.

### 22.3 Tool versions

Record exact:

- Rust toolchain;
- Cargo lock identity;
- compiler tool version;
- backend/linker/transaction/vector/release tool versions;
- Elements target/node revision;
- document toolchain version where required.

Tool version is provenance. Bundle/report hashes remain the direct artifact
identity.

### 22.4 No environment leakage

Canonical release metadata excludes:

- build hostname;
- username;
- absolute paths;
- temporary directories;
- process IDs;
- local timezone;
- ambient locale;
- credentials.

### 22.5 Source-date environment

Publication orchestration should set one stable source date for:

- TeX;
- Biber;
- archive creation;
- any generated metadata.

The exact environment belongs to implementation/ADR after Phase 0 resolves PDF
reproducibility.

---

## 23. Publication model

### 23.1 Staging

Publication writes to a unique staging directory under the destination
filesystem where possible.

Steps:

1. verify destination policy;
2. render every asset into staging;
3. compute and verify hashes;
4. validate asset census;
5. validate release manifest;
6. validate deployment profile bytes;
7. verify reproducibility settings;
8. atomically rename or publish staging;
9. return publication receipt.

### 23.2 Existing destination

If destination already exists:

- reject by default;
- permit explicit replacement only under a typed option;
- never merge two releases;
- never leave stale unowned files.

### 23.3 Asset census

The release owns the exact file census.

Unexpected staged or existing files cause failure.

### 23.4 Atomicity

No partially published final release should appear after a failed validation.

Where filesystem-level atomic directory replacement is unavailable, use a
documented safe sequence and final completion marker.

The completion marker is valid only after all files verify.

### 23.5 Non-writing checker

`check-release` reads an existing release and compares it with the expected
validated typed release without writing.

### 23.6 Release mirrors

Repository-local mirrors, if retained, are explicit assets written by a
separate command or publication option.

Release validation must not rely on mutable repository mirrors as the semantic
source.

---

## 24. Command-line tools

Potential binaries include:

```text
calibrate-deployment
assemble-release
check-release
publish-release
```

Names are illustrative.

### 24.1 ADR-010 classification

#### `calibrate-deployment`

Likely stdout-result command:

- emits one JSON calibration result/report on success;
- writes large reports to explicit output paths when requested;
- refuses stdout TTY if result data is emitted.

#### `assemble-release`

Likely stdout-result command:

- non-writing;
- emits a JSON candidate/validation report;
- does not publish assets.

#### `check-release`

Stdout-result command:

- emits one JSON check report;
- never repairs;
- TTY refusal applies.

#### `publish-release`

Side-effect command:

- writes only to `--output`;
- stdout empty;
- JSON diagnostics/status on stderr;
- no implicit current-directory output.

### 24.2 Secret safety

Never log:

- private keys;
- signer credentials;
- RPC cookies;
- raw credential URLs;
- secret blinding data;
- production witness secrets.

Release should not receive private keys at all.

### 24.3 Exit codes

Use ADR-010:

```text
0 success
1 runtime/validation/release failure
2 usage
```

A failed release gate is exit 1, not usage.

Invalid CLI arguments are exit 2.

### 24.4 Panics

Use the shared panic hook with payload hidden unless debug was explicitly
enabled after parsing.

### 24.5 Reproducible diagnostics

Canonical release reports are written as assets or stdout result objects.

Tracing diagnostics are not included in release identity.

---

## 25. Release error model

Errors should identify the failing assurance boundary.

Candidate classes include:

```rust
pub enum ReleaseError {
    UnsupportedReleasePolicy,
    UnsupportedReleaseManifestSchema,
    UnsupportedDeploymentProfileSchema,
    CandidateArtifactNotFinal,

    ArchitectureReleaseFailed,
    ArchitectureEnvelopeInvalid,
    ArchitectureHashMismatch,
    BehaviouralVersionGateFailed,
    AttestationPinMismatch,
    ArchitecturePublicationMismatch,

    RealizationIdentityMismatch,
    RealizationScopeIncomplete,
    RealizationValidationFailed,
    ModelConformanceReportMissing,
    ModelConformanceReportFailed,

    CompilerIdentityMismatch,
    CompilerScopeMismatch,
    CompilerAnalysisIncomplete,
    CompilerReportMissing,
    CompilerReportFailed,

    TargetIdentityMismatch,
    DeploymentInstanceMismatch,
    NetworkIdMismatch,
    GenesisIdMismatch,
    ActivationEvidenceMissing,
    ProductionTargetEvidenceMissing,

    BackendConfigurationMismatch,
    RelocatableBundleMismatch,
    LinkedBundleNotFinal,
    LinkedBundleIdentityMismatch,
    UnresolvedLinkedReference,
    RelationCarrierCensusMismatch,

    TransactionAbiIdentityMismatch,
    TransactionAbiIncomplete,
    TransactionAbiBundleMismatch,
    UnsupportedRepresentationClaim,
    PermissionlessLifecycleIncomplete,

    MissingBoundCalibration(architecture::BoundId),
    DuplicateBoundCalibration(architecture::BoundId),
    InvalidBoundCalibration(architecture::BoundId),
    CalibrationBundleMismatch,
    CalibrationAbiMismatch,
    CalibrationReportStale,
    ResourcePredictionMismatch,
    TargetResourceLimitExceeded,

    MissingDependencyEvidence(architecture::DependencyId),
    DuplicateDependencyEvidence(architecture::DependencyId),
    DependencyEvidenceFailed(architecture::DependencyId),
    DependencyEvidenceIdentityMismatch(architecture::DependencyId),

    MissingEvidenceReport(EvidenceKind),
    DuplicateEvidenceReport(EvidenceKind),
    EvidenceReportFailed(EvidenceKind),
    EvidenceReportIncomplete(EvidenceKind),
    EvidenceIdentityMismatch(EvidenceKind),
    EvidenceSchemaUnsupported(EvidenceKind),
    ZeroEvidenceHash(EvidenceKind),

    IndependentEventReportMissing,
    IndependentQueryReportMissing,
    IndependentAccountingReportMissing,
    IndependentObserverRequirementNotMet(EvidenceKind),

    GeneratedArtifactStale(PublicationAssetKind),
    GeneratedArtifactMismatch(PublicationAssetKind),
    UnknownPublicationAsset(String),
    MissingPublicationAsset(PublicationAssetKind),
    DuplicatePublicationPath(String),
    UnsafePublicationPath(String),

    ArtifactHashMismatch(PublicationAssetKind),
    SourceTreeHashFailure,
    ArchiveHashMismatch,
    NonReproducibleArtifact(PublicationAssetKind),

    DeploymentProfileConstructionFailed,
    DeploymentReleaseValidationFailed,
    DeploymentProfileHashMismatch,

    SourceTreeDirty,
    SourceRevisionMismatch,
    ToolchainIdentityMissing,
    ReleaseDateMissing,

    DestinationExists,
    PublicationStagingFailed,
    PublicationVerificationFailed,
    AtomicPublicationFailed,

    NoWaiverPolicy,
    NonDeterministicRelease,
}
```

> Illustrative vocabulary; not frozen.

Errors may carry typed nested causes and lists of architecture deployment
errors.

Error display must not expose secrets.

---

## 26. Validation order

The release package should use a stable validation order so diagnostics and
reports remain predictable.

Recommended order:

1. release policy and input schema;
2. source revision and cleanliness;
3. architecture release;
4. architecture publications/document weld;
5. realization identity/completeness;
6. model conformance;
7. compiler analysis and target plan;
8. target/deployment identity;
9. backend/linked bundle;
10. transaction ABI;
11. calibration;
12. target dependency evidence;
13. relation/backend execution evidence;
14. representation/lifecycle evidence;
15. independent observer reports;
16. artifact census and hashes;
17. deployment-profile construction;
18. architecture deployment-release validation;
19. deployment-profile hash;
20. release manifest;
21. reproducibility checks;
22. publication staging.

This ordering is for deterministic attribution.

Release success still requires every check.

---

## 27. Testing strategy

### 27.1 Unit tests

Cover:

- release-state transitions;
- identity graph validation;
- report census;
- artifact census;
- path normalization;
- hash recipes;
- deployment-profile construction;
- deployment error mapping;
- release manifest identity;
- candidate/final status;
- no-waiver behavior;
- deterministic ordering.

### 27.2 Identity mismatch tests

Mutate each binding independently:

- architecture;
- realization;
- compiler;
- target;
- bundle;
- ABI;
- vector set;
- calibration;
- observer context;
- deployment profile.

Require precise failure.

### 27.3 Missing evidence tests

Remove each required report in turn and require failure.

Repeat for:

- zero hash;
- failed status;
- incomplete status;
- stale bundle binding;
- unsupported schema;
- duplicate report.

### 27.4 Calibration tests

- complete valid synthetic calibration;
- missing bound;
- duplicate bound;
- value below minimum;
- measurement above target limit;
- stale candidate;
- wrong bundle;
- wrong ABI;
- shared-bound operation omitted;
- final remeasurement mismatch;
- nonmonotone search policy rejected when binary search assumed.

### 27.5 Artifact tests

- exact architecture publications;
- stale architecture JSON;
- stale architecture TOML;
- unexpected release file;
- missing release file;
- duplicate path;
- unsafe path;
- wrong file hash;
- wrong archive hash;
- host-dependent metadata normalization;
- no partial final publication after injected failure.

### 27.6 Profile tests

- valid final profile;
- draft profile;
- wrong architecture hash;
- wrong network/genesis;
- missing dependency evidence;
- missing test report;
- wrong artifact hash;
- unsupported schema;
- deployment-profile hash mutation.

Reuse architecture profile fixtures where possible without creating a second
profile semantics implementation.

### 27.7 Reproducibility tests

- repeated release assembly equality;
- repeated publication bytes equality;
- different staging paths produce equal output;
- different host time with same release date produces equal output;
- canonical archive equality;
- changed semantic input changes expected identity;
- changed nonidentity diagnostic does not change release identity.

### 27.8 Publication failure tests

Inject failure during:

- asset render;
- hash verification;
- archive creation;
- final rename.

Require no valid final release marker and no partially accepted destination.

### 27.9 CLI subprocess tests

For every release binary:

- help;
- version;
- usage errors;
- JSON diagnostics;
- TTY refusal where applicable;
- empty stdout for side-effect commands;
- no raw secret-bearing argv;
- correct exit classes;
- no plain text.

### 27.10 Public API tests

An external integration test should prove a downstream release runner can:

- assemble typed inputs;
- validate candidate;
- inspect failures;
- obtain validated release;
- check an existing publication;
- publish to explicit output;
- do so without mutating semantic internals or using private constructors.

---

## 28. Determinism and identity

### 28.1 Release identity

The release identity is domain-separated from:

- architecture hashes;
- realization identity;
- compiler identity;
- target identity;
- bundle identity;
- ABI identity;
- deployment-profile hash;
- archive-byte hash.

It should bind:

- release manifest schema;
- release policy identity;
- source revision;
- explicit release date/label where policy includes them;
- all semantic/target/bundle/ABI/calibration identities;
- evidence index;
- publication asset index;
- deployment-profile identity/hash.

### 28.2 Candidate and final identity

Candidate identity may bind incomplete status and is not interchangeable with
final release identity.

Final release identity is produced only after validation.

### 28.3 Canonical ordering

Order:

- operations by architecture code;
- bounds by BoundId code;
- dependencies by DependencyId code;
- evidence by typed EvidenceKind/ID;
- assets by canonical release path;
- reports by typed identity;
- errors by stable validation order and typed ID.

### 28.4 No ambient nondeterminism

No canonical release output depends on:

- current time;
- directory enumeration;
- hash-map order;
- process completion order;
- absolute paths;
- host locale;
- user name;
- random archive metadata.

### 28.5 Explicit randomness

Release assembly itself should not require randomness.

Any cryptographic release signing introduced later must be a separate explicit
input/output and decision. It must not alter unsigned canonical release
identity silently.

---

## 29. Generated artifacts

### 29.1 Candidate final release tree

A future release tree may resemble:

```text
release/
├── release-manifest.json
├── deployment-profile.json
├── architecture.json
├── architecture.toml
├── declassification.json
├── model-labels.json
│
├── bundle/
│   ├── bundle-manifest.json
│   ├── constructors/
│   ├── programs/
│   └── disassembly/
│
├── abi/
│   └── transaction-abi.json
│
├── vectors/
│   ├── vector-manifest.json
│   └── canonical-transactions/
│
├── reports/
│   ├── unit-tests.json
│   ├── property-tests.json
│   ├── model-conformance.json
│   ├── compiler-analysis.json
│   ├── relation-coverage.json
│   ├── representation-safety.json
│   ├── representation-minimality.json
│   ├── resource-calibration.json
│   ├── script-integration.json
│   ├── independent-events.json
│   ├── independent-query.json
│   ├── independent-accounting.json
│   └── substrate/
│
└── documents/
    ├── realization.md
    ├── attestation.pdf
    └── realization-or-release-report.pdf
```

This tree is illustrative, not a frozen file census.

The exact census must be decided before first final publication.

### 29.2 Source versus generated assets

Source documents may be included as release assets without becoming machine
semantic inputs.

Generated publications must match typed expected values.

### 29.3 Optional documents

A deployment release may or may not include rendered documents depending on
policy.

If included, reproducibility and exact document identity become release gates.

### 29.4 Artifact ownership

The release package assembles assets owned by earlier packages.

It does not become the generator of their semantics.

---

## 30. Binaries and ADR-010 details

### 30.1 `calibrate-deployment`

Potential interface:

```text
calibrate-deployment
    --target-config <typed input>
    --deployment <typed input>
    --output <directory>
```

Behavior:

- candidate measurements written to output;
- one JSON summary on stdout if classified as result command;
- no implicit source-tree writes;
- target/node diagnostics on JSON stderr;
- exit 1 on any failed required candidate/final measurement.

### 30.2 `assemble-release`

Non-writing validation:

```text
assemble-release
    --input <release-input descriptor>
```

Returns one JSON candidate or validation report.

It does not publish final assets.

### 30.3 `check-release`

Reads one existing release and emits one JSON report.

It never repairs.

### 30.4 `publish-release`

Requires a `ValidatedRelease` input or reconstructs and validates one before
writing.

It writes to one explicit destination and keeps stdout empty.

### 30.5 CLI input schema

If a release input descriptor is serialized, it is a typed external
orchestration schema.

It contains references to typed artifacts and expected identities.

It is not the semantic source for architecture or realization.

Unknown fields reject.

---

## 31. Dependency and unsafe-code policy

The package inherits ADR-011.

Requirements:

- Rust edition 2024;
- workspace MSRV;
- workspace lints;
- `unsafe_code = "deny"`;
- Cargo `--locked`;
- permissive dependencies;
- deterministic output;
- no network access in pure library APIs;
- no hidden target/node probing;
- no source-tree mutation during checks.

Likely dependencies include:

```text
architecture
realization
compiler
target-elements
tapscript
linker
transaction
vectors
artifacts
serde
serde_json
sha2
tempfile
thiserror/anyhow as appropriate
```

A deterministic archive library may be added after its metadata behavior is
reviewed.

Avoid shelling out from pure release validation. Explicit orchestration binaries
may invoke documented tools with exact versions and machine-readable reports.

---

## 32. Milestones

### REL1 — Package and state model

Deliver:

- workspace package;
- release candidate/validated types;
- policy/error/identity skeleton;
- no publication yet.

### REL2 — Identity graph and evidence census

Deliver:

- cross-binding validation;
- report registry;
- missing/mismatch failures;
- no-waiver policy.

### REL3 — Artifact registry and hashing

Deliver:

- typed asset paths;
- canonical file/source-tree recipes;
- deterministic manifest;
- artifact tests.

### REL4 — Architecture and realization gates

Deliver:

- architecture release invocation;
- publication equality;
- document weld binding;
- realization completeness;
- model-conformance binding.

### REL5 — Compiler/target/bundle/ABI gates

Deliver exact identity and scope validation.

### REL6 — Calibration orchestrator

Deliver:

- candidate loop;
- whole-transaction measurement;
- shared-bound coverage;
- final remeasurement;
- final linked bundle/ABI.

### REL7 — Evidence gates

Deliver:

- model/property;
- relation coverage;
- representation;
- substrate;
- script integration;
- independent observer validation.

### REL8 — Deployment-profile assembly

Deliver:

- typed profile construction;
- architecture deployment validation;
- profile hash;
- canonical profile publication.

### REL9 — Release manifest

Deliver complete canonical release index.

### REL10 — Non-writing checker

Deliver exact release-directory/archive check.

### REL11 — Atomic publisher

Deliver staging and deterministic publication.

### REL12 — CLI tools

Deliver ADR-010-compliant commands and subprocess tests.

### REL13 — Reproducibility gate

Deliver two-clean-build byte equality for release assets.

### REL14 — First final deployment release

Deliver only after Phase-12 roadmap gate passes.

---

## 33. Preliminary release-package exit criteria

The package foundation is ready before final evidence completion when:

- [ ] `packages/release` is a workspace member;
- [ ] package metadata follows workspace policy;
- [ ] dependency direction is acyclic;
- [ ] release candidate and validated release are distinct types/states;
- [ ] release policy is typed and identity-bound;
- [ ] no general waiver mechanism exists;
- [ ] identity graph validation is implemented;
- [ ] evidence report census is typed;
- [ ] artifact registry and safe paths are implemented;
- [ ] deployment-profile construction uses the architecture-owned type;
- [ ] architecture release/deployment validators are called rather than
      reimplemented weakly;
- [ ] release assembly is pure/non-writing;
- [ ] candidate status cannot be published as final through the safe API;
- [ ] errors identify assurance boundaries;
- [ ] deterministic ordering tests pass;
- [ ] public API tests pass;
- [ ] debug/release tests pass;
- [ ] Clippy with `-D warnings` passes;
- [ ] the checkout remains clean.

---

## 34. Final release exit criteria

A production release may be published only when:

- [ ] source revision is exact and production policy's clean-source rule
      passes;
- [ ] explicit release date is supplied;
- [ ] anchor-set identity and anchor set are correct;
- [ ] architecture release passes;
- [ ] architecture JSON/TOML equal typed canonical values;
- [ ] realization document appendix and masthead welds pass;
- [ ] complete realization scope is final;
- [ ] model/realization conformance passes;
- [ ] model unit and property reports pass;
- [ ] compiler relation census equals realization census;
- [ ] compiler target plan is complete;
- [ ] exact production target and deployment instance are bound;
- [ ] activation/network/genesis evidence passes;
- [ ] final linked bundle identity verifies;
- [ ] every mandatory reference and relocation is resolved;
- [ ] every relation carrier is reachable;
- [ ] final transaction ABI identity verifies;
- [ ] permissionless construction/lifecycle checks pass;
- [ ] every deployment-calibrated bound appears exactly once;
- [ ] whole-transaction calibration covers every affected operation;
- [ ] final relink/ABI/regeneration was remeasured;
- [ ] all resource limits pass;
- [ ] every required dependency report is verified;
- [ ] relation-indexed target execution report passes;
- [ ] relation coverage is complete;
- [ ] representation safety report passes;
- [ ] every claimed minimality mode has a passing report;
- [ ] script-integration report passes;
- [ ] independent event report passes;
- [ ] independent query report passes;
- [ ] independent accounting report passes;
- [ ] observer independence declarations are present;
- [ ] every artifact hash is nonzero and correct;
- [ ] every publication asset is present exactly once;
- [ ] no unexpected asset exists;
- [ ] architecture deployment-release validation passes;
- [ ] deployment-profile hash verifies;
- [ ] release manifest identity verifies;
- [ ] every canonical report and publication is reproducible;
- [ ] release check is non-writing and passes;
- [ ] staged publication revalidates before final rename;
- [ ] final release archive/directory is byte-reproducible;
- [ ] no production secret appears in publication;
- [ ] no required check is skipped, waived, incomplete, unsupported, or
      infrastructure-error;
- [ ] the source checkout remains clean after checks/publication unless the
      caller explicitly requested a repository mirror;
- [ ] the published release receipt records exact final paths and hashes.

---

## 35. Non-goals

The release package does not:

- define or edit protocol semantics;
- decide architecture versioning after the fact;
- repair failed evidence;
- invent missing reports;
- waive failed dependencies;
- generate production private keys;
- sign deployment transactions;
- broadcast transactions;
- manage production funds;
- guarantee network liveness;
- replace source review;
- replace independent observer implementations;
- treat finite vectors as universal proof;
- parse planning Markdown as release policy;
- use generated architecture files as semantic source;
- silently update `Cargo.lock`;
- write to the source tree during checks;
- publish a candidate as final.

---

## 36. Open questions

### 36.1 Release manifest schema

Define before first committed compiler-era release publication.

### 36.2 Additional deployment-profile fields

Profile schema 2 may not contain dedicated fields for:

- realization identity;
- compiler analysis report;
- relation coverage;
- representation reports;
- transaction ABI.

Initial release manifest can bind them.

A future profile schema revision may add fields, requiring architecture schema
and deployment-profile policy review.

### 36.3 Source-tree hash recipe

Must be decided before populating `normative_rust`.

### 36.4 Compiler configuration artifact

Define exactly which typed configurations and source/tool versions are included
in the compiler-configuration hash.

### 36.5 Reference indexer artifact

The deployment profile names `reference_indexer`.

Define whether this means:

- model/reference indexer binary;
- source snapshot;
- canonical package/archive;
- implementation plus configuration.

Keep it distinct from the independent candidate indexer report.

### 36.6 Release archive format

Choose a deterministic format and implementation.

### 36.7 Release signing

No release-signature policy is accepted yet.

If introduced, define:

- unsigned release identity;
- signer role;
- signature artifact;
- key custody;
- rotation;
- threshold policy;
- verification.

### 36.8 Calibration package extraction

If calibration orchestration becomes large or useful outside release, consider
a dedicated package through a later decision.

### 36.9 Production target runner

Decide how the final production-equivalence and activation evidence is
obtained without embedding production credentials in release tooling.

### 36.10 Repository mirrors

Decide whether final bundles/reports/documents are mirrored into tracked
`archive/` or distributed only outside Git.

---

## 37. Risks

### 37.1 Release package becomes a second semantic implementation

It may be tempted to recalculate formulas or operation scope.

Mitigation:

- consume typed upstream identities;
- call upstream validators;
- compare relation censuses;
- no operation-specific formulas;
- package review.

### 37.2 Evidence identity graph is incomplete

A passed report may be attached to changed bytes.

Mitigation:

- explicit cross-binding matrix;
- typed report envelopes;
- zero/placeholder rejection;
- mutation tests.

### 37.3 Profile schema lags toolchain evidence

New reports may be bound only by the release manifest.

Mitigation:

- canonical release manifest;
- explicit profile relationship;
- future schema revision rather than field overloading.

### 37.4 Source-tree hash is nonreproducible

Filesystem metadata or incomplete census may alter it.

Mitigation:

- explicit canonical recipe;
- typed census;
- path normalization;
- reproducibility tests.

### 37.5 Calibration search misses a coupled maximum

A shared bound or representation combination may exceed limits.

Mitigation:

- typed affected-operation derivation;
- objective-specific fixtures;
- final full remeasurement;
- relation/branch census;
- fail closed.

### 37.6 Independent evidence is only nominally independent

Candidate tools may reuse reference code.

Mitigation:

- independence declaration;
- source/dependency review;
- separate tool identity;
- no model linking under required policy.

### 37.7 Publication is partially written

A failure could leave a directory appearing final.

Mitigation:

- staging;
- final verification;
- atomic rename/completion marker;
- failure injection tests.

### 37.8 Archive bytes vary by host

Timestamps, modes, ownership, or compression can differ.

Mitigation:

- normalized metadata;
- pinned implementation;
- byte reproducibility checks;
- explicit source date.

### 37.9 Strict gate creates operational pressure for waivers

Teams may be tempted to bypass a failing report.

Mitigation:

- no waiver policy;
- clear development/candidate mode;
- fix evidence before final status;
- separate research residuals from final obligations.

### 37.10 Secrets enter reports

Target runners or transaction fixtures may leak credentials/blinding data.

Mitigation:

- release never receives private keys;
- typed secret-free report projections;
- secret-leak tests;
- ADR-010;
- external diagnostic attachments excluded from release assets unless reviewed.

### 37.11 Candidate/final confusion

A regtest or pilot release may be labeled production-ready.

Mitigation:

- typed policy/status;
- scope and target identity;
- final-only publication API;
- prominent release manifest status.

---

## 38. Definition of done

The release package plan is fulfilled for the first production deployment when
the repository can assemble one exact typed release candidate from the final
architecture, complete realization, compiler analysis, production target,
linked bundle, transaction ABI, calibration results, model/backend/substrate
reports, and separately implemented event/query/accounting reports; validate
every identity and evidence binding; construct and validate the
architecture-owned final deployment profile; compute its domain-separated
identity; produce a canonical release manifest and exact asset census; and
publish one byte-reproducible, secret-free release atomically—while providing a
non-writing checker and refusing every missing, stale, mismatched, candidate,
waived, skipped, or incomplete obligation.

---

## 39. One-line package contract

> `release` is the final fail-closed assembly boundary: it consumes exact typed
> architecture, realization, compiler, target, bundle, ABI, calibration,
> artifact, and evidence identities; verifies their complete cross-binding;
> constructs and validates the final deployment profile; produces one canonical
> release manifest and reproducible asset set; and publishes only a validated
> final release—without redefining semantics, generating secrets, repairing
> evidence, accepting waivers, or confusing a candidate with a deployment.
