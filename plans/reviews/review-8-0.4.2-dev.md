# Eighth static review — tree 0.4.2-dev

## Review summary

I found several substantive issues in the selected tree. The most important is an evidence-boundary flaw: callers can mark canonical relation-coverage rows as discharged without executing a target or presenting a validated transcript. I also found a noncanonical transaction-decoding path, an operation planner that records—but does not propagate—a fatal resource mismatch, and multiple shape/linker exactness defects.

This was a **static review of the supplied 154-file selection** at tree `0.4.2-dev`. I did not run Cargo, Meson, the native executor, or the excluded tests.

---

# Findings

## 1. [P0] Canonical coverage can be forged without any execution

**Files:**

- `packages/vectors/src/plan.rs`
- `packages/vectors/src/materialize.rs`
- `packages/vectors/src/lib.rs`

**Affected APIs:**

- `CompactAshEvidencePlan::discharge`
- `CompactAshEvidencePlan::discharge_mutants`
- `CoverageObservation::is_discharged`
- public `ProjectionComparison`
- public `ObservedOutcomeLayer`

The canonical evidence plan has a checked constructor, but its evidence state can subsequently be mutated from caller-authored tuples:

```rust
pub fn discharge(&mut self, outcomes: &[Outcome])
```

where:

```rust
pub type Outcome = (
    TargetVectorId,
    ObservedOutcomeLayer,
    ProjectionComparison,
);
```

A caller does not need an executor transcript, validated report, target binding, accepted transaction, or independently computed projection. It can obtain a real vector ID from the plan and state the desired answer directly:

```rust
let mut plan = derive_evidence_plan(&fixture_bundle()?)?;
let id = plan.target_cases()[0].subject().id();

plan.discharge(&[(
    id,
    ObservedOutcomeLayer::Accepted,
    ProjectionComparison::Matched,
)]);

assert!(plan.discharged_rows() > 0);
```

The negative path has the same issue:

```rust
plan.discharge_mutants(&[(
    NegativeMutation::SplitSuccessorInTwo,
    id,
    ObservedOutcomeLayer::ScriptPathRejection,
)]);
```

For a linked mutation, this creates `CoverageObservation::ObservedRefusal`, which `is_discharged` accepts unconditionally.

This defeats the canonical/experimental distinction established by `CanonicalSubject`. The plan’s initial contents are canonical, but the supposedly observed evidence is not provenance-bearing. In particular, the API can manufacture the same coverage counts attributed elsewhere to live execution.

### Impact

A downstream report or gate reading `observed_rows`, `discharged_rows`, or `coverage_complete` cannot know whether those rows came from:

- a supervised target run;
- a validated operation report;
- or caller-authored enum values.

This is exactly the class of defect the repository’s evidence policy is intended to prevent.

### Recommendation

Do not accept raw `Outcome` or `MutantObservation` tuples at the public boundary.

Instead:

1. introduce a validated compact-ASH operation report/transcript wrapper with private fields;
2. construct it only by binding:
   - the exact target and deployment;
   - exact operation requests;
   - exact responses;
   - accepted bytes;
   - target-reported coins;
   - independently recomputed projection comparisons;
3. make coverage discharge consume that validated wrapper;
4. make the low-level discharge helpers crate-private;
5. add compile-fail or public-API tests proving an external caller cannot assert `Accepted + Matched` or a target refusal directly.

---

## 2. [P1] The target transaction decoder accepts a noncanonical “superfluous witness” encoding

**File:** `packages/transaction/src/bytes.rs`
**Affected function:** `TargetTransaction::decode`

The encoder determines witness presence from actual witness content:

```rust
pub fn has_witness(&self) -> bool {
    self.witnesses.iter().any(|witness| !witness.is_null())
}
```

and therefore emits `NO_WITNESS_FLAG` if every input witness is empty.

The decoder, however, accepts `WITNESS_FLAG`, parses entirely empty input and output witness records, and returns a valid `TargetTransaction` without checking that the witness section contained anything:

```rust
let with_witness = match flags {
    NO_WITNESS_FLAG => false,
    WITNESS_FLAG => true,
    ...
};
```

After decoding such bytes:

```text
decoded.encode() ≠ original_bytes
```

because re-encoding observes that all witnesses are null and emits the witnessless form.

The source comments already identify the target rule:

> “A transaction whose every witness is empty must be serialized without the witness section; writing an all-empty section is an error the target asserts on rather than tolerates.”

The decoder does not enforce that rule.

### Impact

- Two byte strings are accepted for one typed transaction.
- External bytes are not converted immediately into a canonical validated value as claimed.
- `read_accepted` in `packages/vectors/src/comparison.rs` calls `TargetTransaction::decode` without requiring an exact round trip, so a dishonest or malformed observation can acquire a semantic projection even though the target would reject the encoding.
- Any identity or comparison later based on the decoded value rather than the original bytes can conflate distinct wire subjects.

### Recommendation

After parsing the witness section, reject:

```text
with_witness = true
∧ every input witness is null
∧ every output witness is null
```

with a dedicated typed refusal such as:

```rust
TransactionRefusal::SuperfluousWitnessRecord
```

Then add tests proving:

1. witnessless canonical bytes decode;
2. genuinely witnessed bytes decode;
3. witness-flagged all-empty bytes reject;
4. every successfully decoded transaction satisfies:

```rust
decoded.encode() == input
```

where canonical decoding is the intended contract.

---

## 3. [P1] A resource-prediction mismatch does not actually refuse the operation plan

**File:** `packages/vectors/src/operation.rs`
**Affected functions:**

- `CompactAshOperationPlanner::settle_submission`
- `TargetOperationPlanner::next_step`

`settle_submission` detects a predicted/observed weight mismatch and records a `PlanRefusal`:

```rust
if observed_weight.is_some_and(|observed| observed != predicted_weight) {
    self.transcript.refusal = Some(PlanRefusal::WeightObservationDisagrees {
        ...
    });
}
```

But it returns `()` and does not set `Stage::Done`. `next_step` then continues to schedule further submissions and eventually returns `Ok(None)`.

Consequently:

- `execute_operations` may return `Ok(ExecutionTranscript)`;
- the rendered run may say `"run": "completed"`;
- while `OperationTranscript::refusal()` simultaneously contains a fatal plan refusal.

The focused test notices the stored refusal, but deliberately does not assert that the planner returned `PlanRefused`. Thus the test preserves the contradictory state rather than catching it.

### Impact

The operation API’s control-plane result disagrees with the operation report:

```text
executor result: success
planner transcript: refused
```

A caller that checks only `execute_operations(...) -> Ok(...)` can accept a run whose resource comparison is known to be false. The ignored live test later asserts the mismatch, but that does not repair the public API.

### Recommendation

Make `settle_submission` return `Result<(), PlanRefusal>` and propagate the mismatch through:

```rust
return Err(self.refuse(refusal));
```

Alternatively, check `self.transcript.refusal` immediately after settling any stage and stop.

Add a test requiring all three properties:

```text
weight mismatch
⇒ next_step returns Err(PlanRefused)
⇒ execute_operations returns Err(OperationPlanRefused)
⇒ no later operation step is emitted
```

A transcript should never simultaneously say “completed” and carry a fatal plan refusal.

---

## 4. [P1] `CandidateShapeSet` does not prove that its shapes lie within its stated bounds

**Files:**

- `packages/tapscript/src/shape.rs`
- `packages/tapscript/src/policy.rs`
- `packages/tapscript/src/bundle.rs`

**Affected constructors:**

- `CandidateShapeSet::new`
- `CompactAshBackendPolicy::new`
- `ConcreteCandidate::new`

`CompactAshShape` validates against bounds when the shape itself is created, but it does not retain those bounds. A shape created under a wide bound can therefore be inserted into a set that declares narrower bounds:

```rust
let wide = CompactAshShapeBounds::new(nonzero(8), 4)?;
let shape = CompactAshShape::new(wide, nonzero(8), 4, ...)?;

let narrow = CompactAshShapeBounds::new(nonzero(4), 1)?;
let set = CandidateShapeSet::new(narrow, BTreeSet::from([shape]), false);
```

`CandidateShapeSet::new` is infallible and performs no membership validation.

`emit_candidate_bundle` iterates the actual shape set, so it can emit programs for the 8-input/4-sponsor shape while the constructor and symbol census advertise 4/1 as the candidate bounds. For example, `BundleSymbol::CandidateAshBound` is resolved from:

```rust
constructor.shapes().bounds().ash_inputs()
```

not from the maximum shape actually emitted.

### Impact

The candidate can carry two incompatible statements:

```text
declared bound: 4
emitted specialization: 8
```

That inconsistency can flow into:

- constructor symbols;
- candidate resource claims;
- ABI shape sets;
- bound reporting;
- selection policy;
- calibration studies.

This violates the rule that candidate bounds and emitted shapes are one typed fact.

### Recommendation

Make `CandidateShapeSet::new` return `Result<Self, ShapeRejection>` and validate, at minimum:

- every shape satisfies the set’s ASH bound;
- every shape satisfies the set’s sponsor bound;
- every shape’s sponsor-change relation is valid;
- no empty set where a consumer requires a candidate;
- when `sparse_counts_declared == false`, the claimed density condition actually holds.

Also validate that `ConcreteCandidate` and `CompactAshBackendPolicy::select` only operate on candidates compatible with the policy’s exact:

- shape set;
- representation;
- pattern set;
- target projection;
- layout family.

At present, `select` merely minimizes `measure`; it does not check that an offered candidate belongs to the policy selecting it.

---

## 5. [P1] Valid public shapes can overflow their sponsor range

**File:** `packages/tapscript/src/shape.rs`
**Affected function:** `CompactAshShape::sponsor_range`

The public constructors admit the full `u8` range independently for ASH and sponsor counts:

```rust
ash_inputs: NonZeroU8,
sponsor_inputs: u8,
```

But the sponsor range is computed in `u8`:

```rust
pub const fn sponsor_range(self) -> (u8, u8) {
    (
        self.ash_inputs.get(),
        self.ash_inputs.get() + self.sponsor_inputs,
    )
}
```

A value accepted by the constructors can therefore panic in a debug build and wrap in a release build:

```rust
let bounds = CompactAshShapeBounds::new(NonZeroU8::new(255).unwrap(), 1)?;
let shape = CompactAshShape::new(
    bounds,
    NonZeroU8::new(255).unwrap(),
    1,
    SponsorChangePresence::Absent,
)?;

shape.sponsor_range(); // 255 + 1
```

In release mode this becomes `(255, 0)`, causing sponsor loops and layout derivation to treat a nonempty sponsor suffix as empty or malformed.

This is not merely an extreme arithmetic issue: `inputs()` already returns `u16`, showing that the total transaction count is expected to exceed one byte, while other APIs still model positions as `u8`.

### Recommendation

Choose one consistent index domain:

- either reject shapes whose total input count exceeds `u8::MAX`, with a typed `ShapeRejection::TotalInputsAboveIndexDomain`;
- or make ranges and introspection indices `u16`/script-number values throughout.

Do not leave a safe public constructor capable of producing a value whose accessor panics or wraps.

---

## 6. [P1] The Python executor logs prohibited raw child stderr and argv-derived paths

**File:** `scripts/elements-native-executor.py`
**Affected locations:**

- `DisposableNode.call`
- startup logging in `serve`
- several framework/node exception logs

The script’s documentation says raw child stderr and caller paths do not enter first-party diagnostics. The implementation nevertheless logs both.

Raw `elements-cli` stderr is emitted verbatim, with only whitespace collapsing and truncation:

```python
log(
    "rpc %s failed with status %d, and the client said: %s"
    % (method, completed.returncode, one_line(completed.stderr))
)
```

The caller-supplied framework path is also logged:

```python
log("framework loaded from %s" % framework_path)
```

This conflicts directly with ADR-010’s field classification:

- raw stderr from an argument-supplied external executable is omitted;
- raw child argv and caller-supplied executable paths are not diagnostics;
- heuristic rewriting is not a security boundary.

The fact that the Rust harness normally routes the adapter’s stderr to `/dev/null` does not make the script’s diagnostic behavior compliant. The script can be run directly, under another supervisor, or from a failing launcher that retains stderr.

### Impact

An external child can place arbitrary content in `elements-cli` stderr, including:

- paths;
- credentials supplied outside the intended interface;
- environment-derived details;
- node or wallet diagnostics;
- terminal-control content.

The first-party executor then republishes it as its own diagnostic line.

### Recommendation

Remove raw child stderr from `log(...)` entirely. Retain it in memory only where needed for exact classification, and emit only:

- fixed method identity;
- process status;
- typed phase;
- recognized failure class.

Likewise, remove the framework-path log or replace it with fixed safe metadata such as “framework loaded”.

Add a subprocess test that makes the configured child write a unique marker on stderr and proves the marker appears nowhere in:

- executor stderr;
- protocol output;
- report fields;
- parent diagnostics.

---

## 7. [P2] The taproot oracle incorrectly rejects a zero tweak

**Files:**

- `packages/target-elements-conformance/src/constructor/curve.rs`
- `packages/target-elements-conformance/src/constructor/tree.rs`
- `packages/target-elements-conformance/src/constructor/totality.rs`

**Affected function:** `is_valid_scalar`

The constructor oracle defines a valid taproot tweak as:

```rust
pub fn is_valid_scalar(scalar: &[u8; FIELD_ELEMENT_BYTES]) -> bool {
    let value = BigUint::from_bytes_be(scalar);
    !value.is_zero() && value < *GROUP_ORDER
}
```

For taproot public-key tweaking, zero is not invalid merely because it is zero. The invalid scalar condition is overflow, \(t ≥ n\). With \(t = 0\),

\[
Q = P + 0G = P
\]

which is a valid non-identity result for a valid internal key.

The oracle separately checks whether the resulting point is the identity, which is the appropriate second failure condition. Rejecting zero before addition invents an extra target rule.

### Impact

The independent constructor oracle can reject a valid target construction and classify it as retryable:

```rust
TweakDefect::TweakNotAScalar
```

The probability of a SHA-256-derived tweak being exactly zero is negligible, but this repository explicitly requires exact totality and refuses “negligible means impossible” reasoning. The bug also distorts the totality-policy model and any exhaustive synthetic tests that deliberately state a zero tweak.

### Recommendation

Accept zero:

```rust
value < *GROUP_ORDER
```

and retain the existing post-addition identity check.

Add direct tests for:

- tweak \(0\): accepted, output key equals internal key;
- tweak \(n\): rejected;
- tweak \(n-1\): accepted unless the point sum is the identity;
- an explicitly constructed identity-result case: rejected for `TweakedKeyIsIdentity`, not `TweakNotAScalar`.

---

## 8. [P2] The “exact” taptree optimizer uses saturating arithmetic

**File:** `packages/linker/src/taptree.rs`
**Affected functions:**

- `huffman`
- `assemble`
- `exact_minimum_cost`
- `enumerated_minimum_cost`

The module describes its objective and oracle as exact, but all relevant totals use saturating `u64` arithmetic:

```rust
low.0.saturating_add(high.0)
```

```rust
weight.saturating_mul(depth)
```

```rust
best[left].saturating_add(best[right])
```

```rust
best[mask].saturating_add(total[mask])
```

`TapLeafInput` accepts arbitrary `NonZeroU64` weights. Therefore overflow is reachable through the public API.

Once saturation occurs:

- distinct candidate costs collapse to `u64::MAX`;
- Huffman pool ordering can change because multiple subtree weights become equal;
- the exact oracle can “agree” with the construction only because both lost information;
- `DeterministicTaptree::cost()` reports a value that is not the mathematical cost.

### Impact

This violates the exact-search contract and can certify a nonoptimal tree as optimal for large legal weights.

Current production callers use equal unit weights, so the demonstrated compact-ASH candidate is unlikely to trigger it. The public linker API and its assurance claim are nevertheless broader than the implementation.

### Recommendation

Use checked arithmetic and return a typed `TreeCostOverflow`, or use an exact wider domain such as `BigUint`. `u128` is sufficient only if the admitted weight and leaf-count bounds prove it sufficient; otherwise it merely moves the overflow point.

The construction and both oracles must use the same exact numeric domain, while still remaining algorithmically independent.

---

## 9. [P2] Active planning documents materially contradict the implemented tree

**Files include:**

- `plans/backlog.md`
- `plans/phases/README.md`
- `plans/packages/tapscript.md`
- `plans/packages/compiler.md`
- `plans/phases/04-compact-ash.md`
- `plans/roadmap.md`

The active planning surfaces contain mutually incompatible current-state claims.

Examples:

- `plans/phases/README.md` lists Phase 3 as **Active** and Phase 4 as **Planned**.
- `plans/roadmap.md` says the current phase is Phase 4.
- `plans/phases/04-compact-ash.md` says Phase 4 is active and Waves through 15 were delivered/audited.
- `plans/backlog.md` §3.3 says `tripod-linker`, `tripod-transaction`, and `tripod-vectors` are not implemented, although those packages and substantial implementations are present.
- The same backlog later marks the corresponding T4 work done.
- `plans/packages/tapscript.md` says the ASH constructor and relocatable bundle are not implemented, while `packages/tapscript/src/bundle.rs` implements both and the backlog records them delivered.
- The backlog’s final one-line task still says to check Phase 3 and begin Phase 4, despite earlier sections saying Phase 3 and Phase 4 passed.

Some archived guide prose may legitimately describe the historical state it was written under, but these examples are in active indexes, package contracts, or the current backlog.

### Impact

The repository currently offers different answers to basic questions such as:

- which phase is active;
- whether Phase 4 exited;
- whether the linker/transaction/vector packages exist;
- whether the tapscript candidate bundle exists.

This undermines the “one fact has one owner” planning rule and can cause later work to reopen delivered boundaries or plan against packages said not to exist.

### Recommendation

Perform a focused current-state reconciliation:

1. update the phase index from the authoritative gate record;
2. make the backlog’s implemented/not-implemented and readiness sections agree with its T4 table;
3. update package contracts to the actual public/type state;
4. replace the stale one-line backlog;
5. add plan-checker assertions for:
   - phase-index status versus active phase card;
   - package-contract status versus registered package milestone state;
   - “not implemented” package names that exist in the workspace.

Do not rewrite archived historical records; update only active owners and let archives remain dated evidence.

---

# Additional design issue worth resolving

## Bounded-depth taptree construction refuses feasible trees

**File:** `packages/linker/src/taptree.rs`

`assemble` computes an unrestricted Huffman optimum, proves that unrestricted cost against `exact_minimum_cost`, and only afterwards checks the maximum depth. It does not solve the minimum-cost tree subject to the declared depth limit.

For skewed weights, unrestricted Huffman can exceed the maximum even though a slightly more expensive bounded-depth tree exists. The implementation then returns `TreeDepthExceeded`.

This limitation is acknowledged in `plans/packages/linker.md`, so I have not treated it as an undisclosed defect above. Before the linker claims general support for bounded-depth tree inputs, the objective should either be explicitly named “unrestricted optimum, refused if too deep” or replaced by an exact length-limited construction and oracle.

---

# Overall assessment

The tree demonstrates unusually strong attention to typed boundaries, exact censuses, deterministic projections, and explicit non-claims. Several implementations are carefully structured to fail closed. However, the highest-priority issue cuts across that architecture: **canonical coverage status is currently a caller assertion, not a transcript-derived trust state**.

Recommended repair order:

1. close the forgeable coverage-discharge API;
2. reject superfluous witness encodings;
3. propagate weight mismatches as actual planner failures;
4. validate candidate shape sets against their bounds;
5. close the shape index overflow;
6. remove raw child stderr/path logging;
7. correct zero-tweak semantics;
8. make taptree cost arithmetic genuinely exact;
9. reconcile active planning state.

Because most test files were excluded from the supplied content, I cannot determine whether additional unseen tests already expose any of these paths. The code shown, however, leaves each reported path reachable through public or production-facing APIs.

# Second static review

I performed a second pass over the supplied 154-file selection at tree:

```text
0.4.2-dev
```

I deliberately looked for **additional findings** rather than repeating the first review. I found one issue capable of manufacturing false named evidence, four further trusted-boundary defects, and several determinism/protocol-completeness defects.

No Cargo, Meson, Python, or live-node command was run. Most test files, manifests, the workspace root, and several implementation modules were excluded by the supplied filter, so this is a static review of the selected content only.

---

## Summary

| ID | Priority | Finding |
|---|---:|---|
| `SR2-01` | **P0** | The canonical `sponsor-change-present` positive vector never contains sponsor change. |
| `SR2-02` | **P1** | Abstract stack validation gives an impossible success path to `EQUAL; VERIFY` over unequal known literals. |
| `SR2-03` | **P1** | The live compact-ASH lane discards the provenance-bound execution transcript and publishes an unvalidated, nondeterministic summary. |
| `SR2-04` | **P1** | A sponsored request can silently become a sponsorless transaction when the sponsor capability offers no inputs. |
| `SR2-05` | **P1** | Lifecycle and operation response validators admit role/verdict-contradictory records. |
| `SR2-06` | **P2** | Duplicate taptree inputs are resolved by “last value wins,” making construction declaration-order-dependent. |
| `SR2-07` | **P2** | Contradictory public-output views and duplicate sponsor inputs are silently collapsed. |
| `SR2-08` | **P2** | The compiler’s “complete capability census” helper cannot establish census completeness. |
| `SR2-09` | **P2** | The Python side of protocol revision 4 does not implement the protocol’s strict, bounded framing rules. |

---

# Findings

## `SR2-01` — [P0] The `sponsor-change-present` vector never contains sponsor change

**Files:**

- `packages/vectors/src/fixture.rs`
- `packages/vectors/src/materialize.rs`
- `packages/vectors/src/operation.rs`
- `packages/vectors/src/projection.rs`
- `packages/vectors/src/plan.rs`

### Problem

The positive semantic census includes distinct classes:

```text
sponsor-change-present
sponsor-change-absent
```

But the semantic fixture type cannot represent that distinction. It contains:

```rust
pub enum SponsorCase {
    Absent,
    Present(u16),
}
```

This records only whether a sponsor region exists and how many inputs it has. It has no sponsor-change-presence field.

The two relevant rows are consequently distinguished only by name and amounts:

```rust
(&[23, 29], SponsorCase::Present(1)),
(&[31, 37], SponsorCase::Present(1)),
```

Target materialization then hard-codes sponsor change to absent:

```rust
let shape = shape_of(ash_inputs, sponsors, false, id)?;
```

The out-of-process sponsor implementation does the same:

```rust
SponsorOffer::new(
    self.coins.iter().map(SponsorCoin::outpoint),
    self.fee,
    None,
)
```

The comment makes the behavior explicit:

> “No change output.”

Thus, the target transaction submitted under the canonical class named `sponsor-change-present` uses a shape with:

```text
SponsorChangePresence::Absent
```

The accepted semantic projection cannot detect this mismatch because `SponsorRegion` carries only presence and member count:

```rust
pub struct SponsorRegion {
    present: bool,
    members: u16,
}
```

It does not carry sponsor-change presence.

### Consequence

A real target can accept a transaction with no sponsor-change output, the semantic projection can match, and the run can be recorded as successful evidence for:

```text
sponsor-change-present
```

That is false named evidence. The class label is being treated as proof of a property that neither the semantic fixture, target vector, nor projection represents.

This also explains how all sponsored projections can match while the “present” case is never exercised.

### Recommended repair

Add an explicit non-amount-bearing sponsor-change dimension to the semantic fixture:

```rust
pub enum SponsorChangeCase {
    Absent,
    Present,
}
```

or include the existing backend type in a target-independent equivalent.

Then require:

```text
SponsorChangeCase::Present
⇒ sponsor member count > 0
⇒ selected shape has SponsorChangePresence::Present
⇒ transaction contains exactly one SponsorChange output role
```

The operation planner must fund a sponsor input with:

\[
\text{sponsor input} = \text{fee} + \text{change}
\]

rather than setting the fee equal to the sponsor’s entire contribution.

The accepted projection or a separate class-witness projection must retain sponsor-change presence. The fixture-plan constructor should verify the class predicate, not merely carry the class name.

Focused regressions should prove:

1. `sponsor-change-present` selects a shape with change present;
2. the resulting transaction carries the sponsor-change output role;
3. `sponsor-change-absent` carries no such role;
4. swapping the two fixtures causes a canonical-plan rejection;
5. the two target transactions remain distinct even if all protocol-semantic amounts are otherwise equal.

---

## `SR2-02` — [P1] Known unequal literals can acquire an impossible success path through `EQUAL; VERIFY`

**File:** `packages/tapscript/src/stack.rs`

**Related files:**

- `packages/tapscript/src/pattern.rs`
- `packages/target-elements/src/opcode.rs`
- `packages/target-elements/src/success.rs`

### Problem

The exact-literal machinery correctly handles verifying equality directly:

```text
PUSH x
PUSH y
EQUALVERIFY
```

because `EQUALVERIFY` has `UnequalOperands` as a failure cause and `LiteralFacts` can suppress the impossible success branch.

It does not propagate the result of the non-verifying `EQUAL` opcode as a known literal. Consider:

```text
PUSH 0x01
PUSH 0x02
EQUAL
VERIFY
```

On the real target:

```text
0x01 ≠ 0x02
EQUAL pushes false
VERIFY always aborts
```

There is no successful execution.

In the abstract validator:

1. both pushed values are present in `KnownLiterals`;
2. `EQUAL` has a successful result typed as `StackValueType::Bool`;
3. the computed Boolean is not recorded in `KnownLiterals`;
4. `VERIFY` sees an abstract `Bool`, not a known false literal;
5. `Bool` admits both false and true widths/forms;
6. the validator retains both a successful and an aborting branch.

The key loss occurs in `apply_case`:

```rust
ResultValue::Computed(value) => main.push(value.clone()),
```

A computed value’s type is retained, but its value is always forgotten. `LiteralFacts` can suppress cases and failures, yet cannot attach a known result literal to the successful state.

### Consequence

The validator can report a successful state for a program that always aborts. That can affect:

- `build_pattern`;
- pattern success contracts;
- `LeafNeverSucceeds` checks;
- candidate leaf admission;
- negative-vector comparisons;
- any future program using `EQUAL` followed by `VERIFY` rather than `EQUALVERIFY`.

A leaf such as:

```text
PUSH 0x01
PUSH 0x02
EQUAL
VERIFY
PUSH 0x01
```

can be abstractly reported as having a successful path even though the target cannot reach it.

This is the same soundness class as the earlier exact-false-literal issue, but through a computed literal rather than a pushed one.

### Recommended repair

Make result transfer capable of retaining exact computed facts. At minimum:

- for `EQUAL` with two known operands:
  - equal operands produce known true;
  - unequal operands produce known false;
- for fixed-width comparisons with known operands, propagate the exact Boolean;
- for stack operations, preserve literals as already done;
- for conversions and arithmetic, propagate an exact result only where the operation and all operands settle it exactly.

A small internal result type could make the distinction explicit:

```rust
enum AbstractValue {
    Typed(StackValueType),
    Known(StackItem),
}
```

Alternatively, extend `Transfer`/`KnownLiterals` with opcode-specific computed-literal effects.

Required regressions:

```text
PUSH 01; PUSH 02; EQUAL; VERIFY
    success = ∅

PUSH 01; PUSH 01; EQUAL; VERIFY
    success ≠ ∅

PUSH 01; PUSH 02; EQUAL
    final top is known false

PUSH 01; PUSH 01; EQUAL
    final top is known true
```

The same tests should be applied to any reviewed comparison opcode whose exact operands are known.

---

## `SR2-03` — [P1] The live compact-ASH lane discards the subject-bound execution transcript

**File:** `packages/vectors/tests/compact_ash_operation.rs`

**Related files:**

- `packages/target-elements-conformance/src/executor.rs`
- `packages/vectors/src/operation.rs`
- `packages/vectors/src/plan.rs`
- `plans/history/guide-12-completion-report.md`

### Problem

`execute_operations` returns the target-generic `ExecutionTranscript`, which retains:

- exact target projection;
- exact deployment projection;
- executor handshake;
- observed environment;
- executor trust declaration;
- exact requests;
- exact responses.

That is the value capable of binding a run to its subject.

The live integration test discards it:

```rust
let outcome = execute_operations(&target, &binding, &configuration, &mut planner);
...
let transcript = planner.transcript();
```

The rendered artifact uses only `vectors::operation::OperationTranscript`. That type contains operation-local outcomes but no:

- target projection;
- deployment projection;
- network or genesis binding;
- executor handshake;
- adapter identity;
- binary revision;
- intended tip;
- upstream base;
- local-topic census;
- exact request/response maps.

The rendered artifact also has no schema and includes ambient timing:

```rust
"wall_seconds": ...
```

So equal semantic inputs do not produce equal bytes.

No report validator recomputes the rendered object. The test writes it directly:

```rust
std::fs::write(&report, &out)
```

The code comments accurately call this a transcript rather than a gate verdict. Active completion material, however, uses these runs to support exact target-execution and coverage statements.

### Consequence

The operation artifact cannot establish:

- which executable produced it;
- which target/deployment it ran under;
- whether its operation outcomes correspond to the requests actually sent;
- whether a reported target identity was observed or caller-supplied;
- byte reproducibility.

This is especially important because `ExecutorTrust::ReviewedNonMock` is only a caller declaration. The operation path shown does not validate executor provenance against an explicit expected provenance before publishing its summary.

The stronger generic transcript exists and is thrown away at precisely the point the operation report is created.

### Recommended repair

Introduce a validated operation-report path parallel to native primitive reports:

```rust
pub struct ValidatedCompactAshOperationReport {
    report: CompactAshOperationReport,
}
```

Its constructor should consume both:

- the returned `ExecutionTranscript`;
- the planner’s operation-specific transcript.

It should recompute and compare:

- target/deployment binding;
- request/response census;
- operation step subjects;
- funding and submission results;
- projection comparisons;
- weight comparisons;
- executor provenance;
- summary counts;
- coverage changes.

The serialized report should exclude wall time from canonical bytes. Timing can be emitted as a separate noncanonical diagnostic.

If the current artifact is intentionally experimental, active planning and completion records should explicitly stop treating it as canonical or gate-eligible evidence.

---

## `SR2-04` — [P1] A sponsored request can silently build the sponsorless form

**Files:**

- `packages/transaction/src/request.rs`
- `packages/transaction/src/sponsor.rs`
- `packages/transaction/src/construct.rs`

### Problem

A `CompactAshRequest` can require sponsorship:

```rust
CompactAshRequest::new(ash, true)
```

`construct` checks only whether a sponsor capability object was supplied:

```rust
if request.sponsored() && sponsor.is_none() {
    return Err(TransactionRefusal::SponsorRequestedWithoutCapability);
}
```

It does not require that the capability’s `SponsorOffer` contain any sponsor input.

`SponsorOffer::new` admits an empty set:

```rust
SponsorOffer::new([], fee, None)
```

Construction then proceeds as follows:

```rust
let sponsors: Vec<Outpoint> = sponsor_inputs.iter().copied().collect();
```

so `sponsors.len() == 0`.

Shape selection uses the actual input count rather than the request’s `sponsored` flag:

```rust
let shape_abi = select_shape(abi, ash.len(), sponsors.len(), change_wanted)?;
```

This selects a sponsorless shape.

Because that shape has no target-fee position, the sponsor’s declared fee is ignored. No signing request is generated because there are no sponsor inputs. The resulting report identifies the sponsorless form.

### Consequence

The caller asks for:

```text
sponsored compact ASH
```

and receives:

```text
sponsorless compact ASH
```

without a refusal.

That is an undocumented fallback across transaction forms with different:

- target versions;
- relay behavior;
- fee roles;
- witness requirements;
- sponsor semantics.

It directly contradicts the documented rule that the constructor must refuse rather than silently downgrade to the sponsorless form.

### Recommended repair

Require exact consistency:

```text
request.sponsored()
⇔ sponsor capability supplied
⇔ sponsor offer has at least one input
⇔ selected ShapeAbi.form() == Sponsored
```

and the converse:

```text
!request.sponsored()
⇒ no sponsor capability
⇒ no sponsor inputs
⇒ no sponsor change
⇒ no sponsor fee role
⇒ selected form == Sponsorless
```

Add a typed refusal such as:

```rust
TransactionRefusal::EmptySponsorOffer
```

Also reject duplicate sponsor inputs before they enter a `BTreeSet`.

Focused tests should cover:

- sponsored request + no capability;
- sponsored request + capability with zero inputs;
- sponsored request + one input;
- sponsorless request + capability;
- sponsorless request + hidden nonempty offer;
- request/form equality in the construction report.

---

## `SR2-05` — [P1] Protocol response validators admit contradictory role and verdict records

**File:** `packages/target-elements-conformance/src/protocol.rs`

### A. Lifecycle responses are not checked against their role

`NativeLifecycleResponse::validate_shape` checks only whether a step that did not run nevertheless carries observations:

```rust
if !self.outcome.step_ran() && (...) {
    return Err(...);
}
```

For a step that did run, it performs no check against:

```rust
self.case.lifecycle
```

Therefore a `Construct` response may legally claim:

- `LifecycleOutcome::Verified`;
- Process-B checks;
- a Process-B spend;
- no handoff.

Likewise a `Verify` response may carry:

- a newly constructed handoff;
- creator authorization profile;
- creator witness sizes;
- creator outputs.

The module documentation claims that the two roles’ field separation is enforced by `validate_shape`; the implementation does not do so.

This is load-bearing for a fresh-process claim. If construct-side fields can appear in a verify response, the record no longer proves that Process B had only the public handoff.

### B. Operation responses permit success artifacts on rejection

`NativeOperationResponse::validate_shape` enforces that an accepted step reports something, but it does not enforce the converse.

Examples that currently pass:

```text
Submit
observed_layer = ConsensusRejectionBeforeScript
accepted_txid = Some(...)
```

```text
Fund
observed_layer = ScriptPathRejection
funded_outputs = nonempty
```

```text
SignSponsor
observed_layer = ConsensusRejectionBeforeScript
sponsor_witness = nonempty
signature_bound_to = Some(...)
```

An accepted transaction identity cannot coexist with a target rejection. A confirmed funded output cannot be an observation of a funding operation that failed. A sponsor witness should not be produced by a signing step reported as rejected.

### Consequence

The typed protocol can carry two answers to one question. Downstream code may select whichever field it happens to inspect:

- the layer says rejection;
- the attached success artifact says completion.

This is the exact response-shape ambiguity the protocol’s strict types are intended to prevent.

### Recommended repair

Make validation exhaustive over both role and outcome.

For lifecycle:

```text
Construct + Constructed:
    handoff required
    verify checks/spend forbidden

Verify + Verified:
    handoff/auth profile/creator outputs forbidden
    checks required
    spend required where the matrix says so
```

and enumerate the permitted fields for every refusal outcome.

For operation steps:

```text
Fund/FundSponsor:
    funded_outputs nonempty iff Accepted
    accepted_txid always absent
    signature fields always absent

Submit:
    accepted_txid present iff Accepted
    funding/signature fields always absent

SignSponsor:
    witness and signature_bound_to present iff Accepted
    funding and accepted_txid always absent
```

Add negative round-trip tests for every cross-kind field and every “rejected but carries success artifact” combination.

---

## `SR2-06` — [P2] Duplicate taptree declarations make input order semantic

**File:** `packages/linker/src/taptree.rs`

**Affected function:** `TaptreeInput::new`

### Problem

The constructor collects leaves directly into a `BTreeMap`:

```rust
let leaves: BTreeMap<LeafRole, TapLeafInput> = leaves
    .into_iter()
    .map(|input| (input.leaf, input))
    .collect();
```

If one `LeafRole` is supplied twice, the later value silently wins.

A duplicate may disagree in:

- `ProgramRole`;
- `weight`.

For example, these two inputs produce different `TaptreeInput` values:

```text
[A(weight 1), A(weight 10), B(weight 2)]
[A(weight 10), A(weight 1), B(weight 2)]
```

The map’s key order is canonical, but the value retained for `A` depends on declaration order.

The later “reverse the declaration order and rebuild” check cannot detect this. It reverses values after the lossy map construction has already selected a winner.

`TapLeafInput::new` also permits a mismatch such as:

```text
LeafRole::Coordinator
ProgramRole::Member
```

even though the role is derivable from the leaf identity.

### Consequence

The public tree constructor’s output can depend on declaration order despite its determinism claim. A conflicting duplicate can also alter the weighted objective and selected tree while keeping the same leaf-key set.

### Recommended repair

Build the map explicitly and reject a duplicate:

```rust
LinkRefusal::DuplicateTreeLeaf(LeafRole)
```

If exact duplicate declarations are to be tolerated, permit only typed-equal duplicates and reject conflicting ones. Rejection is simpler and more consistent with the repository’s exact-census policy.

Also derive `ProgramRole` from `LeafRole` rather than accepting it independently, or validate equality in `TapLeafInput::new`.

Required tests:

- duplicate equal leaf;
- duplicate conflicting weight;
- duplicate conflicting role;
- all declaration permutations produce one tree or the same typed refusal.

---

## `SR2-07` — [P2] Duplicate public views and sponsor inputs are silently collapsed

**Files:**

- `packages/transaction/src/view.rs`
- `packages/transaction/src/sponsor.rs`
- `packages/transaction/src/construct.rs`

### Problem

`PublicConstructionView::new` collects into a map:

```rust
outputs: outputs
    .into_iter()
    .map(|view| (view.outpoint(), view))
    .collect(),
```

Two views for one outpoint are silently resolved by keeping the later one. The documentation even says:

> “A later entry for one outpoint replaces an earlier one…”

and then describes contradictory pairs as unrepresentable. They are representable; they are resolved by iterator order.

Likewise `SponsorOffer::new` collects sponsor inputs into a `BTreeSet`:

```rust
inputs: inputs.into_iter().collect(),
```

so a caller naming one sponsor input twice is silently changed into a smaller request.

This differs from `CompactAshRequest::new`, which correctly rejects duplicate ASH outpoints before sorting.

### Consequence

Two independent statements about one chain object can enter a public construction boundary and one is discarded without a diagnostic.

For `PublicConstructionView`, order selects which asset/value/program description the constructor believes. For sponsor inputs, a duplicated request is normalized into a different sponsor family rather than refused.

Even if the target later rejects the resulting transaction, the failure is then attributed to target execution instead of the malformed construction input.

### Recommended repair

Make both constructors fallible:

```rust
PublicConstructionView::new(...) -> Result<_, TransactionRefusal>
SponsorOffer::new(...) -> Result<_, TransactionRefusal>
```

Add dedicated refusals:

```rust
ConflictingPublicOutputView(Outpoint)
DuplicateSponsorOutpoint(Outpoint)
```

Reject duplicate entries even when their values happen to be equal unless exact deduplication is a stated interface feature.

---

## `SR2-08` — [P2] `canonical_census` does not prove that `ALL` is complete

**Files:**

- `packages/compiler/src/capability.rs`
- `packages/compiler/src/target.rs`
- `packages/tapscript/src/capability.rs`

### Problem

`RequiredCapability::ALL` is documented as the complete census, and `canonical_census` claims:

> “a member added to the enum and forgotten here fails at the boundary.”

The helper checks only:

1. that the supplied census is strictly increasing;
2. that every member actually present in one analysis occurs in the census.

```rust
if let Some(unknown) = present.iter().find(|member| !census.contains(member)) {
    return Err(...);
}
```

It cannot detect an enum variant omitted from `ALL` if that variant is absent from the current analysis.

This is especially visible in:

```rust
assess_complete_census
```

which builds its “complete” input from `RequiredCapability::ALL` itself. If a new enum variant is added, handled in the exhaustive target mapping, but omitted from `ALL`, then the complete-census API silently excludes it.

The same structural issue applies to `ExternalEvidenceRole::ALL` and several downstream census APIs.

### Consequence

The type called the upper bound of all requirement sets can be incomplete while still passing every check in `canonical_census`.

Current concrete analyses may catch an omitted capability if they actually emit it, but that is weaker than the advertised compile-time completeness property.

### Recommended repair

Generate the enum and its `ALL` constant from one source. A local macro is sufficient and avoids a dependency:

```rust
capability_enum! {
    RequiredCapability {
        AuthenticatedObjectRecognition,
        ...
    }
}
```

The macro should generate:

- enum variants;
- `ALL`;
- stable rank/order where needed;
- optionally exhaustive display or projection mappings.

Do the same for other evidence-critical enums whose `ALL` arrays are used as completeness authorities.

At minimum, narrow the documentation: current code validates canonical order and inclusion of observed members, not enum completeness.

---

## `SR2-09` — [P2] The Python executor does not implement strict, bounded protocol input framing

**Files:**

- `scripts/elements-native-executor.py`
- `packages/target-elements-conformance/src/protocol.rs`
- `packages/target-elements-conformance/src/executor.rs`

### Problem

The protocol documentation says transport is strict NDJSON:

```text
one nonempty JSON object
one newline
blank records fail
unknown fields fail
each record has an explicit byte bound
```

The Rust reader enforces those rules for child output.

The Python reader does not enforce the corresponding rules for harness input.

#### Blank records are skipped

```python
for line in sys.stdin:
    if line.strip() == "":
        continue
```

The protocol says a blank record is a failure, not filler.

#### Records are unbounded

Both:

```python
sys.stdin.readline()
```

and iteration over `sys.stdin` allocate until newline or EOF. There is no protocol byte ceiling.

#### Handshake unknown fields are accepted

The handshake checks only:

```python
isinstance(handshake, dict)
handshake.get("schema") == NATIVE_PROTOCOL_SCHEMA
```

It does not require the exact field census, so a revision-4 handshake with arbitrary additional fields is accepted despite `deny_unknown_fields` on the Rust type.

#### Whitespace/framing differs by direction

The Rust side distinguishes:

- EOF;
- blank record;
- malformed JSON;
- oversized record;
- unterminated record.

The Python input side collapses or tolerates several of those.

### Consequence

The two implementations claiming protocol revision 4 do not implement one transport contract in both directions.

The canonical Rust harness happens to generate well-formed requests, so this is primarily a protocol-integrity and resource-boundary defect rather than an immediate canonical-run exploit. It becomes relevant for:

- direct use of the executor;
- alternate first-party harnesses;
- cross-language round-trip claims;
- malformed-input tests;
- any future externally consumed executor implementation.

### Recommended repair

Add request-side bounds to the typed protocol contract, not just implementation-local constants. Then implement a bounded binary record reader in Python that:

1. reads at most \(N+1\) bytes;
2. requires a terminating newline;
3. rejects blank or whitespace-only records;
4. decodes UTF-8 and JSON;
5. enforces the exact handshake/request field census;
6. distinguishes malformed, oversized, unterminated, and EOF cases.

Cross-language tests should feed the same malformed records to the Rust and Python readers and require matching refusal classes.

---

# Broader observations

## The positive-vector plan needs class witnesses, not only class names

`SR2-01` is the concrete failure, but the underlying pattern is broader. A `VectorClass` name is attached to a semantic case, and plan construction verifies census size and uniqueness, but it does not generally prove that the case exhibits the property its class names.

Some classes are naturally witnessed by their data:

- minimum two inputs;
- three inputs;
- candidate maximum;
- sum at \(2^{51}-1\).

Others require a separate executable property:

- sponsor change present;
- canonical input-order normalization;
- repeated execution gives equal bytes;
- exact successor amount.

The canonical plan should associate each positive class with a typed predicate or witness and evaluate it before admission. A class name should navigate evidence, not create it.

## The operation protocol has stronger raw material than the operation report

The generic executor layer already retains the exact binding and exchange required for a trustworthy report. The main operation-evidence defect is therefore not missing infrastructure; it is that the operation-specific path does not consume the infrastructure it already has.

That makes `SR2-03` relatively tractable: the subject-bound transcript exists and can be welded into a validated operation report without redesigning process supervision.

---

# Recommended repair order

1. **Fix `SR2-01` first.** The current canonical positive census contains a named class whose property is not present in the transaction.
2. **Close `SR2-02`.** Prevent impossible abstract success from admitting future patterns or leaves.
3. **Build the validated operation-report boundary in `SR2-03`.**
4. **Reject the sponsored-to-sponsorless downgrade in `SR2-04`.**
5. **Make response shape validation exhaustive as in `SR2-05`.**
6. **Close duplicate/order ambiguity in `SR2-06` and `SR2-07`.**
7. **Generate evidence-critical enum censuses from one source.**
8. **Unify strict protocol framing in both directions.**

---

# Overall assessment

The second pass reinforces the first review’s central theme: the repository is strongest where it uses private constructors and recomputed exact censuses, and weakest where a descriptive name or caller-provided status is allowed to stand in for an observed fact.

The most urgent new result is concrete: **the target lane can report success for `sponsor-change-present` while constructing no sponsor-change output at all**. That should be treated as a false-evidence defect, not merely as missing coverage.

