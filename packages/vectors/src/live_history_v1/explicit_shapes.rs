//! Historical explicit-shape acceptance observations.

use crate::live_explicit_shapes::{
    RECEIPT_AMOUNT as CURRENT_RECEIPT_AMOUNT, SELF_PAID_FEE_AMOUNT as CURRENT_SELF_PAID_FEE_AMOUNT,
};

/// The disposable asset every run issued.
pub const ISSUED_ASSET: &str = "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

/// One receipt consumed, one created, sponsorless.
///
/// Cited by `one-input-to-one-output` and by `sponsorless`. The
/// smallest submission this lane has made, 593 bytes, for the
/// structural reason that an explicit transfer carries no range
/// proof at all.
pub const ONE_TO_ONE_ACCEPTED_TXID: &str =
    "872a2294da5ea650a7a74ffd8a5932210930ab70d6a08a991eb3ea471ee29abb";

crate::recorded_acceptance::mint_recorded_acceptance!(
    one_to_one_accepted,
    ONE_TO_ONE_ACCEPTED_TXID
);

/// How many bytes the one-to-one handed the node.
pub const ONE_TO_ONE_SUBMITTED_BYTES: usize = 593;

/// One receipt consumed, TWO created for two distinct owners.
///
/// Cited by `one-input-split-into-two` and by
/// `several-destination-owners`.
pub const SPLIT_ACCEPTED_TXID: &str =
    "0fcf267058e83e06e87a950bbeec920a15e3641ef7f76c55df0a8fff544e64c1";

crate::recorded_acceptance::mint_recorded_acceptance!(split_accepted, SPLIT_ACCEPTED_TXID);

/// How many bytes the split handed the node.
pub const SPLIT_SUBMITTED_BYTES: usize = 751;

/// TWO receipts consumed, ONE created.
///
/// Cited by `several-inputs-merged-into-one` and by
/// `canonical-input-normalization`, the second because the
/// normalization run offered the same two receipts in the reverse
/// order and the node computed this same identity for what it built.
pub const MERGE_ACCEPTED_TXID: &str =
    "7a0ac33f0268e48ebeb1316dbc262c8d40569ba5c96274d1b8262f394c6f7c39";

crate::recorded_acceptance::mint_recorded_acceptance!(merge_accepted, MERGE_ACCEPTED_TXID);

/// How many bytes the merge handed the node.
pub const MERGE_SUBMITTED_BYTES: usize = 1_006;

/// TWO receipts consumed, TWO created.
pub const SEVERAL_TO_SEVERAL_ACCEPTED_TXID: &str =
    "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029";

crate::recorded_acceptance::mint_recorded_acceptance!(
    several_to_several_accepted,
    SEVERAL_TO_SEVERAL_ACCEPTED_TXID
);

/// TWO receipts under ONE owner, three outputs created.
///
/// The repetition is the subject: one published owner authorized two
/// separate inputs, each at its own position and each over its own
/// recomputed message, and both signatures verify out of the node's
/// own copy.
pub const REPEATED_OWNER_ACCEPTED_TXID: &str =
    "3f833570061c28f1b6cae0cd2abda65ed2c573bf62f418e847836a7999382114";

crate::recorded_acceptance::mint_recorded_acceptance!(
    repeated_owner_accepted,
    REPEATED_OWNER_ACCEPTED_TXID
);

/// TWO receipts under two DISTINCT published owners.
///
/// Its destinations are the several-to-several run's exactly, and
/// the identities differ anyway — because the SPENT programs differ,
/// one coin having been paid to each owner's explicit constructor.
/// That the two runs diverge on their input side alone is what makes
/// this run about its input owners.
pub const SEVERAL_DISTINCT_OWNERS_ACCEPTED_TXID: &str =
    "f87e1ef327f69fe1f6de5f763cc73d14edbe9425372f7a451d79d8e30e42b660";

crate::recorded_acceptance::mint_recorded_acceptance!(
    several_distinct_owners_accepted,
    SEVERAL_DISTINCT_OWNERS_ACCEPTED_TXID
);

/// TWO destinations, both created for ONE owner.
pub const ONE_DESTINATION_OWNER_ACCEPTED_TXID: &str =
    "c9bd2bd7ea47f2e7df3d95751d008e2db2448d6b9611425114b06e09d7a2a0a8";

crate::recorded_acceptance::mint_recorded_acceptance!(
    one_destination_owner_accepted,
    ONE_DESTINATION_OWNER_ACCEPTED_TXID
);

/// Destinations at the boundary values the request type admits.
///
/// One and the remainder. The smallest is ONE because
/// [`transaction::live_request::ProtocolValue`] refuses zero by name, so
/// the value is the boundary the type states rather than a small number somebody
/// picked — and the node accepted it, which is the fact worth
/// having: nothing on this chain turned a one-unit output away.
pub const BOUNDARY_VALUES_ACCEPTED_TXID: &str =
    "a53626927129a23c37681973eede8d23996743a1fb04adc71853e2bfe0d608be";

crate::recorded_acceptance::mint_recorded_acceptance!(
    boundary_values_accepted,
    BOUNDARY_VALUES_ACCEPTED_TXID
);

/// THREE receipts consumed, the candidate's stated input bound.
///
/// The widest submission of this table at 1421 bytes.
pub const MAXIMUM_INPUTS_ACCEPTED_TXID: &str =
    "fce6e069897f841297803e36da5d51f3e7b4e15422ff0083a1cac4735147c112";

crate::recorded_acceptance::mint_recorded_acceptance!(
    maximum_inputs_accepted,
    MAXIMUM_INPUTS_ACCEPTED_TXID
);

/// THREE destinations created, the candidate's stated output bound.
pub const MAXIMUM_OUTPUTS_ACCEPTED_TXID: &str =
    "6a5617cc547f0fe22cefa261ed2fb885a82a9e7293aefad76be5c35f0b531613";

crate::recorded_acceptance::mint_recorded_acceptance!(
    maximum_outputs_accepted,
    MAXIMUM_OUTPUTS_ACCEPTED_TXID
);

// --- The fourteenth shape: the sponsorless self-paid fee ----------
//
// The last cell of the owner fee matrix, and the only row here that
// is not a §15.1 row at all. Its figures are kept in this register
// rather than in the §15 tables because §15.1's explicit positive
// table is complete at sixteen rows and none of the sixteen is a
// self-paying transfer.

/// The identity the target computed for the self-paying run.
///
/// An OPTION carrying a value: the node answered, and this is what
/// it answered. The confidential lane's fee-bearing identity is
/// written the same way for the same reason -- a record of what a
/// node answered holds a digest only once one has, and a placeholder
/// shaped like an identity would be indistinguishable from an
/// observation.
///
/// It collides with no other row's identity, which the collision
/// census checks rather than assumes.
pub const SELF_PAID_FEE_ACCEPTED_IDENTITY: Option<&str> =
    Some("72fad04b346a8ea93c97cd415d24434529d8a5d9ae148a13da3694047e117328");

/// The witness identity of the same accepted transaction.
///
/// Recorded beside the identity because they DIFFER, and the
/// difference is the ordinary one: the transaction carries a
/// witness, so the two hashes are taken over different bytes.
pub const SELF_PAID_FEE_WITNESS_IDENTITY: &str =
    "fafc82e7396f27e6379572cc958c710412b1ae0e0d69912a04212aa79c7d2822";

/// The disposable asset the self-paying run issued.
pub const SELF_PAID_FEE_ISSUED_ASSET: &str =
    "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

/// How many bytes the self-paying candidate handed the node.
pub const SELF_PAID_FEE_SUBMITTED_BYTES: usize = 782;

/// The weight the TARGET reported, read off `decoderawtransaction`.
///
/// The node's own figure rather than one computed here, so §20.5's
/// comparison is against an observation.
pub const SELF_PAID_FEE_TARGET_WEIGHT: u64 = 1_304;

/// What the one consumed receipt held.
pub const SELF_PAID_FEE_CONSUMED: u64 = CURRENT_RECEIPT_AMOUNT;

/// The fee the self-paying run paid, in the PROTOCOL asset.
///
/// The protocol asset because the fee is funded out of the consumed
/// receipts and Elements balances per asset: a reserve-asset fee
/// beside no reserve-asset input dies at the tally.
pub const SELF_PAID_FEE_AMOUNT: u64 = CURRENT_SELF_PAID_FEE_AMOUNT;

/// What reached the one destination, the fee having been taken.
pub const SELF_PAID_FEE_DESTINATION: u64 = 4_750;

/// Whether the self-paying candidate crossed RELAY and then a block.
///
/// TRUE, and this is the figure the wave existed to obtain. The
/// adapter offers every submission to `testmempoolaccept` first and
/// records an acceptance only when the mempool ALLOWED it and a
/// block then included it, so an accepted layer here is a relay
/// verdict and a consensus verdict together.
///
/// It matters because this is the FIRST sponsorless form to face
/// relay on its own. Its predecessors paid no fee and travelled as
/// package children, which is why the ABI builds a sponsorless form
/// at the topology-restricted version; a form that pays its own fee
/// needs no package parent, and the open question was whether that
/// version would still relay standalone. It did.
pub const SELF_PAID_FEE_CROSSED_RELAY_AND_BLOCK: bool = true;

/// Seconds of wall time the self-paying run took.
pub const SELF_PAID_FEE_WALL_SECONDS: f64 = 6.0;
