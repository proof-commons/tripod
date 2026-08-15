# Static review of `0.3.3-dev`

Overall, this is a remarkably disciplined tree. The separation among architecture, realization, model, compiler, target contract, adapter, and native evidence is unusually explicit. The validation-before-identity wrappers, exact canonical partitions, typed failure vocabularies, deterministic projections, sponsor-value erasure, and extensive non-claim documentation are all strong.

The most important remaining issues are concentrated in the **target-native evidence boundary**. The lower layers are generally careful about making invalid states unconstructible, but the final native-report gate is currently easier to satisfy than its documentation claims.

I reviewed the supplied 266-file concatenation statically. I did **not** execute Cargo, Meson, the native executor, or the document build. Tests, `Cargo.lock`, the paper sources, and several tooling packages were excluded from the supplied content, so this is not a green-build, dependency, licence, advisory, or full-test claim.

## Findings

| ID | Severity | Area | Summary |
|---|---|---|---|
| R5-01 | **High** | Native report gate | `gate()` accepts incomplete or caller-edited reports, including an empty evidence census |
| R5-02 | **High** | Evidence coverage | Broad evidence rows pass when only a subset of the claimed semantics has cases |
| R5-03 | **High** | Evidence binding | Native reports do not bind the exact fixture scripts, contexts, layers, or expected resources |
| R5-04 | **Medium–High** | Deployment binding | Reported network/genesis IDs are caller declarations, not identities observed from the executed chain |
| R5-05 | **Medium** | Executor provenance | The report cannot satisfy ADR-018’s executed-tip/base/topic provenance, and the revision fallback can misattribute a binary |
| R5-06 | **Low** | Executor protocol | Blank protocol lines and trailing blank data are silently accepted despite the documented fail-closed protocol |
| R5-07 | **Low** | Documentation | Several public README/Rustdoc statements still claim the instruction core or target-native evidence is absent |

---

## R5-01 — The native report gate fails open on incomplete reports

**Severity:** High
**Files:**

- `packages/target-elements-conformance/src/validate.rs`
- `packages/target-elements-conformance/src/report.rs`

The public gate is:

```rust
pub fn gate(report: &NativeConformanceReport) -> Result<(), NativeConformanceError> {
    if report.executor.declaration == ExecutorDeclaration::Mock {
        return Err(...);
    }

    for row in &report.evidence {
        if row.plan != EvidencePlanClass::Required {
            continue;
        }

        // Check rows which happen to be present.
    }

    Ok(())
}
```

This validates required rows that are present, but it does not validate that the required rows are present at all.

Consequently, given an otherwise nonmock report:

```rust
report.evidence.clear();
assert!(gate(&report).is_ok());
```

The same problem permits less obvious edits:

- remove a failed required row;
- change a failed row’s `plan` from `Required` to `UnresolvedByDesign`;
- omit all evidence rows;
- provide duplicate rows while omitting another required row;
- alter `summary.completeness`;
- alter the report schema;
- omit or rewrite cases.

None of those are checked by `gate()`.

This is especially important because `NativeConformanceReport` and its components:

- derive `Deserialize`;
- expose public fields;
- can be assembled or edited by any caller;
- are accepted directly by the public `gate()` function.

The CLI’s current path is safer because it calls `evaluate()` and immediately passes that result to `gate()`. But the type and public API claim a stronger reusable boundary than is actually enforced. Any future consumer that reads a report and calls `gate()` can manufacture a false native-evidence pass simply by deleting unfavorable rows.

### Recommended repair

Introduce a validated report state, following the pattern already used successfully elsewhere:

```rust
pub struct ValidatedNativeConformanceReport {
    report: NativeConformanceReport,
}
```

Then either:

1. make `evaluate()` return the validated wrapper; or
2. add a complete `validate_native_report(...)` that returns it.

The validator should independently require:

- supported report schema;
- exact target contract and development binding;
- exact evidence-plan census, once each;
- each row’s plan class equal to the first-party plan;
- exact case census, once each;
- each case bound to its expected fixture;
- recomputed case statuses;
- recomputed evidence dispositions;
- recomputed summary;
- no missing, duplicate, or unexpected rows.

Then:

```rust
pub fn gate(
    report: &ValidatedNativeConformanceReport,
) -> Result<(), NativeConformanceError>;
```

At minimum, the present `gate()` must compare:

\[
\{\text{reported evidence IDs}\} = \{\text{Guide-9 evidence-plan IDs}\}
\]

with duplicate-sensitive equality, rather than iterating only over whatever the caller supplied.

---

## R5-02 — Required evidence rows pass on partial semantic coverage

**Severity:** High
**Files:**

- `packages/target-elements-conformance/src/validate.rs`
- `packages/target-elements-conformance/src/census/introspection.rs`
- `packages/target-elements-conformance/src/census/crypto.rs`
- `packages/target-elements-conformance/src/census/encoding.rs`
- `packages/target-elements-conformance/src/census/context.rs`
- `packages/target-elements-conformance/src/report.rs`

`evidence_rows()` effectively uses this rule:

```text
if any bearing case has infrastructure trouble:
    InfrastructureError
else if any bearing case failed:
    Failed
else if there is at least one bearing case:
    Passed
else if required:
    Failed
else:
    UnresolvedByDesign
```

That establishes only:

> every case that happened to be assigned to this broad row passed.

It does **not** establish:

> every semantic subclaim represented by this evidence requirement was covered.

Several source comments explicitly document missing subclaims, while the broad evidence row can still report `Passed`.

### Concrete example: issuance

`census/introspection.rs` says:

> “Issuance introspection, over inputs that carry none.”

and:

> “The present form is six items and is not stated here: the reviewed executor cannot yet materialize an issuing transaction.”

Nevertheless:

```rust
TargetEvidenceRequirementId::IssuanceIntrospection
```

is `Required`, and the issuance-absent cases bear on that row. If they pass, the entire issuance-introspection row passes, despite no case establishing:

- issuance-present stack shape;
- issued asset amount;
- inflation-keys amount;
- entropy;
- zero/nonzero blinding nonce;
- issuance versus reissuance.

That conflicts with the evidence requirement’s own stated subject and with the Guide-9 acceptance language.

### Concrete example: transaction signatures

`census/crypto.rs` says that accepting cases for transaction-sighash signature primitives cannot be static fixtures. The cases cover:

- empty signature;
- invalid signature;
- invalid key;
- underflow.

But there is no valid transaction-sighash signature case for `CheckSig` or `CheckSigVerify`. The broad `SignatureSemantics` row can nevertheless pass because stack-message signature cases and transaction-signature rejection cases exist.

The report therefore does not distinguish:

```text
transaction signature rejection behavior tested
```

from:

```text
transaction signature complete semantics tested
```

### Concrete example: confidential encodings

The context documentation explicitly records that there are no blinded:

- assets;
- amounts;
- nonces;
- issuing inputs.

Yet broad rows such as:

- `EncodingSemantics`;
- `InputIntrospectionSemantics`;
- `OutputIntrospectionSemantics`;

can pass from explicit-form cases.

A passing row does not establish all alternatives represented by the reviewed contract.

### Concrete example: policy resource limits

`requirements_of(NativeCaseGroup::Resource)` assigns every resource case to both:

```rust
ConsensusResourceLimits
PolicyResourceLimits
```

But the resource cases in `census/encoding.rs` are authored at the consensus layer. The nonminimal-push fixtures shown there state **consensus acceptance**, not relay-policy rejection.

Therefore `PolicyResourceLimits` can pass without a resource-group case actually testing relay policy.

Similarly, `PushEncodingSemantics` includes the claim that minimal push encoding is enforced by relay policy, but the shown nonminimal push cases establish only that consensus accepts them. I did not find a corresponding `NativeCaseGroup::PushEncoding` relay-rejection fixture in the selected production sources.

### Recommended repair

Evidence needs a typed subclaim census below the broad report rows. For example:

```rust
pub enum NativeEvidenceClaim {
    IssuanceAbsent,
    IssuancePresentExplicit,
    IssuancePresentConfidential,
    ReissuancePresent,

    TransactionSignatureAccept,
    TransactionSignatureEmptyReject,
    TransactionSignatureInvalidReject,

    InputValueExplicit,
    InputValueConfidential,
    OutputValueExplicit,
    OutputValueConfidential,

    PushMinimalityConsensusAcceptance,
    PushMinimalityRelayRejection,

    ConsensusElementLimit,
    RelayPolicyLimit,
    // ...
}
```

Each `TargetEvidenceRequirementId` should own an exact required set of such claims. Each fixture should state which claims it exercises. Then require exact coverage:

\[
\operatorname{requiredClaims}(e) \subseteq \bigcup_{c\text{ passed}}\operatorname{claims}(c)
\]

For claims deliberately not covered, the evidence row must remain partial or be split into narrower requirement identities. A row should never say `Passed` merely because it has one passing case.

The current report’s honest residual prose is good; the typed report needs to preserve that same honesty instead of collapsing partial evidence into a passing broad row.

---

## R5-03 — Reports do not bind the exact fixtures that were executed

**Severity:** High
**Files:**

- `packages/target-elements-conformance/src/fixture.rs`
- `packages/target-elements-conformance/src/report.rs`
- `packages/target-elements-conformance/src/validate.rs`

A `PrimitiveFixture` binds the important subject material:

- target contract version;
- network and genesis;
- execution domain;
- leaf version and reviewed status;
- enforcement layer;
- script source;
- exact script bytes;
- initial stack;
- transaction context;
- expected outcome;
- expected resource observations.

But `NativeCaseResult` retains only:

```rust
pub struct NativeCaseResult {
    pub case: NativeCaseId,
    pub expected: ExpectedPrimitiveOutcome,
    pub observed: ObservedNativeOutcome,
    pub status: CaseStatus,
}
```

It omits:

- exact script bytes;
- initial stack;
- transaction context;
- enforcement layer;
- leaf version;
- leaf-version status;
- script source;
- expected resources.

This is not repaired by `NativeCaseId`. Its own documentation says:

> “The ordinal is stable only within the declared fixture set and is not a public semantic identity.”

Thus two different fixture sets may reuse the same case IDs and expected outcomes and produce indistinguishable report rows. In particular, the report cannot show whether a case was asked at:

```text
consensus
```

or:

```text
relay_policy
```

That omission is directly relevant to R5-02.

A future reader cannot determine from the report:

- which exact bytes were executed;
- which exact transaction context was used;
- whether the result was a consensus or relay observation;
- which expected resource figures were compared;
- whether an old fixture definition has become stale.

The report therefore does not fully bind its evidence subject, contrary to the otherwise excellent role/subject discipline in ADR-016.

### Recommended repair

No new digest is necessary.

The simplest repair is to include a canonical fixture projection in each result:

```rust
pub struct NativeCaseResult {
    pub fixture: PrimitiveFixtureProjection,
    pub observed: ObservedNativeOutcome,
    pub status: CaseStatus,
}
```

The projection should contain all evidence-relevant fixture fields, including:

- case ID;
- exact script;
- initial stack;
- context;
- execution domain;
- leaf version and status;
- enforcement layer;
- script source;
- expected outcome;
- expected resources.

If report size becomes a concern, introduce a typed fixture-set identity only when it satisfies ADR-016 admission. Until then, embedding the complete typed subject is preferable to relying on a non-semantic ordinal.

The report should also bind the exact reviewed target definition more strongly than a bare version if the version discipline does not guarantee every semantic contract change bumps it.

---

## R5-04 — The reported development binding is not verified against the chain that executed

**Severity:** Medium–High
**Files:**

- `packages/target-elements-conformance/src/bin/check-target-elements-native.rs`
- `packages/target-elements-conformance/src/fixture.rs`
- `packages/target-elements-conformance/src/executor.rs`
- `packages/target-elements-conformance/src/protocol.rs`
- `scripts/elements-native-executor.py`

The checker constructs a `DevelopmentDeploymentBinding` from caller-supplied:

```text
--network-id
--genesis-id
```

Those values are copied into fixtures and then into the report.

However, the external executor handshake does not report an observed network or genesis identity, and the Python executor’s `parse_fixture()` validates that the fields exist but discards them from the returned fixture representation.

The Python adapter then boots whichever chain is configured externally:

```text
-chain=$ELEMENTS_NATIVE_EXECUTOR_CHAIN
```

with no comparison between:

- the binding’s `network_id`;
- the binding’s `genesis_id`;
- the actual chain;
- the actual chain’s genesis block.

Therefore the same execution can be reported under arbitrary nonzero network and genesis identifiers. The type says these identify the development environment, but mechanically they are only caller declarations.

The backlog even records synthetic identifiers (`32×09` and `32×07`), which confirms that the report fields are not actual observed chain identities.

### Why this matters

The report claims to be:

> “What one executor observed, for one exact reviewed contract and one development binding.”

It currently observes the contract behavior, but not the binding identity.

### Recommended repair

Extend the handshake or a distinct environment-observation message with typed values such as:

```rust
pub struct ExecutorEnvironmentObservation {
    pub environment: WireEnvironment,
    pub chain_name: String,
    pub network_id: [u8; 32],
    pub genesis_id: [u8; 32],
    pub leaf_version_active: bool,
}
```

The harness should compare the observed identifiers with the validated binding **before executing any fixture**.

For the Python adapter:

- derive genesis from the node, e.g. `getblockhash 0`;
- derive or explicitly define the development network identifier;
- report those values;
- refuse any fixture whose binding differs.

If synthetic IDs are intentionally run labels rather than network identities, rename the fields accordingly. They should not inhabit types named `network_id` and `genesis_id`.

---

## R5-05 — Executor provenance cannot satisfy ADR-018 and may misattribute the binary

**Severity:** Medium
**Files:**

- `adr/018-upstream-elements-workspace.md`
- `packages/target-elements-conformance/src/protocol.rs`
- `packages/target-elements-conformance/src/report.rs`
- `scripts/elements-native-executor.py`

ADR-018 requires an evidence record to state:

- executed tip;
- upstream base;
- census of included local branches.

The handshake/report currently carry only:

```rust
implementation_name
implementation_version
upstream_revision: Option<String>
```

They have no typed fields for:

- upstream base;
- included fix branches;
- notes branch;
- merged integration tip;
- whether the binary revision matches the intended tip.

There is also a provenance hazard in `implementation_provenance()`:

```python
if revision is None and upstream_repo is not None:
    revision = git -C upstream_repo rev-parse HEAD
```

A checkout’s current `HEAD` is not evidence of which source produced a previously built binary. The module documentation acknowledges this distinction, but still fills the binary’s `upstream_revision` field from the checkout fallback.

That can attribute an unrelated or later source revision to the executable.

### Recommended repair

Separate the concepts in the protocol:

```rust
pub struct ExecutorProvenance {
    pub adapter_name: String,
    pub adapter_version: String,
    pub framework_revision: Option<String>,

    pub node_name: String,
    pub node_version: String,
    pub binary_reported_revision: Option<String>,

    pub executed_tip: Option<String>,
    pub upstream_base: Option<String>,
    pub included_local_topics: BTreeSet<String>,
}
```

Then:

- never fill `binary_reported_revision` from a checkout;
- record checkout `HEAD` only as `review_checkout_revision` or `intended_tip`;
- require the binary-reported revision to match the intended executed tip for evidence;
- record ADR-018’s upstream base and local branch census explicitly;
- keep adapter/framework provenance distinct from node provenance.

The current handshake identifies the node but not the Python adapter and functional-test framework that materially construct the tested transactions.

---

## R5-06 — Blank protocol lines are silently accepted

**Severity:** Low
**Files:**

- `packages/target-elements-conformance/src/executor.rs`
- `packages/target-elements-conformance/src/bin/mock-native-executor.rs`

The protocol documentation says:

- one complete JSON object per line;
- malformed lines fail;
- any data after the final response fails.

But `read_line()` loops over blank lines:

```rust
loop {
    let mut line = String::new();
    let read = reader.read_line(&mut line)?;
    if read == 0 {
        return Ok(None);
    }
    if !line.trim().is_empty() {
        return Ok(Some(line));
    }
}
```

As a result:

- blank lines during the exchange are silently ignored;
- trailing blank lines after the final response are silently accepted.

That is more permissive than the stated protocol. It is small, but protocol strictness is specifically part of this package’s assurance claim.

### Recommended repair

Choose one contract and implement it consistently:

1. **Strict NDJSON:** every non-EOF line must contain a JSON object; blank lines are malformed, and any line after the final response is trailing data.
2. **Whitespace-permitted framing:** document blank lines as permitted framing.

Given the package’s fail-closed design, strict NDJSON is the more consistent choice.

---

## R5-07 — Public documentation has stale implementation-state claims

**Severity:** Low
**Files:**

- `README.md`
- `packages/target-elements/README.md`
- `packages/target-elements/src/lib.rs`
- `packages/target-elements/src/evidence.rs`

The root README still says:

```text
packages/tapscript/
    ... capability adapter only; no instruction core.
```

But the package now contains:

- typed instructions;
- stack items;
- serializer;
- parser;
- abstract stack validator.

The root layout also omits `packages/target-elements-conformance/`.

Meanwhile `packages/target-elements/README.md` says:

> “Target-native deployment evidence has not been produced”

and the crate documentation says:

> “No node has been asked anything.”

Those statements conflict with the Guide-9 gate record and target reference, which claim a 398-case native run.

The static target crate is correct not to carry mutable evidence status. The documentation should say:

```text
This crate carries requirements only and consumes no completion status.
Development native evidence is produced by target-elements-conformance
and recorded separately.
Production evidence remains absent.
```

That preserves the important package boundary without making a stale repository-wide factual claim.

---

# Additional observations

## Strong points

A few things deserve explicit praise:

1. **Validation before identity is now type-enforced.**
   `ValidatedDraftArchitecture`, `ValidatedReleaseArchitecture`, and `ValidatedDeploymentProfile` are the right pattern.

2. **The model’s exact canonical partition is well-designed.**
   The move from aggregate conservation to source/destination-exact flows closes a large class of offsetting errors.

3. **Sponsor opacity is handled structurally.**
   `ObservedValue::SponsorOpaque`, protocol/sponsor flow-role separation, and compiler-level read rejection are much stronger than a diagnostic-only privacy convention.

4. **Model execution and realization conformance are separated.**
   `ExecutedTransition<T>` and post-execution observation avoid a circular “the model asks the specification whether it should accept” design.

5. **The compiler is unusually honest about partiality.**
   Per-operation factorization, explicit architecture-scope status, lifecycle incompleteness, unresolved evidence, and absence of speculative identities are all good boundaries.

6. **Target failure semantics are represented rather than flattened.**
   Abort, consume-and-false, and retain-and-false remain distinct through the target contract and abstract stack validator.

7. **The static target/reviewed target distinction is correct.**
   `ValidatedTargetDefinition` versus `ReviewedElementsTapscriptDefinition` is exactly the kind of trust-state separation the native report should emulate.

8. **The repository’s non-claims are unusually precise.**
   It is clear throughout that architecture/model success is not production deployment readiness.

## Broader architectural recommendation

The evidence package would benefit from applying its neighboring packages’ strongest pattern:

```text
raw declaration/report
    ↓
complete owner validation
    ↓
validated typed wrapper
    ↓
gate or identity consumer
```

At present the target contract, architecture, realization, and compiler largely follow this law, while `NativeConformanceReport` remains a public mutable DTO accepted directly by a partial validator.

That asymmetry is the central issue in this review.

---

# Suggested repair order

I would address the findings in this order:

1. **R5-01:** make the report gate exact and wrapper-based;
2. **R5-02:** add typed claim-level evidence coverage;
3. **R5-03:** bind complete fixture projections into report rows;
4. **R5-04:** bind the report to the actually executed chain;
5. **R5-05:** upgrade executor/node/framework provenance for ADR-018;
6. **R5-06:** make protocol whitespace behavior match its contract;
7. **R5-07:** reconcile public status documentation.

The first three should land together if possible: validating the report exactly is much easier once the report contains the exact fixture subject and claim-level coverage it is supposed to validate.

## Bottom line

The core semantic architecture looks thoughtful and substantially hardened. I did not find an obvious valid-world path in the selected production code for unauthorized issuance, reserve extraction, receipt-class escape, or recipient redirection.

The main concern is narrower but important: **the native evidence layer can currently say “passed” without proving that the complete claimed subject was present and covered.** Fixing the report trust state, exact evidence census, and fixture binding would bring that layer up to the standard already established by the rest of the repository.

# Full second static review

**Tree:** `0.3.3-dev`
**Scope supplied:** 266 files, approximately 3.58 MiB
**Method:** independent static pass over the supplied production code, policy, build wiring, scripts, and planning material.

I did **not** run Cargo, Meson, TeX, the native executor, or Git-dependent gates. The supplied filter excluded most tests, `Cargo.lock`, the paper sources, `cli-common`, `labels`, `artifacts`, `execwrap`, `document-stamps`, and some generated files. Consequently, this review is not a current green-build, dependency-resolution, licence, advisory, test-coverage, or reproducibility claim.

## Executive assessment

The core architecture remains strong. In particular, I did not find an obvious valid-world route in the selected model code for:

- unauthorized closed-asset issuance;
- unauthorized reserve extraction;
- live/time-locked receipt-class escape;
- sponsor-funded payout substitution;
- recipient redirection;
- post-sealing pool revival;
- history residue influencing monetary or attestation computation.

The second pass confirms that the most consequential risks are concentrated in three boundaries:

1. **native-evidence completeness and report validation;**
2. **the static target’s signature abstraction;**
3. **typed bindings that retain only a version instead of the exact contract they were validated against.**

This pass found several issues beyond the first review. The strongest new finding is that the abstract tapscript validator cannot faithfully represent the target’s documented empty-signature and unknown-key-type behavior.

---

# Consolidated findings

| ID | Severity | Status in this review | Summary |
|---|---:|---|---|
| SR5-01 | **High** | Confirmed | The public native-report gate validates only required rows that happen to be present |
| SR5-02 | **High** | Confirmed and strengthened | Broad evidence rows pass when only a subset of their semantic claim has native cases |
| SR5-03 | **High** | Confirmed | Native reports do not bind the exact fixtures, contexts, enforcement layers, or expected resources |
| SR5-04 | **High** | **New** | A development binding can be validated against one target definition and combined with another definition of the same version |
| SR5-05 | **High** | **New** | The static signature model cannot represent empty-signature and unknown-key-type behavior faithfully |
| SR5-06 | **Medium–High** | Confirmed | Report network/genesis fields are declarations, not identities observed from the executed chain |
| SR5-07 | **Medium** | **New** | Relation IDs are not generically welded to relation bodies; `ExpressionPredicate` has no corresponding `RelationKind` |
| SR5-08 | **Medium** | Confirmed | Native executor provenance cannot express ADR-018’s executed tip, upstream base, and local-topic census |
| SR5-09 | **Medium** | **New** | Executor timeout kills only the immediate child and can leak the real adapter’s node and temporary directory |
| SR5-10 | **Medium** | **New** | Executor protocol messages have no input-size bound and can exhaust memory before typed rejection |
| SR5-11 | **Medium** | **New** | Schema-2 deployment profiles can acquire a type named “validated deployment release” despite a documented production-blocking ABI gap |
| SR5-12 | **Low** | Confirmed | Blank protocol lines and trailing blank data are silently accepted |
| SR5-13 | **Low** | Confirmed | Public repository documentation contains stale implementation and evidence-status claims |

---

# SR5-01 — Native report gating is not census-complete

**Severity:** High
**Files:**

- `packages/target-elements-conformance/src/report.rs`
- `packages/target-elements-conformance/src/validate.rs`

The public gate accepts a mutable, deserializable `NativeConformanceReport` directly:

```rust
pub fn gate(report: &NativeConformanceReport) -> Result<(), NativeConformanceError>
```

Its substantive logic is:

```rust
for row in &report.evidence {
    if row.plan != EvidencePlanClass::Required {
        continue;
    }

    // Validate the required row which happens to be present.
}

Ok(())
```

It does not prove that:

- every Guide-9 evidence row is present;
- each row occurs exactly once;
- each row has its first-party plan classification;
- the exact case census is present;
- case statuses were correctly recomputed;
- evidence dispositions were correctly recomputed;
- summary counts and completeness were correctly recomputed;
- the report schema is supported.

Because report fields are public, a caller can conceptually do this:

```rust
let mut report = previously_failed_report;
report.evidence.clear();
assert!(gate(&report).is_ok());
```

Likewise, a caller may:

- remove a failed required row;
- change its plan class from `Required` to `UnresolvedByDesign`;
- duplicate a passing row and omit another;
- clear the cases;
- set an arbitrary summary;
- alter the report schema.

### Scope refinement

The current checker command is less exposed than the public API because it calls:

```text
evaluate → gate → publish
```

in one process, and `evaluate()` constructs the evidence rows from the first-party plan. Thus this is not an immediate command-line bypass by editing a report file after publication.

It is nevertheless a serious typed-boundary defect:

- `gate()` is public;
- the report is publicly constructible and mutable;
- a future release/report consumer can naturally deserialize a report and call `gate()`;
- the function name and documentation imply complete gate validation.

### Repair

Follow the validation-before-consumption pattern used elsewhere:

```rust
pub struct ValidatedNativeConformanceReport {
    report: NativeConformanceReport,
}
```

Add a complete owner validator:

```rust
pub fn validate_native_report(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ValidatedDevelopmentBinding,
    fixtures: &PrimitiveFixtureSet,
    plan: &EvidencePlan,
    report: NativeConformanceReport,
) -> Result<ValidatedNativeConformanceReport, Vec<NativeConformanceError>>;
```

Require duplicate-sensitive exact equalities:

\[
\operatorname{reportedEvidenceIds}=\operatorname{Guide9EvidenceIds}
\]

and

\[
\operatorname{reportedCaseIds}=\operatorname{fixtureCaseIds}.
\]

The validator should independently recompute:

- per-case status;
- evidence disposition;
- summary counts;
- completeness.

Then gate only the validated wrapper.

At minimum, the current gate must reject:

- missing evidence rows;
- duplicate evidence rows;
- unexpected evidence rows;
- a row whose `plan` differs from `guide_nine_evidence_plan()`;
- a failed summary;
- failed or infrastructure-error cases;
- unsupported report schemas.

---

# SR5-02 — Broad evidence rows pass on partial semantic coverage

**Severity:** High
**Files:**

- `packages/target-elements-conformance/src/validate.rs`
- `packages/target-elements-conformance/src/census/crypto.rs`
- `packages/target-elements-conformance/src/census/encoding.rs`
- `packages/target-elements-conformance/src/census/introspection.rs`
- `packages/target-elements-conformance/src/census/context.rs`

The current evidence aggregation proves:

> Every authored case assigned to this broad evidence row passed.

It does not prove:

> Every semantic subclaim represented by this evidence requirement had a case.

The aggregation effectively follows:

```text
any infrastructure-error case → InfrastructureError
else any failed case          → Failed
else at least one case        → Passed
else required row             → Failed
else                          → UnresolvedByDesign
```

That is existential at the row level. One passing case is sufficient to move a broad row to `Passed`.

## Concrete instance: issuance-present semantics are absent

The issuance census explicitly says:

> “Issuance introspection, over inputs that carry none.”

and:

> “The present form is six items and is not stated here.”

The implemented cases establish the **absence form** of `InspectInputIssuance`. They do not establish:

- issuance-present stack shape;
- issued asset amount;
- inflation-keys amount;
- entropy;
- zero versus nonzero blinding nonce;
- issuance versus reissuance.

Nevertheless, `IssuanceIntrospection` is required and receives cases from the issuance group. The absent-form cases can make the whole evidence row pass.

That report status is broader than the native evidence.

## Concrete instance: transaction-sighash signature acceptance is absent

The signature census explicitly records that valid transaction-sighash signatures cannot be static fixtures in the current design.

The transaction-signature cases cover:

- empty signatures;
- invalid signatures;
- invalid keys;
- underflow.

They do not cover an accepting transaction signature for:

- `CheckSig`;
- `CheckSigVerify`.

Stack-message signature fixtures provide valid signatures, but that is a different primitive and a different message source.

The broad `SignatureSemantics` row can therefore pass without establishing successful transaction-sighash verification.

## Concrete instance: confidential field alternatives are absent

The transaction context documentation records the absence of:

- blinded assets;
- blinded values;
- blinded nonces;
- issuing inputs.

Yet broad rows such as:

- `EncodingSemantics`;
- `InputIntrospectionSemantics`;
- `OutputIntrospectionSemantics`;

can pass using explicit-only cases.

This matters because the static target contract publishes distinct success alternatives for explicit and confidential encodings.

## Concrete instance: relay-policy resource evidence is not established

`requirements_of(NativeCaseGroup::Resource)` assigns resource cases to both:

```rust
ConsensusResourceLimits
PolicyResourceLimits
```

But the resource fixtures in `census/encoding.rs` are authored at the consensus layer.

The nonminimal-push examples there state:

> consensus accepts this nonminimal form.

That does not establish:

> relay policy refuses this nonminimal form.

Similarly, relay-layer nonminimal-number cases are grouped as conversion or timelock evidence, not as policy-resource evidence.

Thus `PolicyResourceLimits` may report `Passed` without a policy-layer resource case.

## Concrete instance: push minimality

The target’s push contract says:

- truncation and oversize are consensus rules;
- nonminimal push form is a relay rule.

`PushEncodingSemantics` gets the push-encoding group. The shown nonminimal push fixtures are in the resource group and state consensus acceptance. I did not find a `PushEncoding` case that asks relay policy to refuse the nonminimal form.

A passing push-encoding row therefore does not establish the complete typed push contract.

### Repair

Introduce a claim-level evidence census beneath the broad requirement IDs:

```rust
pub enum NativeEvidenceClaim {
    IssuanceAbsent,
    IssuancePresentExplicit,
    IssuancePresentConfidential,
    ReissuancePresent,

    TransactionSignatureAccept,
    TransactionSignatureEmptyFailure,
    TransactionSignatureInvalidFailure,
    StackMessageSignatureAccept,

    InputValueExplicit,
    InputValueConfidential,
    OutputValueExplicit,
    OutputValueConfidential,

    PushMinimalConsensusAccept,
    PushMinimalRelayReject,

    ConsensusElementLimit,
    PolicyTransactionWeightLimit,
    // ...
}
```

Each evidence requirement owns an exact set of required claims:

```rust
fn required_claims(
    requirement: TargetEvidenceRequirementId,
) -> BTreeSet<NativeEvidenceClaim>;
```

Each fixture states the claims it exercises. A row passes only when:

\[
\operatorname{requiredClaims}(e)\subseteq
\bigcup_{\substack{c\text{ passed}\\c\text{ bears on }e}}\operatorname{claims}(c).
\]

Claims deliberately outside the present census must remain unresolved. Do not collapse them into a broad passing row.

This also provides a clean place to state the current honest boundary:

```text
issuance absent:
    passed

issuance present:
    unresolved

transaction-sighash rejection behavior:
    passed

transaction-sighash successful verification:
    unresolved
```

---

# SR5-03 — Native reports do not bind their exact fixture subjects

**Severity:** High
**Files:**

- `packages/target-elements-conformance/src/fixture.rs`
- `packages/target-elements-conformance/src/report.rs`
- `packages/target-elements-conformance/src/validate.rs`

`PrimitiveFixture` correctly carries the full evidence subject:

- exact script bytes;
- exact initial stack;
- complete transaction context;
- enforcement layer;
- leaf version;
- leaf-version status;
- script source;
- expected outcome;
- expected resource observations;
- target version;
- development identifiers.

But `NativeCaseResult` retains only:

```rust
pub struct NativeCaseResult {
    pub case: NativeCaseId,
    pub expected: ExpectedPrimitiveOutcome,
    pub observed: ObservedNativeOutcome,
    pub status: CaseStatus,
}
```

It drops:

- exact script;
- initial stack;
- transaction context;
- consensus versus relay enforcement layer;
- leaf version and reviewed status;
- script source;
- expected resource values.

`NativeCaseId` cannot carry this binding. Its documentation explicitly says:

> “The ordinal is stable only within the declared fixture set and is not a public semantic identity.”

Therefore two different fixture sets can reuse the same IDs and expected outcomes while producing indistinguishable report rows.

A report consumer cannot tell:

- what bytes ran;
- which input stack ran;
- what transaction fields were materialized;
- whether the verdict was a consensus or relay verdict;
- which resource figures were exact expectations;
- whether a fixture definition changed after the report was produced.

This is especially consequential for SR5-02: the report does not retain the enforcement layer needed to audit whether a policy claim was actually exercised.

### Repair

No hash is required.

Embed a complete canonical fixture projection:

```rust
pub struct NativeCaseResult {
    pub fixture: PrimitiveFixtureProjection,
    pub observed: ObservedNativeOutcome,
    pub status: CaseStatus,
}
```

The projection should retain every evidence-relevant field.

If a future report-size concern justifies a fixture-set identity, admit that identity under ADR-016 with:

- exact typed subject;
- immediate consumer;
- decision;
- canonical encoding;
- stale conditions;
- migration.

Until that consumer exists, embedding the typed fixture is the stronger and simpler boundary.

---

# SR5-04 — A validated deployment binding is not bound to the exact target definition that validated it

**Severity:** High
**Files:**

- `packages/target-elements/src/deployment.rs`
- `packages/target-elements/src/definition.rs`
- `packages/target-elements-conformance/src/fixture.rs`

`validate_development_binding()` validates a binding against a particular `ValidatedTargetDefinition`, but the returned wrapper stores only:

```rust
pub struct ValidatedDevelopmentBinding {
    binding: DevelopmentDeploymentBinding,
}
```

The binding contains:

- target contract version;
- environment;
- network/genesis;
- activation declaration;
- resource overrides.

It does **not** contain or retain the target definition or its projection.

Later, `bind_development_target()` combines the binding with a target definition by checking only:

```rust
if deployment.binding().target_version() != definition.definition().version() {
    return Err(...);
}
```

Thus this sequence is permitted in principle:

```text
1. Construct generic validated target A, version V1.
2. Validate binding B against A.
3. Obtain another validated target C, also version V1.
4. bind_development_target(C, B).
5. Combination succeeds without proving A = C.
```

The semantic difference can matter. For example:

- A may mark a required capability reviewed;
- C may mark it unsupported or incomplete;
- B was accepted because A supported it;
- the final combination contains C.

The same issue affects:

```rust
PrimitiveFixture::state(
    target: &ReviewedElementsTapscriptDefinition,
    binding: &ValidatedDevelopmentBinding,
    ...
)
```

The reviewed target and binding are accepted separately; no exact contract binding is checked.

### Why the version check is insufficient

The repository explicitly distinguishes:

```text
internally coherent target
≠
reviewed Elements target
```

because many internally coherent definitions may share one contract version. A version number therefore cannot stand in for exact typed contract equality.

### Repair

Either retain the validated target projection in the binding:

```rust
pub struct ValidatedDevelopmentBinding {
    binding: DevelopmentDeploymentBinding,
    target: TargetProjection,
}
```

and compare it exactly when combining, or use trust-state-specific wrappers:

```rust
pub struct ReviewedDevelopmentBinding {
    binding: ValidatedDevelopmentBinding,
}
```

constructed only by:

```rust
validate_reviewed_development_binding(
    target: &ReviewedElementsTapscriptDefinition,
    binding: DevelopmentDeploymentBinding,
)
```

Then require the reviewed binding for native fixtures and native reports.

The generic `ElementsTarget` combination must prove exact typed equality with the target against which the binding was validated, not merely version equality.

---

# SR5-05 — Signature behavior is not representable faithfully in the abstract target model

**Severity:** High
**Files:**

- `packages/target-elements/src/authorization.rs`
- `packages/target-elements/src/opcode.rs`
- `packages/target-elements/src/weld.rs`
- `packages/tapscript/src/stack.rs`
- `packages/target-elements-conformance/src/census/crypto.rs`

This is the strongest new finding.

The typed contract correctly documents two important target behaviors:

1. an empty signature takes a distinct failure path;
2. an unknown public-key type succeeds without verification for a nonempty signature.

But the opcode operand types cannot represent either behavior accurately.

## Empty signatures

`CheckSig` declares its signature operand as:

```rust
StackValueType::Encoded(EncodingClass::SchnorrSignature)
```

and `SchnorrSignature` has exact width 64.

Its failure contract also declares:

```rust
EmptySignature
    → ConsumeOperandsPushFalse
```

The abstract stack validator checks operand compatibility before applying failure effects.

If the incoming item is:

```rust
StackValueType::Empty
```

then it does not satisfy an exact 64-byte encoded signature, and validation returns:

```text
StackTypeMismatch
```

before the empty-signature failure can be represented.

If the caller instead labels the item:

```rust
StackValueType::Encoded(SchnorrSignature)
```

then the validator treats it as simultaneously capable of:

- successful signature verification;
- empty-signature failure;
- invalid-signature failure;

even though the exact-width encoding excludes emptiness.

Thus neither abstract representation is faithful:

```text
Empty
    rejected too early

Encoded(SchnorrSignature)
    includes impossible empty behavior
```

The documented empty-signature branch is not exactly representable.

## Unknown public-key types

`SignaturePrimitiveContract` records:

```rust
UnknownPublicKeyTypeRule::SucceedsWithoutVerification
```

But signature opcodes declare their public-key operand as:

```rust
StackValueType::Encoded(EncodingClass::XOnlyPublicKey)
```

which admits only the reviewed 32-byte x-only form.

An unknown nonempty key type is therefore outside the opcode operand domain. The abstract validator rejects it as a type mismatch rather than representing the target’s documented “succeeds without verification” path.

The cross-contract signature weld checks that the recognized key/signature encodings appear. It does not prove that the unknown-key behavior is represented by the opcode stack relation.

## Native coverage also misses the sharp edge

The native signature census covers:

- valid x-only keys;
- absent keys;
- invalid signatures;
- empty signatures;
- underflow.

I did not find a fixture for:

```text
unknown nonempty public-key encoding
+
nonempty signature
→ succeeds without verification
```

Yet `SignatureSemantics` can pass.

This is both:

- a static-contract completeness defect;
- another instance of SR5-02’s broad evidence-row problem.

### Repair

The operand contract needs alternatives, not one exact value type. For example:

```rust
pub enum OperandContract {
    Exact(StackValueType),
    OneOf(BTreeSet<StackValueType>),
    Signature {
        nonempty_encoding: EncodingClass,
        empty_allowed: bool,
    },
    PublicKey {
        recognized: EncodingClass,
        unknown_nonempty_allowed: bool,
    },
}
```

Failure effects also need trigger conditions or typed cases:

```rust
pub struct FailureCase {
    pub condition: FailureCondition,
    pub cause: FailureCause,
    pub effect: FailureOutcome,
}
```

Possible conditions:

```rust
SignatureEmpty
SignatureNonemptyInvalid
PublicKeyEmpty
PublicKeyRecognized
PublicKeyUnknownNonempty
```

Then the abstract validator can behave exactly:

```text
empty signature:
    no success path
    consume operands
    push false

valid recognized signature:
    success path

nonempty invalid recognized signature:
    abort

unknown nonempty key type:
    target’s documented forward-compatible path
```

Add independent native fixtures for the unknown-key behavior before the signature evidence row can be complete.

---

# SR5-06 — The report’s network and genesis IDs are not observations of the chain that executed

**Severity:** Medium–High
**Files:**

- `packages/target-elements-conformance/src/bin/check-target-elements-native.rs`
- `packages/target-elements-conformance/src/protocol.rs`
- `packages/target-elements-conformance/src/report.rs`
- `scripts/elements-native-executor.py`

The checker receives:

```text
--network-id
--genesis-id
```

and uses them to build the development binding. Those identifiers then flow into fixtures and the final report.

The executor handshake does not report:

- observed network ID;
- observed genesis ID;
- observed activation state.

The Python executor validates that the fixture fields exist, but its parsed fixture discards the fixture’s network and genesis fields. It then boots whichever chain is externally selected by its launcher.

No comparison establishes:

\[
\operatorname{reportedGenesis}
=
\operatorname{executedChainGenesis}.
\]

The same target execution can therefore be labeled with arbitrary nonzero network and genesis values.

The backlog’s recorded synthetic IDs further indicate that these fields are being used as run labels rather than observed chain identities.

### Repair

Extend the protocol with an environment observation:

```rust
pub struct ExecutorEnvironmentObservation {
    pub environment: WireEnvironment,
    pub chain_name: String,
    pub network_id: [u8; 32],
    pub genesis_id: [u8; 32],
    pub reviewed_leaf_active: bool,
}
```

The harness must compare it with the validated binding before executing cases.

For the real adapter:

- derive genesis from the node;
- derive or formally define the network identity;
- report activation/configuration observations;
- reject a mismatch.

If the existing synthetic values are intentionally test-run identifiers, rename them. They should not inhabit fields named `network_id` and `genesis_id`.

---

# SR5-07 — Relation identity is not generically welded to relation meaning

**Severity:** Medium
**Files:**

- `packages/realization/src/identity.rs`
- `packages/realization/src/relation.rs`
- `packages/realization/src/validate.rs`

`RelationId` is presented as the stable semantic identity of a relation:

```rust
pub struct RelationId {
    operation: OperationId,
    kind: RelationKind,
    subject: RelationSubject,
}
```

But I did not find a generic validator proving that:

```text
RelationId.kind
RelationId.subject
```

agree with:

```text
RelationDeclaration.relation
```

for every relation variant.

Current declarations use helper constructors which generally pair them correctly, but helper correctness is not owner validation.

## Direct vocabulary mismatch

`RelationKind` contains:

```rust
ClassClosure
```

but `Relation` has no `ClassClosure` variant.

Conversely, `Relation` contains:

```rust
ExpressionPredicate { expression }
```

but `RelationKind` has no `ExpressionPredicate` member.

Therefore a future expression-predicate relation cannot be given a relation kind that honestly describes its body.

## Consequences

A same-operation relation can, in principle, carry:

- an ID saying one relation family;
- a body implementing another family.

Compiler and coverage structures key by `RelationId` while behavior classification matches the body. That permits two answers to:

> What relation is this?

The architecture-family validator catches some expected pilot cardinality and recognition mismatches, but it is not a general relation-ID/body weld and does not reject every surplus relation.

### Repair

Add an exhaustive owner validator:

```rust
fn validate_relation_identity(
    declaration: &RelationDeclaration,
) -> Result<(), RealizationError>;
```

It should derive the expected `RelationKind` and compatible `RelationSubject` from every `Relation` variant and compare them with the ID.

Add `RelationKind::ExpressionPredicate`, or redesign the kind vocabulary before expression-bearing production scope begins.

Either remove `ClassClosure` until there is a matching semantic relation or add the exact intended body variant.

Also make architecture-owned family relation validation bidirectional:

\[
\operatorname{declaredArchitectureFamilyRelations}
=
\operatorname{expectedArchitectureFamilyRelations}.
\]

Today it proves the expected rows are present, but not generically that no architecture-family row exists for an undeclared operation family.

No public realization digest exists, so this is a good time to correct the key vocabulary without identity migration.

---

# SR5-08 — Native provenance is weaker than ADR-018

**Severity:** Medium
**Files:**

- `adr/018-upstream-elements-workspace.md`
- `packages/target-elements-conformance/src/protocol.rs`
- `packages/target-elements-conformance/src/report.rs`
- `scripts/elements-native-executor.py`

ADR-018 requires evidence records to state:

- executed tip;
- upstream base;
- census of included local branches.

The native handshake/report carries only:

```rust
implementation_name
implementation_version
upstream_revision
```

It cannot express:

- the integration tip;
- the upstream merge base;
- included `fix/` branches;
- the notes branch;
- the exact derived `merged` state;
- whether the binary revision matches the intended integration tip.

There is also a possible misattribution path:

```python
if binary embeds no revision:
    upstream_revision = git -C upstream_repo rev-parse HEAD
```

A checkout’s `HEAD` does not identify a previously built binary. It is review/intended-source provenance, not binary provenance.

### Repair

Separate provenance fields:

```rust
pub struct ExecutorProvenance {
    pub adapter_name: String,
    pub adapter_version: String,
    pub framework_revision: Option<String>,

    pub node_name: String,
    pub node_version: String,
    pub binary_reported_revision: Option<String>,

    pub intended_executed_tip: Option<String>,
    pub upstream_base: Option<String>,
    pub included_local_topics: BTreeSet<String>,
}
```

Never populate `binary_reported_revision` from a checkout fallback.

Require the binary-reported revision to agree with the intended tip where the node supports that evidence.

Also record the Python adapter and functional-test framework separately from the node. They materially construct the transactions under test.

---

# SR5-09 — Timeout termination does not clean up the real executor’s descendants

**Severity:** Medium
**Files:**

- `packages/target-elements-conformance/src/executor.rs`
- `scripts/elements-native-executor.py`
- `scripts/elements-native-executor.sh`

The watchdog expires by calling:

```rust
child.kill()
```

On Unix this kills the immediate executor process forcefully. It does not:

- terminate the executor’s process group;
- terminate descendants;
- allow the Python adapter’s `finally` cleanup to run;
- guarantee that the executor’s stdout pipe closes if a descendant inherited it.

The real Python adapter starts `elementsd` as a child and cleans it up in:

```python
finally:
    close()
```

A forceful kill of Python bypasses that `finally`. The result may be:

- orphaned `elementsd`;
- retained disposable datadir and RPC cookie;
- retained open file descriptors;
- port/resource leakage across later runs.

The cookie is disposable test material, not a production secret, but the cleanup and timeout contract still claims the environment is destroyed.

For an arbitrary caller-selected executor, a descendant may inherit stdout. Killing only the direct child can leave the Rust reader blocked waiting for EOF despite the timeout.

### Repair

Supervise the complete executor process tree.

A POSIX implementation can:

1. start the executor in a new process group;
2. on timeout send a graceful termination to the group;
3. wait a bounded cleanup interval;
4. kill the group if still alive;
5. reap the direct child;
6. close protocol pipes.

The real launcher could additionally establish a parent-death signal or run the node under a dedicated supervisor.

Add a subprocess regression where an executor:

- forks a descendant;
- keeps stdout open;
- ignores graceful termination;
- outlives the parent unless the complete process group is terminated.

This is not a sandbox claim. It is enforcement of the harness’s own timeout and cleanup contract.

---

# SR5-10 — Executor protocol framing has no size limit

**Severity:** Medium
**Files:**

- `packages/target-elements-conformance/src/executor.rs`

Protocol input is read with:

```rust
BufRead::read_line
```

into an unbounded `String`.

A malformed or buggy executor can write an arbitrarily large line without a newline. The harness will continue allocating until:

- memory is exhausted;
- the OS kills it;
- the watchdog eventually kills the child, potentially after the harness has already failed from memory pressure.

That bypasses the typed malformed-message result. The package documents a fail-closed protocol, but no bound exists on:

- handshake line size;
- response line size;
- trailing-data line size.

### Repair

Define explicit protocol limits, for example:

```rust
pub struct ProtocolLimits {
    pub maximum_handshake_bytes: usize,
    pub maximum_response_bytes: usize,
    pub maximum_trailing_line_bytes: usize,
}
```

Read with a bounded buffer or `take(maximum + 1)`, and reject before deserializing if the line exceeds the bound.

The bound should account for the largest possible fixture-independent response and be tested at:

- exactly the maximum;
- maximum + 1;
- no newline;
- deeply nested or very long strings within the byte limit.

A bounded protocol message is a correctness property, not hostile-code containment.

---

# SR5-11 — The deployment “release” validator can validate a profile known to be production-incomplete

**Severity:** Medium
**Files:**

- `packages/architecture/src/deployment.rs`
- `packages/architecture/src/lib.rs`
- `adr/016-semantic-identities-and-evidence-binding.md`

The deployment profile documentation explicitly says schema 2 cannot bind the final transaction ABI/configuration used by calibration:

> “A future profile schema must add that binding before any production release.”

Nevertheless, the public API is named:

```rust
validate_deployment_release
validate_deployment_profile
ValidatedDeploymentProfile
```

and accepts a final schema-2 profile if the currently represented fields pass.

The documentation around `validate_deployment_release()` also says:

> “a deployment is release-ready only when this validation passes”

which conflicts with the schema-2 residual saying the result is insufficient for production release.

The identity is dormant and no release consumer currently exists, so this is not an active production bypass. It is a trust-state naming defect waiting at the release boundary.

### Repair

Until schema 3 or another reviewed migration binds the final ABI:

- rename the wrapper to something like `ValidatedPreReleaseDeploymentProfile`; or
- make `validate_deployment_release()` reject schema 2 with a typed `ProductionReleaseUnsupported`; or
- split validation:

```rust
validate_deployment_profile_structure(...)
validate_production_deployment_release(...)
```

Only the second may return a type named production/release-valid, and it must remain unconstructible under schema 2.

This follows the same principle already applied successfully to:

```text
ValidatedTargetDefinition
≠
ReviewedElementsTapscriptDefinition.
```

---

# SR5-12 — Blank protocol records are silently accepted

**Severity:** Low
**Files:**

- `packages/target-elements-conformance/src/executor.rs`
- `packages/target-elements-conformance/src/bin/mock-native-executor.rs`

The protocol says one JSON object per line and says trailing protocol data fails.

But `read_line()` skips blank lines:

```rust
if !line.trim().is_empty() {
    return Ok(Some(line));
}
```

Thus:

- blank lines during the exchange are ignored;
- trailing blank lines after the last response are accepted.

Choose and document one rule:

- strict NDJSON: every line must be a JSON record; or
- whitespace-tolerant framing.

The repository’s general fail-closed posture favors strict NDJSON.

---

# SR5-13 — Public status documentation remains stale

**Severity:** Low
**Files:**

- `README.md`
- `packages/target-elements/README.md`
- `packages/target-elements/src/lib.rs`
- `packages/target-elements/src/evidence.rs`

The root README still describes `packages/tapscript/` as:

> “capability adapter only; no instruction core.”

The current package includes:

- typed instructions;
- checked stack items;
- serializer;
- parser;
- abstract stack validator.

The root layout also omits `packages/target-elements-conformance/`.

Meanwhile, target package documentation says:

> “Target-native deployment evidence has not been produced”

and:

> “No node has been asked anything.”

That conflicts with the Guide-9 gate and reference record claiming a 398-case development native run.

The correct package-boundary statement is:

```text
The static target crate carries evidence requirements but no mutable
evidence-completion status. Development native evidence is produced and
recorded separately by target-elements-conformance. Production evidence
remains absent.
```

That keeps evidence out of the static contract without making a stale repository-wide factual claim.

---

# Additional lower-severity observations

These did not rise to separate principal findings, but they are worth addressing alongside the owning repairs.

## Accepted responses are not protocol-shape validated

`NativeExecutionResponse` permits combinations such as:

```text
verdict = accepted
observed_failure = Some(...)
```

The comparison for an expected acceptance checks the verdict but does not reject the contradictory failure field.

Likewise, response fields are not validated against handshake capabilities. An executor may advertise final-stack reporting and then omit every stack; the comparison treats absence as vacuously acceptable.

Recommended protocol validator:

```text
Accepted:
    observed_failure must be absent

Rejected:
    failure required iff failure-class reporting advertised

InfrastructureError:
    no target failure class
    no target final stack claim

FinalStackReporting advertised:
    stack presence follows the declared protocol contract
```

## Fixture construction does little semantic context validation

`PrimitiveFixture::state()` binds the script path to the fixture, which is good, but does not independently validate every transaction-context relationship. The canonical census may be correct, but public fixture construction should distinguish:

```text
well-shaped fixture
validated executable fixture
```

before another package treats arbitrary caller fixtures as evidence subjects.

## The Python adapter tolerates some fixture-path mismatches it could reject

For example, it checks a context script only when the context script is nonempty:

```python
if declared_path["script"] and declared_path["script"] != fixture["script"]:
```

A context with an empty path script and a nonempty fixture script is therefore accepted. Canonical Rust construction prevents this today, but the executor protocol should still enforce exact equality rather than rely on the sender.

---

# Positive results from the second pass

The second pass strengthened my confidence in several important areas.

## 1. Model authorization and structural validity remain separated

The model consistently distinguishes:

```text
kernel structural validity
≠
signer authorization
≠
compiler signature placement
≠
deployment sighash behavior
```

That is a sound assurance boundary.

## 2. Exact flow partitioning is a strong design

The canonical partition requires each canonical source and destination exactly once. This is substantially stronger than aggregate conservation and blocks offsetting-flow attacks.

## 3. Sponsor-value opacity is genuinely structural

The system does not merely promise not to log sponsor values. It removes them from:

- realization observations;
- compiler operands;
- source requirements;
- layout;
- coverage;
- analyzed programs.

That is the correct shape for an opacity claim.

## 4. History residue remains well isolated

The model’s monetary queries read committed state only. Residue appears in invariant/external-auditor paths and is excluded from attestation and monetary queries. I found no selected production path that reintroduces it into payout, issuance, or floor calculations.

## 5. Architecture identity handling is much stronger than ordinary hash-envelope designs

The separation among:

- draft validation;
- release validation;
- untrusted envelope self-consistency;
- trusted expected-value comparison;
- semantic versus behavioural identities;

is careful and mostly type-enforced.

## 6. The compiler remains explicit about partiality

The compiler does not manufacture a complete protocol claim from the two pilot operations. It retains:

- partial architecture scope;
- lifecycle incompleteness;
- unresolved external evidence;
- no public compiler identity;
- no target-specific type in compiler core.

That honesty is a substantial strength.

---

# Recommended repair order

I recommend the following sequence.

## Batch A — Close typed trust-state gaps

1. **SR5-04:** bind a development binding to the exact validated target;
2. **SR5-11:** split pre-release profile validation from production-release validation;
3. **SR5-07:** weld relation IDs to relation bodies.

These are type-model changes and should land before more downstream APIs depend on the current shapes.

## Batch B — Repair the signature abstraction

4. **SR5-05:** introduce operand alternatives and conditional failure semantics;
5. add unknown-key-type native vectors;
6. require abstract/native agreement for empty and unknown-key paths.

This should precede any backend pattern that depends on signature authorization.

## Batch C — Make native evidence complete and self-validating

7. **SR5-03:** put complete fixture projections into report rows;
8. **SR5-02:** add a typed evidence-claim census;
9. **SR5-01:** return and gate a validated report wrapper;
10. **SR5-06:** bind the report to the observed chain;
11. **SR5-08:** add complete executor/node/framework/upstream provenance.

These changes belong together because exact report validation needs both the exact subject and exact coverage claims.

## Batch D — Harden executor mechanics

12. **SR5-09:** supervise the full process group and cleanup path;
13. **SR5-10:** bound protocol records;
14. **SR5-12:** make blank-line behavior strict;
15. validate response field combinations and handshake capability promises.

## Batch E — Reconcile documentation

16. **SR5-13:** update root and package status claims after the implementation changes settle.

---

# Suggested focused regressions

At minimum, add or retain focused tests equivalent to the following.

## Report gate

```text
empty evidence census rejects
one missing required row rejects
one duplicated row rejects
one unexpected row rejects
required row relabeled unresolved rejects
wrong report schema rejects
wrong summary rejects
failed case with passing evidence rows rejects
infrastructure-error case rejects
```

## Exact fixture/report binding

```text
same case ID, different script → reports differ
same case ID, different context → reports differ
same case ID, consensus versus relay → reports differ
changed expected resource → reports differ
changed fixture set stales prior report
```

## Evidence-claim coverage

```text
issuance-absent alone does not complete issuance semantics
explicit value alone does not complete confidential value semantics
transaction-signature rejection alone does not complete signature semantics
consensus nonminimal-push acceptance does not complete relay minimality
consensus resource case does not complete policy resource evidence
```

## Target/deployment binding

```text
binding validated against target A cannot bind target B of same version
reviewed target requires reviewed development binding
capability-status change stales a generic binding
resource-contract change stales a generic binding
```

## Signature abstraction

```text
empty signature reaches consume-and-false
nonempty valid signature reaches success
nonempty invalid signature reaches abort
unknown nonempty key type reaches documented compatibility path
empty public key rejects
exact64 signature cannot simultaneously be classified empty
```

## Executor lifecycle

```text
timeout terminates descendant process
timeout removes disposable datadir
descendant retaining stdout cannot hang the harness
maximum protocol line accepted
maximum+1 line rejected
no-newline oversized line rejected without unbounded allocation
trailing blank line follows the documented contract
```

---

# Final conclusion

The repository’s semantic core remains notably strong. The second pass did not expose an obvious model-level economic exploit in the selected production sources.

The most important correction is that the final evidence layer must adopt the same discipline already used elsewhere:

\[
\text{raw value} \rightarrow \text{complete owner validation} \rightarrow \text{validated wrapper} \rightarrow \text{consumer}.
\]

At present:

- target definitions follow that law;
- architecture identities follow it;
- realization and compiler inputs largely follow it;
- native reports do not.

The other major issue is independent of reporting: the static signature contract says the target has meaningful empty-signature and unknown-key-type paths, while the abstract operand model cannot represent those paths faithfully. That should be fixed before signature-dependent backend work begins.

**Overall verdict:** strong architecture and model foundation; no production-readiness claim; native evidence and target-signature abstraction require another hardening batch before they should serve as downstream release evidence.
