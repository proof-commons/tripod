# Ninth static review — tree 0.5.17-dev

## Archival note

An external adversarial review of the phase-5 closing arc, commissioned against the range `0.5.12-dev..0.5.17-dev` at exact HEAD `0.5.17-dev`. It is archived here as the record of what was found; its findings register is backlog §5.10 and the remediating batch is the `T5-071` gate-repair wave.

Two MECHANICAL deviations from the received bytes, both named here so the archive is not silently edited. No word of any finding is changed by either.

- Fifty-five source citations arrived as Markdown links to machine-local absolute paths, which resolve nowhere for a reader and which the plan-tree checker cannot follow. Each is rendered as the plain `path:line` code span its link text already was.
- The verdict heading carried the exit-gate label as a bare backticked span, which the planning harvest reads as a MINT and which would therefore duplicate the label the phase card mints. It is parens-wrapped as a citation. This is the review's own finding F8 applied to the review's own bytes.

---

# Phase-5 closing-arc adversarial review

Range reviewed: `0.5.12-dev..0.5.17-dev` at exact HEAD `0.5.17-dev`.

## 1. Per-wave verdicts

| Wave | Verdict | Justification |
|---|---|---|
| T5-063 | SOUND | Codes 7/8, both narrowing refusals, `proven_by: None`, projection limitation, and the recomputed 480-cell map all match the report. |
| T5-065 | SOUND | All 21 historical gaps were retyped, none driven; the 7/9/2/2/1 disposition is grounded by the emitted covenant, request shape, and first-party recognition. |
| T5-066 | SOUND-WITH-FINDINGS | The seven mutants, ordering, control and pairwise-distinct separators exist, but the native test and canonical standing do not bind all the observation facts claimed. |
| T5-067 | SOUND-WITH-FINDINGS | The two driven arrangements and 29→27 split exist, but native assertions are weak and the exact outcomes predicted for the two unbuilt typed partners are not established here. |

### Review-dimension verdicts

| Dimension | Verdict | Findings |
|---|---|---|
| A. Claim-vs-code fidelity | PASS-WITH-FINDINGS | All arithmetic and declarations recompute; F3–F4 weaken observation claims. |
| B. Attributability | PASS-WITH-MAJOR-FINDINGS | Mutant-first/control-last and separator uniqueness hold; F3–F4. |
| C. Honesty of determinations | PASS-WITH-FINDINGS | T5-063 and T5-065 are grounded; T5-066 records the boundary mismatch; F4 affects the leaf partners. |
| D. Exit assessment | FAIL | F1 and F2. |
| E. Code quality | PASS-WITH-FINDINGS | F3–F5; no additional census-row-vanishing weakness found. |
| F. Records and prose | FAIL | F1, F2, F6–F8. |

## 2. Findings, ranked

### BLOCKER — F1: Exit row 4 contradicts the canonical completeness predicate

Exact claim touched: “explicit transfer passes complete safety evidence,” “every gate row is met,” and “81 observed or first-party-established” in `plans/phases/05-live-transfer.md:206`, `221`, and `225`.

The canonical evidence plan says the opposite:

- `NativeRunRequired` is still an unanswered run obligation: `live_evidence.rs:565-646`.
- Completeness requires `native_run_required == 0`: `live_evidence.rs:942-959`.
- Tests deliberately require a nonzero required count and `every_required_row_is_answered() == false`: `live_evidence.rs:2628-2657`.
- The 27 rows remain explicitly registered as required: `live_negative_half.rs:277-496`.

Only 80 rows are `is_answered`: `34 + 24 + 16 + 1 + 3 + 2`. The claimed 81st is `OperationVocabularyClosed`, which the source explicitly calls “not answered” and “not evidence”: `live_evidence.rs:740-754`, `2121-2147`.

Missing evidence: answers for the 27 `NativeRunRequired` rows, or an authoritative change to the completeness policy accompanied by reclassification out of `NativeRunRequired`. Closing prose alone does neither.

### BLOCKER — F2: Exit row 12 says mixed-program vectors reject, but no such vector or verdict exists

Exact claim touched: “mixed-program vectors reject,” at `plans/phases/05-live-transfer.md:214`, repeated as MET at `line 223`.

The implementation says a mixed-operation program is unrepresentable, no layer can be offered one, and no verdict exists: `live_safety.rs:225-261`, `949-961`. Its test pins that as the sole no-layer row: `live_safety.rs:1872-1885`. The evidence test again states it is not a refusal or evidence: `live_evidence.rs:2121-2147`.

Missing evidence: a stageable mixed-program input and its owning refusal. Alternatively, the gate must be amended to require operation-vocabulary closure rather than rejection.

### MAJOR — F3: Native tests do not bind the recorded refusal details, control acceptance, or constants

Exact claims touched: T5-066’s seven exact consensus refusals and accepted control, and T5-067’s two exact script refusals and accepted control.

The native test:

- accepts any observed layer for the bare-u mutant and control, and checks readback only conditionally: `guide13_live_native.rs:1239-1270`;
- checks the seven consensus layers and mutual separator uniqueness, but not row names, exact refusal detail, accepted control layer/txid, or equality to the run constants: `guide13_live_native.rs:1325-1358`;
- checks only `observed_layer().is_some()` and arrangement uniqueness for the leaf mutants: `guide13_live_native.rs:1286-1323`.

The record exposes the missing detail and accepted txid, so they could be checked: `live_owner_signing_negatives.rs:378-435`, `593-639`. Instead, authored constants supply the claims afterward: `live_owner_signing_negatives.rs:1878-1993`.

The canonical evidence arm then maps all seven consensus rows to the identical `(control, "bad-txns-in-ne-out")` pair, dropping their separators: `live_evidence.rs:1449-1481`. `NativeRefusalObserved` has no layer or separator field: `live_evidence.rs:618-645`.

Thus the lane could remain green if the control were rejected, a leaf mutant answered at the wrong layer, or details/txid changed.

### MAJOR — F4: The two typed leaf-partner outcomes are predictions, not demonstrated non-separations

Exact claims touched: `wrong-coordinator` would duplicate `two-coordinators`, and `member-coordinator-leaf-exchange` would duplicate `no-coordinator`.

Only `[0,0]` and `[1,1]` are declared and built; the typed partners are strings associated with those two variants: `live_owner_signing_negatives.rs:301-369`. No typed-partner mutant is constructed or submitted. Nevertheless, the register states exact hypothetical failure clauses and words: `live_negative_half.rs:436-458`.

The in-repo covenant establishes the per-input coordinator `EqualVerify` and member-bound `Verify` clauses: `pattern.rs:898-915`, `917-967`. It does not establish which failure a candidate with the reported “second failing input” returns as its target-level detail. The tests examine only the two driven arrangements.

The typed deferrals may be reasonable because the faults cannot be isolated cleanly, but the stronger “would duplicate this exact partner verdict” language is UNVERIFIABLE-HERE. Settling target multi-input failure selection would require out-of-scope Elements evidence or actually driving the partner arrangement.

### MINOR — F5: Several comments falsely say all seven consensus mutants have distinct ranges

Examples:

- “seven distinct regions” at `live_owner_signing_negatives.rs:49-53`;
- the non-claim says every consensus refusal separates by a distinct range: `lines 705-721`;
- native-test prose repeats “distinct field range”: `guide13_live_native.rs:1275-1278`.

In fact, `private-output-omitted` and `hidden-private-u-output` share `88..245` and separate by shapes `(2,1)` and `(2,3)`: `live_owner_signing_negatives.rs:1934-1948`. The implemented tuple test is correct; the surrounding prose is stale. The non-claim’s “only the bare-u mutant is a script-path verdict” is also stale after adding the two leaf-script mutants.

### MINOR — F6: The filed cross-chain separator finding gives the wrong separating fact

The omission is honestly recorded at `plans/backlog.md:342`, but it says an eighth “same-verdict” row separates “by verdict string.”

Both lanes record exactly `bad-txns-in-ne-out`: `live_owner_signing_negatives.rs:1904-1914`, `live_conservation_negatives.rs:978-984`. The true additional separator is the different accepted control (`40cb…` versus `1af…`), plus the mutation coordinates within that control: `live_owner_signing_negatives.rs:1879-1885`, `live_conservation_negatives.rs:942-971`.

Manual cross-chain comparison found no equal `(range, shape)` tuple.

### MINOR — F7: The gate says nine waves closed while five remain `ACTIVE`

Section 2.20 says T5-059 through T5-067 closed: `plans/backlog.md:340-342`. At `0.5.17-dev`, T5-062, T5-063, T5-065, T5-066 and T5-067 remain `ACTIVE`: `plans/backlog.md:1299-1307`. T5-063 explicitly says its DONE flip belongs at the exit gate, so this is not merely stylistic.

### MINOR — F8: The closing assessment duplicates a colon-label mint

The label is minted in the heading at `plans/phases/05-live-transfer.md:199`, then appears bare again in prose at `line 227`. It should be parenthesized as a citation.

This is mechanically significant: bare Markdown label spans mint, while parenthesized spans cite: `markdown.rs:9-25`. Planning harvest inserts every bare label as a mint: `repository.rs:819-838`, and duplicates are errors: `registry.rs:37-57`. The other new shape/gate references follow the parenthesized discipline.

### NOTE

The shared `worker-check` collision is recorded both in §2.20 and in the bundle, with the affected conformance lane reported rerun afterward: `materials.md:93-95`. No further same-class shared-checkout incident appears in this range.

## 3. Exit-assessment verdict

**Would I sign (`gate:phase5:exit`) as MET? NO.**

| Gate rows | Review |
|---|---|
| 1–3 | Sign. Positive-half authorization, live-class closure and closed-asset classification have in-tree standings and run records. |
| 4 | **Do not sign.** Twenty-seven rows remain `NativeRunRequired`, and the canonical completeness predicate is false. |
| 5–8 | Sign. The confidential, sponsor-value, four fee-form, and two crossing acceptances are represented in the positive observation census: `live_evidence.rs:2335-2365`. |
| 9 | Sign. `proven_by: None` is honest here: both registry walls are removed, no acceptance is claimed, and the uncovered projection wall is censused independently: `shape_census.rs:1871-1926`, `4328-4358`. |
| 10–11 | Sign. Report separation and lifecycle incompleteness remain explicit. |
| 12 | **Do not sign.** No mixed-program vector rejects; the architecture cannot state one and records no verdict. |
| 13–15 | Sign on the available tree evidence. HEAD is `0.5.17-dev`, the worktree is clean, and no contradictory bundle/ABI, resource or relation record was found. |

Exact missing evidence is therefore confined to rows 4 and 12. Either one independently prevents an overall MET signature.

## 4. Recomputation appendix

### Register evolution

Direct `git grep -c '^    entry('`:

| Tip | Count | Arithmetic |
|---|---:|---|
| `0.5.13-dev` after T5-063 | 36 | Shape-register work did not move the safety register. |
| `0.5.14-dev` after T5-065 | 36 | Twenty-one retypes, zero drives. |
| `0.5.15-dev` after T5-066 | 29 | `36 − 7`. |
| `0.5.16-dev` after T5-067 | 27 | `29 − 2`. |
| `0.5.17-dev` closing records | 27 | No code movement. |

T5-065’s 21 retypes recompute as:

`7 ConsensusAnswersBeforeScript + 9 FacetNeedsADifferentTransaction + 2 NoIndependentCovenantClause + 2 NoAdmittedShapeCarriesTheFault + 1 NoAdmittedRepresentationCarriesTheField = 21`.

### Final 108-row census

Including zero buckets:

`34 FP-discharged + 0 FP-undischarged + 27 native-required + 24 native-run + 16 native-refusal + 1 determinism + 3 FP-fact + 0 infrastructure + 2 report + 1 vocabulary-closed + 0 experimental = 108`.

The 27 required rows are complete and consistent with the phase enumeration:

- 9 different-transaction: issuance, destruction, second offsetting flow, sponsor/protocol overlap, sponsor change in protocol range, any root, burn/specialized event, omitted transition certificate, receipt/sponsor exchange.
- 5 no-admitted-shape: amount outside domain, ash/time-locked routing, two sponsor envelopes, foreign sponsor asset, unclassified sponsor member.
- 3 non-separating: copied commitment, wrong coordinator, member/coordinator leaf exchange.
- 2 no independent covenant clause: unclassified `u`, sponsor-or-fee-role `u`.
- 2 no admitted representation field: malformed surjection proof, output claimed through two flows.
- 6 singles: paired projection, key-path escape, malformed control path, confidential sponsor values, witness reorder, target bytes changed after ABI validation.

Source: `live_negative_half.rs:277-496`.

### 480-cell form census

Axes: `2 × 4 × 2 × 6 × 5 = 480`, declared at `shape_census.rs:1205-1562` and checked for totality/uniqueness at `3695-3723`.

Status sum:

`282 + 115 + 49 + 13 + 12 + 8 + 1 = 480`.

The full expected map is pinned at `shape_census.rs:4362-4424`. `unsupported-here` is absent, and `480 − 282 = 198` cells are in-space.

T5-063 role facts also hold:

- transcript codes 7 and 8: `confidential_fixture.rs:459-475`;
- code sequence pinned as `[1,2,3,4,5,6,7,8]`: `confidential_fixture_tests.rs:1110-1132`;
- programless explicit change still gets `OutputProgramEmpty`, and an undeclared solver still gets `BalancingRoleNotUnique { found: 0 }`: `confidential_fixture_tests.rs:1240-1279`, `1387-1405`.

### Separator distinctness

| Row | Range | Shape |
|---|---|---|
| wrong explicit asset | `90..122` | `(2,2)` |
| confidential asset commitment | `167..168` | `(2,2)` |
| total below | `130..131` | `(2,2)` |
| total above | `208..209` | `(2,2)` |
| private output omitted | `88..245` | `(2,1)` |
| hidden output added | `88..245` | `(2,3)` |
| source omitted | `5..80` | `(1,2)` |

All `C(7,2) = 21` comparisons are unequal. The shared `88..245` range separates by shape. Constants: `live_owner_signing_negatives.rs:1916-1954`.

Cross-chain checks:

- prior wrong-blinder: `(81..114, (1,2))`;
- prior private-CT imbalance: `(215..248, (1,2))`;
- prior control: `1af38f…`, versus new control `40cb6c…`.

The raw ranges overlap some new coordinates, but no exact `(range, shape)` tuple matches, and the accepted controls differ. Verdict text does not separate them; both are `bad-txns-in-ne-out`.

## 5. Pass walls and deviations

Measured read-command wall, using `/usr/bin/time` at 0.01-second resolution:

| Pass | Wall |
|---|---:|
| Materials bundle, full 355-line read | `0.00s` |
| History, stat/numstat and full ten-file diff | `0.21s` |
| A — arithmetic and claim/code recomputation | `0.01s` |
| B — attribution, ordering and cross-chain comparison | `0.01s` |
| C — determination grounding | `0.01s` |
| D — 15-row exit audit | `0.01s` |
| E — Rust/test quality and leaf audit | `0.04s` |
| F — records, labels and prose | `0.02s` |
| Final HEAD/cleanliness check | `0.00s` |

These are command-execution walls; the interface does not expose model deliberation time.

Verification obeyed the brief: no toolchain, build, network, SSH, or external Elements-tree access. No files or git state were changed. Final worktree remained clean.

Deviations from the brief: **NONE**.

Command anomalies, reported rather than hidden: one read extraction used the wrong fixture-test path and exited 2 at `0.00s`; it was immediately rerun against the correct path. One quoted aggregate-count loop produced blank output with exit 0; it was replaced by five direct `git grep` counts. No review step or evidence was skipped.