//! Historical sponsored-shape observations.

use target_elements_conformance::protocol::ObservedOutcomeLayer;

/// The identity the target computed for the accepted sponsor-signed
/// explicit control that takes NO change.
///
/// 1480 bytes submitted and 1480 read back from the node's own copy,
/// equal to the submitted bytes; mined at height 6; a two-item
/// sponsor witness of 72 and 33 bytes replayed from the adapter's
/// answer; both owner signatures verified out of that copy against
/// messages recomputed here, per input rather than per transaction.
pub const SPONSORED_ACCEPTED_TXID: &str =
    "8528d455cfd7e2cc92e88f2f0432bd0faed8c6f6417c675573a6b4963e1c01b2";

crate::recorded_acceptance::mint_recorded_acceptance!(sponsored_accepted, SPONSORED_ACCEPTED_TXID);

/// How many bytes the without-change control handed the node.
pub const SPONSORED_SUBMITTED_BYTES: usize = 1_480;

/// The weight the target itself computed for it.
pub const SPONSORED_TARGET_WEIGHT: u64 = 2_482;

/// The fee the target weighed, in the reserve asset.
///
/// The same figure BOTH runs declare. Holding it equal is what makes
/// the two submissions comparable at all.
pub const SPONSORED_FEE_WEIGHED: u64 = 250;

/// Whether the without-change control crossed the relay boundary
/// before the mine.
///
/// Read off the submission path rather than assumed: the adapter
/// offers a submission to `testmempoolaccept` first and reports an
/// acceptance only where that answered allowed, then confirms with
/// `generateblock`.
pub const SPONSORED_CROSSED_RELAY_AND_BLOCK: bool = true;

/// The identity the target computed for the accepted sponsor-signed
/// explicit control that TAKES CHANGE.
///
/// The first sponsored control in this workspace to carry a change
/// role. 1636 bytes submitted and read back equal, mined at height
/// 6, and the change output observed in the node's own copy at
/// position 2 rather than inferred from the request: the reserve
/// asset, the offered amount, and the deployment's own
/// sponsor-change program.
pub const SPONSORED_CHANGE_ACCEPTED_TXID: &str =
    "f4341effcab6f205c15d863f9c54071dc0ade4ca538d0247e4213baaa5e42d31";

crate::recorded_acceptance::mint_recorded_acceptance!(
    sponsored_change_accepted,
    SPONSORED_CHANGE_ACCEPTED_TXID
);

/// How many bytes the with-change control handed the node.
///
/// One hundred and fifty-six more than the without-change control,
/// which is the change output and the sponsor input's larger amount.
pub const SPONSORED_CHANGE_SUBMITTED_BYTES: usize = 1_636;

/// The weight the target itself computed for it.
pub const SPONSORED_CHANGE_TARGET_WEIGHT: u64 = 2_872;

/// What the sponsor coin was funded to for the with-change run.
///
/// Above the offer, which is the whole of what a sponsored control
/// taking change was missing: the construction places a change
/// output only where the offer states a change amount, and an offer
/// can only state one where the coin holds more than the fee.
pub const SPONSORED_CHANGE_SPONSOR_FUNDED: u64 = 1_250;

/// What the sponsor took back.
pub const SPONSORED_CHANGE_TAKEN: u64 = 1_000;

/// Which output position the change role occupied.
///
/// Read out of the node's copy of the mined transaction, located by
/// the deployment's own sponsor-change program rather than by
/// counting: a position is what a shape degraded to the
/// without-change form would still have, and the program is what it
/// would not.
pub const SPONSORED_CHANGE_OUTPUT_POSITION: usize = 2;

/// The identity the target computed for the transaction that FUNDED
/// the committed sponsor coin.
///
/// An ACCEPTANCE, and the one this wave's flag rests on. This
/// transaction is the executor's own rather than a candidate, so
/// what its acceptance establishes is that a coin of this form
/// exists on a chain — and nothing whatever about a candidate that
/// spends one.
///
/// Recorded rather than described, on the rule every identity in
/// this module follows: a ceremony edited after its run cites an
/// identity for something else unless something fails.
pub const COMMITTED_SPONSOR_FUNDING_TXID: &str =
    "4f253dd1e4f731c58401442232b689a70187241a555b6e55696951d0428d4a57";

crate::recorded_acceptance::mint_recorded_acceptance!(
    committed_sponsor_funding_accepted,
    COMMITTED_SPONSOR_FUNDING_TXID
);

/// The weight the target computed for that funding transaction.
///
/// Wide beside the candidate's, and the width is the representation
/// rather than an inefficiency: two committed outputs carry a range
/// proof each, and a range proof is most of what a confidential
/// output weighs.
pub const COMMITTED_SPONSOR_FUNDING_WEIGHT: u64 = 9_904;

/// What the target answered a candidate spending a COMMITTED sponsor
/// coin, at the layer it answered.
///
/// A refusal, and a refusal this arc went looking for an acceptance
/// of. It is recorded as the target typed it because that is the
/// whole value of it: the shape was offered, and the answer came
/// back from consensus rather than from a reading.
pub const COMMITTED_SPONSOR_REFUSAL: &str = "bad-txns-in-ne-out";

/// How many bytes the committed candidate handed the node.
///
/// The SAME number the explicit control handed it, and the equality
/// is the attributability. A candidate names the coin it spends by
/// outpoint alone, so the sponsor coin's value form is not in these
/// bytes at all: the two submissions are the same shape at the same
/// width, differing in which coin they reach for. The refusal is
/// therefore attributable to the value form and to nothing the
/// candidate did differently.
pub const COMMITTED_SPONSOR_SUBMITTED_BYTES: usize = 1_636;

/// The weight the target computed for it, which is likewise the
/// control's.
pub const COMMITTED_SPONSOR_TARGET_WEIGHT: u64 = 2_872;

/// Whether any ceremony in this workspace FUNDS a sponsor coin whose
/// value is committed.
///
/// `true`, and a real node holds one.
///
/// # What was built, and what a node said about it
///
/// The executor grew a funding stage that mines a coin whose VALUE
/// is a commitment and whose ASSET stays explicit, against a
/// registered fixture, and it retains the value FIELD rather than an
/// amount so a later signing step has something to sign against. The
/// transaction that creates the coin is accepted and mined, and the
/// commitment the chain holds is the one this workspace derives from
/// published constants and no chain at all.
///
/// So the question this constant asks is answered: a blinded sponsor
/// value is funded, and the site that funds it is
/// `fund_confidential_sponsor` in the native executor, bound to the
/// `ctf-v1/sponsor-reserve-dual-parity` case and paid to the program
/// that executor can authorize a spend of.
///
/// # What the SAME run established that no reading had
///
/// That such a coin cannot be spent by an explicit-lane candidate at
/// all. The candidate was built, owner-signed, sponsor-signed and
/// offered, and the target refused it at consensus before script
/// with [`COMMITTED_SPONSOR_REFUSAL`], which is its balance check.
///
/// The workspace's reading of that verdict, stated as a reading: the
/// reserve sub-equation is the sponsor input against the fee and the
/// change, the input now carries a blinder, and both outputs that
/// spend it are explicit and therefore carry none. Nothing in the
/// transaction absorbs the input's blinder, so the sum cannot close
/// whatever the amounts are. A fee is mandatorily explicit, so the
/// only term that COULD absorb it is the sponsor's change, which
/// means a candidate spending a committed sponsor coin must return
/// COMMITTED change -- the materializer's shape rather than the
/// explicit lane's.
///
/// # This overturns a reading recorded elsewhere, and running is
/// what overturned it
///
/// `recognize_sponsors` records that the two value forms are
/// independently choosable and that an explicit transfer's sponsor
/// could carry a commitment, filed as unbuilt. The first half
/// stands: nothing guards the form, and construction produced the
/// transaction without complaint. The second does not. The shape is
/// not merely unbuilt, it is one a target refuses, and the refusal
/// is arithmetic rather than policy. Building it is what found that
/// out, which is the sixth reading in this arc that running
/// overturned.
///
/// # What this does NOT settle
///
/// Whether the sponsored confidential shape is accepted. No such
/// shape has been offered to a node, and the section 15.2
/// `private-sponsor-values` row does not move on this run. What
/// moved is that the coin exists on a chain and that the explicit
/// route to spending one is closed by consensus rather than by
/// effort.
pub const A_BLINDED_SPONSOR_VALUE_IS_FUNDED_ANYWHERE: bool = true;

/// Whether a sponsored PRIVATE successor can be REGISTERED.
///
/// `true`, and a registered case is what a real node accepted.
///
/// # What the stop was, and what answered it
///
/// The private lane admitted a sponsored request at every site that
/// had refused one, and the candidate it would build was
/// unregistrable one layer below: `fixture_of` requires EVERY
/// destination to name the same fixture handle, and a sponsored
/// private successor has two protocol-asset receipt outputs beside a
/// RESERVE-asset sponsor change. So the three had to come from ONE
/// registered case declaring two assets across its positions, and a
/// registry output carried a role, an amount and a program with no
/// asset at all while the manifest carried a single explicit asset
/// for the whole case.
///
/// The widening was ROLE-KEYED, which is what made it affordable. A
/// `SponsorChange` member joins `FixtureOutputRole` carrying its own
/// asset, and the transcript's per-output asset field — which it
/// already emitted, sourced from the manifest — is now sourced by
/// ROLE. No field was added, no presence flag, no position moved, so
/// a manifest with no sponsor change hashes the bytes it always
/// hashed. Every recorded digest re-derives bit-for-bit under the
/// byte-identity tests, and the whole live set reproduced at the
/// widened tip.
///
/// # The two walls between the registry and the chain
///
/// Neither was the registry's, and both were met by running rather
/// than by reading. The first was this ceremony seeding the owner
/// message with the genesis identity as a target PRINTS it rather
/// than as it HASHES it — the recorded `DeploymentSeedInPrintedOrder`
/// negative control, reproduced by accident and drawing the node's
/// own invalid-signature verdict while every balance check passed.
/// The second was the deployment vocabulary: a sponsored request
/// bears a fee by construction, so the candidate executes the
/// covenant's fee clause, and the DEMONSTRATION deployment is welded
/// to a fixture fee digest no program hashes to. That is the defect
/// the sponsor arc met on the explicit lane, met here for the same
/// reason and answered the same way, by linking the fee-bearing
/// vocabulary — whose taptree is its own, so the demonstration's
/// committed identity did not move to buy it.
pub const A_SPONSORED_PRIVATE_SUCCESSOR_IS_REGISTRABLE: bool = true;

/// The identity of the accepted sponsored CONFIDENTIAL control.
///
/// The run the section 15.2 `private-sponsor-values` row moved on,
/// and the first acceptance anywhere of a sponsored private
/// successor. Its shape: a blinded sponsor coin in at an explicit
/// asset, two blinded receipt destinations, a COMMITTED sponsor
/// change in the reserve asset, and an explicit reserve fee in the
/// non-protocol funding region, outside both balance equations.
pub const SPONSORED_PRIVATE_TXID: &str =
    "ebd8a02a47e6c97c2e75fb5db9deaf9d5566cc55688a752e365299de39023ed9";

crate::recorded_acceptance::mint_recorded_acceptance!(
    sponsored_private_accepted,
    SPONSORED_PRIVATE_TXID
);

/// The bytes that reached the node for it.
pub const SPONSORED_PRIVATE_SUBMITTED_BYTES: usize = 13_873;

/// The identity of the accepted sponsored private control whose
/// sponsor value is EXPLICIT and whose offer asks no change back.
///
/// §16.1's sponsor pair's private member, run as its own shape. Its
/// shape: an explicit sponsor coin funded to EXACTLY the fee, ONE
/// blinded receipt destination declaring the fully-solved form, no
/// change role anywhere, and an explicit reserve fee in the
/// non-protocol funding region.
///
/// # It does not contradict the committed run beside it
///
/// The sponsor arc observed that a COMMITTED sponsor value requires
/// committed change, and that observation is about a blinded
/// input's blinder needing a term to absorb it. An explicit sponsor
/// coin brings the all-zero blinder, so there is nothing to absorb
/// and no change is owed. The two runs are the two halves of that
/// rule rather than a rule and an exception to it.
///
/// # What the acceptance rules out
///
/// A change output. The target balances per asset, so the reserve
/// sub-equation is `sponsor_input == fee + change`; this coin was
/// funded to exactly the fee, so any change term at all would leave
/// it short and this candidate would have been refused. It was
/// accepted, so the change term is zero -- which is a stronger
/// statement than a byte scan for the reserve asset could make,
/// the fee carrying that asset too.
pub const SPONSORED_PRIVATE_EXPLICIT_NO_CHANGE_TXID: &str =
    "d1f22066ded41dc3ded004910ff89e3c9a7c432206dc8d3ed47072cc379b1ac0";

crate::recorded_acceptance::mint_recorded_acceptance!(
    sponsored_private_explicit_no_change_accepted,
    SPONSORED_PRIVATE_EXPLICIT_NO_CHANGE_TXID
);

/// The bytes that reached the node for the explicit no-change run.
///
/// A THIRD of the committed run's, and the ratio is the shape: one
/// blinded output carries one range proof where three blinded
/// outputs carry three.
pub const SPONSORED_PRIVATE_EXPLICIT_NO_CHANGE_SUBMITTED_BYTES: usize = 5_122;

/// The weight the target computed for the explicit no-change run.
pub const SPONSORED_PRIVATE_EXPLICIT_NO_CHANGE_TARGET_WEIGHT: u64 = 5_935;

/// The explicit no-change run's wall time, in seconds.
pub const SPONSORED_PRIVATE_EXPLICIT_NO_CHANGE_WALL_SECONDS: f64 = 14.2;

/// The weight the target computed for it.
pub const SPONSORED_PRIVATE_TARGET_WEIGHT: u64 = 15_490;

/// Whether any ceremony in this workspace builds a sponsored control
/// that TAKES CHANGE.
///
/// `true`, and the running is what changed it.
///
/// # The obstacle was not the one the spike predicted, and running
/// decided it
///
/// Two readings stood against each other. One said the demonstration
/// deployment's sponsor-change program symbol is a fixture pattern
/// no program hashes to, so a control taking change would die at its
/// own change-role check the way the first sponsored controls died
/// at the fee-role check. The other said the two symbols only look
/// alike: the FEE role's program is target-structural, so
/// construction wrote the empty program while the symbol was
/// arbitrary and the two disagreed, whereas the CHANGE role's
/// program is a deployment's own choice and construction writes the
/// change output FROM the symbol — refusing outright if a sponsor
/// capability offers any other destination.
///
/// The second reading is the one that survived. The control was
/// accepted at the first attempt with the symbol untouched: nothing
/// was threaded, nothing was repointed, the committed taptree did
/// not move, and [`SPONSORED_ACCEPTED_TXID`] reproduced beside it.
/// What was actually missing was an OFFER that carries change, and
/// what supplies one is a sponsor funding step that funds above the
/// fee.
pub const A_SPONSORED_CONTROL_TAKING_CHANGE_EXISTS: bool = true;

/// What the target said to a sponsored control whose sponsor input
/// carries NO authorization.
///
/// Its own words, verbatim, from a run that offered the mutant
/// FIRST and then the unmutated control to one node on one chain.
/// The mutant is the recording pass's own completion — one
/// finalization, every owner really signing, the sponsor capability
/// answering with an empty stack — so it differs from the control in
/// the sponsor witness and in nothing else.
///
/// # The verdict reads like an earlier wave's and is NOT it
///
/// This exact string is what the first sponsored controls drew from
/// the covenant's own FEE-ROLE comparison, before the fee program's
/// digest was threaded. It is not that comparison here, and what
/// says so is the control accepted in the SAME run: the fee-role
/// check passes for this deployment, so the comparison that failed
/// is one the sponsor witness reaches.
///
/// Which comparison that is follows from the sponsor program's
/// class. The admitted class is the target's version-zero key hash,
/// whose evaluation duplicates the offered public key, hashes it,
/// and compares the digest against the one the program commits to.
/// The mutant offers an EMPTY item where the key goes, so the hash
/// of nothing meets the committed digest and the comparison fails
/// before any signature is judged. The row asks for a missing
/// authorization to be refused, and it was refused at the first
/// check a missing authorization reaches.
pub const MISSING_SPONSOR_AUTHORIZATION_REFUSAL: &str =
    "mandatory-script-verify-flag-failed (Script failed an OP_EQUALVERIFY operation)";

/// How many bytes the unauthorized mutant handed the node.
///
/// One hundred and five fewer than the control, which is exactly the
/// seventy-two-byte signature and the thirty-three-byte public key
/// the adapter returned and this offering does not carry. MEASURED
/// at the node rather than argued from the code that built the two.
pub const MISSING_SPONSOR_AUTHORIZATION_SUBMITTED_BYTES: usize = 1_375;

/// The layer the unauthorized sponsor mutant was refused at, TYPED.
///
/// The words above trace the refusal to a comparison inside the
/// sponsor program's own evaluation, which is the leaf running — so
/// the layer is the script path, and this constant is that fact in
/// the vocabulary rather than in prose.
pub const MISSING_SPONSOR_AUTHORIZATION_OBSERVED_LAYER: ObservedOutcomeLayer =
    ObservedOutcomeLayer::ScriptPathRejection;
