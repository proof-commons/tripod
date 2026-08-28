//! Historical native transcript and resource observations.

use std::collections::{BTreeMap, BTreeSet};

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements_conformance::protocol::ObservedOutcomeLayer;

use crate::live_native::{
    LiveFormNotSubmitted, LiveNativeStep, LiveNativeTranscript, PredictedTransferResources,
    recorded,
};

/// How many bytes the explicit transfer of record serialized to.
///
/// From the run [`transcript`] describes: the exact length of the byte
/// string handed to the node, recorded so the weight beside it can be
/// read as a weight *of something* rather than as a bare figure.
pub const RECORDED_EXPLICIT_SERIALIZED_BYTES: u64 = 1_164;

/// The weight this workspace computed for those exact bytes.
///
/// §18.4's prediction half, taken by decoding the submitted
/// serialization and weighing the result.
pub const RECORDED_EXPLICIT_PREDICTED_WEIGHT: u64 = 1_911;

/// The weight the node computed for those exact bytes.
///
/// §18.4's observation half, and the figure this whole comparison
/// rests on. It exists because the executor reads a weight back from
/// the node's own `decoderawtransaction` even for a transaction the
/// node refused — which is the only reason a candidate that cannot be
/// accepted (§1.7) has any target resource figure at all.
///
/// It is a *separate constant* from the prediction above, and equal to
/// it only because the run made it so. Spelling one constant and using
/// it twice would have made the agreement true by construction, which
/// is the one thing §18.4's comparison must never be.
pub const RECORDED_EXPLICIT_OBSERVED_WEIGHT: u64 = 1_911;

/// The run of record: what one real node did, on one host, once.
///
/// # This is a transcription, and it is not evidence
///
/// Nothing here discharges a §15 row, and nothing here is recomputed from
/// anything. It is a *record*, committed so that a reader without a node
/// can see what the target actually said — and so that the blocker
/// [`crate::live_evidence::LiveInfrastructureBlocker::OwnerSighashNotComputable`]
/// is readable as a thing that was observed rather than a thing that was
/// argued for.
///
/// The observed verdict is the sharpest single fact this wave produced.
/// The explicit transfer was refused at
/// [`ObservedOutcomeLayer::ScriptPathRejection`] with the node's own
/// words: `mandatory-script-verify-flag-failed (Invalid Schnorr
/// signature)`. That means the covenant *ran*. Over a real transaction
/// spending real coins at the linked constructors' own programs, the
/// coordinator's asset check, constructor recognition, class closure,
/// count and conservation arithmetic all passed, and evaluation reached
/// the owner's signature check before anything failed. What stands
/// between this candidate and an accepted transfer is one digest.
///
/// # What a reader may not do with it
///
/// Read it as a verdict about a *relation*. §19.2 requires a negative
/// case to have been refused for its own intended relation, and this
/// refusal is about the signature offered — which is opaque bytes by
/// construction, because no first-party component computes the message
/// they would have to be over.
#[must_use]
pub fn transcript() -> LiveNativeTranscript {
    LiveNativeTranscript {
        issued_asset: Some(
            "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb".to_owned(),
        ),
        relinked: true,
        // The outpoints the run funded are deliberately absent. They name
        // a chain that was destroyed when the run ended, and a record
        // carrying them would look like something a later run could
        // resume from.
        explicit_program: Vec::new(),
        private_program: Vec::new(),
        explicit_coins: Vec::new(),
        private_coins: Vec::new(),
        not_submitted: BTreeSet::from([(
            LiveTransferRepresentationPlan::PrivateCommitted,
            LiveFormNotSubmitted::NoConfidentialPredecessorCanBeFunded,
        )]),
        // §18.4's first-party half, from the run that produced the
        // observation below. Both figures are the same run's: a
        // prediction transcribed from some other run would be a
        // prediction about other bytes, which is the substitution §18.4's
        // "same exact bytes" is there to refuse.
        predicted: BTreeMap::from([(
            LiveTransferRepresentationPlan::Explicit,
            PredictedTransferResources {
                serialized_bytes: RECORDED_EXPLICIT_SERIALIZED_BYTES,
                weight: Some(RECORDED_EXPLICIT_PREDICTED_WEIGHT),
            },
        )]),
        observations: vec![
            recorded(
                LiveNativeStep::IssueProtocolAsset,
                ObservedOutcomeLayer::Accepted,
                None,
                2,
                None,
            ),
            recorded(
                LiveNativeStep::FundExplicitConstructor,
                ObservedOutcomeLayer::Accepted,
                None,
                2,
                None,
            ),
            recorded(
                LiveNativeStep::FundPrivateConstructor,
                ObservedOutcomeLayer::Accepted,
                None,
                2,
                None,
            ),
            recorded(
                LiveNativeStep::SubmitExplicitTransfer,
                ObservedOutcomeLayer::ScriptPathRejection,
                Some("mandatory-script-verify-flag-failed (Invalid Schnorr signature)"),
                0,
                Some(RECORDED_EXPLICIT_OBSERVED_WEIGHT),
            ),
        ],
        refusal: None,
    }
}
