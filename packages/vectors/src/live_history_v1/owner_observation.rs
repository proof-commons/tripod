//! Historical selected-owner observation and its seven expected outcomes.

use crate::live_owner_observation::OwnerObservationCase;
use target_elements_conformance::protocol::ObservedOutcomeLayer;

/// Whether the recorded case was refused or accepted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpectedAcceptance {
    /// The target refused the submitted case and returned no accepted
    /// transaction identity.
    Refused,
    /// The target accepted the case at one identity and the subsequent
    /// reverification named the target's readback by identity.
    Accepted {
        /// The identity returned by the submission.
        identity: &'static str,
        /// The identity carried by the reverification record.
        reverification_identity: &'static str,
    },
}

/// One case's typed expected outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExpectedCaseOutcome {
    /// The in-tree evidence records the complete expected result.
    Recorded {
        /// The typed ceremony case.
        case: OwnerObservationCase,
        /// The case name written into the transcript.
        name: &'static str,
        /// The target boundary the recorded answer reached.
        layer: ObservedOutcomeLayer,
        /// Whether the answer carried an accepted identity.
        acceptance: ExpectedAcceptance,
    },
    /// The in-tree evidence does not recover an outcome for this case.
    ///
    /// A future batched rerun may fill such a member forward after owner
    /// review; callers must never infer a value for it.
    NotRecorded {
        /// The typed ceremony case whose result is absent.
        case: OwnerObservationCase,
        /// The case name whose result is absent.
        name: &'static str,
    },
}

/// The identity the target computed for the selected-profile acceptance.
pub const SELECTED_PROFILE_ACCEPTED_TXID: &str =
    "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029";

crate::recorded_acceptance::mint_recorded_acceptance!(
    selected_profile_accepted,
    SELECTED_PROFILE_ACCEPTED_TXID
);

/// The accepted identity carried by the target readback that was
/// reverified independently.
pub const SELECTED_PROFILE_REVERIFICATION_IDENTITY: &str = SELECTED_PROFILE_ACCEPTED_TXID;

/// The empty-output-witness-vector control's recorded refusal.
pub const EMPTY_OUTPUT_WITNESS_VECTOR: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
    case: OwnerObservationCase::EmptyOutputWitnessVector,
    name: "control-empty-output-witness-vector",
    layer: ObservedOutcomeLayer::ScriptPathRejection,
    acceptance: ExpectedAcceptance::Refused,
};

/// The another-deployment control's recorded refusal.
pub const ANOTHER_DEPLOYMENT: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
    case: OwnerObservationCase::AnotherDeployment,
    name: "control-another-deployment",
    layer: ObservedOutcomeLayer::ScriptPathRejection,
    acceptance: ExpectedAcceptance::Refused,
};

/// The non-selected-type-byte control's recorded refusal.
pub const NON_SELECTED_TYPE_BYTE: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
    case: OwnerObservationCase::NonSelectedTypeByte,
    name: "control-non-selected-type-byte",
    layer: ObservedOutcomeLayer::ScriptPathRejection,
    acceptance: ExpectedAcceptance::Refused,
};

/// The another-owner's-key control's recorded refusal.
pub const ANOTHER_OWNERS_KEY: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
    case: OwnerObservationCase::AnotherOwnersKey,
    name: "control-another-owners-key",
    layer: ObservedOutcomeLayer::ScriptPathRejection,
    acceptance: ExpectedAcceptance::Refused,
};

/// The another-candidate control's recorded refusal.
pub const ANOTHER_CANDIDATE: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
    case: OwnerObservationCase::AnotherCandidate,
    name: "control-another-candidate",
    layer: ObservedOutcomeLayer::ScriptPathRejection,
    acceptance: ExpectedAcceptance::Refused,
};

/// The printed-order deployment-seed control's recorded refusal.
pub const DEPLOYMENT_SEED_IN_PRINTED_ORDER: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
    case: OwnerObservationCase::DeploymentSeedInPrintedOrder,
    name: "control-deployment-seed-in-printed-order",
    layer: ObservedOutcomeLayer::ScriptPathRejection,
    acceptance: ExpectedAcceptance::Refused,
};

/// The selected profile's recorded acceptance and reverification.
pub const SELECTED_PROFILE: ExpectedCaseOutcome = ExpectedCaseOutcome::Recorded {
    case: OwnerObservationCase::SelectedProfile,
    name: "selected-profile-authorization",
    layer: ObservedOutcomeLayer::Accepted,
    acceptance: ExpectedAcceptance::Accepted {
        identity: SELECTED_PROFILE_ACCEPTED_TXID,
        reverification_identity: SELECTED_PROFILE_REVERIFICATION_IDENTITY,
    },
};

/// Every T5-026 case outcome, in the ceremony's submission order.
pub const EXPECTED_CASE_OUTCOMES: [ExpectedCaseOutcome; 7] = [
    EMPTY_OUTPUT_WITNESS_VECTOR,
    ANOTHER_DEPLOYMENT,
    NON_SELECTED_TYPE_BYTE,
    ANOTHER_OWNERS_KEY,
    ANOTHER_CANDIDATE,
    DEPLOYMENT_SEED_IN_PRINTED_ORDER,
    SELECTED_PROFILE,
];
