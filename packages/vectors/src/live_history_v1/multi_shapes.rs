//! Historical private multi-shape observations.

/// The disposable asset all three runs issued.
///
/// The same identity the earlier steps issued, because the issuing
/// step is theirs and the chain is created fresh per run.
pub const ISSUED_ASSET: &str = "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

/// The split shape's recorded successor fixture digest under fixture-digest v1.
///
/// This value and `SPLIT_ACCEPTED_TXID` are the immutable halves of one historical observation.
/// The same fixture's forward v2 digest is separate and awaits its own node-accepted run.
pub const SPLIT_SUCCESSOR_DIGEST: &str =
    "43e15876204c04feecc0dc49387479288923392c9b6c3a0cc7be14e35edd4d99";

/// The identity the target computed for the accepted split.
///
/// One receipt consumed, THREE outputs created: two recipients and
/// the balancing change. The row `private-split` moves on THIS
/// acceptance and cites it.
pub const SPLIT_ACCEPTED_TXID: &str =
    "aa9f26931472872045c457e8f3e716add68c15287ee1c0d9734305a576cdb40d";

crate::recorded_acceptance::mint_recorded_acceptance!(split_accepted, SPLIT_ACCEPTED_TXID);

/// How many bytes the split handed to the node.
pub const SPLIT_SUBMITTED_BYTES: usize = 13_499;

/// The range-proof bytes each of the split's three outputs carried.
pub const SPLIT_OUTPUT_WITNESS_PROOF_BYTES: [usize; 3] = [4_174, 4_174, 4_174];

/// The split's wall time, in seconds.
pub const SPLIT_WALL_SECONDS: f64 = 12.9;

/// The many-to-many shape's recorded successor fixture digest under fixture-digest v1.
///
/// This value and `MANY_TO_MANY_ACCEPTED_TXID` are the immutable halves of one historical
/// observation. The same fixture's forward v2 digest is separate and awaits its own
/// node-accepted run.
pub const MANY_TO_MANY_SUCCESSOR_DIGEST: &str =
    "31162852b1f393b74be3bfa9ef2f44bacd17d0126947911937d5ae708792f421";

/// The identity the target computed for the accepted many-to-many.
///
/// TWO receipts consumed and THREE outputs created — the
/// representative case, whose input and output counts both exceed the
/// one-to-one control's, so it is not a one-to-many or a many-to-one
/// under another name.
pub const MANY_TO_MANY_ACCEPTED_TXID: &str =
    "3b61056ac96cf47a73f35370f9fea1faad69fc3f025359339765230da2f57e12";

crate::recorded_acceptance::mint_recorded_acceptance!(
    many_to_many_accepted,
    MANY_TO_MANY_ACCEPTED_TXID
);

/// How many bytes the many-to-many handed to the node.
pub const MANY_TO_MANY_SUBMITTED_BYTES: usize = 13_882;

/// The many-to-many's wall time, in seconds.
pub const MANY_TO_MANY_WALL_SECONDS: f64 = 13.9;

/// The several-distinct-owners shape's recorded successor fixture digest under fixture-digest
/// v1.
///
/// This value and `SEVERAL_OWNERS_ACCEPTED_TXID` are the immutable halves of one historical
/// observation. The same fixture's forward v2 digest is separate and awaits its own
/// node-accepted run.
pub const SEVERAL_OWNERS_SUCCESSOR_DIGEST: &str =
    "cfecf21f58fcc4d0571cccb701915f09025a7c8066415fd4d82f62839aba7dcc";

/// The identity the target computed for the accepted
/// several-distinct-owners transfer.
///
/// TWO receipts consumed under two DISTINCT published owners, each
/// input carrying the leaf its own position executes, and two outputs
/// created. Its subject is the input owners rather than the
/// cardinality.
pub const SEVERAL_OWNERS_ACCEPTED_TXID: &str =
    "cfabc99a9560de40d235ea891d1c234844bc516ca9b371ce7e888fd863544f95";

crate::recorded_acceptance::mint_recorded_acceptance!(
    several_owners_accepted,
    SEVERAL_OWNERS_ACCEPTED_TXID
);

/// How many bytes the several-owners transfer handed to the node.
pub const SEVERAL_OWNERS_SUBMITTED_BYTES: usize = 9_519;

/// The several-owners transfer's wall time, in seconds.
pub const SEVERAL_OWNERS_WALL_SECONDS: f64 = 15.6;

/// The identity the target computed for the accepted STRICT
/// ONE-TO-ONE.
///
/// # What this identity is evidence of
///
/// ONE receipt consumed and ONE output created — a shape this
/// workspace's own fixture registry refused to express until the
/// two-output floor was structurally removed. The consensus shape
/// census recorded it source-derived-possible and refused, with the
/// floor named as a first-party convention rather than a protocol
/// rule; this is the acceptance that moves that entry off the
/// derivation and onto a chain.
///
/// It moves NO matrix row. The guide's §15.2 positive private table
/// has no member for the strict one-to-one, and the census entry is
/// what an acceptance of it moves.
///
/// The lone output's value blinder is FORCED to the input blinder
/// sum, which for one consumed receipt is that coin's own blinder.
/// Nothing here is claimed about a merge: a merge consumes two coins
/// and this consumed one.
pub const STRICT_ONE_TO_ONE_ACCEPTED_TXID: &str =
    "45d846b0a57b15612012f432d34a36842d26860f853fff82b1a9389529e8dcbf";

crate::recorded_acceptance::mint_recorded_acceptance!(
    strict_one_to_one_accepted,
    STRICT_ONE_TO_ONE_ACCEPTED_TXID
);

/// The strict one-to-one's recorded successor fixture digest under fixture-digest v1.
///
/// This value and `STRICT_ONE_TO_ONE_ACCEPTED_TXID` are the immutable halves of one historical
/// observation. The same fixture's forward v2 digest is separate and awaits its own
/// node-accepted run.
pub const STRICT_ONE_TO_ONE_SUCCESSOR_DIGEST: &str =
    "00d0179914058b9a1f59ec71de77b3dfd4f48928313a9d52f41f12f003d735b2";

/// How many bytes the strict one-to-one handed to the node.
///
/// The smallest submission of any shape this lane has run, and for a
/// structural reason rather than by chance: one output means one range
/// proof, and the range proof is most of a confidential transaction.
pub const STRICT_ONE_TO_ONE_SUBMITTED_BYTES: usize = 4_773;

/// The range-proof bytes its one output witness carried.
pub const STRICT_ONE_TO_ONE_PROOF_BYTES: [usize; 1] = [4_174];

/// The strict one-to-one's wall time, in seconds.
pub const STRICT_ONE_TO_ONE_WALL_SECONDS: f64 = 11.2;

/// The path the accepted strict one-to-one actually took, recorded
/// because a shape's acceptance is only as strong as the door it came
/// through.
///
/// It crossed BOTH. The adapter offers a submission to
/// `testmempoolaccept` first and reports an acceptance only where that
/// answered allowed, then confirms it with `generateblock` so the
/// acceptance is one by block validation too. So this shape was
/// admitted by mempool policy and then included in a block, rather
/// than reaching a block as a package child or by consensus retry
/// after a policy refusal — the retry path the adapter keeps for a
/// transaction standardness turns away.
///
/// What that does NOT establish is anything about relay on a network
/// this workspace does not run. The chain is a disposable development
/// one the run created and destroyed, carrying its own policy, and the
/// transaction pays no fee because it carries no fee output at all.
/// The claim is that this node's own mempool admitted it, which is
/// what was observed and the whole of what is recorded.
pub const STRICT_ONE_TO_ONE_CROSSED_RELAY_AND_BLOCK: bool = true;

/// How many receipt inputs each shape consumed, in the order the
/// restart runs them.
///
/// Recorded rather than assumed, so a shape whose cardinality drifted
/// is readable here rather than inferred from a name.
///
/// The order is these arrays' OWN and is not
/// [`crate::live_multi_shapes::PrivateShape::ALL`]'s: split, many-to-many,
/// several-distinct-owners, strict one-to-one, one-to-one-with-fee,
/// merge, pure split. The two crossings are absent because a
/// crossing's sides are read under different representation plans
/// and a single receipt count would state one side as though it were
/// both. The pure split is LAST for the reason it is last in `ALL`:
/// it was appended after the runs before it were recorded.
pub const RECEIPT_LEAVES: [usize; 7] = [1, 2, 2, 1, 1, 2, 1];

/// How many outputs each shape created, in the same order.
pub const OUTPUT_COUNTS: [usize; 7] = [3, 3, 2, 1, 2, 1, 2];

// --- The private merge: the row this wave moves --------------------

/// The merge's recorded successor fixture digest under fixture-digest v1.
///
/// This value and `MERGE_ACCEPTED_TXID` are the immutable halves of one historical observation.
/// The same fixture's forward v2 digest is separate and awaits its own node-accepted run.
pub const MERGE_SUCCESSOR_DIGEST: &str =
    "f1bf90b46d16a814c0a1069bb8fa2ab9ea9708141fb7454d7e6c7720484588aa";

/// The identity the target computed for the accepted private merge.
///
/// TWO receipts consumed and ONE output created. The row
/// `private-merge` moves on THIS acceptance and cites it.
///
/// It is the first acceptance of a shape that had met two walls: the
/// registry's two-output floor, and then the zero blinder its only
/// available predecessor forced. The floor was removed by the
/// sole-balancing form and the zero blinder by a predecessor whose
/// coins do not cancel, and neither removal was worth anything until
/// this identity existed.
pub const MERGE_ACCEPTED_TXID: &str =
    "146df7852c8ad0ded0c1fdb209ff2dc93d1680e7f446f3ce2d8e2f39e710c018";

crate::recorded_acceptance::mint_recorded_acceptance!(merge_accepted, MERGE_ACCEPTED_TXID);

/// How many bytes the merge handed to the node.
pub const MERGE_SUBMITTED_BYTES: usize = 5_156;

/// The range-proof bytes its one output witness carried.
///
/// One proof for one output, and two inputs' worth of witness beside
/// it -- which is why the merge is larger than the strict one-to-one
/// despite having the same output count.
pub const MERGE_PROOF_BYTES: [usize; 1] = [4_174];

/// The commitment prefixes the merge's three funded coins carried.
///
/// NOT the admitted pair in fixed order, and that is the arity rule
/// working rather than a defect. A three-output fixture is held to
/// membership -- each commitment carries one of the two admitted
/// prefixes -- because the reviewed target contract states no
/// fixed-order convention for a third output. Two of these three are
/// the same prefix, which a fixed-order rule would have rejected and
/// which the contract does not.
pub const MERGE_PREDECESSOR_PREFIXES: [u8; 3] = [0x09, 0x08, 0x09];

/// Whether the merge's forced blinder came out ZERO.
///
/// False, and observed rather than argued. The ceremony summed the
/// two coins it actually consumed, compared that sum with the blinder
/// the registry solved for the sole output, and wrote both answers
/// into its transcript. Nothing in this workspace claims hiding for a
/// zero-blinder commitment, so a merge that could not say this is
/// false would not be a merge worth recording.
pub const MERGE_FORCED_BLINDER_IS_ZERO: bool = false;

/// Whether the two consumed coins' blinders cancel.
///
/// False. That is the whole difference between this merge and the one
/// the registry refuses: merging both halves of the dual-parity
/// predecessor's inverse pair gives a consumed sum of zero, and
/// merging two coins of a three-output predecessor does not, because
/// three blinders summing to zero cancel in no pair.
pub const MERGE_CONSUMED_PAIR_CANCELS: bool = false;

/// The merge's wall time, in seconds.
pub const MERGE_WALL_SECONDS: f64 = 10.9;

// --- The fee-bearing shape: ACCEPTED, after three refusals ---------

/// The fee-bearing one-to-one's recorded successor fixture digest under fixture-digest v1.
///
/// This value and `FEE_BEARING_SUCCESSOR_IDENTITY` are the immutable halves of one historical
/// observation. The same fixture's forward v2 digest is separate and awaits its own
/// node-accepted run.
pub const FEE_BEARING_SUCCESSOR_DIGEST: &str =
    "32d0a5a76d75c91fab564731e19f29518b4dda492a830f3d4b9ee578ea25f3f8";

/// The identity a real node computed for the fee-bearing transfer.
///
/// One receipt consumed, one blinded destination created, and the
/// transaction's own fee paid out of the value it consumed, with no
/// sponsor anywhere in it. The node accepted it and mined it, and the
/// bytes it handed back are equal to the bytes submitted.
///
/// # Three refusals stood between the vocabulary and this figure
///
/// Each was a layer the one before it uncovered, and each was a
/// first-party defect rather than a property of the target. The
/// registry had no fee output role. The materializer had no fee
/// projection, so a fee would have been blinded. Then the shape
/// vocabulary had no sponsorless fee-bearing member, so a
/// two-destination request selected a two-receipt-output shape and
/// the receipt covenant demanded a receipt program where the fee's
/// empty one sat.
///
/// Giving the vocabulary the member uncovered a fourth, which is the
/// pattern holding rather than breaking: the deployment was welded to
/// a fee-program digest of 0xb5 bytes that no program hashes to, kept
/// deliberately so the demonstration's taptree would not move, and
/// nothing had ever executed the clause that reads it.
pub const FEE_BEARING_ACCEPTED_IDENTITY: Option<&str> =
    Some("0df30bc5d4832115fe68aa1082f3bafd762d5d50e1aa08b72f4f0187386a8922");

/// The same identity, as the register cites an acceptance.
///
/// The `Option` above says whether an acceptance exists; a consensus
/// verdict needs the identity itself. Written once and read from
/// there, so the two can never disagree about what was accepted.
pub const FEE_BEARING_SUCCESSOR_IDENTITY: &str =
    "0df30bc5d4832115fe68aa1082f3bafd762d5d50e1aa08b72f4f0187386a8922";

/// How many bytes the fee-bearing one-to-one handed to the node.
pub const FEE_BEARING_SUBMITTED_BYTES: usize = 4_927;

/// The output-witness entries the fee-bearing candidate carried.
///
/// This array is the fee projection's own evidence, and it is the
/// reason the run is worth recording despite the refusal. The blinded
/// output carries a range proof of the usual size; the FEE output
/// carries an empty entry. A fee that had been mapped onto the
/// balancing role would read `[4_174, 4_174]` here — a blinded fee,
/// and not a fee at all.
pub const FEE_BEARING_PROOF_BYTES: [usize; 2] = [4_174, 0];

/// The verdict the target returned when the shape vocabulary had no
/// member for this form.
///
/// KEPT, and kept deliberately, though the shape is now accepted. It
/// is the diagnosis that located the third layer, and the register's
/// discipline is that a wall's history survives its removal -- a
/// removal whose refusal has been deleted cannot be checked against
/// what it claims to have removed.
///
/// A script-path rejection, at the workspace's OWN receipt covenant
/// rather than at any confidential rule. The candidate's value balance
/// was never reached and nothing here is a statement about the target's
/// fee rules: Elements admits a fee output in a non-policy asset at
/// consensus and at policy alike, and this node runs with a zero
/// minimum relay feerate, so the transaction did not fail for carrying
/// a fee.
///
/// What it failed is the covenant the shape selection built for it.
/// The request states two destinations, the reviewed live-transfer
/// shape vocabulary read a two-destination sponsorless shape as TWO
/// RECEIPT OUTPUTS, and the receipt covenant therefore required the
/// second output to carry the second owner's private receipt
/// constructor program. The second output is the fee, whose program is
/// empty, so the comparison failed.
pub const FEE_BEARING_OBSERVED_DETAIL: &str =
    "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)";

/// The fee-bearing run's wall time, in seconds.
pub const FEE_BEARING_WALL_SECONDS: f64 = 12.4;

// --- The exit crossing: representation crossed at a real node -------

/// The exit crossing's recorded successor fixture digest under fixture-digest v1.
///
/// This value and `EXIT_CROSSING_ACCEPTED_TXID` are the immutable halves of one historical
/// observation. The same fixture's forward v2 digest is separate and awaits its own
/// node-accepted run.
pub const EXIT_CROSSING_SUCCESSOR_DIGEST: &str =
    "9c2b302c4becdff2ed90545006f0d371e69e0a7eabce71c33a2a8cbe60f44fbe";

/// The identity the target computed for the accepted exit crossing.
///
/// TWO blinded receipts consumed, TWO EXPLICIT destinations created,
/// and ONE blinded absorber beside them. It is the first transaction
/// this workspace has built whose consumed and created sides are read
/// under DIFFERENT representation plans, and the first evidence that
/// the target admits one -- which it always did, upstream having
/// tested the shape directly; what did not exist was a vocabulary in
/// which the candidate could be stated.
pub const EXIT_CROSSING_ACCEPTED_TXID: &str =
    "89a372c9876ef3d95b30567bbd0e54f945820c2156d90f6223717ab7abe52243";

crate::recorded_acceptance::mint_recorded_acceptance!(
    exit_crossing_accepted,
    EXIT_CROSSING_ACCEPTED_TXID
);

/// How many bytes the exit crossing handed to the node.
pub const EXIT_CROSSING_SUBMITTED_BYTES: usize = 5_412;

/// The output-witness proof bytes the exit crossing carried.
///
/// THE CENSUS THAT MAKES THE CROSSING VISIBLE IN THE BYTES, and it
/// is read off the candidate rather than predicted: two entries
/// EMPTY and one carrying a range proof. An explicit value admits no
/// range proof and an explicit asset no surjection proof, so the two
/// explicit destinations carry neither, and the single blinded
/// absorber carries the one proof the transaction has.
///
/// A wholly private shape of this arity would carry three proofs and
/// a wholly explicit one none, so this vector is a shape no
/// homogeneous transfer can produce.
pub const EXIT_CROSSING_PROOF_BYTES: [usize; 3] = [0, 0, 4_174];

/// Whether the exit crossing's consumed pair cancels.
///
/// FALSE, and it is load-bearing rather than incidental. A canceling
/// pair presents a zero blinder sum, the absorber's solved blinder
/// would be zero, and an absorber that hides nothing is not an
/// absorber -- the registry refuses exactly that as a degenerate
/// balancing scalar. This shape spends the merge's own non-canceling
/// predecessor for that reason.
pub const EXIT_CROSSING_CONSUMED_PAIR_CANCELS: bool = false;

/// The weight the target reported for the exit crossing.
pub const EXIT_CROSSING_TARGET_WEIGHT: u64 = 6_561;

/// The exit crossing's wall time, in seconds.
pub const EXIT_CROSSING_WALL_SECONDS: f64 = 11.5;

// --- The entry crossing: the other direction, at a real node --------

/// The entry crossing's recorded successor fixture digest under fixture-digest v1.
///
/// This value and `ENTRY_CROSSING_ACCEPTED_TXID` are the immutable halves of one historical
/// observation. The same fixture's forward v2 digest is separate and awaits its own
/// node-accepted run.
pub const ENTRY_CROSSING_SUCCESSOR_DIGEST: &str =
    "2a2d580164d9ce541b9892d0ab6692e4c0770f8458b0534a2dc2f8e3af08da3f";

/// The identity the target computed for the accepted entry crossing.
///
/// ONE EXPLICIT receipt consumed and TWO blinded destinations
/// created. This workspace has performed the SHAPE every ceremony,
/// as the funding step that mints a confidential predecessor; what
/// this identity records is the first time the coin it spent sat at
/// a receipt constructor's program, so the transfer was governed by
/// the covenant rather than by the adapter.
pub const ENTRY_CROSSING_ACCEPTED_TXID: &str =
    "6692af349d8a5aec218c3f91643b85e17df0a1852e7aaa10d64c1271c11c7b75";

crate::recorded_acceptance::mint_recorded_acceptance!(
    entry_crossing_accepted,
    ENTRY_CROSSING_ACCEPTED_TXID
);

/// How many bytes the entry crossing handed to the node.
pub const ENTRY_CROSSING_SUBMITTED_BYTES: usize = 9_133;

/// The output-witness proof bytes the entry crossing carried.
///
/// TWO proofs for two blinded destinations, and the count is the
/// claim: a single blinded output would have been forced to a ZERO
/// blinder, because an explicit input contributes one, and its
/// commitment would have hidden nothing. Two is the floor, and it is
/// the registry's own arithmetic rather than a preference.
pub const ENTRY_CROSSING_PROOF_BYTES: [usize; 2] = [4_174, 4_174];

/// Whether the entry crossing's consumed coin carried a blinder.
///
/// FALSE. Its value was explicit, so the blinder it contributed to
/// the transaction-wide sum was the all-zero one every explicit
/// value is committed with -- which is why the balancing output's
/// blinder is the negation of a searched non-zero primary rather
/// than a consumed sum.
pub const ENTRY_CROSSING_CONSUMED_A_BLINDER: bool = false;

/// The weight the target reported for the entry crossing.
///
/// Larger than the exit crossing's, and the reason is the crossing
/// itself rather than the arity: two blinded outputs carry two range
/// proofs where the exit crossing's one blinded absorber carries
/// one, and a range proof is most of what a confidential output
/// weighs. The exit direction is CHEAPER on the wire, which is the
/// same fact its shorter form obligation states in the covenant.
pub const ENTRY_CROSSING_TARGET_WEIGHT: u64 = 10_093;

/// The entry crossing's wall time, in seconds.
pub const ENTRY_CROSSING_WALL_SECONDS: f64 = 9.9;

/// The identity the target computed for the accepted PURE split.
///
/// One receipt consumed, TWO created, both of them receipts, no
/// change and no fee. It moves no §15.2 row: `private-split` moved
/// on the three-output split and a row does not move twice. What it
/// answers is §16.2's second conjunct for the SPLIT pair, whose
/// private member is this shape and not that one.
pub const PURE_SPLIT_ACCEPTED_TXID: &str =
    "46bfe77d90b32ac47a9a7ef5bb1f0fa4723ec02b2a9fb3a3e50a5ea3777f4d72";

crate::recorded_acceptance::mint_recorded_acceptance!(
    pure_split_accepted,
    PURE_SPLIT_ACCEPTED_TXID
);

/// How many bytes the pure split handed to the node.
pub const PURE_SPLIT_SUBMITTED_BYTES: usize = 9_136;

/// The output-witness proof bytes the pure split carried.
///
/// TWO proofs for two created outputs, and the count is the whole
/// claim this run exists to make. A range proof is what a BLINDED
/// output carries and a fee output carries none, so two proofs over
/// two outputs is the measured form of "both created outputs are
/// receipts" -- the conjunct the fee-bearing one-in-two-out run
/// could not satisfy, its second output having been a fee.
pub const PURE_SPLIT_PROOF_BYTES: [usize; 2] = [4_174, 4_174];

/// The weight the target reported for the pure split.
pub const PURE_SPLIT_TARGET_WEIGHT: u64 = 10_096;

/// The pure split's recorded successor fixture digest under fixture-digest v1.
///
/// This value and `PURE_SPLIT_ACCEPTED_TXID` are the immutable halves of one historical
/// observation. The same fixture's forward v2 digest is separate and awaits its own
/// node-accepted run.
pub const PURE_SPLIT_SUCCESSOR_DIGEST: &str =
    "e9a68f1d3d93350fdf02f879f392e4b86f57ad3295cda42735c301564bd9c3a3";

/// The pure split's wall time, in seconds.
pub const PURE_SPLIT_WALL_SECONDS: f64 = 14.1;
