# Tenth static review — tree 0.6.1-dev

## Archival note

Two static review passes over the 51-file admission selection at tree `0.6.1-dev`, supplied by the owner and archived here verbatim as the record of what was found. The verdict is changes-requested. Neither pass was run against a build: no Cargo, Meson, or native target was executed for it.

This review opens no backlog findings register. Its actionable disposition register is `G14-R01`..`G14-R16` in [the Guide-14 draft](../guides/guide_fourteen.md) §4, dispositioned in Guide-14 Wave 0.

Three of the findings were independently confirmed at source by the orchestrator before archival: the seven-row wrong-boundary defect, the recorded-randomness digest omission, and the completed-closeout-over-empty-ledger match.

---

## Review scope and verdict

I performed a static review of the 51-file supplied concatenation at tree:

```text
0.6.1-dev
```

I did **not** run Cargo, Meson, the Python executor, or a live Elements node. The inclusion filter omitted most implementations and tests, so this is not a repository-wide assurance verdict.

**Verdict: changes requested.** The design is unusually disciplined about typed boundaries, evidence classes, construction refusals, and non-claims, but I found several concrete integrity defects. The two most serious are:

1. representation-crossing coordinator bytes are misreported by the live fragment census; and
2. recorded-randomness fixture digests do not bind confidential semantic amounts.

---

# Findings

## 1. P1 — Crossing coordinators are represented as the wrong emitted fragments

**Files**

- `packages/tapscript/src/live_pattern.rs`
- `packages/tapscript/src/live_plan.rs`

**Relevant symbols**

- `live_coordinator_program`
- `LiveFragmentId`
- `emitted_fragments`
- `every_emitted_fragment`
- `coordinator_placements`
- `patterns_for_composition`

The actual coordinator program correctly dispatches its destination-value obligation on `constructor.composition()`:

```rust
match constructor.composition() {
    LiveTransferComposition::HomogeneousExplicit => {
        // explicit conservation
    }
    LiveTransferComposition::HomogeneousPrivate
    | LiveTransferComposition::EntryBlinding => {
        // private destination form
    }
    LiveTransferComposition::ExitUnblinding => {
        // crossing destination form
    }
}
```

The fragment census does not model that dispatch. `emitted_fragments` accepts only a `LiveTransferRepresentationPlan` and reports:

```text
Explicit          → ExplicitConservation
PrivateCommitted  → PrivateDestinationForm
```

That is wrong for both crossing compositions:

| Composition | Consumed-side representation | Actual emitted obligation | Census reports |
|---|---|---|---|
| `EntryBlinding` | `Explicit` | `PrivateDestinationForm` | `ExplicitConservation` |
| `ExitUnblinding` | `PrivateCommitted` | crossing positional form | `PrivateDestinationForm` |

The mismatch is deeper than the function signature:

- `LiveTransferPatternId` contains `LiveCrossingDestinationFormV1`;
- the coordinator really emits `crossing_destination_form_fragment`;
- but `LiveFragmentId` has no crossing-destination-form member;
- `every_emitted_fragment()` therefore cannot represent the crossing fragment at all;
- `coordinator_placements()` names only explicit conservation and homogeneous private destination form.

Consequently, a placement census can validate while describing a different value obligation from the one present in the emitted coordinator bytes. This defeats the stated purpose of the census: keeping claims about emitted fragments aligned with actual programs.

### Recommended repair

1. Add a fragment identity such as:

```rust
LiveFragmentId::CrossingDestinationForm
```

2. Make the census composition-aware:

```rust
pub fn emitted_fragments(
    role: LiveProgramRole,
    composition: LiveTransferComposition,
) -> BTreeSet<LiveFragmentId>
```

3. Dispatch the value fragment using exactly the same exhaustive `match` as `live_coordinator_program`.

4. Include the crossing fragment in the relevant `CoordinatorGlobalCheck` placements, especially:

- `RepresentationSpecificConservation`;
- `DestructionAbsent`, if that slot continues to cite the representation closure.

5. Add a regression comparing, for every composition, the selected pattern identity and fragment census against the coordinator’s actual composition branch.

The central invariant should be:

\[
\operatorname{census}(\text{composition})=\operatorname{fragments actually concatenated}(\text{composition})
\]

---

## 2. P1 — Recorded-randomness fixture digests omit confidential semantic amounts

**Files**

- `packages/target-elements-conformance/src/confidential_fixture.rs`
- `scripts/elements-native-executor.py`

**Relevant symbols**

- Rust: `digest_transcript`
- Python: `confidential_digest_transcript`

The Rust module says that under `ReproducibilityContract::RecordedRandomness`, the digest binds the semantic transcript while omitting only openings that do not yet exist.

That is not what `digest_transcript` does for opening-bearing roles:

```rust
if output.role.carries_an_opening() {
    if let FixtureOpenings::Derived { openings, .. } = openings
        && let Some(opening) = openings.get(index).and_then(Option::as_ref)
    {
        transcript.quad(output.semantic_amount);
        // opening fields...
    }
} else {
    transcript.quad(output.semantic_amount);
}
```

Under `FixtureOpenings::RunProduced`, the first branch emits **no semantic amount at all** for:

- `Primary`;
- `Balancing`;
- `SoleBalancing`;
- committed `SponsorChange`;
- `BalancingSponsorChange`.

Therefore, under recorded randomness, two manifests may differ only in a confidential output’s semantic amount and still produce the same digest transcript.

For suitable manifests \(M_1\) and \(M_2\):

\[
M_1.\text{amount}\ne M_2.\text{amount}
\quad\text{but}\quad
D(M_1)=D(M_2)
\]

That means the digest does not detect exactly the semantic drift the recorded-randomness contract says it binds.

The Python mirror has the same structure: when `openings is None`, it emits neither the amount nor opening data for the outputs. The current adapter advertises only byte identity, so the Python side may not expose this today, but the two implementations encode the same latent contract defect.

### Recommended repair

Emit every output’s semantic amount unconditionally, before the contract-specific opening block:

```rust
transcript.quad(output.semantic_amount);

if output.role.carries_an_opening() {
    if let FixtureOpenings::Derived { openings, .. } = openings {
        // emit opening-only members
    }
}
```

Apply the equivalent correction in Python before recorded randomness is advertised.

Add cross-contract regressions proving:

- changing an opening changes a byte-identity digest;
- changing an opening does not change a recorded-randomness digest;
- changing a semantic amount changes **both** digests;
- changing role, asset, program, order, or profile changes both digests;
- Rust and Python derive identical transcripts for every advertised contract.

The same review should decide explicitly whether `input_blinder_sum` belongs in the recorded-randomness semantic transcript. It is currently omitted directly; under byte identity it is indirectly reflected through derived openings, but under recorded randomness it is not bound by those openings.

---

## 3. P1 — A validated closeout can cite an arbitrary string as a target acceptance

**File**

- `packages/vectors/src/live_closeout.rs`

**Relevant symbols**

- `MovedMatrixRow`
- `moved_on_acceptance`
- `validate_closeout`

The documentation says every moved matrix row is structurally grounded by “the identity the TARGET computed for a transaction the target accepted.”

The only public constructor does not validate that claim:

```rust
pub fn moved_on_acceptance(
    class: PositivePrivateClass,
    accepted_identity: impl Into<String>,
) -> Result<MovedMatrixRow, CloseoutRefusal>
```

Any string is accepted, including:

```rust
moved_on_acceptance(PositivePrivateClass::OneToOne, "")
```

`validate_closeout` checks only:

- the cleared residual set;
- pre-sighash delta;
- duplicate classes;
- disposition/ledger agreement.

It never validates the acceptance identity. Therefore a `ConfidentialFundingCloseoutReport` can validate while grounding a moved row on an empty string, malformed hex, a non-transaction label, or an unrelated value.

The tests check 64-character identities only for the repository’s own assembled report; that does not protect the public constructor.

### Recommended repair

Replace the free-form string with a validated type, ideally an existing transaction identity type or a dedicated printed-target-identity newtype:

```rust
pub struct AcceptedTransactionIdentity([u8; 32]);
```

If the report must retain printed order, parse exactly 64 hexadecimal digits at construction and render canonically. Avoid validating merely by string length.

Also validate that every `MovedMatrixRow` in `validate_closeout` carries the typed identity, rather than relying only on the helper having been used.

---

## 4. P1 — Whole coordinator patterns under-report their evidence and source dependencies

**File**

- `packages/tapscript/src/live_pattern.rs`

**Relevant symbols**

- `live_transfer_patterns`
- `LiveTransferPattern::evidence`
- `LiveTransferPattern::sources`
- `LiveCoordinatorProgramV1`
- `LiveMemberProgramV1`

The complete coordinator program concatenates:

- coordinator role;
- cardinality;
- local recognition;
- owner authorization;
- destination closure;
- sponsor isolation;
- issuance absence;
- explicit/private/crossing value obligation;
- final truth.

But when the composed patterns are created, both coordinator and member patterns receive the same manually authored source and evidence sets:

```rust
BTreeSet::from([
    Source::AuthenticatedInputObject,
    Source::AuthenticatedFamilyCensus,
    Source::InputOwnerWitness,
])
```

and:

```rust
[
    Evidence::OpcodeSemantics,
    Evidence::EncodingSemantics,
    Evidence::InputIntrospectionSemantics,
    Evidence::TransactionIntrospectionSemantics,
    Evidence::ComparisonSemantics,
    Evidence::ConversionSemantics,
    Evidence::SignatureSemantics,
    Evidence::SighashSemantics,
]
```

For the coordinator, this omits dependencies visibly present in its bytes, including, depending on composition and shape:

- `AuthenticatedOutputObject`;
- `AuthenticatedConsensusValue`;
- `OutputIntrospectionSemantics`;
- `ArithmeticSemantics`;
- `IssuanceIntrospection`;
- `FeeOutputForm`;
- `ConfidentialValueConservation`.

Individual fragment pattern records often carry those dependencies correctly, but the composed coordinator’s own `evidence()` and `sources()` do not. Since `LiveTransferPattern` documents these fields as what the pattern routes and what its correctness depends on, the whole-program record is weaker than the program it describes.

### Recommended repair

Derive a composed pattern’s sources and evidence as the union of its component pattern records, or centralize fragment metadata in one table used by both fragment and composed-pattern construction.

For example:

```rust
let evidence = components
    .iter()
    .flat_map(|component| component.evidence().iter().copied())
    .collect();
```

Do the same for sources and residuals where appropriate.

Add a regression for each composition and shape asserting:

- every opcode family used by the complete program has its required evidence role;
- every output-reading fragment contributes output-source evidence;
- explicit arithmetic contributes consensus-value source and arithmetic evidence;
- confidential or crossing contexts retain the external conservation dependency.

---

## 5. P1 — Rejected conservation responses may carry accepted-only openings

**File**

- `packages/target-elements-conformance/src/protocol.rs`

**Relevant symbol**

- `NativeConservationResponse::validate_shape`

`NativeConservationResponse::observed_openings` is documented as empty when:

- the transaction was never built;
- the target did not accept it;
- no confidential output is present.

The shape validator only forbids observations when `observed_layer.is_target_verdict()` is false:

```rust
if !self.observed_layer.is_target_verdict()
    && (... || !self.observed_openings.is_empty())
{
    return Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation);
}
```

A target rejection is a target verdict. Consequently, this passes shape validation:

```text
observed_layer = ConsensusRejectionBeforeScript
observed_openings = nonempty
```

That is precisely the impossible provenance the surrounding documentation warns about: openings are described as read back from outputs a confirmed transaction created, but a rejected transaction created no such outputs.

The validator also does not require an accepted conservation response to carry materialized transaction bytes, although an acceptance necessarily implies a transaction was built.

### Recommended repair

Add acceptance/refusal-specific rules:

- `Accepted` must carry the minimum observation required by the conservation record, including transaction bytes;
- any rejecting target verdict must carry no `observed_openings`;
- transaction bytes and decoded commitments may remain permitted on rejection where they describe the submitted candidate rather than accepted outputs.

Prefer a dedicated `ResponseShapeDefect` if reusing `RefusedOperationCarriesObservation` would blur operation and conservation records.

Add explicit tests for:

- accepted response with no transaction;
- consensus rejection with openings;
- relay-policy rejection with openings;
- infrastructure failure with any target observation;
- rejection with transaction bytes but no openings, where that shape is intentionally valid.

---

## 6. P2 — Operation resource absence is still serialized as a measured zero

**Files**

- `packages/target-elements-conformance/src/protocol.rs`
- `scripts/elements-native-executor.py`
- `plans/guides/guide_thirteen_feature_requests.md`

**Relevant symbols**

- `NativeResourceObservation`
- `answer_operation_step`
- `write_operation_failure`

`NativeResourceObservation` makes these fields mandatory numeric values:

```rust
pub script_bytes: u64,
pub initial_stack_items: u64,
```

The Python operation path does not measure either one and writes:

```python
"script_bytes": 0,
"initial_stack_items": 0,
```

for both ordinary operation responses and infrastructure failures.

The adapter’s own comments acknowledge that no interpreter observation is made. Nevertheless, the wire representation turns absence into zero, and a consumer cannot distinguish:

```text
measured value = 0
```

from:

```text
no measurement exists
```

This is already recognized in Guide-13 feature request 4, but remains present in the current protocol and executor.

### Impact

- resource reports can accidentally treat absence as agreement;
- infrastructure failures carry numbers that look like observations;
- the protocol’s “absent is not zero” doctrine is not true for these two fields.

### Recommended repair

Make both presence-bearing:

```rust
pub script_bytes: Option<u64>,
pub initial_stack_items: Option<u64>,
```

and write JSON `null` when unmeasured.

Because this changes record shape and old peers may interpret absence differently, update the protocol revision on both sides together. Add cross-language tests proving:

- primitive fixtures report measured values;
- operation submissions report absence unless actually measured;
- infrastructure failures report absence;
- no validator normalizes `None` to zero.

---

## 7. P2 — The typed diagnostic stream admits path disclosure and line injection

**File**

- `scripts/elements-native-executor.py`

The module states that the typed diagnostic stream contains only bounded, adapter-authored facts and that no caller configuration path is interpolated into it.

At least one direct violation remains:

```python
log("confidential materializer ready at %s" % library)
```

`library` can come directly from the caller’s `--zk-library` argument or be derived from the caller-selected `elementsd` path. This publishes an operator filesystem path in the typed stream despite the module’s explicit prohibition.

There is also a broader integrity problem: `DiagnosticStreams.typed` writes strings without escaping embedded newlines:

```python
self.output.write("%s: %s\n" % (COMMAND_NAME, message))
```

Several fatal and adapter errors interpolate request-controlled field names or values before reaching `log`, for example:

```python
raise FatalAdapterError(
    "the harness sent a request field named %s" % key
)
```

A JSON object key may contain a newline. Such a request can inject additional apparent diagnostic lines into `--output`, defeating the “typed facts only” format.

### Recommended repair

- Never log `library`, framework paths, executable paths, or arbitrary exception text in the typed stream.
- Use fixed diagnostic codes plus typed numeric or enum fields.
- Quarantine arbitrary field names and third-party exception messages, referring to them by record number.
- Make `DiagnosticStreams.typed` reject or escape `\r` and `\n` as defense in depth.
- Add marker tests with field names and paths containing newlines, tabs, and credential-shaped text.

---

## 8. P2 — The resource study still reports a blocker that the repository says is cleared

**File**

- `packages/vectors/src/live_measurements.rs`

The current evidence code says `OwnerSighashNotComputable` is carried by zero rows and the owner message is computable. Multiple recorded target acceptances use real signatures over independently recomputed messages.

The resource study nevertheless assigns every measured transaction:

```rust
DimensionStanding::NotClaimable(
    LiveResourceNonClaim::NoTargetVerdictExists(
        LiveInfrastructureBlocker::OwnerSighashNotComputable,
    ),
)
```

The exact measured transactions do have no target verdict, but not because the component is absent. The study deliberately fills signature slots with `UNAUTHORIZING_SIGNATURE`.

That is an important distinction:

```text
component does not exist
≠
this measurement lane deliberately did not use it
```

The module also still describes confidential proof bytes as unavailable because its synthetic private construction serializes no proof, while the current proof-bearing lane does serialize and submit real range proofs.

### Recommended repair

Choose one of two honest dispositions:

1. **Current study:** rebuild measurements through the current proof-finalized and real-signing paths; or
2. **Synthetic sizing study:** retain the current transactions but replace the stale global blocker with lane-specific standings such as:
   - `UnauthorizingWitnessUsedForSizing`;
   - `SyntheticPrivateConstructionSerializesNoProof`;
   - `NoTargetRunForTheseExactBytes`.

Do not use `LiveInfrastructureBlocker::OwnerSighashNotComputable` for a component that the same repository says exists and has accepted runs behind it.

---

## 9. P2 — Current-state and assurance documentation materially contradicts the code

This is broader than ordinary comment drift because several of the affected texts are presented as current evidence or operating state.

### Examples

#### `plans/backlog.md`

The header and phase index say Phase 5 exited and Phase 6 is current, but §11 says:

```text
Phase 5 is not exited
```

The “latest static review” section still names the sixth review as current, even though the ninth review and its remediation appear later in the same file.

#### `packages/vectors/src/live_evidence.rs`

The module header says one positive row remains at `NativeRunRequired`:

```text
projection-equality-with-paired-explicit
```

But the implementation now contains `PairedRelationObserved`, the pair arc has two accepted identities, and the tests assert all 26 positive rows are answered through:

```text
24 native acceptances
+ 1 determinism observation
+ 1 paired relation
```

#### `packages/vectors/src/live_closeout.rs`

The module header and refusal documentation say the sponsor row cannot enter the delta, but:

```rust
PositivePrivateClass::may_enter_the_delta(self) -> bool {
    true
}
```

and the completed closeout includes `PrivateSponsorValues`.

`CloseoutRefusal::SponsorRowMoved` is therefore effectively historical/unreachable, while comments still describe it as a current invariant.

#### `scripts/elements-native-executor.py`

The opening documentation says:

```text
This adapter speaks protocol revision 4.
```

while both the Rust and Python constants are revision 6.

#### Archived execution guides

Several guides still open with “not yet executed” even though current planning records describe them as completed and archived. If preserving original text is intentional, their headers should explicitly identify them as historical snapshots rather than current execution status.

### Recommended repair

Update the smallest owning documents, preserving historical records where policy requires but clearly time-scoping them. In particular:

- correct the backlog’s current-gate section;
- refresh `live_evidence`’s headline census;
- remove or mark historical the obsolete sponsor-row prohibition;
- correct the executor’s revision statement;
- distinguish archived-original status from current project status.

Add focused consistency tests for facts already represented in typed code where practical; prose totals and protocol revisions should be rendered from or checked against their owning constants rather than maintained independently.

---

# Additional observations

## Stale refusal vocabulary in the closeout module

`CloseoutRefusal::SponsorRowMoved` remains in the public vocabulary after every positive private class became admissible to the delta. Keeping historical vocabulary can be reasonable, but the variant is not marked historical and its docs state a rule that no longer exists. Either:

- retire it through an explicit schema change; or
- document it as a historical compatibility variant and stop presenting it as reachable validation behavior.

## Validation asymmetry is a recurring risk

Several areas correctly derive instruction prerequisites but manually author adjacent evidence metadata. The crossing-fragment and composed-pattern findings are both instances of the same structural problem:

```text
actual bytes derived one way
claim census authored another way
```

Where possible, derive:

- fragment identities;
- evidence dependencies;
- source dependencies;
- disclosures;
- resource dependencies;

from the same composition record used to emit the program. Exhaustive matches help only when every consumer matches on the same discriminant.

---

# What is strong

Several design choices are notably good and should be preserved during repair:

- strict separation of construction failures, infrastructure failures, and target verdicts;
- typed, exhaustive operation and evidence vocabularies;
- mutant-first/control-last ordering for attribution;
- exact distinction between target acceptance, determinism observations, paired relations, and first-party facts;
- explicit public-disposable-test-material classification;
- no claim that a refused key-path attempt proves discrete-log unspendability;
- role-keyed fixture evolution that attempts to preserve existing digest bytes;
- repeated use of private fields and validated construction transitions instead of caller-authored booleans;
- honest distinction between source-derived possibility and observed acceptance.

The main repairs should preserve that architecture while eliminating the places where a manually maintained claim has drifted from the bytes or semantics it is supposed to describe.

## Suggested fix order

1. Fix recorded-randomness digest binding.
2. Make fragment accounting composition-aware.
3. Repair composed-pattern source/evidence derivation.
4. Type and validate closeout acceptance identities.
5. Tighten conservation-response shape validation.
6. Correct resource absence on the wire with a coordinated revision.
7. Harden the Python diagnostic stream.
8. Reclassify or update the stale resource study.
9. Reconcile current-state documentation and obsolete public refusal vocabulary.

# Second static review

## Scope

I treated this as a fresh pass over the same 51-file concatenation at:

```text
0.6.1-dev
```

No Cargo, Meson, Python, or live-node execution was performed. Most repository files—including many implementations and tests—remain outside the supplied filter, so findings are limited to what the selected files establish.

## Verdict

**Changes requested.**

The first review’s findings remain reproducible in this same snapshot. This second pass found an additional gate-level evidence problem: **seven safety rows declared as script-path obligations are counted as answered by transactions that consensus rejected before any script ran.** That conflicts directly with the matrix’s own boundary contract and affects the Phase-5 exit assessment.

I also found independent integrity gaps in the owner-signing census, closeout validation, proof-negative attribution, and restart ledger.

---

# New findings

## 1. P1 / gate-blocking — Seven rows are counted as answered at the wrong evidence boundary

**Files**

- `packages/vectors/src/live_safety.rs`
- `packages/vectors/src/live_evidence.rs`
- `packages/vectors/src/live_owner_signing_negatives.rs`
- `packages/vectors/tests/guide13_live_native.rs`
- `plans/phases/05-live-transfer.md`

### The declared contract

`LiveSafetyRow` records an expected `EvidenceBoundary`, and its documentation is explicit:

> A row that did not say in advance where its verdict belongs could be “passed” by a refusal at any layer at all.

It also states that a refusal at a different boundary is a finding, not a pass.

The following seven rows declare `EvidenceBoundary::ScriptPathRejection`:

1. `wrong-explicit-asset`
2. `confidential-asset-commitment`
3. `output-total-one-below-input`
4. `output-total-one-above-input`
5. `private-output-omitted`
6. `hidden-private-u-output`
7. `omitted-source`

### What was actually observed

The owner-signing negative ceremony explicitly says these mutations break per-asset conservation and are rejected before script execution. Its native test asserts:

```rust
Some(ObservedOutcomeLayer::ConsensusRejectionBeforeScript)
```

for every one of those seven mutants.

The ceremony’s documentation also correctly acknowledges the consequence:

> the consensus verdict is not the row’s declared script class

and:

> a mutant refused `bad-txns-in-ne-out` reached no leaf

### Where the evidence model loses the mismatch

`LiveRowStanding::NativeRefusalObserved` stores only:

```rust
control_identity
refusal_detail
```

It stores neither:

- the observed outcome layer; nor
- the expected matrix boundary.

Correspondingly, `observed_row_refusal` returns only:

```rust
Option<(&'static str, &'static str)>
```

and `classify` converts that directly into an answered standing without comparing the actual layer with `row.refusing_layer()`.

Thus:

\[
\text{expected script-path refusal}\ne\text{observed consensus refusal}
\]

but the classifier records the row as answered anyway.

### Impact

This is not just missing metadata. For these transactions, the intended script carrier did not execute. The repository’s own translation-validation rules require the carrier to be reached before a target refusal can discharge that relation.

The Phase-5 assessment counts these seven rows among its 17 `NativeRefusalObserved` answers and then claims the matrix is disposed according to each row’s own gate. On the matrix’s current declarations, that is false.

At least these seven rows must either:

1. remain unanswered, with the unexpected earlier refusal recorded separately; or
2. be authoritatively retyped to `ConsensusRejectionBeforeScript`.

The observation alone cannot silently perform option 2.

### Recommended repair

Add the layer to the standing:

```rust
NativeRefusalObserved {
    control_identity: &'static str,
    observed_layer: ObservedOutcomeLayer,
    refusal_detail: &'static str,
}
```

Then implement an exact mapping between target observations and declared boundaries:

```rust
fn observed_boundary(layer: ObservedOutcomeLayer) -> Option<EvidenceBoundary>
```

During classification, require:

```rust
observed_boundary(observed_layer) == row.refusing_layer()
```

Where they differ, use a non-answer standing such as:

```rust
NativeRefusalAtUnexpectedBoundary {
    expected: EvidenceBoundary,
    observed: ObservedOutcomeLayer,
    control_identity: &'static str,
    refusal_detail: &'static str,
}
```

`is_answered()` must return `false` for that standing.

Add a regression that iterates every `NativeRefusalObserved` row and checks exact boundary equality. The seven rows above should initially fail that test, forcing an explicit disposition.

---

## 2. P1 — The owner-signing census does not fully bind itself to its candidate and selected leaf

**File**

- `packages/transaction/src/live_census.rs`

**Relevant symbols**

- `OwnerSigningCensus::over_foreign_bytes_for_negative_evidence`
- `OwnerSigningCensus::from_explicit_finalized`
- `OwnerSigningCensus::assemble`
- `OwnerSigningInputRequest`
- `OwnerCensusRefusal::ProtectedBytesAreNotTheCandidates`

There are two related binding holes.

### 2.1 Foreign protected bytes are accepted without comparison to the candidate

The negative-evidence route accepts both:

```rust
candidate: TargetTransaction
protected_bytes: Vec<u8>
```

`assemble` stores both but never checks that `protected_bytes` are the protected encoding of `candidate`.

The existing refusal:

```rust
OwnerCensusRefusal::ProtectedBytesAreNotTheCandidates
```

does not close this. `check_offered` only compares later offered bytes with the bytes already stored in the census:

```rust
if offered == self.protected_bytes
```

If a caller constructed the census with unrelated protected bytes, offering those same unrelated bytes passes `check_offered`.

Therefore a census can retain:

```text
candidate structure from A
protected byte binding from B
```

while still being a successfully constructed `OwnerSigningCensus`.

This contradicts the route’s documentation that it runs the same clause list and binds to the mutated candidate’s own bytes.

### 2.2 Production requests are not bound to the exact leaf selected by finalization

`from_explicit_finalized` takes a finalized value, but it also accepts caller-authored `OwnerSigningInputRequest` entries.

`check_leaf_commits` proves only that the requested leaf commits under the spent output’s taptree. It does not prove that the request names the exact leaf script and control block selected in the matching `ReceiptInputRecord`.

If a tree contains several valid leaves, a caller can potentially provide another committed leaf and control path. The commitment check passes because the other leaf genuinely belongs to the tree, but the census then computes the message for a leaf different from the one finalization intends to place in the witness.

The same problem applies to:

```rust
codeseparator_position
```

The selected profile fixes:

```rust
OWNER_CODESEPARATOR_POSITION = 0xffff_ffff
```

because emitted leaves contain no moving code separator, but `assemble` never compares the request’s position with that constant.

A malformed caller can therefore obtain a valid census for a message the eventual target witness does not execute. The resulting target refusal would occur at signature verification and could be mistaken for evidence about a later mutation.

### Recommended repair

For production routes, do not accept independently authored leaf requests where finalization already owns the answer.

Either derive the census requests internally from `FinalizedLiveTransfer`, or validate each request against the exact finalized receipt record:

```text
request.input_index            = record.position
request.tapleaf_hash           = leaf_hash(record.leaf_script)
request.leaf_version           = TAPSCRIPT
request.codeseparator_position = OWNER_CODESEPARATOR_POSITION
request.control_block          = record.control_block
```

For the foreign-byte negative route:

- derive protected bytes from the candidate inside the function; or
- accept a typed representation/treatment and independently recompute the expected protected encoding before construction.

A raw `(candidate, protected_bytes)` pair should not produce a trusted census without an exact binding check.

Add focused negative tests for:

- candidate A with protected bytes from B;
- another committed leaf from the same tree;
- correct leaf with a wrong code-separator position;
- correct leaf hash with another valid control path;
- a production route attempting the leaf rearrangement that only the explicitly named negative-evidence route should admit.

---

## 3. P1 — A closeout can validate as `Completed` with an empty or partial restart ledger

**Files**

- `packages/vectors/src/live_closeout.rs`
- `packages/vectors/src/live_restart.rs`

**Relevant symbols**

- `validate_closeout`
- `CloseoutDisposition`
- `RestartLedger::stopped_at`
- `RestartLedger::expected_step`

The closeout checks disposition agreement using only:

```rust
let ledger_stopped_at = parts.ledger.stopped_at();
let agrees = match (&parts.disposition, ledger_stopped_at) {
    (CloseoutDisposition::Completed, None) => true,
    ...
};
```

But `RestartLedger::stopped_at()` is also `None` when:

- no step has been recorded;
- one accepted step has been recorded;
- any proper accepted prefix shorter than all seven steps has been recorded.

Therefore, after satisfying the unrelated residual and delta checks, this can validate:

```text
disposition = Completed
ledger       = empty
stopped_at   = None
```

An incomplete order and a completed order are currently indistinguishable through the property `validate_closeout` uses.

### Additional stop-integrity defect

For `TypedStopped`, validation compares only the step:

```rust
*step == stopped
```

It does not compare the claimed blocker with the blocker in the ledger’s `RestartStepResult::StoppedTyped`.

A closeout can therefore claim:

```text
stopped at step S because of blocker B₁
```

beside a ledger that records:

```text
stopped at step S because of blocker B₂
```

and still validate.

### Impact

`ConfidentialFundingCloseoutReport` is documented as being assembled from, rather than independently authored beside, its ledger. These two cases violate that central invariant.

### Recommended repair

Expose a complete ledger status:

```rust
pub enum RestartLedgerStatus<'a> {
    InProgress { expected: RestartStep },
    Stopped {
        step: RestartStep,
        blocker: LiveInfrastructureBlocker,
    },
    Completed,
}
```

`Completed` should require exactly all seven steps, each with an accepted result.

Then match exactly:

```rust
match (&parts.disposition, parts.ledger.status()) {
    (Completed, RestartLedgerStatus::Completed) => ...
    (
        TypedStopped { step, blocker },
        RestartLedgerStatus::Stopped {
            step: actual_step,
            blocker: actual_blocker,
        },
    ) if step == actual_step && blocker == actual_blocker => ...
    _ => refusal,
}
```

Required regressions:

- empty ledger plus `Completed` refuses;
- partial accepted ledger plus `Completed` refuses;
- full accepted ledger plus `Completed` succeeds;
- same stop step with another blocker refuses;
- same blocker at another step refuses;
- exact typed stop succeeds.

---

## 4. P1 — The “wrong blinder” proof-negative attributes a serialized commitment mutation to a nonexistent blinder field

**Files**

- `packages/vectors/src/live_restart.rs`
- `packages/vectors/src/live_conservation_negatives.rs`

**Relevant symbols**

- `ProofNegativeCase::WrongBlinder`
- `ProofNegativeCase::mutated_field`
- `MutatedField::ValueBlinder`
- `build_mutants`
- `attribute_proof_negative`

The wrong-blinder mutant is constructed by recomputing an output’s value commitment with another blinder and replacing the serialized 33-byte commitment:

```rust
replace_output_value(control, MUTATED_OUTPUT, wrong_bytes)
```

The declared byte range is accordingly the serialized value-commitment field.

But the case reports:

```rust
ProofNegativeCase::WrongBlinder => MutatedField::ValueBlinder
```

No value-blinder field exists in the serialized transaction. What changed in the target bytes is the commitment, not a blinder.

The rendered result can therefore say, in effect:

```text
mutated field: value-blinder
declared byte range: 33-byte value commitment
```

Those are different facts.

### Why it matters

There are two useful classifications here:

1. **construction cause:** the commitment was generated from a wrong blinder;
2. **serialized mutation:** the value commitment bytes changed.

Conflating them weakens attribution and makes the exact-field claim literally false.

It also obscures the distinction from `PrivateCtImbalance`, which changes a value commitment for a different construction reason.

### Recommended repair

Carry both dimensions:

```rust
pub enum ProofNegativeCause {
    WrongValueBlinder,
    WrongCommittedValue,
    MissingRangeproof,
    MalformedRangeproof,
}

pub enum SerializedMutatedField {
    ValueCommitment,
    RangeproofBytes,
}
```

Then:

```text
WrongBlinder:
    cause = WrongValueBlinder
    serialized field = ValueCommitment

PrivateCtImbalance:
    cause = WrongCommittedValue
    serialized field = ValueCommitment
```

Their distinct output positions and ranges remain the separating facts; the taxonomy no longer claims a blinder is present on the wire.

---

## 5. P1/P2 — Report-layer “answerability” is counted as completed evidence without a report observation

**File**

- `packages/vectors/src/live_evidence.rs`

**Relevant symbols**

- `LiveRowStanding::ReportLayerAnswerable`
- `LiveRowStanding::is_answered`
- `classify`
- `LiveEvidenceCensus::every_required_row_is_answered`

For any row whose declared boundary is:

```rust
EvidenceBoundary::ReportSemanticProjectionRejection
```

`classify` immediately returns:

```rust
LiveRowStanding::ReportLayerAnswerable
```

No report bytes, validated report, renderer result, or checked exclusion are passed into the classifier.

Nevertheless, `is_answered` includes:

```rust
Self::ReportLayerAnswerable
```

as completed evidence.

The standing’s own name describes a capability:

```text
this row can be answered at the report layer
```

not an observation:

```text
these exact report bytes were checked and answered it
```

This is the same distinction the rest of the evidence model carefully preserves:

```text
answerable ≠ answered
capability ≠ evidence
```

### Affected rows

The two sponsor-report rows are counted in the answered total merely because they name a report boundary:

- `report-publishes-sponsor-amount`
- `report-publishes-sponsor-opening`

Within the selected evidence plan, no validated report value is retained to substantiate either result.

There may be report tests in excluded files, but this standing does not bind to them. Even if such tests exist, the evidence plan’s answer is structurally independent of whether they passed.

### Recommended repair

Split the states:

```rust
ReportLayerRequired
```

and:

```rust
ReportLayerObserved {
    report_schema: ...,
    checked_bytes: ...,
    result: ...,
}
```

Only the observed/validated state should make `is_answered()` true.

If the property is guaranteed structurally by a sealed renderer rather than by inspecting bytes, represent it as first-party evidence with the exact validator or type-level fact, not as bare “answerability.”

Add a mutation test that constructs or simulates a report containing a sponsor amount/opening and proves the standing no longer reports the row answered.

---

## 6. P2 — Proof-negative attribution trusts a caller-authored byte range broad enough to excuse any mutation

**File**

- `packages/vectors/src/live_restart.rs`

**Relevant symbol**

- `attribute_proof_negative`

The function accepts:

```rust
declared_field_range: (usize, usize)
```

and checks only that the observed changed interval lies inside it:

```rust
if observed_start < declared_start || observed_end > declared_end {
    // refuse
}
```

It does not validate that:

- `declared_start < declared_end`;
- `declared_end <= control_bytes.len()`;
- the range corresponds to the field named by `ProofNegativeCase`;
- the range was derived from the control’s decoded structure rather than chosen by the caller.

A caller can supply:

```rust
(0, usize::MAX)
```

and any non-identical mutation will satisfy the containment test.

Thus the API does not itself establish “exactly one field.” It establishes only:

```text
the mutation fits in the range the caller claimed
```

where the caller may claim the entire address space.

### Recommended repair

Best: derive field ranges from a decoded control and a typed locator:

```rust
pub struct LocatedMutationField {
    field: SerializedMutatedField,
    output: usize,
    range: Range<usize>,
}
```

Make construction private and obtain it from the transaction decoder/locator.

If a public range must remain:

- require a valid nonempty in-bounds range;
- bind it to a typed field locator;
- compare it with an independently derived expected range;
- refuse a case/field mismatch.

Add tests for:

- reversed range;
- empty range;
- end past control length;
- whole-transaction range;
- correct field kind at the wrong output;
- correct range paired with the wrong `ProofNegativeCase`.

---

## 7. P2 — `RestartLedger::record` can panic after successful completion

**File**

- `packages/vectors/src/live_restart.rs`

After all seven steps have been recorded as accepted:

```text
stopped_at = None
entries.len() = RestartStep::ALL.len()
expected_step() = None
```

A subsequent public call to `record` reaches:

```rust
let expected = self
    .expected_step()
    .expect("a ledger that has not stopped and is not full expects a step");
```

The assertion’s premise is false: the ledger is full.

The method’s documentation says the function never panics, but a caller can reach this state through the public API without corruption.

### Recommended repair

Add:

```rust
RestartOrderRefusal::AlreadyComplete {
    attempted: RestartStep,
}
```

and check completion before unwrapping `expected_step()`.

Regression:

```rust
let mut ledger = fully_accepted_ledger();
assert_eq!(
    ledger.record(...),
    Err(RestartOrderRefusal::AlreadyComplete { ... }),
);
```

This repair also supports the complete/in-progress distinction needed by finding 3.

---

# Confirmation of the first review

Because the supplied tree reference is unchanged, I cross-checked the first pass’s findings. They remain present.

| First-review finding | Second-pass status |
|---|---|
| Crossing coordinator fragment census reports the consumed plan instead of the composition | **Confirmed** |
| Recorded-randomness fixture digest omits confidential semantic amounts | **Confirmed** |
| Closeout rows accept arbitrary strings as target identities | **Confirmed** |
| Composed coordinator patterns omit output/arithmetic/issuance evidence dependencies | **Confirmed** |
| Rejected conservation responses may carry accepted-only openings | **Confirmed** |
| Operation resource absence is encoded as numeric zero | **Confirmed** |
| Python typed diagnostics can disclose configuration paths and admit line injection | **Confirmed** |
| Resource study still cites the cleared global sighash blocker | **Confirmed** |
| Current-state documentation materially disagrees with current code | **Confirmed** |

The new closeout finding is stronger than the earlier identity finding: even with perfectly typed acceptance identities, the current validator can still certify an empty ledger as completed.

---

# Revised priority order

I recommend repairing in this order:

1. **Stop counting wrong-boundary refusals as answered.**
   - This directly affects the safety census and Phase-5 exit assessment.

2. **Bind `OwnerSigningCensus` to one candidate, protected encoding, and selected leaf.**
   - Otherwise target refusals can be produced from a message different from the witness actually executed.

3. **Make closeout completion derive from a genuinely complete ledger.**
   - Also compare the exact typed-stop blocker.

4. **Repair composition-aware fragment accounting.**
   - Ensure crossing coordinator claims describe actual bytes.

5. **Repair recorded-randomness digest semantics.**
   - Semantic amounts must affect both contracts.

6. **Separate proof-negative construction causes from serialized mutated fields.**

7. **Derive mutation ranges rather than trusting caller-provided intervals.**

8. **Require actual validated report evidence for report-layer rows.**

9. **Then close the lower-level protocol, diagnostic, resource, and documentation findings from the first review.**

---

# Suggested gate regressions

A compact set of high-value tests would be:

```text
a native refusal answers a row only at its declared boundary

a consensus-before-script refusal cannot answer a script-path row

a foreign-byte census rejects candidate/protected-byte disagreement

an explicit finalized census rejects another committed leaf from the same tree

an explicit finalized census rejects a noncanonical codeseparator position

an empty restart ledger cannot validate a completed closeout

a partial accepted restart ledger cannot validate a completed closeout

a typed stop must name the ledger’s exact blocker

recording after a completed restart ledger returns AlreadyComplete

wrong-blinder attribution names the serialized value-commitment field

an unbounded caller-declared mutation range is refused

a report-layer row remains unanswered until validated report evidence exists
```

## Final assessment

The repository’s strongest quality remains its insistence on distinctions—construction versus execution, target verdict versus first-party fact, acceptance versus determinism, and semantic meaning versus serialized representation. The principal defects found in both reviews are places where those distinctions exist in prose or type names but are lost at one conversion point.

The most important instance is now clear:

\[
\text{observed refusal}\land\text{wrong boundary}\not\Rightarrow\text{row answered}
\]

Until that invariant is encoded, I would not rely on the current 82-answer safety census or the Phase-5 exit assessment derived from it.
