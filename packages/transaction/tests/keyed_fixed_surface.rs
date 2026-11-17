//! Fixed-form public construction controls at their declared boundaries.

use architecture::keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId};
use bitcoin_hashes::{Hash, hash160};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_fixed_plan::{KeyedFixedPlan, PlacementSearchLimits, plan},
};
use linker::{
    keyed_fixed_bundle::{LinkedKeyedFixedBundle, link_keyed_fixed_bundle},
    keyed_fixed_reference::keyed_fixed_parameter_offers,
};
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use secp256k1_zkp::{Message, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use tapscript::keyed_fixed_bundle::{KeyedFixedBundleSelection, emit_keyed_fixed_bundle};
use tapscript::keyed_fixed_constructor::KeyedFixedLeafRecipe;
use target_elements::reviewed_elements_tapscript;
use transaction::keyed_fixed_construct::{
    KeyedFixedConstructionError, KeyedFixedConstructionRequest, KeyedFixedOutputOffer,
    KeyedFixedPolicy, admit_keyed_fixed, construct_keyed_fixed, keyed_fixed_policy_floor,
    keyed_fixed_policy_member_maximum,
};
use transaction::keyed_fixed_inputs::{
    KeyedFixedInputOffer, KeyedFixedInputRefusal, KeyedFixedInputRole,
};
use transaction::keyed_fixed_request::{
    KeyedFixedRequestRefusal, keyed_fixed_merge_request, keyed_fixed_source_request,
};
use transaction::keyed_fixed_taproot::{
    KeyedFixedAuthenticationRefusal, KeyedFixedCommitment, authenticate_keyed_fixed_path,
    authenticate_keyed_fixed_predecessor, commit_keyed_fixed_bundle,
};
use transaction::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, SignerRole, SponsorCapability,
    SponsorOffer, SponsorSignature, SponsorSigningRequest, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};

fn parameters(value: u64) -> KeyedFixedParameters {
    KeyedFixedParameters {
        key: [42; 32],
        asset: [3; 32],
        value: KeyedFixedValue::new(value).unwrap(),
    }
}

fn linked(q: KeyedFixedParameters, operation: KeyedFixedOperationId) -> LinkedKeyedFixedBundle {
    let target = reviewed_elements_tapscript().unwrap();
    let manifest = KeyedFixedManifest::contract().unwrap();
    let declaration = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
        .unwrap()
        .validate(&manifest)
        .unwrap();
    let limit = std::num::NonZeroU64::new(100_000).unwrap();
    let input = bind_input(
        &manifest,
        declaration,
        vec![operation],
        AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
    )
    .unwrap();
    let plan: KeyedFixedPlan = plan(&input, PlacementSearchLimits::new(limit, limit)).unwrap();
    let selection = KeyedFixedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    let recipe = KeyedFixedLeafRecipe::new(q);
    let emission = emit_keyed_fixed_bundle(&target, &plan, selection, recipe).unwrap();
    link_keyed_fixed_bundle(
        &target,
        &emission,
        &keyed_fixed_parameter_offers(&target, recipe),
    )
    .unwrap()
}

fn commitment(q: KeyedFixedParameters, source: bool) -> KeyedFixedCommitment {
    let target = reviewed_elements_tapscript().unwrap();
    let operation = if source {
        KeyedFixedOperationId::Source
    } else {
        KeyedFixedOperationId::Merge
    };
    commit_keyed_fixed_bundle(&target, &linked(q, operation)).unwrap()
}

const fn output(value: u64, program: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal([3; 32])),
        ValueField::Explicit(value),
        NonceField::Null,
        program,
    )
}

fn parent(program: &[u8], value: u64, salt: u32) -> Vec<u8> {
    TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([1; 32]), salt).unwrap(),
            123,
        )],
        vec![output(value, program.to_vec())],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode()
}

fn input<'a>(
    parent: &[u8],
    index: usize,
    role: KeyedFixedInputRole,
) -> KeyedFixedInputOffer<'_, 'a> {
    KeyedFixedInputOffer {
        parent,
        input_index: u32::try_from(index).unwrap(),
        outpoint: Outpoint::new(
            Txid::from_internal(Sha256::digest(Sha256::digest(parent)).into()),
            0,
        )
        .unwrap(),
        sequence: 123,
        flags: 0,
        role,
        capability: None,
    }
}

fn public() -> Vec<u8> {
    PublicKey::from_secret_key(&Secp256k1::new(), &SecretKey::from_slice(&[1; 32]).unwrap())
        .serialize()
        .to_vec()
}

fn program() -> Vec<u8> {
    let mut bytes = vec![0, 20];
    bytes.extend(hash160::Hash::hash(&public()).to_byte_array());
    bytes
}

fn outputs(count: usize, value: u64, sponsor_amount: u64, fee: u64) -> KeyedFixedOutputOffer {
    let total = sponsor_amount + count as u64 * value;
    KeyedFixedOutputOffer {
        policy_asset: AssetId::from_internal([3; 32]),
        successor_value: value,
        additional_outputs: vec![output(total - value - fee, program())],
        fee: Some(fee),
    }
}

struct Signer {
    offer: SponsorOffer,
    spent: TargetOutput,
    wrong_digest: bool,
}

impl SponsorCapability for Signer {
    fn offer(&self) -> SponsorOffer {
        self.offer.clone()
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, program()[2..].to_vec()))
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        assert_eq!(request.role(), SignerRole::KeyedCollateral);
        let mut digest = transaction::keyed_fixed_sponsor::keyed_fixed_sponsor_digest(
            request.transaction(),
            usize::from(request.input()),
            &self.spent,
        )
        .unwrap();
        if self.wrong_digest {
            digest[0] ^= 1;
        }
        let mut signature = Secp256k1::new()
            .sign_ecdsa(
                &Message::from_digest(digest),
                &SecretKey::from_slice(&[1; 32]).unwrap(),
            )
            .serialize_der()
            .to_vec();
        signature.push(1);
        Some(SponsorSignature::new(
            request.transaction().to_vec(),
            vec![signature, public()],
        ))
    }
}

fn policy(minimum: u64) -> KeyedFixedPolicy {
    KeyedFixedPolicy::new(
        "supplied offline reading".into(),
        AssetId::from_internal([3; 32]),
        ([100; 4], true),
        vec![minimum, 1, 1],
        [400_000; 3],
        0,
        72,
    )
    .unwrap()
}

#[test]
fn fixed_commitment_binds_each_coordinate() {
    let target = reviewed_elements_tapscript().unwrap();
    let q = parameters(1_000);
    let base = commitment(q, true);
    let mut programs = std::collections::BTreeSet::new();
    programs.insert(base.output_program().to_vec());
    let mut changed = q;
    changed.key[0] ^= 1;
    programs.insert(commitment(changed, true).output_program().to_vec());
    let mut changed = q;
    changed.asset[0] ^= 1;
    programs.insert(commitment(changed, true).output_program().to_vec());
    let mut changed = q;
    changed.value = KeyedFixedValue::new(1_001).unwrap();
    programs.insert(commitment(changed, true).output_program().to_vec());
    assert_eq!(programs.len(), 4);
    assert_eq!(base.leaf().len(), 104);
    assert_eq!(base.control().len(), 33);
    assert_eq!(base.output_program().len(), 34);
    assert_eq!(
        authenticate_keyed_fixed_path(&base, base.leaf(), base.control()),
        Ok(())
    );
    let mut wrong_control = base.control().to_vec();
    wrong_control[0] ^= 1;
    assert_eq!(
        authenticate_keyed_fixed_path(&base, base.leaf(), &wrong_control),
        Err(KeyedFixedAuthenticationRefusal::WrongParity)
    );
    assert_eq!(
        keyed_fixed_source_request(&target, changed, &base, &[]).unwrap_err(),
        KeyedFixedRequestRefusal::Link(
            linker::keyed_fixed_bundle::KeyedFixedLinkRefusal::RecipeMismatch
        )
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Keep each parent refusal beside its accepted request control.
fn fixed_parent_authentication_and_request_roles() {
    let target = reviewed_elements_tapscript().unwrap();
    let q = parameters(1_000);
    let c = commitment(q, false);
    let bytes = parent(c.output_program(), q.value.get(), 1);
    let member = input(&bytes, 0, KeyedFixedInputRole::Member);
    assert!(keyed_fixed_merge_request(&target, q, &c, &[member]).is_ok());
    let mut wrong_hash = input(&bytes, 0, KeyedFixedInputRole::Member);
    wrong_hash.outpoint = Outpoint::new(Txid::from_internal([9; 32]), 0).unwrap();
    assert!(matches!(
        keyed_fixed_merge_request(&target, q, &c, &[wrong_hash]),
        Err(KeyedFixedRequestRefusal::Input {
            refusal: KeyedFixedInputRefusal::ParentHash { .. },
            ..
        })
    ));
    let wrong_pair = parent(c.output_program(), 1_001, 2);
    assert!(matches!(
        keyed_fixed_merge_request(
            &target,
            q,
            &c,
            &[input(&wrong_pair, 0, KeyedFixedInputRole::Member)]
        ),
        Err(KeyedFixedRequestRefusal::Input {
            refusal: KeyedFixedInputRefusal::WrongMemberValue,
            ..
        })
    ));
    let wrong_asset = TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([1; 32]), 22).unwrap(),
            123,
        )],
        vec![TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal([4; 32])),
            ValueField::Explicit(1_000),
            NonceField::Null,
            c.output_program().to_vec(),
        )],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode();
    assert!(matches!(
        keyed_fixed_merge_request(
            &target,
            q,
            &c,
            &[input(&wrong_asset, 0, KeyedFixedInputRole::Member)]
        ),
        Err(KeyedFixedRequestRefusal::Input {
            refusal: KeyedFixedInputRefusal::WrongMemberAsset,
            ..
        })
    ));
    let wrong_form = TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([1; 32]), 23).unwrap(),
            123,
        )],
        vec![TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal(q.asset)),
            ValueField::Commitment([8; 33]),
            NonceField::Null,
            c.output_program().to_vec(),
        )],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode();
    assert!(matches!(
        keyed_fixed_merge_request(
            &target,
            q,
            &c,
            &[input(&wrong_form, 0, KeyedFixedInputRole::Member)]
        ),
        Err(KeyedFixedRequestRefusal::Input {
            refusal: KeyedFixedInputRefusal::WrongMemberForm,
            ..
        })
    ));
    let later = TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([1; 32]), 24).unwrap(),
            123,
        )],
        vec![
            output(1_000, program()),
            output(1_000, c.output_program().to_vec()),
        ],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode();
    let mut later_offer = input(&later, 0, KeyedFixedInputRole::Member);
    later_offer.outpoint = Outpoint::new(later_offer.outpoint.txid(), 1).unwrap();
    assert!(matches!(
        keyed_fixed_merge_request(&target, q, &c, &[later_offer]),
        Err(KeyedFixedRequestRefusal::Input {
            refusal: KeyedFixedInputRefusal::LaterBornMember(1),
            ..
        })
    ));
    let foreign = parent(&program(), 1_000, 3);
    assert!(matches!(
        keyed_fixed_merge_request(
            &target,
            q,
            &c,
            &[input(&foreign, 0, KeyedFixedInputRole::Member)]
        ),
        Err(KeyedFixedRequestRefusal::Input {
            refusal: KeyedFixedInputRefusal::Authentication(
                KeyedFixedAuthenticationRefusal::WrongProgram
            ),
            ..
        })
    ));
    let mut other_version = c.output_program().to_vec();
    other_version[0] ^= 1;
    let foreign_version = parent(&other_version, 1_000, 4);
    assert!(matches!(
        keyed_fixed_merge_request(
            &target,
            q,
            &c,
            &[input(&foreign_version, 0, KeyedFixedInputRole::Member)]
        ),
        Err(KeyedFixedRequestRefusal::Input {
            refusal: KeyedFixedInputRefusal::Authentication(
                KeyedFixedAuthenticationRefusal::WrongProgram
            ),
            ..
        })
    ));
    let source_commitment = commitment(q, true);
    let ordinary = parent(&program(), 1_000, 25);
    assert!(
        keyed_fixed_source_request(
            &target,
            q,
            &source_commitment,
            &[input(&ordinary, 0, KeyedFixedInputRole::Collateral)]
        )
        .is_ok()
    );
    let source_member = parent(source_commitment.output_program(), 1_000, 26);
    assert!(matches!(
        keyed_fixed_source_request(
            &target,
            q,
            &source_commitment,
            &[input(&source_member, 0, KeyedFixedInputRole::Collateral)]
        ),
        Err(KeyedFixedRequestRefusal::Input {
            refusal: KeyedFixedInputRefusal::FalseCollateralRole,
            ..
        })
    ));
    assert_eq!(
        authenticate_keyed_fixed_predecessor(&c, c.output_program()),
        Ok(())
    );
    let present_nonce = TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([1; 32]), 99).unwrap(),
            123,
        )],
        vec![TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal(q.asset)),
            ValueField::Explicit(q.value.get()),
            NonceField::Commitment([2; 33]),
            c.output_program().to_vec(),
        )],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode();
    assert_eq!(
        keyed_fixed_merge_request(
            &target,
            q,
            &c,
            &[input(&present_nonce, 0, KeyedFixedInputRole::Member)],
        )
        .unwrap_err(),
        KeyedFixedRequestRefusal::MemberNonce { index: 0 }
    );
}

#[test]
fn fixed_value_bound_and_aggregate_limits() {
    let target = reviewed_elements_tapscript().unwrap();
    let maximum = target_elements::reviewed_stated_amount_bound().maximum();
    let q = parameters(maximum);
    let c = commitment(q, true);
    assert!(keyed_fixed_source_request(&target, q, &c, &[]).is_ok());
    let high = parameters(maximum + 1);
    let c_high = commitment(high, true);
    assert_eq!(
        keyed_fixed_source_request(&target, high, &c_high, &[]).unwrap_err(),
        KeyedFixedRequestRefusal::AmountRange
    );
    let q = parameters(1_000);
    let c = commitment(q, true);
    let sponsor = parent(&program(), 1_000_000, 5);
    let offered = input(&sponsor, 0, KeyedFixedInputRole::Collateral);
    let request = keyed_fixed_source_request(&target, q, &c, &[offered]).unwrap();
    assert!(
        construct_keyed_fixed(
            &target,
            KeyedFixedConstructionRequest::Source(&request),
            outputs(0, 1_000, 1_000_000, 1_000),
        )
        .is_ok()
    );
    let mut too_much = outputs(0, 1_000, 1_000_000, 1_000);
    too_much.additional_outputs[0] = output(maximum, program());
    assert_eq!(
        construct_keyed_fixed(
            &target,
            KeyedFixedConstructionRequest::Source(&request),
            too_much
        )
        .unwrap_err(),
        KeyedFixedConstructionError::AmountRange
    );
    let mut offered = input(&sponsor, 0, KeyedFixedInputRole::Collateral);
    let signer = Signer {
        offer: SponsorOffer::new([offered.outpoint], 26, Some(ValueField::Explicit(998_974)))
            .unwrap(),
        spent: output(1_000_000, program()),
        wrong_digest: false,
    };
    offered.capability = Some(&signer);
    let request = keyed_fixed_source_request(&target, q, &c, &[offered]).unwrap();
    assert!(matches!(
        admit_keyed_fixed(
            &target,
            KeyedFixedConstructionRequest::Source(&request),
            outputs(0, 1_000, 1_000_000, 26),
            &policy(1),
        ),
        Err(KeyedFixedConstructionError::FeeShortage {
            offered: 26,
            required: 27
        })
    ));
}

#[test]
fn fixed_source_merge_bytes_and_surplus() {
    let target = reviewed_elements_tapscript().unwrap();
    let q = parameters(1_000);
    let source_commitment = commitment(q, true);
    let malformed_source_parent = parent(source_commitment.output_program(), 1_000, 8);
    assert_eq!(
        keyed_fixed_source_request(
            &target,
            q,
            &source_commitment,
            &[input(
                &malformed_source_parent,
                0,
                KeyedFixedInputRole::Member
            )],
        )
        .unwrap_err(),
        KeyedFixedRequestRefusal::SourceConsumesMember
    );
    for source in [true, false] {
        let c = commitment(q, source);
        let count = if source { 0 } else { 2 };
        let mut parents = vec![parent(&program(), 1_000_000, 10)];
        for salt in 0..count {
            parents.push(parent(
                c.output_program(),
                q.value.get(),
                u32::try_from(salt + 11).unwrap(),
            ));
        }
        let offered: Vec<_> = parents
            .iter()
            .enumerate()
            .map(|(index, bytes)| {
                input(
                    bytes,
                    index,
                    if index == 0 {
                        KeyedFixedInputRole::Collateral
                    } else {
                        KeyedFixedInputRole::Member
                    },
                )
            })
            .collect();
        let source_request;
        let merge_request;
        let request = if source {
            source_request = keyed_fixed_source_request(&target, q, &c, &offered).unwrap();
            KeyedFixedConstructionRequest::Source(&source_request)
        } else {
            merge_request = keyed_fixed_merge_request(&target, q, &c, &offered).unwrap();
            KeyedFixedConstructionRequest::Merge(&merge_request)
        };
        let structural = construct_keyed_fixed(
            &target,
            request,
            outputs(count, q.value.get(), 1_000_000, 1_000),
        )
        .unwrap();
        let tx = structural.transaction();
        assert_eq!(tx.outputs()[0].program(), c.output_program());
        assert_eq!(tx.outputs()[0].value(), ValueField::Explicit(q.value.get()));
        assert_eq!(tx.outputs()[0].nonce(), NonceField::Null);
        assert_eq!(tx.witnesses().len(), count + 1);
        for witness in &tx.witnesses()[1..] {
            assert_eq!(witness.stack(), [c.leaf().to_vec(), c.control().to_vec()]);
        }
        let mut other = outputs(count, q.value.get(), 1_000_000, 1_000);
        let mut destination = program();
        destination[2] ^= 1;
        other.additional_outputs[0] = output(
            1_000_000 + count as u64 * q.value.get() - q.value.get() - 1_000,
            destination,
        );
        assert!(construct_keyed_fixed(&target, request, other).is_ok());
    }
}

#[test]
fn fixed_sponsor_authority_is_local() {
    let target = reviewed_elements_tapscript().unwrap();
    let q = parameters(1_000);
    let c = commitment(q, true);
    let merge_commitment = commitment(q, false);
    let sponsor = parent(&program(), 1_000_000, 31);
    let mut offered = input(&sponsor, 0, KeyedFixedInputRole::Collateral);
    let signer = Signer {
        offer: SponsorOffer::new(
            [offered.outpoint],
            1_000,
            Some(ValueField::Explicit(998_000)),
        )
        .unwrap(),
        spent: output(1_000_000, program()),
        wrong_digest: false,
    };
    let member = parent(merge_commitment.output_program(), 1_000, 32);
    let mut member_offer = input(&member, 1, KeyedFixedInputRole::Member);
    member_offer.capability = Some(&signer);
    assert!(matches!(
        keyed_fixed_merge_request(
            &target,
            q,
            &merge_commitment,
            &[
                input(&sponsor, 0, KeyedFixedInputRole::Collateral),
                member_offer,
            ],
        ),
        Err(KeyedFixedRequestRefusal::Input {
            index: 1,
            refusal: transaction::keyed_fixed_inputs::KeyedFixedInputRefusal::MemberCapability,
        })
    ));
    offered.capability = Some(&signer);
    let request = keyed_fixed_source_request(&target, q, &c, &[offered]).unwrap();
    assert!(
        admit_keyed_fixed(
            &target,
            KeyedFixedConstructionRequest::Source(&request),
            outputs(0, 1_000, 1_000_000, 1_000),
            &policy(1),
        )
        .is_ok()
    );
    let wrong = Signer {
        offer: signer.offer.clone(),
        spent: signer.spent.clone(),
        wrong_digest: true,
    };
    let mut changed = input(&sponsor, 0, KeyedFixedInputRole::Collateral);
    changed.capability = Some(&wrong);
    let request = keyed_fixed_source_request(&target, q, &c, &[changed]).unwrap();
    assert!(matches!(
        admit_keyed_fixed(
            &target,
            KeyedFixedConstructionRequest::Source(&request),
            outputs(0, 1_000, 1_000_000, 1_000),
            &policy(1),
        ),
        Err(KeyedFixedConstructionError::Sponsor(
            transaction::keyed_fixed_sponsor::KeyedFixedSponsorRefusal::InvalidSignature
        ))
    ));
}

#[test]
fn fixed_envelope_weight_and_n_plus_one() {
    let target = reviewed_elements_tapscript().unwrap();
    let maximum = keyed_fixed_policy_member_maximum(&target).unwrap();
    assert_eq!(maximum, 1_299);
    let q = parameters(1_000);
    let source_commitment = commitment(q, true);
    let source_parent = parent(&program(), 1_000_000, 99);
    let source_offer = input(&source_parent, 0, KeyedFixedInputRole::Collateral);
    let source =
        keyed_fixed_source_request(&target, q, &source_commitment, &[source_offer]).unwrap();
    let source_tx = construct_keyed_fixed(
        &target,
        KeyedFixedConstructionRequest::Source(&source),
        outputs(0, q.value.get(), 1_000_000, 100_000),
    )
    .unwrap();
    let tx = source_tx.transaction();
    let source_bounded = TargetTransaction::new(
        tx.version(),
        tx.inputs().to_vec(),
        tx.outputs().to_vec(),
        tx.lock_time(),
        vec![InputWitness::new(vec![vec![0; 72], vec![0; 33]])],
    )
    .unwrap();
    assert_eq!(source_bounded.weight(), 1_077);
    let c = commitment(q, false);
    for count in [251, 252, maximum, maximum + 1] {
        let mut parents = vec![parent(&program(), 1_000_000, 100)];
        for salt in 0..count {
            parents.push(parent(
                c.output_program(),
                q.value.get(),
                u32::try_from(salt + 101).unwrap(),
            ));
        }
        let offered: Vec<_> = parents
            .iter()
            .enumerate()
            .map(|(index, bytes)| {
                input(
                    bytes,
                    index,
                    if index == 0 {
                        KeyedFixedInputRole::Collateral
                    } else {
                        KeyedFixedInputRole::Member
                    },
                )
            })
            .collect();
        let request = keyed_fixed_merge_request(&target, q, &c, &offered).unwrap();
        let result = construct_keyed_fixed(
            &target,
            KeyedFixedConstructionRequest::Merge(&request),
            outputs(count, q.value.get(), 1_000_000, 100_000),
        );
        if count <= maximum {
            let structural = result.unwrap();
            let tx = structural.transaction();
            let mut witnesses = tx.witnesses().to_vec();
            witnesses[0] = InputWitness::new(vec![vec![0; 72], vec![0; 33]]);
            let bounded = TargetTransaction::new(
                tx.version(),
                tx.inputs().to_vec(),
                tx.outputs().to_vec(),
                tx.lock_time(),
                witnesses,
            )
            .unwrap();
            let expected = if count == 251 {
                78_134
            } else if count == 252 {
                78_449
            } else {
                399_878
            };
            assert_eq!(bounded.weight(), expected);
        } else {
            assert!(matches!(
                result,
                Err(KeyedFixedConstructionError::HandleCountExceedsPolicy {
                    offered: 1_300,
                    maximum: 1_299
                })
            ));
        }
    }
}

#[test]
fn fixed_floor_uses_supplied_policy() {
    let target = reviewed_elements_tapscript().unwrap();
    let policy_asset = AssetId::from_internal([3; 32]);
    let canonical = output(15, vec![0x51, 32].into_iter().chain([0; 32]).collect());
    assert_eq!(
        keyed_fixed_policy_floor(&canonical, policy_asset, 100, true),
        Ok(15)
    );
    assert_eq!(
        keyed_fixed_policy_floor(&canonical, policy_asset, 200, true),
        Ok(29)
    );
    assert_eq!(
        keyed_fixed_policy_floor(&canonical, policy_asset, 100, false),
        Ok(1)
    );
    assert_eq!(
        keyed_fixed_policy_floor(&canonical, AssetId::from_internal([4; 32]), 100, true),
        Ok(1)
    );
    for (value, accepted) in [(14, false), (15, true)] {
        let q = parameters(value);
        let c = commitment(q, true);
        let sponsor = parent(&program(), 1_000_000, u32::try_from(value).unwrap());
        let mut offered = input(&sponsor, 0, KeyedFixedInputRole::Collateral);
        let signer = Signer {
            offer: SponsorOffer::new(
                [offered.outpoint],
                1_000,
                Some(ValueField::Explicit(1_000_000 - value - 1_000)),
            )
            .unwrap(),
            spent: output(1_000_000, program()),
            wrong_digest: false,
        };
        offered.capability = Some(&signer);
        let request = keyed_fixed_source_request(&target, q, &c, &[offered]).unwrap();
        let result = admit_keyed_fixed(
            &target,
            KeyedFixedConstructionRequest::Source(&request),
            outputs(0, value, 1_000_000, 1_000),
            &policy(1),
        );
        if accepted {
            assert!(result.is_ok());
        } else {
            assert!(matches!(
                result,
                Err(KeyedFixedConstructionError::OutputPolicy { index: 0 })
            ));
        }
    }
}
