# Tripod Implementation Backlog

> **Status:** ACTIVE
> **Current gate:** Phase 4 — End-to-end compact ASH
> **Current condition:** Phases 1 and 2 are complete. The target-independent compiler constructs one deterministic, validated scoped analyzed program for the complete pilot scope, factorized per operation, with relation-owned requirements, a corruption-resistant assembly validator, and independent assembly oracles; the Phase-2 exit gate record is in §2.11, including passed document reproducibility. Phase 3 then delivered the Guide-8 target foundation, the Guide-9 native primitive gate, the Guide-10 constructor and wide-floor prototypes, and the Guide-11 declassification result, each with its own gate record. Public declassification is no longer an open blocker: an initial policy is selected — an explicit boundary reached by owner-authorized normalization, with the direct and public-committed paths deferred against three named target blockers — and the gate record is in §2.4. Phase-3 exit is recorded: the Guide-12 preflight register closed all sixteen seventh-review rows, and (`gate:phase3:exit`) was checked green on that tree with the record in §2.6. The current work is the first complete compiler-to-target operation, compact ASH, which consumes the declassification policy rather than reopening it.
> **Next gate:** Phase 4 — compact-ash end-to-end pipeline
> **Authority:** Current execution queue only. The specification, the realization document, typed architecture, implemented ADRs, accepted decisions, package contracts, phase cards, and accepted research results take precedence.

This file contains only:

- the current repository and readiness state;
- the latest static-review basis and open findings;
- compact historical milestones and gate records;
- the active Phase-2 compiler queue;
- immediate algorithm preparation;
- dependency posture;
- verification and clean-tree requirements.

Detailed historical implementation narratives belong to Git history, annotated
tags, phase cards, ADRs, accepted decisions, and focused evidence records—not
to the active backlog.

---

## 1. Backlog contract · `sec:backlog:contract`

### 1.1 Authority · `rule:backlog:authority`

This backlog sequences work. It is not protocol, realization, compiler, target,
ABI, evidence, deployment, or release input.

When artifacts disagree:

| Subject | Owner |
|---|---|
| abstract economic interface | the Attestation specification |
| realization meaning, invariants, obligations, and residuals | the realization document |
| finite registries and stable architecture IDs | typed architecture |
| current executable reference behavior | executable model |
| repository engineering policy | implemented ADRs |
| accepted cross-package implementation direction | planning decisions |
| package boundary | package contract |
| phase entry, deliverables, and exit | phase card |
| unresolved design question | research note |
| current task ordering | this backlog |

A lower owner never overrides an upper owner on the upper owner’s subject.

### 1.2 Status vocabulary · `tab:backlog:status`

| Status | Meaning |
|---|---|
| **TODO** | Ready when its named dependencies are complete. |
| **IN PROGRESS** | Actively being implemented, reviewed, or verified. |
| **BLOCKED** | A named dependency or correctness defect prevents safe progress. |
| **PARKED** | Deliberately inactive until a concrete consumer or measured need exists. |
| **DONE** | Implementation, focused evidence, required gates, documentation, and clean-tree evidence are recorded. |
| **DROPPED** | Deliberately not implemented; rationale and replacement are recorded. |
| **HISTORICAL** | Immutable evidence about an earlier revision; not a claim about the current checkout. |

Code resembling an intended result is not sufficient for `DONE`.

A static-review finding remains open until one of these occurs:

1. the finding is reproduced and fixed with focused coverage;
2. a typed proof shows the reported state is unconstructible;
3. reproduction shows the finding is false;
4. the owning policy or assurance claim is deliberately corrected.

“Existing tests pass” does not close a finding unless a named test reaches the
reported path.

### 1.3 Priority vocabulary · `tab:backlog:priority`

| Priority | Meaning |
|---|---|
| **P0** | Can manufacture false release, deployment, provenance, or semantic evidence. |
| **P1** | Trusted semantic/evidence boundary defect or active phase blocker. |
| **P2** | Required correctness, determinism, publication, identity, or documentation work before phase exit. |
| **P3** | Maintainability or evidence-quality work required by the active gate. |
| **POST** | Later-phase work that does not block the current gate. |

### 1.4 Definition of done · `rule:backlog:done`

An implementation task is `DONE` only when it records:

1. implementing source files;
2. focused positive and negative tests;
3. affected ADRs, decisions, package contracts, phase cards, or research notes;
4. exact verification commands and outcomes;
5. generated-publication and label impact;
6. semantic, identity, schema, and migration impact;
7. dependency impact;
8. final clean-tree output.

A review finding additionally records whether reproduction confirmed,
refuted, or reclassified it.

A dependency task additionally records:

- selected source and exact version;
- enabled features;
- transitive graph;
- licence;
- MSRV;
- unsafe boundary;
- determinism and parallelism implications;
- advisory status;
- lockfile impact.

A research task additionally records:

- exact question and constraints;
- prototype and tool versions;
- accepted and rejected candidates;
- positive and negative evidence;
- measurements;
- permanent implementation handoff.

### 1.5 Identity admission · `rule:backlog:identity-admission`

No new digest or digest-bearing field enters merely because an object is
important.

A proposed identity must satisfy
(`req:identity:admission-record`) and name:

- the complete typed object or exact bytes;
- owner and producer;
- a present consumer;
- the exact decision the consumer makes;
- assurance class;
- canonical projection and encoding;
- domain separator and recipe identifier;
- stale conditions;
- migration behavior;
- explicit non-claims.

No named consumer means no digest. No distinct decision means no digest.

---

## 2. Review basis and current evidence · `sec:backlog:review-basis`

### 2.1 Latest static review · `tab:backlog:review-basis`

The current review is the sixth static review, performed in two independent
passes over the supplied concatenation of the tree:

```text
tree:
   

selected files:
    220
```

The supplied filter excluded 352 files, among them:

```text
Cargo.lock
the paper sources
most unit and integration tests
most compiler implementation files
most executable-model implementation files
the label-tool implementation
several build/tooling packages and scripts
```

No Cargo, Meson, TeX, the native executor, advisory, or reproducibility
command was run as part of this review.

Therefore:

- the review makes no current green-build claim;
- the excluded packages and test directories received no content review;
- lockfile checksums and resolved features were not independently verified;
- advisory status was not checked;
- licence compatibility was not independently checked;
- historical gate records remain historical evidence only.

Its findings are recorded in §5.6. Its central verdict is that the native
evidence layer still does not bind evidence to its subject: an execution
transcript retains neither the requested subjects nor the exact fixtures
sent to the child, so it can be rebound to a census, matrix, target, or
deployment that was never executed; and the claim a fixture files under is
caller-authored, so claim-bearing metadata can be attached to an unrelated
or trivially true script. Both passes rank those as release-blocking for
that subsystem, and no public-declassification prototype begins while they
remain open. The review text is archived at
[plans/reviews/review-6-0.3.4-dev.md](reviews/review-6-0.3.4-dev.md).

### 2.2 Earlier review basis

The repository retains seven earlier reviewed trees as historical context:

```text
initial reviewed tree:
   

follow-up reviewed tree:
   

proof-planning reviewed tree:
   

second-review tree:
   

third-review tree:
   

fourth-review tree:
   

fifth-review tree:
   
```

Those reviews and their findings are evidence about their exact trees. They are
not current-checkout execution evidence. The second through sixth review
texts are archived under [plans/reviews/](reviews/README.md).

### 2.3 Historical gate records

The completed gate records for Phase 1, the T1–T5 remediation, the P2-004
compiler-input batch, and Guides 2 through 10 stood here as §2.3–§2.14.
They are archived verbatim, with their original numbering and labels, in
[backlog history](history/backlog-history.md), so a reference to §2.8 or
§2.11 anywhere below resolves to the heading of that number there.

### 2.4 Guide-11 declassification gate (archived)

Closed and archived verbatim, with its label, in
[backlog history](history/backlog-history.md) beside the earlier
records; §2.4 references below resolve to the heading of that number
there.

### 2.5 Draft-promotion gate · `gate:backlog:draft-promotion`

The 2026-08-19 batch executed the user's promotion ruling: the
identity-adjudication and interchange drafts take priority as the
disciplines, ADR-016 is deleted rather than kept superseded, every
overlap is deleted rather than restated, and ADR-010 stays untouched as
a different scope. Wave 1 minted [ADR-021](../adr/021-identity-adjudication.md)
carrying only adoption data — the single local recipe convention, the
current-identities table, the separation migration verbatim, the
ADR-011 amendment linkage, and three recorded divergences — and
retargeted all ten imported citations of the retired owner; the
released realization contract took one prose sentence with a three-way
proof no identity moved. Wave 2 minted
[ADR-022](../adr/022-interchange-conventions.md) adopting the
fourth-edition interchange conventions wholesale, stated the ADR-010
boundary one-directionally, restated the executor wire-frame stop from
the identities register with its revisit condition intact, and
discharged DI-005 — the integration queue of §13 is closed. CI-005 was
recorded in the same batch under the user's wall-time ruling.

Gate, content-scoped per the cadence ruling and CI-005: the light lane
registry passed 9 lanes with the two test lanes skipped by design, 331
cumulative seconds inside the runner, 5 m 31 s wall; the corpus checker
oracle passed all four checkers in 5.6 s on the merged tree; the full
workspace suite ran green once, inside Wave 1, immediately after the
batch's only Rust-touching edits, and the delta since is markdown only.
No identity moved: the generated manifest is byte-identical and both
registers regenerate byte-identically. Weight: this record's room came
from archiving the closed Guide-11 gate record to history per the
CI-004 pattern, 9470 bytes net. Lane wall-times, recorded per the
wall-time ruling: Wave 1 two stints (interrupted by a harness restart,
resumed with work intact), final stint 9 m; Wave 2 two stints, final
stint 20 m 19 s; both workers' reports carry their own lane timings.
Found in passing and owed to CI-005: the golden-capture script prints
each checker's exit status but exits 0 itself, so a chained commit can
proceed past a red checker — it did once here, caught on the next run;
the archival also taught that a block moved into history must have its
relative links re-based, which the checker catches as broken links.

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
[backlog history](history/backlog-history.md) under the CI-004 pattern
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
misleading balance diagnostic, and the money-bound row the user ruled
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
the issuance diagnosis remains recorded and unfiled pending the
user's ruling.

The runtime half of `G12-R04` is discharged, 2026-08-21, by re-reading
these transcripts rather than by running anything new. The fifty-four
first-party record files — eight adapter response streams and
forty-six report and comparison documents, under
`/workspace/loops/attestation/` in the eight wave directories from
`wave11` to `w14`, fetched 2026-08-21 from the ephemeral shared
instance and pinned by an aggregate sha256
taken over their sorted per-file digests — carry 481 record objects
and 5769 string leaves. Every value of every diagnostic-bearing field
the executor can write is a mapped script-error message, a mempool
reject reason arriving on the JSON-RPC result path, or fixed typed
text; none carries a filesystem path, a client stderr frame, or an
exception message. The counter-check is what makes that evidence
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

### The verification harness's two standing hazards · `rem:backlog:verification-harness`

A verdict is read from the report wrapper's own report line, never from a pipeline's shell status: the wrapper propagates its exit code faithfully, and a pipe to a filter truncates the status to the last stage's. And a single shared build-target directory is poisoned when checkouts of two different commits build the same crates into it, so a lane that moves between commits pins a target directory of its own.

### Publication metadata · `rem:backlog:publication-metadata`

The published documents carry their attribution and their terms: the attribution position names Proof Commons, a commons rather than a company, and the document set is dedicated to the public domain. A specification exists to be reproduced, cited and implemented, and an attribution condition taxes exactly that use, so the attribution that matters is the one the title page carries. The rights line states the dedication, the licence-URL field names the canonical deed for machine readers, and the document licence file holds the legal code verbatim.

### The crates carry the product's name · `rem:backlog:crate-naming`

Each crate manifest and the build system's project name the product, so the product the repository describes is the product its artifacts are named after. The name feeds nothing that hashes, exports or gates: the versioning derivation reads the version alone, the binary and library target names are their own, and the generated model artifacts never carried it.

### The realization's fixed body states its law · `rem:backlog:law-delimitation`

The realization's fixed body states the law without restating the era. The versioning law leaves its worked example to the masthead, which prints the live binding; the hash pins defer to the algorithm the attached envelope names; and the schema discriminants are read at their carriers — the envelope for the architecture manifest, the canonical query context for the wire, the bound profile for deployment. The workspace authors field names the publishing organization.

### The shared crate's acknowledgement · `rem:backlog:cli-common-thanks`

The shared command-line crate's README thanks the developers whose work inspired the tooling. Gratitude for inspiration, stated once, naming no repository and claiming no lineage: the crate's own manual remains the authority on what it does.

### The tip's tests assert their own consistency · `rem:backlog:tip-convergence`

The release profile's identity test derives its expected value from the profile itself rather than comparing against a hand-pinned constant, so the tree asserts its own internal consistency instead of a value a reader would have to take on trust. The paper is Attestation, the product is Tripod, and the protocol object is the attestation contract.

### The licence terms and their citation families · `rem:backlog:licence-convergence`

The code licence is stated once, in the licence file, and the manifest key, the toolchain licence rule and the dependency comments agree with it rather than restating it a second way. The dependency-review clauses name what a dropped feature pulled in and which rule refuses it, which holds under any workspace licence, and the toolchain record says why a copyleft workspace strengthens rather than relaxes the permissive-dependency rule. Documents stay under the public-domain dedication.

### The hash-citation audit the lint suite runs · `rem:backlog:hash-citation-audit`

The audit holds every published hexadecimal value to the identity rule, as a labels binary in the lint suite wired exactly as the forbidden-text check is: it asks git for the complete tracked set and requires every value to be described by a rule in the citation-families table, which names what each kind of value measures and the program that writes it. The audit places rather than recomputes — whether a digest is right belongs to the test that owns it — and what it establishes is that every value has an owner on the record.

### The cut the settled records carry · `rem:backlog:record-cut`

A planning record earns its place by what it establishes, so the passages that spent themselves on a document's own corrections are gone and what survives argues the result. A citation is a promise that a reader can fetch what it names, so the sites that cited tags no hub carries name the phase cards and the archived record as the durable record, with nothing hash-shaped substituted. The archive index states its rule for when a settled record may be edited at all, and states it there rather than here: a wave record that paraphrases a rule it does not own carries a second copy of it, and the copy stops being true the moment the rule is sharpened.

### The discipline every published value follows · `rem:backlog:hash-citation-discipline`

Every hexadecimal value the tree publishes declares how a reader establishes it: recomputable from bytes the tree carries under a named algorithm, resolvable to a referent a reader can fetch, or defined by something other than its author. A value that nothing regenerates, nothing resolves and nothing defines stands nowhere, including inside an encoded rendering, which a decode-and-assert check reads. The cascade above the captures recomputes from the bytes it is taken over, and each chosen constant is a derivation from a string published beside it.

### The separators spell the product's name · `rem:backlog:separator-convergence`

A domain separator is hashed input that identifies a recipe, and the product's name is part of that identity. Each separator spells it at the version it carries, because no projection, encoding or algorithm moved. The identities that follow those bytes are re-taken by the tree's own generator rather than edited, so the typed pin, the generated artifacts, the realization's envelope and the versioning gate carry one result.

## 3. Current repository state · `sec:backlog:state`

### 3.1 Implemented areas · `tab:backlog:implemented`

| Area | Current source state |
|---|---|
| Specification | Released *Attestation* paper, version owned by the paper source |
| Realization document | Attestation Realization with final architecture appendix and tracked version binding |
| `architecture` | Typed architecture, validation, semantic/behavioural hashes, deployment-profile scaffolding |
| `model` | Executable state machine, invariants, property/corruption suites, indexer and accounting projections |
| `realization` | Target-independent typed semantics for compact ASH and live receipt transfer |
| `compiler` input | Owner-revalidated architecture/realization binding, explicit scope, strict policy |
| `compiler` graphs | Scoped relation and expression DAGs with stable typed projections |
| `compiler` folding | Conservative checked constant folding with independent oracle |
| `compiler` source analysis | Typed operands, authenticatable source requirements, sponsor-value read rejection |
| `compiler` constructibility | Authorization-case analysis and source/constructibility weld |
| `compiler` disclosure | Inherited and added disclosure with typed reasons |
| `compiler` lifecycle | Representation choices and required-exit analysis |
| `compiler` proof search | Exact deterministic feasible-plan enumeration; fixed external-evidence requirements retained, representation modes static; reaccepted in §2.8 |
| `compiler` placement | Typed execution cases, discharge classification, carrier eligibility, exact feasible placement sets, layout requirements, independent placement oracle; gate record §2.9 |
| `compiler` coverage | Relation-indexed coverage requirements, typed mutation catalogue, coverage dependency graph with forbidden-cycle SCC policy, independent coverage oracle; gate record §2.10 |
| `compiler` analyzed program | Relation-indexed requirement bundles, factorized operation analysis, scoped analyzed programs with a corruption-resistant validator and independent assembly oracles; gate record §2.11 |
| `artifacts` | Generated-publication derivation, writer/checker, realization-document weld |
| `labels` | Owner-aware Markdown/Rust label graph, census, plan checks, register rendering |
| `cli-common` | ADR-010 streams, diagnostics, checker report/stamp publication, batch publication |
| Meson | Explicit source census, stamp-backed checks, mocked document graph |
| Security policy | Public-data interfaces and external execution-environment boundary |
| Path policy | Central tracked-mode audit, lexical output roles, explicit host-filesystem non-claims |

### 3.2 Current published identities · `tab:backlog:identities`

Exact hash and version values are deliberately not duplicated here. Read each
identity from its authority:

| Identity | Authority |
|---|---|
| Specification version | `papers/attestation/main.tex` and `sections/00_title.tex` |
| Architecture schema, semantic/behavioural algorithms and hashes, anchor-set hash | `packages/model/generated/architecture.json`, checked against typed architecture |
| Realization version (tracked binding) | `docs/attestation/realization.md` masthead |
| Attestation wire schema | typed model/architecture constants |
| Deployment-profile schema | typed architecture constants |

The generated architecture publication is authoritative. Planning prose must
not become a competing identity source.

Architecture finality does not imply:

- complete realization scope;
- compiler completeness;
- target support;
- linked bundle or ABI existence;
- deployment evidence;
- production readiness.

### 3.3 Not implemented · `tab:backlog:not-implemented`

```text
public complete compiler-analysis API beyond the target-requirement boundary
compiler-plan identity
backend proof patterns
direct authenticated opening and public-committed representation
public opening capsule
production target-native evidence

tripod-linker
tripod-transaction
tripod-vectors
tripod-release

independent deployment observers
production deployment
production signer or wallet
production key management
```

### 3.4 Readiness statement · `rem:backlog:readiness`

```text
Attestation specification:          published
Realization contract:              published
Typed architecture:                final and pinned
Executable model:                  implemented
Typed realization pilots:          implemented
Compiler input/graph/folding:       implemented internally
Compiler source/disclosure/lifecycle:
                                    implemented internally
Compiler exact planning:            implemented and reaccepted
Compiler placement/layout:          implemented internally, section 2.9
Compiler coverage:                  implemented internally, section 2.10
Complete analyzed pilots:           implemented internally, section 2.11
Compiler target-requirement boundary:
                                    public and validated, section 2.12
Typed Elements target contract:     reviewed and welded, section 2.13
Capability and evidence adapter:    implemented, section 2.13
Typed instruction core:             implemented, section 2.13
Development target-native evidence: recorded, section 2.13
Production target evidence:        absent
Backend patterns/linker/ABI:       absent
Independent deployment evidence:   absent
Production deployment:             absent
Phase-2 exit gate:                 passed and recorded, section 2.11
```

Current packages are public-data tools. They do not legitimately accept private
keys, seed material, signing nonces, blinding factors, private openings,
credentials, or production authority.

---

## 4. Compact historical record · `sec:backlog:history`

### 4.1 Completed phases · `tab:backlog:completed-phases`

| Phase | Status | Durable record |
|---|---|---|
| Phase 0 | HISTORICAL | the recorded baseline and identities on [the Phase-0 card](phases/00-baseline.md) |
| Phase 1 | HISTORICAL | the completion evidence on [the Phase-1 card](phases/01-realization.md), and the gate record in [the backlog archive](history/backlog-history.md) §2.3 |
| Phase 2 | HISTORICAL | Guide-4 through Guide-7 gate records, §2.8–§2.11 |
| Phase 3 | HISTORICAL | exit recorded in §2.6 and the Phase-3 card |
| Phase 4 | Active | current backlog, Guide 12, and the Phase-4 card |

### 4.2 Historical finding families · `tab:backlog:historical-findings`

| Family | Status | Scope |
|---|---|---|
| `F1` | HISTORICAL | Phase-1 remediation |
| `F2` | HISTORICAL | Post-Phase-1 boundary remediation |
| `F3-001`–`F3-010` | HISTORICAL | Meson, publication, projection, ownership, and stamp remediation |
| `F4-001`–`F4-005` | DONE or DROPPED | Predicate, lifecycle, and labels findings |
| `A17-001`–`A17-005` | DONE | ADR-017 implementation |
| `R1`–`R6` | DONE | Path, ADR status, dependency, CI, recipe, and compiler-status review |
| `S1`–`S7` | DONE | Semantic-boundary review and status weld |
| `T1` | DONE | Bound conformance observations to executed requests |
| `T2` | DONE | Typed substrate-conservation evidence requirement |
| `T3` | DONE | Batch-staged multi-output publication |
| `T4` | DONE | Runtime-bound conformance in the global invariant |
| `T5` | DONE | Removed duplicated volatile identities from planning prose |

### 4.3 Identity work · `tab:backlog:identity-work`

| ID | Status | Result |
|---|---|---|
| `I1-001` | DONE | ADR-016 adopted |
| `I1-002` | DONE | Current identity inventory |
| `I1-003` | DONE | Future immediate-edge identity DAG |
| `I1-004` | PARKED | Typed evidence envelopes; waits for a persistent report consumer |
| `I1-005` | BLOCKED | Deployment-profile migration; waits for bundle, ABI, and evidence types |
| `I1-006` | BLOCKED | Release root; waits for the release package |

The identity freeze is lifted only under
(`req:identity:admission-record`). Phase 2 still mints
no public realization or compiler digest without a real consumer.

---

## 5. Current static-review findings · `sec:backlog:findings`

### 5.1 Summary · `tab:backlog:findings-current`

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `T6` | P1 | DONE | External-evidence relations bypassed abstract capability validation during proof planning. |
| `T7` | P1 | DONE | Representation-relation proof choices were not constrained by the selected representation mode. |
| `T8` | P2 | DONE | Current-phase declarations disagreed and the plan checker did not detect all copies. |
| `T9` | P2 | DONE | Compiler status documentation materially understated implemented internal analysis. |

All four findings were reproduced or directly corrected, repaired, and
verified in the Guide-4 batch; the gate record is in §2.8. Their evidence
records follow. Implementation detail beyond these records belongs to Git
history. The later two-pass review recorded in §2.1 opened the SR2 findings
in §5.2.

### 5.2 Closed review registers

The finding registers of the second, third, fourth, and fifth static
reviews stood here as §5.2–§5.5. Every row was remediated and closed, in
the Guide-7, Guide-8, Guide-9, and Guide-10 preflight waves
respectively. They are archived verbatim, with their original numbering
and labels, in [backlog history](history/backlog-history.md), so a
reference to §5.3 below resolves to the heading of that number there.


### 5.6 Sixth-review findings

The sixth static review’s finding register stood here. Every row was
remediated and closed in the Guide-11 preflight and declassification
waves. It is archived verbatim, with its original numbering and its
label, in [backlog history](history/backlog-history.md), so a reference
to §5.6 below resolves to the heading of that number there.


### 5.7 Confidential-value target review · `tab:backlog:findings-ct-review`

The Guide-11 §5 target review read the confidential-value machinery at
merged tip `78499c2`, upstream base `b7fc5d0`, and landed its accepted
facts as typed Rust in `packages/target-elements/src/confidential.rs`
with human provenance in (`tab:elements-ref:review-ct`). The twenty
questions the guide sets are all answered from source.

The review's own numbering note: the preflight arc in §5.6 already
spends the label "Wave 5" on the row that closed `G11-H01`. This record
belongs to the guide's §5 execution, a different arc, and is filed by
content rather than by wave number.

Three dispositions come out of it, and one is a negative result that the
guide asks not to be softened.

| ID | Priority | Status | Finding |
|---|---:|---|---|
| `G11-C01` | P1 | DONE | The confidential encodings record whether the y coordinate is a quadratic residue, while the curve-checking primitives accept only the compressed public-key prefixes, which record whether y is odd. The two conventions select different points, so no pattern may be carried from one to the other by analogy. Typed as `PointParityConvention`. |
| `G11-C02` | P1 | DONE | The commitment relation is the conceptual one with both terms positive: the blinder multiplies the base point and the amount multiplies the asset generator. The opening scalar is thirty-two bytes big-endian, refused at or above the group order, admitted at zero. Typed as `CommitmentRelation`. |
| `G11-C03` | P0 | OPEN | An authenticated public opening has no complete on-script form under the reviewed revision. Three independent blockers are named and typed as `OpeningBlocker`; the capability stays `Unsupported` for that reason. This is the §11 disposition input and it is a review result, not a scope decision. |

`G11-C03` is the row every later declassification candidate must be
judged against. The blockers are that the asset generator cannot be
derived on-script, that the confidential and curve-primitive encoding
domains do not meet, and that a witness-supplied parity byte is bound to
nothing, so a relation a program could verify holds for a point or its
negation. Candidate C in the guide names parity as load-bearing; this
review supplies the source reason.

Two questions are recorded as OPTIONAL follow-ups rather than as
findings, because neither changes a typed fact.

| ID | Priority | Status | Item |
|---|---:|---|---|
| `G11-C04` | OPTIONAL | OPEN | The capability contract calls commitment equality `Unsupported`, which is exact for a primitive: no reviewed instruction compares two commitments. Byte equality over the exposed canonical items is nonetheless a composition a backend might build, and the guide's disposition vocabulary has a name for that state which the typed enum does not carry. Deciding whether `ConfidentialCapabilityState` should gain the backend-pattern and named-blocker states belongs to a wave that owns the contract shape, since the states are part of the versioned schema. |
| `G11-C05` | OPTIONAL | OPEN | Value introspection presents an absent value as an explicit zero, prefix `0x01` over eight zero bytes, so a program cannot distinguish an absent field from a zero one at that primitive. No current pattern depends on the difference. Recorded so that one which does is not written in the belief that the distinction survives. |

The typed facts are deliberately not wired into `TargetDefinition`.
Growing that schema is a versioned change under
(`rule:guide11-exec:target-version-honesty`), and this review states
facts rather than revising a contract shape; the contract version did
not move.

### 5.8 Dependency decision and commitment oracle (archived)

The Guide-11 dependency decision and the independent commitment
oracle record are archived verbatim, with their labels, in [backlog
history](history/backlog-history.md); references resolve there. The
decision to add nothing was later revised by the user's 2026-08-20
ruling recorded in section 2.7, which adopted the reference crates
as testing oracles.

### 5.9 Seventh-review preflight findings

The seventh static review’s preflight register stood here. Its sixteen
rows are the Guide-12 preflight identifiers `G12-R01` through
`G12-R16`; fifteen are CONFIRMED and DONE and `G12-R08` is
RECLASSIFIED and DONE, so the blocking rule
(`gate:guide12-exec:preflight`) is satisfied and no confirmed P0 or P1
row remains open. Wave 0 adjudicated and reproduced, and Waves 1a, 1b,
and 2 repaired: each row’s witness was written to assert its defect and
was flipped by the wave that fixed it. The register is archived
verbatim, with its numbering and its label, in [backlog
history](history/backlog-history.md), so a reference to §5.9 or to its
label resolves there. Guide-12 preflight dispositions themselves are
owned by the guide’s own table.

### Guide-11 execution wave records (archived)

The three Guide-11 execution wave records — CT fixtures and the
conservation matrix, candidate dispositions and normalization, and the
fresh-process lifecycle — are archived verbatim, with their numbering
and their labels, in [backlog history](history/backlog-history.md);
a reference to any of their task labels resolves there. All three
closed DONE at the Guide-11 gate (section 2.4).

### T6 — Validate capabilities for external-evidence obligations · `task:review:external-evidence-capability`

**Priority:** P1
**Status:** DONE

Reproduction: with every capability available except whole-transaction value
conservation, production planning returned feasible plans for both pilots
(two compact-ASH and four live-transfer candidates) that carried the
substrate-conservation evidence requirement while omitting the conservation
capability entirely.

Repair: the external-evidence obligation class now retains its approved proof
alternative, required capabilities, and source requirements. Classification
validates that the realization approves exactly one substrate-conservation
alternative, with a typed error for any other shape. Enumeration tests the
fixed capability set against the capability view before the search begins and
seeds every candidate from the fixed requirements. The evidence requirement
remains unresolved, and no sponsor amount enters any fixed requirement.

Focused tests, in the compiler proof suite:

- `a_missing_whole_transaction_capability_blocks_substrate_conservation`
- `adding_the_whole_transaction_capability_restores_feasibility`
- `external_evidence_carries_its_capability_and_source_into_every_candidate`
- `fixed_external_requirements_name_no_sponsor_amount`
- `an_unexpected_external_evidence_alternative_is_rejected`

### T7 — Constrain every representation-sensitive proof choice · `task:review:representation-proof-compatibility`

**Priority:** P1
**Status:** DONE

Reproduction: unconstrained live-transfer planning returned four candidates,
two of which paired a selected mode with a contradictory arithmetic proof on
the representation relation itself (private committed with public arithmetic,
and explicit with confidential conservation).

Repair: representation relations are declared relation-only in both pilot
declarations, with the approved mode sets unchanged, and the compiler
classifies them as statically validated alongside lifecycle exits. The
conservation relation keeps exact proof/mode compatibility. Live transfer now
yields exactly one candidate per approved mode, and the selected mode remains
in each candidate's representations for later placement and backend work.

Focused tests, in the compiler proof suite:

- `representation_relations_carry_no_proof_variable`
- `live_transfer_conservation_agrees_with_every_selected_mode`
- `compact_ash_keeps_both_modes_under_public_arithmetic`
- `every_pilot_relation_is_classified_exactly_once`, extended

The independent oracle no longer calls production obligation classification.
It derives the substrate-conservation capability and its typed external
source row itself, creates no proof variable for representation relations,
and validates fixed requirements before enumeration. New oracle regressions
cover the missing whole-transaction capability across all scopes, one
candidate per live mode, and random capability views over all thirteen
capabilities, while still distinguishing infeasibility from unexpected
errors.

### T8 — Weld every current-phase declaration · `task:review:phase-declaration-drift`

**Priority:** P2
**Status:** DONE

Repair: the plans README declares Phase 2 in the fixed declaration form. The
plan checker now compares the backlog current gate, the roadmap current
phase, the plans README current phase, and exactly one active numbered phase
card, normalized to the numeric phase. Diagnostics are order-stable and name
the stale file, the observed phase, and the expected phase; malformed or
missing declarations fail explicitly.

Focused tests: ten phase-weld tests in the labels plans module covering
agreement, each stale copy, zero and two active cards, malformed and missing
declarations, a gate without a phase card, and traversal-order independence.

### T9 — Refresh compiler status documentation · `task:review:compiler-status-drift`

**Priority:** P2
**Status:** DONE

Repair: the compiler crate Rustdoc lists the implemented internal stages and
the remaining absent work, and keeps the boundary statement that no public
value can be mistaken for a completed analysis. The compiler package
contract and the packages index carry an accurate status and milestone
states, and the phase card separates delivered-analysis requirements from
the open placement, coverage, and pilot deliverables. The stale-phrase sweep
found two current-state matches; both were corrected.

Verification: compiler package tests, a warning-free Rustdoc build, and the
plans check all passed.

---

## 6. Phase-2 implementation queue · `sec:backlog:phase2`

### 6.1 Summary · `tab:backlog:phase2`

| ID | Priority | Status | Deliverable |
|---|---:|---|---|
| `P2-001` | P1 | DONE | Immutable, canonical, ownership-validated realization boundary |
| `P2-002` | P2 | DONE | Petgraph dependency and lockfile review |
| `P2-003` | P1 | DONE | Compiler crate boundary and typed error root |
| `P2-004` | P1 | DONE | Bind architecture, realization, policy, and explicit scope |
| `P2-005` | P1 | DONE | Canonical compiler relation and expression DAGs |
| `P2-006` | P1 | DONE | Checked constant folding |
| `P2-007` | P1 | DONE | Exact proof planning; reaccepted after T6 and T7 closed, gate record §2.8 |
| `P2-008` | P1 | DONE | Disclosure, source, and constructibility analysis |
| `P2-009` | P1 | DONE | Representation lifecycle analysis |
| `P2-010` | P1 | DONE | Execution-case placement and layout requirements; gate record §2.9 |
| `P2-011` | P1 | DONE | Relation-indexed coverage requirements; gate record §2.10 |
| `P2-012` | P1 | DONE | Compact-ASH and live-transfer analyzed pilots; gate record §2.11 |
| `P2-013` | Gate | DONE | Complete Phase-2 evidence and exit; gate record §2.11 |

### 6.2 Completed foundation

The following are implemented internally and remain subject to their package
tests and owner validation:

```text
P2-001  realization boundary
P2-002  Petgraph review
P2-003  compiler crate and error root
P2-004  compiler input binding
P2-005  relation/expression DAGs
P2-006  checked constant folding
P2-007  exact proof planning, reaccepted
P2-008  source, constructibility, and disclosure analysis
P2-009  representation lifecycle
P2-010  execution-case placement and layout requirements
P2-011  relation-indexed coverage requirements
```

No complete analyzed-program public API is exposed yet. This is deliberate:
partial analyses must not be mistaken for a completed compiler result.

### P2-007 — Repair and reaccept exact proof planning · `task:phase2:proof-planning`

**Priority:** P1
**Status:** DONE
**Depends on:** T6, T7, P2-005, P2-006
**Blocks:** P2-010 through P2-013

Reaccepted after T6 and T7 closed; the gate record is in §2.8.

Reacceptance required, and the recorded evidence establishes:

1. every in-scope relation classified exactly once;
2. every approved proof alternative retained or rejected for a typed reason;
3. external-evidence capabilities validated;
4. proof/representation compatibility applied completely;
5. capability and source requirements consistent;
6. hard constraints applied before objective or candidate retention;
7. exact feasible set compared with an independently derived exhaustive oracle;
8. no partial result on complexity exhaustion;
9. deterministic candidate ordering and equality under declaration
   permutations.

The search retains the exact feasible set; it does not silently select one
candidate under an unstated target policy.

### P2-010 — Derive execution-case placement and layout requirements · `task:phase2:placement`

**Priority:** P1
**Status:** DONE
**Depends on:** accepted P2-007, P2-008, P2-009, C1-009
**Blocks:** P2-011 and P2-012

Delivered in the Guide-5 batch; the gate record and evidence are in §2.9.

Relations are classified per execution case on independent axes — discharge
boundary, semantic scope, activation, and carrier multiplicity — rather than
one local/global/conditional/duplicated category. Execution cases expand only
the sponsor dimension; the representation is fixed by each feasible proof
plan. Every active runtime relation case has an eligible abstract carrier;
carrier eligibility, exact feasible placement sets, and target-independent
layout requirements are validated against an independent exhaustive oracle.
Complexity limits are explicit typed parameters and exhaustion returns no
partial result. No target bytes or concrete target positions enter this
stage.

### P2-011 — Derive relation-indexed coverage requirements · `task:phase2:coverage`

**Priority:** P1
**Status:** DONE
**Depends on:** P2-010, C1-010, C1-013
**Blocks:** P2-012

Delivered in the Guide-6 batch; the gate record and evidence are in §2.10.

Exact equality holds among the realization relation scope, the compiler
relation scope, the relation-case plan scope, and the coverage scope. Every
active relation-case carries positive and negative requirements at each of
its boundaries, a carrier requirement when runtime-carried, and an accepted
semantic-projection requirement; inactive relation-cases carry an explicit
inactive-valid requirement; conditional relations satisfy the
inactive-valid, active-valid, active-invalid triplet across the case set,
and representation modes are covered across the complete feasible plan
set. A broad operation test does not substitute for relation coverage.

### P2-012 — Analyze both pilots end to end · `task:phase2:pilots`

**Priority:** P1
**Status:** DONE
**Depends on:** T6, T7, P2-005 through P2-011
**Blocks:** P2-013

Delivered in the Guide-7 batch; the gate record and evidence are in §2.11.

Both pilots analyze end to end into one validated scoped analyzed program
per scope: the acceptance relation lists are the exact relation censuses,
every requirement in the acceptance matrices holds, rejected proof and
representation pairings are absent from the plan set, repeated analysis
from equal typed inputs produces equal stable projections, and the
combined two-pilot scope stores per-operation factors whose product
equals the retired global enumeration exactly. The analyzed value claims
no deployment lifecycle completeness: the pilots retain their future
target and operation obligations explicitly.

### P2-013 — Phase-2 evidence and exit · `gate:backlog:phase2`

**Priority:** Gate
**Status:** DONE
**Depends on:** T6–T9, P2-004 through P2-012, C1-005, C1-008,
C1-009, C1-010, C1-013

Passed; the complete exit evidence is the Guide-7 gate record in §2.11,
including the declared-MSRV and current-stable lanes, the meson-required
CI run, the explicit product regressions, and passed document byte
reproducibility. Every exit condition held: findings closed, censuses
exact, capabilities failing closed, deterministic pilot analysis,
independent oracles agreeing, gates recorded, current-phase declarations
moving together, and a clean final tree.

Phase completion does not itself justify a persistent compiler digest;
none was minted.

---

## 7. Immediate algorithm preparation · `sec:backlog:algorithms`

### 7.1 Current status · `tab:backlog:algorithms`

| ID | Status | Deliverable |
|---|---|---|
| `C1-001` | DONE | Compiler/linker/mathematics/solver research notes |
| `C1-002` | DONE | Direct Petgraph decision |
| `C1-003` | DONE | Exact/certified mathematics decision |
| `C1-004` | DONE | Petgraph dependency review |
| `C1-005` | DONE | Canonical direct-Petgraph compiler graph prototype |
| `C1-006` | PARKED | Exact keyed linear systems until a consumer exists |
| `C1-007` | PARKED | Certified numerical analysis until a consumer exists |
| `C1-008` | DONE | Exact proof-plan search; reaccepted after T6/T7, gate record §2.8 |
| `C1-009` | DONE | Execution-case-aware placement; gate record §2.9 |
| `C1-010` | DONE | Typed symbol resolution and SCC policy in coverage scope; gate record §2.10 |
| `C1-011` | BLOCKED | Structured relocation; linker phase |
| `C1-012` | BLOCKED | Deterministic bounded-depth target tree; linker phase |
| `C1-013` | DONE | Independent small-instance oracles; placement oracle §2.9, coverage/collateral/SCC oracles §2.10 |
| `C1-014` | DONE | Preparation review and Phase-2 handoff; gate record §2.11 |

### 7.2 C1-008 — Proof-plan search reacceptance

Reaccepted in the Guide-4 batch, gate record §2.8. The recorded evidence
establishes each former condition:

- external-evidence capability requirements are retained;
- representation-sensitive proofs are checked against selected modes;
- candidate capabilities and sources agree;
- the independent oracle derives those rules independently;
- infeasibility and complexity exhaustion remain distinct;
- no partial result is returned.

### 7.3 C1-009 — Execution-case-aware placement

Delivered in the Guide-5 batch with P2-010; the gate record is in §2.9.
Every active relation case is covered by an eligible carrier, production
search is compared with an independent exhaustive assignment oracle on
small instances and both pilot scopes, and the adversarial case list —
optional-carrier defects, global-on-local defects, source unavailability,
duplicate enforcement, permutation equality, and both budget exhaustions —
is retained as focused regressions.

### 7.4 C1-010 — Typed symbols and SCC policy

Delivered in the Guide-6 batch within the coverage scope; the gate record is
in §2.10. Two-pass typed resolution runs a complete definition census before
complete reference resolution; duplicate keys, unknown references, and
cross-operation references fail; SCC members and diagnostics are normalized
by stable key and every current cycle is rejected. SCC membership does not
authorize a semantic cycle — an accepted cycle would require a new typed
resolution strategy and policy review. This work prepares linker policy but
pulled no linker or target types into compiler core.

### 7.5 C1-013 — Independent algorithm oracles

| Production analysis | Independent oracle |
|---|---|
| canonical topology | valid-order enumeration plus least-key rule |
| SCC | mutual-reachability equivalence |
| expression interning | non-interned evaluator |
| dependency closure | repeated complete scan |
| proof selection | exhaustive candidate enumeration with independently derived hard constraints |
| placement | exhaustive carrier subsets |
| relation census | direct set equality |
| constant folding | non-folded evaluator |
| stable projection | declaration-order permutation |
| coverage | direct relation × case matrix equality |

The proof-selection oracle must not call the same obligation-classification or
compatibility helper as production for the property it is meant to verify.

Delivered across the Guide-5 and Guide-6 batches: the placement oracle in
§2.9; the relation-by-case coverage oracle, the repeated-scan collateral
oracle, and the mutual-reachability SCC oracle in §2.10. Each restates its
hard predicates independently of the production helper it verifies.

---

## 8. Dependency posture · `sec:backlog:dependencies`

### 8.1 Current and deferred dependencies · `tab:backlog:dependencies`

| Dependency | Status | Role |
|---|---|---|
| `petgraph = 0.8.3` | Adopted and reviewed; `serde-1` only | Graph storage and standard algorithms |
| `num-bigint` | Existing | Exact arbitrary-size integers |
| `num-integer` | Existing | Exact integer helpers |
| `num-traits` | Existing | Numeric traits |
| `proptest` | Existing | Property and independent-oracle testing |
| `num-rational` | Deferred | Future exact-rational consumer |
| `faer` | Deferred | Future certified numerical diagnostics |
| `fixedbitset` | Deferred | Dense local coverage sets if measured |
| Elements libraries | Phase-3 review | Target transaction and consensus types |
| SAT/LP/MILP solver | Deferred | Larger exact planning only after measured need |
| `salsa` | Deferred | Incremental compiler queries |
| `egg` | Deferred | Equality saturation |
| `rayon` | Removed and deferred | Requires measured need and schedule-independent results |

The latest static review excluded `Cargo.lock`; no lockfile claim is made by
that review.

### 8.2 Dependency-entry rule · `rule:backlog:dependency-entry`

A deferred dependency enters only when:

1. a concrete consumer exists;
2. current implementation demonstrates missing functionality;
3. simpler exact first-party code is insufficient;
4. source, version, licence, MSRV, unsafe boundary, transitive graph,
   determinism, and advisories are reviewed;
5. public API leakage is considered;
6. focused tests and an independent oracle exist;
7. lockfile changes are reviewed;
8. required gates remain green and clean.

Unused dependencies are not added to advertise intent.

---

## 9. Mathematical and algorithmic laws · `sec:backlog:laws`

### 9.1 Exactness · `rule:backlog:exactness`

Semantic, conservation, authorization, identity, calibration, and release
claims use:

- checked bounded integers;
- arbitrary-precision integers;
- reduced exact rationals;
- exact finite search;
- independently checked certificates;
- target-native execution where the claim concerns the target.

A floating residual is diagnostic evidence, not exact equality.

### 9.2 Handles and identity · `rule:backlog:handles`

Always distinguish:

```text
local handle:
    process-local graph, arena, matrix, or solver position

stable key:
    complete typed semantic identity

digest:
    optional domain-separated commitment, admitted per the
    identity adjudication procedure
```

Never use as semantic identity:

- Petgraph indices;
- matrix positions;
- solver variable numbers;
- insertion or traversal order;
- source path or line;
- pivot order;
- floating-point bits;
- thread schedule;
- temporary path.

### 9.3 Complexity failure · `rule:backlog:complexity`

An analysis exceeding its explicit budget returns a typed complexity error.

It must not:

- drop a relation;
- weaken authorization;
- increase disclosure silently;
- remove a lifecycle exit;
- switch to an undocumented greedy fallback;
- claim optimality from incomplete search;
- accept the best partial result seen before exhaustion.

### 9.4 Evidence separation · `rule:backlog:evidence-separation`

Keep separate:

- typed validation;
- capability availability;
- witness/source availability;
- model execution;
- realization conformance;
- compiler completeness;
- backend pattern evidence;
- linked-bundle execution;
- target consensus/policy results;
- event projection;
- attestation query;
- receipt accounting;
- calibration;
- release validation.

A report or status from one class never silently satisfies another.

In particular:

```text
target capability exists
≠
required evidence was produced

sponsor role structure is valid
≠
substrate conservation was evidenced

model relation is conformant
≠
deployment is ready
```

---

## 10. Verification matrix · `sec:backlog:verification`

### 10.1 Working cadence

For Rust changes:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Do not run the full Meson/release surface after every small edit.

Documentation-only changes do not require the Rust lane, but they still require
the relevant documentation and census checks.

### 10.2 Focused commands · `tab:backlog:focused-tests`

| Area | Command |
|---|---|
| architecture/profile | `cargo test -p tripod-architecture` |
| realization | `cargo test -p tripod-realization` |
| compiler | `cargo test -p tripod-compiler` |
| compiler planning | `cargo test -p tripod-compiler proof` |
| compiler oracle | `cargo test -p tripod-compiler oracle` |
| model conformance | `cargo test -p tripod-model realization_conformance` |
| model bounds | `cargo test -p tripod-model bound_conformance` |
| artifacts | `cargo test -p tripod-artifacts` |
| labels/plans | `cargo test -p tripod-labels` |
| checker report/stamps | `cargo test -p cli-common` |
| document stamps | `cargo test -p tripod-document-stamps` |
| flattener | `cargo test -p flatten-latex-main` |
| process wrapper | `cargo test -p execwrap` |
| mocked Meson graph | `meson test -C build meson-mock-contract` |
| executor classification | `meson test -C build executor-classification` |

Focused filters supplement but never replace complete package and workspace
runs. Every gate lane is a Meson test (CI-009), so any of them narrows the
same way: `scripts/ci.sh --suite lint`, or a lane by name.

### 10.3 Full gate

After a completed batch:

```sh
scripts/ci.sh
meson compile -C build
meson test -C build --print-errorlogs
```

The first line is the whole lane surface with the TeX toolchain mocked; the
second and third add the real document build, which the mocked run cannot
cover. `scripts/ci.sh` is a shim over `meson test` and forwards its arguments,
and `scripts/ci-timing-report.py` reads the finished run's logs for the
per-suite, per-lane, and per-test timing.

The canonical build directory is `build/`.

Use:

```sh
meson setup build
```

only if `build/` does not exist.

For a complete release-oriented result, additionally run the required MSRV and
stable lanes and the separate byte-reproducibility check:

```sh
scripts/check-document-reproducibility.sh
```

### 10.4 Documentation and census

```sh
scripts/check-plans.sh
meson compile -C build lint
git diff --check
git diff --cached --check
```

Every new tracked subject must join its nearest `meson.build` census.

### 10.5 Dependency and advisory evidence

```sh
cargo tree -e features
cargo metadata
cargo audit
```

A missing `cargo-audit` is recorded as skipped, never passed.

### 10.6 Clean repository

The final check is:

```sh
git status --porcelain=v1 --untracked-files=all
```

It must be empty.

### 10.7 Execution trust

Repository source, tests, Cargo manifests, Meson definitions, scripts, TeX,
and `.latexmkrc` are executable.

Untrusted contributions run only in an externally established,
credential-free isolated environment under ADR-015. A clean-tree result is a
correctness check, not malicious-code containment.

---

## 11. Current gate · `gate:backlog:current`

The Phase-2 gate is **passed** (§2.11) and the Phase-3 gate is
**passed** (§2.6 with the Phase-3 card). The current gate is Phase 4 —
end-to-end compact ASH, chartered by Guide 12; its preflight register
is closed and its core waves are unblocked.

The Guide-8 target foundation is complete; the gate record is §2.12. The
Phase-3 task state is:

| ID | Status | Task |
|---|---|---|
| `T3-001` | DONE | Typed target package boundary |
| `T3-002` | DONE | Reviewed initial target definition |
| `T3-003` | DONE | Encoding and opcode contracts |
| `T3-004` | DONE | Development deployment binding |
| `T3-005` | DONE | Compiler capability adapter |
| `T3-006` | DONE | Target-native primitive conformance, gate record §2.13 |
| `T3-007` | DONE | Typed tapscript instruction foundation, gate record §2.13 |
| `T3-008` | DONE | STATE-constructor prototype accepted, gate record §2.14 |
| `T3-009` | DONE | Exact wide floor-arithmetic prototype accepted, gate record §2.14 |
| `T3-010` | DONE | Public-declassification initial policy selected, gate record §2.4 |

The Phase-4 task state follows Guide 12's wave plan, from
(`task:guide12-exec:wave4`) onward; each wave lands with its own
evidence and the batch gate record extends §2.6:

| ID | Status | Task |
|---|---|---|
| `T4-001` | DONE | Validated compact-ASH target operation plan — compiler `operation_plan` public boundary, projections recomputed against the analysis (23 relations, 2 cases, 46 relation-cases, 69 layout requirements), Guide-11 representation stated, nine corruption oracles |
| `T4-002` | DONE | Target transaction-form and dependency review — target-elements `transaction_form`, fee role recognized by form, both forms consensus-admitted, sponsor asset explicit and value uninspected, first-party substrate selected awaiting dependency ratification, three requirements deferred to transaction evidence |
| `T4-003` | DONE | Static target assessment and typed tapscript proof patterns — tapscript `shape`, `policy`, `pattern`, `operation_assessment`; `CompactAshShape` minted and the (`rule:guide12-exec:plan-selection`) tie-break stated in `CompactAshBackendPolicy`; eight pattern identities earned by machine-checked schedules; all 79 plan requirements assessed (74 complete-pattern, 3 structural, 2 external evidence) with emission admissible |
| `T4-004` | DONE | Candidate relocatable bundle — tapscript `bundle`; static ASH constructor over 9 shapes as 9 coordinator leaves and 3 recomputation-shared member leaves, 3939 exact encoded bytes, 61 relocations located by rebuilding each program with one symbol replaced plus 9 introspection references read from the emitted instructions, candidate status held by construction refusing a release-complete plan and by a read-only artifact status |
| `T4-005` | DONE | Linker foundation — `packages/linker`; two-pass resolution over 22 typed symbols of which 21 are defined, frozen reference graph whose one strongly connected component is the ASH constructor's self-commitment, 61 relocations applied by structured substitution to 3975 exact encoded bytes, deterministic 12-leaf taptree checked against an exact subset oracle that is itself checked against literal enumeration, `ControlPathDepth` settled at 4, all 30 relation-cases uniquely carried; the demonstration link refused without an explicit authenticated strategy for the self-commitment, which is the finding this wave was for |
| `T4-005a` | DONE | Self-commitment resolved by identity introspection — the recognition fragments read the constructor's program off the input the leaf is spending instead of pushing a literal for it, which is the resolution the Wave-8 refusal named; `AshConstructorProgramVersion` retired, `AshConstructorProgram` rebound `ReadFromTargetAtSpendTime` and carried by an introspection-reference census, the sound strategy now links with no equality outstanding, and every emitted program shrank (coordinator model base 116 to 85 and per-ash-input 109 to 78, member 102 to 64) |
| `T4-006` | DONE | Candidate transaction ABI — `packages/transaction`; the ABI states every item Guide-12 section 15.4 lists over the 9 admitted shapes, with a first-party explicit-field encoder and decoder checked against hand-written byte strings (193 sponsorless, 374 sponsored) and control blocks over a committed tree checked against digests computed outside the crate; the three whole-transaction dimensions settle per constructed transaction; the taproot output key is pinned rather than recomputed and the pin-against-tree equality is carried as an outstanding obligation |
| `T4-007` | DONE | Canonical semantic and target fixtures — `packages/vectors`; the complete Guide-12 section-18 matrix transcribed as 153 named classes each naming its polarity, mutation layer, and expected section-1.5 boundary; canonical and experimental standing separated by a type whose canonical constructor is crate-private and has no promotion path; the section-16.4 evidence plan recomputing every census two ways and comparing them (23 relations, 2 cases, 46 relation-cases of which 4 vacuous, 211 coverage requirements split 139 positive and 72 negative), the section-1.3 relation closure checked in both directions; 14 positive semantic cases carrying no target vocabulary, with the successor derived from the realization layer's own checked sum, and 9 of them materialized to exact byte-stable target transactions; no coverage row discharged, every row naming why it is outstanding |
| `T4-008` | DONE | Real target execution (Waves 11 and 12); the full narratives live in the merge commits for those waves and compress here to the evidence. Built: the section 16.2 edge taken end to end — `CompactAshOperationPlanner` maps the thirteen obligations onto operation steps, materialization takes outpoints from funding answers, the adapter funds and submits for real, and `CoverageObservation` gained its observed arm whose discharge requires acceptance and a matched projection both. `bundle::ceremony_bundle` links a second bundle at the asset a ceremony issues, because an Elements asset identifier derives from the issuing outpoint and the canonical fixture bundle is therefore unfundable on any real chain; `ExecutorCapability` gained the `FreshProcessLifecycle` variant the adapter had advertised since Guide 11, and Wave 12 added `ExecutorCapability::TestSponsorAuthorization` — one capability whose sponsor-funding and sponsor-signing halves are inseparable, the adapter authorizing with a published constant key under an RFC 6979 nonce (ADR-015 rule test-material), the protocol revision unmoved because the added records are capability-gated per the Guide-10 schema-migration rule and the added response members are defaulted, with a parse-compatibility test. The run of record, elementsregtest at pinned genesis: the ceremony issues its asset, records the money-bound row as a divergence, funds and submits all twelve remaining vectors — eight sponsorless and four sponsored — every one accepted and mined, and every one matched on all thirteen section 17.4 terms, the observed side read from accepted bytes and `gettxout` rather than echoed. Coverage discharged by the run's own outcomes: 99 of 211 (every positive target-execution row active in either case). Three runs, byte-identical transcripts, both waves. Defects found on the way and fixed: the fee-role digest was an invention — a fee output carries no witness program, so its digest is the SHA-256 of the empty script and the linked constant was wrong; computing it moved the pin at even parity and the conformance cross-check caught the drift as designed. Also fixed: step-kind spelling divergence, sponsor change paid to the wrong program form, authorization written as hex where the boundary uses octets. Structural: the two-sponsor row is unbuilt in this candidate (own census column, the target never asked); a sponsored row is executable without being a byte fixture, so `is_materializable` split into its two questions |
| `T4-012` | DONE | Issuance refused `bad-txns-in-ne-out` with no first-party code involved, and the cause was not an amount: Elements checks an issuance only when the transaction carries a witness section — the guard ahead of the issuing input's `VerifyIssuanceAmount` call in `src/confidential_validation.cpp` returns false and `Consensus::CheckTxInputs` reports that as a balance failure — so the issued asset weighs an empty input side against the whole issuance and the explicit amounts are never consulted. An empty witness section cannot be encoded (`Superfluous witness record`, the earlier hypothesis's misread disproof), and explicit issuances and outputs may carry no proofs, so the only fillable field is a spending input's witness stack, which a bare anyone-can-spend coin lacks. Seven-arm live bisect with amounts held fixed: forcing a witness section on the refused bytes flips the answer to `bad-witness-nonstandard`; a witness-script-hash coin spent with its committed program on the stack is accepted and mined; a blinded funding coin is a real second effect but not this one. Fixed first-party: the free-coin split pays the working slice to the witness-carrying form of the program and the issuing input names it on the stack, no key or signature (ADR-015 rule test-material). The ceremony then issues live and stops at `T4-013`. Gripe material for the Elements register: a balance diagnostic for a non-balance condition, and `rawissueasset` returns a transaction that can never be accepted from the coin it was handed, acknowledged only in a source comment |
| `T4-013` | DONE | Kept and reclassified,. The `maximum` fixture is unchanged at the protocol's own 51-bit bound and no target constant entered any semantic fixture; the plan and target layers learned to carry a row this target's money bound forbids. `target-elements::transaction_form` records the reviewed stated-amount bound, sourced to `MAX_MONEY` and the explicit-output loop of `CheckTransaction`, with the guard test recomputing the bound from its two factors; `vectors::divergence` derives each row's standing by comparing every amount the row makes the target state against that bound, proven by a test that narrows the bound and watches an ordinary row become divergent; the planner schedules a divergent row's one forbidden input and nothing else, records an `ObservedDivergence` carrying stated amount, bound, excess and the target's verbatim layer, and refuses the whole plan if the forbidden amount is ever accepted, because that would falsify the reviewed bound. `RequiredTargetWork::TargetAmountDivergence` keeps the thirteen work items summing: 9 materialized as 8 submittable plus 1 divergent, and the divergent row reaches no coverage row because nothing is submitted for it. Confirmed live: refused with excess while the other eight rows funded and submitted in the same run. Honest qualification: the observed refusal is the adapter's own reserve arithmetic and the transcript says so in a `target_verdict` field — that the target itself answers `bad-txns-vout-toolarge` rests on the reviewed source and its guard test, not on an observation |
| `T4-009` | TODO | Negative relation coverage (Wave 13). Open: none of the 72 negative requirements is discharged, and the wave says so rather than rounding up. What landed is the machinery and one finding. The 72 are classified by where a refusal could be observed, derived from each requirement's own evidence role rather than marked by hand: 48 target-executable, 18 first-party (10 compiler-analysis and 8 emitted-structure), 6 unreachable because their evidence is an external report nobody has written. The three columns are recomputed in `PlanCensus` and checked against the negative total, and a role naming no column is refused rather than filed under the nearest one. `vectors::mutation` builds a negative vector the only way a safe constructor allows — by changing one thing about a transaction the target accepted — and opens with the gate rather than the mutations: a vector that does not decode and re-encode to its own bytes is refused, because surgery on bytes that normalize would add a second difference and the refusal would be attributable to neither. All nine materialized vectors pass it. Eight arms each name the section-18 class they stage and take the expected boundary from the matrix by lookup, so a class that changes its mind takes the module with it; tests assert per arm that everything it does not claim to touch compares equal, and that value balance moves only where the arm says it does. That last property is load-bearing: Elements checks per-asset conservation before it runs a script, so an arm that unbalances the closed asset cannot reach a script-path expectation whatever the covenant would have said. One defect was found by the first live run and fixed. Mutations were staged from vectors already submitted, so the target refused them for spending coins it had seen spent — seven read `missing-inputs`, one read `txn-already-known`, and the witness-reorder arm had not even changed the txid. Eight results, none attributable to the mutation it was about. The order is now inverted: one vector is held back, its mutations are offered while its coins are unspent, and the un-mutated subject follows as the control, so a refused mutation and an accepted control differ by exactly the mutation. The ordering is itself a test, and attributability is a per-row order-dependent reading rather than one flag, because a mutation the target accepts spends the subject and invalidates only what comes after it. The run of record, on the shared instance against elementsregtest at pinned genesis, issued asset, subject the four-input row: four mutations refused at exactly the boundary their class names, each naming the operation that failed — two-ash-outputs and ordinary-wallet-u-output and shorten-successor-and-grow-another-output on OP_EQUALVERIFY, noncanonical-ash-ordering on OP_VERIFY. The finding is the fifth. **`wrong-sequence` was ACCEPTED**: the class expects a script-path refusal and the target took the transaction, so either the covenant does not constrain the sequence field or the class's boundary claim is wrong. Wave 13b settled which, and it was the class. No emitted program reads a sequence — `InspectInputSequence` sits in the opcode vocabulary and the capability map and at no emission site in tapscript's pattern module — and an ASH input's witness is the leaf program and its control block, so no signature covers the field either. The ABI has typed `SequenceConstraint` as a convention, in those words, since it was derived, and section 18.11 asks only that a wrong-sequence case exist, naming no boundary for it; the boundary came from the matrix, beside a transaction-version row the target really does answer, and the resemblance did not hold. The row moves to `AbiConstructionRejection`, beside the two other rows whose mutation only first-party code catches, and what actually pins the field — `construct` writing the ABI's sequence on every input, against a request carrying no field to argue with it — is now tested rather than assumed. The run also withholds any arm whose boundary precedes the target: submitting one buys an acceptance, and an acceptance spends the subject and abandons every arm behind it. **The withheld arm exposed the next one: `wrong-transaction-version` was ACCEPTED.** Same defect: `InspectVersion` has no emission site either, no signature covers the sponsorless form, and the ABI itself records that consensus admits that form at either version — so `ConsensusRejectionBeforeScript` cannot be right. Relay policy is ruled out too, because the vectors pay a thousand satoshis and the chain runs at zero min-relay fee, so no floor refuses the flip from the topology-restricted version to the standard one. It is reported and not reclassified: a second class respec deserves its own evidence, and its correction is the next bite. Its acceptance spent the subject in turn, so the last two arms are again recorded as not submitted with the reason, and re-testing them needs either per-mutation funding or a classify-without-broadcast capability. Three runs each wave, byte-identical transcripts. Nothing is discharged because the link from a section-18 class to a relation-indexed coverage requirement is not established, and inventing it to move 4 rows would be the discharge-by-intent the plan exists to prevent. Wave 13c respecified the row Wave 13b reported and left for its own bite: `wrong-transaction-version` moves to `AbiConstructionRejection` too. `InspectVersion` has no emission site either, the ABI's own type doc records the topology-restricted version as a policy choice rather than a consensus requirement — consensus admits the sponsorless form at either version — and the relay floor stays moot for the reason it always was. The generalized pre-target withholding needed no change to cover it. Review also found a second defect: `MutantOutcome` dropped an accepted mutation's txid, though the adapter reports one at the same submission path a positive row uses; the field now mirrors `SubmissionOutcome`'s, and the report writer, which had silently dropped it too, now emits it beside `detail`. **With both boundaries withheld, the control was finally observed ACCEPTED**: ordinal 2, txid, projection matched. Nothing spent the subject early, so both remaining arms reached the target for the first time — `witness-item-reorder` and `successor-one-below-the-sum` — and matched their `ScriptPathRejection` boundary, joining the four already seen. All eight arms are now answered: two withheld, six matched, none accepted. The txid field is exercised by test and by structure; no mutation was accepted in this run, so it reads null throughout the live transcript too. Three runs, byte-identical transcripts. Wave 13d establishes the link and reports what it is worth, which is one row. Section 19.2 requires a negative case to name its intended violated relation before it runs, and nothing did; that declaration was the missing half. An arm now declares the relation and the semantic mutation class its change falls in, and the requirement is resolved against the published plan where it must hit exactly one row — a table from arms to requirement identities would have been a third source of truth agreeing with neither side. Section 18's tables are lists of names, and no row there names a relation, a mutation class, or a boundary, so only two arms are determined at all, each by its own class name read in the relation vocabulary: `two-ash-outputs` to the ASH-output cardinality relation above its declared maximum of one, and `successor-one-below-the-sum` to the closed asset's conservation relation at an amount mismatch. Both resolve to exactly one published requirement per case. The other six carry typed reasons rather than guesses: `noncanonical-ash-ordering` and `witness-item-reorder` have no member in the negative mutation vocabulary at all, a gap between two authorities rather than a defect in either; `ordinary-wallet-u-output` and `shorten-successor-and-grow-another-output` each fit two published classes equally well; `wrong-sequence` and `wrong-transaction-version` expect a refusal before any target sees the bytes, and no requirement is indexed at the constructor's boundary, so reaching them needs the ABI-validation entry point section 16.5 names as its own report role and which does not exist. Discharge further requires the intended carrier to have executed, and the covenant script is that carrier, so only a script-path refusal counts: a transaction the target threw out before running any script established that it was invalid and not which relation refused it. **The live run found a third boundary defect and an adapter defect underneath it.** `successor-one-below-the-sum` was reported as a script-path rejection carrying the detail `bad-txns-in-ne-out`, a conservation failure attributed to an opening script that never ran. The submission path had asked a block to classify a refusal the mempool already described precisely, and block validation runs the amount checks in the same queue as the script checks, so an imbalance arrives there wearing a mandatory-script error. The conservation judgement documents that exact trap and records an earlier revision of itself committing it; the submission path reached it by the other door. The mempool's own non-script reason now settles the layer, the block's text is read only when the mempool named nothing to contradict, and a regression test drives the classifier directly. With that fixed the arm reads consensus rejection before script, which its own module already predicted: the arm does not preserve value balance, and Elements checks per-asset conservation before it runs a script. So the class's script-path claim cannot be right either — the third row of the family after `wrong-sequence` and `wrong-transaction-version`, reported here and left its own bite as those were. Six arms refused at the script path and exactly one of them names a relation, so exactly one negative row moves: ASH-output cardinality above maximum, sponsorless case; its sponsored twin waits on a mutated sponsored subject. **Coverage is 100 of 211, being 99 positive and 1 negative**, with 71 negative rows outstanding and each naming why in the code rather than in prose. The 18 first-party rows got the pass Wave 13's grep was not. None is discharged, and the guide is the reason before the evidence is: section 19.1 says positive coverage of a compiler-static or backend-structural relation uses typed structural evidence instead of inventing target execution, and 19.2 states no such rule for the negative half — its conditions are a complete mutated target transaction, an executed carrier and an observed target rejection, none of which a compiler-static relation can have. A boundary being first-party is not the same claim as a first-party test discharging the row. The evidence is thin independently of that: the constructibility witness and permissionless private dependency classes have a live typed refusal that no compiler-level test drives, every test asserting either driving the realization twin at another boundary; the required lifecycle exit is refused only at another layer on the compiler-static side and not at all on the emitted side, where the layout stage is a declared no-op; an unsupported representation is made unrepresentable rather than refused; an unauthenticated representation and an unexpected protocol secret have requirement types and predicates but no error at all. Three Wave-13d runs, byte-identical transcripts. Per-mutation funding is not built and is the next bite: until it lands the eight arms still share one subject's coins and no sponsored subject can be mutated. Wave 13e lands per-mutation funding: the subject earns one extra replica per submittable arm, funded and materialized independently, retiring the abandon-on-earlier-acceptance path so every arm answers from its own coins. It also respecifies successor-one-below-the-sum a third time, from ScriptPathRejection to ConsensusRejectionBeforeScript, alongside wrong-sequence and wrong-transaction-version: the arm's own module already records it does not preserve value balance, and Elements checks conservation before any script runs, so the arm answers its class without discharging the conservation row, which stays outstanding. Three runs, byte-identical: all six submittable arms answer independently, five still match ScriptPathRejection and successor-one-below-the-sum now matches ConsensusRejectionBeforeScript, two pre-target arms withheld as before, control accepted. Coverage is unchanged at 100 of 211 |
| `T4-010` | DONE | Candidate resource study (Wave 14). `vectors::resource_study` enumerates the section 20.2 product of six ASH and six sponsor research bounds and measures all thirty-six, recomputing from programs it emitted rather than quoting Wave 8. The unrolling became one authored source, so study and pipeline compare sets one loop produced, and a test ties the basis to the pipeline's: emitting at the resolved symbols reproduces the linked bundle's total script bytes, leaf count, and both role byte models exactly. All thirty-six emit; the rendered matrix is the study's table, recomputed on every run rather than transcribed here. Operation cost and validation budget are zero throughout, both reviewed facts rather than holes: no reviewed primitive charges operation cost, and a zero budget says this candidate emits no signature or curve primitive. **Linking, not weight or script size, is what bounds a candidate today.** The exact taptree oracle costs three-to-the-n set operations and refuses above sixteen leaves; leaves are the shape count plus the distinct batch sizes, so seven of thirty-six link and the linkable corner is an awkward shape rather than a prefix — which is what section 20.2 means by refusing monotonicity. Observed, not read off the constant: fourteen leaves link and the tree is counted, eighteen answer `TreeOracleBudgetExceeded`. Section 20.5 previously compared nothing: the ABI settled a weight the report never emitted, and the adapter answered every operation step with an all-null resource record. Weight is the one dimension where the comparison can be made and it is now made live. The adapter reports it from the node's own `decoderawtransaction` rather than computing it, writes it only on a target verdict since a non-verdict response may carry no observation, and a disagreement refuses the whole plan because both sides weighed the same bytes; unit tests stage agreement, disagreement by one unit, and an executor observing nothing, which stays an unmade comparison rather than a passing one. The run of record, same instance and pinned genesis, issued asset and submitted twelve: **all twelve weighed, all twelve matched**, at 1293 for two ASH inputs, 1736 for three, 2177 for four, 1861 and 1862 for the sponsored pair. Weight is measurably not affine in batch size, stepping 443 then 441, while the byte models are — so a weight predicted from a fitted model would have been wrong. The largest measured transaction uses 0.05 percent of the 4000000 consensus weight limit and 0.54 percent of the 400000 policy limit. Eleven of section 20.3's eighteen measures are answered; the other seven say why rather than reporting zero. The peaks and the maximum element are not observable through this adapter, because a validating node exposes no interpreter stack, no reviewed primitive touches the alternate stack, and the only static peak walk is private to the prototype lane; arithmetic and comparison counts have no charged dimension to carry them; control bytes and initial witness items are structural and simply not surfaced by this wave. No bound is selected and none calibrated, per section 20.6. Three runs, byte-identical transcripts |
| `T4-011` | TODO | Phase-4 gate and handoff (Wave 15). The exit audit is done; the gate is not, since the authoritative ci.sh, canonical meson, and document-reproducibility lanes stay the orchestrator's. Section 29 holds 105 items across eight sections, not the roughly eighty across seven an earlier reading assumed; each is dispositioned 94 PASS, 5 QUALIFIED, 1 FAIL, 5 DEFERRED-TO-GATE. The FAIL is that every negative relation reaches its owning evidence layer, which 100 of 211 cannot support, and the remedy named is the three guide gaps rather than more first-party tests. Fixed in lane, four stale facts: the vectors contract and package index claiming no target had executed anything, the Phase-4 card still opening at Wave 4, and the executor docstring saying revision 3 where its constant says 4. Record and filled section-30 report: [the Guide-12 completion record](history/guide-12-completion-report.md), archive-class, so the core tree gains 301 bytes |

Current blockers are:

```text
Phase-4 work:
    real target execution (T4-008) is closed. T4-012 and T4-013 are
    closed with it, and a live node has run the whole lane in both
    execution cases: the ceremony issues its asset, observes the chain's
    reserve, funds twelve rows, records the thirteenth as a money-bound
    divergence, and submits all twelve, which the target accepted and
    mined. The section 17.4 comparison ran against every acceptance and
    all twelve matched, discharging 99 of the 211 coverage requirements.
    The sponsor authorization is the executor's and is test-scoped by
    construction; what the run establishes is what the candidate owes.
    One row is left unanswered by this candidate rather than by this
    target: the two-member sponsor region is above the demonstration
    bounds, so no program for its shape was ever emitted, and reaching
    it needs a candidate linked under wider bounds
```

Standing rules that survive the phase exit:

- Phases 1 and 2 remain evidence about their recorded trees;
- no compiler public complete-analysis API is frozen;
- no public realization/compiler digest is minted without a real consumer;
- target-specific fields remain forbidden in realization and compiler core;
- no stable linker or transaction ABI exists;
- no floating value or local graph handle enters semantic identity;
- no draft bound becomes deployment calibration;
- no raw report digest is treated as evidence identity without typed role and
  subject binding;
- no self-consistent model checkpoint is described as independent
  target-chain evidence;
- no architecture, model, realization, hash, build, or test success is
  described as deployment readiness;
- no current checkout is described as green without a fresh complete execution
  record.

---

## 12. Execution order · `sec:backlog:order`

Execute in this order unless reproduction changes dependencies:

```text
1. Follow Guide 12's wave order for Phase 4; no later wave begins
   before Wave 3's recorded Phase-3 exit, which is §2.6, and no wave
   skips its predecessor's typed handoff.
2. Keep target types out of compiler core; the target packages consume
   the analyzed boundary, never the reverse.
3. Mint no target hash and claim no production activation without a real
   consumer and reviewed evidence.
```

No new hash, target prototype, report field, or publication may defer a current
typed-boundary or correctness repair.

---

## 13. Adopted-draft integration queue · `sec:backlog:drafts`

Four externally authored normative drafts were accepted 2026-08-16, audited (28 findings), corrected upstream by the author, and re-adopted and
archived verbatim under [plans/drafts/](drafts/README.md): the label
calculus, the environment-kind registry, the identity-adjudication
procedure, and the interchange conventions. Integration is chartered as
its own batch; the drafts bind nothing until the integrating changes
land. The user has ruled that a superseded ADR is deleted, not kept
marked superseded: retired records belong to Git history, not the
active tree.

### 13.1 Integration tasks

The draft-integration queue stood here and is closed: every DI row is
DONE or withdrawn, the queue was discharged by the draft-promotion
batch (gate record §2.5), and the table moved verbatim, with its
label, to [backlog history](history/backlog-history.md); §13.1
references resolve to the heading of that number there.

### 13.2 Checker findings so far · `tab:backlog:draft-findings`

A finding identifier is a permanent key. It names one finding for good,
so a resolved row stays here with its status rather than being deleted
into git history, and a resolved identifier is never reissued.

Two rows once shared `DI-F02`. The one still numbered so is the standing
finding the DI-001 gap census restates under that identifier; the other
was a narrower defect found while repairing it, which took the same
number by accident and now carries the next free one, `DI-F04`. Renaming
the standing finding instead would have moved an identifier the census
already cites (G11-R12).

| ID | Status | Finding |
|---|---|---|
| `DI-F01` | DONE | The plans-tree link scanner read bracketed patterns inside fenced blocks as Markdown links, so a CDDL regex in an archived draft failed as a broken link. Fenced interiors are now blanked before link scanning; the systematic single-scanner repair landed as DI-003 W1, which folded this blanking into the shared scanner. |
| `DI-F04` | DONE | The Realization harvest computed the boundaries of the generated upward-citation index from the raw source while reading its spans from the fence-aware scanner. A section heading displayed inside a fenced block therefore opened the index region, which stayed open across the fence close and swallowed the body citation below it, losing an anchor and reporting the genuine index stale. The region walk now reads participating lines only. Found and resolved by DI-003 W1; no occurrence existed in the tree, so no diagnostic moved. |
| `DI-F02` | DONE | Participation is enforced inconsistently across checks: the label scanner honors fences, the link scanner did not, and the scaffolding, placeholder, and confidence hygiene checks still scan fenced material. One participation model must feed every check. Closed by DI-003 W1, which landed exactly that single shared scanner. |
| `DI-F05` | OPTIONAL | Wave 4 observed that the DI-001 gap census and this table describe `DI-F01` through `DI-F03` in different terms, so a reader reconciling them must infer which wording is authoritative. The identifiers and statuses agree; the drift is semantic, in what each row is said to be about. Optional cleanup: state one wording as the finding and let the other cite it. No diagnostic and no gate depends on the difference. |
| `DI-F06` | OPEN | The label carrier stops at the Python adapter. `scripts/` is excluded from the census by `census-args.sh` and by the meson exclusion pattern, and ADR-019 fixes the scanned languages as Markdown, Rust, and LaTeX, so no citation can be written in `scripts/elements-native-executor.py`. The upstream-friction sweep made the cost visible: two frictions adapt only there — the mining path and the pegin-validation flag — and their rows say the register alone because a citation would be unreachable, not because nothing adapts. Closing it is an adoption change — a scanned-region row for Python comments, an owner for the script tree, and the census entries — and therefore a new record, not a checker patch. |
| `DI-F03` | DONE | The calculus's owner signatures, imported-citation prefixes, synthetic citations, anchor harvests, and acute-delimiter hard failure are only partially realized in the present checker; the gap census of DI-001 owns the exact delta. Closed by DI-003: W1 and W2 landed the owner signatures with registered prefixes, the imported and synthetic citations, the anchor harvests, and the kind registry as the checker's kind vocabulary; the acute-delimiter hard failure is enforced in `participation.rs` and asserted by `an_unclosed_opening_acute_fails_at_its_delimiter`. |

### 13.3 Toolchain engineering · `tab:backlog:toolchain-tasks`

| ID | Status | Task |
|---|---|---|
| `CI-001` | DONE (superseded by `CI-009`) | Rewrite the shell CI driver in Python with a typed lane tracker: every lane declared with status and skip reason, every lane and the whole run wall-timed, and a timing report emitted as the success output and on failure alike. Delivered as scripts/ci.py with an eleven-lane registry and the ci.sh shim; the first dataset shows the two test lanes at eighty-four percent of a twenty-minute gate. The dataset is what the row was for, and it holds; the driver that produced it does not. `CI-009` retired scripts/ci.py outright, because meson's harness already declares, times, and statuses every lane, and a second registry of the same lanes is a second thing that can disagree with the first. |
| `CI-002` | FOLDED into `CI-009` | Move test execution to the meson layer per the user's ruling: no workspace-level cargo test in the gate; each package's test groups run as individual meson-driven lanes, and per-test timing uses the nightly libtest JSON output, which the user has admitted as not affecting what the tests prove. The timing report gains per-package and per-test figures; attribution replaces the aggregate block. Contention on the shared cargo target directory is measured and the chosen serialization or partitioning recorded honestly. |
| `CI-003` | DONE | User ruling: the generated registers are archive-budget, by role not directory; labels/README.md stays prose. Combined 783755 to 724577 bytes; archive 1168518 to 1228130. |
| `CI-004` | DONE | Second half of the same ruling: closed backlog sections move to archived history. New archive-class directory plans/history/ holds the twelve completed gate records (§2.3–2.14) and the four remediated review registers (§5.2–5.5), moved verbatim with their labels; the backlog keeps a numbered stub at each origin. plans/README.md now separates superseded prose, which is deleted, from a closed record, which moves. Combined 786243 to 744305 bytes, headroom 189 to 42127. |
| `CI-005` | TODO | The workspace test lane costs too much for what most changes need (user, 2026-08-19). Two halves. First, honor the cadence the backlog already states: §10.1 exempts documentation-only changes from the Rust lane, yet the working ritual has been running the full workspace suite before every commit regardless of content — the promotion batch itself proved the corpus oracle for documentation edits is the label/plans/forbidden-text checker run plus register regeneration, with the workspace suite contributing nothing to those edits. The per-change cadence becomes content-scoped: documentation-only tranches run the checker oracle alone, Rust tranches run the focused package lanes of §10.2 plus clippy, and the full workspace lane runs once per batch before merge rather than per commit. Second, measure where the workspace wall time actually goes with the per-test timing CI-002 already charters, and split or serialize the worst suites so the full lane itself gets cheaper. Third, the user's deeper ruling: wall time is a first-class concept — every lane, test, and gate reports its duration or that silence is itself a defect to report; each recurring action carries an expected-duration budget beside its definition, and exceeding budget is a finding. |
| `CI-006` | DONE (sweep only) | Wave-1a worker finding, 2026-08-20: every workspace manifest named its lint groups in kebab case — nine names of the pattern `lints.rust.deprecated-safe` through `unsafe-code` per manifest — and cargo warned for each that the name will not work in a future edition, roughly ninety warnings on every build. The mechanical snake-case sweep is done: only the workspace root carried the nine keys (package manifests just inherit via `[lints] workspace = true`), the rename landed there, and a clippy lane confirms zero warnings of that class. Still open: one recorded decision, `PointParityConvention::ImpliedEvenY` documents the x-only convention but became formally unconstructible when `G12-R10` made the convention derive from the prefix pair, so either a future encoding earns it or the variant is removed. |
| `CI-007` | DONE (deletion) | Wave-1b worker findings, 2026-08-20, dispositioned by user ruling: the dead executor smoke script `scripts/test-elements-native-executor.sh` — unrunnable since revision 3 landed, hidden behind its live-node skip — is deleted outright rather than patched; rebuilding a native-executor smoke test is proper engineering and stands as the low-priority row `CI-010`. The second finding, no Python test lane anywhere in CI, is subsumed by the `CI-009` migration ruling: the lane arrives as a first-class meson test, not as a `ci.py` registry entry. |

| `CI-008` | DONE | Wave-2 worker finding and repair, 2026-08-20: the combined-Markdown headroom CI-004 restored was spent. The base tree measured 785909 bytes against the 786432 hard cap — 523 bytes — so the Wave-2 register rows could not be written at all without failing the plans lane, and a lane could no longer record its own evidence. Repaired by the mechanism CI-004 built, on the record that had just become eligible for it: §5.9's preflight register is complete, so it moved verbatim to history with a numbered stub at its origin, and the combined figure returned to about 762 kilobytes. Two things to watch rather than rediscover: the headroom is consumed by whichever wave is recording evidence, at roughly a kilobyte a row, and the next eligible record is not obvious — §5.7 still carries an OPEN P0 and §13 an ACTIVE row, so neither is movable yet. |
| `CI-009` | DONE | User ruling 2026-08-20: `ci.py` is dead weight — the CI driver migrates to the meson layer and test execution becomes first class there, completing what CI-002 started rather than growing the Python registry further. Deliverables: every `ci.py` lane restated as a meson test or target with its timing reported by meson's own harness; a first-class Python test lane for node-free executor logic, carrying the exhaustive script-error mapped-message check from the `G12-R07` repair; `ci.sh` reduced to a thin shim or retired; the wall-time budgets restated per lane where meson reports them. CI-002 folds into this row. Delivered: scripts/ci.py is deleted and scripts/ci.sh is a shim that configures the mocked build directory, runs `meson test`, and prints the timing report. Four lanes were not yet meson tests and are now — `tracked-path-argv-audit`, `cargo-audit`, `meson-mock-contract`, `clean-tree` — with the advisory lane exiting 77 so meson reports its own SKIP rather than a private vocabulary. The Python lane is `executor-classification`, an exhaustive node-free oracle over the executor's script-error classification that fails on the pre-`G12-R07` parser. Per-lane budgets sit beside each `test()` call in meson.build, cold and warm figures stated separately where they differ by more than a factor. Evidence: 47 of 47 lanes green through `scripts/ci.sh` under the nightly SDK toolchain, 2026-08-20 — 48:29 total on a cold target directory, of which 33:20 is the test phase and the rest the debug and release warm-up builds. The release suite is 30:30 of that test phase and the debug suite 15:45, so the two Rust suites remain the whole cost of the gate, which is the measurement CI-005 argues its cadence from. Per-test attribution survives the driver in scripts/ci-timing-report.py, which reads the finished run's logs and gates nothing. One explicit exception, stated rather than dropped: `scripts/check-document-reproducibility.sh` stays a script, because it builds the document twice in disposable directories and refuses a dirty worktree, so it answers a release question and cannot be a lane of the run whose cleanliness it depends on. |
| `CI-010` | LOW | User ruling 2026-08-20: rebuild the native-executor smoke test as proper engineering, replacing the deleted revision-2 script. It speaks protocol revision 4, exercises the real handshake against a live node, is validated in a session that holds one, and fails loudly rather than skipping silently when its preconditions are absent. Blocked on a live node like the rest of Wave 11. |

## 14. Backlog hygiene · `sec:backlog:hygiene`

### 13.1 Adding work · `rule:backlog:add`

A new task states:

- phase or lane;
- priority and status;
- dependencies;
- concrete output;
- assurance class;
- affected files and packages;
- focused exit test;
- verification command;
- identity and schema impact;
- dependency impact.

A new digest additionally satisfies
(`req:identity:admission-record`).

### 13.2 Splitting work · `rule:backlog:split`

Split a task when it:

- crosses semantic, compiler, linker, target, transaction, evidence, or release
  ownership;
- mixes exact correctness with numerical diagnostics;
- mixes dependency adoption with algorithm acceptance;
- combines independent security consequences;
- contains one part that can complete while another remains research-blocked.

### 13.3 Dropping or parking work · `rule:backlog:drop`

A dropped or parked task records:

- why it is unnecessary or premature;
- supporting evidence;
- activation condition;
- replacement, if any;
- identity and release consequences.

“No present consumer” is sufficient reason to park an identity, report digest,
dependency, or publication.

### 13.4 Retention · `rule:backlog:retention`

After a phase or remediation series:

- move durable evidence into the owning phase card, ADR, decision, research
  result, release record, or annotated tag;
- retain permanent task IDs in compact tables;
- delete implementation diaries from the active backlog;
- rely on Git history rather than creating a second archive under `plans/`;
- keep only current and immediately preparatory work detailed here.

---

## 15. One-line backlog · `rem:backlog:one-line`

> Check the Phase-3 exit gate now that all three foundational questions are answered — the STATE-constructor and wide-floor prototypes accepted in §2.14, the declassification policy selected in §2.4 — then begin the first complete compiler-to-target operation without reopening the policy, minting speculative identities, or claiming production activation.
