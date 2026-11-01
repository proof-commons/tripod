//! STATE proof construction controls and one-coordinate refusals.
//! The collaborators are byte-producing doubles, not cryptographic evidence.

use super::state_support::{FixtureStateCurve, MaturitySponsor, maturity_sponsor, validated_view};
use super::{outpoint, reviewed_target};
use crate::bytes::{AssetId, NonceField, OutputWitness, TargetTransaction, ValueField};
use crate::live_materialize::{
    CommitmentOrigin, ConfidentialProofMaterializer, IndependentCommitment,
    IndependentCommitmentCheck, MaterializedRangeproof, MaterializerCommitment, RangeproofRequest,
    SCALAR_BYTES,
};
use crate::state_confidential::{
    MaturityConfidentialCommitment as Coordinate, MaturityConfidentialConstructionInputs,
    MaturityConfidentialRefusal as Refusal, MaturityConfidentialSponsorMaterial,
    construct_confidential_maturity_announcement,
};
use crate::state_construct::MaturitySponsorContext;
use crate::state_finalize::FinalizedMaturityAnnouncement;
use crate::view::{PublicConstructionView, PublicOutputView};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Fault {
    #[default]
    None,
    Commitment,
    ChangeUnavailable,
    Nonce,
    InvalidNonce,
    Proof,
    Binding,
    CommitmentBinding,
    AssetBinding,
    Empty,
    Surjection,
    Solve,
    WrongSolve,
    InputDisagreement,
    ChangeDisagreement,
    Origin,
    SameOrigin,
}

/// The deterministic bytes distinguish amount, asset and scalar changes.
fn commitment(asset: AssetId, amount: u64, scalar: &[u8; SCALAR_BYTES]) -> [u8; 33] {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(asset.internal());
    digest.update(amount.to_be_bytes());
    digest.update(scalar);
    let mut bytes = [8; 33];
    bytes[1..].copy_from_slice(&digest.finalize());
    bytes
}

pub(super) struct Materializer(pub Fault);

impl ConfidentialProofMaterializer for Materializer {
    fn origin(&self) -> CommitmentOrigin {
        if self.0 == Fault::SameOrigin {
            CommitmentOrigin::FirstPartyBignumOracle
        } else {
            CommitmentOrigin::ConstructionMaterializer
        }
    }

    fn value_commitment(
        &self,
        asset: AssetId,
        amount: u64,
        scalar: &[u8; SCALAR_BYTES],
    ) -> Option<MaterializerCommitment> {
        (self.0 != Fault::Commitment && !(amount == 40 && self.0 == Fault::ChangeUnavailable))
            .then(|| MaterializerCommitment::from_materializer(commitment(asset, amount, scalar)))
    }

    fn nonce_commitment(&self, input: &[u8; SCALAR_BYTES]) -> Option<[u8; 33]> {
        let mut bytes = [2; 33];
        bytes[1..].copy_from_slice(&crate::taproot::tagged_hash("STATE-test/nonce", input));
        if self.0 == Fault::InvalidNonce {
            bytes[0] = 0xff;
        }
        (self.0 != Fault::Nonce).then_some(bytes)
    }

    fn range_proof(&self, request: &RangeproofRequest<'_>) -> Option<MaterializedRangeproof> {
        assert_eq!(request.output(), 1);
        assert_eq!(request.semantic_amount(), 40);
        assert_eq!(request.value_blinder(), &[3; 32]);
        if self.0 == Fault::Proof {
            return None;
        }
        let mut program = request.output_program().to_vec();
        if self.0 == Fault::Binding {
            program.push(0);
        }
        let mut bound_commitment = *request.value_commitment().bytes();
        if self.0 == Fault::CommitmentBinding {
            bound_commitment[1] ^= 1;
        }
        let mut bound_asset = *request.explicit_asset().internal();
        if self.0 == Fault::AssetBinding {
            bound_asset[0] ^= 1;
        }
        Some(MaterializedRangeproof::new(
            if self.0 == Fault::Empty {
                Vec::new()
            } else {
                crate::taproot::tagged_hash("STATE-test/proof", request.seed()).to_vec()
            },
            if self.0 == Fault::Surjection {
                vec![1]
            } else {
                Vec::new()
            },
            bound_commitment,
            AssetId::from_internal(bound_asset),
            program,
        ))
    }
}

pub(super) struct Checker(pub Fault);

impl IndependentCommitmentCheck for Checker {
    fn origin(&self) -> CommitmentOrigin {
        if self.0 == Fault::Origin {
            CommitmentOrigin::ReferenceImplementation
        } else {
            CommitmentOrigin::FirstPartyBignumOracle
        }
    }

    fn recompute(
        &self,
        asset: AssetId,
        amount: u64,
        scalar: &[u8; SCALAR_BYTES],
    ) -> Option<IndependentCommitment> {
        if self.0 == Fault::Commitment || (amount == 40 && self.0 == Fault::ChangeUnavailable) {
            return None;
        }
        let mut bytes = commitment(asset, amount, scalar);
        if (amount == 130 && self.0 == Fault::InputDisagreement)
            || (amount == 40 && self.0 == Fault::ChangeDisagreement)
        {
            bytes[1] ^= 1;
        }
        Some(IndependentCommitment::from_independent_recomputation(bytes))
    }

    fn solve_balancing_blinder(
        &self,
        sum: &[u8; SCALAR_BYTES],
        others: &[[u8; SCALAR_BYTES]],
    ) -> Option<[u8; SCALAR_BYTES]> {
        assert!(
            others.is_empty(),
            "only sponsor change absorbs the input sum"
        );
        match self.0 {
            Fault::Solve => None,
            Fault::WrongSolve => Some([9; 32]),
            _ => Some(*sum),
        }
    }
}

pub(super) fn fixture() -> (
    MaturitySponsor,
    PublicConstructionView,
    MaturityConfidentialSponsorMaterial,
) {
    let (mut sponsor, public) = maturity_sponsor(true);
    sponsor.inputs.truncate(1);
    let point = sponsor.inputs[0];
    let spent = public.get(point).unwrap();
    let crate::bytes::AssetField::Explicit(asset) = spent.asset() else {
        panic!("explicit reserve");
    };
    let material = MaturityConfidentialSponsorMaterial {
        outpoint: point,
        semantic_input_amount: 130,
        input_blinder_sum: [3; 32],
        nonce_input: [4; 32],
        rangeproof_seed: [5; 32],
    };
    let public = PublicConstructionView::new([PublicOutputView::new(
        point,
        spent.asset(),
        ValueField::Commitment(commitment(asset, 130, &[3; 32])),
        spent.program().to_vec(),
    )])
    .unwrap();
    (sponsor, public, material)
}

pub(super) fn construct(
    sponsor: &MaturitySponsor,
    public: &PublicConstructionView,
    material: &MaturityConfidentialSponsorMaterial,
    materializer: Fault,
    checker: Fault,
) -> Result<FinalizedMaturityAnnouncement, Refusal> {
    let target = reviewed_target();
    let validated = validated_view();
    let abi = crate::derive_maturity_announcement_abi(&target, &validated).unwrap();
    let request = crate::MaturityAnnouncementRequest::new(
        realization::Cycle::new(8),
        crate::RequestedForm::Sponsored,
        crate::SponsorChangeRequest::Requested,
    )
    .unwrap();
    let context = MaturitySponsorContext {
        reserve_asset: AssetId::from_internal(super::live_support::LIVE_RESERVE_ASSET),
        view: public,
        capability: sponsor,
    };
    let result = construct_confidential_maturity_announcement(
        MaturityConfidentialConstructionInputs {
            target: &target,
            abi: &abi,
            validated: &validated,
            request: &request,
            curve: &FixtureStateCurve,
            sponsor: &context,
            material,
        },
        &Materializer(materializer),
        &Checker(checker),
    );
    let bytes = result
        .as_ref()
        .map_or(&[][..], |value| value.protected_bytes());
    assert_material_absent(material, bytes, &result);
    result
}

pub(super) fn assert_material_absent(
    material: &MaturityConfidentialSponsorMaterial,
    bytes: &[u8],
    value: &impl std::fmt::Debug,
) {
    let rendered = format!("{value:?}");
    for private in [
        material.input_blinder_sum,
        material.nonce_input,
        material.rangeproof_seed,
    ] {
        assert!(!bytes.windows(private.len()).any(|window| window == private));
        assert!(!rendered.contains(&format!("{private:?}")));
    }
}

pub(super) fn finalized() -> FinalizedMaturityAnnouncement {
    let (sponsor, public, material) = fixture();
    construct(&sponsor, &public, &material, Fault::None, Fault::None).unwrap()
}

pub(super) fn changed_proof(candidate: &TargetTransaction) -> TargetTransaction {
    let mut witnesses = candidate.output_witnesses().to_vec();
    let mut proof = witnesses[1].range_proof().to_vec();
    proof[0] ^= 1;
    witnesses[1] = OutputWitness::new(Vec::new(), proof);
    TargetTransaction::with_output_witnesses(
        candidate.version(),
        candidate.inputs().to_vec(),
        candidate.outputs().to_vec(),
        candidate.lock_time(),
        candidate.witnesses().to_vec(),
        witnesses,
    )
    .unwrap()
}

#[test]
fn commitments_agree_across_origins_for_input_and_change() {
    let (sponsor, public, material) = fixture();
    let control = construct(&sponsor, &public, &material, Fault::None, Fault::None).unwrap();
    assert_eq!(control.outputs().sponsor_change_position(), Some(1));
    assert_eq!(control.outputs().fee_position(), Some(2));
    assert_eq!(
        control.protected().outputs()[1].nonce(),
        NonceField::Commitment(
            Materializer(Fault::None)
                .nonce_commitment(&material.nonce_input)
                .unwrap()
        )
    );
    assert!(matches!(
        control.protected().outputs()[0].value(),
        ValueField::Explicit(1)
    ));
    assert_eq!(
        control.protected().outputs()[2].value(),
        ValueField::Explicit(90)
    );
    for (fault, coordinate) in [
        (Fault::InputDisagreement, Coordinate::SponsorInput),
        (Fault::ChangeDisagreement, Coordinate::SponsorChange),
    ] {
        assert_eq!(
            construct(&sponsor, &public, &material, Fault::None, fault),
            Err(Refusal::CommitmentDisagreement { coordinate })
        );
    }
}

#[test]
fn the_change_absorbs_the_input_blinder_and_refuses_a_different_solve() {
    let (sponsor, public, material) = fixture();
    let control = construct(&sponsor, &public, &material, Fault::None, Fault::None).unwrap();
    let asset = AssetId::from_internal(super::live_support::LIVE_RESERVE_ASSET);
    assert_eq!(
        control.protected().outputs()[1].value(),
        ValueField::Commitment(commitment(asset, 40, &material.input_blinder_sum))
    );
    for (fault, refusal) in [
        (Fault::Solve, Refusal::InvalidBalancingBlinder),
        (Fault::WrongSolve, Refusal::BalancingBlinderMismatch),
    ] {
        assert_eq!(
            construct(&sponsor, &public, &material, Fault::None, fault),
            Err(refusal)
        );
    }
}

#[test]
fn unavailable_material_is_refused_without_retry() {
    let (sponsor, public, material) = fixture();
    assert!(construct(&sponsor, &public, &material, Fault::None, Fault::None).is_ok());
    for (fault, refusal) in [
        (
            Fault::Commitment,
            Refusal::CommitmentUnavailable {
                coordinate: Coordinate::SponsorInput,
                origin: CommitmentOrigin::ConstructionMaterializer,
            },
        ),
        (Fault::Nonce, Refusal::NonceUnavailable),
        (Fault::Proof, Refusal::RangeproofUnavailable),
    ] {
        assert_eq!(
            construct(&sponsor, &public, &material, fault, Fault::None),
            Err(refusal)
        );
    }
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::Commitment),
        Err(Refusal::CommitmentUnavailable {
            coordinate: Coordinate::SponsorInput,
            origin: CommitmentOrigin::FirstPartyBignumOracle
        })
    );
}

#[test]
fn proof_binding_nonempty_range_and_empty_surjection_are_required() {
    let (sponsor, public, material) = fixture();
    assert!(construct(&sponsor, &public, &material, Fault::None, Fault::None).is_ok());
    for (fault, refusal) in [
        (Fault::Binding, Refusal::ProofBindingMismatch),
        (Fault::CommitmentBinding, Refusal::ProofBindingMismatch),
        (Fault::AssetBinding, Refusal::ProofBindingMismatch),
        (Fault::Empty, Refusal::RangeproofEmpty),
        (Fault::Surjection, Refusal::UnexpectedSurjectionProof),
    ] {
        assert_eq!(
            construct(&sponsor, &public, &material, fault, Fault::None),
            Err(refusal)
        );
    }
}

#[test]
fn reference_origin_cannot_be_the_independent_check() {
    let (sponsor, public, material) = fixture();
    assert!(construct(&sponsor, &public, &material, Fault::None, Fault::None).is_ok());
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::Origin),
        Err(Refusal::IndependentOriginNotAdmitted)
    );
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::SameOrigin, Fault::None),
        Err(Refusal::IndependentOriginNotAdmitted)
    );
}

#[test]
fn the_material_outpoint_must_name_the_offered_coin() {
    let (sponsor, public, mut material) = fixture();
    assert!(construct(&sponsor, &public, &material, Fault::None, Fault::None).is_ok());
    material.outpoint = outpoint(0xef, 0);
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::None),
        Err(Refusal::UnsupportedSponsorShape)
    );
}

#[test]
fn either_origin_can_refuse_the_change_commitment() {
    let (sponsor, public, material) = fixture();
    assert!(construct(&sponsor, &public, &material, Fault::None, Fault::None).is_ok());
    for (materializer, checker, origin) in [
        (
            Fault::ChangeUnavailable,
            Fault::None,
            CommitmentOrigin::ConstructionMaterializer,
        ),
        (
            Fault::None,
            Fault::ChangeUnavailable,
            CommitmentOrigin::FirstPartyBignumOracle,
        ),
    ] {
        assert_eq!(
            construct(&sponsor, &public, &material, materializer, checker),
            Err(Refusal::CommitmentUnavailable {
                coordinate: Coordinate::SponsorChange,
                origin
            })
        );
    }
}

#[test]
fn invalid_materialized_nonce_refuses_at_the_strict_round_trip() {
    let (sponsor, public, material) = fixture();
    assert!(construct(&sponsor, &public, &material, Fault::None, Fault::None).is_ok());
    assert_eq!(
        construct(
            &sponsor,
            &public,
            &material,
            Fault::InvalidNonce,
            Fault::None
        ),
        Err(Refusal::SerializationRoundTripMismatch)
    );
}

#[test]
fn the_input_opening_must_match_the_retained_coin() {
    let (sponsor, public, mut material) = fixture();
    assert!(construct(&sponsor, &public, &material, Fault::None, Fault::None).is_ok());
    material.input_blinder_sum[0] ^= 1;
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::None),
        Err(Refusal::SponsorOpeningMismatch)
    );
}

#[test]
fn multiple_or_explicit_sponsor_coins_are_outside_the_confidential_shape() {
    let (mut sponsor, public, material) = fixture();
    assert!(construct(&sponsor, &public, &material, Fault::None, Fault::None).is_ok());
    let spent = public.get(material.outpoint).unwrap();
    let extra = outpoint(0xef, 0);
    sponsor.inputs.push(extra);
    let multiple = PublicConstructionView::new([
        spent.clone(),
        PublicOutputView::new(
            extra,
            spent.asset(),
            spent.value(),
            spent.program().to_vec(),
        ),
    ])
    .unwrap();
    assert_eq!(
        construct(&sponsor, &multiple, &material, Fault::None, Fault::None),
        Err(Refusal::UnsupportedSponsorShape)
    );
    sponsor.inputs.pop();
    let explicit = PublicConstructionView::new([PublicOutputView::new(
        spent.outpoint(),
        spent.asset(),
        ValueField::Explicit(130),
        spent.program().to_vec(),
    )])
    .unwrap();
    assert_eq!(
        construct(&sponsor, &explicit, &material, Fault::None, Fault::None),
        Err(Refusal::UnsupportedSponsorShape)
    );
}

#[test]
fn change_and_fee_must_exhaust_the_input_with_positive_change() {
    let (mut sponsor, public, material) = fixture();
    assert!(construct(&sponsor, &public, &material, Fault::None, Fault::None).is_ok());
    sponsor.fee = 131;
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::None),
        Err(Refusal::AmountImbalance)
    );
    sponsor.fee = 130;
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::None),
        Err(Refusal::UnsupportedSponsorShape)
    );
    sponsor.fee = 90;
    sponsor.change = Some(ValueField::Explicit(39));
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::None),
        Err(Refusal::AmountImbalance)
    );
    sponsor.change = Some(ValueField::Explicit(0));
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::None),
        Err(Refusal::UnsupportedSponsorShape)
    );
    sponsor.change = None;
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::None),
        Err(Refusal::Construction(Box::new(
            crate::TransactionRefusal::SponsorChangeRequestedWithoutDestination
        )))
    );
}

#[test]
fn an_offered_committed_change_must_equal_the_computed_one() {
    let (mut sponsor, public, material) = fixture();
    let control = construct(&sponsor, &public, &material, Fault::None, Fault::None).unwrap();
    sponsor.change = Some(control.protected().outputs()[1].value());
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::None),
        Ok(control)
    );
    sponsor.change = Some(ValueField::Commitment([8; 33]));
    assert_eq!(
        construct(&sponsor, &public, &material, Fault::None, Fault::None),
        Err(Refusal::OfferedChangeMismatch)
    );
}

#[test]
fn a_range_proof_on_an_explicit_output_refuses_at_decoding() {
    let control = finalized();
    let candidate = control.protected();
    assert_eq!(
        TargetTransaction::decode(&candidate.encode()).unwrap(),
        *candidate
    );
    for position in [0, 2] {
        let mut witnesses = candidate.output_witnesses().to_vec();
        witnesses[position] = OutputWitness::new(Vec::new(), vec![1]);
        let subject = TargetTransaction::with_output_witnesses(
            candidate.version(),
            candidate.inputs().to_vec(),
            candidate.outputs().to_vec(),
            candidate.lock_time(),
            candidate.witnesses().to_vec(),
            witnesses,
        )
        .unwrap();
        assert_eq!(
            TargetTransaction::decode(&subject.encode()),
            Err(crate::TransactionRefusal::RangeProofRefused)
        );
    }
}
