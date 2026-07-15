# Elements Tapscript Backend Package Plan

> **Status:** PLANNED / PROTOTYPE-DEPENDENT
> **Planned source directory:** `packages/tapscript`
> **Planned Cargo package:** `tripod-tapscript`
> **Planned Rust library name:** `tapscript`
> **Implementation phases:** Phase 3 — foundational backend prototypes;
> Phase 4 onward — operation emission
> **Depends on packages:** `tripod-compiler`,
> `tripod-target-elements`
> **Depends on decisions:**
> [D001](../decisions/001-typed-rust-is-normative.md),
> [D002](../decisions/002-target-independent-realization-layer.md),
> [D003](../decisions/003-multiple-backends-tapscript-first.md),
> [D004](../decisions/004-translation-validation-over-compiler-trust.md),
> [D005](../decisions/005-value-parametric-asset-rigid.md),
> [D006](../decisions/006-canonical-transaction-layout-abi.md)
> **Open research dependencies:**
> [`../research/state-object-constructor.md`](../research/state-object-constructor.md),
> [`../research/wide-arithmetic.md`](../research/wide-arithmetic.md),
> [`../research/public-declassification.md`](../research/public-declassification.md),
> [`../research/settlement-layout.md`](../research/settlement-layout.md)
> **Authority:** Target-specific implementation beneath the typed realization,
> compiler analysis, and exact typed Elements target
> **Machine-consumed planning document:** no

---

## 1. Purpose

The `tapscript` package will implement the first production backend for the
attestation compiler.

It consumes:

- a validated target-independent analyzed program;
- a target-selected or target-compatible proof plan;
- an exact typed Elements target;
- typed backend configuration;
- target-specific deployment parameters required before linking.

It produces deterministic typed relocatable Elements tapscript programs and
associated target artifacts.

The package owns:

- typed tapscript construction;
- target opcode selection;
- target proof-pattern lowering;
- stack-effect validation;
- stack scheduling;
- verified peephole rewrites;
- explicit/confidential representation guards;
- transaction-introspection patterns;
- signature and timelock patterns;
- narrow arithmetic lowering;
- wide arithmetic lowering after prototype acceptance;
- hashing and structured-preimage lowering;
- object-constructor verification patterns;
- target-specific obligation placement;
- target-specific canonical layout lowering;
- preliminary witness requirements;
- relocations and constructor references;
- target resource formulas;
- pattern-level evidence;
- source relation and compiler-plan provenance.

It does not own:

- attestation-contract semantic relations;
- architecture registries;
- declassification policy;
- exact Elements target semantics;
- final symbol/constructor resolution;
- final taptree assembly;
- final transaction and witness ABI publication;
- concrete production transaction construction;
- deployment calibration;
- release acceptance.

The planned direction is:

```text
AnalyzedProgram
        +
target-compatible proof candidates
        +
ElementsTarget
        +
TapscriptConfiguration
        ↓
target proof selection and tapscript lowering
        ↓
typed relocatable tapscript programs
        ├── object-constructor templates
        ├── operation leaves/programs
        ├── symbol references
        ├── relocations
        ├── placement and layout result
        ├── preliminary witness schemas
        ├── resource formulas
        └── relation provenance
        ↓
linker
        ↓
linked bundle
        ↓
transaction ABI and concrete construction
```

The first complete emitted operation is:

```text
compact-ash
```

The second is:

```text
transfer-live-receipts
```

The backend grows operation by operation according to
[`../roadmap.md`](../roadmap.md).

---

## 2. Backend assurance boundary

The tapscript backend translates selected target-independent proof obligations
into Elements tapscript programs.

It establishes, through implementation and evidence:

- each selected proof plan has a concrete target pattern;
- emitted scripts have valid typed stack behavior;
- every relation has an executable carrier;
- target-specific guards enforce representation and encoding assumptions;
- concrete layouts satisfy compiler layout requirements;
- emitted programs retain relation provenance;
- relocations and constructor references are explicit;
- resource formulas correspond to emitted programs;
- emission is deterministic.

It does not establish by itself:

- that the realization relation is complete;
- that the executable model is correct;
- that the typed Elements target matches one running deployment;
- that final relocations are correct;
- that concrete transactions satisfy the emitted witness ABI;
- that calibrated bounds fit complete transactions;
- that an independent indexer recognizes events correctly;
- that a deployment profile is final.

Those claims belong to adjacent assurance boundaries.

A green backend unit suite is not a deployment release.

---

## 3. Dependency direction

### 3.1 Required dependencies

The backend is expected to depend on:

```text
compiler
target-elements
```

It may depend directly on:

```text
architecture
realization
```

only where stable IDs or source provenance cannot be accessed cleanly through
compiler-owned types. Such dependencies must not let the backend bypass
compiler analysis.

Likely generic workspace dependencies include:

```text
thiserror
sha2
serde, if typed backend publications are introduced
```

### 3.2 Prohibited dependencies

The package must not depend on:

```text
model
simplicity
transaction
vectors
release
artifacts
```

The exact dependency relationship with `linker` must remain acyclic.

Preferred initial direction:

```text
tapscript
    owns its relocatable tapscript output types

linker
    consumes those types through a target-specific linker adapter
```

Therefore:

```text
linker → tapscript
```

is preferable to:

```text
tapscript → linker → tapscript
```

If common relocatable artifact roles require a shared interface, introduce or
locate that interface deliberately in a lower package only after concrete type
pressure exists.

Do not create an abstract artifact crate prematurely.

### 3.3 No model dependency

The backend must not call model operation code to decide what to emit.

Expected behavior comes through:

```text
RealizationSpec
    ↓
AnalyzedProgram
```

The vector package later compares emitted execution with the model.

### 3.4 No source scraping

The backend must not parse:

- architecture publications;
- realization publications;
- planning documents;
- model source;
- Elements source at runtime;
- generated compiler reports.

It consumes typed Rust values under D001.

---

## 4. Normative typed inputs

### 4.1 Analyzed program

Primary semantic/compiler input:

```rust
&compiler::AnalyzedProgram
```

The backend requires:

- complete relation graph for the selected operation scope;
- source provenance;
- proof alternatives;
- disclosure analysis;
- fact-source requirements;
- constructibility requirements;
- lifecycle requirements;
- placement requirements;
- layout requirements;
- target capability requirements;
- coverage requirements.

The backend must reject an analyzed program whose schema or identity is
unsupported.

### 4.2 Target-selected plan

The backend may consume a compiler-produced target-compatible plan or perform
the last deterministic selection among compiler-approved alternatives.

Conceptually:

```rust
&compiler::TargetCompilationPlan
```

The selected plan must bind:

- analyzed-program identity;
- target identity;
- proof alternatives retained;
- selected representation modes;
- constructibility result;
- lifecycle result;
- unresolved backend requirements.

If final selection depends on backend resource formulas, the backend must:

1. choose only among compiler-approved alternatives;
2. apply deterministic policy;
3. record the selected proof;
4. preserve the original relation ID;
5. emit an identity-bound selection report.

It must not invent a semantically weaker proof.

### 4.3 Exact typed Elements target

The backend consumes:

```rust
&target_elements::ElementsTarget
```

The target must define:

- exact opcode registry;
- exact stack contracts;
- execution domain;
- encoding rules;
- sighash capabilities;
- timelock capabilities;
- confidential transaction capabilities;
- issuance capabilities;
- consensus and policy limits;
- evidence requirements;
- target and deployment-instance identities.

The backend must not hardcode a second copy of target opcode semantics.

### 4.4 Backend configuration

The backend consumes typed configuration controlling implementation choices
that do not alter protocol semantics.

Conceptually:

```rust
pub struct TapscriptConfiguration {
    pub schema_version: TapscriptConfigurationSchema,
    pub pattern_library: PatternLibraryVersion,
    pub stack_scheduler: StackSchedulerPolicy,
    pub peephole_rules: PeepholeRuleSetVersion,
    pub layout_policy: TapscriptLayoutPolicy,
    pub placement_policy: TapscriptPlacementPolicy,
    pub constructor_policy: ConstructorPolicy,
    pub taptree_requirements: TaptreeRequirementPolicy,
    pub resource_policy: ResourcePolicy,
    pub proof_selection_tiebreak: ProofSelectionTiebreak,
}
```

> Illustrative API; exact fields and names are not frozen.

Configuration is typed and identity-bound. It is not parsed from planning
Markdown.

### 4.5 Deployment parameters

Before linking, the backend may need typed placeholders or concrete values for:

- operator/public keys;
- network-specific constants;
- calibrated bounds;
- object-constructor domain separators;
- metadata schema versions;
- selected sighash profile.

Values unknown until linking remain typed relocations.

Do not silently substitute draft defaults for unresolved deployment
calibration.

---

## 5. Forbidden inputs and behaviors

The backend must not consume:

- architecture JSON/TOML;
- declassification JSON;
- realization Markdown;
- files under `plans/`;
- model source or model tests;
- target opcode Markdown;
- live node capability probes in pure emission APIs;
- environment variables;
- current time;
- filesystem order;
- release reports as semantic input.

The backend must not:

- redefine architecture operations;
- change formulas;
- weaken a semantic relation;
- add owner/operator authorization to a permissionless path;
- remove a lifecycle exit;
- choose undeclared recipients;
- disclose a value without compiler provenance;
- accept confidential closed-asset identity under the initial policy;
- omit an obligation because placement is difficult;
- emit target programs for an unsupported execution domain;
- silently fall back to a different target revision;
- resolve final symbols or taptrees inside emission unless that responsibility
  is explicitly moved by a later decision;
- write final publication files from library APIs.

---

## 6. Typed outputs

The primary output is a deterministic relocatable tapscript program set.

Conceptually:

```rust
pub struct RelocatableTapscriptBundle {
    pub schema_version: RelocatableSchemaVersion,

    pub architecture: ArchitectureBinding,
    pub realization: RealizationBinding,
    pub analyzed_program: AnalyzedProgramBinding,
    pub target: TargetBinding,
    pub configuration: TapscriptConfigurationBinding,

    pub constructors: Vec<RelocatableObjectConstructor>,
    pub operation_programs: Vec<RelocatableOperationProgram>,
    pub symbols: SymbolRegistry,
    pub relocations: Vec<Relocation>,

    pub placement: TapscriptPlacementPlan,
    pub layouts: Vec<TapscriptOperationLayout>,
    pub witness_requirements: Vec<PrelinkedWitnessSchema>,
    pub resource_formulas: Vec<TapscriptResourceFormula>,
    pub coverage_bindings: Vec<RelationCarrierBinding>,
}
```

> Illustrative API; exact fields and names are not frozen.

### 6.1 Relocatable object constructor

A constructor may contain:

- object kind;
- static script/leaf templates;
- dynamic metadata parameters;
- internal key requirement;
- static subtree references;
- metadata commitment rule;
- successor-verification requirements;
- symbol definitions;
- symbol references;
- resource formula;
- source relation provenance.

### 6.2 Relocatable operation program

An operation program may contain:

- operation ID;
- semantic relation IDs carried;
- selected proof methods;
- target leaf/program role;
- typed script representation;
- stack contract;
- required symbols;
- relocations;
- preliminary witness schema;
- layout role;
- resource formula;
- target requirements;
- source provenance.

### 6.3 Relocations

A relocation must identify:

- relocation kind;
- target program/constructor;
- location in typed script/template structure;
- symbol or deployment value required;
- expected type and width;
- source relation or constructor provenance;
- whether link resolution is mandatory or conditional.

A relocation is not an untyped byte offset without semantic context.

The linker may eventually translate typed relocations into byte patches after
script serialization.

### 6.4 Prelinked witness schema

The backend emits witness requirements tied to target programs.

The final ABI is produced only after linking resolves:

- exact leaf/program identity;
- control path;
- constructor commitments;
- calibrated limits;
- final target program bytes.

The prelinked schema must not be published as final ABI unless all unresolved
references are closed.

---

## 7. Public API boundary

### 7.1 Emission API

The planned backend API is conceptually:

```rust
pub fn emit(
    plan: &compiler::TargetCompilationPlan,
    target: &target_elements::ElementsTarget,
    configuration: &TapscriptConfiguration,
) -> Result<RelocatableTapscriptBundle, TapscriptError>;
```

> Illustrative API; not frozen.

The function must be:

- pure;
- deterministic;
- independent of filesystem and environment;
- fail-closed;
- identity-bound;
- non-writing;
- complete for the selected operation scope.

### 7.2 Foundational prototype APIs

Research prototypes may expose temporary internal APIs for:

- STATE constructor verification;
- wide arithmetic;
- public declassification;
- settlement layout.

Prototype APIs must be:

- clearly nonpublic or experimental;
- excluded from release identity unless accepted;
- removed or stabilized after the research decision;
- unable to become accidental production fallback.

### 7.3 Script inspection API

Downstream linker/vector tooling needs read-only access to:

- typed instructions;
- stack contract;
- relation carriers;
- relocations;
- symbol references;
- resource formulas;
- witness requirements;
- selected proof plans.

Avoid exposing mutable script internals that allow downstream code to alter
emitted semantics without changing bundle identity.

### 7.4 Serialization API

Typed script serialization should be deterministic.

Conceptually:

```rust
pub fn encode_program(
    program: &TapscriptProgram,
    target: &ElementsTargetDefinition,
) -> Result<Vec<u8>, TapscriptError>;
```

Encoding must validate target domain and operand forms.

The backend should not accept arbitrary raw script bytes as if they were typed
programs.

### 7.5 No process-global API state

Backend behavior must not depend on:

- global selected target;
- global configuration;
- environment variables;
- current directory;
- mutable singleton pattern registry.

Every behavior-affecting input is explicit.

---

## 8. Internal module plan

A likely module structure is:

```text
tapscript/src/
├── lib.rs
├── error.rs
├── configuration.rs
├── identity.rs
├── program.rs
├── instruction.rs
├── builder.rs
├── stack.rs
├── value.rs
├── symbol.rs
├── relocation.rs
├── emit.rs
│
├── patterns/
│   ├── mod.rs
│   ├── explicit.rs
│   ├── introspection.rs
│   ├── authorization.rs
│   ├── timelock.rs
│   ├── conservation.rs
│   ├── commitment.rs
│   ├── constructor.rs
│   └── hashing.rs
│
├── arithmetic/
│   ├── mod.rs
│   ├── narrow.rs
│   ├── limbs.rs
│   ├── wide_product.rs
│   ├── comparison.rs
│   └── floor.rs
│
├── constructor/
│   ├── mod.rs
│   ├── metadata.rs
│   ├── tapleaf.rs
│   ├── tapbranch.rs
│   ├── tweak.rs
│   ├── xonly.rs
│   └── object.rs
│
├── layout/
│   ├── mod.rs
│   ├── index.rs
│   ├── range.rs
│   ├── coordinator.rs
│   ├── placement.rs
│   └── operation.rs
│
├── witness/
│   ├── mod.rs
│   ├── item.rs
│   ├── schema.rs
│   └── availability.rs
│
├── optimize/
│   ├── mod.rs
│   ├── schedule.rs
│   └── peephole.rs
│
└── resources/
    ├── mod.rs
    ├── formula.rs
    ├── stack.rs
    ├── crypto.rs
    └── script.rs
```

> Illustrative module layout; create modules only when implementation requires
> them.

Avoid a large empty package skeleton.

---

## 9. Typed tapscript program representation

### 9.1 No raw-byte-first construction

The backend should construct a typed instruction sequence, then serialize it.

A typed program allows:

- stack-effect validation;
- target-domain validation;
- source relation provenance;
- deterministic encoding;
- resource accounting;
- pattern testing;
- structured relocation;
- verified rewrites.

### 9.2 Instruction representation

Conceptually:

```rust
pub enum Instruction {
    Push(TypedPush),
    Opcode(target_elements::OpcodeName),
    Relocation(RelocationId),
    BeginConditional,
    Else,
    EndConditional,
}
```

> Illustrative API; not frozen.

If target control-flow instructions are ordinary opcodes, the internal type may
represent them more structurally to validate branch stack effects.

### 9.3 Typed stack values

The backend should distinguish stack item semantics where useful.

Examples:

```text
ScriptNumber
Le64
Le32
Bool
AssetId
AssetPrefix
ValueBytes
ValuePrefix
XOnlyPublicKey
CompressedPublicKey
Scalar
Hash256
ScriptProgram
MetadataCommitment
Sha256Context
Signature
OpaqueBytes
```

The type system need not model every byte at compile time, but it must be
strong enough to catch:

- wrong operand order;
- wrong fixed width;
- wrong result interpretation;
- script-number/fixed-width confusion;
- asset/value-prefix confusion;
- wrong public-key form;
- invalid target opcode use.

### 9.4 Stack contract

Each instruction and pattern must have a stack contract.

Conceptually:

```rust
pub struct StackContract {
    pub before: StackShape,
    pub after_success: StackShape,
    pub after_failure: FailureStackBehavior,
    pub altstack_before: StackShape,
    pub altstack_after: StackShape,
}
```

> Illustrative API; not frozen.

Failure behavior matters for opcodes that:

- abort script;
- push a false success flag;
- leave operands on stack after overflow;
- have different stack results on optional/empty values.

### 9.5 Branch validation

For conditionals:

- both reachable branches must have compatible stack effects at join;
- branch-specific relations must remain source-provenanced;
- inactive branches must not carry unconditional obligations;
- script truth result must be canonical;
- target-specific minimal-if rules, if applicable, must be enforced.

### 9.6 Final truth condition

Every emitted leaf/program must have one explicit success contract.

Do not rely on incidental leftover stack truthiness without builder validation.

---

## 10. Pattern library

### 10.1 Purpose

Patterns lower target-independent proof-plan nodes into reusable typed
instruction fragments.

A pattern must declare:

- pattern identity and version;
- semantic proof class;
- required target capabilities;
- input fact-source requirements;
- target instructions;
- stack contract;
- failure behavior;
- resource formula;
- disclosure implications;
- witness requirements;
- positive/negative pattern vectors;
- source provenance.

### 10.2 Pattern selection

The initial instruction selection policy should be deterministic
maximal-munch or another simple canonical rule over a small approved pattern
library.

Do not implement global target-code optimization in the first backend.

### 10.3 Pattern completeness

Emission fails when no pattern supports a selected proof obligation.

The backend must not:

- omit the obligation;
- weaken it;
- emit an untyped custom byte fragment;
- use a target capability not present in the selected target.

### 10.4 Pattern composition

Pattern composition must validate:

- stack compatibility;
- fact-source compatibility;
- witness-item compatibility;
- branch behavior;
- duplicate disclosure;
- resource formula composition;
- relation provenance.

### 10.5 Pattern identity

Pattern identities must be deterministic and configuration-bound.

Changing a pattern's instructions, stack contract, or resource formula moves
the backend pattern-library/configuration identity.

---

## 11. Explicit and confidential representation guards

### 11.1 Typed prefix checks

The backend must use target-owned typed prefix classes.

It must not duplicate raw prefix bytes in operation emitters.

Patterns should exist for:

- explicit asset requirement;
- explicit value requirement;
- confidential value requirement where selected;
- null/zero handling;
- unsupported/unknown prefix rejection;
- output nonce requirements where relevant.

### 11.2 Closed asset identity

Under D005, every closed protocol asset seam requires explicit asset identity in
the initial Elements backend.

The backend must enforce complete classification over every output capable of
carrying a closed protocol asset.

This requires both:

- local asset checks;
- global output-family closure.

A local input check alone does not prevent confidential closed-asset
exfiltration.

### 11.3 Value representation

The backend may select:

- explicit value arithmetic;
- confidential transaction conservation;
- commitment equality;
- public committed opening after research acceptance;
- normalization requirement.

The selected representation must match compiler-approved alternatives.

### 11.4 Unknown encodings

Unknown asset/value prefixes fail closed.

Do not treat them as open or confidential-safe by default.

### 11.5 Sponsor representation

Initial policy:

```text
asset:
    explicit L-BTC

value:
    explicit or confidential according to selected target proof

change asset:
    explicit L-BTC

change value:
    explicit or confidential according to selected target proof
```

Sponsor value confidentiality must not affect protocol relation values.

---

## 12. Transaction introspection patterns

### 12.1 Fact-source mapping

The backend maps compiler fact-source requirements to concrete Elements
introspection.

Examples:

```text
semantic input asset
    → inspect input asset at authenticated layout index

semantic output value
    → inspect output value at authenticated layout index

transaction input count
    → inspect target input count

current input family membership
    → current input index + authenticated family range + constructor check
```

### 12.2 Index authentication

A script must not inspect an attacker-selected index without proving it belongs
to the canonical layout role.

Index sources include:

- fixed ABI index;
- current input index;
- coordinator-derived bounded family index;
- authenticated output reference.

### 12.3 Tuple ordering

Introspection patterns must use exact target result ordering and byte encoding
from `target-elements`.

### 12.4 Bounds and failure

Out-of-range indexes must fail in the expected target manner.

Pattern tests must include:

- first valid index;
- last valid index;
- one past end;
- negative script-number form where relevant;
- wrong family slot;
- correct index with wrong constructor.

### 12.5 ScriptPubKey/program inspection

Program inspection patterns must distinguish:

- native witness program;
- witness version;
- x-only taproot program;
- non-native program hash/sentinel behavior.

Constructor verification must not assume every inspected program is tapscript
without checking its version/domain.

---

## 13. Authorization patterns

### 13.1 Input-owner authorization

The backend must map owner-authorization relations to output-committing target
signature patterns.

Requirements include:

- owner key authenticated from the input constructor/metadata;
- signature uses the selected target sighash profile;
- all required economic outputs are committed;
- multi-owner operations require every owner;
- missing owner cannot be hidden by coordinator success.

### 13.2 Refund-key authorization

The request input authenticates the refund key.

The target signature must not accept the receipt owner as a substitute.

### 13.3 Sponsor authorization

Each sponsor input authenticates its owner.

The sponsor signature and open-flow closure must prevent sponsor value from
altering formula-bound protocol outputs.

### 13.4 Operator authorization

Operator-key programs must use typed deployment key input and selected sighash
profile.

The operator check must appear only on operations/branches whose semantic
authorization requires it.

### 13.5 Permissionless paths

Permissionless paths must contain no owner or operator signature requirement.

The backend should support a mechanical post-emission lint that checks no
forbidden signature pattern appears on:

- admission;
- delayed cycle;
- settlement;
- relabel;
- compact-ash;
- clear.

Sponsor input signatures remain allowed because they authorize only sponsor
funds and are separately classified.

### 13.6 Cadence band

Cadence is a composite target relation.

The backend must eventually emit target programs preserving:

```text
before MIN:
    no valid cycle path

MIN <= age < MAX:
    operator-authorized path

age >= MAX:
    permissionless path
```

The exact leaf arrangement is target-specific and remains later work.

---

## 14. Arithmetic lowering

### 14.1 Narrow arithmetic

The target's signed 64-bit arithmetic may directly support relations whose
proven bounds fit the target signed domain.

The backend must prove or consume compiler proof that operand/result bounds are
safe.

Every arithmetic opcode with an explicit success flag must verify that flag.

### 14.2 Narrow ratio

The initial narrow ratio pattern may implement:

```text
floor(value * numerator / denominator)
```

when the realization/compiler bounds prove the product fits signed 64-bit
target arithmetic.

The pattern must validate:

- operand widths;
- nonnegative domain;
- denominator nonzero;
- multiplication success;
- division success;
- quotient/remainder interpretation;
- result domain.

### 14.3 Wide arithmetic

Relations such as:

```text
floor(a * b / d)
```

with large protocol amounts require a wide proof.

No production wide-arithmetic pattern is accepted before
[`../research/wide-arithmetic.md`](../research/wide-arithmetic.md) reaches a
decision.

Likely proof form:

```text
a * b = q * d + r
0 <= r < d
```

with bounded limb arithmetic.

The exact limb base, stack schedule, and evidence remain prototype-dependent.

### 14.4 Arithmetic pattern evidence

Every accepted arithmetic pattern requires:

- Rust reference evaluator;
- boundary vectors;
- malformed-width vectors;
- overflow vectors;
- quotient one below/above;
- zero-divisor vector;
- randomized differential tests;
- stack/resource formula;
- target-native execution;
- optional formal equivalence evidence if adopted.

### 14.5 No hidden host arithmetic

Compile-time target script construction may use wide Rust arithmetic for
constants and test expectations.

It must not silently replace a runtime target proof.

---

## 15. Hashing and structured preimages

### 15.1 Structured hash expressions

The compiler/backend boundary should preserve structured hash intent.

The backend selects:

- concatenation plus ordinary SHA-256;
- streaming SHA-256;
- target-specific tagged-hash construction.

### 15.2 `OP_CAT` path

Where the target supports byte concatenation and the complete preimage fits
target element limits, a deterministic concatenation path may be used.

### 15.3 Streaming path

Use streaming SHA-256 when:

- preimage exceeds one stack element;
- concatenation creates excessive stack pressure;
- metadata encoding requires chunked processing;
- target policy selects it.

### 15.4 Domain separation

Every protocol/constructor hash must include explicit domain separation owned
by the relevant typed schema.

Do not rely on script context alone for domain separation.

### 15.5 Hash resource formula

Patterns must account for:

- pushed constants;
- dynamic field sizes;
- intermediate stack elements;
- context size;
- opcode cost;
- maximum message length.

---

## 16. Object constructors

### 16.1 Constructor role

A protocol output's target program may depend on metadata such as:

- owner;
- class;
- state fields;
- target cycle;
- distribution counters;
- object kind;
- schema version.

Therefore a symbol is generally a constructor, not one constant scriptPubKey.

### 16.2 Constructor categories

The backend should distinguish at least:

1. fixed program constructor;
2. metadata-parameterized constructor;
3. STATE/root constructor requiring predecessor/successor continuity;
4. unspendable data-output constructor;
5. externally owned open-output constructor.

### 16.3 Candidate taproot construction

One research candidate is:

```text
fixed internal key
+
static code subtree
+
dynamic unspendable metadata leaf
→ taptree root
→ tweaked output key
```

This is not accepted merely by appearing in the package plan.

The state/object-constructor research note owns the decision process.

### 16.4 Metadata leaf

If a metadata leaf is used, it must be provably unspendable.

The backend must produce:

- metadata encoding;
- leaf version;
- leaf hash;
- branch construction;
- spend-rejection vectors;
- schema/domain separation.

### 16.5 Predecessor authentication

A target program reading metadata must authenticate it against the actual
consumed object constructor.

It must not trust separately supplied metadata without reconstructing or
verifying the predecessor commitment.

### 16.6 Successor continuity

A state/object successor must bind:

- intended metadata;
- intended static program subtree;
- intended internal key;
- intended target schema.

A successor committed under a different code subtree must reject even when
metadata is correct.

### 16.7 Reused authenticated root

If constructor continuity relies on a witnessed static code root, predecessor
and successor checks must reuse one authenticated value or otherwise prove
equality.

Separate attacker-selected predecessor and successor roots are insufficient.

### 16.8 X-only/compressed bridge

Where program inspection returns an x-only key and target tweak verification
requires a compressed point, the backend must implement a typed, tested bridge:

- witness or derive parity;
- restrict parity encoding;
- construct compressed point;
- verify target curve/tweak relation;
- reject wrong parity and malformed key.

### 16.9 Hash-to-scalar totality

If target tweak verification interprets a hash as a scalar with restricted
range, the constructor policy must define behavior for every hash result.

Possible policies remain research-dependent:

- reject rare out-of-range result;
- deterministic retry/nonce rule;
- another canonical scalar derivation.

Do not leave this behavior undefined.

---

## 17. Layout and placement lowering

### 17.1 Compiler requirements

The backend consumes:

- semantic families;
- cardinality minima;
- bound references;
- local/global relation classification;
- candidate semantic carriers;
- duplication policy;
- layout requirements;
- sponsor policy;
- coverage requirements.

### 17.2 Target-specific layout

The backend produces one `TapscriptOperationLayout` for each operation in scope.

It may define:

- concrete family order;
- fixed indexes;
- bounded contiguous ranges;
- coordinator input;
- optional output positions;
- sponsor suffix;
- data-output order;
- local leaf assignment;
- relation carrier assignment.

### 17.3 Counts propose ranges

Every count witness is validated against:

- target transaction counts;
- architecture minimum;
- calibrated maximum placeholder/value;
- concrete constructor checks;
- range disjointness;
- complete protocol-family coverage.

### 17.4 Coordinator selection

For repeated families, choose one canonical coordinator, expected initially to
be the lowest-index family member.

Coordinator status must be derivable from authenticated layout/index facts.

### 17.5 Local and global carriers

Local input programs enforce:

- current input constructor;
- current input class/asset;
- current owner authorization;
- local operation branch.

Coordinator enforces:

- family counts;
- complete output closure;
- aggregate conservation;
- sponsor boundary;
- event shape;
- transaction-global state relation.

The actual split is operation- and target-specific.

### 17.6 Unplaced obligation

If no target program can authenticate the facts required by a relation,
emission fails.

The backend does not defer the relation to the transaction builder unless the
normative contract explicitly makes it an off-chain construction policy rather
than an on-chain predicate.

### 17.7 Mixed-branch analysis

The backend must eventually test whether incompatible leaves/programs from one
or several object families can be combined in one transaction.

The first implementation may use:

- bounded enumeration;
- target-layout constraint checking;
- explicit mixed-leaf vectors.

A future solver may improve coverage.

---

## 18. Witness requirement derivation

### 18.1 Prelinked schema

The backend derives witness requirements from:

- selected proof patterns;
- constructor verification;
- authorization;
- arithmetic;
- layout counts;
- target program selection.

The schema records:

- item role;
- type/encoding;
- availability;
- secrecy;
- length bounds;
- source relation;
- target program.

### 18.2 Final ABI handoff

The linker resolves:

- exact program/leaf;
- taptree path;
- control data;
- constructor constants;
- relocations.

The transaction package then produces the final ABI under D006.

The backend must not claim its prelinked witness schema is complete if linker
data remains unresolved.

### 18.3 Witness deduplication

Conceptually shared facts may need duplicated witness items because tapscript
inputs have separate stacks.

The backend must account for duplication explicitly.

It cannot assume one transaction-global witness is automatically visible to
every input.

### 18.4 Public and secret witness separation

The schema must distinguish:

- public opening;
- owner secret;
- operator secret;
- sponsor secret;
- constructor-public data;
- deployment constant.

Canonical publications describe item classes, not secret values.

---

## 19. Relocations and symbols

### 19.1 Symbol categories

Potential symbols include:

- fixed object program;
- parameterized object constructor;
- static code subtree root;
- internal key;
- target operation leaf;
- deployment key;
- asset ID;
- calibrated bound;
- metadata schema/domain separator;
- sibling constructor reference.

### 19.2 Relocation categories

Potential relocation forms include:

```text
fixed byte constant
target asset ID
public key
constructor reference
subtree root
operation leaf hash
calibrated integer
metadata schema version
network/genesis binding
```

### 19.3 Typed relocation safety

Every relocation defines:

- expected semantic type;
- encoded width;
- byte order;
- allowed value domain;
- target program location;
- source provenance;
- whether multiple uses must resolve identically.

### 19.4 No unresolved final emission

The backend may return unresolved relocations.

The release cannot accept a bundle with unresolved mandatory relocations.

The linker owns closure.

---

## 20. Stack scheduling and peephole rewriting

### 20.1 Initial scheduler

Use a deterministic greedy stack scheduler.

Priorities:

- preserve typed stack contracts;
- minimize unnecessary deep stack access;
- stable output;
- simple diagnostics;
- predictable resource formulas.

Optimal stack shuffling is not a Phase-3 goal.

### 20.2 Scheduler input

The scheduler receives a typed pattern-level program with:

- operand dependencies;
- stack types;
- branch boundaries;
- values that may be consumed;
- values that must be preserved;
- witness ordering constraints.

### 20.3 Peephole rules

A small verified rewrite set may simplify patterns.

Each rewrite must have:

- typed precondition;
- before/after instruction sequence;
- stack-equivalence proof by test or formal reasoning;
- resource delta;
- target capability assumptions;
- canonical application order.

Examples might include:

```text
redundant swap pair removal
duplicate followed by drop removal
canonical shallow pick replacement
constant boolean simplification
```

Exact rules depend on target semantics.

### 20.4 Fixed-point application

Apply rules deterministically to a fixed point under a stable rule order.

Reject rewrite cycles during rule-set validation.

### 20.5 No semantic reordering

Peephole rewrites must not alter:

- fail-closed behavior;
- target error path in a way relevant to validity;
- branch activation;
- signature message;
- witness consumption;
- relation provenance;
- disclosure.

---

## 21. Resource formulas

### 21.1 Backend responsibility

The backend provides resource formulas for emitted programs and witness
requirements.

It does not calibrate complete operation bounds by itself.

### 21.2 Formula dimensions

At minimum track:

- encoded script bytes;
- constant push bytes;
- witness item count;
- witness item byte bounds;
- initial stack count;
- peak stack plus altstack count;
- maximum element size;
- executed opcode count or project metric;
- crypto operation count;
- target crypto budget consumption;
- branch-specific cost;
- target capabilities used.

### 21.3 Symbolic family counts

Resource formulas may contain variables tied to calibrated bound references.

For example:

```text
compact-ash coordinator cost =
    fixed cost
    + ash_count × per-ash inspection/check cost
```

The formula must state assumptions and monotonicity where claimed.

### 21.4 Complete transaction handoff

The transaction/calibration runner combines:

- per-input program formulas;
- witness ABI;
- transaction overhead;
- outputs;
- control data;
- confidential proofs;
- sponsor use.

The backend must not claim a bound fits based only on one script formula.

### 21.5 Formula validation

Compare formulas with measured concrete programs and transactions.

A mismatch is an evidence failure and may invalidate calibration.

---

## 22. Initial operation: `compact-ash`

### 22.1 Semantic input

The backend receives compiler analysis requiring:

- two or more ASH inputs;
- bound by `ASH_BATCH_MAX`;
- one ASH output;
- ownerless exact `U` conservation;
- permissionless construction;
- optional sponsor isolation;
- no root use;
- no specialized event.

### 22.2 Initial target policy

Expected target choices:

- explicit closed `U` asset identity;
- public/openable ASH values;
- explicit value arithmetic for the first implementation;
- local ASH constructor check on each ASH input;
- first ASH input as coordinator;
- contiguous ASH input range;
- one ASH output;
- optional sponsor suffix/change.

### 22.3 Local program obligations

Each ASH input program should enforce:

- current input is an authenticated ASH constructor;
- asset identity is explicit `U`;
- value representation matches selected public ASH mode;
- operation branch is `compact-ash`;
- current input lies in the authenticated ASH family range;
- no owner signature or operator secret is required.

### 22.4 Coordinator obligations

The coordinator should enforce:

- input count/range;
- minimum and maximum;
- complete ASH family classification;
- one ASH output;
- output constructor;
- exact aggregate value;
- no undeclared closed-asset output;
- sponsor boundary;
- transaction-level constraints;
- transition projection shape as enforceable on-chain.

### 22.5 Relocatable output

The operation must emit:

- ASH constructor reference;
- operation leaf/program;
- bound relocation if calibration unresolved;
- layout result;
- witness requirements;
- resource formulas;
- relation carrier bindings.

### 22.6 Required pattern vectors

Include:

- minimum valid batch;
- larger valid batch;
- too few inputs;
- too many inputs;
- wrong asset;
- wrong constructor;
- wrong output value;
- two ASH outputs;
- no ASH output;
- unclassified `U` output;
- confidential closed asset output;
- hidden signature gate;
- malformed family count;
- wrong coordinator;
- sponsor overlap;
- target index out of bounds.

---

## 23. Second operation: `transfer-live-receipts`

### 23.1 Semantic input

The backend receives:

- bounded nonempty live receipt input family;
- bounded nonempty live receipt output family;
- every owner authorization;
- same-class closure;
- aggregate `U` conservation;
- optional sponsor isolation;
- explicit and confidential value proof alternatives;
- no root use.

### 23.2 Initial target policies

Support at least:

1. explicit-value transfer;
2. confidential-value transfer when target/transaction support is ready.

Both require:

- explicit closed `U` asset identity;
- authenticated live receipt constructor;
- owner metadata;
- owner signatures with required output commitment;
- complete output-family closure.

### 23.3 Local obligations

Every receipt input:

- authenticates live receipt constructor;
- checks explicit `U` asset identity;
- authenticates owner;
- requires owner authorization;
- proves family membership and selected operation branch.

### 23.4 Coordinator obligations

The coordinator:

- authenticates family ranges/counts;
- checks every output constructor/class;
- checks output closure;
- enforces selected aggregate conservation proof;
- enforces sponsor boundary;
- prevents confidential closed-asset exfiltration;
- binds representation mode.

### 23.5 Confidential transfer evidence

A valid confidential transfer must:

- preserve semantic aggregate value;
- preserve explicit `U` identity;
- use target CT balance correctly;
- keep all `U` outputs classified;
- preserve authorization;
- produce the same public semantic projection as explicit transfer.

The backend unit suite alone does not establish target CT behavior; target-native
and bundle-level vectors remain required.

---

## 24. Foundational research integration

### 24.1 STATE constructor

No production STATE program lands until the constructor research produces an
accepted design.

Prototype code should remain feature-gated, test-only, or in a clearly
experimental module.

Required research outcomes include:

- predecessor authentication;
- successor reconstruction;
- static code continuity;
- dynamic metadata commitment;
- unspendable metadata leaf;
- x-only/compressed bridge;
- hash-to-scalar policy;
- resource measurement.

### 24.2 Wide arithmetic

No production wide floor pattern lands until:

- exact relation selected;
- limb schedule implemented;
- boundary vectors pass;
- target-native execution passes;
- resource formula is measured;
- optional formal evidence status is known.

### 24.3 Public declassification

No production `PublicCommitted` synchronization pattern lands until:

- opening format selected;
- commitment binding proven by target pattern;
- residual blinding routed;
- public availability established;
- permissionless successor construction demonstrated;
- malformed opening rejected.

### 24.4 Settlement layout

No general settlement emitter lands until the batch-size-2 prototype selects a
layout and placement strategy.

---

## 25. Error model

Errors should be typed, deterministic, and source-provenanced.

Candidate classes include:

```rust
pub enum TapscriptError {
    UnsupportedAnalyzedProgramSchema,
    AnalyzedProgramIdentityMismatch,
    UnsupportedTarget,
    TargetIdentityMismatch,
    UnsupportedConfigurationSchema,

    MissingRelation(realization::RelationId),
    MissingSelectedProof(realization::RelationId),
    UnsupportedProofAlternative(realization::RelationId),
    MissingTargetCapability(target_elements::ElementsCapability),
    WeakenedProofAttempt(realization::RelationId),

    MissingFactSource(realization::FactId),
    UnsupportedFactEncoding(realization::FactId),
    PermissionlessSecretDependency(realization::FactId),

    MissingPattern(ProofPatternId),
    PatternCapabilityMismatch(ProofPatternId),
    PatternStackMismatch(ProofPatternId),

    StackUnderflow,
    StackTypeMismatch,
    BranchStackMismatch,
    StackLimitExceeded,
    ElementSizeExceeded,

    InvalidLayout(architecture::OperationId),
    UnauthenticatedRange,
    MissingCoordinator,
    AmbiguousCoordinator,
    MissingPlacement(realization::RelationId),
    InvalidPlacement(realization::RelationId),

    UnsupportedRepresentation(RepresentationRequirementId),
    ConfidentialClosedAssetForbidden,
    UnauthenticatedOpening,

    UnsupportedArithmeticRelation(realization::RelationId),
    ArithmeticBoundNotProven,
    WideArithmeticPrototypeRequired,

    MissingConstructor(architecture::ObjectId),
    ConstructorPrototypeRequired(architecture::ObjectId),
    ConstructorContinuityFailure,

    DuplicateSymbol(SymbolId),
    UnknownSymbol(SymbolId),
    InvalidRelocation(RelocationId),
    UnresolvedRequiredValue(RelocationId),

    ResourceFormulaMissing(ProgramId),
    ResourceLimitExceeded,

    NonDeterministicSelection,
}
```

> Illustrative vocabulary; not frozen.

Errors must include typed operation/relation/pattern context without secrets.

---

## 26. Determinism and identity

### 26.1 Backend configuration identity

Bind:

- configuration schema;
- pattern-library version;
- stack scheduler;
- peephole rule set;
- layout-lowering policy;
- placement policy;
- constructor policy;
- proof-selection tie-break;
- resource formula version.

### 26.2 Relocatable bundle identity

The relocatable bundle identity should eventually bind:

- architecture identity;
- realization identity;
- analyzed-program identity;
- target identity;
- backend configuration identity;
- selected proofs;
- selected representations;
- typed programs;
- constructors;
- symbols;
- relocations;
- layouts;
- preliminary witness schemas;
- resource formulas;
- relation carrier bindings.

It is distinct from the final linked-bundle identity.

### 26.3 Canonical ordering

Use stable ordering for:

- constructors by architecture object ID;
- operations by architecture operation ID;
- programs/leaves by typed program role;
- relation carriers by relation ID;
- relocations by program and semantic location;
- symbols by typed symbol ID;
- witness items by program and canonical role;
- resource formulas by program ID.

### 26.4 No ambient nondeterminism

Emission must not depend on:

- hash-map order;
- thread completion;
- filesystem paths;
- current time;
- random pattern search;
- target node responses;
- environment variables.

### 26.5 Cryptographic constants

Deployment keys and target constants are explicit typed inputs or relocations.

Do not generate production keys or random constructor nonces inside emission.

---

## 27. Generated artifacts

### 27.1 Phase-3 default

The backend may initially return typed values only.

### 27.2 Candidate publications

Future derivative artifacts may include:

```text
tapscript-relocatable.json
tapscript-pattern-report.json
tapscript-layouts.json
tapscript-resource-formulas.json
```

Names are illustrative.

The final linked script bundle is linker-owned, not emitted directly as the
backend's publication.

### 27.3 Artifact law

Any committed backend artifact requires:

- typed source;
- schema version;
- deterministic rendering;
- canonical ordering;
- generator;
- non-writing checker;
- unknown-field rejection;
- identity binding;
- no reverse semantic dependency.

### 27.4 Human disassembly

A deterministic human-readable disassembly is useful for review.

It must:

- retain relation/pattern annotations outside executable bytes or in a separate
  report;
- avoid secrets;
- use canonical formatting;
- not become the source consumed by linker or vectors.

---

## 28. Testing strategy

### 28.1 Instruction encoding tests

For every used target instruction:

- typed opcode maps to exact byte;
- pushes are minimally/canonically encoded;
- unsupported execution domain rejects;
- malformed typed operand cannot serialize.

### 28.2 Stack-effect tests

Test:

- success stack shape;
- failure/overflow stack shape where non-aborting;
- branch joins;
- altstack behavior;
- witness initial stack;
- target stack limits;
- maximum element size.

### 28.3 Pattern tests

Every pattern has:

- positive vector;
- malformed operand vectors;
- wrong prefix/type vectors;
- target capability mismatch;
- stack contract test;
- resource formula comparison;
- deterministic encoding test.

### 28.4 Scheduler and rewrite tests

Test:

- deterministic schedule;
- stack correctness;
- rewrite equivalence;
- rewrite fixed-point termination;
- no rewrite cycles;
- expected resource deltas;
- identical semantic carrier provenance.

### 28.5 Layout tests

For pilot operations:

- canonical family ranges;
- count validation;
- coordinator selection;
- sponsor separation;
- optional-family handling;
- wrong-family slot rejection;
- complete protocol output classification.

### 28.6 Constructor tests

After prototype acceptance:

- valid predecessor;
- valid successor;
- wrong metadata;
- wrong code subtree;
- wrong internal key;
- wrong parity;
- wrong schema;
- metadata-leaf spend;
- malformed public opening where relevant.

### 28.7 Arithmetic tests

As described in the arithmetic section.

### 28.8 Target-native tests

The vector/target harness executes patterns and complete operation programs
against the exact pinned Elements regtest target.

Backend unit tests using a local interpreter are useful but not release target
evidence by themselves.

### 28.9 Public API tests

An external test should prove linker/vector code can:

- emit a pilot operation;
- inspect typed programs;
- inspect symbols/relocations;
- inspect relation carriers;
- inspect layout;
- inspect witness requirements;
- inspect resource formulas;
- serialize target programs deterministically;
- do so without mutable backend internals.

### 28.10 Mutation tests

Mutate:

- selected proof;
- relation carrier;
- target capability;
- stack contract;
- constructor reference;
- layout count;
- witness ordering;
- relocation;
- resource formula.

Require validation or target execution failure.

---

## 29. Phase milestones

### TS1 — Crate and typed instruction foundation

Deliver:

- package skeleton;
- configuration/error types;
- typed instruction representation;
- deterministic script encoding;
- no model/linker/transaction dependency.

### TS2 — Stack-effect engine

Deliver:

- typed stack values;
- instruction contracts from target package;
- branch validation;
- limit checks;
- diagnostics.

### TS3 — Basic pattern library

Deliver:

- explicit asset/value checks;
- transaction introspection;
- fixed-width narrow arithmetic;
- owner/sponsor signature patterns;
- basic hash construction;
- target capability validation.

### TS4 — Layout and placement lowering

Deliver:

- target-specific range/index types;
- coordinator selection;
- relation carrier bindings;
- sponsor region;
- prelinked witness schemas.

### TS5 — Relocatable artifact model

Deliver:

- constructors;
- programs;
- symbols;
- relocations;
- deterministic identity;
- typed validation.

### TS6 — `compact-ash` emission

Deliver:

- complete pilot operation;
- layout;
- placement;
- witness schema;
- resource formulas;
- pattern vectors.

### TS7 — End-to-end `compact-ash` handoff

Deliver typed relocatable output consumable by linker and transaction packages.

### TS8 — `transfer-live-receipts`

Deliver:

- owner authorization;
- receipt constructors;
- explicit-value path;
- confidential-value path when transaction/target support is ready;
- safety/minimality vectors.

### TS9 — STATE constructor integration

Deliver after accepted research decision.

### TS10 — Burn/clear, redemption, admission, settlement, cycle

Deliver in roadmap order.

---

## 30. Foundational Phase-3 exit criteria

The backend foundation is ready for the first end-to-end operation only when:

- [ ] `packages/tapscript` is a workspace member;
- [ ] package metadata follows workspace policy;
- [ ] the package depends on compiler and target-elements, not model;
- [ ] pure emission APIs read no files or environment;
- [ ] typed instructions serialize deterministically;
- [ ] every used opcode comes from the exact target registry;
- [ ] stack contracts include success and relevant failure behavior;
- [ ] branch joins are validated;
- [ ] stack/element limits are checked;
- [ ] explicit asset/value guard patterns exist;
- [ ] unknown prefixes fail closed;
- [ ] basic introspection patterns exist;
- [ ] narrow arithmetic patterns verify success flags;
- [ ] signature patterns bind the selected target sighash profile;
- [ ] permissionless-path signature lint exists;
- [ ] pattern IDs/configuration are deterministic;
- [ ] placement/layout lowering exists for pilot requirements;
- [ ] no relation remains unplaced;
- [ ] prelinked witness requirements are typed;
- [ ] relocations and symbols are typed;
- [ ] resource formulas exist for emitted patterns;
- [ ] prototype-only constructor/arithmetic code is not used as production
      fallback;
- [ ] target-native pattern tests pass;
- [ ] debug/release tests pass;
- [ ] Clippy with `-D warnings` passes;
- [ ] the checkout remains clean.

---

## 31. `compact-ash` backend exit criteria

The first complete backend operation is ready for linker/transaction
integration only when:

- [ ] every `compact-ash` realization relation is present in compiler analysis;
- [ ] every selected proof has an Elements pattern;
- [ ] every relation has a carrying program;
- [ ] every ASH input is locally authenticated;
- [ ] the coordinator is canonical;
- [ ] the ASH family range is authenticated;
- [ ] minimum and bound references are enforced or relocated;
- [ ] exactly one ASH output is enforced;
- [ ] exact semantic `U` conservation is enforced;
- [ ] every closed-asset-capable output is classified;
- [ ] confidential closed-asset exfiltration rejects;
- [ ] no owner/operator signature is required;
- [ ] sponsor region is isolated;
- [ ] no specialized burn/clear/residue projection is accepted;
- [ ] relocatable symbols and constructors validate;
- [ ] prelinked witness schema validates;
- [ ] resource formulas match emitted program measurements;
- [ ] relation coverage bindings are complete;
- [ ] emitted bytes are deterministic;
- [ ] target-native valid and invalid program vectors pass.

The final complete transaction, linked bundle, and calibrated bound remain
downstream gates.

---

## 32. Non-goals

The tapscript backend does not:

- define protocol semantics;
- redefine architecture policy;
- parse model source;
- parse generated manifests;
- implement Simplicity;
- provide a general Bitcoin Script compiler;
- optimize arbitrary programs;
- own final taptree/link resolution;
- own final transaction ABI;
- generate production keys;
- run deployment ceremonies;
- calibrate complete transactions by itself;
- produce independent indexer reports;
- release a deployment;
- prove the entire compiler correct;
- support every Elements opcode;
- support confidential closed protocol asset identity;
- add arbitrary foreign sponsor sidecars;
- accept unsupported proof plans.

---

## 33. Open questions

### 33.1 Relocatable interface ownership

Should relocatable program roles be:

- tapscript-owned concrete types consumed by linker;
- defined by a small target-neutral interface package;
- compiler-associated backend artifact types?

Initial preference:

> tapscript owns concrete relocatable types; linker consumes them through a
> target-specific adapter. Avoid a new shared crate until a second backend
> demonstrates the common abstraction.

### 33.2 Final proof selection

Should resource-sensitive selection occur:

- in compiler target planning;
- in tapscript backend;
- in linker/calibration orchestration?

Requirement:

- only compiler-approved semantic alternatives;
- deterministic selection;
- selected-plan identity;
- no semantic weakening.

### 33.3 Stack scheduler complexity

Begin with deterministic greedy scheduling.

Upgrade only if measured target programs fail resource limits.

### 33.4 Script annotation

Determine how relation/pattern provenance appears in:

- typed programs;
- human disassembly;
- reports;
- linked bundle manifests.

Do not place nonsemantic annotation bytes into executable scripts unless
intended and identity-bound.

### 33.5 Constructor architecture

Blocked on state-object-constructor research.

### 33.6 Public committed values

Blocked on public-declassification research.

### 33.7 Wide arithmetic

Blocked on wide-arithmetic research.

### 33.8 Settlement placement

Blocked on settlement-layout research.

### 33.9 Target interpreter for local tests

Decide whether to:

- use only target-native node execution;
- implement a small local typed interpreter;
- use an existing Rust interpreter compatible with the exact target.

A local interpreter improves unit diagnostics but must not replace target-native
release evidence.

### 33.10 Taptree weighting responsibility

The backend may emit leaf/program roles and suggested weights. The linker owns
final taptree assembly.

Define the handoff before linking Phase 4.

---

## 34. Risks

### 34.1 Stack-machine complexity

Correct semantic relations may lower to brittle stack choreography.

Mitigation:

- typed stack contracts;
- deterministic scheduler;
- verified patterns;
- target-native vectors;
- simple initial operations;
- no optimizer-first work.

### 34.2 Constructor design may reshape interfaces

STATE/object constructor research may require changes to:

- metadata schema;
- witness ABI;
- relocations;
- linker symbols;
- resource formulas.

Mitigation:

- prototype before API freeze;
- keep Phase-3 APIs provisional;
- separate static constructor roles from exact target encoding.

### 34.3 Wide arithmetic cost

Exact floor proofs may exceed practical target limits.

Mitigation:

- standalone prototype;
- measurement before operation integration;
- calibrated bounds;
- alternative target proof methods;
- future Simplicity backend.

### 34.4 Placement omissions

A relation may be analyzed but receive no executable carrier.

Mitigation:

- typed placement plan;
- emission validation;
- relation carrier report;
- D004 coverage matrix;
- fail closed.

### 34.5 Hidden asset escape

Confidential/unclassified outputs may carry closed assets.

Mitigation:

- explicit closed-asset checks;
- complete output closure;
- negative vectors;
- D005 policy;
- target CT evidence.

### 34.6 Permissionless ransom regression

A convenience signature pattern may enter a permissionless branch.

Mitigation:

- authorization provenance;
- post-emission lint;
- operation vectors;
- no pattern insertion without relation carrier.

### 34.7 Backend/target divergence

The backend may hardcode semantics differing from `target-elements`.

Mitigation:

- typed opcode registry;
- no raw duplicate constants;
- source conformance;
- target-native tests;
- target identity binding.

### 34.8 Resource formula drift

Symbolic formulas may not match final serialized programs or transactions.

Mitigation:

- measured pattern comparison;
- linked-program comparison;
- complete transaction measurement;
- release calibration rerun.

### 34.9 Premature abstraction for Simplicity

Trying to make every concrete backend type universal may slow delivery.

Mitigation:

- target-neutral semantics above backend;
- concrete tapscript types below;
- defer shared artifact abstraction until a second backend exists.

### 34.10 Evidence correlation

Backend pattern tests and bundle vectors may use the same faulty helper.

Mitigation:

- target-native execution;
- independent semantic expectations;
- fixed critical vectors;
- relation provenance;
- later second-backend comparison;
- independent public observers.

---

## 35. Definition of done

The tapscript package plan is fulfilled for the first production backend when
the repository can deterministically lower every selected compiler relation
into typed, stack-validated, source-provenanced Elements tapscript constructor
and operation programs under one exact target; emit complete target-specific
placement, layout, witness, relocation, and resource artifacts; pass
pattern-level and target-native evidence; hand one validated relocatable bundle
to the linker; and do so without redefining protocol semantics, reading
generated publications, depending on model source, accepting confidential
closed-asset identity, hiding secret requirements in permissionless paths, or
claiming that backend unit tests alone constitute deployment proof.

---

## 36. One-line package contract

> `tapscript` consumes a target-independent analyzed proof plan and an exact
> typed Elements target, then deterministically emits typed relocatable
> tapscript constructors and operation programs with validated stack effects,
> explicit representation guards, concrete relation carriers, canonical layout
> lowering, witness requirements, relocations, resource formulas, and evidence
> provenance—while leaving semantic authority, final linking, transaction
> construction, calibration, and release to their owning packages.
