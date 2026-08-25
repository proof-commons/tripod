# Confidential-funding execution guide, Wave 6 closeout · `sec:ctf-w6:closeout`

The follow-up evidence wave's exit record, authored here because it is closed on the day it is written. It states what one execution against a real node observed and what was claimed from it. It is history and never authority: it says what passed on a named tree at a named time, and it does not override the backlog, an ADR, or a package contract.

The machine-checked form of this record is `vectors::live_closeout::wave_six_closeout`, whose invariants are refused rather than asserted. Where this document and that function disagree, the function is right, because it is the one a test can fail.

## Disposition · `rem:ctf-w6:disposition`

TYPED STOPPED, at step five of the mandatory restart order, on the ground that the remaining positive shapes need multi-output and multi-input fixtures this wave does not build. Four of the seven steps accepted, and steps one through four are the interlock ruling's own deliverable: the two steps Wave 5 accepted, re-run on fresh chains, plus the two steps Wave 5 stopped before.

## The Wave-5 mis-typing this corrects · `rem:ctf-w6:correction`

Wave 5 stopped at step three and typed the stop `LiveInfrastructureBlocker::NoAcceptingControlExists`. That blocker is derived while no positive control exists, and one did — the ledger's own first entry was an accepted control, and `a_positive_control_exists()` computes to true, `carried_residuals()` does not hold the blocker, and `has_accepted_control()` returns true. The stop was mis-typed. This wave records step three as accepted, on the orchestrator's interlock ruling: §10.5 step 3 asks for CT conservation recorded against a balance-valid control and §11.2's gate is a balance-valid accepted control, neither requiring a refused case, so the conservation record was reachable at step three all along. The mis-typing is named here rather than deleted, because a record that quietly dropped it would leave the correction unauditable.

## The restart order, step by step · `tab:ctf-w6:order`

| Step | What it asked for | Result |
|---|---|---|
| 1 | one accepted sponsorless private one-to-one control | ACCEPTED at `1af38f8a…b368b89e8e`, re-run on a fresh chain, reproducing the prior identity byte for byte |
| 2 | both predecessor commitment parities in complete accepted successors | ACCEPTED: prefix `0x08` at step one's identity, prefix `0x09` at `2e93c863…d2b85cc8`, both re-run |
| 3 | target CT conservation against a balance-valid control | ACCEPTED — the conserving half at the accepted control, the non-conserving half the wrong-blinder mutant's balance-layer refusal |
| 4 | wrong-blinder, missing-rangeproof, malformed-rangeproof | ACCEPTED — all three submitted to the mempool boundary and refused at consensus-rejection-before-script, each attributed by its mutated field |
| 5 | the remaining positive private shapes | TYPED STOP — the shapes need fixtures this wave does not build, and private-merge is structurally unconstructible |
| 6 | sponsor cases | NOT REACHED, and not this guide's to run: the signer dependency `SponsorEnvelopeSignerAbsent` is not closed and this guide does not close it |
| 7 | disclosure-minimality pairs | NOT REACHED |

Steps six and seven are recorded as not reached rather than omitted.

## The one observed run that is both of step three's halves · `rem:ctf-w6:disclosure`

Step three's conservation record has two halves: an accepted conserving control, and a refused non-conserving case. The non-conserving case is the wrong-blinder mutant, and the wrong-blinder mutant is step four's first proof-negative. These are the SAME observed run, not two. The pinned target refuses the wrong-blinder mutant at the balance layer — the balance check at `src/confidential_validation.cpp:364` is queued before the range-proof loop at `:368`, and the mempool path at `src/validation.cpp:1097` runs each queued check inline in source order — so one submission is at once conservation's non-conserving half and the fourth step's wrong-blinder case. Both step three's and step four's ledger entries disclose this in words, so a reader of two accepted entries does not double-count the one run as two observations. This disclosure is the condition on which the interlock ruling rests.

## The three proof-negatives, attributed by field · `tab:ctf-w6:negatives`

All three returned the one identical refusal, so attribution is by the construction field each moved and not by a target layer the three could differ on. The undischargeability of the guide's "each attributed to its own layer" wording is filed as an erratum against the guide, not repaired here.

| Case | Mutated field | Declared byte range | Node's words | Layer |
|---|---|---|---|---|
| wrong-blinder | the output's 33-byte value commitment | 81..114 | `bad-txns-in-ne-out` | consensus-rejection-before-script |
| missing-rangeproof | the output-witness range-proof bytes | 781..4958 | `bad-txns-in-ne-out` | consensus-rejection-before-script |
| malformed-rangeproof | the output-witness range-proof bytes | 781..4958 | `bad-txns-in-ne-out` | consensus-rejection-before-script |

The wrong-blinder mutant recomputes the output's value commitment under a blinder that is not its own — a valid Pedersen commitment whose blinding factor does not close the balance — and does not regenerate the range proof, so its change is confined to the 33-byte commitment field. The two range-proof mutants leave the commitments untouched, so the balance check passes and the range-proof check is the one that fails.

## The row delta · `tab:ctf-w6:delta`

Three of the ten positive private classes moved, each on an observed acceptance of its own shape. The target-ct-conservation row is the one Wave 5 stopped before; this wave moves it.

| Class | Before | After | The acceptance that moved it |
|---|---|---|---|
| one-to-one | native run observed (Wave 5) | re-run observed | `1af38f8a…b368b89e8e` |
| both commitment parity forms | native run observed (Wave 5) | re-run observed | `2e93c863…d2b85cc8` |
| target CT conservation | native run required | native run observed | `1af38f8a…b368b89e8e`, conserving control beside the wrong-blinder refusal |
| split | native run required | unchanged | — |
| merge | native run required | unchanged, and structurally unconstructible | — |
| many-to-many representative | native run required | unchanged | — |
| several distinct owners | native run required | unchanged | — |
| deterministic public fixture openings | native run required | unchanged | — |
| projection equality with paired explicit | native run required | unchanged | — |
| private sponsor values | native run required | unchanged, and MAY NOT MOVE | — |

The pre-sighash delta is zero and has to be, because the external closure was already recorded when this wave began.

## Residuals · `tab:ctf-w6:residuals`

Cleared by this guide, exactly one: `NoConfidentialPredecessorCanBeFunded`. Still carried, unchanged: `SponsorEnvelopeSignerAbsent` and `PredecessorConstructorAbsent`. The proof-negatives cleared no residual, and the closeout refuses a report that says they did.

## Divergences from the guide, reported and not repaired · `tab:ctf-w6:divergences`

| What | Where | Disposition |
|---|---|---|
| §10.5 step 4's "each attributed to its own layer" is undischargeable: the target emits one identical `bad-txns-in-ne-out` for all three mutations, and the observed-layer vocabulary has a single member covering them. Attribution is by mutated field. | §10.5 step 4 against the target's own refusal | Filed as an erratum in the Guide-13 feature-request register; not repaired, this wave owning neither the guide nor the target. |
| A private-merge is one output, and the fixture registry refuses a manifest with fewer than two, so §14.5's merge predicate — inputs ≥ 2 and outputs = 1 — is unsatisfiable on the confidential live lane. | §14.5 against the §6 registry rule | Reported as a typed divergence, not repaired and not relaxed. Merge stays unmoved as structurally unconstructible rather than as work deferred. |
| The remaining constructible shapes — split, many-to-many, several distinct owners — need multi-output and multi-input fixtures the current construction does not build. | §10.5 step 5 against the current fixtures | Typed as `MultiOutputShapeConstructorAbsent`, a constructor absence like the predecessor one and not a target verdict; a following wave builds the fixtures. |
| The disclosure-minimality pairs (step 7) come after the sponsor step (step 6) in the mandatory order, and the sponsor step does not run, so the order does not reach step 7. | §10.5 order against the sponsor residual | Reported as the order's own consequence; minimality is not reached rather than refused. |

## Non-claims carried by the result · `rem:ctf-w6:non-claims`

All ten, unchanged, and carried by the record rather than by this prose. In particular: three accepted private transfers and three observed proof-negatives do not establish privacy, do not establish production custody or cryptography, and do not substitute for the seven rows that did not move. A malformed private rejection does not replace an accepted control, and the conservation record rests on an accepted control beside the wrong-blinder refusal rather than on the refusal alone.

## Target facts · `tab:ctf-w6:target`

Elements Core v28.99.0-b7fc5d080a7e, at the tip the lane pins itself to, on disposable development chains each run created and destroyed. The predecessor fixture digest is `ca43b210…b9d51fc2`; the accepted control submitted 9136 bytes at commitment prefix `0x08`. The two runs that re-ran steps one and two reproduced Wave 5's identities byte for byte, which is a determinism observation about the ceremony and not a matrix move. Steps one, two, and the conservation-and-proof-negatives run took 12.3, 12.4, and 12.7 seconds of wall time.

## For Guide 14 · `rem:ctf-w6:handoff`

Consume `wave_six_closeout` without reopening any ruling and without inheriting any digest design. What is available to build on: the two accepted receipt-covenant transfers and both commitment parities of Wave 5, now re-run; a recorded target CT conservation against a balance-valid control; three proof-negatives observed at the mempool boundary and attributed by field; and a restart ledger whose remaining steps have their entry conditions met at step five. What is not available: any positive shape beyond the one-to-one control, any minimality relation, and the sponsor row.
