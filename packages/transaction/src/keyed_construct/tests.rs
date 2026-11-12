use super::*;
use crate::keyed_abi::KeyedRequestObligation as Obl;
use crate::keyed_inputs::KeyedInputOffer;
use crate::keyed_request::{keyed_merge_request, keyed_source_request};
use crate::keyed_taproot::commit_keyed_bundle;
use crate::{Outpoint, SponsorCapability, SponsorOffer, SponsorSignature, Txid};
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use bitcoin_hashes::{Hash, hash160};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{PlacementSearchLimits, plan},
};
use linker::{
    keyed_bundle::{LinkedKeyedBundle, link_keyed_bundle},
    keyed_reference::keyed_parameter_offers,
};
use secp256k1_zkp::{Message, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use tapscript::{
    keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle},
    keyed_constructor::KeyedLeafRecipe,
};
use target_elements::reviewed_elements_tapscript;
fn linked(key: [u8; 32], source: bool) -> LinkedKeyedBundle {
    linked_scope(
        key,
        if source {
            vec![O::Keyed(K::Source)]
        } else {
            vec![O::Keyed(K::Merge)]
        },
    )
}
fn linked_scope(key: [u8; 32], scope: Vec<O>) -> LinkedKeyedBundle {
    let target = reviewed_elements_tapscript().expect("target");
    let manifest = KeyedManifest::contract().expect("manifest");
    let declaration = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("derive")
        .validate(&manifest)
        .expect("validate");
    let limit = std::num::NonZeroU64::new(100_000).expect("positive");
    let input = bind_input(
        &manifest,
        declaration,
        scope,
        AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
    )
    .expect("bind");
    let plan = plan(&input, PlacementSearchLimits::new(limit, limit)).expect("plan");
    let selection = KeyedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    let recipe = KeyedLeafRecipe::new(key);
    let emission = emit_keyed_bundle(&target, &plan, selection, recipe).expect("emit");
    link_keyed_bundle(&target, &emission, &keyed_parameter_offers(&target, recipe)).expect("link")
}
fn output(n: u64, parent_bytes: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal([3; 32])),
        ValueField::Explicit(n),
        NonceField::Null,
        parent_bytes,
    )
}
fn parent(parent_bytes: &[u8], salt: u32) -> Vec<u8> {
    TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([1; 32]), salt).unwrap(),
            123,
        )],
        vec![output(1_000_000, parent_bytes.to_vec())],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode()
}
fn input<'commitment>(
    parent: &[u8],
    index: u32,
    role: KeyedInputRole,
) -> KeyedInputOffer<'_, 'commitment> {
    KeyedInputOffer {
        parent,
        input_index: index,
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
struct Signer {
    offer: SponsorOffer,
    calls: std::cell::Cell<usize>,
    signs: std::cell::Cell<usize>,
    mode: u8,
}
fn public() -> Vec<u8> {
    PublicKey::from_secret_key(&Secp256k1::new(), &SecretKey::from_slice(&[1; 32]).unwrap())
        .serialize()
        .to_vec()
}
fn program() -> Vec<u8> {
    let mut parent_bytes = vec![0, 20];
    parent_bytes.extend(hash160::Hash::hash(&public()).to_byte_array());
    parent_bytes
}
impl SponsorCapability for Signer {
    fn offer(&self) -> SponsorOffer {
        self.calls.set(self.calls.get() + 1);
        self.offer.clone()
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        match self.mode {
            7 => None,
            8 => Some((0, vec![0; 20])),
            _ => Some((0, program()[2..].to_vec())),
        }
    }
    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        self.signs.set(self.signs.get() + 1);
        assert_eq!(request.role(), SignerRole::KeyedCollateral);
        if self.mode == 1 {
            return None;
        }
        let mut digest = crate::keyed_sponsor::keyed_sponsor_digest(
            request.transaction(),
            usize::from(request.input()),
            &output(1_000_000, program()),
        )
        .unwrap();
        if self.mode == 6 {
            digest[0] ^= 1;
        }
        let mut sig = Secp256k1::new()
            .sign_ecdsa(
                &Message::from_digest(digest),
                &SecretKey::from_slice(&[1; 32]).unwrap(),
            )
            .serialize_der()
            .to_vec();
        sig.push(1);
        if self.mode == 2 {
            sig.resize(73, 1);
        }
        let mut bytes = request.transaction().to_vec();
        if self.mode == 3 {
            bytes[0] ^= 1;
        }
        Some(SponsorSignature::new(bytes, vec![sig, public()]))
    }
}
fn policy() -> KeyedPolicy {
    KeyedPolicy::new(
        "offline explicit test policy, not node admission".into(),
        AssetId::from_internal([3; 32]),
        [100; 3],
        vec![100, 100, 1],
        [400_000; 3],
        0,
        72,
    )
    .unwrap()
}
fn outputs(count: usize, fee: u64) -> KeyedOutputOffer {
    KeyedOutputOffer {
        policy_asset: AssetId::from_internal([3; 32]),
        successor_value: 1000,
        additional_outputs: vec![output(
            (count as u64 + 1) * 1_000_000 - 1000 - fee,
            program(),
        )],
        fee: Some(fee),
    }
}
#[test]
fn source_and_merge_construct_sign_and_keep_the_census_exact() {
    let target = reviewed_elements_tapscript().unwrap();
    for source in [true, false] {
        let commitment = commit_keyed_bundle(&target, &linked([42; 32], source)).unwrap();
        let count = if source { 0 } else { 2 };
        let fee = 1000;
        let sponsor_program = program();
        let parents: Vec<_> = (0..=count)
            .map(|n| {
                parent(
                    if n == 0 {
                        &sponsor_program
                    } else {
                        commitment.output_program()
                    },
                    u32::try_from(n).unwrap(),
                )
            })
            .collect();
        let first = input(&parents[0], 0, KeyedInputRole::Collateral);
        let signer = Signer {
            offer: SponsorOffer::new(
                [first.outpoint],
                fee,
                Some(outputs(count, fee).additional_outputs[0].value()),
            )
            .unwrap(),
            calls: 0.into(),
            signs: 0.into(),
            mode: 0,
        };
        let mut inputs: Vec<_> = parents
            .iter()
            .enumerate()
            .map(|(n, parent_bytes)| {
                input(
                    parent_bytes,
                    u32::try_from(n).unwrap(),
                    if n == 0 {
                        KeyedInputRole::Collateral
                    } else {
                        KeyedInputRole::Member
                    },
                )
            })
            .collect();
        inputs[0].capability = Some(&signer);
        let src;
        let merge;
        let request = if source {
            src = keyed_source_request(&target, [42; 32], &commitment, &inputs).unwrap();
            KeyedConstructionRequest::Source(&src)
        } else {
            merge = keyed_merge_request(&target, [42; 32], &commitment, &inputs).unwrap();
            KeyedConstructionRequest::Merge(&merge)
        };
        let structural = construct_keyed(&target, request, outputs(count, fee)).unwrap();
        assert!(!structural.policy_admitted());
        let admitted = admit_keyed(&target, request, outputs(count, fee), &policy()).unwrap();
        assert_eq!(signer.calls.get(), 1);
        assert_eq!(
            admitted.transaction().encode_without_witness(),
            structural.transaction().encode_without_witness()
        );
        assert_eq!(signer.signs.get(), 1);
        for (built, offered) in admitted.transaction().inputs().iter().zip(&inputs) {
            assert_eq!(built.outpoint(), offered.outpoint);
            assert_eq!(built.sequence(), offered.sequence);
        }
        for w in &admitted.transaction().witnesses()[1..] {
            assert_eq!(
                w.stack(),
                [commitment.leaf().to_vec(), commitment.control().to_vec()]
            );
        }
        assert!(admitted.transaction().weight() <= admitted.weight_bound());
    }
}
#[test]
fn large_envelope_boundary_precedes_serialization_and_matches_weight_formula() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    for count in [1, 251, 252, 1564, 1565] {
        let sponsor_program = program();
        let parents: Vec<_> = (0..=count)
            .map(|n| {
                parent(
                    if n == 0 {
                        &sponsor_program
                    } else {
                        commitment.output_program()
                    },
                    u32::try_from(n).unwrap(),
                )
            })
            .collect();
        let mut inputs: Vec<_> = parents
            .iter()
            .enumerate()
            .map(|(n, parent_bytes)| {
                input(
                    parent_bytes,
                    u32::try_from(n).unwrap(),
                    if n == 0 {
                        KeyedInputRole::Collateral
                    } else {
                        KeyedInputRole::Member
                    },
                )
            })
            .collect();
        let signer = Signer {
            offer: SponsorOffer::new(
                [inputs[0].outpoint],
                100_000,
                Some(outputs(count, 100_000).additional_outputs[0].value()),
            )
            .unwrap(),
            calls: 0.into(),
            signs: 0.into(),
            mode: 0,
        };
        inputs[0].capability = Some(&signer);
        let request = keyed_merge_request(&target, [42; 32], &commitment, &inputs).unwrap();
        let admitted = admit_keyed(
            &target,
            KeyedConstructionRequest::Merge(&request),
            outputs(count, 100_000),
            &policy(),
        );
        if count == 1565 {
            assert_eq!(signer.signs.get(), 0);
            assert_eq!(
                construct_keyed(
                    &target,
                    KeyedConstructionRequest::Merge(&request),
                    outputs(count, 100_000)
                )
                .unwrap()
                .member_count(),
                count
            );
            assert_eq!(
                admitted.unwrap_err(),
                KeyedConstructionError::HandleCountExceedsPolicy {
                    offered: 1565,
                    maximum: 1564
                }
            );
            continue;
        }
        assert!(admitted.is_ok());
        let structural = construct_keyed(
            &target,
            KeyedConstructionRequest::Merge(&request),
            outputs(count, 100_000),
        )
        .unwrap();
        let mut w = structural.transaction.witnesses().to_vec();
        w[0] = InputWitness::new(vec![vec![0; 72], vec![0; 33]]);
        let tx = TargetTransaction::new(
            2,
            structural.transaction.inputs().to_vec(),
            structural.transaction.outputs().to_vec(),
            0,
            w,
        )
        .unwrap();
        assert_eq!(
            tx.weight(),
            1077 + 255 * count as u64 + if count >= 252 { 8 } else { 0 }
        );
        assert_eq!(tx.witness_bytes(), 91 * count as u64 + 111 + 6);
    }
}
#[test]
fn construction_and_policy_subjects_refuse_beside_control() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let parent_bytes = parent(&program(), 0);
    let offered = input(&parent_bytes, 0, KeyedInputRole::Collateral);
    let signer = Signer {
        offer: SponsorOffer::new(
            [offered.outpoint],
            1000,
            Some(ValueField::Explicit(998_000)),
        )
        .unwrap(),
        calls: 0.into(),
        signs: 0.into(),
        mode: 0,
    };
    let mut offered = offered;
    offered.capability = Some(&signer);
    let request = keyed_source_request(&target, [42; 32], &commitment, &[offered]).unwrap();
    let req = KeyedConstructionRequest::Source(&request);
    assert!(admit_keyed(&target, req, outputs(0, 1000), &policy()).is_ok());
    signer.signs.set(0);
    let mut bad = outputs(0, 1000);
    bad.fee = None;
    assert_eq!(
        construct_keyed(&target, req, bad).unwrap_err(),
        KeyedConstructionError::MissingFee
    );
    let mut bad = outputs(0, 1000);
    bad.successor_value += 1;
    assert_eq!(
        construct_keyed(&target, req, bad).unwrap_err(),
        KeyedConstructionError::Balance
    );
    let mut bad = policy();
    bad.relay_sat_per_kvb = 1_000_000;
    assert!(matches!(
        admit_keyed(&target, req, outputs(0, 1000), &bad),
        Err(KeyedConstructionError::FeeShortage { .. })
    ));
    let mut bad = policy();
    bad.output_minimums[0] = 1001;
    assert_eq!(
        admit_keyed(&target, req, outputs(0, 1000), &bad).unwrap_err(),
        KeyedConstructionError::OutputPolicy { index: 0 }
    );
    let mut bad = policy();
    bad.sponsor_signature_bound = 73;
    assert_eq!(
        admit_keyed(&target, req, outputs(0, 1000), &bad).unwrap_err(),
        KeyedConstructionError::SignatureBoundOutOfRange
    );
    let mut bad = policy();
    bad.provenance.clear();
    assert_eq!(
        admit_keyed(&target, req, outputs(0, 1000), &bad).unwrap_err(),
        KeyedConstructionError::ResourceContext
    );
    let empty = keyed_source_request(&target, [42; 32], &commitment, &[]).unwrap();
    assert_eq!(
        construct_keyed(
            &target,
            KeyedConstructionRequest::Source(&empty),
            outputs(0, 1000)
        )
        .unwrap_err(),
        KeyedConstructionError::EmptySource
    );
    assert_eq!(signer.signs.get(), 0);
}
#[test]
fn extra_outputs_and_redistribution_stay_structural_not_measured_admission() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let parent_bytes = parent(&program(), 0);
    let offered = input(&parent_bytes, 0, KeyedInputRole::Collateral);
    let request = keyed_source_request(&target, [42; 32], &commitment, &[offered]).unwrap();
    let req = KeyedConstructionRequest::Source(&request);
    let control = construct_keyed(&target, req, outputs(0, 1000)).unwrap();
    assert_eq!(
        control.transaction.outputs()[0].value(),
        ValueField::Explicit(1000)
    );
    let mut extra = outputs(0, 1000);
    extra.additional_outputs[0] = output(997_900, program());
    extra.additional_outputs.push(output(100, vec![0x51]));
    let result = construct_keyed(&target, req, extra.clone()).unwrap();
    assert_eq!(result.transaction.outputs()[2].program(), [0x51]);
    assert!(!result.policy_admitted());
    assert_eq!(
        admit_keyed(&target, req, extra, &policy()).unwrap_err(),
        KeyedConstructionError::Envelope
    );
    let mut redistributed = outputs(0, 1000);
    redistributed.successor_value = 200;
    redistributed.additional_outputs[0] = output(998_800, program());
    assert!(construct_keyed(&target, req, redistributed).is_ok());
}
#[test]
fn sealed_material_mutants_refuse_output_and_covenant_witness_changes() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let parent_bytes = parent(commitment.output_program(), 0);
    let request = keyed_merge_request(
        &target,
        [42; 32],
        &commitment,
        &[input(&parent_bytes, 0, KeyedInputRole::Member)],
    )
    .unwrap();
    let req = KeyedConstructionRequest::Merge(&request);
    let mut offered = outputs(0, 1000);
    offered.additional_outputs[0] = output(998_000, program());
    let mut structural = construct_keyed(&target, req, offered).unwrap();
    assert_eq!(check_material(&structural), Ok(()));
    let original = structural.transaction.clone();
    let mut wrong = original.outputs().to_vec();
    wrong.swap(0, 1);
    structural.transaction = TargetTransaction::new(
        2,
        original.inputs().to_vec(),
        wrong,
        0,
        original.witnesses().to_vec(),
    )
    .unwrap();
    assert_eq!(
        check_material(&structural),
        Err(KeyedConstructionError::OutputZero)
    );
    let mut wrong = original.witnesses().to_vec();
    wrong[0] = InputWitness::new(vec![vec![0x51], commitment.control().to_vec()]);
    structural.transaction = TargetTransaction::new(
        2,
        original.inputs().to_vec(),
        original.outputs().to_vec(),
        0,
        wrong,
    )
    .unwrap();
    assert_eq!(
        check_material(&structural),
        Err(KeyedConstructionError::CovenantWitness)
    );
    structural.transaction = original;
    assert_eq!(check_material(&structural), Ok(()));
}
fn surface_locked(source: &str) -> bool {
    for name in [
        "KeyedStructuralTransaction",
        "KeyedAdmittedTransaction",
        "KeyedPolicy",
    ] {
        let Some(body) = source.split(&format!("pub struct {name}")).nth(1) else {
            return false;
        };
        let body = body.split("\n}").next().unwrap();
        if body
            .lines()
            .skip(1)
            .any(|l| l.trim_start().starts_with("pub "))
        {
            return false;
        }
    }
    // The checked readings constructor is intentional, not an artifact constructor.
    let source = if let Some((before, policy)) = source.split_once("impl KeyedPolicy {") {
        let Some((_, after)) = policy.split_once("\n}\n") else {
            return false;
        };
        format!("{before}{after}")
    } else {
        source.to_owned()
    };
    [
        "pub fn new(",
        "pub const fn new(",
        "&mut self",
        "DerefMut for",
        "AsMut<",
        "impl Default for",
    ]
    .iter()
    .all(|token| !source.contains(token))
}
#[test]
fn construction_surface_exposes_no_totals_scripts_or_mutation() {
    let source = include_str!("../keyed_construct.rs");
    assert!(surface_locked(source));
    assert!(!surface_locked(&source.replace(
        "    transaction: TargetTransaction,",
        "    pub transaction: TargetTransaction,"
    )));
    assert!(!surface_locked(&format!(
        "{source}\npub fn new(bytes: [u8;32]) {{}}"
    )));
    let offer = source
        .split("pub struct KeyedOutputOffer {")
        .nth(1)
        .unwrap()
        .split("\n}")
        .next()
        .unwrap();
    for forbidden in ["total", "script", "control", "inputs"] {
        assert!(!offer.contains(forbidden));
    }
}

#[test]
fn keyed_construction_boundaries_trace_exactly_one_own_terminal_event() {
    use crate::tests::trace_tests::{capture, run_isolated_capture_test};
    if run_isolated_capture_test(
        module_path!(),
        "keyed_construction_boundaries_trace_exactly_one_own_terminal_event",
    ) {
        return;
    }
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let parent_bytes = parent(&program(), 0);
    let offered = input(&parent_bytes, 0, KeyedInputRole::Collateral);
    let signer = Signer {
        offer: SponsorOffer::new(
            [offered.outpoint],
            1000,
            Some(ValueField::Explicit(998_000)),
        )
        .unwrap(),
        calls: 0.into(),
        signs: 0.into(),
        mode: 0,
    };
    let mut offered = offered;
    offered.capability = Some(&signer);
    let request = keyed_source_request(&target, [42; 32], &commitment, &[offered]).unwrap();
    let req = KeyedConstructionRequest::Source(&request);
    for admitted in [false, true] {
        for refused in [false, true] {
            let mut offered = outputs(0, 1000);
            if refused {
                offered.fee = None;
            }
            let (ok, trace) = capture(|| {
                if admitted {
                    admit_keyed(&target, req, offered, &policy()).is_ok()
                } else {
                    construct_keyed(&target, req, offered).is_ok()
                }
            });
            assert_eq!(ok, !refused);
            let own: Vec<_> = trace
                .iter()
                .filter(|request| request.target == "transaction::keyed_construct")
                .collect();
            assert_eq!(
                own.iter().filter(|request| request.kind == "span").count(),
                1
            );
            let events: Vec<_> = own
                .iter()
                .filter(|request| request.kind == "event")
                .collect();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].level, tracing::Level::DEBUG);
            assert_eq!(
                events[0].fields.get("message").unwrap(),
                if refused {
                    "keyed construction refused"
                } else {
                    "keyed construction completed"
                }
            );
            assert_eq!(events[0].fields.contains_key("refusal_kind"), refused);
            assert_eq!(events[0].fields.get("family").unwrap(), "keyed");
        }
    }
}
#[test]
fn keyed_sponsor_boundaries_trace_exactly_one_own_terminal_event() {
    use crate::tests::trace_tests::{capture, run_isolated_capture_test};
    if run_isolated_capture_test(
        module_path!(),
        "keyed_sponsor_boundaries_trace_exactly_one_own_terminal_event",
    ) {
        return;
    }
    let tx = TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([4; 32]), 0).unwrap(),
            1,
        )],
        vec![output(1000, program())],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap();
    let bytes = tx.encode_without_witness();
    let spent = output(1000, program());
    let digest = crate::keyed_sponsor::digest_core(&bytes, 0, &spent).unwrap();
    let mut sig = Secp256k1::new()
        .sign_ecdsa(
            &Message::from_digest(digest),
            &SecretKey::from_slice(&[1; 32]).unwrap(),
        )
        .serialize_der()
        .to_vec();
    sig.push(1);
    let witness = vec![sig, public()];
    for verify in [false, true] {
        for refused in [false, true] {
            let index = usize::from(refused);
            let (ok, trace) = capture(|| {
                if verify {
                    crate::keyed_sponsor::verify_keyed_sponsor(&bytes, index, &spent, &witness)
                        .is_ok()
                } else {
                    crate::keyed_sponsor::keyed_sponsor_digest(&bytes, index, &spent).is_ok()
                }
            });
            assert_eq!(ok, !refused);
            let own: Vec<_> = trace
                .iter()
                .filter(|request| request.target == "transaction::keyed_sponsor")
                .collect();
            assert_eq!(
                own.iter().filter(|request| request.kind == "span").count(),
                1
            );
            let events: Vec<_> = own
                .iter()
                .filter(|request| request.kind == "event")
                .collect();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].fields.contains_key("refusal_kind"), refused);
            assert_eq!(events[0].fields.get("family").unwrap(), "keyed");
            assert_eq!(
                events[0].fields.get("message").unwrap(),
                match (verify, refused) {
                    (false, false) => "keyed sponsor digest completed",
                    (false, true) => "keyed sponsor digest refused",
                    (true, false) => "keyed sponsor verification completed",
                    (true, true) => "keyed sponsor verification refused",
                }
            );
        }
    }
}
#[test]
fn snapshot_coverage_overlap_and_signature_failures_keep_their_boundaries() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let parent_bytes = parent(&program(), 0);
    let m = parent(commitment.output_program(), 1);
    let sponsor = input(&parent_bytes, 0, KeyedInputRole::Collateral).outpoint;
    let member = input(&m, 1, KeyedInputRole::Member).outpoint;
    for mode in [0, 1, 2, 3, 4, 5, 6] {
        let offered = match mode {
            4 => vec![sponsor, member],
            5 => vec![sponsor, Outpoint::new(sponsor.txid(), 1).unwrap()],
            _ => vec![sponsor],
        };
        let signer = Signer {
            offer: SponsorOffer::new(offered, 1000, Some(ValueField::Explicit(1_998_000))).unwrap(),
            calls: 0.into(),
            signs: 0.into(),
            mode,
        };
        let mut inputs = vec![
            input(&parent_bytes, 0, KeyedInputRole::Collateral),
            input(&m, 1, KeyedInputRole::Member),
        ];
        inputs[0].capability = Some(&signer);
        let request = keyed_merge_request(&target, [42; 32], &commitment, &inputs).unwrap();
        let result = admit_keyed(
            &target,
            KeyedConstructionRequest::Merge(&request),
            outputs(1, 1000),
            &policy(),
        );
        if mode == 0 {
            assert!(result.is_ok());
        } else {
            assert_eq!(
                result.unwrap_err(),
                match mode {
                    1 => KeyedConstructionError::SignatureMissing,
                    2 => KeyedConstructionError::SignatureBoundExceeded {
                        reached: 73,
                        bound: 72
                    },
                    3 => KeyedConstructionError::SignatureBinding,
                    4 => KeyedConstructionError::CapabilityOverlap,
                    6 => KeyedConstructionError::Sponsor(KeyedSponsorRefusal::InvalidSignature),
                    _ => KeyedConstructionError::CapabilityCoverage,
                }
            );
        }
        assert_eq!(signer.calls.get(), 1);
        assert_eq!(signer.signs.get(), usize::from(!matches!(mode, 4 | 5)));
    }
}

#[test]
fn policy_asset_obligations_and_inclusive_ceilings_are_explicit() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let bytes = parent(&program(), 0);
    let mut offered = input(&bytes, 0, KeyedInputRole::Collateral);
    let signer = Signer {
        offer: SponsorOffer::new(
            [offered.outpoint],
            1000,
            Some(ValueField::Explicit(998_000)),
        )
        .unwrap(),
        calls: 0.into(),
        signs: 0.into(),
        mode: 0,
    };
    offered.capability = Some(&signer);
    let request = keyed_source_request(&target, [42; 32], &commitment, &[offered]).unwrap();
    let req = KeyedConstructionRequest::Source(&request);
    let control = admit_keyed(&target, req, outputs(0, 1000), &policy()).unwrap();
    assert_eq!(
        control.obligations(),
        &[
            Obl::Inclusion,
            Obl::Unspentness,
            Obl::Retrieval,
            Obl::Finality,
            Obl::FinalBytePolicyObservation,
            Obl::MemberExecutionAndRelationEvidence
        ]
    );
    let mut wrong = policy();
    wrong.policy_asset = AssetId::from_internal([4; 32]);
    signer.signs.set(0);
    assert_eq!(
        admit_keyed(&target, req, outputs(0, 1000), &wrong).unwrap_err(),
        KeyedConstructionError::PolicyAssetMismatch
    );
    assert_eq!(signer.signs.get(), 0);
    let weight = control.weight_bound();
    for dimension in 0..3 {
        let mut exact = policy();
        exact.maximum_transaction_weight = weight;
        exact.maximum_deployment_weight = weight;
        exact.maximum_package_weight = weight;
        assert!(admit_keyed(&target, req, outputs(0, 1000), &exact).is_ok());
        let expected = match dimension {
            0 => {
                exact.maximum_transaction_weight -= 1;
                KeyedConstructionError::TransactionWeightExceeded {
                    reached: weight,
                    bound: weight - 1,
                }
            }
            1 => {
                exact.maximum_deployment_weight -= 1;
                KeyedConstructionError::DeploymentWeightExceeded {
                    reached: weight,
                    bound: weight - 1,
                }
            }
            _ => {
                exact.maximum_package_weight -= 1;
                KeyedConstructionError::PackageWeightExceeded {
                    reached: weight,
                    bound: weight - 1,
                }
            }
        };
        signer.signs.set(0);
        assert_eq!(
            admit_keyed(&target, req, outputs(0, 1000), &exact).unwrap_err(),
            expected
        );
        assert_eq!(signer.signs.get(), 0);
    }
}

#[test]
fn weight_cross_check_detects_member_residue_and_policy_maximum_is_tied_to_target() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let bytes = parent(commitment.output_program(), 0);
    let request = keyed_merge_request(
        &target,
        [42; 32],
        &commitment,
        &[input(&bytes, 0, KeyedInputRole::Member)],
    )
    .unwrap();
    let structural = construct_keyed(
        &target,
        KeyedConstructionRequest::Merge(&request),
        outputs(0, 1000),
    )
    .unwrap();
    assert_eq!(
        check_weight_formula(&target, &structural, structural.transaction()),
        Ok(())
    );
    let original = structural.transaction();
    let mut stack = original.witnesses()[0].stack().to_vec();
    stack.push(vec![1]);
    let wrong = TargetTransaction::new(
        original.version(),
        original.inputs().to_vec(),
        original.outputs().to_vec(),
        original.lock_time(),
        vec![InputWitness::new(stack)],
    )
    .unwrap();
    assert_eq!(
        check_weight_formula(&target, &structural, &wrong),
        Err(KeyedConstructionError::WeightFormulaMismatch {
            serialized: wrong.weight(),
            formula: original.weight()
        })
    );
    let maximum = target
        .definition()
        .resources()
        .policy()
        .bounds()
        .get(&target_elements::ResourceDimension::TransactionWeight)
        .unwrap()
        .maximum()
        .unwrap();
    for count in [KEYED_POLICY_HANDLE_MAXIMUM, KEYED_POLICY_HANDLE_MAXIMUM + 1] {
        // Three-byte input count: 242 base bytes + 41 per keyed input; 117 other witness bytes.
        let weight = tapscript::keyed_resources::keyed_transaction_weight(
            &target,
            count as u64,
            242 + 41 * count as u64,
            117,
        )
        .unwrap();
        assert_eq!(weight, 1085 + 255 * count as u64);
        assert_eq!(weight <= maximum, count == KEYED_POLICY_HANDLE_MAXIMUM);
    }
}

#[test]
fn spending_flags_are_excluded_by_types_and_frozen_decoder() {
    for flag in [1_u32 << 30, 1_u32 << 31] {
        assert!(Outpoint::new(Txid::from_internal([1; 32]), flag).is_err());
    }
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let bytes = parent(&program(), 0);
    let request = keyed_source_request(
        &target,
        [42; 32],
        &commitment,
        &[input(&bytes, 0, KeyedInputRole::Collateral)],
    )
    .unwrap();
    let result = construct_keyed(
        &target,
        KeyedConstructionRequest::Source(&request),
        outputs(0, 1000),
    )
    .unwrap();
    let decoded =
        TargetTransaction::decode_core(&result.transaction().encode_without_witness()).unwrap();
    assert!(
        decoded
            .inputs()
            .iter()
            .all(|row| row.outpoint().index() & 0xc000_0000 == 0)
    );
}

#[test]
fn hash160_compressed_generator_matches_the_published_known_answer() {
    // BIP-173's example witness program; expected bytes are not computed by a library.
    let generator = [
        0x02, 0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87,
        0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16,
        0xf8, 0x17, 0x98,
    ];
    assert_eq!(
        hash160::Hash::hash(&generator).to_byte_array(),
        [
            0x75, 0x1e, 0x76, 0xe8, 0x19, 0x91, 0x96, 0xd4, 0x54, 0x94, 0x1c, 0x45, 0xd1, 0xb3,
            0xa3, 0x23, 0xf1, 0x43, 0x3b, 0xd6
        ]
    );
}

struct SnapshotCapability {
    offers: Vec<SponsorOffer>,
    calls: std::cell::Cell<usize>,
}
impl SponsorCapability for SnapshotCapability {
    fn offer(&self) -> SponsorOffer {
        let index = self.calls.get();
        self.calls.set(index + 1);
        self.offers[index.min(self.offers.len() - 1)].clone()
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        None
    }
    fn sign(&self, _: &SponsorSigningRequest) -> Option<SponsorSignature> {
        panic!("structural only")
    }
}
#[test]
fn capability_groups_require_exact_identical_snapshots_and_reconcile_fees_once() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let parents = [parent(&program(), 0), parent(&program(), 1)];
    let points: Vec<_> = parents
        .iter()
        .enumerate()
        .map(|(index, bytes)| {
            input(
                bytes,
                u32::try_from(index).unwrap(),
                KeyedInputRole::Collateral,
            )
            .outpoint
        })
        .collect();
    for mode in 0..4 {
        let complete = SponsorOffer::new(points.iter().copied(), 1000, None).unwrap();
        let offers = if mode == 2 {
            points
                .iter()
                .map(|p| SponsorOffer::new([*p], 500, None).unwrap())
                .collect()
        } else {
            vec![complete.clone(), complete.clone()]
        };
        let first = SnapshotCapability {
            offers,
            calls: 0.into(),
        };
        let second = SnapshotCapability {
            offers: vec![complete],
            calls: 0.into(),
        };
        let inputs: Vec<_> = parents
            .iter()
            .enumerate()
            .map(|(index, bytes)| {
                let mut row = input(
                    bytes,
                    u32::try_from(index).unwrap(),
                    KeyedInputRole::Collateral,
                );
                row.capability = Some(if mode == 1 && index == 1 {
                    &second
                } else {
                    &first
                });
                row
            })
            .collect();
        let request = keyed_source_request(&target, [42; 32], &commitment, &inputs).unwrap();
        let result = construct_keyed(
            &target,
            KeyedConstructionRequest::Source(&request),
            outputs(1, if mode == 3 { 999 } else { 1000 }),
        );
        match mode {
            0 => assert!(result.is_ok()),
            1 => assert_eq!(
                result.unwrap_err(),
                KeyedConstructionError::CapabilityCoverage
            ),
            2 => assert_eq!(
                result.unwrap_err(),
                KeyedConstructionError::CapabilitySnapshot
            ),
            _ => assert_eq!(result.unwrap_err(), KeyedConstructionError::CapabilityFee),
        }
        assert_eq!(first.calls.get() + second.calls.get(), 2);
    }
}

#[test]
fn envelope_causes_are_named_and_never_request_a_signature() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let bytes = parent(&program(), 0);
    for mode in [0, 7, 8, 9, 10] {
        let mut row = input(&bytes, 0, KeyedInputRole::Collateral);
        let signer = Signer {
            offer: SponsorOffer::new(
                [row.outpoint],
                1000,
                Some(ValueField::Explicit(if mode == 9 {
                    997_999
                } else {
                    998_000
                })),
            )
            .unwrap(),
            calls: 0.into(),
            signs: 0.into(),
            mode,
        };
        if mode != 10 {
            row.capability = Some(&signer);
        }
        let request = keyed_source_request(&target, [42; 32], &commitment, &[row]).unwrap();
        let result = admit_keyed(
            &target,
            KeyedConstructionRequest::Source(&request),
            outputs(0, 1000),
            &policy(),
        );
        if mode == 0 {
            assert!(result.is_ok());
            assert_eq!(signer.signs.get(), 1);
        } else {
            assert_eq!(
                result.unwrap_err(),
                match mode {
                    7 => KeyedConstructionError::ChangeDestinationMissing,
                    8 => KeyedConstructionError::ChangeDestinationMismatch,
                    9 => KeyedConstructionError::ChangeOfferMismatch,
                    _ => KeyedConstructionError::SponsorCapabilityMissing,
                }
            );
            assert_eq!(signer.signs.get(), 0);
        }
    }
}

#[test]
fn construction_output_forms_have_adjacent_refusals() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let bytes = parent(&program(), 0);
    let request = keyed_source_request(
        &target,
        [42; 32],
        &commitment,
        &[input(&bytes, 0, KeyedInputRole::Collateral)],
    )
    .unwrap();
    let req = KeyedConstructionRequest::Source(&request);
    assert!(construct_keyed(&target, req, outputs(0, 1000)).is_ok());
    for (mode, expected) in [
        (0, KeyedConstructionError::ExtraFee),
        (1, KeyedConstructionError::ExplicitPolicyAsset),
        (2, KeyedConstructionError::AmountRange),
        (3, KeyedConstructionError::NonPositiveOutput),
        (4, KeyedConstructionError::MissingFee),
    ] {
        let mut subject = outputs(0, 1000);
        match mode {
            0 => subject.additional_outputs.push(output(1, vec![])),
            1 => subject.policy_asset = AssetId::from_internal([9; 32]),
            2 => subject.successor_value = u64::MAX,
            3 => subject.successor_value = 0,
            _ => subject.fee = Some(0),
        }
        assert_eq!(
            construct_keyed(&target, req, subject).unwrap_err(),
            expected
        );
    }
}

#[test]
fn exact_rounded_fee_and_every_output_floor_precede_signing() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let bytes = parent(&program(), 0);
    let mut readings = policy();
    readings.relay_sat_per_kvb = 101;
    // Bound source weight is 1077, hence vsize 270 and ceil(270*101/1000)=28.
    let required = 28;
    for fee in [required, required - 1] {
        let mut row = input(&bytes, 0, KeyedInputRole::Collateral);
        let signer = Signer {
            offer: SponsorOffer::new(
                [row.outpoint],
                fee,
                Some(outputs(0, fee).additional_outputs[0].value()),
            )
            .unwrap(),
            calls: 0.into(),
            signs: 0.into(),
            mode: 0,
        };
        row.capability = Some(&signer);
        let request = keyed_source_request(&target, [42; 32], &commitment, &[row]).unwrap();
        let req = KeyedConstructionRequest::Source(&request);
        let result = admit_keyed(&target, req, outputs(0, fee), &readings);
        if fee == required {
            assert_eq!(result.unwrap().weight_bound(), 1077);
            assert_eq!(signer.signs.get(), 1);
            for index in [1, 2] {
                let mut wrong = readings.clone();
                wrong.output_minimums[index] = if index == 1 {
                    1_000_000 - 1000 - fee + 1
                } else {
                    fee + 1
                };
                signer.signs.set(0);
                assert_eq!(
                    admit_keyed(&target, req, outputs(0, fee), &wrong).unwrap_err(),
                    KeyedConstructionError::OutputPolicy { index }
                );
                assert_eq!(signer.signs.get(), 0);
            }
        } else {
            assert_eq!(
                result.unwrap_err(),
                KeyedConstructionError::FeeShortage {
                    offered: fee,
                    required
                }
            );
            assert_eq!(signer.signs.get(), 0);
        }
    }
}

#[test]
fn admission_uses_the_actual_nonzero_sponsor_index() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let parents = [
        parent(commitment.output_program(), 0),
        parent(&program(), 1),
    ];
    let mut inputs = vec![
        input(&parents[0], 0, KeyedInputRole::Member),
        input(&parents[1], 1, KeyedInputRole::Collateral),
    ];
    let signer = Signer {
        offer: SponsorOffer::new(
            [inputs[1].outpoint],
            1000,
            Some(ValueField::Explicit(1_998_000)),
        )
        .unwrap(),
        calls: 0.into(),
        signs: 0.into(),
        mode: 0,
    };
    inputs[1].capability = Some(&signer);
    let request = keyed_merge_request(&target, [42; 32], &commitment, &inputs).unwrap();
    let result = admit_keyed(
        &target,
        KeyedConstructionRequest::Merge(&request),
        outputs(1, 1000),
        &policy(),
    )
    .unwrap();
    assert_eq!(
        result.transaction().witnesses()[0].stack()[0],
        commitment.leaf()
    );
    assert_eq!(result.transaction().witnesses()[1].stack()[1], public());
    assert_eq!(signer.signs.get(), 1);
}

#[test]
fn keyed_construction_refusal_tables_are_snake_case() {
    for source in [
        include_str!("../keyed_construct.rs"),
        include_str!("../keyed_sponsor.rs"),
    ] {
        assert!(!crate::tests::trace_tests::trace_kind_table(source).is_empty());
    }
}

#[test]
fn structural_weight_keeps_the_existing_consensus_refusal() {
    let target = reviewed_elements_tapscript().unwrap();
    let commitment = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let bytes = parent(&program(), 0);
    let request = keyed_source_request(
        &target,
        [42; 32],
        &commitment,
        &[input(&bytes, 0, KeyedInputRole::Collateral)],
    )
    .unwrap();
    let req = KeyedConstructionRequest::Source(&request);
    assert!(construct_keyed(&target, req, outputs(0, 1000)).is_ok());
    let mut oversized = outputs(0, 1000);
    oversized.additional_outputs[0] = output(998_000, vec![0; 1_000_001]);
    assert!(matches!(construct_keyed(&target,req,oversized),
        Err(KeyedConstructionError::Target(TransactionRefusal::ResourceBoundExceeded {
            dimension: target_elements::ResourceDimension::TransactionWeight, reached, bound
        })) if reached > bound));
}

#[test]
fn checked_policy_constructor_refuses_incomplete_readings() {
    let make = |name: &str, floors, ceilings, signature| {
        KeyedPolicy::new(
            name.into(),
            AssetId::from_internal([3; 32]),
            [100; 3],
            floors,
            ceilings,
            0,
            signature,
        )
    };
    assert!(make("reading", vec![100, 100, 1], [400_000; 3], 72).is_ok());
    assert_eq!(
        make("", vec![100, 100, 1], [400_000; 3], 72).unwrap_err(),
        KeyedConstructionError::ResourceContext
    );
    assert_eq!(
        make("reading", vec![], [400_000; 3], 72).unwrap_err(),
        KeyedConstructionError::ResourceContext
    );
    for index in 0..3 {
        let mut ceilings = [400_000; 3];
        ceilings[index] = 0;
        assert_eq!(
            make("reading", vec![100, 100, 1], ceilings, 72).unwrap_err(),
            KeyedConstructionError::ResourceContext
        );
    }
    for signature in [0, 73] {
        assert_eq!(
            make("reading", vec![100, 100, 1], [400_000; 3], signature).unwrap_err(),
            KeyedConstructionError::SignatureBoundOutOfRange
        );
    }
}
