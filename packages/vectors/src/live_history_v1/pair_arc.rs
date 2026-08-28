//! Historical paired explicit/private observation.

/// The identity the target computed for the accepted EXPLICIT member.
///
/// 593 bytes submitted and read back equal from the node's own copy,
/// mined at height 5, at the weight below; its one input's signature
/// verified out of that copy against a message recomputed here.
pub const EXPLICIT_MEMBER_ACCEPTED_IDENTITY: Option<&str> =
    Some("5b5dd09fc4e9e525c78f21965cff2039dd2e08b6afd59d8408c014ce3165c718");

/// The identity the target computed for the accepted PRIVATE member.
///
/// 4773 bytes submitted and read back equal, mined at height 8. The
/// eight-fold width beside its explicit twin is the REPRESENTATION
/// and not an inefficiency: one blinded output carries a range proof
/// and an explicit one carries nothing, and the two transactions move
/// the same amount between the same owners.
pub const PRIVATE_MEMBER_ACCEPTED_IDENTITY: Option<&str> =
    Some("4c7a48e8430a79d3b6198705f4a4874aeddd00a3e65ed09b439e64f1d5caa66b");

/// The disposable asset BOTH members carry.
///
/// One, which is what one issuance buys and what §6.6's exact
/// explicit `U` term needs. Two runs would have carried two.
pub const PAIR_ISSUED_ASSET: &str =
    "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

/// How many bytes the explicit member handed the node.
pub const EXPLICIT_MEMBER_SUBMITTED_BYTES: usize = 593;

/// The weight the target itself computed for it.
pub const EXPLICIT_MEMBER_TARGET_WEIGHT: u64 = 983;

/// How many bytes the private member handed the node.
pub const PRIVATE_MEMBER_SUBMITTED_BYTES: usize = 4_773;

/// The weight the target itself computed for it.
///
/// The disclosure-minimality measurement this arc can make that the
/// registry's own could not: both figures are the TARGET's, over two
/// transactions it accepted, rather than weights of serializations
/// whose proof slots are empty.
pub const PRIVATE_MEMBER_TARGET_WEIGHT: u64 = 5_331;

/// How many §6.6 terms the private member withholds the exact value
/// of.
///
/// TWO — the input amount multiset and the destination value half of
/// the destination multiset — and the figure is the whole reason the
/// pair is evidence about DISCLOSURE. Zero would mean the private
/// representation published everything the explicit one did.
pub const TERMS_WITHHELD_BY_THE_PRIVATE_MEMBER: usize = 2;

/// Whether the arc has produced a ledger at all.
///
/// The one flag the evidence matrix reads, and the flag
/// `A_PAIRED_ACCEPTED_PROJECTION_COMPARISON_EXISTS` is derived from.
/// It moves with the two identities above and its own test holds the
/// three together, so a flag claiming a ledger that no identity backs
/// does not compile past the suite.
pub const A_PAIR_ARC_LEDGER_EXISTS: bool = true;

/// How long the run took, in seconds.
pub const WALL_SECONDS: f64 = 20.6;
