//! Public structural requests: canonical bytes and offers, never authorization.
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{PlacementSearchLimits, plan},
};
use linker::{
    keyed_bundle::{LinkedKeyedBundle, link_keyed_bundle},
    keyed_reference::keyed_parameter_offers,
};
use tapscript::{
    keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle},
    keyed_constructor::KeyedLeafRecipe,
};
use target_elements::reviewed_elements_tapscript;
use target_elements_conformance::constructor::tagged::sha256;
use transaction::keyed_inputs::{KeyedInputOffer, KeyedInputRefusal, KeyedInputRole};
use transaction::keyed_request::{KeyedRequestRefusal, keyed_merge_request, keyed_source_request};
use transaction::keyed_taproot::commit_keyed_bundle;
use transaction::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};
fn linked(key: [u8; 32], source: bool) -> LinkedKeyedBundle {
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
        if source {
            vec![O::Keyed(K::Source)]
        } else {
            vec![O::Keyed(K::Merge)]
        },
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

fn parent(program: &[u8], salt: u8) -> Vec<u8> {
    let output = TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal([3; 32])),
        ValueField::Explicit(1000),
        NonceField::Null,
        program.to_vec(),
    );
    TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([salt; 32]), 0).unwrap(),
            123,
        )],
        vec![output.clone(), output],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode()
}
fn offer<'capability>(
    parent: &[u8],
    input_index: u32,
    role: KeyedInputRole,
) -> KeyedInputOffer<'_, 'capability> {
    let tx = TargetTransaction::decode(parent).unwrap();
    let txid = Txid::from_internal(sha256(&sha256(&tx.encode_without_witness())));
    KeyedInputOffer {
        input_index,
        parent,
        outpoint: Outpoint::new(txid, 0).unwrap(),
        sequence: 123,
        flags: 0,
        role,
        capability: None,
    }
}
struct OfferedCapability(transaction::SponsorOffer);
impl transaction::SponsorCapability for OfferedCapability {
    fn offer(&self) -> transaction::SponsorOffer {
        self.0.clone()
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        None
    }
    fn sign(
        &self,
        _: &transaction::SponsorSigningRequest,
    ) -> Option<transaction::SponsorSignature> {
        None
    }
}
#[test]
fn public_keyed_requests_bind_parent_facts_without_claiming_collateral_authority() {
    let target = reviewed_elements_tapscript().unwrap();
    let source = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let merge = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let collateral = parent(&[0x51], 1); // deliberately not the measured P2WPKH envelope
    let a = parent(merge.output_program(), 2);
    let b = parent(merge.output_program(), 3);
    let outpoint = offer(&collateral, 0, KeyedInputRole::Collateral).outpoint;
    let capability =
        OfferedCapability(transaction::SponsorOffer::new([outpoint], 0, None).unwrap());
    let wrong = OfferedCapability(
        transaction::SponsorOffer::new([Outpoint::new(outpoint.txid(), 1).unwrap()], 0, None)
            .unwrap(),
    );
    let mut inputs = vec![
        offer(&collateral, 0, KeyedInputRole::Collateral),
        offer(&a, 1, KeyedInputRole::Member),
        offer(&b, 2, KeyedInputRole::Member),
    ];
    inputs[0].capability = Some(&capability);
    let request = keyed_source_request(&target, [42; 32], &source, &inputs[..1]).unwrap();
    assert_eq!(request.inputs()[0].role(), KeyedInputRole::Collateral);
    assert_eq!(
        request.inputs()[0].parent().output(),
        &TargetTransaction::decode(&collateral).unwrap().outputs()[0]
    );
    assert!(request.inputs()[0].capability().is_some());
    let merged = keyed_merge_request(&target, [42; 32], &merge, &inputs).unwrap();
    assert_eq!(merged.members().coverage().len(), 2);
    assert!(
        merged.members().coverage().values().all(|members| members
            .iter()
            .map(|m| m.input_index)
            .collect::<Vec<_>>()
            == [1, 2])
    );
    inputs[0].capability = Some(&wrong);
    assert_eq!(
        keyed_source_request(&target, [42; 32], &source, &inputs[..1]).unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 0,
            refusal: KeyedInputRefusal::CapabilityDoesNotNameOutpoint(outpoint)
        }
    );
    inputs[0].capability = None;
    assert!(keyed_source_request(&target, [42; 32], &source, &inputs[..1]).is_ok());
}

#[test]
fn public_parent_binding_uses_stripped_txid_and_retains_witness_bytes() {
    let target = reviewed_elements_tapscript().unwrap();
    let c = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let plain = parent(c.output_program(), 1);
    let decoded = TargetTransaction::decode(&plain).unwrap();
    let witnessed = TargetTransaction::new(
        decoded.version(),
        decoded.inputs().to_vec(),
        decoded.outputs().to_vec(),
        decoded.lock_time(),
        vec![InputWitness::new(vec![vec![1, 2]])],
    )
    .unwrap()
    .encode();
    let accepted = offer(&witnessed, 0, KeyedInputRole::Member);
    assert_eq!(
        accepted.outpoint,
        offer(&plain, 0, KeyedInputRole::Member).outpoint
    );
    let result = keyed_merge_request(&target, [42; 32], &c, &[accepted]).unwrap();
    assert_eq!(result.inputs()[0].parent().parent_bytes(), witnessed);
    let mut subject = offer(&witnessed, 0, KeyedInputRole::Member);
    let computed = subject.outpoint.txid();
    let offered = Txid::from_internal(sha256(&sha256(&witnessed)));
    assert_ne!(computed, offered);
    subject.outpoint = Outpoint::new(offered, 0).unwrap();
    assert_eq!(
        keyed_merge_request(&target, [42; 32], &c, &[subject]).unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 0,
            refusal: KeyedInputRefusal::ParentHash { offered, computed }
        }
    );
    let mut trailing = witnessed.clone();
    trailing.push(0);
    let mut subject = offer(&witnessed, 0, KeyedInputRole::Member);
    subject.parent = &trailing;
    assert!(matches!(
        keyed_merge_request(&target, [42; 32], &c, &[subject]),
        Err(KeyedRequestRefusal::Input {
            refusal: KeyedInputRefusal::Decode(
                transaction::TransactionRefusal::TrailingTargetBytes { .. }
            ),
            ..
        })
    ));
}

struct ChangingOffer {
    calls: std::cell::Cell<usize>,
    first: transaction::SponsorOffer,
    later: transaction::SponsorOffer,
}
impl transaction::SponsorCapability for ChangingOffer {
    fn offer(&self) -> transaction::SponsorOffer {
        let n = self.calls.get();
        self.calls.set(n + 1);
        if n == 0 {
            self.first.clone()
        } else {
            self.later.clone()
        }
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        None
    }
    fn sign(
        &self,
        _: &transaction::SponsorSigningRequest,
    ) -> Option<transaction::SponsorSignature> {
        None
    }
}

#[test]
fn requests_outlive_parent_buffers_and_keep_the_checked_offer_snapshot() {
    let target = reviewed_elements_tapscript().unwrap();
    let c = commit_keyed_bundle(&target, &linked([42; 32], true)).unwrap();
    let outpoint = offer(&parent(&[0x51], 1), 0, KeyedInputRole::Collateral).outpoint;
    let first =
        transaction::SponsorOffer::new([outpoint], 7, Some(ValueField::Explicit(3))).unwrap();
    let capability = ChangingOffer {
        calls: std::cell::Cell::new(0),
        first: first.clone(),
        later: transaction::SponsorOffer::new([], 99, None).unwrap(),
    };
    let request = {
        let bytes = parent(&[0x51], 1);
        let mut input = offer(&bytes, 0, KeyedInputRole::Collateral);
        input.capability = Some(&capability);
        keyed_source_request(&target, [42; 32], &c, &[input]).unwrap()
    }; // Parent buffer and offer are gone; only commitment/capability remain borrowed.
    assert_eq!(request.inputs()[0].role(), KeyedInputRole::Collateral);
    assert_eq!(request.inputs()[0].capability_offer(), Some(&first));
    assert_eq!(capability.calls.get(), 1);
    let later = transaction::SponsorCapability::offer(&capability);
    assert_ne!(later, first);
    assert_eq!(request.inputs()[0].capability_offer(), Some(&first));
    assert_eq!(capability.calls.get(), 2);
    assert_eq!(request.inputs()[0].parent().outpoint(), outpoint);
}

#[test]
fn members_refuse_capabilities_and_foreign_or_unspendable_collateral_is_not_authorized() {
    let target = reviewed_elements_tapscript().unwrap();
    let c = commit_keyed_bundle(&target, &linked([42; 32], false)).unwrap();
    let other = commit_keyed_bundle(&target, &linked([43; 32], false)).unwrap();
    let bytes = parent(c.output_program(), 1);
    let outpoint = offer(&bytes, 0, KeyedInputRole::Member).outpoint;
    let capability = ChangingOffer {
        calls: std::cell::Cell::new(0),
        first: transaction::SponsorOffer::new([outpoint], 0, None).unwrap(),
        later: transaction::SponsorOffer::new([], 0, None).unwrap(),
    };
    assert!(
        keyed_merge_request(
            &target,
            [42; 32],
            &c,
            &[offer(&bytes, 0, KeyedInputRole::Member)]
        )
        .is_ok()
    );
    let mut offered = offer(&bytes, 0, KeyedInputRole::Member);
    offered.capability = Some(&capability);
    assert_eq!(
        keyed_merge_request(&target, [42; 32], &c, &[offered]).unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 0,
            refusal: KeyedInputRefusal::MemberCapability
        }
    );
    assert_eq!(capability.calls.get(), 0);
    for program in [other.output_program(), &[][..], &[0x6a][..]] {
        let collateral = parent(program, 2);
        let accepted = keyed_merge_request(
            &target,
            [42; 32],
            &c,
            &[
                offer(&bytes, 0, KeyedInputRole::Member),
                offer(&collateral, 1, KeyedInputRole::Collateral),
            ],
        )
        .unwrap();
        assert_eq!(accepted.inputs()[1].role(), KeyedInputRole::Collateral);
        assert!(accepted.inputs()[1].predecessor().is_none());
        assert_eq!(
            accepted.inputs()[1].parent().output().is_fee(),
            program.is_empty()
        );
        assert_eq!(
            keyed_merge_request(
                &target,
                [42; 32],
                &c,
                &[
                    offer(&bytes, 0, KeyedInputRole::Member),
                    offer(&collateral, 1, KeyedInputRole::Member)
                ]
            )
            .unwrap_err(),
            KeyedRequestRefusal::Input {
                index: 1,
                refusal: KeyedInputRefusal::Authentication(
                    transaction::keyed_taproot::KeyedAuthenticationRefusal::WrongProgram
                )
            }
        );
    }
}
