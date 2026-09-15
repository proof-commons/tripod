//! Shared enhanced native capture writer and its original supporting vocabulary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use target_elements::DeploymentEnvironment;
use target_elements_conformance::executor::NativeOperationCapture;
use target_elements_conformance::protocol::{
    ExecutorCapability, FundingCustodyProfile, FundingMaterializerProfile,
    FundingRepresentationProfile, NativeVerdict, ObservedOutcomeLayer, OperationSubject,
    WireEnvironment, WireExecutionDomain,
};
use vectors::live_report::{LiveMutantKind, LiveMutationLocator, LiveWitnessPathRole};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum CeremonyId {
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
    SplitCommitmentNegatives,
    SponsoredChangeAbsent,
    SponsoredChangePresent,
    SponsoredCommittedValue,
    SponsoredMissingAuthorization,
    SponsoredOwnerSigningNegatives,
    SponsoredPrivateExplicitNoChange,
    SponsoredPrivateWithChange,
    ConfidentialPredecessorSetup,
}

impl CeremonyId {
    pub(crate) const ALL: [Self; 43] = [
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
        Self::SplitCommitmentNegatives,
        Self::SponsoredChangeAbsent,
        Self::SponsoredChangePresent,
        Self::SponsoredCommittedValue,
        Self::SponsoredMissingAuthorization,
        Self::SponsoredOwnerSigningNegatives,
        Self::SponsoredPrivateExplicitNoChange,
        Self::SponsoredPrivateWithChange,
        Self::ConfidentialPredecessorSetup,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
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
            Self::SplitCommitmentNegatives => "split-commitment-negatives",
            Self::SponsoredChangeAbsent => "sponsored-change-absent",
            Self::SponsoredChangePresent => "sponsored-change-present",
            Self::SponsoredCommittedValue => "sponsored-committed-value",
            Self::SponsoredMissingAuthorization => "sponsored-missing-authorization",
            Self::SponsoredOwnerSigningNegatives => "sponsored-owner-signing-negatives",
            Self::SponsoredPrivateExplicitNoChange => "sponsored-private-explicit-no-change",
            Self::SponsoredPrivateWithChange => "sponsored-private-with-change",
            Self::ConfidentialPredecessorSetup => "confidential-predecessor",
        }
    }

    /// The name of the `#[ignore]` test that runs this ceremony.
    ///
    /// Split in two at the explicit family. Neither half is arbitrary: the
    /// TAIL is exhaustive, so a ceremony added tomorrow must be given an
    /// arm there or the crate does not compile, which is the property the
    /// split was not allowed to cost.
    pub(crate) const fn rust_test_name(self) -> &'static str {
        match self {
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
            other => other.rust_test_name_beyond_the_explicit_family(),
        }
    }

    /// What the explicit half returns for a variant it does not answer.
    ///
    /// Unreachable through [`Self::rust_test_name`], which never delegates
    /// an explicit variant here. It exists so the match below can stay
    /// exhaustive over the whole enum while still being the half that does
    /// not answer for the explicit family, and if it were ever reached the
    /// roster's own uniqueness gate would fail on the duplicate.
    pub(crate) const ANSWERED_BY_THE_EXPLICIT_HALF: &'static str = "answered-by-the-explicit-half";

    /// The test name for every ceremony outside the explicit family.
    pub(crate) const fn rust_test_name_beyond_the_explicit_family(self) -> &'static str {
        match self {
            Self::ConservationNegatives => {
                "conservation_is_recorded_against_a_control_the_proof_negatives_mutate"
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
            Self::SplitCommitmentNegatives => {
                "one_copied_value_commitment_is_refused_before_the_split_control_is_accepted"
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
            Self::SponsoredOwnerSigningNegatives => {
                "the_three_sponsor_range_rearrangements_are_refused_behind_their_control"
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
            Self::ExplicitBoundaryValues
            | Self::ExplicitMaximumInputs
            | Self::ExplicitMaximumOutputs
            | Self::ExplicitMerge
            | Self::ExplicitNormalization
            | Self::ExplicitOneDestinationOwner
            | Self::ExplicitOneToOne
            | Self::ExplicitRepeatedOwner
            | Self::ExplicitSelfPaidFee
            | Self::ExplicitSeveralDestinationOwners
            | Self::ExplicitSeveralOwners
            | Self::ExplicitSeveralToSeveral
            | Self::ExplicitSplit
            | Self::ExplicitSponsorless
            | Self::ExplicitWitnessNegatives => Self::ANSWERED_BY_THE_EXPLICIT_HALF,
        }
    }

    pub(crate) const fn is_setup(self) -> bool {
        matches!(self, Self::ConfidentialPredecessorSetup)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RequestRole {
    Acceptance,
    Control,
    Refusal,
    PairedExplicit,
    PairedPrivate,
    Auxiliary,
}

impl RequestRole {
    pub(crate) const fn as_str(self) -> &'static str {
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
pub(crate) struct CaptureDigest {
    pub(crate) name: &'static str,
    pub(crate) value: [u8; 32],
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProjectionInput {
    pub(crate) owners: Vec<Vec<u8>>,
    pub(crate) amounts: Vec<u64>,
    pub(crate) destinations: BTreeMap<Vec<u8>, (Vec<u8>, u64)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CeremonyOperationFacts {
    pub(crate) operation_id: String,
    pub(crate) case_step: String,
    pub(crate) role: RequestRole,
    pub(crate) control_request_id: Option<String>,
    pub(crate) control_identity: Option<String>,
    pub(crate) mutant: Option<LiveMutantKind>,
    pub(crate) locator: Option<LiveMutationLocator>,
    pub(crate) projection: Option<ProjectionInput>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CeremonyCaptureFacts {
    pub(crate) digests: Vec<CaptureDigest>,
    pub(crate) operations: Vec<CeremonyOperationFacts>,
}

impl CeremonyCaptureFacts {
    pub(crate) fn from_capture(ceremony: CeremonyId, capture: &NativeOperationCapture) -> Self {
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

    pub(crate) fn with_digest(mut self, name: &'static str, value: Option<[u8; 32]>) -> Self {
        if let Some(value) = value {
            self.digests.push(CaptureDigest { name, value });
        }
        self
    }

    pub(crate) fn operation(&self, operation_id: &str) -> Option<&CeremonyOperationFacts> {
        self.operations
            .iter()
            .find(|operation| operation.operation_id == operation_id)
    }
}

pub(crate) fn request_role(
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

pub(crate) const fn is_negative_ceremony(ceremony: CeremonyId) -> bool {
    matches!(
        ceremony,
        CeremonyId::ConservationNegatives
            | CeremonyId::ExplicitWitnessNegatives
            | CeremonyId::KeypathProbe
            | CeremonyId::OffsettingFlowNegatives
            | CeremonyId::OwnerSigningNegatives
            | CeremonyId::SplitCommitmentNegatives
            | CeremonyId::SponsoredMissingAuthorization
            | CeremonyId::SponsoredOwnerSigningNegatives
    )
}

pub(crate) fn control_submission(
    ceremony: CeremonyId,
    capture: &NativeOperationCapture,
    submissions: &[usize],
) -> Option<usize> {
    let named = match ceremony {
        CeremonyId::ConservationNegatives => Some("submit-balance-valid-control"),
        CeremonyId::KeypathProbe => Some("script-path-control"),
        CeremonyId::OffsettingFlowNegatives => Some("submit-two-in-two-out-control"),
        CeremonyId::OwnerSigningNegatives => Some("vault-control-entitlement-control"),
        CeremonyId::SplitCommitmentNegatives => Some("submit-split-control"),
        CeremonyId::SponsoredMissingAuthorization => Some("submit-sponsor-signed-control"),
        CeremonyId::SponsoredOwnerSigningNegatives => Some("submit-sponsored-control"),
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

pub(crate) fn mutation_fact(step: &str) -> (Option<LiveMutantKind>, Option<LiveMutationLocator>) {
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
        // The same FIELD KIND as the row above and deliberately so: the
        // two share a class and an arithmetic, and what separates them is
        // the output index together with the shape it sits on.
        "copied-commitment" => (
            LiveMutantKind::CopiedCommitment,
            field(2, SerializedOutputField::ValueCommitment),
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

pub(crate) fn structural_mutation_fact(
    step: &str,
) -> (Option<LiveMutantKind>, Option<LiveMutationLocator>) {
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
        // The three sponsor-range rows separate by RANGE and by nothing
        // else: they ride one sponsored control, sit behind one leaf's
        // signature check, and draw the same generic equality failure. The
        // ranges below are placeholders in the same sense every other
        // witnessless row's are — the ceremony measures each one over the
        // bytes it actually submitted and files it through the capture's
        // own locator, which is the only value a declaration may carry.
        "sponsor-receipt-range-exchange" => (
            LiveMutantKind::ReceiptSponsorRangeExchange,
            LiveMutationLocator::WitnesslessRange { start: 0, end: 0 },
        ),
        "sponsor-change-in-protocol-range" => (
            LiveMutantKind::SponsorChangeInProtocolRange,
            LiveMutationLocator::WitnesslessRange { start: 0, end: 0 },
        ),
        "sponsor-protocol-overlap" => (
            LiveMutantKind::SponsorProtocolOverlap,
            LiveMutationLocator::WitnesslessRange { start: 0, end: 0 },
        ),
        _ => return arrangement_mutation_fact(step),
    };
    (Some(fact.0), Some(fact.1))
}

/// The facts the owner-signing ceremony's leaf arrangements declare.
///
/// Split from its caller when the sponsor-range family pushed that
/// function past the line bound, but taken at the seam that was already
/// there rather than by moving the newest arrivals out. Every arm here
/// declares a COMMITTED LEAF ARRANGEMENT — which input reveals which
/// committed leaf — while every arm left behind declares a byte range or
/// a transaction shape. That is a difference in the kind of fact and not
/// in how many lines it takes to write, so the three move together and a
/// fourth arrangement joins them here.
///
/// The chain's contract is unchanged: a step this half does not answer
/// falls through to the same `(None, None)` the caller used to return, so
/// every step keeps the fact it had. The exhaustive-tail idiom the
/// ceremony test-name match uses does not transfer, because that match is
/// over an enum the compiler can check and this one is over a string
/// where no arm set is exhaustive; total coverage is what is preserved.
pub(crate) fn arrangement_mutation_fact(
    step: &str,
) -> (Option<LiveMutantKind>, Option<LiveMutationLocator>) {
    let fact = match step {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaptureDestination {
    pub(crate) directory: PathBuf,
    pub(crate) short_sha: String,
    pub(crate) suite_commit: String,
    pub(crate) suite_tree: String,
}

impl CaptureDestination {
    pub(crate) fn new(
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

    pub(crate) fn capture_path(
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
pub(crate) struct CaptureGuard {
    pub(crate) ceremony: CeremonyId,
    pub(crate) destination: Option<CaptureDestination>,
    pub(crate) path: Option<PathBuf>,
    pub(crate) transcript_written: bool,
}

impl CaptureGuard {
    pub(crate) fn for_test(ceremony: CeremonyId, destination: CaptureDestination) -> Self {
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

    pub(crate) fn capture_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub(crate) const fn mark_transcript_written(&mut self) {
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

pub(crate) fn write_capture_before_gates(
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

pub(crate) fn render_enhanced_capture(
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

pub(crate) fn render_run_identity(
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

pub(crate) fn render_executor_context(
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

pub(crate) fn render_digests(out: &mut String, facts: &CeremonyCaptureFacts) {
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

pub(crate) fn render_operations(
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

pub(crate) fn validate_capture_facts(
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

pub(crate) fn validate_control_attribution(
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

pub(crate) fn render_domains(
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

pub(crate) fn render_leaf_versions(
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

pub(crate) fn render_capabilities(out: &mut String, capabilities: &BTreeSet<ExecutorCapability>) {
    let _ = writeln!(out, "environment-capability-count {}", capabilities.len());
    for (ordinal, capability) in capabilities.iter().copied().enumerate() {
        let _ = writeln!(
            out,
            "environment-capability {ordinal} {}",
            executor_capability(capability),
        );
    }
}

pub(crate) fn render_funding_advertisement(
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

pub(crate) fn render_locator(out: &mut String, locator: Option<&LiveMutationLocator>) {
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

pub(crate) fn render_projection(out: &mut String, projection: Option<&ProjectionInput>) {
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

pub(crate) fn write_text_field(out: &mut String, field: &str, value: &str) {
    let _ = writeln!(
        out,
        "{field} {} {}",
        value.len(),
        hex_bytes(value.as_bytes()),
    );
}

pub(crate) fn write_optional_text_field(out: &mut String, field: &str, value: Option<&str>) {
    write_text_field(out, field, value.unwrap_or_default());
}

pub(crate) fn write_optional_control_id(out: &mut String, field: &str, value: Option<&str>) {
    match value {
        Some(value) => write_text_field(out, field, value),
        None => {
            let _ = writeln!(out, "{field} none");
        }
    }
}

pub(crate) fn write_bytes_field(out: &mut String, field: &str, value: &[u8]) {
    let _ = writeln!(out, "{field} {} {}", value.len(), hex_bytes(value));
}

pub(crate) fn decimal_list(values: &[usize]) -> String {
    let members = values
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("{}:{members}", values.len())
}

pub(crate) fn byte_list(values: &[Vec<u8>]) -> String {
    let members = values
        .iter()
        .map(|value| format!("{}:{}", value.len(), hex_bytes(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{}:{members}", values.len())
}

pub(crate) fn require_lower_hex(value: &str, width: usize, name: &str) -> Result<(), String> {
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

pub(crate) fn hex_bytes(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    out
}

pub(crate) const fn deployment_environment(environment: DeploymentEnvironment) -> &'static str {
    match environment {
        DeploymentEnvironment::Development => "development",
        _ => "unknown",
    }
}

pub(crate) const fn wire_environment(environment: WireEnvironment) -> &'static str {
    match environment {
        WireEnvironment::Development => "development",
        _ => "unknown",
    }
}

pub(crate) const fn execution_domain(domain: WireExecutionDomain) -> &'static str {
    match domain {
        WireExecutionDomain::Tapscript => "tapscript",
        _ => "unknown",
    }
}

pub(crate) fn target_contract(version: target_elements::TargetContractVersion) -> &'static str {
    if version == target_elements::TargetContractVersion::V2 {
        "elements-tapscript-v2"
    } else {
        "unknown"
    }
}

pub(crate) const fn executor_capability(capability: ExecutorCapability) -> &'static str {
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
        ExecutorCapability::TestScriptPathAuthorization => "test-script-path-authorization",
        ExecutorCapability::ConfidentialValueTestFunding => "confidential-value-test-funding",
        ExecutorCapability::ConfidentialValueSponsorAuthorization => {
            "confidential-value-sponsor-authorization"
        }
        _ => "unknown",
    }
}

pub(crate) const fn funding_representation(profile: FundingRepresentationProfile) -> &'static str {
    match profile {
        FundingRepresentationProfile::ExplicitAssetConfidentialValue => {
            "explicit-asset-confidential-value"
        }
        _ => "unknown",
    }
}

pub(crate) const fn funding_custody(profile: FundingCustodyProfile) -> &'static str {
    match profile {
        FundingCustodyProfile::CentralPublicFixtures => "central-public-fixtures",
        _ => "unknown",
    }
}

pub(crate) const fn funding_materializer(profile: FundingMaterializerProfile) -> &'static str {
    match profile {
        FundingMaterializerProfile::GuideCtfDeterministicV1 => "guide-ctf-deterministic-v1",
        _ => "unknown",
    }
}

pub(crate) const fn response_verdict(verdict: NativeVerdict) -> &'static str {
    match verdict {
        NativeVerdict::Accepted => "accepted",
        NativeVerdict::Rejected => "refused",
        _ => "incomplete",
    }
}

pub(crate) const fn response_layer(layer: ObservedOutcomeLayer) -> &'static str {
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

pub(crate) const fn witness_path_role(role: LiveWitnessPathRole) -> &'static str {
    match role {
        LiveWitnessPathRole::KeyPath => "key-path",
        LiveWitnessPathRole::ScriptPath => "script-path",
    }
}

pub(crate) const SHA256_INITIAL: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

pub(crate) const SHA256_ROUND: [u32; 64] = [
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

pub(crate) fn sha256(input: &[u8]) -> [u8; 32] {
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

pub(crate) fn environment(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

pub(crate) fn identifier(text: &str) -> [u8; 32] {
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
pub(crate) fn timing_path(report: &Path) -> PathBuf {
    let mut path = report.to_path_buf();
    let mut name = path.file_name().map_or_else(
        || "live-native-run".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    name.push_str(".timing");
    path.set_file_name(name);
    path
}

pub(crate) fn test_directory(label: &str) -> PathBuf {
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

pub(crate) fn test_capture_destination(directory: &Path) -> CaptureDestination {
    CaptureDestination::new(
        directory.to_path_buf(),
        "abc123".to_owned(),
        "1".repeat(40),
        "2".repeat(40),
    )
    .expect("the test capture destination is valid")
}

pub(crate) fn scripted_adapter(capabilities: &[&str], responses: &[String]) -> String {
    // The current protocol revision keeps scripted exchanges aligned with the harness.
    let schema = target_elements_conformance::protocol::NATIVE_PROTOCOL_SCHEMA;
    let capabilities = capabilities
        .iter()
        .map(|capability| format!("\"{capability}\""))
        .collect::<Vec<_>>()
        .join(",");
    let network = std::iter::repeat_n("17", 32).collect::<Vec<_>>().join(",");
    let genesis = std::iter::repeat_n("34", 32).collect::<Vec<_>>().join(",");
    let handshake = format!(
        "{{\"protocol_schema\":{schema},\"adapter_name\":\"capture-adapter\",\"adapter_version\":\"1.2.3\",\"framework_revision\":\"framework-tip\",\"node_name\":\"elementsd\",\"node_version\":\"23.2.1\",\"binary_reported_revision\":\"binary-tip\",\"intended_executed_tip\":\"intended-tip\",\"upstream_base\":\"upstream-base\",\"included_local_topics\":[\"topic-a\",\"topic-b\"],\"supported_domains\":[\"tapscript\"],\"supported_leaf_versions\":[196],\"capabilities\":[{capabilities}],\"confidential_funding\":{{\"representation_profiles\":[\"explicit_asset_confidential_value\"],\"custody_profiles\":[\"central_public_fixtures\"],\"materializer_profiles\":[\"guide_ctf_deterministic_v1\"],\"reproducibility_contracts\":[\"byte_identity\"]}}}}",
    );
    let observed_environment = format!(
        "{{\"schema\":{schema},\"environment\":\"development\",\"chain_name\":\"elementsregtest\",\"network_id\":[{network}],\"genesis_id\":[{genesis}],\"active_domains\":[\"tapscript\"],\"active_leaf_versions\":[196]}}",
    );
    let mut script = format!(
        "#!/bin/sh\nIFS= read -r request\nprintf '%s\\n' '{handshake}'\nprintf '%s\\n' '{observed_environment}'\n",
    );
    for response in responses {
        let _ = writeln!(script, "IFS= read -r request\nprintf '%s\\n' '{response}'");
    }
    script
}
