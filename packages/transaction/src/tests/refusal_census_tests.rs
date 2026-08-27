//! Exhaustiveness guards for the two refusal vocabularies that had
//! none.
//!
//! # What this file guards, and what it deliberately does not
//!
//! The owner census's refusals have carried
//! `every_refusal_variant_is_reached_by_a_test_in_this_file` since they
//! were written, and the consensus-exclusion register records what its
//! absence cost the two vocabularies here: the issuance-proof refusal
//! was reached by no test anywhere in the repository, and the peg-in
//! refusal's witness-field raise site was likewise unreached while its
//! outpoint-marker site was covered. Neither gap was visible, and the
//! reason neither was visible is that nothing failed when a variant went
//! unexercised.
//!
//! The guard below is the compile-time half of the answer. Each function
//! matches its whole vocabulary with **no wildcard arm**, so a variant
//! added to either enum stops this crate compiling until somebody edits
//! this file — which is the point at which they read the paragraph
//! telling them a test is owed. A catch-all here would have made the
//! guard silent again, so there is none, and neither enum acquires one.
//!
//! What it does not do is claim every variant is exercised. A match arm
//! is not a test, and a guard that asserted coverage it had not measured
//! would be exactly the false assurance the register warned a reviser
//! against. The two gaps the register named are closed by tests in the
//! files that own those refusals, not here; this file makes the *next*
//! gap loud instead of invisible.
//!
//! # Why the arms are grouped rather than named one by one
//!
//! One arm per vocabulary, listing every variant. The alternative — one
//! arm per variant returning its own name — buys a second copy of a
//! hundred and six identifiers that the compiler already checks, and
//! every copy is a place for a name to drift. What the guard needs from
//! the match is exhaustiveness, and exhaustiveness is a property of the
//! pattern set rather than of the arm bodies.

use crate::error::TransactionRefusal as Refusal;
use crate::live_materialize::MaterializationRefusal as Materialization;

/// Every decoder and construction refusal, matched with no catch-all.
///
/// The body does nothing and is meant to: the guard is the pattern list,
/// and a variant added to [`Refusal`] makes this fail to compile.
///
/// The length is the whole point and cannot be shortened without
/// shortening the guard. Splitting the match across two functions would
/// need a catch-all in each for either to compile, which is exactly what
/// this file exists not to have.
#[expect(clippy::too_many_lines, reason = "one pattern per censused variant")]
fn every_transaction_refusal_is_censused(refusal: &Refusal) {
    match refusal {
        Refusal::BundleIsNotACandidate
        | Refusal::ContractRevisionMismatch
        | Refusal::MissingLayout(..)
        | Refusal::MissingLeaf(..)
        | Refusal::MissingInputRole { .. }
        | Refusal::MissingOutputRole { .. }
        | Refusal::MissingDeploymentSymbol { .. }
        | Refusal::MalformedDeploymentSymbol { .. }
        | Refusal::CoordinatorNotAtAnchor { .. }
        | Refusal::MalformedPinnedProgram { .. }
        | Refusal::PinnedInternalKeyMismatch
        | Refusal::PinnedLeafVersionMismatch { .. }
        | Refusal::EmptyAshSelection
        | Refusal::DuplicateOutpoint(..)
        | Refusal::DuplicatePublicOutputView(..)
        | Refusal::DuplicateSponsorOutpoint(..)
        | Refusal::OverlappingOutpoint(..)
        | Refusal::UnsupportedShape { .. }
        | Refusal::AshInputCarriesForeignAsset(..)
        | Refusal::AshInputCarriesForeignProgram(..)
        | Refusal::AshInputAmountNotExplicit(..)
        | Refusal::SponsorInputAssetNotExplicit(..)
        | Refusal::SponsorInputCarriesForeignAsset(..)
        | Refusal::SponsorProgramClassNotAdmitted(..)
        | Refusal::SponsorChangeWithoutSponsor
        | Refusal::SponsorRequestedWithoutCapability
        | Refusal::SponsorCapabilityWithoutRequest
        | Refusal::EmptySponsorOffer
        | Refusal::SuccessorAmountOutOfRange
        | Refusal::SponsorValueDoesNotCoverFee
        | Refusal::SponsorSignatureMissing(..)
        | Refusal::SponsorSignatureBindingMismatch(..)
        | Refusal::SponsorWitnessShapeRefused { .. }
        | Refusal::ConservationFailed { .. }
        | Refusal::ResourceBoundExceeded { .. }
        | Refusal::LayoutCensusMismatch { .. }
        | Refusal::TruncatedTargetBytes { .. }
        | Refusal::TrailingTargetBytes { .. }
        | Refusal::NonMinimalCompactSize { .. }
        | Refusal::UnrecognizedFieldPrefix { .. }
        | Refusal::UnrecognizedWitnessFlag { .. }
        | Refusal::SuperfluousWitnessRecord
        | Refusal::MalformedAssetIdentifier { .. }
        | Refusal::OutpointIndexOutOfRange { .. }
        | Refusal::IssuanceInputRefused
        | Refusal::PeginInputRefused
        | Refusal::IssuanceProofRefused
        | Refusal::SurjectionProofRefused
        | Refusal::RangeProofRefused
        | Refusal::RangeProofRequired { .. }
        | Refusal::OutputWitnessCensusMismatch { .. }
        | Refusal::EmptyInputCensus
        | Refusal::EmptyOutputCensus
        | Refusal::WitnessCensusMismatch { .. }
        | Refusal::LiveBundleIsNotACandidate
        | Refusal::LiveContractRevisionMismatch
        | Refusal::LiveLeafVersionDisagreement
        | Refusal::MissingLiveLeaf(..)
        | Refusal::MissingLiveShapeRanges { .. }
        | Refusal::MissingLiveFamilyRange { .. }
        | Refusal::LiveCoordinatorNotAtAnchor { .. }
        | Refusal::MissingLiveDeploymentSymbol { .. }
        | Refusal::MalformedLiveDeploymentSymbol { .. }
        | Refusal::LiveInternalKeyMalformed { .. }
        | Refusal::OwnerKeyIsNotACurvePoint { .. }
        | Refusal::DestinationOutputKeyUndetermined
        | Refusal::InheritedObligationUnaccounted
        | Refusal::DestinationValueIsZero
        | Refusal::EmptyReceiptSelection
        | Refusal::DuplicateReceiptOutpoint(..)
        | Refusal::EmptyDestinationCensus
        | Refusal::SponsorChangeWithoutSponsoredForm
        | Refusal::DeclaredRolesDoNotCoverDestinations { .. }
        | Refusal::SelfPaidFeeUnderSponsoredForm
        | Refusal::SelfPaidFeeDeclaredMoreThanOnce { .. }
        | Refusal::SelfPaidFeeHasNoShapePosition
        | Refusal::SelfPaidFeePositionDisagreesWithShape { .. }
        | Refusal::PublicTestRandomnessWithoutPrivateForm
        | Refusal::PrivateFormWithoutPublicTestRandomness
        | Refusal::DestinationOwnerHasNoConstructor { .. }
        | Refusal::RepresentationNotLinked
        | Refusal::LiveSponsorRequestedWithoutCapability
        | Refusal::LiveSponsorCapabilityWithoutRequest
        | Refusal::EmptyLiveSponsorOffer
        | Refusal::SponsorChangeRequestedWithoutDestination
        | Refusal::SponsorChangeOfferedWithoutRequest
        | Refusal::LiveFormDisagreesWithShape
        | Refusal::UnsupportedLiveShape { .. }
        | Refusal::SponsorOverlapsReceiptFamily(..)
        | Refusal::MissingPublicReceiptView(..)
        | Refusal::ReceiptInputIsNotALiveReceipt(..)
        | Refusal::ReceiptInputCarriesForeignAsset(..)
        | Refusal::ReceiptInputValueFormRefused(..)
        | Refusal::MissingPublicSponsorView(..)
        | Refusal::LiveSponsorInputCarriesForeignAsset(..)
        | Refusal::DestinationTotalOutOfRange
        | Refusal::LiveConservationFailed { .. }
        | Refusal::ReceiptPositionOutsideFamily { .. }
        | Refusal::ConfidentialConstructionModelNotAdmitted
        | Refusal::PrivateValueCapabilityAbsent
        | Refusal::PrivateValueCapabilityWithoutPrivateForm
        | Refusal::DestinationValueCommitmentUndetermined { .. }
        | Refusal::PrivateFinalizationIsNotTheExplicitLane { .. }
        | Refusal::PrivateOpeningsDoNotCoverTheRequest { .. }
        | Refusal::PrivateOpeningRegionDisagreesWithPosition { .. }
        | Refusal::PrivateSponsorRegionAmountDisagreesWithOffer { .. }
        | Refusal::PrivateMaterializationRefused(..)
        | Refusal::OwnerResponseMissing { .. }
        | Refusal::OwnerResponseDuplicated { .. }
        | Refusal::UnexpectedSigner { .. }
        | Refusal::ResponseFromWrongOwner { .. }
        | Refusal::ResponseForWrongInput { .. }
        | Refusal::ResponseUnderWrongSighashProfile { .. }
        | Refusal::ResponseBoundToDifferentBytes { .. }
        | Refusal::OutputMutatedAfterSigning { .. }
        | Refusal::InputExtendedAfterSigning { .. }
        | Refusal::OutputOmittedAfterSigning { .. } => (),
    }
}

/// Every materialization refusal, matched with no catch-all.
///
/// The same guard for the second vocabulary the register named. This
/// enum is the transaction-wide materializer's, and it lands closed with
/// no catch-all by its own charter — which made the absence of a guard
/// the one way a variant could still be added and never reached.
fn every_materialization_refusal_is_censused(refusal: &Materialization) {
    match refusal {
        Materialization::PredecessorOpeningMissing { .. }
        | Materialization::PredecessorOpeningMismatch { .. }
        | Materialization::SemanticValueImbalance
        | Materialization::ValueBlinderImbalance
        | Materialization::InvalidScalar { .. }
        | Materialization::InvalidCommitment { .. }
        | Materialization::BoundedParitySearchExhausted
        | Materialization::NonceMaterializationFailed { .. }
        | Materialization::RangeproofMaterializationFailed { .. }
        | Materialization::SerializationRoundTripMismatch
        | Materialization::PostFinalizationMutation { .. }
        | Materialization::UnknownFixtureHandle { .. }
        | Materialization::FixtureDigestMismatch { .. }
        | Materialization::FixtureBindingAmbiguous { .. }
        | Materialization::FixtureOutputOrderMismatch
        | Materialization::SignerInputCensusExceedsPositionDomain
        | Materialization::DuplicateInputOutpoint { .. }
        | Materialization::IncompleteFamilyClassification
        | Materialization::ProtocolAssetMismatch { .. }
        | Materialization::ConfidentialProtocolAsset { .. }
        | Materialization::NonzeroProtocolAssetBlinder { .. }
        | Materialization::NonProtocolRegionOverlapsProtocol { .. }
        | Materialization::NonProtocolRegionAffectsProtocolBalance
        | Materialization::UnsupportedProfileCombination
        | Materialization::IndependentCommitmentMismatch { .. }
        | Materialization::IndependentCommitmentOriginNotDistinct { .. }
        | Materialization::ProofBindingMismatch { .. }
        | Materialization::RangeproofEmpty { .. }
        | Materialization::UnexpectedSurjectionProof { .. }
        | Materialization::OutputWitnessCensusMismatch
        | Materialization::OpeningBindingCensusMismatch
        | Materialization::SignerInputWouldExposeOpening { .. }
        | Materialization::PerOutputMaterializationRefused
        | Materialization::FeeOutputProgramNotEmpty { .. }
        | Materialization::FeeOutputCarriesAnOpening { .. }
        | Materialization::FeeOutputValueZero { .. }
        | Materialization::FeeOutputNotRecognizable { .. }
        | Materialization::OutputProgramEmpty { .. }
        | Materialization::ExplicitDestinationIsAFee { .. } => (),
    }
}

#[test]
fn both_refusal_vocabularies_are_censused_by_a_match_with_no_catch_all() {
    // The test exists so the guard is reachable from a test run and so
    // this paragraph has somewhere to live; the guarantee is the two
    // matches above, and it is a compile-time one. Calling each guard on
    // one variant is enough to keep the functions used, and which
    // variant is deliberately not significant.
    //
    // The two chosen are the ones the register recorded as unreached,
    // because if either name ever stops existing this is one of the
    // places that has to be edited.
    every_transaction_refusal_is_censused(&Refusal::IssuanceProofRefused);
    every_materialization_refusal_is_censused(&Materialization::SemanticValueImbalance);
}
