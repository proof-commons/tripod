//! Classification and role offers are rebuilt from constructed transactions, never
//! from hand-assembled bundles or ABIs. The S1 control uses encoded witness
//! programs here: the shared sponsor fixture carries raw payload bytes,
//! which are not programs of the admitted witness class.

use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::time::Instant;

use architecture::{
    AssetId as SemanticAsset, DeltaKind, ObjectId, OperationId, ProjectionId, TagId,
};
use realization::{
    Count, Cycle, ObservedCanonicalFlow, ObservedCanonicalPartition, ObservedObjectRef,
    ObservedSide, RealizationScope, RelationFailure, RelationId, RelationKind, RelationStatus,
    RelationSubject, ScopedRealizationSpec, TransactionSide,
};

use super::live_support::LIVE_RESERVE_ASSET;
use super::state_support::{
    FixtureStateCurve, construct_with_sponsor, maturity_sponsor, sponsored_construction,
    validated_view,
};
use crate::bytes::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, OutputWitness, TargetInput,
    TargetOutput, TargetTransaction, Txid, ValueField,
};
use crate::state_abi::{
    CandidateMaturityAnnouncementAbi, MaturityAbiClassificationRefusal as Refusal,
    MaturityAbiOffer, MaturityPositionClaim as Claim, ValidatedMaturityAbiClassification,
    derive_maturity_announcement_abi, validate_maturity_abi_classification,
};
use crate::{
    MaturityAnnouncementRequest, RequestedForm, SpentOutputCensusEntry, SponsorChangeRequest,
    TransactionRefusal, ValidatedMaturityStateView,
};

#[derive(Clone)]
struct Offer {
    bytes: Vec<u8>,
    spent: Vec<SpentOutputCensusEntry>,
    inputs: Vec<Claim>,
    outputs: Vec<Claim>,
    canonical: ObservedCanonicalPartition,
}

impl Offer {
    fn borrowed(&self) -> MaturityAbiOffer<'_> {
        MaturityAbiOffer {
            bytes: &self.bytes,
            spent: &self.spent,
            reserve_asset: AssetId::from_internal(LIVE_RESERVE_ASSET),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            canonical: self.canonical.clone(),
        }
    }

    fn validate(&self) -> Result<ValidatedMaturityAbiClassification, Refusal> {
        let fixture = fixture();
        validate_maturity_abi_classification(
            &fixture.abi,
            &fixture.view,
            &fixture.spec,
            &self.borrowed(),
        )
    }

    fn transaction(&self) -> TargetTransaction {
        TargetTransaction::decode(&self.bytes).unwrap()
    }

    fn append_output(&mut self, output: TargetOutput, claim: Claim) {
        let transaction = self.transaction();
        let mut outputs = transaction.outputs().to_vec();
        outputs.push(output);
        let mut witnesses = transaction.output_witnesses().to_vec();
        witnesses.push(OutputWitness::empty());
        self.bytes = TargetTransaction::with_output_witnesses(
            transaction.version(),
            transaction.inputs().to_vec(),
            outputs,
            transaction.lock_time(),
            transaction.witnesses().to_vec(),
            witnesses,
        )
        .unwrap()
        .encode();
        self.outputs.push(claim);
    }

    fn append_input(&mut self, spent: SpentOutputCensusEntry, claim: Claim) {
        self.append_input_at(
            Outpoint::new(Txid::from_internal([0xe9; 32]), 1).unwrap(),
            spent,
            claim,
        );
    }

    fn append_input_at(&mut self, outpoint: Outpoint, spent: SpentOutputCensusEntry, claim: Claim) {
        let transaction = self.transaction();
        let mut inputs = transaction.inputs().to_vec();
        inputs.push(TargetInput::new(outpoint, u32::MAX));
        let mut witnesses = transaction.witnesses().to_vec();
        witnesses.push(InputWitness::default());
        self.bytes = TargetTransaction::with_output_witnesses(
            transaction.version(),
            inputs,
            transaction.outputs().to_vec(),
            transaction.lock_time(),
            witnesses,
            transaction.output_witnesses().to_vec(),
        )
        .unwrap()
        .encode();
        self.spent.push(spent);
        self.inputs.push(claim);
    }

    fn replace_input(
        &mut self,
        position: usize,
        input: TargetInput,
        witness: InputWitness,
        spent: SpentOutputCensusEntry,
        claim: Claim,
    ) {
        let transaction = self.transaction();
        let mut inputs = transaction.inputs().to_vec();
        let mut witnesses = transaction.witnesses().to_vec();
        inputs[position] = input;
        witnesses[position] = witness;
        self.bytes = TargetTransaction::with_output_witnesses(
            transaction.version(),
            inputs,
            transaction.outputs().to_vec(),
            transaction.lock_time(),
            witnesses,
            transaction.output_witnesses().to_vec(),
        )
        .unwrap()
        .encode();
        self.spent[position] = spent;
        self.inputs[position] = claim;
    }

    fn swap_inputs(&mut self, first: usize, second: usize) {
        let transaction = self.transaction();
        let mut inputs = transaction.inputs().to_vec();
        let mut witnesses = transaction.witnesses().to_vec();
        inputs.swap(first, second);
        witnesses.swap(first, second);
        self.bytes = TargetTransaction::with_output_witnesses(
            transaction.version(),
            inputs,
            transaction.outputs().to_vec(),
            transaction.lock_time(),
            witnesses,
            transaction.output_witnesses().to_vec(),
        )
        .unwrap()
        .encode();
        self.spent.swap(first, second);
        self.inputs.swap(first, second);
    }

    fn swap_outputs(&mut self, first: usize, second: usize) {
        let transaction = self.transaction();
        let mut outputs = transaction.outputs().to_vec();
        let mut witnesses = transaction.output_witnesses().to_vec();
        outputs.swap(first, second);
        witnesses.swap(first, second);
        self.bytes = TargetTransaction::with_output_witnesses(
            transaction.version(),
            transaction.inputs().to_vec(),
            outputs,
            transaction.lock_time(),
            transaction.witnesses().to_vec(),
            witnesses,
        )
        .unwrap()
        .encode();
        self.outputs.swap(first, second);
    }

    fn replace_output(&mut self, position: usize, output: TargetOutput) {
        let transaction = self.transaction();
        let mut outputs = transaction.outputs().to_vec();
        outputs[position] = output;
        self.bytes = TargetTransaction::with_output_witnesses(
            transaction.version(),
            transaction.inputs().to_vec(),
            outputs,
            transaction.lock_time(),
            transaction.witnesses().to_vec(),
            transaction.output_witnesses().to_vec(),
        )
        .unwrap()
        .encode();
    }
}

struct Fixture {
    abi: CandidateMaturityAnnouncementAbi,
    view: ValidatedMaturityStateView,
    spec: ScopedRealizationSpec,
    sponsorless: Offer,
    sponsored: Offer,
}

fn fixture() -> &'static Fixture {
    static FIXTURE: LazyLock<Fixture> = LazyLock::new(|| {
        let target = super::reviewed_target();
        let view = validated_view();
        let abi = derive_maturity_announcement_abi(&target, &view).unwrap();
        let spec = realization::derive(
            &architecture::ARCHITECTURE,
            RealizationScope::from_operations([OperationId::AnnounceMaturity]).unwrap(),
        )
        .unwrap();
        let request = MaturityAnnouncementRequest::new(
            Cycle::new(8),
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
        )
        .unwrap();
        let control = crate::construct_maturity_announcement(
            &target,
            &abi,
            &view,
            &request,
            &FixtureStateCurve,
        )
        .unwrap();
        let predecessor = SpentOutputCensusEntry::new(
            view.view().asset(),
            view.view().value(),
            view.view().predecessor_program().to_vec(),
        );
        let sponsorless = Offer {
            bytes: control.transaction().encode(),
            spent: vec![predecessor.clone()],
            inputs: vec![Claim::State],
            outputs: vec![Claim::State],
            canonical: ObservedCanonicalPartition::default(),
        };
        let (sponsor, raw) = maturity_sponsor(false);
        let public = crate::PublicConstructionView::new(raw.outputs().values().map(|spent| {
            crate::PublicOutputView::new(
                spent.outpoint(),
                spent.asset(),
                spent.value(),
                crate::taproot::witness_program_script(&target, 0, &[0xb4; 20]).unwrap(),
            )
        }))
        .unwrap();
        let construction = construct_with_sponsor(false, &sponsor, &public).unwrap();
        assert_eq!(
            construction.transaction(),
            sponsored_construction(false).transaction()
        );
        let mut spent = vec![predecessor];
        for (input, output) in construction
            .transaction()
            .inputs()
            .iter()
            .skip(1)
            .zip(construction.sponsor_spent())
        {
            assert_eq!(input.outpoint(), output.outpoint());
            spent.push(SpentOutputCensusEntry::new(
                output.asset(),
                output.value(),
                output.program().to_vec(),
            ));
        }
        let sponsored = Offer {
            bytes: construction.transaction().encode(),
            spent,
            inputs: vec![Claim::State, Claim::SponsorMember, Claim::SponsorMember],
            outputs: vec![Claim::State, Claim::TargetFee],
            canonical: ObservedCanonicalPartition::default(),
        };
        Fixture {
            abi,
            view,
            spec,
            sponsorless,
            sponsored,
        }
    });
    &FIXTURE
}

fn relation(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::AnnounceMaturity, kind, subject)
}

fn closure(side: ObservedSide) -> RelationId {
    relation(
        RelationKind::AllowedObjectFamilies,
        RelationSubject::TransactionSide {
            side: match side {
                ObservedSide::Input => TransactionSide::Input,
                ObservedSide::Output => TransactionSide::Output,
            },
        },
    )
}

fn family_failure(side: ObservedSide, canonical_delta_collateral: bool) -> Refusal {
    let mut failed = BTreeMap::from([(closure(side), RelationFailure::UndeclaredObjectFamily)]);
    if canonical_delta_collateral {
        failed.insert(
            relation(
                RelationKind::CanonicalDeltaPolicy,
                RelationSubject::Operation,
            ),
            RelationFailure::CanonicalDeltaPolicy,
        );
    }
    Refusal::RelationVerdicts { failed }
}

fn field_asset(family: ObjectId) -> AssetField {
    let asset = architecture::ARCHITECTURE.object(family).unwrap().asset;
    let bytes = if asset == SemanticAsset::Lbtc {
        LIVE_RESERVE_ASSET
    } else {
        [u8::try_from(asset.code()).unwrap() + 0x80; 32]
    };
    AssetField::Explicit(AssetId::from_internal(bytes))
}

fn output(asset: AssetField, value: u64, program: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        asset,
        ValueField::Explicit(value),
        NonceField::Null,
        program,
    )
}

struct Subject {
    name: &'static str,
    offer: Offer,
    positions: Vec<(ObservedSide, usize)>,
    expected: Refusal,
}

fn family_subject(
    name: &'static str,
    family: ObjectId,
    side: ObservedSide,
    canonical_delta_collateral: bool,
) -> Subject {
    let mut offer = fixture().sponsorless.clone();
    let asset = field_asset(family);
    match side {
        ObservedSide::Input => offer.append_input(
            SpentOutputCensusEntry::new(asset, ValueField::Explicit(1), vec![0x51]),
            Claim::Family(family),
        ),
        ObservedSide::Output => {
            offer.append_output(output(asset, 1, vec![0x51]), Claim::Family(family));
        }
    }
    Subject {
        name,
        offer,
        positions: vec![(side, 1)],
        expected: family_failure(side, canonical_delta_collateral),
    }
}

fn data_subject(name: &'static str, claim: Claim, program: Vec<u8>, destruction: bool) -> Subject {
    let mut offer = fixture().sponsorless.clone();
    let (asset, value) = if destruction {
        (fixture().view.view().asset(), 1)
    } else {
        (
            AssetField::Explicit(AssetId::from_internal(LIVE_RESERVE_ASSET)),
            0,
        )
    };
    offer.append_output(output(asset, value, program), claim);
    Subject {
        name,
        offer,
        positions: vec![(ObservedSide::Output, 1)],
        expected: Refusal::UnadmittedOutputRole { position: 1, claim },
    }
}

fn tagged_program(tag: TagId) -> Vec<u8> {
    let payload = tag.as_str().as_bytes();
    let mut program = vec![0x6a, u8::try_from(payload.len()).unwrap()];
    program.extend_from_slice(payload);
    program
}

fn canonical_subject(name: &'static str, family: ObjectId) -> Subject {
    let mut subject = family_subject(name, family, ObservedSide::Input, true);
    subject.offer.append_output(
        output(field_asset(family), 1, vec![0x51]),
        Claim::Family(family),
    );
    subject.positions.push((ObservedSide::Output, 1));
    subject.offer.canonical.flows.push(ObservedCanonicalFlow {
        asset: architecture::ARCHITECTURE.object(family).unwrap().asset,
        sources: vec![ObservedObjectRef {
            side: ObservedSide::Input,
            ordinal: 1,
        }],
        destinations: vec![ObservedObjectRef {
            side: ObservedSide::Output,
            ordinal: 1,
        }],
        movement_kind: Some(DeltaKind::Lateral),
        destructions: Vec::new(),
    });
    subject.expected = Refusal::RelationVerdicts {
        failed: BTreeMap::from([
            (
                closure(ObservedSide::Input),
                RelationFailure::UndeclaredObjectFamily,
            ),
            (
                closure(ObservedSide::Output),
                RelationFailure::UndeclaredObjectFamily,
            ),
            (
                relation(
                    RelationKind::CanonicalDeltaPolicy,
                    RelationSubject::Operation,
                ),
                RelationFailure::CanonicalDeltaPolicy,
            ),
        ]),
    };
    subject
}

fn issuance_subject() -> Subject {
    let mut offer = fixture().sponsorless.clone();
    let control = offer.transaction().encode_without_witness();
    assert_eq!(control, fixture().sponsorless.bytes);
    // One empty-script input: version, witness flag, count, txid, index,
    // empty script prefix and sequence. Assert the boundary before surgery.
    assert_eq!(control[4], 0);
    assert_eq!(control[5], 1);
    assert_eq!(control[42], 0);
    assert_eq!(&control[43..47], &u32::MAX.to_le_bytes());
    assert_eq!(control[47], 1);
    let mut body = vec![0; 32];
    body.extend_from_slice(&[0x77; 32]);
    body.push(1);
    body.extend_from_slice(&1_u64.to_be_bytes());
    body.push(0);
    assert_eq!(body.len(), 74);
    offer.bytes = control[..47].to_vec();
    offer.bytes[41] |= 0x80;
    offer.bytes.extend_from_slice(&body);
    offer.bytes.extend_from_slice(&control[47..]);
    assert_eq!(&offer.bytes[47..121], body.as_slice());
    assert_eq!(&offer.bytes[121..], &control[47..]);
    assert_eq!(offer.bytes.len(), control.len() + 74);
    Subject {
        name: "add-issuance",
        offer,
        positions: vec![(ObservedSide::Input, 0)],
        expected: Refusal::IssuanceOffered,
    }
}

fn subjects() -> Vec<Subject> {
    use ObservedSide::{Input, Output};
    // Each row names its side's family closure; the last column declares
    // CanonicalDeltaPolicy collateral. Exact partition membership tracks U,
    // ENT and DIST_CTL (architecture::canonical_value_asset), so an unpaired
    // position of those assets also fails that policy. Other assets do not.
    let families = [
        ("live-receipt-input", ObjectId::ReceiptLive, Input, true),
        ("live-receipt-output", ObjectId::ReceiptLive, Output, true),
        (
            "time-locked-receipt",
            ObjectId::ReceiptTimeLocked,
            Output,
            true,
        ),
        ("ash", ObjectId::Ash, Output, true),
        ("entitlement", ObjectId::DepositEntitlement, Output, true),
        ("request", ObjectId::DepositRequest, Output, false),
        (
            "distribution-control",
            ObjectId::DistributionControl,
            Output,
            true,
        ),
        (
            "distribution-vault",
            ObjectId::DistributionVault,
            Output,
            true,
        ),
        ("resv-input", ObjectId::Resv, Input, false),
        ("resv-output", ObjectId::Resv, Output, false),
        ("pace", ObjectId::Pace, Output, false),
        (
            "entitlement-authority",
            ObjectId::EntitlementAuthority,
            Output,
            false,
        ),
        (
            "distribution-authority",
            ObjectId::DistributionAuthority,
            Output,
            false,
        ),
    ];
    let mut subjects = families
        .into_iter()
        .map(|(name, family, side, collateral)| family_subject(name, family, side, collateral))
        .collect::<Vec<_>>();
    subjects.push(issuance_subject());
    subjects.push(data_subject(
        "destruction",
        Claim::Destruction,
        vec![0x6a],
        true,
    ));
    subjects.push(data_subject(
        "burn-record",
        Claim::DataOutput(Some(TagId::Burn)),
        tagged_program(TagId::Burn),
        false,
    ));
    subjects.push(data_subject(
        "clear-event",
        Claim::DataOutput(Some(TagId::Recon)),
        tagged_program(TagId::Recon),
        false,
    ));
    let mut residue = data_subject(
        "residue-projection",
        Claim::Projection(ProjectionId::DistributionResidue),
        tagged_program(TagId::DistributionResidue),
        false,
    );
    residue.expected = Refusal::RelationVerdicts {
        failed: BTreeMap::from([(
            relation(RelationKind::ProjectionPolicy, RelationSubject::Operation),
            RelationFailure::ProjectionPolicy,
        )]),
    };
    subjects.push(residue);
    subjects.extend([
        canonical_subject("canonical-u", ObjectId::ReceiptLive),
        canonical_subject("canonical-ent", ObjectId::DepositEntitlement),
        canonical_subject("canonical-dist-ctl", ObjectId::DistributionControl),
    ]);
    subjects
}

fn claims() -> Vec<Option<Claim>> {
    let mut claims = vec![
        Claim::State,
        Claim::SponsorMember,
        Claim::SponsorChange,
        Claim::TargetFee,
        Claim::Destruction,
        Claim::DataOutput(None),
    ];
    claims.extend(ObjectId::ALL.iter().copied().map(Claim::Family));
    claims.extend(ProjectionId::ALL.iter().copied().map(Claim::Projection));
    claims.extend(
        TagId::ALL
            .iter()
            .copied()
            .map(|tag| Claim::DataOutput(Some(tag))),
    );
    assert_eq!(claims.len(), 30);
    let mut alternatives = claims.into_iter().map(Some).collect::<Vec<_>>();
    alternatives.push(None);
    alternatives
}

fn replace_claim(offer: &mut Offer, (side, position): (ObservedSide, usize), claim: Option<Claim>) {
    let claims = match side {
        ObservedSide::Input => &mut offer.inputs,
        ObservedSide::Output => &mut offer.outputs,
    };
    if let Some(claim) = claim {
        claims[position] = claim;
    } else {
        claims.truncate(position);
    }
}

#[test]
fn both_controls_accept_without_claiming_sponsor_calibration() {
    let fixture = fixture();
    for (offer, bound) in [
        (&fixture.sponsorless, None),
        (&fixture.sponsored, Some(Count::new(2))),
    ] {
        let validated = offer.validate().unwrap();
        assert_eq!(validated.transaction().encode(), offer.bytes);
        assert_eq!(validated.input_claims(), offer.inputs);
        assert_eq!(validated.output_claims(), offer.outputs);
        assert_eq!(validated.sponsor_input_bound_uncalibrated(), bound);
        assert_eq!(validated.report().operation, OperationId::AnnounceMaturity);
        for owned in [
            closure(ObservedSide::Input),
            closure(ObservedSide::Output),
            relation(RelationKind::OpenFlowPolicy, RelationSubject::Operation),
            relation(
                RelationKind::CanonicalDeltaPolicy,
                RelationSubject::Operation,
            ),
            relation(RelationKind::ProjectionPolicy, RelationSubject::Operation),
        ] {
            assert_eq!(
                validated.report().verdict(&owned).unwrap().status,
                RelationStatus::Passed
            );
        }
        assert!(validated.report().verdicts.len() > 5);
    }
    assert_eq!(fixture.sponsored.spent[1].value(), ValueField::Explicit(75));
    let sponsored = fixture.sponsored.validate().unwrap();
    let sponsor_input = RelationSubject::ObjectFamily {
        side: TransactionSide::Input,
        object: ObjectId::PlainLbtc,
    };
    let recognition = relation(RelationKind::Recognition, sponsor_input.clone());
    for (id, status) in [
        (
            recognition.clone(),
            RelationStatus::Failed {
                reason: RelationFailure::ObjectRecognition,
            },
        ),
        (
            relation(RelationKind::Cardinality, sponsor_input),
            RelationStatus::Blocked {
                prerequisites: vec![recognition.clone()],
            },
        ),
        (
            relation(RelationKind::SponsorIsolation, RelationSubject::Sponsor),
            RelationStatus::Blocked {
                prerequisites: vec![recognition],
            },
        ),
    ] {
        assert_eq!(sponsored.report().verdict(&id).unwrap().status, status);
    }
    assert_eq!(fixture.sponsored.spent[2].value(), ValueField::Explicit(55));
    assert_eq!(
        fixture.sponsored.transaction().outputs()[1].value(),
        ValueField::Explicit(130)
    );
}

#[test]
fn all_twenty_one_honest_subjects_have_the_exact_refusal() {
    let subjects = subjects();
    assert_eq!(subjects.len(), 21);
    assert_eq!(
        subjects
            .iter()
            .filter(|subject| subject.positions.len() == 1)
            .count(),
        18
    );
    assert_eq!(
        subjects
            .iter()
            .filter(|subject| subject.positions.len() == 2)
            .count(),
        3
    );
    for subject in subjects {
        assert_eq!(
            subject.offer.validate(),
            Err(subject.expected),
            "{}",
            subject.name
        );
    }
}

#[test]
fn classification_is_independent_of_all_changed_position_claims() {
    use std::io::Write as _;
    let subjects = subjects();
    let alternatives = claims();
    let started = Instant::now();
    let mut calls = 0;
    for subject in &subjects {
        for first in &alternatives {
            let second_choices = if subject.positions.len() == 2 {
                alternatives.as_slice()
            } else {
                &alternatives[..1]
            };
            for second in second_choices {
                let mut offer = subject.offer.clone();
                replace_claim(&mut offer, subject.positions[0], *first);
                if let Some(position) = subject.positions.get(1) {
                    replace_claim(&mut offer, *position, *second);
                }
                let refusal = offer.validate().expect_err(subject.name);
                assert!(
                    !matches!(refusal, Refusal::ClaimCensus { .. }),
                    "{}",
                    subject.name
                );
                // Issuance refuses at decoding, before the claim census.
                if subject.name != "add-issuance" {
                    let omitted = if first.is_none() {
                        Some(subject.positions[0])
                    } else if second.is_none() {
                        subject.positions.get(1).copied()
                    } else {
                        None
                    };
                    if let Some((side, position)) = omitted {
                        assert_eq!(
                            refusal,
                            Refusal::Unclassified { side, position },
                            "{}",
                            subject.name
                        );
                    }
                }
                calls += 1;
            }
        }
    }
    assert_eq!(calls, 3441);
    fixture().sponsorless.validate().unwrap();
    fixture().sponsored.validate().unwrap();
    calls += 2;
    assert_eq!(calls, 3443);
    writeln!(
        std::io::stdout().lock(),
        "census classification: {calls} calls; table wall {:?}",
        started.elapsed()
    )
    .unwrap();
}

#[test]
fn the_callers_scope_must_include_the_announcement() {
    let fixture = fixture();
    let other = realization::derive(
        &architecture::ARCHITECTURE,
        RealizationScope::phase1_pilots(),
    )
    .unwrap();
    assert_eq!(
        validate_maturity_abi_classification(
            &fixture.abi,
            &fixture.view,
            &other,
            &fixture.sponsorless.borrowed()
        ),
        Err(Refusal::Realization(Box::new(
            realization::RealizationError::OperationOutsideScope(OperationId::AnnounceMaturity)
        )))
    );
}

#[test]
fn policy_pair_guard_refuses_a_distinct_revision_without_forging_an_abi() {
    use crate::state_abi::check_maturity_policy_pair;
    use target_elements::TargetContractVersion;
    // End-to-end mismatch is unreachable: PublicMaturityStateView::validate
    // checks the reviewed target policy, derive_maturity_announcement_abi
    // repeats that check, and its result copies the bundle policy. Exercise
    // the future second-policy guard directly, without assembling an ABI.
    assert_eq!(
        TargetContractVersion::SUPPORTED,
        &[TargetContractVersion::V2]
    );
    assert_eq!(
        check_maturity_policy_pair(TargetContractVersion::V2, TargetContractVersion::V2),
        Ok(())
    );
    assert_eq!(
        check_maturity_policy_pair(TargetContractVersion::V2, TargetContractVersion::V1),
        Err(Refusal::ContractMismatch)
    );
}

#[test]
fn decoding_preserves_every_other_refusal_including_pegin() {
    let issuance = issuance_subject();
    assert_eq!(issuance.offer.validate(), Err(Refusal::IssuanceOffered));
    let mut pegin = fixture().sponsorless.clone();
    pegin.bytes = pegin.transaction().encode_without_witness();
    pegin.bytes[41] |= 0x40;
    assert_eq!(
        pegin.validate(),
        Err(Refusal::Decode(Box::new(
            TransactionRefusal::PeginInputRefused
        )))
    );
    let mut truncated = fixture().sponsorless.clone();
    truncated.bytes.pop();
    let error = TargetTransaction::decode(&truncated.bytes).unwrap_err();
    assert_eq!(truncated.validate(), Err(Refusal::Decode(Box::new(error))));
}

#[test]
fn the_spent_and_claim_censuses_are_exact() {
    let mut offer = fixture().sponsorless.clone();
    offer.spent.clear();
    assert_eq!(
        offer.validate(),
        Err(Refusal::SpentCensus {
            inputs: 1,
            entries: 0
        })
    );
    for side in [ObservedSide::Input, ObservedSide::Output] {
        let mut offer = fixture().sponsorless.clone();
        replace_claim(&mut offer, (side, 0), None);
        assert_eq!(
            offer.validate(),
            Err(Refusal::Unclassified { side, position: 0 })
        );
        let mut offer = fixture().sponsorless.clone();
        match side {
            ObservedSide::Input => offer.inputs.push(Claim::State),
            ObservedSide::Output => offer.outputs.push(Claim::State),
        }
        assert_eq!(
            offer.validate(),
            Err(Refusal::ClaimCensus {
                side,
                positions: 1,
                claims: 2
            })
        );
    }
}

#[test]
fn state_claims_authenticate_asset_singleton_and_program_on_both_sides() {
    for claim in [Claim::State, Claim::Family(ObjectId::State)] {
        for side in [ObservedSide::Input, ObservedSide::Output] {
            let control = fixture().sponsorless.clone();
            let spent = &control.spent[0];
            for (asset, value, program) in [
                (
                    AssetField::Explicit(AssetId::from_internal(LIVE_RESERVE_ASSET)),
                    spent.value(),
                    spent.program().to_vec(),
                ),
                (
                    spent.asset(),
                    ValueField::Explicit(2),
                    spent.program().to_vec(),
                ),
                (spent.asset(), spent.value(), vec![0x51]),
            ] {
                let mut offer = control.clone();
                replace_claim(&mut offer, (side, 0), Some(claim));
                match side {
                    ObservedSide::Input => {
                        offer.spent[0] = SpentOutputCensusEntry::new(asset, value, program);
                    }
                    ObservedSide::Output => offer.replace_output(
                        0,
                        TargetOutput::new(asset, value, NonceField::Null, program),
                    ),
                }
                assert_eq!(
                    offer.validate(),
                    Err(Refusal::ClaimContradictsBytes { side, position: 0 })
                );
            }
        }
    }
}

#[test]
fn admitted_family_aliases_do_not_bypass_authentication() {
    let mut state = fixture().sponsorless.clone();
    state.inputs[0] = Claim::Family(ObjectId::State);
    state.outputs[0] = Claim::Family(ObjectId::State);
    state.validate().unwrap();
    let mut sponsor = fixture().sponsored.clone();
    sponsor.inputs[1] = Claim::Family(ObjectId::PlainLbtc);
    sponsor.validate().unwrap();
    let spent = sponsor.spent[1].clone();
    sponsor.spent[1] = SpentOutputCensusEntry::new(spent.asset(), spent.value(), vec![0xb4; 32]);
    assert_eq!(
        sponsor.validate(),
        Err(Refusal::ClaimContradictsBytes {
            side: ObservedSide::Input,
            position: 1
        })
    );
    let mut projection = fixture().sponsorless.clone();
    projection.outputs[0] = Claim::Projection(ProjectionId::TransitionCertificate);
    assert_eq!(
        projection.validate(),
        Err(Refusal::ClaimContradictsBytes {
            side: ObservedSide::Output,
            position: 0
        })
    );
}

#[test]
fn state_identity_is_checked_against_the_resolved_deployment_not_just_the_view() {
    let fixture = fixture();
    let original = fixture.view.view();
    for (asset, value) in [
        (
            AssetField::Explicit(AssetId::from_internal([0xee; 32])),
            original.value(),
        ),
        (original.asset(), ValueField::Explicit(2)),
    ] {
        let mut statements = super::state_support::statements(
            original.accepted_linked_bundle(),
            original.predecessor_metadata(),
            original.predecessor_representation_nonce(),
            original.predecessor_program().to_vec(),
        );
        statements[1] = crate::MaturityViewStatement::AssetAndAmount(asset, value);
        let view = crate::PublicMaturityStateView::new(statements)
            .unwrap()
            .validate(&super::reviewed_target(), &FixtureStateCurve)
            .unwrap();
        let mut offer = fixture.sponsorless.clone();
        offer.spent[0] =
            SpentOutputCensusEntry::new(asset, value, original.predecessor_program().to_vec());
        assert_eq!(
            validate_maturity_abi_classification(
                &fixture.abi,
                &view,
                &fixture.spec,
                &offer.borrowed()
            ),
            Err(Refusal::ClaimContradictsBytes {
                side: ObservedSide::Input,
                position: 0
            })
        );
    }
}

#[test]
fn sponsor_asset_program_and_side_are_authenticated_but_values_are_opaque() {
    let control = &fixture().sponsored;
    let expected = control.validate().unwrap();
    let spent = &control.spent[1];
    let mut commitment = [0x31; 33];
    commitment[0] = 8;
    let mut confidential = control.clone();
    confidential.spent[1] = SpentOutputCensusEntry::new(
        spent.asset(),
        ValueField::Commitment(commitment),
        spent.program().to_vec(),
    );
    assert_eq!(confidential.validate().unwrap().report(), expected.report());
    for (asset, program) in [
        (fixture().view.view().asset(), spent.program().to_vec()),
        (spent.asset(), vec![0xb4; 32]),
    ] {
        let mut offer = control.clone();
        offer.spent[1] = SpentOutputCensusEntry::new(asset, spent.value(), program);
        assert_eq!(
            offer.validate(),
            Err(Refusal::ClaimContradictsBytes {
                side: ObservedSide::Input,
                position: 1
            })
        );
    }
    let mut offer = control.clone();
    offer.inputs[1] = Claim::SponsorChange;
    assert_eq!(
        offer.validate(),
        Err(Refusal::ClaimContradictsBytes {
            side: ObservedSide::Input,
            position: 1
        })
    );
}

#[test]
fn sponsor_change_accepts_an_encoded_program_and_an_opaque_value() {
    let mut offer = fixture().sponsored.clone();
    let program =
        crate::taproot::witness_program_script(&super::reviewed_target(), 0, &[0xb4; 32]).unwrap();
    let asset = AssetField::Explicit(AssetId::from_internal(LIVE_RESERVE_ASSET));
    let mut commitment = [0x32; 33];
    commitment[0] = 8;
    offer.append_output(
        TargetOutput::new(
            asset,
            ValueField::Commitment(commitment),
            NonceField::Null,
            program,
        ),
        Claim::SponsorChange,
    );
    offer.validate().unwrap();
    offer.outputs[2] = Claim::Family(ObjectId::PlainLbtc);
    offer.validate().unwrap();
    offer.outputs[2] = Claim::SponsorMember;
    assert_eq!(
        offer.validate(),
        Err(Refusal::ClaimContradictsBytes {
            side: ObservedSide::Output,
            position: 2
        })
    );
}

#[test]
fn constructed_sponsor_change_accepts_the_constructors_program_class() {
    constructed_change_offer().validate().unwrap();
}

fn constructed_change_offer() -> Offer {
    let target = super::reviewed_target();
    let (mut sponsor, raw) = maturity_sponsor(true);
    // Exercise a nonzero witness version, as permitted by the constructor.
    sponsor.destination = Some((
        crate::taproot::TAPROOT_WITNESS_VERSION,
        vec![0xb4; crate::taproot::TAPROOT_PROGRAM_BYTES],
    ));
    let public = crate::PublicConstructionView::new(raw.outputs().values().map(|spent| {
        crate::PublicOutputView::new(
            spent.outpoint(),
            spent.asset(),
            spent.value(),
            crate::taproot::witness_program_script(
                &target,
                0,
                &[0xb4; crate::sponsor::WITNESS_V0_KEYHASH_PROGRAM_BYTES],
            )
            .unwrap(),
        )
    }))
    .unwrap();
    let construction = construct_with_sponsor(true, &sponsor, &public).unwrap();
    let mut offer = fixture().sponsored.clone();
    offer.bytes = construction.transaction().encode();
    offer.spent.truncate(1);
    for (input, output) in construction
        .transaction()
        .inputs()
        .iter()
        .skip(1)
        .zip(construction.sponsor_spent())
    {
        assert_eq!(input.outpoint(), output.outpoint());
        offer.spent.push(SpentOutputCensusEntry::new(
            output.asset(),
            output.value(),
            output.program().to_vec(),
        ));
    }
    offer.outputs = vec![Claim::State, Claim::SponsorChange, Claim::TargetFee];
    assert_eq!(
        construction.transaction().outputs()[1].program(),
        crate::taproot::witness_program_script(
            &target,
            crate::taproot::TAPROOT_WITNESS_VERSION,
            &[0xb4; crate::taproot::TAPROOT_PROGRAM_BYTES],
        )
        .unwrap(),
    );
    offer
}

#[test]
fn fee_claim_checks_fields_without_recomputing_or_requiring_a_positive_fee() {
    let control = &fixture().sponsored;
    let fee = control.transaction().outputs()[1].clone();
    for amount in [0, 1, 999] {
        let mut offer = control.clone();
        offer.replace_output(1, output(fee.asset(), amount, Vec::new()));
        offer.validate().unwrap();
    }
    let mut commitment = [0x33; 33];
    commitment[0] = 8;
    for changed in [
        output(fixture().view.view().asset(), 130, Vec::new()),
        output(fee.asset(), 130, vec![0x51]),
        TargetOutput::new(
            fee.asset(),
            ValueField::Commitment(commitment),
            NonceField::Null,
            Vec::new(),
        ),
    ] {
        let mut offer = control.clone();
        offer.replace_output(1, changed);
        assert_eq!(
            offer.validate(),
            Err(Refusal::ClaimContradictsBytes {
                side: ObservedSide::Output,
                position: 1
            })
        );
    }
    let mut offer = control.clone();
    offer.inputs[1] = Claim::TargetFee;
    assert_eq!(
        offer.validate(),
        Err(Refusal::ClaimContradictsBytes {
            side: ObservedSide::Input,
            position: 1
        })
    );
}

#[test]
fn unadmitted_outputs_refuse_before_an_invalid_partition_can_be_projected() {
    for claim in [
        Claim::Destruction,
        Claim::DataOutput(None),
        Claim::DataOutput(Some(TagId::Recon)),
    ] {
        let mut subject = data_subject("unadmitted", claim, vec![0x6a], false);
        subject.offer.canonical.flows.push(ObservedCanonicalFlow {
            asset: SemanticAsset::U,
            sources: vec![ObservedObjectRef {
                side: ObservedSide::Input,
                ordinal: 99,
            }],
            destinations: Vec::new(),
            movement_kind: Some(DeltaKind::Lateral),
            destructions: Vec::new(),
        });
        assert_eq!(subject.offer.validate(), Err(subject.expected));
    }
}

#[test]
fn state_cardinalities_and_other_failed_verdicts_are_retained_not_refused() {
    let mut offer = fixture().sponsorless.clone();
    let state = offer.transaction().outputs()[0].clone();
    offer.append_output(state, Claim::State);
    let result = offer.validate().unwrap();
    let cardinality = relation(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::State,
        },
    );
    assert_eq!(
        result.report().verdict(&cardinality).unwrap().status,
        RelationStatus::Failed {
            reason: RelationFailure::CardinalityAboveMaximum
        }
    );
    assert_eq!(
        result
            .report()
            .verdict(&relation(
                RelationKind::RootPolicy,
                RelationSubject::Operation
            ))
            .unwrap()
            .status,
        RelationStatus::Failed {
            reason: RelationFailure::RootPolicy
        }
    );
    assert!(!result.report().is_evidence_complete());
}

fn every_classification_refusal_is_censused(refusal: &Refusal) {
    match refusal {
        Refusal::Decode(_)
        | Refusal::IssuanceOffered
        | Refusal::ContractMismatch
        | Refusal::SpentCensus { .. }
        | Refusal::Unclassified { .. }
        | Refusal::ClaimContradictsBytes { .. }
        | Refusal::UnadmittedOutputRole { .. }
        | Refusal::RelationVerdicts { .. }
        | Refusal::Realization(_)
        | Refusal::ClaimCensus { .. } => {}
    }
}

#[test]
fn the_classification_refusal_vocabulary_has_no_wildcard_guard() {
    every_classification_refusal_is_censused(&Refusal::IssuanceOffered);
}

struct RoleSubject {
    name: &'static str,
    offer: Offer,
    positions: Vec<(ObservedSide, usize)>,
    expected: crate::MaturityAbiRoleRefusal,
    cardinalities: [RelationStatus; 2],
}

fn state_cardinality(side: TransactionSide) -> RelationId {
    relation(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side,
            object: ObjectId::State,
        },
    )
}

fn role_subjects() -> Vec<RoleSubject> {
    use crate::MaturityAbiRoleRefusal as RoleRefusal;
    let fixture = fixture();
    let control = &fixture.sponsorless;
    let predecessor = fixture.view.view().current_state_outpoint();
    let other = Outpoint::new(Txid::from_internal([0xe9; 32]), 1).unwrap();
    assert_ne!(other, predecessor);
    let mut wrong = control.clone();
    wrong.replace_input(
        0,
        TargetInput::new(other, u32::MAX),
        control.transaction().witnesses()[0].clone(),
        control.spent[0].clone(),
        Claim::State,
    );
    let mut duplicate = control.clone();
    duplicate.append_input_at(predecessor, control.spent[0].clone(), Claim::State);
    let mut missing = control.clone();
    let sponsored = fixture.sponsored.transaction();
    missing.replace_input(
        0,
        sponsored.inputs()[1].clone(),
        sponsored.witnesses()[1].clone(),
        fixture.sponsored.spent[1].clone(),
        Claim::SponsorMember,
    );
    // The matrix's sponsorless control has no sponsor position to exchange.
    // S1 is this row's per-row control; the matrix control remains unchanged.
    let mut exchanged = fixture.sponsored.clone();
    exchanged.swap_inputs(0, 1);
    let mut surplus_input = control.clone();
    surplus_input.append_input_at(other, control.spent[0].clone(), Claim::State);
    let mut surplus_output = control.clone();
    surplus_output.append_output(control.transaction().outputs()[0].clone(), Claim::State);
    let passed = RelationStatus::Passed;
    let above = RelationStatus::Failed {
        reason: RelationFailure::CardinalityAboveMaximum,
    };
    let below = RelationStatus::Failed {
        reason: RelationFailure::CardinalityBelowMinimum,
    };
    vec![
        RoleSubject {
            name: "wrong-coordinator",
            offer: wrong,
            positions: vec![(ObservedSide::Input, 0)],
            expected: RoleRefusal::CoordinatorIsNotTheValidatedPredecessor {
                expected: predecessor,
                offered: other,
            },
            cardinalities: [passed.clone(), passed.clone()],
        },
        RoleSubject {
            name: "duplicate-coordinator",
            offer: duplicate,
            positions: vec![(ObservedSide::Input, 1)],
            expected: RoleRefusal::CoordinatorRepeated {
                positions: vec![0, 1],
            },
            cardinalities: [above.clone(), passed.clone()],
        },
        RoleSubject {
            name: "no-coordinator",
            offer: missing,
            positions: vec![(ObservedSide::Input, 0)],
            expected: RoleRefusal::CoordinatorMissing,
            cardinalities: [below, passed.clone()],
        },
        RoleSubject {
            name: "state-and-sponsor-positions-exchanged",
            offer: exchanged,
            positions: vec![(ObservedSide::Input, 0), (ObservedSide::Input, 1)],
            expected: RoleRefusal::StateAndSponsorPositionsExchanged {
                state: 1,
                sponsor: 0,
            },
            cardinalities: [passed.clone(), passed.clone()],
        },
        RoleSubject {
            name: "wrong-input-count",
            offer: surplus_input,
            positions: vec![(ObservedSide::Input, 1)],
            expected: RoleRefusal::Cardinality {
                relation: state_cardinality(TransactionSide::Input),
                failure: RelationFailure::CardinalityAboveMaximum,
            },
            cardinalities: [above.clone(), passed.clone()],
        },
        RoleSubject {
            name: "wrong-output-count",
            offer: surplus_output,
            positions: vec![(ObservedSide::Output, 1)],
            expected: RoleRefusal::Cardinality {
                relation: state_cardinality(TransactionSide::Output),
                failure: RelationFailure::CardinalityAboveMaximum,
            },
            cardinalities: [passed, above],
        },
    ]
}

#[test]
fn all_six_role_subjects_classify_before_their_exact_role_refusal() {
    let fixture = fixture();
    let subjects = role_subjects();
    assert_eq!(subjects.len(), 6);
    for subject in subjects {
        let classified = subject.offer.validate().expect(subject.name);
        for (side, expected) in [TransactionSide::Input, TransactionSide::Output]
            .into_iter()
            .zip(subject.cardinalities)
        {
            // STATE projection has declared PID, singleton value and no owner.
            // Recognition passes, vacuously for an empty family, and releases
            // cardinality: zero is below one, two above one, one passes.
            let recognition = relation(
                RelationKind::Recognition,
                RelationSubject::ObjectFamily {
                    side,
                    object: ObjectId::State,
                },
            );
            assert_eq!(
                classified.report().verdict(&recognition).unwrap().status,
                RelationStatus::Passed
            );
            assert_eq!(
                classified
                    .report()
                    .verdict(&state_cardinality(side))
                    .unwrap()
                    .status,
                expected,
                "{}",
                subject.name
            );
        }
        // FeeSponsor is an admitted kind even with a lone source and no
        // destination; open-flow policy does not require owner recognition.
        assert_eq!(
            classified
                .report()
                .verdict(&relation(
                    RelationKind::OpenFlowPolicy,
                    RelationSubject::Operation
                ))
                .unwrap()
                .status,
            RelationStatus::Passed
        );
        assert_eq!(
            crate::validate_maturity_abi_roles(&fixture.abi, &fixture.view, &classified),
            Err(subject.expected),
            "{}",
            subject.name
        );
    }
}

#[test]
fn roles_borrow_classified_controls_and_preserve_uncalibrated_residuals() {
    let fixture = fixture();
    for offer in [
        fixture.sponsorless.clone(),
        fixture.sponsored.clone(),
        constructed_change_offer(),
    ] {
        let classified = offer.validate().unwrap();
        let roles =
            crate::validate_maturity_abi_roles(&fixture.abi, &fixture.view, &classified).unwrap();
        assert!(std::ptr::eq(roles.classification(), &raw const classified));
        assert!(std::ptr::eq(roles.transaction(), classified.transaction()));
        assert!(std::ptr::eq(roles.report(), classified.report()));
        assert_eq!(roles.input_claims(), classified.input_claims());
        assert_eq!(roles.output_claims(), classified.output_claims());
        assert_eq!(
            roles.sponsor_input_bound_uncalibrated(),
            classified.sponsor_input_bound_uncalibrated()
        );
        for side in [TransactionSide::Input, TransactionSide::Output] {
            // Singleton ownerless STATE objects recognize before cardinality.
            assert_eq!(
                roles
                    .report()
                    .verdict(&state_cardinality(side))
                    .unwrap()
                    .status,
                RelationStatus::Passed
            );
        }
        assert_sponsor_projection_statuses(&classified);
    }
}

#[test]
fn sponsor_change_without_sponsor_inputs_retains_the_presence_residual() {
    let fixture = fixture();
    let mut offer = fixture.sponsorless.clone();
    let change = constructed_change_offer();
    offer.append_output(
        change.transaction().outputs()[1].clone(),
        Claim::SponsorChange,
    );
    let classified = offer.validate().unwrap();
    // Placement passes; the presence half remains a residual for the
    // comparisons-and-record step, rather than admission evidence.
    let roles =
        crate::validate_maturity_abi_roles(&fixture.abi, &fixture.view, &classified).unwrap();
    assert_eq!(roles.input_claims(), &[Claim::State]);
    assert_eq!(roles.output_claims(), &[Claim::State, Claim::SponsorChange]);
    assert_eq!(roles.sponsor_input_bound_uncalibrated(), None);
}

fn assert_sponsor_projection_statuses(classified: &ValidatedMaturityAbiClassification) {
    let mut blockers = Vec::new();
    for (side, claims) in [
        (TransactionSide::Input, classified.input_claims()),
        (TransactionSide::Output, classified.output_claims()),
    ] {
        let subject = RelationSubject::ObjectFamily {
            side,
            object: ObjectId::PlainLbtc,
        };
        let recognition = relation(RelationKind::Recognition, subject.clone());
        let cardinality = relation(RelationKind::Cardinality, subject);
        let present = claims
            .iter()
            .any(|claim| matches!(claim, Claim::SponsorMember | Claim::SponsorChange));
        // PlainLbtc requires an owner. Present sponsor objects lack one;
        // empty families recognize vacuously and satisfy the zero minimum.
        let (recognized, counted) = if present {
            blockers.push(recognition.clone());
            (
                RelationStatus::Failed {
                    reason: RelationFailure::ObjectRecognition,
                },
                RelationStatus::Blocked {
                    prerequisites: vec![recognition.clone()],
                },
            )
        } else {
            (RelationStatus::Passed, RelationStatus::Passed)
        };
        assert_eq!(
            classified.report().verdict(&recognition).unwrap().status,
            recognized
        );
        assert_eq!(
            classified.report().verdict(&cardinality).unwrap().status,
            counted
        );
    }
    if !blockers.is_empty() {
        blockers.sort();
        // Open-flow policy and multiplicity pass; only failed recognition
        // on the present sponsor sides blocks sponsor isolation.
        assert_eq!(
            classified
                .report()
                .verdict(&relation(
                    RelationKind::SponsorIsolation,
                    RelationSubject::Sponsor
                ))
                .unwrap()
                .status,
            RelationStatus::Blocked {
                prerequisites: blockers
            }
        );
    }
}

#[test]
fn role_refusals_are_independent_of_changed_position_claims() {
    use std::io::Write as _;
    let fixture = fixture();
    let alternatives = claims();
    let started = Instant::now();
    let mut classification_calls = 0;
    let mut role_calls = 0;
    for subject in role_subjects() {
        let before = role_calls;
        for first in &alternatives {
            let seconds = if subject.positions.len() == 2 {
                alternatives.as_slice()
            } else {
                &alternatives[..1]
            };
            for second in seconds {
                let mut offer = subject.offer.clone();
                if let Some(position) = subject.positions.get(1) {
                    replace_claim(&mut offer, *position, *second);
                }
                // Replace the later position before an omitted earlier claim
                // truncates the same side's claim census.
                replace_claim(&mut offer, subject.positions[0], *first);
                classification_calls += 1;
                match offer.validate() {
                    Err(refusal) => {
                        // This refusal proves alternative-claim independence,
                        // never the honest subject's role-boundary evidence.
                        every_classification_refusal_is_censused(&refusal);
                    }
                    Ok(classified) => {
                        role_calls += 1;
                        let refusal = crate::validate_maturity_abi_roles(
                            &fixture.abi,
                            &fixture.view,
                            &classified,
                        )
                        .expect_err(subject.name);
                        assert_eq!(refusal, subject.expected, "{}", subject.name);
                    }
                }
            }
        }
        // Each position's honest claim and its family alias survive classification;
        // every other claim refuses at authentication, a family closure or the
        // projection policy. The exchange has their product.
        assert_eq!(
            role_calls - before,
            if subject.positions.len() == 2 { 4 } else { 2 },
            "{}",
            subject.name
        );
    }
    assert_eq!(classification_calls, 1116);
    assert_eq!(role_calls, 14);
    for control in [&fixture.sponsorless, &fixture.sponsored] {
        let classified = control.validate().unwrap();
        classification_calls += 1;
        crate::validate_maturity_abi_roles(&fixture.abi, &fixture.view, &classified).unwrap();
        role_calls += 1;
    }
    assert_eq!(classification_calls, 1118);
    assert_eq!(role_calls, 16);
    writeln!(std::io::stdout().lock(), "census roles: {classification_calls} classification calls; {role_calls} role calls; {} entry calls; table wall {:?}",
        classification_calls + role_calls, started.elapsed()).unwrap();
}

#[test]
fn output_placements_refuse_through_the_role_entry() {
    use crate::MaturityAbiRoleRefusal as RoleRefusal;
    let fixture = fixture();
    let mut successor = fixture.sponsored.clone();
    successor.swap_outputs(0, 1);
    let mut change = constructed_change_offer();
    change.swap_outputs(1, 2);
    let mut fee = fixture.sponsored.clone();
    fee.append_output(fee.transaction().outputs()[1].clone(), Claim::TargetFee);
    for (offer, expected) in [
        (
            successor,
            RoleRefusal::SuccessorDisplaced {
                expected: fixture.abi.outputs().successor_position(),
                offered: 1,
            },
        ),
        (change, RoleRefusal::SponsorChangeDisplaced { offered: 2 }),
        (fee, RoleRefusal::FeeNotFinal { offered: 1 }),
    ] {
        let classified = offer.validate().unwrap();
        assert_eq!(
            crate::validate_maturity_abi_roles(&fixture.abi, &fixture.view, &classified),
            Err(expected)
        );
    }
}

#[test]
fn sponsor_suffix_guard_is_tested_without_forging_an_abi() {
    use crate::state_abi::check_maturity_sponsor_suffix;
    let rule = fixture().abi.inputs().sponsor_suffix();
    // The full entry cannot reach this guard's refusal: coordinator at zero,
    // exactly one STATE input, suffix beginning at one. Earlier checks own
    // the exchange. Exercise the placement guard against the real rule.
    assert_eq!(
        check_maturity_sponsor_suffix(rule, &fixture().sponsored.inputs),
        Ok(())
    );
    for sponsor in [Claim::SponsorMember, Claim::Family(ObjectId::PlainLbtc)] {
        assert_eq!(
            check_maturity_sponsor_suffix(rule, &[sponsor, Claim::State]),
            Err(crate::MaturityAbiRoleRefusal::SponsorSuffixDisplaced {
                first: rule.first(),
                offered: 0
            })
        );
    }
}

fn every_role_refusal_is_censused(refusal: &crate::MaturityAbiRoleRefusal) {
    use crate::MaturityAbiRoleRefusal;
    match refusal {
        MaturityAbiRoleRefusal::CoordinatorMissing
        | MaturityAbiRoleRefusal::CoordinatorIsNotTheValidatedPredecessor { .. }
        | MaturityAbiRoleRefusal::CoordinatorRepeated { .. }
        | MaturityAbiRoleRefusal::StateAndSponsorPositionsExchanged { .. }
        | MaturityAbiRoleRefusal::Cardinality { .. }
        | MaturityAbiRoleRefusal::CardinalityUnavailable { .. }
        | MaturityAbiRoleRefusal::SuccessorDisplaced { .. }
        | MaturityAbiRoleRefusal::SponsorSuffixDisplaced { .. }
        | MaturityAbiRoleRefusal::SponsorChangeDisplaced { .. }
        | MaturityAbiRoleRefusal::FeeNotFinal { .. } => {}
    }
}

#[test]
fn the_role_refusal_vocabulary_has_no_wildcard_guard() {
    every_role_refusal_is_censused(&crate::MaturityAbiRoleRefusal::CoordinatorMissing);
}
