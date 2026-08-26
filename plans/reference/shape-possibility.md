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
| One blinded input to one blinded output | Possible | Observed | Constructible after removal | (`rule:shapes:two-output-floor`), removed |
| One blinded input to one blinded output beside a fee output | Possible | Observed | Constructible after removal | (`rule:shapes:absent-fee-role`), all removed |
| One blinded input to two blinded outputs | Possible | Observed | Constructible and observed | none |
| One blinded input to three blinded outputs | Possible | Observed | Constructible and observed | none |
| Two blinded inputs merged to one blinded output | Possible | Observed | Constructible after removal | (`rule:shapes:canceling-predecessor`), removed |
| Two blinded inputs to two blinded outputs | Possible | Observed | Constructible and observed | none |
| Two blinded inputs to three blinded outputs | Possible | Observed | Constructible and observed | none |
| One blinded input to a fee output and nothing else | Impossible | Source-derived | Refusal guards consensus | (`rule:shapes:fee-only`) |

Every limitation tag in the last column is cited above in a parenthesized group, because each is minted at its own section below and a bare occurrence here would be a second mint of the same name.

Six of eight shapes have been run and accepted. Two have not.

The counts moved because four limitations have now been structurally removed, and the first-party statuses in the fourth column are different things that a single word would have flattened. Constructible and observed is a shape this lane always built. Constructible after removal is a shape it was refused until a convention was removed, and the row keeps citing the removed convention, because a row saying only that a shape works loses the fact that a wall stood there — and a wall nobody remembers is one that gets rebuilt. Submitted and refused is the fee-bearing shape: it was BUILT with a real fee output and OFFERED to a node, and the node turned it away, which is more than nobody having tried and less than an acceptance.

Expressible and unrun is now nobody's status, and the register keeps the words because that is what the fee-bearing shape was until it was offered. A status vacated by work is not a status deleted.

Of the two unrun shapes, one is consensus-impossible and one is the fee-bearing shape, refused at a covenant this workspace wrote rather than at anything the protocol says about fees.

The one-to-two row carries the first of two identities the lane recorded for that shape; the second, is the same shape spending the opposite commitment parity, and the register cites one because a shape needs one acceptance and not because the other is doubted.

## The two-output floor · `rule:shapes:two-output-floor`

**Refused.** The confidential fixture registry refuses any manifest stating fewer than two outputs, with the typed refusal OutputSetTooSmall carrying the count found. This refuses the strict one-to-one and the private merge, both of which consensus admits.

**Refused at.** packages/target-elements-conformance/src/confidential_fixture.rs, the outputs.len() < 2 clause of register_with_source. It is the first clause after the handle, duplicate and material-class checks, and it runs before any output is inspected — so it refuses a one-output manifest whatever that output is.

**Convention.** The balancing-output model. A manifest names exactly one balancing output whose blinder is solved to close the tally, and the registry's parity discipline searches over the freely chosen blinders of the others. With one output there is no other, so the model has nothing to search.

That is a fact about the model and not about the arithmetic. A lone output's blinder is fully determined by the input blinder sum, and determining it is exactly what solving means. The floor is therefore a construction-model convention, not a consensus rule, and the register records it as one.

**Removal path, TAKEN.** Extend the registry with a single-output fully-solved balancing form: a manifest whose one output is balancing and carries no freely chosen blinder, taking the input blinder sum directly. The parity search then has nothing to search and degenerates to a well-formedness check, which the code should say plainly rather than claim a discriminating power it would not have. This admits both the strict one-to-one and the private merge, and the two-output case stays bit-for-bit as it is.

**REMOVED, under T5-041.** The cardinality clause stopped counting outputs and started asking whether a short manifest DECLARES the form, which a new sole-balancing role states. The floor still refuses a lone output that does not declare it, so nothing was relaxed and nothing that registered before registers differently; what changed is that a caller can now ask for the form by name. The declaration rides in the output role rather than in a new manifest member, which is what kept every existing fixture's digest identical — the digest transcript already emits a role code per output, so adding roles perturbs nothing a previous manifest hashed, while a manifest-level field would have moved every registered digest.

The degeneracy was answered by both admitted paths at once rather than by choosing between them. The registry's existing zero-solution refusal was left standing and is now load-bearing: for a sole output the solve returns the input blinder sum unchanged, so the refusal fires exactly when the consumed coins' blinders cancel. And the shape that ran was built from a predecessor that cannot cancel, a one-input transfer whose blinder sum is a single coin's blinder.

A target accepted the strict one-to-one: 4773 bytes, one range proof of 4174, its readback witness verified against an independently recomputed message. It is the smallest submission this lane has made, for a structural reason rather than by chance, one output meaning one range proof. That acceptance is what makes this a removal rather than a claim about a registry.

The removal did NOT free the merge, and the register says so rather than letting one acceptance stand for two shapes. What stopped the merge next is recorded at (`rule:shapes:canceling-predecessor`), and it has since been removed too — the merge was accepted, which took a second removal and not this one. The sentence above stays as written because it was true when written and because what it guarded against is exactly what a later reader might do: read one acceptance as freeing two shapes.

**The degeneracy the removal must name.** The forced blinder can be zero, and then it hides nothing. A merge's single output takes the sum of the consumed coins' blinders, so merging the two halves of this repository's inverse-pair dual-parity predecessor — whose blinders cancel by construction, which is what makes it an inverse pair — forces that sum to zero. The output commitment is then exactly v\*H: a point anyone can recompute from a guessed value, carrying a blinded output's form and none of its hiding. The form is still sound and the tally still balances; what fails is confidentiality, silently. A predecessor whose blinders do not cancel avoids it, so the removal must either require a non-canceling predecessor or refuse a solved zero blinder outright. Filing the form without this warning would file a confidentiality hole as a feature.

**This floor guarded consensus only by accident, and that accident has ended.** It counted outputs. It refused the consensus-impossible fee-only shape and the perfectly possible merge with the same message and the same indifference, so a reader who took the refusal as a consensus verdict would have been wrong about one of the two.

The coincidence is now gone, and ending it is most of what the removal was worth. The merge no longer meets a cardinality wall at all; it meets the zero blinder its only available inputs would force, which is the confidentiality property that actually separates a merge worth building from one that hides nothing. The fee-only shape still meets the floor, because a lone fee output can never declare the solved form — a fee is explicit, so it can never be the output that solves. The two draw different refusals now and each refusal is about its own shape, which the census tests hold by driving both and comparing.

The floor still guards the impossible shape for a reason that has nothing to do with consensus, so the register keeps saying so.

**Revising this limitation touches:** this section; the registry clause and its typed refusal, together with the parity search whose degeneration the new form has to state; the fixture manifest vocabulary, which today cannot express a zero-free-output manifest; the census module's rows for the one-to-one and merge shapes and the four tests that recompute them; the ceremony test in the multi-shape module that records the one-output floor as the merge wall; the merge erratum in the Guide-13 feature-request register, whose ground is this section; the positive private class vocabulary's merge member and whatever records it as unconstructible; and every backlog paragraph that describes the one-output shapes as typed at the registry floor, which is true only while the floor stands.

## The absent fee role · `rule:shapes:absent-fee-role`

**Refused.** The fixture output role vocabulary has no fee member, and every fixture output must carry a nonempty output program. A fee output carries an empty scriptPubKey by the target's own definition of a fee, so it is refused OutputProgramEmpty naming the output. This refuses the one-to-one-with-fee shape, which consensus admits.

**Refused at.** packages/target-elements-conformance/src/confidential_fixture.rs, the output_program.is_empty() clause of register_with_source. It runs per output, after the cardinality and balancing-role clauses, which is why a two-output fee-bearing manifest reaches it at all.

**Convention.** The absent fee role. The vocabulary has no member for an output that is explicit, unspendable, and outside the blinder solve, so the shape is inexpressible rather than rejected — nothing in the registry ever decided against fee outputs, and that is the point: no decision was recorded because none was made.

A second face of the same absence shows in the shared manifest builder, which casts the final output as the balancing one. The builder has no way to say explicit and outside the solve, so the one output that must never balance arrives cast as the output that does.

**Removal path, TAKEN at the registry.** Add a fee member to the fixture output role vocabulary: explicit-valued, held out of the blinder solve at a zero blinder, and required to carry an empty output program rather than merely permitted one, so the role is checked and not just excused from the nonempty-program clause. The clause then reads on the role instead of on every output alike, and the balancing role stops being assigned by position.

**REMOVED, under T5-042.** All of it. The role exists, the empty program is required of it rather than tolerated, the clause reads on the role and a non-fee output with no program is refused exactly as before, and the positional assignment is retired in both the manifest builder and the openings layer — every call site now states the roles it used to be handed implicitly, which is why no fixture's digest moved.

Two further corrections came with it, and both are the same correction said twice. A fee output's opening is ABSENT rather than zero-filled, because a record of zeroes reads like an opening and an explicit output has none; and the admitted-prefix rule reads on the outputs that have commitments rather than on the output count, so a two-output fixture of one blinded output beside a fee is not the dual-parity case whatever its output count says.

**And the shape still has not run, which is a different sentence.** The row is recorded expressible and unrun, not observed, and its removal carries no identity. A fee-bearing manifest registers, derives and digests; nothing between the registry and a chain has learned the role. The materializer's own output-role vocabulary has no fee member, its per-output stage would compute a commitment and a range proof for an output that must carry an explicit value and no witness at all, and the executor adapter's fixture catalogue and parity search read every output as a committed one.

The projection therefore refuses by name rather than mapping a fee onto the balancing role. That mapping would have compiled and would have produced a candidate whose fee output was blinded — not a fee at the target, and a silently wrong transaction rather than an honest stop. A register that recorded this shape as observed because its vocabulary could express it would be committing the exact error the register was built to prevent.

**The projection was REMOVED under T5-045, and the shape was offered to a node.** The materializer's role vocabulary gained a fee member; its per-output stage gained a fee stage emitting an explicit value, an explicit asset, a null nonce, an empty program and an empty witness entry, which then asks the built output whether it IS a fee by the target's own three-conjunct predicate rather than trusting that it built one; the projected view's three opening scalars became optional, because a fee output's opening is absent and not zero; and both projections state the fee arm instead of letting a catch-all solve a blinder for it. The typed refusal that named the missing projection is gone from the vocabulary rather than left standing, because a refusal nothing can return is a stopper that gets cited as current.

The fee output built by that chain is a fee. The run's own output-witness census reads one range proof of 4174 bytes and one EMPTY entry, which is the evidence the projection worked and the thing a fee mapped onto the balancing role could not have produced: that mapping would have read 4174 twice.

**The node refused the candidate, and the refusal is not about fees.** The candidate was offered to testmempoolaccept, which is the same door every acceptance this register cites came through, and testmempoolaccept did not allow it: mandatory-script-verify-flag-failed, Script failed an OP_EQUALVERIFY operation, at 4870 bytes submitted. So the fee-bearing shape has a relay fact and it is a refusal, and the refusal happened at SCRIPT VERIFICATION rather than at any fee gate — the transaction never reached the question of what its fee was worth. Elements admits a fee output in a non-policy asset at consensus and at policy alike — CTxOut::IsFee at src/primitives/transaction.h:324-327 constrains the form and never the asset, HasValidFee at src/confidential_validation.cpp:33-49 checks only that the amount is nonzero and in money range, and nothing anywhere in the tree rejects a transaction for the asset its fee output names — and the adapter's node runs at a zero minimum relay feerate. The candidate did not fail for carrying a fee.

**The third layer, uncovered by the second removal exactly as the second was uncovered by the first.** The reviewed live-transfer shape vocabulary has no sponsorless fee-bearing member. A sponsorless form is defined there as one that pays no fee at all, on the reviewed target's own representation of a zero fee by the ABSENCE of the output; so a two-destination sponsorless request selects a shape of two RECEIPT outputs, and the receipt covenant requires the second output to carry the second owner's private receipt constructor program. The second output is the fee, whose program is empty, and the comparison fails.

**Removal path, TAKEN under T5-047, on the owner's fee-matrix ruling recorded at the exit gate.** The ruling that a fee-bearing transfer be supported in all four sponsored and sponsorless forms is what changed a reviewed reading from an edit into a decision, and the path filed here is the one that was taken. The reviewed shape vocabulary gained a fee axis of its own, so a sponsorless form may declare the target fee role and a fee destination is no longer counted as a receipt output.

The axis is carried on the BOUNDS rather than on the demonstration shape set, and that is the load-bearing choice rather than a filing convenience. A candidate's shape set becomes one coordinator leaf per shape, the leaves tweak the taproot output key, and that key is the destination program every recorded fixture digest was taken over — so widening the demonstration set would have moved digests belonging to runs a pinned node had already accepted. The fee-bearing ceremony links against a SEPARATE candidate instead, and exactly one recorded digest moved: the fee-bearing successor's, which had never carried an acceptance and therefore cost no evidence to re-record. Every other recorded digest re-derives untouched, and that is a run rather than a claim.

The covenant needed less than the filing expected and more than it said. The discriminator was indeed already owned — the fee is recognized by form, at a negative version marker against the digest of the empty program, never by amount — but three further readings decided WHERE the fee sits and had to move with the output count: the family-range census, the isolation fragment's own emission, and the pattern census that decides whether that fragment is emitted at all. That last one had been a pure alias of the sponsor question, and had it stayed one the fragment would have carried the fee clause while the census recorded an empty leaf.

**Two corrections the filing did not anticipate, and both were forced by the target rather than chosen.** The fee clause demanded the RESERVE asset, which is right for a sponsored form whose fee is paid from the sponsor region; a sponsorless form has no such region and its only inputs are receipts, and Elements balances per asset, so a reserve-asset fee beside no reserve-asset input cannot balance at all. The clause now reads its asset off who funded the fee. And the conservation relation gained the fee as a TERM: receipts equal destinations plus fee where the transfer pays its own, against receipts equal destinations where a sponsor pays, because a self-paid fee leaves through the fee position in the protocol asset out of the very receipts being summed.

**§10.4's closure argument is restated rather than weakened.** Its discharge had been that the isolation fragment requires the reserve asset at every non-destination output position, and a self-paying fee position breaks that sentence. It does not break the argument. What §10.4 forbids is an UNACCOUNTED protocol-asset output — value leaving through a position no fragment speaks for — and the fee position is declared by the shape, fixed at an index the exact output count determines, and named by the fragment in both its asset and its form. It is spoken for as completely as a destination is; what it is not is a receipt, which is why it sits outside the range rather than inside it.

**A fourth layer was uncovered, exactly as the third was.** Giving the vocabulary its member moved the refusal one comparison further on, to the fee clause itself: the deployment was welded to a fee-program digest of 0xb5 bytes that no program hashes to, kept deliberately so the demonstration's committed taptree would not move, and nothing had ever executed the clause that reads it. The constant's own documentation had predicted this and named the answer — a deployment that means to spend a control carrying a fee supplies the digest of the fee program it constructs — and the fee-bearing deployment now does, at no cost to the demonstration, whose taptree and identities are untouched.

**The shape then RAN.** One receipt consumed, one blinded destination created, the transaction's own fee paid out of the value it consumed with no sponsor anywhere in it, accepted and mined — 4927 bytes submitted, the mined bytes read back equal to them, the witness verified against an independently recomputed message. The fee output is a fee by measurement and not by claim: the run's output-witness census reads one range proof of 4174 bytes and one EMPTY entry, which a fee mapped onto the balancing role could not have produced.

**Revising this limitation touches:** this section; the output role vocabulary and every match over it; the empty-program clause and its typed refusal; the balancing-role uniqueness clause, which today counts a positional assignment; the shared multi-output manifest builder's last-output-balances rule; the census rows for the fee-bearing shapes and the tests that recompute them; the ceremony test recording a fee output as inexpressible beyond the cardinality floor; and the observation in this register that no accepted identity carries a fee output, which stops being a statement about what the lane happens to build and becomes one about what it chooses to.

## The predecessor that cancels · `rule:shapes:canceling-predecessor`

**Refused.** A merge of this lane's own coins is refused at the registry's zero-solution clause, because the only two coins it can offer a merge are the two halves of an inverse pair.

**Refused at.** packages/target-elements-conformance/src/confidential_fixture.rs, the zero-solution clause of the derivation, reached because packages/vectors/src/live_multi_shapes.rs funds one predecessor whose two output blinders are ordered additive inverses.

**This limitation was UNCOVERED by a removal, not created by one.** It is worth recording as such. Removing the two-output floor was supposed to free two shapes and freed one; the merge walked forward and met a second wall that the first had been hiding. A register that reported only the removal would have implied the merge now runs, and a register that reported only the merge's continued absence would have implied nothing had changed.

**Convention.** The single funded predecessor. This ceremony funds one confidential predecessor from an explicit input, so that predecessor's own input blinder sum is zero and its two output blinders come out ordered additive inverses. Every two-input merge the ceremony could offer therefore consumes both halves of an inverse pair and presents a zero input blinder sum, which forces the lone output's blinder to zero — a commitment of exactly the value times the value generator, a point anyone recomputes from a guessed amount, hiding nothing while the tally still balances.

The registry refuses that, and refusing it is correct. This is the one limitation in the register whose refusal nobody wants removed: what wants removing is the ceremony's inability to offer any other pair of coins.

**It is a different KIND of limitation from the others.** Not a rule the registry states, but a predecessor the ceremony happens to fund. A merge of coins whose blinders do not cancel registers today, with no change to any rule.

**Removal path.** Chain a precursor submission whose outputs do not cancel, and merge two of those. A three-output precursor's blinders sum to the coin it consumed, so any two of them sum to that total less the third, which is nonzero for no reason anybody has to arrange. The ceremony already mines each acceptance rather than leaving it in the mempool, precisely so that the coins an acceptance creates are visible to a later step, so what is missing is a second submission stage and not a capability.

**REMOVED, under T5-045, and a target accepted the merge.** Two receipts consumed, one output created, 5156 bytes, one range proof of 4174, the mined bytes equal to the submitted ones and the read-back witness verifying against an independently recomputed message.

**The path taken diverges from the path filed, and the divergence is the honest half of this record.** The filed path chains a precursor SUBMISSION; what was built widens the FUNDING stage. The ceremony now funds a second predecessor of three outputs, identical to the dual-parity one in every respect but its output count — one explicit input, the same zero input blinder sum, the same profiles, the same issuing step — and merges two of its coins.

The arithmetic is the arithmetic the filed path named, and the register said as much when it filed it: three blinders summing to zero cancel in no pair, any two of them summing to the negation of the third. What differs is which stage produces the coins, and that difference is why the cheaper form was taken. A funding response carries the decoded coins WITH their openings; a submission response carries a transaction identity and mined bytes and no coin set at all. Reaching a non-canceling pair through a second submission stage would therefore have meant building the coin-return path the ceremony does not have, while reaching it through a wider funding stage needed nothing that did not already exist. The wall was the predecessor's arity, and the cheaper honest way through an arity wall is to fund a wider predecessor rather than to chain one.

**The forced blinder is nonzero, and that is observed rather than argued.** The argument is good — the merged pair's sum is the negation of the third coin's blinder, a derived blinder, and a derived blinder is searched upward until it is nonzero and never admitted zero — but an argument is not an observation and this register's whole discipline is not blurring the two. So the ceremony sums the coins it actually consumes, compares that sum with the blinder the registry solved for the sole output, and writes four facts into its own transcript: how many coins the sum was taken over, whether that sum is zero, whether the forced blinder is zero, and whether the two agree. Never the scalar itself, which would be publishing an opening.

**What did NOT change is the refusal this section records.** The registry still refuses a merge of the inverse pair, by name, with `DegenerateBalancingScalar`, and nobody wanted that removed — a zero-blinder commitment hides nothing and no part of this workspace may claim hiding for one. What was removed is the ceremony's inability to offer any other pair of coins, which is exactly what this section said wanted removing.

**One further change came with it.** The consumed blinder sum is now SUMMED over the coins a shape names as consumed, rather than stated from the one predecessor's structure. The stated zero was true of that predecessor and of no other; against a wider one it would have built a candidate whose value balance does not close, which is a thing one learns from a chain.

**Revising this limitation touches:** this section; the ceremony's stage vocabulary and its submission step names, which must stay distinct or the executor refuses a duplicate step; the input blinder sum rule, which today states the two-input case's zero from the predecessor's structure rather than summing the consumed coins; the fixture reference the merge's inputs resolve against, which would be the precursor's successor rather than the funded predecessor; the census row for the merge and the test that recomputes it; the §15.2 private-merge matrix row, which an acceptance of this shape would move; and the merge erratum in the Guide-13 feature-request register.

## The one impossible shape · `rule:shapes:fee-only`

**Impossible.** A transaction whose only output is the fee output cannot balance for a nonzero input blinder sum. The fee output is mandatorily explicit and contributes a zero blinder, so the output blinder sum is zero while the input blinder sum is not, and the tally at src/confidential_validation.cpp:73-81 fails.

This is source-derived and not observed. No node has refused one of these, because none was ever built; the ground is the arithmetic, not a verdict.

**The escapes both change the shape.** A zero-blinder input would balance, but an input whose blinder is zero is not a blinded input and the shape is a different one. An added blinded dummy output would balance, and is then the one-to-one-with-fee shape under another name. Neither is the fee-only shape, and recording them as escapes rather than as solutions is the honest form.

**What refuses it here, and why that is not the reason.** The registry refuses it OutputSetTooSmall — the cardinality floor, which fires before any output is inspected and never reaches the empty program at all. The refusal is correct and its reasoning has nothing to do with consensus.

That sentence was written when the floor's removal was a filed path, and it predicted what the removal would do to this row: the shape would need the fee role to be expressible and would still be impossible. Both halves happened, and this row did not move. The fee role now exists, so the shape is expressible in the sense that the vocabulary has words for it; the floor still refuses it, because a lone fee output can never declare the single-output solved form — a fee is explicit, so it can never be the output that solves — and the tally still cannot balance, because there is no blinded output to absorb the input blinder sum.

So the wall did not move without anybody deciding it should, which is what the prediction was guarding against. What did change is that the floor no longer refuses a POSSIBLE shape with the same message, so the refusal here no longer has a twin that would mislead a reader about which of the two consensus forbids.

**Revising this section touches:** nothing in the registry, because nothing in the registry is what makes this shape impossible. It touches this section and the census row alone, and would only ever be revised by a consensus change to how explicit outputs enter the tally.

## What the register does not claim · `rem:shapes:non-claims`

It does not claim that the source-derived-possible shapes would be accepted. It claims the balance rule does not forbid them. A transaction has to satisfy range proofs, script validity, policy and relay besides, and none of that is in scope here.

It does not claim the observed shapes exhaust what has been run; it claims each cites an acceptance of its own shape.

It does not claim that a removed limitation means the shapes it refused now run. Four limitations have been removed across three waves and each removal freed at most one shape. The merge met a second wall the first had been hiding, and the fee-bearing shape met a third the second had been hiding — both recorded as what they are rather than as consequences of a removal that did not have them. A register that reported only the removals would imply every refused shape now runs.

It does not claim the fee-bearing shape's remaining removal path is scheduled, designed in detail, or agreed. It is named, which is what the ruling asks for at this stage. No part of it is taken, and taking it means changing a reviewed reading of the target rather than a registry clause.

It does not extend to issuance, which is excluded (`rule:exclusions:issuance-bytes`), nor to sponsored shapes, whose signer dependency this workspace does not close.
