# Consensus Shape Possibility · `ref:shapes:census`

This register records which blinded transaction shapes the confidential live lane can take, on what evidence each verdict rests, and where a shape consensus admits is refused by this workspace's own conventions rather than by the protocol.

It exists because two very different facts had been collapsing into one word. A shape this workspace does not build because the target's balance rule cannot admit it, and a shape it does not build because its own fixture registry declines to express it, were both being described as unconstructible. The second is a convention this repository chose and could unchoose; the first is arithmetic nobody votes on. Recorded under one name, a local convention reads as a law of the protocol, and the workspace loses the ability to tell which of its walls it is allowed to move.

So every row carries two verdicts, computed and cited separately: what consensus admits, and what this workspace does about it. Where they disagree, the row carries the limitation and the named path that would end it.

The machine-checked form of this register is the `shape_census` module of the vectors package, whose tests recompute every refusal against the live registry rather than asserting a remembered one. This document is the prose half; neither is the summary of the other, and where they could drift the tests are what fails.

## Discipline · `rule:shapes:discipline`

A shape's consensus verdict is stated in one of three evidence classes, and the class is part of the claim rather than a note about it. A derivation may never be reported as an observation, however confident the derivation is.

A first-party refusal is recorded with the exact typed refusal the registry returns, recomputed by driving the registry, never transcribed from a comment.

Every shape consensus admits and this workspace refuses carries a limitation section: what refuses it, why the convention exists, and the named removal path. A refusal recorded without its reason gets defended later as though it were consensus, which is the failure this register was built to prevent.

Each limitation section carries its revision surface: what removing or revising it would have to touch. The tests fail when the refusal moves; the revision surface is the wider answer no test can give — the prose that argues the convention, the vocabularies whose members rest on it, and the guide clauses written on the assumption it holds.

This register is not evidence. It moves no matrix row, no blocker and no residual, and it records no run. A shape consensus admits and nobody has submitted is recorded source-derived, and stays that way until somebody runs one.

## The balance rule the verdicts rest on · `rule:shapes:tally`

Elements checks value conservation as a Pedersen tally over commitments. At the pinned tip b7fc5d080a, src/confidential_validation.cpp:73-81 runs secp256k1_pedersen_verify_tally over every input commitment against every output commitment, queued at :363-366.

An explicit value does not sit outside that sum. It is committed at :345-349 with an all-zero blinder and joins the same tally, which is the whole reason the derivations below work at all.

Write a commitment as v\*H + r\*G, for value v under blinder r. The tally holds exactly when both coordinates balance: the values sum equal, and the blinders sum equal. The second half is what decides shape possibility, because an explicit output contributes r = 0 and can never absorb a blinder.

The whole consensus question therefore reduces to a single predicate: does the output set contain at least one blinded output? If it does, that output's blinder can be set to whatever closes the sum, and the shape is possible. If every output is explicit, the output blinder sum is fixed at zero, and the shape is possible only where the input blinder sum is zero too — which for a genuinely blinded input it is not.

A fee output is mandatorily explicit. CTxOut::IsFee, at src/primitives/transaction.h:324-327, holds only for an output with an empty scriptPubKey and an explicit value and asset. A fee output therefore always contributes a zero blinder and can never be the output that absorbs the input blinder sum.

A fee output is not mandatory in the other direction, and this is an observation rather than a derivation: every accepted identity this register cites was built sponsorless and carries no fee output at all, and every one of them was accepted into a block.

One further clause is worth recording because it closes an escape a reader might otherwise imagine. A zero-value explicit output is admitted only where its scriptPubKey is unspendable (:332-341); an empty script is not unspendable, so a zero-value fee output is refused outright. A fee output that balances by carrying nothing is not available.

## Evidence classes · `tab:shapes:evidence`

| Class | What it means | What it may cite |
|---|---|---|
| Observed-accepted | A node accepted a transaction of this shape into a block | A run-of-record identity the target computed |
| Source-derived-possible | The balance rule admits the shape; nobody has submitted one | The tally arithmetic and the pinned source, and explicitly not a run |
| Source-derived-impossible | The balance rule cannot admit the shape | The tally arithmetic and the pinned source |

The middle class is the one that needs the discipline. No node has been offered a transaction of any shape in it, and the register says so in the class name rather than trusting a footnote to survive.

## The enumeration and why it is closed · `rule:shapes:closure`

The shape axis is the pair (blinded inputs, outputs), together with whether one of those outputs is the mandatorily explicit fee output. Nothing else about a transaction changes either verdict.

The enumeration is the small-shape window: at least one blinded input and at most two, one to three outputs, plus the two fee-bearing members that window does not otherwise reach. Eight shapes.

It is closed, and a reader can see that nothing is missing, because of what the tally rule shows: the consensus verdict depends on nothing whatever but whether at least one output is blinded. It does not depend on the input count. It does not depend on the output count beyond the difference between some blinded output and none. Both cases already appear in the window — seven members are the first, and fee-only is the second — so every shape outside the window inherits the verdict of the case it falls into, and adding it would restate a row rather than add one.

The window is chosen for the first-party half, where the counts do matter: the registry's own clauses are cardinality clauses, and every shape this lane has built or been refused falls inside it.

## Index · `tab:shapes:index`

| Shape | Consensus verdict | Evidence | First-party status | Limitation |
|---|---|---|---|---|
| One blinded input to one blinded output | Possible | Source-derived | Refused | (`rule:shapes:two-output-floor`) |
| One blinded input to one blinded output beside a fee output | Possible | Source-derived | Refused | (`rule:shapes:absent-fee-role`) |
| One blinded input to two blinded outputs | Possible | Observed | Constructible and observed | none |
| One blinded input to three blinded outputs | Possible | Observed | Constructible and observed | none |
| Two blinded inputs merged to one blinded output | Possible | Source-derived | Refused | (`rule:shapes:two-output-floor`) |
| Two blinded inputs to two blinded outputs | Possible | Observed | Constructible and observed | none |
| Two blinded inputs to three blinded outputs | Possible | Observed | Constructible and observed | none |
| One blinded input to a fee output and nothing else | Impossible | Source-derived | Refusal guards consensus | (`rule:shapes:fee-only`) |

Both limitation tags in the last column are cited above in a parenthesized group, because each is minted at its own section below and a bare occurrence here would be a second mint of the same name.

Four of eight shapes have been run and accepted. Four have not, and none of those four has ever been offered to a node.

The one-to-two row carries the first of two identities the lane recorded for that shape; the second, is the same shape spending the opposite commitment parity, and the register cites one because a shape needs one acceptance and not because the other is doubted.

## The two-output floor · `rule:shapes:two-output-floor`

**Refused.** The confidential fixture registry refuses any manifest stating fewer than two outputs, with the typed refusal OutputSetTooSmall carrying the count found. This refuses the strict one-to-one and the private merge, both of which consensus admits.

**Refused at.** packages/target-elements-conformance/src/confidential_fixture.rs, the outputs.len() < 2 clause of register_with_source. It is the first clause after the handle, duplicate and material-class checks, and it runs before any output is inspected — so it refuses a one-output manifest whatever that output is.

**Convention.** The balancing-output model. A manifest names exactly one balancing output whose blinder is solved to close the tally, and the registry's parity discipline searches over the freely chosen blinders of the others. With one output there is no other, so the model has nothing to search.

That is a fact about the model and not about the arithmetic. A lone output's blinder is fully determined by the input blinder sum, and determining it is exactly what solving means. The floor is therefore a construction-model convention, not a consensus rule, and the register records it as one.

**Removal path.** Extend the registry with a single-output fully-solved balancing form: a manifest whose one output is balancing and carries no freely chosen blinder, taking the input blinder sum directly. The parity search then has nothing to search and degenerates to a well-formedness check, which the code should say plainly rather than claim a discriminating power it would not have. This admits both the strict one-to-one and the private merge, and the two-output case stays bit-for-bit as it is.

**The degeneracy the removal must name.** The forced blinder can be zero, and then it hides nothing. A merge's single output takes the sum of the consumed coins' blinders, so merging the two halves of this repository's inverse-pair dual-parity predecessor — whose blinders cancel by construction, which is what makes it an inverse pair — forces that sum to zero. The output commitment is then exactly v\*H: a point anyone can recompute from a guessed value, carrying a blinded output's form and none of its hiding. The form is still sound and the tally still balances; what fails is confidentiality, silently. A predecessor whose blinders do not cancel avoids it, so the removal must either require a non-canceling predecessor or refuse a solved zero blinder outright. Filing the form without this warning would file a confidentiality hole as a feature.

**This floor guards consensus only by accident.** It counts outputs. It refuses the consensus-impossible fee-only shape and the perfectly possible merge with the same message and the same indifference, which the census tests hold by driving both and comparing the two refusals. A reader who took the refusal as a consensus verdict would be wrong about one of the two, and the register says which.

**Revising this limitation touches:** this section; the registry clause and its typed refusal, together with the parity search whose degeneration the new form has to state; the fixture manifest vocabulary, which today cannot express a zero-free-output manifest; the census module's rows for the one-to-one and merge shapes and the four tests that recompute them; the ceremony test in the multi-shape module that records the one-output floor as the merge wall; the merge erratum in the Guide-13 feature-request register, whose ground is this section; the positive private class vocabulary's merge member and whatever records it as unconstructible; and every backlog paragraph that describes the one-output shapes as typed at the registry floor, which is true only while the floor stands.

## The absent fee role · `rule:shapes:absent-fee-role`

**Refused.** The fixture output role vocabulary has no fee member, and every fixture output must carry a nonempty output program. A fee output carries an empty scriptPubKey by the target's own definition of a fee, so it is refused OutputProgramEmpty naming the output. This refuses the one-to-one-with-fee shape, which consensus admits.

**Refused at.** packages/target-elements-conformance/src/confidential_fixture.rs, the output_program.is_empty() clause of register_with_source. It runs per output, after the cardinality and balancing-role clauses, which is why a two-output fee-bearing manifest reaches it at all.

**Convention.** The absent fee role. The vocabulary has no member for an output that is explicit, unspendable, and outside the blinder solve, so the shape is inexpressible rather than rejected — nothing in the registry ever decided against fee outputs, and that is the point: no decision was recorded because none was made.

A second face of the same absence shows in the shared manifest builder, which casts the final output as the balancing one. The builder has no way to say explicit and outside the solve, so the one output that must never balance arrives cast as the output that does.

**Removal path.** Add a fee member to the fixture output role vocabulary: explicit-valued, held out of the blinder solve at a zero blinder, and required to carry an empty output program rather than merely permitted one, so the role is checked and not just excused from the nonempty-program clause. The clause then reads on the role instead of on every output alike, and the balancing role stops being assigned by position.

**Revising this limitation touches:** this section; the output role vocabulary and every match over it; the empty-program clause and its typed refusal; the balancing-role uniqueness clause, which today counts a positional assignment; the shared multi-output manifest builder's last-output-balances rule; the census rows for the fee-bearing shapes and the tests that recompute them; the ceremony test recording a fee output as inexpressible beyond the cardinality floor; and the observation in this register that no accepted identity carries a fee output, which stops being a statement about what the lane happens to build and becomes one about what it chooses to.

## The one impossible shape · `rule:shapes:fee-only`

**Impossible.** A transaction whose only output is the fee output cannot balance for a nonzero input blinder sum. The fee output is mandatorily explicit and contributes a zero blinder, so the output blinder sum is zero while the input blinder sum is not, and the tally at src/confidential_validation.cpp:73-81 fails.

This is source-derived and not observed. No node has refused one of these, because none was ever built; the ground is the arithmetic, not a verdict.

**The escapes both change the shape.** A zero-blinder input would balance, but an input whose blinder is zero is not a blinded input and the shape is a different one. An added blinded dummy output would balance, and is then the one-to-one-with-fee shape under another name. Neither is the fee-only shape, and recording them as escapes rather than as solutions is the honest form.

**What refuses it here, and why that is not the reason.** The registry refuses it OutputSetTooSmall — the cardinality floor, which fires before any output is inspected and never reaches the empty program at all. The refusal is correct and its reasoning has nothing to do with consensus. Were the floor removed tomorrow by the path recorded above, this shape would need the fee role to be expressible and would still be impossible, and the wall that stopped it would have moved without anybody deciding it should.

**Revising this section touches:** nothing in the registry, because nothing in the registry is what makes this shape impossible. It touches this section and the census row alone, and would only ever be revised by a consensus change to how explicit outputs enter the tally.

## What the register does not claim · `rem:shapes:non-claims`

It does not claim that the four source-derived-possible shapes would be accepted. It claims the balance rule does not forbid them. A transaction has to satisfy range proofs, script validity, policy and relay besides, and none of that is in scope here.

It does not claim the four observed shapes exhaust what has been run; it claims each cites an acceptance of its own shape.

It does not claim the removal paths are scheduled, designed in detail, or agreed. They are named, which is what the ruling asks for at this stage, and filed in the feature-request register. No part of any of them is taken here.

It does not extend to issuance, which is excluded (`rule:exclusions:issuance-bytes`), nor to sponsored shapes, whose signer dependency this workspace does not close.
