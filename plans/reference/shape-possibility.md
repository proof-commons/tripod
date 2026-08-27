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

The shape axis is the count of blinded inputs together with a partition of the outputs into three kinds: blinded, explicit receipt destinations, and the mandatorily explicit fee output. Nothing else about a transaction changes either verdict.

The enumeration is the small-shape window: at most two blinded inputs and one to three outputs, plus the fee-bearing and crossing members that window does not otherwise reach. Eleven shapes.

It is closed, and a reader can see that nothing is missing, because of what the tally rule shows: the consensus verdict depends on nothing whatever but whether the consumed blinder sum has somewhere to land. Exactly two facts decide that and no third does — whether any input is blinded, and whether any output is. Every combination of those two appears in the window, so every shape outside it inherits the verdict of the case it falls into, and adding one would restate a row rather than add one.

The window is otherwise chosen for the first-party half, where the counts do matter: the registry's own clauses are cardinality clauses, and every shape this lane has built or been refused falls inside it.

**What the crossing wave widened, and why the old rule was narrower than it read.** The rule used to open "at least one blinded input", and that was not a simplification for brevity. It was the premise that let the register DERIVE the blinded-output count as the output count less the fee: on a lane where every input is blinded and every non-fee output with it, there are only two kinds of output, so the fee count determines the rest.

Representation crossing breaks both halves of that premise, in opposite directions. An entry crossing consumes no blinded input at all, which is outside the old domain rather than unlisted within it. An exit crossing carries explicit non-fee outputs, which the subtraction would have counted as blinded — and the register would then have computed absorbability for a shape by counting outputs that absorb nothing, recording a possible verdict on arithmetic that denies it. So the blinded-output count is now an axis each shape states, the explicit destinations are the leftover, and a test requires the three counts to partition the output set.

The tally rule gained the second way a sum lands. A shape with no blinded input presents a sum that is already zero, and zero is absorbed by an all-explicit output set without anything having to hold it. The correct statement is therefore not "a blinded input forces a blinded output" but **the input blinder sum must equal the output blinder sum, and explicit outputs contribute zero** — and the entry crossing is the member that makes the difference between those two readings visible.

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
| No blinded input to two blinded outputs | Possible | Observed | Constructible after removal | (`rule:shapes:homogeneous-representation`), removed |
| Two blinded inputs to two explicit destinations and one blinded absorber | Possible | Observed | Constructible after removal | (`rule:shapes:homogeneous-representation`), removed |
| Two blinded inputs to explicit outputs only | Impossible | Source-derived | Refusal guards consensus | (`rule:shapes:fully-unblinding`) |

Every limitation tag in the last column is cited above in a parenthesized group, because each is minted at its own section below and a bare occurrence here would be a second mint of the same name.

Nine of eleven shapes have been run and accepted. The two that have not are the two the tally forbids.

The counts moved because five limitations have now been structurally removed, and the first-party statuses in the fourth column are different things that a single word would have flattened. Constructible and observed is a shape this lane always built. Constructible after removal is a shape it was refused until a convention was removed, and the row keeps citing the removed convention, because a row saying only that a shape works loses the fact that a wall stood there — and a wall nobody remembers is one that gets rebuilt.

Expressible and unrun is nobody's status, and the emptiness is a result rather than a loosening. Both crossings sat there when the vocabulary landed, and both left by the only honest exit, an acceptance of their own shape. The register keeps the words because the next removal nobody has run must be able to say so.

The crossing removal is the first that freed TWO shapes and saw both run. Every earlier one freed at most one, so the register had never had to say which shape carried a removal to a chain; it says so now, naming the first, and two rows cite one removal without either claiming the other's acceptance.

Submitted and refused is nobody's status. It was the fee-bearing shape's, which was BUILT with a real fee output and OFFERED to a node and turned away; the shape has since been given the vocabulary member it lacked and ACCEPTED, so it left by the second and last exit. The register keeps the words because the next shape a target refuses must be able to say so. A status vacated by work is not a status deleted.

The one-to-two row carries the first of two identities the lane recorded for that shape; the second, is the same shape spending the opposite commitment parity, and the register cites one because a shape needs one acceptance and not because the other is doubted.

## The form product, and how to look a form up · `rule:shapes:form-product`

The index above answers eleven shapes. The ruling this section implements asks a different question, and asks it of every form rather than of eleven: we do not need to support everything, but we must know what we do not support explicitly. An index of eleven rows answers that question for eleven forms and returns silence for the rest, and silence is the one answer the ruling forbids.

So the register also states the PRODUCT of the four axes those eleven shapes are points in. Its machine-checked form is the transfer-form half of the `shape_census` module, and it is a closure rule rather than a table: four hundred and eighty cells, about twenty written facts, and a derivation that computes the rest. The tests drive the live fixture registry and the live shape vocabulary for every cell and require the derived verdict to be the one those layers actually return, so an inherited claim is recomputed exactly as hard as a written one.

**The four axes.** How many receipts the transfer consumes, one or two. How many receipt destinations it creates, none through three — destinations only, the fee and the sponsor's change being separate axes so that a count never means two different things in two cells. Whether it carries the mandatorily explicit fee output. What the sponsor region contributes. And which value form each SIDE is written in.

**The sponsor axis has six members and that is the point.** It had no census vocabulary at all before this section, and the obvious repair — sponsored or not — would have been the second version of the error this whole register exists to prevent. Three facts about a sponsor region change a consensus verdict and they vary independently: whether the sponsor's coin carries a committed value, whether it takes change back, and whether that change is committed in turn. One member is impossible on the tally and a node has said so in its own words; one has never been stated anywhere; and the axis now carries four acceptances across three of its members, the fourth of which this register recorded as expressible before anybody ran it. No sponsored-or-not axis could tell those apart.

**The representation axis carries five members**, four of them pairings of the two plans §6.1 states exhaustively — explicit into explicit, private into private, explicit into private, private into explicit — plus the corner where a private consumed side meets a wholly explicit created side and nothing absorbs. The wholly explicit lane is a member so that a reader asking about an explicit form gets a status rather than silence; its sponsorless cells are answered by naming the register that owns them, because restating a verdict would create a second authored source for it.

**The tally rule needed one sharpening and no second predicate.** A commitment is v\*H_asset + r\*G. The value coordinate rides a per-asset generator, produced from the asset id at src/confidential_validation.cpp:321-324 and used to commit at :347-349, so a value in one asset can never cancel a value in another and each asset conserves separately. The blinder coordinate rides the single generator G for every asset alike, so the blinder sum is ONE sum across the whole transaction. That asymmetry is what makes a sponsor region computable without a second rule: its reserve-asset value must balance against the fee and the change in its own asset, which the funding can always arrange and which therefore forbids no cell, while its blinder joins the same global sum as every receipt's and does not care which asset holds it.

**Every unblinding cell states its own absorber.** This used to be recorded once, at the fully-unblinding corner, as the observation that an exit crossing without its absorber is impossible. Read at the corner it looks like a fact about one shape; it is the load-bearing structure of every unblinding cell in the product, and a reader deciding whether to build a two-explicit-output transfer needs the sentence attached to the cell they are reading. So each cell names the output that holds its consumed blinder sum, and a test recomputes the claim by taking the absorber away and requiring the counterpart cell's own verdict to change.

**Writing that test refuted the plain reading, and the correction is worth more than the reading was.** An exit crossing BESIDE a committed sponsor change has two blinded outputs, so removing the absorber does not reach the corner at all — the sponsor's change still holds the sum and consensus still admits the form. What the stripped cell loses is not possibility but a SOLVING role, because the registry's balancing model asks which output is solved and a sponsor's change was not one. The target admitted it and this workspace could not state it, which is this register's central distinction turning up in a place nobody had looked — and it is the half-written form of the solving-role wall whose removal (`rule:shapes:solving-role`) records: the registry can now state the solving role on a sponsor change, and the stripped cell is expressible and unrun rather than unsupported.

**What a reader gets back.** Every cell answers in one of eight words, and the answers a reader most needs are the unwelcome ones.

| Status | What it means | Cells |
|---|---|---|
| Not a form of this space | The axes contradict, and the contradiction is named | 282 |
| Expressible and unrun | Consensus admits it, this workspace states it, nothing has built one | 115 |
| Impossible, derived | The tally forbids it | 49 |
| Unsupported here | Consensus admits it and a named layer here refuses it | 0 |
| Supported and run | A node accepted one, and the cell cites its identity | 13 |
| Stated in another register | Answered by the register that owns it, rather than twice | 12 |
| Refused to protect hiding | The tally is content and the one blinded output would hide nothing | 8 |
| Impossible, observed | Built, offered to a node, and refused on the balance rule itself | 1 |

Unsupported-here holds ZERO cells, and the zero is a result rather than a loosening. Thirty-six cells sat there when the two walls that asked removal stood; both walls were taken down under T5-063 along their filed paths, recorded at (`rule:shapes:sponsor-change-role`) and (`rule:shapes:solving-role`), and every freed cell was re-verdicted by the unchanged cascade to expressible-and-unrun rather than relabeled — a freed cell is a cell nobody has offered, and the standing says so. The status is kept in the vocabulary for the next wall a wave meets, exactly as the enumeration keeps its vacated statuses.

Two hundred and eighty-two of the four hundred and eighty are combinations of axes that contradict, so the space this register answers about is the remaining one hundred and ninety-eight — and every one of them carries a verdict. The largest single contradiction is a sponsor region beside no fee output: a sponsor region is DEFINED by the fee it funds, the reviewed shape vocabulary derives the fee's presence from a nonzero sponsor-input count and refuses the combination by name, so a sponsored form paying no fee is not a form this space can state. Nothing about consensus forbids one. It is a definition, and the register says which.

**The wider fee-bearing arities are expressible and unrun, and that is a proof rather than a hope.** The fixture registry places no cardinality rule on the fee role and no ceiling on the output count, so every fee-bearing manifest registers. The reviewed shape vocabulary carries a sponsorless fee-bearing member at every receipt-input and receipt-output count in its bounds. Both are LINKED and asked by the tests. What is missing is a ceremony: one fee-bearing arity has run and the other five have not, and the register names which.

**One caveat every fee-bearing row depends on.** Those members live in the SECOND deployment. The demonstration deployment carries none of them, because admitting the member would move every taproot output key and therefore every recorded fixture digest, so the fee-bearing vocabulary was landed as a separate deployment. The same cell is expressible from one deployment and unreachable from the other. This is a limitation nobody wants removed — the workspace would choose it again — and it is recorded all the same, because a register able to record only walls it wanted torn down would quietly stop recording the other kind.

**Crossing composed with a sponsor is stated everywhere and built nowhere.** Nothing refuses it: the isolation fragment that handles the sponsor region takes no representation and reads no value field, the coordinator appends that fragment and then matches on the composition, and the positional value-form leaf runs over the destination range alone, which the fee and the sponsor change sit outside of by construction. The crossing taptree already carries the sponsored coordinator leaves. What is missing is a caller — the composing finalization entry point takes a composition and a sponsor capability in the same parameter list, every sponsored caller here takes the homogeneous wrapper instead, and the one crossing ceremony pins sponsorlessness and passes no sponsor capability at all. Absence, not refusal, and the register says which.

## Two vocabularies, and the row that reads broader than it is · `rule:shapes:two-vocabularies`

This section exists because the owner read a census row as a claim it does not make, and the row was not at fault. Two registers in this workspace describe transfer shapes and they count DIFFERENT THINGS, so a sentence true in one is false in the other while both look like statements about the same transaction.

**The census counts blinded outputs. The minimality pair registry counts roles.** A census row saying one receipt into two blinded outputs is a claim about the value forms of two outputs and about nothing else. The pair registry's split member is a claim about two RECIPIENTS — two outputs that pay somebody other than the sender — which is a different claim at the same cardinality.

**The recorded one-to-two runs are one recipient plus one balancing change.** The private restart module states it in its own header and its request is built from a recipient and the sender's change in that order, with the conservation test destructuring the pair as recipient and change. The other recorded private one-in-two-out run is the fee-bearing shape, whose second output is a fee role rather than a receipt. So for as long as this trap stood open, the pure two-recipient split had never run, and the pair registry pinned it as awaiting a run of its own shape rather than borrowing the census row sitting at the same arity — which is exactly the discipline that kept the gap visible until somebody closed it.

**The census cannot see the difference, and this is not a defect in it.** Its output axes are the blinded count, the explicit destination count and the fee count, which partition the output set by VALUE FORM. A recipient and a balancing change are both blinded outputs. Recipient and balancing are not among the census's axes and could not be added without the census becoming a second pair registry.

**The asymmetry is sharper than a difference of axes, and it is the part worth remembering.** The census's axes are typed, stated per member, and checked — a member whose axes disagree with its own output count fails a test rather than computing a wrong verdict. The pair registry's fixture type has NO role axis at all: it carries source and destination endpoints of an owner and an amount, and §14.3 forbids it carrying a position, a script, a key or a blinding factor, so role was swept out with them. The roles a pair claims therefore live in English, inside the strings that explain why a member has no run. One vocabulary is checkable and one is narrated, and a reader who takes them for two views of one model will believe the narrated one is enforced.

**The trap has a second face inside the pair registry itself.** The split fixture's second destination pays the SOURCE owner, which is semantically change; the registry types it as an ordinary destination because the fixture type has nowhere to record a role. So even the pin's own two-recipient claim is one its type cannot express and only its prose asserts.

**Both of those gaps closed while this section was being written, and the sequence is the evidence.** Row T5-056 drove the pure private split to an acceptance — one receipt consumed, two created, both of them receipts, no change and no fee — which answers the split pair's private member as its own shape rather than by borrowing the census row. It also drove the private sponsored form with an explicit sponsor coin and no change, accepted. Each moved on an acceptance of its own shape and not on a wave's momentum, which is the rule every row here is held to.

**The second of those is worth more to this register than one more acceptance.** That cell was recorded expressible and unrun, at exactly those axes, BEFORE it ran: the register said the manifest registers, the shape member exists, and no ceremony asks for it — and then a ceremony asked and a node agreed. A knowledge map that predicts a cell and is then confirmed is describing the real space, rather than the space somebody happened to have built already.

**Its arithmetic confirmed the sponsor derivation from outside.** The run's own record states that an explicit sponsor coin brings an all-zero blinder, so nothing needs absorbing and no change is owed, and that the reserve sub-equation is sponsor equals fee plus change because the target balances per asset. That is the two-coordinate rule this register derives every sponsor cell from, reached independently. The committed-sponsor refusal and this acceptance are the two halves of one rule rather than a rule and an exception to it.

**The trap itself is not closed by either run, and it is worth keeping the distinction.** What closed is one instance: a pin that stood open is now answered. What stands is the reason the pin was needed — a census row still counts blinded outputs and still claims nothing about roles, so the next reader at the next arity can make the same mistake unless the two registers keep saying which question they answer.

**What this means for reading any row here.** A row of this register claims exactly what its axes say: how many inputs carry blinded values, how many outputs do, how many are explicit destinations, and whether there is a fee. It claims nothing about who is paid, nothing about which output is change, and nothing about whether a shape at the same cardinality but with different roles has ever run. When those questions matter, the pair registry is the register that answers them, and its unrun pins are the honest statement of what has not been done.

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

## The second impossible shape · `rule:shapes:fully-unblinding`

**Impossible.** A transaction consuming blinded inputs into nothing but explicit outputs cannot balance for a nonzero input blinder sum, for the same reason and at the same line as the fee-only shape: every explicit output contributes a zero blinder, so the output blinder sum is zero while the input blinder sum is not, and the tally at src/confidential_validation.cpp:73-81 fails. Upstream tests it directly — src/test/blind_tests.cpp:155-191 spends an explicit and a blinded input into three explicit outputs and asserts VerifyAmounts false.

There is no structural rule anywhere saying a blinded input forces a blinded output. The transaction passes every shape check, because all-explicit outputs mean every proof is required-empty and trivially satisfied, and it dies at the tally alone.

**Why it is filed separately from the fee-only shape.** Because the first-party half is a different refusal for a different reason, and this is the row that makes the register's central distinction concrete. The fee-only shape dies at the cardinality floor, having one output. This shape has three, clears the floor, and is refused because none of them SOLVES — BalancingRoleNotUnique carrying found zero. Consensus and this workspace refuse the same shape on independent grounds, and a register that recorded only one of them would be claiming a coincidence was a reason.

Both refusals are recomputed rather than transcribed: the census drives the live registry for this row, and a conformance test registers the manifest directly.

**The escape is real, and it is another row's enabling condition rather than a way to take this one.** If the spent coins' blinders sum to zero the tally passes with blinded inputs and wholly explicit outputs. That is reachable by construction and not by luck — a parent with all-explicit inputs has blinder sum zero, so Elements makes its blinded outputs' blinders sum to zero at src/blind.cpp:574-577, and spending all of them together presents a zero input blinder sum. This workspace's dual-parity predecessor is such a parent, and its inverse-pair property is the very thing (`rule:shapes:canceling-predecessor`) was filed as a limitation about. The wall for one shape is the enabling condition for another.

It does not move this verdict, because it is a property of the coins spent rather than of the shape. A shape whose inputs' blinders cancel is a shape with no effective blinded input, which is a different row.

**Revising this section touches:** nothing in the registry, for the reason the fee-only section gives. It touches this section, the census row, and the exit-crossing row it sits beside, since the two differ by exactly one output's value form.

## One representation per transfer · `rule:shapes:homogeneous-representation`

**Refused.** Neither crossing direction could be stated at all. A live transfer's representation was one variable for the whole transaction, so an explicit-valued receipt could not be spent into a blinded destination nor a blinded receipt into an explicit one — refused before any arithmetic, not because a blinder failed to solve or a proof failed to build, but because no vocabulary existed in which the candidate could be written.

**Refused at.** packages/compiler/src/live_transfer_plan.rs, whose representation plan is one value per transfer, read at every site that decides a form: packages/tapscript/src/live_pattern.rs, whose recognition fragment pins the spent value's form to that one plan and whose coordinator dispatches the destinations' obligation on it, and packages/transaction/src/live_construct.rs, which reads it for both the receipts it recognizes and the destinations it pays.

**Convention.** One transfer, one representation. Guide §6.5 states the initial scope as homogeneous explicit required and homogeneous private committed required, with mixed unsupported unless separately admitted — so mixed was never forbidden and never built, and the vocabulary took the scope literally. The clause that kept it that way is the guide's next sentence rather than the scope: an ad hoc mixed transaction accepted by the target does not widen the ABI. No run could ever have supplied the admission and only a ruling could. None was made, so the single variable stood.

This is a limitation of this repository and not of the protocol. Consensus admits both directions and is upstream-tested doing so: src/test/blind_tests.cpp:229-238 builds an explicit non-fee output between two blinded ones with a blinded input and asserts VerifyAmounts true, and test/functional/feature_confidential_transactions.py:370-384 has a live node accept the same shape.

**Removal path, TAKEN under T5-054 and RUN in BOTH directions.** Admit a pairing of the two representation plans, one per side of a transfer, rather than a third representation or a per-reference variable. §6.5 already wrote the escape clause, so what the path needs is a ruling and a vocabulary rather than a guide amendment.

The pairing threads to four decisions that each used to read the single variable: which value form the recognition fragment pins a spent receipt to, which obligation the coordinator emits over the destinations, which constructor a destination is paid to, and which key a deployment seats a constructor at. The exit direction additionally needs a positional value-form leaf, because its created side is per-position heterogeneous — explicit everywhere but the one declared absorber — and no fragment that speaks about a whole range can say that.

**The absorber is a declared destination position.** Not a fourth output family, and the distinction is what keeps §10.4's closure argument untouched. The shape's exact output count and the three-family position census leave no position outside the destinations, the sponsor change and the fee, so an absorber outside the destination range is unrepresentable rather than merely unsound. Inside the range it adds no position and moves none; the leaf only says which of the positions the closure already speaks for carries which form.

**The entry direction has a degeneracy, and it is the single-output form's met from the other side.** An entry crossing spends explicit coins, whose blinders are zero, so a single blinded output would be forced to a zero blinder and its commitment would hide nothing. Two or more blinded outputs avoid it, the balancing blinder being the negation of a searched non-zero primary. The floor of two is therefore the registry's own arithmetic rather than a preference — DegenerateBalancingScalar catches it either way — and Elements' own wallet refuses the same shape for the same reason at src/blind.cpp:578-586. Both are conventions about confidentiality and neither is a protocol rule; this register claims neither as one.

**The exit direction RAN.** A real node accepted and mined it: two blinded receipts consumed, two explicit destinations created, one blinded absorber beside them, 5412 bytes submitted and the mined bytes read back equal to them, an owner signature verified against an independently recomputed message, 11.5 seconds.

The crossing is visible in the bytes and not only in the construction. The output-witness proof census is 0, 0, 4174 — an explicit value admits no range proof and an explicit asset no surjection proof, so the two explicit destinations carry neither and the single blinded absorber carries the transaction's only proof. A wholly private shape of this arity would carry three proofs and a wholly explicit one none, so that vector is a shape no homogeneous transfer can produce.

**The entry direction RAN too.** Accepted and mined: one explicit receipt consumed, two blinded destinations created, 9133 bytes submitted and read back equal, an owner signature verified against an independently recomputed message, 9.9 seconds.

This workspace has performed the shape every ceremony, as the funding step that mints a confidential predecessor. What the acceptance records is the first time the coin it spent sat at a receipt constructor's program, so the transfer was governed by the covenant rather than by the adapter — which is the whole difference between a funding step and a transfer, and it is why an accepted funding transaction could never have stood in for this row.

Its proof census is two range proofs for two blinded destinations, and the count is the claim rather than a detail. A single blinded output would have been forced to a zero blinder, because an explicit input contributes one, and its commitment would have hidden nothing.

**What the entry direction needed, and it was not consensus.** Three first-party readings, each of which had conflated a side with the transaction. A consumed receipt could not be stated without naming a registered confidential fixture output, though an explicit coin has no opening to name. The randomness rule keyed on the request's plan rather than on the side being blinded, and so refused randomness to the one transfer that most needs it. And the opening-binding census counted verified references against every input rather than against the inputs that have them.

**Revising this limitation touches:** the compiler's representation plan and its deferral census, the constructor and the leaf roles keyed by representation, the coordinator's value-obligation dispatch and the positional fragment, the deployment's seating of constructors at table keys, the construction lane's two lookups, the fixture registry's output role vocabulary and the materializer's, the shape census's output axes, and guide §6.5 — which is exercised rather than amended, its own escape clause being what the ruling took.

## The sponsor-change role · `rule:shapes:sponsor-change-role`

**Refused.** A confidential manifest whose sponsor takes EXPLICIT change back could not be stated at all. The fixture output role vocabulary's one sponsor member was the committed change, whose carries_an_opening is true, and an explicit output has no opening to carry — so the thirty consensus-possible cells of the product whose sponsor takes explicit change on a confidential lane were unsupported here, refused by a vocabulary rather than by any clause anybody wrote.

**Refused at.** packages/target-elements-conformance/src/confidential_fixture.rs, the FixtureOutputRole vocabulary as it stood: one sponsor member, committed, and no other kind.

**Convention.** The lane a form runs on decides which registry states it. An explicit sponsor change belongs to the explicit lane, which registers nothing, so the confidential registry was given the one sponsor role the confidential lane needed. Nothing decided against the other; no decision was recorded because none was made — the sentence this register has now written five times.

**Removal path, TAKEN under T5-063.** Give the vocabulary an explicit sponsor-change member beside the committed one, carrying its own asset exactly as the committed member does and carrying no opening, so the empty-program clause and the parity rule read it as the explicit output it is. That is what was built, and nothing else: the member rides transcript code 7, the next unused one, its reserve asset travels through the same one-seam accessor the committed member rides, and no manifest registered before it hashes a byte differently — the declaration rides in a role code the transcript already emits per output, which is the same property that let the committed member, the fee, the sole-balancing form and the explicit destination land without moving a recorded digest.

The removal is a narrowing and not a relaxation, held by driving rather than argued. A programless explicit change draws the same refusal a programless destination draws; a manifest whose outputs are the explicit change, an explicit destination and a fee draws the refusal such a manifest has always drawn, with the same count of zero solving roles; and a manifest that does not name the member registers exactly as before. Nothing that was refused registers now unless it says which role it means.

**And no freed cell has run, which is a different sentence.** The thirty cells re-verdicted to expressible-and-unrun by the unchanged cascade, and the removal's proof of a chain acceptance is deliberately absent — the T5-042 precedent exactly. A freed manifest registers, derives and digests; nothing between the registry and a node has learned the role.

**The wall behind the wall, censused rather than folded in.** The materializer's projection carries one sponsor role and it is the committed change. Its closed-view arm refuses a registry role with no member in the view's own vocabulary rather than substituting one — the honest stop the fee role's history demanded, since a substitution would solve a blinder for an output that publishes its amount. The census records it as its own limitation with its own path: carry the freed roles through the projection as the fee role was carried under T5-045, and behind that a ceremony still owes a stage that asks for a freed form by name, which is a second absence and not the same one. Removing a wall does not always reveal open ground; this register has now recorded that lesson four times, and recording it is the point.

**Revising this limitation touches:** this section; the vocabulary member and its five predicate accessors, together with the digest-stability test that pins the transcript codes; the census's layer derivation, its drive helper and the three tests that pin the freed counts; the projection's closed-view arm, which is the uncovered wall's own site; and the census row for the projection limitation, which a later removal moves exactly as this one moved the sponsor-change row.

## The solving role, confined to the destinations · `rule:shapes:solving-role`

**Refused.** A form whose only blinded output is the sponsor's committed change could not register. The balancing-output model names exactly one output whose blinder is SOLVED to close the tally, the solving election counted Balancing and SoleBalancing alone, and the committed sponsor change carries an opening and does not solve — so the six consensus-possible cells whose blinded output set is the sponsor's change alone drew BalancingRoleNotUnique with a count of zero, the target admitting a form this workspace could not state.

**Refused at.** packages/target-elements-conformance/src/confidential_fixture.rs, the solves_the_balance clause of register_with_source, counting the vocabulary as it stood.

**Convention.** The balancing-output model, met at its edge. The model was written when every blinded output was a destination; a sponsor's committed change is a blinded output that is not a destination, so the model had a blinded output it would not solve for — and the tally, which does not know what a destination is, would have let it. The exit-crossing correction above found the wall's half-written form before anybody drove it: a stripped crossing loses not possibility but a solving role.

**Removal path, TAKEN under T5-063, along the filed path's first alternative.** Let the solving role be stated on a sponsor change. The vocabulary gained a balancing sponsor-change member at transcript code 8: the committed change's arithmetic under the balancing election — its own reserve asset through the same one-seam accessor, an opening the registry derives, a commitment built against its own asset generator — and a blinder solved from the others rather than derived. The filed path's second alternative, a manifest-level statement of which output solves, was NOT taken and the reason is recorded: a manifest-level field would have shifted every registered digest, while a role rides in a code the transcript already emits per output. No recorded digest moved.

The removal is a narrowing and not a relaxation, held by driving. A committed change that does not declare the solving member still draws BalancingRoleNotUnique with found zero — the same refusal with the same count the wall always drew — and the uniqueness clause counts the new member exactly as it counts the old solving roles, refusing two solvers with found two.

**The degeneracy warning travels with the role, and it is the T5-041 condition carried verbatim.** A sole solved output over a zero consumed sum hides nothing: the solve returns the zero unchanged and the commitment is exactly the value times its asset generator, a point anyone recomputes from a guessed amount. DegenerateBalancingScalar stands load-bearing — a solving sponsor change over a canceling sum is refused rather than built, recomputed by test — so a version admitting the degenerate case is not this removal done, and this one does not admit it.

**And no freed cell has run, which is a different sentence.** The six cells re-verdicted to expressible-and-unrun by the unchanged cascade, and the removal's proof of a chain acceptance is deliberately absent, for exactly the sponsor-change removal's reason: the projection between the registry and a candidate has not learned the role. The uncovered wall is censused once, at the sponsor-change section above, and covers both freed members; folding a second copy in here would be a second authored source for one fact.

**Revising this limitation touches:** this section; the vocabulary member, the solving election it joins and the degeneracy refusal it makes load-bearing a second time; the census's layer derivation, drive helper and count pins; the absorber derivation's two sponsor-bearing answers, which state the stripped cell's standing; and the exit-crossing correction in the form-product section, whose half-written finding this removal completes.

## What the register does not claim · `rem:shapes:non-claims`

It does not claim that the source-derived-possible shapes would be accepted. It claims the balance rule does not forbid them. A transaction has to satisfy range proofs, script validity, policy and relay besides, and none of that is in scope here.

It does not claim the observed shapes exhaust what has been run; it claims each cites an acceptance of its own shape.

It does not claim that a removed limitation means the shapes it refused now run. Five limitations have been removed across four waves and each removal freed at most one shape. The merge met a second wall the first had been hiding, and the fee-bearing shape met a third the second had been hiding — both recorded as what they are rather than as consequences of a removal that did not have them. A register that reported only the removals would imply every refused shape now runs.

The crossing removal is the exception that proves the rule rather than a counterexample to it: it freed two shapes and both ran, which is the first time that has happened. It is recorded as one removal with one proof and two rows citing it, not as two removals, because what was removed was one reading. Two of the five removals still carry no chain identity at all, and the register keeps that visible rather than letting a filed path read as a taken one.

It does not extend to issuance, which is excluded (`rule:exclusions:issuance-bytes`).

It DOES now extend to sponsored shapes, and the sentence that stood here — that it does not, the signer dependency being unclosed — was refuted by running. Three sponsored forms have been accepted into blocks and a fourth was refused by a node on the balance rule itself, so the axis had four facts and no register, which is why (`rule:shapes:form-product`) mints one. The correction is recorded rather than quietly applied, because a claim this register made and had to withdraw is worth as much to a later reader as one it kept.

It does not claim the product's expressible-and-unrun cells would be accepted, for the reason the first non-claim gives, and it does not claim any of them is scheduled. Expressible and unrun is a status, not a plan.
