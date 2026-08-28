//! Historical private-restart values and the complete typed v1 record.

use crate::live_private_restart::ConsumedReceipt;
use crate::recorded_acceptance::RecordedAcceptance;
use transaction::TransactionIdentityParseError;

/// One accepted member of the immutable historical V1 ceremony.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoricalPrivateRestartAcceptedMember {
    successor_digest: &'static str,
    acceptance: RecordedAcceptance,
    commitment_prefix: u8,
}

impl HistoricalPrivateRestartAcceptedMember {
    /// The historical V1 successor digest recorded beside this acceptance.
    #[must_use]
    pub const fn successor_digest(self) -> &'static str {
        self.successor_digest
    }

    /// The provenance-bearing accepted identity from the historical run.
    #[must_use]
    pub const fn acceptance(self) -> RecordedAcceptance {
        self.acceptance
    }

    /// The commitment prefix the historical run observed on its consumed coin.
    #[must_use]
    pub const fn commitment_prefix(self) -> u8 {
        self.commitment_prefix
    }
}

/// The historical ceremony's two distinct accepted parity members.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoricalPrivateRestartTwoAcceptanceLink {
    primary: HistoricalPrivateRestartAcceptedMember,
    balancing: HistoricalPrivateRestartAcceptedMember,
}

impl HistoricalPrivateRestartTwoAcceptanceLink {
    /// The run that consumed the predecessor's primary output.
    #[must_use]
    pub const fn primary(self) -> HistoricalPrivateRestartAcceptedMember {
        self.primary
    }

    /// The run that consumed the predecessor's balancing output.
    #[must_use]
    pub const fn balancing(self) -> HistoricalPrivateRestartAcceptedMember {
        self.balancing
    }

    /// Select the historical member for one consumed receipt.
    #[must_use]
    pub const fn for_receipt(
        self,
        consumed: ConsumedReceipt,
    ) -> HistoricalPrivateRestartAcceptedMember {
        match consumed {
            ConsumedReceipt::Primary => self.primary,
            ConsumedReceipt::Balancing => self.balancing,
        }
    }
}

/// The complete immutable payload of the historical V1 ceremony.
#[derive(Clone, Copy, Debug)]
pub struct HistoricalPrivateRestartV1 {
    issued_asset: &'static str,
    predecessor_digest: &'static str,
    acceptances: HistoricalPrivateRestartTwoAcceptanceLink,
    submitted_bytes: usize,
    output_witness_proof_bytes: [usize; 2],
    receipt_leaves: usize,
    wall_seconds: f64,
}

impl HistoricalPrivateRestartV1 {
    /// The disposable asset issued by the historical ceremony.
    #[must_use]
    pub const fn issued_asset(self) -> &'static str {
        self.issued_asset
    }

    /// The predecessor's historical fixture-digest V1 identity.
    #[must_use]
    pub const fn predecessor_digest(self) -> &'static str {
        self.predecessor_digest
    }

    /// Both accepted historical runs, one for each commitment parity.
    #[must_use]
    pub const fn acceptances(self) -> HistoricalPrivateRestartTwoAcceptanceLink {
        self.acceptances
    }

    /// How many bytes the primary historical run submitted.
    #[must_use]
    pub const fn submitted_bytes(self) -> usize {
        self.submitted_bytes
    }

    /// The proof byte counts carried by the primary historical run's outputs.
    #[must_use]
    pub const fn output_witness_proof_bytes(self) -> [usize; 2] {
        self.output_witness_proof_bytes
    }

    /// How many receipt leaves the historical control consumed.
    #[must_use]
    pub const fn receipt_leaves(self) -> usize {
        self.receipt_leaves
    }

    /// The primary historical run's wall time in seconds.
    #[must_use]
    pub const fn wall_seconds(self) -> f64 {
        self.wall_seconds
    }
}

/// The immutable historical private-restart run.
///
/// Its sole variant is V1 because the recorded identities belong to the
/// historical fixture-digest V1 ceremony. The inner payload has no public
/// constructor, so a forward V2 payload cannot be relabeled as history.
///
/// ```compile_fail
/// use vectors::live_history_v1::private_restart::HistoricalPrivateRestartRun;
/// use vectors::live_private_restart::forward_v2::{
///     ForwardPrivateRestartExpectation, forward_private_restart_expectation,
/// };
///
/// let ForwardPrivateRestartExpectation::V2(forward) =
///     forward_private_restart_expectation();
/// let _ = HistoricalPrivateRestartRun::V1(forward);
/// ```
#[derive(Clone, Copy, Debug)]
pub enum HistoricalPrivateRestartRun {
    /// The fixture-digest V1 ceremony already recorded by this module.
    V1(HistoricalPrivateRestartV1),
}

/// The disposable asset the run issued.
pub const ISSUED_ASSET: &str = "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

/// The predecessor fixture's digest.
pub const PREDECESSOR_DIGEST: &str =
    "ca43b210d6e74b76f7b3d3f79a6123556f150a7af9fa78e2571e2812b9d51fc2";

/// The successor fixture's digest.
pub const SUCCESSOR_DIGEST: &str =
    "31501b776502ee48d48b115d8bc80f55ba01bfc8cb6e163e3f2882848d025aa0";

/// The identity the target computed for the accepted control.
///
/// The whole of step one's evidence, in one string. Every row this
/// wave moves is moved on THIS acceptance and cites it.
pub const ACCEPTED_TXID: &str = "1af38f8a5292afcdb4dd38f78a146ff84d36db20d9916e768b7fd9b368b89e8e";

crate::recorded_acceptance::mint_recorded_acceptance!(accepted, ACCEPTED_TXID);

/// How many bytes were handed to the node.
pub const SUBMITTED_BYTES: usize = 9_136;

/// The range-proof bytes each of the candidate's outputs carried.
pub const OUTPUT_WITNESS_PROOF_BYTES: [usize; 2] = [4_174, 4_174];

/// How many receipt inputs the control consumed.
///
/// One. It is a one-to-one control, and the figure is here so that a
/// later reader does not have to take the word "one-to-one" for it.
pub const RECEIPT_LEAVES: usize = 1;

/// The commitment prefix the consumed coin carried.
///
/// The first of the target's two admitted parities, as the NODE
/// reported the commitment.
pub const CONSUMED_COMMITMENT_PREFIX: u8 = 0x08;

/// The run's wall time, in seconds.
pub const WALL_SECONDS: f64 = 11.5;

/// The successor fixture's digest for the second parity's run.
///
/// Different from [`SUCCESSOR_DIGEST`] because the two runs consume
/// different receipts and therefore balance against different
/// blinders and split different amounts. A pair of runs whose
/// successor digests agreed would be one run reported twice.
pub const PARITY_SUCCESSOR_DIGEST: &str =
    "b97f100ae568991cb33e3a671636152070ce9a88bbd0893dd338396609c1aac9";

/// The identity the target computed for the second parity's
/// accepted successor.
///
/// The run that COMPLETED the pair. The first parity's acceptance is
/// [`ACCEPTED_TXID`]; both are complete accepted successors, and it
/// takes both to say that both parities were exercised.
pub const PARITY_ACCEPTED_TXID: &str =
    "2e93c863726f5e8e3d4d5e3e3ceeccfbaa367179c7ca303dd2346de2d2b85cc8";

crate::recorded_acceptance::mint_recorded_acceptance!(parity_accepted, PARITY_ACCEPTED_TXID);

/// The commitment prefix the second run's consumed coin carried.
pub const PARITY_CONSUMED_COMMITMENT_PREFIX: u8 = 0x09;

/// Materialize the immutable historical V1 ceremony as typed data.
///
/// # Errors
///
/// [`TransactionIdentityParseError`] if either committed historical
/// identity is not exactly 64 ASCII hexadecimal digits.
pub fn historical_private_restart_run()
-> Result<HistoricalPrivateRestartRun, TransactionIdentityParseError> {
    Ok(HistoricalPrivateRestartRun::V1(
        HistoricalPrivateRestartV1 {
            issued_asset: ISSUED_ASSET,
            predecessor_digest: PREDECESSOR_DIGEST,
            acceptances: HistoricalPrivateRestartTwoAcceptanceLink {
                primary: HistoricalPrivateRestartAcceptedMember {
                    successor_digest: SUCCESSOR_DIGEST,
                    acceptance: accepted()?,
                    commitment_prefix: CONSUMED_COMMITMENT_PREFIX,
                },
                balancing: HistoricalPrivateRestartAcceptedMember {
                    successor_digest: PARITY_SUCCESSOR_DIGEST,
                    acceptance: parity_accepted()?,
                    commitment_prefix: PARITY_CONSUMED_COMMITMENT_PREFIX,
                },
            },
            submitted_bytes: SUBMITTED_BYTES,
            output_witness_proof_bytes: OUTPUT_WITNESS_PROOF_BYTES,
            receipt_leaves: RECEIPT_LEAVES,
            wall_seconds: WALL_SECONDS,
        },
    ))
}
