# Linker Package Plan

> **Status:** PLANNED
> **Planned source directory:** `packages/linker`
> **Planned Cargo package:** `tripod-linker`
> **Planned Rust library name:** `linker`
> **Implementation phases:** Phase 4 onward — first linked `compact-ash`
> bundle; later operation and release phases extend the same bundle
> **Depends on active backend:** initially `tripod-tapscript`
> **Likely typed dependencies:** architecture/realization/compiler identities,
> `tripod-target-elements`, and backend-owned relocatable artifacts
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-is-normative.md),
> [D002](../decisions/002-target-independent-realization-layer.md),
> [D003](../decisions/003-multiple-backends-tapscript-first.md),
> [D004](../decisions/004-translation-validation-over-compiler-trust.md),
> [D005](../decisions/005-value-parametric-asset-rigid.md),
> [D006](../decisions/006-canonical-transaction-layout-abi.md)
> **Open research dependencies:**
> [`../research/state-object-constructor.md`](../research/state-object-constructor.md),
> [`../research/public-declassification.md`](../research/public-declassification.md),
> and [`../research/settlement-layout.md`](../research/settlement-layout.md)
> affect constructor, relocation, and layout closure
> **Authority:** Target-specific artifact resolution and deterministic bundle
> construction beneath typed realization/compiler/backend artifacts
> **Machine-consumed planning document:** no

---

## 1. Purpose

The `linker` package will transform typed relocatable backend output into one
deterministic linked deployment bundle.

For the first backend, it consumes:

- the exact typed Elements target;
- a validated relocatable tapscript bundle;
- typed deployment constants;
- typed constructor/link policy;
- calibrated finite bounds, once available;
- deterministic taptree weighting/configuration;
- target-specific resource formulas.

It owns:

- symbol and constructor resolution;
- typed relocation validation and substitution;
- object-constructor reference graphs;
- strongly connected component analysis;
- reference-strategy validation;
- static program and subtree resolution;
- deterministic tapscript leaf ordering;
- deterministic taptree assembly;
- control-path metadata required by later transaction construction;
- deployment constant substitution;
- resource-formula closure over linked programs;
- relation/proof/placement provenance preservation;
- candidate and final linked-bundle construction;
- linked-bundle identity and canonical manifest;
- link-time reproducibility checks.

It does not own:

- protocol semantics;
- target-independent relations;
- proof-alternative selection outside compiler/backend-approved choices;
- backend instruction selection;
- transaction and witness construction;
- production key generation;
- Elements issuance/genesis ceremony;
- worst-case transaction construction;
- measurement execution;
- independent indexer/auditor evidence;
- final deployment-profile release validation.

The planned direction is:

```text
validated relocatable backend bundle
        +
typed target and deployment constants
        +
typed linker configuration
        ↓
symbol/reference graph construction
        ↓
reference classification and SCC analysis
        ↓
constructor/program/taptree resolution
        ↓
typed relocation substitution
        ↓
linked resource and provenance closure
        ↓
CandidateLinkedBundle or final LinkedBundle
        ↓
transaction ABI and transaction construction
        ↓
measurement/evidence/release
```

The linker is not a semantic compiler. It must preserve exactly the obligations
and provenance it receives.

---

## 2. Why a linker is required

The backend cannot know every concrete deployment value at target-program
emission time.

Unknown or late-bound facts may include:

- protocol asset IDs derived during issuance setup;
- operator and deployment public keys;
- exact network and genesis identity;
- calibrated finite bounds;
- static code-subtree roots;
- sibling object-constructor references;
- taptree roots;
- operation leaf hashes;
- target program commitments;
- metadata schema/domain constants;
- final control paths;
- target-specific constructor commitment values.

Some references are acyclic and can be resolved directly.

Others participate in cycles.

Examples include:

- a constructor whose operation programs recreate the same constructor;
- STATE and RESV relations that authenticate each other;
- metadata-dependent successors whose program commitment depends on static
  code and dynamic metadata;
- roots whose target programs need continuity with the predecessor code
  subtree.

Treating these values as ad hoc compiler constants would create:

- circular byte-generation code;
- duplicated symbol logic;
- hidden fixed-point assumptions;
- non-deterministic build order;
- late manual patching;
- deployment-specific semantics in the backend;
- release bundles that cannot be independently reproduced.

The linker makes these dependencies explicit, classifies them, resolves every
valid reference under a declared strategy, and rejects unresolved or
ill-founded cycles.

---

## 3. Assurance boundary

The linker establishes:

- every declared symbol/reference resolves under one typed strategy;
- every mandatory relocation is applied exactly once as specified;
- linked program bytes match typed backend programs plus typed substitutions;
- taptree/program assembly is deterministic;
- final constructor references are internally consistent;
- relation/proof/placement provenance survives linking;
- linked resource formulas reference final program shapes;
- no unresolved mandatory symbol remains;
- candidate/final bundle identities are reproducible.

The linker does not establish:

- that the realization relation is complete;
- that compiler proof planning is semantically sound;
- that backend patterns correctly implement their contracts;
- that target software behaves as declared;
- that a concrete transaction satisfies the linked programs;
- that a bound fits complete worst-case transactions;
- that deployment keys or issued assets were created correctly;
- that a deployment profile has complete evidence.

Those remain separate evidence boundaries under D004.

---

## 4. Package and dependency direction

### 4.1 Initial concrete linker path

The initial linker is allowed to be concrete for the tapscript backend.

A likely dependency direction is:

```text
linker
    → tapscript
    → compiler
    → realization

linker
    → target-elements
```

The linker may depend directly on architecture IDs where required for
constructor and operation indexing.

This direction allows the linker to consume concrete types such as:

```text
RelocatableTapscriptBundle
RelocatableObjectConstructor
TapscriptProgram
TapscriptOperationLayout
TapscriptResourceFormula
```

without introducing a `tapscript ↔ linker` cycle.

### 4.2 Transaction dependency direction

The transaction package consumes linked-bundle output:

```text
transaction → linker
```

The linker must not depend directly on transaction if transaction already
depends on linker.

Calibration is therefore orchestrated above both packages.

### 4.3 Calibration orchestration

The conceptual loop is:

```text
candidate bound assignment
        ↓
link candidate
        ↓
construct ABI-valid worst-case transactions
        ↓
measure target execution/resources
        ↓
choose next candidate
        ↺
```

This must not become a Cargo dependency cycle.

A higher-level calibration runner—initially under release orchestration or a
future dedicated package—will call:

1. linker candidate construction;
2. transaction worst-case construction;
3. target/vector measurement;
4. deterministic bound search;
5. final linker construction.

The linker may provide pure helper functions for:

- candidate linking;
- validating bound assignments;
- applying final calibrated values;
- closing resource formulas.

It must not invoke the transaction package internally.

### 4.4 Future second backend

A future Simplicity backend may require a different concrete linker adapter.

Do not force the first linker package to define one fully generic target
artifact abstraction before a second backend exists.

Preferred initial approach:

- concrete tapscript linker modules;
- target-neutral semantic/provenance bindings retained;
- backend-neutral roles named carefully;
- common abstraction extracted only after demonstrated by a second backend.

---

## 5. Normative typed inputs

### 5.1 Relocatable backend bundle

Primary input:

```rust
&tapscript::RelocatableTapscriptBundle
```

The linker requires:

- supported relocatable schema;
- valid backend configuration identity;
- matching analyzed-program identity;
- matching target identity;
- typed constructors;
- typed operation programs;
- symbol registry;
- relocation registry;
- placement/layout results;
- preliminary witness requirements;
- resource formulas;
- relation carrier bindings;
- source provenance.

The linker must validate this input before resolution.

### 5.2 Exact target

The linker consumes:

```rust
&target_elements::ElementsTarget
```

It uses target facts for:

- script serialization;
- leaf version;
- target hash/tweak construction;
- taptree/control-path rules;
- program limits;
- encoding and byte order;
- constructor verification requirements;
- target identity binding.

The linker must reject a relocatable bundle compiled for a different target.

### 5.3 Deployment parameters

Typed deployment inputs may include:

```rust
pub struct LinkDeploymentParameters {
    pub network_id: [u8; 32],
    pub genesis_id: [u8; 32],

    pub keys: DeploymentKeys,
    pub assets: DeploymentAssetIds,
    pub bounds: BoundAssignment,

    pub metadata_schema: MetadataSchemaBinding,
    pub domain_separators: DomainSeparatorSet,

    pub additional_constants: DeploymentConstantSet,
}
```

> Illustrative API; exact fields are not frozen.

Each value must have:

- typed identity;
- source/provenance;
- validation;
- canonical encoding;
- target compatibility.

### 5.4 Linker configuration

Conceptually:

```rust
pub struct LinkerConfiguration {
    pub schema_version: LinkerConfigurationSchema,
    pub reference_policy: ReferenceResolutionPolicy,
    pub taptree_policy: TaptreePolicy,
    pub constructor_policy: ConstructorLinkPolicy,
    pub resource_policy: LinkedResourcePolicy,
    pub canonical_tiebreak: LinkTiebreakPolicy,
}
```

> Illustrative API; not frozen.

Configuration must not redefine semantic operation behavior.

### 5.5 Taptree weights

If expected-frequency weighting is used, weights are explicit typed
configuration.

Weights must be:

- nonzero where required by the chosen algorithm;
- deterministic;
- keyed by stable leaf/program role;
- complete for the relevant tree;
- bound into linker/backend configuration identity;
- absent from realization identity.

A default weighting policy may exist, but it must be typed and documented.

### 5.6 Calibrated bounds

A final linked release bundle consumes one unambiguous calibrated value for
each required architecture bound.

The linker must reject:

- missing bound;
- duplicate bound;
- zero bound;
- value below manifest minimum;
- value inconsistent with backend layout;
- value outside target representable domain.

Candidate bundles may use candidate assignments during calibration, but they
must be clearly typed as candidates rather than final release bundles.

---

## 6. Forbidden inputs and behavior

The linker must not consume:

- architecture JSON/TOML;
- declassification JSON;
- realization Markdown;
- plans;
- model source;
- target Markdown;
- backend disassembly as the authoritative program;
- live node state in pure linking APIs;
- environment variables;
- current time;
- filesystem iteration order;
- deployment profile files as untyped source.

The linker must not:

- invent a missing protocol relation;
- remove a relation carrier;
- choose an undeclared recipient;
- change operation cardinality;
- weaken authorization;
- choose an unsupported proof method;
- silently use draft bound defaults;
- generate keys;
- run the issuance/genesis ceremony;
- measure full transactions;
- mark candidate calibration final;
- resolve a cycle by arbitrary iteration until bytes happen to stabilize;
- accept unresolved mandatory symbols;
- accept unknown relocation kinds;
- patch untyped raw offsets without checking expected value type;
- rewrite backend programs outside accepted typed transformations;
- write final release files from pure library calls.

---

## 7. Typed outputs

The linker should distinguish candidate, partial, and final bundles.

### 7.1 Candidate linked bundle

A candidate bundle is linked under one explicit candidate bound assignment and
deployment parameter set.

Conceptually:

```rust
pub struct CandidateLinkedBundle {
    pub schema_version: LinkedBundleSchemaVersion,

    pub architecture: ArchitectureBinding,
    pub realization: RealizationBinding,
    pub analyzed_program: AnalyzedProgramBinding,
    pub target: TargetBinding,
    pub backend_configuration: BackendConfigurationBinding,
    pub linker_configuration: LinkerConfigurationBinding,

    pub deployment: LinkDeploymentBinding,
    pub candidate_bounds: BoundAssignment,

    pub constructors: Vec<LinkedObjectConstructor>,
    pub operation_programs: Vec<LinkedOperationProgram>,
    pub taptrees: Vec<LinkedTaptree>,
    pub control_paths: Vec<LinkedControlPath>,

    pub placements: LinkedPlacementPlan,
    pub layouts: Vec<LinkedOperationLayout>,
    pub witness_requirements: Vec<LinkedWitnessRequirement>,
    pub resource_formulas: Vec<LinkedResourceFormula>,

    pub relation_carriers: Vec<LinkedRelationCarrier>,
    pub reference_report: ReferenceResolutionReport,
    pub relocation_report: RelocationReport,
}
```

> Illustrative API; exact fields and names are not frozen.

A candidate bundle is suitable for:

- transaction construction;
- target execution;
- resource measurement;
- vector materialization.

It is not a final release bundle.

### 7.2 Final linked bundle

A final bundle adds:

- validated calibrated bounds;
- measurement-report bindings;
- final resource-limit validation;
- no candidate/unresolved status;
- final linked-bundle identity;
- canonical bundle manifest.

Conceptually:

```rust
pub struct LinkedBundle {
    pub candidate: CandidateLinkedBundle,
    pub calibration: CalibrationBinding,
    pub identity: LinkedBundleIdentity,
}
```

> Illustrative API; not frozen.

The final bundle may avoid nesting candidate data physically. The distinction
must remain semantically explicit.

### 7.3 Bundle manifest

A typed bundle manifest should summarize:

- all upstream identities;
- deployment constants;
- target;
- constructor/program IDs;
- taptrees/control paths;
- operation coverage;
- relation carrier census;
- calibrated bounds;
- ABI input references;
- resource-report references;
- linked-bundle hash.

The manifest is derivative of the typed linked bundle.

### 7.4 Link reports

The linker should return typed reports for:

- reference graph and SCCs;
- selected resolution strategies;
- symbol definitions and references;
- relocation applications;
- taptree assembly;
- linked resource formulas;
- relation carrier preservation;
- unresolved optional references;
- deterministic ordering.

Reports may later be canonically published and bound by release.

---

## 8. Public API boundary

### 8.1 Candidate-link API

Conceptually:

```rust
pub fn link_candidate(
    relocatable: &tapscript::RelocatableTapscriptBundle,
    target: &target_elements::ElementsTarget,
    deployment: &LinkDeploymentParameters,
    bounds: &BoundAssignment,
    configuration: &LinkerConfiguration,
) -> Result<CandidateLinkedBundle, LinkError>;
```

> Illustrative API; not frozen.

The function must be:

- pure;
- deterministic;
- non-writing;
- fail-closed;
- complete for the input scope;
- independent of transaction construction.

### 8.2 Finalization API

Conceptually:

```rust
pub fn finalize(
    candidate: CandidateLinkedBundle,
    calibration: &CalibrationEvidenceBinding,
    limits: &ValidatedTargetLimits,
) -> Result<LinkedBundle, LinkError>;
```

> Illustrative API; not frozen.

Finalization must verify:

- candidate identity;
- measured bundle/ABI identity;
- complete bound census;
- target-limit compliance;
- no changed program bytes after measurement;
- no unresolved mandatory reference;
- calibration report applies to the exact candidate.

If finalization changes program bytes, prior measurements are stale and must
be regenerated.

### 8.3 Inspection API

Downstream transaction/vector/release code needs immutable typed access to:

- linked constructors;
- operation programs;
- target program bytes/commitments;
- taptrees/control paths;
- layouts;
- witness requirements;
- resource formulas;
- relation carriers;
- identities;
- calibration bindings.

Do not expose untracked mutable access to linked bytes.

### 8.4 Serialization API

The linker may provide deterministic encoders for:

- linked target programs;
- control data;
- bundle manifest;
- canonical human disassembly.

Raw byte serialization must be derived from typed linked structures.

### 8.5 No process-global link context

No global symbol table, target, or deployment configuration.

Every link-affecting value is an explicit input.

---

## 9. Symbol model

### 9.1 Symbols represent typed roles

A symbol is not merely a string name.

Conceptual symbol categories include:

```rust
pub enum SymbolKind {
    ObjectConstructor(architecture::ObjectId),
    OperationProgram {
        object: architecture::ObjectId,
        operation: architecture::OperationId,
        role: ProgramRoleId,
    },
    StaticCodeSubtree(architecture::ObjectId),
    InternalKey(ConstructorKeyRole),
    MetadataSchema(architecture::ObjectId),
    AssetId(architecture::AssetId),
    BoundValue(architecture::BoundId),
    DeploymentKey(DeploymentKeyRole),
    NetworkConstant(NetworkConstantRole),
    DomainSeparator(DomainRole),
}
```

> Illustrative vocabulary; not frozen.

### 9.2 Symbol identity

Symbol identity should derive from:

- symbol kind;
- architecture/operation/object identity;
- backend/target role;
- schema;
- semantic parameters that determine the symbol family.

It must not derive only from a display string.

### 9.3 Constructor symbols

An object constructor is usually parameterized.

Examples:

```text
STATE(state)
RECEIPT_L(owner)
RECEIPT_T(owner)
DEPOSIT_ENTITLEMENT(owner, target_cycle)
DISTRIBUTION_CONTROL(cycle, counters)
DISTRIBUTION_VAULT(cycle)
ASH()
```

The symbol registry should distinguish:

- constructor family;
- static linked components;
- dynamic semantic parameters;
- output encoding rule;
- target commitment rule;
- predecessor/successor verification support.

### 9.4 Definition and reference census

The linker must require:

- every mandatory reference has exactly one compatible definition;
- duplicate incompatible definitions reject;
- optional references are explicitly typed;
- unused definitions are reported;
- dead definitions are not removed unless the removal policy is explicit and
  provenance-safe.

### 9.5 No stringly resolution

Avoid:

```rust
symbols.get("SPK_STATE")
```

Prefer typed IDs and compile-time-checked role enums.

Strings remain for diagnostics and publication.

---

## 10. Reference graph

### 10.1 Graph nodes

Reference-graph nodes represent linked constructor/program components whose
identity depends on other components.

Possible nodes:

- static object code subtree;
- metadata constructor recipe;
- operation leaf/program;
- target program commitment;
- internal key/tweak constructor;
- sibling constructor constant;
- deployment asset/key constant.

### 10.2 Graph edges

An edge records:

- source node;
- target symbol/node;
- reference kind;
- whether target value must be statically known;
- whether target may be authenticated at execution;
- whether dynamic metadata participates;
- source relation/constructor provenance;
- candidate resolution strategies.

### 10.3 Edge classification

A reference may be classified as:

1. **static link-time constant**
   The target value can be computed after its dependencies resolve.

2. **identity introspection**
   The target program verifies byte-identical predecessor/successor identity at
   execution.

3. **in-script constructor reconstruction**
   The target program reconstructs a metadata-dependent target constructor from
   authenticated semantic fields.

4. **authenticated witnessed-root continuity**
   A static code/program root is supplied as public witness data,
   authenticated against the predecessor, and reused in successor
   construction.

5. **deployment relocation**
   A typed deployment constant is supplied externally and patched at link
   time.

6. **unsupported cycle/reference**
   No accepted strategy can establish the dependency; linking fails.

The backend should propose allowed strategies for each reference. The linker
validates graph consistency and selects only under typed configuration.

### 10.4 SCC analysis

Run deterministic strongly connected component analysis, such as Tarjan's
algorithm, over the static reference graph.

Use SCCs to identify:

- acyclic components;
- self-reference;
- mutual constructor/program cycles;
- resolution order after condensation.

SCC membership alone does **not** determine the final strategy.

A metadata-dependent successor inside an SCC may require constructor
reconstruction rather than identity introspection.

The decision depends on:

- reference semantics;
- target capabilities;
- constructor category;
- backend-emitted proof strategy;
- dynamic parameter role.

### 10.5 Condensation order

Condense SCCs into a DAG and process components in deterministic topological
order.

Tie-break by stable node identity.

### 10.6 Cycle validation

For every cyclic SCC, require:

- at least one accepted runtime/authenticated resolution strategy;
- no edge requiring an impossible static fixed point;
- consistent reuse of authenticated roots where required;
- complete relation provenance;
- target capability support;
- vector requirements for cycle/continuity faults.

Do not attempt arbitrary repeated hashing until a stable value appears.

### 10.7 Reference-resolution report

Publish a deterministic typed report:

```text
node
SCC
edge
reference kind
selected strategy
target capability
backend carrier
resolved value or runtime obligation
source provenance
```

This report makes the weld strategy reviewable.

---

## 11. Constructor linking

### 11.1 Static and dynamic parts

A constructor may contain:

- fixed internal key;
- static code subtree;
- operation leaves;
- dynamic metadata leaf or commitment;
- target schema/domain separator;
- deployment constants;
- dynamic semantic parameters.

The linker resolves static components and produces one final constructor recipe
for later transaction construction.

### 11.2 Static code subtree

The linker assembles operation programs associated with one object constructor
into a deterministic static subtree or target program collection.

The subtree identity must bind:

- operation leaf/program bytes;
- leaf versions;
- canonical ordering/tree policy;
- backend configuration;
- target identity.

### 11.3 Dynamic metadata

Dynamic metadata remains a typed constructor parameter.

The linker must not enumerate every possible owner/state/counter instance.

Instead it resolves the recipe needed to instantiate:

```text
static linked constructor
+
typed metadata
→ concrete target output program/commitment
```

### 11.4 Metadata schema

The linked constructor binds one exact metadata schema:

- object kind;
- schema version;
- field order;
- field domains;
- canonical encoding;
- domain separator;
- target commitment rule.

The transaction package later instantiates it.

### 11.5 Constructor continuity

For constructors requiring predecessor/successor continuity, the linked bundle
must preserve:

- authenticated static code identity;
- metadata reconstruction rule;
- internal key;
- target commitment rule;
- relation carrier;
- witness requirement;
- selected runtime strategy.

A constructor that authenticates only metadata but not code continuity is
invalid.

### 11.6 Metadata-leaf unspendability

If the accepted constructor design uses an unspendable metadata leaf, the
linker must include the leaf and its unspendability rule in:

- constructor identity;
- taptree assembly;
- control-path report;
- vector requirements.

No alternative spendable metadata path may remain.

---

## 12. Relocations

### 12.1 Typed relocation

A relocation must identify:

- relocation ID;
- relocation kind;
- target program/constructor;
- semantic location;
- expected typed value;
- encoding rule;
- width/domain;
- symbol/deployment source;
- multiplicity;
- source provenance.

Conceptually:

```rust
pub struct Relocation {
    pub id: RelocationId,
    pub target: RelocationTarget,
    pub location: ProgramLocation,
    pub value_type: RelocationValueType,
    pub source: RelocationSource,
    pub encoding: RelocationEncoding,
    pub required: bool,
}
```

> Illustrative API; not frozen.

### 12.2 Relocation sources

Sources include:

- architecture asset ID;
- deployment-issued asset ID;
- operator/deployment public key;
- calibrated bound;
- constructor root;
- sibling program commitment;
- network/genesis constant;
- metadata schema;
- domain separator;
- leaf/program hash.

### 12.3 Relocation validation

Before patching, verify:

- source exists;
- source type matches;
- target location expects the type;
- encoding and width match;
- value is in domain;
- relocation multiplicity valid;
- relocation has not already been applied;
- program target identity matches.

### 12.4 Structured patching

Prefer applying relocations to typed script/constructor structures before final
serialization.

If raw byte patching is necessary:

- byte location derives from typed serialization metadata;
- expected placeholder bytes/type are verified;
- patch cannot change program structure unexpectedly;
- final decode/revalidation is performed where possible.

### 12.5 Relocation closure

A final bundle requires:

```text
all mandatory relocations resolved
all optional unresolved relocations explicitly permitted
no duplicate application
no unknown relocation
no untracked program mutation
```

### 12.6 Relocation report

Record:

- relocation ID;
- source symbol/value identity;
- target program/location;
- before/after program identity;
- encoding;
- status.

Do not include secret deployment values if any relocation source is secret.
Prefer public keys and public deployment constants only. Private keys must
never enter the linker.

---

## 13. Taptree assembly

### 13.1 Linker ownership

For the tapscript backend, final taptree assembly belongs to the linker because
it depends on:

- linked leaf bytes;
- leaf versions;
- deployment constants;
- constructor policy;
- deterministic weighting;
- dynamic metadata constructor structure;
- control-path output required by the ABI.

### 13.2 Leaf set

Every leaf must have:

- typed leaf/program ID;
- object constructor;
- operation ID;
- program role;
- leaf version;
- linked script bytes;
- relation carriers;
- expected frequency weight or deterministic default;
- source provenance.

### 13.3 Canonical tree policy

The initial tree policy should be explicit and deterministic.

Possible policy:

- Huffman-style weighting by typed expected-frequency weights;
- stable tie-break by leaf/program ID;
- length-limited construction if target depth limits require it;
- deterministic canonical left/right ordering.

Do not claim ordinary Huffman is sufficient if target depth constraints can be
violated. Use a length-limited algorithm when required.

### 13.4 Weight policy

Weights are compiler/linker configuration, not protocol semantics.

Changing them may alter:

- taptree root;
- control paths;
- bundle identity;
- ABI identity;
- transaction weight.

It should not alter realization identity.

### 13.5 Dynamic metadata leaf

If constructors combine a static code subtree with a dynamic metadata leaf,
the linker produces:

- static code-subtree root;
- metadata-leaf construction rule;
- branch ordering rule;
- constructor commitment recipe;
- control-path derivation rule.

The transaction package supplies concrete metadata and derives the concrete
constructor output.

### 13.6 Internal key

The constructor's internal key may be:

- fixed by backend configuration;
- supplied by deployment parameters;
- derived under an accepted policy.

It must be public, typed, validated, and identity-bound.

The linker does not generate private keys.

### 13.7 Control paths

The linker should provide enough typed information for the transaction package
to construct:

- selected leaf/program;
- control path;
- parity/control bits;
- target witness data.

Concrete control data may depend on dynamic metadata and therefore be produced
by transaction construction from a linked recipe rather than stored for every
possible instance.

### 13.8 Taptree report

Record:

- constructor;
- leaf set;
- weights;
- tree algorithm;
- depth per leaf;
- static root;
- dynamic metadata rule;
- internal key;
- target commitment rule;
- relation provenance.

---

## 14. Program and bundle closure

### 14.1 Program validation after linking

After relocation/taptree assembly, revalidate every linked program:

- target script encoding;
- target execution domain;
- stack contract metadata remains applicable;
- no unresolved symbol;
- no altered instruction boundary;
- relation carrier set unchanged;
- selected proof plan unchanged;
- target capability identity unchanged;
- resource formula still matches structure.

### 14.2 Relation carrier census

Compare:

```text
compiler required relations
backend emitted relation carriers
linked relation carriers
```

Require exact selected-scope coverage.

The linker must reject:

- carrier lost during assembly;
- carrier attached to missing program;
- duplicate incompatible carrier;
- relation linked only into unreachable leaf;
- program removed while carrying unique relation.

### 14.3 Dead-code elimination

Initial policy:

> Do not perform semantic dead-code elimination beyond removing artifacts
> explicitly marked optional and unreachable under the selected bundle scope.

Any future dead-code elimination must preserve:

- relation carrier coverage;
- constructor lifecycle;
- client ABI;
- evidence vectors;
- source provenance.

### 14.4 Operation coverage

A linked bundle has an explicit operation scope.

A pilot bundle may contain only `compact-ash`. It must be labeled as a pilot or
partial bundle and cannot become a full deployment release bundle.

Full release validation requires complete approved operation coverage.

### 14.5 Constructor coverage

Every output family in scope must reference one linked constructor or approved
open-output rule.

No target output family may remain “to be supplied by wallet convention.”

---

## 15. Resource-formula closure

### 15.1 Inputs

The linker consumes backend formulas over:

- program size;
- witness item sizes;
- family counts;
- tree depths;
- control data;
- crypto operations;
- target capabilities.

### 15.2 Link-time resolution

Linking resolves variables such as:

- final script bytes;
- pushed constant sizes;
- leaf depth;
- control-path size;
- calibrated bounds;
- concrete constructor-root sizes;
- program count.

Other variables remain for transaction construction, such as:

- actual family count up to bound;
- signature size if target permits variation;
- confidential proof size under selected representation.

### 15.3 Linked formula

The linker emits formulas bound to:

- linked program identity;
- constructor identity;
- operation layout;
- target identity;
- calibrated bounds.

### 15.4 Formula-to-measurement check

The transaction/vector/calibration runner later compares predicted and observed
costs.

Finalization rejects a formula/measurement mismatch under the accepted
tolerance policy.

The initial policy should prefer exact formulas where target serialization is
deterministic.

### 15.5 No isolated-leaf release claim

A linked leaf fitting target limits does not prove a complete transaction fits.

The linker reports only linked-program formulas. Final calibration reports
whole transactions.

---

## 16. Calibration interface

### 16.1 Candidate bounds

A `BoundAssignment` should contain exactly one typed value per required bound.

Conceptually:

```rust
pub struct BoundAssignment {
    pub values: BTreeMap<architecture::BoundId, u64>,
}
```

> Illustrative API; not frozen.

Validation checks:

- complete required census;
- no duplicate entry;
- nonzero;
- manifest minimum;
- target representability;
- layout consistency.

### 16.2 Candidate identity

Each candidate bundle has an identity including candidate bounds.

Measurements apply only to that candidate identity.

### 16.3 Search orchestration

A higher package may perform deterministic monotone search:

1. choose candidate assignment;
2. link candidate;
3. build worst-case transactions;
4. measure all affected operations;
5. accept/reject candidate;
6. choose next assignment.

The search must account for shared bounds.

For example:

```text
FEE_SPONSOR_INPUT_MAX
```

affects many operation families.

A bound is valid only if every affected worst-case transaction fits.

### 16.4 Monotonicity

Binary search is permitted only when the measured feasibility predicate is
known or verified monotone over the searched bound.

If program structure changes at thresholds or target policy is nonmonotone,
the runner must use another deterministic search and record the policy.

### 16.5 Final remeasurement

After final values are selected:

- relink the exact final bundle;
- regenerate the exact final ABI;
- regenerate all worst-case transactions;
- remeasure every affected family;
- bind reports to final identities;
- reject any mismatch.

### 16.6 Calibration evidence handoff

Finalization consumes a typed binding to:

- final candidate bundle;
- ABI identity;
- target identity;
- measurement tool version;
- per-operation worst-case report;
- per-bound evidence hash;
- script-bundle hash.

The linker does not trust a report for another candidate.

---

## 17. Deployment constants and ceremony boundary

### 17.1 Public constants only

The linker consumes public deployment values such as:

- public keys;
- asset IDs;
- network/genesis IDs;
- calibrated bounds;
- domain separators;
- metadata schema IDs.

Private keys remain outside the linker.

### 17.2 Issuance-derived asset IDs

Some asset IDs may depend on issuance transaction outpoints.

This creates a deployment ceremony sequence:

```text
funding/issuance outpoints fixed
        ↓
asset IDs derived
        ↓
link deployment constants
        ↓
construct final genesis/deployment transactions
```

The linker accepts the resulting typed asset IDs.

It does not own the ceremony or claim they were generated correctly.

### 17.3 Ceremony dependency graph

Future genesis/release tooling should model ceremony dependencies as a typed
DAG.

If one proposed constructor requires a genesis transaction ID that itself
depends on the constructor, the cycle must be detected and resolved explicitly.

The linker must not hide ceremony cycles through manual patching.

### 17.4 Deployment evidence

Release evidence must bind:

- ceremony outputs;
- asset IDs;
- linked bundle;
- genesis/network identity;
- issuance evidence;
- final deployment profile.

---

## 18. Linked bundle identity

### 18.1 Identity domain

The linked-bundle identity is domain-separated from:

- architecture hashes;
- realization identity;
- compiler configuration;
- target identity;
- relocatable bundle identity;
- transaction ABI identity;
- deployment profile hash.

### 18.2 Included fields

The linked-bundle identity should bind at least:

- linked-bundle schema;
- architecture identity;
- realization identity;
- analyzed-program identity;
- selected proof-plan identity;
- target definition and deployment-instance identities;
- backend configuration identity;
- linker configuration identity;
- public deployment constants;
- calibrated bounds;
- linked constructors;
- linked operation programs;
- static subtree/program commitments;
- taptree assembly;
- control-path recipes;
- relation carrier census;
- layout handoff;
- witness-requirement handoff;
- resource formulas;
- reference-resolution report identity;
- relocation report identity.

### 18.3 Excluded fields

Exclude:

- private keys;
- production signatures;
- blinding factors;
- temporary paths;
- wall-clock timestamps;
- host information;
- human comments;
- execution duration.

### 18.4 Hash algorithm

Before publication, define:

- algorithm identifier;
- canonical typed projection;
- domain separator;
- ordering;
- encoding;
- migration policy;
- mutation tests.

Do not reuse the architecture semantic-hash algorithm identifier for bundle
bytes.

### 18.5 Bundle manifest bytes

The emitted script-bundle artifact hash in the deployment profile may hash one
canonical serialized bundle or archive.

The bundle identity and archive-byte hash may be the same only if the
serialization contract is explicit. Otherwise keep them separate and bind one
to the other.

---

## 19. Error model

Errors should be typed, deterministic, and provenance-aware.

Candidate classes include:

```rust
pub enum LinkError {
    UnsupportedRelocatableSchema,
    UnsupportedLinkedBundleSchema,
    TargetIdentityMismatch,
    ArchitectureIdentityMismatch,
    RealizationIdentityMismatch,
    AnalyzedProgramIdentityMismatch,
    BackendConfigurationMismatch,

    InvalidDeploymentParameters,
    ZeroNetworkId,
    ZeroGenesisId,
    MissingDeploymentKey(DeploymentKeyRole),
    InvalidDeploymentKey(DeploymentKeyRole),
    MissingAssetId(architecture::AssetId),
    InvalidAssetId(architecture::AssetId),

    MissingBound(architecture::BoundId),
    DuplicateBound(architecture::BoundId),
    ZeroBound(architecture::BoundId),
    BoundBelowMinimum(architecture::BoundId),
    BoundNotRepresentable(architecture::BoundId),

    DuplicateSymbol(SymbolId),
    MissingSymbol(SymbolId),
    AmbiguousSymbol(SymbolId),
    IncompatibleSymbolType(SymbolId),

    InvalidReferenceGraph,
    UnsupportedReferenceCycle(SccId),
    MissingReferenceStrategy(ReferenceEdgeId),
    InvalidReferenceStrategy(ReferenceEdgeId),
    ConstructorContinuityUnresolved(architecture::ObjectId),

    DuplicateRelocation(RelocationId),
    UnknownRelocation(RelocationId),
    RelocationTypeMismatch(RelocationId),
    RelocationEncodingMismatch(RelocationId),
    RelocationOutOfDomain(RelocationId),
    UnresolvedRelocation(RelocationId),

    MissingProgram(ProgramId),
    InvalidLinkedProgram(ProgramId),
    MissingRelationCarrier(realization::RelationId),
    UnreachableRelationCarrier(realization::RelationId),
    CarrierCensusMismatch,

    InvalidTaptree(ConstructorId),
    MissingLeafWeight(ProgramId),
    TreeDepthExceeded(ProgramId),
    NonDeterministicTree,
    MetadataLeafSpendable(ConstructorId),

    MissingResourceFormula(ProgramId),
    ResourceFormulaMismatch(ProgramId),

    CalibrationIdentityMismatch,
    CalibrationIncomplete,
    CalibrationStale,
    TargetLimitExceeded,

    NonDeterministicLink,
}
```

> Illustrative vocabulary; not frozen.

Errors must not expose private ceremony data or credentials.

---

## 20. Validation

### 20.1 Input identity validation

Require exact agreement among:

- relocatable bundle target;
- supplied target;
- architecture;
- realization;
- analyzed program;
- backend configuration;
- operation scope.

### 20.2 Symbol validation

Require:

- unique typed definition;
- compatible reference types;
- complete mandatory reference census;
- no string-only resolution;
- no unknown symbol kind.

### 20.3 Reference graph validation

Require:

- all graph nodes known;
- all edges typed;
- SCCs deterministic;
- every cycle has accepted strategy;
- selected strategy target-supported;
- constructor dynamic/static classification consistent;
- no impossible static fixed point.

### 20.4 Relocation validation

As described above.

### 20.5 Constructor validation

Require:

- all operation leaves/programs expected by constructor present;
- static tree/program assembly deterministic;
- metadata schema complete;
- internal key valid;
- metadata unspendability where required;
- predecessor/successor strategy complete;
- target commitment valid.

### 20.6 Taptree validation

Require:

- unique leaves;
- valid leaf versions;
- canonical weighting/tie-break;
- target depth/control constraints;
- reproducible root;
- every operation program reachable;
- no undeclared leaf.

### 20.7 Provenance validation

Require every final program/constructor to retain:

- backend program ID;
- architecture object/operation;
- realization relations;
- compiler proof plan;
- placement;
- target capability requirements.

### 20.8 Resource validation

Require linked formulas complete and within static hard limits where they can
be determined at link time.

### 20.9 Candidate/final distinction

A candidate must carry explicit nonfinal status.

A final bundle must have:

- final calibration binding;
- no unresolved mandatory values;
- exact final identity;
- complete approved operation scope.

---

## 21. Determinism

### 21.1 Ordered registries

Canonical order:

- symbols by typed SymbolId;
- graph nodes/edges by typed ID;
- SCC members by node ID;
- SCC condensation order by topological order with ID tie-break;
- relocations by target program and typed location;
- constructors by architecture object code;
- operation programs by operation code and program role;
- leaves by tree policy and stable ID;
- reports by typed ID.

### 21.2 Deterministic graph algorithms

Tarjan/SCC output can depend on traversal order.

Feed nodes and edges in canonical order and canonicalize component/member
ordering before publication.

### 21.3 Deterministic taptrees

Tie-breaking must be explicit.

If two leaves have equal weights, use stable program ID.

If two subtrees have equal weights, order by canonical subtree identity.

### 21.4 No random balancing

Do not use random tree construction, randomized hash-map order, or heuristic
iteration without a fixed deterministic seed included as configuration.

Prefer no randomness.

### 21.5 Reproducibility tests

Require:

- repeated link equality;
- input map permutation invariance where semantically set-like;
- same source/deployment inputs produce equal bytes;
- changed weight changes tree/bundle identity predictably;
- changed bound changes candidate identity;
- changed unused presentation text does not alter bundle;
- different cwd/temp path does not alter output.

---

## 22. Generated artifacts

### 22.1 Candidate artifacts

Potential publications include:

```text
linked-bundle.json
reference-resolution.json
relocations.json
taptree-layout.json
linked-resource-formulas.json
linked-programs/
```

Names are illustrative.

### 22.2 Final script bundle

The final bundle may be a deterministic archive or directory tree containing:

- constructor manifests;
- program bytes;
- program disassembly;
- taptree/control recipes;
- bundle manifest;
- relation-carrier map;
- ABI handoff;
- target identity reference.

The exact archive format requires a separate schema decision before release.

### 22.3 Artifact law

Every committed artifact has:

- typed source;
- deterministic serialization;
- canonical path order;
- no filesystem metadata dependence;
- generator;
- non-writing checker;
- identity verification;
- no reverse semantic dependency.

### 22.4 Atomic writing

Writing final bundle artifacts belongs to an explicit generation/release
command.

Use staging and atomic publication where practical.

Library linking remains in-memory/non-writing.

---

## 23. Testing strategy

### 23.1 Unit tests

Cover:

- typed symbols;
- duplicate/missing definitions;
- graph construction;
- SCC analysis;
- condensation ordering;
- reference classification;
- relocation type checking;
- relocation application;
- constructor assembly;
- taptree policy;
- control paths;
- resource formula closure;
- relation carrier census;
- candidate/final distinction;
- identity stability.

### 23.2 Synthetic graph fixtures

Create small deterministic fixtures for:

- acyclic chain;
- self-cycle with identity introspection;
- mutual cycle with accepted runtime strategy;
- impossible static hash cycle;
- metadata-dependent constructor cycle;
- mixed static/runtime edges;
- missing strategy;
- duplicate symbol.

### 23.3 Continuity fault tests

After constructor prototype acceptance, test:

- correct predecessor and successor;
- successor metadata under wrong static code root;
- distinct predecessor/successor witnessed roots;
- wrong internal key;
- wrong parity;
- wrong metadata schema;
- omitted metadata leaf;
- spendable metadata leaf;
- sibling constructor substitution.

### 23.4 Relocation fault tests

- missing value;
- wrong type;
- wrong width;
- wrong byte order;
- duplicate application;
- wrong target program;
- stale location;
- value out of domain;
- unresolved mandatory relocation;
- changed source after report generation.

### 23.5 Taptree tests

- deterministic root;
- equal-weight tie-break;
- expected leaf depth;
- depth limit;
- length-limited algorithm where required;
- wrong leaf version;
- missing leaf;
- duplicate leaf;
- changed weight changes root;
- dynamic metadata recipe reproduces expected concrete constructor.

### 23.6 Relation carrier tests

- exact relation census preserved;
- removing a carrier fails;
- removing a uniquely carrying leaf fails;
- duplicated carrier remains reported;
- unreachable leaf fails;
- out-of-scope operation not treated as covered.

### 23.7 Resource formula tests

Compare linked formulas to:

- serialized linked programs;
- control path sizes;
- deterministic witness schema bounds;
- measured pilot transactions downstream.

### 23.8 Public API tests

An external integration test should prove transaction/vector/release code can:

- inspect linked constructors/programs;
- instantiate public constructor recipes;
- inspect operation layouts;
- inspect witness requirements;
- inspect relation carriers;
- inspect resource formulas;
- verify bundle identity;
- do so without mutable linker internals.

### 23.9 Determinism tests

As described above.

---

## 24. Milestones

### L1 — Package and typed symbol model

Deliver:

- workspace package;
- configuration/errors;
- symbol/reference/relocation IDs;
- no transaction dependency.

### L2 — Reference graph and SCC analysis

Deliver:

- deterministic graph;
- Tarjan SCC;
- condensation DAG;
- typed edge classification;
- resolution report.

### L3 — Typed relocation engine

Deliver:

- validation;
- typed substitution;
- structured patching;
- relocation report.

### L4 — Constructor linker

Deliver:

- static program sets;
- metadata schema binding;
- constructor recipes;
- continuity strategy integration.

### L5 — Taptree assembly

Deliver:

- deterministic weighting;
- stable tree algorithm;
- roots/control recipes;
- taptree report.

### L6 — Program/provenance closure

Deliver:

- linked program validation;
- relation carrier census;
- resource formula closure;
- operation scope.

### L7 — Candidate `compact-ash` bundle

Deliver:

- linked ASH constructor;
- linked operation programs;
- target/deployment constants;
- candidate bound;
- candidate identity;
- transaction-package handoff.

### L8 — Calibration integration

Deliver pure candidate/final APIs and calibration identity validation.

### L9 — Final `compact-ash` bundle

Deliver after whole-transaction measurement.

### L10 — Additional operations

Extend in roadmap order.

---

## 25. Phase-4 candidate linker exit criteria

The linker is ready to hand `compact-ash` to the transaction package when:

- [ ] `packages/linker` is a workspace member;
- [ ] the package consumes typed tapscript relocatable output;
- [ ] all input identities match;
- [ ] symbol and relocation schemas are typed;
- [ ] reference graph is deterministic;
- [ ] SCC analysis is deterministic;
- [ ] every graph edge has a validated resolution strategy;
- [ ] no impossible static fixed point is accepted;
- [ ] every mandatory relocation is resolved;
- [ ] linked ASH constructor is complete;
- [ ] operation leaf/program is present;
- [ ] taptree/static program assembly is deterministic;
- [ ] metadata schema and internal key are bound;
- [ ] relation carrier census matches compiler scope;
- [ ] linked layout/witness handoff is complete;
- [ ] resource formulas are closed as far as link-time values permit;
- [ ] candidate bound assignment is explicit;
- [ ] candidate bundle is marked nonfinal;
- [ ] candidate identity is deterministic;
- [ ] public API tests pass;
- [ ] debug/release tests pass;
- [ ] Clippy with `-D warnings` passes;
- [ ] the checkout remains clean.

---

## 26. Final linked-bundle exit criteria

A final linked bundle is release-eligible only when:

- [ ] exact target and deployment-instance identities are bound;
- [ ] all approved operations in deployment scope are linked;
- [ ] all constructors are complete;
- [ ] every mandatory symbol/reference/relocation is resolved;
- [ ] all cyclic references use accepted authenticated strategies;
- [ ] all relation carriers are reachable;
- [ ] linked programs revalidate;
- [ ] taptrees/program commitments are deterministic;
- [ ] final calibrated bound census is complete and unambiguous;
- [ ] final bundle/ABI/worst-case transaction identities agree;
- [ ] calibration evidence applies to exact final bytes;
- [ ] final resource limits pass;
- [ ] no post-measurement linking mutation occurred;
- [ ] bundle manifest is canonical;
- [ ] linked-bundle identity verifies;
- [ ] generation/check paths are separate;
- [ ] bundle reproduction is byte-identical;
- [ ] release package can consume the typed bundle without reparsing planning
      documents or disassembly.

The final deployment profile still requires separate evidence beyond the
linker gate.

---

## 27. Non-goals

The linker does not:

- define protocol semantics;
- derive realization relations;
- choose arbitrary semantic proof alternatives;
- emit operation scripts from relations;
- schedule stacks;
- construct transaction witnesses;
- generate private keys;
- run node RPC;
- create issuance transactions;
- derive independent indexer output;
- measure complete transactions by itself;
- choose protocol economics;
- silently recalibrate bounds;
- release a deployment;
- support every future backend from day one;
- solve arbitrary recursive program fixed points;
- accept manual unresolved patches.

---

## 28. Open questions

### 28.1 Common versus concrete linker interfaces

Should future backend-neutral linker roles be extracted before Simplicity
reactivation?

Initial preference:

> keep the first linker concrete for tapscript and preserve semantic role names;
> extract shared interfaces only when a second backend demonstrates them.

### 28.2 Static-root witness strategy

Blocked on state-object-constructor research.

Need to decide:

- which root is witnessed;
- how predecessor authenticates it;
- how successor reuses it;
- how linker binds it;
- how vectors mutate it.

### 28.3 Taptree weighting

Define the initial weight source and default policy.

Weights are implementation policy, not protocol semantics.

### 28.4 Length-limited tree algorithm

Confirm target maximum control-path depth and select a deterministic
length-limited algorithm if ordinary Huffman can violate it.

### 28.5 Bundle archive format

Define only when a final bundle needs publication.

### 28.6 Calibration runner ownership

Candidates:

- `release`;
- `vectors`;
- a future dedicated calibration package;
- a small release-tooling module.

Requirement:

- no crate cycle;
- complete transaction measurement;
- deterministic search;
- evidence binding.

### 28.7 Genesis ceremony package

The current planned package map does not include a dedicated genesis package.

Decide later whether ceremony construction belongs in:

- `transaction`;
- `release`;
- a future dedicated deployment package.

The linker itself remains pure and keyless.

### 28.8 Program-byte versus semantic bundle identity

Decide whether one canonical archive hash is the linked-bundle identity or
whether a typed semantic bundle identity separately binds archive bytes.

### 28.9 Partial bundle publication

Pilot bundles need explicit nonrelease status and scope.

Define the schema before first publication.

---

## 29. Risks

### 29.1 Constructor cycles are misunderstood

SCC analysis may find a cycle but not prove the chosen runtime strategy
authenticates the intended relation.

Mitigation:

- typed edge semantics;
- backend proof strategy;
- continuity vectors;
- no SCC-only automatic resolution;
- fail closed.

### 29.2 Metadata-dependent constructors complicate linking

Dynamic metadata prevents precomputing every output program.

Mitigation:

- constructor family/instance distinction;
- static code root plus dynamic recipe;
- transaction package instantiation;
- typed schema.

### 29.3 Relocation mutates program semantics

A wrong patch can silently alter target behavior.

Mitigation:

- typed relocations;
- structured patching;
- post-link validation;
- relation provenance;
- bundle vectors.

### 29.4 Taptree policy changes ABI costs

Changing weights changes control paths and transaction weight.

Mitigation:

- configuration identity;
- ABI regeneration;
- recalibration;
- release binding.

### 29.5 Candidate/final confusion

A candidate bundle might be published as deployable.

Mitigation:

- distinct types/status;
- release validator;
- incomplete-scope marker;
- no final identity without calibration.

### 29.6 Calibration feedback invalidates bytes

Changing bounds may change scripts/layouts and invalidate measurements.

Mitigation:

- candidate identity;
- relink per candidate;
- final remeasurement;
- exact report binding.

### 29.7 Resource formulas drift

Formulas may disagree with actual serialization.

Mitigation:

- linked program measurement;
- transaction measurement;
- exact formula comparison;
- fail release on mismatch.

### 29.8 Backend-specific linker becomes hard to generalize

The first linker may expose tapscript details broadly.

Mitigation:

- concrete naming;
- isolate target modules;
- preserve semantic provenance;
- second-backend boundary audit;
- avoid false generic types.

### 29.9 Ceremony dependencies leak into linker

Pressure to generate keys or issuance transactions may expand scope.

Mitigation:

- public deployment parameters only;
- pure APIs;
- ceremony handled above linker;
- no RPC/network access.

### 29.10 Bundle identity freezes immature format

Publishing too early may make migrations costly.

Mitigation:

- pilot/nonfinal schema;
- delay final hash algorithm;
- explicit version;
- mutation tests;
- migration record.

---

## 30. Definition of done

The linker package plan is fulfilled for the first production backend when the
repository can take one validated relocatable tapscript bundle plus exact target
and public deployment parameters; deterministically construct and validate its
typed symbol/reference graph; classify and resolve acyclic and cyclic
constructor dependencies under explicit authenticated strategies; apply every
typed relocation; assemble final constructors, operation programs, taptrees,
and control recipes; preserve every semantic relation carrier and resource
formula; produce candidate bundles for ABI-valid whole-transaction calibration;
and finalize one reproducible linked bundle bound to exact calibration
evidence—without redefining semantics, generating secrets, depending on
transaction construction, accepting manual unresolved patches, or treating
candidate output as a release.

---

## 31. One-line package contract

> `linker` consumes typed relocatable backend programs, an exact target, public
> deployment constants, and explicit bounds; deterministically resolves typed
> symbols, constructor graphs, SCCs, runtime/static reference strategies,
> relocations, taptrees, relation carriers, and resource formulas into
> candidate and final linked bundles whose identities are reproducible and
> whose exact bytes can be consumed by transaction construction, measurement,
> vectors, and release—without owning protocol semantics, private keys,
> transaction assembly, or deployment acceptance.
