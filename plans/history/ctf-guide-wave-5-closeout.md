# Confidential-funding execution guide, Wave 5 closeout · `sec:ctf-w5:closeout`

The restart wave's exit record, authored here because it is closed on the day it is written. It states what one execution against a real node observed and what was claimed from it. It is history and never authority: it says what passed on a named tree at a named time, and it does not override the backlog, an ADR, or a package contract.

The machine-checked form of this record is `vectors::live_closeout::wave_five_closeout`, whose three invariants are refused rather than asserted. Where this document and that function disagree, the function is right, because it is the one a test can fail.

## Disposition · `rem:ctf-w5:disposition`

TYPED STOPPED, at step three of the mandatory restart order, on the ground that a conservation claim needs a refused non-conserving case beside the accepted conserving one and the non-conserving case is step four's wrong-blinder mutation, which this wave did not run. Two of the seven steps accepted. The guide's own closing statement anticipates exactly this shape of result and calls it the only kind worth carrying forward.

## The restart order, step by step · `tab:ctf-w5:order`

| Step | What it asked for | Result |
|---|---|---|
| 1 | one accepted sponsorless private one-to-one control | ACCEPTED |
| 2 | both predecessor commitment parities in complete accepted successors | ACCEPTED: prefix `0x08` at step one's identity, prefix `0x09` |
| 3 | target CT conservation against a balance-valid control | TYPED STOP — the balance-valid control exists; the refused non-conserving half does not |
| 4 | wrong-blinder, missing-rangeproof, malformed-rangeproof | NOT REACHED |
| 5 | the remaining positive private shapes | NOT REACHED |
| 6 | sponsor cases | NOT RUN, and not this guide's to run: the signer dependency `SponsorEnvelopeSignerAbsent` is not closed and this guide does not close it |
| 7 | disclosure-minimality pairs | NOT REACHED |

Steps four through seven are recorded as not reached rather than omitted. A report whose length depended on how far the run got is one in which a stopped run reads as a shorter complete one.

## The row delta · `tab:ctf-w5:delta`

Two of the ten positive private classes moved, each on an observed acceptance of its own shape and on nothing else. The eight that did not move are named, so a reader counts ten either way.

| Class | Before | After | The acceptance that moved it |
|---|---|---|---|
| one-to-one | native run required | native run observed | `1af38f8a…b368b89e8e` |
| both commitment parity forms | native run required | native run observed | `2e93c863…d2b85cc8`, completing the pair begun at step one |
| split | native run required | unchanged | — |
| merge | native run required | unchanged | — |
| many-to-many representative | native run required | unchanged | — |
| several distinct owners | native run required | unchanged | — |
| deterministic public fixture openings | native run required | unchanged | — |
| target CT conservation | native run required | unchanged | — |
| projection equality with paired explicit | native run required | unchanged | — |
| private sponsor values | native run required | unchanged, and MAY NOT MOVE | — |

Across the whole matrix: twenty-six positive rows, two answered, twenty-four awaiting runs nobody has taken. The pre-sighash delta is zero and has to be, because the external closure was already recorded when this wave began.

## What was minted to make a row movable · `rem:ctf-w5:standing`

Until this wave the matrix could say a row NEEDED a target-native run and could not say that a run had ANSWERED one. Recording an acceptance would have meant filing a target's verdict under a first-party validator's refusal, which is the layer blur the failure-layer rule forbids. `LiveRowStanding::NativeRunObserved` carries the identity the target computed, so the claim is checkable against a chain by somebody who does not trust this workspace.

## Residuals · `tab:ctf-w5:residuals`

Cleared by this guide, exactly one: `NoConfidentialPredecessorCanBeFunded`. Still carried, unchanged: `SponsorEnvelopeSignerAbsent` and `PredecessorConstructorAbsent`. Funding cleared no digest blocker, and the closeout refuses a report that says it did.

## Divergences from the guide, reported and not repaired · `tab:ctf-w5:divergences`

| What | Where | Disposition |
|---|---|---|
| A strict private one-to-one — one confidential input, one confidential output — is not constructible under the guide's own rules. The fixture registry refuses a manifest with fewer than two outputs and the materializer refuses an intent whose destination count differs from its fixture's. | §6 registry rule against §10.5 step 1 | Step one was run under the reading "one receipt consumed, one recipient created, balancing output back to the sender as change", stated in the ceremony module. Reported, not repaired: this wave owns neither rule. |
| §11.1 lists four carried residuals including `SighashProfileUnreviewed`. The repository carries two; that one left by verdict before this wave began. | §11.1 against `carried_residuals()` | Read as a description of entry state rather than a contradiction. |
| The narrower condition blocking the minimality pairs still has no name. After this wave the transaction-wide path EXISTS, so what remains is an unwired call site — an attempt not made, not a component that does not exist — and the blocker vocabulary admits only the latter. | §11.2 against the blocker vocabulary's own rule | NOT MINTED, and reported as not qualifying rather than as overlooked. |
| The `NoObservedTargetVerdict` design tension: its doc promises a run clears it while `MissingResidual` refuses an empty residual set. | owner-authorization census | UNTOUCHED. Those are the sighash guide's authorization cases; this wave's runs do not reach them. |

## Non-claims carried by the result · `rem:ctf-w5:non-claims`

All ten, unchanged, and carried by the record rather than by this prose. In particular: two accepted private transfers do not establish privacy, do not establish production custody or cryptography, do not establish that the stock RPC surface supports the selected representation, and do not substitute for the eight rows that did not move. A malformed private rejection would not have replaced an accepted control, and no malformed private rejection was run.

## Target facts · `tab:ctf-w5:target`

Elements Core v28.99.0-b7fc5d080a7e, at the tip the lane pins itself to, on a disposable development chain each run created and destroyed. The predecessor fixture digest; the two successor fixtures differ, as two runs consuming different receipts must. Each control submitted 9136 bytes and carried two output-witness entries of 4174 range-proof bytes each. Step one's run took 11.5 seconds of wall time.

Worth recording and NOT converted: step one's ceremony reproduced its accepted identity byte for byte across two separate executions. That is a determinism observation about a successor, and the deterministic-fixture-openings row is about fixture openings, so the row did not move on it.

## For Guide 14 · `rem:ctf-w5:handoff`

Consume `wave_five_closeout` without reopening any ruling and without inheriting any digest design. What is available to build on: a private lane with its own transaction-wide finalization entry point, two accepted receipt-covenant private transfers with verified witnesses, both commitment parities exercised, and a restart ledger whose remaining five steps have their entry conditions already met at step three. What is not available: any conservation claim, any negative-case attribution, any minimality relation, and the sponsor row.
