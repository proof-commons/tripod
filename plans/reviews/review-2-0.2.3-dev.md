# Static review

Overall, this is an unusually disciplined repository. The separation among Attestation, realization, architecture, model, compiler analysis, target evidence, and release evidence is strong; the code generally follows the stated fail-closed and deterministic-design principles. In particular, the exact canonical partitions, typed evidence boundaries, owner-aware label graph, checked arithmetic, and explicit “model validity ≠ deployment readiness” language are all very good.

I found one current CI-gating defect and several correctness/API issues worth fixing before Phase 2 closes.

## Findings

### 1. High — `scripts/ci.sh` can report “CI green” without running the tracked-entry mode audit

**Locations**

- `scripts/ci.sh`, lane 10
- `scripts/test-meson-mock.sh`
- top-level `meson.build`, custom target `census-audit`

**Problem**

`scripts/ci.sh` describes the mocked Meson lane as the check for:

> the hand-managed ADR-014 census … and repository shape

But `scripts/test-meson-mock.sh` only invokes explicit targets such as:

```sh
meson compile -C "$build" attestation
meson compile -C "$build" generate-artifacts
```

The `census-audit` target is `build_by_default`, but it is not a dependency of the explicit `attestation` or `generate-artifacts` targets. With an explicit Ninja target, unrelated default targets are not built. Consequently, the mocked Meson lane does not execute `census-audit`.

The label checker indirectly catches many stale hand-maintained subject lists because its on-disk discovery is compared with Meson-supplied arguments. It does **not**, however, replace the complete Git-mode audit. In particular, only `census-audit` enforces over the complete tracked set:

```text
100644
100755
```

and rejects tracked symlinks (`120000`) and gitlinks (`160000`), including categorically excluded paths.

Therefore a clean repository containing a tracked symlink or gitlink in an otherwise excluded area may pass `scripts/ci.sh` and still produce:

```text
CI green
```

The later full `meson test -C build` should execute the audit through its test dependency, but `scripts/ci.sh` explicitly presents itself as a runner-agnostic CI gate and supports protected use through `CI_REQUIRE_MESON=1`. That claim is currently stronger than the command graph it runs.

**Recommended fix**

Make the mocked contract invoke the audit explicitly, for example:

```sh
meson compile -C "$build" census-audit
```

and assert that both outputs exist and are well-formed:

```text
census.ok
census-audit.json
```

Alternatively, perform one unqualified default build in the mocked tree before the focused repair tests:

```sh
meson compile -C "$build"
```

The explicit audit is likely cheaper and more focused.

Also update the mock-contract test to prove the audit actually executed—checking the output alone is preferable to relying on `build_by_default`.

---

### 2. Medium — `SuccessionOrTermination` root policies reject valid succession observations

**Locations**

- `packages/model/src/conformance.rs`, `observe_root_effects`
- `packages/realization/src/evaluate.rs`, `root_policy_holds`
- `packages/realization/src/relation.rs`, `Relation::RootPolicy`

**Problem**

The architecture’s `RootUse` value is a **policy**:

```rust
Forbidden
Succession
SuccessionOrTermination
```

A transition certificate’s `RootEdge` is an **actual effect**:

```rust
Succ { ... }
Term { ... }
```

The conformance adapter currently maps actual effects back into policy values:

```rust
RootEdge::Succ { .. } => architecture::RootUse::Succession,
RootEdge::Term { .. } => architecture::RootUse::SuccessionOrTermination,
```

Then the realization evaluator requires exact equality:

```rust
if actual != expected {
    return false;
}
```

That is wrong for an operation whose declared policy is `SuccessionOrTermination`. For example, `redeem` declares:

```text
RESV: succession-or-termination
```

A normal, non-sealing redemption produces `RootEdge::Succ`. The observation therefore reports `RootUse::Succession`, while the expected policy is `RootUse::SuccessionOrTermination`; exact comparison fails even though succession is explicitly permitted.

A sealing redemption happens to pass because termination is encoded as `SuccessionOrTermination`. Thus the current encoding makes one branch of the same policy pass and the other fail.

This is not reached by the present Phase-1 realization scope—compact ASH and live transfer use no roots—but it will block correct conformance as soon as redemption enters realization scope.

**Recommended fix**

Do not use `RootUse` for observed effects. Introduce a separate realization-owned type such as:

```rust
pub enum ObservedRootEffectKind {
    Succession,
    Termination,
}
```

Then validate policy compatibility:

```text
Forbidden               accepts no observed effect
Succession              accepts Succession only
SuccessionOrTermination accepts Succession or Termination
```

Add focused tests for both branches of `redeem`:

1. non-sealing redemption with RESV succession;
2. sealing redemption with RESV termination;
3. termination under a `Succession`-only policy must reject;
4. any edge under `Forbidden` must reject.

---

### 3. Medium — the public deployment-profile hash API can mint identities for invalid profiles

**Locations**

- `packages/architecture/src/deployment.rs`
  - `validate_deployment_release`
  - `deployment_profile_hash`
  - `deployment_profile_hash_hex`
- ADR-016, producer/consumer validation rules

**Problem**

The repository’s identity policy correctly states that validation precedes semantic hashing. But the public API permits:

```rust
deployment_profile_hash(&profile)
```

without an architecture argument and without calling:

```rust
validate_deployment_release(architecture, profile)
```

As a result, a caller can compute a canonical “deployment profile identity” for a profile with, for example:

- unsupported schema;
- non-final status;
- zero network or genesis ID;
- wrong architecture binding;
- missing required dependency evidence;
- pending or failed required evidence;
- missing artifact/report hashes;
- calibration against a different script bundle;
- measurements exceeding declared limits.

The hash is currently documented as dormant, so this is not yet a live release bypass. It is nevertheless an API-level contradiction of the rule:

```text
validate complete typed object
→ canonical projection
→ identity
```

Once a release consumer appears, this API makes it easy to accidentally treat “hashable” as “valid.”

**Recommended fix**

Represent validation in the type system. For example:

```rust
pub struct ValidatedDeploymentProfile<'a> {
    architecture: &'a Architecture,
    profile: &'a DeploymentProfile,
}
```

with construction only through validation:

```rust
pub fn validate_deployment_profile<'a>(
    architecture: &'a Architecture,
    profile: &'a DeploymentProfile,
) -> Result<ValidatedDeploymentProfile<'a>, Vec<DeploymentError>>;
```

Then hash only that validated wrapper:

```rust
pub fn deployment_profile_hash(
    validated: &ValidatedDeploymentProfile<'_>,
) -> [u8; 32];
```

An unchecked canonical-byte helper can remain crate-private for mutation tests if needed.

At minimum, change the public hashing function to accept the architecture and run release validation before hashing.

---

### 4. Medium/latent — representation-conditional activation is not tied to an object

**Locations**

- `packages/compiler/src/source.rs`
  - `RequirementActivation::WhenRepresentation`
- `packages/compiler/src/case.rs`
  - `is_active`
- `packages/compiler/src/placement.rs`
  - `ActivationCondition::WhenRepresentation`
  - `resolve_activity`

**Problem**

Representation choices are keyed correctly by object:

```rust
BTreeMap<ObjectId, RepresentationMode>
```

But conditional activation carries only a mode:

```rust
WhenRepresentation(RepresentationMode)
```

and resolves using:

```rust
case.representations.values().any(|value| *value == mode)
```

This means a requirement intended to activate for object A in `PrivateCommitted`
mode will activate if **any other object** in the same operation uses
`PrivateCommitted`.

Example:

```text
A = Explicit
B = PrivateCommitted
```

A requirement intended for:

```text
A when PrivateCommitted
```

would incorrectly activate because B has that mode.

No current source requirement appears to construct `WhenRepresentation`, and the current pilots each have one representation choice, so this is presently dormant. The type is nevertheless under-specified for the multi-object operations anticipated by the plans.

**Recommended fix**

Carry the object in both activation types:

```rust
WhenRepresentation {
    object: ObjectId,
    mode: RepresentationMode,
}
```

and resolve with:

```rust
case.representations.get(&object) == Some(&mode)
```

Add a synthetic two-object regression in which only one object uses the selected mode. Test both source-requirement activation and relation activity.

---

### 5. Low — current planning status is internally inconsistent

**Locations**

- `plans/packages/README.md`, package index
- `plans/packages/compiler.md`
- `packages/compiler/README.md`
- `plans/backlog.md`, Guide-5 and Guide-6 gate records

**Problem**

The package index currently describes the compiler as:

```text
Active — analysis foundations internal; placement, layout, coverage open
```

But the compiler package contract, crate README, source tree, and backlog gate records all say that placement, layout, and coverage are implemented internally and independently oracle-checked.

This is exactly the type of status drift that T9 says was repaired:

> compiler status documentation materially understated implemented internal analysis

The stale package-index row has reintroduced that understatement.

The backlog’s “latest static review” also names tree:

```text
0.2.1-dev
```

while the supplied report is for:

```text
0.2.3-dev
```

That may be intentional historical provenance, but the heading “Latest static review” now reads as current when it is not.

**Recommended fix**

Update the compiler row in `plans/packages/README.md` to something like:

```text
Active — proof, placement, layout, and coverage foundations implemented internally; complete analyzed pilots open
```

For the review basis, either update the recorded tree when this review is adopted or rename the section to clearly state that it is the latest **recorded prior** review.

A small cross-document status weld may also be worthwhile, since the existing plan checker only welds phase declarations and backlog task-table statuses, not package milestone summaries.

---

## Additional hardening observations

These are lower priority than the findings above.

### Publication files are replaced with temporary-file permissions

`cli_common::publish_batch` stages through `tempfile`, whose files are normally mode `0600`, and then persists them over committed generated publications. This can silently change generated artifacts and label registers from ordinary public-readable files to owner-only files while leaving Git’s executable-bit-based status clean.

Affected users include:

- `generate-all`;
- `generate-label-registers`.

ADR-017 intentionally does not make broad host-permission security claims, so this is not a sandbox issue. It is still undesirable publication behavior. Consider preserving the destination’s existing permissions or explicitly setting the intended ordinary-file mode before persistence on Unix.

### The document reproducibility script has a successful partial path

`scripts/check-document-reproducibility.sh` skips the reused-build epoch probe when the worktree is dirty and exits successfully. It prints the skip, which is honest for a developer convenience run, but callers receive no nonzero or machine-readable partial result.

For a release-oriented gate, either:

- fail immediately on a dirty tree; or
- split the optional reused-build probe into a separately named command whose skip cannot be confused with a complete pass.

The normal full-gate sequence first runs a clean-tree check, so this is primarily a robustness issue rather than a demonstrated current release bypass.

---

## What looks especially strong

A few aspects deserve explicit praise:

1. **Exact closed-asset partitioning is substantially better than aggregate conservation.**
   The source/destination uniqueness checks and flow-level equations close the standard “balanced but unauthorized movement” class of bugs.

2. **Sponsor-value opacity is implemented structurally rather than by convention.**
   `ObservedValue::SponsorOpaque`, read-set rejection, and capability/source analysis all align well with the documented projection.

3. **Evidence classes are kept separate.**
   Model conformance, external substrate evidence, event recognition, canonical query computation, and receipt accounting are not casually collapsed.

4. **The compiler’s independent oracles are unusually thorough.**
   Exact proof search, placement, coverage, dependency closure, and SCC policy each have an independently restated test strategy.

5. **The architecture publication boundary is careful about self-consistency versus trusted identity.**
   `validate_envelope` versus `validate_against_expected` is the right conceptual separation.

6. **Path and filesystem non-claims are honest.**
   ADR-017 avoids the common mistake of presenting incomplete symlink/inode checks as TOCTOU containment.

---

## Review limitations

This was a static review of the supplied concatenation. I did **not** execute Cargo, Meson, TeX, advisory, or reproducibility commands.

The report excluded, among other things:

- `Cargo.lock`;
- architecture/model/realization unit-test directories;
- `packages/document-stamps`;
- `packages/execwrap`;
- `packages/flatten-latex-main`;
- `packages/model/generated/architecture.json`;
- `docs/attestation/human.md`;
- licences.

Therefore I am not making claims about:

- current build or test status;
- lockfile resolution, checksums, or feature closure;
- advisory status;
- excluded package correctness;
- licence compatibility;
- PDF byte reproducibility;
- equivalence of generated JSON with the included TOML;
- complete test coverage for the findings above.

## Suggested order

I would address the findings in this order:

1. make `scripts/ci.sh` actually execute `census-audit`;
2. separate observed root effects from root-use policies;
3. require deployment-profile validation before hashing;
4. make representation activation object-specific;
5. reconcile planning status text;
6. optionally harden publication permissions and dirty-tree reproducibility behavior.

After those changes, the appropriate repository cadence is:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

The full gate should still be reported as deferred until actually run.

# Full second static review

## Executive summary

I performed a second, broader static pass over the supplied 291-file, 3.5 MiB concatenation, revisiting the build/CI wiring, architecture identities, model transition kernel, realization boundary, compiler planning/placement/coverage pipeline, label graph, planning checks, and publication tooling.

The repository remains impressively disciplined. I did **not** find an obvious currently reachable defect that accepts an invalid transition through the implemented model’s public `execute` path for the present Phase-1 pilots. The strongest parts remain:

- exact canonical and open-flow partitions rather than aggregate-only conservation;
- kernel-derived transition certificates;
- separation of model, realization, compiler, target, evidence, and release claims;
- sponsor-value erasure as a typed boundary;
- exact planning and independent compiler oracles;
- deterministic typed projections;
- explicit non-claims around filesystem security and deployment readiness.

However, the second pass confirms all substantive findings from the first review and adds several compiler/realization validation gaps. The most important current issue remains the false completeness claim in `scripts/ci.sh`: it can print **“CI green” without executing the complete tracked-entry mode audit**.

## Finding summary

| ID | Severity | Area | Finding |
|---|---:|---|---|
| SR2-01 | High | CI / Meson | `scripts/ci.sh` can report “CI green” without running `census-audit`; `lint` also omits it. |
| SR2-02 | Medium | Model ↔ realization | `SuccessionOrTermination` rejects ordinary succession observations because policy and actual effect share one enum. |
| SR2-03 | Medium | Realization | Lifecycle relations and lifecycle graph declarations are not welded bidirectionally. |
| SR2-04 | Medium | Compiler placement | `validate_placement` accepts noncanonical carrier supersets, duplicate carriers, and surplus layout requirements. |
| SR2-05 | Medium | Compiler plan set | `validate_placed_proof_plans` does not compare the exact placed proof-plan set with the offered set. |
| SR2-06 | Medium, dormant | Deployment identity | Public profile hashing permits identities for profiles that failed deployment validation. |
| SR2-07 | Medium/latent | Realization observations | Open-flow source/destination sides and cross-flow uniqueness are not enforced by observation normalization. |
| SR2-08 | Low/medium, latent | Compiler activation | Representation-dependent activation is keyed only by mode, not by object. |
| SR2-09 | Low/medium | Coverage validation | Coverage derivation is oracle-checked, but its validator does not reject unexpected requirement classes. |
| SR2-10 | Low | Documentation | Compiler status and static-review provenance have drifted again. |
| SR2-H1 | Hardening | Publication | Tempfile persistence can replace public generated files with owner-only permissions. |
| SR2-H2 | Hardening | Reproducibility | A dirty-tree reused-build probe is skipped with exit status 0. |

---

# Detailed findings

## SR2-01 — High — CI can report “green” without the tracked-entry mode audit

### Locations

- `scripts/ci.sh`
- `scripts/test-meson-mock.sh`
- top-level `meson.build`
  - `census_audit`
  - `alias_target('lint', ...)`

### Problem

The repository centrally assigns Git mode and tracked-shape validation to `census-audit`. That is the only complete check rejecting tracked entries outside:

```text
100644
100755
```

including:

```text
120000  tracked symlink
160000  gitlink/submodule
```

The check covers the complete tracked set, including files categorically excluded from label or plan linting.

But `scripts/ci.sh` does not invoke `census-audit` directly. It relies on lane 10:

```text
mocked Meson contract
```

The mock script invokes explicit targets:

```sh
meson compile -C "$build" attestation
meson compile -C "$build" generate-artifacts
```

Neither explicit target depends on the default-only `census-audit` target. An explicit Ninja target does not implicitly build unrelated `build_by_default` targets.

There is a second omission: the top-level lint alias is:

```meson
alias_target('lint', labels_stamp, generated_stamp, plans_stamp, forbidden_text_stamp)
```

It excludes `census_audit`.

Thus neither:

```sh
scripts/ci.sh
```

nor:

```sh
meson compile -C build lint
```

necessarily executes the complete Git mode audit.

The label checker’s discovery verifier catches many missing Markdown/Rust/TeX census entries, but it cannot replace the tracked-mode audit. For example, a clean tracked symlink under `scripts/` or another categorically excluded path can evade label discovery and still allow `scripts/ci.sh` to print:

```text
CI green
```

That contradicts ADR-014, ADR-017, `AGENTS.md`, and the comments in `scripts/ci.sh`.

### Recommended fix

Include the audit in the lint alias:

```meson
alias_target(
  'lint',
  census_audit,
  labels_stamp,
  generated_stamp,
  plans_stamp,
  forbidden_text_stamp,
)
```

Then make the mocked contract explicitly compile `lint`:

```sh
meson compile -C "$build" lint >/dev/null
```

and verify:

```text
census.ok
census-audit.json
```

exist and that the report says `"valid": true`.

For strongest coverage, add a mock-contract mutation that makes the fake Git listing contain a `120000` entry and verifies the target fails. The subprocess test already checks the binary policy; the Meson test should check that the production graph actually invokes it.

### Focused regression

A useful graph-level regression is:

1. configure the mock build;
2. run `meson compile -C "$build" lint`;
3. verify the Ninja log contains the `census-audit` output edge;
4. run again and verify the always-stale audit reruns without cascading incremental suites.

---

## SR2-02 — Medium — root-use policy is confused with actual root effect

### Locations

- `packages/model/src/conformance.rs`
  - `observe_root_effects`
- `packages/realization/src/observation.rs`
  - `ObservedRootEffect`
- `packages/realization/src/evaluate.rs`
  - `root_policy_holds`
- `packages/realization/src/relation.rs`
  - `Relation::RootPolicy`

### Problem

`architecture::RootUse` describes an **allowed policy**:

```rust
Forbidden
Succession
SuccessionOrTermination
```

`model::RootEdge` describes an **actual event**:

```rust
Succ { input, output }
Term { input }
```

The model conformance adapter converts an actual edge back into the policy enum:

```rust
RootEdge::Succ { .. } => architecture::RootUse::Succession,
RootEdge::Term { .. } => architecture::RootUse::SuccessionOrTermination,
```

The realization evaluator then requires exact equality between observed and expected values.

This makes a policy of:

```text
SuccessionOrTermination
```

accept only termination, not ordinary succession.

The concrete future failure is normal redemption:

```text
redeem RESV policy:
    succession-or-termination

non-sealing redemption actual effect:
    succession
```

The observed effect becomes `RootUse::Succession`, while the expected policy remains `RootUse::SuccessionOrTermination`, so conformance fails even though succession is explicitly permitted.

The sealing branch happens to pass because termination is encoded as `SuccessionOrTermination`.

This is latent because the current realization scope contains only root-free compact ASH and live transfer, but it will surface when redemption enters scope.

### Recommended fix

Use a separate effect type:

```rust
pub enum ObservedRootEffectKind {
    Succession,
    Termination,
}
```

Then validate effect membership in policy:

```text
Forbidden:
    no effect

Succession:
    Succession only

SuccessionOrTermination:
    Succession or Termination
```

Do not overload a policy enum as an event enum.

### Focused tests

Add synthetic realization tests for:

1. `Succession` policy + succession effect → pass;
2. `Succession` policy + termination effect → fail;
3. `SuccessionOrTermination` + succession → pass;
4. `SuccessionOrTermination` + termination → pass;
5. `Forbidden` + any effect → fail;
6. `Forbidden` + no effect → pass.

When redemption is realized, add model-bound conformance tests for both ordinary and sealing redemption.

---

## SR2-03 — Medium — lifecycle relations and the lifecycle graph are not bidirectionally welded

### Locations

- `packages/realization/src/validate.rs`
  - `validate_scoped_realization`
  - `validate_pilot_lifecycle`
  - `validate_operation_ownership`
- `packages/realization/src/lifecycle.rs`
- `packages/realization/src/derive.rs`
- operation declarations under `packages/realization/src/declarations/`

### Problem

The realization carries lifecycle information twice:

1. semantic relations:

```rust
Relation::Representation { object, allowed }
Relation::LifecycleExit { object, exit }
```

2. lifecycle graph nodes and edges:

```rust
LifecycleNodeId::Representation { object, mode }
LifecycleNodeId::RequiredExit { object, operation }
LifecycleEdge::RequiresExit
```

Validation currently proves selected required paths for the two built-in pilots through `validate_pilot_lifecycle`, but it does not establish exact bidirectional equality between the relation declarations and lifecycle graph.

A coherent omission can therefore evade validation. For example:

1. remove the compact-ASH `LifecycleExit { Ash, Clear }` relation;
2. remove the corresponding relation dependency;
3. leave the lifecycle representation node, required-exit node, and edge intact.

Then:

- relation ownership passes;
- relation graph construction passes;
- hard-coded pilot lifecycle reachability still passes;
- architecture family validation does not require lifecycle relations;
- the realization validates;
- the compiler relation census never sees the missing lifecycle relation;
- coverage therefore omits the corresponding semantic lifecycle obligation.

The reverse direction is better protected because a relation whose required graph path is absent may fail the hard-coded pilot checks. But exact closure is not generic and will not automatically extend to future operation scopes.

Extra lifecycle nodes and exits not represented by semantic relations can also enter the stable scoped projection.

### Recommended fix

Derive and compare exact lifecycle declarations generically.

For every representation relation:

```rust
Relation::Representation { object, allowed }
```

require exactly one representation node for every allowed mode.

For every lifecycle relation:

```rust
Relation::LifecycleExit { object, exit }
```

require:

- exactly one matching required-exit node;
- one valid path from every allowed representation of that object.

Conversely require:

- every representation node is owned by a representation relation;
- every required-exit node is owned by a lifecycle relation;
- every `RequiresExit` edge connects a declared allowed representation to a declared lifecycle exit;
- no extra nodes or edges exist.

Then the hard-coded `validate_pilot_lifecycle` can become either a narrow pilot assertion layered on the generic weld or disappear if fully subsumed.

### Focused mutations

Add mutation tests that:

- remove a lifecycle relation and its relation dependency while leaving the graph;
- add a graph exit with no lifecycle relation;
- add an allowed representation to the relation but not the graph;
- add a graph representation absent from the relation;
- connect a valid representation to an undeclared exit;
- remove one required edge while leaving both nodes.

Each should fail derivation with a focused lifecycle-census error.

---

## SR2-04 — Medium — placement validation accepts noncanonical and duplicate carrier assignments

### Locations

- `packages/compiler/src/placement.rs`
  - `validate_placement`
  - `retained_options`
  - `PlacementAssignment`
  - `PlacementCandidate`

### Problem

The production search intentionally retains inclusion-minimal carrier choices:

- `ExactlyOne` → one carrier;
- `EveryMember` → one quantified per-member role or one complete-family proof;
- `AtLeastOne` → inclusion-minimal carrier set;
- `DeliberateDuplication` → the exact complete admitted set.

But `validate_placement` does not enforce the same canonical assignment policy.

For:

```rust
CarrierMultiplicity::EveryMember | CarrierMultiplicity::AtLeastOne
```

the validator accepts any nonempty carrier vector whose individual entries are admissible, including a redundant multi-carrier superset that production search would never emit.

It also does not reject duplicate entries inside:

```rust
PlacementAssignment::carriers: Vec<PlacedCarrier>
```

For deliberate duplication, it compares a deduplicated `BTreeSet` of carrier roles with the admissible set. A duplicated raw entry therefore disappears from the set comparison and can still pass.

Finally, `validate_placement` requires selected carriers’ layout dependencies to be present but does not reject surplus placement-local layout requirements.

Consequences:

- a corrupted placement can validate even though it is outside the exact production candidate set;
- duplicate enforcement can be silently normalized away by stable projections;
- a raw invalid value and a valid value can project identically;
- future `ScopedAnalyzedProgram` validation cannot safely delegate exactness to this function.

### Recommended fix

For each assignment:

1. reject duplicate `PlacedCarrier` entries;
2. derive the exact retained option set using the same semantic policy but through an independent validation helper;
3. require the assignment’s carrier set to equal one retained option;
4. derive exact layout requirements from the selected carrier set;
5. require exact equality with the placement-local layout set, not subset containment.

If validator independence is desired, do not simply call `retained_options`; restate the minimality policy in the validator or compare against the exhaustive placement oracle in test-only code.

### Focused tests

Add corruption tests for:

- duplicated carrier entry;
- per-member role plus complete-family proof selected together;
- two admissible exactly-one carriers selected together;
- redundant second carrier on `AtLeastOne`;
- missing layout requirement;
- extra unrelated layout requirement;
- layout requirement belonging to another relation-case.

---

## SR2-05 — Medium — exact proof-plan set validation is incomplete

### Locations

- `packages/compiler/src/placement.rs`
  - `place_feasible_proof_plans`
  - `validate_placed_proof_plans`
  - `PlacedProofPlans`

### Problem

`validate_placed_proof_plans` detects duplicate placed plans and compares the union of execution-case IDs with the case census derived from the offered candidates.

It does **not** compare:

```text
set of offered ProofPlanCandidate values
=
set of placed ProofPlanCandidate values
```

This matters because different proof plans may produce the same execution-case identities. Representation choices are part of case identity, but proof-alternative choices are not.

A future pair of proof plans can differ only by proof selection while sharing:

- operation;
- sponsor case;
- representation map.

If one such plan is dropped, the union of execution cases remains unchanged and current validation can pass.

The function’s documentation claims the complete plan set cannot drop a candidate, but that is only true for the current pilots because their surviving proof choices correlate with different representation modes.

### Recommended fix

Compute exact sets:

```rust
let offered = candidates.iter().cloned().collect::<BTreeSet<_>>();
let placed = analysis
    .placed
    .iter()
    .map(|entry| entry.proof_plan.clone())
    .collect::<BTreeSet<_>>();
```

Require equality and separately reject duplicates.

If carrying complete proof plans in errors is too large, add a count-only or typed summary error such as:

```rust
PlacedProofPlanCensusMismatch {
    missing: usize,
    unexpected: usize,
}
```

or identify the first differing plan by a compact stable typed summary—not a speculative digest or vector index.

### Focused test

Construct two synthetic feasible proof plans with:

```text
same operation
same representation
different proof alternative
```

Place both, remove one, and require validation to fail even though their case identities are identical.

---

## SR2-06 — Medium, dormant — invalid deployment profiles are publicly hashable

### Locations

- `packages/architecture/src/deployment.rs`
  - `validate_deployment_release`
  - `deployment_profile_hash`
  - `deployment_profile_hash_hex`
- `packages/architecture/src/lib.rs`

### Problem

The identity policy says validation must precede semantic hashing:

```text
validate complete typed object
→ canonical projection
→ identity
```

But the public API accepts any raw `DeploymentProfile`:

```rust
deployment_profile_hash(profile)
```

without architecture input or release validation.

A caller can therefore mint a canonical profile hash for a profile with:

- unsupported schema;
- draft status;
- zero network/genesis ID;
- wrong architecture hash;
- missing calibrations;
- calibration against another bundle;
- failed required dependency evidence;
- missing artifact hashes;
- missing report hashes;
- measurements above target limits.

The repository correctly labels the current profile hash dormant, so this is not yet a live release bypass. It remains an unsafe identity API to carry into the future release package.

### Recommended fix

Hash a validated wrapper:

```rust
pub struct ValidatedDeploymentProfile<'a> {
    architecture: &'a Architecture,
    profile: &'a DeploymentProfile,
}
```

Construct it only through:

```rust
pub fn validate_deployment_profile<'a>(
    architecture: &'a Architecture,
    profile: &'a DeploymentProfile,
) -> Result<ValidatedDeploymentProfile<'a>, Vec<DeploymentError>>;
```

Then:

```rust
pub fn deployment_profile_hash(
    profile: &ValidatedDeploymentProfile<'_>,
) -> [u8; 32];
```

Unchecked canonical rendering can remain crate-private for mutation and recipe tests.

If changing the type now is considered premature because the identity is dormant, at least mark the hash function explicitly unchecked and keep it crate-private until the release consumer activates.

---

## SR2-07 — Medium/latent — open-flow observation normalization does not enforce sides or global uniqueness

### Locations

- `packages/realization/src/observation.rs`
  - `OperationObservation::validate_and_normalize`
- `packages/realization/src/evaluate.rs`
  - `flow_role_is_exact`
  - `sponsor_is_isolated`
  - `Relation::OpenFlowPolicy` evaluation

### Problem

Canonical partition references are checked with exact sides:

```text
flow sources       → Input
flow destinations  → Output
issuance authority → Input
issuance outputs   → Output
```

Open-flow normalization only verifies that references exist:

```rust
for reference in flow.sources.iter().chain(&flow.destinations) {
    if !known.contains(reference) {
        ...
    }
}
```

It does not require:

```text
open-flow source      = input reference
open-flow destination = output reference
```

It also does not enforce source/destination uniqueness across all open flows during normalization.

For the current pilots, `SponsorIsolation` performs additional side and uniqueness checks for fee-sponsor flows, so a malformed current sponsor observation should fail semantically. But `OperationObservation` is a general public type, and future non-sponsor open-flow roles may rely on `OpenFlowPolicy`, which currently checks only that the role is allowed.

This creates an under-specified observation boundary relative to the model kernel’s exact open-flow partition.

### Recommended fix

During `validate_and_normalize`:

```rust
for source in &flow.sources {
    check_reference(*source, ObservedSide::Input, &known)?;
}

for destination in &flow.destinations {
    check_reference(*destination, ObservedSide::Output, &known)?;
}
```

Also enforce exact cross-flow membership:

- no source reference appears in two open flows;
- no destination reference appears in two open flows;
- if the observation contract claims complete L-BTC partitioning, every relevant declared open-value object is claimed exactly once, with CPFP anchor exclusion represented structurally.

The generic normalization should establish structural invariants. Relation evaluation should establish operation-specific role semantics.

### Focused tests

- output reference used as an open-flow source;
- input reference used as an open-flow destination;
- source reused by two open flows;
- destination reused by two open flows;
- unclaimed ordinary L-BTC member;
- CPFP anchor cannot join the ordinary open-flow partition.

---

## SR2-08 — Low/medium, latent — representation activation is not object-specific

### Locations

- `packages/compiler/src/source.rs`
  - `RequirementActivation::WhenRepresentation`
- `packages/compiler/src/case.rs`
  - `is_active`
- `packages/compiler/src/placement.rs`
  - `ActivationCondition::WhenRepresentation`
  - `resolve_activity`

### Problem

An execution case stores representation choices by object:

```rust
BTreeMap<ObjectId, RepresentationMode>
```

But activation stores only a mode:

```rust
WhenRepresentation(RepresentationMode)
```

and evaluates it using:

```rust
case.representations.values().any(|value| *value == mode)
```

Thus a requirement intended for:

```text
object A when A is PrivateCommitted
```

activates if some unrelated object B is `PrivateCommitted`.

Current pilots each have one representation choice and do not appear to construct representation-conditional source requirements, so this is dormant. It becomes incorrect as soon as one operation has multiple independently represented object families.

### Recommended fix

Use:

```rust
WhenRepresentation {
    object: ObjectId,
    mode: RepresentationMode,
}
```

for both source requirements and relation activation.

Resolve with:

```rust
case.representations.get(&object) == Some(&mode)
```

### Focused regression

Create a synthetic case:

```text
A = Explicit
B = PrivateCommitted
```

and verify:

```text
WhenRepresentation { A, PrivateCommitted } → inactive
WhenRepresentation { B, PrivateCommitted } → active
```

Test both source-row filtering and relation activity.

---

## SR2-09 — Low/medium — coverage validation is weaker than coverage derivation

### Locations

- `packages/compiler/src/coverage.rs`
  - `derive_relation_coverage`
  - `validate_coverage_census`
  - `validate_relation_coverage`
- `packages/compiler/src/tests/coverage_oracle_tests.rs`

### Problem

Production coverage derivation and the independent oracle are impressively complete. But the reusable validator checks only coarse conditions:

- exact relation-case key set;
- at least one positive requirement at each boundary;
- at least one negative requirement at each boundary;
- correct broad inactive/active shape.

It does not reject:

- unexpected mutation classes;
- extra positive purposes;
- wrong evidence role where another valid requirement at the boundary remains;
- duplicate semantic requirements that project into set equality;
- wrong collateral policy if another negative exists;
- extra boundary-compatible requirements not admitted by the relation’s exhaustive mutation catalogue.

The test suite explicitly demonstrates one version of this: an unexpected mutation can be inserted, the production validator accepts the shape, and only comparison with the independent oracle detects the mismatch.

That is acceptable while coverage objects are entirely internal and always freshly derived. It is insufficient for the planned corruption-resistant `ScopedAnalyzedProgram` validator, which should not trust the assembler merely because it called production derivation.

### Recommended fix

Either:

1. strengthen `validate_relation_coverage` to derive the exact admitted requirement set from the relation declaration, case plan, placement alternatives, and dependency closure; or
2. make the future analyzed-program validator compare exact stable coverage projections against independently re-derived expected projections.

The second option better preserves oracle independence.

At minimum, document clearly that `validate_coverage_census` validates coverage **shape and relation-case census**, not exact requirement semantics.

### Focused mutations

Require rejection of:

- extra unrelated mutation;
- wrong mutation boundary;
- wrong evidence role;
- missing one required mutation while another remains at the boundary;
- incorrect collateral policy;
- external evidence accepted by target-execution role;
- structural rejection represented as runtime rejection.

---

## SR2-10 — Low — planning status has drifted again

### Locations

- `plans/packages/README.md`
- `plans/packages/compiler.md`
- `packages/compiler/README.md`
- `plans/backlog.md`

### Problem

`plans/packages/README.md` says:

```text
compiler:
    Active — analysis foundations internal; placement, layout, coverage open
```

But the compiler package contract, compiler README, source, and Guide-5/Guide-6 gate records say placement, layout, and coverage are implemented internally and oracle-checked. Only the complete analyzed-program assembly remains open.

This recreates the same class of documentation understatement that T9 claims to have repaired.

The backlog also labels tree:

```text
0.2.1-dev
```

as the “latest static review,” while this supplied report is for:

```text
0.2.3-dev
```

That old value may be intentionally historical, but the heading now makes it sound current.

### Recommended fix

Change the compiler package-index row to something like:

```text
Active — proof, placement, layout, and coverage foundations implemented internally; complete analyzed pilots open
```

When adopting this review, update the review-basis record or rename it to:

```text
Latest recorded prior static review
```

until a new formal review record is committed.

A small package-status weld may be worthwhile, analogous to the phase-declaration weld:

```text
package index status
↔ package contract status
↔ crate README state summary
↔ backlog implementation table
```

This does not need to infer semantics from prose; a narrower structured declaration would suffice.

---

# Hardening observations

## SR2-H1 — publication through `tempfile` may change generated-file permissions

### Locations

- `packages/cli-common/src/publication.rs`
- `packages/cli-common/src/lib.rs`
  - `write_json_report_if_changed`
- generator consumers:
  - artifacts
  - label registers

### Observation

`tempfile` normally creates mode `0600` files on Unix. Persisting the staged file over an existing generated publication can therefore replace a normal `0644` tracked file with an owner-only file.

Git tracks only the executable bit, so this permission change may not appear in ordinary Git status.

This is not a hostile-filesystem vulnerability and ADR-017 correctly makes no such claim. It is a publication-quality and collaborative-worktree issue.

### Suggested hardening

For committed textual publications, explicitly set the intended mode before persistence, or preserve the existing destination mode when it exists.

For example, on Unix:

```rust
use std::os::unix::fs::PermissionsExt;

temporary
    .as_file()
    .set_permissions(std::fs::Permissions::from_mode(0o644))?;
```

A cross-platform helper can define the intended publication-mode policy without turning mode into semantic identity.

Add a Unix-only test verifying generation leaves ordinary publications world-readable.

---

## SR2-H2 — dirty-tree reproducibility probe exits successfully after skipping work

### Location

- `scripts/check-document-reproducibility.sh`

### Observation

The script first performs the two-build PDF byte comparison. It then checks the worktree and, if dirty, skips the reused-build epoch probe and exits 0.

The skip is printed, so it is not silent. But the process status cannot distinguish:

```text
complete reproducibility gate passed
```

from:

```text
first half passed; reused-build probe skipped
```

For a release-oriented command, that is an unnecessarily ambiguous branch.

### Suggested hardening

Prefer failing early on a dirty worktree:

```sh
if [ -n "$tree_status" ]; then
  echo "ERROR: reproducibility gate requires a clean tree" >&2
  exit 1
fi
```

Alternatively split the reused-build test into its own explicitly optional command and make the full release wrapper require both.

---

# Reassessment of the first review

The second pass reconfirms the earlier findings:

| Earlier finding | Second-pass status |
|---|---|
| CI omits complete census/mode audit | Confirmed and strengthened: `lint` also omits it. |
| Root succession-or-termination mismatch | Confirmed. |
| Raw deployment profile is publicly hashable | Confirmed; dormant but should be fixed before activation. |
| Representation activation lacks object identity | Confirmed. |
| Planning/compiler status drift | Confirmed. |
| Generated publication permission hardening | Still applicable. |
| Dirty-tree reproducibility partial-success branch | Still applicable. |

I did not find evidence in the supplied source that any of these has already been repaired.

---

# Areas reviewed without a new blocking finding

## Model transition kernel

The exact canonical partition and exact open-flow partition remain strong. In particular:

- every canonical source is claimed exactly once;
- every canonical destination is claimed by exactly one flow or issuance;
- flow-level source/destination/destruction equations are checked;
- issuance destinations exhaust the issued amount;
- root effects derive from actual consumed and created objects;
- postcommit branch semantics are rechecked;
- rejected transitions are structurally non-mutating.

I did not identify a currently reachable aggregate-offset or balanced-theft acceptance in the public `execute` path.

## Sponsor opacity

The design is consistently carried through:

- `ObservedValue::SponsorOpaque`;
- source-operand rejection;
- disclosure rejection;
- constructibility rejection;
- coverage/layout rejection;
- family-based model flow membership;
- substrate conservation retained as external evidence.

That is much stronger than merely promising not to inspect sponsor values.

## Ledger and evidence separation

The separation among:

- raw event recognition;
- query computation;
- residue/accounting audit;
- checkpoint cache;
- independent deployment evidence

is unusually clear. The public diagnostic snapshot is not promotable back into a query-capable index, and the model checkpoint is opaque.

## Architecture envelope validation

The distinction among:

```text
body hash verifies
envelope validates
publication is final
publication equals independently derived expected value
```

is well designed. The unchecked deployment-profile hash API is the main place where the same discipline has not yet been fully applied.

## Labels and documentation graph

The direct Petgraph graph uses stable owner/label keys rather than graph indices. Generated registers are nonparticipating, architecture labels enter as synthetic citations, and the attestation index derives from body citations rather than sustaining itself. These are all good design choices.

## Compiler independent oracles

The compiler has exceptional test architecture. Exact proof planning, placement, coverage, closure, and SCC behavior are independently restated rather than merely round-tripped through the production helper. The remaining issues are mostly about reusable validators being weaker than those derivations and oracles.

---

# Recommended repair order

## Batch 1 — repository gate correctness

1. Add `census_audit` to `lint`.
2. Explicitly run `lint` or `census-audit` in the mocked Meson contract.
3. Add a graph-level tracked-symlink rejection regression.
4. Run:
   ```sh
   cargo test --locked -p tripod-labels
   scripts/test-meson-mock.sh .
   ```

## Batch 2 — semantic boundary corrections

1. Separate root policy from observed root effect.
2. Add ordinary/sealing redemption-policy tests.
3. Enforce open-flow source/destination sides in observation normalization.
4. Add exact open-flow reference-uniqueness tests.
5. Run:
   ```sh
   cargo test --locked -p tripod-realization
   cargo test --locked -p tripod-model realization_conformance
   ```

## Batch 3 — realization closure

1. Add exact relation ↔ lifecycle graph census validation.
2. Add coherent omission and extra-node mutation tests.
3. Ensure the validator is generic rather than pilot-name-based.
4. Run:
   ```sh
   cargo test --locked -p tripod-realization lifecycle
   cargo test --locked -p tripod-realization derivation
   ```

## Batch 4 — compiler validator exactness

1. Require exact offered/placed proof-plan set equality.
2. Reject duplicate and nonminimal placement carriers.
3. Require exact placement-local layout equality.
4. Make representation activation object-specific.
5. Decide whether to strengthen coverage validation now or in the Guide-7 analyzed-program validator.
6. Run:
   ```sh
   cargo test --locked -p tripod-compiler placement
   cargo test --locked -p tripod-compiler oracle
   cargo test --locked -p tripod-compiler coverage
   ```

## Batch 5 — dormant identity and maintenance hardening

1. Hash only validated deployment profiles.
2. Preserve intended permissions on generated publications.
3. Make reproducibility partial execution distinguishable.
4. Reconcile compiler status and review provenance.

---

# Suggested full verification after repairs

Following `AGENTS.md`, the working Rust cadence is:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

After the complete repair batch:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

For a complete release-oriented result:

```sh
scripts/check-document-reproducibility.sh
cargo audit
git status --porcelain=v1 --untracked-files=all
```

The declared MSRV and current-stable lanes remain separate required evidence.

---

# Review limitations

This was a static review of the supplied concatenation. I did not execute any command.

The supplied report excluded important material, including:

- `Cargo.lock`;
- architecture/model/realization internal test directories;
- `packages/document-stamps`;
- `packages/execwrap`;
- `packages/flatten-latex-main`;
- `packages/model/generated/architecture.json`;
- `docs/attestation/human.md`;
- licence files.

Consequently, this review does not establish:

- current compilation or test status;
- lockfile resolution, checksums, or resolved features;
- advisory status;
- licence compatibility;
- excluded-package correctness;
- exact JSON/TOML generated-artifact equality;
- document stamp correctness;
- real TeX behavior;
- PDF byte reproducibility;
- production deployment readiness.

## Bottom line

The repository’s core architecture remains very strong, and I found no obvious currently reachable invalid model transition in the implemented pilot surface. Before closing Phase 2, I would treat these as the primary blockers:

```text
1. CI must actually execute the complete census/mode audit.
2. Root policy and root effect must be typed separately.
3. Lifecycle relations and lifecycle graph declarations need exact closure.
4. Placement and placed-plan validators need exact-set validation.
5. Deployment identities must be unavailable before validation.
```

The remaining findings are important hardening and future-scope correctness work, especially before Guide 7 introduces a corruption-resistant complete analyzed-program value.
