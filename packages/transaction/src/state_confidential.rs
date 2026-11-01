//! Balanced confidential sponsorship for the maturity announcement.
//!
//! Exactly one committed sponsor input funds one committed change output
//! and an explicit fee. A bare asserted sum binds no individual opening
//! in a multiple-input offer, so that shape is refused. The STATE successor
//! stays explicit, and sponsor erasure leaves every relation's inputs
//! unchanged. The independent collaborator solves the change scalar over
//! an empty set of other blinders and checks both value commitments.
//!
//! Sponsor-local material is borrowed during construction and is absent
//! from the returned finalized candidate, refusals and signing requests.
//! The materializer supplies nonce and proof bytes once, without retry;
//! the completed output-witness vector is frozen before any signature.

use tapscript::StateCurveCapability;
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::bytes::{
    AssetId, NonceField, Outpoint, OutputWitness, TargetOutput, TargetTransaction, ValueField,
};
use crate::error::TransactionRefusal;
use crate::live_materialize::{
    CommitmentOrigin, ConfidentialProofMaterializer, IndependentCommitmentCheck,
    MaterializerCommitment, RangeproofRequest, SCALAR_BYTES, commitments_agree,
};
use crate::state_abi::CandidateMaturityAnnouncementAbi;
use crate::state_construct::{MaturitySponsorContext, construct_maturity_candidate};
use crate::state_finalize::{FinalizedMaturityAnnouncement, finalize_maturity_announcement};
use crate::state_request::MaturityAnnouncementRequest;
use crate::state_view::ValidatedMaturityStateView;

/// Borrowed construction material for the sole sponsor coin.
///
/// The input sum is that coin's value blinder. No result retains this record.
pub struct MaturityConfidentialSponsorMaterial {
    /// The offered coin this material opens.
    pub outpoint: Outpoint,
    /// The sponsor-local amount used to balance change and fee.
    pub semantic_input_amount: u64,
    /// The single consumed coin's value blinder, in scalar encoding.
    pub input_blinder_sum: [u8; SCALAR_BYTES],
    /// Injected nonce material; canonical fixtures derive it deterministically.
    pub nonce_input: [u8; SCALAR_BYTES],
    /// Injected proof seed; canonical fixtures derive it deterministically.
    pub rangeproof_seed: [u8; SCALAR_BYTES],
}

/// The ordinary STATE construction inputs and its sponsor-local proof material.
pub struct MaturityConfidentialConstructionInputs<'a, C: StateCurveCapability> {
    /// The reviewed target definition.
    pub target: &'a ReviewedElementsTapscriptDefinition,
    /// The derived announcement ABI.
    pub abi: &'a CandidateMaturityAnnouncementAbi,
    /// The validated public predecessor view.
    pub validated: &'a ValidatedMaturityStateView,
    /// The sponsored announcement request with change.
    pub request: &'a MaturityAnnouncementRequest,
    /// The supplied STATE constructor curve.
    pub curve: &'a C,
    /// Public spent fields and the sponsor's offer and destination.
    pub sponsor: &'a MaturitySponsorContext<'a>,
    /// Private construction inputs, never copied into the finalized form.
    pub material: &'a MaturityConfidentialSponsorMaterial,
}

impl<C: StateCurveCapability> Copy for MaturityConfidentialConstructionInputs<'_, C> {}

impl<C: StateCurveCapability> Clone for MaturityConfidentialConstructionInputs<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Which value commitment a construction refusal identifies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityConfidentialCommitment {
    /// The sole sponsor input's retained value.
    SponsorInput,
    /// The change at output one.
    SponsorChange,
}

/// A refusal carries public coordinates and reasons, never construction material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaturityConfidentialRefusal {
    /// The shared constructor or strict encoding boundary refused.
    Construction(Box<TransactionRefusal>),
    /// The offer does not contain exactly one committed input and positive change.
    UnsupportedSponsorShape,
    /// The fee exceeds the input or the explicit offered change disagrees.
    AmountImbalance,
    /// The independent solve returned no nonzero scalar.
    InvalidBalancingBlinder,
    /// The empty-slice solve did not return the input sum.
    BalancingBlinderMismatch,
    /// The checker is not an independent first-party origin.
    IndependentOriginNotAdmitted,
    /// An origin could not compute a value commitment.
    CommitmentUnavailable {
        /// The public role of the commitment.
        coordinate: MaturityConfidentialCommitment,
        /// The collaborator that could not answer.
        origin: CommitmentOrigin,
    },
    /// The two origins disagree about a value commitment.
    CommitmentDisagreement {
        /// The public role of the commitment.
        coordinate: MaturityConfidentialCommitment,
    },
    /// The computed input commitment differs from the retained spent value.
    SponsorOpeningMismatch,
    /// The offered committed change differs from the computed commitment.
    OfferedChangeMismatch,
    /// The materializer could not produce the nonce field.
    NonceUnavailable,
    /// The materializer could not produce the range proof.
    RangeproofUnavailable,
    /// The returned proof names different commitment, asset or program bytes.
    ProofBindingMismatch,
    /// The committed change has no range proof.
    RangeproofEmpty,
    /// The explicit-asset form cannot carry a surjection proof.
    UnexpectedSurjectionProof,
    /// The completed candidate failed strict decoding or decoded to a different value.
    SerializationRoundTripMismatch,
}

impl From<TransactionRefusal> for MaturityConfidentialRefusal {
    fn from(refusal: TransactionRefusal) -> Self {
        Self::Construction(Box::new(refusal))
    }
}

/// Construct, prove and finalize the balanced confidential sponsored STATE form.
///
/// The offer is read once by the shared constructor. Only the sponsor change
/// becomes committed; the fee and successor remain explicit. Neither a
/// partial candidate nor its construction material escapes this boundary.
///
/// # Errors
/// Returns a shared construction refusal, an unsupported sponsor shape,
/// an amount or scalar imbalance, an origin or commitment disagreement,
/// missing or misbound proof material, or a strict serialization refusal.
///
/// # Panics
/// Panics only if a linked bundle lost its retained constructor, which a link cannot arrange.
pub fn construct_confidential_maturity_announcement(
    inputs: MaturityConfidentialConstructionInputs<'_, impl StateCurveCapability>,
    materializer: &dyn ConfidentialProofMaterializer,
    independent: &dyn IndependentCommitmentCheck,
) -> Result<FinalizedMaturityAnnouncement, MaturityConfidentialRefusal> {
    let construction = construct_maturity_candidate(
        inputs.target,
        inputs.abi,
        inputs.validated,
        inputs.request,
        inputs.curve,
        Some(inputs.sponsor),
        true,
    )?;
    let material = inputs.material;
    let [spent] = construction.sponsor_spent() else {
        return Err(MaturityConfidentialRefusal::UnsupportedSponsorShape);
    };
    if spent.outpoint() != material.outpoint
        || !matches!(spent.value(), ValueField::Commitment(_))
        || construction.sponsor_change_position() != Some(1)
        || construction.fee_position() != Some(2)
    {
        return Err(MaturityConfidentialRefusal::UnsupportedSponsorShape);
    }
    let candidate = construction.transaction();
    let change = &candidate.outputs()[1];
    let ValueField::Explicit(fee) = candidate.outputs()[2].value() else {
        return Err(MaturityConfidentialRefusal::UnsupportedSponsorShape);
    };
    let amount = material
        .semantic_input_amount
        .checked_sub(fee)
        .ok_or(MaturityConfidentialRefusal::AmountImbalance)?;
    if amount == 0 {
        return Err(MaturityConfidentialRefusal::UnsupportedSponsorShape);
    }
    if let ValueField::Explicit(offered) = change.value()
        && offered != amount
    {
        return Err(MaturityConfidentialRefusal::AmountImbalance);
    }
    if !independent.origin().may_check_a_construction()
        || independent.origin() == materializer.origin()
    {
        return Err(MaturityConfidentialRefusal::IndependentOriginNotAdmitted);
    }
    let solved = independent
        .solve_balancing_blinder(&material.input_blinder_sum, &[])
        .ok_or(MaturityConfidentialRefusal::InvalidBalancingBlinder)?;
    if solved != material.input_blinder_sum {
        return Err(MaturityConfidentialRefusal::BalancingBlinderMismatch);
    }
    let asset = inputs.sponsor.reserve_asset;
    let input_commitment = checked_commitment(
        MaturityConfidentialCommitment::SponsorInput,
        asset,
        material.semantic_input_amount,
        &material.input_blinder_sum,
        materializer,
        independent,
    )?;
    if spent.value() != ValueField::Commitment(*input_commitment.bytes()) {
        return Err(MaturityConfidentialRefusal::SponsorOpeningMismatch);
    }
    let commitment = checked_commitment(
        MaturityConfidentialCommitment::SponsorChange,
        asset,
        amount,
        &solved,
        materializer,
        independent,
    )?;
    if let ValueField::Commitment(offered) = change.value()
        && offered != *commitment.bytes()
    {
        return Err(MaturityConfidentialRefusal::OfferedChangeMismatch);
    }
    let (output, witness) =
        prove_change(change, asset, amount, &commitment, material, materializer)?;
    let mut witnesses = vec![OutputWitness::empty(); candidate.outputs().len()];
    witnesses[1] = witness;
    let construction = construction.with_confidential_change(output, witnesses)?;
    let transaction = construction.transaction();
    let bytes = transaction.encode();
    let decoded = TargetTransaction::decode(&bytes)
        .map_err(|_| MaturityConfidentialRefusal::SerializationRoundTripMismatch)?;
    if &decoded != transaction || decoded.encode() != bytes {
        return Err(MaturityConfidentialRefusal::SerializationRoundTripMismatch);
    }
    Ok(finalize_maturity_announcement(construction))
}

fn checked_commitment(
    coordinate: MaturityConfidentialCommitment,
    asset: AssetId,
    amount: u64,
    scalar: &[u8; SCALAR_BYTES],
    materializer: &dyn ConfidentialProofMaterializer,
    independent: &dyn IndependentCommitmentCheck,
) -> Result<MaterializerCommitment, MaturityConfidentialRefusal> {
    let produced = materializer
        .value_commitment(asset, amount, scalar)
        .ok_or_else(|| MaturityConfidentialRefusal::CommitmentUnavailable {
            coordinate,
            origin: materializer.origin(),
        })?;
    let checked = independent
        .recompute(asset, amount, scalar)
        .ok_or_else(|| MaturityConfidentialRefusal::CommitmentUnavailable {
            coordinate,
            origin: independent.origin(),
        })?;
    if !commitments_agree(&produced, &checked) {
        return Err(MaturityConfidentialRefusal::CommitmentDisagreement { coordinate });
    }
    Ok(produced)
}

fn prove_change(
    change: &TargetOutput,
    asset: AssetId,
    amount: u64,
    commitment: &MaterializerCommitment,
    material: &MaturityConfidentialSponsorMaterial,
    materializer: &dyn ConfidentialProofMaterializer,
) -> Result<(TargetOutput, OutputWitness), MaturityConfidentialRefusal> {
    let nonce = materializer
        .nonce_commitment(&material.nonce_input)
        .ok_or(MaturityConfidentialRefusal::NonceUnavailable)?;
    let request = RangeproofRequest::new(
        1,
        commitment,
        asset,
        amount,
        &material.input_blinder_sum,
        &material.rangeproof_seed,
        change.program(),
    );
    let proof = materializer
        .range_proof(&request)
        .ok_or(MaturityConfidentialRefusal::RangeproofUnavailable)?;
    if !proof.binds(&request) {
        return Err(MaturityConfidentialRefusal::ProofBindingMismatch);
    }
    if proof.proof().is_empty() {
        return Err(MaturityConfidentialRefusal::RangeproofEmpty);
    }
    if !proof.surjection_proof().is_empty() {
        return Err(MaturityConfidentialRefusal::UnexpectedSurjectionProof);
    }
    Ok((
        TargetOutput::new(
            change.asset(),
            ValueField::Commitment(*commitment.bytes()),
            NonceField::Commitment(nonce),
            change.program().to_vec(),
        ),
        OutputWitness::range_proof_only(proof.proof().to_vec()),
    ))
}
