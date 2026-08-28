//! Historical conservation control and refusal observations.

use target_elements_conformance::protocol::ObservedOutcomeLayer;

/// The disposable asset the run issued.
pub const ISSUED_ASSET: &str = "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

/// The predecessor fixture's digest.
pub const PREDECESSOR_DIGEST: &str =
    "ca43b210d6e74b76f7b3d3f79a6123556f150a7af9fa78e2571e2812b9d51fc2";

/// The successor fixture's digest.
pub const SUCCESSOR_DIGEST: &str =
    "31501b776502ee48d48b115d8bc80f55ba01bfc8cb6e163e3f2882848d025aa0";

/// The identity the target computed for the accepted balance-valid
/// control.
///
/// The conserving half of step three's conservation record and the
/// control step four's four mutants are derived from. Byte-identical
/// to the one-to-one control's own run of record, because the ceremony
/// is deterministic: two independent runs on two fresh chains produced
/// this same identity.
pub const CONTROL_ACCEPTED_TXID: &str =
    "1af38f8a5292afcdb4dd38f78a146ff84d36db20d9916e768b7fd9b368b89e8e";

crate::recorded_acceptance::mint_recorded_acceptance!(control_accepted, CONTROL_ACCEPTED_TXID);

/// How many bytes the accepted control submitted.
pub const CONTROL_SUBMITTED_BYTES: usize = 9_136;

/// The commitment prefix the consumed coin carried, the first of the
/// two admitted parities.
pub const CONSUMED_COMMITMENT_PREFIX: u8 = 0x08;

/// The half-open byte range the wrong-blinder mutant declared and
/// stayed within: the mutated output's 33-byte value-commitment field.
pub const WRONG_BLINDER_FIELD_RANGE: (usize, usize) = (81, 114);

/// The half-open byte range the private-ct-imbalance mutant declared
/// and stayed within: the SECOND output's 33-byte value-commitment field.
///
/// Distinct from [`WRONG_BLINDER_FIELD_RANGE`] because the two mutants
/// sit at different outputs, which is what separates `private-ct-imbalance`
/// from `wrong-private-blinding-balance` when the target draws the same
/// `bad-txns-in-ne-out` for both.
pub const PRIVATE_CT_IMBALANCE_FIELD_RANGE: (usize, usize) = (215, 248);

/// The half-open byte range the two range-proof mutants declared: the
/// mutated output-witness entry's range-proof region, length prefix
/// included so the emptying and the corruption both fall inside it.
pub const RANGEPROOF_FIELD_RANGE: (usize, usize) = (781, 4_958);

/// The one identical refusal the target gave every mutant, at the
/// [`ObservedOutcomeLayer::ConsensusRejectionBeforeScript`] layer.
///
/// The wrong-blinder, range-proof and private-ct-imbalance mutants all
/// draw it; the FIELD each declared is what tells their rows apart, the
/// words being the same.
pub const MUTANT_REJECT_DETAIL: &str = "bad-txns-in-ne-out";

/// The layer the target refused every mutant at, TYPED.
///
/// The doc above already named it and only a reader could act on
/// that. This is the same fact in the vocabulary, so the classifier
/// can compare the layer a run REACHED against the boundary a row
/// DECLARED instead of assuming the two agree.
pub const MUTANT_OBSERVED_LAYER: ObservedOutcomeLayer =
    ObservedOutcomeLayer::ConsensusRejectionBeforeScript;

/// The run's wall time, in seconds.
pub const WALL_SECONDS: f64 = 12.7;
