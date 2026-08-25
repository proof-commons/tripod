# Confidential-funding execution guide, Wave 7 closeout · `sec:ctf-w7:closeout`

The shape wave's exit record, authored here because it is closed on the day it is written. It states what one execution against a real node observed and what was claimed from it. It is history and never authority: it says what passed on a named tree at a named time, and it does not override the backlog, an ADR, or a package contract.

The machine-checked form of this record is `vectors::live_closeout::wave_seven_closeout`, whose invariants are refused rather than asserted. Where this document and that function disagree, the function is right, because it is the one a test can fail.

## Disposition · `rem:ctf-w7:disposition`

TYPED STOPPED, at step six of the mandatory restart order, on `SponsorEnvelopeSignerAbsent` — a residual this guide does not clear and this wave does not touch. Five of the seven steps accepted, and the fifth is this wave's own deliverable: the step the previous wave stopped at is now taken.

The frontier moved from step five to step six. That is the whole of the wave's advance, and it is stated as one step rather than as a clearance of everything downstream.

## The constructor absence, and what it actually was · `rem:ctf-w7:constructor`

The previous wave typed step five `MultiOutputShapeConstructorAbsent` and named the manifest builder as the missing piece. That was true and it was not the whole of it. The deepest layer was the fixture registry's own bounded parity search: `prefixes_match` compared the target's admitted commitment-prefix pair against a manifest's derived openings BY LENGTH, so a manifest of any width but two matched no counter and exhausted the search after four thousand and ninety-six attempts. A three-output fixture did not fail fast — it spun through the whole bound and then refused.

Nothing in the target contract says a confidential transaction has two outputs. The cardinality was the conformance file's own assumption, and it was found by running rather than by reading: the first verification lane against a three-output fixture ran for minutes and had to be stopped.

The repair keeps the two-output case bit-for-bit. A two-output fixture is still held to the admitted pair in FIXED ORDER — first output at the first prefix, second at the second — which is what makes the dual-parity predecessor carry one of each parity rather than one of them twice; a test asserts exactly that, so a predecessor carrying one parity twice would still fail. A wider fixture is held to the only rule the contract states about it: every commitment carries one of the two admitted prefixes. The consequence is written into the code rather than hidden: for a wider fixture the search degenerates to a well-formedness check that the first counter satisfies, and no discriminating power is claimed where none exists.

## The restart order, step by step · `tab:ctf-w7:order`

| Step | What it asked for | Result |
|---|---|---|
| 1 | one accepted sponsorless private one-to-one control | ACCEPTED at `1af38f8a…b368b89e8e` |
| 2 | both predecessor commitment parities in complete accepted successors | ACCEPTED: prefix `0x08` at step one's identity, prefix `0x09` at `2e93c863…d2b85cc8` |
| 3 | target CT conservation against a balance-valid control | ACCEPTED — the conserving half at the accepted control, the non-conserving half the wrong-blinder mutant's balance-layer refusal |
| 4 | wrong-blinder, missing-rangeproof, malformed-rangeproof | ACCEPTED — all three refused at consensus-rejection-before-script, each attributed by its mutated field |
| 5 | the remaining positive private shapes | ACCEPTED — three shapes built and accepted, each moving its own row; private-merge unconstructible and unmoved |
| 6 | sponsor cases | TYPED STOP — `SponsorEnvelopeSignerAbsent`, the independent signer dependency this guide does not close |
| 7 | disclosure-minimality pairs | NOT REACHED — it follows step six in the mandatory order |

## The three shapes, as the node answered them · `tab:ctf-w7:shapes`

All three ran serialized against one pinned node on a disposable development chain the run created and destroyed. Each carries a readback witness verified against an independently recomputed message, and each reports the bytes the node returned as the bytes it was handed.

| Shape | Inputs | Outputs | Accepted identity | Submitted bytes | Range proofs | Wall |
|---|---|---|---|---|---|---|
| private-split | 1 | 3 | `aa9f2693…76cdb40d` | 13499 | 4174 × 3 | 12.9 s |
| private-many-to-many | 2 | 3 | `3b61056a…0da2f57e12` | 13882 | 4174 × 3 | 13.9 s |
| private-several-distinct-owners | 2 | 2 | `cfabc99a…d863544f95` | 9519 | 4174 × 2 | 15.6 s |

The three successor fixture digests differ, which is how the record says these are three runs rather than one reported three times.

The many-to-many case is a REPRESENTATIVE and is named as one. It is the smallest case whose input and output counts both exceed the one-to-one control's, so it is not a one-to-many or a many-to-one under another name. No claim is made about any other cardinality.

The several-distinct-owners case is about its input OWNERS rather than its cardinality: the predecessor pays its two outputs to two published owners' private receipt constructors, and consuming both makes a transfer whose inputs have distinct owners, each input carrying the leaf its own position executes.

## The one-output shapes, and where they stop · `tab:ctf-w7:one-output`

Three shapes in the catalogue have a single output, and all three hit the same wall in the same place. The registry counts outputs before it inspects any of them, so the refusal is a cardinality refusal whatever the output is.

| Shape | Refusal | Where |
|---|---|---|
| private-merge | `OutputSetTooSmall { found: 1 }` | the registry's two-output floor |
| strict one-to-one (one in, one out) | `OutputSetTooSmall { found: 1 }` | the same floor |
| fee-only (one in, one fee output) | `OutputSetTooSmall { found: 1 }`, and independently `OutputProgramEmpty` | the floor, and the empty output program |

The fee-only case is unconstructible a second and deeper way, which is recorded because a reader who saw only the cardinality refusal might think a relaxed floor would admit it. A fee output carries an empty scriptPubKey, and the confidential fixture vocabulary has no fee role and refuses an empty output program; a two-output probe whose balancing output has an empty program is refused `OutputProgramEmpty` rather than admitted. Even were the floor relaxed, a fee output is inexpressible in this vocabulary.

Neither refusal is a target verdict. Both are first-party construction refusals reached before any node is asked, and they are recorded as such.

## The row delta · `tab:ctf-w7:delta`

Six of the ten positive private classes are answered, each on an observed acceptance of its own shape. Three were answered by earlier waves and three are this wave's.

| Class | Before | After | The acceptance that moved it |
|---|---|---|---|
| one-to-one | native run observed | unchanged | `1af38f8a…b368b89e8e` |
| both commitment parity forms | native run observed | unchanged | `2e93c863…d2b85cc8` |
| target CT conservation | native run REQUIRED in the matrix, moved in the previous closeout | native run observed | `1af38f8a…b368b89e8e` |
| split | native run required | native run OBSERVED | `aa9f2693…76cdb40d` |
| many-to-many representative | native run required | native run OBSERVED | `3b61056a…0da2f57e12` |
| several distinct owners | native run required | native run OBSERVED | `cfabc99a…d863544f95` |
| merge | native run required | unchanged, and structurally unconstructible | — |
| deterministic public fixture openings | native run required | unchanged — see below | — |
| projection equality with paired explicit | native run required | unchanged, its pair not reached | — |
| private sponsor values | native run required | unchanged, and MAY NOT MOVE | — |

The pre-sighash delta is zero and has to be, because the external closure was already recorded when this wave began.

### A defect this delta repairs · `rem:ctf-w7:matrix-repair`

The target-ct-conservation row moved in the previous wave's CLOSEOUT and did not move in the MATRIX: the closeout delta named three rows while `observed_row_acceptance` carried two arms, so the two artifacts disagreed about whether a row had moved. A row that has moved in one artifact and not the other is a row nobody is checking. The matrix now cites the previous wave's own run of record for that row. This is the matrix catching up with an observation already made, not a new claim on a new run.

## Deterministic public fixture openings, and why it did not move · `rem:ctf-w7:determinism`

The row is satisfied by the byte-identity contract (§6.7), which asks that a run recompute a fixture from its manifest and compare. This wave produces that observation as a first-party test: registering the same three-output manifest twice yields the same digest and the same per-output value blinders byte for byte.

It does not move the row, and the ground is the delta's own rule. A row moves on an observed ACCEPTANCE and the delta carries the identity the TARGET computed. A determinism fact over fixture openings has no such identity — no target was asked and none accepted anything — so filing it in a vocabulary whose members are target-computed identities would be the one error a run of record exists to prevent. The row therefore stays unmoved with its evidence recorded beside it rather than moved on evidence of a different kind. Whether the row's gate should be an acceptance at all, given §11.2 lists its gate as the byte-identity contract rather than an acceptance, is a question for the guide's owner and is reported rather than decided here.

## Step seven, and an ordering question for the orchestrator · `rem:ctf-w7:step-seven`

The mandatory order's preamble states that "The order is mandatory, and each step's entry is the previous step's observed result", and its seventh entry reads "disclosure-minimality pairs last, only after both sides of each pair accept". Its sixth reads "sponsor cases, only after their independent signer dependency closes by observation".

Read together, step seven's entry condition is step six's observed result, and step six has none: its dependency has not closed and this guide does not close it. Step seven is therefore not reached. The repository encodes this rather than merely restating it — `RestartLedger::record` refuses a step recorded after a stop with `RestartOrderRefusal::AfterStop`, whose own words are that the later run "is not evidence about its own subject, because its entry condition never held" — and a test asserts that recording the minimality pairs after the sponsor stop is refused exactly so.

There is a genuine tension worth a ruling, and it is REPORTED and not ruled here. Step seven's own clause names only "both sides of each pair accept" and says nothing about sponsors; on that clause alone, a non-sponsor minimality pair whose two sides both accepted would satisfy its stated condition. It is the preamble's chain rule, not step seven's own text, that makes step six a gate. And step six is not merely unrun but CONDITIONAL — it runs "only after" a dependency that may never close for this guide — so a strict chain reading makes step seven permanently unreachable rather than merely deferred, which is a stronger consequence than the guide's text anywhere states it intends. Whether the chain rule or step seven's own clause governs is the guide owner's question. This wave took the strict reading, ran no pairs, and moved no minimality row.

## Residuals · `tab:ctf-w7:residuals`

Cleared by this guide, exactly one: `NoConfidentialPredecessorCanBeFunded`, unchanged from the previous wave. Still carried, unchanged: `SponsorEnvelopeSignerAbsent` and `PredecessorConstructorAbsent`.

`MultiOutputShapeConstructorAbsent` is NOT in the cleared set, and the omission is the site's own rule rather than an oversight. It was never a member of `carried_residuals`, which holds two. It lived in the previous wave's ledger as the fifth step's stop, and the way a constructor absence leaves is that the step it stopped is recorded accepted instead — which is what this wave's ledger does. A closeout that also listed it as cleared would be counting one clearance twice, and `validate_closeout` refuses a cleared set holding more than one member.

## Divergences, reported and not repaired · `tab:ctf-w7:divergences`

| What | Where | Disposition |
|---|---|---|
| The guide's merge predicate — inputs ≥ 2 and outputs = 1 — is unsatisfiable against the registry's two-output floor, because a confidential balance needs a balancing output. | §5.3 and §15.2 against the §6 registry rule | Filed as an erratum in the Guide-13 feature-request register, scoped to that conflict only. Neither rule relaxed; merge stays unmoved as structurally unconstructible rather than as work deferred. |
| The fixture registry's parity search assumed exactly two outputs, so no wider fixture could derive. | the conformance registry against its own manifest builder, which had always accepted wider sets | REPAIRED in the owning module, the two-output rule preserved bit-for-bit and both halves held by tests. Reported prominently because it was a real defect found by running. |
| The deterministic-public-fixture-openings row's §11.2 gate is the byte-identity contract, but the delta admits only target-computed identities. | §11.2 against the delta's own rule | Reported. The determinism observation is produced and recorded; the row stays unmoved because it has no acceptance to cite. |
| Step seven's entry condition is step six's observed result by the order's preamble, while step seven's own clause names only its pairs' acceptance — and step six is conditional on a dependency that may never close for this guide. | §10.5 preamble against §10.5 step 7 | Reported as a conflict in the order's own text. The strict reading was taken: no pairs run, no minimality row moved. |
| The shared worker lane runs the ignored native tests in parallel, which the instance does not sustain at this many concurrent nodes. | the lane script against the instance | The verdict was taken through a one-off serialized lane script beside it, with `--test-threads=1`; the shared script was not edited. |

## Non-claims carried by the result · `rem:ctf-w7:non-claims`

All ten, unchanged, and carried by the record rather than by this prose. In particular: six accepted private transfers do not establish privacy, do not establish production custody or cryptography, and do not substitute for the four rows that did not move. A representative many-to-many is a representative and not a proof over cardinalities. Three accepted shapes say nothing about the sponsor row, the minimality relation, or any resource result.

## Target facts · `tab:ctf-w7:target`

Elements Core v28.99.0-b7fc5d080a7e, at the tip the lane pins itself to, on disposable development chains each run created and destroyed. The predecessor fixture digest is `ca43b210…b9d51fc2` and both predecessor coins matched the ceremony's expectation on every run, at prefixes `0x08` and `0x09`. The three shape runs took 12.9, 13.9, and 15.6 seconds of wall time, serialized; the whole ignored native lane was 11 of 11 green in 133.8 seconds.

## For the next wave · `rem:ctf-w7:handoff`

Consume `wave_seven_closeout` without reopening any ruling. What is available to build on: six answered positive private rows, each on an acceptance of its own shape; a multi-output and multi-input fixture constructor that derives at any width, with the two-output dual-parity rule intact; and a restart ledger whose entry conditions are met through step five. What is not available: the sponsor row, any minimality relation, the merge shape, and any resource result. The two open questions handed up are the deterministic-fixture-openings gate and the step-six/step-seven ordering, both reported and neither ruled.
