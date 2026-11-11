//! The default library consumes sealed plans without creator secrets or a planner.
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{KeyedPlan, PlacementSearchLimits, plan},
};
use std::collections::BTreeMap;
use tapscript::keyed_bundle::*;

fn fixture(scope: Vec<K>) -> KeyedPlan {
    let manifest = KeyedManifest::contract().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("derive")
        .validate(&manifest)
        .expect("validate");
    let states = std::num::NonZeroU64::new(1_000_000).expect("positive");
    let candidates = std::num::NonZeroU64::new(100_000).expect("positive");
    let input = bind_input(
        &manifest,
        source,
        scope.into_iter().map(O::Keyed).collect(),
        AnalysisPolicy::strict(ProofSearchLimits::new(states, candidates)),
    )
    .expect("bind");
    plan(&input, PlacementSearchLimits::new(states, candidates)).expect("plan")
}

fn selection(plan: &KeyedPlan) -> KeyedBundleSelection {
    KeyedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn default_consumer_retains_three_scopes_and_exact_fragment_bytes() {
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    for (scope, relations, evidence, clauses) in [
        (vec![K::Source], 4, 4, 0),
        (vec![K::Merge], 5, 3, 2),
        (vec![K::Source, K::Merge], 9, 7, 2),
    ] {
        let plan = fixture(scope);
        let request = selection(&plan);
        let bundle = emit_keyed_bundle(
            &target,
            &plan,
            request.clone(),
            KeyedLeafRecipe::new([42; 32]),
        )
        .expect("bundle");
        let view = bundle.project();
        assert_eq!(view.selection, request);
        assert_eq!(view.selected, plan.project().plans[0]);
        assert_eq!(
            view.selected
                .operations
                .values()
                .map(|o| o.relations.len())
                .sum::<usize>(),
            relations
        );
        assert_eq!(view.evidence.len(), evidence);
        assert_eq!(view.evidence, plan.project().evidence);
        assert_eq!(view.fragments.len(), clauses);
        let mut expected = vec![32];
        expected.extend([42; 32]);
        expected.extend([
            0x75, 0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0xcd, 0xca, 0, 0xd1, 0x7b, 0x88,
            0x88, 0x51,
        ]);
        assert_eq!(view.successor.program().encode(&target), expected);
        for (id, fragment) in &view.fragments {
            use realization::relation::KeyedRelationKind;
            let bytes: &[u8] = match id.kind {
                KeyedRelationKind::BornFirst => &[0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75],
                KeyedRelationKind::ContinueFirst => &[0xcd, 0xca, 0, 0xd1, 0x7b, 0x88, 0x88],
                _ => panic!("evidence is not runtime"),
            };
            assert_eq!(fragment.encode(&target), bytes);
        }
    }
}

#[test]
fn default_consumer_refuses_changed_binding_scope_and_candidate_selection() {
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let plan = fixture(vec![K::Source, K::Merge]);
    let mut control = selection(&plan);
    control.proof = plan.project().plans.len() - 1;
    let merge = O::Keyed(K::Merge);
    let placement_len = plan.project().plans[control.proof].operations[&merge]
        .placements
        .len();
    control.placements.insert(merge, placement_len - 1);
    let recipe = KeyedLeafRecipe::new([0; 32]);
    let check = |subject, refusal| {
        assert!(emit_keyed_bundle(&target, &plan, control.clone(), recipe).is_ok());
        assert_eq!(
            emit_keyed_bundle(&target, &plan, subject, recipe),
            Err(refusal)
        );
    };
    let mut subject = control.clone();
    subject.binding.digest[0] ^= 1;
    check(subject, KeyedBundleRefusal::BindingMismatch);
    let mut subject = control.clone();
    subject.scope.pop();
    check(subject, KeyedBundleRefusal::ScopeMismatch);
    let mut subject = control.clone();
    subject.proof = plan.project().plans.len();
    check(
        subject,
        KeyedBundleRefusal::ProofSelection {
            index: plan.project().plans.len(),
        },
    );
    let mut subject = control.clone();
    subject.placements.remove(&merge);
    check(subject, KeyedBundleRefusal::PlacementScopeMismatch);
    let mut subject = control.clone();
    subject.placements.insert(merge, placement_len);
    check(
        subject,
        KeyedBundleRefusal::PlacementSelection {
            operation: O::Keyed(K::Merge),
            index: placement_len,
        },
    );
    let source = fixture(vec![K::Source]);
    let control = selection(&source);
    assert!(emit_keyed_bundle(&target, &source, control.clone(), recipe).is_ok());
    let mut subject = control;
    subject.placements.insert(merge, 0);
    assert_eq!(
        emit_keyed_bundle(&target, &source, subject, recipe),
        Err(KeyedBundleRefusal::PlacementScopeMismatch)
    );
}

#[test]
fn default_bundle_surface_is_immutable_and_contains_no_engine_record() {
    let source = include_str!("../src/keyed_bundle.rs");
    assert!(sealed_surface(source));
    for addition in [
        "pub fn reselect(bundle: &mut KeyedEmissionBundle) {}",
        "pub fn forge(p: KeyedBundleProjection) -> KeyedEmissionBundle {}",
        "impl KeyedEmissionBundle { pub fn new(p: KeyedBundleProjection) -> Self {} }",
        "impl From<KeyedBundleProjection> for KeyedEmissionBundle {}",
        "impl TryFrom<KeyedBundleProjection> for KeyedEmissionBundle {}",
        "impl DerefMut for KeyedEmissionBundle {}",
        "impl AsMut<KeyedBundleProjection> for KeyedEmissionBundle {}",
        "impl BorrowMut<KeyedBundleProjection> for KeyedEmissionBundle {}",
    ] {
        assert!(
            !sealed_surface(&format!("{source}\n{addition}")),
            "{addition}"
        );
    }
    let default = source.replace(
        "pub struct KeyedEmissionBundle",
        "#[derive(Default)]\npub struct KeyedEmissionBundle",
    );
    assert!(!sealed_surface(&default));
    let field = source.replace(
        "    projection: KeyedBundleProjection,",
        "    projection: KeyedBundleProjection,\n    pub extra: bool,",
    );
    assert!(!sealed_surface(&field));
    assert!(sealed_surface(&format!(
        "{source}\nfn internal() {{ map.get_mut(key); }}"
    )));
}

// Deliberately textual: no macro expansion or type-alias resolution is claimed.
fn sealed_surface(source: &str) -> bool {
    let text = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let compact = text.split_whitespace().collect::<String>();
    if compact.contains("engine_") || compact.contains("Default") {
        return false;
    }
    let Some(body) = compact
        .split("pubstructKeyedEmissionBundle{")
        .nth(1)
        .and_then(|s| s.split('}').next())
    else {
        return false;
    };
    if body.contains("pub") {
        return false;
    }
    for implementation in compact.split("impl").skip(1) {
        let header = implementation.split('{').next().unwrap_or(implementation);
        if header.contains("forKeyedEmissionBundle")
            && ["From<", "TryFrom<", "DerefMut", "AsMut<", "BorrowMut<"]
                .iter()
                .any(|t| header.contains(t))
        {
            return false;
        }
    }
    let mut bundle_impl = false;
    let mut depth = 0_i32;
    for section in text.split_inclusive(['{', '}', ';']) {
        let head = section.trim();
        if head.starts_with("impl KeyedEmissionBundle") {
            bundle_impl = true;
            depth = 0;
        }
        // Each section ends at a body delimiter; its public signature may span lines.
        if let Some(start) = head.find("pub ") {
            let signature = &head[start..];
            if signature.contains("fn ") {
                let flat = signature.split_whitespace().collect::<String>();
                let allowed = flat.starts_with("pubfnemit_keyed_bundle(");
                let returned = flat.split("->").nth(1).unwrap_or("");
                let constructs = returned.contains("KeyedEmissionBundle")
                    || (bundle_impl && returned.contains("Self"));
                if flat.contains("&mut") || flat.contains("_mut(") || (!allowed && constructs) {
                    return false;
                }
            }
        }
        if section.ends_with('{') {
            depth += 1;
        }
        if section.ends_with('}') {
            depth -= 1;
        }
        if depth == 0 {
            bundle_impl = false;
        }
    }
    true
}

#[test]
fn default_recipe_retains_complete_sole_leaf_construction_material() {
    use tapscript::keyed_constructor::{KeyedInternalKeyDerivation, KeyedSemanticKey};
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    for key in [[0; 32], [42; 32], [43; 32]] {
        let recipe = KeyedLeafRecipe::new(key);
        let material = recipe.construct(&target).expect("material");
        assert_eq!(recipe.schema(), 1);
        assert_eq!(recipe.key(), KeyedSemanticKey::new(key).bytes());
        assert_eq!(recipe.semantic_key(), KeyedSemanticKey::new(key));
        assert_eq!(recipe, KeyedLeafRecipe::from(KeyedSemanticKey::new(key)));
        assert_eq!(
            recipe.internal_key_derivation(),
            KeyedInternalKeyDerivation::Sha256UncompressedGenerator
        );
        assert_eq!(material.recipe(), &recipe);
        assert_eq!(material.leaf_version(), target.definition().leaf_version());
        assert_eq!(material.siblings(), [[0_u8; 32]; 0]);
        assert_eq!(
            material,
            KeyedLeafRecipe::new(key)
                .construct(&target)
                .expect("fresh material")
        );
        let source = fixture(vec![K::Source]);
        let bundle =
            emit_keyed_bundle(&target, &source, selection(&source), recipe).expect("SOURCE");
        assert_eq!(bundle.project().successor, material);
        assert!(bundle.project().fragments.is_empty());
        assert_eq!(bundle.project().evidence.len(), 4);
    }
    assert!(
        !include_str!("../Cargo.toml")
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .any(|line| line.trim_start().starts_with("keyed-constructor ="))
    );
    assert!(!include_str!("../src/lib.rs").contains("keyed-constructor"));
}
