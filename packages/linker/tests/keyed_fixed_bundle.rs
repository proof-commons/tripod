//! Public fixed-form consumers, with record literals independent of emission.
use architecture::keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId as K};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_fixed_plan::{KeyedFixedPlan, PlacementSearchLimits, plan},
};
use linker::keyed_fixed_reference::{KeyedFixedLiteralKind as L, KeyedFixedReference as R};
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use tapscript::keyed_fixed_bundle::*;

fn fixture(scope: Vec<K>) -> KeyedFixedPlan {
    let manifest = KeyedFixedManifest::contract().unwrap();
    let source = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
        .unwrap()
        .validate(&manifest)
        .unwrap();
    let states = std::num::NonZeroU64::new(1_000_000).unwrap();
    let candidates = std::num::NonZeroU64::new(100_000).unwrap();
    let input = bind_input(
        &manifest,
        source,
        scope,
        AnalysisPolicy::strict(ProofSearchLimits::new(states, candidates)),
    )
    .unwrap();
    plan(&input, PlacementSearchLimits::new(states, candidates)).unwrap()
}

fn selection(plan: &KeyedFixedPlan) -> KeyedFixedBundleSelection {
    KeyedFixedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    }
}

fn recipe() -> KeyedFixedLeafRecipe {
    KeyedFixedLeafRecipe::new(KeyedFixedParameters {
        key: [0; 32],
        asset: [0x23; 32],
        value: KeyedFixedValue::new(0x0102_0304_0506_0708).unwrap(),
    })
}

#[test]
#[allow(clippy::too_many_lines)] // Keep the literal occurrence census beside its refusal controls.
fn exact_occurrences_and_adjacent_offer_refusals() {
    use linker::keyed_fixed_bundle::*;
    use linker::keyed_fixed_reference::*;
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let plan = fixture(vec![K::Source, K::Merge]);
    let emitted = emit_keyed_fixed_bundle(&target, &plan, selection(&plan), recipe()).unwrap();
    let offers = keyed_fixed_parameter_offers(&target, recipe());
    let linked = link_keyed_fixed_bundle(&target, &emitted, &offers).unwrap();
    assert_eq!(linked.bytes().len(), 104);
    // SHA-256 of the record's 42-byte transcript, reconstructed with Python
    // hashlib independently of the recipe and linker. Remaining bytes are the
    // record's literal instruction schedule.
    let pair: [u8; 32] = [
        86, 10, 92, 40, 128, 20, 148, 195, 143, 120, 96, 34, 142, 168, 252, 201, 65, 8, 251, 109,
        24, 190, 94, 166, 203, 98, 108, 123, 16, 23, 107, 110,
    ];
    let expected = [
        vec![32],
        vec![0; 32],
        vec![0x75, 32],
        pair.to_vec(),
        vec![
            0x76, 0, 0xce, 0x7e, 0xc4, 0, 0xcf, 0x7e, 0xc6, 0x88, 0xcd, 0xc8, 0x7e, 0xc4, 0xcd,
            0xc9, 0x7e, 0xc6, 0x88, 0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0xcd, 0xca, 0,
            0xd1, 0x7b, 0x88, 0x88, 0x51,
        ],
    ]
    .concat();
    assert_eq!(linked.bytes(), expected);
    assert_eq!(linked.closure().constructors().len(), 2);
    let literal_bytes = vec![
        vec![0; 32],
        pair.to_vec(),
        vec![],
        vec![],
        vec![0; 4],
        vec![],
        vec![1],
    ];
    assert_eq!(
        linked
            .closure()
            .literals()
            .iter()
            .map(|row| row.bytes.clone())
            .collect::<Vec<_>>(),
        literal_bytes
    );
    assert_eq!(
        linked
            .closure()
            .literals()
            .iter()
            .map(|row| (row.instruction, row.kind))
            .collect::<Vec<_>>(),
        vec![
            (0, L::SemanticKey),
            (2, L::PairDigest),
            (4, L::SuccessorAssetIndex),
            (8, L::SuccessorValueIndex),
            (25, L::BornOutputIndex),
            (30, L::SuccessorOutputIndex),
            (35, L::FinalTruth),
        ]
    );
    assert_eq!(
        linked
            .closure()
            .references()
            .iter()
            .map(|row| (row.instruction, row.reference))
            .collect::<Vec<_>>(),
        vec![
            (5, R::SuccessorAsset),
            (9, R::SuccessorValue),
            (13, R::OwnAssetCurrentInput),
            (14, R::OwnAsset),
            (17, R::OwnValueCurrentInput),
            (18, R::OwnValue),
            (22, R::BirthCurrentInput),
            (23, R::ConsumedOutpoint),
            (28, R::ContinuationCurrentInput),
            (29, R::CurrentProgram),
            (31, R::SuccessorProgram),
        ]
    );
    for index in 0..offers.len() {
        let mut subject = offers.clone();
        let removed = subject.remove(index);
        let KeyedFixedParameter::Constructor(field) = removed.parameter else {
            panic!("constructor");
        };
        assert_eq!(
            link_keyed_fixed_bundle(&target, &emitted, &subject),
            Err(KeyedFixedLinkRefusal::MissingConstructor(field))
        );
        let mut subject = offers.clone();
        subject.push(removed);
        assert_eq!(
            link_keyed_fixed_bundle(&target, &emitted, &subject),
            Err(KeyedFixedLinkRefusal::DuplicateConstructor(field))
        );
    }
    for coordinate in 0..3 {
        let mut q = recipe().parameters();
        match coordinate {
            0 => q.key[0] ^= 1,
            1 => q.asset[0] ^= 1,
            _ => q.value = KeyedFixedValue::new(q.value.get() + 1).unwrap(),
        }
        let subject = keyed_fixed_parameter_offers(&target, KeyedFixedLeafRecipe::new(q));
        assert_eq!(
            link_keyed_fixed_bundle(&target, &emitted, &subject),
            Err(KeyedFixedLinkRefusal::RecipeMismatch)
        );
    }
    let mut subject = offers.clone();
    subject.push(KeyedFixedParameterOffer {
        parameter: KeyedFixedParameter::Introspection(R::OwnAsset),
        value: KeyedFixedParameterValue::Recipe(recipe()),
    });
    assert_eq!(
        link_keyed_fixed_bundle(&target, &emitted, &subject),
        Err(KeyedFixedLinkRefusal::IntrospectionSubstitution(
            R::OwnAsset
        ))
    );
    let mut subject = offers;
    subject[1].value = KeyedFixedParameterValue::TargetVersion(99);
    assert_eq!(
        link_keyed_fixed_bundle(&target, &emitted, &subject),
        Err(KeyedFixedLinkRefusal::TargetMismatch)
    );
}
