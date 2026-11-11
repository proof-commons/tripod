//! Offline production reconstruction against the independent first-party arithmetic.
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
    keyed_constructor::{KeyedLeafRecipe, KeyedSemanticKey},
};
use target_elements::reviewed_elements_tapscript;
use transaction::keyed_taproot::{
    KeyedAuthenticationRefusal, KeyedSpendPath, authenticate_keyed_path,
    authenticate_keyed_predecessor, commit_keyed_bundle,
};
use vectors::keyed_constructor::{PublicKeyedCurve, construct_keyed, reconstruct_keyed};
fn linked(key: [u8; 32]) -> LinkedKeyedBundle {
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
        vec![O::Keyed(K::Source), O::Keyed(K::Merge)],
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

#[test]
fn production_keyed_commitments_match_fresh_independent_descriptors() {
    let target = reviewed_elements_tapscript().expect("target");
    let mut programs = Vec::new();
    for key in [[0x31; 32], [0x32; 32]] {
        let commitment = commit_keyed_bundle(&target, &linked(key)).expect("production");
        let oracle = construct_keyed(&target, KeyedSemanticKey::new(key), &PublicKeyedCurve)
            .expect("independent oracle");
        assert_eq!(
            reconstruct_keyed(&target, &oracle, &PublicKeyedCurve),
            Ok(oracle.clone())
        );
        assert_eq!(
            commit_keyed_bundle(&target, &linked(key)),
            Ok(commitment.clone())
        );
        assert_eq!(commitment.recipe().key(), &oracle.semantic_key);
        assert_eq!(commitment.recipe().schema(), oracle.schema);
        assert_eq!(commitment.leaf(), oracle.leaf_bytes);
        assert_eq!(commitment.leaf_version(), oracle.leaf_version);
        assert_eq!(commitment.internal_key(), &oracle.internal_key);
        assert_eq!(commitment.root(), &oracle.leaf_hash);
        assert_eq!(commitment.root(), &oracle.merkle_root);
        assert_eq!(
            commitment.tweak(),
            &target_elements_conformance::constructor::tweak(
                &oracle.internal_key,
                &oracle.merkle_root
            )
        );
        assert_eq!(commitment.output_key(), &oracle.output_key);
        assert_eq!(commitment.output_program(), oracle.output_program);
        assert_eq!(commitment.control().as_slice(), oracle.control.bytes);
        assert_eq!(commitment.parity(), oracle.control.parity);
        assert_eq!(
            commitment.paths(),
            [
                KeyedSpendPath::ScriptWithBothClauses,
                KeyedSpendPath::ConditionalNumsKeyPath
            ]
        );
        assert!(authenticate_keyed_predecessor(&commitment, &oracle.output_program).is_ok());
        assert_eq!(
            authenticate_keyed_path(&commitment, &oracle.leaf_bytes, &oracle.control.bytes),
            Ok(())
        );
        programs.push(commitment);
    }
    assert_ne!(programs[0].output_program(), programs[1].output_program());
    for (own, other) in [(&programs[0], &programs[1]), (&programs[1], &programs[0])] {
        assert!(authenticate_keyed_predecessor(own, own.output_program()).is_ok());
        assert_eq!(
            authenticate_keyed_predecessor(own, other.output_program()),
            Err(KeyedAuthenticationRefusal::WrongProgram)
        );
    }
}

#[test]
fn production_lift_refuses_an_in_field_non_point_against_the_independent_oracle() {
    use tapscript::keyed_constructor::KeyedCurveCapability;
    use target_elements_conformance::constructor::curve::{generator, lift_x};
    use transaction::keyed_curve::KeyedCurve;
    let control = generator().x_only_bytes();
    assert!(PublicKeyedCurve.internal_key_is_a_point(&control));
    assert!(KeyedCurve.internal_key_is_a_point(&control));
    // Every candidate is below 256, hence below the field modulus. The
    // independent first-party lift, not the production parser, selects the subject.
    let subject = (0_u8..=255)
        .map(|last| {
            let mut x = [0; 32];
            x[31] = last;
            x
        })
        .find(|x| lift_x(x).is_err())
        .expect("small field non-point");
    assert!(!PublicKeyedCurve.internal_key_is_a_point(&subject));
    assert!(!KeyedCurve.internal_key_is_a_point(&subject));
    assert!(KeyedCurve.internal_key_is_a_point(&control));
    assert_eq!(
        KeyedCurve.output_key(&subject, &[0; 32]),
        tapscript::keyed_constructor::KeyedTweakOutcome::InternalKeyNotAPoint
    );
}
