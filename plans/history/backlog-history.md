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
