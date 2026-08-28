//! Historical schema-1 proof-bearing records and observation identity.

use std::collections::BTreeMap;

use target_elements::ObservationIdentity;
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use transaction::bytes::{AssetField, AssetId, ValueField};
use transaction::taproot::Digest32;

use crate::live_owner_observation::decode_hex;
use crate::live_proof_bearing_observation::{
    ProofBearingCase, ProofBearingConstructionControl, ProofBearingObservation,
    ProofBearingObservationRecord, ProofBearingReverification, RecordedConfidentialCoin,
    RecordedConstructionRefusals, RecordedMaterializationRefusal,
    RecordedProofBearingConstructionRefusal, RunOfRecordProjectionRefusal,
};
use crate::live_report::FixtureDigestAlgorithm;

/// The schema version of [`ProofBearingRunOfRecord`].
///
/// The recorded ceremony-generation V2 run uses this first archival
/// schema, whose fixture digest is historical v1. Ceremony generation,
/// archive schema and digest algorithm are three separate dimensions.
pub const PROOF_BEARING_RUN_OF_RECORD_SCHEMA_VERSION: u32 = 1;

/// A complete, versioned and outpoint-free archival projection of one
/// proof-bearing ceremony.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofBearingRunOfRecord {
    schema_version: u32,
    issued_asset: String,
    predecessor_digest: Digest32,
    coins: Vec<RecordedConfidentialCoin>,
    output_witness_vector_length: usize,
    output_witness_proof_bytes: Vec<usize>,
    spent_value_prefixes: Vec<u8>,
    observations: Vec<ProofBearingObservation>,
    construction_refusals: RecordedConstructionRefusals,
    reverification: ProofBearingReverification,
    candidate_messages: BTreeMap<ProofBearingCase, Digest32>,
}

impl ProofBearingRunOfRecord {
    /// The archival schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// The digest algorithm of every digest fact in schema 1.
    ///
    /// The marker is structural rather than a new stored field, so the
    /// archived schema-1 value remains byte-identical.
    #[must_use]
    pub const fn fixture_digest_algorithm(&self) -> FixtureDigestAlgorithm {
        FixtureDigestAlgorithm::HistoricalV1
    }

    /// The asset identity the target chose.
    #[must_use]
    pub fn issued_asset(&self) -> &str {
        &self.issued_asset
    }

    /// The predecessor fixture digest.
    #[must_use]
    pub const fn predecessor_digest(&self) -> &Digest32 {
        &self.predecessor_digest
    }

    /// The outpoint-free node-reported predecessor coins.
    #[must_use]
    pub fn coins(&self) -> &[RecordedConfidentialCoin] {
        &self.coins
    }

    /// The output-witness vector length.
    #[must_use]
    pub const fn output_witness_vector_length(&self) -> usize {
        self.output_witness_vector_length
    }

    /// The range-proof bytes in each output-witness entry.
    #[must_use]
    pub fn output_witness_proof_bytes(&self) -> &[usize] {
        &self.output_witness_proof_bytes
    }

    /// The spent value prefixes in input order.
    #[must_use]
    pub fn spent_value_prefixes(&self) -> &[u8] {
        &self.spent_value_prefixes
    }

    /// Every submitted-case observation in ceremony order.
    #[must_use]
    pub fn observations(&self) -> &[ProofBearingObservation] {
        &self.observations
    }

    /// The explicit construction-refusal capture state.
    #[must_use]
    pub const fn construction_refusals(&self) -> &RecordedConstructionRefusals {
        &self.construction_refusals
    }

    /// The exact second-origin reverification outcome.
    #[must_use]
    pub const fn reverification(&self) -> &ProofBearingReverification {
        &self.reverification
    }

    /// Every candidate message in case order.
    #[must_use]
    pub const fn candidate_messages(&self) -> &BTreeMap<ProofBearingCase, Digest32> {
        &self.candidate_messages
    }
}

impl TryFrom<&ProofBearingObservationRecord> for ProofBearingRunOfRecord {
    type Error = RunOfRecordProjectionRefusal;

    fn try_from(record: &ProofBearingObservationRecord) -> Result<Self, Self::Error> {
        if record.fixture_digest_algorithm() != FixtureDigestAlgorithm::HistoricalV1 {
            return Err(RunOfRecordProjectionRefusal::HistoricalV1DigestRequired);
        }
        Err(RunOfRecordProjectionRefusal::HistoricalV1ProjectionRetired)
    }
}

/// T5-031 did not capture either construction refusal in its committed
/// conversion.
///
/// This is an evidence-binding erratum, not a replacement record. The
/// historical DONE standing remains, and the omitted refusals are not
/// reconstructed from current code.
pub const T5_031_CONSTRUCTION_REFUSALS: RecordedConstructionRefusals =
    RecordedConstructionRefusals::NotCaptured;

/// Whether the ceremony-generation V2 historical-v1 archive is present.
///
/// This archival name is retained because it was minted with the run.
/// Its `V2` means ceremony generation, not forward-v2 digest semantics;
/// fresh native selection uses
/// [`crate::live_proof_bearing_observation::forward_v2_proof_bearing_run_of_record`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofBearingRunOfRecordV2 {
    /// The authorized native rerun has not minted constants yet.
    Pending,
    /// The exact record minted by the authorized native rerun.
    Recorded(&'static ProofBearingRunOfRecord),
}

impl ProofBearingRunOfRecordV2 {
    /// The report spelling of this evidence state.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Recorded(_) => "recorded",
        }
    }
}

/// Exact constants from the serialized ceremony-generation V2 native
/// run of record, carrying historical-v1 fixture digest semantics.
///
/// Minted from the 2026-08-27 serialized native lane at tree `bf211dd9`
/// against elements tip `b7fc5d080a`. The evidence source is
/// `rerun-bf211dd.proof-bearing-observation`; its enclosing RUN-REPORT
/// records 40 of 40 green at exit zero and 420.7 seconds wall time.
mod historical_v1_run_of_record {
    use std::sync::OnceLock;

    use super::{
        AssetField, AssetId, BTreeMap, Digest32, ObservedOutcomeLayer,
        PROOF_BEARING_RUN_OF_RECORD_SCHEMA_VERSION, ProofBearingCase,
        ProofBearingConstructionControl, ProofBearingObservation, ProofBearingReverification,
        ProofBearingRunOfRecord, RecordedConfidentialCoin, RecordedConstructionRefusals,
        RecordedMaterializationRefusal, RecordedProofBearingConstructionRefusal, ValueField,
        decode_hex,
    };

    const ISSUED_ASSET: &str = "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";
    const PREDECESSOR_DIGEST: &str =
        "00da5ef7aaef159237ef5479b419abeee6307e913cc4e244f3226c64c5489262";
    const COIN_ASSET_INTERNAL: &str =
        "fb93ca056d92e29c27986a508f4bd3566f64ea04543f65aa51825fd8d4c84fd7";
    const COIN_VALUES: [&str; 2] = [
        "0828d616da18038066f8af4af94a5c6afa9195ece494b9520fc7c81916dc68f000",
        "096543b29336752436d03b1bfc6e6cbc46b1667e203856c58ba0ec67f0d6a37071",
    ];
    const COIN_PROGRAMS: [&str; 2] = [
        "5120508f7d2b9339123105ec650f9e93ae232b9e21b1d8781295d7a34a80acf8a235",
        "512065078b646dd98a4bb31f69ae7264fe713a7b167605dfca99f451ce8639719136",
    ];
    const RANGEPROOF_BYTES: usize = 4174;
    const SPENT_VALUE_PREFIXES: [u8; 2] = [0x08, 0x09];
    const ACCEPTED_TXID: &str = "89dbf3dbaa733c073eef67aa459b9731fa7bd119062a6b1d13a1ec5357dcd7d2";
    const WITNESS_TXID: &str = "568a86a83a0ba787a77d13e2d4a694c82655b958bcc1a21bc1568d08ca31c368";
    const RECOMPUTED_MESSAGE: &str =
        "8f9c33563a24bab025514edd9c17d8e49902adff87f7b4688a194f976d10c322";
    const SIGNATURE: &str = "582cc46a31e0111a7894a702d976741fb16711500bdcd719dcc16e211e89e39c\
ebcdbb8675319b48b2e04fae3ca4fbcaa4030aa618f8d2b6084a6f45ba9b7408";
    const REFUSAL_DETAIL: &str = "mandatory-script-verify-flag-failed (Invalid Schnorr signature)";
    const SUBMITTED_BYTES: usize = 8993;
    const MESSAGES: [(ProofBearingCase, &str); 4] = [
        (
            ProofBearingCase::ProofBearingVectorEmptied,
            "581e05bef763a117bb6c904acaa6b2244b6c01137f5a80595904bd6f0e596e08",
        ),
        (
            ProofBearingCase::PreimageOnlySigner,
            "5f2048390051d9c3d11ddccdde9ceeab1f69b7b17e17b2cf7e631387ad39a218",
        ),
        (
            ProofBearingCase::AnotherProofBearingCandidate,
            "694198f69417504ab5cff57dbe3b48dfb0691455da61c60b6333ec68aef560bb",
        ),
        (ProofBearingCase::SelectedProfile, RECOMPUTED_MESSAGE),
    ];

    fn fixed<const N: usize>(text: &str) -> [u8; N] {
        decode_hex(text)
            .and_then(|bytes| bytes.try_into().ok())
            .expect("the minted transcript literal has the required byte length")
    }

    fn coins() -> Vec<RecordedConfidentialCoin> {
        COIN_VALUES
            .iter()
            .zip(COIN_PROGRAMS)
            .map(|(value, program)| RecordedConfidentialCoin {
                asset: AssetField::Explicit(AssetId::from_internal(fixed(COIN_ASSET_INTERNAL))),
                value: ValueField::Commitment(fixed(value)),
                program: decode_hex(program).expect("the minted coin program is hexadecimal"),
                rangeproof_bytes: RANGEPROOF_BYTES,
                matches_expectation: true,
            })
            .collect()
    }

    fn observations() -> Vec<ProofBearingObservation> {
        ProofBearingCase::ALL
            .iter()
            .copied()
            .map(|case| ProofBearingObservation {
                case,
                layer: if matches!(case, ProofBearingCase::SelectedProfile) {
                    ObservedOutcomeLayer::Accepted
                } else {
                    ObservedOutcomeLayer::ScriptPathRejection
                },
                detail: case
                    .is_negative_control()
                    .then(|| REFUSAL_DETAIL.to_owned()),
                accepted_txid: matches!(case, ProofBearingCase::SelectedProfile)
                    .then(|| ACCEPTED_TXID.to_owned()),
                submitted_bytes: SUBMITTED_BYTES,
            })
            .collect()
    }

    fn construction_refusals() -> RecordedConstructionRefusals {
        RecordedConstructionRefusals::Captured(
            ProofBearingConstructionControl::ALL
                .iter()
                .copied()
                .map(|control| RecordedProofBearingConstructionRefusal {
                    control,
                    refusal: RecordedMaterializationRefusal::PredecessorOpeningMismatch,
                })
                .collect(),
        )
    }

    fn reverification() -> ProofBearingReverification {
        ProofBearingReverification {
            accepted_txid: ACCEPTED_TXID.to_owned(),
            witness_txid: WITNESS_TXID.to_owned(),
            block_height: 6,
            readback_matches_submission: true,
            recomputed_message: fixed(RECOMPUTED_MESSAGE),
            signature_from_readback: decode_hex(SIGNATURE)
                .expect("the minted readback signature is hexadecimal"),
            verified: Ok(()),
            verifies_against_emptied_vector_message: false,
        }
    }

    fn candidate_messages() -> BTreeMap<ProofBearingCase, Digest32> {
        MESSAGES
            .iter()
            .map(|(case, message)| (*case, fixed(message)))
            .collect()
    }

    fn build() -> ProofBearingRunOfRecord {
        ProofBearingRunOfRecord {
            schema_version: PROOF_BEARING_RUN_OF_RECORD_SCHEMA_VERSION,
            issued_asset: ISSUED_ASSET.to_owned(),
            predecessor_digest: fixed(PREDECESSOR_DIGEST),
            coins: coins(),
            output_witness_vector_length: 2,
            output_witness_proof_bytes: vec![RANGEPROOF_BYTES, RANGEPROOF_BYTES],
            spent_value_prefixes: SPENT_VALUE_PREFIXES.to_vec(),
            observations: observations(),
            construction_refusals: construction_refusals(),
            reverification: reverification(),
            candidate_messages: candidate_messages(),
        }
    }

    pub(super) fn get() -> &'static ProofBearingRunOfRecord {
        static RUN: OnceLock<ProofBearingRunOfRecord> = OnceLock::new();
        RUN.get_or_init(build)
    }
}

/// The recorded ceremony-generation V2 historical-v1 run-of-record
/// state.
///
/// Minted only from the persisted transcript named above. Returning the
/// recorded variant flips both live equality gates from conditional to
/// enforcing while retaining the typed state used by the renderer.
///
/// # Panics
///
/// Panics only if a committed transcript literal is not valid hexadecimal
/// of its recorded byte length, which a caller cannot arrange.
#[must_use]
pub fn historical_v1_construction_run_of_record() -> ProofBearingRunOfRecordV2 {
    ProofBearingRunOfRecordV2::Recorded(historical_v1_run_of_record::get())
}

/// The issued asset the run of record was funded against.
pub const RECORDED_ASSET: &str = "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

/// The digest the predecessor fixture was registered under, on the run
/// of record.
pub const RECORDED_PREDECESSOR_DIGEST: &str =
    "00da5ef7aaef159237ef5479b419abeee6307e913cc4e244f3226c64c5489262";

/// How many range-proof bytes each output-witness entry carried.
///
/// The single figure that separates this lane from the explicit one,
/// where every entry is an empty surjection proof and an empty range
/// proof — two bytes in total. Four thousand one hundred and
/// seventy-four bytes of it are in the message the owner signed, and
/// none of them is recoverable from the witnessless serialization at any
/// length.
pub const RECORDED_RANGEPROOF_BYTES: usize = 4174;

/// The serialized prefixes the two spent value commitments carried.
///
/// The two the target admits, one square and one non-square, which is
/// what the other guide's bounded parity search settles on. They are
/// written down here because the accepted message hashed them; what they
/// establish about the target's reading of a confidential value field is
/// that guide's question and not this one's.
pub const RECORDED_SPENT_VALUE_PREFIXES: [u8; 2] = [0x08, 0x09];

/// The identity the target computed over the bytes it accepted.
pub const RECORDED_ACCEPTED_TXID: &str =
    "89dbf3dbaa733c073eef67aa459b9731fa7bd119062a6b1d13a1ec5357dcd7d2";

/// The witness identity the target reported for it.
pub const RECORDED_WITNESS_TXID: &str =
    "568a86a83a0ba787a77d13e2d4a694c82655b958bcc1a21bc1568d08ca31c368";

/// The height the target confirmed it at.
pub const RECORDED_BLOCK_HEIGHT: u32 = 6;

/// The message the accepted authorization was taken over.
pub const RECORDED_ACCEPTED_MESSAGE: &str =
    "8f9c33563a24bab025514edd9c17d8e49902adff87f7b4688a194f976d10c322";

/// The signature as it stood in the target's own copy.
pub const RECORDED_SIGNATURE: &str = "582cc46a31e0111a7894a702d976741fb16711500bdcd719dcc16e211e89e39c\
ebcdbb8675319b48b2e04fae3ca4fbcaa4030aa618f8d2b6084a6f45ba9b7408";

/// The node's own words for every refused control.
///
/// All three drew the same sentence, which is the result rather than a
/// simplification: each control moved a different term of the message
/// and the target's answer to a message it did not form is one answer.
pub const RECORDED_REFUSAL_DETAIL: &str =
    "mandatory-script-verify-flag-failed (Invalid Schnorr signature)";

/// The messages each submitted case's signatures were taken over.
///
/// Four distinct digests, in case order. They are committed because
/// their DISTINCTNESS is what gives the three refusals content: a run in
/// which two of them had coincided would have offered one control twice
/// and reported it as two.
pub const RECORDED_MESSAGES: [(ProofBearingCase, &str); 4] = [
    (
        ProofBearingCase::ProofBearingVectorEmptied,
        "581e05bef763a117bb6c904acaa6b2244b6c01137f5a80595904bd6f0e596e08",
    ),
    (
        ProofBearingCase::PreimageOnlySigner,
        "5f2048390051d9c3d11ddccdde9ceeab1f69b7b17e17b2cf7e631387ad39a218",
    ),
    (
        ProofBearingCase::AnotherProofBearingCandidate,
        "694198f69417504ab5cff57dbe3b48dfb0691455da61c60b6333ec68aef560bb",
    ),
    (ProofBearingCase::SelectedProfile, RECORDED_ACCEPTED_MESSAGE),
];

/// The identity of the run of record, in the shape the reviewed
/// contract already names an observation by.
///
/// The explicit lane's run is named this way where the six established
/// dimensions cite it, and the Wave-4 audit found the gap on this side:
/// a ceremony whose evidence was a file at a path an operator chose
/// named nothing a later reader could cite. So this run carries the same
/// three members — the ceremony's own name for the case, the identity
/// the TARGET computed, and where the run is recorded.
///
/// It is deliberately NOT added to the reviewed contract's dimension
/// table. Those six dimensions are established, they were established on
/// the explicit run, and a second identity beside them would read as a
/// second establishment of things this run did not re-establish.
pub const PROOF_BEARING_OBSERVATION: ObservationIdentity = ObservationIdentity::new(
    "selected-profile-proof-bearing-authorization",
    RECORDED_ACCEPTED_TXID,
    "plans/backlog.md T5-031",
);

/// How many bytes each case handed the node, on the run of record.
///
/// The same figure for every case, which is the point: the SUBMITTED
/// candidate is one candidate and only the message its signatures were
/// taken over varies, so a refusal is attributable to the term that
/// moved rather than to a different transaction.
pub const RECORDED_SUBMITTED_BYTES: usize = 8993;
