# Static review of tree 0.3.2-dev

## Review summary

This is an unusually disciplined pre-production codebase. The strongest parts are:

- clear authority and package boundaries;
- pervasive typed IDs rather than stringly identities;
- exact canonical/open-flow partitioning in the model;
- explicit distinction among semantic validation, target capability, evidence, provenance, and release readiness;
- fail-closed handling of missing compiler relations, proof alternatives, carriers, lifecycle paths, and evidence;
- careful sponsor-value erasure;
- deterministic source and graph construction;
- strong publication, census, and path policies;
- honest non-claims about production readiness, secrets, filesystem containment, and target evidence.

I did **not** find an obvious currently reachable model path for unauthorized issuance, reserve extraction, class crossing, or recipient redirection in the selected production sources.

However, I found several important issues in the newly introduced target boundary. The first three should be treated as **Phase-3 blockers before the typed instruction core or backend patterns rely on these contracts**.

### Review limitations

This was a static review of the supplied concatenation at tree `0.3.2-dev`. I did not execute the repository. The supplied content excluded:

- `Cargo.lock`;
- most unit and integration test bodies;
- `document-stamps`, `execwrap`, and `flatten-latex-main`;
- `docs/attestation/human.md`;
- `packages/model/generated/architecture.json`;
- licence files.

Consequently, this review makes no fresh build, test, advisory, lockfile, licence, or reproducibility claim.

---

# Findings

## 1. High: several opcode contracts cannot represent all successful target stack shapes

**Affected code**

- `packages/target-elements/src/opcode.rs`
- `StackContract`
- `StackValueType`
- `input_introspection_opcodes`
- `input_sequence_and_issuance_opcodes`
- `output_introspection_opcodes`
- `timelock_opcodes`

The target package claims that every reviewed primitive has a complete typed contract for operands, successful results, failure effects, and resources. The current `StackContract` has only one successful result sequence:

```rust
pub struct StackContract {
    operands: Vec<StackValueType>,
    success_results: Vec<StackValueType>,
    failure: FailureContract,
}
```

Several reviewed primitives have multiple successful result shapes that this type cannot express.

### Concrete cases

#### `InspectInputIssuance`

The code comments correctly say:

> an input carrying an issuance pushes six items; an input carrying none pushes a single empty item.

But the contract records only the six-item issuance-present result:

```rust
vec![
    EncodedPayload(ExplicitValue),
    EncodingPrefix(ExplicitValue),
    EncodedPayload(ExplicitValue),
    EncodingPrefix(ExplicitValue),
    Encoded(IssuanceEntropy),
    Encoded(IssuanceBlindingNonce),
]
```

The valid no-issuance success result is absent from the type.

#### Input and output value inspection

The comments say the returned value may be explicit or confidential, with different payload widths. Nevertheless, the result is typed only as:

```rust
EncodedPayload(ExplicitValue)
EncodingPrefix(ExplicitValue)
```

That does not describe a confidential value payload. In particular, an explicit value payload is 8 bytes while a confidential value payload is 32 bytes. A later stack checker relying on the declared result type can calculate the wrong width and accept or reject the wrong program shape.

The same issue affects both:

- `InspectInputValue`;
- `InspectOutputValue`.

#### Program inspection

The comments say a witness program is returned directly, while a non-witness program is replaced by a SHA-256 digest with a negative version marker. The contract records only:

```rust
Encoded(WitnessProgram)
ScriptNumber
```

It cannot represent the `ScriptPubKeySha256` result as such.

#### Output nonce inspection

The result may be explicit, confidential, or absent, but the contract records only:

```rust
Encoded(ExplicitNonce)
```

#### `CheckSequenceVerify`

The code comments say:

> The operand is inspected and left in place; this primitive pushes and pops nothing.

But the stack contract records one operand and no success results. Under the ordinary interpretation used elsewhere in this module—operands are consumed and results are pushed—that describes a net stack reduction by one, not an unchanged stack. The resource row says growth zero, creating two contradictory descriptions of successful execution.

### Impact

No target program emitter exists yet, so this is not a deployed exploit. It is nevertheless a high-severity foundation defect: the next phase’s instruction builder and stack scheduler are supposed to trust this contract. A scheduler based on it could:

- calculate the wrong successful stack depth;
- treat a valid confidential result as malformed;
- accept a branch join whose actual target stacks differ;
- fail to handle an absent issuance;
- misclassify a digest as a witness program;
- accidentally consume the CSV operand in its abstract execution model.

### Recommendation

Replace the single `success_results` field with a typed success relation capable of representing discriminated outcomes and retained operands. For example:

```rust
pub enum SuccessContract {
    Fixed {
        consumed_operands: usize,
        results: Vec<StackValueType>,
    },
    Alternatives {
        cases: Vec<SuccessCase>,
    },
    RetainsOperands {
        results: Vec<StackValueType>,
    },
}
```

A `SuccessCase` should carry a typed condition such as:

- explicit encoding;
- confidential encoding;
- null/absent field;
- witness program;
- non-witness program;
- issuance present;
- issuance absent.

The type should make the actual post-success stack explicit rather than relying on prose.

Add independent exact fixtures for every successful form, especially:

1. explicit/confidential input value;
2. explicit/confidential output value;
3. explicit/confidential/null nonce;
4. witness/non-witness program;
5. issuance present/absent;
6. CSV success retaining its operand.

---

## 2. High: target-definition validation does not weld redundant subcontracts together

**Affected code**

- `packages/target-elements/src/definition.rs`
- `validate_authorization`
- `validate_confidential_and_issuance`
- `validate_resources`
- `validate_capabilities`
- `validate_evidence`

The target definition deliberately carries some facts in several typed views:

- opcode-level signature behavior;
- `SignaturePrimitiveContract`;
- opcode-level relative-timelock behavior;
- `RelativeTimelockContract`;
- opcode resource costs;
- authorization budget;
- capability statuses;
- confidential-value statuses;
- issuance fields and issuance introspection;
- resource limits;
- evidence requirements.

That duplication can be reasonable if the validator welds the views together. It currently does not.

### Examples of contradictory definitions that appear able to validate

A caller can construct public `TargetDefinitionParts` using the public component constructors and create inconsistencies such as:

- `CheckSig` says an empty signature consumes operands and pushes false, while `SignaturePrimitiveContract::empty_signature` says it aborts;
- the signature opcode charges validation budget 50, while `SignaturePrimitiveContract::budget_per_check` says another value;
- `CheckSequenceVerify` says an unsatisfied lock aborts, while `RelativeTimelockContract::unsatisfied` says it retains operands and pushes false;
- the issuance contract omits one or more members of `IssuanceField::ALL`;
- the confidential-value contract classifies a claim differently from the corresponding `ElementsCapability`;
- the issuance contract names an introspection opcode whose detailed result contract is incompatible with the issuance field set;
- authorization, confidential-value, issuance, or resource subcontracts carry an empty evidence set.

The existing validator checks local shape but not these cross-view equalities.

For example, `validate_authorization` currently checks:

- contradictory or unclassified sighash dimensions;
- sequence-field overlap;
- nonempty timelock modes;
- existence of the signature encodings.

It does not compare the authorization contract against the opcode registry.

Similarly, `validate_confidential_and_issuance` checks that confidential claims are classified and that the issuance opcode/absent encoding exists, but it does not establish exact closure or semantic agreement.

### Impact

`ValidatedTargetDefinition` is the public trust boundary consumed by `tapscript`. Its documentation says consumers need not revalidate a half-specified contract. At present, the wrapper proves mostly local well-formedness, not internal semantic agreement among the contract’s redundant views.

A contradictory custom target can therefore become “validated” and be assessed by the adapter. That undermines the principal assurance provided by the wrapper.

### Recommendation

Add explicit cross-contract weld validators. At minimum:

#### Signature weld

Require agreement among:

- `SignaturePrimitiveContract`;
- `CheckSig`;
- `CheckSigVerify`;
- stack-message signature primitives;
- `OpcodeResourceCost::validation_budget`;
- relevant encoding contracts;
- required evidence.

#### Relative-timelock weld

Require agreement among:

- `RelativeTimelockContract`;
- `CheckSequenceVerify`;
- sequence encoding;
- transaction-version prerequisite;
- failure outcome;
- evidence.

#### Confidential-value weld

Require agreement among:

- `ConfidentialValueContract`;
- `ElementsCapability` status rows;
- participating encodings;
- relevant introspection capabilities;
- evidence.

#### Issuance weld

Require:

```text
issuance.fields = IssuanceField::ALL
```

for the reviewed V1 contract, and exact agreement among:

- issuance introspection opcode;
- outpoint issuance flags;
- null marker;
- issuance encodings;
- `IssuanceIntrospection` and `ReissuanceIntrospection` capability contracts;
- evidence.

#### Resource weld

Require agreement among:

- per-opcode resource costs;
- signature/curve per-check budget;
- validation-budget offset;
- consensus/policy resource dimensions.

Each weld needs focused mutation tests showing that a locally coherent but globally contradictory definition is rejected.

---

## 3. High: capability assessment ignores the status of transitive prerequisites

**Affected code**

- `packages/target-elements/src/definition.rs`
  - `validate_capabilities`
- `packages/target-elements/src/capability.rs`
  - `CapabilityContract`
- `packages/tapscript/src/capability.rs`
  - `assess_capability`

The target capability graph records typed prerequisites, but status propagation is not enforced.

`validate_capabilities` verifies that prerequisites exist and that the graph is acyclic. It does not reject a `Reviewed` capability whose prerequisite is `Incomplete` or `Unsupported`.

`assess_capability` then checks only the status of the directly named primitives:

```rust
match contracts.get(primitive).map(CapabilityContract::status) {
    Some(Reviewed) => {}
    Some(Unsupported) => unsupported.insert(...),
    _ => missing.insert(...),
}
```

It does not traverse the primitive’s prerequisite closure.

### Concrete counterexample

A custom target definition could say:

```text
TapscriptExecution:
    Unsupported

InputAssetInspection:
    Reviewed
    prerequisite = TapscriptExecution

InputProgramInspection:
    Reviewed
    prerequisite = TapscriptExecution
```

The capability graph is acyclic and every prerequisite resolves, so the target validator can accept it.

The tapscript adapter can then assess:

```text
AuthenticatedObjectRecognition
```

as `BackendPatternRequired`, because all directly requested inspection capabilities are marked `Reviewed`, even though their execution-domain prerequisite is explicitly unsupported.

The same issue can affect arithmetic, signature, timelock, and introspection capabilities.

### Impact

This violates the adapter’s stated non-weakening guarantee. A target capability is not usable merely because its own row says `Reviewed`; every prerequisite needed to realize it must also be usable.

### Recommendation

Define and enforce status closure over the transitive prerequisite graph.

At minimum:

\[\operatorname{status}(c)=\mathrm{Reviewed}\Longrightarrow \forall p\in\operatorname{prereq}^{+}(c),\ \operatorname{status}(p)=\mathrm{Reviewed}\]

You should decide and type the corresponding rule for `Incomplete`:

- if any prerequisite is `Unsupported`, is the dependent capability itself `Unsupported`?
- if any prerequisite is `Incomplete`, must the dependent be at most `Incomplete`?

Then:

1. reject status-incoherent target definitions; and
2. have the adapter resolve the complete prerequisite closure defensively rather than trusting only direct statuses.

Add a mutation test for every status transition:

- reviewed child / incomplete parent;
- reviewed child / unsupported parent;
- incomplete child / unsupported parent;
- complete transitive chain;
- permutation-invariant closure.

---

## 4. Medium: `EncodingSpec` infers “numeric” from the byte order, making byte-order validation circular

**Affected code**

- `packages/target-elements/src/encoding.rs`
  - `EncodingSpec::new`
- `packages/target-elements/src/definition.rs`
  - `validate_encodings`

`EncodingSpec::new` sets:

```rust
numeric: byte_order.is_some(),
byte_order,
```

The validator then checks:

```rust
if spec.is_numeric() && spec.byte_order().is_none() { ... }
if !spec.is_numeric() && spec.byte_order().is_some() { ... }
```

Those branches are tautologically unreachable because `numeric` is derived from exactly the condition being checked.

More importantly, the semantic key does not independently determine whether a byte order is required. A caller can construct:

```text
SignedLittleEndian64
byte_order = None
```

The constructor sets `numeric = false`, and validation accepts it as an opaque encoding despite the class’s meaning.

Conversely, assigning a byte order to an opaque class makes it “numeric” by definition and therefore also passes the local consistency test.

### Impact

A V1 encoding can pass validation while contradicting its stable encoding class. This is especially dangerous for:

- `ExplicitValue`;
- `ScriptNumber`;
- fixed-width signed and unsigned integers;
- outpoint index;
- sequence;
- transaction version and locktime.

The target’s typed byte-order contract is therefore not actually enforced for caller-supplied definitions.

### Recommendation

Make numeric interpretation independent of the supplied byte order.

Possible approaches:

```rust
pub enum PayloadInterpretation {
    Opaque,
    SignedInteger,
    UnsignedInteger,
}
```

or derive it from a total package-owned match over `EncodingClass`.

Validation should then require:

- numeric class ⇒ exactly one byte order;
- opaque class ⇒ no byte order;
- class-specific expected width/domain/canonicality where V1 fixes them.

Add mutations for every numeric encoding with its byte order removed or reversed, and for opaque encodings with spurious orders.

---

## 5. Medium: the tapscript adapter drops the compiler’s external-evidence-role census

**Affected code**

- `packages/compiler/src/target.rs`
  - `ExternalEvidenceRole`
  - `TargetRequirementSet`
- `packages/tapscript/src/capability.rs`
  - `assess_requirements`
  - `CapabilityAssessmentSet`

The compiler’s public target boundary intentionally carries two censuses:

```rust
TargetRequirementSet {
    capabilities,
    external_evidence,
}
```

But `assess_requirements` consumes only:

```rust
requirements.capabilities()
```

The `requirements.external_evidence()` iterator is never consulted, and the returned `CapabilityAssessmentSet` has no corresponding compiler-evidence-role map.

For the current pilots this is partly masked because substrate conservation also introduces `WholeTransactionValueConservation`, which maps to a target evidence requirement. But there is no exact checked equality such as:

```text
compiler external-evidence roles
=
adapter external-evidence assessments
```

A new `ExternalEvidenceRole` can be added to the compiler without forcing the tapscript adapter to classify it. In contrast, adding a new `RequiredCapability` makes the adapter’s exhaustive match fail to compile.

### Impact

The first downstream consumer of the compiler target boundary silently discards part of that boundary. A future compiler evidence role could disappear without a compiler error or adapter census failure.

It also weakens traceability: the adapter can say that a target evidence requirement exists, but cannot state which compiler-owned external evidence role caused it.

### Recommendation

Add an evidence-role assessment alongside capability assessment:

```rust
pub struct TargetAssessmentSet {
    capabilities: BTreeMap<RequiredCapability, CapabilityAssessment>,
    external_evidence: BTreeMap<ExternalEvidenceRole, ExternalEvidenceAssessment>,
}
```

Use an exhaustive match over `ExternalEvidenceRole`, and require exact census equality in both directions.

For the current role, the mapping should explicitly say:

```text
SubstrateConservation
    → TargetEvidenceRequirementId::ConfidentialValueConservation
```

The exact naming may differ, but the role must remain visible.

---

## 6. Medium-low: the active architecture semantic hash does not follow ADR-016’s domain-separated recipe

**Affected code**

- `packages/architecture/src/canonical.rs`
  - `semantic_hash`
  - `export_body_hash_hex`
- `adr/016-semantic-identities-and-evidence-binding.md`

ADR-016 defines semantic identity as:

\[
I_X=H(D_X\parallel V_X\parallel C(P_X(X)))
\]

The behavioural and deployment-profile hashes follow this pattern with explicit domain prefixes. The architecture semantic hash does not:

```rust
let bytes = canonical_json_bytes(architecture)?;
let digest = Sha256::digest(bytes);
```

The algorithm identifier is carried beside the hash in the publication envelope, but neither the domain nor recipe identifier is included in the hashed input itself.

The current identity register accurately documents the actual recipe, so this is not hidden. It is nevertheless inconsistent with the subsequently adopted general identity rule.

### Impact

This is not an authenticity vulnerability—ADR-016 correctly says an unkeyed digest proves no authenticity—but it leaves one active semantic identity outside the repository’s own domain-separation policy.

### Recommendation

Choose one of two explicit resolutions:

1. **Migrate the recipe** to a new algorithm identifier, e.g. a domain-separated v3, following ADR-016’s migration rule; or
2. **Document a reviewed grandfathered exception** in ADR-016 for the existing architecture semantic hash, including why contextual typing of the field is considered sufficient.

Do not silently add the prefix under the existing algorithm identifier.

---

## 7. Low: current package-index documentation still marks implemented packages as planned

**Affected code**

- `plans/packages/README.md`
- root `README.md`

`plans/packages/README.md` currently says:

```text
target-elements | Planned
tapscript        | Planned
```

But the package contracts, backlog, Phase-3 card, Cargo workspace, and source tree all say that:

- `target-elements` is active and its typed static contract is implemented;
- `tapscript` is active and its capability adapter is implemented.

The root README’s layout section also omits the now-important compiler/target/tapscript package roles or does not describe the current package set completely.

### Impact

This is documentation drift rather than a code defect, but it occurs in a package index that readers are likely to consult before the more detailed package contracts.

### Recommendation

Update the package index to the same status vocabulary used by the owning package contracts, for example:

```text
target-elements:
    Active — typed static target contract and development binding implemented;
    target-native evidence absent

tapscript:
    Active — capability adapter implemented;
    instruction core and backend patterns absent
```

A focused plan-check rule could compare index status against each package contract’s headline status, although that may be more machinery than this particular drift warrants.

---

# Suggested remediation order

I would close these in this order:

1. **Redesign successful opcode stack contracts** so every target result form is representable.
2. **Add target-definition cross-contract welds.**
3. **Enforce transitive capability-status closure** in both target validation and adapter assessment.
4. **Fix `EncodingSpec` interpretation/byte-order validation.**
5. **Add exact external-evidence-role assessment in tapscript.**
6. Resolve the architecture semantic-hash domain-separation policy.
7. Reconcile package-index documentation.

The first four belong together as a “target contract soundness” batch. They should land before Guide 9 begins relying on the current typed opcode contracts.

---

# Focused tests I would require

## Opcode result algebra

- explicit and confidential input-value result shapes;
- explicit and confidential output-value result shapes;
- explicit, confidential, and null nonce results;
- witness-program and non-witness-program results;
- issuance-present and issuance-absent results;
- CSV successful stack unchanged;
- every success branch’s exact stack depth and types.

## Cross-contract welds

- signature empty-failure disagreement rejected;
- signature budget disagreement rejected;
- CSV failure disagreement rejected;
- missing issuance field rejected;
- issuance opcode/result disagreement rejected;
- confidential capability/status disagreement rejected;
- missing authorization/CT/issuance evidence rejected;
- opcode resource versus resource-contract disagreement rejected.

## Capability closure

- reviewed capability with incomplete prerequisite rejected;
- reviewed capability with unsupported prerequisite rejected;
- adapter follows transitive prerequisites;
- prerequisite order permutation leaves assessment unchanged.

## Encoding validation

- numeric encoding with missing order rejected;
- numeric encoding with reversed order rejected where V1 fixes it;
- opaque encoding with byte order rejected;
- wrong field domain rejected;
- wrong fixed width rejected.

## Evidence role closure

- exact equality between compiler evidence roles and adapter evidence assessments;
- duplicate/missing/unexpected role rejection;
- adding a synthetic evidence role requires an adapter disposition.

---

# Overall verdict

The architecture/model/realization/compiler side is impressively careful, and the code consistently avoids the usual failure modes of this kind of project: undocumented authority, incidental graph identity, aggregate-only conservation, conflated evidence, and speculative hashes.

The main weakness is concentrated in the newest target layer:

> The target package’s prose and types promise a complete, internally coherent execution contract, but several target result alternatives are not representable and several redundant contract views are only locally validated rather than welded.

That is exactly the sort of defect worth finding now, while there is no emitted program, transaction ABI, production binding, or deployed authority. I would keep Phase 3 active and treat Findings 1–4 as blockers for the next instruction-core milestone.

# Second full static review

## Executive summary

I completed a second, independent pass over the supplied tree at:

```text
0.3.2-dev
```

The second pass confirms the broad conclusion of the first:

- the architecture/model/realization/compiler stack is unusually disciplined;
- I still found no obvious reachable valid-world model path for unauthorized issuance, reserve extraction, receipt-class escape, or recipient redirection;
- the most important weaknesses remain concentrated in the new `target-elements` and `tapscript` boundary;
- several target contracts currently prove **internal shape**, but not yet the stronger claim that they are the reviewed Elements contract they purport to represent.

Because this is the same supplied tree and no remediation diff was provided, all findings from the first review remain open for purposes of this review. The second pass found **four additional substantive issues**, led by one important trust-state problem:

> An arbitrary caller-assembled target definition can acquire the same `ValidatedTargetDefinition` type as the built-in reviewed Elements definition, while choosing its own opcode semantics, capability statuses, resource limits, and evidence declarations.

That means the type currently records “locally self-consistent” but is easy to read downstream as “reviewed Elements contract.” The distinction needs to be represented explicitly before target-native work or backend emission relies on it.

---

# Scope and limitations

This was a static source review of the 292 selected files in the supplied concatenation.

The review did **not** execute:

```text
cargo fmt
cargo clippy
cargo test
scripts/ci.sh
meson compile
meson test
cargo audit
document reproducibility
```

The supplied report excluded, among other things:

- `Cargo.lock`;
- most unit and integration test bodies;
- licence files;
- `packages/document-stamps`;
- `packages/execwrap`;
- `packages/flatten-latex-main`;
- `docs/attestation/human.md`;
- generated `architecture.json`.

Therefore, this report makes no fresh claim about:

- compilation;
- test outcomes;
- MSRV;
- lockfile integrity;
- dependency features;
- advisories;
- licence compatibility;
- generated-artifact freshness;
- document reproducibility.

The excluded test names and planning gate records provide useful context, but they are not substitutes for independently reading or running those tests.

---

# Overall assessment

## Strong areas reconfirmed

The second pass reinforced several positive conclusions.

### 1. Semantic authority is unusually clear

The repository consistently distinguishes:

```text
Attestation
realization contract
typed architecture
executable model
compiler analysis
target contract
backend adapter
evidence
release
```

That separation is not merely documented. It is reflected in package dependencies and types.

### 2. Model value flow is substantially stronger than aggregate conservation

The model does not stop at:

\[
\sum \text{inputs}+\text{issued}=\sum \text{outputs}+\text{destroyed}
\]

It maintains exact canonical-flow and open-flow partitions, including:

- source uniqueness;
- destination uniqueness;
- issuance destination exhaustion;
- tagged destruction;
- role-local open-flow balancing;
- formula-bound payout validation;
- root succession;
- branch-specific semantic postconditions.

This closes many common “balanced theft” and “aggregate alibi” failures.

### 3. Authorization and recipient safety are treated separately

The code distinguishes:

- input authorization;
- operation authorization;
- value conservation;
- recipient closure;
- permissionless constructibility.

That separation is correct and valuable.

### 4. The compiler is consistently fail-closed

The compiler generally refuses to:

- drop relations;
- hide lifecycle obligations;
- invent weaker proof alternatives;
- accept partial exact searches;
- treat local graph handles as identities;
- conflate runtime, compiler-static, structural, and external-evidence boundaries.

### 5. Evidence non-claims are strong

The repository repeatedly and correctly states that:

```text
hash equality ≠ validity
model conformance ≠ target correctness
static target review ≠ node evidence
architecture finality ≠ deployment readiness
```

The main second-review concern is that one target type boundary does not yet encode all of those distinctions as strongly as the prose does.

---

# Consolidated finding register

## Previously reported and reconfirmed

| ID | Severity | Finding |
|---|---:|---|
| `R2-C01` | High | `StackContract` cannot represent every valid target success shape. |
| `R2-C02` | High | Redundant target subcontracts are not bidirectionally welded. |
| `R2-C03` | High | Capability assessment ignores transitive prerequisite status. |
| `R2-C04` | Medium | Encoding numericity is inferred from byte-order presence, making validation circular. |
| `R2-C05` | Medium | Tapscript assessment drops the compiler’s external-evidence-role census. |
| `R2-C06` | Medium–Low | Active architecture semantic hashing does not follow ADR-016’s domain-separated form. |
| `R2-C07` | Low | Package-index documentation still contains stale implementation statuses. |

## Additional findings from this second pass

| ID | Severity | Finding |
|---|---:|---|
| `R2-N01` | High | A caller-authored target definition can acquire the same trusted wrapper as the built-in reviewed Elements definition. |
| `R2-N02` | Medium–High | The tapscript adapter accepts an `ElementsTarget` but ignores its deployment binding, activation declaration, and resource overrides. |
| `R2-N03` | Medium | Public architecture identity APIs can hash invalid, unvalidated architecture values. |
| `R2-N04` | Medium | Compare-if-changed publication ignores required file mode, so mode corruption is not repaired—including executable helper binaries. |

The findings are detailed below.

---

# Additional finding R2-N01

## High: the “reviewed target” trust state is caller-assertable

**Affected files**

- `packages/target-elements/src/definition.rs`
- `packages/target-elements/src/capability.rs`
- `packages/target-elements/src/opcode.rs`
- `packages/target-elements/src/deployment.rs`
- `packages/tapscript/src/capability.rs`

## Problem

The repository distinguishes a reviewed Elements target from an arbitrary target declaration in prose, but not in the principal validated type.

A caller can publicly construct:

```rust
TargetDefinition::new(TargetDefinitionParts { ... })
```

using public constructors for:

- opcode contracts;
- encoding contracts;
- authorization contracts;
- capability contracts;
- confidential-value contracts;
- issuance contracts;
- resources;
- evidence requirements.

The caller can then invoke:

```rust
validate_target_definition(...)
```

and receive:

```rust
ValidatedTargetDefinition
```

That is the same type returned by:

```rust
reviewed_elements_tapscript()
```

The two values differ only in contents. Their trust class is indistinguishable at the type boundary.

## Why this matters

The validation function checks internal properties such as:

- complete ID censuses;
- unique opcode bytes;
- local encoding shape;
- prerequisite existence;
- prerequisite acyclicity;
- nonempty evidence links;
- resource consistency.

It does **not** establish equality with the reviewed first-party Elements contract.

An arbitrary caller can therefore create a locally coherent V1 target definition that changes, for example:

- opcode bytes;
- opcode stack semantics;
- failure behavior;
- resource costs;
- capability statuses;
- evidence claim classes;
- encoding widths or prefixes;
- consensus resource limits.

Provided the altered contract remains locally coherent, it can receive `ValidatedTargetDefinition`.

The caller can then produce a `ValidatedDevelopmentBinding`, combine both into `ElementsTarget`, and pass the result to:

```rust
tapscript::assess_requirements
```

The adapter treats the target’s `Reviewed` capability statuses as meaningful static review facts.

## Example class of false reviewed target

Conceptually, an external caller can state:

```text
TargetContractVersion = V1
leaf version = reviewed 0xc4

CheckSig:
    caller-selected semantics

SignatureVerification:
    status = Reviewed

InputAssetInspection:
    status = Reviewed

resource limits:
    caller-selected

evidence registry:
    structurally complete
```

Nothing currently turns that into the built-in reviewed Elements contract rather than a caller’s claim about one.

## Why contract version does not solve it

`TargetContractVersion::V1` is a compatibility marker, not authenticity and not complete-value equality.

If opcode semantics or resource meaning can vary freely under V1, then V1 does not identify one compatibility contract. If they are not meant to vary, validation must enforce the fixed V1 values or compare with the reviewed expected contract.

This is analogous to the architecture envelope’s explicit distinction between:

```text
self-consistent envelope
```

and:

```text
publication equal to independently derived expected value
```

The target package currently lacks that second state.

## Impact

No target program or production deployment exists, so this does not create a deployed exploit today.

It does create an assurance-boundary defect:

- `ValidatedTargetDefinition` is stronger-sounding than the guarantee it carries;
- `ElementsTarget` does not prove the target is the reviewed Elements contract;
- the tapscript adapter can rely on caller-asserted “reviewed” capabilities;
- future target-native evidence could accidentally bind to a different typed contract than the one reviewed in first-party source.

## Recommendation

Represent at least two distinct states:

```rust
ValidatedTargetDefinition
ReviewedElementsTapscriptDefinition
```

Possible design:

```rust
pub struct ValidatedTargetDefinition {
    definition: TargetDefinition,
}

pub struct ReviewedElementsTapscriptDefinition {
    definition: ValidatedTargetDefinition,
}
```

Only the built-in reviewed derivation, or an exact equality check against it, should construct the reviewed wrapper:

```rust
pub fn reviewed_elements_tapscript()
    -> Result<ReviewedElementsTapscriptDefinition, Vec<TargetError>>;
```

For externally supplied typed definitions, provide an explicit expected-value check:

```rust
pub fn validate_against_reviewed_elements(
    offered: ValidatedTargetDefinition,
    expected: &ReviewedElementsTapscriptDefinition,
) -> Result<ReviewedElementsTapscriptDefinition, TargetError>;
```

Then either:

1. make `ElementsTarget` and the tapscript adapter require the reviewed wrapper; or
2. keep a generic target path, but make the distinction explicit in the resulting assessment and prohibit generic values from being described as reviewed Elements.

If custom V1 definitions are intentionally allowed, define exactly which fields may vary under V1. Any field outside that variability must equal the reviewed V1 value.

## Focused tests

Require tests proving:

- a permuted opcode-byte table cannot become reviewed V1;
- changed signature failure behavior cannot become reviewed V1;
- changed capability status cannot become reviewed V1;
- changed consensus resource limits cannot become reviewed V1;
- an internally valid but nonmatching target remains only `ValidatedTargetDefinition`;
- only the exact reviewed expected value gains the reviewed wrapper;
- tapscript cannot accept an unreviewed generic target where reviewed Elements is required.

---

# Additional finding R2-N02

## Medium–High: tapscript ignores the deployment half of `ElementsTarget`

**Affected files**

- `packages/target-elements/src/deployment.rs`
- `packages/tapscript/src/capability.rs`

## Problem

The adapter API accepts:

```rust
&ElementsTarget
```

An `ElementsTarget` contains:

```text
validated static target definition
+
validated development binding
```

The binding includes:

- environment class;
- network ID;
- genesis ID;
- activation declaration;
- required leaf version;
- required capability declaration;
- optional resource overrides.

But `assess_capability` reads only:

```rust
target.definition().definition().capabilities()
```

It does not read:

```rust
target.deployment()
```

at all.

Therefore target assessments are invariant under all deployment-binding differences.

## Concrete consequences

Two targets with identical static definition but different:

- network IDs;
- genesis IDs;
- activation declarations;
- resource overrides;

produce identical capability assessments.

More sharply, a development binding can validly state:

```text
tapscript_expected_active = false
required_capabilities = empty
```

and still be combined into `ElementsTarget`.

The tapscript adapter can then assess compiler requirements against it and return static dispositions such as:

```text
BackendPatternRequired
BackendStructural
ExternalEvidenceRequired
```

without acknowledging that the binding explicitly does not expect the execution domain to be active.

Similarly, a deployment may impose a very restrictive resource override, but the adapter’s assessment does not change.

## Why this is a contract problem

The adapter documentation says it assesses compiler requirements against:

> one validated target definition and development binding

But the implementation assesses only the static target definition.

That leaves two coherent designs, but the code and API currently mix them.

### Design A: static-contract assessment

If the assessment intentionally answers only:

> What obligations follow from this static target contract?

then the function should accept:

```rust
&ValidatedTargetDefinition
```

not `&ElementsTarget`.

Deployment activation and resource feasibility would remain a later, explicitly separate assessment.

### Design B: deployment-aware assessment

If it answers:

> What can this concrete development target declaration support?

then it must incorporate:

- expected domain activation;
- declared required capabilities;
- binding version;
- deployment resource overrides;
- possibly network-specific evidence requirements.

## Impact

Today this mainly creates a false impression of binding-aware assessment.

Later, it can become a correctness issue if a target plan is considered usable despite:

- inactive execution domain;
- deployment policy forbidding its resources;
- capability declaration not including its prerequisites;
- deployment binding changing while assessment remains silently reusable.

## Recommendation

Pick one boundary explicitly.

### Preferred immediate repair

Because target-native evidence and resource-aware emission do not yet exist, narrow the API:

```rust
pub fn assess_capability(
    target: &ValidatedTargetDefinition,
    required: RequiredCapability,
) -> CapabilityAssessment;
```

Name the result accordingly:

```text
StaticCapabilityAssessment
```

Then add a future deployment assessment only when there is a real consumer.

### Alternative

Keep `ElementsTarget`, but add typed dispositions such as:

```rust
ExecutionDomainNotDeclaredActive
DeploymentCapabilityNotDeclared
DeploymentResourceOverrideBlocks
DeploymentEvidenceRequired
```

and require exact deployment-sensitive closure.

## Focused tests

- inactive activation declaration changes the assessment or is outside the API;
- two different network/genesis bindings do not accidentally claim equivalent deployment evidence;
- restrictive resource override is visible at the appropriate boundary;
- static and deployment-aware assessments are different types;
- no function accepting `ElementsTarget` silently ignores `deployment()`.

---

# Additional finding R2-N03

## Medium: architecture identities can be computed over invalid architecture values

**Affected files**

- `packages/architecture/src/canonical.rs`
- `packages/architecture/src/export.rs`
- `packages/architecture/src/lib.rs`
- `packages/realization/src/binding.rs`

## Problem

ADR-016 says validation precedes semantic identity.

The deployment-profile API encodes that rule:

```rust
ValidatedDeploymentProfile
```

is the only public input to:

```rust
deployment_profile_hash(...)
```

The architecture identity API does not.

Public functions accept an arbitrary public `Architecture` directly:

```rust
pub fn semantic_hash(
    architecture: &Architecture,
) -> Result<[u8; 32], serde_json::Error>;

pub fn behavioural_hash(
    architecture: &Architecture,
) -> Result<[u8; 32], serde_json::Error>;

pub fn canonical_json_bytes(
    architecture: &Architecture,
) -> Result<Vec<u8>, serde_json::Error>;
```

`Architecture` is publicly constructible from public fields and slices.

Likewise:

```rust
PublishedArchitecture::from_architecture(...)
```

does not itself run `validate_draft`.

Current first-party consumers usually validate before calling:

- `ArchitectureBinding::from_architecture` validates;
- artifact generation validates;
- deployment release validates architecture separately.

But the public identity API itself allows an invalid object to bear the active architecture semantic hash recipe.

## Concrete invalid inputs that can be hashed

Examples include an architecture with:

- missing operation IDs;
- duplicate set-like declarations;
- issuance/authority mismatch;
- missing root;
- malformed object lifecycle;
- wrong reader firewall;
- incompatible branch cardinality.

The hash is self-consistent over the invalid body, but ADR-016 explicitly rejects treating rehashing as validation.

## Impact

This is not currently a model safety failure because the principal first-party consumers validate.

It is a public API and policy inconsistency:

- invalid architecture values can acquire a semantic hash;
- callers can mistakenly use that hash as though it represented validated architecture meaning;
- the architecture API is weaker than the deployment-profile API despite the architecture identity being active and more widely consumed.

## Recommendation

Introduce a validated architecture wrapper:

```rust
pub struct ValidatedArchitecture<'a> {
    architecture: &'a Architecture,
}
```

with separate constructors if draft and release validation need distinct states:

```rust
ValidatedDraftArchitecture
ValidatedReleaseArchitecture
```

Then require the validated wrapper for public identity functions:

```rust
pub fn semantic_hash(
    validated: &ValidatedDraftArchitecture<'_>,
) -> Result<[u8; 32], serde_json::Error>;
```

Keep unchecked projection helpers crate-private for mutation tests.

At minimum, if changing the public API now is too disruptive, add checked functions that return a combined validation/serialization error and clearly mark unchecked hash helpers as such. The active public semantic identity path should be the checked one.

`PublishedArchitecture::from_architecture` should likewise either:

- accept a validated wrapper; or
- run owner validation itself.

## Focused tests

- invalid architecture cannot receive a public semantic identity;
- invalid architecture cannot produce a trusted publication;
- draft-valid architecture can receive semantic identity if that is the intended policy;
- release identity requires final/pinned validation where applicable;
- mutation tests use crate-private unchecked projections only;
- no active consumer calls unchecked hashing.

---

# Additional finding R2-N04

## Medium: compare-if-changed ignores required publication mode

**Affected files**

- `packages/cli-common/src/publication.rs`
- `packages/cli-common/src/lib.rs`
- `scripts/cargo-bin-sync.sh`
- `scripts/sync-publication.sh`

## Problem

The repository correctly fixed the common tempfile problem where `mktemp` creates mode `0600` and the file is then renamed as a supposedly public artifact.

The repair sets the mode on staged files before publication.

However, every compare-if-changed path compares only bytes. If bytes already match, it leaves the destination untouched without checking the expected mode.

### `publish_batch`

```rust
if std::fs::read(asset.path).is_ok_and(|current| current == asset.bytes) {
    staged.push(None);
    continue;
}
```

No mode check occurs.

### JSON checker report publication

```rust
if std::fs::read(path).is_ok_and(|current| current == bytes) {
    return Ok(());
}
```

Again, no mode check.

### `cargo-bin-sync.sh`

```sh
if ! cmp -s "$built" "$dest"; then
    ...
    chmod 755 "$staged"
    mv "$staged" "$dest"
fi
```

If the bytes match but the destination lost its executable bit, the script does not repair it.

### `sync-publication.sh`

The same pattern applies to PDF and flattened-source mirrors.

## Concrete operational failure

Suppose a synced Meson helper binary exists with correct bytes but mode `0644`.

Because `cargo-bin-sync.sh` is always stale:

1. Cargo confirms the built binary is fresh.
2. `cmp` reports equal bytes.
3. The script skips the copy and `chmod`.
4. The destination remains non-executable.
5. The consumer fails with permission denied.
6. Re-running the build repeats the same failure indefinitely.

Deleting the destination repairs it, but ordinary rebuild does not.

For public reports and publications, mode `0600` similarly survives indefinitely when bytes are current.

## Why the tracked-mode audit does not catch this

Git records only the executable distinction, not complete worktree permission bits.

A tracked `100644` file can exist in the worktree as owner-only and still appear as ordinary `100644` to the central Git mode census.

Untracked build products are not covered by the tracked-entry mode audit at all.

## Impact

This is primarily build robustness and publication usability, not semantic corruption.

It does undermine the intended `PublicationMode` contract and the claim that publication mode is explicit rather than inherited from the environment.

The executable case is especially significant because it creates a persistent unrepaired build failure.

## Recommendation

Treat expected mode as part of the compare-if-changed publication state.

On Unix, equality should mean:

```text
bytes equal
∧
mode equal
```

For example:

```rust
fn destination_matches(
    path: &Path,
    bytes: &[u8],
    mode: PublicationMode,
) -> io::Result<bool>;
```

If bytes match but mode differs:

- repair the mode in place; or
- republish through the staged path.

Mode-only repair should be reported explicitly:

```rust
PublicationResult {
    bytes_changed: false,
    mode_changed: true,
}
```

For `cargo-bin-sync.sh`, check both:

```sh
cmp -s "$built" "$dest"
test -x "$dest"
```

and run `chmod 755 "$dest"` when bytes match but mode is wrong.

For public mirrors and reports, repair to `0644`.

## Focused tests

For each publication path:

1. absent destination;
2. stale bytes, correct mode;
3. current bytes, correct mode;
4. current bytes, owner-only mode;
5. executable bytes, missing executable bit;
6. mode-only repair preserves bytes;
7. mode-only repair does not spuriously rewrite content;
8. repeated post-repair run is a no-op.

---

# Previously reported findings, reconfirmed and refined

## R2-C01 — High: incomplete success-shape algebra

This remains the highest-priority implementation defect in the target contract.

`StackContract` has one success result sequence, while reviewed primitives have alternative successful shapes. Important affected cases include:

- issuance present versus absent;
- explicit versus confidential value;
- explicit/confidential/null nonce;
- witness versus non-witness script program;
- retained CSV operand.

This needs a typed success-case algebra, not more prose comments.

---

## R2-C02 — High: missing cross-contract welds

The second pass strengthened this finding.

The target currently carries overlapping claims in:

```text
opcode registry
authorization contract
encoding registry
capability registry
confidential-value contract
issuance contract
resource contract
evidence registry
```

Validation mostly proves each part is locally coherent. It does not prove all parts describe one target.

Particularly important cross-checks include:

- `CheckSig` failure semantics ↔ `SignaturePrimitiveContract`;
- opcode budget ↔ signature per-check budget;
- CSV stack/failure semantics ↔ relative-timelock contract;
- issuance result alternatives ↔ issuance field census;
- capability status ↔ confidential-value claim state;
- opcode costs ↔ resource limits;
- evidence links ↔ every non-opcode subcontract.

This finding and R2-N01 are distinct:

- R2-C02: one target definition can contradict itself;
- R2-N01: a self-consistent target definition can still be caller-authored rather than reviewed.

Both need closing.

---

## R2-C03 — High: transitive capability-status closure is absent

The second pass reconfirmed that:

```text
prerequisite existence
+
acyclicity
```

does not imply:

```text
prerequisite usability
```

The validator and adapter need a total rule for status propagation. At minimum:

\[
\operatorname{status}(c)=\mathrm{Reviewed}\implies\forall p\in\operatorname{prereq}^{+}(c),\ \operatorname{status}(p)=\mathrm{Reviewed}
\]

The adapter should defensively traverse the prerequisite closure even after target validation.

---

## R2-C04 — Medium: byte-order validation is circular

This remains valid.

`EncodingSpec::new` defines:

```rust
numeric = byte_order.is_some()
```

and validation checks numericity against the same byte-order presence. The validation can never detect a numeric encoding whose caller omitted its byte order, because omission reclassifies it as nonnumeric.

Numeric interpretation should be:

- an explicit independent field; or
- derived from a total match over `EncodingClass`.

V1 should also validate class-specific expected domain, width, canonicality, and order where the reviewed contract fixes them.

---

## R2-C05 — Medium: compiler external-evidence roles disappear in tapscript

The adapter still consumes only:

```rust
requirements.capabilities()
```

and ignores:

```rust
requirements.external_evidence()
```

That is separate from target evidence requirements inferred from capabilities.

The exact compiler-owned external-evidence-role census should survive as an assessed census with an exhaustive mapping. Otherwise a new compiler evidence role can disappear without forcing adapter work.

---

## R2-C06 — Medium–Low: architecture semantic hash predates the general domain-separation law

The architecture semantic hash is currently:

```text
SHA-256(canonical architecture JSON)
```

rather than:

\[
H(D\parallel V\parallel C(P(X)))
\]

The behavioural and deployment hashes do use explicit domain prefixes.

Because this identity is already active and published, it must not be silently changed under the existing algorithm identifier.

Choose explicitly between:

1. a new domain-separated recipe identifier and migration; or
2. a documented grandfathered exception in ADR-016.


---

## R2-C07 — Low: status documentation drift

The package index still marks implemented target packages as planned in places where owning package contracts and the backlog say active.

Also worth correcting:

- root README package layout should mention `compiler`, `target-elements`, and `tapscript`.

These are presentation defects, not semantic identity defects, but correcting them is consistent with the repository’s documentation discipline.

---

# Security-oriented model review

I specifically rechecked the major model transition classes for obvious value or authorization escapes.

## Issuance

I did not find an obvious valid-world bypass of:

- authority consumption;
- exact issued amount;
- destination exhaustion;
- sole issuing operation;
- singleton authority conservation.

`U`, `ENT`, and `DIST_CTL` issuance is checked through both architecture declarations and model flow/issuance validation.

## Redemption

The redemption path pins:

- live receipt class;
- receipt owner authorization;
- exact floored payout;
- exact STATE decrement;
- exact RESV decrement;
- separate sponsor flow;
- exact sealing condition.

The balanced-theft class—short payout plus larger sponsor change—is rejected by branch postconditions rather than sponsor positivity, which is correct.

## Burn and clear

Burn is lateral into fresh ASH and does not directly decrement supply.

Clear:

- authenticates actual ASH values;
- clamps by `Y_L` and `Y−1`;
- destroys exact `U`;
- preserves RESV by nonuse;
- re-emits residual ASH.

The event/value dual-anchor design is sound at the model level.

## Settlement

Settlement preserves the important ordering:

```text
per-entitlement floor
before
physical output aggregation
```

The control/vault continuation and terminal residue cases are independently checked.

I did not find an obvious aggregate-before-floor substitution path in the selected production source.

## Root history

The root replay checks:

- strict order;
- unique transaction IDs;
- created/consumed set integrity;
- active predecessor equality;
- succession/termination membership;
- sealed tombstone behavior;
- final cursor equality.

This remains a strong part of the model.

## Caveat

This conclusion applies to transitions from model-valid worlds under the documented public transition contract. `World` is intentionally mutable and `Transition::apply` has a valid-world precondition. That is documented and is not treated here as a capability-safety claim.

---

# Build and tooling review

## Strong points

- checker report/stamp ordering is explicit;
- nonempty checker stamps are refused nondestructively;
- generator and checker modes are separated;
- generated outputs stage before final rename;
- tracked symlink/gitlink prohibition is centralized;
- output-role identity is lexical rather than falsely inode-secure;
- CI distinguishes “green” from “partial” when lanes are skipped;
- raw argument-supplied child stderr is omitted from first-party diagnostics;
- the repository explicitly disclaims TOCTOU and hostile-filesystem containment.

## Principal tooling concern

The compare-if-changed mode issue in R2-N04 is the main new tooling defect.

Beyond that, I did not find a clear second-pass bypass of the checker stamp contract or the Git mode census in the selected sources.

---

# Recommended remediation plan

## Batch A — target trust-state and target-contract correctness

Close together:

```text
R2-N01
R2-C01
R2-C02
R2-C03
R2-C04
```

This should be the next target-foundation batch.

Suggested objective:

> A target consumed by tapscript is not merely internally well-formed; it is explicitly the reviewed Elements V1 contract, with complete success/failure stack algebra, exact cross-contract welds, and transitively coherent capability status.

### Required deliverables

- reviewed-target wrapper distinct from generic validation;
- success alternatives and retained-operand semantics;
- signature/timelock/issuance/CT/resource welds;
- transitive prerequisite-status validation;
- independently typed encoding interpretation;
- mutation tests for every join.

## Batch B — adapter boundary exactness

Close:

```text
R2-N02
R2-C05
```

Choose whether capability assessment is:

- static-definition-only; or
- deployment-aware.

Then carry compiler external-evidence roles through an exact assessment census.

## Batch C — identity API and publication repair

Close:

```text
R2-N03
R2-N04
R2-C06
```

Deliver:

- validated architecture identity input;
- mode-aware compare-if-changed;
- explicit architecture/anchor digest migration or grandfather policy.

## Batch D — documentation reconciliation

Close `R2-C07` after code contracts are settled, so documentation reflects the actual repaired boundary.

---

# Suggested verification commands after remediation

These are recommendations only; I did not run them.

## Focused target and adapter tests

```sh
cargo fmt --all
cargo test --locked -p tripod-target-elements
cargo test --locked -p tripod-tapscript
cargo test --locked -p tripod-compiler target
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p tripod-target-elements --no-deps
RUSTDOCFLAGS='-D warnings' cargo doc --locked -p tripod-tapscript --no-deps
```

## Publication and architecture tests

```sh
cargo test --locked -p tripod-architecture
cargo test --locked -p cli-common
cargo test --locked -p tripod-artifacts
```

## Workspace cadence

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

## Full batch gate

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

If the batch affects paper inputs or makes a release-oriented claim:

```sh
scripts/check-document-reproducibility.sh
```

Finally:

```sh
git diff --check
git status --porcelain=v1 --untracked-files=all
```

---

# Final verdict

The second review did not overturn the favorable assessment of the core protocol model. The architecture, model, realization, and compiler code show strong ownership discipline and no obvious valid-world monetary exploit in the selected production sources.

The principal risk is now clearer:

> The target foundation currently conflates three different claims: “well-shaped typed target,” “internally coherent target,” and “the reviewed Elements target.” Those are not yet the same thing, but the public wrapper and downstream adapter can treat them as though they are.

Before Guide 9 or a typed instruction core begins consuming these contracts operationally, I recommend treating the following as Phase-3 blockers:

1. complete target success-stack algebra;
2. target cross-contract welds;
3. transitive capability-status closure;
4. independently enforced encoding semantics;
5. a distinct reviewed-target trust state;
6. a deliberate static-versus-deployment adapter boundary.

Once those are repaired, the repository will have a much stronger foundation for target-native conformance: not merely a detailed target declaration, but a type boundary that accurately records what was validated, what was reviewed, what was declared by the deployment, and what still requires evidence.
