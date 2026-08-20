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
