//! The live-transfer lane: one candidate run against a real node.
//!
//! # Why these gates are ignored in the ordinary suite
//!
//! It needs a live Elements node, so it cannot run in an ordinary lane,
//! and the whole workspace suite stays green without a node present.
//! The `#[ignore]` separates this serialized native lane from ordinary
//! tests; it does not make every native result observation-only. Where a
//! committed run of record supplies active evidence, its own carrier may
//! be a strict reproduction gate after it writes the fresh artifacts.
//!
//! Run it as:
//!
//! ```text
//! TRIPOD_LIVE_EXECUTOR=<adapter> \
//! TRIPOD_LIVE_NETWORK_ID=<64 hex> \
//! TRIPOD_LIVE_GENESIS_ID=<64 hex> \
//! TRIPOD_LIVE_REPORT=<path> \
//!   cargo test -p tripod-vectors --test guide13_live_native -- --ignored --nocapture
//! ```
//!
//! # Recorded verdicts fail closed at their own gates
//!
//! Most assertions are about the *shape* of a run that completed. The
//! private-restart carrier additionally binds the fresh in-memory record
//! to every stable field of its committed run of record. A changed honest
//! answer is written to the transcript first and then leaves that gate
//! red; superseding it requires an explicit decision recorded as a new
//! forward record, never an overwrite of history
//! `(´[PLAN-rule:guide12-exec:failure-layers]´)`.
//!
//! # And nothing here discharges a matrix row
//!
//! The submitted transfers carry a witness whose signature position holds
//! bytes that authorize nothing, because no first-party component
//! computes the digest the target's verifying primitive forms. Every
//! rejection is attributable to that rather than to the row a reader
//! might wish it were about, and `vectors::live_evidence` records the
//! whole matrix as blocked for exactly that reason.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    ReproducibilityContract, reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::executor::{
    DEFAULT_EXECUTOR_TIMEOUT, ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust,
    NativeOperationCapture, OperationStep, PlanRefused, TargetOperationPlanner,
    execute_operations_captured,
};
use target_elements_conformance::protocol::{
    ConfidentialFixtureDigest, ConfidentialFixtureHandle, ConfidentialFundingBinding,
    ConfidentialFundingDestination, ConfidentialFundingProfiles, ExecutorCapability,
    FundingCustodyProfile, FundingMaterializerProfile, FundingRepresentationProfile,
    NativeOperationResponse, NativeVerdict, ObservedOutcomeLayer, OperationCaseId,
    OperationSubject, TargetConfidentialFundingSubject, TargetConfidentialSponsorFundingSubject,
    TargetFundingSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    TargetSubmissionSubject, WireEnvironment, WireExecutionDomain, WireOutpoint,
    WireSighashProfile,
};
use vectors::live_native::{LiveNativeStep, LiveTransferOperationPlanner, render_live_native_run};
use vectors::live_report::{LiveMutantKind, LiveMutationLocator, LiveWitnessPathRole};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum CeremonyId {
    ConservationNegatives,
    ExplicitBoundaryValues,
    ExplicitMaximumInputs,
    ExplicitMaximumOutputs,
    ExplicitMerge,
    ExplicitNormalization,
    ExplicitOneDestinationOwner,
    ExplicitOneToOne,
    ExplicitRepeatedOwner,
    ExplicitSelfPaidFee,
    ExplicitSeveralDestinationOwners,
    ExplicitSeveralOwners,
    ExplicitSeveralToSeveral,
    ExplicitSplit,
    ExplicitSponsorless,
    ExplicitWitnessNegatives,
    KeypathProbe,
    MultiEntryCrossing,
    MultiExitCrossing,
    MultiManyToMany,
    MultiOneToOneWithFee,
    MultiPrivateMerge,
    MultiPureSplit,
    MultiSeveralOwners,
    MultiSplit,
    MultiStrictOneToOne,
    OffsettingFlowNegatives,
    OwnerObservation,
    OwnerSigningNegatives,
    PairsArc,
    PrivateRestartControl,
    PrivateRestartParity,
    ProofBearingObservation,
    Report,
    SponsoredChangeAbsent,
    SponsoredChangePresent,
    SponsoredCommittedValue,
    SponsoredMissingAuthorization,
    SponsoredPrivateExplicitNoChange,
    SponsoredPrivateWithChange,
    ConfidentialPredecessorSetup,
}

impl CeremonyId {
    const ALL: [Self; 41] = [
        Self::ConservationNegatives,
        Self::ExplicitBoundaryValues,
        Self::ExplicitMaximumInputs,
        Self::ExplicitMaximumOutputs,
        Self::ExplicitMerge,
        Self::ExplicitNormalization,
        Self::ExplicitOneDestinationOwner,
        Self::ExplicitOneToOne,
        Self::ExplicitRepeatedOwner,
        Self::ExplicitSelfPaidFee,
        Self::ExplicitSeveralDestinationOwners,
        Self::ExplicitSeveralOwners,
        Self::ExplicitSeveralToSeveral,
        Self::ExplicitSplit,
        Self::ExplicitSponsorless,
        Self::ExplicitWitnessNegatives,
        Self::KeypathProbe,
        Self::MultiEntryCrossing,
        Self::MultiExitCrossing,
        Self::MultiManyToMany,
        Self::MultiOneToOneWithFee,
        Self::MultiPrivateMerge,
        Self::MultiPureSplit,
        Self::MultiSeveralOwners,
        Self::MultiSplit,
        Self::MultiStrictOneToOne,
        Self::OffsettingFlowNegatives,
        Self::OwnerObservation,
        Self::OwnerSigningNegatives,
        Self::PairsArc,
        Self::PrivateRestartControl,
        Self::PrivateRestartParity,
        Self::ProofBearingObservation,
        Self::Report,
        Self::SponsoredChangeAbsent,
        Self::SponsoredChangePresent,
        Self::SponsoredCommittedValue,
        Self::SponsoredMissingAuthorization,
        Self::SponsoredPrivateExplicitNoChange,
        Self::SponsoredPrivateWithChange,
        Self::ConfidentialPredecessorSetup,
    ];

    const fn as_str(self) -> &'static str {
        match self {
            Self::ConservationNegatives => "conservation-negatives",
            Self::ExplicitBoundaryValues => "explicit-boundary-values",
            Self::ExplicitMaximumInputs => "explicit-maximum-inputs",
            Self::ExplicitMaximumOutputs => "explicit-maximum-outputs",
            Self::ExplicitMerge => "explicit-merge",
            Self::ExplicitNormalization => "explicit-normalization",
            Self::ExplicitOneDestinationOwner => "explicit-one-destination-owner",
            Self::ExplicitOneToOne => "explicit-one-to-one",
            Self::ExplicitRepeatedOwner => "explicit-repeated-owner",
            Self::ExplicitSelfPaidFee => "explicit-self-paid-fee",
            Self::ExplicitSeveralDestinationOwners => "explicit-several-destination-owners",
            Self::ExplicitSeveralOwners => "explicit-several-owners",
            Self::ExplicitSeveralToSeveral => "explicit-several-to-several",
            Self::ExplicitSplit => "explicit-split",
            Self::ExplicitSponsorless => "explicit-sponsorless",
            Self::ExplicitWitnessNegatives => "explicit-witness-negatives",
            Self::KeypathProbe => "keypath-probe",
            Self::MultiEntryCrossing => "multi-entry-crossing",
            Self::MultiExitCrossing => "multi-exit-crossing",
            Self::MultiManyToMany => "multi-many-to-many",
            Self::MultiOneToOneWithFee => "multi-one-to-one-with-fee",
            Self::MultiPrivateMerge => "multi-private-merge",
            Self::MultiPureSplit => "multi-pure-split",
            Self::MultiSeveralOwners => "multi-several-owners",
            Self::MultiSplit => "multi-split",
            Self::MultiStrictOneToOne => "multi-strict-one-to-one",
            Self::OffsettingFlowNegatives => "offsetting-flow-negatives",
            Self::OwnerObservation => "owner-observation",
            Self::OwnerSigningNegatives => "owner-signing-negatives",
            Self::PairsArc => "pairs-arc",
            Self::PrivateRestartControl => "private-restart-control",
            Self::PrivateRestartParity => "private-restart-parity",
            Self::ProofBearingObservation => "proof-bearing-observation",
            Self::Report => "report",
            Self::SponsoredChangeAbsent => "sponsored-change-absent",
            Self::SponsoredChangePresent => "sponsored-change-present",
            Self::SponsoredCommittedValue => "sponsored-committed-value",
            Self::SponsoredMissingAuthorization => "sponsored-missing-authorization",
            Self::SponsoredPrivateExplicitNoChange => "sponsored-private-explicit-no-change",
            Self::SponsoredPrivateWithChange => "sponsored-private-with-change",
            Self::ConfidentialPredecessorSetup => "confidential-predecessor",
        }
    }

    const fn rust_test_name(self) -> &'static str {
        match self {
            Self::ConservationNegatives => {
                "conservation_is_recorded_against_a_control_the_proof_negatives_mutate"
            }
            Self::ExplicitBoundaryValues => {
                "the_explicit_boundary_values_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitMaximumInputs => {
                "the_explicit_maximum_inputs_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitMaximumOutputs => {
                "the_explicit_maximum_outputs_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitMerge => "the_explicit_merge_shape_is_submitted_to_a_real_target",
            Self::ExplicitNormalization => {
                "the_explicit_normalization_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitOneDestinationOwner => {
                "the_explicit_one_destination_owner_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitOneToOne => "the_explicit_one_to_one_shape_is_submitted_to_a_real_target",
            Self::ExplicitRepeatedOwner => {
                "the_explicit_repeated_owner_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitSelfPaidFee => {
                "the_explicit_self_paid_fee_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitSeveralDestinationOwners => {
                "the_explicit_several_destination_owners_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitSeveralOwners => {
                "the_explicit_several_owners_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitSeveralToSeveral => {
                "the_explicit_several_to_several_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitSplit => "the_explicit_split_shape_is_submitted_to_a_real_target",
            Self::ExplicitSponsorless => {
                "the_explicit_sponsorless_shape_is_submitted_to_a_real_target"
            }
            Self::ExplicitWitnessNegatives => {
                "the_witness_content_negatives_are_offered_beside_their_control"
            }
            Self::KeypathProbe => "one_key_path_spend_attempt_is_offered_to_a_real_target",
            Self::MultiEntryCrossing => "the_entry_crossing_shape_is_submitted_to_a_real_target",
            Self::MultiExitCrossing => "the_exit_crossing_shape_is_submitted_to_a_real_target",
            Self::MultiManyToMany => "the_many_to_many_shape_is_submitted_to_a_real_target",
            Self::MultiOneToOneWithFee => {
                "the_fee_bearing_one_to_one_shape_is_submitted_to_a_real_target"
            }
            Self::MultiPrivateMerge => "the_private_merge_shape_is_submitted_to_a_real_target",
            Self::MultiPureSplit => "the_pure_split_shape_is_submitted_to_a_real_target",
            Self::MultiSeveralOwners => {
                "the_several_distinct_owners_shape_is_submitted_to_a_real_target"
            }
            Self::MultiSplit => "the_split_shape_is_submitted_to_a_real_target",
            Self::MultiStrictOneToOne => {
                "the_strict_one_to_one_shape_is_submitted_to_a_real_target"
            }
            Self::OffsettingFlowNegatives => {
                "one_offsetting_flow_is_refused_before_the_narrower_control_is_accepted"
            }
            Self::OwnerObservation => "one_owner_authorization_is_observed_on_the_explicit_lane",
            Self::OwnerSigningNegatives => {
                "one_bare_u_output_mutant_is_refused_before_the_control_is_accepted"
            }
            Self::PairsArc => "the_pairs_arc_submits_both_members_of_one_fixture_to_a_real_target",
            Self::PrivateRestartControl => {
                "one_private_one_to_one_control_is_submitted_to_a_real_target"
            }
            Self::PrivateRestartParity => {
                "the_other_commitment_parity_is_exercised_in_a_complete_successor"
            }
            Self::ProofBearingObservation => {
                "one_owner_authorization_is_observed_on_the_proof_bearing_lane"
            }
            Self::Report => "the_live_transfer_candidate_runs_against_a_real_target",
            Self::SponsoredChangeAbsent => {
                "the_sponsored_change_absent_shape_is_submitted_to_a_real_target"
            }
            Self::SponsoredChangePresent => {
                "the_sponsored_change_present_shape_is_submitted_to_a_real_target"
            }
            Self::SponsoredCommittedValue => {
                "the_committed_sponsor_value_shape_is_submitted_to_a_real_target"
            }
            Self::SponsoredMissingAuthorization => {
                "the_missing_sponsor_authorization_negative_is_refused_behind_its_control"
            }
            Self::SponsoredPrivateExplicitNoChange => {
                "the_sponsored_explicit_no_change_shape_is_submitted_to_a_real_target"
            }
            Self::SponsoredPrivateWithChange => {
                "the_sponsored_confidential_with_change_shape_is_submitted_to_a_real_target"
            }
            Self::ConfidentialPredecessorSetup => {
                "one_confidential_predecessor_is_funded_mined_and_read_back"
            }
        }
    }

    const fn is_setup(self) -> bool {
        matches!(self, Self::ConfidentialPredecessorSetup)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RequestRole {
    Acceptance,
    Control,
    Refusal,
    PairedExplicit,
    PairedPrivate,
    Auxiliary,
}

impl RequestRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Acceptance => "acceptance",
            Self::Control => "control",
            Self::Refusal => "refusal",
            Self::PairedExplicit => "paired-explicit",
            Self::PairedPrivate => "paired-private",
            Self::Auxiliary => "auxiliary",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CaptureDigest {
    name: &'static str,
    value: [u8; 32],
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ProjectionInput {
    owners: Vec<Vec<u8>>,
    amounts: Vec<u64>,
    destinations: BTreeMap<Vec<u8>, (Vec<u8>, u64)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CeremonyOperationFacts {
    operation_id: String,
    case_step: String,
    role: RequestRole,
    control_request_id: Option<String>,
    control_identity: Option<String>,
    mutant: Option<LiveMutantKind>,
    locator: Option<LiveMutationLocator>,
    projection: Option<ProjectionInput>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CeremonyCaptureFacts {
    digests: Vec<CaptureDigest>,
    operations: Vec<CeremonyOperationFacts>,
}

impl CeremonyCaptureFacts {
    fn from_capture(ceremony: CeremonyId, capture: &NativeOperationCapture) -> Self {
        let submission_indices: Vec<_> = capture
            .operations()
            .iter()
            .enumerate()
            .filter_map(|(index, operation)| {
                matches!(operation.request().subject, OperationSubject::Submission(_))
                    .then_some(index)
            })
            .collect();
        let control_index = control_submission(ceremony, capture, &submission_indices);
        let control = control_index.and_then(|index| capture.operations().get(index));
        let control_request_id = control.map(|operation| operation.request_id().to_owned());
        let control_identity = control
            .and_then(|operation| operation.target_identity())
            .map(str::to_owned);

        let operations = capture
            .operations()
            .iter()
            .enumerate()
            .map(|(index, operation)| {
                let is_submission =
                    matches!(operation.request().subject, OperationSubject::Submission(_));
                let (mutant, locator) = mutation_fact(operation.request().case.step.as_str());
                let role = request_role(
                    ceremony,
                    operation.request().case.step.as_str(),
                    is_submission,
                    Some(index) == control_index,
                    mutant.is_some(),
                );
                let attributed = role == RequestRole::Refusal;
                CeremonyOperationFacts {
                    operation_id: operation.operation_id().to_owned(),
                    case_step: operation.request().case.step.clone(),
                    role,
                    control_request_id: attributed.then(|| control_request_id.clone()).flatten(),
                    control_identity: attributed.then(|| control_identity.clone()).flatten(),
                    mutant,
                    locator,
                    projection: None,
                }
            })
            .collect();
        Self {
            digests: Vec::new(),
            operations,
        }
    }

    fn with_digest(mut self, name: &'static str, value: Option<[u8; 32]>) -> Self {
        if let Some(value) = value {
            self.digests.push(CaptureDigest { name, value });
        }
        self
    }

    fn with_projection(mut self, role: RequestRole, projection: ProjectionInput) -> Self {
        if let Some(operation) = self
            .operations
            .iter_mut()
            .find(|operation| operation.role == role)
        {
            operation.projection = Some(projection);
        }
        self
    }

    fn with_locator(mut self, step: &str, locator: LiveMutationLocator) -> Self {
        // Origin records also carry ceremony-only controls such as
        // `missing-rangeproof`, which has no row in the closed report-mutant
        // vocabulary. A locator completes an already typed mutation; it does
        // not promote every origin negative into a report mutation.
        if let Some(operation) = self
            .operations
            .iter_mut()
            .find(|operation| operation.case_step == step && operation.mutant.is_some())
        {
            operation.locator = Some(locator);
        }
        self
    }

    fn operation(&self, operation_id: &str) -> Option<&CeremonyOperationFacts> {
        self.operations
            .iter()
            .find(|operation| operation.operation_id == operation_id)
    }
}

fn request_role(
    ceremony: CeremonyId,
    step: &str,
    is_submission: bool,
    is_control: bool,
    is_mutant: bool,
) -> RequestRole {
    if !is_submission {
        return RequestRole::Auxiliary;
    }
    if is_control {
        return RequestRole::Control;
    }
    if ceremony == CeremonyId::PairsArc {
        return match step {
            "explicit-paired-one-to-one" => RequestRole::PairedExplicit,
            "private-paired-one-to-one" => RequestRole::PairedPrivate,
            _ => RequestRole::Auxiliary,
        };
    }
    if is_negative_ceremony(ceremony) {
        return if is_mutant {
            RequestRole::Refusal
        } else {
            RequestRole::Auxiliary
        };
    }
    // These observation ceremonies intentionally submit controls or probes
    // whose verdict is data. Only their typed selected case is an acceptance
    // carrier; the other submissions remain auxiliary even when refused.
    match ceremony {
        CeremonyId::OwnerObservation => {
            if step == vectors::live_owner_observation::OwnerObservationCase::SelectedProfile.name()
            {
                RequestRole::Acceptance
            } else {
                RequestRole::Auxiliary
            }
        }
        CeremonyId::ProofBearingObservation => {
            if step
                == vectors::live_proof_bearing_observation::ProofBearingCase::SelectedProfile.name()
            {
                RequestRole::Acceptance
            } else {
                RequestRole::Auxiliary
            }
        }
        CeremonyId::Report | CeremonyId::SponsoredCommittedValue => RequestRole::Auxiliary,
        _ => RequestRole::Acceptance,
    }
}

const fn is_negative_ceremony(ceremony: CeremonyId) -> bool {
    matches!(
        ceremony,
        CeremonyId::ConservationNegatives
            | CeremonyId::ExplicitWitnessNegatives
            | CeremonyId::KeypathProbe
            | CeremonyId::OffsettingFlowNegatives
            | CeremonyId::OwnerSigningNegatives
            | CeremonyId::SponsoredMissingAuthorization
    )
}

fn control_submission(
    ceremony: CeremonyId,
    capture: &NativeOperationCapture,
    submissions: &[usize],
) -> Option<usize> {
    let named = match ceremony {
        CeremonyId::ConservationNegatives => Some("submit-balance-valid-control"),
        CeremonyId::KeypathProbe => Some("script-path-control"),
        CeremonyId::OffsettingFlowNegatives => Some("submit-two-in-two-out-control"),
        CeremonyId::OwnerSigningNegatives => Some("vault-control-entitlement-control"),
        CeremonyId::SponsoredMissingAuthorization => Some("submit-sponsor-signed-control"),
        _ => None,
    };
    named
        .and_then(|wanted| {
            submissions
                .iter()
                .copied()
                .find(|index| capture.operations()[*index].request().case.step == wanted)
        })
        .or_else(|| {
            is_negative_ceremony(ceremony)
                .then(|| submissions.last().copied())
                .flatten()
        })
}

fn mutation_fact(step: &str) -> (Option<LiveMutantKind>, Option<LiveMutationLocator>) {
    use transaction::bytes::{SerializedFieldLocator, SerializedOutputField};

    let witness_item = || LiveMutationLocator::WitnessItem {
        input_index: 0,
        item_index: 0,
    };
    let field = |output_index, field| {
        LiveMutationLocator::SerializedOutputField(SerializedFieldLocator::new(output_index, field))
    };
    let fact = match step {
        "wrong-blinder" => (
            LiveMutantKind::WrongPrivateBlindingBalance,
            field(0, SerializedOutputField::ValueCommitment),
        ),
        "private-ct-imbalance" => (
            LiveMutantKind::PrivateCtImbalance,
            field(1, SerializedOutputField::ValueCommitment),
        ),
        "malformed-rangeproof" => (
            LiveMutantKind::MalformedRangeproof,
            field(0, SerializedOutputField::RangeproofBytes),
        ),
        "empty-signature" => (LiveMutantKind::EmptySignature, witness_item()),
        "malformed-signature" => (LiveMutantKind::MalformedSignature, witness_item()),
        "key-path-spend-attempt" => (
            LiveMutantKind::KeyPathEscape,
            LiveMutationLocator::WitnessPathShape {
                input_index: 0,
                control_stack_items: 3,
                mutant_stack_items: 1,
                changed_positions: vec![0, 1, 2],
                control_role: LiveWitnessPathRole::ScriptPath,
                mutant_role: LiveWitnessPathRole::KeyPath,
                witnessless_serialization_equal: true,
            },
        ),
        "submit-unauthorized-sponsor-control" => {
            (LiveMutantKind::MissingSponsorAuthorization, witness_item())
        }
        // Item TWO rather than item zero: the witness stack is signature,
        // leaf script, control block, and this surgery resizes the control
        // block. The other witness mutants move item zero, which is what
        // keeps this row's locator its own.
        "malformed-control-path" => (
            LiveMutantKind::MalformedControlPath,
            LiveMutationLocator::WitnessItem {
                input_index: 0,
                item_index: 2,
            },
        ),
        "bare-u-output-mutant" => (
            LiveMutantKind::VaultControlEntitlementOrBareUOutput,
            LiveMutationLocator::WitnesslessRange { start: 0, end: 0 },
        ),
        _ => return structural_mutation_fact(step),
    };
    (Some(fact.0), Some(fact.1))
}

fn structural_mutation_fact(step: &str) -> (Option<LiveMutantKind>, Option<LiveMutationLocator>) {
    let fact = match step {
        "consensus-wrong-explicit-asset" => (
            LiveMutantKind::WrongExplicitAsset,
            LiveMutationLocator::WitnesslessRange { start: 0, end: 0 },
        ),
        "consensus-confidential-asset-commitment" => (
            LiveMutantKind::ConfidentialAssetCommitment,
            LiveMutationLocator::WitnesslessRange { start: 0, end: 0 },
        ),
        "consensus-output-total-one-below-input" => (
            LiveMutantKind::OutputTotalOneBelowInput,
            LiveMutationLocator::WitnesslessRange { start: 0, end: 0 },
        ),
        "consensus-output-total-one-above-input" => (
            LiveMutantKind::OutputTotalOneAboveInput,
            LiveMutationLocator::WitnesslessRange { start: 0, end: 0 },
        ),
        "consensus-private-output-omitted" => (
            LiveMutantKind::PrivateOutputOmitted,
            LiveMutationLocator::TransactionShape {
                control_inputs: 2,
                mutant_inputs: 2,
                control_outputs: 2,
                mutant_outputs: 1,
            },
        ),
        "consensus-hidden-private-u-output" => (
            LiveMutantKind::HiddenPrivateUOutput,
            LiveMutationLocator::TransactionShape {
                control_inputs: 2,
                mutant_inputs: 2,
                control_outputs: 2,
                mutant_outputs: 3,
            },
        ),
        "consensus-omitted-source" => (
            LiveMutantKind::OmittedSource,
            LiveMutationLocator::TransactionShape {
                control_inputs: 2,
                mutant_inputs: 1,
                control_outputs: 2,
                mutant_outputs: 2,
            },
        ),
        // The added flow widens BOTH sides by one, which is what keeps the
        // candidate balanced and the refusal the covenant's rather than the
        // consensus tally's.
        "offsetting-flow" => (
            LiveMutantKind::SecondOffsettingUFlow,
            LiveMutationLocator::TransactionShape {
                control_inputs: 2,
                mutant_inputs: 3,
                control_outputs: 2,
                mutant_outputs: 3,
            },
        ),
        "consensus-amount-outside-semantic-domain" => (
            LiveMutantKind::AmountOutsideSemanticDomain,
            LiveMutationLocator::WitnesslessRange { start: 0, end: 0 },
        ),
        // Exactly ONE coordinator, at input one rather than input zero.
        // The two collapsing arrangements give two coordinators or none;
        // this one keeps the control's count and moves the position, so
        // the mutant indices are what separate it from the control.
        "leaf-arrangement-member-coordinator-leaf-exchange" => (
            LiveMutantKind::MemberCoordinatorLeafExchange,
            LiveMutationLocator::CommittedLeafArrangement {
                input_indices: vec![0, 1],
                control_coordinator_leaf_indices: vec![0],
                mutant_coordinator_leaf_indices: vec![1],
                control_committed_leaf_programs: Vec::new(),
                mutant_committed_leaf_programs: Vec::new(),
            },
        ),
        "leaf-arrangement-two-coordinators" => (
            LiveMutantKind::TwoCoordinators,
            LiveMutationLocator::CommittedLeafArrangement {
                input_indices: vec![0, 1],
                control_coordinator_leaf_indices: vec![0],
                mutant_coordinator_leaf_indices: vec![0, 1],
                control_committed_leaf_programs: Vec::new(),
                mutant_committed_leaf_programs: Vec::new(),
            },
        ),
        "leaf-arrangement-no-coordinator" => (
            LiveMutantKind::NoCoordinator,
            LiveMutationLocator::CommittedLeafArrangement {
                input_indices: vec![0, 1],
                control_coordinator_leaf_indices: vec![0],
                mutant_coordinator_leaf_indices: Vec::new(),
                control_committed_leaf_programs: Vec::new(),
                mutant_committed_leaf_programs: Vec::new(),
            },
        ),
        _ => return (None, None),
    };
    (Some(fact.0), Some(fact.1))
}

fn pair_projection_input(programs: &[Vec<u8>]) -> Result<ProjectionInput, String> {
    let fixture = vectors::live_pair_arc::pair_arc_fixture();
    let owners = fixture
        .sources()
        .iter()
        .map(|source| vectors::live_pair_arc::published_scalar(source.owner()).to_vec())
        .collect();
    let amounts = fixture
        .sources()
        .iter()
        .map(|source| source.amount())
        .collect();
    let destinations = fixture
        .destinations()
        .iter()
        .map(|destination| {
            programs
                .get(destination.owner())
                .cloned()
                .map(|program| {
                    (
                        vectors::live_pair_arc::published_scalar(destination.owner()).to_vec(),
                        (program, destination.amount()),
                    )
                })
                .ok_or_else(|| "the pair projection program census differs".to_owned())
        })
        .collect::<Result<_, _>>()?;
    Ok(ProjectionInput {
        owners,
        amounts,
        destinations,
    })
}

fn sponsor_capture_facts(
    ceremony: CeremonyId,
    capture: &NativeOperationCapture,
    predecessor_digest: Option<[u8; 32]>,
) -> CeremonyCaptureFacts {
    CeremonyCaptureFacts::from_capture(ceremony, capture)
        .with_digest("predecessor", predecessor_digest)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CaptureDestination {
    directory: PathBuf,
    short_sha: String,
    suite_commit: String,
    suite_tree: String,
}

impl CaptureDestination {
    fn from_environment() -> Result<Option<Self>, String> {
        let Some(directory) = environment("TRIPOD_LIVE_REPORT_DIR") else {
            return Ok(None);
        };
        let short_sha = environment("TRIPOD_LIVE_SUITE_SHORT_SHA").ok_or_else(|| {
            "TRIPOD_LIVE_SUITE_SHORT_SHA is required for enhanced capture".to_owned()
        })?;
        let suite_commit = environment("TRIPOD_LIVE_SUITE_COMMIT")
            .ok_or_else(|| "TRIPOD_LIVE_SUITE_COMMIT is required".to_owned())?;
        let suite_tree = environment("TRIPOD_LIVE_SUITE_TREE")
            .ok_or_else(|| "TRIPOD_LIVE_SUITE_TREE is required".to_owned())?;
        Self::new(
            PathBuf::from(directory),
            short_sha,
            suite_commit,
            suite_tree,
        )
        .map(Some)
    }

    fn new(
        directory: PathBuf,
        short_sha: String,
        suite_commit: String,
        suite_tree: String,
    ) -> Result<Self, String> {
        if short_sha.is_empty()
            || !short_sha
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err("TRIPOD_LIVE_SUITE_SHORT_SHA is not canonical lower-case hex".to_owned());
        }
        require_lower_hex(&suite_commit, 40, "suite commit")?;
        require_lower_hex(&suite_tree, 40, "suite tree")?;
        Ok(Self {
            directory,
            short_sha,
            suite_commit,
            suite_tree,
        })
    }

    fn capture_path(
        &self,
        ceremony: CeremonyId,
        current_test_name: &str,
    ) -> Result<PathBuf, String> {
        if current_test_name != ceremony.rust_test_name() {
            return Err(format!(
                "ceremony {} belongs to {}, not {current_test_name}",
                ceremony.as_str(),
                ceremony.rust_test_name(),
            ));
        }
        let filename = if ceremony.is_setup() {
            format!("{}.{}.setup", self.short_sha, ceremony.as_str())
        } else {
            format!("{}.{}.capture", self.short_sha, ceremony.as_str())
        };
        let path = self.directory.join(filename);
        if path.exists() {
            return Err(format!(
                "capture destination already exists: {}",
                path.display()
            ));
        }
        Ok(path)
    }
}

#[derive(Debug)]
struct CaptureGuard {
    ceremony: CeremonyId,
    destination: Option<CaptureDestination>,
    path: Option<PathBuf>,
    transcript_written: bool,
}

impl CaptureGuard {
    fn new(ceremony: CeremonyId) -> Self {
        let capture = capture_path(ceremony).expect("the enhanced capture path is valid");
        let (destination, path) = capture.unzip();
        Self {
            ceremony,
            destination,
            path,
            transcript_written: false,
        }
    }

    fn for_test(ceremony: CeremonyId, destination: CaptureDestination) -> Self {
        let path = destination
            .capture_path(ceremony, ceremony.rust_test_name())
            .expect("the test capture path is valid");
        Self {
            ceremony,
            destination: Some(destination),
            path: Some(path),
            transcript_written: false,
        }
    }

    fn capture_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    const fn mark_transcript_written(&mut self) {
        self.transcript_written = true;
    }
}

impl Drop for CaptureGuard {
    fn drop(&mut self) {
        let Some(path) = self.path.as_deref() else {
            return;
        };
        if self.ceremony.is_setup() {
            return;
        }
        let status = if !self.transcript_written {
            "incomplete"
        } else if std::thread::panicking() {
            "failed-after-transcript"
        } else {
            "passed"
        };
        let sidecar = format!(
            "timing-schema 1\nceremony-id {}\nstatus {status}\n",
            self.ceremony.as_str(),
        );
        let result = std::fs::write(timing_path(path), sidecar);
        assert!(
            result.is_ok() || std::thread::panicking(),
            "the enhanced timing sidecar is written",
        );
    }
}

fn capture_path(ceremony: CeremonyId) -> Result<Option<(CaptureDestination, PathBuf)>, String> {
    let Some(destination) = CaptureDestination::from_environment()? else {
        return Ok(None);
    };
    let thread = std::thread::current();
    let current = thread
        .name()
        .and_then(|name| name.rsplit("::").next())
        .ok_or_else(|| "the current Rust test has no name".to_owned())?;
    let path = destination.capture_path(ceremony, current)?;
    Ok(Some((destination, path)))
}

fn capture_diagnostics(ceremony: CeremonyId, legacy_report: Option<&Path>) -> PathBuf {
    if let Some(directory) = environment("TRIPOD_LIVE_REPORT_DIR") {
        return Path::new(&directory)
            .join("diagnostics")
            .join(ceremony.as_str());
    }
    legacy_report
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn legacy_report(extension: Option<&str>) -> Option<PathBuf> {
    let enhanced_directory = environment("TRIPOD_LIVE_REPORT_DIR").map(PathBuf::from);
    let legacy_base = environment("TRIPOD_LIVE_REPORT").map(PathBuf::from);
    legacy_report_for(
        enhanced_directory.as_deref(),
        legacy_base.as_deref(),
        extension,
    )
}

fn legacy_report_for(
    enhanced_directory: Option<&Path>,
    legacy_base: Option<&Path>,
    extension: Option<&str>,
) -> Option<PathBuf> {
    if enhanced_directory.is_some() {
        return None;
    }
    let base = legacy_base?;
    Some(extension.map_or_else(|| base.to_path_buf(), |value| base.with_extension(value)))
}

fn execute_and_capture(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
    binding: &target_elements::ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    planner: &mut dyn TargetOperationPlanner,
) -> (
    Result<
        target_elements_conformance::executor::ExecutionTranscript,
        target_elements_conformance::error::NativeConformanceError,
    >,
    NativeOperationCapture,
) {
    let mut capture = NativeOperationCapture::default();
    let outcome =
        execute_operations_captured(target, binding, configuration, planner, &mut capture);
    (outcome, capture)
}

fn write_capture_before_gates(
    guard: &mut CaptureGuard,
    capture: &NativeOperationCapture,
    facts: &CeremonyCaptureFacts,
    legacy_rendering: &str,
) {
    let Some(path) = guard.capture_path().map(Path::to_path_buf) else {
        return;
    };
    if guard.ceremony.is_setup() {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .expect("the setup artifact is opened for append");
        std::io::Write::write_all(&mut file, legacy_rendering.as_bytes())
            .expect("the setup artifact is written");
        guard.mark_transcript_written();
        return;
    }
    let destination = guard
        .destination
        .as_ref()
        .expect("an enhanced path carries its capture destination");
    let rendered = render_enhanced_capture(
        destination,
        guard.ceremony,
        capture,
        facts,
        legacy_rendering,
    )
    .expect("the enhanced transcript facts are complete");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("the enhanced capture destination is new");
    std::io::Write::write_all(&mut file, rendered.as_bytes())
        .expect("the enhanced capture is written");
    guard.mark_transcript_written();
}

fn render_enhanced_capture(
    destination: &CaptureDestination,
    ceremony: CeremonyId,
    capture: &NativeOperationCapture,
    facts: &CeremonyCaptureFacts,
    legacy_rendering: &str,
) -> Result<String, String> {
    validate_capture_facts(capture, facts)?;
    let mut out = String::new();
    render_run_identity(&mut out, destination, ceremony, capture)?;
    render_executor_context(&mut out, capture)?;
    render_digests(&mut out, facts);
    render_operations(&mut out, capture, facts)?;
    write_bytes_field(&mut out, "legacy-rendering", legacy_rendering.as_bytes());
    let terminal = capture
        .terminal_state()
        .map_or("incomplete", |state| state.as_str());
    let _ = writeln!(out, "terminal-state {terminal}");
    let _ = writeln!(out, "run-id-input end");
    let content_hash = sha256(out.as_bytes());
    let _ = writeln!(out, "capture-content-sha256 {}", hex_bytes(&content_hash));
    let _ = writeln!(out, "native-capture-end {}", ceremony.as_str());
    Ok(out)
}

fn render_run_identity(
    out: &mut String,
    destination: &CaptureDestination,
    ceremony: CeremonyId,
    capture: &NativeOperationCapture,
) -> Result<(), String> {
    let deployment = capture
        .deployment()
        .ok_or_else(|| "the journal carries no deployment".to_owned())?;
    let target = capture
        .target()
        .ok_or_else(|| "the journal carries no target".to_owned())?;
    let observed_environment = capture
        .environment()
        .ok_or_else(|| "the journal carries no environment".to_owned())?;
    if deployment_environment(deployment.environment())
        != wire_environment(observed_environment.environment)
    {
        return Err("the deployment and observed environment differ".to_owned());
    }
    let suite_commit = &destination.suite_commit;
    let suite_tree = &destination.suite_tree;
    let _ = writeln!(out, "native-capture-schema 1");
    let _ = writeln!(out, "ceremony-id {}", ceremony.as_str());
    write_text_field(out, "rust-test-name", ceremony.rust_test_name());
    let _ = writeln!(out, "suite-commit {suite_commit}");
    let _ = writeln!(out, "suite-tree {suite_tree}");
    let _ = writeln!(out, "fixture-digest-algorithm forward-v2");
    let _ = writeln!(out, "run-id-input begin");
    let _ = writeln!(
        out,
        "deployment-environment {}",
        deployment_environment(deployment.environment()),
    );
    write_text_field(
        out,
        "deployment-network-id",
        &hex_bytes(&deployment.network_id()),
    );
    write_text_field(
        out,
        "deployment-genesis-id",
        &hex_bytes(&deployment.genesis_id()),
    );
    write_text_field(
        out,
        "deployment-target-contract",
        target_contract(target.version()),
    );
    Ok(())
}

fn render_executor_context(
    out: &mut String,
    capture: &NativeOperationCapture,
) -> Result<(), String> {
    let handshake = capture
        .handshake()
        .ok_or_else(|| "the journal carries no handshake".to_owned())?;
    let observed_environment = capture
        .environment()
        .ok_or_else(|| "the journal carries no environment".to_owned())?;
    let _ = writeln!(
        out,
        "handshake-protocol-schema {}",
        handshake.protocol_schema,
    );
    write_text_field(out, "handshake-adapter-name", &handshake.adapter_name);
    write_text_field(out, "handshake-adapter-version", &handshake.adapter_version);
    write_optional_text_field(
        out,
        "handshake-framework-revision",
        handshake.framework_revision.as_deref(),
    );
    write_text_field(out, "handshake-node-name", &handshake.node_name);
    write_text_field(out, "handshake-node-version", &handshake.node_version);
    write_optional_text_field(
        out,
        "handshake-binary-reported-revision",
        handshake.binary_reported_revision.as_deref(),
    );
    write_optional_text_field(
        out,
        "handshake-intended-executed-tip",
        handshake.intended_executed_tip.as_deref(),
    );
    write_optional_text_field(
        out,
        "handshake-upstream-base",
        handshake.upstream_base.as_deref(),
    );
    let _ = writeln!(
        out,
        "handshake-topic-count {}",
        handshake.included_local_topics.len(),
    );
    for (ordinal, topic) in handshake.included_local_topics.iter().enumerate() {
        let _ = writeln!(
            out,
            "handshake-topic {ordinal} {} {}",
            topic.len(),
            hex_bytes(topic.as_bytes()),
        );
    }
    let _ = writeln!(out, "environment-schema {}", observed_environment.schema);
    write_text_field(out, "environment-chain", &observed_environment.chain_name);
    write_text_field(
        out,
        "environment-network-id",
        &hex_bytes(&observed_environment.network_id),
    );
    write_text_field(
        out,
        "environment-genesis-id",
        &hex_bytes(&observed_environment.genesis_id),
    );
    render_domains(out, handshake, observed_environment);
    render_leaf_versions(out, handshake, observed_environment);
    render_capabilities(out, &handshake.capabilities);
    render_funding_advertisement(out, handshake);
    Ok(())
}

fn render_digests(out: &mut String, facts: &CeremonyCaptureFacts) {
    let _ = writeln!(out, "digest-count {}", facts.digests.len());
    for (ordinal, digest) in facts.digests.iter().enumerate() {
        let _ = writeln!(
            out,
            "digest {ordinal} {} forward-v2 {}",
            digest.name,
            hex_bytes(&digest.value),
        );
    }
}

fn render_operations(
    out: &mut String,
    capture: &NativeOperationCapture,
    facts: &CeremonyCaptureFacts,
) -> Result<(), String> {
    let _ = writeln!(out, "operation-count {}", capture.operations().len());
    for (ordinal, operation) in capture.operations().iter().enumerate() {
        let operation_facts = facts
            .operation(operation.operation_id())
            .ok_or_else(|| format!("ceremony facts omit operation {}", operation.operation_id()))?;
        let _ = writeln!(out, "operation {ordinal} begin");
        write_text_field(out, "operation-id", operation.operation_id());
        write_text_field(out, "request-id", operation.request_id());
        let _ = writeln!(out, "request-role {}", operation_facts.role.as_str());
        let request_bytes = operation.transaction_bytes().unwrap_or_default();
        let _ = writeln!(
            out,
            "request-bytes {} {}",
            request_bytes.len(),
            hex_bytes(request_bytes),
        );
        write_optional_text_field(out, "response-id", operation.response_id());
        write_optional_text_field(out, "response-request-id", operation.response_request_id());
        write_optional_text_field(
            out,
            "response-operation-id",
            operation.response_operation_id(),
        );
        let _ = writeln!(
            out,
            "response-verdict {}",
            operation.verdict().map_or("none", response_verdict),
        );
        let _ = writeln!(
            out,
            "response-layer {}",
            operation.observed_layer().map_or("none", response_layer),
        );
        match operation.target_identity() {
            Some(identity) => {
                let _ = writeln!(out, "response-target-identity {identity}");
            }
            None => {
                let _ = writeln!(out, "response-target-identity none");
            }
        }
        write_bytes_field(
            out,
            "response-detail",
            operation.detail().unwrap_or_default().as_bytes(),
        );
        write_optional_control_id(
            out,
            "attribution-control-request-id",
            operation_facts.control_request_id.as_deref(),
        );
        match operation_facts.control_identity.as_deref() {
            Some(identity) => {
                let _ = writeln!(out, "attribution-control-identity {identity}");
            }
            None => {
                let _ = writeln!(out, "attribution-control-identity none");
            }
        }
        let _ = writeln!(
            out,
            "mutation-kind {}",
            operation_facts.mutant.map_or("none", LiveMutantKind::row),
        );
        render_locator(out, operation_facts.locator.as_ref());
        render_projection(out, operation_facts.projection.as_ref());
        let _ = writeln!(out, "operation {ordinal} end");
    }
    Ok(())
}

fn validate_capture_facts(
    capture: &NativeOperationCapture,
    facts: &CeremonyCaptureFacts,
) -> Result<(), String> {
    if facts.operations.len() != capture.operations().len() {
        return Err("the ceremony fact census differs from the journal".to_owned());
    }
    let mut seen = BTreeSet::new();
    for fact in &facts.operations {
        if !seen.insert(fact.operation_id.as_str()) {
            return Err(format!(
                "duplicate ceremony fact for operation {}",
                fact.operation_id,
            ));
        }
        let operation = capture
            .operations()
            .iter()
            .find(|operation| operation.operation_id() == fact.operation_id)
            .ok_or_else(|| format!("ceremony fact names absent operation {}", fact.operation_id))?;
        let expected_verdict = match fact.role {
            RequestRole::Acceptance
            | RequestRole::Control
            | RequestRole::PairedExplicit
            | RequestRole::PairedPrivate => Some(NativeVerdict::Accepted),
            RequestRole::Refusal => Some(NativeVerdict::Rejected),
            RequestRole::Auxiliary => None,
        };
        if expected_verdict.is_some() && operation.verdict() != expected_verdict {
            return Err(format!(
                "operation {} role {} disagrees with its journal verdict",
                fact.operation_id,
                fact.role.as_str(),
            ));
        }
        if fact.mutant.is_some() != fact.locator.is_some() {
            return Err(format!(
                "operation {} has an incomplete mutation declaration",
                fact.operation_id,
            ));
        }
        if fact.mutant.is_some() && fact.role != RequestRole::Refusal {
            return Err(format!(
                "operation {} carries a mutation outside the refusal role",
                fact.operation_id,
            ));
        }
        if fact.projection.is_some()
            && !matches!(
                fact.role,
                RequestRole::PairedExplicit | RequestRole::PairedPrivate
            )
        {
            return Err(format!(
                "operation {} carries a projection outside a paired role",
                fact.operation_id,
            ));
        }
        validate_control_attribution(capture, fact)?;
    }
    Ok(())
}

fn validate_control_attribution(
    capture: &NativeOperationCapture,
    fact: &CeremonyOperationFacts,
) -> Result<(), String> {
    if fact.role != RequestRole::Refusal {
        if fact.control_request_id.is_some() || fact.control_identity.is_some() {
            return Err(format!(
                "operation {} carries control attribution outside a refusal",
                fact.operation_id,
            ));
        }
        return Ok(());
    }
    let request_id = fact.control_request_id.as_deref().ok_or_else(|| {
        format!(
            "refusal operation {} has no control request link",
            fact.operation_id,
        )
    })?;
    let control = capture
        .operations()
        .iter()
        .find(|operation| operation.request_id() == request_id)
        .ok_or_else(|| format!("refusal control request {request_id} is absent"))?;
    if control.verdict() != Some(NativeVerdict::Accepted)
        || control.target_identity() != fact.control_identity.as_deref()
    {
        return Err(format!(
            "refusal operation {} has a non-accepted or mismatched control",
            fact.operation_id,
        ));
    }
    Ok(())
}

fn render_domains(
    out: &mut String,
    handshake: &target_elements_conformance::protocol::ExecutorHandshake,
    observed_environment: &target_elements_conformance::protocol::ExecutorEnvironmentObservation,
) {
    let domains: BTreeSet<_> = handshake
        .supported_domains
        .union(&observed_environment.active_domains)
        .copied()
        .collect();
    let _ = writeln!(out, "environment-domain-count {}", domains.len());
    for (ordinal, domain) in domains.into_iter().enumerate() {
        let _ = writeln!(
            out,
            "environment-domain {ordinal} {} supported {} active {}",
            execution_domain(domain),
            handshake.supported_domains.contains(&domain),
            observed_environment.active_domains.contains(&domain),
        );
    }
}

fn render_leaf_versions(
    out: &mut String,
    handshake: &target_elements_conformance::protocol::ExecutorHandshake,
    observed_environment: &target_elements_conformance::protocol::ExecutorEnvironmentObservation,
) {
    let leaves: BTreeSet<_> = handshake
        .supported_leaf_versions
        .union(&observed_environment.active_leaf_versions)
        .copied()
        .collect();
    let _ = writeln!(out, "environment-leaf-count {}", leaves.len());
    for (ordinal, leaf) in leaves.into_iter().enumerate() {
        let _ = writeln!(
            out,
            "environment-leaf {ordinal} {leaf} supported {} active {}",
            handshake.supported_leaf_versions.contains(&leaf),
            observed_environment.active_leaf_versions.contains(&leaf),
        );
    }
}

fn render_capabilities(out: &mut String, capabilities: &BTreeSet<ExecutorCapability>) {
    let _ = writeln!(out, "environment-capability-count {}", capabilities.len());
    for (ordinal, capability) in capabilities.iter().copied().enumerate() {
        let _ = writeln!(
            out,
            "environment-capability {ordinal} {}",
            executor_capability(capability),
        );
    }
}

fn render_funding_advertisement(
    out: &mut String,
    handshake: &target_elements_conformance::protocol::ExecutorHandshake,
) {
    let Some(advertisement) = handshake.confidential_funding.as_ref() else {
        let _ = writeln!(out, "environment-funding-count 0");
        return;
    };
    let count = advertisement.representation_profiles.len()
        + advertisement.custody_profiles.len()
        + advertisement.materializer_profiles.len()
        + advertisement.reproducibility_contracts.len();
    let _ = writeln!(out, "environment-funding-count {count}");
    let mut ordinal = 0_usize;
    for profile in advertisement.representation_profiles.iter().copied() {
        let _ = writeln!(
            out,
            "environment-funding {ordinal} representation {}",
            funding_representation(profile),
        );
        ordinal += 1;
    }
    for profile in advertisement.custody_profiles.iter().copied() {
        let _ = writeln!(
            out,
            "environment-funding {ordinal} custody {}",
            funding_custody(profile),
        );
        ordinal += 1;
    }
    for profile in advertisement.materializer_profiles.iter().copied() {
        let _ = writeln!(
            out,
            "environment-funding {ordinal} materializer {}",
            funding_materializer(profile),
        );
        ordinal += 1;
    }
    for contract in &advertisement.reproducibility_contracts {
        let _ = writeln!(
            out,
            "environment-funding {ordinal} reproducibility {}",
            contract.code(),
        );
        ordinal += 1;
    }
}

fn render_locator(out: &mut String, locator: Option<&LiveMutationLocator>) {
    match locator {
        None => {
            let _ = writeln!(out, "mutation-locator none");
        }
        Some(LiveMutationLocator::SerializedOutputField(locator)) => {
            let _ = writeln!(
                out,
                "mutation-locator serialized-output-field {} {}",
                locator.output_index(),
                locator.field().name(),
            );
        }
        Some(LiveMutationLocator::WitnessItem {
            input_index,
            item_index,
        }) => {
            let _ = writeln!(
                out,
                "mutation-locator witness-item {input_index} {item_index}",
            );
        }
        Some(LiveMutationLocator::WitnesslessRange { start, end }) => {
            let _ = writeln!(out, "mutation-locator witnessless-range {start} {end}");
        }
        Some(LiveMutationLocator::TransactionShape {
            control_inputs,
            mutant_inputs,
            control_outputs,
            mutant_outputs,
        }) => {
            let _ = writeln!(
                out,
                "mutation-locator transaction-shape {control_inputs} {mutant_inputs} {control_outputs} {mutant_outputs}",
            );
        }
        Some(LiveMutationLocator::WitnessPathShape {
            input_index,
            control_stack_items,
            mutant_stack_items,
            changed_positions,
            control_role,
            mutant_role,
            witnessless_serialization_equal,
        }) => {
            let positions = changed_positions
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let _ = writeln!(
                out,
                "mutation-locator witness-path-shape input {input_index} control-stack-count {control_stack_items} mutant-stack-count {mutant_stack_items} changed-count {} changed {positions} control-role {} mutant-role {} witnessless-equal {witnessless_serialization_equal}",
                changed_positions.len(),
                witness_path_role(*control_role),
                witness_path_role(*mutant_role),
            );
        }
        Some(LiveMutationLocator::CommittedLeafArrangement {
            input_indices,
            control_coordinator_leaf_indices,
            mutant_coordinator_leaf_indices,
            control_committed_leaf_programs,
            mutant_committed_leaf_programs,
        }) => {
            let _ = writeln!(
                out,
                "mutation-locator committed-leaf-arrangement inputs {} control-coordinators {} mutant-coordinators {} control-programs {} mutant-programs {}",
                decimal_list(input_indices),
                decimal_list(control_coordinator_leaf_indices),
                decimal_list(mutant_coordinator_leaf_indices),
                byte_list(control_committed_leaf_programs),
                byte_list(mutant_committed_leaf_programs),
            );
        }
    }
}

fn render_projection(out: &mut String, projection: Option<&ProjectionInput>) {
    let Some(projection) = projection else {
        let _ = writeln!(out, "projection-input none");
        return;
    };
    let _ = writeln!(out, "projection-input begin");
    let _ = writeln!(
        out,
        "projection-input-owner-count {}",
        projection.owners.len(),
    );
    for (ordinal, owner) in projection.owners.iter().enumerate() {
        let _ = writeln!(
            out,
            "projection-input-owner {ordinal} {} {}",
            owner.len(),
            hex_bytes(owner),
        );
    }
    let _ = writeln!(
        out,
        "projection-input-amount-count {}",
        projection.amounts.len(),
    );
    for (ordinal, amount) in projection.amounts.iter().enumerate() {
        let _ = writeln!(out, "projection-input-amount {ordinal} {amount}");
    }
    let _ = writeln!(
        out,
        "projection-destination-count {}",
        projection.destinations.len(),
    );
    for (ordinal, (_name, (program, amount))) in projection.destinations.iter().enumerate() {
        let _ = writeln!(
            out,
            "projection-destination {ordinal} {} {} {amount}",
            program.len(),
            hex_bytes(program),
        );
    }
    let _ = writeln!(out, "projection-input end");
}

fn write_text_field(out: &mut String, field: &str, value: &str) {
    let _ = writeln!(
        out,
        "{field} {} {}",
        value.len(),
        hex_bytes(value.as_bytes()),
    );
}

fn write_optional_text_field(out: &mut String, field: &str, value: Option<&str>) {
    write_text_field(out, field, value.unwrap_or_default());
}

fn write_optional_control_id(out: &mut String, field: &str, value: Option<&str>) {
    match value {
        Some(value) => write_text_field(out, field, value),
        None => {
            let _ = writeln!(out, "{field} none");
        }
    }
}

fn write_bytes_field(out: &mut String, field: &str, value: &[u8]) {
    let _ = writeln!(out, "{field} {} {}", value.len(), hex_bytes(value));
}

fn decimal_list(values: &[usize]) -> String {
    let members = values
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("{}:{members}", values.len())
}

fn byte_list(values: &[Vec<u8>]) -> String {
    let members = values
        .iter()
        .map(|value| format!("{}:{}", value.len(), hex_bytes(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{}:{members}", values.len())
}

fn require_lower_hex(value: &str, width: usize, name: &str) -> Result<(), String> {
    if value.len() == width
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(format!("{name} is not {width} lower-case hex digits"))
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    out
}

const fn deployment_environment(environment: DeploymentEnvironment) -> &'static str {
    match environment {
        DeploymentEnvironment::Development => "development",
        _ => "unknown",
    }
}

const fn wire_environment(environment: WireEnvironment) -> &'static str {
    match environment {
        WireEnvironment::Development => "development",
        _ => "unknown",
    }
}

const fn execution_domain(domain: WireExecutionDomain) -> &'static str {
    match domain {
        WireExecutionDomain::Tapscript => "tapscript",
        _ => "unknown",
    }
}

fn target_contract(version: target_elements::TargetContractVersion) -> &'static str {
    if version == target_elements::TargetContractVersion::V2 {
        "elements-tapscript-v2"
    } else {
        "unknown"
    }
}

const fn executor_capability(capability: ExecutorCapability) -> &'static str {
    match capability {
        ExecutorCapability::FinalStackReporting => "final-stack-reporting",
        ExecutorCapability::FinalAltstackReporting => "final-altstack-reporting",
        ExecutorCapability::FailureClassReporting => "failure-class-reporting",
        ExecutorCapability::TransactionContext => "transaction-context",
        ExecutorCapability::ResourceObservation => "resource-observation",
        ExecutorCapability::TreeMaterialization => "tree-materialization",
        ExecutorCapability::ConfidentialConservation => "confidential-conservation",
        ExecutorCapability::OwnerAuthorizedNormalization => "owner-authorized-normalization",
        ExecutorCapability::CompoundPrototypeFixtures => "compound-prototype-fixtures",
        ExecutorCapability::FreshProcessLifecycle => "fresh-process-lifecycle",
        ExecutorCapability::TestFundingCeremony => "test-funding-ceremony",
        ExecutorCapability::TargetTransactionSubmission => "target-transaction-submission",
        ExecutorCapability::TestSponsorAuthorization => "test-sponsor-authorization",
        ExecutorCapability::ConfidentialValueTestFunding => "confidential-value-test-funding",
        ExecutorCapability::ConfidentialValueSponsorAuthorization => {
            "confidential-value-sponsor-authorization"
        }
        _ => "unknown",
    }
}

const fn funding_representation(profile: FundingRepresentationProfile) -> &'static str {
    match profile {
        FundingRepresentationProfile::ExplicitAssetConfidentialValue => {
            "explicit-asset-confidential-value"
        }
        _ => "unknown",
    }
}

const fn funding_custody(profile: FundingCustodyProfile) -> &'static str {
    match profile {
        FundingCustodyProfile::CentralPublicFixtures => "central-public-fixtures",
        _ => "unknown",
    }
}

const fn funding_materializer(profile: FundingMaterializerProfile) -> &'static str {
    match profile {
        FundingMaterializerProfile::GuideCtfDeterministicV1 => "guide-ctf-deterministic-v1",
        _ => "unknown",
    }
}

const fn response_verdict(verdict: NativeVerdict) -> &'static str {
    match verdict {
        NativeVerdict::Accepted => "accepted",
        NativeVerdict::Rejected => "refused",
        _ => "incomplete",
    }
}

const fn response_layer(layer: ObservedOutcomeLayer) -> &'static str {
    match layer {
        ObservedOutcomeLayer::FixtureConstructionFailure => "fixture-construction-failure",
        ObservedOutcomeLayer::ExecutorInfrastructureFailure => "executor-infrastructure-failure",
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript => "consensus-rejection-before-script",
        ObservedOutcomeLayer::ScriptPathRejection => "script-path-rejection",
        ObservedOutcomeLayer::KeyPathRejection => "key-path-rejection",
        ObservedOutcomeLayer::RelayPolicyRejection => "relay-policy-rejection",
        ObservedOutcomeLayer::Accepted => "accepted",
        _ => "unknown",
    }
}

const fn witness_path_role(role: LiveWitnessPathRole) -> &'static str {
    match role {
        LiveWitnessPathRole::KeyPath => "key-path",
        LiveWitnessPathRole::ScriptPath => "script-path",
    }
}

const SHA256_INITIAL: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

const SHA256_ROUND: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

fn sha256(input: &[u8]) -> [u8; 32] {
    let mut padded = input.to_vec();
    let bit_length = u64::try_from(input.len())
        .unwrap_or(u64::MAX)
        .wrapping_mul(8);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_length.to_be_bytes());

    let mut state = SHA256_INITIAL;
    let (blocks, remainder) = padded.as_chunks::<64>();
    assert!(
        remainder.is_empty(),
        "SHA-256 padding must produce whole 64-byte blocks",
    );
    for block in blocks {
        let mut schedule = [0_u32; 64];
        let (words, remainder) = block.as_chunks::<4>();
        assert!(
            remainder.is_empty(),
            "a SHA-256 block must contain whole four-byte words",
        );
        for (index, word) in words.iter().enumerate() {
            schedule[index] = u32::from_be_bytes(*word);
        }
        for index in 16..64 {
            let left = schedule[index - 15];
            let right = schedule[index - 2];
            let sigma0 = left.rotate_right(7) ^ left.rotate_right(18) ^ (left >> 3);
            let sigma1 = right.rotate_right(17) ^ right.rotate_right(19) ^ (right >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(sigma0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(sigma1);
        }
        let [
            mut state_zero,
            mut state_one,
            mut state_two,
            mut state_three,
            mut state_four,
            mut state_five,
            mut state_six,
            mut state_seven,
        ] = state;
        for index in 0..64 {
            let upper = state_four.rotate_right(6)
                ^ state_four.rotate_right(11)
                ^ state_four.rotate_right(25);
            let choose = (state_four & state_five) ^ ((!state_four) & state_six);
            let first = state_seven
                .wrapping_add(upper)
                .wrapping_add(choose)
                .wrapping_add(SHA256_ROUND[index])
                .wrapping_add(schedule[index]);
            let lower = state_zero.rotate_right(2)
                ^ state_zero.rotate_right(13)
                ^ state_zero.rotate_right(22);
            let majority =
                (state_zero & state_one) ^ (state_zero & state_two) ^ (state_one & state_two);
            let second = lower.wrapping_add(majority);
            state_seven = state_six;
            state_six = state_five;
            state_five = state_four;
            state_four = state_three.wrapping_add(first);
            state_three = state_two;
            state_two = state_one;
            state_one = state_zero;
            state_zero = first.wrapping_add(second);
        }
        state[0] = state[0].wrapping_add(state_zero);
        state[1] = state[1].wrapping_add(state_one);
        state[2] = state[2].wrapping_add(state_two);
        state[3] = state[3].wrapping_add(state_three);
        state[4] = state[4].wrapping_add(state_four);
        state[5] = state[5].wrapping_add(state_five);
        state[6] = state[6].wrapping_add(state_six);
        state[7] = state[7].wrapping_add(state_seven);
    }
    let mut digest = [0_u8; 32];
    let (chunks, remainder) = digest.as_chunks_mut::<4>();
    assert!(
        remainder.is_empty(),
        "a SHA-256 digest must contain whole four-byte words",
    );
    for (chunk, word) in chunks.iter_mut().zip(state) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
    digest
}

fn environment(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn identifier(text: &str) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    let raw = text.as_bytes();
    let (pairs, _) = raw.as_chunks::<2>();
    for (slot, pair) in bytes.iter_mut().zip(pairs) {
        let digits = std::str::from_utf8(pair).expect("the identifier is hex");
        *slot = u8::from_str_radix(digits, 16).expect("the identifier is hex");
    }
    bytes
}

/// Where the run's wall time is written.
///
/// Beside the report and never inside it (§13.6). Duration is a property
/// of the machine the run happened on rather than of the run's result.
fn timing_path(report: &Path) -> PathBuf {
    let mut path = report.to_path_buf();
    let mut name = path.file_name().map_or_else(
        || "live-native-run".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    name.push_str(".timing");
    path.set_file_name(name);
    path
}

fn test_directory(label: &str) -> PathBuf {
    let base = environment("TMPDIR").map_or_else(std::env::temp_dir, PathBuf::from);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let directory = base.join(format!(
        "guide13-live-native-{label}-{}-{nonce}",
        std::process::id(),
    ));
    std::fs::create_dir(&directory).expect("the test directory is created");
    directory
}

fn test_capture_destination(directory: &Path) -> CaptureDestination {
    CaptureDestination::new(
        directory.to_path_buf(),
        "abc123".to_owned(),
        "1".repeat(40),
        "2".repeat(40),
    )
    .expect("the test capture destination is valid")
}

#[derive(Clone)]
struct ScriptedPlan {
    steps: Vec<OperationStep>,
    next: usize,
}

impl ScriptedPlan {
    const fn new(steps: Vec<OperationStep>) -> Self {
        Self { steps, next: 0 }
    }

    fn two_submissions() -> Self {
        Self::new(vec![
            OperationStep::new(
                "empty-signature",
                OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                    transaction_bytes: vec![0x00, 0x01, 0x80, 0xff],
                })),
            ),
            OperationStep::new(
                "explicit-one-to-one",
                OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                    transaction_bytes: vec![0x02, 0x03],
                })),
            ),
        ])
    }
}

impl TargetOperationPlanner for ScriptedPlan {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if previous.is_some() {
            self.next += 1;
        }
        Ok(self.steps.get(self.next).cloned())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptedOperationKind {
    Funding,
    Submission,
    SponsorFunding,
    SponsorSigning,
    ConfidentialFunding,
    ConfidentialSponsorFunding,
}

impl ScriptedOperationKind {
    const fn wire(self) -> &'static str {
        match self {
            Self::Funding => "fund",
            Self::Submission => "submit",
            Self::SponsorFunding => "fund_sponsor",
            Self::SponsorSigning => "sign_sponsor",
            Self::ConfidentialFunding => "fund_confidential",
            Self::ConfidentialSponsorFunding => "fund_confidential_sponsor",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScriptedCeremonyOperation {
    kind: ScriptedOperationKind,
    step: &'static str,
    accepted: bool,
}

const fn scripted_operation(
    kind: ScriptedOperationKind,
    step: &'static str,
    accepted: bool,
) -> ScriptedCeremonyOperation {
    ScriptedCeremonyOperation {
        kind,
        step,
        accepted,
    }
}

fn scripted_binding() -> ConfidentialFundingBinding {
    ConfidentialFundingBinding {
        fixture_handle: ConfidentialFixtureHandle::new("capture-scripted-fixture".to_owned()),
        fixture_digest: ConfidentialFixtureDigest::new([0x44; 32]),
        profiles: ConfidentialFundingProfiles {
            representation: FundingRepresentationProfile::ExplicitAssetConfidentialValue,
            custody: FundingCustodyProfile::CentralPublicFixtures,
            materializer: FundingMaterializerProfile::GuideCtfDeterministicV1,
            reproducibility_contract: ReproducibilityContract::ByteIdentity,
        },
    }
}

fn scripted_step(operation: ScriptedCeremonyOperation) -> OperationStep {
    let subject = match operation.kind {
        ScriptedOperationKind::Funding => {
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: true,
                asset: None,
                output_program: vec![0x51],
                outputs: 1,
                amount_per_output: 1,
            }))
        }
        ScriptedOperationKind::Submission => {
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: vec![0x02, 0x00, 0x00, 0x00],
            }))
        }
        ScriptedOperationKind::SponsorFunding => {
            OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: 1,
                amount_per_sponsor_output: 1,
            }))
        }
        ScriptedOperationKind::SponsorSigning => {
            OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                finalized_transaction: vec![0x02, 0x00, 0x00, 0x00],
                sponsor_input_index: 0,
                sponsor_outpoint: WireOutpoint {
                    txid: "00".repeat(32),
                    vout: 0,
                },
                sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            }))
        }
        ScriptedOperationKind::ConfidentialFunding => {
            OperationSubject::ConfidentialFunding(Box::new(TargetConfidentialFundingSubject {
                issue_asset: false,
                asset: Some("11".repeat(32)),
                destinations: vec![ConfidentialFundingDestination {
                    output_program: vec![0x51],
                }],
                binding: scripted_binding(),
            }))
        }
        ScriptedOperationKind::ConfidentialSponsorFunding => {
            OperationSubject::ConfidentialSponsorFunding(Box::new(
                TargetConfidentialSponsorFundingSubject {
                    destinations: vec![ConfidentialFundingDestination {
                        output_program: vec![0x51],
                    }],
                    binding: scripted_binding(),
                },
            ))
        }
    };
    OperationStep::new(operation.step, subject)
}

fn scripted_response(operation: ScriptedCeremonyOperation) -> String {
    let common = format!(
        "\"schema\":7,\"case\":{{\"operation\":\"{}\",\"step\":\"{}\"}}",
        operation.kind.wire(),
        operation.step,
    );
    if operation.accepted {
        let accepted_txid = "aa".repeat(32);
        return format!(
            "{{{common},\"observed_layer\":\"accepted\",\"observed_detail\":\"accepted exactly\",\"issued_asset\":null,\"funded_outputs\":[],\"confidential_funded_outputs\":[],\"mined_readback\":{{\"transaction_id\":\"{accepted_txid}\",\"witness_transaction_id\":\"{}\",\"block_hash\":\"{}\",\"block_height\":17,\"raw_transaction\":[2,0,0,0]}},\"accepted_txid\":\"{accepted_txid}\",\"sponsor_witness\":[],\"signature_bound_to\":null,\"resources\":{{\"script_bytes\":null,\"initial_stack_items\":null,\"peak_stack_items\":null,\"peak_altstack_items\":null,\"maximum_element_bytes\":null,\"validation_budget_used\":null,\"transaction_weight\":200}}}}",
            "bb".repeat(32),
            "cc".repeat(32),
        );
    }
    format!(
        "{{{common},\"observed_layer\":\"script_path_rejection\",\"observed_detail\":\"scripted refusal\",\"issued_asset\":null,\"funded_outputs\":[],\"confidential_funded_outputs\":[],\"mined_readback\":null,\"accepted_txid\":null,\"sponsor_witness\":[],\"signature_bound_to\":null,\"resources\":{{\"script_bytes\":null,\"initial_stack_items\":null,\"peak_stack_items\":null,\"peak_altstack_items\":null,\"maximum_element_bytes\":null,\"validation_budget_used\":null,\"transaction_weight\":null}}}}",
    )
}

fn scripted_ceremony_operations(ceremony: CeremonyId) -> Vec<ScriptedCeremonyOperation> {
    use ScriptedOperationKind::{
        ConfidentialFunding, ConfidentialSponsorFunding, Funding, SponsorFunding, SponsorSigning,
        Submission,
    };

    match ceremony {
        CeremonyId::ConservationNegatives => vec![
            scripted_operation(Funding, "issue-confidential-protocol-asset", false),
            scripted_operation(ConfidentialFunding, "fund-confidential-predecessor", false),
            scripted_operation(Submission, "wrong-blinder", false),
            scripted_operation(Submission, "missing-rangeproof", false),
            scripted_operation(Submission, "private-ct-imbalance", false),
            scripted_operation(Submission, "malformed-rangeproof", false),
            scripted_operation(Submission, "submit-balance-valid-control", true),
        ],
        CeremonyId::OwnerObservation => {
            let mut operations = vec![
                scripted_operation(Funding, "issue-protocol-asset", false),
                scripted_operation(Funding, "fund-explicit-constructor", false),
            ];
            operations.extend(
                vectors::live_owner_observation::OwnerObservationCase::ALL
                    .iter()
                    .copied()
                    .map(|case| {
                        scripted_operation(Submission, case.name(), !case.is_negative_control())
                    }),
            );
            operations
        }
        CeremonyId::ProofBearingObservation => {
            let mut operations = vec![
                scripted_operation(Funding, "issue-proof-bearing-protocol-asset", false),
                scripted_operation(ConfidentialFunding, "fund-proof-bearing-predecessor", false),
            ];
            operations.extend(
                vectors::live_proof_bearing_observation::ProofBearingCase::ALL
                    .iter()
                    .copied()
                    .map(|case| {
                        scripted_operation(Submission, case.name(), !case.is_negative_control())
                    }),
            );
            operations
        }
        CeremonyId::Report => vec![
            scripted_operation(Funding, "issue-protocol-asset", false),
            scripted_operation(Funding, "fund-explicit-constructor", false),
            scripted_operation(Funding, "fund-private-constructor", false),
            scripted_operation(Submission, "submit-explicit-transfer", false),
        ],
        CeremonyId::SponsoredCommittedValue => vec![
            scripted_operation(Funding, "issue-protocol-asset", false),
            scripted_operation(SponsorFunding, "fund-sponsor-region", false),
            scripted_operation(
                ConfidentialSponsorFunding,
                "fund-confidential-sponsor-reserve",
                false,
            ),
            scripted_operation(Funding, "fund-explicit-constructor", false),
            scripted_operation(SponsorSigning, "authorize-sponsor-input", false),
            scripted_operation(Submission, "submit-sponsor-signed-control", false),
        ],
        _ => panic!("no scripted census for this ceremony"),
    }
}

fn scripted_adapter(capabilities: &[&str], responses: &[String]) -> String {
    let capabilities = capabilities
        .iter()
        .map(|capability| format!("\"{capability}\""))
        .collect::<Vec<_>>()
        .join(",");
    let network = std::iter::repeat_n("17", 32).collect::<Vec<_>>().join(",");
    let genesis = std::iter::repeat_n("34", 32).collect::<Vec<_>>().join(",");
    let handshake = format!(
        "{{\"protocol_schema\":7,\"adapter_name\":\"capture-adapter\",\"adapter_version\":\"1.2.3\",\"framework_revision\":\"framework-tip\",\"node_name\":\"elementsd\",\"node_version\":\"23.2.1\",\"binary_reported_revision\":\"binary-tip\",\"intended_executed_tip\":\"intended-tip\",\"upstream_base\":\"upstream-base\",\"included_local_topics\":[\"topic-a\",\"topic-b\"],\"supported_domains\":[\"tapscript\"],\"supported_leaf_versions\":[196],\"capabilities\":[{capabilities}],\"confidential_funding\":{{\"representation_profiles\":[\"explicit_asset_confidential_value\"],\"custody_profiles\":[\"central_public_fixtures\"],\"materializer_profiles\":[\"guide_ctf_deterministic_v1\"],\"reproducibility_contracts\":[\"byte_identity\"]}}}}",
    );
    let observed_environment = format!(
        "{{\"schema\":7,\"environment\":\"development\",\"chain_name\":\"elementsregtest\",\"network_id\":[{network}],\"genesis_id\":[{genesis}],\"active_domains\":[\"tapscript\"],\"active_leaf_versions\":[196]}}",
    );
    let mut script = format!(
        "#!/bin/sh\nIFS= read -r request\nprintf '%s\\n' '{handshake}'\nprintf '%s\\n' '{observed_environment}'\n",
    );
    for response in responses {
        let _ = writeln!(script, "IFS= read -r request\nprintf '%s\\n' '{response}'");
    }
    script
}

#[cfg(unix)]
fn run_scripted_capture(
    label: &str,
    capabilities: &[&str],
    steps: Vec<OperationStep>,
    responses: &[String],
) -> NativeOperationCapture {
    use std::os::unix::fs::PermissionsExt as _;

    let directory = test_directory(label);
    let adapter = directory.join("adapter.sh");
    let script = scripted_adapter(capabilities, responses);
    std::fs::write(&adapter, script).expect("the scripted adapter is written");
    let mut permissions = std::fs::metadata(&adapter)
        .expect("the scripted adapter has metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&adapter, permissions).expect("the scripted adapter is executable");

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            [0x11; 32],
            [0x22; 32],
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");
    let configuration = ExecutorConfiguration::new(
        &adapter,
        ExecutorTrust::Mock,
        Duration::from_secs(2),
        ExecutorDiagnostics::in_directory(&directory.join("diagnostics")),
    );
    let mut planner = ScriptedPlan::new(steps);
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    outcome.expect("the scripted adapter exchange completes");
    std::fs::remove_dir_all(directory).expect("the scripted test directory is removed");
    capture
}

#[cfg(unix)]
fn scripted_capture() -> NativeOperationCapture {
    let refused = concat!(
        "{\"schema\":7,\"case\":{\"operation\":\"submit\",",
        "\"step\":\"empty-signature\"},",
        "\"observed_layer\":\"script_path_rejection\",",
        "\"observed_detail\":\"mutant refused\",",
        "\"issued_asset\":null,\"funded_outputs\":[],",
        "\"confidential_funded_outputs\":[],\"mined_readback\":null,",
        "\"accepted_txid\":null,\"sponsor_witness\":[],",
        "\"signature_bound_to\":null,\"resources\":{",
        "\"script_bytes\":4,\"initial_stack_items\":1,",
        "\"peak_stack_items\":2,\"peak_altstack_items\":0,",
        "\"maximum_element_bytes\":64,\"validation_budget_used\":50,",
        "\"transaction_weight\":100}}",
    )
    .to_owned();
    let accepted = scripted_response(scripted_operation(
        ScriptedOperationKind::Submission,
        "explicit-one-to-one",
        true,
    ));
    run_scripted_capture(
        "scripted",
        &[
            "target_transaction_submission",
            "confidential_value_test_funding",
        ],
        ScriptedPlan::two_submissions().steps,
        &[refused, accepted],
    )
}

#[cfg(unix)]
fn scripted_ceremony_capture(ceremony: CeremonyId) -> NativeOperationCapture {
    let operations = scripted_ceremony_operations(ceremony);
    let steps = operations.iter().copied().map(scripted_step).collect();
    let responses = operations
        .iter()
        .copied()
        .map(scripted_response)
        .collect::<Vec<_>>();
    run_scripted_capture(
        ceremony.as_str(),
        &[
            "test_funding_ceremony",
            "target_transaction_submission",
            "test_sponsor_authorization",
            "confidential_value_test_funding",
            "confidential_value_sponsor_authorization",
        ],
        steps,
        &responses,
    )
}

#[test]
fn the_ceremony_roster_matches_the_driver_and_each_test_name_is_unique() {
    const SEMANTIC_IDS: [&str; 40] = [
        "conservation-negatives",
        "explicit-boundary-values",
        "explicit-maximum-inputs",
        "explicit-maximum-outputs",
        "explicit-merge",
        "explicit-normalization",
        "explicit-one-destination-owner",
        "explicit-one-to-one",
        "explicit-repeated-owner",
        "explicit-self-paid-fee",
        "explicit-several-destination-owners",
        "explicit-several-owners",
        "explicit-several-to-several",
        "explicit-split",
        "explicit-sponsorless",
        "explicit-witness-negatives",
        "keypath-probe",
        "multi-entry-crossing",
        "multi-exit-crossing",
        "multi-many-to-many",
        "multi-one-to-one-with-fee",
        "multi-private-merge",
        "multi-pure-split",
        "multi-several-owners",
        "multi-split",
        "multi-strict-one-to-one",
        "offsetting-flow-negatives",
        "owner-observation",
        "owner-signing-negatives",
        "pairs-arc",
        "private-restart-control",
        "private-restart-parity",
        "proof-bearing-observation",
        "report",
        "sponsored-change-absent",
        "sponsored-change-present",
        "sponsored-committed-value",
        "sponsored-missing-authorization",
        "sponsored-private-explicit-no-change",
        "sponsored-private-with-change",
    ];
    let observed: Vec<_> = CeremonyId::ALL
        .iter()
        .copied()
        .filter(|ceremony| !ceremony.is_setup())
        .map(CeremonyId::as_str)
        .collect();
    assert_eq!(observed, SEMANTIC_IDS);
    assert_eq!(CeremonyId::ALL.len(), 41);
    let names: BTreeSet<_> = CeremonyId::ALL
        .iter()
        .copied()
        .map(CeremonyId::rust_test_name)
        .collect();
    assert_eq!(names.len(), CeremonyId::ALL.len());
}

#[test]
fn a_second_capture_at_one_derived_destination_is_refused() {
    let directory = test_directory("path");
    let destination = test_capture_destination(&directory);
    let first = destination
        .capture_path(CeremonyId::Report, CeremonyId::Report.rust_test_name())
        .expect("the first path is derived");
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&first)
        .expect("the first writer creates the destination");
    let second = destination.capture_path(CeremonyId::Report, CeremonyId::Report.rust_test_name());
    assert!(second.is_err(), "the second writer was admitted");
    assert!(
        destination
            .capture_path(
                CeremonyId::Report,
                CeremonyId::OwnerObservation.rust_test_name(),
            )
            .is_err(),
        "a different Rust test claimed the report ceremony",
    );
    std::fs::remove_dir_all(directory).expect("the path test directory is removed");
}

#[test]
fn the_legacy_report_variable_keeps_its_original_path_contract() {
    let directory = test_directory("legacy-path");
    let base = directory.join("report.txt");
    assert_eq!(
        legacy_report_for(None, Some(&base), None),
        Some(base.clone())
    );
    assert_eq!(
        legacy_report_for(None, Some(&base), Some("explicit-one-to-one")),
        Some(base.with_extension("explicit-one-to-one")),
    );
    assert_eq!(legacy_report_for(Some(&directory), Some(&base), None), None);
    std::fs::remove_dir_all(directory).expect("the legacy path test directory is removed");
}

#[cfg(unix)]
#[test]
fn conservation_fact_assembly_matches_its_seven_operation_census() {
    use transaction::bytes::{SerializedFieldLocator, SerializedOutputField};

    let capture = scripted_ceremony_capture(CeremonyId::ConservationNegatives);
    let mut facts = CeremonyCaptureFacts::from_capture(CeremonyId::ConservationNegatives, &capture)
        .with_digest("predecessor", Some([0x31; 32]))
        .with_digest("successor", Some([0x32; 32]));
    for step in [
        "wrong-blinder",
        "missing-rangeproof",
        "private-ct-imbalance",
        "malformed-rangeproof",
    ] {
        let locator = mutation_fact(step).1.unwrap_or_else(|| {
            LiveMutationLocator::SerializedOutputField(SerializedFieldLocator::new(
                0,
                SerializedOutputField::RangeproofBytes,
            ))
        });
        facts = facts.with_locator(step, locator);
    }

    validate_capture_facts(&capture, &facts).expect("the conservation facts are complete");
    assert_eq!(facts.operations.len(), 7);
    let missing = facts
        .operation("operation-3")
        .expect("the missing-rangeproof operation is present");
    assert_eq!(missing.case_step, "missing-rangeproof");
    assert_eq!(missing.role, RequestRole::Auxiliary);
    assert_eq!((missing.mutant, missing.locator.as_ref()), (None, None));
    assert_eq!(facts.digests.len(), 2);
}

#[cfg(unix)]
#[test]
fn owner_observation_fact_assembly_distinguishes_controls_from_the_acceptance() {
    let capture = scripted_ceremony_capture(CeremonyId::OwnerObservation);
    let facts = CeremonyCaptureFacts::from_capture(CeremonyId::OwnerObservation, &capture);

    validate_capture_facts(&capture, &facts).expect("the owner-observation facts are complete");
    assert_eq!(facts.operations.len(), 9);
    assert!(
        facts.operations[2..8]
            .iter()
            .all(|operation| operation.role == RequestRole::Auxiliary),
    );
    assert_eq!(facts.operations[8].role, RequestRole::Acceptance);
    assert_eq!(facts.digests.as_slice(), &[]);
}

#[cfg(unix)]
#[test]
fn proof_bearing_fact_assembly_distinguishes_controls_from_the_acceptance() {
    let capture = scripted_ceremony_capture(CeremonyId::ProofBearingObservation);
    let facts = CeremonyCaptureFacts::from_capture(CeremonyId::ProofBearingObservation, &capture)
        .with_digest("predecessor", Some([0x41; 32]));

    validate_capture_facts(&capture, &facts).expect("the proof-bearing facts are complete");
    assert_eq!(facts.operations.len(), 6);
    assert!(
        facts.operations[2..5]
            .iter()
            .all(|operation| operation.role == RequestRole::Auxiliary),
    );
    assert_eq!(facts.operations[5].role, RequestRole::Acceptance);
    assert_eq!(facts.digests.len(), 1);
}

#[cfg(unix)]
#[test]
fn report_fact_assembly_keeps_the_observational_submission_auxiliary() {
    let capture = scripted_ceremony_capture(CeremonyId::Report);
    let facts = CeremonyCaptureFacts::from_capture(CeremonyId::Report, &capture);

    validate_capture_facts(&capture, &facts).expect("the report facts are complete");
    assert_eq!(facts.operations.len(), 4);
    assert!(
        facts
            .operations
            .iter()
            .all(|operation| operation.role == RequestRole::Auxiliary),
    );
    assert_eq!(facts.digests.as_slice(), &[]);
}

#[cfg(unix)]
#[test]
fn committed_sponsor_fact_assembly_keeps_the_observation_and_fixture_digest() {
    let capture = scripted_ceremony_capture(CeremonyId::SponsoredCommittedValue);
    let facts = sponsor_capture_facts(
        CeremonyId::SponsoredCommittedValue,
        &capture,
        Some([0x44; 32]),
    );

    validate_capture_facts(&capture, &facts).expect("the committed-sponsor facts are complete");
    assert_eq!(facts.operations.len(), 6);
    assert!(
        facts
            .operations
            .iter()
            .all(|operation| operation.role == RequestRole::Auxiliary),
    );
    assert_eq!(facts.digests.len(), 1);
    assert_eq!(facts.digests[0].value, [0x44; 32]);
}

#[cfg(unix)]
#[test]
fn a_gate_panic_after_a_response_leaves_capture_and_timing() {
    let capture = scripted_capture();
    let facts = CeremonyCaptureFacts::from_capture(CeremonyId::ExplicitWitnessNegatives, &capture);
    let directory = test_directory("panic");
    let destination = test_capture_destination(&directory);
    let path = destination
        .capture_path(
            CeremonyId::ExplicitWitnessNegatives,
            CeremonyId::ExplicitWitnessNegatives.rust_test_name(),
        )
        .expect("the panic test path is derived");
    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut guard = CaptureGuard::for_test(CeremonyId::ExplicitWitnessNegatives, destination);
        write_capture_before_gates(&mut guard, &capture, &facts, "legacy\n");
        panic!("post-response gate");
    }));
    assert!(panic_result.is_err());
    let transcript = std::fs::read_to_string(&path).expect("the transcript survived the panic");
    assert!(transcript.ends_with("native-capture-end explicit-witness-negatives\n"));
    let timing =
        std::fs::read_to_string(timing_path(&path)).expect("the timing survived the panic");
    assert!(timing.contains("status failed-after-transcript\n"));
    std::fs::remove_dir_all(directory).expect("the panic test directory is removed");
}

#[test]
fn the_sha256_implementation_matches_the_published_empty_and_abc_vectors() {
    assert_eq!(
        hex_bytes(&sha256(b"")),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    assert_eq!(
        hex_bytes(&sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
}

const GOLDEN_CAPTURE_PREFIX: &str = concat!(
    "native-capture-schema 1\n",
    "ceremony-id explicit-witness-negatives\n",
    "rust-test-name 62 7468655f7769746e6573735f636f6e74656e745f6e65676174697665735f6172655f6f6666657265645f6265736964655f74686569725f636f6e74726f6c\n",
    "suite-commit 1111111111111111111111111111111111111111\n",
    "suite-tree 2222222222222222222222222222222222222222\n",
    "fixture-digest-algorithm forward-v2\n",
    "run-id-input begin\n",
    "deployment-environment development\n",
    "deployment-network-id 64 31313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131\n",
    "deployment-genesis-id 64 32323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232\n",
    "deployment-target-contract 21 656c656d656e74732d7461707363726970742d7632\n",
    "handshake-protocol-schema 7\n",
    "handshake-adapter-name 15 636170747572652d61646170746572\n",
    "handshake-adapter-version 5 312e322e33\n",
    "handshake-framework-revision 13 6672616d65776f726b2d746970\n",
    "handshake-node-name 9 656c656d656e747364\n",
    "handshake-node-version 6 32332e322e31\n",
    "handshake-binary-reported-revision 10 62696e6172792d746970\n",
    "handshake-intended-executed-tip 12 696e74656e6465642d746970\n",
    "handshake-upstream-base 13 757073747265616d2d62617365\n",
    "handshake-topic-count 2\n",
    "handshake-topic 0 7 746f7069632d61\n",
    "handshake-topic 1 7 746f7069632d62\n",
    "environment-schema 7\n",
    "environment-chain 15 656c656d656e747372656774657374\n",
    "environment-network-id 64 31313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131313131\n",
    "environment-genesis-id 64 32323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232323232\n",
    "environment-domain-count 1\n",
    "environment-domain 0 tapscript supported true active true\n",
    "environment-leaf-count 1\n",
    "environment-leaf 0 196 supported true active true\n",
    "environment-capability-count 2\n",
    "environment-capability 0 target-transaction-submission\n",
    "environment-capability 1 confidential-value-test-funding\n",
    "environment-funding-count 4\n",
    "environment-funding 0 representation explicit-asset-confidential-value\n",
    "environment-funding 1 custody central-public-fixtures\n",
    "environment-funding 2 materializer guide-ctf-deterministic-v1\n",
    "environment-funding 3 reproducibility byte_identity\n",
    "digest-count 1\n",
    "digest 0 successor forward-v2 3333333333333333333333333333333333333333333333333333333333333333\n",
);

#[cfg(unix)]
#[test]
fn the_enhanced_capture_format_matches_exact_golden_bytes() {
    let directory = test_directory("format");
    let destination = test_capture_destination(&directory);
    let capture = scripted_capture();
    let mut facts =
        CeremonyCaptureFacts::from_capture(CeremonyId::ExplicitWitnessNegatives, &capture)
            .with_digest("successor", Some([0x33; 32]));
    facts.operations[1].role = RequestRole::PairedExplicit;
    facts.operations[1].projection = Some(ProjectionInput {
        owners: vec![b"alice".to_vec(), b"bob".to_vec()],
        amounts: vec![5, 7],
        destinations: BTreeMap::from([
            (b"alice".to_vec(), (vec![0xaa, 0xbb], 11)),
            (b"bob".to_vec(), (vec![0xcc], 13)),
        ]),
    });
    let rendered = render_enhanced_capture(
        &destination,
        CeremonyId::ExplicitWitnessNegatives,
        &capture,
        &facts,
        "legacy\n",
    )
    .expect("the exhaustive capture renders");
    let expected_content = [
        GOLDEN_CAPTURE_PREFIX,
        concat!(
        "operation-count 2\n",
        "operation 0 begin\n",
        "operation-id 11 6f7065726174696f6e2d30\n",
        "request-id 9 726571756573742d30\n",
        "request-role refusal\n",
        "request-bytes 4 000180ff\n",
        "response-id 10 726573706f6e73652d30\n",
        "response-request-id 9 726571756573742d30\n",
        "response-operation-id 11 6f7065726174696f6e2d30\n",
        "response-verdict refused\n",
        "response-layer script-path-rejection\n",
        "response-target-identity none\n",
        "response-detail 14 6d7574616e742072656675736564\n",
        "attribution-control-request-id 9 726571756573742d31\n",
        "attribution-control-identity aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
        "mutation-kind empty-signature\n",
        "mutation-locator witness-item 0 0\n",
        "projection-input none\n",
        "operation 0 end\n",
        "operation 1 begin\n",
        "operation-id 11 6f7065726174696f6e2d31\n",
        "request-id 9 726571756573742d31\n",
        "request-role paired-explicit\n",
        "request-bytes 2 0203\n",
        "response-id 10 726573706f6e73652d31\n",
        "response-request-id 9 726571756573742d31\n",
        "response-operation-id 11 6f7065726174696f6e2d31\n",
        "response-verdict accepted\n",
        "response-layer accepted\n",
        "response-target-identity aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
        "response-detail 16 61636365707465642065786163746c79\n",
        "attribution-control-request-id none\n",
        "attribution-control-identity none\n",
        "mutation-kind none\n",
        "mutation-locator none\n",
        "projection-input begin\n",
        "projection-input-owner-count 2\n",
        "projection-input-owner 0 5 616c696365\n",
        "projection-input-owner 1 3 626f62\n",
        "projection-input-amount-count 2\n",
        "projection-input-amount 0 5\n",
        "projection-input-amount 1 7\n",
        "projection-destination-count 2\n",
        "projection-destination 0 2 aabb 11\n",
        "projection-destination 1 1 cc 13\n",
        "projection-input end\n",
        "operation 1 end\n",
        "legacy-rendering 7 6c65676163790a\n",
        "terminal-state complete\n",
        "run-id-input end\n",
        ),
    ]
    .concat();
    let expected = format!(
        "{expected_content}capture-content-sha256 750db245b5d6cacde581c3d4ce90add65ee5ac7b465319a37e09d7c5bcd104cc\nnative-capture-end explicit-witness-negatives\n",
    );
    assert_eq!(rendered, expected);
    let mut mismatched = facts;
    mismatched.operations[0].role = RequestRole::Acceptance;
    assert!(
        render_enhanced_capture(
            &destination,
            CeremonyId::ExplicitWitnessNegatives,
            &capture,
            &mismatched,
            "legacy\n",
        )
        .is_err(),
        "a role that disagrees with the journal verdict was rendered",
    );
    std::fs::remove_dir_all(directory).expect("the format test directory is removed");
}

#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_live_transfer_candidate_runs_against_a_real_target() {
    let mut capture_guard = CaptureGuard::new(CeremonyId::Report);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(None);

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        // The operator's declaration about the program they selected. It
        // establishes nothing about it, and this lane produces a
        // transcript rather than a gate verdict.
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::Report,
            report.as_deref(),
        )),
    );

    let mut planner = LiveTransferOperationPlanner::new().expect("the planner builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let transcript = planner.transcript();
    let rendered = render_live_native_run(transcript);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts = CeremonyCaptureFacts::from_capture(CeremonyId::Report, &capture);
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    // A refused run is written down beside the transcript rather than
    // printed: the file is the artifact, and a lane whose only record was
    // captured console output would leave nothing behind.
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A run that reached the node at all funded both constructors and
    // recorded an observation for every step it got an answer to. What
    // those answers were is written down and asserted nowhere.
    if outcome.is_ok() {
        // Every step is accounted for: it produced an observation, or the
        // plan recorded why its form was never submitted. A step that was
        // neither would be one the run quietly dropped.
        assert_eq!(
            transcript.observations().len() + transcript.not_submitted().len(),
            LiveNativeStep::ALL.len(),
            "a step was neither observed nor accounted for",
        );
        for step in LiveNativeStep::ALL {
            let submitted = transcript.observation(*step).is_some();
            let accounted = match step {
                LiveNativeStep::SubmitPrivateTransfer => transcript
                    .gap_for(linker::live_backend::LiveTransferRepresentationPlan::PrivateCommitted)
                    .is_some(),
                _ => false,
            };
            assert!(
                submitted || accounted,
                "{step:?} produced neither an observation nor a stated gap",
            );
        }
        // The deployment was welded to the chain before anything was
        // funded, so every program the ceremony paid to belongs to a
        // deployment of the asset the target issued.
        assert!(transcript.relinked(), "the run funded before it linked");
        // The ceremony materialized predecessors at both constructors, or
        // it did not reach the submissions at all — and either way the
        // transcript says which rather than reporting a smaller run.
        let materialized = transcript.materialized();
        assert_ne!(
            materialized.values().sum::<usize>(),
            0,
            "the ceremony created no predecessor at any constructor",
        );
        assert!(transcript.issued_asset().is_some());
    }

    // The run discharges nothing, and the rendering says so in its own
    // bytes rather than leaving a reader to infer it.
    assert!(rendered.contains("discharges_no_matrix_row true"));
}

/// Every line one validated record contributes to the transcript.
///
/// Split out because the census is eight members over two outputs and a
/// function that both ran a ceremony and rendered it would be two
/// functions sharing a name.
fn census_lines(
    record: &target_elements_conformance::confidential_record::ConfidentialFundingRecord,
    attempt: u8,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("run {attempt} handle {}", record.fixture_handle()));
    lines.push(format!(
        "run {attempt} contract {}",
        record.summary().contract().code()
    ));
    lines.push(format!(
        "run {attempt} parities {:?}",
        record.summary().observed_parities()
    ));
    lines.push(format!(
        "run {attempt} protocol_outputs {} non_protocol_members {}",
        record.summary().protocol_outputs(),
        record.summary().non_protocol_members()
    ));
    for census in record.agreement() {
        for entry in census.fields() {
            lines.push(format!(
                "run {attempt} output {} field {} {} {} vs {}",
                census.output(),
                entry.field(),
                if entry.agrees() { "agree" } else { "disagree" },
                entry.expectation(),
                entry.observation(),
            ));
        }
    }
    for (index, check) in record.independent_commitments().iter().enumerate() {
        lines.push(format!(
            "run {attempt} output {index} independent_commitment {} checker {}",
            if check.agrees() { "agree" } else { "disagree" },
            check.checker(),
        ));
    }
    lines.push(format!(
        "run {attempt} witness_transaction_id {}",
        record.readback().witness_transaction_id
    ));

    lines
}

/// What one confidential ceremony produced: its report lines and the
/// mined bytes a second run is compared against.
struct ConfidentialAttempt {
    lines: Vec<String>,
    mined: Vec<u8>,
    capture: NativeOperationCapture,
}

/// Runs one confidential predecessor ceremony end to end and validates it.
///
/// Split out of the test because the byte-identity comparison is a claim
/// about two runs: the test runs this twice and compares, and this
/// function knows nothing about the comparison it will be part of.
fn fund_one_confidential_predecessor(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
    binding: &target_elements::ReviewedDevelopmentBinding,
    configuration: &ExecutorConfiguration,
    attempt: u8,
    capture_guard: &mut CaptureGuard,
) -> ConfidentialAttempt {
    use target_elements_conformance::confidential_funding::{
        ConfidentialFundingOracles, ConfidentialReadbackDecoder as _,
    };
    use target_elements_conformance::confidential_oracles::{
        ReferenceRangeproofVerifier, ReferenceReadbackDecoder,
    };
    use target_elements_conformance::confidential_record::{
        ConfidentialFundingEvidence, validate_confidential_funding_record,
    };
    use vectors::confidential_predecessor::{
        AdapterReportedInclusion, ConfidentialPredecessorPlan,
    };

    let mut lines = Vec::new();
    let mut plan = ConfidentialPredecessorPlan::new();
    let (outcome, capture) = execute_and_capture(target, binding, configuration, &mut plan);
    lines.push(format!("run {attempt}"));

    let transcript =
        complete_confidential_capture(outcome, &plan, &capture, &mut lines, attempt, capture_guard);
    let written_lines = lines.len();

    let case = ConfidentialPredecessorPlan::funding_case();
    let response = transcript
        .operation_responses()
        .get(&case)
        .expect("the confidential step was answered");
    let subject = plan.subject().expect("the confidential subject was sent");
    let registry = plan.registry().expect("the fixture was registered");
    let fixture = registry
        .resolve(
            &subject.binding.fixture_handle,
            &subject.binding.fixture_digest,
        )
        .expect("the registry resolves its own case");

    let readback = response
        .mined_readback
        .as_ref()
        .expect("the answer carries a mined readback");
    let reader = ReferenceReadbackDecoder::new();
    let mined = reader
        .decode(&readback.raw_transaction)
        .expect("the mined bytes decode");
    let inclusion = AdapterReportedInclusion::recomputed_from(mined.transaction_id.clone());
    let verifier = ReferenceRangeproofVerifier::new();
    let oracles = ConfidentialFundingOracles {
        decoder: &reader,
        inclusion: &inclusion,
        rangeproofs: &verifier,
    };
    let evidence = ConfidentialFundingEvidence {
        handshake: transcript.handshake(),
        environment: transcript.environment(),
        request: subject,
        response,
        fixture,
        decoded: &mined,
        // Absent, because this funding creates no sponsor coin. A
        // program stated here would name a member the transaction does
        // not carry, and the classifier would place nothing differently
        // for it.
        sponsor_reserve_program: None,
        // Never stated here. A byte comparison is the other contract's
        // own claim, and this function has seen one run.
        materialized_bytes_compared: false,
    };
    let validated = validate_confidential_funding_record(&evidence, &oracles)
        .expect("the confidential funding record validates");
    let record = validated.record();

    lines.extend(census_lines(record, attempt));

    assert_confidential_funding_record(record);

    let facts =
        CeremonyCaptureFacts::from_capture(CeremonyId::ConfidentialPredecessorSetup, &capture);
    let validated = lines[written_lines..].join("\n") + "\n";
    write_capture_before_gates(capture_guard, &capture, &facts, &validated);

    ConfidentialAttempt {
        lines,
        mined: readback.raw_transaction.clone(),
        capture,
    }
}

fn complete_confidential_capture(
    outcome: Result<
        target_elements_conformance::executor::ExecutionTranscript,
        target_elements_conformance::error::NativeConformanceError,
    >,
    plan: &vectors::confidential_predecessor::ConfidentialPredecessorPlan,
    capture: &NativeOperationCapture,
    lines: &mut Vec<String>,
    attempt: u8,
    capture_guard: &mut CaptureGuard,
) -> target_elements_conformance::executor::ExecutionTranscript {
    // A stopped result is a valid outcome and is written as one. It is
    // an infrastructure or construction fact and never a target verdict.
    match outcome {
        Ok(transcript) => {
            let facts = CeremonyCaptureFacts::from_capture(
                CeremonyId::ConfidentialPredecessorSetup,
                capture,
            );
            let prelude = lines.join("\n") + "\n";
            write_capture_before_gates(capture_guard, capture, &facts, &prelude);
            transcript
        }
        Err(error) => {
            lines.push(format!("run {attempt} executor_refused {error}"));
            if let Some(refusal) = plan.refusal() {
                lines.push(format!("run {attempt} plan_refused {refusal:?}"));
            }
            let facts = CeremonyCaptureFacts::from_capture(
                CeremonyId::ConfidentialPredecessorSetup,
                capture,
            );
            let refused = lines.join("\n") + "\n";
            write_capture_before_gates(capture_guard, capture, &facts, &refused);
            panic!("the confidential predecessor ceremony did not reach the target: {error}");
        }
    }
}

fn assert_confidential_funding_record(
    record: &target_elements_conformance::confidential_record::ConfidentialFundingRecord,
) {
    use target_elements_conformance::confidential_record::{FieldAgreement, FundingAgreementField};

    assert_eq!(record.agreement().len(), 2);
    for census in record.agreement() {
        assert_eq!(census.fields().len(), FundingAgreementField::ALL.len());
        assert!(census.fields().iter().all(FieldAgreement::agrees));
    }
    assert_eq!(record.summary().observed_parities(), &[0x08, 0x09]);
    assert_eq!(record.non_claims().len(), 10);
}

/// One confidential predecessor, funded, mined, and read back.
///
/// # What this run establishes, and the six things it reports
///
/// Confidential funding capability and schema negotiation; deterministic
/// materialization under the selected custody profile; the exact hybrid
/// representation of both mined outputs; both accepted commitment
/// parities; valid proof-bearing predecessor outputs and target
/// readback; and a stable opening reference for a later transaction-wide
/// materializer. It reports no transfer, no authorization, no CT
/// conservation, no acceptance of any safety row, no minimality, no
/// production privacy, and no matrix discharge.
///
/// # Why it runs the ceremony twice
///
/// The selected reproducibility contract is byte identity, and byte
/// identity is a claim about two runs rather than about one. A single
/// run that reported deterministic bytes would be reporting a property
/// it never observed, so the ceremony runs twice against two disposable
/// chains and the funding transactions are compared byte for byte. A
/// difference is written down as a difference; it is never a downgrade
/// to the other contract.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_confidential_predecessor_is_funded_mined_and_read_back() {
    let mut capture_guard = CaptureGuard::new(CeremonyId::ConfidentialPredecessorSetup);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("confidential-predecessor"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");
    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::ConfidentialPredecessorSetup,
            report.as_deref(),
        )),
    );

    let first =
        fund_one_confidential_predecessor(&target, &binding, &configuration, 0, &mut capture_guard);
    let second =
        fund_one_confidential_predecessor(&target, &binding, &configuration, 1, &mut capture_guard);

    let identical = first.mined == second.mined;
    let mut lines = first.lines;
    lines.extend(second.lines);
    let summary = [
        format!("byte_identity_satisfied {identical}"),
        "discharges_no_matrix_row true".to_owned(),
        "clears_owner_sighash_blockers false".to_owned(),
    ];
    lines.extend(summary.iter().cloned());
    let rendered = lines.join("\n") + "\n";
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
    }
    let facts = CeremonyCaptureFacts::from_capture(
        CeremonyId::ConfidentialPredecessorSetup,
        &second.capture,
    );
    let enhanced_summary = summary.join("\n") + "\n";
    write_capture_before_gates(
        &mut capture_guard,
        &second.capture,
        &facts,
        &enhanced_summary,
    );

    assert!(
        identical,
        "the run selected byte identity and did not achieve it",
    );
}

/// One owner authorization, produced against the Wave-2 message and
/// observed on the explicit lane.
///
/// # What this run is for
///
/// The accepted evidence ruling names three things a dimension needs: a
/// source citation, an independent recomputation, and one observed
/// acceptance. The first two are landed. This is the third, and it is
/// the only thing in this file that submits a candidate carrying a
/// signature rather than bytes that authorize nothing.
///
/// # What it asserts, and what it merely records
///
/// It asserts the shape of a completed ceremony and binds all seven
/// answers to T5-026's run of record: each case name, each target layer,
/// whether an accepted identity was present, and the selected case's
/// accepted and reverified identity. A changed answer is still written
/// into the artifact first, and then the lane fails closed pending owner
/// review rather than silently advancing the historical record.
///
/// The two-origin agreement is unconditional for the selected case. A
/// run that accepted a candidate and then could not verify its read-back
/// witness against the independently recomputed message has found
/// something, and it must say so by failing rather than by writing a
/// false line.
///
/// # It clears nothing by running
///
/// A blocker moves on an observed result and never on a capability
/// existing. This test existing establishes nothing at all; what
/// establishes anything is the artifact one run of it produced.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_owner_authorization_is_observed_on_the_explicit_lane() {
    use vectors::live_owner_observation::{
        OwnerObservationCase, OwnerObservationPlanner, render_owner_observation,
    };

    let mut capture_guard = CaptureGuard::new(CeremonyId::OwnerObservation);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("owner-observation"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::OwnerObservation,
            report.as_deref(),
        )),
    );

    // The genesis block hash the message hasher is seeded with, taken
    // from the run's own deployment binding rather than from anything a
    // candidate carries: no candidate determines it, and two identical
    // candidates on two chains have different messages.
    let mut planner =
        OwnerObservationPlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_owner_observation(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts = CeremonyCaptureFacts::from_capture(CeremonyId::OwnerObservation, &capture);
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    outcome.expect("the ceremony reached the target");
    assert_owner_observation_matches_run_of_record(record);

    // Every case was submitted and answered. A case that was neither is
    // one the ceremony quietly dropped, and a run that dropped a control
    // would be reporting a narrower comparison than it claims.
    assert_eq!(
        record.observations().len(),
        OwnerObservationCase::ALL.len(),
        "a case was not submitted",
    );
    assert!(record.relinked(), "the ceremony funded before it linked");
    assert_eq!(record.coins().len(), 2);

    // The signed-over spent-output triple is the node's own report of
    // the coins, and the ceremony's expectation agreed with it. A
    // disagreement is a finding about the funding boundary rather than
    // about the message, and it must not pass silently.
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_owner_observation::ObservedFundedCoin::matches_expectation),
        "the node reported a coin the ceremony did not ask for",
    );

    // Every case's message was computed, and the empty-vector control's
    // message differs from the selected profile's. Two candidates that
    // coincided would make any verdict about which one a signature
    // verifies against a coincidence.
    let messages = record.candidate_messages();
    assert_eq!(messages.len(), OwnerObservationCase::ALL.len());
    assert_ne!(
        messages.get(&OwnerObservationCase::SelectedProfile),
        messages.get(&OwnerObservationCase::EmptyOutputWitnessVector),
        "the two candidate messages coincided",
    );

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("establishes_the_proof_bearing_lane false"));
    assert!(rendered.contains("clears_sighash_profile_unreviewed false"));
    assert!(rendered.contains("discharges_no_matrix_row true"));
}

fn current_native_v2_corpus()
-> &'static vectors::live_corpus_native_v2_r7::ValidatedNativeV2R7Corpus {
    vectors::live_corpus_native_v2_r7::run_of_record()
        .expect("the reviewed native-v2/revision-7 corpus validates")
}

fn current_ceremony_projection(
    ceremony: &str,
) -> &'static vectors::live_corpus_native_v2_r7::NativeV2CeremonyProjection {
    current_native_v2_corpus()
        .ceremony_projection(ceremony)
        .unwrap_or_else(|| panic!("the current corpus projects {ceremony}"))
}

fn current_outcome_projections(
    ceremony: &str,
) -> &'static [vectors::live_corpus_native_v2_r7::NativeV2OutcomeProjection] {
    current_native_v2_corpus()
        .outcome_projections(ceremony)
        .unwrap_or_else(|| panic!("the current corpus projects outcomes for {ceremony}"))
}

fn current_acceptance_projections(
    ceremony: &str,
) -> &'static [vectors::live_corpus_native_v2_r7::NativeV2AcceptanceProjection] {
    current_native_v2_corpus()
        .acceptance_projections(ceremony)
        .unwrap_or_else(|| panic!("the current corpus projects acceptances for {ceremony}"))
}

fn sole_current_acceptance(
    ceremony: &str,
) -> &'static vectors::live_corpus_native_v2_r7::NativeV2AcceptanceProjection {
    let [acceptance] = current_acceptance_projections(ceremony) else {
        panic!("the current {ceremony} ceremony has one acceptance")
    };
    acceptance
}

fn current_accepted_outcome(
    ceremony: &str,
) -> &'static vectors::live_corpus_native_v2_r7::NativeV2OutcomeProjection {
    let acceptance = sole_current_acceptance(ceremony);
    current_outcome_projections(ceremony)
        .iter()
        .find(|outcome| outcome.target_identity() == Some(acceptance.identity()))
        .unwrap_or_else(|| panic!("the current {ceremony} acceptance has its typed outcome"))
}

fn current_outcome_for_mutant(
    ceremony: &str,
    kind: LiveMutantKind,
) -> &'static vectors::live_corpus_native_v2_r7::NativeV2OutcomeProjection {
    current_outcome_projections(ceremony)
        .iter()
        .find(|outcome| outcome.mutant_kind() == Some(kind))
        .unwrap_or_else(|| panic!("the current {ceremony} ceremony carries {}", kind.row()))
}

fn current_semantic_value(ceremony: &str, field: &str) -> &'static str {
    current_ceremony_projection(ceremony)
        .semantic_value(field)
        .unwrap_or_else(|| panic!("the current {ceremony} ceremony carries {field}"))
}

fn current_semantic_u64(ceremony: &str, field: &str) -> u64 {
    current_semantic_value(ceremony, field)
        .parse()
        .unwrap_or_else(|_| panic!("the current {ceremony} field {field} is an integer"))
}

fn current_semantic_line(ceremony: &str, prefix: &str) -> &'static str {
    let rendering = std::str::from_utf8(current_ceremony_projection(ceremony).semantic_rendering())
        .expect("the validated semantic rendering is UTF-8");
    let mut matches = rendering.lines().filter(|line| line.starts_with(prefix));
    let line = matches
        .next()
        .unwrap_or_else(|| panic!("the current {ceremony} ceremony carries {prefix}"));
    assert!(
        matches.next().is_none(),
        "the current {ceremony} ceremony carries {prefix} once",
    );
    line
}

fn current_semantic_line_value(ceremony: &str, prefix: &str, field: &str) -> &'static str {
    current_semantic_line(ceremony, prefix)
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|pair| match pair {
            [candidate, value] if *candidate == field => Some(*value),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the current {ceremony} line carries {field}"))
}

fn current_semantic_line_u64(ceremony: &str, prefix: &str, field: &str) -> u64 {
    current_semantic_line_value(ceremony, prefix, field)
        .parse()
        .unwrap_or_else(|_| panic!("the current {ceremony} line's {field} is an integer"))
}

fn current_transaction(bytes: &[u8]) -> transaction::bytes::TargetTransaction {
    transaction::bytes::TargetTransaction::decode(bytes)
        .expect("validated current-corpus transaction bytes decode")
}

/// Bind every T5-026 answer after the transcript has been written.
fn assert_owner_observation_matches_run_of_record(
    record: &vectors::live_owner_observation::OwnerObservationRecord,
) {
    use vectors::live_owner_observation::OwnerObservationCase;

    let expected = current_outcome_projections("owner-observation");
    assert_eq!(
        record.observations().len(),
        expected.len(),
        "the ceremony did not answer the full run-of-record census",
    );
    assert_eq!(expected.len(), OwnerObservationCase::ALL.len());

    let mut expected_reverification_identity = None;
    for ((observation, case), expected) in record
        .observations()
        .iter()
        .zip(OwnerObservationCase::ALL)
        .zip(expected)
    {
        let name = case.name();
        assert_eq!(observation.case(), *case, "the recorded case order drifted");
        assert_eq!(
            observation.layer(),
            expected.layer(),
            "{name} changed layer"
        );
        assert_eq!(
            observation.detail(),
            (!expected.detail().is_empty()).then_some(expected.detail()),
            "{name} drew different target words",
        );
        assert_eq!(
            observation.submitted_bytes(),
            expected.submitted_bytes().len(),
            "{name} submitted different bytes",
        );
        let identity = expected
            .target_identity()
            .map(|identity| identity.to_target_display());
        assert_eq!(
            observation.accepted_txid(),
            identity.as_deref(),
            "{name} changed accepted identity",
        );
        if let Some(identity) = identity {
            assert_eq!(*case, OwnerObservationCase::SelectedProfile);
            assert!(
                expected_reverification_identity.replace(identity).is_none(),
                "the current corpus names more than one reverification",
            );
        }
    }

    // Checked UNCONDITIONALLY. The selected case is asserted Accepted
    // above, so an absent second-origin record is itself a binding failure.
    let expected_reverification_identity = expected_reverification_identity
        .expect("the selected case names its recorded reverification identity");
    let check = record
        .reverification()
        .expect("the selected acceptance carries its two-origin readback check");
    assert_eq!(
        check.accepted_txid(),
        expected_reverification_identity.as_str(),
        "the reverification named a different accepted transaction",
    );
    assert!(
        check.readback_matches_submission(),
        "the bytes the node reported are not the bytes it was handed",
    );
    assert!(
        check.verified().is_ok(),
        "the accepted witness does not verify against the recomputed message: {:?}",
        check.verified(),
    );
    assert!(
        !check.verifies_against_empty_vector_message(),
        "the accepted witness verifies against both candidate messages",
    );
}

/// One owner authorization observed on the PROOF-BEARING lane.
///
/// # What this run is for
///
/// The explicit-lane observation was forbidden to say anything about a
/// candidate whose outputs carry range proofs, because the term that
/// makes an explicit candidate authorizable from a preimage — an
/// output-witness vector recoverable from the output count — has no
/// counterpart there. This asks the same question where the vector
/// carries real proofs.
///
/// # What belongs to the other guide
///
/// The confidential predecessor is funded by that guide's own ceremony
/// machinery and adapter, and the candidate is frozen by that guide's
/// transaction-wide materializer. Neither is evidenced here, and the
/// record says so in its own bytes rather than leaving it to this
/// comment.
///
/// # What it asserts, and what it merely records
///
/// The shape of a completed ceremony: every case submitted and
/// answered, the vector at its real proof-bearing length, both
/// construction controls refused before any message was formed, and the
/// two-origin agreement where an acceptance was observed. No schema-1
/// layer or acceptance is compared with this fresh run; once schema 2 is
/// recorded, its own exact observation and acceptance members are the
/// comparison.
///
/// The schema-1 record remains immutable historical-v1 data and validates
/// only its own archived bytes. The live fixture-digest algorithm is v2,
/// so this fresh run projects only to schema 2; its forward observation
/// and acceptance members remain pending until their own owner-authorized
/// accepted run is recorded.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_owner_authorization_is_observed_on_the_proof_bearing_lane() {
    use vectors::live_proof_bearing_observation::{
        ProofBearingObservationPlanner, forward_v2_proof_bearing_run_of_record,
        render_proof_bearing_observation,
    };

    let mut capture_guard = CaptureGuard::new(CeremonyId::ProofBearingObservation);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("proof-bearing-observation"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::ProofBearingObservation,
            report.as_deref(),
        )),
    );

    let mut planner =
        ProofBearingObservationPlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_proof_bearing_observation(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts = CeremonyCaptureFacts::from_capture(CeremonyId::ProofBearingObservation, &capture)
        .with_digest("predecessor", record.predecessor_digest().copied());
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    outcome.expect("the ceremony reached the target");

    assert_live_construction_refusal_shape(record);
    assert_proof_bearing_record_matches_forward_run_of_record(
        record,
        forward_v2_proof_bearing_run_of_record(),
    );
    assert!(rendered.contains("forward_v2_run_of_record"));
    assert!(rendered.contains("run_of_record_projection ready schema_version 2"));
    assert!(!rendered.contains("run_of_record_v2 recorded"));
    check_proof_bearing_record(record, &rendered);
}

#[test]
fn a_fresh_v2_projection_refuses_schema_one_before_node_execution() {
    use vectors::live_proof_bearing_observation::{
        ProofBearingObservationPlanner, ProofBearingRunOfRecord, RunOfRecordProjectionRefusal,
    };

    let planner = ProofBearingObservationPlanner::new([0_u8; 32])
        .expect("the node-free forward-v2 planner builds");

    assert_eq!(
        ProofBearingRunOfRecord::try_from(planner.record()),
        Err(RunOfRecordProjectionRefusal::HistoricalV1DigestRequired),
    );
}

/// Check every live construction refusal without consulting schema 1.
///
/// Every refusal must project to the stable vocabulary, the controls
/// must be exactly the closed control census in order, every full
/// refusal must name the first consumed coin, and no historical bytes
/// participate in the comparison.
///
/// # Panics
///
/// If the completed live record omits a coin or control, carries an
/// unrecognized refusal, or names the wrong consumed coin.
fn assert_live_construction_refusal_shape(
    record: &vectors::live_proof_bearing_observation::ProofBearingObservationRecord,
) {
    use transaction::live_materialize::MaterializationRefusal;
    use vectors::live_proof_bearing_observation::{
        ProofBearingConstructionControl, RecordedProofBearingConstructionRefusal,
    };

    let projected = record
        .construction_refusals()
        .iter()
        .map(RecordedProofBearingConstructionRefusal::try_from)
        .collect::<Result<Vec<_>, _>>()
        .expect("every live construction refusal has a stable archival reason");
    let controls: Vec<_> = projected
        .iter()
        .map(RecordedProofBearingConstructionRefusal::control)
        .collect();
    assert_eq!(
        controls.as_slice(),
        ProofBearingConstructionControl::ALL,
        "the live construction-control census drifted",
    );

    let first_outpoint = record
        .coins()
        .first()
        .expect("the completed ceremony consumed a predecessor coin")
        .outpoint();
    for refusal in record.construction_refusals() {
        let MaterializationRefusal::PredecessorOpeningMismatch { outpoint } = refusal.refusal()
        else {
            panic!("a construction control drew an unrecognized live refusal");
        };
        assert_eq!(
            *outpoint, first_outpoint,
            "a construction refusal names a coin other than the first consumed coin",
        );
    }
}

/// Bind a fresh live projection only to the schema-2 forward record.
///
/// # Panics
///
/// If the live record cannot project as forward v2, the fresh projection
/// omits either member, the expected members have mixed states, or a
/// recorded forward member differs from the fresh value.
fn assert_proof_bearing_record_matches_forward_run_of_record(
    actual: &vectors::live_proof_bearing_observation::ProofBearingObservationRecord,
    expected: &vectors::live_proof_bearing_observation::ForwardV2ProofBearingRunOfRecord,
) {
    use vectors::live_proof_bearing_observation::{
        ForwardProofBearingRecordMember::{Pending, Recorded},
        ForwardV2ProofBearingRunOfRecord,
    };

    let projected = ForwardV2ProofBearingRunOfRecord::try_from(actual)
        .expect("the completed live ceremony projects to forward schema 2");
    assert_eq!(projected.schema_version(), expected.schema_version());
    assert_eq!(
        projected.fixture_digest_algorithm(),
        expected.fixture_digest_algorithm(),
    );
    let (projected_observations, projected_acceptance) =
        match (projected.observations(), projected.acceptance()) {
            (Recorded(observations), Recorded(acceptance)) => (observations, acceptance),
            (Pending, Pending) => {
                panic!("a completed forward projection must record both members")
            }
            (Pending, Recorded(_)) => {
                panic!("a completed forward projection must record its observations")
            }
            (Recorded(_), Pending) => {
                panic!("a completed forward projection must record its acceptance")
            }
        };

    match (expected.observations(), expected.acceptance()) {
        (Pending, Pending) => {
            // The caller has already rendered and preserved the fresh record. A
            // successful `try_from` above proves both member censuses and the
            // acceptance/reverification identity agree. That complete schema-2
            // projection is candidate mint material while no owner-authorized
            // forward record exists; Pending is not an equality expectation.
        }
        (Recorded(expected_observations), Recorded(expected_acceptance)) => {
            assert_eq!(
                projected_observations, expected_observations,
                "the fresh observations drifted from the forward-v2 record",
            );
            assert_eq!(
                projected_acceptance, expected_acceptance,
                "the fresh acceptance drifted from the forward-v2 record",
            );
        }
        (Pending, Recorded(_)) | (Recorded(_), Pending) => {
            panic!("the forward-v2 record carries mixed Pending and Recorded members")
        }
    }
}

/// Everything the completed proof-bearing ceremony owes its reader.
///
/// Split from the test body because the run's setup and the run's
/// checks are two different readings, and a body that outgrew a hundred
/// lines is one nobody reviews as a whole. Nothing moved into here
/// decides what the target should have found: every assertion is about
/// the SHAPE of a completed ceremony, and the one content assertion is
/// the two-origin agreement.
fn check_proof_bearing_record(
    record: &vectors::live_proof_bearing_observation::ProofBearingObservationRecord,
    rendered: &str,
) {
    use vectors::live_proof_bearing_observation::{
        ProofBearingCase, ProofBearingConstructionControl,
    };

    // Every case was submitted and answered, and both construction
    // controls fired. A control that quietly did not run is a narrower
    // comparison than the report claims.
    assert_eq!(
        record.observations().len(),
        ProofBearingCase::ALL.len(),
        "a case was not submitted",
    );
    assert_eq!(
        record.construction_refusals().len(),
        ProofBearingConstructionControl::ALL.len(),
        "a construction control did not refuse",
    );

    // The predecessor is confidential and the node's report of it is
    // what the ceremony asked for. A divergence is a finding about the
    // funding boundary rather than about the message.
    assert_eq!(record.coins().len(), 2);
    assert!(
        record.coins().iter().all(
            vectors::live_proof_bearing_observation::ObservedConfidentialCoin::matches_expectation
        ),
        "the node reported a confidential coin the ceremony did not ask for",
    );

    // The deliverable's own figure: the census was built at the vector's
    // REAL proof-bearing length, and every entry carries a proof rather
    // than the two zero bytes the explicit lane's entries carry.
    assert_eq!(record.output_witness_vector_length(), Some(2));
    assert_eq!(record.output_witness_proof_bytes().len(), 2);
    assert!(
        record
            .output_witness_proof_bytes()
            .iter()
            .all(|bytes| *bytes > 2),
        "an output-witness entry carried no range proof: {:?}",
        record.output_witness_proof_bytes(),
    );

    // Every case's message was computed, and the two witness-vector
    // controls' messages differ from the selected profile's and from
    // each other. Candidates that coincided would make any verdict about
    // which one a signature verifies against a coincidence.
    let messages = record.candidate_messages();
    assert_eq!(messages.len(), ProofBearingCase::ALL.len());
    let selected = messages.get(&ProofBearingCase::SelectedProfile);
    assert_ne!(
        selected,
        messages.get(&ProofBearingCase::ProofBearingVectorEmptied),
    );
    assert_ne!(
        selected,
        messages.get(&ProofBearingCase::PreimageOnlySigner)
    );
    assert_ne!(
        messages.get(&ProofBearingCase::ProofBearingVectorEmptied),
        messages.get(&ProofBearingCase::PreimageOnlySigner),
    );
    assert_ne!(
        selected,
        messages.get(&ProofBearingCase::AnotherProofBearingCandidate),
    );

    // The two origins, where an acceptance was observed.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.verified().is_ok(),
            "the accepted witness does not verify against the recomputed message: {:?}",
            check.verified(),
        );
        assert!(
            !check.verifies_against_emptied_vector_message(),
            "the accepted witness verifies against both candidate messages",
        );
    }

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_the_other_guides_funding false"));
    assert!(rendered.contains("evidences_the_other_guides_materialization false"));
    assert!(rendered.contains("evidences_the_other_guides_blinding false"));
    assert!(rendered.contains("evidences_the_receipt_covenant false"));
    assert!(rendered.contains("discharges_no_matrix_row true"));
}

// Bind the freshly written private-restart transcript to the forward-v2
// fixture selector and its recorded acceptance. T7-010 minted the link from
// the validated current corpus, so no historical fallback remains honest or
// reachable here.
fn assert_private_restart_matches_the_run_of_record(
    record: &vectors::live_private_restart::PrivateRestartRecord,
    consumed: vectors::live_private_restart::ConsumedReceipt,
) {
    use vectors::live_private_restart::forward_v2::{
        ForwardPrivateRestartExpectation, forward_private_restart_expectation,
    };

    let ceremony = match consumed {
        vectors::live_private_restart::ConsumedReceipt::Primary => "private-restart-control",
        vectors::live_private_restart::ConsumedReceipt::Balancing => "private-restart-parity",
    };
    let current_outcome = current_accepted_outcome(ceremony);
    let ForwardPrivateRestartExpectation::V2(forward) = forward_private_restart_expectation();
    assert_eq!(
        record.predecessor_digest(),
        Some(identifier(forward.fixtures().predecessor())),
    );
    assert_eq!(
        record.successor_digest(),
        Some(identifier(forward.fixtures().successor(consumed))),
    );
    assert_eq!(record.consumed_receipt(), Some(consumed.name()));
    assert_eq!(record.observed_layer(), Some(current_outcome.layer()));
    assert!(record.produced_an_accepted_control());

    let link = forward
        .acceptance()
        .recorded_link()
        .expect("T7-010 records the corpus-backed private-restart link");
    let member = link.for_receipt(consumed);
    assert_eq!(
        record.consumed_commitment_prefix(),
        Some(member.commitment_prefix()),
    );
    let expected_identity = member.identity();
    let expected_identity = expected_identity.to_string();
    let current_identity = current_outcome
        .target_identity()
        .map(|identity| identity.to_target_display());
    assert_eq!(
        current_identity.as_deref(),
        Some(expected_identity.as_str())
    );
    assert_eq!(record.accepted_txid(), Some(expected_identity.as_str()));

    let reverification = record
        .reverification()
        .expect("the accepted run carries unconditional reverification");
    assert_eq!(reverification.accepted_txid(), expected_identity.as_str());
    assert!(reverification.readback_matches_submission());
    assert!(reverification.verified());
}

/// The restart order's first step, against a real node.
///
/// # What this run is for
///
/// One accepted sponsorless private one-to-one control, which is the
/// entry condition for every later step of the mandatory restart order
/// (task:guide-ctf-exec:restart-order). Nothing else runs here: no
/// negative case, no mutation, no parity pair, because the order forbids
/// them until this one accepts.
///
/// # What it asserts, after preserving the fresh record
///
/// It writes the transcript, timing, and any executor refusal first, then
/// asserts the forward fixture digests, selected receipt, accepted layer,
/// receipt-indexed identity, and unconditional readback reverification.
/// The surrounding shape checks require two matching predecessor coins,
/// proof-bearing outputs, a submitted candidate, and an admitted parity.
/// A changed honest answer remains preserved in the artifacts while this
/// reproduction gate fails.
///
/// Superseding a changed result requires an explicit decision recorded
/// as a new forward run of record. This test never rewrites the
/// historical constants to accommodate drift.
///
/// Historical v1 fixture digests remain immutable and renderable, but the
/// sole live fixture-digest algorithm is v2. Fresh digest comparisons use
/// only the forward expectation; its acceptance pair remains pending its
/// own owner-authorized run, so the pending bridge checks only the matching
/// Primary or Balancing transaction identity whose deterministic bytes do
/// not depend on how the fixture is named.
///
/// # It moves nothing by running
///
/// A row moves on an observed acceptance, and the observation is the
/// artifact this produces rather than the existence of this test.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_private_one_to_one_control_is_submitted_to_a_real_target() {
    use vectors::live_private_restart::ConsumedReceipt;

    run_one_private_control(ConsumedReceipt::Primary, "private-restart-control");
}

/// The restart order's second step: the other predecessor commitment
/// parity, in a complete accepted successor.
///
/// # What this run is for, and why it is a second run
///
/// Step two asks for both predecessor commitment parities exercised in
/// complete accepted successors. The two parities are carried by the
/// predecessor's two outputs, so the honest way to exercise both is to
/// consume each of them in its own complete successor rather than to
/// assert that a candidate touching both must have covered them.
///
/// Its entry condition is step one's observed acceptance, which is why
/// it is a separate test and not a loop: a lane that ran both and
/// reported one number could not say which of them the order was
/// entitled to.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_other_commitment_parity_is_exercised_in_a_complete_successor() {
    use vectors::live_private_restart::ConsumedReceipt;

    run_one_private_control(ConsumedReceipt::Balancing, "private-restart-parity");
}

/// One private control, consuming one named predecessor output.
fn run_one_private_control(
    consumed: vectors::live_private_restart::ConsumedReceipt,
    extension: &str,
) {
    use vectors::live_private_restart::{PrivateRestartPlanner, render_private_restart};

    let ceremony = match extension {
        "private-restart-control" => CeremonyId::PrivateRestartControl,
        "private-restart-parity" => CeremonyId::PrivateRestartParity,
        _ => panic!("unknown private-restart ceremony"),
    };
    let mut capture_guard = CaptureGuard::new(ceremony);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some(extension));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(ceremony, report.as_deref())),
    );

    let mut planner = PrivateRestartPlanner::spending(identifier(&genesis), consumed)
        .expect("the restart ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_private_restart(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts = CeremonyCaptureFacts::from_capture(ceremony, &capture)
        .with_digest("predecessor", record.predecessor_digest())
        .with_digest("successor", record.successor_digest());
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as
    // one. It is never a target verdict, so it is reported and the test
    // stops here rather than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the restart ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    assert_private_control(record, consumed, &rendered);
}

fn assert_private_control(
    record: &vectors::live_private_restart::PrivateRestartRecord,
    consumed: vectors::live_private_restart::ConsumedReceipt,
    rendered: &str,
) {
    assert_private_restart_matches_the_run_of_record(record, consumed);

    // The predecessor is confidential and is the one the ceremony asked
    // for. A divergence is a finding about the funding boundary.
    assert_eq!(record.coins().len(), 2);
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_private_restart::RestartConfidentialCoin::matches_expectation),
        "the node reported a confidential coin the ceremony did not ask for",
    );

    // One receipt consumed, and its outputs carry real proofs.
    assert_eq!(record.receipt_leaves(), 1);
    assert_eq!(record.output_witness_proof_bytes().len(), 2);
    assert!(
        record
            .output_witness_proof_bytes()
            .iter()
            .all(|bytes| *bytes > 2),
        "an output-witness entry carried no range proof: {:?}",
        record.output_witness_proof_bytes(),
    );

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    // The two origins, where an acceptance was observed.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.verified(),
            "the accepted witness does not verify against the recomputed message",
        );
    }

    // The parity this run exercised is the node's answer about the coin
    // it consumed, and it is one of the two the target admits.
    let prefix = record
        .consumed_commitment_prefix()
        .expect("a consumed confidential coin carries a commitment prefix");
    assert!(
        prefix == 0x08 || prefix == 0x09,
        "the node reported a commitment prefix outside the admitted pair: {prefix:#04x}",
    );

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_no_negative_case true"));
    assert!(rendered.contains("moves_the_sponsor_row false"));
}

/// The restart order's third and fourth steps: target CT conservation
/// recorded against a balance-valid control, and the three proof-negatives
/// run from that control.
///
/// # What this run is for
///
/// Step three records the target's own commitment-balance rule accepting a
/// conserving private transaction, and step four mutates one field of that
/// same control, one case at a time, and observes the target refuse each.
/// The two are one ceremony because they share a control: the control step
/// three records the conservation of is the control step four mutates.
///
/// # What it asserts, and why the standing changed
///
/// This run's verdicts are RECORDED as run-of-record constants, and a
/// record nothing checks can drift silently — the standing an adversarial
/// review found this lane resting on. So every verdict this ceremony's
/// constants carry is now ASSERTED against them: the control's ACCEPTANCE,
/// typed identity, submitted size and consumed prefix; unconditional
/// readback reverification; and every mutant's case, layer, verbatim detail
/// and typed located range against the same balance-valid control.
///
/// The transcript and wall time are written to disk BEFORE the binding
/// runs, so a changed honest answer is preserved in the artifacts while
/// the lane fails instead of passing over drift. Wall time itself remains
/// machine telemetry rather than a reproducible run-of-record result.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn conservation_is_recorded_against_a_control_the_proof_negatives_mutate() {
    use vectors::live_conservation_negatives::{
        ConservationNegativePlanner, assert_conservation_matches_the_run_of_record,
        render_conservation_negatives,
    };

    let mut capture_guard = CaptureGuard::new(CeremonyId::ConservationNegatives);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("conservation-negatives"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::ConservationNegatives,
            report.as_deref(),
        )),
    );

    let mut planner =
        ConservationNegativePlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_conservation_negatives(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let mut facts = CeremonyCaptureFacts::from_capture(CeremonyId::ConservationNegatives, &capture)
        .with_digest("predecessor", record.predecessor_digest())
        .with_digest("successor", record.successor_digest());
    for mutant in record.mutants() {
        facts = facts.with_locator(
            mutant.case().name(),
            LiveMutationLocator::SerializedOutputField(mutant.capture_locator()),
        );
    }
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as one.
    // It is never a target verdict, so it is reported and the test stops
    // here rather than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the conservation ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    assert_conservation_matches_the_run_of_record(record);

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("moves_the_sponsor_row false"));
}

/// One owner-signing script-path negative, signed over its own mutated
/// bytes and refused at the leaf's own clause.
///
/// # What this run is for
///
/// The `second-offsetting-u-flow` row drives a candidate that consensus
/// has no reason to refuse: its added input-and-output pair offsets
/// exactly, so the per-asset sum still closes and the covenant's own
/// cardinality clause is what answers.
///
/// # Why the assertions are first-party only
///
/// The frozen run of record predates this ceremony and carries no outcome
/// to bind it to, and the lookup that fetches one panics rather than
/// returning nothing. So the gate here checks what the ceremony itself
/// establishes — the shapes it built, that the wider candidate was not
/// accepted, and that the narrower control was — and the binding to
/// recorded words arrives with the capture that observes it. The capture
/// is written BEFORE any of it runs, so a failure still leaves a
/// diagnosable artifact.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_offsetting_flow_is_refused_before_the_narrower_control_is_accepted() {
    use vectors::live_offsetting_flow_negatives::{
        CONTROL_STEP, MUTANT_STEP, OffsettingFlowNegativePlanner, render_offsetting_flow_negatives,
    };

    let mut capture_guard = CaptureGuard::new(CeremonyId::OffsettingFlowNegatives);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("offsetting-flow-negatives"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::OffsettingFlowNegatives,
            report.as_deref(),
        )),
    );

    let mut planner =
        OffsettingFlowNegativePlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_offsetting_flow_negatives(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let mut facts =
        CeremonyCaptureFacts::from_capture(CeremonyId::OffsettingFlowNegatives, &capture);
    for operation in capture.operations() {
        let step = operation.request().case.step.as_str();
        if let Some(locator) = record.capture_locator(step) {
            facts = facts.with_locator(step, locator);
        }
    }
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as one.
    // It is never a target verdict, so it is reported and the test stops
    // here rather than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the offsetting-flow ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    assert!(record.relinked(), "the ceremony funded before it linked");
    assert_eq!(
        record.coins().len(),
        3,
        "the ceremony did not fund the coin the added flow spends",
    );
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_owner_observation::ObservedFundedCoin::matches_expectation),
        "the node reported a coin the ceremony did not ask for",
    );

    let mutant = record.mutant().expect("the offsetting flow was built");
    assert_eq!(
        mutant.control_shape(),
        (2, 2),
        "the control is not the two-in two-out successor",
    );
    assert_eq!(
        mutant.mutant_shape(),
        (3, 3),
        "the mutant is not one balanced flow wider than the control",
    );

    // The weakest claim that still fails a broken drive: a negative row
    // whose candidate is ACCEPTED has driven nothing. Which clause refuses
    // it is deliberately not named — the equality failure is the covenant
    // fragment's own and reads the same for any count fault.
    assert_ne!(
        mutant.observed_layer(),
        Some(ObservedOutcomeLayer::Accepted),
        "the offsetting flow was accepted, so it drove nothing",
    );

    let control = record.control().expect("the control was submitted");
    assert_eq!(
        control.observed_layer(),
        Some(ObservedOutcomeLayer::Accepted),
        "the control was not accepted, so the mutant's refusal separates nothing",
    );

    // The separating fact is the shape, recorded as the locator the import
    // will read. The control step carries none, because it is not a
    // mutation and a locator on an acceptance would claim a fault.
    assert_eq!(
        record.capture_locator(MUTANT_STEP),
        Some(LiveMutationLocator::TransactionShape {
            control_inputs: 2,
            mutant_inputs: 3,
            control_outputs: 2,
            mutant_outputs: 3,
        }),
        "the offsetting flow did not declare its shape locator",
    );
    assert_eq!(
        record.capture_locator(CONTROL_STEP),
        None,
        "the control declared a mutation locator",
    );
}

/// The `vault-control-entitlement-or-bare-u-output` row declares a
/// script-path refusal, and a mutant with a stale signature would die at
/// the signature gate before the leaf ran. This ceremony re-signs the
/// mutant over its own mutated bytes through the negative-evidence census
/// route, so it passes the signature gate and reaches the coordinator
/// leaf's `InspectOutputScriptPubKey` version clause — which refuses the
/// bare-u output. The unmutated control is accepted afterwards on the same
/// chain, which is what makes the mutant's refusal attributable.
///
/// # What it asserts, and why the standing changed
///
/// This run's verdicts are RECORDED as run-of-record constants, and a
/// record nothing checks can drift silently — the standing an adversarial
/// review found this lane resting on. So every verdict this ceremony's
/// constants carry is now ASSERTED against them: each mutant's layer and
/// verbatim detail, each declared separator, the control's ACCEPTANCE and
/// its accepted identity, and the two-origin readback unconditionally. The
/// lane fails if the control were rejected, if a mutant answered at the
/// wrong layer, or if a detail, a range, a shape or an identity moved.
///
/// The reason this used to be left unasserted — that a lane which asserts a
/// verdict fails rather than reports on the day the honest answer changes —
/// is answered by the order of operations rather than by silence: the
/// transcript and the wall time are written to disk BEFORE the first
/// assertion runs, so the artifact carries what the node actually said
/// either way, and a drift is reported AND failed rather than passed over.
/// The first-party construction facts stay asserted beside them: the
/// mutation is confined to its declared field, which is what makes the
/// refusal attributable to that field and to no other.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_bare_u_output_mutant_is_refused_before_the_control_is_accepted() {
    use vectors::live_owner_signing_negatives::{
        OwnerSigningNegativePlanner, render_owner_signing_negatives,
    };

    let mut capture_guard = CaptureGuard::new(CeremonyId::OwnerSigningNegatives);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("owner-signing-negatives"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::OwnerSigningNegatives,
            report.as_deref(),
        )),
    );

    let mut planner =
        OwnerSigningNegativePlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_owner_signing_negatives(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let mut facts = CeremonyCaptureFacts::from_capture(CeremonyId::OwnerSigningNegatives, &capture);
    for operation in capture.operations() {
        let step = operation.request().case.step.as_str();
        if let Some(locator) = record.capture_locator(step) {
            facts = facts.with_locator(step, locator);
        }
    }
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as one.
    // It is never a target verdict, so it is reported and the test stops
    // here rather than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the owner-signing negative ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    assert!(record.relinked(), "the ceremony funded before it linked");
    assert_eq!(record.coins().len(), 2);
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_owner_observation::ObservedFundedCoin::matches_expectation),
        "the node reported a coin the ceremony did not ask for",
    );

    // The mutant and the control were each submitted and answered, and each
    // answered WHERE the run of record says, in the words it recorded.
    assert_mutant_and_control_match_the_record(record);

    // The two candidates were signed over different messages: a mutant whose
    // message coincided with the control's would be signed over the same
    // bytes and the comparison would be vacuous.
    let mutant = record.mutant().expect("the mutant was built and submitted");
    let control = record
        .control()
        .expect("the control was built and submitted");
    assert_ne!(
        mutant.message(),
        control.message(),
        "the mutant and the control were signed over one message",
    );
    assert!(rendered.contains("messages_differ true"));

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("each_row_by_its_own_mutant true"));

    // Every consensus-conservation mutant was built and submitted; each row
    // the run of record carries was refused in its recorded words on the
    // recorded (range, shape) separator, and the revision-8 row is checked
    // on what the ceremony establishes until a capture observes it. The
    // separators are distinct across all of them, so no two rows rest on
    // one observation.
    assert_consensus_mutants_separate(record);

    // Every leaf-arrangement mutant was built and submitted; the recorded
    // rows were refused at the script path in their recorded words on the
    // recorded arrangement, and the revision-8 exchange is checked on the
    // arrangement it declares. The arrangements are distinct across all of
    // them, so no two rows rest on one observation.
    assert_leaf_arrangements_drive(record);
}

/// The bare-u mutant and the control were each answered WHERE the run of
/// record says, in the words it recorded, at the identity it names.
///
/// Split out so the test body stays under the line bound.
///
/// WHAT THIS REPLACED, and why. These facts were checked as "answered at
/// SOME layer", with the control's readback checked only where a
/// reverification happened to be present. That left the lane green on
/// exactly the drifts the recorded constants exist to make checkable: a
/// REJECTED control (which makes every refusal in the run unattributable),
/// a mutant answered at the wrong layer, or a detail or an accepted
/// identity that moved. The transcript is written to disk BEFORE any
/// assertion in this test runs, so binding these costs no report on the day
/// an answer changes — the artifact carries what the node said either way,
/// and the lane now FAILS instead of passing over a record nothing reads.
fn assert_mutant_and_control_match_the_record(
    record: &vectors::live_owner_signing_negatives::OwnerSigningNegativeRecord,
) {
    let expected = current_outcome_for_mutant(
        "owner-signing-negatives",
        LiveMutantKind::VaultControlEntitlementOrBareUOutput,
    );
    let mutant = record.mutant().expect("the mutant was built and submitted");
    assert_eq!(
        mutant.observed_layer(),
        Some(expected.layer()),
        "the bare-u mutant was not refused at the script path",
    );
    assert_eq!(
        mutant.observed_detail(),
        Some(expected.detail()),
        "the bare-u mutant drew words the run of record does not carry",
    );
    let LiveMutationLocator::WitnesslessRange { start, end } = expected
        .mutation_locator()
        .expect("the validated bare-u mutant carries its locator")
    else {
        panic!("the validated bare-u mutant has another locator shape")
    };
    assert_eq!(
        mutant.declared_field_range(),
        (*start, *end),
        "the mutation did not stay in the range the run of record declares",
    );
    assert_eq!(
        mutant.submitted_bytes(),
        expected.submitted_bytes().len(),
        "the mutant handed the node a different number of bytes",
    );

    let expected_control = sole_current_acceptance("owner-signing-negatives");
    let control = record
        .control()
        .expect("the control was built and submitted");
    assert_eq!(
        control.observed_layer(),
        Some(current_accepted_outcome("owner-signing-negatives").layer()),
        "the control was not ACCEPTED, so no refusal in this run is attributable",
    );
    assert_eq!(
        control.accepted_txid(),
        Some(expected_control.identity_display()),
        "the control was accepted at an identity the run of record does not carry",
    );

    // Checked UNCONDITIONALLY. The control is asserted ACCEPTED just above,
    // so an ABSENT two-origin check is itself the failure the conditional
    // form used to skip.
    let check = control
        .reverification()
        .expect("an accepted control carries its two-origin readback check");
    assert!(
        check.readback_matches_submission(),
        "the bytes the node reported are not the bytes it was handed",
    );
}

fn arrangement_indices(reference: &[Vec<u8>], programs: &[Vec<u8>]) -> Vec<usize> {
    programs
        .iter()
        .map(|program| {
            reference
                .iter()
                .position(|candidate| candidate == program)
                .expect("the validated locator names a committed control leaf")
        })
        .collect()
}

/// Each leaf-arrangement row and its corpus-proven revealed-leaf arrangement.
fn recorded_leaf_arrangements() -> [(LiveMutantKind, Vec<usize>, Vec<usize>); 2] {
    [
        LiveMutantKind::TwoCoordinators,
        LiveMutantKind::NoCoordinator,
    ]
    .map(|kind| {
        let outcome = current_outcome_for_mutant("owner-signing-negatives", kind);
        let LiveMutationLocator::CommittedLeafArrangement {
            input_indices,
            control_committed_leaf_programs,
            mutant_committed_leaf_programs,
            ..
        } = outcome
            .mutation_locator()
            .expect("the validated leaf mutant carries its locator")
        else {
            panic!("the validated leaf mutant has another locator shape")
        };
        assert_eq!(input_indices.len(), mutant_committed_leaf_programs.len());
        (
            kind,
            arrangement_indices(
                control_committed_leaf_programs,
                mutant_committed_leaf_programs,
            ),
            arrangement_indices(
                control_committed_leaf_programs,
                control_committed_leaf_programs,
            ),
        )
    })
}

/// Every leaf-arrangement mutant declared a distinct revealed-leaf
/// arrangement, distinct also from the control's, and each row the run of
/// record carries was refused at the script path in its recorded words.
///
/// Split out for the same reason the consensus assertion is: the fact the
/// drive rests on — that each mutant is a distinct candidate — is stated
/// once and the test body stays under the line bound.
fn assert_leaf_arrangements_drive(
    record: &vectors::live_owner_signing_negatives::OwnerSigningNegativeRecord,
) {
    use std::collections::BTreeSet;
    use vectors::live_owner_signing_negatives::LeafArrangementObservation;
    let arrangements = record.leaf_arrangements();
    let recorded = recorded_leaf_arrangements();
    assert_eq!(
        arrangements.len(),
        recorded.len() + PENDING_LEAF_ARRANGEMENTS.len(),
        "the leaf-arrangement mutants — the recorded rows and the revision-8 exchange — were built",
    );
    // Set equality: the built rows are exactly the recorded ones plus the
    // rows declared pending, so a renamed or substituted row fails rather
    // than passing as "three of something".
    let built: BTreeSet<&str> = arrangements
        .iter()
        .map(LeafArrangementObservation::row)
        .collect();
    assert_eq!(
        built,
        recorded
            .iter()
            .map(|(kind, ..)| kind.row())
            .chain(PENDING_LEAF_ARRANGEMENTS.iter().map(|(kind, _)| kind.row()))
            .collect::<BTreeSet<_>>(),
        "the leaf-arrangement mutants are not the recorded rows and the pending row",
    );
    for mutant in arrangements {
        let Some((kind, arrangement, control_arrangement)) = recorded
            .iter()
            .find(|(kind, ..)| kind.row() == mutant.row())
        else {
            assert_pending_leaf_arrangement(mutant);
            continue;
        };
        let expected = current_outcome_for_mutant("owner-signing-negatives", *kind);
        assert_eq!(
            mutant.observed_layer(),
            Some(expected.layer()),
            "{} was not refused at the script path",
            mutant.row(),
        );
        let revealed = mutant
            .revealed_arrangement()
            .iter()
            .copied()
            .map(usize::from)
            .collect::<Vec<_>>();
        assert_eq!(
            revealed.as_slice(),
            arrangement.as_slice(),
            "{} revealed an arrangement the run of record does not carry",
            mutant.row(),
        );
        assert_eq!(
            mutant.observed_detail(),
            Some(expected.detail()),
            "{} drew words the run of record does not carry",
            mutant.row(),
        );
        assert_eq!(
            mutant.submitted_bytes(),
            expected.submitted_bytes().len(),
            "{} submitted a different current candidate",
            mutant.row(),
        );
        assert_ne!(
            revealed.as_slice(),
            control_arrangement.as_slice(),
            "{} reveals the control's own arrangement and rearranges nothing",
            mutant.row(),
        );
    }
    // The separating fact is the revealed-leaf arrangement: the mutants keep
    // the control's witnessless serialization and differ only in which
    // committed leaf each input reveals, so a distinct arrangement per row
    // is what keeps no two rows resting on one observation.
    let mut arrangements: Vec<Vec<u16>> = arrangements
        .iter()
        .map(|mutant| LeafArrangementObservation::revealed_arrangement(mutant).to_vec())
        .collect();
    arrangements.sort_unstable();
    arrangements.dedup();
    assert_eq!(
        arrangements.len(),
        record.leaf_arrangements().len(),
        "two leaf-arrangement mutants share a revealed-leaf arrangement and do not separate",
    );
    // The pending row is included in that distinctness check deliberately.
    // Its outcome is not yet recorded anywhere, but the fact its row rests
    // on — that its arrangement is its own — is established by construction
    // and is checkable now, so the check that matters does not wait for the
    // capture.
}

/// One consensus row's separating fact: the half-open witnessless byte
/// range its surgery declared, together with the transaction shape the
/// mutant handed the node.
type ConsensusSeparator = ((usize, usize), (usize, usize));

/// The consensus rows this ceremony drives that the CURRENT corpus cannot
/// yet answer for.
///
/// The revision-8 rows are built and submitted by the same ceremony as the
/// recorded ones, but the frozen run of record predates them and carries no
/// outcome to bind them to. Asserting them against it would not be a
/// stricter test — it would be a lookup that panics. They are checked here
/// on what the ceremony itself establishes and bound to a recorded outcome
/// when the capture that observes them is imported.
const OWNER_SIGNING_PENDING_CONSENSUS_MUTANTS: [LiveMutantKind; 1] =
    [LiveMutantKind::AmountOutsideSemanticDomain];

/// The leaf-arrangement rows the current corpus cannot yet answer for, with
/// the arrangement each declares.
const PENDING_LEAF_ARRANGEMENTS: [(LiveMutantKind, [u16; 2]); 1] =
    [(LiveMutantKind::MemberCoordinatorLeafExchange, [1, 0])];

/// The arrangement the control reveals: the coordinator leaf at input zero
/// and the member leaf at input one.
const CONTROL_LEAF_ARRANGEMENT: [u16; 2] = [0, 1];

/// One leaf-arrangement mutant the current corpus cannot answer for, checked
/// on what the ceremony itself establishes.
///
/// Three facts, none of which needs a recorded outcome: it declared the
/// arrangement its row is defined by, that arrangement is not the control's,
/// and the target did not ACCEPT it. The third is the weakest statement that
/// still fails a broken drive — a negative row whose candidate is accepted
/// has driven nothing — and it deliberately stops short of naming a clause,
/// because which of two failing inputs this row's target reports is the
/// target's own abort selection.
fn assert_pending_leaf_arrangement(
    mutant: &vectors::live_owner_signing_negatives::LeafArrangementObservation,
) {
    let (_, declared) = PENDING_LEAF_ARRANGEMENTS
        .iter()
        .find(|(kind, _)| kind.row() == mutant.row())
        .expect("every built leaf arrangement is a recorded or a pending row");
    assert_eq!(
        mutant.revealed_arrangement(),
        declared.as_slice(),
        "{} revealed an arrangement its row does not declare",
        mutant.row(),
    );
    assert_ne!(
        mutant.revealed_arrangement(),
        CONTROL_LEAF_ARRANGEMENT.as_slice(),
        "{} reveals the control's own arrangement and rearranges nothing",
        mutant.row(),
    );
    assert_ne!(
        mutant.observed_layer(),
        Some(ObservedOutcomeLayer::Accepted),
        "{} was accepted, so the arrangement drove nothing",
        mutant.row(),
    );
}

/// One consensus mutant the current corpus cannot answer for, checked on
/// what the ceremony itself establishes: it is a row this ceremony declares
/// as pending, and the target did not accept it.
fn assert_pending_consensus_mutant(
    mutant: &vectors::live_owner_signing_negatives::ConsensusMutantObservation,
) {
    assert!(
        OWNER_SIGNING_PENDING_CONSENSUS_MUTANTS
            .iter()
            .any(|kind| kind.row() == mutant.row()),
        "{} is neither a recorded nor a pending consensus row",
        mutant.row(),
    );
    assert_ne!(
        mutant.observed_layer(),
        Some(ObservedOutcomeLayer::Accepted),
        "{} was accepted, so the surgery drove nothing",
        mutant.row(),
    );
}

const OWNER_SIGNING_RECORDED_CONSENSUS_MUTANTS: [LiveMutantKind; 7] = [
    LiveMutantKind::WrongExplicitAsset,
    LiveMutantKind::ConfidentialAssetCommitment,
    LiveMutantKind::OutputTotalOneBelowInput,
    LiveMutantKind::OutputTotalOneAboveInput,
    LiveMutantKind::PrivateOutputOmitted,
    LiveMutantKind::HiddenPrivateUOutput,
    LiveMutantKind::OmittedSource,
];

fn changed_current_range(left: &[u8], right: &[u8]) -> (usize, usize) {
    let prefix = left.iter().zip(right).take_while(|(a, b)| a == b).count();
    let suffix = left
        .iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(left.len() - prefix)
        .min(right.len().saturating_sub(prefix));
    (prefix, left.len() - suffix)
}

fn current_consensus_separator(kind: LiveMutantKind) -> ConsensusSeparator {
    let outcome = current_outcome_for_mutant("owner-signing-negatives", kind);
    let mutant = current_transaction(outcome.submitted_bytes());
    let shape = (mutant.inputs().len(), mutant.outputs().len());
    let range = match outcome
        .mutation_locator()
        .expect("the validated consensus mutant carries its locator")
    {
        LiveMutationLocator::WitnesslessRange { start, end } => (*start, *end),
        LiveMutationLocator::TransactionShape {
            control_inputs,
            mutant_inputs,
            control_outputs,
            mutant_outputs,
        } => {
            assert_eq!(shape, (*mutant_inputs, *mutant_outputs));
            let control = current_transaction(
                sole_current_acceptance("owner-signing-negatives").submitted_bytes(),
            );
            assert_eq!(
                (control.inputs().len(), control.outputs().len()),
                (*control_inputs, *control_outputs),
            );
            changed_current_range(
                &control.encode_without_witness(),
                &mutant.encode_without_witness(),
            )
        }
        _ => panic!("the validated consensus mutant has another locator shape"),
    };
    (range, shape)
}

/// Each RECORDED consensus row and the `(range, shape)` separator the run of
/// record declares for it.
///
/// The four field surgeries keep the control's 2-in-2-out shape and separate
/// by four distinct ranges; the two output-cardinality surgeries share the
/// structural range `changed_range` cannot localize past the output-count
/// varint and separate by shape; `omitted-source` separates by both.
fn recorded_consensus_separators() -> [(&'static str, ConsensusSeparator); 7] {
    OWNER_SIGNING_RECORDED_CONSENSUS_MUTANTS
        .map(|kind| (kind.row(), current_consensus_separator(kind)))
}

/// The consensus-conservation mutants are the recorded rows together with
/// the rows declared pending; each recorded one was refused at consensus
/// before script in the recorded words on the separator the run of record
/// declares for it, and the separators are pairwise distinct across all of
/// them.
///
/// Split from the test body so the assertion the run rests on — that no
/// two rows share one observation — is stated once and the test stays
/// under the line bound.
fn assert_consensus_mutants_separate(
    record: &vectors::live_owner_signing_negatives::OwnerSigningNegativeRecord,
) {
    use std::collections::BTreeSet;
    use vectors::live_owner_signing_negatives::ConsensusMutantObservation;
    let consensus = record.consensus_mutants();
    let recorded = recorded_consensus_separators();
    assert_eq!(
        consensus.len(),
        recorded.len() + OWNER_SIGNING_PENDING_CONSENSUS_MUTANTS.len(),
        "the consensus mutants — the recorded rows and the revision-8 out-of-domain write — were built",
    );
    // Set equality: the built rows are exactly the recorded ones plus the
    // rows declared pending, so a renamed or substituted row fails rather
    // than passing as "eight of something".
    let built: BTreeSet<&str> = consensus
        .iter()
        .map(ConsensusMutantObservation::row)
        .collect();
    assert_eq!(
        built,
        recorded
            .iter()
            .map(|(row, _)| *row)
            .chain(
                OWNER_SIGNING_PENDING_CONSENSUS_MUTANTS
                    .iter()
                    .map(|kind| kind.row())
            )
            .collect::<BTreeSet<_>>(),
        "the consensus mutants are not the recorded rows and the pending row",
    );
    for mutant in consensus {
        let Some(kind) = OWNER_SIGNING_RECORDED_CONSENSUS_MUTANTS
            .iter()
            .copied()
            .find(|kind| kind.row() == mutant.row())
        else {
            assert_pending_consensus_mutant(mutant);
            continue;
        };
        let expected = current_outcome_for_mutant("owner-signing-negatives", kind);
        let (_, separator) = recorded
            .iter()
            .find(|(row, _)| *row == mutant.row())
            .expect("every recorded mutant carries its separator");
        assert_eq!(
            mutant.observed_layer(),
            Some(expected.layer()),
            "{} was not refused at consensus before script",
            mutant.row(),
        );
        assert_eq!(
            mutant.observed_detail(),
            Some(expected.detail()),
            "{} drew words the run of record does not carry",
            mutant.row(),
        );
        assert_eq!(
            mutant.submitted_bytes(),
            expected.submitted_bytes().len(),
            "{} submitted a different current candidate",
            mutant.row(),
        );
        assert_eq!(
            mutant.separator(),
            *separator,
            "{} drifted off the separator the run of record declares",
            mutant.row(),
        );
    }
    // The separating fact is the byte range together with the shape: the
    // field surgeries keep the control's shape and separate by range, the
    // structural surgeries separate by shape where the output-count varint
    // defeats a localized range, and the out-of-domain write separates from
    // the one-below surgery at the SAME field because an absolute write of
    // a high value moves different bytes than a delta of one. The tuple is
    // distinct across all of them, recorded and pending alike, so no two
    // rows rest on one observation.
    let mut separators: Vec<((usize, usize), (usize, usize))> = consensus
        .iter()
        .map(ConsensusMutantObservation::separator)
        .collect();
    separators.sort_unstable();
    separators.dedup();
    assert_eq!(
        separators.len(),
        consensus.len(),
        "two consensus mutants share a range-and-shape separator and do not separate",
    );
}

/// One KEY-PATH spend attempt against a funded explicit constructor.
///
/// # What this run is for
///
/// Every constructor here is spent by its script path, under an internal
/// key that is a published nothing-up-my-sleeve point. Nobody had ever
/// offered this target a one-item witness at one of these programs, so
/// nothing was known about what layer such a candidate lands at, what
/// the node says about it, or whether the generic submission wire
/// carries it at all. This asks.
///
/// # What a refusal discharges
///
/// That the attempt was observed and refused. Nothing else. The
/// signature offered is by a published test key that is not the output
/// key, so a target refusing it is refusing a signature that does not
/// verify — which says nothing whatever about who knows the internal
/// key's discrete logarithm. The residual assumption stands regardless,
/// and the artifact says so in its own bytes.
///
/// # What it asserts, and what it merely records
///
/// The shape of a completed attempt: the deployment relinked before it
/// funded, the node's own fields agreed with the ceremony's expectation,
/// the constructor's internal key is the published point, the witness is
/// the one-item shape, and the signing key is not the output key. It also
/// asserts the PAIR: that the control offered after the attempt is the
/// same candidate — measured off the two witnessless serializations — and
/// that the two submissions drew different verdicts.
///
/// It also asserts the two verdicts THEMSELVES, against phase B's own run
/// of record: the attempt's exact layer and the target's verbatim words,
/// the control's acceptance and the identity it was accepted at. The
/// earlier form recorded both and asserted neither, so that an unexpected
/// layer reached a reader rather than a panic. That protection is kept
/// where it belongs — the transcript is written to disk BEFORE any
/// assertion here runs, so the artifact carries what the node said either
/// way — and the lane now FAILS on the drifts the recorded constants
/// exist to make checkable.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn one_key_path_spend_attempt_is_offered_to_a_real_target() {
    use vectors::live_keypath_probe::{KeyPathProbePlanner, ProbeProvenance, render_keypath_probe};

    let mut capture_guard = CaptureGuard::new(CeremonyId::KeypathProbe);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("keypath-probe"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::KeypathProbe,
            report.as_deref(),
        )),
    );

    // The target's provenance, as the run's own environment reports it.
    // The declared source tip is the operator's declaration about the
    // binary the adapter was pointed at; this probe records it and
    // verifies nothing about it, which is what its name says.
    let provenance = ProbeProvenance {
        network_id: network,
        genesis_id: genesis.clone(),
        target_version: format!("{:?}", target.definition().version()),
        declared_source_tip: environment("ELEMENTS_NATIVE_EXECUTOR_INTENDED_TIP"),
        executor_trust: format!("{:?}", ExecutorTrust::ReviewedNonMock),
    };

    let mut planner =
        KeyPathProbePlanner::new(identifier(&genesis), provenance).expect("the ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_keypath_probe(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let mut facts = CeremonyCaptureFacts::from_capture(CeremonyId::KeypathProbe, &capture);
    if let Some(locator) = record.capture_locator() {
        facts = facts.with_locator(vectors::live_keypath_probe::ATTEMPT_STEP, locator);
    }
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    outcome.expect("the ceremony reached the target");

    assert_keypath_probe(record, &rendered);
}

fn assert_keypath_probe(record: &vectors::live_keypath_probe::KeyPathProbeRecord, rendered: &str) {
    // The deployment was welded to the chain before anything was funded,
    // so the program the attempt spends belongs to a deployment of the
    // asset the target issued.
    assert!(record.relinked(), "the ceremony funded before it linked");
    assert_eq!(record.coins().len(), 1, "the probe funds exactly one coin");
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_owner_observation::ObservedFundedCoin::matches_expectation),
        "the node reported a coin the ceremony did not ask for",
    );

    // The constructor under probe is the one this run is about. A
    // deployment whose internal key was not the published point would
    // make the whole record a report on a different question.
    let program = record
        .binding()
        .expect("the ceremony carries the constructor it funded");
    assert!(
        program.internal_key_is_the_published_point(),
        "the funded constructor did not inherit the published internal key",
    );

    // The attempt was built and offered, and it is the shape the run
    // claims: one witness item, and a signing key that is not the output
    // key the program carries. The second is the assertion that keeps
    // the refusal honest — an attempt signed by the output key would be
    // a different experiment entirely.
    let attempt = record
        .attempt()
        .expect("the ceremony built the key-path attempt");
    assert!(
        attempt.is_the_one_item_shape(),
        "the witness is not the one-item key-path shape",
    );
    assert_ne!(
        program.output_key().as_slice(),
        attempt.signing_public_key().as_slice(),
        "the attempt was signed by the output key, which is not this probe",
    );
    // The target answered, which is the precondition every assertion
    // below rests on: a run that reached no verdict at all fails here,
    // where the failure names what happened, rather than inside a
    // comparison against a figure the run never produced.
    assert!(
        record.observation().is_some(),
        "the attempt was not answered",
    );

    assert_the_pair_is_one_candidate_answered_twice(record);
    assert_the_verdicts_match_the_run_of_record(record);

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("residual_internal_key_unspendability_stands true"));
    assert!(rendered.contains("discharges_no_residual true"));
}

/// The pair's own assertions, kept beside the run that produces them.
///
/// Extracted rather than inlined because the probe's test asserts two
/// different things — the attempt's shape and the pair's relation — and a
/// reader looking for the second should not have to find it inside the
/// first.
fn assert_the_pair_is_one_candidate_answered_twice(
    record: &vectors::live_keypath_probe::KeyPathProbeRecord,
) {
    // The control is the SAME candidate: the two witnessless
    // serializations were compared byte for byte and the comparison is
    // what is asserted, not the construction that produced them. A
    // control finalized over some other coin would be a second candidate
    // wearing the control's name.
    let control = record
        .control()
        .expect("the ceremony built the script-path control");
    assert!(
        control.shares_the_attempts_witnessless_bytes(),
        "the control and the attempt are not one candidate",
    );
    assert_eq!(
        control.witness_items(),
        3,
        "the control is not the script-path shape",
    );

    // The two verdicts are DIFFERENT, which is the whole content of the
    // pair. WHICH each of them was is bound separately, in
    // [`assert_the_verdicts_match_the_run_of_record`]: inequality alone
    // holds for an accepted attempt against a refused control just as it
    // holds for the run that happened.
    let control_observation = record
        .control_observation()
        .expect("the control was not answered");
    assert_ne!(
        record
            .observation()
            .expect("the attempt was answered")
            .layer(),
        control_observation.layer(),
        "the attempt and its control drew one verdict, so the pair separates nothing",
    );
}

// The two verdicts and the two candidates, held against phase B's run of
// record.
//
// Why this was unbound, and why binding it costs no report:
//
// The probe's first form recorded both verdicts and asserted neither, so
// that an unexpected layer would reach a reader instead of a panic. The
// rendered transcript is what delivers that, and it is written to disk
// BEFORE any assertion in this test runs — so nothing here costs the
// report on the day an answer changes. What the unasserted form actually
// left green was every drift the run of record exists to make checkable:
// an ACCEPTED attempt against a REFUSED control differ in layer just as
// the run that happened does, so the pair's one assertion passed on the
// exact inversion that makes the seventeenth refusal row unattributable.
// Each figure below is phase B's own, and each now fails on drift.
fn assert_keypath_phase_a_matches_current_corpus(
    record: &vectors::live_keypath_probe::KeyPathProbeRecord,
    expected_attempt: &vectors::live_corpus_native_v2_r7::NativeV2OutcomeProjection,
    input_index: usize,
    mutant_stack_items: usize,
) -> transaction::bytes::TargetTransaction {
    // One chain, one issuance: the asset the probe's own run recorded.
    let ceremony = current_ceremony_projection("keypath-probe");
    assert_eq!(
        record.issued_asset(),
        ceremony.issued_asset_display(),
        "the probe ran against an asset the run of record does not carry",
    );

    let expected_attempt_transaction = current_transaction(expected_attempt.submitted_bytes());
    let expected_attempt_witness = expected_attempt_transaction
        .witnesses()
        .get(input_index)
        .expect("the validated key-path locator names an input")
        .stack();
    assert_eq!(expected_attempt_witness.len(), mutant_stack_items);
    let attempt = record
        .attempt()
        .expect("the ceremony built the key-path attempt");
    assert_eq!(
        attempt.submitted_bytes().len(),
        expected_attempt.submitted_bytes().len(),
        "the attempt handed the node a different number of bytes",
    );
    assert_eq!(
        attempt.witness_stack().len(),
        expected_attempt_witness.len(),
        "the attempt is not the recorded one-item witness",
    );
    assert_eq!(
        attempt
            .witness_stack()
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>(),
        expected_attempt_witness
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>(),
        "the attempt's witness items do not have the recorded widths",
    );
    // The bytes carry a whole transaction and not only the witness item,
    // which is the cheapest check that the attempt was assembled rather
    // than merely signed. Kept beside the recorded width it reads.
    assert!(
        attempt.submitted_bytes().len() > attempt.witness_stack()[0].len(),
        "the submitted bytes are no larger than the witness item",
    );
    expected_attempt_transaction
}

fn assert_keypath_phase_b_matches_current_corpus(
    record: &vectors::live_keypath_probe::KeyPathProbeRecord,
    expected_attempt: &vectors::live_corpus_native_v2_r7::NativeV2OutcomeProjection,
    expected_attempt_transaction: &transaction::bytes::TargetTransaction,
    input_index: usize,
    control_stack_items: usize,
    witnessless_serialization_equal: bool,
) {
    // The ATTEMPT's verdict, exactly. Phase B's whole content is the name
    // the refusal is filed under, so the layer is asserted as the enum AND
    // the recorded spelling is held against that enum: a constant that had
    // drifted from the vocabulary would otherwise stay green beside it.
    let observation = record.observation().expect("the attempt was answered");
    assert_eq!(
        observation.layer(),
        expected_attempt.layer(),
        "the attempt was not refused at the key path",
    );
    assert_eq!(
        format!("{:?}", observation.layer()),
        format!("{:?}", expected_attempt.layer()),
        "the recorded layer name is not the vocabulary's own spelling",
    );
    assert_eq!(
        observation.detail(),
        Some(expected_attempt.detail()),
        "the attempt drew words the run of record does not carry",
    );
    assert_eq!(
        observation.accepted_txid(),
        expected_attempt
            .target_identity()
            .map(|identity| identity.to_target_display())
            .as_deref(),
        "a refused attempt was given an accepted identity",
    );

    // The CONTROL's verdict, exactly. A refused control makes every
    // refusal in the run unattributable, and its identity is the half of
    // the pair a reader can check against a chain.
    let expected_control = sole_current_acceptance("keypath-probe");
    let expected_control_outcome = current_accepted_outcome("keypath-probe");
    let control_observation = record
        .control_observation()
        .expect("the control was answered");
    assert_eq!(
        control_observation.layer(),
        expected_control_outcome.layer(),
        "the control was not ACCEPTED, so the attempt's refusal is not attributable",
    );
    assert_eq!(
        control_observation.accepted_txid(),
        Some(expected_control.identity_display()),
        "the control was accepted at an identity the run of record does not carry",
    );

    // The control's own bytes. This record carries no two-origin readback
    // — the reverification the owner-signing ceremony records has no
    // counterpart in this probe — so what stands in its place is the
    // measured witnessless-bytes relation the pair rests on, held against
    // the recorded value rather than against a literal.
    let control = record
        .control()
        .expect("the ceremony built the script-path control");
    let expected_control_transaction = current_transaction(expected_control.submitted_bytes());
    let expected_control_witness = expected_control_transaction
        .witnesses()
        .get(input_index)
        .expect("the validated key-path control carries the linked input")
        .stack();
    assert_eq!(expected_control_witness.len(), control_stack_items);
    assert_eq!(
        control.submitted_bytes().len(),
        expected_control.submitted_bytes().len(),
        "the control handed the node a different number of bytes",
    );
    assert_eq!(
        control.witness_items(),
        expected_control_witness.len(),
        "the control is not the recorded script-path witness census",
    );
    assert_eq!(
        control.shares_the_attempts_witnessless_bytes(),
        witnessless_serialization_equal,
        "the pair no longer differs in the witness alone",
    );
    assert_eq!(
        expected_control_transaction.encode_without_witness(),
        expected_attempt_transaction.encode_without_witness(),
        "the current corpus's pair differs outside the witness",
    );
}

fn assert_the_verdicts_match_the_run_of_record(
    record: &vectors::live_keypath_probe::KeyPathProbeRecord,
) {
    let expected_attempt =
        current_outcome_for_mutant("keypath-probe", LiveMutantKind::KeyPathEscape);
    let LiveMutationLocator::WitnessPathShape {
        input_index,
        control_stack_items,
        mutant_stack_items,
        witnessless_serialization_equal,
        ..
    } = expected_attempt
        .mutation_locator()
        .expect("the current key-path outcome carries its validated locator")
    else {
        panic!("the current key-path outcome has another locator shape")
    };
    let expected_attempt_transaction = assert_keypath_phase_a_matches_current_corpus(
        record,
        expected_attempt,
        *input_index,
        *mutant_stack_items,
    );
    assert_keypath_phase_b_matches_current_corpus(
        record,
        expected_attempt,
        &expected_attempt_transaction,
        *input_index,
        *control_stack_items,
        *witnessless_serialization_equal,
    );
}

/// The restart order's fifth step, the split shape: one receipt in, three
/// outputs, against a real node.
///
/// # What this run is for
///
/// Step five asks for the remaining positive private shapes, each where it
/// accepts. This is the split: one confidential receipt consumed and split
/// into two recipients and a balancing change output, the smallest of the
/// remaining shapes and the one that needs only a three-output successor
/// and one input. Its acceptance moves the `private-split` matrix row, and
/// the move is the observed identity this run records, not the fact of the
/// test existing.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_split_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::Split, "multi-split");
}

/// The split PAIR's private member: one receipt in, TWO blinded receipts
/// out, no change and no fee, against a real node.
///
/// # Why the split run above does not answer this
///
/// §16.1's split pair states one semantic fixture and materializes it
/// twice. Its explicit member ran and was accepted; its private member
/// creates exactly two outputs, and neither private run this lane has
/// recorded is that shape. The split above creates THREE outputs, keeping
/// a balancing change back for the sender, and the only other recorded
/// one-in-two-out private run is the fee-bearing shape, whose second
/// output is a fee role rather than a receipt. A pair member is not
/// answered by a run of a different cardinality, and it is not answered
/// by a run whose second output is a different role.
///
/// So this run exists to be the pair member's own shape, and the pair's
/// acceptance conjunct moves on it or on nothing.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_pure_split_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::PureSplit, "multi-pure-split");
}

/// The restart order's fifth step, the many-to-many shape: two receipts in,
/// three outputs, against a real node.
///
/// # Why this representative case
///
/// The matrix names a representative many-to-many, not a proof over every
/// cardinality. The case chosen is the smallest whose input and output
/// counts both exceed the one-to-one control's: two receipts consumed and
/// three outputs created, so that "many to many" describes both halves and
/// is not a one-to-many or many-to-one in disguise.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_many_to_many_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::ManyToMany, "multi-many-to-many");
}

/// The restart order's fifth step, the several-distinct-owners shape: two
/// receipts under two distinct owners in, two outputs, against a real node.
///
/// # What distinguishes it from the many-to-many run
///
/// Its subject is the input owners rather than the cardinality: the two
/// consumed receipts are owned by two distinct published owners, and each
/// input carries the leaf its own position executes. The predecessor pays
/// its two outputs to the two owners' private receipt constructors, so
/// consuming both is a transfer whose inputs have several distinct owners.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_several_distinct_owners_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::SeveralDistinctOwners, "multi-several-owners");
}

/// The strict one-to-one: ONE receipt in and ONE output out, against a
/// real node.
///
/// # The shape the registry used to refuse
///
/// This is not a step of the restart order and it moves no matrix row.
/// It is a row of the CONSENSUS shape census, which enumerates what the
/// target's balance rule admits rather than what the guide's own class
/// table names, and it sat there recorded source-derived-possible and
/// refused: the fixture registry's two-output floor turned it away before
/// looking at its one output. The census typed that floor as a
/// first-party convention rather than a protocol rule, and this run is
/// what a structural removal is worth — the shape's lone output declares
/// the fully-solved balancing form, takes the consumed coin's own value
/// blinder, and is offered to a node.
///
/// # Why THIS one-output shape and not the merge
///
/// It consumes one receipt, so its input blinder sum is a single coin's
/// blinder with nothing to cancel against. A merge of this ceremony's
/// predecessor consumes an inverse pair whose blinders sum to zero, and a
/// forced zero blinder hides nothing — the registry refuses it, so no
/// merge is submitted from here and none is claimed.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_strict_one_to_one_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::StrictOneToOne, "multi-strict-one-to-one");
}

/// One blinded input, one blinded output, and one REAL fee output.
///
/// # The shape the register called expressible and unrun
///
/// The fixture registry gained a fee role in the structural-removals
/// wave and nothing downstream of it could carry one, so the shape
/// registered, derived and digested and then stopped at the projection
/// with a typed refusal naming the missing projection. The stop was
/// honest and it was not a run: a register that had recorded the shape
/// observed because its vocabulary could express it would have been
/// committing the exact error the register exists to prevent.
///
/// This run is what that removal is worth. The fee output is a fee at
/// the target and not a blinded output wearing the name -- explicit
/// value, explicit asset, empty scriptPubKey, and no witness entry of
/// its own -- and the blinded output beside it is the balancing one,
/// whose blinder is solved over no other freely chosen blinder and
/// therefore comes out as the consumed coin's own.
///
/// # What it does not establish
///
/// It is not `private-sponsor-values` and it moves no matrix row. Nobody
/// sponsors anything here: the transaction pays its own fee out of its
/// own consumed coin, in the disposable protocol asset, which is the
/// only asset whose tally that fee can close. The sponsor rows' signer
/// dependency is untouched.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_fee_bearing_one_to_one_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::OneToOneWithFee, "multi-one-to-one-with-fee");
}

/// TWO blinded inputs merged into ONE blinded output.
///
/// # The shape that met two walls
///
/// The first was the fixture registry's two-output floor, which refused
/// any manifest of fewer than two outputs and turned the merge away
/// before looking at it. The sole-balancing form removed that floor, and
/// the merge walked forward into a second wall the first had been
/// hiding: the only coins the ceremony could offer it were the two
/// halves of an inverse pair, whose blinders sum to zero, so the lone
/// output's forced blinder was zero -- a commitment of exactly the value
/// times the value generator, which anybody recomputes from a guessed
/// amount. The registry refused it by name, and refusing it was right.
///
/// # What makes this one different, in one sentence
///
/// It spends a THREE-output predecessor, whose blinders cancel in no
/// pair.
///
/// Three blinders summing to zero leave any two of them summing to the
/// negation of the third. The third here is a DERIVED blinder, and a
/// derived blinder is searched upward until it is nonzero and never
/// admitted zero -- so the forced blinder is nonzero for a reason that
/// can be stated. The registry would refuse a zero one by name if the
/// reasoning were wrong, which is what makes the successor registering
/// at all a proof and not a hope.
///
/// # What it establishes
///
/// The row `private-merge` of the positive private table, on an
/// acceptance of THIS shape and nothing wider. It is a two-input
/// one-output transfer and it is not a claim about merges in general.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_private_merge_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::PrivateMerge, "multi-private-merge");
}

/// The sponsored CONFIDENTIAL with-change shape, offered to a real node.
///
/// The shape the section 15.2 `private-sponsor-values` row moves on, and
/// the one no run in this workspace had ever offered: a blinded sponsor
/// coin in at an explicit asset, blinded receipt destinations, a
/// COMMITTED sponsor change, and an explicit reserve fee held outside
/// both balance equations.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsored_confidential_with_change_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsored_private::{SponsoredPrivatePlanner, render_sponsored_private};

    let mut capture_guard = CaptureGuard::new(CeremonyId::SponsoredPrivateWithChange);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("sponsored-private-with-change"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::SponsoredPrivateWithChange,
            report.as_deref(),
        )),
    );

    let mut planner =
        SponsoredPrivatePlanner::new(identifier(&genesis)).expect("the ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_sponsored_private(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts =
        CeremonyCaptureFacts::from_capture(CeremonyId::SponsoredPrivateWithChange, &capture)
            .with_digest("successor", planner.capture_successor_digest());
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as
    // one. It is never a target verdict, so it is reported and the test
    // stops here rather than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the sponsored private ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // The blinded sponsor coin, censused by named check and never by a
    // scalar. Counted against the whole vocabulary rather than against
    // the checks this test happens to name, so a check added later is
    // one this run has to have seen hold.
    let solve = record
        .solve()
        .expect("the sponsor funding stage censused the coin");
    assert!(
        solve.every_check_held(),
        "the blinded sponsor coin failed checks: {:?}",
        solve.missing(),
    );

    // The sponsor's round trip: the adapter signed the bytes it was
    // handed, and returned a witness.
    let round = record
        .round()
        .expect("the staging pass recorded a signing request");
    assert!(
        round.echo_matches_what_was_sent(),
        "the adapter authorized bytes that are not the ones it was handed",
    );
    assert!(round.witness_items() > 0, "the sponsor returned no witness");

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    let check = record
        .reverification()
        .expect("an acceptance was observed and read back");
    assert!(
        check.readback_matches_submission(),
        "the bytes the node reported are not the bytes it was handed",
    );
    assert!(
        check.owner_signature_verified(),
        "an owner's signature did not verify against an independently recomputed message",
    );
    assert_eq!(
        check.sponsor_change_located(),
        Some(true),
        "the sponsor's committed change was not found in the mined bytes",
    );
}

/// The sponsor PAIR's private member: an EXPLICIT sponsor coin funded
/// exactly to the fee, no change role, one blinded destination.
///
/// # Why the with-change run above does not answer this
///
/// §16.1's sponsor pair states one semantic fixture and materializes it
/// twice. Its explicit member ran and was accepted; its private member
/// states an explicit sponsor coin funded exactly to the fee, no change
/// role, and ONE destination, and the run above is none of those three:
/// it carries a blinded sponsor coin, a committed sponsor change, and two
/// blinded destinations.
///
/// # The arithmetic that lets this shape exist
///
/// The sponsor arc observed that a COMMITTED sponsor value REQUIRES
/// committed change -- a blinded input's blinder must be absorbed by
/// something, a fee is mandatorily explicit, and the change is the only
/// remaining term. That observation does not reach this shape and does
/// not forbid it. An explicit sponsor coin is committed with the all-zero
/// blinder, so there is nothing to absorb and no change is owed: the
/// sponsor input equals the fee output in the reserve asset, and the
/// protocol asset closes over the receipts alone.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsored_explicit_no_change_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsored_private::{
        SponsoredPrivatePlanner, SponsoredPrivateShape, render_sponsored_private,
    };

    let mut capture_guard = CaptureGuard::new(CeremonyId::SponsoredPrivateExplicitNoChange);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("sponsored-private-explicit-no-change"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::SponsoredPrivateExplicitNoChange,
            report.as_deref(),
        )),
    );

    let mut planner = SponsoredPrivatePlanner::for_shape(
        SponsoredPrivateShape::ExplicitWithoutChange,
        identifier(&genesis),
    )
    .expect("the ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_sponsored_private(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts =
        CeremonyCaptureFacts::from_capture(CeremonyId::SponsoredPrivateExplicitNoChange, &capture);
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    if let Some(refusal) = record.refusal() {
        panic!("the sponsored private ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // NO solve census, and its absence is asserted rather than left
    // unmentioned. The census is a statement about a BLINDED sponsor
    // coin -- that the chain reported a commitment, and that the
    // commitment is the one the registry derives -- and this shape funds
    // no such coin. A census present here would mean the committed
    // funding stage had run, which is the one thing this shape's step
    // plan removes.
    assert!(
        record.solve().is_none(),
        "an explicit sponsor coin produced a committed-coin census",
    );

    // The sponsor's round trip still happens: an explicit coin is still
    // somebody else's coin, and spending it still needs its owner's
    // authorization over the exact finalized bytes.
    let round = record
        .round()
        .expect("the staging pass recorded a signing request");
    assert!(
        round.echo_matches_what_was_sent(),
        "the adapter authorized bytes that are not the ones it was handed",
    );
    assert!(round.witness_items() > 0, "the sponsor returned no witness");

    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    let check = record
        .reverification()
        .expect("an acceptance was observed and read back");
    assert!(
        check.readback_matches_submission(),
        "the bytes the node reported are not the bytes it was handed",
    );
    assert!(
        check.owner_signature_verified(),
        "an owner's signature did not verify against an independently recomputed message",
    );
    // The change question is NOT ASKED of this shape, and that is
    // asserted rather than left implicit. The located-check is a byte
    // scan for the reserve asset and a sponsored transaction's FEE
    // carries the reserve asset too, so on a no-change shape the scan
    // answers about the fee and a reader would take it for a change
    // output that is not there.
    //
    // What rules the change out is arithmetic over the acceptance just
    // asserted, and it is the stronger statement. The target balances
    // per asset, so the reserve sub-equation is
    // `sponsor_input == fee + change`; this sponsor's coin was funded to
    // EXACTLY the fee, so any change output at all would leave that
    // equation short and the node would have refused the candidate. It
    // accepted it.
    assert_eq!(
        check.sponsor_change_located(),
        None,
        "a shape with no change role was asked whether its change was located",
    );
}

/// Every committed output carries a range proof, and every fee output
/// carries none.
///
/// Two assertions rather than one weakened to "some entries carry
/// proofs". A blinded output that lost its proof is exactly what this
/// check exists to catch, and a fee output that GREW one would be a fee
/// that had been blinded — the failure the fee role was built to make
/// impossible, and the one worth a second assertion of its own.
/// The ENTRY CROSSING against a real node.
///
/// Explicit receipts spent into two blinded destinations. This workspace
/// has performed the shape every ceremony as a FUNDING step; what is new
/// is that the coin it spends sits at a receipt constructor's program,
/// so the transfer is governed by the covenant rather than by the
/// adapter.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_entry_crossing_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::EntryCrossing, "multi-entry-crossing");
}

/// The EXIT CROSSING against a real node.
///
/// Blinded receipts spent into explicit destinations beside the blinded
/// absorber a nonzero consumed blinder sum requires. It is registered
/// exactly as the six homogeneous shapes are, because it runs through
/// the same ceremony: the crossing changes what is built, not how it is
/// funded, signed or submitted.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_exit_crossing_shape_is_submitted_to_a_real_target() {
    use vectors::live_multi_shapes::PrivateShape;

    run_one_multi_shape(PrivateShape::ExitCrossing, "multi-exit-crossing");
}

fn assert_proofs_match_the_shape(
    shape: vectors::live_multi_shapes::PrivateShape,
    record: &vectors::live_multi_shapes::MultiShapeRecord,
) {
    // TWO kinds of output carry an explicit value and therefore no range
    // proof, and they are counted separately rather than added together.
    // A fee has no program and an explicit destination has one, so a
    // ceremony that built a fee where a destination belonged would still
    // satisfy a single combined count -- and that substitution is
    // precisely the one that turns a spendable receipt into value the
    // chain treats as paid away.
    let explicit = shape.fee_output_count() + shape.explicit_destination_count();
    let proving = record.output_count() - explicit;
    assert_eq!(
        record
            .output_witness_proof_bytes()
            .iter()
            .filter(|bytes| **bytes > 2)
            .count(),
        proving,
        "a committed output carried no range proof: {:?}",
        record.output_witness_proof_bytes(),
    );
    assert_eq!(
        record
            .output_witness_proof_bytes()
            .iter()
            .filter(|bytes| **bytes == 0)
            .count(),
        explicit,
        "an explicit-valued output's witness entry is empty, and only one's is: {:?}",
        record.output_witness_proof_bytes(),
    );
}

/// One multi-output or multi-input private shape, against the node.
///
/// The same shape-only discipline the one-to-one control ran under: what
/// the node decided is written into the artifact and asserted nowhere, so
/// a lane that asserted an acceptance would fail rather than report on the
/// day the honest answer changed. What is asserted is first-party
/// construction facts — the shape's own input and output counts, that the
/// candidate reached the node, and, where an acceptance was observed, the
/// two-origin agreement.
fn run_one_multi_shape(shape: vectors::live_multi_shapes::PrivateShape, extension: &str) {
    use vectors::live_multi_shapes::{MultiShapePlanner, render_multi_shape};

    let ceremony = match extension {
        "multi-entry-crossing" => CeremonyId::MultiEntryCrossing,
        "multi-exit-crossing" => CeremonyId::MultiExitCrossing,
        "multi-many-to-many" => CeremonyId::MultiManyToMany,
        "multi-one-to-one-with-fee" => CeremonyId::MultiOneToOneWithFee,
        "multi-private-merge" => CeremonyId::MultiPrivateMerge,
        "multi-pure-split" => CeremonyId::MultiPureSplit,
        "multi-several-owners" => CeremonyId::MultiSeveralOwners,
        "multi-split" => CeremonyId::MultiSplit,
        "multi-strict-one-to-one" => CeremonyId::MultiStrictOneToOne,
        _ => panic!("unknown multi-shape ceremony"),
    };
    let mut capture_guard = CaptureGuard::new(ceremony);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some(extension));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(ceremony, report.as_deref())),
    );

    let mut planner = MultiShapePlanner::for_shape(shape, identifier(&genesis))
        .expect("the shape ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_multi_shape(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts = CeremonyCaptureFacts::from_capture(ceremony, &capture)
        .with_digest("predecessor", record.predecessor_digest())
        .with_digest("successor", record.successor_digest());
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as one.
    // It is never a target verdict, so it is reported and the test stops
    // here rather than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the shape ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    assert_multi_shape(shape, record, &rendered);
}

fn assert_multi_shape(
    shape: vectors::live_multi_shapes::PrivateShape,
    record: &vectors::live_multi_shapes::MultiShapeRecord,
    rendered: &str,
) {
    // The predecessor is the confidential one the ceremony asked for,
    // and its COUNT is the shape's own choice of predecessor rather than
    // a constant: the merge funds a three-output predecessor because a
    // two-output one funded from an explicit input can only offer it an
    // inverse pair.
    //
    // An EXPLICITLY funded shape names a predecessor it never funds, so
    // its width is the count it asked the node for rather than the
    // fixture's. Stated as its own case rather than folded into the
    // other: the two are different questions, and a single expression
    // covering both would stop catching a confidential ceremony that
    // funded the wrong predecessor.
    let expected_coins = match shape.explicit_funding() {
        Some((outputs, _)) => outputs as usize,
        None => shape.predecessor().outputs(),
    };
    assert_eq!(
        record.coins().len(),
        expected_coins,
        "the node funded a predecessor of a different width than the shape asked for",
    );
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_private_restart::RestartConfidentialCoin::matches_expectation),
        "the node reported a confidential coin the ceremony did not ask for",
    );

    // The shape's own cardinalities, read off the record rather than the
    // shape's name: the receipts it consumed and the outputs it created.
    assert_eq!(
        record.receipt_leaves(),
        record.input_count(),
        "the consumed receipt count is the shape's input count",
    );
    assert_eq!(
        record.output_witness_proof_bytes().len(),
        record.output_count(),
        "one output-witness entry per created output",
    );
    assert_proofs_match_the_shape(shape, record);

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    // The two origins, where an acceptance was observed.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.verified(),
            "the accepted witness does not verify against the recomputed message",
        );
    }

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_no_negative_case true"));
    assert!(rendered.contains("moves_the_sponsor_row false"));
}

// --- §15.1: the positive explicit table, one run per row ---

/// One §15.1 shape, funded, submitted, mined and read back.
///
/// The explicit-lane sibling of [`run_one_multi_shape`], and deliberately
/// the same shape of function: the two lanes differ in what they build and
/// not in how a run is judged. Nothing here asserts what the node decided.
/// What it asserts is that a run completed, that the shape the ceremony
/// reports is the shape it was asked for, and — where an acceptance
/// happened — that the two origins agree.
fn run_one_explicit_shape(shape: vectors::live_explicit_shapes::ExplicitShape, extension: &str) {
    use vectors::live_explicit_shapes::{ExplicitShapePlanner, render_explicit_shape};

    let ceremony = match extension {
        "explicit-boundary-values" => CeremonyId::ExplicitBoundaryValues,
        "explicit-maximum-inputs" => CeremonyId::ExplicitMaximumInputs,
        "explicit-maximum-outputs" => CeremonyId::ExplicitMaximumOutputs,
        "explicit-merge" => CeremonyId::ExplicitMerge,
        "explicit-normalization" => CeremonyId::ExplicitNormalization,
        "explicit-one-destination-owner" => CeremonyId::ExplicitOneDestinationOwner,
        "explicit-one-to-one" => CeremonyId::ExplicitOneToOne,
        "explicit-repeated-owner" => CeremonyId::ExplicitRepeatedOwner,
        "explicit-self-paid-fee" => CeremonyId::ExplicitSelfPaidFee,
        "explicit-several-destination-owners" => CeremonyId::ExplicitSeveralDestinationOwners,
        "explicit-several-owners" => CeremonyId::ExplicitSeveralOwners,
        "explicit-several-to-several" => CeremonyId::ExplicitSeveralToSeveral,
        "explicit-split" => CeremonyId::ExplicitSplit,
        "explicit-sponsorless" => CeremonyId::ExplicitSponsorless,
        _ => panic!("unknown explicit-shape ceremony"),
    };
    let mut capture_guard = CaptureGuard::new(ceremony);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some(extension));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(ceremony, report.as_deref())),
    );

    let mut planner = ExplicitShapePlanner::for_shape(shape, identifier(&genesis))
        .expect("the shape ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_explicit_shape(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts = CeremonyCaptureFacts::from_capture(ceremony, &capture);
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    // A construction refusal is a valid outcome and is written down as
    // one. It is never a target verdict, so the run stops here rather
    // than pretending the node said anything.
    if let Some(refusal) = record.refusal() {
        panic!("the explicit shape ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    assert_explicit_shape(shape, record, &rendered);
}

fn assert_explicit_shape(
    shape: vectors::live_explicit_shapes::ExplicitShape,
    record: &vectors::live_explicit_shapes::ExplicitShapeRecord,
    rendered: &str,
) {
    // The shape's own cardinalities, read off the record rather than off
    // the shape's name.
    assert_eq!(
        record.coins().len(),
        shape.input_count(),
        "the node funded a different number of coins than the shape asked for",
    );
    assert!(
        record
            .coins()
            .iter()
            .all(vectors::live_explicit_shapes::ObservedShapeCoin::matches_expectation),
        "the node reported a coin the ceremony did not ask for",
    );
    assert_eq!(record.input_count(), shape.input_count());
    assert_eq!(record.output_count(), shape.output_count());

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    // The two origins, where an acceptance was observed. Both are
    // conditions on an acceptance rather than assertions that one
    // happened.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.every_input_verified(),
            "an accepted signature does not verify against its recomputed message",
        );
        assert_eq!(
            check.inputs().len(),
            shape.input_count(),
            "the read-back copy carries a witness for every input",
        );
    }

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_no_negative_case true"));
    assert!(rendered.contains("builds_no_sponsor_region true"));
}

/// §15.1 `one-input-to-one-output`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_one_to_one_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::OneToOne, "explicit-one-to-one");
}

/// §15.1 `one-input-split-into-two`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_split_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::SplitIntoTwo, "explicit-split");
}

/// §15.1 `several-inputs-merged-into-one`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_merge_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::MergedIntoOne, "explicit-merge");
}

/// §15.1 `several-inputs-to-several-outputs`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_several_to_several_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::SeveralToSeveral,
        "explicit-several-to-several",
    );
}

/// §15.1 `repeated-owner`: one owner authorizes two separate inputs.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_repeated_owner_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::RepeatedOwner, "explicit-repeated-owner");
}

/// §15.1 `several-distinct-owners`: two inputs under two published owners.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_several_owners_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::SeveralDistinctOwners,
        "explicit-several-owners",
    );
}

/// §15.1 `one-destination-owner`: every output created for one owner.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_one_destination_owner_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::OneDestinationOwner,
        "explicit-one-destination-owner",
    );
}

/// §15.1 `several-destination-owners`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_several_destination_owners_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::SeveralDestinationOwners,
        "explicit-several-destination-owners",
    );
}

/// §15.1 `semantic-boundary-values`: the smallest destination the request
/// type admits, and the remainder.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_boundary_values_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::SemanticBoundaryValues,
        "explicit-boundary-values",
    );
}

/// §15.1 `canonical-input-normalization`: the receipts offered in the
/// reverse of their canonical order.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_normalization_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(
        ExplicitShape::CanonicalInputNormalization,
        "explicit-normalization",
    );
}

/// §15.1 `sponsorless`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_sponsorless_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::Sponsorless, "explicit-sponsorless");
}

/// §15.1 `candidate-maximum-inputs`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_maximum_inputs_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::MaximumInputs, "explicit-maximum-inputs");
}

/// §15.1 `candidate-maximum-outputs`.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_maximum_outputs_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::MaximumOutputs, "explicit-maximum-outputs");
}

/// The owner fee matrix's last cell: sponsorless, EXPLICIT, self-paid.
///
/// Not a §15.1 row. The explicit positive table is complete at sixteen
/// rows and a transfer that pays its own fee is none of the sixteen
/// classes, so what this run answers is the owner fee matrix and its
/// evidence is the explicit run of record.
///
/// # What only this run can establish
///
/// The explicit conservation leaf carries the fee as a TERM for
/// fee-bearing shapes, and until this run nothing had ever executed that
/// clause: the leaf was emitted at link time by a vocabulary no request
/// could select, because the explicit lane declared no fee destination.
/// A leaf that is emitted and never run is a leaf whose arithmetic has
/// been reviewed and never checked against a target, so this submission
/// is the first thing that can tell the two apart.
///
/// It is also the first sponsorless form to face RELAY on its own. The
/// sponsorless forms before it paid no fee and travelled as package
/// children, which is why the ABI builds them at the topology-restricted
/// version; a form that pays its own fee needs no package parent, and
/// what the node does with it at that version is recorded here rather
/// than predicted.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_explicit_self_paid_fee_shape_is_submitted_to_a_real_target() {
    use vectors::live_explicit_shapes::ExplicitShape;

    run_one_explicit_shape(ExplicitShape::SelfPaidFee, "explicit-self-paid-fee");
}

/// §15.3's two witness-content rows, with their control, on one chain.
///
/// # Why the control and the mutants are one test
///
/// A refusal is attributable to a row's own class only when the
/// unmutated form is ACCEPTED and the mutated form is refused. This run
/// submits the unmutated one-input one-output candidate first, then the
/// same finalized candidate twice more with its one signature position
/// offering something else — nothing at all, and then bytes of the
/// selected width that are not a signature.
///
/// Nothing here asserts what the node decided. What it asserts is that
/// all three submissions were answered and that the mutants differ from
/// the control in one run of bytes, which is the condition under which
/// the verdicts mean anything at all.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_witness_content_negatives_are_offered_beside_their_control() {
    use vectors::live_explicit_shapes::{ExplicitShapePlanner, render_explicit_shape};

    let mut capture_guard = CaptureGuard::new(CeremonyId::ExplicitWitnessNegatives);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("explicit-witness-negatives"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::ExplicitWitnessNegatives,
            report.as_deref(),
        )),
    );

    let mut planner = ExplicitShapePlanner::for_witness_negatives(identifier(&genesis))
        .expect("the negative ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_explicit_shape(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts = CeremonyCaptureFacts::from_capture(CeremonyId::ExplicitWitnessNegatives, &capture);
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    if let Some(refusal) = record.refusal() {
        panic!("the negative ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // Every case was answered.
    assert!(
        record.observed_layer().is_some(),
        "the control was answered"
    );
    assert_eq!(
        record.negatives().len(),
        2,
        "one observation per witness-content mutation",
    );

    // The attributability condition, measured rather than argued: each
    // mutant differs from the control in exactly one run of bytes.
    for negative in record.negatives() {
        assert!(
            negative.differs_from_control_in_one_item(),
            "{:?} does not differ from the control in one run of bytes",
            negative.mutation(),
        );
    }

    // Where the control was accepted, the two origins hold for it.
    if let Some(check) = record.reverification() {
        assert!(check.readback_matches_submission());
        assert!(check.every_input_verified());
    }
}

// --- §15.1: the sponsored pair, one run per row ---

/// One sponsored shape, funded, signed, submitted, mined and read back.
///
/// The sponsored sibling of [`run_one_explicit_shape`] and deliberately
/// the same shape of function. What it asserts is that a run completed,
/// that the sponsor round trip bound to the exact finalized bytes, and —
/// where an acceptance happened — that the node's own copy agrees with
/// what was handed to it and carries the shape that was asked for.
///
/// It asserts nothing about what the node decided. A refusal is written
/// down as the target typed it and the run stops, which is what a typed
/// stop is made of.
fn run_one_sponsor_shape(
    shape: vectors::live_sponsor_shapes::SponsorShape,
    value_form: vectors::live_sponsor_shapes::SponsorValueForm,
    extension: &str,
) {
    use vectors::live_sponsor_shapes::{
        SponsorShapePlanner, SponsorValueForm, render_sponsor_shape,
    };

    let ceremony = match extension {
        "sponsored-change-absent" => CeremonyId::SponsoredChangeAbsent,
        "sponsored-change-present" => CeremonyId::SponsoredChangePresent,
        "sponsored-committed-value" => CeremonyId::SponsoredCommittedValue,
        _ => panic!("unknown sponsor-shape ceremony"),
    };
    let mut capture_guard = CaptureGuard::new(ceremony);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some(extension));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(ceremony, report.as_deref())),
    );

    let mut planner = match value_form {
        SponsorValueForm::Explicit => SponsorShapePlanner::for_shape(shape, identifier(&genesis)),
        SponsorValueForm::Committed => {
            SponsorShapePlanner::for_committed_sponsor_value(shape, identifier(&genesis))
        }
    }
    .expect("the sponsored ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_sponsor_shape(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts = sponsor_capture_facts(ceremony, &capture, planner.capture_predecessor_digest());
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    if let Some(refusal) = record.refusal() {
        panic!("the sponsored ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    judge_one_sponsor_shape(record, &rendered, shape, value_form);
}

/// What a completed sponsored run must hold.
///
/// Split from the arranging on the rule the sponsor ceremony already
/// followed before it was lifted: a function that both arranges a run
/// and judges it makes the judging hard to read past the arranging.
fn judge_one_sponsor_shape(
    record: &vectors::live_sponsor_shapes::SponsorShapeRecord,
    rendered: &str,
    shape: vectors::live_sponsor_shapes::SponsorShape,
    value_form: vectors::live_sponsor_shapes::SponsorValueForm,
) {
    // The sponsor round trip happened, and it bound to the exact bytes.
    let round = record.round().expect("the sponsor round trip completed");
    assert!(
        round.echo_matches(),
        "the adapter signed bytes other than the ones it was sent",
    );
    assert!(
        round.replay_changed_the_control(),
        "the replay changed nothing, so no returned witness reached the control",
    );
    assert!(
        round.witness_reached_the_control(),
        "a returned witness item is absent from the replayed control",
    );

    // The binding is enforced rather than announced: the same witness,
    // bound to one mutated byte, is refused.
    let mutated = round
        .mutated_refusal()
        .expect("a signature bound to mutated bytes was refused");
    assert!(
        mutated.contains("SponsorSignatureBindingMismatch"),
        "the refusal names something other than the binding: {mutated}",
    );

    // The candidate reached the node.
    assert!(record.submitted_bytes() > 0);
    assert!(record.observed_layer().is_some(), "no layer was observed");

    // The origins, where an acceptance was observed. Every one is a
    // condition ON an acceptance rather than an assertion that one
    // happened.
    if let Some(check) = record.reverification() {
        assert!(
            check.readback_matches_submission(),
            "the bytes the node reported are not the bytes it was handed",
        );
        assert!(
            check.every_owner_verified(),
            "an accepted owner signature does not verify against its recomputed message",
        );
        assert!(
            check.sponsor_witness_in_readback(),
            "the adapter's witness is absent from the node's own copy",
        );

        // The change role, read out of the node's copy rather than off
        // the request. A with-change run that reached a node carrying no
        // change output is already a hard stop in the ceremony; this
        // states the positive half where a reader of the test sees it.
        match shape.change() {
            None => assert!(
                check.change().is_none(),
                "a without-change run carries a change output",
            ),
            Some(offered) => {
                let change = check.change().expect("a with-change run carries change");
                assert!(
                    change.is_the_declared_change(offered),
                    "the change output is not the one the deployment declares",
                );
            }
        }
    }

    judge_the_sponsor_value_form(record, rendered, value_form);

    assert!(rendered.contains("evidences_no_negative_case true"));
}

/// What the sponsor's VALUE FORM obliges the run to have observed.
///
/// Split from [`judge_one_sponsor_shape`] because it judges a different
/// axis. That function asks what any sponsored run owes — a round trip
/// bound to its own bytes, a candidate that reached the node, and the
/// change role where an acceptance happened. This asks what THIS run's
/// sponsor coin was, and the two grew independent enough that reading
/// one past the other had become the work.
fn judge_the_sponsor_value_form(
    record: &vectors::live_sponsor_shapes::SponsorShapeRecord,
    rendered: &str,
    value_form: vectors::live_sponsor_shapes::SponsorValueForm,
) {
    use vectors::live_sponsor_shapes::{CommittedSponsorCheck, SponsorValueForm};

    // The value form, judged where a reader of the test sees it.
    match value_form {
        SponsorValueForm::Explicit => {
            assert!(
                record.committed().is_none(),
                "an explicit run reported a committed sponsor census",
            );
            assert!(
                record.sponsor_funded().is_some(),
                "an explicit run observed no amount for its sponsor coin",
            );
            // The run says in its own bytes what it did not establish.
            assert!(rendered.contains("does_not_establish confidential-sponsor-values"));
        }
        SponsorValueForm::Committed => {
            let ceremony = "sponsored-committed-value";
            let census = record
                .committed()
                .expect("a committed run reported no committed sponsor census");
            // Named one at a time, so a failure says WHICH check failed
            // rather than that some did.
            for check in CommittedSponsorCheck::ALL {
                assert!(
                    census.holds(check),
                    "the committed sponsor coin failed the check {}",
                    check.name(),
                );
            }
            assert!(census.every_check_held());
            // The coin exists on a chain: its funding transaction was
            // accepted and mined, and the node computed an identity for
            // it. That is a separate acceptance from the candidate's and
            // is not a substitute for one.
            assert_eq!(
                record.committed_funding_txid(),
                Some(current_semantic_value(
                    ceremony,
                    "committed_sponsor_funding_txid",
                )),
                "the committed sponsor coin was mined under another identity",
            );
            assert_eq!(
                record.committed_funding_weight(),
                Some(current_semantic_u64(
                    ceremony,
                    "committed_sponsor_funding_weight",
                )),
            );
            // No amount was observed for the coin the control spends,
            // which is the whole difference the axis makes.
            assert!(
                record.sponsor_funded().is_none(),
                "a committed run reported an amount for a coin whose value is a point",
            );
            // And the disclaimer moved with the subject: this run
            // establishes that confidential sponsor values are FUNDED,
            // so it no longer says it establishes nothing about them.
            assert!(!rendered.contains("does_not_establish confidential-sponsor-values"));
            assert!(rendered.contains("does_not_establish confidential-receipt-values"));
            assert!(rendered.contains("committed_sponsor_every_check_held true"));

            // The candidate that SPENDS the coin is refused, and the
            // refusal is the finding rather than a disappointment. An
            // ACCEPTANCE here would mean the arithmetic below is wrong,
            // which is why it is asserted rather than tolerated.
            //
            // The reserve sub-equation is the sponsor input against the
            // fee and the change. The input carries a blinder now, both
            // outputs that spend it are explicit and carry none, and
            // nothing in the transaction absorbs the difference — so the
            // target's balance check cannot close whatever the amounts
            // are.
            let [candidate] = current_outcome_projections(ceremony) else {
                panic!("the current committed-sponsor ceremony submits one candidate")
            };
            assert_eq!(
                record.observed_layer(),
                Some(candidate.layer()),
                "a committed sponsor value was not refused at the balance check",
            );
            let accepted_identity = candidate
                .target_identity()
                .map(|identity| identity.to_target_display());
            assert_eq!(record.accepted_txid(), accepted_identity.as_deref());
            assert!(
                rendered.contains(candidate.detail()),
                "the target named something other than its balance check",
            );

            // Attributability, MEASURED. A candidate names the coin it
            // spends by outpoint alone, so the value form is not in
            // these bytes at all: this submission and the explicit
            // control's are the same shape at the same width, and the
            // refusal is attributable to which coin was reached for.
            assert_eq!(record.submitted_bytes(), candidate.submitted_bytes().len(),);
            let control = sole_current_acceptance("sponsored-change-present");
            assert_eq!(record.submitted_bytes(), control.submitted_bytes().len(),);
            let candidate_weight = current_semantic_u64(ceremony, "target_weight");
            assert_eq!(
                candidate_weight,
                current_transaction(candidate.submitted_bytes()).weight(),
            );
            assert_eq!(record.target_weight(), Some(candidate_weight),);
            let control_weight = current_semantic_u64("sponsored-change-present", "target_weight");
            assert_eq!(
                control_weight,
                current_transaction(control.submitted_bytes()).weight(),
            );
            assert_eq!(record.target_weight(), Some(control_weight),);
        }
    }
}

/// §15.1 `sponsor-change-absent`: the sponsor funds the fee exactly.
///
/// The ceremony the sponsor wave ran, now driven from a lane rather than
/// from inside a test. It is kept because it is the lift's own control:
/// what it builds did not move, so its recorded identity must not
/// either.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsored_change_absent_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsor_shapes::SponsorShape;

    run_one_sponsor_shape(
        SponsorShape::ChangeAbsent,
        vectors::live_sponsor_shapes::SponsorValueForm::Explicit,
        "sponsored-change-absent",
    );
}

/// §15.1 `sponsor-change-present`: the sponsor takes change back.
///
/// The first sponsored control in this workspace that takes change. The
/// sponsor coin is funded ABOVE the offer and the offer states the
/// residue, which is the whole of what was missing.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsored_change_present_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsor_shapes::SponsorShape;

    run_one_sponsor_shape(
        SponsorShape::ChangePresent,
        vectors::live_sponsor_shapes::SponsorValueForm::Explicit,
        "sponsored-change-present",
    );
}

/// The with-change sponsored control, funded by a sponsor coin whose
/// VALUE is committed.
///
/// The first transaction this workspace offers a target that spends a
/// blinded sponsor value. Its asset stays explicit, because the covenant
/// introspects it; its fee stays explicit, because consensus defines a
/// fee by its explicitness; and the receipts stay explicit, because this
/// run varies ONE thing against the control above it and the receipts
/// are not it.
///
/// What the run has to establish before it reaches a node is that the
/// value the chain holds for the sponsor coin is the value this
/// workspace derives from published constants and no chain at all. That
/// check is in the ceremony rather than here, and it stops the run
/// rather than reporting a finding: a run whose two copies disagreed
/// would have funded something blinded while being unable to say what.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_committed_sponsor_value_shape_is_submitted_to_a_real_target() {
    use vectors::live_sponsor_shapes::{SponsorShape, SponsorValueForm};

    run_one_sponsor_shape(
        SponsorShape::ChangePresent,
        SponsorValueForm::Committed,
        "sponsored-committed-value",
    );
}

/// §15.6 `missing-sponsor-authorization`, the mutant offered FIRST.
///
/// One run, one chain, two submissions: a sponsored control whose sponsor
/// input carries no authorization, and then the unmutated control. The
/// order is the evidence — a sponsor-witness mutation leaves the identity
/// alone, so a control submitted first would make its own mutant come
/// back `txn-already-known` at a layer before script evaluation.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_missing_sponsor_authorization_negative_is_refused_behind_its_control() {
    use vectors::live_sponsor_shapes::{SponsorShape, SponsorShapePlanner, render_sponsor_shape};

    let mut capture_guard = CaptureGuard::new(CeremonyId::SponsoredMissingAuthorization);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("sponsored-missing-authorization"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::SponsoredMissingAuthorization,
            report.as_deref(),
        )),
    );

    let mut planner = SponsorShapePlanner::for_missing_authorization_negative(
        SponsorShape::ChangeAbsent,
        identifier(&genesis),
    )
    .expect("the sponsored ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_sponsor_shape(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let sponsor_input = record
        .round()
        .map(vectors::live_sponsor_shapes::SponsorRoundTrip::input)
        .map(usize::from);
    let mut facts =
        CeremonyCaptureFacts::from_capture(CeremonyId::SponsoredMissingAuthorization, &capture);
    if let Some(input_index) = sponsor_input {
        facts = facts.with_locator(
            "submit-unauthorized-sponsor-control",
            LiveMutationLocator::WitnessItem {
                input_index,
                item_index: 0,
            },
        );
    }
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);

    if let Some(refusal) = record.refusal() {
        panic!("the sponsored ceremony refused before the node: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    // The mutant was offered and answered.
    assert_eq!(
        record.negatives().len(),
        1,
        "one observation per mutant offering",
    );
    let negative = &record.negatives()[0];

    // Attributability, measured rather than argued: the mutant and its
    // control come from ONE finalization and differ in the sponsor
    // witness and in nothing else.
    assert!(
        negative.differs_from_control_in_the_sponsor_witness(),
        "the mutant differs from its control somewhere other than the sponsor witness",
    );

    // A sponsored control whose sponsor input authorizes nothing must
    // not be accepted. This is the finding if it fires.
    assert_ne!(
        negative.layer(),
        target_elements_conformance::protocol::ObservedOutcomeLayer::Accepted,
        "a sponsor input carrying no authorization was ACCEPTED",
    );
    assert!(
        negative.detail().is_some_and(|detail| !detail.is_empty()),
        "the target refused and said nothing, so the row has no verdict to cite",
    );

    // And the CONTROL that followed was accepted, which is the whole of
    // what makes the refusal attributable rather than merely recorded.
    let check = record
        .reverification()
        .expect("the unmutated control was accepted behind the mutant");
    assert!(check.readback_matches_submission());
    assert!(check.every_owner_verified());
    assert!(check.sponsor_witness_in_readback());

    assert!(rendered.contains("evidences_no_negative_case false"));
}

// --- §16.1: the pairs arc, one fixture materialized twice -------------

/// The PAIRS ARC: one §16.1 semantic fixture, both its materializations
/// submitted to ONE node, and the relation over the two acceptances.
///
/// # Why this is one test and cannot be two
///
/// §6.6 asks a pair's members to carry the same exact explicit `U`, and
/// each of these ignored tests spins its own disposable chain and issues
/// its own asset. Two tests would therefore be two assets and two chains,
/// and the pair's fifth term would disagree for a reason that has nothing
/// to do with representation. So the arc issues once, submits the
/// explicit member, hands the private half the asset the first one
/// issued, and submits the private member — one ceremony, one node.
///
/// # What this run establishes that no earlier run could
///
/// The campaign already had an accepted explicit one-to-one and an
/// accepted private strict one-to-one, and they are not a pair: §16.1
/// requires ONE semantic fixture materialized twice, and those two are
/// independent ceremonies whose shapes merely match. Both members here
/// read the same fixture and neither shape carries a literal of its own,
/// so what is submitted is the pair rather than two things that resemble
/// one.
///
/// # Nothing here decides what the node should have said
///
/// The assertions are about a run that COMPLETED and about the arc's own
/// record of it. Where a member was not accepted the arc refuses before
/// it has a ledger, and the refusal is reported with the target's own
/// words in the transcript rather than asserted away.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_pairs_arc_submits_both_members_of_one_fixture_to_a_real_target() {
    use vectors::live_pair_arc::{PairArcPlanner, render_pair_arc};

    let mut capture_guard = CaptureGuard::new(CeremonyId::PairsArc);
    let executor =
        environment("TRIPOD_LIVE_EXECUTOR").expect("TRIPOD_LIVE_EXECUTOR names the adapter to run");
    let network = environment("TRIPOD_LIVE_NETWORK_ID")
        .expect("TRIPOD_LIVE_NETWORK_ID states the bound development network");
    let genesis = environment("TRIPOD_LIVE_GENESIS_ID")
        .expect("TRIPOD_LIVE_GENESIS_ID states the chain the run is bound to");
    let report = legacy_report(Some("pairs-arc"));

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_LIVE_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(&capture_diagnostics(
            CeremonyId::PairsArc,
            report.as_deref(),
        )),
    );

    let mut planner = PairArcPlanner::new(identifier(&genesis)).expect("the arc ceremony builds");
    let started = Instant::now();
    let (outcome, capture) = execute_and_capture(&target, &binding, &configuration, &mut planner);
    let wall = started.elapsed();

    let record = planner.record();
    let rendered = render_pair_arc(record);
    if let Some(report) = report.as_deref() {
        std::fs::write(report, &rendered).expect("the transcript is written");
        std::fs::write(
            timing_path(report),
            format!("wall_seconds {:.1}\n", wall.as_secs_f64()),
        )
        .expect("the run's wall time is written");
    }
    let facts = pair_capture_facts(&planner, &capture);
    write_capture_before_gates(&mut capture_guard, &capture, &facts, &rendered);
    if let (Some(report), Err(error)) = (report.as_deref(), &outcome) {
        std::fs::write(
            report.with_extension("executor-refusal"),
            format!("{error}\n"),
        )
        .expect("the executor's refusal is written");
    }

    if let Some(refusal) = record.refusal() {
        panic!("the pairs arc refused before it had a ledger: {refusal:?}");
    }
    outcome.expect("the ceremony reached the target");

    assert_pair_arc(record, &rendered);
}

fn pair_capture_facts(
    planner: &vectors::live_pair_arc::PairArcPlanner,
    capture: &NativeOperationCapture,
) -> CeremonyCaptureFacts {
    let mut facts = CeremonyCaptureFacts::from_capture(CeremonyId::PairsArc, capture);
    if let Some(programs) =
        planner.capture_destination_programs(vectors::live_pair_arc::PairArcMember::Explicit)
        && let Ok(projection) = pair_projection_input(&programs)
    {
        facts = facts.with_projection(RequestRole::PairedExplicit, projection);
    }
    if let Some(programs) =
        planner.capture_destination_programs(vectors::live_pair_arc::PairArcMember::Private)
        && let Ok(projection) = pair_projection_input(&programs)
    {
        facts = facts.with_projection(RequestRole::PairedPrivate, projection);
    }
    facts
}

fn assert_pair_arc(record: &vectors::live_pair_arc::PairArcRecord, rendered: &str) {
    // The arc's own fixture, and the fact every later claim rests on.
    assert!(record.fixture_conserves());

    let ledger = record.ledger().expect("a completed arc writes a ledger");

    // ONE asset, which is what one issuance buys and what §6.6's fifth
    // term needs.
    assert_eq!(
        record.issued_asset(),
        Some(ledger.issued_asset()),
        "the ledger names an asset the run did not issue",
    );
    assert_eq!(
        ledger.explicit().projection().explicit_asset(),
        ledger.private().projection().explicit_asset(),
        "the two members do not carry one exact explicit U",
    );

    // Both members at the acceptance bar every recorded run meets: an
    // identity the target computed, a copy read back equal to what it was
    // handed, and every input's signature verified against a message
    // recomputed here.
    for member in [ledger.explicit(), ledger.private()] {
        assert!(
            member.meets_the_acceptance_bar(),
            "{:?} did not meet the acceptance bar",
            member.member(),
        );
    }

    // The relation, in the row's own terms.
    let observation = ledger.observation();
    assert!(
        observation.projections_are_equal(),
        "a §6.6 term disagrees: {:?}",
        observation.terms(),
    );

    // And the pair is evidence of MINIMALITY rather than of similarity:
    // the private member withholds exact amounts the explicit member
    // publishes. A pair whose private half published everything would
    // establish nothing about disclosure.
    assert!(observation.terms_withheld_by_the_private_member() > 0);
    assert!(
        ledger
            .explicit()
            .projection()
            .publishes_every_exact_amount()
    );
    assert!(
        !ledger.private().projection().publishes_every_exact_amount(),
        "the private member published an exact amount",
    );
    assert!(ledger.supports_the_projection_equality_row());

    assert_the_arc_ledger_matches_the_run_of_record(record);

    // The run says in its own bytes what it did not establish.
    assert!(rendered.contains("evidences_no_negative_case true"));
    assert!(rendered.contains("builds_no_sponsor_region true"));
}

/// The arc's ledger, held against the identities and figures its own run
/// of record carries.
///
/// # Why this was unbound, and why binding it costs no report
///
/// The assertions above recompute PROPERTIES — the acceptance bar, the
/// projection relation, the disclosure asymmetry — and the strongest
/// thing they said about identity was that the two accepted txids
/// DIFFER. Two txids differ in every run, so the exact provenance the
/// phase card and `PairedRelationObserved` cite could move while the
/// lane and the static standing both stayed green. That is F3's
/// silent-drift hazard on the row this arc answers. The transcript is
/// written to disk BEFORE any assertion in this test runs, so binding
/// these costs no report on the day a figure changes: the artifact
/// carries what the node said either way, and the lane now FAILS instead
/// of passing over a record nothing reads.
fn assert_the_arc_ledger_matches_the_run_of_record(record: &vectors::live_pair_arc::PairArcRecord) {
    use vectors::live_pair_arc::REPRESENTATION_EQUIVALENCE_TERMS;

    let acceptances = current_acceptance_projections("pairs-arc");
    let [explicit, private] = acceptances else {
        panic!("the current pairs arc carries two accepted members")
    };
    let ledger = record.ledger().expect("a completed arc writes a ledger");
    assert_eq!(
        record.ledger().is_some(),
        !acceptances.is_empty(),
        "the flag the evidence matrix reads disagrees with the run",
    );

    // ONE asset, and the one the run of record names. §6.6's exact
    // explicit U term is a claim about THIS asset, not about some asset.
    assert_eq!(
        Some(ledger.issued_asset()),
        current_ceremony_projection("pairs-arc").issued_asset_display(),
        "the arc issued an asset the run of record does not carry",
    );

    // The two accepted identities, each against its own recorded
    // constant. These are the figures the phase card prints and the
    // paired-relation standing rests on, and they are what a reader
    // checks against a chain.
    let explicit_identity =
        current_semantic_line_value("pairs-arc", "member explicit ", "accepted_txid");
    assert_eq!(explicit.identity_display(), explicit_identity);
    assert_eq!(
        Some(ledger.explicit().accepted_txid()),
        Some(explicit_identity),
        "the explicit member was accepted at an identity the run of record does not carry",
    );
    let private_identity =
        current_semantic_line_value("pairs-arc", "member private ", "accepted_txid");
    assert_eq!(private.identity_display(), private_identity);
    assert_eq!(
        Some(ledger.private().accepted_txid()),
        Some(private_identity),
        "the private member was accepted at an identity the run of record does not carry",
    );
    assert_ne!(
        ledger.explicit().accepted_txid(),
        ledger.private().accepted_txid(),
        "the two members are one transaction",
    );

    // The two widths and the two weights, each the target's own figure.
    // The eight-fold gap between them is the REPRESENTATION, and it is
    // the measurement the arc exists to make.
    let explicit_bytes = usize::try_from(current_semantic_line_u64(
        "pairs-arc",
        "member explicit ",
        "submitted_bytes",
    ))
    .expect("the current explicit member's width fits usize");
    assert_eq!(explicit.submitted_bytes().len(), explicit_bytes);
    assert_eq!(
        ledger.explicit().submitted_bytes(),
        explicit_bytes,
        "the explicit member handed the node a different number of bytes",
    );
    let private_bytes = usize::try_from(current_semantic_line_u64(
        "pairs-arc",
        "member private ",
        "submitted_bytes",
    ))
    .expect("the current private member's width fits usize");
    assert_eq!(private.submitted_bytes().len(), private_bytes);
    assert_eq!(
        ledger.private().submitted_bytes(),
        private_bytes,
        "the private member handed the node a different number of bytes",
    );
    let explicit_weight = current_semantic_line_u64("pairs-arc", "member explicit ", "weight");
    assert_eq!(
        explicit_weight,
        current_transaction(explicit.submitted_bytes()).weight(),
    );
    assert_eq!(
        ledger.explicit().target_weight(),
        Some(explicit_weight),
        "the target computed a weight for the explicit member the run of record does not carry",
    );
    let private_weight = current_semantic_line_u64("pairs-arc", "member private ", "weight");
    assert_eq!(
        private_weight,
        current_transaction(private.submitted_bytes()).weight(),
    );
    assert_eq!(
        ledger.private().target_weight(),
        Some(private_weight),
        "the target computed a weight for the private member the run of record does not carry",
    );

    // The withheld count, EXACTLY, and out of the full §6.6 term list
    // rather than out of however many terms happened to be compared. Two
    // of eleven is the whole disclosure-minimality claim: zero would mean
    // the private representation published everything, and a shortened
    // term list would let a compared-nothing run report the same two.
    let observation = ledger.observation();
    assert_eq!(
        observation.terms().len(),
        REPRESENTATION_EQUIVALENCE_TERMS.len(),
        "the pair was compared on fewer §6.6 terms than the section names",
    );
    let withheld = usize::try_from(current_semantic_u64(
        "pairs-arc",
        "terms_withheld_by_the_private_member",
    ))
    .expect("the current pair's withheld-term count fits usize");
    assert_eq!(
        observation.terms_withheld_by_the_private_member(),
        withheld,
        "the private member withheld a different number of terms than the run of record carries",
    );
}
