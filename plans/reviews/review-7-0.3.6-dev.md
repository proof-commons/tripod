# Seventh static review — tree 0.3.6-dev

## Review summary

This is a notably disciplined codebase. The strongest recurring design choices are:

- validation-before-identity wrappers;
- exact typed censuses rather than best-effort containment;
- explicit separation of semantic, artifact, provenance, evidence, and release claims;
- canonical versus experimental evidence subjects;
- exact request/transcript/report binding;
- careful distinction between target rejection, construction failure, and infrastructure failure;
- honest non-claims around untrusted execution, filesystem trust, mock evidence, and production readiness.

I nevertheless found several concrete issues. The first three are the most important.

---

## Findings

### 1. High — `ExpectedExecutorProvenance` can be constructed with abbreviated “expected” revisions

**Files**

- `packages/target-elements-conformance/src/provenance.rs`
- Functions/types:
  - `ExpectedExecutorProvenance`
  - `ExpectedExecutorProvenance::new`
  - `RevisionId::new`
  - `RevisionId::full`
  - `RevisionId::matches_full`
  - `validate_executor_provenance`

**Problem**

`ExpectedExecutorProvenance::new` correctly requires full 40-character object IDs through `RevisionId::full`. However, all fields of `ExpectedExecutorProvenance` are public:

```rust
pub struct ExpectedExecutorProvenance {
    pub intended_tip: RevisionId,
    pub upstream_base: RevisionId,
    pub included_local_topics: BTreeSet<TopicName>,
}
```

`RevisionId::new` publicly constructs identifiers as short as seven characters. Therefore an external caller can bypass `ExpectedExecutorProvenance::new`:

```rust
let short = RevisionId::new("abcdef0").unwrap();

let expected = ExpectedExecutorProvenance {
    intended_tip: short.clone(),
    upstream_base: short,
    included_local_topics: BTreeSet::new(),
};
```

`validate_executor_provenance` does not defensively reassert full width for the expected revisions. It compares reported abbreviated values directly against the abbreviated expected values and then calls:

```rust
binary.matches_full(&expected.intended_tip)
```

Despite the parameter name and documentation, `matches_full` does not check that its second operand is actually full-width. With the construction above, matching seven-character values can pass.

**Impact**

The public type does not uphold the invariant claimed by its documentation:

> “The two revisions must be full object identifiers.”

Any external consumer of the public gate API can weaken expected provenance from a full object ID to a short prefix. This undermines the fail-closed ADR-018 binding at the API boundary, even though the current CLI path uses the safe constructor.

**Recommendation**

Prefer a distinct type:

```rust
pub struct FullRevisionId(RevisionId);
```

with a single validating constructor, then use it in `ExpectedExecutorProvenance`.

At minimum:

1. make `ExpectedExecutorProvenance` fields private;
2. expose read-only accessors;
3. defensively check full width again in `validate_executor_provenance`;
4. make `RevisionId::matches_full` accept `&FullRevisionId`, not `&RevisionId`;
5. add a public-API regression proving struct-literal or abbreviated construction is impossible.

---

### 2. High — the public attestation anchor-set identity API permits ambiguous preimages and bypasses validation-before-identity

**File**

- `packages/architecture/src/canonical.rs`
- Function: `anchor_set_hash`

**Problem**

The anchor-set recipe is:

```rust
sha256(domain_prefix || join("\n", sorted(distinct anchor names)))
```

but `anchor_set_hash` accepts unrestricted `&str` values:

```rust
pub fn anchor_set_hash<'a>(
    anchor_names: impl IntoIterator<Item = &'a str>,
) -> [u8; 32]
```

This permits two distinct input sets to have exactly the same preimage before hashing:

```text
{"a\nb"}  → "a\nb"
{"a", "b"} → "a\nb"
```

Thus:

```rust
anchor_set_hash(["a\nb"]) == anchor_set_hash(["a", "b"])
```

without invoking any cryptographic collision.

The actual repository label grammar excludes newline characters, so the current harvested corpus is presumably safe. The problem is that the public identity function does not encode or validate that precondition. This differs from the architecture and deployment identities, which carefully accept only validated wrappers.

**Impact**

The function can mint an “anchor-set identity” for values that are not valid anchor names, contrary to ADR-016’s validation-before-identity rule. Its canonical framing is not injective over its public argument domain.

That makes the safety of the identity depend on an undocumented caller discipline rather than on the API.

**Recommendation**

Best option without changing the current pinned recipe:

1. introduce a validated `AnchorName` or `ValidatedAnchorSet`;
2. enforce the exact label grammar and prohibit `\n`;
3. make the hash function accept only the validated wrapper;
4. return `Result` from the raw-input constructor;
5. add a regression for the `{"a\nb"}` versus `{"a","b"}` ambiguity.

For a future recipe migration, use an unambiguous canonical encoding such as a canonical JSON/CBOR array or length-prefixed members. That would change the recipe and therefore must follow ADR-016 migration rules.

---

### 3. High — several shipped Rust executables violate ADR-010’s command-line contract

**Files**

- `packages/target-elements-conformance/src/bin/emit-conservation-matrix.rs`
- `packages/target-elements-conformance/src/bin/emit-normalization-matrix.rs`
- `packages/target-elements-conformance/src/bin/emit-lifecycle-report.rs`
- `packages/target-elements-conformance/src/bin/emit-normalization-report.rs`

**Problem**

These binaries bypass `cli-common` and implement their own process interfaces.

The matrix emitters:

- write directly to stdout;
- do not perform TTY refusal;
- use `expect`, so failures can emit default panic behavior unless another mechanism outside the shown code intervenes;
- do not install the shared panic hook;
- do not use the shared result runner;
- do not use the shared exit-class machinery.

The report emitters additionally:

- parse positional argv through `std::env::args`;
- emit plain-text diagnostics with `writeln!(stderr, ...)`;
- emit plain-text usage;
- include the argument-supplied path in diagnostics;
- return their own `ExitCode` branches instead of the shared command runner.

For example:

```rust
drop(writeln!(std::io::stderr(), "emit-lifecycle-report: {note}"));
```

and:

```rust
let path = std::env::args()
    .nth(1)
    .ok_or_else(|| "usage: emit-lifecycle-report RUN-RECORD".to_owned())?;
```

ADR-010 explicitly requires JSON-or-nothing on both streams, TTY refusal for stdout-result commands, shared panic handling, stable exit classes, and no raw argv/path echo in diagnostics.

Using `Write` instead of `println!` merely evades a macro lint; it does not satisfy the contract.

**Impact**

These commands can:

- emit non-JSON diagnostics;
- print machine results to a terminal;
- produce plain panic output;
- expose caller-provided argv values;
- behave differently from all other first-party commands;
- corrupt automation expecting ADR-010 control-plane records.

**Recommendation**

Move all four binaries to `cli-common`:

- use `BaseArgs`;
- use `parse_args_or_exit`;
- install `install_json_panic_hook` before parsing;
- use `run_stdout_command`/`run_check_command` or the corresponding current shared runner;
- use `CheckOutputArgs` for report/stamp publication where appropriate;
- emit one canonical JSON result through the shared writer;
- remove path values from diagnostics;
- add subprocess-contract tests for help, version, malformed argv, TTY refusal, runtime failure, panic, and successful output.

The same audit should be applied to the Python scripts because several of them also behave as first-party commands with plain stderr and ad hoc exit handling.

---

### 4. High — raw child stderr is incorporated into first-party diagnostics and protocol detail

**File**

- `scripts/elements-native-executor.py`
- Relevant code:
  - `DisposableNode.call`
  - `one_line`
  - various `ConstructionError`/`AdapterError` propagation paths

**Problem**

`DisposableNode.call` captures arbitrary stderr from `elements-cli`:

```python
completed = subprocess.run(
    command,
    stdin=subprocess.DEVNULL,
    capture_output=True,
    text=True,
    check=False,
)
```

and then includes it in an error:

```python
raise AdapterError(
    "rpc %s failed: %s" % (method, one_line(completed.stderr))
)
```

Those bytes subsequently flow into:

- first-party stderr logs;
- `observed_detail`;
- run records;
- report text;
- other derived diagnostics.

ADR-010 explicitly says raw stderr from an argument-selected external executable is arbitrary child output and must be omitted rather than logged or heuristically redacted. ADR-015 likewise says selecting the executable grants authority but does not make its output safe.

The script also logs the caller-supplied framework path:

```python
log("framework loaded from %s" % framework_path)
```

which is an argv-derived path value.

**Impact**

A selected child can place arbitrary data into first-party diagnostics or report records. Even in the current public-test-data environment, this violates the field-classification boundary and creates a future leak path if the execution environment contains sensitive paths, tokens, cookie diagnostics, or other host data.

**Recommendation**

Replace child stderr with fixed typed information:

```text
RPC method
protocol phase
process exit status
fixed failure class
```

Do not include:

- raw stderr;
- executable path;
- argv;
- datadir path;
- environment-derived text.

If detailed child output is intentionally required as result data, give it an explicit, documented relay channel with the same deliberate status as `execwrap`; do not smuggle it through diagnostics or report-detail fields.

---

### 5. Medium–high — normalization report ingestion accepts duplicate and unexpected response rows

**File**

- `packages/target-elements-conformance/src/bin/emit-normalization-report.rs`
- Function: `run`

**Problem**

Responses are indexed as follows:

```rust
let mut answered: BTreeMap<String, NativeNormalizationResponse> = BTreeMap::new();

for response in responses {
    response.validate_shape()?;
    answered.insert(response.case.normalization.clone(), response);
}
```

The return value of `insert` is ignored.

Consequences:

- two responses for one mutation are silently accepted;
- the later response overwrites the earlier one;
- contradictory observations can be hidden by ordering the preferred response last;
- responses for unknown/noncanonical mutations are retained in the map and then ignored when the report is built;
- no exact key-set equality is checked between the response census and `canonical_mutation_matrix()`.

The comments elsewhere in this subsystem repeatedly insist on exact censuses in both directions, so this is inconsistent with the evidence model.

**Impact**

A malformed or edited run record can produce an apparently normal typed report while containing duplicate or extraneous observations. Even though this lane is currently experimental and non-gating, the resulting report can misstate the historical run.

**Recommendation**

Reject duplicates immediately:

```rust
if answered.insert(key.clone(), response).is_some() {
    return Err(format!("the run answered {key} more than once"));
}
```

Then compare exact key sets:

```text
answered keys = canonical matrix mutation keys
```

Reject both missing and unexpected rows. Ideally parse a strict typed top-level run record with `deny_unknown_fields` instead of indexing through `serde_json::Value`.

---

### 6. Medium–high — lifecycle matrix completeness is checked only for the first reading pass

**Files**

- `packages/target-elements-conformance/src/lifecycle_report.rs`
- `packages/target-elements-conformance/src/bin/emit-lifecycle-report.rs`
- Method: `FreshProcessLifecycleReport::matrix_is_complete`

**Problem**

`matrix_is_complete` checks only:

```rust
let Some(first) = self.passes.first() else {
    return false;
};

canonical_lifecycle_matrix().into_iter().all(|expectation| {
    first.rows.iter().any(|row| row.row == expectation.row)
        || self.unbuilt_rows.iter().any(|row| row.row == expectation.row)
})
```

It does not establish that:

- every pass has the complete row census;
- a pass contains each row exactly once;
- no pass duplicates a row;
- every pass has the same row set;
- every pass has a unique attempt ordinal.

`emit-lifecycle-report` calls `matrix_is_complete`, so a record with a complete first pass and an empty or partial second pass can be emitted as matrix-complete. The distinct-process check still passes if the PIDs differ.

This directly conflicts with the report’s intended “hidden-cache dependency” claim, which relies on comparing complete repeated passes.

**Impact**

A partial second pass can be presented as a completed fresh-process lifecycle experiment. This weakens the central claim that independent cold starts reached the same matrix verdicts.

**Recommendation**

Validate every pass:

1. derive the canonical row set once;
2. require each pass to contain exactly that set minus globally unbuilt rows;
3. reject duplicate rows within a pass;
4. reject unexpected rows;
5. require distinct attempt identifiers as well as distinct PIDs;
6. require at least two complete passes for the cache-independence claim;
7. consider making `matrix_is_complete` imply `passes_agree` only if that is truly what callers expect, or split the two predicates with explicit names.

Add a regression where pass 1 is complete and pass 2 is empty.

---

### 7. Medium — consensus script-error parsing truncates messages containing parentheses

**File**

- `scripts/elements-native-executor.py`
- Method: `CaseExecutor.judge_in_block`

**Problem**

Consensus errors are extracted with the first closing parenthesis after the prefix:

```python
start = error.note.index(CONSENSUS_SCRIPT_PREFIX) + len(CONSENSUS_SCRIPT_PREFIX)
end = error.note.index(")", start)
return rejection(error.note[start:end])
```

At least one explicitly supported error string contains parentheses:

```text
OP_CHECKMULTISIG(VERIFY) is not available in tapscript
```

For such a message, extraction stops after `VERIFY`, yielding approximately:

```text
OP_CHECKMULTISIG(VERIFY
```

which does not match the mapping table.

Because the executor advertises failure-class reporting, the Rust harness can then reject the response as malformed for omitting a class.

**Impact**

A real, classified target rejection can be transformed into a protocol failure. The result depends on punctuation in the node’s human error string rather than the intended mapping.

**Recommendation**

Do not parse a parenthesized wrapper with the first `)`.

If the surrounding text is stable, strip an exact prefix and final suffix:

```python
if text.startswith(prefix) and text.endswith(")"):
    script_error = text[len(prefix):-1]
```

Better still, consume a structured error code from the node/adapter boundary and use text only as recorded detail.

Add a focused test for every mapped string containing `(` or `)`.

---

### 8. Medium — current planning documents materially contradict one another

**Files**

- `plans/guides/guide_eleven.md`
- `plans/backlog.md`
- `plans/phases/03-target-foundation.md`
- `packages/target-elements-conformance/src/lifecycle.rs`

**Problem**

The current Guide 11 header says:

```text
Status: Execution guide; not yet executed
```

and its exit checklist remains entirely unchecked.

However:

- the backlog records the Guide-11 gate as completed;
- the declassification policy is selected;
- the normalization and lifecycle findings are recorded as done;
- Phase 3 says the initial declassification policy is selected.

There is also a current lifecycle-count inconsistency:

- `canonical_lifecycle_matrix()` has 9 rows;
- the backlog’s later Wave-11 record says 18/18 observations over two passes;
- earlier current-state prose still says 16/16;
- the Phase-3 card says “8 lifecycle rows over two passes.”

The stale-evidence row is described as closed later, so the eight-row count is obsolete.

**Impact**

A reader cannot determine from the active planning tree whether Guide 11 is unexecuted, completed, or partially completed, nor what the accepted lifecycle evidence census is. This is especially problematic because the repository’s process emphasizes exact status and census agreement.

**Recommendation**

Choose one of two models:

1. **Living execution guide:** mark Guide 11 executed/completed and update its checklist and counts; or
2. **Immutable execution charter:** explicitly mark it “historical execution charter; completion recorded at …” rather than “not yet executed.”

Normalize lifecycle evidence wording to:

```text
9 canonical rows × 2 fresh-process passes = 18 row observations
```

and state separately whether any rows were initially unbuilt in an earlier run.

A plans check should compare guide status against the backlog’s recorded gate state where a guide is designated current/completed.

---

### 9. Low — evidence-registry documentation says no node evidence exists, contradicting the current repository state

**Files**

- `packages/target-elements/src/evidence_registry.rs`
- `packages/target-elements/src/evidence.rs`
- `plans/backlog.md`
- `plans/reference/elements-tapscript.md`

**Problem**

`evidence_registry.rs` says:

> “Every requirement in the registry is unresolved. None has been evidenced against any node.”

Elsewhere the repository records substantial development target-native evidence, and `evidence.rs` correctly notes that such evidence exists in the conformance package.

It is valid—and desirable—for the immutable target definition not to carry mutable evidence status. But that is different from saying no evidence exists.

**Impact**

The static-contract boundary is described inaccurately and contradictorily. A reader may infer that the native evidence record is absent rather than merely external to this package.

**Recommendation**

Reword to:

> “The registry carries requirements only and records no completion status. Development evidence may exist in external conformance reports; production evidence remains absent.”

That preserves the ownership boundary without making a false repository-state claim.

---

## Broader observation: the non-gating experimental lanes need a common ingestion contract

The normalization, conservation, and lifecycle paths have strong typed internal report models, but their runner-to-report boundary is substantially weaker than the primitive/prototype native-evidence path:

- top-level records are often parsed through `serde_json::Value`;
- unknown fields are often accepted;
- duplicates are not always rejected;
- exact row-set equality is not always checked;
- process supervision and timeouts are implemented ad hoc in Python;
- command output does not follow ADR-010;
- the lanes explicitly “record and do not gate.”

The backlog already acknowledges parts of this for conservation. I would treat this as one coherent future tranche rather than fixing each script independently:

```text
strict typed run-record schema
→ exact census validation
→ shared process supervision
→ validated experimental report wrapper
→ optional gate only when policy chooses one
```

No digest is needed; exact typed comparison remains sufficient.

---

## Positive review notes

Several parts deserve explicit praise:

1. **Architecture identity API design** is generally strong. `ValidatedDraftArchitecture` and `ValidatedReleaseArchitecture` correctly make validation state part of the type, and unchecked projections stay crate-private.

2. **Published architecture ingestion** clearly distinguishes:
   - hash-envelope self-consistency;
   - release status;
   - equality to an independently derived expected value.

   The non-authenticity documentation is unusually good.

3. **Deployment release refusal** is honest. `validate_production_deployment_release` explicitly rejects every schema-2 profile because the ABI/configuration binding is absent, rather than allowing a structurally complete profile to masquerade as production-ready.

4. **Compiler analysis validation** is robustly framed. The production constructor invokes the complete re-derivation validator, rather than leaving corruption resistance to tests alone.

5. **Evidence request design** is substantially improved by protocol revision 3. Keeping expectations out of executor requests is the correct way to make the boundary structural.

6. **Canonical versus experimental subjects** are separated well in the primitive and prototype paths. Private fields plus regeneration checks provide both type-level and value-level defenses.

7. **Sponsor-value opacity** is carried consistently through realization, compiler, target assessment, and model rules. The distinction between role/membership authentication and amount inspection is clear and technically meaningful.

8. **Failure-layer separation**—construction, infrastructure, consensus, script, relay, report—is one of the strongest aspects of the conformance design.

9. **The no-digest discipline** is consistently applied to reports and intermediate compiler values, which aligns well with ADR-016.

---

## Review limitations

This was a static review of the supplied 142-file selection at tree `0.3.6-dev`.

I did **not** run:

- Cargo formatting, Clippy, or tests;
- Meson configuration, compile, or tests;
- Python/native executor integration;
- document reproducibility;
- advisory or license checks.

The supplied filter excluded 466 files, notably:

- `Cargo.toml` and `Cargo.lock`;
- most package test suites;
- `cli-common` implementation;
- several compiler/model/realization modules;
- Meson build definitions;
- the labels implementation;
- many subprocess-contract tests.

Accordingly, these findings should be reproduced against the full checkout. The most direct first follow-up tests would be:

1. abbreviated public construction of `ExpectedExecutorProvenance`;
2. newline ambiguity in `anchor_set_hash`;
3. duplicate normalization responses;
4. incomplete second lifecycle pass;
5. a consensus script error containing inner parentheses;
6. subprocess-contract coverage for every `emit-*` binary.

# Second review

This pass focused on different seams from the first review: wire-schema consistency, target-fact correctness, abstract execution, resource accounting, and subprocess lifecycle. I have intentionally not repeated the earlier findings unless needed for context.

## Executive summary

I found two particularly important new issues:

1. **The Python executor and Rust protocol no longer describe the same schema despite sharing protocol revision 3.** Strict Rust consumers will reject records produced by the experimental wallet-enabled lanes.
2. **The typed confidential-review record assigns the wrong parity convention to confidential nonces.** Prefixes `0x02/0x03` encode compressed-point oddness, not quadratic residuosity.

I also found correctness gaps in tapscript resource accounting and abstract execution, plus several fail-closed and supervision issues.

---

# Findings

## 1. High — protocol revision 3 has forked into incompatible Rust and Python schemas

### Files

- `packages/target-elements-conformance/src/protocol.rs`
- `packages/target-elements-conformance/src/executor.rs`
- `scripts/elements-native-executor.py`
- `scripts/run-conservation-matrix.py`
- `scripts/run-fresh-process-lifecycle.py`
- `scripts/run-normalization-matrix.py`

### Problem

The Rust and Python implementations both claim to speak:

```text
NATIVE_PROTOCOL_SCHEMA = 3
```

but they do not accept or emit the same set of records.

### A. Python advertises a capability Rust cannot deserialize

The Python executor advertises this in wallet-enabled mode:

```python
"fresh_process_lifecycle"
```

But `ExecutorCapability` in Rust has no `FreshProcessLifecycle` variant.

Because `ExecutorHandshake` is deserialized through Serde into:

```rust
pub capabilities: BTreeSet<ExecutorCapability>
```

a strict Rust consumer will reject the wallet-enabled handshake as malformed.

The custom Python lifecycle runner succeeds only because it parses the handshake as an untyped JSON dictionary instead of using the Rust protocol type.

### B. Python emits a conservation-response field Rust explicitly rejects

Rust defines:

```rust
#[serde(deny_unknown_fields)]
pub struct NativeConservationResponse {
    pub schema: u32,
    pub case: ConservationRowId,
    pub observed_layer: ObservedOutcomeLayer,
    pub observed_detail: Option<String>,
    pub transaction_bytes: Option<Vec<u8>>,
    pub observed_value_commitments: Vec<Vec<u8>>,
    pub observed_asset_commitments: Vec<Vec<u8>>,
}
```

The Python executor additionally emits:

```python
"observed_openings": body["observed_openings"]
```

The Python conservation runner consumes that field and uses it for the commitment-oracle comparison. A Rust deserializer, however, must reject it because of `deny_unknown_fields`.

### C. Lifecycle requests and responses have no Rust protocol type

The Python executor accepts records shaped like:

```json
{"case":{"lifecycle":"construct"}, ...}
```

and:

```json
{"case":{"lifecycle":"verify"}, ...}
```

but the Rust protocol has no lifecycle case identity, request, response, capability, or typed transcript entry for this exchange.

### Impact

The repository currently has at least two incompatible protocols carrying the same revision number:

1. the Rust primitive/prototype protocol;
2. the Python wallet-enabled conservation/normalization/lifecycle extension.

That defeats the purpose of a strict protocol revision. A consumer cannot determine record compatibility from `schema = 3`, and the custom Python runners bypass the unknown-field and enum checks the Rust protocol is designed to enforce.

It also creates a migration trap: implementing the planned Rust conservation or lifecycle driver against the existing Rust types will immediately fail against the current Python executor.

### Recommendation

Choose one explicit design.

#### Preferred option: bump and unify

Create protocol revision 4 with typed Rust definitions for:

- `FreshProcessLifecycle` executor capability;
- lifecycle case identity;
- lifecycle request and response;
- observed commitment openings;
- normalization request and response if these are part of the same protocol;
- conservation request and response;
- transcript bindings for all workloads.

Then update the Python executor and all runners together.

#### Alternative: declare separate experimental protocols

If these lanes are intentionally outside the native protocol, stop labeling them with `NATIVE_PROTOCOL_SCHEMA`. Give each an independent schema, for example:

```text
tripod-conservation-executor-1
tripod-normalization-executor-1
tripod-fresh-process-lifecycle-1
```

In either model, add round-trip tests that serialize in Rust, deserialize in Python, serialize in Python, and deserialize in Rust. Unknown fields must fail on both sides.

---

## 2. Medium–high — confidential nonces are assigned the wrong point-parity convention

### File

- `packages/target-elements/src/confidential.rs`
- Function: `reviewed_confidential_review_facts`

### Problem

The typed review facts construct the nonce encoding as:

```rust
ConfidentialFieldEncoding::new(
    33,
    33,
    1,
    (2, 3),
    PointParityConvention::QuadraticResidue,
)
```

Prefixes `0x02` and `0x03` are compressed public-key point encodings. Their bit distinguishes the oddness of \(y\), not whether \(y\) is a quadratic residue.

The same file already defines the appropriate vocabulary:

```rust
PointParityConvention::CompressedOddness
```

The nearby comment recognizes that the nonce is a transported point rather than a value commitment:

> “The nonce shares the shape. Its committed form is a transported point rather than a commitment…”

But the type does not express “no parity claim”; it positively records `QuadraticResidue`.

That convention is appropriate for:

- value commitments under `0x08/0x09`;
- asset generators under `0x0a/0x0b`;

not for a compressed nonce point under `0x02/0x03`.

### Impact

`reviewed_confidential_review_facts()` is presented as a typed transcription of target facts. Any later opening, nonce, or point-encoding analysis that consumes its `nonce().parity()` value will be told a false target fact.

This is especially risky because the project correctly treats parity-domain mismatches as load-bearing for authenticated-opening safety.

The current `TargetDefinition` does not appear to embed this review-facts object, which limits immediate operational impact, but the public typed review API is still incorrect.

### Recommendation

Change the nonce convention to:

```rust
PointParityConvention::CompressedOddness
```

If the project intentionally does not want to assert nonce-point semantics, add an explicit variant such as:

```rust
PointParityConvention::OpaqueTransportedPoint
```

rather than representing “no claim” as a known-false claim.

Add tests covering all three families:

| Field | Prefixes | Required convention |
|---|---|---|
| value commitment | `0x08`, `0x09` | quadratic residuosity |
| asset generator | `0x0a`, `0x0b` | quadratic residuosity |
| confidential nonce | `0x02`, `0x03` | compressed oddness |

The source provenance in `plans/reference/elements-tapscript.md` should state this distinction explicitly.

---

## 3. Medium–high — tapscript resource projection omits all pushed bytes

### File

- `packages/tapscript/src/stack.rs`
- Function: `resource_projection`

### Problem

The resource projection loops over program instructions but skips every push:

```rust
for instruction in program.instructions() {
    let TapscriptInstruction::Opcode(id) = instruction else {
        continue;
    };

    let cost = opcode(target, *id).resources();
    // ...
}
```

It then accumulates `ResourceDimension::ScriptBytes` only from opcode resource rows.

A push instruction occupies:

- its push opcode;
- possibly a width prefix;
- its payload bytes.

None of those bytes is counted.

For example, a program containing only a 520-byte push encodes to approximately 523 script bytes:

```text
OP_PUSHDATA2 + 2-byte length + 520-byte payload
```

but `resource_projection` returns no `ScriptBytes` contribution at all.

Even a one-byte literal pushed through a small-number opcode occupies a byte and is currently omitted.

### Impact

The function’s documentation says it reports:

> “The resource cost one program can charge, by dimension.”

For script bytes, that result is materially lower than the exact encoded program size. Any future backend, linker, or candidate-bound analysis relying on this projection can understate script size, especially for constructor and metadata-heavy programs where literals dominate.

This conflicts with the project’s broader requirement that predicted and observed resources agree.

### Recommendation

Compute script bytes from the actual canonical encoding:

```rust
let script_bytes = u64::try_from(program.encode(target).len()).unwrap_or(u64::MAX);
totals.insert(ResourceDimension::ScriptBytes, script_bytes);
```

Then accumulate only non-byte opcode dimensions in the existing loop:

- operation cost;
- validation budget;
- any future per-opcode dimensions.

Do not add the encoded byte length on top of the current opcode-byte sum, or opcode bytes will be counted twice.

Add focused tests for:

1. empty program;
2. one ordinary opcode;
3. empty push;
4. small-number push;
5. direct push;
6. `PUSHDATA1`;
7. `PUSHDATA2` at 520 bytes;
8. mixed pushes and opcodes;
9. equality between `ScriptBytes` and `program.encode(target).len()` for a generated corpus.

---

## 4. Medium–high — the abstract stack validator reports impossible successful executions for exact pushed literals

### Files

- `packages/tapscript/src/stack.rs`
- Relevant functions and types:
  - `literal_type`
  - `KnownConstants`
  - `is_definitely_false`
  - `step`
  - `apply_opcode`

### Problem

A pushed literal is reduced to its width:

```rust
fn literal_type(item: &StackItem) -> StackValueType {
    if item.is_empty() {
        StackValueType::Empty
    } else {
        StackValueType::Bytes {
            minimum: item.len(),
            maximum: item.len(),
        }
    }
}
```

The validator therefore remembers that a pushed item was one byte wide, but not that its byte was `0x00`.

`is_definitely_false` recognizes only:

```rust
StackValueType::Empty
```

or a type whose only width is zero.

But under target script truth semantics, nonempty values such as:

```text
00
0000
000000
80        negative zero
```

are also false.

Thus a typed program equivalent to:

```text
push [0x00]
VERIFY
```

is reported as having a successful path, even though the exact program always aborts with false verification.

The same information loss affects exact comparisons. Two exact literals with the same width but different bytes are treated as potentially equal even when the program itself fixed both byte strings.

### Impact

The module describes its result as the complete set of successful, non-aborting-failure, and aborting states. For exact pushed literals, it is instead an over-approximation.

The most dangerous direction is false liveness: an always-aborting program can appear to have a successful state. A future backend could therefore treat an unusable leaf as satisfiable or reachable based on abstract validation alone.

The project already tracks `KnownConstants` alongside abstract stack types, so the architecture recognizes that type shape alone is insufficient. The tracked facts are simply too narrow: they preserve some script-number constants for width reasoning, but not exact truth or byte equality.

### Recommendation

Extend the path-local known-value state to retain exact pushed bytes, or at least enough facts to settle:

- script truth;
- exact byte equality;
- script-number value where canonical;
- fixed-width value where appropriate.

For example:

```rust
enum KnownValue {
    ExactBytes(Vec<u8>),
    ScriptNumber(i64),
    Unknown,
}
```

Propagate known values through `OperandCopy` and stack rearrangements, and invalidate them through genuinely computed operations unless the result is statically decidable.

At minimum, add regressions for:

```text
push []      → VERIFY always aborts
push [00]    → VERIFY always aborts
push [80]    → VERIFY always aborts
push [01]    → VERIFY succeeds
push [01], push [02] → EQUALVERIFY always aborts
push [01], push [01] → EQUALVERIFY succeeds
```

If over-approximation is intentional, the API and documentation should say so, and no consumer should infer target liveness merely from a nonempty abstract success set.

---

## 5. Medium — the program decoder applies its instruction work limit only after doing unbounded work

### File

- `packages/tapscript/src/program.rs`
- Function: `TapscriptProgram::decode`

### Problem

`decode` parses the entire byte slice and appends every decoded instruction:

```rust
while offset < bytes.len() {
    // ...
    instructions.push(...);
}
```

Only after parsing finishes does it call:

```rust
Self::new(instructions)
```

which enforces:

```rust
MAXIMUM_PROGRAM_INSTRUCTIONS = 10_000
```

A script containing millions of one-byte reviewed opcodes will therefore cause the parser to:

- scan all millions of bytes;
- allocate and retain millions of typed instructions;
- only then reject the result for exceeding 10,000 instructions.

The stated work limit bounds accepted output but does not bound parser work.

### Impact

`decode` is the boundary for untrusted target bytes. A caller can induce memory and CPU use far beyond the explicit first-party instruction limit.

This is not sandbox escape or authority escalation, but it defeats the package’s typed resource-failure model: a supposed `InstructionLimitExceeded` branch arrives only after the excessive work has already happened.

### Recommendation

Check the count during parsing, before pushing another instruction:

```rust
if instructions.len() >= MAXIMUM_PROGRAM_INSTRUCTIONS as usize {
    return Err(TapscriptError::InstructionLimitExceeded {
        maximum: MAXIMUM_PROGRAM_INSTRUCTIONS,
    });
}
```

Perform the check for both opcode and push instructions.

Consider a separate byte-size ceiling for parser work if the supported subset can receive very large byte slices even with relatively few pushes. The input slice already exists, but the parser should not allocate a second unbounded representation.

Add a test using `MAXIMUM_PROGRAM_INSTRUCTIONS + 1` one-byte instructions and assert immediate typed failure. A test-only instrumented iterator or allocation counter would help establish that parsing stops at the boundary.

---

## 6. Medium — infrastructure-error responses can carry target resource observations

### File

- `packages/target-elements-conformance/src/protocol.rs`
- Function: `validate_observation_shape`

### Problem

For an infrastructure response, shape validation checks only stacks and failure class:

```rust
NativeVerdict::InfrastructureError => {
    if names_failure || reports_stack || reports_altstack {
        return Err(
            ResponseShapeDefect::InfrastructureResponseCarriesObservation
        );
    }

    return Ok(());
}
```

The function returns before checking `NativeResourceObservation`.

Consequently, an infrastructure response can carry:

- `peak_stack_items`;
- `peak_altstack_items`;
- `maximum_element_bytes`;
- `validation_budget_used`;
- `transaction_weight`;

even though the same module states:

> “A run that did not happen observed nothing.”

The same helper validates primitive and prototype responses, so both paths are affected.

Related asymmetries exist in the custom response shapes. For example, `NativeNormalizationResponse::validate_shape` does not reject nonempty `observed_witness_sizes` on a non-target outcome.

### Impact

The gate still refuses infrastructure cases, so this does not directly convert infrastructure trouble into passing evidence. It does, however, permit internally contradictory protocol records and reports: a case can say “no target run happened” while carrying target-derived execution measurements.

That weakens the strict distinction between infrastructure and target evidence which the rest of the subsystem treats as foundational.

### Recommendation

For `InfrastructureError`, require all target-observation fields to be absent:

```rust
if names_failure
    || reports_stack
    || reports_altstack
    || observed.observes_interpreter()
{
    return Err(
        ResponseShapeDefect::InfrastructureResponseCarriesObservation
    );
}
```

Decide explicitly whether `script_bytes` and `initial_stack_items` are allowed. They are request-derived restatements rather than target observations, so either is defensible:

- permit them but document that they are request metadata;
- require the all-default resource record for maximal simplicity.

Apply the same policy to:

- normalization witness-size observations;
- conservation commitment observations;
- lifecycle checks;
- future typed opening observations.

Add a mutation test for each optional resource field under `InfrastructureError`.

---

## 7. Medium — the experimental Python runners have no bounded protocol supervision or cleanup guarantee

### Files

- `scripts/run-conservation-matrix.py`
- `scripts/run-normalization-matrix.py`
- `scripts/run-fresh-process-lifecycle.py`

### Problem

The Rust executor path has a carefully designed supervisor:

- process group;
- explicit timeout;
- graceful termination;
- bounded cleanup interval;
- forceful termination;
- reap;
- cleanup guard before supervisor adoption.

The Python experimental runners bypass all of that.

For example:

```python
child = subprocess.Popen(...)
line = stream.readline()
```

`readline()` has no timeout. If the executor never terminates a record, the runner can block indefinitely.

There is also no encompassing `try/finally` that reliably terminates and reaps the child if:

- JSON parsing fails;
- environment comparison fails;
- a response is malformed;
- matrix lookup fails;
- report writing fails;
- an exception occurs after spawning but before normal shutdown.

In `run-fresh-process-lifecycle.py`, failure during `AdapterProcess.__enter__` occurs before the context manager is entered, so `__exit__` will not run. In `__exit__`, a timed-out `wait` raises but does not proceed to terminate/kill/reap.

These processes may themselves start nodes and descendants.

### Impact

A malformed or hung selected executor can:

- hang the runner indefinitely;
- leave the adapter or node running;
- leave unreaped direct children;
- leave temporary state behind;
- prevent deterministic failure reporting.

This is not hostile-process containment—the repository correctly disclaims that—but it is a correctness and lifecycle defect in the lanes that produced the current experimental evidence records.

The backlog acknowledges that these lanes “record and do not gate,” but non-gating status does not make subprocess lifecycle optional.

### Recommendation

Prefer reusing the Rust `ExecutorSupervisor` through a target-generic driver rather than maintaining a second supervisor in Python.

If these runners remain Python:

1. start the child in a new session/process group;
2. enforce total-run and per-record deadlines;
3. enforce maximum record sizes;
4. reject blank records consistently;
5. put every post-spawn path under `try/finally`;
6. terminate the group gracefully;
7. wait for a bounded cleanup interval;
8. kill the group;
9. reap the direct child;
10. never reuse a partially completed run record.

Add subprocess tests for:

- no handshake;
- unterminated handshake;
- oversized response;
- malformed JSON;
- hang during a row;
- exit during a row;
- report-write failure;
- failure in `__enter__`;
- descendant retaining stdout.

---

## 8. Low — the evidence-registry documentation contradicts the repository’s current evidence state

### Files

- `packages/target-elements/src/evidence_registry.rs`
- `packages/target-elements/src/evidence.rs`
- `plans/backlog.md`
- `plans/reference/elements-tapscript.md`

### Problem

`evidence_registry.rs` says:

> “Every requirement in the registry is unresolved. None has been evidenced against any node.”

Elsewhere the repository records development target-native evidence, including primitive and prototype runs. `evidence.rs` gives the more accurate boundary:

> requirements live here; produced evidence belongs to the conformance package; production evidence remains absent.

The intended invariant is clearly:

```text
the static target definition contains no mutable completion status
```

That is not the same as:

```text
no evidence exists anywhere in the repository
```

### Impact

The target package’s module-level documentation misstates repository status and conflicts with the evidence package and planning records. This can cause reviewers to interpret an intentionally external evidence state as wholly absent.

### Recommendation

Replace the statement with something like:

> “Every value in this registry is an immutable requirement and carries no completion status. Development conformance evidence may exist in external typed reports owned by `target-elements-conformance`; no such report mutates this registry. Production evidence remains absent.”

That preserves the package boundary while accurately describing the repository.

---

# Additional observations

## A. The experimental lanes are drifting toward a second evidence subsystem

The primitive/prototype path has:

- canonical subject wrappers;
- strict typed protocol records;
- transcript binding;
- validated reports;
- gate-only trust states;
- supervision;
- exact censuses.

The conservation/normalization/lifecycle path currently has:

- partially typed payloads;
- custom Python dispatch;
- generic JSON run records;
- schema extensions not represented in Rust;
- different process supervision;
- no common validated transcript type;
- experimental report generation.

That is understandable as research scaffolding, but it is now large enough that “temporary” is becoming architectural. The protocol-fork finding above is the clearest signal.

A coherent next tranche would be:

```text
typed experimental workload records
→ one strict executor protocol revision
→ one supervised transcript boundary
→ exact experimental report validators
→ optional gates added only when policy admits them
```

This can still retain distinct report roles and must not collapse conservation, normalization, and lifecycle into one assurance claim.

## B. The target-fact model is strongest when “unknown” is representable

The nonce-parity error illustrates a recurring rule: if the review intends to make no claim, it needs a type-level “unclassified” or “opaque” state. Reusing a nearby positive value to mean “not relevant here” creates a false typed fact.

The repository already follows this discipline well elsewhere:

- unsupported versus incomplete versus reviewed capabilities;
- expected versus observed outcomes;
- absent versus disagreeing comparison legs;
- target verdict versus infrastructure failure.

The same discipline should be applied to point-encoding conventions.

---

# Suggested remediation order

1. **Resolve the protocol revision fork.** It affects the meaning and future consumability of all wallet-enabled experimental records.
2. **Correct the nonce parity convention** and add source-backed tests.
3. **Fix `resource_projection`** before Guide 12 begins using tapscript resource predictions.
4. **Clarify and fix abstract execution of exact literals** before backend proof patterns rely on successful-state existence.
5. **Move decoder instruction-limit enforcement into the parse loop.**
6. **Close infrastructure-response observation shapes.**
7. **Unify subprocess supervision for the experimental lanes.**
8. **Correct the stale evidence-registry documentation.**

---

# Focused regression list

I would add at least these tests:

```text
protocol:
    wallet-enabled Python handshake round-trips through Rust
    conservation response with observed openings round-trips through Rust
    lifecycle request/response has an owned schema
    every protocol schema-3 record is accepted by both implementations
    every unknown field is rejected by both implementations

target confidential facts:
    08/09 → quadratic-residue convention
    0a/0b → quadratic-residue convention
    02/03 nonce → compressed-oddness convention

tapscript resources:
    ScriptBytes == encode(program).len() for push-only programs
    ScriptBytes == encode(program).len() for mixed programs
    520-byte push includes opcode and width prefix

abstract execution:
    push 00; VERIFY has no success state
    push 80; VERIFY has no success state
    push 01; VERIFY succeeds
    known-unequal literals; EQUALVERIFY has no success state

parser:
    MAXIMUM_PROGRAM_INSTRUCTIONS + 1 fails during parsing

response shape:
    infrastructure + transaction_weight is malformed
    infrastructure + peak stack is malformed
    infrastructure + observed witness sizes is malformed

supervision:
    hung experimental executor is timed out, killed, and reaped
    malformed response leaves no child or node
```

---

# Review limitations

This remains a static review of the supplied 142-file selection at tree:

```text
0.3.6-dev
```

I did not execute Cargo, Meson, native-node, Python integration, reproducibility, advisory, or clean-tree lanes. Most test modules and several implementation modules were excluded by the supplied filter, so some of these issues may already have partial tests elsewhere. The source-level contradictions—especially the protocol field mismatch, missing capability variant, nonce parity value, and push-byte resource omission—are nevertheless directly visible in the selected files.
