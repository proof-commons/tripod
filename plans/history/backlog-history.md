# Backlog history

Closed records moved verbatim out of [the backlog](../backlog.md). Each
group below names the backlog sections it came from and the date it was
moved. Nothing here is edited after the move: these are finished records
of work that closed, kept so the backlog can carry only live work.

Section references inside the moved text (`§2.1`, `§5.3`, and the like)
point at the backlog as it stood when the record was written. Plan-local
labels moved with their sections and are unchanged, so every citation of
them still resolves.

## Index · `tab:history:index`

| Group | Origin | Moved |
|---|---|---|
| [Historical gate records](#historical-gate-records) | backlog §2.3–2.14 | 2026-08-19 |
| [Closed review registers](#closed-review-registers) | backlog §5.2–5.5 | 2026-08-19 |
| [Sixth-review findings](#56-sixth-review-findings--tabbacklogfindings-sr6) | backlog §5.6 | 2026-08-19 |
| [Seventh-review preflight findings](#59-seventh-review-preflight-findings--tabbacklogfindings-sr7) | backlog §5.9 | 2026-08-20 |
| [Guide-12 records](#guide-12-records) | backlog §2.6–2.8 and the Phase-4 wave narratives | 2026-08-21 |
| [Guide-13 records](#guide-13-records) | backlog §2.9 and the Phase-5 wave narratives | 2026-08-25 |

## Historical gate records

Moved from [the backlog](../backlog.md) §2.3–2.14 on 2026-08-19: the
completed gate records for Phase 1, the T1–T5 remediation, the P2-004
compiler-input batch, and Guides 2 through 10.

### 2.3 Historical Phase-1 gate · `gate:backlog:phase1`

The repository recorded the Phase-1 gate as green on the commit that closed the phase; the Phase-1 card states what that gate ran. A recorded gate establishes what held at that commit, and does not establish that the current checkout passes.

### 2.4 Historical T1–T5 remediation gate · `gate:backlog:t1-t4-remediation`

The repository records a completed remediation batch for T1–T5. Its recorded
result was:

- `scripts/ci.sh`: every available lane passed;
- `cargo-audit`: skipped because unavailable, so the run was partial-green;
- real Meson compile: passed;
- Meson tests: passed;
- document byte reproducibility: deferred;
- final clean-tree check: empty.

This is historical evidence for the remediation tree, not a current-tree gate.

### 2.5 Historical compiler-input gate · `gate:backlog:p2-004`

The repository records a completed P2-004 compiler-input batch:

- typed owner revalidation;
- explicit canonical compiler scope;
- strict analysis policy;
- immutable bound compiler input;
- focused public-API tests;
- available CI and Meson lanes passed;
- advisory lane skipped where unavailable;
- document byte reproducibility deferred because paper inputs were unchanged;
- final clean-tree check empty.

### 2.6 Historical Guide-2 gate · `gate:backlog:guide2`

The repository records a completed Guide-2 batch for:

```text
C1-005
P2-005
P2-006
```

The batch covered:

- scoped relation DAG;
- scoped expression DAG;
- predicate binding;
- checked constant folding;
- independent folding oracle;
- declaration-permutation determinism;
- dependency review for the compiler’s Petgraph edge.

The latest static review did not find a defect in these components.

### 2.7 Historical Guide-3 gate · `gate:backlog:guide3`

The repository records a completed Guide-3 batch for:

```text
C1-008
P2-007
P2-008
P2-009
```

The batch covered:

- relation-obligation classification;
- exact proof-plan enumeration;
- source requirements;
- constructibility;
- disclosure;
- representation lifecycle;
- independent exhaustive planning oracle.

The latest static review found T6 and T7 in this area. The historical gate
remains evidence that its recorded commands passed; it is not evidence that the
tested assertions were complete. P2-007 and C1-008 were therefore reopened as
blocked correctness work and closed again by the Guide-4 batch in §2.8.

### 2.8 Guide-4 reacceptance gate · `gate:backlog:guide4`

The repository records a completed Guide-4 batch for:

```text
T6
T7
T8
T9
C1-008
P2-007
```

The batch covered:

- external-evidence capability retention and fixed-requirement filtering;
- representation relations as static mode constraints;
- an oracle that derives fixed evidence requirements independently;
- the current-phase declaration weld across all four planning entry points;
- accurate compiler implementation-status documentation.

Its recorded result, on the tree ending at the crate-state reconciliation
commit:

- `scripts/ci.sh`: every available lane passed; `cargo-audit` skipped because
  unavailable, so the run was partial-green;
- real Meson compile and test: 10/10 passed, including check-generated
  (architecture publications byte-identical), labels, plans, census, and the
  reproducibility stamps;
- document byte reproducibility: deferred because paper inputs were unchanged;
- final clean-tree check: empty.

Identity impact of the batch: the architecture identity, generated
architecture publications, public schema, and dependency graph are unchanged;
no realization public identity or compiler-plan identity exists.

### 2.9 Guide-5 placement gate · `gate:backlog:guide5`

The repository records a completed Guide-5 batch for:

```text
C1-009
P2-010
```

The batch delivered, all crate-private:

- typed execution cases derived per feasible proof plan (representation
  fixed by the plan; sponsor absent/present the only expansion; four
  semantic cases per pilot across the plan set);
- relation discharge classification on independent axes (boundary, scope,
  activation, multiplicity) with an exhaustive per-variant match;
- abstract carrier roles with input-family coordinator anchors, quantified
  every-member roles, and availability-based eligibility;
- target-independent layout requirements citing relation IDs as semantic
  owners;
- exact feasible placement enumeration with explicit typed limits, no
  partial results, and inclusion-minimal candidate retention;
- an independent exhaustive placement oracle restating every hard
  predicate, agreeing with production on every pilot scope and fifteen
  adversarial synthetic cases;
- end-to-end pilot analysis: two feasible plans per pilot, four cases per
  pilot, 36 feasible placements per compact-ASH plan and 216 per
  live-transfer plan, with the combined scope proven to be the exact
  product of its operations.

Findings recorded for later work:

- multi-operation placement factorizes as a product across operations;
  the analyzed-program work should consider per-operation storage;
- compact ASH declares sponsor recognition without a sponsor cardinality
  relation, so its sponsor family is bounded by no cardinality census;
  live transfer declares both. This is a realization-level asymmetry, not
  a compiler defect.

Its recorded result, on the tree ending at the placement documentation
commit:

- `scripts/ci.sh`: every available lane passed; `cargo-audit` skipped
  because unavailable, so the run was partial-green;
- real Meson compile and test: 10/10 passed, including check-generated,
  labels, plans, census, and the reproducibility stamps;
- document byte reproducibility: deferred because paper inputs were
  unchanged;
- final clean-tree check: empty.

Identity impact of the batch: the architecture identity, generated
architecture publications, public schema, and dependency graph are
unchanged; no compiler-plan identity was minted; placement limits are an
explicit parameter, not public policy.

### 2.10 Guide-6 coverage gate · `gate:backlog:guide6`

The repository records a completed Guide-6 batch for:

```text
P2-011
C1-010
C1-013
```

Sponsor-cardinality ruling: the Guide-5 finding that compact ASH declared
sponsor recognition without sponsor cardinality was confirmed as a
realization omission and repaired. The realization now declares PLAIN_LBTC
input cardinality (minimum zero, maximum the fee-sponsor-input-max bound)
and output cardinality (minimum zero, maximum exactly one) for compact ASH,
with recognition-before-cardinality dependencies, and typed validation
derives the expected family-relation census from the architecture operation
rows so a missing cardinality or recognition relation now fails derivation.
The architecture identity and generated publications are unchanged; both
pilots now have identical placement shape.

The batch delivered, all crate-private:

- relation-indexed coverage requirements: stable boundary-carrying
  requirement identities, an exhaustive typed mutation catalogue with no
  wildcard arm, positive/negative/inactive-valid derivation per boundary,
  per-operation aggregation with no cross-operation product;
- carrier coverage compressed from the feasible placement product
  (216 placements per plan compress to at most three assignment
  alternatives per relation-case) with multiplicity preserved;
- accepted semantic-projection requirements answered by the evidence role
  of their own boundary;
- typed coverage dependencies: two-pass definition census and reference
  resolution, a direct Petgraph graph, strict active-descendant collateral
  closure, canonical SCC diagnostics with cycles forbidden;
- an independent coverage oracle restating the acceptance matrices,
  collateral closure by repeated scan, and SCC by mutual reachability,
  plus property-generated bounded instances;
- end-to-end pilot coverage: two plans and four cases per pilot,
  46 and 48 relation-cases per plan, graphs of 344/365 and 362/489
  nodes/edges, combined scope factorizing per operation, sponsor opacity
  over the complete aggregate projection, determinism under declaration,
  candidate, placement, and relation-case permutations.

Defects surfaced and repaired inside the batch:

- the independent oracle caught the projection binder assigning the
  target-execution evidence role to every boundary's accepted projection;
  each boundary's projection now carries its answering role, guarded in
  the owning module;
- pilot integration removed a quadratic carrier-compression rescan over
  the combined placement product; identical verdicts, polynomial cost.

Its recorded result, on the tree ending at the coverage documentation
commit:

- `scripts/ci.sh`: every available lane passed; `cargo-audit` skipped
  because unavailable, so the run was partial-green;
- real Meson compile and test: 10/10 passed, including check-generated
  (architecture publications byte-identical), labels, plans, census, and
  the reproducibility stamps;
- document byte reproducibility: deferred because paper inputs were
  unchanged;
- final clean-tree check: empty.

Identity impact of the batch: the architecture identity, generated
architecture publications, public schema, and dependency graph are
unchanged; the realization relation census expanded internally to match
architecture cardinalities; no realization or compiler identity exists or
was minted; no new dependency entered.

### 2.11 Guide-7 Phase-2 exit gate · `gate:backlog:guide7`

The repository records the completed Guide-7 batch for:

```text
P2-012
P2-013
C1-014
```

together with the second-review preflight repairs recorded in §5.2.

The batch delivered, all crate-private:

- relation-indexed requirement bundles whose capability, source, and
  evidence unions equal the candidate aggregates exactly in both
  directions;
- factorized operation analysis: cases, relation-case requirement
  bundles, carrier eligibility, placements, layout, coverage, and the
  coverage dependency graph derived per operation, with the existing
  exact search reused unchanged on the operation-restricted census;
- one scoped analyzed program per scope, binding its exact typed source,
  reporting partial architecture scope with the eleven out-of-scope
  operations derived from the architecture census, explicit lifecycle
  incompleteness, and unresolved external evidence, with the execution
  report excluded from the stable semantic projection;
- a corruption-resistant assembly validator rechecking all thirteen
  closure families against fresh component re-derivation, with
  sixty-four corruption mutations each rejected for a focused typed
  reason; coverage closure is exact in both directions, resolving SR2-09;
- an independent assembly census oracle restating every expected census
  without the production expectation derivation, a requirement-union
  oracle, and a two-level factorization oracle;
- end-to-end pilot acceptance: the acceptance relation lists are the
  censuses, rejected proof/mode pairings are absent, and the absence of
  a compiler identity is enforced by two compile-time locks;
- the combined-scope product regressions demoted to explicit phase-exit
  lanes after replacement signal landed, taking the ordinary compiler
  suite from minutes to seconds.

Measured pilot results: 23 and 24 relations, 46 and 48 relation-cases,
216 placements and 69 layout requirements per operation factor, coverage
graphs of 344/365 and 362/489 nodes/edges, 4 combined plans storing 432
placements while denoting the never-materialized 46,656-member product,
2 unresolved substrate-conservation obligations, 3 outstanding lifecycle
exits, 11 of 13 architecture operations outside scope.

Phase-2 exit evidence, on the final batch tree:

- declared MSRV lane, rustc 1.88.0: `scripts/ci.sh` — every available
  lane passed; advisories and the Meson contract skipped in that
  environment;
- current stable lane, rustc 1.97.1: same result, after repairing four
  new stable-toolchain pedantic findings;
- nightly lane with `CI_REQUIRE_MESON=1`: every available lane passed
  including the mocked Meson contract and its census-audit edge;
  advisories skipped;
- real Meson compile and test: 10/10 passed;
- the three explicit product regressions: factor sizes 216 × 216,
  product 46,656, exact set equality against the retired global
  enumeration;
- document byte reproducibility: PASSED — fresh and reused builds
  produce identical PDF bytes, and the reused build refreshes the
  source epoch;
- `cargo-audit`: SKIPPED, tool unavailable; the run is partial in the
  advisory dimension only;
- dependency evidence: `Cargo.toml` and `Cargo.lock` unchanged;
- final clean-tree check: empty.

One transient test failure during a deliberately concurrent triple-lane
run did not reproduce in isolation and the affected lane was rerun
uncontended and passed; the recorded results are the uncontended runs.

Identity impact of the batch: Attestation version, architecture schema and
hashes, and generated publications unchanged; no realization or compiler
identity exists or was minted; no compiler publication was added; the
deployment-profile identity remains dormant behind validation; no new
dependency entered.

### 2.12 Guide-8 target foundation gate · `gate:backlog:guide8`

The Guide-8 batch delivered the third-review preflight (§5.3, all fifteen
rows), the `tripod-target-elements` crate, the minimal public
compiler target-requirement boundary, and the `tripod-tapscript`
capability adapter. Starting revision 0.3.1-dev; batch branch merged
fast-forward after this record.

Preflight: sponsor erasure is fee-sponsor-flow-role based with two-point
overlap rejection and a kernel balanced-theft regression; the compiler
refuses protocol-claimed ordinary-L-BTC scopes with a typed error and both
pilot analyzed projections were verified byte-identical; the complete
analyzed-program validator runs on the construction path with the execution
report validated; projections compare exactly and duplicate-sensitively;
the coverage graph uses one enabler-toward-dependent orientation with
endpoint classes checked on insertion; exact-search counters are checked
before increment.

Target foundation: 38 opcodes reviewed against upstream Elements source
with review provenance recorded in the target reference and excluded from
the semantic projection; the tapscript leaf version is the reviewed 0xc4;
arithmetic failure retains operands and pushes false; asset and value
introspection push payload and prefix separately; 27 encoding classes,
37 capabilities, and 20 evidence requirements validate with mutation and
permutation coverage; sighash dimensions are recorded unreviewed and
commitment equality and authenticated opening remain unsupported; the
development binding rejects production, zero identifiers, and version
mismatch and carries no credential field. The compiler exposes
RequiredCapability with a complete census and a read-only requirement set
constructible only from fully validated analysis; the adapter classifies
all thirteen capabilities exhaustively with an independent oracle, the
authorization rows block on the unreviewed sighash dimensions, and the
complete-backend-pattern claim is unconstructible by type.

Gate evidence, all on the merged batch tree: ci.sh eleven lanes green
under the pinned SDK toolchain with the advisory lane RUNNING through an
isolated cargo-audit shim (149 crates scanned, no advisories) and the
mocked Meson contract required; canonical meson compile and test 10/10;
the workspace suite green across 46 test binaries; document byte
reproducibility not required this batch because no paper input changed
(the one-line realization-document correction is covered by the label and
weld lanes). Toolchain note: the SDK extension bundles clippy 0.1.88
against a 1.99-nightly cargo, and the current upstream clippy 0.1.99 adds
an assert-is-empty lint; its machine-applicable rewrites were adopted, and
roughly ninety non-machine-applicable test assertion sites remain OPEN as
drift work for when the lint reaches a supported gate toolchain.

Identity impact: Attestation version, architecture schema and hashes, and
generated publications unchanged; no target, deployment, compiler, or
report digest minted; Cargo.lock gained exactly the two first-party
package stanzas; no third-party dependency entered.

### 2.13 Guide-9 target-native gate · `gate:backlog:guide9`

The Guide-9 batch closed the fourth-review register (§5.4, all eleven
rows), made the target contract sound, and delivered the typed instruction
core with development target-native primitive evidence. Starting revision
0.3.2-dev; batch branch merged fast-forward after this record.

Contract soundness: a reviewed-Elements trust state distinct from generic
validation, constructible only by the first-party derivation or exact
typed equality; a typed success algebra with retained operands and
condition-discriminated alternatives re-reviewed against upstream with
line provenance; six cross-subcontract welds; transitive capability-status
closure under an explicit strength order; package-owned encoding
interpretation; a reviewed literal-push contract whose minimality cascade
is ordered data and whose relay-versus-consensus enforcement is typed; a
five-byte timelock operand class making the sequence disable flag
stateable. Identity and publication repairs: the draft and release
validators return the only wrappers accepted by the public architecture
identity functions with published values proven unmoved; publication
freshness now includes the required mode with in-place mode-only repair;
ADR-016 records the bounded grandfathered-recipe exception.

Instruction core and adapter: typed instructions, checked stack items,
canonical minimal-push serialization, a strict supported-subset parser
with round-trip equality, and an abstract stack validator carrying every
success alternative and all three failure shapes under checked work
limits, agreeing with an independent bounded oracle; the static adapter
consumes only the reviewed contract and answers both compiler censuses
with exhaustive matches and both-direction exact equality.

Native evidence: a secretless conformance harness with a lock-step
protocol that fails closed on every malformation and types mock runs as
unable to satisfy the gate; a 398-case fixture census whose expectations
come from the typed contract and independent published vectors; a
first-party executor adapter driving a disposable wallet-free
elementsregtest node, judging consensus fixtures by block validation and
relay fixtures by mempool acceptance, with an explicit reason-string
mapping that leaves target-collapsed causes coarse rather than guessed.
The run: 398 of 398 cases passed, 18 of 18 required evidence rows, zero
infrastructure errors, report bytes identical across fresh-node runs and
the build lane; triage attributed and repaired every first-contact
divergence in its owning module, including a transcription error in a
published signature vector that only real verification exposed. Executor
provenance: Elements Core v28.99.0-6f43e3ffe730, rebuilt from a clean
tree at the reviewed revision after an earlier stale binary predating the
elliptic-curve stack-size fix was refused for evidence; synthetic
development identifiers 32x09 and 32x07.

Gate evidence on the merged tree: ci.sh eleven lanes green under the
pinned SDK toolchain with the advisory lane running through the isolated
shim and the mocked Meson contract required; canonical meson compile and
test eleven of eleven; the native lane on the batch tip with the stamp
produced; document byte reproducibility not required because no paper
input changed. Honest bounds: sighash semantics remain unresolved by
design, commitment equality and authenticated opening remain unsupported,
confidential-value conservation remains transaction-level evidence, the
report completeness is partial-unresolved-remains, and the Guide-9
concept text stays in the session archive because carrying it breached
the plan-tree hard byte cap.

Identity impact: Attestation version, architecture schema and hashes, and
generated publications unchanged; no digest of any kind minted; the
conformance package is the only Cargo.lock addition; no new third-party
dependency beyond workspace-existing crates plus a test-only PTY helper.

### 2.14 Guide-10 prototype gate · `gate:backlog:guide10`

The Guide-10 batch closed the fifth-review register (§5.5, all thirteen
rows), made the native evidence layer self-validating, and delivered
both Phase-3 constructor and arithmetic prototypes with accepted
research decisions. Starting revision 0.3.3-dev; batch branch merged
fast-forward after this record.

Evidence-boundary repairs: protocol and report schema 2 with strict
bounded NDJSON; a validated report wrapper that recomputes every case
status, evidence disposition, and summary with duplicate-sensitive
censuses in both directions; complete fixture projections in every
report row; a typed claim census beneath the broad evidence rows with
deliberately unresolved claims stated; reviewed development binding
retaining the exact target projection; observed executor environment
compared before any case runs; separated adapter, node, and integration
provenance with the checkout-fallback misattribution removed;
process-group timeout supervision with a descendant-survival regression;
the signature operand model made representable for empty, invalid, and
unknown-key paths; the relation-identity weld; and the pre-release
deployment-profile trust-state split.

Substrate: target contract V2 expands the reviewed census from
thirty-eight to fifty-five opcodes with seventeen compound-proof
primitives reviewed at the ADR-018 merged tip; canonical byte ordering
is recorded unsupported; the reach bound, that no reviewed primitive
reads below the third stack item, is machine-checked and governed every
compound layout; operand-conditioned result widths and width-only tweak
operands were review-based contract corrections.

Prototype decisions, recorded in the research owners: the
metadata-dependent constructor is an accepted prototype under the
dynamic-metadata-leaf candidate, with the successor derived on-stack
from the predecessor's authenticated bytes, one authenticated static
root, fixed-order branch hashing with creator-side nonce grinding, and
a machine-checked unspendable metadata leaf; exact wide floor
arithmetic is an accepted prototype under derived limbs at base two to
the twenty-six through the reviewed Euclidean sixty-four-bit division,
witnessing only the five semantic amounts. Public declassification
remains the open third prototype.

Native evidence on the merged tree: both complete matrices ran against
the reviewed executor, Elements Core v28.99.0-0b3bffd93138 at the
ADR-018 intended tip 0b3bffd with upstream base b7fc5d0 and local
topics fix/tapscript-opcodes and notes, chain elementsregtest, through
the check-target-elements-prototypes command: constructor continuity
thirty-six of thirty-six cases with nine of nine required claims, and
wide floor thirty-nine of thirty-nine cases with eleven of eleven
required claims, each report byte-identical over two consecutive gated
runs. First native contact surfaced two constructor fixture defects and
zero program defects; both fixtures were repaired with review
justification. Measured resources: constructor 699 script and 924
witness bytes with the 520-byte element bound binding at a fifth of
maximum; wide floor 523 script and 606 witness bytes. The reports
themselves are run output and are not checked in; the research files
record the run facts as historical narrative and no digest of any report
is kept, which is what the identity register's §3.5 stop record calls
for (G11-R11).

Gate evidence on the merged tree: the working debug lane green through
every wave with the workspace suite growing from 1913 to 2151 tests;
ci.sh lanes green under the pinned SDK toolchain with the advisory lane
through the isolated shim; canonical meson compile and test green; the
native primitive lane and both native prototype lanes run on the batch
tip. Honest bounds: the five newly typed failure classes are unexercised
by any native row; compound fixtures state no admitted failure-class
set, so observed classes are recorded but not compared; the request
still carries the fixture expectation under the typed boundary; mock
runs remain typed as unable to satisfy any gate; nothing here is an
operation, linked bundle, transaction ABI, calibration, or production
claim.

Identity impact: Attestation version, architecture schema and hashes, and
generated publications unchanged; the target contract version moved to
V2 as an explicit reviewed revision; no digest of any kind minted; the
nix crate moved from development to production dependency of the
conformance package for process-group supervision; no new third-party
package entered the graph.

---

### 2.4 Guide-11 declassification gate · `gate:backlog:guide11`

The Guide-11 batch closed the sixth-review register (§5.6, all sixteen
rows), redesigned the evidence boundary, reviewed the target's
confidential-value machinery, built an independent commitment oracle, and
answered the last open Phase-3 foundational prototype question with a
selected initial representation policy. Starting revision 0.3.5-dev, tree
clean; the Guide-10 constructor and wide-floor decisions stood accepted
and were not reopened; the public-declassification note opened at *Open;
prototype required*.

Preflight (§5.6 owns the register): all fourteen numbered rows are DONE,
thirteen CONFIRMED and one RECLASSIFIED, and `G11-H01` is DONE with
fifty-five of fifty-five opcodes welded. `G11-H02` is not a gate item and
is not closed — it is a standing prohibition against adding issuance
realization scope until the authority fields are enforced or externally
evidenced, and none is proposed.

Evidence boundary: canonical wrappers for all three matrices; ad hoc
reports demoted below any gate; transcripts bound to their exact subjects
and requests; expectations removed from executor requests, which is what
made the protocol schema **3** — a breaking change, numbered as one, with
revision 2 kept as a historical value nothing parses as revision 3.
Primitive report schema 2, prototype report schema 1, and a string schema
of its own for the fresh-process handoff. Exact case, claim, and evidence
censuses; environment rechecked after the run as well as before; failed
completeness refused; report bytes deterministic over two consecutive
runs. Provenance is Elements Core at the ADR-018 merged tip on
`elementsregtest`, with adapter, framework, node, binary revision,
intended tip, upstream base, local topics, observed network, and observed
genesis each a separate field; the executor class is explicitly selected
and fails closed, and Meson declares no trust by default.

Target review (§5.7 carries the rows): the commitment relation is the
conceptual one with both terms positive, over a thirty-two-byte big-endian
scalar; conservation closes as an exact tally to the identity with no
excess term; commitment equality has no reviewed primitive and is exactly
`Unsupported`. The decisive result is that authenticated opening has **no
complete on-script form**, for three independent reasons: the generator is
not derivable on script, the encoding domains disagree, and a
witness-supplied parity byte is bound to nothing. The selected sighash
profile is the default all-outputs non-anyone-can-pay key-path spend, read
out of the witness rather than asserted. The target-contract version did
not move: these are facts about the reviewed revision, not a change to the
algebra.

Dependency and oracle (§5.8 carries the rows): nothing was added, and
`Cargo.lock` is unchanged across the batch. The bindings to the
zero-knowledge secp256k1 fork were refused on independence — they wrap the
same C library the node vendors, so their agreement would be a tautology —
and `target-elements` stays standard-library-only. Sixty-eight upstream
vectors reproduce exactly, every pinned byte string was independently
recomputed in another language, and the oracle predicts the same
commitment bytes a real node produced for three observed openings. The
construction-library leg stays **absent** and is reported as absent.

Confidential transactions: twelve rows stated, eleven executed, one
deferred as a typed row. Ten of the eleven agreed with expectations
written before the target was asked — three accepted, six rejected at
consensus before script, and the hidden-confidential-output row accepted,
which is its stated expectation and the finding that consensus does not
police hidden value. The eleventh, several confidential inputs to one
explicit output, **could not be constructed** and reached no target
verdict: residual blinding has nowhere to go without a blinded output.
Randomness is `FixtureInputsOnly`, since the node's blinding cannot be
seeded. Failure layers are decided by the mempool reason, because block
validation answers every amount and script failure with one string.

Deferrals: no opening prototype was built, and the public-committed
representation defers on the same three blockers, so no amount domain,
relation, parity coverage, durable public evidence, or resource
measurement exists for either. Two shapes are refused outright — the proof
outline as instantiated on the reviewed primitives, under the parity-loss
criterion, and an opening checked off-script and asserted on-script, under
the rule against a host library being the only verifier — while the class
is deferred rather than rejected, because a pattern proving the required
normalization or negation was not found, which is not the same as shown
impossible. The capsule is not applicable while they are deferred: it
carries an opening and the fields binding it to one output, and there is
neither.

Normalization: private to explicit with private change is **built and
run**; full consumption is not constructible, per the conservation row
above. Owner authorization is real and output-committing — every consumed
input carried a single sixty-four-byte witness item, a Schnorr signature
with no trailing sighash byte. Amount, owner, explicit asset, and object
role are preserved and checked, and the explicit and normalized semantic
projections agree over those properties. Closure is exact multiset
equality in both directions, because a subset test passes a hidden output
and a count test passes a swap. Nine of nine mutation rows agreed with
expectations committed before the run; three of them are consensus-valid
transactions the report layer alone refuses, and the target accepted all
three, which is the substantive finding rather than a gap. Resources are
not measured: the path adds no verifier program to measure.

Fresh-process lifecycle: a real operating-system boundary — the
constructing process publishes and exits, each reading process is a fresh
process and node with a distinct pid, the wallet destroyed and the chain
kept. The public record is a typed twelve-field schema refusing unknown
fields and banning field *names* that denote owner-private material.
Sixteen of sixteen rows agreed over two passes against the canonical
matrix rather than the run record. The reading process locates by block
locator, parses with its own deserializer, rebuilds the output script from
the public address alone, and spends its own funds — it cannot spend the
owned object, so permissionless future *use* is demonstrated and
permissionless future *maintenance* is not.

Final representation policy: the boundary is **explicit**, reached from a
private value by **owner-authorized normalization to an explicit output
carrying private change**. Lateral transfer is supported directly in both
representations and needs no boundary; public and permissionless
maintenance and formula-bound payout are explicit only; direct private
support and public-committed representation are deferred with named
blockers; full private consumption is unsupported, which is a
constructibility finding rather than a target rejection. The matrix, cell
by cell against landed evidence, is in
[the research owner](../research/public-declassification.md).

Reports carry the `Experimental` role throughout: they establish what the
target does and nothing about a candidate being selected. Minimality is
**not** claimed and no minimality report exists — the result is explicitly
not disclosure-minimal, and the normalization path's three
declassifications are every one `DeploymentPolicy`, each stating the
alternative a deployment that wanted the fact private would have to take.
Zero failed canonical cases and zero required infrastructure errors. The
reports are run output, are not checked in, and no digest of any is kept.

Identity impact: nothing moved. Across the whole batch the diff touches no
file under `packages/model/generated`, `packages/architecture`, `papers/`,
or `docs/`, so the Attestation version, realization major and letter,
architecture schema, architecture semantic hash, architecture behavioural
hash, anchor-set hash, and generated architecture publications are
all unchanged. No realization, compiler, target, opening-pattern, capsule,
or report identity was minted. The deployment-profile identity remains
dormant and not production-release-valid. The target-contract version
stays at V2, with V1 removed from the supported set in preflight rather
than bumped. The only schema that moved is the native executor protocol,
to revision 3. Two new modules compute hashes and neither mints an
identity: the oracle's generator derivation is the reviewed upstream
recipe evaluated as arithmetic, and the conservation module's
fixture-scalar derivation is domain-separated disposable-chain test
material that authorizes nothing.

Planning handoff: the research note is closed with a selected initial
policy and its residuals, D005 records that policy under the proof
alternatives it already admitted, the Phase-3 card records the result
without claiming the exit, and both package READMEs carry the new modules.
The next guide consumes this policy rather than reopening it.

Honest bounds: the conservation, normalization, and lifecycle lanes record
and do not gate — the executor drivers, published assets, and Meson
targets that would make them refusable in CI are not built (`G11-W7-08`).
Both public-committed conservation rows remain deferred
(`G11-W7-09`). Policy-resource evidence is `UnresolvedByDesign`. Nothing
here is an operation, a linked bundle, a transaction ABI, a calibration,
or production output.

## Closed review registers

Moved from [the backlog](../backlog.md) §5.2–5.5 on 2026-08-19: the
finding registers of the second, third, fourth, and fifth static
reviews, every row of which was remediated and closed.

### 5.2 Second-review findings · `tab:backlog:findings-sr2`

| ID | Severity | Status | Finding |
|---|---|---|---|
| `SR2-01` | High | DONE | CI could report green without the tracked-entry mode audit; lint alias and mocked contract omitted census-audit. |
| `SR2-02` | Medium | DONE | Root-use policy was reused as the observed root effect, rejecting valid succession under succession-or-termination. |
| `SR2-03` | Medium | DONE | Lifecycle relations and lifecycle graph declarations were not bidirectionally welded. |
| `SR2-04` | Medium | DONE | Placement validation accepted noncanonical, duplicate, and surplus-layout assignments. |
| `SR2-05` | Medium | DONE | Placed proof-plan validation did not compare the exact offered and placed sets. |
| `SR2-06` | Medium | DONE | Public deployment-profile hashing accepted unvalidated profiles; hashing now requires the validated wrapper and the identity stays dormant. |
| `SR2-07` | Medium | DONE | Open-flow observation normalization did not enforce reference sides, cross-flow uniqueness, or anchor exclusion. |
| `SR2-08` | Low | DONE | Representation-conditional activation was keyed by mode only; it is now keyed by object and mode. |
| `SR2-09` | Low | DONE | Coverage shape validation was weaker than derivation; the analyzed-program validator compares exact independently re-derived coverage projections in both directions, gate record §2.11. |
| `SR2-10` | Low | DONE | Compiler package-index status and review provenance had drifted; both reconciled. |
| `SR2-H1` | Hardening | DONE | Tempfile publication can replace generated files with owner-only permissions; closed by the SR3-04 repair in the Guide-8 preflight wave. |
| `SR2-H2` | Hardening | DONE | Dirty-tree document-reproducibility probe skips with exit 0; closed by the SR3-06 repair in the Guide-8 preflight wave. |

The DONE rows were repaired in the Guide-7 preflight wave with focused tests
and the working debug lane; the batch full gate is recorded with the Guide-7
exit record. Open-flow complete partitioning deliberately remains a
sponsor-isolation relation verdict rather than observation parsing, so an
unclaimed member stays an evaluable semantic failure. The two hardening rows
were parked with explicit activation conditions; the third review re-raised
both, which satisfies those conditions, so they are active again and tracked
with the §5.3 register.

### 5.3 Third-review findings · `tab:backlog:findings-sr3`

The third static review reported two passes over the tree recorded in §2.1.
First-pass findings carry SR3 identifiers; second-pass findings retain the
review's S2 identifiers.

| ID | Severity | Status | Finding |
|---|---|---|---|
| `SR3-01` | High | DONE | A malformed same-owner citation whose one-sided parenthesis test is suppressed by an unrelated parenthesis elsewhere on the line falls back to a bare occurrence and silently mints the label. Citations now classify by their immediate parenthesized group with a focused malformed-group diagnostic in both parsers; the previously test-locked vulnerable assertion was inverted; a corpus scan showed zero live classification changes. |
| `SR3-02` | Medium | DONE | Census filesystem traversal suppresses read-dir and entry errors, so an unreadable tree with an empty declared group can verify as an empty census. Traversal failures are now typed census-unreadable diagnostics consumed by verification, with unreadable-directory, failed-entry, and scoped-derivation regressions; container-nested fences are additionally rejected. |
| `SR3-03` | Medium | DONE | The complete analyzed-program validator is not on the production construction path; the constructor runs only the narrow assembly-closure check. The constructor now runs the complete fresh re-derivation validator, and the execution report is validated rather than excluded; the measured pilot construction cost stays seconds-scale. |
| `SR3-04` | Medium | DONE | Atomic publication staging leaves published public files with owner-only tempfile permissions; supersedes the parked SR2-H1. A typed publication-mode helper now sets 0644 or 0755 on the staged file before rename in batch publication, report writing, and the shell sync path, with absent, changed, and unchanged destination mode tests. |
| `SR3-05` | Medium | DONE | Git-derived checker census arguments are expanded unquoted, so an unusual tracked path can split or alter checker argv. A shared census-argument library now emits shell-quoted role-tagged argv re-read as quoted words, fails closed on an empty census, and a loud preflight audit rejects tracked paths outside a safe grammar; derived argv verified byte-identical on the live tree. |
| `SR3-06` | Medium | DONE | The document-reproducibility gate exits 0 after skipping its reused-build epoch probe on a dirty tree; supersedes the parked SR2-H2. The gate now fails a dirty tree by default, reserves exit 0 for both advertised checks passing, and reports an explicit partial status under an opt-in flag; fixing it exposed and repaired a signing-config defect that had prevented the epoch probe from ever running here. |
| `SR3-07` | Low | DONE | Proof-search rejection statistics count source and constructibility failures as capability rejections. Local feasibility now returns a typed rejection reason with a matching counter per cause; repairing it exposed a miscounted source-derivation branch and a counter with no increment site, both corrected. |
| `SR3-08` | Low | DONE | The package index still says complete analyzed pilots are open, contradicting the recorded Guide-7 state. The index row now states the implemented-internally claim with the public boundary and target adapter absent. |
| `S2-01` | High | DONE | Sponsor erasure is keyed to the ordinary L-BTC object family instead of fee-sponsor flow membership, erasing protocol-role amounts that future operations must read. The model and realization layers derive the sponsor region from exact fee-sponsor flow membership with two-point overlap rejection, protocol-role readability, and a kernel balanced-theft regression; the compiler derives a three-valued flow role from the declared open flows and refuses protocol-claimed ordinary L-BTC scopes with a typed error, so the remaining family-shaped guards rest on an enforced gate whose lifting condition is documented at each use; both pilot analyzed projections were verified byte-identical. |
| `S2-03` | Medium | DONE | The coverage dependency graph mixes opposite edge orientations against its documented prerequisite-to-dependent convention. All edges are normalized to enabler-toward-dependent with declared endpoint classes checked on insertion; a reversed prerequisite edge between same-class endpoints remains caught by census comparison, and that limit is stated. |
| `S2-04` | Medium | DONE | Analyzed-program coverage-graph validation compares sets rather than exact projections, silently accepting duplicated nodes and edges. Stored and required projections now compare exactly with duplicate, order, and length-preserving corruption regressions. |
| `S2-05` | Low | DONE | Exact-search counters use unchecked addition and can overflow instead of returning the promised typed complexity failure. A checked counter module tests the limit before incrementing under an inclusive root-counted rule, diagnostic counters saturate, and boundary regressions pin a budget of one, exact-limit completion, and limit-plus-one. |
| `S2-06` | Low | DONE | The paper subproject declares C as a project language with no C target, requiring an undocumented compiler. The language declaration is removed; setup no longer probes a C toolchain and the mocked contract passes. |
| `S2-07` | Low | DONE | Compiler planning status remains stale and a resolved factorization question is still listed as open, although the drift finding was marked closed. The resolved per-operation-factor decision is now recorded as a rule and removed from the open questions; the genuinely open boundary, ownership, objective, and identity questions remain. |

The Guide-8 preflight wave owned SR3-01 through SR3-08 and S2-01 through
S2-07; all were closed before the compiler target-requirement boundary
became public, gate record §2.12.

### 5.4 Fourth-review findings · `tab:backlog:findings-r2`

The fourth static review reported two passes over the tree recorded in §2.1.
Reconfirmed first-pass findings carry R2-C identifiers; second-pass additions
carry R2-N identifiers. The Guide-9 preflight owns the register; the review
ranks the target-contract rows as Phase-3 blockers before the typed
instruction core consumes the contracts.

| ID | Severity | Status | Finding |
|---|---|---|---|
| `R2-C01` | High | DONE | The opcode stack contract carries one success result sequence, so primitives with alternative valid success shapes, retained operands, or discriminated encodings cannot be represented. A typed success algebra now carries fixed, retained-operand, and condition-discriminated cases re-reviewed against the upstream interpreter with recorded line provenance; the retained timelock operand, issuance push order, and independently confidential issuance amounts are all representable, and the timelock contradiction with its resource row is repaired. |
| `R2-C02` | High | DONE | Redundant target subcontracts are only locally validated; signature, timelock, issuance, confidential-value, resource, and evidence views can contradict one another inside one validated definition. Six weld validators now require cross-view agreement with focused globally-contradictory mutation coverage and a welded-reviewed-contract control. |
| `R2-C03` | High | DONE | Capability assessment reads only direct primitive statuses, so a reviewed capability whose transitive prerequisite is unsupported still assesses as usable. Statuses now compose under an explicit strength order and a capability may not exceed its weakest transitive prerequisite; the reviewed registry verified closure-coherent with no downgrades, and the target validator rejects violations with the offending edge named. |
| `R2-C04` | Medium | DONE | Encoding numericity is derived from byte-order presence, making the byte-order validation circular and letting a numeric class validate without any order. Interpretation is now an exhaustive package-owned match over the encoding class with independent V1 shape expectations, making the missing and spurious byte-order rejections genuinely reachable. |
| `R2-C05` | Medium | DONE | The adapter consumes only the capability census and drops the compiler's external-evidence-role census, so a new role can disappear without adapter work. The assessment set now carries both censuses with exhaustive role matching, both-direction exact equality, typed duplicate and mismatch rejections, and an independent evidence-role oracle. |
| `R2-C06` | Medium | DONE | The active architecture semantic hash predates the domain-separated identity recipe; the exception or migration must be explicit, never a silent prefix change. ADR-016 now records a bounded grandfathered-recipe rule for the architecture semantic and anchor-set hashes, welded into the identity table; future identities keep the domain-separated form. |
| `R2-C07` | Low | DONE | The package index still marks the implemented target packages as planned, and the root README layout omits the current package set; paper end-marker comment drift is noted but deferred with the paper-edit obligations. The index rows and the root layout now state the active honest claims; the paper end-marker fixes remain deferred to the next deliberate paper edit. |
| `R2-N01` | High | DONE | A caller-assembled target definition can acquire the same validated wrapper as the built-in reviewed Elements definition, so the reviewed trust state is caller-assertable. A distinct reviewed-Elements wrapper is now constructible only by the first-party derivation or by exact typed equality with it; mutated-but-coherent definitions stay generic under public-API tests. |
| `R2-N02` | Medium | DONE | The adapter accepts a combined target-and-binding value but ignores the deployment half entirely, so assessments are invariant under activation, network, and resource-override differences. The assessment API now consumes the reviewed static definition and is named static; the combined value is no longer named anywhere in the adapter crate, and deployment-aware assessment remains deliberately deferred until a real consumer exists. |
| `R2-N03` | Medium | DONE | Public architecture identity functions accept unvalidated architecture values, so an invalid architecture can bear the active semantic hash recipe. The draft and release validators now return borrowing validated wrappers that are the only public path to the identity and publication functions; unchecked projections are crate-private for mutation tests, and an equivalence test plus the pinned gates prove no published identity moved. |
| `R2-N04` | Medium | DONE | Compare-if-changed publication checks bytes only, so mode-only corruption survives indefinitely, including a synced helper binary that lost its executable bit. Destination freshness now means bytes and required mode; mode-only mismatches are repaired in place without rewriting bytes, the repair is visible in the typed result, both sync scripts repair equal-byte destinations, and a mock-toolchain shell test runs the full matrix. |

### 5.5 Fifth-review findings · `tab:backlog:findings-sr5`

The fifth static review reported two passes over the tree recorded in §2.1.
The second pass consolidates the first: first-pass identifiers R5-01 through
R5-07 map into the SR5 register as SR5-01, SR5-02, SR5-03, SR5-06, SR5-08,
SR5-12, and SR5-13 respectively; the remaining SR5 rows are second-pass
additions. The Guide-10 preflight waves owned the register and closed
every row before prototype evidence was recorded; the gate record is
§2.14. Each row states the reviewed defect and the repair the batch
delivered.

| ID | Severity | Status | Finding |
|---|---|---|---|
| `SR5-01` | High | DONE | The public native-report gate validates only required evidence rows that happen to be present; an empty, row-deleted, or relabeled report census passes. Repair delivered: complete owner validation returning a validated report wrapper with exact duplicate-sensitive case and evidence-row censuses and recomputed statuses, dispositions, and summary. |
| `SR5-02` | High | DONE | Broad evidence rows pass when only a subset of their semantic claim has cases: issuance-absent cases complete issuance introspection, rejection-only signature cases complete signature semantics, explicit-form cases complete confidential encodings, and consensus resource cases complete relay-policy resource evidence. Repair delivered: typed claim-level evidence census beneath the broad requirement identifiers with exact required-claim coverage. |
| `SR5-03` | High | DONE | Native reports retain only the case ordinal, expected outcome, observed outcome, and status, omitting the exact script, stack, context, enforcement layer, leaf version, and expected resources; two fixture sets can produce indistinguishable reports. Repair delivered: complete canonical fixture projection embedded in each report row. |
| `SR5-04` | High | DONE | A development binding validated against one target definition can later combine with a different definition of the same contract version, because the binding retains only the version. Repair delivered: retain the validated target projection in the binding or introduce a reviewed-development-binding wrapper constructible only against the reviewed Elements definition. |
| `SR5-05` | High | DONE | The static signature model cannot represent the target's documented empty-signature and unknown-public-key-type behavior: exact-width operand types reject the empty form before the failure contract applies and exclude the succeeds-without-verification path. Repair delivered: operand alternatives with conditional failure semantics, plus native unknown-key vectors before signature evidence is complete. |
| `SR5-06` | Medium | DONE | Reported network and genesis identifiers are caller declarations copied through the pipeline; the executor never reports what chain it actually ran, and the recorded synthetic identifiers confirm the fields are run labels. Repair delivered: a typed executor environment observation compared against the validated binding before any case executes. |
| `SR5-07` | Medium | DONE | Relation identifiers are not generically welded to relation bodies; the kind vocabulary contains an unused member and lacks a member for expression predicates. Repair delivered: an exhaustive relation-identity validator deriving expected kind and subject from every body variant, with the kind vocabulary corrected. |
| `SR5-08` | Medium | DONE | Native executor provenance cannot express the executed tip, upstream base, and local-topic census that ADR-018 requires, and a checkout-HEAD fallback can misattribute a binary's revision. Repair delivered: separated adapter, node, and integration-provenance fields with no checkout fallback into the binary-reported revision. |
| `SR5-09` | Medium | DONE | Executor timeout kills only the immediate child, so a real adapter's node, temporary datadir, cookie, and inherited pipes can outlive the run. Repair delivered: process-group supervision with graceful-then-forced group termination and a descendant-retaining regression test. |
| `SR5-10` | Medium | DONE | Executor protocol lines are read into an unbounded buffer, so one unterminated line can exhaust memory before typed rejection. Repair delivered: explicit per-phase protocol record limits enforced with bounded reads. |
| `SR5-11` | Medium | DONE | Schema-2 deployment profiles can acquire a type named validated deployment release despite the documented production-blocking ABI gap. Repair delivered: split structural profile validity from production-release validity, with the latter unconstructible under schema 2. |
| `SR5-12` | Low | DONE | Blank protocol lines and trailing blank data are silently accepted despite the documented fail-closed protocol. Repair delivered: strict NDJSON framing with blank and trailing records rejected. |
| `SR5-13` | Low | DONE | Public status documentation is stale: the root README denies the instruction core and omits the conformance package, and the target package still claims no native evidence exists. Repair delivered: reconcile the statements with the package-boundary claim that the static crate owns requirements while development native evidence lives in the conformance package. The record was premature by one file: the two READMEs and `evidence.rs` were reconciled, but `evidence_registry.rs` kept the claim that none of its requirements has been evidenced against any node, which is what `G12-R16` found and closed. |

The review also recorded three lower-severity observations to close alongside
the owning repairs: accepted responses are not protocol-shape validated
against the advertised handshake capabilities, public fixture construction
does little semantic context validation, and the Python adapter tolerates an
empty context script path where the fixture script is nonempty.

## Sixth-review findings

Moved from [the backlog](../backlog.md) §5.6 on 2026-08-19: the sixth
static review’s finding register, complete and closed. The one
repository link inside it is re-based one level for this file’s depth;
nothing else is changed.

### 5.6 Sixth-review findings · `tab:backlog:findings-sr6`

The sixth static review reported two passes over the tree recorded in §2.1.
First-pass and second-pass findings are consolidated into one SR6 register
whose identifiers are the Guide-11 preflight identifiers `G11-R01` through
`G11-R14`, with the two hardening rows carried as `G11-H01` and `G11-H02`.
The Guide-11 preflight waves own the register; the summaries below are the
required dispositions recorded in that guide's own preflight table.

Register state after the batch: **COMPLETE**. All sixteen rows are
disposed — every numbered row `G11-R01` through `G11-R14` is DONE, and
`G11-H01` is DONE with the stack resource rows welded. `G11-H02` is not
a gate item and never was: the guide's gate bullets name no issuance
condition, and the row's own disposition is a standing prohibition —
issuance realization scope may not be added until the authority fields
are enforced or externally evidenced. None is proposed, so the
constraint stands open by design rather than blocking the gate.

The register closed at the preflight gate, and the batch that followed it
— the target review, the dependency decision and oracle, the
confidential-transaction substrate, the candidate dispositions, the
normalization prototype, and the fresh-process lifecycle — reopened none
of its rows. The declassification result those waves produced is recorded
in §2.4, and the research owner is
[public-declassification.md](../research/public-declassification.md).

The review basis is tree `0.3.4-dev…`, not the working tree: under the
guide's own rule each finding is a hypothesis until it is reproduced
against the working tree, and is then either fixed, disproved with a
typed argument, or reclassified with a narrower assurance claim. The
Wave-0 column records that adjudication. Wave 0 repaired nothing, so
every row opened TODO; Wave 1 closed the two claim-laundering rows,
Wave 2 closed the transcript-rebinding row, Wave 3 closed the gate,
provenance, and startup-cleanup rows, and Wave 4 closed the remaining
six: relation subjects, target versions, the signature weld, retry
classification, the report digests, and this reconciliation.

| ID | Priority | Status | Wave 0 | Finding |
|---|---:|---|---|---|
| `G11-R01` | P0 | DONE | CONFIRMED | A transcript can be rebound to another fixture census, prototype matrix, target, or deployment binding; bind the transcript to its exact subjects and requests. |
| `G11-R02` | P0 | DONE | CONFIRMED | Primitive claims can be manufactured by attaching claim-bearing case metadata to an unrelated script; gate only a canonical validated primitive plan. |
| `G11-R03` | P0 | DONE | CONFIRMED | Prototype claims are caller-authored and can certify a trivial true script as constructor or wide-floor evidence; gate only relation-specific canonical matrices. |
| `G11-R04` | P1 | DONE | CONFIRMED | Consensus resource cases are credited to policy-resource evidence because evidence ownership derives from case ID without the enforcement layer; derive evidence from the complete fixture and correct the plan class. |
| `G11-R05` | P1 | DONE | CONFIRMED | The primitive native gate can accept a report whose summary is failed; gate every canonical case and reject failed completeness. |
| `G11-R06` | P1 | DONE | CONFIRMED | ADR-018 execution provenance is recorded but not enforced by evidence gates; validate executable provenance before gate eligibility. |
| `G11-R07` | P1 | DONE | CONFIRMED | Meson defaults an executor to reviewed-non-mock; require explicit caller selection and fail closed. |
| `G11-R08` | P1/P2 | DONE | CONFIRMED | Relation bodies can admit several semantic relation subjects; derive exactly one canonical subject from each body. |
| `G11-R09` | P1/P2 | DONE | CONFIRMED | Target V1 is advertised as supported while current validation applies the V2 census and algebra; remove V1 support or implement genuine version dispatch. |
| `G11-R10` | P1/P2 | DONE | CONFIRMED | The signature weld omits unknown-key behavior and much of the success algebra; weld the complete signature relation. |
| `G11-R11` | P2 | DONE | CONFIRMED | Bare prototype report digests persist despite the recorded no-report-identity decision; remove them or admit typed retained report references. |
| `G11-R12` | P2 | DONE | RECLASSIFIED | Current backlog state contradicts recorded Guide-8 through Guide-10 completion and duplicates finding IDs; reconcile current state and enforce unique IDs. |
| `G11-R13` | P2/P3 | DONE | CONFIRMED | Process-group establishment failure can leave the direct child unreaped; kill and reap on every pre-supervisor failure. |
| `G11-R14` | P3 | DONE | CONFIRMED | Constructor retry retries internal-key defects no metadata nonce can repair; share a typed retryability predicate. |
| `G11-H01` | Hardening | DONE | CONFIRMED | Opcode resource stack-growth rows are not generically welded to success and non-aborting failure effects; derive and compare exact maximum stack growth. |
| `G11-H02` | Future blocker | TODO | CONFIRMED | Issuance observations carry authority fields the realization evaluator does not yet enforce; add no issuance realization scope until enforced or externally evidenced. |

Wave-1 evidence for the two closed rows. `G11-R02`: the evidence path
takes only `CanonicalPrimitiveFixtureSet`, whose one constructor is
`canonical_fixture_set`, and `evaluate` regenerates the census and
compares complete projections — asserted by
`g11_r02_a_trivial_true_script_cannot_bear_signature_evidence`,
`g11_r02_an_arbitrary_census_is_refused_on_provenance_not_completeness`,
`g11_r02_changing_a_canonical_fixture_member_removes_gate_eligibility`,
`g11_r02_a_canonical_case_whose_claims_change_is_refused`, and
`g11_r02_permuting_canonical_declaration_order_is_harmless`. `G11-R03`:
`prototype_gate` reads only a report validated against a
`ConstructorPrototypeMatrix` or `WideFloorPrototypeMatrix`, whose rows are
private and whose one constructor each is the canonical generator —
asserted by `g11_r03_a_trivial_leaf_cannot_certify_the_wide_floor_relation`,
`g11_r03_the_canonical_wide_floor_matrix_is_the_evidence_subject`,
`g11_r03_a_canonical_case_with_one_added_claim_fails`,
`g11_r03_a_canonical_case_with_one_removed_claim_fails`,
`g11_r03_replacing_the_canonical_program_under_the_same_case_name_fails`,
`g11_r03_replacing_the_constructor_successor_program_fails`, and
`g11_r03_raw_bytes_are_not_reported_as_a_typed_program`.

Wave-2 evidence for `G11-R01`. `ExecutionTranscript` retains the target
projection, the deployment projection, and the exact subject sent for
every case; `evaluate`, `evaluate_prototypes`, and both report validators
compare their own inputs against those retained values by exact typed
equality, and the executor's observed environment is compared a second
time when a report is built and again at the validator's own boundary.
Protocol revision 3 removes every expectation from an executor request,
so the subject a transcript retains is the whole of what crossed the
wire. Asserted by
`g11_r01_a_transcript_cannot_rebind_to_a_deployment_it_never_ran_on`,
`g11_r01_a_report_cannot_name_a_script_the_executor_never_ran` (an
equal-width substitution, which defeats the old width-only defence),
`g11_r01_a_report_cannot_name_an_initial_stack_the_executor_never_ran`,
`g11_r01_a_response_for_a_case_never_sent_is_refused`,
`g11_r01_a_case_that_was_never_sent_is_refused`,
`g11_r01_a_case_sent_and_never_answered_is_refused`,
`g11_r01_a_prototype_construction_cannot_be_substituted`,
`g11_r01_a_changed_expected_verdict_is_refused` (refused by the canonical
comparison rather than by the transcript, since an expectation is not part
of the subject and no longer crosses the wire at all),
`a_request_carries_no_expectation_of_any_kind`, and
`an_executor_of_the_previous_revision_fails_loudly`.

Wave 1 also closed the arbitrary-census route that two open rows were
reproduced through. `G11-R01`'s script-substitution reproduction moved to
the experimental path, where Wave 2 flipped it: the binding is a property
of the transcript rather than of the trust state. `G11-R05`'s
construction added a fixture to the canonical census and is no longer
expressible; Wave 3 repaired the gate itself, and its test now records
both halves closed by putting that report at the gate directly.

Wave-3 evidence, beyond the flipped suites listed below. The provenance
comparison rule is minimum-width exact prefix — a reported revision of at
least seven lowercase hex digits that is an exact prefix of the expected
full tip, with equality as its width-forty case — chosen because the
reviewed adapter reads an abbreviation out of the node's version line and
a binary embeds no full object identifier. The `G11-R04` invariant found
four further required rows owning no required claim (encoding, stack
rearrangement, byte string, verification); each is recorded as a typed
exception naming where its claim-level decomposition actually lives.
`G11-R07` has no Rust regression by nature: it is verified by configuring
all four states, of which two now fail.

`G11-R08` through `G11-R10` are closed, so the bar they set on the
public-declassification prototype is met.

Wave-4 evidence. `G11-R08`: `expected_relation_subject` is a function of
the body, and `validate_relation_identity` compares its one result for
equality — the four flipped `g11_r08_` suites assert that each body's
former alternatives are now refused, and
`g11_r08_changing_only_the_relation_subject_fails_derivation` mutates
the subject alone across five body classes. The internal relation keys
moved for the two closures and the two operation-wide policies of each
pilot; `packages/model/generated/` is byte-identical and no pinned
identity moved, which is the recorded consequence the guide called for.
`G11-R09`: `SUPPORTED` holds V2 alone, `supported(1)` fails, and
`validate_target_definition` now checks the offered revision, since the
crate names revisions as public constants and a caller reaches V1
without the constructor. `G11-R10`: `weld_signature` derives one
complete expected behaviour from the subcontract and compares every
signature opcode against it; six mutations of the reviewed contract are
refused, each naming the signature weld. `G11-R14`: one `retryable`
predicate, consulted by both retry implementations, with the canonical
construction's exact bytes pinned across the change. `G11-R11`: the two
research files keep their run facts and no digest. `G11-R12`: this
section, §3.3, the one-line backlog, the duplicate identifier, and
`DI-F03`, plus a `check-plans` rule that a backticked row identifier
names one row per table.

`G11-H01` is **DONE**, closed by Wave 5 on source evidence. Wave 4 had
derived the generic weld and run it before any code changed: fifty-three
of the fifty-five primitives agreed exactly, and the two verifying
signature forms declared their branching counterpart's figure instead of
their surviving one. Source review settled it in the declared rows'
favour. The interpreter shares one case block between a branching
primitive and its verifying counterpart, popping the operands, pushing
the truth value, and only then — if the opcode is the verifying one —
popping that value again on success and aborting on failure
(`src/script/interpreter.cpp:1476-1499` for the transaction-signature
pair and `1689-1734` for the stack-message pair). The verifying form
therefore transiently occupies its counterpart's depth, which is what
"at any point during its execution" measures and what a stack scheduler
must size against.

The weld landed with that transient as a typed term, derived from the
verifying primitives the signature weld already distinguishes rather
than from new per-opcode data, so no reviewed row moved. All fifty-five
rows now agree, altstack growth is checked to stay zero, and the source
citation is recorded in (`tab:elements-ref:verifying-transient`). The
regressions refuse a growth row inconsistent with its stack contract, a
verifying form declaring only its surviving depth, a success form moved
without its row, and a nonzero altstack claim; the reviewed contract
validates unmutated as their control.

Wave-0 evidence, one pointer per row. The Rust pointers name the
crate-internal `guide11_reproductions` suites, which pass by asserting the
defective behaviour and which the repairing waves invert.

```text
G11-R01  conformance g11_r01_a_transcript_rebinds_to_a_deployment_it_never_ran_on
         (Wave 2 flipped: g11_r01_a_transcript_cannot_rebind_to_a_deployment_it_never_ran_on)
G11-R02  conformance g11_r02_a_trivial_true_script_bears_signature_claims
         (Wave 1 flipped: g11_r02_a_trivial_true_script_cannot_bear_signature_evidence)
G11-R03  conformance g11_r03_a_trivial_leaf_certifies_the_whole_wide_floor_relation
         (Wave 1 flipped: g11_r03_a_trivial_leaf_cannot_certify_the_wide_floor_relation)
G11-R04  conformance g11_r04_consensus_resource_cases_pass_the_policy_resource_row
         (Wave 3 flipped: g11_r04_consensus_resource_cases_do_not_reach_the_policy_resource_row)
G11-R05  conformance g11_r05_the_gate_accepts_a_failed_report
         (Wave 3 flipped: g11_r05_the_augmented_census_route_to_the_gate_is_closed, plus
          g11_r05_a_report_whose_summary_says_failed_is_refused and
          g11_r05_a_failed_canonical_case_never_reaches_evidence)
G11-R06  conformance g11_r06_the_gate_accepts_a_run_with_no_workspace_provenance
         (Wave 3 flipped: g11_r06_a_run_with_no_workspace_provenance_is_refused, and the
          six further provenance regressions beside it)
G11-R07  meson.options target_native_executor_class, value reviewed-non-mock
         (Wave 3 closed: the option defaults to unselected, an unclassified executor path
          and a reviewed class missing its ADR-018 provenance both fail configuration)
G11-R08  realization g11_r08_one_closure_body_admits_two_subjects
         (Wave 4 flipped: g11_r08_one_closure_body_has_exactly_the_side_as_its_subject,
          the root, projection, and delta/open-flow suites beside it, and
          g11_r08_changing_only_the_relation_subject_fails_derivation)
G11-R09  target-elements g11_r09_a_v2_body_stamped_v1_validates
         (Wave 4 flipped: g11_r09_a_v2_body_stamped_v1_is_refused,
          g11_r09_the_historical_revision_is_not_constructible_by_number,
          g11_r09_every_supported_revision_has_a_constructible_definition, and
          g11_r09_a_v2_definition_missing_a_v2_primitive_is_refused)
G11-R10  target-elements g11_r10_a_contradictory_unknown_key_rule_still_validates
         (Wave 4 flipped: g11_r10_a_contradictory_unknown_key_rule_is_refused, plus the
          five further mutations and g11_r10_the_reviewed_signature_contract_remains_valid
          as their control)
G11-R11  plans/research/state-constructor.md and wide-arithmetic.md, report digest rows
         (Wave 4 closed: both rows removed, the run facts kept as narrative, each file
          stating why no replacement hash follows, and the Guide-10 gate record corrected)
G11-R12  duplicate DI-F02 rows, DI-F03 active under a DONE parent, and section 3.3
         still listing both accepted prototypes as not implemented; the opening
         condition line is already reconciled, so the finding narrows to those three
         (Wave 4 closed: the incidental defect renumbered DI-F04 so the standing
          identifier stays with the finding the DI-001 census cites, DI-F02 and DI-F03
          marked DONE against what the DI-003 narrative delivered, section 3.3 and the
          one-line backlog naming declassification alone, and check-plans now failing a
          repeated row identifier within one table)
G11-R13  executor.rs ExecutorSupervisor::adopt takes the child by value and drops it
         on the establish failure path, and the standard child destructor neither
         kills nor waits
         (Wave 3 closed: an_unadopted_child_is_killed_and_reaped and
          a_run_refused_at_startup_leaves_no_unreaped_child)
G11-R14  conformance g11_r14_an_invalid_internal_key_is_retried_to_exhaustion
         (Wave 4 flipped: g11_r14_an_invalid_internal_key_fails_after_one_attempt,
          g11_r14_the_canonical_constructor_classifies_the_same_permanent_defect,
          g11_r14_a_tree_defect_fails_after_one_attempt,
          g11_r14_only_the_nonce_movable_tweak_defects_are_retryable,
          g11_r14_exhaustion_reports_the_configured_attempt_count, and
          g11_r14_the_canonical_construction_is_byte_identical)
G11-H01  no weld reads OpcodeResourceCost::maximum_stack_growth at all, the timelock
         weld included, so the field is welded to nothing
         (Wave 4 derived the weld and ran it without landing it: 53 of 55 primitives
          agree exactly; CheckSigVerify declares -1 against a surviving -2 and
          CheckSigFromStackVerify declares -2 against a surviving -3, both carrying
          their branching counterpart's figure. Wave 5 closed it: source review
          confirmed the declared rows, the interpreter pushing the truth value and
          only then popping it for the verifying form, so the weld landed with a
          typed transient term and all 55 rows agree. Guarantees are
          g11_h01_every_reviewed_row_matches_its_derived_stack_growth,
          g11_h01_only_the_verifying_forms_carry_a_transient_above_their_surviving_depth,
          g11_h01_the_branching_counterparts_declare_their_settling_depth,
          g11_h01_a_growth_row_inconsistent_with_its_stack_contract_is_refused,
          g11_h01_a_verifying_form_declaring_its_surviving_growth_is_refused,
          g11_h01_changing_a_success_form_without_its_growth_row_is_refused,
          g11_h01_a_nonzero_altstack_growth_row_is_refused, and
          g11_h01_the_reviewed_contract_remains_valid as their control)
G11-H02  ObservedIssuance::authority is never read in realization, and authority_input
         only for referential existence in observation.rs
```

## Seventh-review preflight findings

Moved from [the backlog](../backlog.md) §5.9 on 2026-08-20: the
seventh static review’s preflight register, complete and closed. The
sixteen rows are the Guide-12 preflight identifiers, adjudicated by
Guide-12 Wave 0 and repaired across Waves 1a, 1b, and 2. Nothing is
changed by the move.

### 5.9 Seventh-review preflight findings · `tab:backlog:findings-sr7`

The seventh static review reported two passes over the tree recorded in
§2.1. Its rows are the Guide-12 preflight identifiers `G12-R01` through
`G12-R16`, and the Guide-12 preflight table owns the required
dispositions. Guide-12 Wave 0 adjudicated every row against the working
tree rather than against the review's snapshot: each row below carries a
disposition, a DONE-or-OPEN state, and the artifact or file location
that establishes it.

Register state after Wave 2: **COMPLETE**. Fifteen rows are CONFIRMED
and DONE and `G12-R08` is RECLASSIFIED and DONE, so the blocking rule
(`gate:guide12-exec:preflight`) is satisfied and no confirmed P0 or P1
row remains open. Wave 0 adjudicated and reproduced without repairing,
Waves 1a, 1b, and 2 repaired: each row's witness was written to assert
the defect and was flipped by the wave that fixed it, so every witness
below now stands as the guarantee that its repair holds.

| ID | Priority | Disposition | Evidence and trust boundary |
|---|---:|---|---|
| `G12-R01` | P1 | CONFIRMED, DONE — Wave 1a | The three fields were public and `RevisionId::new` admits a seven-digit prefix, so a struct literal reached the gate with an abbreviated expectation that `ExpectedExecutorProvenance::new` would refuse, and the whole match then compared prefix against prefix. Repaired by making the width a type: `FullRevisionId` has one validating constructor, the expectation's members are private behind read-only accessors, `RevisionId::matches_full` takes the full-width type so a prefix cannot be an expectation, and the gate re-asserts the width before comparing. `RevisionId::full` is replaced by `FullRevisionId::new`, and the validated provenance carries the full identifiers rather than the reported text. Boundary: executor provenance, ADR-018 tip attestation. Witness `an_abbreviated_expectation_is_refused_before_it_can_be_expected`, with the public surface checked by `an_expectation_is_reachable_only_through_its_validating_constructor`. The struct-literal route is now a compile error rather than a runtime refusal. |
| `G12-R02` | P1 | CONFIRMED, DONE — Wave 1a | `anchor_set_hash` took any string iterator and framed by newline join, so one name holding a newline and two names had one preimage. Repaired by validation before identity: `AnchorName` enforces the three-part label grammar and refuses the separator by name, `ValidatedAnchorSet` is all-or-nothing over its members, and the hash accepts only that set. The recipe is untouched — same domain prefix, same newline join over the same sorted distinct names — which the published-recipe test still checks against a digest computed outside this crate. Boundary: anchor-set pin, the manifest value release validation refuses to leave unset. Witness `no_anchor_name_can_hold_the_separator_the_framing_joins_with`. The label checker now builds the validated set and reports a diagnostic rather than hashing a census it could not parse; all 136 harvested attestation anchor names satisfy the grammar, so the pinned value is unchanged. The recipe test's vector gained an area segment on its one two-part name, which was never a well-formed anchor name; the same independent computation over the old vector still returns the digest the test carried before. |
| `G12-R03` | P1 | CONFIRMED, DONE — Wave 1b | The four `emit-*` binaries in `target-elements-conformance` were absent from every `cli_common` user: no shared panic hook, no stdout TTY refusal, `std::env::args().nth(1)` argv handling, and plain-text stderr diagnostics that interpolate the caller's path. Repaired by splitting each command in two rather than by decorating it. The documents move to `src/emit.rs` — `conservation_matrix_document`, `normalization_matrix_document`, `normalization_report_document`, and `lifecycle_report_document` — and the binaries keep only what cannot be tested any other way: `clap` argument parsing behind `BaseArgs`, `install_json_panic_hook` before parsing so an early panic still fails closed, and `run_stdout_json_command`, which supplies the terminal refusal, the ADR-010 exit classes, and the write. Two things stopped being crashes on the way: the normalization matrix asserted its claim conserves and now returns a refusal a caller can observe, and the lifecycle report's gates moved into the library beside the document they gate. The run-record path became `--run-record PATH`; no in-tree caller passes it positionally, the emitters being invoked by hand under the Guide-11 evidence procedure, and the output is now the compact one-JSON-result line ADR-010 specifies rather than a pretty-printed block, which the Python readers consume unchanged through `json.load`. Boundary: shipped-binary subprocess contract, ADR-010. The source-read disposition is discharged: `src/tests/emit_tests.rs` exercises all four documents without spawning anything — the conservation requests carry the case, schema, and subject and nothing else, every executed row is asked and no deferred row is, exactly one normalization row is the unmutated claim, and the two report builders refuse a record that is not a census, a record carrying no public handoff, and a malformed check. The contract half was also confirmed by running the built binaries: a clean single-line JSON result with empty stderr and status 0, `--help` as a control-plane record, an unknown argument as a usage refusal with status 2, and a missing run record as status 1 with a JSON stderr diagnostic that names the failure without interpolating the path — `G12-R04`'s property holding through this migration. `src/emit.rs` is the seam Wave 2's subprocess harness consumes: it checks the contract, this checks the content. Wave 2 built that harness: `tests/emit_subprocess_contract.rs` runs all four commands as real subprocesses over one census, so a contract holding for one and not another shows as a difference between them. It proves the clean run is one compact JSON line with an empty stderr and status 0, help and version are control-plane records with an empty stdout, an unknown flag and a missing or positional `--run-record` are usage class 2, an absent record, a malformed one, and a well-formed one that is not a run are each status 1 with JSON-only stderr, no command defines a credential argument, and — under a real pseudo-terminal, the property inherited from `cli_common` that nothing had ever exercised — every one of the four refuses a terminal stdout as usage class 2 before doing any work. Every failure class is also asserted to name no path: the fixture record sits under a distinctively named directory, and no stream repeats it. |
| `G12-R04` | P1 | CONFIRMED, DONE — Wave 1b | The executor raised `AdapterError` carrying `one_line(completed.stderr)` from the `elements-cli` child, and that note reaches first-party protocol records as `observed_detail` and `detail`. The module docstring's claim that the harness never reads child stderr described the harness, not this path: this side never read the executor's stderr, but the executor read its own child's into a field this side does read, so the same class of bytes arrived by the back door. Repaired by making the boundary transitive rather than by filtering what crosses it. The RPC note now states only the method the harness itself named and the client's exit status, both fixed and typed, and says that the reason is omitted; the reason is logged on the adapter's own stderr, which the harness nulls, so the diagnostic is honest about where it went rather than silently shorter. The same treatment is applied to the framework's taproot-construction exception, whose message is uncontrolled third-party text that can carry an operator's filesystem path: the record now names the exception's type, which is bounded and typed, and the message goes to the nulled stderr. The adapter's module docstring states the second direction of the contract, and `protocol.rs` records that the closure is transitive and why it previously was not. Boundary: target-adapter to first-party diagnostic and report layer. Python lane; the source-read disposition is discharged by the two call sites being the only readers of a child's stderr in the file, which `grep` over `.stderr` now shows is a single `log` call. The runtime half remains blocked on a live node, and no unit witness can stand in for it: what a real `elements-cli` writes on a real failure is exactly what cannot be observed here. |
| `G12-R05` | P1/P2 | CONFIRMED, DONE — Wave 1a | Report ingestion indexed responses into a `BTreeMap` keyed by mutation name, so a second answer for one row silently replaced the first, and the report loop read the matrix rather than the census, so a response naming a row the matrix does not carry was dropped without a word. Repaired by moving the ingestion into `normalization_report`, the module that owns the report, where `ingest_normalization_responses` refuses rather than repairs: a duplicate, an unexpected row, and an unanswered row are each a named defect, and the census is compared with `canonical_mutation_matrix` in both directions. Boundary: normalization safety report. The source-read disposition is discharged — the library seam is what makes the row testable, and the witness is `a_normalization_run_answers_the_matrix_exactly_once_each`. The binary keeps its own argument and output handling; the ADR-010 migration of the shipped emitters remains `G12-R03`. |
| `G12-R06` | P1/P2 | CONFIRMED, DONE — Wave 1a | `matrix_is_complete` consulted `self.passes.first()` alone, so a later pass could omit rows; it also asked only whether a row was present, so a duplicated row satisfied it. The emitting binary gated on `boundary_holds` and `matrix_is_complete` and never called `passes_agree`, and `passes_ran_in_distinct_processes` was satisfied by one pass. Repaired by making the census exact for every pass alike: each pass must carry every canonical row minus the globally unbuilt ones, once each and nothing else; `passes_ran_in_distinct_processes` now requires distinct attempt ordinals as well as distinct pids; and `cache_independence_established` states the whole claim in one predicate, which the emitting binary gates on. Boundary: fresh-process lifecycle evidence, the cache-independence claim. Witness `matrix_completeness_is_decided_by_every_pass`, whose required regression is pass one complete and pass two empty, with `two_passes_under_one_ordinal_are_one_pass_recorded_twice` beside it. |
| `G12-R07` | P2 | CONFIRMED, DONE — Wave 1b | The consensus judgement took the substring to the first `)` after the wrapper, and two mapped messages carry a `)` of their own, so each truncated to a string the class table does not hold and the rejection lost its class. The two are `Signature must be zero for failed CHECK(MULTI)SIG operation`, which truncated after `CHECK(MULTI`, and `OP_CHECKMULTISIG(VERIFY) is not available in tapscript`, which truncated after `OP_CHECKMULTISIG(VERIFY`. Nothing failed loudly, which is the sharp edge of the row: a lost class reads as a target that refused for an unclassified reason rather than as a parser that mislaid one. Repaired by closing the wrapper's own parenthesis through depth counting, which is what parsing the exact outer wrapper means when the payload may itself contain parentheses, and by expressing the relay judgement's already-correct whole-wrapper reading through the same `script_error_in` helper, because two spellings of one rule are two things that can disagree. An absent or unbalanced wrapper returns nothing rather than a guess, so a caller still distinguishes a reason that is not a script verdict from a script verdict naming nothing. Verified by recomputation over the whole class table rather than over the two known offenders: all 34 mapped messages were extracted in both the consensus lane, embedded in the surrounding client stderr, and the relay lane, as the bare reject reason, with no misses, and both negative shapes return nothing. Entangled with `G12-R04` and resolved together: the consensus script error is stated only on the `elements-cli` child's stderr, so removing that stderr from the note would have silently disabled every consensus classification. `AdapterError` therefore carries the client's bytes as `client_detail`, separate from the note that reaches records, and the judgement classifies them into the typed vocabulary rather than propagating them — an unmapped message becomes no class, never a record's text by another route. Boundary: target failure classification. Python lane; source-read disposition discharged by exhaustive recomputation. |
| `G12-R08` | P1/P2 | RECLASSIFIED, DONE | The count disagreement the review saw was already gone from the backlog and the code: the matrix states nine rows and the Wave-11 record states 18 of 18 over two passes. Two statements had not caught up — the declassification research file reported 16 of 16 and stopped its finding list at `G11-W11-05`, and the Phase-3 card's status blockquote broke off mid-clause after the exit-gate sentence. Both are reconciled here, which is the whole of the row: a prose row's reproduction is the textual comparison, so finding it and fixing it are one act. Boundary: active planning-document agreement. Reclassified from a census disagreement to two stale statements. |
| `G12-R09` | P0/P1 | CONFIRMED, DONE — Wave 1b | Both sides declared protocol revision 3 while implementing two different schemas. The executor wrote `observed_openings` on every conservation response and `NativeConservationResponse` carried `deny_unknown_fields` without that field, so the Rust type could not read the Python's own answers; and the lifecycle exchange had no Rust protocol type at all, so the one workload whose evidence is a public record was read out of an untyped value tree. Repaired by unifying rather than by splitting the schemas: revision 4 states the union both sides were already implementing, and the two constants move together, because a bump that reached one side alone would reproduce the fault it exists to end. `ConservationOpening` is now a declared member of the conservation response, and `LifecycleStepRole`, `LifecycleCaseId`, `LifecycleSubject`, `NativeLifecycleRequest`, `LifecycleCheck`, `LifecycleSpend`, and `NativeLifecycleResponse` give the lifecycle step the typed records every other workload had. The openings join the infrastructure refusal, which is the adjudication the row required: an opening is read back out of a transaction the node created and confirmed, so one beside a layer saying the execution never happened is a value with no possible provenance, and the section 7.4 three-way comparison would be resting on it. `NativeLifecycleResponse::validate_shape` states the same rule over the handoff, checks, spend, and witness figures, exempting only the detail, because a failure is entitled to a reason. A check's two sides are now carried as text and the agreement is decided over the values before either is spelled, so a member that was sometimes a number and sometimes a flag is gone; `emit-lifecycle-report` parses the checks into `LifecycleCheck` and refuses a malformed one instead of defaulting its name to the empty string and its agreement to false. Boundary: harness-executor protocol. Witnesses `a_conservation_response_round_trips_through_its_own_type`, `an_infrastructure_conservation_response_carries_no_openings`, `a_verify_lifecycle_answer_is_read_by_its_protocol_type`, `an_infrastructure_lifecycle_answer_carries_no_observation`, and `a_lifecycle_request_is_read_by_its_protocol_type`, the round trips being asserted against the adapter's literal wire shape rather than against this crate's own serialization agreeing with itself. Phase-4 blocker, now cleared. |
| `G12-R10` | P1 | CONFIRMED, DONE — Wave 1a | The nonce field was typed `PointParityConvention::QuadraticResidue` beside a comment saying no parity claim is made, while its committed prefixes are the compressed-point pair. Repaired by removing the claim as a separate statement rather than by correcting it: `ConfidentialFieldEncoding` no longer takes a convention, and `parity` is read off the committed prefix pair, so a field naming a convention its bytes do not use is not a value anyone can write. The nonce therefore reports `CompressedOddness`, which is what the standard compressed pair records, and the reviewed table now exhibits the disagreement the opening blockers rest on instead of flattening it — the alternative of an opaque no-claim state was rejected because it would have deleted the table's own instance of `EncodingDomainMismatch`. Boundary: reviewed target facts, the opening-feasibility reasoning that rests on the two conventions differing. Witness `the_nonce_field_claims_a_parity_convention_its_prefixes_do_not_use`, which pins the nonce's pair against the encoding registry's compressed-public-key pair and then asserts both commitment fields disagree with the nonce. The accessor doc that called the low prefix a square y, and the neighbouring test comment that said the same of all three fields, are corrected with it. |
| `G12-R11` | P1 | CONFIRMED, DONE — Wave 2 | `resource_projection` skipped every non-opcode instruction outright, so no push opcode, width prefix, or payload byte reached the `ScriptBytes` total, and opcode script bytes came from the per-primitive cost table rather than from an encoded length. The undercount ran in the one direction that matters: a program pushing a key and a digest looked admissible against a target bound it exceeds. Repaired by deriving the dimension rather than by adding the missing terms — script bytes are now `TapscriptProgram::encoded_length`, so the projection cannot omit a form the encoder knows about. `encode` and `encoded_length` share `encode_instruction`, the single place an instruction's layout is decided, so the two cannot drift; the length is measured one instruction at a time into a reused buffer, so nothing assembles the whole program to size it. The other two dimensions stay a tally over the primitives, which is what the contracts price, and all three are now stated whatever the program holds, so a program that pushes and calls nothing reports zero rather than an absent dimension. Boundary: resource prediction against target bounds. Witness `every_pushed_payload_reaches_the_projected_script_bytes`, renamed with the flip: it recomputes 99 bytes by hand for the two-literal fixture rather than asking the encoder, then checks the projection against the serialized length over eleven widths chosen either side of every push-form boundary — 0, 1, 32, 64, 75, 76, 77, 254, 255, 256, and 520. `a_resource_projection_states_each_unit_separately` moves from 2 to 100 for the same reason. The conformance package had already declined to use the dimension, measuring `program.encode(target).len()` itself in `project_resources`; that reading was the correct one and is now the projection's too. Phase-4 blocker, now cleared. |
| `G12-R12` | P1 | CONFIRMED, DONE — Wave 2 | A pushed literal entered the abstract state as a width range, and its value survived only where it was a minimal script number. A single zero byte is not one — zero's minimal form is the empty push — so the walk carried nothing for the position, the verifying primitive branched both ways, and the success set gained a state the target cannot reach. Repaired by keeping the item rather than the number it would denote: the walk's side table is `KnownLiterals`, a position-indexed map of `StackItem`, and the number is derived only where a contract asks for one, which is the width-from-operand result. Truth and equality are questions about bytes, so they are answered from bytes: `reads_as_false` states the target's own reading once — every byte zero, or every byte but a trailing sign bit — and `LiteralFacts` applies it, in both directions. A verified literal read as false removes the successful form; one read as true removes the false-verification abort; two compared literals settle the inequality abort and the successful form alike. Removing an abort is a claim, so it is made only from exact bytes, and the width-only narrowing that existed before is kept for the case where no literal is known: a type admitting no width but zero admits no value but the false one. `SettledTruth` and `SettledEquality` are three-valued rather than Boolean, because not-known-false and known-true are different findings and only the second licenses dropping the abort. Boundary: abstract program validation, any liveness conclusion drawn from the success set. Witnesses `an_exact_pushed_literal_settles_the_verified_truth_value` and `two_exact_pushed_literals_settle_the_compared_equality`, replacing `a_pushed_zero_byte_leaves_a_success_state_the_target_cannot_reach`; between them they carry all six required regressions of (`tab:guide12-exec:tapscript-tests`). The siblings were sought rather than assumed absent and are covered in the same table: the empty push was already decided by its abstract type, `0x80` was not and is the same defect as `0x00`, and the non-minimal all-zero encodings `0x0000`, `0x0080`, and thirty-two zero bytes were all reported as possibly true and now are not, with `0x8000`, `0x0001`, and `0x81` beside them as the true neighbours. Every expected reading is written out in the table rather than recomputed from the rule the crate applies. What is deliberately not narrowed is the pushing comparison: `Equal` yields a `Bool` result whose value stays open, which over-approximates and so stays sound. Cost recorded rather than waved past: a state now carries bytes instead of numbers, bounded by the stack depth, the target's literal bound, and the state budget. Phase-4 blocker, now cleared. |
| `G12-R13` | P2 | CONFIRMED, DONE — Wave 2 | `decode` ran its parse loop to the end of the input and applied the instruction limit only in the constructor it finally called, so the work and the allocation were unbounded by the limit. Made visible by a script whose bytes pass the limit long before an invalid byte arrives: the parser reported the invalid byte rather than the limit, which it could only have reached by continuing. Repaired by moving the bound into the loop rather than by shortening the input: the head of each iteration refuses once the sequence already holds `MAXIMUM_PROGRAM_INSTRUCTIONS` and bytes remain, so no further byte is examined, no further instruction is allocated, and the answer is the limit whatever those bytes were. The constructor keeps its own check, which is still the reachable one for a caller assembling typed instructions directly. Boundary: untrusted-script parsing. Witness `decoding_stops_at_the_work_bound_before_it_reads_further`, renamed with the flip because the assertion is now the guarantee; `a_script_of_too_many_instructions_is_refused` continues to hold the at-limit script, so the bound admits exactly ten thousand instructions and refuses the ten thousand and first. |
| `G12-R14` | P1/P2 | CONFIRMED, DONE — Wave 1a | The infrastructure arm of the shape rules checked the failure class and the two stacks and returned, leaving the resource observation unread, so a response saying the run never happened could still carry interpreter figures. Repaired in `validate_observation_shape`, which the execution and prototype paths share rather than restate, so one edit closes both: the arm now also refuses `observes_interpreter`. The refusal could not be left to the advertised-observation rule further down, because the arm returns before reaching it — an executor advertising resource observation would have kept its figures. Where the line falls is `observes_interpreter`'s to say and is not redrawn: the script's size and the initial stack's depth are the fixture's own, restated by every executor, and stay legal, which is the decided disposition of the request-derived fields. The same gap is closed for `observed_witness_sizes` in `NativeNormalizationResponse::validate_shape` — they are read out of the transaction the response may not claim to have built and are the evidence the authorization profile rests on, so refusing the profile while admitting its support was the same contradiction; `claimed_outputs` stay legal, being resolved from the claim before any mutation. Boundary: evidence-layer separation. Witness `an_infrastructure_response_may_still_carry_interpreter_figures`, extended to the prototype path and to the fixture figures that remain legal, with `a_normalization_run_that_did_not_happen_reports_no_witness_sizes` beside it, which reaches the rule through `ingest_normalization_responses` as a run does. Phase-4 blocker cleared. |
| `G12-R15` | P1/P2 | CONFIRMED, DONE — Wave 1b | The runners read records with an unbounded `readline`, spawned without a new session so the node group was not theirs to kill, carried no total deadline over the row loop, and cleaned up only on the two handshake refusals — a failure inside the loop left the child unkilled and unreaped. Repaired by stating the four guarantees once, in `scripts/executor_supervision.py`, and putting all three runners on it, so they can no longer drift from each other or from the reviewed supervisor: `read_bounded` gives `readline` a size and refuses a record that reaches the bound without ending, `SupervisedExecutor` spawns with `start_new_session` so the executor and the node it starts are one group this runner can signal, `Deadline` bounds the whole run rather than any single row, and the context manager kills the group and reaps the child on every exit path including an exception from any row. The lifecycle lane's `AdapterProcess` keeps only what is particular to it — the three arguments that distinguish one process from another, and the capability its handshake must advertise — and its two reading passes share one deadline, because two passes each just under a per-process bound are unbounded together. Two things the Rust supervisor's constants could not simply be copied into: the cleanup grace is 180 seconds rather than that supervisor's five, because these executors stop a real node and delete the datadir they own, and killing that group after five seconds would leave the datadir behind and record the creator as killed rather than exited — which is exactly the exit status the lifecycle destruction record states as evidence the process boundary held; and that exit status is now captured before the child is released, since the boundary check reads it after the block that owned the process. Boundary: harness process supervision. Verified behaviourally rather than by reading: a probe confirmed the bounded read refuses a 200 000-unit flood carrying no newline, the spawned child leads its own group, a group kill reaches a grandchild the executor started, an exception raised inside the block leaves no live child, the deadline refuses once overrun, and the exit status survives cleanup. |
| `G12-R16` | P2 | CONFIRMED, DONE — Wave 1a | The evidence-registry module stated that every requirement is unresolved and that none has been evidenced against any node, while the readiness statement records development target-native evidence and the crate carries a native evidence path. Repaired by stating the boundary the module was reaching for instead of the status it was not entitled to report: an entry is immutable, so it carries no status by design rather than by omission, and whether evidence exists is the conformance package's question — development native evidence recorded there, production evidence absent. The wording now matches `evidence.rs`, which already carried the reconciled statement. No typed value gains a status, and the registry's own contents are untouched. Boundary: documentation only. This closes the half of `SR5-13` that its DONE record claimed prematurely — the two READMEs and `evidence.rs` had been reconciled and this module had not; the history record is annotated to say so. |

### 13.1 Integration tasks · `tab:backlog:draft-tasks`

| ID | Status | Task |
|---|---|---|
| `DI-001` | ACTIVE | Gap census: map each draft clause onto the present ADR-012/ADR-013 text, the ADR-016 identity rules, and the implemented labels package; classify every clause as already-implemented, divergent, or new; record the checker-engineering findings register. |
| `DI-002` | DONE | Adopt the corrected label calculus and kind registry as ADRs; delete the ADR text they retire; migrate the eight conflicting kind tokens to the registry forms corpus-wide per the ruling, with every citation updated in the same commit; record the hyphenated-area amendment and the local-extension register. |
| `DI-002b` | DONE | Replace the second-edition adopted drafts with the author's third-edition texts and repair every citation that dangled as a result. The third edition was verified against the audit register before the swap: all 26 findings fixed, the 7 defects among them included, and a mechanical re-audit clean — every per-document citation resolves, every mint is unique, and the kind registry's headline counts of 333 names, 349 rows, 208 kinds, 3 declared hybrids and 4 device classes derive exactly from its tables. Fourteen citations across three plan files were retargeted in the same commit as the swap, per the calculus's same-commit rule. Both adopting records were then refreshed to the editions they adopt: ADR-019 restates all seven adoption parameters and records that the checker implements the authorship warrant species only, ADR-020 names this repository as the registry's acceptee and recasts its extension register as the recorded extension set with located first-hand evidence, and a second kind-migration round settled two further tokens. |
| `DI-003` | DONE | Re-engineer the labels checker to the calculus: a single participation scanner shared by every check (mints, citations, links, hygiene, inline-code discipline), owner signatures with registered prefixes, imported and synthetic citations, anchor harvests, and the kind registry as the checker's kind vocabulary. W1 landed the participation scanner and the DI-F02 repair, W2 the seven adoption parameters as typed data with the kind vocabulary and warrant totality, and W3 the near-miss warnings, the companion attestation register, and the gate reconciliation across both records. |
| `DI-004` | DONE | Adopt the identity-adjudication procedure against ADR-016: classify every existing digest through the benefit criterion with admission records, and MIGRATE the two grandfathered recipes to domain-separated forms per the ruling — the architecture semantic and anchor-set hashes change under a recorded recipe migration, superseding the ADR-016 grandfather clause. Both halves delivered: the migration, and a census of the whole tree read from the owning code, recorded as six admission records and eleven stop records in the identities register. |
| `DI-005` | DONE | Interchange conventions: promoted wholesale per the ruling of 2026-08-19 and adopted by [ADR-022](../../adr/022-interchange-conventions.md) as the standing wire-format discipline for externally consumed documents. The edition adopted is the fourth, refreshed 2026-08-18: ceiling-based acceptance with downward-closed holding, open-companion tolerant validation, never-assigned stamps rejected as checkably false claims; the fourth-edition audit's two defects and one editorial finding all fixed on resupply, delta purely additive, nothing dangling. ADR-022 carries adoption data only — the boundary against ADR-010, the restated executor stop, the labeling and identity alignment records, and the no-implementation standing — and restates no clause of the discipline. Nothing is built: normative now, unimplemented by design. |
| `DI-006` | DONE | Three-part labels: the paper's 16 two-segment labels take the area `layer0`, a division's home being the document itself; `abs` takes the registry's `abst` per the ruling. 16 mints and 59 sites moved in one commit; the anchor-set pin moved, retired value reproduced; the semantic hash followed, the behavioural did not. Attestation kinds enforced. W-B holds the arity in one rule over every entry point: a label-intended occurrence that is not three-part fails as `malformed_label_shape`, and the realization's 20 two-segment divisions are frozen by name, not exempted as a surface. |
| `DI-008` | DONE | Promote the identity-adjudication draft: delete ADR-016 and mint [ADR-021](../../adr/021-identity-adjudication.md) as the adopting record, carrying only adoption data — the single local recipe convention, the current-identities table, the concrete identity chain, the recorded separation migration verbatim, the ADR-011 amendment linkage, and three recorded divergences. Every generic mechanism, class, flow, edge, duty, evidence and rejected-alternative section is deleted rather than restated: the draft holds them and is cited at the `PLAN` prefix. Ten imported citations to the retired owner were retargeted in the same commit, seven of them to the draft's own mints and three to ADR-021's local records. |

The ruling of 2026-08-19 promoted the identity-adjudication
draft: ADR-016 is deleted, the draft takes priority as the discipline,
and a single local-environment convention is added to the adopting
record for what deletion would otherwise lose. ADR-021 is that record.
The convention is the domain-separated construction ADR-016 prescribed,
kept as recipe data rather than as a competing prescription — the draft
reduces every construction question to its property table and leaves
the scheme to the recipe record, so the construction is adoption data
and not an amendment. This settles the first open ruling of the gap
census, which asked exactly whether the prescription moves into recipe
records or stays in the record.

Deletion took the overlap with it. The mechanisms table, the admission
and rejection lists, the identity classes, the three flow diagrams, the
immediate-edge, migration, verification and evidence rules, the seven
rejected alternatives and the generic gate items are the draft's, and
ADR-021 restates none of them. Three divergences are recorded rather
than reconciled: the adjudication walk is not a standing checked
requirement on new proposals, the evidence and release surfaces do not
exist, and the well-founded-graph rule is vacuous with one link built.

The same ruling promoted the interchange conventions, and DI-005 closes
on ADR-022. No separate promotion identifier was minted: unlike the
identity draft, the conventions never had a repository record to retire,
so DI-005 was always the promotion itself and DI-009 stays unclaimed.
ADR-022 restates no clause and mints no overlap — the two languages, the
satisfaction judgment, the base theory, acceptance and the open
companion, the assignable fragment, the registry signature, the four
invariants, the Law, the six meta-theorems and the caveats stay the
draft's, cited at the `PLAN` prefix from outside `plans/` and in the
local parenthesized form within it. What the record adds is the boundary
against ADR-010, which owns first-party command-line streams while
ADR-022 owns externally consumed documents; the executor protocol's stop
from section 3.10 of the identities register, restated for the wire
format rather than for a digest, with the register's revisit condition
unchanged; the two alignment records the draft's preamble leaves to its
adopting corpus, on documentation labels and on identity recipes; and
the standing that adoption is normative now and builds nothing until a
real externally consumed document exists. That first consumer class is
named but not chartered.

The user's integration rulings, 2026-08-16: kind-token conflicts migrate
to the registry forms; the two grandfathered identity recipes migrate
now rather than persisting as a recorded divergence; the area grammar is
adopted with a recorded amendment admitting hyphens; per-package owner
prefixes are registered as checker data in DI-003. DI-002 landed
ADR-019 and ADR-020, deleted the retired records, migrated six of the
eight conflicts (task and res adjudicated as genuine local extensions),
and re-pinned the three identities the label rename moved: the attestation
anchor-set hash, the architecture semantic hash, and the
deployment-profile identity, each with the retired pin reproduced
before the new one was taken. The Attestation LaTeX surface keeps three
unadjudicated tokens (motto, a true collision; invest; abs), recorded
in ADR-020.

DI-002b's swap carried three consequences beyond the citation repairs.
The calculus's third edition adds a derivation authority the checker has
no notion of — a profile signature, a reserved-kind set, two warrant
rules, warrant totality and inventory discipline — which is now the
largest unmet item of its implementation gate and belongs to DI-003. The
gap census's postcondition checklist was reframed as a gate checklist,
both the calculus and the registry having moved to Gates. And this
record's own label-calculus gate was renamed from the implementation
name to the adoption name: the third edition mints the implementation
text for its own gate, and although the owner-keyed registries keep two
owners minting one text from being a duplicate, one text naming two
gates is a reader hazard the record yields on.

A second kind-migration round followed the swap. The attestation
verification appendix minted three walkthrough anchors under the token
ver, which the adopted registry assigns to Version and Revision;
verification is verif. The three mints in
`papers/attestation/sections/A1_verification.tex` moved to the verif
token, and the generated specification register followed in the same commit,
six sites in two files and no other occurrence anywhere in the tree.

The identity consequence was measured rather than assumed. The attestation
anchor-set recipe was reproduced from the pre-change tree over the
realization contract's thirty-eight cited anchors and returned the
pinned value
exactly, which is what validates the reproduction; none of the three
renamed labels is in that set, because the contract cites the
walkthrough section but never these three sub-anchors. The same
recomputation after the rename returns the same value, so nothing was
re-pinned: the ceremony ran and its answer was that the anchor set did
not move. No welded witness semantic tag carries a walkthrough anchor
either, so the architecture semantic hash and the deployment-profile
identity are likewise untouched.

DI-002b closes with the refresh of the two adopting records. ADR-019
now fixes all seven adoption parameters the third edition asks for: the
owner signature, the owner partition by tree location, an empty profile
signature, an empty reserved-kind set, the designated typed-data
classes, the two citation-index designations, and the scanned-region
recognition for Markdown, Rust, and LaTeX. The profile signature and the
reserved-kind set are empty together on purpose — a reserved kind no
profile governs admits neither warrant rule, so a nonempty reserved set
under an empty profile signature would reserve kinds nobody could use —
and the record states plainly that the checker implements the authorship
warrant species alone. ADR-020 names this repository as the registry's
acceptee, recasts its register as the recorded extension set in the
edition's own terms with a located occurrence behind each of the
thirteen entries, and adjudicates the six keeps by genre: task against
the registry's exercise and code-asset readings of Task, res against
Result, trap against Pitfall, obl against Requirement, err against
Erratum, and pin against Version. Four tokens left the set because the
third edition carries them as rows of its own — pkg, q, req, and test —
with no mint moved. Postcondition and Requirement remained registry
rows, so the round-one decisions on post and req stand as taken.

Four residual items outlive the close, none of them DI-002b's to
discharge. The derivation-warrant machinery — a profile signature, a
census, standard places, inventory discipline, and the half of warrant
totality that governs reserved kinds — is unbuilt, as are the near-miss
warnings the calculus asks for, the general form of synthetic-citation
totality, and the registry's companion attestation register with its
homonym view; all of these are DI-003, and ADR-019 and ADR-020 record
them as unmet in their own terms. The identity census with admission and
stop records is DI-004. The interchange conventions stay dormant until a
consumer exists, which is DI-005. And the Attestation LaTeX surface keeps
three unadjudicated tokens under ADR-020 — motto, a true collision;
invest; and abs — which enter scope with that surface and not before.

DI-004 opened with the recipe migration the ruling settled, folded into
one recorded migration covering both identities. The retired values were
reproduced before anything moved, and reproduced independently of the
repository's own Rust: the architecture semantic recipe recomputed from
the committed manifest body under sorted-key compact JSON, and
the attestation recipe recomputed over the thirty-eight anchors harvested
straight from the realization document, both
matched the pins exactly, which is what licensed the migration to
proceed.

Each recipe then took a domain-separation prefix folding in its own
identifier, the idiom the behavioural and deployment-profile hashes
already use. The semantic hash moved to `sha256-canonical-json-v3`; the
anchor set moved to `sha256-anchor-set-v2`. Both
new values were predicted from the recipe before the code was run and
then confirmed by the generator, so the implementation was checked
against an independent computation rather than trusted. No projection or
encoding changed: the meaning identified is the same and only the
measurement moved.

The welds moved atomically with the values — the typed pin, both
generated manifests, the realization document's masthead and attached
appendix, the identities register, and the synthetic release-profile
identity, which moved because the deployment profile binds the
architecture semantic hash as a hashed input while its own recipe stayed
at version 1. The behavioural hash did not move and no version was
bumped: its recipe and body are untouched, the versioning gate keys on
it alone, and ADR-016 holds that a recipe migration does not by itself
imply a semantic version change. The grandfather clause is superseded by
the migration record in the same ADR.

DI-004 then closed with the census. It was taken from the code outward
rather than from the ADR's table — every digest, identity and hash the
tree computes was found by sweeping packages and scripts and then
reading the owning modules — and each was put to the benefit criterion:
which named consumer decides what, and in which assurance class. The
answer is six admitted identity classes and eleven recorded stops, all
in `plans/registers/identities.md`, which now opens with a census table
of one line per entry and carries admission records and stop records as
its two register bodies. Nothing new was admitted and no unclassified
digest was found, so the migration remains DI-004's only change to what
the tree computes.

The reading paid for itself twice. Eight of the eleven refusals were
practice, or enforced in code, without ever having been written down as
decided outcomes: the native conformance reports that deliberately carry
no digest and no field reserved for one, the compiler's analyzed
program whose refusal is held by a compile-time probe and an exhaustive
destructuring rather than by convention, the Phase-1 realization that
publishes an architecture binding instead of an identity, the target's
own tagged hashes and sighashes, which are the modelled protocol's
values and not identities of this corpus, the chain identifiers received
as validated typed fields, the executor's wire frames, the CI lane
records, and field-level digests in general. And the profile's raw hash
fields were undercounted: the register named the report and artifact
rows, while the schema also carries one evidence hash per substrate
dependency and the calibration's own evidence and script-bundle hashes.
All four families are recorded together, with the one binding profile
validation actually enforces — the calibration's bundle hash must equal
the released emitted-script-bundle row — separated from the presence
checks that decide nothing.

Two admission records gained precision the ADR's table does not carry.
The architecture semantic hash is the one admitted identity that crosses
a process boundary today, because the ledger's schema-13 attestation
context carries it, and each of its consumers decides by rejection: the
indexer refuses a context whose manifest hash is not the build's
expected typed hash on every construction path, profile validation
refuses a profile bound to another architecture, and the weld fails the
build on a disagreeing masthead. The document and instance identities
are locator-grade provenance, and their 128-bit truncation is admissible
only on that condition — a release-bound provenance identity could not
carry it — which the records now state rather than leave implied.

The cross-check against ADR-016 found no row the census contradicts: all
nine rows of its current-identities table hold against the code, and the
ADR was not amended. One imprecision is recorded rather than corrected —
the table calls the two publication identities UUIDs, which the
producing functions do too, though neither value sets a version or
variant field — because the register already carries that non-claim and
the ADR remains the owner of its own wording. The gap census was
repaired where the migration and the census falsified it: domain
separation and recipe identifiers are implemented rather than divergent,
the no-identity clause is implemented rather than divergent, the
implementation gate's evidence names the three items DI-004 discharged,
and the second open ruling is settled.

What outlives DI-004 belongs elsewhere. The deployment-profile hash and
the profile's raw hash fields stay pre-admission with their revisit
conditions bound to the release surface, not to this batch; the benefit
walk as a standing requirement on new proposals, rather than as the
one-time census just taken, is a change to ADR-016's admission rule that
no task yet owns; and the executor protocol's stop records the reasoning
DI-005 restated in ADR-022 when the interchange conventions were adopted
as the discipline for externally consumed documents.

DI-003 opened with W1, the participation scanner. The survey found the
judgment re-derived in six places: the Markdown scanner's own fence
loop, a second fence loop in the plan-tree link check, and the
double-backtick skip restated inline at each of the four span consumers
in `packages/labels/src/repository.rs`. All six now consume one module,
`packages/labels/src/participation.rs`, whose module documentation
carries the survey as a table — decision site, the logic it used to
carry, where it lives now — so a seventh recognizer has somewhere to be
refused. The Rust comment-and-literal segmentation and the LaTeX comment
strip moved into it as well, making it the single home of the
scanned-region recognition ADR-019 fixes for all three concrete
syntaxes.

One genuine disagreement was found and resolved rather than preserved.
The Realization harvest read its span list from the fence-aware scanner
but computed the boundaries of the generated upward-citation index by
walking the raw source, so a section heading displayed inside a fenced
block could open that index region — and, because the region stayed open
across the fence close, swallow the real body citation below it. The
anchor set derives from body citations, so the effect was a body
citation lost and the genuine index reported stale. The region walk now
reads participating lines only. The resolution is what the calculus
requires, since a displayed heading is not authored text and cannot
partition a document; it is a deliberate behavior change, recorded here
and reproduced by a test that fails with the fix reverted and passes
with it in place. No occurrence of the pattern exists in the tree today,
so no diagnostic moved.

Behavior preservation was measured, not reviewed. A capture script,
`scripts/capture-label-goldens.sh`, runs all four labels binaries over
the whole tree with the argv the CI lanes use, writing each stream and
exit status; the before and after captures are byte-identical once the
tracing wall-clock field is normalized, which includes both generated
registers reproduced whole at 17376 and 35384 bytes. The committed
registers under `plans/labels/` are unchanged. The label census is
identical across the refactor at 136 attestation, 329 realization, 129 ADR,
307 model, 1206 planning and 6 documentation mints with 211 imported
citations.

Two classes of site were surveyed and deliberately left alone, both
recorded in the scanner's module documentation. The forbidden-text audit
is participation-blind by design — a banned token is banned inside a
fence and inside a string literal too — so it keeps its whole-tree grep.
And several plan-tree hygiene checks read raw text: the scaffolding,
placeholder, confidence, deleted-path and machine-input markers, and the
backlog's task-status agreement walk. Their subjects are not label
occurrences, and rewiring them would change what they report, so they
are recorded as participation-blind rather than quietly converted.

W2 made the adoption parameters typed data. All seven now sit in
`packages/labels/src/adoption.rs` for comparison against the ADR-019
table, and the signature drives prefix resolution rather than restating
it: an imported token splits at its first hyphen, exact because a prefix
carries none, and becomes an owner only through the signature. The
package owners are registered there, settling the DI-003 ruling, their
prefixes derived by rule from the directory name and checked against the
census, so a new package is a reviewed registration, never a silent
owner.

The kind vocabulary is now the registry's: the 208 distinct kinds of the
draft's Convention tables and the 13 recorded extensions, committed
rather than parsed at check time so an edition swap shows every moved
token in review, and checked against their documents in each direction
so the tables cannot go stale unseen. The census found 2113 mints over
64 distinct kinds, every one governed except `motto` and `abs`, single
occurrences on the Attestation LaTeX surface that ADR-020 already records as
unadjudicated. That record puts the surface outside its scope, so those
two report rather than fail, and enforcement over the owners it does
govern lands with the tree already conformant.

Warrant totality is enforced ahead of its subject. The reserved set and
the profile signature are empty, so its reserved and inventory arms are
vacuous, and both are implemented and fixture-tested all the same, so
the first registered profile finds the enforcement live. Place detection
is unbuilt and reports no place, which fails closed. Two-pass staging is
structural: the adoption data load before any source is read, and
warrant totality derives from completed registries before any citation
resolves.

One defect surfaced in passing: diagnostics were emitted only when the
check failed, so no passing run could ever show a warning. Warnings now
emit on a passing check, which is what makes the two attestation tokens
visible.

W3 closed the batch with the near-miss warnings, in four families and
warnings only: an interior differing from a label in case, in spacing,
or in bracketing, and a label-shaped backtick span in scanned comment
text where the acute carries the label syntax. Each family repairs the
span before testing it and every repair changes the text, so a form the
grammar accepts can never reach a warning; displayed spans stay silent
and no existing error was demoted. The live tree raises 198, all of them
the comment family — one finding rather than 198, since Rust comments
cite labels in backticks throughout.

The companion attestation register the acceptee owes is generated at
plans/labels/attestation.md, 6176 bytes, by its own target: separate
from the upstream generator because its derivation is corpus-wide where
theirs is scoped, its evidence rows carrying a mint census that answers
to every owner. Three views of one base under the recorded ordering —
the 13 first-hand extension rows with spelling, locator, sense, and
census; the status map, 349 base rows by reference and the three the
edition daggers; and Hom(C_A), 32 pairs over 15 names. The derived base
relation is welded to the registry's published headline counts, so a
parse that drifts fails rather than re-deriving quietly. Task rows in
Hom only because this corpus extends it, which is the register's point.

Both records were then walked bullet by bullet. The calculus's
implementation gate has one unmet bullet left, inventory discipline,
waiting on the decision that registers the first profile rather than on
code; two ADR-019 claims had gone stale the other way, package prefixes
and the profile note, and both now read as built. The registry's
adoption gate lost its recorded unmet item with the register and gained
one this walk found: head validation is unimplemented, the checker
validating a kind token but never the pair a head declares. The 146
environment heads under adr/ and plans/ were read by hand and all carry
catalogued pairs, so nothing rests on the gap, and ADR-020 names it.

Three residuals outlive DI-003, none its own to discharge: the
derivation-warrant machinery waits on a first inventory profile, Π and K
empty by decision; attestation kind scope waits on the motto, invest, and
abs adjudication, entering with that surface; and the interchange
conventions stay dormant until a consumer exists, which is DI-005.

One of those three tokens is now settled. On the ruling this
corpus keeps its opening motto apart from the registry's slogan genre,
so Motto and `motto` enter X_A as its fourteenth entry, on first-hand
evidence at the title section. ADR-020 records the deviation, the
checker's kind and pair tables carry it, and the regenerated companion
register puts Hom(C_A) at 34 pairs over 16 names. Attestation stays
reported: `invest` and `abs` are open.

The acute code syntax now classifies as the calculus fixes it. The
harvester had paired acutes blindly, one comment line at a time, so
every acute opened: a lone one used as an apostrophe failed the file
and a pair swallowed the prose between into a bogus label. Pairing is
now settled over the logical comment region, an acute opens only where
label-shaped text follows, and one that opens nothing stays text, while
an opener whose region ends first still fails hard. No rule anywhere
banned the acute in Rust sources — the forbidden-text audit knows one
token and it is not this — so nothing needed repealing. Both syntaxes
are readable in comments during the migration: the acute participates,
the backtick spelling only warns, and the sweep of the 198 warned sites
follows. Goldens over the tree are byte-identical.

That sweep is done: all 198 warned spans are gone. 196 became live
imported citations, each carrying the registered prefix of the owner
that mints the label — PLAN for the Guide-10 rules, R13, PA, and
numbered ADRs elsewhere — and imported citations rose by exactly 196,
from 211 to 407. Conformance held 132, tapscript 23, architecture 12,
target-elements 11. Nothing was minted to make a citation resolve: no
crate mints a label, so every reference was owed elsewhere. Two sites
quote a label's spelling rather than cite it, in the guard-listing
weld that parses the document for it; both are double-backtick spans,
shown rather than meant. Groups that straddled a line were brought
onto one, a group being read within its own line. The generated
registers regenerate unchanged, so no anchor set moved.

W3 closed the batch with head validation: each head's name is now held
to the kind its label declares, over the decision records and the
authored planning tree. The recognizer finds exactly the 146 heads the
hand reading counted — 4 authored here, 142 in the archive it does not
judge — and all validate by an exact pair, so no head or label moved.
Reduction implements the sub- prefix and the catalogued modifiers only;
the other devices are unimplemented and untriggered, so a numbered or
lettered head fails until they are. The three waves: acute
classification, the 196-site sweep, head validation.

### Wave 7 — CT fixtures and the conservation matrix · `task:guide11:conservation`

**Priority:** P1
**Status:** DONE

The §8 fixture language, a generic confidential-transaction
materializer, and the §8.4 matrix run against a real node. The report
role is **experimental** and the type has one variant: §8 establishes the
CT substrate and the consensus facts, and nothing about a candidate, an
opening prototype, or a normalization policy. A canonical role belongs to
whichever wave earns it.

**The matrix's discriminating power is the layer, not the class.** This
target answers every CT conservation failure with one consensus code, so
§8.3's six layers are what a row can actually distinguish.

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `G11-W7-01` | P1 | DONE | **Byte-level determinism is not achievable through this materializer, and §8.2 is met one level down.** Confidential value on this target can only be produced by the node: the upstream Python framework carries no Pedersen commitment, range proof, or surjection proof. `BlindTransaction` draws every output blinding factor from `GetStrongRandBytes` and generates a fresh ephemeral nonce key, and no RPC on the path takes a seed — `rawblindrawtransaction` included. Established by reading the source and by blinding one identical raw transaction twice and comparing. So the fixture's own inputs are fully determined and reproducible, the transaction bytes are recorded per run, and the level is typed as `FixtureInputsOnly` so a later materializer that can seed its blinding reports a different value rather than quietly improving. |
| `G11-W7-02` | P1 | DONE | **Ten of eleven executed rows agree with expectations written before the target was asked.** Rows 1, 2, 4 accept; 6, 7, 8, 9, 10, 11 are consensus rejections before script; 12 accepts. The one row that reached no verdict is `G11-W7-03`. |
| `G11-W7-03` | P1 | DONE | **Several confidential inputs to a single explicit output is not constructible**, which is the exact evidence §10.2 said it required and refused to take from algebra. The node answers "Add another output to blind in order to complete the blinding": residual blinding has nowhere to go without a blinded output to absorb it. Recorded as a fixture-construction failure, which establishes no target verdict — the target was never asked — and is a finding about the candidate rather than a defect of the row. |
| `G11-W7-04` | P1 | DONE | **The §7.4 third leg lands.** The oracle predicts, and the target produces, the same commitment bytes for three observed openings. Both the asset identifier and the blinding factors are printed reversed from the order the arithmetic reads them; the convention was settled by trying all four combinations against a real observation, and only one reproduces the target's commitment. The construction-library leg stays absent and is reported as absent: the node built these transactions, no third-party library did. |
| `G11-W7-05` | P1 | DONE | **Consensus does not police a hidden confidential output.** §8.4 states the row as "closure reject *where claimed*", and the conditional is load-bearing: a hidden output makes the commitments balance, so conservation is satisfied and consensus has nothing to refuse. The row expects acceptance and names the obligation — any candidate claiming disclosure-completeness has to police it itself. Writing it as a consensus rejection would have failed the run against an expectation the target never owed. |
| `G11-W7-06` | P1 | DONE | **Block validation cannot be read for the layer.** Amount checks run in the same check queue as script checks, so a malformed range proof, a broken surjection proof, and a one-unit imbalance all reach the block layer as `mandatory-script-verify-flag-failed (unknown error)`. Classifying on that string attributes a conservation failure to an opening script that never ran, which is the exact misattribution §8.3 exists to prevent and which the first revision of the adapter's `judge` committed. The mempool names these precisely, so the mempool reason decides the layer and the block only separates unrelayable from consensus-invalid. |
| `G11-W7-07` | P2 | DONE | **Four adapter faults produced false target facts before they were caught**, all in the first run and all recorded here because each is a way a harness manufactures evidence. `getnewaddress` returns a blinded address by default, so every row that said it was explicit was confidential end to end and nothing in the result said so. The wallet's coin selection spent a row's own freshly created coin while funding the next, and `bad-txns-inputs-missingorspent` was recorded as a conservation verdict. A partially signed transaction was judged as if it were the row. And `MalformedRangeProof` serializes as `malformed_range_proof` while the adapter compared against `malformed_rangeproof`, so the branch never fired, the transaction ran undamaged, and *the target accepting a valid transaction was recorded as the target accepting a malformed range proof*. An unknown defect is now refused by name rather than falling through every branch. |

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-W7-08` | P2 | OPEN | The conservation lane runs through `run-conservation-matrix.py`, which records and does not gate. The typed report exists and is tested; what is not yet built is the Rust executor driver, the gate, the published report asset, and the Meson target that would make this lane refusable in CI the way the primitive and prototype lanes are. Until then a conservation run is evidence a reader consults, not a gate a build enforces. |
| `G11-W7-09` | OPTIONAL | OPEN | Both public-committed rows of §8.4 remain deferred against `G11-C03`'s blockers. One of the two is executed in its explicit form (`G11-W7-03`); the public-committed form of both awaits a selected candidate. |

### Waves 8+10 — Candidate dispositions and normalization · `task:guide11:dispositions`

**Priority:** P1
**Status:** PARTIAL

The typed dispositions the evidence already dictated, and the typed
disclosure reasons the one live candidate needs. The normalization
prototype, its threat matrix, and its safety report are **not started**;
the wave was wound down after the disposition half landed.

**A deferral and a rejection are not the same decision, and §11 is both.**
Recording only one of them would have stated something the review never
established, in whichever direction it was collapsed.

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `G11-W8-01` | P1 | DONE | **The direct authenticated opening is deferred as a class and refused as a shape, and the register carries both.** §11.4 rejects any candidate that loses parity, and two of the three reviewed blockers are parity facts, so the §11.3 proof outline as instantiated on the reviewed primitives is refused outright under that criterion — as is an opening checked off-script and asserted on-script, under §21's "only a host library verifies the opening". The class is deferred rather than rejected because a pattern proving the normalization or negation §11.4 asks for was not found, which is not the same as shown impossible. Cites `G11-C03`, `G11-C01`, `G11-O02`, `G11-O03`. |
| `G11-W8-02` | P1 | DONE | **The public-committed representation defers on the same three blockers.** A public committed output exists to publish an amount and opening a later relation can verify; on this target that verification has no on-script form, so an output publishing one would be publishing unauthenticated metadata, which §21 refuses by name. Conservation row 3 already carried the deferral as a typed row rather than as an absence. |
| `G11-W8-03` | P1 | DONE | **The capsule is not-applicable-while-deferred, which is a third state rather than a deferral of its own.** A capsule's contents are an opening and the fields binding it to one output; with no public committed representation there is no opening to carry and no output to bind it to, so §12.4–§12.6 have no subject. Nothing about the capsule was examined and found wanting. The normalization path supplies no subject either: its public output is explicit, and an explicit amount is already recoverable public chain data. |
| `G11-W8-04` | P1 | DONE | **Dispositions belong in conformance, not in `target-elements`.** Wave 5 put `OpeningFeasibility` in the target package correctly — whether the reviewed language *can* carry an opening is a target fact. A disposition is this project's decision taken in the light of target facts, and §6.2 puts candidate work in conformance. The blockers a disposition names are Wave 5's own typed values rather than a restatement, reached through a vocabulary spelling table because the target crate is standard-library-only and derives no serialization. |
| `G11-W8-05` | P1 | DONE | **§14.4's preferred reuse of realization's disclosure types is not available, for two independent reasons.** `realization::DisclosureReason` does express the required distinction. But conformance refuses the realization dependency by decision rather than oversight — the manifest says naming it would import a publication boundary this harness has no asset for — and two of its variants carry an `OperationId` and a `RelationId` that a conformance run does not have. Filling them would mean inventing identities, which in an evidence record is worse than a duplicated enum. The vocabulary is therefore §14.4's own, stated in conformance, with the correspondence documented. |
| `G11-W8-06` | P1 | DONE | **Every normalization disclosure is deployment policy and none is boundary arithmetic.** The relation does not need the amount in the clear: the §8.4 matrix conserved value over commitments without it. Each non-semantic disclosure states what a deployment that did not want the fact public would do instead, so that a policy choice cannot harden into an apparent necessity across waves. §9.3's rule that an explicit-only result is not disclosure-minimal is what this makes checkable. |

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-W10-01` | P1 | DONE | **The normalization prototype is built, and the disposition it was recorded under is now true.** The claim is private → explicit + private change, the constructible variant. The owner's coin and every destination are taproot, so the wallet signs a key-path spend: both inputs carried a single 64-byte witness item on the run, which is a Schnorr signature with no trailing sighash byte — the default all-outputs non-anyone-can-pay profile §10.3 requires, per the reviewed output-committing signature profile table. The profile is read out of the witness by the adapter rather than asserted, so a narrower signature is a construction failure instead of silent evidence. The unmutated claim was accepted by a real node at the declared tip, with closure and preservation both holding. |
| `G11-W10-02` | P1 | DONE | **The closure check is implemented as exact multiset equality in both directions, and all nine §10.4 rows agree with expectations written and committed before the run.** Three rows are consensus-valid transactions the report layer alone refuses — amount changed with the change compensating, owner changed, and a hidden private output — and the target accepted all three, which is the wave's substantive finding rather than a gap. The hidden-output row is what justifies the shape of the check: the owner is paid exactly right so preservation holds, the value comes out of the blinded change so no amount is observable, and the only evidence is an output the claim never named. A subset test would have passed it and a count test would have passed a swap, which is why neither is used. The layer is recorded as report-layer and never as consensus, so `G11-W7-06`'s misattribution is not repeated. A relay-policy refusal now has its own refusal layer as well: no row expects one, so a post-signing row refused for its fee would surface as a disagreement rather than pass as a signature refusal. |
| `G11-W10-03` | P1 | DONE | **The typed §14 safety report is built.** Role is `Experimental`, as `ConservationReportRole` is: it establishes what the target does with the §10.4 matrix and nothing about a candidate being selected, which is §24's question. It carries `declassification::normalization_declassifications` — three disclosures, every one `DeploymentPolicy` and none claiming semantic necessity — the observed authorization profile, the genesis and network the run was bound to, the declared tip and the revision the node binary reported about itself, and every row's expected against observed layer. The judgement lives in the crate that owns the claim: the runner records responses verbatim and `emit-normalization-report` rebuilds the expectations from source, so a run cannot supply the answer it is checked against. |

### Wave 11 — Fresh-process lifecycle · `task:guide11:lifecycle`

**Priority:** P1
**Status:** DONE

The §13 proof that public evidence survives its creator. The boundary is
an operating-system boundary rather than a reset of state inside one
process, and the reading process is held to public chain data alone.

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-W11-01` | P1 | DONE | §13 boundary is an OS boundary: A publishes and exits, each B is a fresh process and node, three distinct pids; wallet destroyed, chain kept. |
| `G11-W11-02` | P1 | DONE | The public record is a typed 12-field schema of chain data, with `deny_unknown_fields` and a field-NAME ban. Both tested. |
| `G11-W11-03` | P1 | DONE | 18/18 rows agree over two passes against `lifecycle::canonical_lifecycle_matrix`, never the run record. All nine canonical rows are now sent; the stale row joined them when `G11-W11-06` closed. |
| `G11-W11-04` | P1 | DONE | B locates by block locator, parses independently, rebuilds the script by bech32m; it cannot spend the owned object and spends its own funds. |
| `G11-W11-05` | P1 | DONE | Cache independence: fresh process and node per pass, both identical. The object needed locking against A's coin selection (`G11-W7-07`). |
| `G11-W11-06` | P2 | DONE | Diagnosed and repaired. The taproot digest commits the output-witness vector at its actual length, so an unblinded spend signs one digest and consensus checks another; paying to a single confidential address did not blind it, because the node declines to balance a lone blinded output against an explicit input and says so only if asked. The spend now carries two confidential outputs and refuses a silent non-blinding. The row builds and both passes answer `refused_output_spent`. |

The `G11-W11-06` diagnosis, because the finding outlives the row. The
target's taproot digest commits the transaction's output-witness vector,
and it hashes that vector at whatever length the vector has rather than
at one entry per output. A transaction carrying no witness deserializes
with that vector empty, and serializing a transaction that has any
witness grows it to one entry per output — so a wallet asked to sign an
unblinded transaction signs a digest those same bytes can never produce
once they are on the wire. The wallet reports the signing complete and
the target refuses an invalid Schnorr signature.

Proven rather than argued: the signature the wallet produced verifies
against the digest computed with the vector empty and fails against the
digest computed with it grown, while a genuinely blinded control spend
verifies the other way round and is accepted. The check is kept runnable
at `scripts/diagnose-taproot-output-witness-digest.py`.

The repair is ours and is in the adapter, so the row is closed here. The
upstream half is a defect worth filing against the target and is drafted
for the register another lane owns: the signing path builds its constant
transaction view before growing the output-witness vector, so it computes
a digest over a shape the wire form cannot have, and reports the result
complete. It is not a consensus question — every transaction that reaches
consensus with a taproot spend already carries the grown vector — so the
correction belongs on the signer's side.

### 5.8 Dependency decision and commitment oracle · `tab:backlog:findings-ct-oracle`

The Guide-11 §6 dependency decision and the §7 independent commitment
oracle. Filed by content for the reason §5.7 gives: the preflight arc
spends the wave numbers on a different sequence.

**The dependency decision is to add nothing.** The guide permits a
generic Elements or secp256k1-zkp dependency in
`target-elements-conformance` for generator derivation, commitment
construction, and independent vectors. It was reviewed against the
ADR-011 dependency policy and declined.

| Candidate | Licence | Disposition |
|---|---|---|
| `secp256k1` / `secp256k1-zkp` (Rust FFI bindings) | MIT | **Refused on independence.** Both wrap the same in-tree C library the node vendors for confidential values. The oracle exists to be a second opinion; binding it to the implementation it checks would make agreement a tautology rather than evidence (§6.3). |
| A pure-Rust curve crate (`k256`, `crypto-bigint`) | Apache-2.0/MIT | **Not taken.** It would preserve independence, but it buys nothing the workspace lacks: the whole computation is one curve map, two point additions, and two scalar multiplications over published constants. Against that it adds a cryptographic dependency, a transitive graph, an advisory surface, and a lock-file delta. |
| First-party arithmetic over the workspace bignum | — | **Taken.** `num-bigint` and `sha2` are existing workspace dependencies with existing first-party consumers, so nothing entered the lock file, and the exact-arithmetic stance of (`dec:math:exact-certified`) is served directly. |

Because no library is shared with the materializer, the §6.3
independence claim is unqualified and its fallback — record the shared
dependency and narrow the claim — does not arise. `target-elements`
remains standard-library-only (§6.1); the oracle lives entirely in the
conformance crate.

**The oracle's expected values rest on three derivations, not one.**
Every pinned byte string was computed by the crate's oracle and, before
being pinned, by a separate implementation written from the same source
citations in a different language. The two agree byte for byte. The
third and strongest leg is external: the vendored library carries its
own published fixed vectors, and all of them reproduce.

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `G11-O01` | P1 | DONE | Upstream low-level confidential vectors **do** exist, contradicting the §5 review's premise. That review read the Python functional framework, which indeed carries no Pedersen or generator helper, and concluded the oracle could not be cross-checked against published vectors. The vendored C library's own unit tests carry three sets: thirty-four curve-map outputs stated to match an independent SAGE program, thirty-two generator derivations for the asset identifiers that are thirty-one zero bytes followed by `i`, and one point under both prefix conventions. All sixty-eight reproduce exactly. The oracle's two pinned assets are drawn from the published table, so its generators rest on upstream bytes. |
| `G11-O02` | P1 | DONE | The upstream fixture named `two_g` does not encode `2G`. It carries prefix `0x0b` and is a parse-and-serialize round trip; parsing takes the point whose y is a quadratic residue and negates it when the prefix is odd, so those bytes name the *negation* of the doubled base point. `2G`'s own y is a square, so `2G` encodes as `0x0a` and `0x08`. Pinned explicitly, because reading the fixture's name as a claim would put a sign error into every later commitment. |
| `G11-O03` | P2 | DONE | A one-bit mutation of a commitment behaves in two entirely different ways depending on where it lands. Flipping a bit of the x coordinate leaves the curve and is refused; flipping the parity bit yields a well-formed encoding of the negated point. Nothing in the encoding binds the prefix to the commitment that was intended, which is the encoding-level face of the reviewed blocker that a witness-supplied parity byte is bound to nothing. |

The three-way comparison of §7.4 is landed as an interface, not as a
result. The oracle's expectation is the subject; the construction-library
and target-introspection legs are optional and arrive from later native
runs. A leg that was never supplied is reported as absent rather than as
agreement, so a comparison that did not happen cannot read as one that
passed.

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-O04` | OPTIONAL | DONE | The oracle derives only the unblinded asset generator. The reviewed recipe also has a blinded form, which prepends a scalar multiple of the base point and is used to check that a reissuance input carries its blinded token. No present consumer needs it, and a wave that reaches reissuance should add it with its own published-vector check rather than by analogy. **Closed by Wave 7, and not by reissuance.** The §8.4 conservation matrix needed it for an ordinary reason the row did not anticipate: every balanced confidential output has a *blinded* asset, so the unblinded generator predicts nothing for exactly the rows the §7.4 comparison matters most for. The check is not by analogy and not a published vector either — it is three commitments observed on a real node, which the oracle reproduces exactly (`G11-W7-04`). |

---

## Guide-12 records

Moved from [the backlog](../backlog.md) on 2026-08-21, at the batch's
close: the three Guide-12 gate records (§2.6–§2.8) and the six long
Phase-4 wave narratives of the task table, whose live rows compress to
their evidence. The narratives are the waves' own reports as they
stood; `T4-011`'s statement that the gate was not yet run is answered
by §2.8 below, which ran it.

### 2.6 Guide-12 preflight and Phase-3 exit gate · `gate:backlog:guide12-preflight`

The Guide-12 preflight campaign closed the seventh review's register in
three waves of one Opus lane each: Wave 1a repaired the identity and
evidence-boundary rows, Wave 1b unified both protocol sides on revision
four and brought the Python runners to the Rust supervisor's
guarantees, and Wave 2 corrected the tapscript projection, the abstract
literal walk, and the bounded decode, then proved the emit subprocess
contract. Every merge was audited against the worker's own gated tree
and each landed byte-identical, so the lane evidence transferred
verbatim; the register itself, complete at sixteen rows, moved to
[backlog history](backlog-history.md) under the CI-004 pattern
when the byte budget fell to 523 bytes mid-wave, and the §5.9 stub
carries the satisfied gate verdict (`gate:guide12-exec:preflight`).

With the register closed, the Phase-3 exit gate was evaluated on the
current tree per Guide 12 Wave 3. Full gate, wall-timed: the CI
registry passed 13 of 13 lanes in 16 m 46 s, meson compiled warm in
2 s, and the meson suite passed 36 of 36 lanes in 5 m 25 s. Native
evidence state, recorded exactly: the development target-native
evidence cited by the readiness statement was produced under protocol
revisions up to three; the executor and the typed protocol moved to
revision four together in Wave 1b, so the next native run reproduces
that evidence under the unified revision, and the runtime half of
`G12-R04` stays blocked on a live node and is recorded as blocked. The
package contracts state resources and protocol at a level the wave did
not move, verified by inspection rather than assumed. The Phase-3 card
records the exit; Phase 4 is chartered by Guide 12 and its core waves
are unblocked by the closed register.


### 2.7 Guide-12 build boundary · `gate:backlog:guide12-build`

The six build waves after the Phase-3 exit delivered the complete
compact-ASH pipeline up to the live-node boundary, one Opus lane per
wave, every merge tree-identical to the worker's gated tree: the
validated compiler operation plan (Wave 4), the transaction-form and
substrate review (Wave 5; the user's 2026-08-20 ruling ratified a
modified form: the first-party substrate stands, and `elements` with
`secp256k1-zkp` are adopted as reference-implementation oracles for
testing — their agreement is conformance-to-target evidence, the
first-party oracles keep the independence claim, and the raw sys FFI
crate is never a direct dependency), the static
assessment and eight operation-proven proof patterns (Wave 6), the
candidate relocatable bundle (Wave 7), the linker foundation (Wave 8),
and the candidate transaction ABI (Wave 9) and canonical fixtures
(Wave 10). The linker wave proved the Wave-7 bundle unlinkable — the
recognition fragments pushed the constructor program as a literal, a
preimage-hard self-commitment — and a dedicated fix wave resolved it
by identity introspection, re-deriving every census the change
touched; the refusal vocabulary stays covered by constructed
fixtures.

Full gate on the boundary tree, wall-timed: the CI registry passed 16
of 16 lanes in 21 m 57 s, meson compiled warm in 2 s, and the meson
suite passed 42 of 42 lanes in 7 m 13 s. Weight: the closed §13.1
integration queue moved to history mid-batch, combined markdown
745826 of the 786432 cap. Waves 11 through 14 are blocked on a live
target node and are recorded as blocked in §11: execution, negative
relation coverage against the target, the resource observations, and
the Phase-4 gate that consumes them. Standing obligations carried,
not discharged: the pinned output key is unverified against the tree,
internal-key unspendability is unverified, the clear lifecycle exit is
outstanding, and no target has executed anything.

The reference-oracle cross-check wave is the first consumer of the
adopted crates. Fifteen tests in
`packages/target-elements-conformance` compare a first-party
computation against the reference implementation of the same object,
each stating the claim class in one line: reference-implementation
conformance, not independent evidence. Byte-equal on every comparison
made — all nine sponsorless fixture transactions round-trip through
the reference decoder and re-encode identically, their structure and
version agree under both decoders, the reference txid and wtxid are
the double digests of the first-party witness-stripped and full
encodings, the two unrelated weight formulas and both virtual-size
rules agree, all nine reference txids stay distinct, every committed
leaf hash equals the reference tapleaf hash under the reviewed leaf
version, every leaf path folds to the first-party merkle root under
reference branch hashing, every control block parses and
re-serializes byte-identically, the reference verifier accepts the
first-party taproot commitment for every leaf, and the first-party
confidential oracle matches the reference library on two asset
generators and six Pedersen commitments. The merkle root and the
reference output key are pinned as vectors in `src/reference.rs`.

Two defects the cross-check found, both recorded by tests that fail
if the defect is repaired. First, the fixture bundle's pinned witness
program was not a curve point, so no control block could ever satisfy
the reference verifier against it and an output created at that
program was unspendable by construction. Second, the fixture declared
even output-key parity while the tweak of the fixture internal key by
the fixture merkle root produces odd parity, so a control block built
from the declared bit stated the wrong y coordinate. Both are the
same root cause: the pin was declared rather than derived. Both
blocked the Wave-11 funding ceremony that would discharge the
pinned-output-key obligation.

Both are now repaired, in the only place the repair belongs — the
substrate. The fixture pin carries the output key its own committed
tree derives together with that key's parity, stated in
`packages/vectors` as literals whose provenance is the minted
reference vector, because the vectors package contract §16.2 forbids
the edge that would let the fixture import the curve arithmetic and
the conformance package already dev-depends on vectors. The
cross-check recomputes the key from the fixture internal key and the
merkle root and asserts equality with the constant, so a literal
cannot drift back into a declaration; the two tests that recorded the
defects now assert the repaired truth, and the reference verifier
accepts the fixture's own unmodified control blocks for all twelve
leaves rather than blocks rebuilt with a corrected parity. The
merkle root did not move, since the committed tree does not depend on
the pin, so every fixture transaction keeps its length, weight, and
virtual size and changes only the thirty-two program bytes it pays to.
Pinning the derived key does not discharge the pinned-output-key
obligation, which still needs an output a real node created and a
spend it accepted; what it buys is that the ceremony can run at all.

The section 16.2 executor-ownership decision Wave 10 left open is now
settled, and settled the way the guide states it: the boundary was
WIDENED, not extracted, and no shared executor package was created.
The test the guide sets is whether a clean target-generic boundary can
be exposed without making the conformance package own compact-ASH
meaning, and it can. What an operation asks a node for is target
generic all the way down — issue a disposable asset, pay outputs to a
witness program, hand the target a complete transaction — and what
comes back is the observed-layer vocabulary that package already owns
for conservation. Nothing about relations, coverage, classes, or
acceptance crosses in either direction.

The widening is one enum variant and one trait. `NativeWorkload` gains
an operations arm carrying a caller-supplied plan rather than a fixed
list of cases, because an operation is the first workload whose steps
are not all statable before the run begins: a transaction cannot be
built until the outputs it spends exist, those outputs are created by
an earlier step of the same run, and each run gets its own disposable
node — so a fixed list could not express the dependency and two runs
would fund one chain and submit to another. Protocol revision 4 gains
the fourth workload section 16.3 names, under two capabilities on the
established pattern that a new record shape is a capability rather than
a revision. Fifteen tests hold the boundary against a written exchange
rather than a spawned process.

The reviewed adapter gains the named seam and nothing more: it
recognizes an operation step, checks its shape, and refuses it as an
infrastructure failure, which is what a step that did not happen is.
It advertises neither capability, so the harness refuses before
sending. Implementing the ceremony and the submissions is Wave 11's and
needs a live node.

The vectors package states what it is waiting for without taking the
edge section 16.2 permits: `RequiredTargetWork` names the whole funding
ceremony census and one submission per materialized vector, thirteen
entries derived from the plan rather than written out. The vocabulary
is authored once on each side — the ceremony steps in `transaction`,
the vector identities in `vectors`, the wire records in
`target-elements-conformance` — so no name has a second source. Wave 11
adds the dependency edge, which also closes a cycle through the
conformance package's reference cross-check lane, and writes the one
mapping from these obligations onto the executor's step records.


### 2.8 Guide-12 Phase-4 exit gate · `gate:backlog:guide12-exit`

The five execution waves and their fix lanes closed Phase 4 against a
live target. Wave 11 took the section 16.2 edge for real and was
unblocked by two diagnoses — the witness-section guard behind a
misleading balance diagnostic, and the money-bound row ruled
kept-and-reclassified — after which eight sponsorless transactions
were accepted and mined at the pinned genesis; Wave 12 added the
test-scoped sponsor authorization capability and the four sponsored
acceptances, closing real target execution with twelve accepted
transactions, twelve matched section 17.4 projections, and the
fee-role digest defect found and computed; Wave 13 with its four fix
and design lanes built the negative machinery, corrected three
mis-specified section 18 boundary claims against live evidence, fixed
the adapter's layer misattribution, derived the class-to-requirement
link from the section 19.2 declaration, and funded every mutation arm
independently; Wave 14 measured the candidate — all thirty-six
section 20.2 assignments statically, twelve predicted weights equal
to observation live, and the finding that linking, not any target
bound, is the candidate ceiling, with seven of thirty-six assignments
linkable under the sixteen-leaf oracle budget. Wave 15 audited the
section 29 exit checklist item by item: 105 items, 94 passing, five
qualified, five deferred to this gate, and one honest structural
failure — negative coverage stands at one of seventy-two, and the
remedy is three guide gaps carried as the handoff's feature-request
material (section 18 naming no relation per row, section 19.2
stating no first-party discharge condition, the section 16.5
ABI-validation entry point absent). Coverage ends at 100 of 211,
every outstanding row naming its reason; three byte-identical runs
per wave; every merge tree-identical to its worker's gated tree. The
completion report is filed at plans/history/guide-12-completion-report.md.

This gate ran the five deferred repository lanes locally as the
verdict of record, wall-timed: scripts/ci.sh — the meson suite in its
own target/ci-meson directory, configured cold — passed 47 of 47
lanes (lane-duration sum 239 minutes across parallel jobs on the idle
build slice; the whole gate including setup took about 38 minutes
wall), and scripts/check-document-reproducibility.sh passed both of
its checks in 2 m 11 s, the reused and fresh builds producing
byte-identical documents. The advisory audit lane ran inside the
suite. The server suite had already answered 47 of 47 on every merged
boundary of this batch, most recently in 323 to 330 seconds per run.

Phase 4 exits. The batch fast-forwards main. Guide 13 inherits the
nine residuals the completion report enumerates, of which the pinned
taproot output key and the linking ceiling are the two that constrain
what a wider candidate can attempt; the Elements gripe material from
the issuance diagnosis remains recorded and unfiled pending a
ruling.

The runtime half of `G12-R04` is discharged, 2026-08-21, by re-reading
these transcripts, not by any new run. The fifty-four first-party
record files — eight adapter response streams and forty-six report
and comparison documents, under
`/workspace/loops/attestation/` in the eight wave directories from
`wave11` to `w14`, fetched 2026-08-21 from the ephemeral shared
instance and pinned by the aggregate sha256
taken over their sorted per-file digests — carry 481 record objects
and 5769 string leaves. Every diagnostic-bearing field value is a
mapped script-error message, a mempool reject reason arriving on the
JSON-RPC result path, or fixed typed text; none carries a filesystem
path, a client stderr frame, or an exception message. The counter-check is what makes that evidence
rather than absence: the adapters' own stderr, which production nulls,
holds eighty diverted client messages — a TX decode failure, a
connection refusal naming a loopback RPC port, and twenty-one block
refusals reading
`mandatory-script-verify-flag-failed (unknown error)`
and a balance-check tail — none of which appears in any record. Three
failures were traced through the source: the live `rawissueasset`
refusal at client exit status 22, whose record states only the method,
the status, and the omission; the mutation refusals, whose detail is
the mempool's own reject reason; and the conservation refusals, where
the child stderr was read, classified, and still withheld while the
mempool's `bad-txns-in-ne-out` stood as the record. Qualified: these
runs wrote operation-lane records only, so the conservation,
normalization, and lifecycle shapes carry the same typed note by
construction rather than by live witness; no framework
taproot-construction exception fired, so that branch rests on the code
half and its unit witness alone; and no unmapped script error
occurred, so the no-class-rather-than-text fallback is likewise
unexercised live.


### Guide-12 execution wave narratives

| ID | Status | Task |
|---|---|---|
| `T4-008` | DONE | Real target execution (Waves 11 and 12); the full narratives live in the merge commits for those waves and compress here to the evidence. Built: the section 16.2 edge taken end to end — `CompactAshOperationPlanner` maps the thirteen obligations onto operation steps, materialization takes outpoints from funding answers, the adapter funds and submits for real, and `CoverageObservation` gained its observed arm whose discharge requires acceptance and a matched projection both. `bundle::ceremony_bundle` links a second bundle at the asset a ceremony issues, because an Elements asset identifier derives from the issuing outpoint and the canonical fixture bundle is therefore unfundable on any real chain; `ExecutorCapability` gained the `FreshProcessLifecycle` variant the adapter had advertised since Guide 11, and Wave 12 added `ExecutorCapability::TestSponsorAuthorization` — one capability whose sponsor-funding and sponsor-signing halves are inseparable, the adapter authorizing with a published constant key under an RFC 6979 nonce (ADR-015 rule test-material), the protocol revision unmoved because the added records are capability-gated per the Guide-10 schema-migration rule and the added response members are defaulted, with a parse-compatibility test. The run of record, elementsregtest: the ceremony issues its asset, records the money-bound row as a divergence, funds and submits all twelve remaining vectors — eight sponsorless and four sponsored — every one accepted and mined, and every one matched on all thirteen section 17.4 terms, the observed side read from accepted bytes and `gettxout` rather than echoed. Coverage discharged by the run's own outcomes: 99 of 211 (every positive target-execution row active in either case). Three runs, byte-identical transcripts, both waves. Defects found on the way and fixed: the fee-role digest was an invention — a fee output carries no witness program, so its digest is the SHA-256 of the empty script and the linked constant was wrong; computing it moved the pin at even parity and the conformance cross-check caught the drift as designed. Also fixed: step-kind spelling divergence, sponsor change paid to the wrong program form, authorization written as hex where the boundary uses octets. Structural: the two-sponsor row is unbuilt in this candidate (own census column, the target never asked); a sponsored row is executable without being a byte fixture, so `is_materializable` split into its two questions |
| `T4-012` | DONE | Issuance refused `bad-txns-in-ne-out` with no first-party code involved, and the cause was not an amount: Elements checks an issuance only when the transaction carries a witness section — the guard ahead of the issuing input's `VerifyIssuanceAmount` call in `src/confidential_validation.cpp` returns false and `Consensus::CheckTxInputs` reports that as a balance failure — so the issued asset weighs an empty input side against the whole issuance and the explicit amounts are never consulted. An empty witness section cannot be encoded (`Superfluous witness record`, the earlier hypothesis's misread disproof), and explicit issuances and outputs may carry no proofs, so the only fillable field is a spending input's witness stack, which a bare anyone-can-spend coin lacks. Seven-arm live bisect with amounts held fixed: forcing a witness section on the refused bytes flips the answer to `bad-witness-nonstandard`; a witness-script-hash coin spent with its committed program on the stack is accepted and mined; a blinded funding coin is a real second effect but not this one. Fixed first-party: the free-coin split pays the working slice to the witness-carrying form of the program and the issuing input names it on the stack, no key or signature (ADR-015 rule test-material). The ceremony then issues live and stops at `T4-013`. Gripe material for the Elements register: a balance diagnostic for a non-balance condition, and `rawissueasset` returns a transaction that can never be accepted from the coin it was handed, acknowledged only in a source comment |
| `T4-013` | DONE | Kept and reclassified, per the ruling. The `maximum` fixture is unchanged at the protocol's own 51-bit bound and no target constant entered any semantic fixture; the plan and target layers learned to carry a row this target's money bound forbids. `target-elements::transaction_form` records the reviewed stated-amount bound, sourced to `MAX_MONEY` and the explicit-output loop of `CheckTransaction`, with the guard test recomputing the bound from its two factors; `vectors::divergence` derives each row's standing by comparing every amount the row makes the target state against that bound, proven by a test that narrows the bound and watches an ordinary row become divergent; the planner schedules a divergent row's one forbidden input and nothing else, records an `ObservedDivergence` carrying stated amount, bound, excess and the target's verbatim layer, and refuses the whole plan if the forbidden amount is ever accepted, because that would falsify the reviewed bound. `RequiredTargetWork::TargetAmountDivergence` keeps the thirteen work items summing: 9 materialized as 8 submittable plus 1 divergent, and the divergent row reaches no coverage row because nothing is submitted for it. Confirmed live: refused with excess while the other eight rows funded and submitted in the same run. Honest qualification: the observed refusal is the adapter's own reserve arithmetic and the transcript says so in a `target_verdict` field — that the target itself answers `bad-txns-vout-toolarge` rests on the reviewed source and its guard test, not on an observation |
| `T4-009` | TODO | Negative relation coverage (Wave 13). Open: none of the 72 negative requirements is discharged, and the wave says so rather than rounding up. What landed is the machinery and one finding. The 72 are classified by where a refusal could be observed, derived from each requirement's own evidence role rather than marked by hand: 48 target-executable, 18 first-party (10 compiler-analysis and 8 emitted-structure), 6 unreachable because their evidence is an external report nobody has written. The three columns are recomputed in `PlanCensus` and checked against the negative total, and a role naming no column is refused rather than filed under the nearest one. `vectors::mutation` builds a negative vector the only way a safe constructor allows — by changing one thing about a transaction the target accepted — and opens with the gate rather than the mutations: a vector that does not decode and re-encode to its own bytes is refused, because surgery on bytes that normalize would add a second difference and the refusal would be attributable to neither. All nine materialized vectors pass it. Eight arms each name the section-18 class they stage and take the expected boundary from the matrix by lookup, so a class that changes its mind takes the module with it; tests assert per arm that everything it does not claim to touch compares equal, and that value balance moves only where the arm says it does. That last property is load-bearing: Elements checks per-asset conservation before it runs a script, so an arm that unbalances the closed asset cannot reach a script-path expectation whatever the covenant would have said. One defect was found by the first live run and fixed. Mutations were staged from vectors already submitted, so the target refused them for spending coins it had seen spent — seven read `missing-inputs`, one read `txn-already-known`, and the witness-reorder arm had not even changed the txid. Eight results, none attributable to the mutation it was about. The order is now inverted: one vector is held back, its mutations are offered while its coins are unspent, and the un-mutated subject follows as the control, so a refused mutation and an accepted control differ by exactly the mutation. The ordering is itself a test, and attributability is a per-row order-dependent reading rather than one flag, because a mutation the target accepts spends the subject and invalidates only what comes after it. The run of record, on the shared instance against elementsregtest,, subject the four-input row: four mutations refused at exactly the boundary their class names, each naming the operation that failed — two-ash-outputs and ordinary-wallet-u-output and shorten-successor-and-grow-another-output on OP_EQUALVERIFY, noncanonical-ash-ordering on OP_VERIFY. The finding is the fifth. **`wrong-sequence` was ACCEPTED**: the class expects a script-path refusal and the target took the transaction, so either the covenant does not constrain the sequence field or the class's boundary claim is wrong. Wave 13b settled which, and it was the class. No emitted program reads a sequence — `InspectInputSequence` sits in the opcode vocabulary and the capability map and at no emission site in tapscript's pattern module — and an ASH input's witness is the leaf program and its control block, so no signature covers the field either. The ABI has typed `SequenceConstraint` as a convention, in those words, since it was derived, and section 18.11 asks only that a wrong-sequence case exist, naming no boundary for it; the boundary came from the matrix, beside a transaction-version row the target really does answer, and the resemblance did not hold. The row moves to `AbiConstructionRejection`, beside the two other rows whose mutation only first-party code catches, and what actually pins the field — `construct` writing the ABI's sequence on every input, against a request carrying no field to argue with it — is now tested rather than assumed. The run also withholds any arm whose boundary precedes the target: submitting one buys an acceptance, and an acceptance spends the subject and abandons every arm behind it. **The withheld arm exposed the next one: `wrong-transaction-version` was ACCEPTED.** Same defect: `InspectVersion` has no emission site either, no signature covers the sponsorless form, and the ABI itself records that consensus admits that form at either version — so `ConsensusRejectionBeforeScript` cannot be right. Relay policy is ruled out too, because the vectors pay a thousand satoshis and the chain runs at zero min-relay fee, so no floor refuses the flip from the topology-restricted version to the standard one. It is reported and not reclassified: a second class respec deserves its own evidence, and its correction is the next bite. Its acceptance spent the subject in turn, so the last two arms are again recorded as not submitted with the reason, and re-testing them needs either per-mutation funding or a classify-without-broadcast capability. Three runs each wave, byte-identical transcripts. Nothing is discharged because the link from a section-18 class to a relation-indexed coverage requirement is not established, and inventing it to move 4 rows would be the discharge-by-intent the plan exists to prevent. Wave 13c respecified the row Wave 13b reported and left for its own bite: `wrong-transaction-version` moves to `AbiConstructionRejection` too. `InspectVersion` has no emission site either, the ABI's own type doc records the topology-restricted version as a policy choice rather than a consensus requirement — consensus admits the sponsorless form at either version — and the relay floor stays moot for the reason it always was. The generalized pre-target withholding needed no change to cover it. Review also found a second defect: `MutantOutcome` dropped an accepted mutation's txid, though the adapter reports one at the same submission path a positive row uses; the field now mirrors `SubmissionOutcome`'s, and the report writer, which had silently dropped it too, now emits it beside `detail`. **With both boundaries withheld, the control was finally observed ACCEPTED**: ordinal 2, projection matched. Nothing spent the subject early, so both remaining arms reached the target for the first time — `witness-item-reorder` and `successor-one-below-the-sum` — and matched their `ScriptPathRejection` boundary, joining the four already seen. All eight arms are now answered: two withheld, six matched, none accepted. The txid field is exercised by test and by structure; no mutation was accepted in this run, so it reads null throughout the live transcript too. Three runs, byte-identical transcripts. Wave 13d establishes the link and reports what it is worth, which is one row. Section 19.2 requires a negative case to name its intended violated relation before it runs, and nothing did; that declaration was the missing half. An arm now declares the relation and the semantic mutation class its change falls in, and the requirement is resolved against the published plan where it must hit exactly one row — a table from arms to requirement identities would have been a third source of truth agreeing with neither side. Section 18's tables are lists of names, and no row there names a relation, a mutation class, or a boundary, so only two arms are determined at all, each by its own class name read in the relation vocabulary: `two-ash-outputs` to the ASH-output cardinality relation above its declared maximum of one, and `successor-one-below-the-sum` to the closed asset's conservation relation at an amount mismatch. Both resolve to exactly one published requirement per case. The other six carry typed reasons rather than guesses: `noncanonical-ash-ordering` and `witness-item-reorder` have no member in the negative mutation vocabulary at all, a gap between two authorities rather than a defect in either; `ordinary-wallet-u-output` and `shorten-successor-and-grow-another-output` each fit two published classes equally well; `wrong-sequence` and `wrong-transaction-version` expect a refusal before any target sees the bytes, and no requirement is indexed at the constructor's boundary, so reaching them needs the ABI-validation entry point section 16.5 names as its own report role and which does not exist. Discharge further requires the intended carrier to have executed, and the covenant script is that carrier, so only a script-path refusal counts: a transaction the target threw out before running any script established that it was invalid and not which relation refused it. **The live run found a third boundary defect and an adapter defect underneath it.** `successor-one-below-the-sum` was reported as a script-path rejection carrying the detail `bad-txns-in-ne-out`, a conservation failure attributed to an opening script that never ran. The submission path had asked a block to classify a refusal the mempool already described precisely, and block validation runs the amount checks in the same queue as the script checks, so an imbalance arrives there wearing a mandatory-script error. The conservation judgement documents that exact trap and records an earlier revision of itself committing it; the submission path reached it by the other door. The mempool's own non-script reason now settles the layer, the block's text is read only when the mempool named nothing to contradict, and a regression test drives the classifier directly. With that fixed the arm reads consensus rejection before script, which its own module already predicted: the arm does not preserve value balance, and Elements checks per-asset conservation before it runs a script. So the class's script-path claim cannot be right either — the third row of the family after `wrong-sequence` and `wrong-transaction-version`, reported here and left its own bite as those were. Six arms refused at the script path and exactly one of them names a relation, so exactly one negative row moves: ASH-output cardinality above maximum, sponsorless case; its sponsored twin waits on a mutated sponsored subject. **Coverage is 100 of 211, being 99 positive and 1 negative**, with 71 negative rows outstanding and each naming why in the code rather than in prose. The 18 first-party rows got the pass Wave 13's grep was not. None is discharged, and the guide is the reason before the evidence is: section 19.1 says positive coverage of a compiler-static or backend-structural relation uses typed structural evidence instead of inventing target execution, and 19.2 states no such rule for the negative half — its conditions are a complete mutated target transaction, an executed carrier and an observed target rejection, none of which a compiler-static relation can have. A boundary being first-party is not the same claim as a first-party test discharging the row. The evidence is thin independently of that: the constructibility witness and permissionless private dependency classes have a live typed refusal that no compiler-level test drives, every test asserting either driving the realization twin at another boundary; the required lifecycle exit is refused only at another layer on the compiler-static side and not at all on the emitted side, where the layout stage is a declared no-op; an unsupported representation is made unrepresentable rather than refused; an unauthenticated representation and an unexpected protocol secret have requirement types and predicates but no error at all. Three Wave-13d runs, byte-identical transcripts. Per-mutation funding is not built and is the next bite: until it lands the eight arms still share one subject's coins and no sponsored subject can be mutated. Wave 13e lands per-mutation funding: the subject earns one extra replica per submittable arm, funded and materialized independently, retiring the abandon-on-earlier-acceptance path so every arm answers from its own coins. It also respecifies successor-one-below-the-sum a third time, from ScriptPathRejection to ConsensusRejectionBeforeScript, alongside wrong-sequence and wrong-transaction-version: the arm's own module already records it does not preserve value balance, and Elements checks conservation before any script runs, so the arm answers its class without discharging the conservation row, which stays outstanding. Three runs, byte-identical: all six submittable arms answer independently, five still match ScriptPathRejection and successor-one-below-the-sum now matches ConsensusRejectionBeforeScript, two pre-target arms withheld as before, control accepted. Coverage is unchanged at 100 of 211 |
| `T4-010` | DONE | Candidate resource study (Wave 14). `vectors::resource_study` enumerates the section 20.2 product of six ASH and six sponsor research bounds and measures all thirty-six, recomputing from programs it emitted rather than quoting Wave 8. The unrolling became one authored source, so study and pipeline compare sets one loop produced, and a test ties the basis to the pipeline's: emitting at the resolved symbols reproduces the linked bundle's total script bytes, leaf count, and both role byte models exactly. All thirty-six emit; the rendered matrix is the study's table, recomputed on every run rather than transcribed here. Operation cost and validation budget are zero throughout, both reviewed facts rather than holes: no reviewed primitive charges operation cost, and a zero budget says this candidate emits no signature or curve primitive. **Linking, not weight or script size, is what bounds a candidate today.** The exact taptree oracle costs three-to-the-n set operations and refuses above sixteen leaves; leaves are the shape count plus the distinct batch sizes, so seven of thirty-six link and the linkable corner is an awkward shape rather than a prefix — which is what section 20.2 means by refusing monotonicity. Observed, not read off the constant: fourteen leaves link and the tree is counted, eighteen answer `TreeOracleBudgetExceeded`. Section 20.5 previously compared nothing: the ABI settled a weight the report never emitted, and the adapter answered every operation step with an all-null resource record. Weight is the one dimension where the comparison can be made and it is now made live. The adapter reports it from the node's own `decoderawtransaction` rather than computing it, writes it only on a target verdict since a non-verdict response may carry no observation, and a disagreement refuses the whole plan because both sides weighed the same bytes; unit tests stage agreement, disagreement by one unit, and an executor observing nothing, which stays an unmade comparison rather than a passing one. The run of record, same instance and pinned genesis, and submitted twelve: **all twelve weighed, all twelve matched**, at 1293 for two ASH inputs, 1736 for three, 2177 for four, 1861 and 1862 for the sponsored pair. Weight is measurably not affine in batch size, stepping 443 then 441, while the byte models are — so a weight predicted from a fitted model would have been wrong. The largest measured transaction uses 0.05 percent of the 4000000 consensus weight limit and 0.54 percent of the 400000 policy limit. Eleven of section 20.3's eighteen measures are answered; the other seven say why rather than reporting zero. The peaks and the maximum element are not observable through this adapter, because a validating node exposes no interpreter stack, no reviewed primitive touches the alternate stack, and the only static peak walk is private to the prototype lane; arithmetic and comparison counts have no charged dimension to carry them; control bytes and initial witness items are structural and simply not surfaced by this wave. No bound is selected and none calibrated, per section 20.6. Three runs, byte-identical transcripts |
| `T4-011` | TODO | Phase-4 gate and handoff (Wave 15). The exit audit is done; the gate is not, since the authoritative ci.sh, canonical meson, and document-reproducibility lanes stay the orchestrator's. Section 29 holds 105 items across eight sections, not the roughly eighty across seven an earlier reading assumed; each is dispositioned 94 PASS, 5 QUALIFIED, 1 FAIL, 5 DEFERRED-TO-GATE. The FAIL is that every negative relation reaches its owning evidence layer, which 100 of 211 cannot support, and the remedy named is the three guide gaps rather than more first-party tests. Fixed in lane, four stale facts: the vectors contract and package index claiming no target had executed anything, the Phase-4 card still opening at Wave 4, and the executor docstring saying revision 3 where its constant says 4. Record and filled section-30 report: [the Guide-12 completion record](guide-12-completion-report.md), archive-class, so the core tree gains 301 bytes |

---

## Guide-13 records

Moved from [the backlog](../backlog.md) on 2026-08-25, at the batch's close: the Guide-13 batch gate record (§2.9) and the nineteen long Phase-5 wave narratives of the task table, `T5-001` through `T5-019`, whose live rows compress to their evidence. The narratives are the waves' own reports as they stood, moved with their bytes and their labels unchanged.

### 2.9 Guide-13 batch gate · `gate:backlog:guide13-batch`

The Guide-13 batch closed 2026-08-24 with fourteen waves merged — T5-001 through T5-019 in §11 — and the full repository suite green on the shared instance at the batch tip: 48 of 48 meson lanes passed, zero failed, 13 m 12 s wall through the run-report wrapper, after one honest bounce whose cause was the batch's own archiving — the generated label register went stale when the executed guide, the eighth review, and the imported Guide-14 concept landed their mints, was regenerated by the meson target on the server, and the two census bumps were reviewed as a diff and committed before the rerun. Per the no-host-compute ruling every verification of the batch ran on the server: worker lanes for each wave, the native live-transfer lane against the server-built target binary with the run of record byte-identical across three runs at three tips, check-plans on every plans-touching commit, and this gate. The Phase-5 result is the honest stopped one the phase card records: the candidate pipeline is complete, the first-party half of the safety matrix is discharged twenty-five of twenty-five, and the entire remaining distance to the Phase-5 exit is the two typed blockers — the owner sighash and confidential predecessor funding — both owned by the chartered intermediate confidential-funding guide, which must close before Guide 14 is drafted. The batch also delivered: the four adopted drafts retired into ADR-019 through ADR-022 with the amendment-listing cleanup recorded open as DI-F07, the root-ADR weight class, the drafts directory restored as a standing template, and the intermediate guide's four charter decisions recorded as accepted in its concept.

### Guide-13 Phase-5 wave narratives

| Task | Status | Evidence |
|---|---|---|
| `T5-001` | DONE | Guide-13 Wave 0 preflight reproduction: all eighteen review rows dispositioned. Sixteen CONFIRMED with committed reproductions in six packages — a confirmed defect is an ignored test stating the property the repair must establish, so the repair un-ignores or replaces it and nothing rounds up. `G13-R09` closed by the phase-state reconciliation. `G13-R06` RECLASSIFIED at reproduction, then ruled a repair 2026-08-22: the executor's diagnostics move to a mandatory `--output` file argument carrying typed facts only, raw child material moves to a mandatory `--elements-output` file, and nothing is written to stderr — so the channel the scope debate was about no longer exists; chartered with the `G13-R18` framing lane. `G13-R18` CONFIRMED on all four sub-claims: blank records skipped, no byte bound, the handshake accepts unknown fields, and exactly one of the five framing cases the Rust side distinguishes exists in the reader. Two couplings the review did not draw: the live lane publishes its coverage through the forgeable discharge API, so `G13-R01` and `G13-R12` repair together, and one caller-authored tuple was measured discharging forty-two rows. Integration lane green on the shared instance: clippy 7.5s, the workspace suite 339.9s |
| `T5-002` | DONE | Guide-13 Wave 0 preflight repairs: all eighteen rows closed across six serial-then-parallel lanes, each merged only after diff review and a server-side integration lane. Transaction: superfluous-witness refusal with the universal re-encode law, form exactness with the empty-offer refusal and the shape's form a fourth conjunct of selection, all duplicates refused. Tapscript: u16 index domain, fallible shape sets, settled-comparison truth recorded without fabricating canonical bytes. Vectors: coverage discharge transcript-derived behind a validated operation report matched on submitted bytes, weight mismatch refuses the plan, the live lane renders from the validated report without wall time, sponsor-change presence represented end to end. Linker: one u128 checked domain with the sixteen-leaf budget promoted to an enforced precondition, duplicate leaves refused, program role derived. Conformance: zero tweak admitted per the target rule with an independent pure-Python recomputation, response validators exhaustive over role and outcome. Compiler and neighbours: thirty enum censuses generated from one declaration. Python executor: the ruled mandatory `--output` and `--elements-output` files with nothing on stderr, marker-proof quarantine, strict bounded request framing with contract-side bounds. Repair lanes surfaced and fixed eight defects beyond the register. Residuals carried: saturating arithmetic in the linker's graph reference counts and bundle resource folds; the vectors planner transcript's public Default; the ADR-018 provenance seam in the validated report; the phase-index table outside the check-plans weld; twenty-one hand-maintained censuses in conformance, vectors, transaction, and labels; the census macro's three-crate triplication; the lifecycle response validator having no production Rust caller; and the live lane plus the change-present sponsored row awaiting a real node |
| `T5-003` | DONE | Two residuals named in the Wave-0 completion record discharged by the first Codex worker lanes (gpt-5.5, read-back gated, edits reviewed and committed by the orchestrator). Linker: reference-edge site counts accumulate with checked arithmetic and refuse instead of saturating, the zero-count fallback became an invariant-stating expectation, and the script-byte total sums exactly under a proven construction bound of eighty-four million bytes, with four focused tests reaching the repaired branches. Labels: the phase-index table joined the check-plans weld — a card without an index row, a row linking a missing card, a status disagreeing with the card's own declaration, a duplicate row, and a missing section or table now fail with nine focused tests; on arrival the weld caught real drift, the index calling phases 03 and 04 Complete while both cards declare Exited, repaired in the same commit. Both lanes gated on the shared instance: the first pass bounced on formatting and two documentation lints because the lane design carries no toolchain, fixed at the gate; the second pass is green with a forced re-check |
| `T5-004` | DONE | Guide-13 Wave 1, the evidence handoff: every canonical negative vector states its whole declaration as typed data — source fixture requirement, intended relation, mutation class, changed target fields, semantic change, evidence boundary, carrier, collateral policy, representation and sponsor case — with the changed fields and semantic change recomputed from the mutated bytes rather than asserted, which corrected two guessed declarations; plan derivation requires exactly one matching requirement and a contradicted declaration is a hard refusal, honest blockage staying a typed pre-resolution reason. The first-party negative policy landed with its proving instance: the permissionless-private-dependency rows discharged end-to-end through the exact owning compiler validator, extracted so an evidence package drives the analysis's own check and never a mirror. The withheld pre-target classes land at a typed ABI-validation boundary whose indexed set is proven the exact complement of the submittable set, with both rows observed safe-constructor-refused over all nine materialized vectors. The ADR-018 provenance seam closed the smaller way: the transcript retains its configured expectation and conformance publishes the comparison, never the operands, so no consumer can compare a report against itself; a failed comparison refuses the whole report and the rendered schema moved to revision two so an old reader refuses rather than conflates. Four resolutions and twelve typed blockages, unchanged from the earlier census — the machinery grew and the honest number did not. Merged after full diff review and a green server gate; the ruled follow-up wiring the discharge into plan derivation is its own record |
| `T5-005` | DONE | The ruled first-party discharge wired into derivation: derive_evidence_plan builds the canonical permissionless-private-dependency case, validates it through the sole validator, and discharges exactly the two first-party rows through the existing path, refusing with a typed error when either half goes missing; reported coverage moves to 102 of 211 — ninety-nine positive rows outstanding for want of a target, one negative runtime row, two first-party rows discharged — with the fresh-plan controls restated at the new baseline and double-counting impossible because coverage rows overwrite rather than increment. Residual carried by name: a first-party validation refusal rides the borrowed unclassifiable-negative-requirement variant, unreachable today and awaiting a named variant in an error-surface wave |
| `T5-006` | DONE | Guide-13 Wave 2, the validated live-transfer target plan: a new compiler module derives every clause of the semantic contract from the typed realization and checks it against the guide's fixed value, so a drifted realization is noticed and drift is never published as contract, with nineteen named clauses and a both-directions exactly-one rule per clause. Two representations planned side by side — the split drawn where the equivalence rule draws it, measured first against the analysis — with both named deferrals derived under trip-wires that refuse when a future realization would widen them silently. The forbidden-class census derived by complement over all object families, the owner quantifier grounded as the agreement of two relations over one family, candidate cardinality bounds carried unresolved, and the roots and certificate projections minted where the wave list dropped a clause. The validator covers every corruption row including the capability and evidence censuses; the compact-ASH validator's omission of those same two checks is a confirmed substrate defect dispatched to its own lane. Merged after diff review with the trust pattern verified and a green server gate: six hundred twelve library tests, twenty-eight boundary tests, formatting and workspace lints clean |
| `T5-007` | DONE | The compact-ASH plan validator gains the aggregate-census checks the live-transfer wave found missing: capabilities and external evidence are re-derived from the relation rows through the two helpers the constructor itself now calls — one authored derivation on both sides — refusing with the variants the live-transfer plan minted, and two corruption tests reach both refusals. First lane run under the worker verify loop: the lane's uncommitted worktree was snapshotted through a temporary index, verified on the shared instance before the diff was reviewed, and came back green in one round, retiring the gate-fix pass the first three toolchain-less lanes each cost |
| `T5-008` | DONE | Guide-13 Wave 3, signature and CT target closure: the confidential-conservation evidence requirement follows the selected proof alternative — committed conservation loses its runtime carrier and lands as external target evidence, splitting the representations' measured censuses (private sixty-one layout and two hundred twenty-five coverage rows against explicit sixty-nine and two hundred twenty-three) with one shared visibility decision read by placement and coverage alike. The owner sighash profile is selected in the adapter from the protocol's fifteen protected data against the target's dimension vocabulary, fail-closed over unconsidered dimensions and honestly assessed review-incomplete. The test-only signing capability reproduces the published BIP-340 vectors byte for byte from their published secret keys under the test-material rule, signs only messages it is handed, and carries no production shape. Sixteen owner-authorization cases each name the protected datum they disturb, with the narrowing guard proven firing per case and every case carrying its residual; the mixed-representation refusal is exercised by hand in both orderings. Merged after diff review and a green server gate over five packages. Findings carried: the SDK and server clippy now diverge in both directions; the rustdoc bar fails on pre-existing links in tapscript and conformance, dispatched to its own lane; the conformance census residual text is stale in one respect, left for the native-signature wave |
| `T5-009` | DONE | Three residual discharges by parallel Codex lanes under the verify loop, merged serially after review. The operation transcript is buildable only by its planner: the public Default that contradicted the type's own claim is gone, a crate-private constructor replaces it, and the preflight control strengthens to no transcript of either kind being constructible outside the crate. The first-party refusal names its own cause: a typed variant carries the first-party module's refusal through both derivation sites, retiring the borrowed variant whose name described a different failure. The rustdoc bar passes for tapscript and conformance: five pre-existing link defects repaired, and the verify loop gained a documentation stage that builds with warnings denied. Model evaluation rode the first lane: gpt-5.6-sol matched gpt-5.5 on read-back fidelity and finished its work step faster than every gpt-5.5 baseline, with one house-lint bounce repaired in one round |
| `T5-010` | DONE | Guide-13 Wave 4, the candidate live-receipt constructor: canonical owner metadata derived from the target's own encoding registry — the guide fixes that one canonical encoding exists, the target fixes which — with the no-secret rule carried by the encoding domain, the only check that separates an approved public key from the scalar a private key is written as, and its ordering before the class gate proven by test. No key-path escape holds as a property three ways: one-variant inherited policies, a spending-route census derived from the leaf set, and construction refusing an empty transfer leaf set, the escape reached by omission. Two disjoint per-representation leaf sets foreclose in-script representation dispatch structurally; the one-to-one shape contributes no member leaf, derived from the position rule rather than assumed. The lifecycle is structurally release-incomplete and the candidate-ABI binding is recorded outstanding rather than invented, the section-7.1 and wave-order disagreement recorded as a guide erratum. Seventeen mutation cases published as executable data: eleven built, run, and refused by the variant that names each; six carry residuals naming the wave they wait for. Eight fail-closed guards over the validated plan are named unreachable-today in the report. Merged after diff review with the pattern-docs collision against the parallel doc lane resolving identically; one server gate-lint fix (assert_ne over assert-not-empty), second gate green including the documentation bar |
| `T5-011` | DONE | Guide-13 Wave 5, recognition and owner authorization: the abstract walk learned to say what section 1.8 asks — the target's two signature successes leave identical stacks, so the walk now reports which successful form each signature primitive can reach beside the states, and the forward-compatibility path became a computable predicate rather than an argument. The owner key is never a witness item: the constructor-committed key is pushed by the program at the approved encoding's exact width, and the walk reports the unverified form unreachable; the six mandated negative cases run as executable mutants with registry-derived widths, the two abstractly indistinguishable ones each naming the first-party oracle that separates them. The curve-point-membership residual is discharged in conformance over the existing curve arithmetic with no new dependency, the emitting adapter deliberately kept free of the checker's opinion. Recognition is proven local by construction — an owner-parameterized constructor forecloses the compact-ASH cross-input comparison, recorded with the mutual-fixed-point argument — and reads no amount, which is what lets one fragment serve the private plan. Coordinator and member placement carries an honest eleven-check census: one established, three partial with named remainders, seven typed-outstanding pointing at the exact later-wave patterns. The empty-key census defect found and fixed: section 1.8's empty key had been mapped onto the missing-signature behaviour, a different target fault. The sighash-review residual is a computed disposition required on every signing pattern, and the sighash profile stays a link-time symbol with the reasons recorded. Merged after diff review with the six load-bearing tests recomputed locally; server gate green in one round including the documentation bar |
| `T5-012` | DONE | Guide-13 Wave 6, the explicit live-transfer plan: the coordinator gains its global checks — destination closure reading the target's own output fields against five newly consumed link-time symbols, exact aggregate conservation whose every addition is followed immediately by the verify that consumes its success flag (proven over every admitted shape, with an executed negative showing the overflow residue an unconsumed flag leaves behind), sponsor isolation that never introspects a value field, and issuance absence tested on every input including sponsors. The eleven-check census flips to seven established and four partial owing exactly two named remainders: private conservation for Wave 7, and the destination-constructor identity — unreachable in script because a destination's constructor is owner-parameterized, so the Wave-5 predecessor residual is re-scoped rather than discharged, the same residual now carried at both ends of an induction stated in three parts where recognition documents itself. Family ranges classify every position of every shape into exactly one authenticated family with four executable refusals, and the explicit programs assemble into a candidate relocatable bundle whose emission runs the final-stack and family-range oracles per leaf and refuses on either. One substrate defect fixed at its root: the abstract walk's byte equality was blind to operands of provably disjoint widths, which made issuance absence unverifiable; the narrowing removes only successful forms, never aborts, and settles nothing for untaught types. Two findings recorded for later work: the compact-ASH bundle's symbol-width table disagrees with its own relocations about leaf-program width, documented at the live symbol table; and the walk treats a per-path operand type mismatch as a whole-program error, foreclosing a class of branch pruning. A permanently empty introspection census was removed under the no-unconsumed-field rule rather than carried as dead weight. Merged after diff review with ten load-bearing tests recomputed locally; server gate green in one round over four packages including the documentation bar |
| `T5-013` | DONE | Guide-13 Wave 7, the private live-transfer plan — delivered, not deferred, with no new dependency and no confidential fixture, because a value's form is a registry prefix and nothing in the plan ever needs an opening. The private coordinator settles its own destination value form and carries every representation-independent global check, while confidential conservation lands as external target evidence through a third placement set and a status that says so — the outstanding-pattern promise the guide forbids is now structurally inexpressible, and the six-line soundness argument is a total census with the conservation row's fragment set asserted empty in both directions. Six of the seven opacity prohibitions are checked properties over the emitted programs, the seventh a named non-claim asserted to be the only one. The candidate bundle accepts both representations under the same per-leaf oracles with the leaf sets still disjoint. The wave's headline is a substrate repair: the confidential value encoding declares two prefixes, one per commitment parity, and the pattern helper compared only the first — so private recognition as merged would have refused about half of all valid confidential receipts, for a parity their owner neither controls nor observes; repaired by an exact masked membership test derived from the registry that refuses rather than widens when a prefix set is not precisely what one mask admits, the explicit plan's bytes unchanged. A second overclaim corrected: destruction absence cited the explicit arithmetic the private coordinator does not carry and now names the external requirement. Both confidential prefixes are flagged for target-native exercise since the abstract walk cannot separate them. Merged after diff review with the full tapscript suite and the named opacity, soundness, and mask tests recomputed locally; server gate green in one round over four packages including the documentation bar |
| `T5-014` | DONE | Guide-13 Wave 8, the linked live-transfer candidate: the linker's tree construction becomes generic over one leaf trait so one Huffman, one exact cost arithmetic, and one oracle pair serve both vocabularies with compact ASH's behavior unchanged, and a closed-form equal-weight optimum joins the search oracles because the candidate commits twenty-nine leaves — past the exhaustive budget — with the closed form cross-checked against both oracles everywhere they run and the committed tree reaching the proven optimum of three leaves at depth four and twenty-six at depth five. The fifteen guide-listed symbol roles land typed, the constructor symbol parameterized by owner and representation, the sighash profile entering from the reviewed contract with no deployment parameter so the exactness ruling cannot be weakened by passing a different value, and role completeness is a checked census scoped to the carried plans. Carrier closure compares required, emitted, and reachable per representation against each plan's own tree — made possible by a plan-neutral case key after finding the existing key embeds the representation, which made every cross-plan comparison a tautology — measuring thirty-two explicit and thirty private carriers with exactly two explicit-only, both the conservation rows the private plan carries as external evidence, and a starved plan is a typed refusal naming itself. The induction's link-time end is established as one linked constructor placement per owner and representation while the output key and destination table stay honestly outstanding, the recognition residual re-scoped on both ends. Five mutation-case flips retire the linker residual and name the ABI dependency it stood in front of; the bundle width disagreement is fixed at its root; a four-axis live resource model replaces the three-axis one that could not express output freedom. Merged after diff review with both suites recomputed and the twenty-nine-leaf optimum verified by hand; one server gate bounce on the emptiness-assert lint at three test sites, fixed typed, second gate green over four packages with the linker joining the documentation bar |
| `T5-015` | DONE | Guide-13 Wave 9, the candidate transaction ABI and signing flow: the typed request carries the eight selectable facets while the fourteen the guide forbids are absent as fields rather than checked as rules, each absence naming where the refused choice actually comes from; canonical input layout with the three rejections landing before the sort, request-order outputs with the semantic comparison on the multiset, and the sponsored form's five-way equivalence tested arrow by arrow with an empty capability proven to refuse rather than downgrade. The finalization boundary holds twice — structurally, since the finalized form has no mutable route and signing requests are constructible only from it, and by typed refusal, since an offered transaction is compared against an independently held census and the mutation named. Owner responses arrive as position-response pairs so a duplicate can exist to be refused, all ten multi-owner rejections carry executable negatives with the wrong-owner and wrong-bytes rows also proven against real published-vector signatures, and the three authorization levels stay distinct types. Both linker handoff obligations discharge: destination owners select linked constructors from a typed table closing the request end of the induction with the predecessor owner recognized rather than selected, and the taproot output key is recomputed in the conformance oracle's own arithmetic, delivered through a capability so the transaction package stays free of curve code, and compared byte for byte in the vectors integration. The curve-point loop closes at the one place every owner passes. The private test construction records the central-public-fixture model as a type that refuses every other model, with a new non-claim that no range proof is produced or checked. The mutation census sharpens to five cases waiting only on the target-native run; two multi-owner authorization cases flip while the sponsor-owner case deliberately does not, its signer being adapter-borne. Found in passing: the vectors package had been failing the documentation bar on two pre-existing links, repaired, and the gate's suite and doc bar both widen to five packages; the conformance meson census is membership-checked but not order-checked, flagged; the brief's may-not-select count was a miscount against the guide's fourteen, the guide's own list implemented and asserted. Merged after diff review with the suites recomputed including the tweak-against-oracle equality; server gate green in one round |
| `T5-016` | DONE | Guide-13 Wave 10, the complete live-transfer safety evidence and the run against a real target: the whole hundred-and-eight-row section-15 matrix lands as constant registries in guide order — sixteen positive explicit, ten positive private, fourteen owner and signature, sixteen object, twenty-two value, thirteen sponsor, seventeen structural — each row resolving against the published plan to exactly one requirement, intending the whole census, or naming its typed reason, with the explicit arithmetic rows inactive under the private plan by the conditional-coverage rule rather than dropped. The canonical evidence plan derives with its seven sources accounted for, the minimality registry honestly typed as not yet built for Wave 11. Eleven owner-and-signature rows discharge first-party, each staged case driving the signing flow or the offer check twice so the refusal names the row's own class; the two sponsor report rows are answered at the report boundary by key rather than substring, since a substring search would have flagged the matrix's own row name. The validated safety report excludes all ten volatile fields from its canonical bytes with a test that proves each marker present in the diagnostic rendering and absent from the canonical one, and the renderer takes no diagnostics argument so the exclusion is structural. The zero-required-infrastructure-errors bar is honestly not achieved and the report validation refuses completeness naming every outstanding row: seventy-nine rows are typed-blocked and sixteen first-party rows undischarged, the root cause being one missing component — no first-party code computes the target's own taproot sighash and the guide forbids asserting one. The native run made that blocker observed rather than argued: against a real Elements daemon the ceremony issued its asset, re-linked the deployment against the issued identity through a new asset-parameterized link, funded both constructors, and submitted the explicit transfer, which the node refused with its own words — mandatory script verify flag failed, invalid Schnorr signature — meaning the covenant ran, every coordinator check passed over real coins, and evaluation failed only at the owner signature; the run of record is committed as a typed transcription labeled not evidence with the destroyed chain's outpoints deliberately absent, and the ceremony proved deterministic when the server rerun reproduced it field for field down to the issued asset. Both parity forms and the range-proof question are recorded unanswerable through this boundary because the funding subject has no confidential form, so no confidential predecessor can exist to spend — a protocol-revision-level gap, not a wave one. Both census flip counts are honestly zero, the worker correcting the brief's claim that two authorization cases waited on the run when they wait on the sighash review no run discharges. Found in passing: the built node was unrunnable outside its build toolbox until its libraries were staged beside it, which prompted the standing move of the native lane to the shared instance where the node is now built from our own hub at the reviewed tip; the ten-second plan derivations are memoized. Merged after diff review with the scoreboard recomputed structurally from source to exactly one hundred and eight, the canonicalization spot-checked, and the suite proven green with no node present; one gate bounce on a package-name typo in the orchestrator's own script, second gate green over five packages in the usual four minutes |
| `T5-017` | DONE | Guide-13 Wave 11, the disclosure-minimality comparison and the first-party completion of the safety matrix: the section-16 pair registry lands with all five pairs claimed against the candidate ABI's own shape table — one-to-one, split, merge, many-to-many, and sponsor — each pair growing from exactly one semantic fixture with both members built by the same function, and all ten members actually finalize with real target bytes, what is missing being only the test-only owner-signing capability. The section-16.2 acceptance conditions are scored one by one: seven of ten hold first-party for every pair — owner and class agreement, explicit reserve agreement, family closure, private amounts absent from public output, no opening in the report, no secret outside the declared model, equal lifecycle — while constructibility and acceptance carry the two observed blockers and projection equality is typed as awaiting both verdicts rather than as a third blocker, because a reader repairing the pipeline needs the dependency direction. Minimality is therefore reported unanswered, in a validated report deliberately separate from the safety report down to a qualified schema identifier on its first line, with the two documents rendered side by side in a test and told apart at the first line and the role line in both directions. The disclosure tables land verbatim from the guide with the counts recorded as transaction-shape leakage under both plans, the two explicit-plan exact-amount disclosures each carrying the typed reason that the program checks conservation over fields it can read. First-party discharge rises from eleven rows to twenty-four of twenty-seven: every staged case drove its owning validator twice across three refusal vocabularies kept unmerged, and the three resisting rows carry specific typed obstacles rather than absence — one row's declared boundary disagrees with the validator that actually owns its class, one mutation is structurally inexpressible in the leaf vocabulary, and one names a genuine gap: nothing on the live-transfer request path enforces the semantic amount ceiling, the transaction crate holding the realization vocabulary as a dev-dependency only, recorded as a design decision for the owner rather than fixed unilaterally. A dead refusal variant was found in passing and left in place, named. The evidence plan's minimality source flips from not-yet-built to built with five pairs and zero supporting, carrying counts and no verdict. Merged after diff review with the scoreboard recomputed to one hundred and eight and the report-separation test read in full; per the no-host-compute ruling the server gate is the sole suite verdict, green in one round over five packages, and the native lane rerun on the server against the wave's own tree produced a report byte-identical to the committed run of record |
| `T5-018` | DONE | Guide-13 Wave 12, the resource study: the three bound symbols land with the guide's own research candidates — two hundred ninety-four assignments, each costed by closed form and checked against the real shape and leaf sets rather than argued from the widest — and the binding constraint turns out to be the deployment's declared taptree depth of eight rather than the linker's subset oracle, asserted from both sides with two hundred fifty-five leaves building and two hundred fifty-eight refusing at the declared capacity, the leaf budget refusing the widest assignment outright. The position domain accumulates in a wider integer precisely so that exceeding the wire width is reportable, and the honest fitting statement is that the tested bundle, publishing a window the enumeration does not name, has a program for every shape of exactly eight enumerated assignments, computed over set membership. The fifteen measurement cases land as seventeen complete transactions through the real construction pipeline, each recorded across all seventeen dimensions: every case at taptree depth five, peak main stack seven, and a measured alternate-stack zero kept honest by requiring the same walk to prove itself on the main stack; the sponsor region costs five hundred twenty-nine weight, the change output three hundred ninety, one private commitment over explicit forty-four; the validation budget tracks exactly fifty per receipt input; and a merge was found to spend coordinator and member leaves at different depths, so no control total is one width times a count. The prediction-against-observation comparison ran against the real target over the same exact bytes: one dimension of seventeen is observable at this boundary — the node reads back transaction weight even for refused bytes — and predicted equals observed at nineteen hundred eleven, cross-confirmed by the study's own independently constructed sponsorless case weighing the same, while the sixteen unobserved dimensions each carry a typed reason and a staged disagreement proves the typed planner failure fires. Both Wave-11 hooks are filled and their deferral standings retired: four sponsorless pairs carry measured weights whose type name says no confidential proof is counted, the sponsor pair carries the signer absence as the reason no complete transaction exists, and the hard-target-limit failure mode is computed within declared limits rather than deferred. The validated resource report has its own schema identifier, the ten volatile fields excluded with a teeth-bearing test, and a result vocabulary holding the guide's candidate-bounds sentence verbatim with no production variant to reach for. Found in passing: Wave 10's wire protocol already carried transaction weight that the observation record dropped, repaired; a duplicate abstract-walk implementation now exists as a named consolidation candidate; the executor writes absent operation figures as zero on the wire, a substitution the guide forbids, recorded as a feature-request candidate; and the two-resource-study ambiguity in the vectors plan is Wave 13's disambiguation. Merged after diff review with the closed forms and boundary assertions recomputed by hand and the weight deltas cross-footed from the table; the wave verified entirely on the server per the no-host-compute protocol, and the merged tip's gate ran six packages' tests including the freshly changed labels crate plus the five-package documentation bar — a first attempt at extending that bar to the labels crate surfaced pre-existing unresolved intra-doc links there, recorded as documentation debt rather than fixed in a resource wave |
| `T5-019` | DONE | Guide-13 Wave 13, the Phase-5 gate and handoff: the three rows that resisted first-party discharge in the T5-017 snapshot are resolved by re-typing what each one claimed rather than by staging cases against falsified boundaries, and the first-party half of the safety matrix closes at twenty-five of twenty-five with the undischarged-fault register deleted outright rather than left empty. The wrong-constructor-schema row moves to a freshly minted constructor-derivation boundary and a static-constructor-schema mutation layer — the guide erratum is filed in the feature-request register, the row's three malformations each driven to their exact typed refusal against one accepted control, and the matrix's shared vocabulary keeps the two guides' counts apart so a member minted for one cannot be read as named by the other. The amount-outside-semantic-domain row is re-typed under the ruling that the ceiling is blockchain-enforced, the same class as conservation: its boundary becomes consensus rejection before script, where the reviewed target refuses a stated amount above its own bound, the transaction layer deliberately gains no request-path check, and the verdict stays typed-blocked behind the absent accepting control like every other target-negative. The mixed-operation-program row leaves the staged-refusal denominator as an operation-vocabulary closure that no layer answers because no typed input names its fault — a two-member row-boundary type carries the distinction, the standing is counted in its own census bucket that no report reads as evidence, and no general inexpressibility discharge is minted. The dead refusal variant the T5-017 record left in place, named, is now wired: an unlinked representation is refused at the head of finalization before the ten construction stages, with both mirror cases tested and the owner-specific refusal proven not swallowed. The safety report schema steps to revision two for the new census line, and the native run of record is byte-identical across three runs at three tips, verified by digest on the server. The Phase-5 card records the candidate pipeline complete, the acceptance lines that hold, the two typed blockers owned by the chartered intermediate guide, the impact and non-claim record, and the four-item handoff queue. Merged after diff review with the twenty-five pre-target boundary occurrences recounted from the row tables and the report digests compared server-side; the wave verified entirely on the server, five packages plus the native lane green in three hundred five seconds at the final tip |


### 2.10 Confidential-funding batch gate · `gate:backlog:ctf-guide`

The intermediate confidential-funding execution guide closed 2026-08-25 with its five waves merged (T5-020, T5-021, T5-023, T5-024, T5-033, T5-034) alongside the parallel owner-sighash track (T5-022, T5-025 through T5-031) and the headroom carve (T5-032). The full repository suite ran green on the shared instance at the batch tip: 48 of 48 meson lanes, zero failures, 567.1 seconds of wall time, both plans validators clean and the register byte-stable. The batch result is the typed stopped one its closeout anticipates: restart steps one and two accepted against a real node, two positive private matrix rows answered by observed acceptances, the stop at target CT conservation named with its ground, and the two carried residuals unchanged. The executed guide is archived in [the guides directory](../guides/guide_confidential_funding.md) and the wave-by-wave closeout in [backlog history](ctf-guide-wave-5-closeout.md); both Guide-13 blockers this guide was chartered to own are resolved — the owner sighash cleared by the parallel track, the confidential predecessor funded and its residual cleared.

### 2.11 Interlock resolution and handoff gate · `gate:backlog:ctf-interlock`

The step-3/step-4 interlock that stopped the confidential-funding evidence restart was researched and discharged 2026-08-25 (T5-037), alongside the two remaining Phase-5 handoffs (T5-035 sponsor signing, T5-036 internal-key probe) and the shared-analysis first slice (T5-038). The ruling: Wave 5's step-3 stop was unwarranted, because the guide asks only for CT conservation recorded against a balance-valid accepted control and the Elements balance check refuses a wrong-blinder mutant before the range-proof loop, so one observed run is both conservation's non-conserving half and step four's first proof-negative; the follow-up wave re-ran steps one through four against the pinned node with that disclosure written into the record, moving the target-CT-conservation row. The full repository suite ran green on the shared instance at the batch tip: 48 of 48 meson lanes, zero failures, 579.9 seconds of wall time, both plans validators clean and the register byte-stable. The restart order remains typed-stopped at step five: the split, many-to-many, and several-owner shapes need multi-output and multi-input fixtures nobody has built, and private-merge is structurally unconstructible against the guide's own one-output merge predicate. The Phase-5 exit is therefore not yet reachable — its relation-coverage and confidential-transfer criteria depend on those shapes — and steps five and seven are handed to a following wave.
