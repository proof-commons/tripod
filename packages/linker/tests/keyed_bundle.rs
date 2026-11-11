use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{PlacementSearchLimits, plan},
};
use linker::{keyed_bundle::*, keyed_reference::*};
use tapscript::keyed_bundle::{KeyedBundleSelection, KeyedEmissionBundle, emit_keyed_bundle};
use tapscript::keyed_constructor::KeyedLeafRecipe;
use target_elements::reviewed_elements_tapscript;

fn fixture(scope: Vec<K>, key: [u8; 32]) -> KeyedEmissionBundle {
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
    let plan = plan(&input, PlacementSearchLimits::new(states, candidates)).expect("plan");
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
    emit_keyed_bundle(
        &reviewed_elements_tapscript().expect("target"),
        &plan,
        selection,
        KeyedLeafRecipe::new(key),
    )
    .expect("emit")
}

#[test]
fn public_keyed_link_preserves_two_keys_three_scopes_and_offer_permutations() {
    let target = reviewed_elements_tapscript().expect("target");
    for (scope, relations, evidence, clauses) in [
        (vec![K::Source], 4, 4, 0),
        (vec![K::Merge], 5, 3, 2),
        (vec![K::Source, K::Merge], 9, 7, 2),
    ] {
        let mut keys = Vec::new();
        for key in [[42; 32], [43; 32]] {
            let emission = fixture(scope.clone(), key);
            let offers = keyed_parameter_offers(&target, KeyedLeafRecipe::new(key));
            let linked = link_keyed_bundle(&target, &emission, &offers).expect("link");
            for shift in 0..offers.len() {
                let mut permuted = offers.clone();
                permuted.rotate_left(shift);
                permuted.reverse();
                assert_eq!(
                    link_keyed_bundle(&target, &emission, &permuted),
                    Ok(linked.clone())
                );
            }
            assert_eq!(linked.project(), emission.project());
            assert_eq!(linked.emission(), &emission);
            assert_eq!(
                linked
                    .project()
                    .selected
                    .operations
                    .values()
                    .map(|operation| operation.relations.len())
                    .sum::<usize>(),
                relations
            );
            assert_eq!(linked.project().evidence.len(), evidence);
            assert_eq!(linked.project().fragments.len(), clauses);
            assert_eq!(linked.closure().constructors(), offers);
            assert_occurrences(&linked, key);
            let mut expected = vec![32];
            expected.extend(key);
            expected.extend([
                0x75, 0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0xcd, 0xca, 0, 0xd1, 0x7b, 0x88,
                0x88, 0x51,
            ]);
            assert_eq!(linked.bytes(), expected);
            keys.push(linked.bytes().to_vec());
            let mut bad = offers;
            bad[0].value = KeyedParameterValue::Recipe(KeyedLeafRecipe::new([99; 32]));
            assert_eq!(
                link_keyed_bundle(&target, &emission, &bad),
                Err(KeyedLinkRefusal::RecipeMismatch)
            );
        }
        assert_ne!(keys[0], keys[1]);
    }
}

fn assert_occurrences(linked: &LinkedKeyedBundle, key: [u8; 32]) {
    assert_eq!(
        linked.closure().literals(),
        [
            KeyedLiteralOccurrence {
                instruction: 0,
                kind: KeyedLiteralKind::SemanticKey,
                bytes: key.to_vec()
            },
            KeyedLiteralOccurrence {
                instruction: 5,
                kind: KeyedLiteralKind::BornOutputIndex,
                bytes: vec![0, 0, 0, 0]
            },
            KeyedLiteralOccurrence {
                instruction: 10,
                kind: KeyedLiteralKind::SuccessorOutputIndex,
                bytes: vec![]
            },
            KeyedLiteralOccurrence {
                instruction: 15,
                kind: KeyedLiteralKind::FinalTruth,
                bytes: vec![1]
            },
        ]
    );
    assert_eq!(
        linked.closure().references(),
        [
            KeyedReferenceOccurrence {
                instruction: 2,
                reference: KeyedReference::BirthCurrentInput
            },
            KeyedReferenceOccurrence {
                instruction: 3,
                reference: KeyedReference::ConsumedOutpoint
            },
            KeyedReferenceOccurrence {
                instruction: 8,
                reference: KeyedReference::ContinuationCurrentInput
            },
            KeyedReferenceOccurrence {
                instruction: 9,
                reference: KeyedReference::CurrentProgram
            },
            KeyedReferenceOccurrence {
                instruction: 11,
                reference: KeyedReference::SuccessorProgram
            },
        ]
    );
}

// Explicit source lock, not macro expansion, aliases or arbitrary trait analysis.
fn immutable_artifact(source: &str, name: &str, producer: &str) -> bool {
    let text = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let flat = text.split_whitespace().collect::<String>();
    let marker = format!("pubstruct{name}{{");
    let Some(body) = flat.split(&marker).nth(1).and_then(|s| s.split('}').next()) else {
        return false;
    };
    if body.contains("pub") || flat.contains("engine_") || flat.contains("Default") {
        return false;
    }
    for implementation in flat.split("impl").skip(1) {
        let head = implementation.split('{').next().unwrap_or(implementation);
        if head.contains(&format!("for{name}"))
            && ["From<", "TryFrom<", "DerefMut", "AsMut<", "BorrowMut<"]
                .iter()
                .any(|word| head.contains(word))
        {
            return false;
        }
    }
    let mut own_impl = false;
    let mut depth = 0_i32;
    for section in text.split_inclusive(['{', '}']) {
        let head = section.trim();
        if head.contains(&format!("impl {name}")) {
            own_impl = true;
            depth = 0;
        }
        if let Some(start) = head.find("pub ") {
            let signature = head[start..].split_whitespace().collect::<String>();
            if signature.contains("fn") {
                let returned = signature.split("->").nth(1).unwrap_or("");
                let creates = (returned.contains(name) && !returned.starts_with('&'))
                    || (own_impl && returned.contains("Self") && !returned.starts_with('&'));
                if signature.contains("&mut")
                    || signature.contains("_mut(")
                    || (creates && !signature.contains(&format!("fn{producer}(")))
                {
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
            own_impl = false;
        }
    }
    true
}

#[test]
fn keyed_link_surface_has_one_producer_and_no_mutation() {
    let source = include_str!("../src/keyed_bundle.rs");
    assert!(immutable_artifact(
        source,
        "LinkedKeyedBundle",
        "link_keyed_bundle"
    ));
    for addition in [
        "impl LinkedKeyedBundle { pub fn new(x: [u8; 32]) -> Self { todo!() } }",
        "pub fn forge(x: [u8; 32]) -> LinkedKeyedBundle { todo!() }",
        "impl LinkedKeyedBundle { pub fn mutate(&mut self) {} }",
        "impl Default for LinkedKeyedBundle {}",
        "impl From<()> for LinkedKeyedBundle {}",
        "impl DerefMut for LinkedKeyedBundle {}",
    ] {
        assert!(!immutable_artifact(
            &format!("{source}\n{addition}"),
            "LinkedKeyedBundle",
            "link_keyed_bundle"
        ));
    }
    assert!(!immutable_artifact(
        &source.replace(
            "emission: KeyedEmissionBundle,",
            "pub emission: KeyedEmissionBundle,"
        ),
        "LinkedKeyedBundle",
        "link_keyed_bundle"
    ));
    assert!(!source.contains("compiler::"));
    assert!(!source.contains("transaction::"));
    assert!(!source.contains("vectors::"));
}
