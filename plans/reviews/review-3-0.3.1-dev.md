# Static review

I reviewed the supplied tree at `0.3.1-dev`, focusing on the 267 included files. This was a source review only: I did **not** run Cargo, Meson, TeX, advisory, or reproducibility commands. The supplied concatenation excluded most test implementations, `Cargo.lock`, several executable tooling packages, and some generated/publication files, so this review cannot confirm test completeness, dependency resolution, or a green build.

## Findings

### 1. High — malformed same-owner citations can silently become label mints

**Affected code**

- `packages/labels/src/markdown.rs`
  - `context`
  - `is_parenthesized_group`
- `packages/labels/src/rust_source.rs`
  - `harvest_segment`
  - `harvest_label`

The Markdown parser detects a one-sided parenthesis only when there is no parenthesis anywhere later or earlier on the line:

```rust
if (open && !close && !line[after..].contains(')'))
    || (close && !open && !line[..start].contains('('))
```

The Rust-comment parser uses the same broad test:

```rust
if (open && !close && !after.contains(')'))
    || (close && !open && !line[..start].contains('('))
```

Consequently, a malformed intended citation such as conceptually:

```text
( `rule:area:name` explanatory text )
```

is not recognized as a valid parenthesized citation, but the later `)` suppresses the asymmetric-citation diagnostic. The occurrence falls back to `Bare`, and the harvesting layer treats it as a **mint**.

That can cause the intended citation to become the only mint of the label. The graph then contains neither an unresolved citation nor necessarily a duplicate mint, so the malformed occurrence can pass while assigning the conceptual home to the wrong location. This violates the ADR-013 distinction between minting and citation and weakens both unique-mint and total-resolution guarantees.

**Recommendation**

Parse the immediate syntactic group rather than searching the rest of the line for any parenthesis:

- if the occurrence is exactly within the allowed parenthesized group grammar, classify it as a citation;
- if either adjacent parenthesis suggests an attempted citation but the exact grammar fails, emit `AsymmetricCitation` or a new invalid-citation-form diagnostic;
- never fall back to `Bare` merely because another parenthesis exists elsewhere on the line.

Add focused regressions for both Markdown and Rust acute syntax, including intervening prose, multiple parenthesized groups, nested punctuation, and unrelated parentheses elsewhere on the line.

---

### 2. Medium — filesystem traversal errors can still become an empty discovered census

**Affected code**

- `packages/labels/src/census.rs`
  - `RepositoryCensus::discover`
  - `files_with_extension`
  - `collect_by_extension`
  - `walk_docs`
- Several uses of:
  - `let Ok(entries) = fs::read_dir(...) else { return; }`
  - `entries.filter_map(Result::ok)`

The census documentation and ADRs explicitly require traversal failures to surface as diagnostics:

> an unreadable tree must never become an empty carrier.

The implementation still suppresses:

- a failure to open a directory, by returning an empty collection;
- an error reading an individual directory entry, through `filter_map(Result::ok)`.

Usually a nonempty declared census will disagree with an empty discovery and indirectly produce `CensusStale`. That does not close the general hole, however. If the corresponding declared group is empty—or an unreadable newly added directory/file has not entered the argument census—declared and discovered sets can both appear empty and verification can pass. Scoped generation is especially relevant because it intentionally runs without the complete repository audit.

**Recommendation**

Make discovery fallible:

```rust
fn discover(root: impl AsRef<Path>) -> Result<RepositoryCensus, Vec<LabelDiagnostic>>
```

or return a value plus diagnostics. Every `read_dir` failure and every failed directory entry should generate a deterministic `Io` diagnostic naming the directory or entry. Do not use `filter_map(Result::ok)` on census traversal.

Add tests for:

- an unreadable subject directory with an empty declared group;
- one failed entry among otherwise readable entries;
- a scoped register/model-label derivation encountering an unreadable scoped directory;
- an absent directory versus an intentionally empty directory, if those have different policy.

---

### 3. Medium, before the first downstream consumer — the complete analyzed-program validator is not on the production construction path

**Affected code**

- `packages/compiler/src/analyzed.rs`
  - `analyze_scoped_program`
- `packages/compiler/src/analyzed_validate.rs`
  - `validate_scoped_analyzed_program`
  - `validate_against_expectations`

`analyze_scoped_program` constructs the complete scoped analyzed value and then calls only:

```rust
validate_assembly_closure(input, &program)?;
```

That validator is deliberately narrow: it checks architecture-scope status and evidence closure. The much stronger validator, which independently re-derives source, foundation, proof-plan, operation, relation-case, placement, coverage, lifecycle, and sponsor-opacity expectations, is not called by the constructor and is marked as dead-code-tolerated.

This does not currently expose an external corruption path because the complete result remains crate-private. It does mean, however, that the production constructor does not actually perform the “corruption-resistant assembly validation” described in package and backlog documentation. A defect in the joins between component stages may be caught by tests that call the full validator, but not by the analysis entry point itself.

This becomes important immediately in Phase 3, when the tapscript adapter is intended to become the first real consumer of compiler target requirements.

**Recommendation**

Before exposing any analyzed projection or target-requirement view, call the complete validator from `analyze_scoped_program`:

```rust
validate_scoped_analyzed_program(input, placement_limits, &program)?;
```

That function already delegates to `validate_assembly_closure`, so the narrow call need not remain separate. If the full re-derivation is intentionally too expensive for ordinary construction, make the assurance distinction explicit in types and documentation—for example, unvalidated assembled state versus a fully validated state—and ensure the first downstream boundary accepts only the latter.

Also validate or clearly exclude `execution_report`; currently the full validator does not appear to authenticate its limits and counters.

---

### 4. Medium — atomic publication changes public files to owner-only mode

**Affected code**

- `packages/cli-common/src/publication.rs`
  - `publish_batch`
- `packages/cli-common/src/lib.rs`
  - `write_json_report_if_changed`
- `scripts/sync-publication.sh`

`tempfile::NamedTempFile` and `mktemp` create files with mode `0600`. The code writes or copies into the already-existing temporary file and then renames it into place. The final generated artifacts, reports, PDF mirrors, and flattened source therefore retain the temporary’s owner-only permissions.

For example:

```rust
let mut temporary = tempfile::Builder::new()
    .prefix(".publication-staged-")
    .tempfile_in(directory)?;
...
temporary.persist(asset.path)?;
```

and:

```sh
staged="$(mktemp ...)"
cp "$source_file" "$staged"
mv "$staged" "$destination"
```

This is already acknowledged as `SR2-H1` in the backlog, but it remains present. Git normally records only the executable bit for ordinary files, so a clean-tree check does not detect a tracked publication changing from broadly readable to `0600`.

**Recommendation**

Set the intended publication mode on the staged file before rename:

- `0644` for public reports, generated text/data, PDFs, and flattened source;
- `0755` only for executables.

Use a shared publication-mode helper where possible. Add tests that begin with an absent destination and with an existing `0644` destination, then assert the published file’s mode after both changed and unchanged publication.

---

### 5. Medium — Git-derived checker arguments are not filename-safe

**Affected code**

- `scripts/ci.sh`
  - `census_args`
  - unquoted `$(census_args ...)`
- `scripts/check-plans.sh`
  - `git ls-files ... | sed ...`
  - unquoted command substitution

The scripts state:

> Paths in this repository never contain whitespace.

That is only a comment; neither the Git mode audit nor another visible policy enforces it. Git permits whitespace, newlines, shell metacharacters, and option-looking components in tracked paths.

The current construction is line-delimited and then expanded unquoted:

```sh
$(git ls-files adr plans | grep '\.md$' | sed 's/^/--subject /')
```

and:

```sh
$(census_args labels)
```

A path containing whitespace is split into several arguments. A path containing option-looking components can be interpreted as checker options rather than as a subject value. Globbing may also occur after unquoted expansion. At minimum, an otherwise valid tracked path can make the direct CI checker census differ from the Meson census; with carefully shaped names it can alter checker output-mode arguments.

This is primarily a correctness problem rather than a meaningful isolation boundary, since untrusted source already controls executable repository scripts. It still contradicts the explicit role-tagged argument contract.

**Recommendation**

Either:

1. enforce and audit a strict tracked-path grammar that excludes whitespace, newlines, and unsafe option-like forms; or, preferably,
2. pass a NUL-delimited census through a checker-owned manifest/response-file interface, with explicit role tags represented structurally.

Avoid constructing argv through unquoted command substitution. If `xargs -0` is used, ensure command invocation cannot be split into multiple independent checker runs and that empty input fails closed.

---

### 6. Medium/Low — the document reproducibility “gate” returns success after skipping one of its checks

**Affected code**

- `scripts/check-document-reproducibility.sh`

After the first two-build byte comparison, the script checks the worktree. If it is dirty, it skips the reused-build epoch probe and exits successfully:

```sh
if [ -n "$tree_status" ]; then
  echo "==> skipping reused-build epoch probe in dirty worktree" >&2
  ...
  exit 0
fi
```

The script describes itself as a reproducibility gate, and repository documentation cites it as a required release/phase-exit check. Exit status `0` is normally consumed as “the gate passed,” even though the reused-build/source-epoch property was not checked.

This is already recorded as parked finding `SR2-H2`, but the present behavior still allows a caller to report a passing command while omitting part of the command’s documented claim.

**Recommendation**

Fail on a dirty worktree by default. If partial operation is useful, require an explicit flag such as `--allow-partial` and return a distinct nonzero or machine-readable partial status that automation cannot confuse with success. At minimum, update the command contract so exit `0` means every advertised reproducibility check ran and passed.

---

### 7. Low — proof-search rejection statistics misclassify constructibility failures as capability failures

**Affected code**

- `packages/compiler/src/proof.rs`
  - `locally_feasible`
  - `visit`

`locally_feasible` returns `None` for several distinct causes:

- missing target capability;
- source-requirement derivation failure;
- constructibility/source mismatch.

But `visit` records every `None` as:

```rust
state.search.rejected_by_capability += 1;
```

The report defines separate counters such as `rejected_by_constructibility`, so the resulting diagnostic provenance can claim that capability pruning occurred when constructibility was actually responsible.

The execution report is excluded from stable semantic identity, which limits the impact, but it is still a typed report presented as an account of how analysis was performed.

**Recommendation**

Return a typed local-feasibility result:

```rust
enum LocalRejection {
    Capability,
    Source,
    Constructibility,
}
```

and increment the corresponding counter. Add a synthetic case where all required capabilities are available but witness constructibility fails, and assert that capability rejection remains zero.

---

### 8. Low — package-index status still contradicts the completed compiler state

**Affected documentation**

- `plans/packages/README.md`
- Compared with:
  - `packages/compiler/README.md`
  - `plans/packages/compiler.md`
  - `plans/backlog.md`
  - `plans/phases/02-compiler.md`

The package index still says:

> complete analyzed pilots open

while the compiler README, package contract, backlog Guide-7 gate, and Phase-2 card say the complete scoped analyzed pilots are implemented internally and Phase 2 is complete.

This is particularly notable because the backlog marks the compiler package-index drift finding `SR2-10` as `DONE`. The actual index remains stale.

**Recommendation**

Update the index row to match the narrower current claim:

```text
Active — complete scoped analyzed pilots implemented internally;
public complete-analysis API and target adapter absent
```

Consider extending the planning checker with a small typed status weld for package index rows if these summaries are expected to stay authoritative enough to close review findings.

---

## Additional parser hardening

The Markdown/Rustdoc fence recognizer handles top-level fences with up to three leading spaces, but it does not recognize CommonMark fenced blocks nested in blockquotes or list containers. ADR-013 says fenced material is nonparticipating without limiting that rule to top-level fences. A label-shaped token inside a quoted/list-nested fenced example may therefore participate unexpectedly.

This is less serious than the citation-to-mint issue, but the same parser-hardening batch should either:

- support fenced-code containers the repository permits; or
- explicitly restrict the repository’s accepted Markdown grammar and lint unsupported nested fences.

## Overall assessment

The repository has unusually strong separation of:

- typed semantic authority versus derivative publications;
- model, realization, compiler, target, and deployment assurance classes;
- semantic identity, artifact bytes, provenance, and evidence;
- exact closed-asset flows versus open-value roles;
- model execution versus post-execution conformance projection;
- compiler requirements versus target evidence;
- public-data tooling versus external execution isolation.

I did not see an obvious direct authorization, issuance, reserve-drain, or closed-asset inflation bypass in the selected implementation source. That is **not** a green or production-readiness conclusion: most focused tests were excluded from the supplied content, the target/backend does not yet exist, and the repository itself correctly says deployment evidence is absent.

Recommended repair order:

1. fix malformed citation classification;
2. make census traversal failures explicit;
3. wire the complete compiler validator before exposing the Phase-3 target boundary;
4. preserve publication modes;
5. make checker census argument transport filename-safe;
6. harden reproducibility status semantics;
7. repair diagnostic counters and stale planning summaries.

# Full second static review

I performed a second, independent pass over the supplied tree at:

```text
0.3.1-dev
```

This is the **same revision** as the first review; no repaired tree was supplied. Accordingly, the findings from the first pass should still be treated as open unless independently fixed elsewhere. This report concentrates on additional issues and deeper cross-package inconsistencies rather than repeating the first report verbatim.

## Scope and limitations

Reviewed:

- the 267 included files;
- approximately 3.23 MiB of selected source;
- architecture, realization, compiler, model, label tooling, Meson, scripts, paper source, and planning contracts.

Not reviewed in content because they were excluded from the supplied concatenation:

- most unit and integration tests;
- `Cargo.lock`;
- `packages/document-stamps`;
- `packages/execwrap`;
- `packages/flatten-latex-main`;
- `docs/attestation/human.md`;
- generated `architecture.json`;
- licence texts.

Not executed:

- Cargo formatting, Clippy, tests, or documentation;
- Meson setup, compile, or tests;
- generated-artifact or label checkers;
- TeX rendering;
- document reproducibility;
- `cargo audit`;
- target-native behavior.

Therefore this remains a **static source review**, not a green-build, dependency, release, or deployment-readiness claim.

---

# Executive summary

The second pass found:

| ID | Severity | Finding |
|---|---:|---|
| S2-01 | High, future semantic blocker | Sponsor erasure is keyed to the `PLAIN_LBTC` object family instead of fee-sponsor flow membership, erasing protocol-role L-BTC amounts needed by future operations. |
| S2-03 | Medium | The coverage graph mixes incompatible edge orientations despite documenting one global prerequisite → dependent convention. |
| S2-04 | Medium | The complete analyzed-program validator accepts duplicated coverage graph nodes and edges because it compares sets rather than exact projections. |
| S2-05 | Low/Medium | Exact proof and placement searches can overflow counters instead of returning their promised typed complexity failures. |
| S2-06 | Low | The document subproject unnecessarily requires a C compiler, contradicting the stated Rust/TeX-only build requirements. |
| S2-07 | Low | Compiler planning status remains internally inconsistent even though the backlog marks the status-drift finding closed. |

The most important new issue is **S2-01**. It does not create an obvious current compact-ASH or live-transfer bypass because those two pilot operations use `PLAIN_LBTC` only for the generic fee-sponsor region. It will, however, make the current realization/compiler abstraction incorrect when scope expands to request creation, cancellation, admission, redemption, or other operations where ordinary L-BTC occupies a protocol role.

The second pass still found no obvious currently reachable unauthorized issuance, reserve theft, closed-asset inflation, time-locked burn/redemption path, or recipient-redirection path in the selected model implementation. That is a narrow static observation, not a proof and not a production claim.

---

# Detailed findings

## S2-01 — High, future semantic blocker: sponsor erasure is attached to an object family rather than a flow role

### Affected code

- `packages/model/src/conformance.rs`
  - `observe_utxo`
- `packages/realization/src/evaluate.rs`
  - `observed_object_shape_holds`
  - `flow_role_is_exact`
  - `sum_selected_amounts`
- `packages/realization/src/validate.rs`
  - `validate_sponsor_value_opacity`
- `packages/compiler/src/case.rs`
  - `is_sponsor_object`
- `packages/compiler/src/source.rs`
  - `is_sponsor_amount_operand`
  - `operand_source`
- `packages/compiler/src/disclosure.rs`
  - `is_sponsor_amount`
- `packages/compiler/src/analyzed_validate.rs`
  - sponsor-opacity traversal

### Problem

The normative sponsor-erasure rule is **role-based**. It partitions L-BTC references into protocol and sponsor regions:

\[
I_{\mathrm{LBTC}} = I_P \uplus I_S,\qquad O_{\mathrm{LBTC}} = O_P \uplus O_S.
\]

It erases individual amounts only for the sponsor region \(I_S,O_S\). Protocol-region amounts remain available where protocol relations require them, such as:

- request gross value and owner change;
- request refund;
- admission principal and reserve successor;
- redemption payout;
- RESV carry;
- formula-bound protocol outputs.

The implementation instead treats the architecture object kind `PLAIN_LBTC` as synonymous with “sponsor object.”

In `model::conformance::observe_utxo`:

```rust
let value = if kind == ObservedObjectKind::Declared(architecture::ObjectId::PlainLbtc) {
    ObservedValue::SponsorOpaque
} else {
    ObservedValue::Protocol(observed_amount(utxo.value)?)
};
```

Thus **every** `PLAIN_LBTC` object is projected as sponsor-opaque, regardless of which open flow claims the reference.

The realization evaluator reinforces that classification:

```rust
ObjectId::PlainLbtc => {
    observed.owner.is_some() && observed.value == ObservedValue::SponsorOpaque
}
```

The compiler repeats it:

```rust
pub const fn is_sponsor_object(object: ObjectId) -> bool {
    matches!(object, ObjectId::PlainLbtc)
}
```

and:

```rust
pub const fn is_sponsor_amount_operand(role: &OperandRole) -> bool {
    matches!(
        role,
        OperandRole::ObjectFamilyAmount {
            object: architecture::ObjectId::PlainLbtc,
            ..
        }
    )
}
```

### Why this matters

The architecture uses `PLAIN_LBTC` for several non-sponsor protocol roles:

| Operation | Protocol-role `PLAIN_LBTC` use |
|---|---|
| `create-request` | mandatory owner-funded inputs and optional owner change |
| `cancel-request` | formula- and destination-bound refund |
| `admit-deposits` | optional admission reward |
| `redeem` | formula-bound owner payout |
| `cycle` | generic sponsor change only in current architecture |
| transfer/burn/etc. | generic sponsor region |

The same object family can therefore occur in both:

- a protocol open flow such as `redemption`, `request-refund`, or `request-creation`; and
- a generic `fee-sponsor` flow.

For redemption in particular, future realization conformance must verify that a `PLAIN_LBTC` payout equals:

\[
p = \left\lfloor \frac{x\Omega}{Y} \right\rfloor.
\]

Under the current observation model, the payout object’s value is `SponsorOpaque`, so the realization cannot read or compare it. Treating the payout as opaque would discard a load-bearing protocol relation; treating it as zero would be worse.

The same issue appears in compiler activation. `WhenSponsorPresent` is derived from an object being `PLAIN_LBTC`, while the semantic distinction should be whether a `fee-sponsor` flow is present. A mandatory `PLAIN_LBTC` request-creation input is not an optional sponsor region.

### Current reachability

The two current realization pilots are:

```text
compact-ash
transfer-live-receipts
```

For those operations, `PLAIN_LBTC` is used only by `fee-sponsor`, so the defect is latent rather than a currently visible pilot acceptance bypass.

It becomes a hard semantic blocker before expanding realization scope to the remaining architecture operations.

### Recommendation

Make sponsor erasure **reference-role based**, not object-kind based.

A sound projection should:

1. derive the exact references claimed by `OpenFlowKind::FeeSponsor`;
2. require sponsor and protocol flow references to be disjoint;
3. project values of fee-sponsor references as `SponsorOpaque`;
4. retain `Protocol(amount)` for `PLAIN_LBTC` references claimed by protocol flows;
5. reject a reference appearing in both regions;
6. key sponsor-presence activation to the presence of a fee-sponsor flow, not the existence of any `PLAIN_LBTC` object.

Possible typed designs include:

```rust
pub enum ObservedValueRole {
    Protocol(ProtocolAmount),
    SponsorOpaque,
}
```

with the role derived from the reference’s exact open-flow membership, or a richer observation:

```rust
pub struct ObservedOpenObject {
    reference: ObservedObjectRef,
    family: ObjectId,
    flow_role: OpenFlowKind,
    value: ObservedValue,
}
```

Compiler operands should similarly distinguish:

```text
protocol open-flow amount
sponsor-region amount
```

rather than treating every `ObjectFamilyAmount(PLAIN_LBTC)` as forbidden.

### Focused tests

Add before expanding realization scope:

- one redemption observation containing:
  - protocol payout `PLAIN_LBTC` with readable amount;
  - sponsor change `PLAIN_LBTC` with opaque amount;
- payout one unit short while sponsor change increases one unit:
  - aggregate substrate balance preserved;
  - protocol payout relation fails;
- request-refund amount remains readable;
- request-creation owner inputs are not classified as an optional sponsor family;
- fee-sponsor input and output values remain unavailable;
- one reference claimed by protocol and sponsor flows rejects;
- compiler sponsor activation depends on fee-sponsor-flow presence;
- no sponsor amount appears in stable projections or evidence fields.

---

## S2-03 — Medium: coverage dependency graph mixes opposite edge orientations

### Affected code

- `packages/compiler/src/coverage_graph.rs`
  - `CoverageEdge`
  - `derive_coverage_dependencies`
  - `attach_dependency_collateral`
  - module-level direction documentation

### Documented convention

The module says:

> Direction is prerequisite → dependent throughout

and source relation dependencies follow that convention:

```rust
CoverageDependency {
    source: prerequisite_relation_case,
    target: dependent_relation_case,
    edge: CoverageEdge::RelationPrerequisite,
}
```

### Actual non-relation edges

For a relation requiring a carrier, the code emits:

```rust
CoverageDependency {
    source: relation_case.clone(),
    target: carrier,
    edge: CoverageEdge::RequiresCarrier,
}
```

But if the carrier is required to discharge the relation, the carrier is the prerequisite and the relation is the dependent under the documented convention.

Likewise, for layout:

```rust
CoverageDependency {
    source: carrier.clone(),
    target: layout,
    edge: CoverageEdge::RequiresLayout,
}
```

The code comment says the carrier depends on layout, which again implies:

```text
layout → carrier
```

under prerequisite → dependent orientation, not carrier → layout.

External evidence and accepted projection requirements have the same issue:

```text
relation → evidence
relation → projection requirement
```

despite those objects being requirements of the relation.

Collateral edges use yet another causal reading:

```rust
negative requirement → blocked dependent relation
```

### Impact

The current collateral calculation explicitly follows only `RelationPrerequisite` edges, so the principal active-descendant closure is not immediately corrupted.

Nevertheless, the graph is described and published as one typed dependency graph. It cannot currently support a generic interpretation such as:

- topological prerequisite ordering;
- general ancestor/descendant analysis;
- cycle analysis over all actual requirements;
- “what must exist before this relation is discharged?” traversal.

Its edge direction means different things depending on the edge variant:

| Edge | Current direction |
|---|---|
| relation prerequisite | prerequisite → dependent |
| carrier requirement | owner → required object |
| layout requirement | dependent → prerequisite |
| projection requirement | owner → required object |
| external evidence | owner → required object |
| collateral | claim → affected relation |

That contradicts the module-level invariant and makes the graph’s stable projection semantically ambiguous.

### Recommendation

Choose one of two designs.

#### Option A — one dependency orientation

Use prerequisite → dependent throughout:

```text
layout → carrier → relation-case
projection requirement → relation-case
external evidence → relation-case
relation prerequisite → dependent relation
```

Give collateral claims a separately documented edge whose semantics fit the direction.

#### Option B — separate graphs

Keep:

1. a prerequisite graph for relation dependency and collateral closure;
2. an ownership/obligation graph for relation → carrier/layout/evidence presentation.

This may be clearer because “owns a requirement” and “depends on a prerequisite” are different relations.

In either design:

- document edge semantics per graph;
- add a validator asserting the intended source/target node classes for each edge variant;
- add topology tests that would fail if a dependency is reversed;
- ensure SCC policy is run only over a graph whose orientation and cycle meaning are coherent.

---

## S2-04 — Medium: analyzed-program validation accepts duplicate coverage graph nodes and edges

### Affected code

`packages/compiler/src/analyzed_validate.rs`:

```rust
fn validate_coverage_graph(
    operation: OperationId,
    stored: &CoverageGraphProjection,
    required: &CoverageGraphProjection,
) -> Result<(), CompileError>
```

### Problem

The validator converts the stored and required vectors to sets:

```rust
let carried = stored.nodes.iter().collect::<BTreeSet<_>>();
let expected = required.nodes.iter().collect::<BTreeSet<_>>();
```

and similarly for edges.

This detects missing or semantically unexpected members, but it silently collapses duplicates. A corrupted projection such as:

```text
[n₁, n₂, n₂]
```

compares equal to:

```text
[n₁, n₂]
```

The same applies to duplicate typed edges.

The construction graph correctly rejects duplicate definitions and dependencies. The complete assembly validator should therefore reject a duplicated projection too. Accepting duplicates breaks the stated exact-census property and weakens the claim that extra coverage requirements are rejected in both directions.

### Impact

The analyzed program remains crate-private, so this is not presently an external input vulnerability. It matters for:

- corruption resistance;
- internal test reliability;
- the first Phase-3 consumer;
- any future serialized or cached projection;
- confidence in “complete validator” claims.

It compounds the first review’s finding that the full analyzed-program validator is not called by `analyze_scoped_program`.

### Recommendation

Require exact canonical projection equality after focused semantic checks:

```rust
if stored.nodes != required.nodes {
    // focused node diagnostic
}

if stored.edges != required.edges {
    // focused edge diagnostic
}
```

or explicitly validate:

```text
vector length = set length
vector is canonically sorted
vector contents = required contents
```

Add focused corruption tests for:

- duplicate node;
- duplicate edge;
- out-of-order node;
- out-of-order edge;
- repeated exact self-edge where graph policy forbids it;
- one duplicate plus one missing member that would otherwise preserve length.

Before exposing any compiler target-requirement projection, the production constructor should invoke the full analyzed-program validator, as recommended in the first review.

---

## S2-05 — Low/Medium: exact searches can overflow counters instead of returning typed complexity failures

### Affected code

- `packages/compiler/src/proof.rs`
  - `visit`
- `packages/compiler/src/placement.rs`
  - `visit_placement`

### Problem

The exact search counters use unchecked addition:

```rust
state.search.states_visited += 1;
state.search.complete_assignments += 1;
state.search.rejected_by_capability += 1;
```

and:

```rust
state.search.states_visited += 1;
state.search.complete_assignments += 1;
```

The configured maximum is `NonZeroU64`, so `u64::MAX` is a valid limit. If a search reaches the maximum count and attempts one more increment:

- debug builds panic on overflow;
- release builds may wrap;
- the promised typed complexity error may not be returned.

This contradicts the compiler policy repeated throughout the repository:

> complexity exhaustion returns a typed error and no partial result.

The search result itself is unlikely to reach \(2^{64}\) states in practice, but correctness contracts should not rely on “the machine will run out of time first,” especially in exact-search infrastructure.

### Recommendation

Check the limit before incrementing or use checked addition:

```rust
if state.search.states_visited == state.limits.maximum_states.get() {
    return Err(CompileError::ProofSearchStateLimitExceeded {
        maximum: state.limits.maximum_states.get(),
    });
}

state.search.states_visited += 1;
```

Be precise about whether the root state counts toward the limit.

For diagnostic-only counters that do not decide search acceptance, use one of:

- checked increments with a typed report-overflow error;
- saturating increments with an explicit comment and test;
- a larger internal integer if justified.

Do not silently wrap.

### Related issue from the first pass

`locally_feasible` still collapses capability, source, and constructibility failures into `None`, while the search records every such rejection as `rejected_by_capability`. That is a separate diagnostic-accounting defect and remains open on this tree.

### Focused tests

- `maximum_states = 1`, requiring more than one visited state;
- exact boundary where the final permitted state completes a result;
- candidate-limit boundary;
- synthetic counter at `u64::MAX - 1` through an internal test helper;
- constructibility-only rejection does not increment capability rejection;
- counters never wrap under debug or release behavior.

---

## S2-06 — Low: the paper subproject unnecessarily requires a C compiler

### Affected code

`papers/attestation/meson.build`:

```meson
project('attestation', 'c', version: '1.0.0', meson_version: '>=1.3.0')
```

### Problem

The paper subproject declares C as a project language, but the supplied build definition contains no C target. That causes Meson to discover and require a C compiler during setup.

The repository requirements list:

- Meson/Ninja;
- Python;
- Rust/Cargo;
- TeX;
- Git;
- POSIX shell tools.

They do not list a C compiler.

The mocked Meson contract also claims to require Meson, Ninja, Cargo, and Git but no TeX. In practice, the unnecessary `'c'` declaration can make a secretless documentation/mock environment fail before any relevant target is configured.

### Recommendation

If no C target exists, change the subproject declaration to a no-language project:

```meson
project(
  'attestation',
  version: '0.6.1',
  meson_version: '>=1.3.0',
)
```

If Meson or a future tool truly requires C, document the compiler as a build prerequisite and add a focused reason. Do not retain an ambient toolchain dependency merely because it is commonly installed.

Add a mock-contract setup test in an environment with no C compiler on `PATH`, if practical.

---

## S2-07 — Low: compiler planning status still contradicts completed Guide-7 state

### Affected documentation

- `plans/packages/README.md`
- `plans/packages/compiler.md`
- `packages/compiler/README.md`
- `plans/backlog.md`
- `plans/phases/02-compiler.md`

### Contradiction

The package index says:

> complete analyzed pilots open

But the compiler README and package contract say complete scoped analyzed pilots are implemented internally. The backlog records the Guide-7 Phase-2 exit as complete, and the Phase-2 card is marked complete.

The compiler package contract also retains this open question:

> Should multi-operation placement remain the exact product across operations, or be stored per operation before the analyzed-program value assembles it?

That question was decided and implemented by Guide 7:

```text
store per-operation factors;
do not materialize the cross-operation product.
```

The backlog marks `SR2-10`, compiler package-index status drift, as `DONE`, yet the stale index and resolved open question remain.

### Impact

This does not alter runtime behavior, but it weakens planning status as a review boundary. A finding should not be marked closed while the stated stale surface is still stale.

It may also mislead Phase-3 work about whether a downstream adapter has a complete internal source to consume.

### Recommendation

Update the package index to something like:

```text
Active — complete scoped analyzed pilots implemented internally;
public complete-analysis boundary and target adapter absent
```

Remove or rewrite the resolved factorization question.

Retain genuinely open questions:

- minimum public target-requirement boundary;
- target-adapter ownership;
- target-selection objective;
- identity activation when a real persistent consumer appears.

Because status drift has recurred, consider extending `check-plans` with a narrow, typed status weld for the package index and phase cards. Avoid broad prose synchronization; only mechanically compare the fields the repository already treats as state.

---

# Interaction with the first review

Because this second pass examines the same tree, the first-pass findings remain relevant:

1. malformed same-owner citations can become mints;
2. traversal I/O errors can collapse into empty census discovery;
3. the complete compiler validator is not wired into analyzed-program construction;
4. publication staging can change output permissions to `0600`;
5. shell-derived checker argv is not filename-safe;
6. the reproducibility script exits success after a dirty-tree partial skip;
7. proof rejection statistics misclassify constructibility/source failures;
8. package-index compiler status is stale.

The second pass strengthens two of those areas:

- S2-04 identifies a concrete hole inside the supposedly complete compiler validator itself;
- S2-07 confirms that the status-drift finding remains materially unclosed.

---

# Cross-layer consistency assessment

## Architecture ↔ model

Strong points observed:

- stable operation, object, asset, delta, tag, root-use, and open-flow codes are exhaustively mapped;
- numeric and name welds protect against silent identifier transposition;
- canonical closed-asset flows preserve source/destination grouping;
- root succession is derived from actual consumed/created objects;
- STATE and RESV edges remain separable;
- terminal RESV behavior is explicitly represented;
- active-backing cap is enforced in genesis, admission, cycle, and invariant checks;
- model closed-asset recognition remains strict while open junk stays inert.

No obvious source-level mismatch was found in the current finite operation registry.

## Model ↔ realization

The current pilot projection is carefully bound to executed transitions through `ExecutedTransition<T>`, avoiding arbitrary mixing of a request with another execution’s certificate.

The principal second-pass concern is the role-insensitive `PLAIN_LBTC` erasure in S2-01. The existing pilots avoid it only because their ordinary L-BTC family is sponsor-only.

## Realization ↔ compiler

Strong points observed:

- architecture family cardinality and recognition relations are bidirectionally derived;
- lifecycle graph and relations are welded;
- proof alternatives remain distinct from target patterns;
- sponsor amount reads are aggressively rejected;
- operation factors avoid the full cross-operation placement product;
- target capabilities remain abstract.

Main concerns:

- sponsor identity is attached to an object family instead of a flow region;
- full assembly validation is not on the production constructor path;
- duplicated coverage graph projection entries can evade complete validation;
- dependency graph edge orientation is semantically inconsistent.

## Compiler ↔ future target

The repository correctly has no target/backend implementation yet and generally avoids claiming one.

Before exposing a Phase-3 target-requirement boundary, I recommend requiring all of:

1. repair S2-01’s role-sensitive sponsor distinction;
2. settle S2-03’s graph orientation;
3. reject duplicate graph projections under S2-04;
4. call the complete analyzed-program validator from construction;
5. expose only the narrow requirement vocabulary actually consumed by the adapter;
6. keep target facts downstream of compiler core;
7. mint no target or compiler digest unless ADR-016 admission is satisfied by a real consumer.

---

# Recommended repair order

## Immediate correctness and normative consistency

2. **First review F1:** fix malformed citation classification.
3. **First review F2:** make census discovery explicitly fallible.
4. **S2-04:** make coverage graph validation exact and duplicate-sensitive.

## Before the first Phase-3 compiler consumer

5. **S2-01:** redesign sponsor erasure around exact fee-sponsor-region membership.
6. **First review F3:** invoke complete analyzed-program validation from construction.
7. **S2-03:** normalize or separate coverage graph edge semantics.
8. **S2-05:** make exact-search counter exhaustion typed and overflow-safe.

## Build and publication hardening

9. Preserve intended file modes during atomic publication.
10. Replace unsafe shell argv construction with filename-safe census transport.
11. Make document reproducibility return success only after all advertised checks run.
12. Remove the unused C-language requirement.

## Documentation hygiene

13. Reconcile compiler status and remove resolved “open” questions.
14. Update the backlog with this review’s exact tree and execution limitations only after findings are triaged.
15. Do not mark findings closed until the stated stale surface and focused regression both change.

---

# Suggested focused verification after repair

## Labels and census

```sh
cargo test --locked -p tripod-labels
cargo test --locked -p tripod-labels markdown
cargo test --locked -p tripod-labels rust_source
cargo test --locked -p tripod-labels census
meson compile -C build lint
```

Required regressions:

- malformed intended citation never mints;
- unrelated parentheses do not suppress citation diagnostics;
- unreadable empty-group directory fails;
- failed directory entry is not skipped;
- generated registers remain nonparticipating.

## Realization/model sponsor boundary

```sh
cargo test --locked -p tripod-realization
cargo test --locked -p tripod-model realization_conformance
```

Required regressions:

- fee-sponsor references are opaque;
- protocol-role `PLAIN_LBTC` references retain required amounts;
- protocol/sponsor overlap rejects;
- zero sponsor member is semantically accepted;
- canonical builder omits known zero change;
- balanced theft fails at the protocol relation, not sponsor positivity.

## Compiler

```sh
cargo test --locked -p tripod-compiler
cargo test --locked -p tripod-compiler analyzed
cargo test --locked -p tripod-compiler coverage_graph
cargo test --locked -p tripod-compiler proof
cargo test --locked -p tripod-compiler placement
```

Required regressions:

- complete constructor invokes complete validation;
- duplicate coverage node rejects;
- duplicate coverage edge rejects;
- edge orientation tests reflect one documented convention;
- complexity counters cannot overflow;
- constructibility rejection is not counted as capability rejection;
- stable projections remain permutation-independent.

## Build and publication

```sh
cargo test --locked -p cli-common
scripts/test-meson-mock.sh .
scripts/check-document-reproducibility.sh
```

Required regressions:

- newly published public assets are `0644`;
- executables remain `0755`;
- no C compiler is needed for mock setup if C is unused;
- dirty-tree reproducibility does not return a complete-success status;
- a tracked unusual filename cannot corrupt checker argv.

## Full batch gate

After the focused repairs:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
scripts/check-document-reproducibility.sh
git diff --check
git status --porcelain=v1 --untracked-files=all
```

A missing `cargo-audit` remains a loud skip, not a passing advisory lane.

---

# Final assessment

This remains an unusually disciplined pre-target codebase. Its strongest properties are:

- clear semantic ownership;
- exact typed arithmetic;
- strict separation of semantic, byte, provenance, evidence, and release identities;
- explicit assurance boundaries;
- canonical closed-asset partitioning;
- root-history replay;
- non-writing checks;
- careful refusal to equate model success with deployment readiness.

The most consequential weakness exposed by the second pass is not in the current pilot transitions themselves. It is in the abstraction chosen for the next expansion step:

> `PLAIN_LBTC` is an object family; “sponsor” is a transaction-region role.

Those concepts coincide in the current two pilots but not in the architecture as a whole. The realization and compiler must separate them before additional operations are brought into scope. Otherwise sponsor opacity will erase protocol facts that future relations must authenticate.

Subject to the review limitations, I found no obvious currently reachable model path for unauthorized reserve withdrawal, unauthorized receipt transfer, unsanctioned issuance, time-locked burn/redemption, or forged closed-asset admission. The repository is nevertheless correctly still pre-production: no target backend, transaction ABI, target-native evidence, final bundle, deployment profile consumer, or release path currently exists.
