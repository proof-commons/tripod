# Guide-12 completion record — tree 0.4.1-dev

This is the Wave-15 exit record for Guide 12 and Phase 4: the §29 exit
checklist audited item by item, and the §30 completion report filled from
the standing evidence.

It is history, not authority, under
[the archive rule](README.md). It states what the tree named above
carried at the moment it was audited. It never overrides
[the backlog](../backlog.md), a package contract, a phase card, an ADR, or
an accepted decision. The authoritative gate verdict is the batch gate
record the orchestrator writes, not this document; the repository lanes
this audit defers are named as deferred below rather than reported as
passed.

Every disposition below is either recomputed here or cited to a record
that can be opened. Where Phase 4 did not produce what the template asks
for, the field says so instead of carrying an invented value.

## Index · `tab:guide12-completion:index`

| Part | Contents |
|---|---|
| [Exit checklist audit](#exit-checklist-audit) | All 105 §29 items, one disposition each |
| [Completion report](#completion-report) | The §30 template, filled |
| [Residuals](#residuals-and-inheritance) | What the next guide inherits |

## Counting note · `rem:guide12-completion:counting`

§29 carries eight sections and 105 checkbox items, not the seven
sections and roughly eighty an earlier reading assumed. The section
totals are Preflight 10, Compiler boundary 11, Target and patterns 14,
Relocatable and linked bundle 12, ABI and transactions 14, Evidence 16,
Resources 11, Repository 17. Every one of the 105 is dispositioned.

Four dispositions are used:

```text
PASS               located evidence discharges the item as written
QUALIFIED          the item holds only inside a stated boundary
FAIL               the item does not hold; a remedy is named
DEFERRED-TO-GATE   the orchestrator's authoritative lane owns the verdict
```

A QUALIFIED item is not a soft pass. It marks an item whose plain
reading overclaims what the evidence supports, and it states the
boundary that the evidence does support.

---

## Exit checklist audit

### Preflight — 10 items

| # | Item | Disposition | Evidence or boundary |
|---|---|---|---|
| 1 | R01–R16 reproduced and dispositioned | PASS | Backlog §5.9: sixteen rows, fifteen CONFIRMED and DONE, R08 RECLASSIFIED and DONE; register archived verbatim in backlog history |
| 2 | every confirmed P0/P1 finding closed | PASS | Same record; the blocking rule is recorded satisfied and no confirmed P0 or P1 row remains open |
| 3 | protocol revisioning coherent | PASS | Both sides declare revision 4 from one constant each: `NATIVE_PROTOCOL_SCHEMA` in the conformance package and the same name in the executor script; R09's two-schema defect is what revision 4 repaired |
| 4 | shipped commands comply with ADR-010 | PASS | R03 closed by moving every shipped binary onto `cli-common`, with subprocess-contract coverage under `packages/*/tests/subprocess_contract.rs` and `emit_subprocess_contract.rs` |
| 5 | target review facts correct | QUALIFIED | R10's nonce parity and R11's script bytes were corrected and are tested. The reviewed money bound gained a guard test in T4-013. The boundary: three §18 classes were found mis-specified during Phase 4 and respecified, so the review facts are correct as they now stand, not as they were written |
| 6 | exact resource accounting includes pushes | PASS | R11's repair derives script bytes from exact encoded length; the study asserts push bytes are a strict positive share of total script bytes |
| 7 | exact-literal abstract execution sound | PASS | R12 closed; T4-001 records nine corruption oracles over the analysis |
| 8 | executor and runner cleanup bounded | QUALIFIED | R15 discharged by a behavioural probe of the supervision module. The boundary: this is a first-party probe, not an observation of a killed live run |
| 9 | Guide-11 and Phase-3 status reconciled | PASS | R08's whole content; backlog §5.9 records it RECLASSIFIED and DONE |
| 10 | Phase-3 exit gate passes | PASS | Backlog §2.6: 13 of 13 CI lanes in 16 m 46 s, meson compile warm 2 s, meson suite 36 of 36 in 5 m 25 s; the Phase-3 card records the exit |

**Preflight residual, recorded rather than hidden.** R04's runtime half
is still marked blocked on a live node, in a source comment in
`packages/target-elements-conformance/src/tests/guide12_reproductions.rs`.
That comment was written before Waves 11 to 14, which ran the Python
executor against a live node many times. The transcripts that would
discharge it now exist and nobody has read them for this question. This
is not a reason to reopen the row, which is closed on its code half; it
is a cheap discharge left on the table, and it is named in the residuals.

### Compiler boundary — 11 items

| # | Item | Disposition | Evidence or boundary |
|---|---|---|---|
| 1 | validated plan has no unchecked constructor | PASS | T4-001, compiler `operation_plan` public boundary |
| 2 | complete analyzed validation before publication | PASS | T4-001 |
| 3 | compact-ASH scope exact | PASS | T4-001 |
| 4 | relation census exact | PASS | 23 relations, recomputed two independent ways and compared in the T4-007 evidence plan |
| 5 | carrier census exact | PASS | T4-005: all 30 relation-cases uniquely carried; T4-005a rebound the constructor symbol under an introspection-reference census |
| 6 | layout census exact | PASS | T4-001: 69 layout requirements |
| 7 | coverage census exact | PASS | T4-007: 211 requirements, 139 positive and 72 negative, each total recomputed twice and compared; the negative total is further split into three columns checked against it |
| 8 | explicit representation policy retained | PASS | T4-001 states the Guide-11 representation |
| 9 | no graph index public | PASS | Standing rule recorded at the phase exit; no public complete-analysis API is frozen |
| 10 | no target opcode or position in compiler core | PASS | Standing rule: target-specific fields remain forbidden in realization and compiler core |
| 11 | no compiler digest minted without a consumer | PASS | Standing rule; the §30 identity block mints nothing |

### Target and patterns — 14 items

| # | Item | Disposition | Evidence or boundary |
|---|---|---|---|
| 1 | reviewed target validates | PASS | T4-002, target-elements `transaction_form` |
| 2 | nonce and point-encoding conventions correct | PASS | R10 repaired; the convention derives from the prefix pair |
| 3 | every required capability assessed | PASS | T4-003: all 79 plan requirements assessed |
| 4 | every selected pattern names prerequisites | PASS | T4-003: eight pattern identities earned by machine-checked schedules |
| 5 | every arithmetic success flag checked | PASS | T4-003 |
| 6 | permissionless leaves carry no protocol signature | PASS | T4-005a: the recognition fragments read the constructor's program off the spent input; no key or signature enters a leaf |
| 7 | sponsor authorization profile explicit | QUALIFIED | T4-008: `ExecutorCapability::TestSponsorAuthorization`, one capability whose funding and signing halves are inseparable. The boundary: the authorization is the executor's and test-scoped by construction, under ADR-015 rule test-material — it is not a protocol-owner profile |
| 8 | explicit ASH follows Guide 11 | PASS | T4-001 |
| 9 | confidential `U` rejects | PASS | T4-003 assessment; the class is in the §18 matrix |
| 10 | abstract success/failure states validate | PASS | R12 repair |
| 11 | exact pushed-literal truth and equality cases pass | PASS | R12 repair |
| 12 | parser work limit enforced during parsing | PASS | R13 repaired: decode stops at the work bound |
| 13 | script-byte accounting equals encoded length | PASS | R11 repair, re-checked by the T4-010 basis test |
| 14 | parser round-trip holds | PASS | T4-009: every materialized vector must decode and re-encode to its own bytes before any mutation is allowed; all nine pass |

### Relocatable and linked bundle — 12 items

| # | Item | Disposition | Evidence or boundary |
|---|---|---|---|
| 1 | every symbol typed | PASS | T4-005: 22 typed symbols |
| 2 | definitions unique | PASS | T4-005: 21 of 22 defined, uniqueness checked in two-pass resolution |
| 3 | references resolve in two passes | PASS | T4-005 |
| 4 | mandatory relocations resolve exactly once | PASS | T4-005: 61 relocations applied by structured substitution |
| 5 | no overlapping or variable-width byte relocation | PASS | T4-004: relocations located by rebuilding each program with one symbol replaced |
| 6 | constructor deterministic | PASS | T4-004, static ASH constructor over 9 shapes |
| 7 | internal-key policy explicit | PASS | T4-005 |
| 8 | taptree deterministic | PASS | T4-005: deterministic 12-leaf taptree |
| 9 | exhaustive small-tree oracle agrees | PASS | T4-005: exact subset oracle, itself checked against literal enumeration |
| 10 | every required carrier reachable | PASS | T4-005: all 30 relation-cases uniquely carried |
| 11 | bundle explicitly candidate-only | PASS | T4-004: construction refuses a release-complete plan; artifact status is read-only |
| 12 | clear lifecycle remains explicit | PASS | §30 lifecycle block records clear as outstanding |

### ABI and transactions — 14 items

| # | Item | Disposition | Evidence or boundary |
|---|---|---|---|
| 1 | input family order canonical | PASS | T4-006 |
| 2 | coordinator derives from canonical ASH order | PASS | T4-006 |
| 3 | finite shape specialization explicit | PASS | T4-006, 9 admitted shapes |
| 4 | sponsor suffix exact | PASS | T4-006, checked against hand-written byte strings at 374 bytes sponsored |
| 5 | output roles exact | PASS | T4-006 |
| 6 | fee role cannot satisfy sponsor change | PASS | T4-008 repaired the underlying constant: the fee-role digest is SHA-256 of the empty script, not an invented value |
| 7 | sponsor change cannot satisfy fee role | PASS | Same repair; the conformance cross-check caught the drift as designed |
| 8 | successor amount derives exactly | PASS | T4-007: the successor derives from the realization layer's own checked sum |
| 9 | caller cannot choose protected outputs or programs | PASS | T4-006 construction pipeline; a request carries no field to argue with the ABI |
| 10 | signatures requested after output finalization | QUALIFIED | Vacuous in this candidate: the emitted programs carry no signature and the validation budget is zero. The only signing is the executor's test-scoped sponsor authorization. Nothing here is false; nothing here is exercised either |
| 11 | witness and control ordering canonical | PASS | T4-006; T4-009's witness-reorder arm is refused at the script path |
| 12 | sponsor amounts outside protocol predicates and reports | PASS | the realization's sponsor-value opacity, guarded in `realization` |
| 13 | synthetic ASH labeled test-only | PASS | T4-008 ceremony bundle; ADR-015 rule test-material |
| 14 | candidate ABI not final | PASS | Standing rule: no stable linker or transaction ABI exists |

**ABI residual.** T4-006 records one outstanding obligation that no later
wave closed: the taproot output key is pinned rather than recomputed, so
the pin-against-tree equality is carried rather than proved. It is
carried honestly and it is named in the residuals.

### Evidence — 16 items

| # | Item | Disposition | Evidence or boundary |
|---|---|---|---|
| 1 | arbitrary fixtures cannot become gate-eligible | PASS | T4-007: canonical standing has a crate-private constructor and no promotion path |
| 2 | canonical evidence plan derives from compiler coverage | PASS | T4-007, §16.4 evidence plan |
| 3 | expected semantics derive independently of target emission | PASS | T4-007: 14 positive semantic cases carrying no target vocabulary |
| 4 | every positive target transaction complete | PASS | T4-008: twelve submitted, every one accepted and mined |
| 5 | every accepted projection matches | PASS | T4-008: all twelve matched on all thirteen §17.4 terms, observed side read from accepted bytes and `gettxout` rather than echoed |
| 6 | every negative relation reaches its owning evidence layer | **FAIL** | See the finding below |
| 7 | construction and target rejection remain distinct | PASS | T4-013 `ObservedDivergence`; T4-009's mempool-reason repair restored the distinction where block validation had blurred it |
| 8 | whole-transaction conservation explicit evidence | PASS | T4-006: the three whole-transaction dimensions settle per constructed transaction |
| 9 | target environment matches binding | PASS | Pinned genesis `209577bd…ca07d`, chain `elementsregtest`, node `v28.99.0-78499c206475` |
| 10 | executor provenance satisfies ADR-018 | PASS | R01 repaired with a full-width revision type; provenance carries intended tip, upstream base, and local topics |
| 11 | no mock satisfies the gate | PASS | T4-008: the adapter funds and submits for real; `CoverageObservation`'s observed arm requires acceptance and a matched projection both |
| 12 | no required case has infrastructure failure | PASS | T4-008 and the Wave 13/14 runs; R14 repaired so a non-target outcome carries no target observation |
| 13 | non-target outcomes carry no target observations | PASS | R14 repair, applied to every workload |
| 14 | protocol records strict and bounded | PASS | R05 duplicate-sensitive census equality; R06 per-pass row census |
| 15 | report bytes reproduce | PASS | Three runs per wave, byte-identical transcripts, in each of Waves 11 to 14 |
| 16 | no report digest minted | PASS | §30 identity block; standing rule on raw report digests |

**Finding — item 6 cannot pass as written.** Coverage is 100 of 211,
being 99 positive and 1 negative. Exactly one negative relation has
reached its owning evidence layer: ASH-output cardinality above maximum,
sponsorless case. The 72 negative rows are classified by where a refusal
could be observed, derived from each requirement's own evidence role
rather than marked by hand, into three columns that are recomputed and
checked against the negative total: 48 target-executable, 18 first-party
— 10 compiler-analysis and 8 emitted-structure — and 6 unreachable. One
of the 48 is discharged, so the 71 outstanding fall out as follows.

- **48 target-executable, 47 outstanding.** Discharge needs a complete
  mutated target transaction, an executed carrier, and an observed
  target rejection. Six mutation arms were refused at the script path
  and exactly one of them names a relation, because §18's tables are
  lists of names: no row there names a relation, a mutation class, or a
  boundary. Two arms have no member in the negative mutation vocabulary
  at all; two fit two published classes equally well.
- **18 first-party rows, all outstanding.** §19.1 lets positive coverage
  of a compiler-static or backend-structural relation use typed
  structural evidence. §19.2 states no such rule for the negative half,
  and its conditions — a complete mutated target transaction, an
  executed carrier, an observed target rejection — are three things a
  compiler-static relation cannot have. The guide is the reason before
  the evidence is. The evidence is independently thin, and T4-009
  enumerates how per class.
- **6 external-report rows, unreachable.** Their evidence is an external
  report nobody has written.

Two further rows sit outside those three columns and outside the
negative half entirely, each carrying its own census column so that
neither is filed under the nearest neighbour. The two-sponsor row is
unbuilt in this candidate: the two-member sponsor region is above the
demonstration bounds, so no program for its shape was ever emitted, and
reaching it needs a candidate linked under wider bounds. This is a limit
of the candidate, not of the target — the target never asked. The
money-bound divergent row reaches no coverage row at all, because
nothing is submitted for it.

**Named remedy.** Three guide gaps must close before item 6 can be read
as written: §18 must name a relation, a mutation class, and a boundary
per row rather than a name only; §19.2 must state a first-party
discharge condition for the negative half, or the 18 rows must be
declared out of scope; and the §16.5 ABI-validation entry point must
exist, since two arms expect a refusal before any target sees the bytes
and no requirement is indexed at the constructor's boundary. These three
are the handoff's feature-request material. Inventing a link to move
four rows would be the discharge-by-intent the plan exists to prevent,
and Wave 13d declined to do it.

### Resources — 11 items

| # | Item | Disposition | Evidence or boundary |
|---|---|---|---|
| 1 | coordinator program measured | PASS | T4-010, recomputed from programs the study emitted |
| 2 | member program measured | PASS | T4-010 |
| 3 | every push byte counted | PASS | T4-010; push bytes asserted a strict positive share of total script bytes |
| 4 | constructor measured | PASS | T4-010 basis test reproduces the linked bundle's totals exactly |
| 5 | complete sponsorless transaction measured | PASS | Weights 1293, 1736, 2177 for two, three, four ASH inputs |
| 6 | complete sponsored transaction measured | PASS | Weights 1861 and 1862 |
| 7 | candidate shape matrix measured | PASS | All 36 of the §20.2 product measured; the rendered matrix is recomputed each run rather than transcribed |
| 8 | prediction equals observation | QUALIFIED | All twelve weighed live and all twelve matched, and a disagreement refuses the whole plan. The boundary: weight is the one dimension where the comparison can be made. Eleven of §20.3's eighteen measures are answered and the other seven say why — see below |
| 9 | consensus and policy verdicts separate | PASS | T4-010: 0.05 percent of the 4000000 consensus weight limit and 0.54 percent of the 400000 policy limit, reported apart |
| 10 | selected bound candidate-only | PASS | §20.6: no bound selected |
| 11 | no final calibration claim | PASS | §20.6: none calibrated; standing rule that no draft bound becomes deployment calibration |

**The seven unmeasured §20.3 items, with their per-item reasons.** Peak
stack, peak altstack, and largest item are not observable through this
adapter, because a validating node exposes no interpreter stack; the
alternate stack is additionally untouched by any reviewed primitive, and
the only static peak walk is private to the prototype lane. Arithmetic
operations and comparison counts have no charged dimension to carry
them. Control bytes and initial witness items are structural and simply
not surfaced by this wave. None reports zero in place of an answer,
which is the property that matters.

**The ceiling finding, recorded because it changes what a later wave
should attempt.** Linking, not weight or script size, is what bounds a
candidate today. The exact taptree oracle costs three-to-the-n set
operations and refuses above sixteen leaves, so seven of thirty-six
assignments link. Fourteen leaves link and the tree is counted; eighteen
answer `TreeOracleBudgetExceeded`. The linkable corner is an awkward
shape rather than a prefix, which is what §20.2 means by refusing
monotonicity. Weight is measurably not affine in batch size — it steps
443 then 441 — while the byte models are, so a weight predicted from a
fitted model would have been wrong.

### Repository — 17 items

| # | Item | Disposition | Evidence or boundary |
|---|---|---|---|
| 1 | package READMEs current | PASS | All sixteen present; audited this wave |
| 2 | package contracts current | PASS (fixed in lane) | Two stale claims found and repaired — see below |
| 3 | Phase-4 card current | PASS (fixed in lane) | Status line still said Wave 4 begins the phase; refreshed |
| 4 | backlog state current | PASS | T4-011 updated in this wave's commits |
| 5 | every new file in the Meson census | PASS | This document joins `plans/history/meson.build` in its own commit |
| 6 | dependency review recorded | PASS | T4-002 substrate review, ratified by the ruling of 2026-08-20; no new dependency in Phase 4 |
| 7 | identity and schema impact recorded | PASS | §30 identity block: nothing minted, protocol revision unmoved since Wave 1b |
| 8 | `cargo fmt --all` | PASS | Run this wave |
| 9 | Clippy with warnings denied | PASS | Run this wave |
| 10 | workspace tests | PASS | Run this wave |
| 11 | `scripts/ci.sh` | DEFERRED-TO-GATE | Last measured 48 m 29 s cold for 47 of 47 lanes; the orchestrator's authoritative gate owns the verdict |
| 12 | canonical Meson compile | DEFERRED-TO-GATE | Same lane |
| 13 | canonical Meson tests | DEFERRED-TO-GATE | Same lane |
| 14 | advisory passed or explicitly skipped | DEFERRED-TO-GATE | The advisory lane exits 77 so meson reports its own SKIP |
| 15 | document reproducibility | DEFERRED-TO-GATE | By policy this is a release question: the checker builds the document twice in disposable directories and refuses a dirty worktree, so it cannot be a lane of the run whose cleanliness it depends on |
| 16 | `git diff --check` | PASS | Run this wave |
| 17 | final repository status empty | PASS | Tree clean at the audited tip |

**Fixed in lane, under fix-don't-weaken.** Three stale factual claims,
each mechanical and local:

- `plans/packages/vectors.md` said execution, mutations, and resources
  were outstanding. All three landed in Waves 11 to 14.
- `plans/packages/README.md` said of vectors that no target has executed
  anything. A target has executed twelve accepted transactions, six
  refused mutations, and one refused divergent row.
- `scripts/elements-native-executor.py` said in its docstring that the
  adapter speaks protocol revision 3, while its own
  `NATIVE_PROTOCOL_SCHEMA` constant is 4 and the comment beside that
  constant explains why. The sentence's substantive point survives; only
  the revision number was wrong.

The Phase-4 card's status line was refreshed to name the delivered
waves. It does not claim the phase exited: that verdict is the
orchestrator's gate record, not this document's.

Nothing larger was repaired here. The two structural gaps this audit
found — the negative-coverage item and the pinned taproot output key —
are recorded as a FAIL with a named remedy and as a residual
respectively.

### Disposition totals

| Section | Items | PASS | QUALIFIED | FAIL | DEFERRED |
|---|---:|---:|---:|---:|---:|
| Preflight | 10 | 8 | 2 | 0 | 0 |
| Compiler boundary | 11 | 11 | 0 | 0 | 0 |
| Target and patterns | 14 | 13 | 1 | 0 | 0 |
| Relocatable and linked bundle | 12 | 12 | 0 | 0 | 0 |
| ABI and transactions | 14 | 13 | 1 | 0 | 0 |
| Evidence | 16 | 15 | 0 | 1 | 0 |
| Resources | 11 | 10 | 1 | 0 | 0 |
| Repository | 17 | 12 | 0 | 0 | 5 |
| **Total** | **105** | **94** | **5** | **1** | **5** |

---

## Completion report

```text
Guide 12 result
===============

Starting state:
    source revision: 0.4.1-dev
    working tree: clean
    architecture schema: 17
    architecture semantic hash: sha256-canonical-json-v3; recomputed by
        the artifact weld at build rather than pinned as a literal, so
        no value is transcribed here
    architecture behavioural hash: sha256-canonical-json-behavioural-v3
        over the schema-17/v13 tree; recomputed by the artifact weld at
        build rather than pinned as a literal, so no value is
        transcribed here
    realization version: v13d
    target contract revision: reviewed Elements contract in
        target-elements, with welds and status closure
    native protocol revision: 4
    Guide-11 result: complete; explicit-only representation with
        normalization
    Phase-3 gate: exited 2026-08-20, 13 of 13 CI lanes and 36 of 36
        meson lanes
    compiler compact-ASH projection: 23 relations, 2 cases,
        46 relation-cases of which 4 vacuous, 69 layout requirements

Preflight:
    G12-R01 provenance invariant: CONFIRMED, DONE
    G12-R02 anchor-set identity: CONFIRMED, DONE
    G12-R03 CLI contract: CONFIRMED, DONE
    G12-R04 child diagnostic boundary: CONFIRMED, DONE on the code half;
        the runtime half is recorded blocked on a live node and was
        never re-read against the Waves 11-14 transcripts that now exist
    G12-R05 normalization census: CONFIRMED, DONE
    G12-R06 lifecycle pass census: CONFIRMED, DONE
    G12-R07 script-error parser: CONFIRMED, DONE, discharged by
        recomputation over the whole mapped-message class table
    G12-R08 planning status: RECLASSIFIED, DONE
    G12-R09 protocol schema: CONFIRMED, DONE; both sides moved to
        revision 4 together
    G12-R10 nonce parity: CONFIRMED, DONE
    G12-R11 script-byte resources: CONFIRMED, DONE
    G12-R12 exact-literal abstraction: CONFIRMED, DONE
    G12-R13 parser work bound: CONFIRMED, DONE
    G12-R14 infrastructure observations: CONFIRMED, DONE
    G12-R15 runner supervision: CONFIRMED, DONE by behavioural probe
    G12-R16 evidence documentation: CONFIRMED, DONE

Compiler boundary:
    public target-operation type: compiler operation_plan
    scope: compact ASH, exact
    representation policy: Guide-11 explicit-only, retained
    relations: 23
    execution cases: 2
    abstract carriers: 30 relation-cases uniquely carried
    layout requirements: 69
    coverage requirements: 211, being 139 positive and 72 negative
    capabilities: all 79 plan requirements assessed
    external evidence: 2 requirements
    lifecycle: compact candidate implemented, clear outstanding
    graph handles exposed: none
    identity minted: none

Target assessment:
    reviewed target: Elements, target-elements transaction_form
    deployment binding: development only; production support not claimed
    missing primitives: none blocking the candidate
    unsupported capabilities: none blocking the candidate
    backend patterns required: 8 pattern identities, each earned by a
        machine-checked schedule
    structural obligations: 3
    external evidence: 2
    sponsor authorization profile: explicit and test-scoped; the
        executor authorizes with a published constant key under an
        RFC 6979 nonce, ADR-015 rule test-material
    transaction-form review: both forms consensus-admitted; the sponsor
        asset is explicit and its value uninspected
    selected representation: explicit

Tapscript patterns:
    object recognition: by identity introspection - the recognition
        fragments read the constructor's program off the input the leaf
        is spending
    family cardinality: checked
    closed-asset closure: checked
    canonical partition: checked
    aggregate arithmetic: checked, exact wide floor arithmetic
    member participation: checked
    coordinator: one canonical coordinator program
    sponsor isolation: checked; sponsor amounts stay outside protocol
        predicates and reports
    root absence: root-free
    projection absence: no projection in the emitted programs
    exact-literal result: sound after the R12 repair
    parser result: work-bounded during parsing after the R13 repair
    program roles: coordinator and member
    stack result: validated abstractly; concrete peaks unobservable
        through this adapter
    candidate status: candidate only

Relocatable bundle:
    programs: 9 coordinator leaves and 3 recomputation-shared member
        leaves
    shapes: 9
    constructor: static ASH constructor, deterministic
    symbols: 22 typed, 21 defined
    relocations: 61
    placements: located by rebuilding each program with one symbol
        replaced, plus 9 introspection references read from the emitted
        instructions
    witness roles: leaf program and control block; no key, no signature
    resource formulas: coordinator base 85 and per-ash-input 78,
        member 64
    unresolved obligations: none at the bundle layer after T4-005a
        retired AshConstructorProgramVersion

Linker:
    package revision: packages/linker, candidate
    symbol census: 22 typed, 21 defined
    reference graph: frozen; one strongly connected component, the ASH
        constructor's self-commitment, resolved by introspection
    SCCs: 1
    relocation result: 61 applied by structured substitution;
        3939 relocatable bytes to 3975 linked bytes
    taptree policy: deterministic
    taptree depth: ControlPathDepth 4
    carrier census: all 30 relation-cases uniquely carried
    candidate bundle: 12 leaves
    deterministic rebuild: yes; the study reproduces the linked bundle's
        total script bytes, leaf count, and both role byte models exactly

Transaction substrate:
    Rust dependency: none added; first-party explicit-field encoder and
        decoder
    version: not applicable
    features: not applicable
    licence: not applicable
    MSRV: unmoved
    unsafe/FFI: none introduced
    advisories: no new dependency to advise on
    lockfile impact: none

Transaction ABI:
    input layout: canonical family order
    output layout: exact output roles
    coordinator rule: derives from canonical ASH order
    shape specialization: finite and explicit over 9 admitted shapes
    sponsor suffix: exact
    sponsor change: cannot satisfy the fee role
    fee role: cannot satisfy sponsor change; the fee-role digest is
        SHA-256 of the empty script, computed rather than invented
    transaction version: topology-restricted for the sponsorless form,
        standard for the sponsored form; a policy choice, since consensus
        admits the sponsorless form at either version
    sequence: final on every input, typed as an ABI convention
    witness order: canonical
    control-path roles: depth 4
    request fields: carry no field that can argue with the ABI
    candidate bounds: demonstration bounds only
    candidate status: not final

Synthetic ASH:
    test asset: issued by the ceremony in the runs of
        record
    issuance/funding method: bundle::ceremony_bundle links a second
        bundle at the asset the ceremony issues, because an Elements
        asset identifier derives from the issuing outpoint and the
        canonical fixture bundle is therefore unfundable on any real
        chain
    constructor: static ASH constructor at the ceremony asset
    values: explicit throughout
    test-only statement: ADR-015 rule test-material
    burn-lineage non-claim: none made
    attestation non-claim: none made

Semantic fixtures:
    model source: realization v13d
    realization report: the successor derives from the realization
        layer's own checked sum
    expected successor: derived, not transcribed
    expected certificate: none minted
    fixture count: 153 named section-18 classes; 14 positive semantic
        cases carrying no target vocabulary, 9 of them materialized to
        exact byte-stable target transactions

Target vectors:
    positive cases: 12 submitted, 8 sponsorless and 4 sponsored
    negative cases: 8 mutation arms; 6 submittable, 2 withheld because
        their boundary precedes the target
    construction failures: 0
    target rejections: 6 - five at the script path and one as consensus
        rejection before script
    infrastructure errors: 0
    accepted projection mismatches: 0
    relation coverage: 100 of 211, being 99 positive and 1 negative
    unresolved coverage: 111 - 71 negative rows outstanding, 40 positive
        rows not active in either case

Target execution:
    protocol schema: 4
    executor: scripts/elements-native-executor.py
    adapter version: 2.1.0
    framework revision: Elements functional framework at the node tree
    node version: v28.99.0-78499c206475
    binary-reported revision: 78499c206475
    intended tip: 78499c2064
    upstream base: recorded per run by ExpectedExecutorProvenance
    local topics: recorded per run by ExpectedExecutorProvenance
    network: elementsregtest
    genesis: recorded per run by the harness, the chain being created
        and destroyed by the run that used it
    consensus cases: 12 accepted and mined; 1 consensus rejection before
        script; 1 money-bound divergence refused before submission
    policy cases: sponsorless form relayed under the topology-restricted
        version; policy weight headroom 0.54 percent at the largest
        measured transaction
    deterministic report bytes: yes; three runs per wave, byte-identical
        transcripts, in each of Waves 11 to 14

Resources:
    coordinator bytes: base 85, per-ash-input 78
    member bytes: 64
    push bytes included: yes, a strict positive share of total script
        bytes
    constructor bytes: 3939 relocatable, 3975 linked
    leaf count: 12 in the candidate bundle; 14 links and is counted,
        18 answers TreeOracleBudgetExceeded
    tree depth: 4
    control bytes: not surfaced by this wave
    witness bytes: carried by the study
    peak stack: not observable through this adapter
    peak altstack: not observable; no reviewed primitive touches it
    largest item: not observable through this adapter
    arithmetic operations: no charged dimension carries them
    validation budget: 0 - this candidate emits no signature or curve
        primitive
    sponsorless weight: 1293 at two ASH inputs, 1736 at three,
        2177 at four
    sponsored weight: 1861 and 1862
    policy result: 0.05 percent of the 4000000 consensus weight limit,
        0.54 percent of the 400000 policy limit
    candidate ASH bounds: 6 research bounds
    candidate sponsor bounds: 6 research bounds
    selected demonstration candidate: none selected
    calibration claim:
        none

Lifecycle:
    compact:
        candidate implemented
    clear:
        outstanding
    release-complete:
        false

Identity impact:
    Attestation: unmoved
    realization: unmoved, v13d
    architecture schema: unmoved, 17
    architecture semantic hash: unmoved
    architecture behavioural hash: unmoved
    anchor-set hash: unmoved; R02 narrowed the preimage to a validated
        anchor-set type without changing the recipe
    compiler identity:
        none
    target-plan identity:
        none
    bundle identity:
        none unless separately admitted
    ABI identity:
        none unless separately admitted
    vector/report identity:
        none
    deployment profile:
        dormant

Dependency impact:
    new first-party packages: linker, transaction, vectors
    third-party additions: none
    Cargo.lock: unmoved by Phase 4
    licences: unchanged
    MSRV: unchanged
    unsafe/FFI: none introduced
    advisories: none new

Verification:
    cargo fmt --all: pass, this wave
    cargo clippy --workspace --all-targets --locked -- -D warnings:
        pass, this wave
    cargo test --workspace --locked: pass, this wave
    architecture: pass
    model: pass
    realization: pass
    compiler: pass
    target-elements: pass
    tapscript: pass
    linker: pass
    transaction: pass
    vectors: pass
    target-elements-conformance: pass
    protocol cross-language: pass, revision 4 both sides
    Rustdoc: deferred to the authoritative gate
    scripts/check-plans.sh: pass, this wave
    meson compile -C build lint: deferred to the authoritative gate
    scripts/ci.sh: deferred to the authoritative gate; last measured
        48 m 29 s cold, 47 of 47 lanes
    meson compile -C build: deferred to the authoritative gate
    meson test -C build --print-errorlogs: deferred to the authoritative
        gate
    real compact-ASH matrices: pass; three runs per wave, byte-identical
        transcripts, Waves 11 to 14
    cargo audit: deferred to the authoritative gate; the lane exits 77
        so meson reports its own SKIP
    document reproducibility: deferred under policy - the checker builds
        the document twice in disposable directories and refuses a dirty
        worktree, so it answers a release question and cannot be a lane
        of the run whose cleanliness it depends on
    git diff --check: pass, this wave
    final git status: clean

Phase-4 verdict:
    accepted candidate

    Read exactly. One complete candidate compiler-to-target operation
    exists and a real target executed it: twelve transactions accepted
    and mined, twelve semantic projections matched on all thirteen
    section 17.4 terms, twelve weights predicted and observed equal.
    The verdict is not that Phase 4's evidence plan is discharged - it
    is 100 of 211 - and not that the negative half is demonstrated. It
    is that the welded chain the closing statement names was built and
    ran end to end in both execution cases.

Residuals:
    1. Negative coverage. 71 of 72 negative requirements outstanding,
       in three columns with distinct reasons: 47 target-executable,
       18 first-party, 6 external-report. Three guide gaps must close
       first - section 18 naming relations, mutation classes and
       boundaries per row; section 19.2 stating a first-party discharge
       condition or declaring the 18 out of scope; and the section 16.5
       ABI-validation entry point, which does not exist.
    2. Positive coverage. 40 positive rows not active in either
       execution case, among them the two-sponsor row, which is unbuilt
       in this candidate because its shape is above the demonstration
       bounds and no program for it was ever emitted. It carries its
       own census column, as does the money-bound divergent row, which
       reaches no coverage row because nothing is submitted for it.
    3. The taproot output key is pinned rather than recomputed, so the
       pin-against-tree equality is carried as an outstanding
       obligation.
    4. Seven of section 20.3's eighteen resource measures are
       unanswered, each with a stated per-item reason.
    5. Linking is the candidate ceiling: 7 of 36 assignments link, and
       the exact taptree oracle refuses above sixteen leaves. A wider
       candidate needs a cheaper oracle before it needs anything else.
    6. G12-R04's runtime half is still recorded blocked on a live node,
       though the transcripts that could discharge it now exist.
    7. Three section-18 boundary classes were respecified during
       Phase 4 - wrong-sequence, wrong-transaction-version, and
       successor-one-below-the-sum. The matrix that produced them was
       wrong three times in one family, which is a fact about the
       matrix.
    8. The Elements gripe material from T4-012 is recorded and unfiled.
       Filing is discretionary, not an engineering blocker.
    9. The sequence field is unconstrained by covenant and by
       signature, so a third party can malleate it. This is an
       availability question and not a safety one - every relation
       still holds under any sequence - and the topology-restricted
       version bounds the pinning surface through the TRUC package
       limits. It is named because a later guide that adds
       output-committing signatures changes this analysis.

Next phase:
    Guide 13, end-to-end live receipt transfer. It consumes the
    validated boundary, the exact target assessment, the typed pattern
    interface, the bundle and linker models, the deterministic taptree
    policy, the candidate ABI framework, the evidence framework, the
    strict executor protocol, the environment and provenance binding,
    the canonical report trust states, and the Guide-11 representation
    policy. It adds owner authorization, multiple protocol outputs,
    split and merge, output-committing signatures, explicit and private
    committed value alternatives, separate safety and minimality
    evidence, and the burn and redemption lifecycle obligations.
```

---

## Residuals and inheritance

The nine residuals above are the whole of what Guide 12 hands forward.
Three of them are feature requests against the guide rather than defects
in the code: §18 does not name a relation, a mutation class, or a
boundary per row; §19.2 states no first-party discharge condition for
the negative half; and the §16.5 ABI-validation entry point does not
exist. A later guide that wants the negative half discharged has to
close those three first, and no amount of first-party test writing
substitutes for them.

The one verdict this record will not soften: the negative evidence layer
is not demonstrated. One negative row of seventy-two reached its owning
layer. Everything else about the negative half is machinery that works,
run against a real target, producing answers nobody is allowed to count
yet.
