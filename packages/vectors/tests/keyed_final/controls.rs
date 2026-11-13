use super::{fixture, prediction};
use crate::keyed_reproduction::fixture as prior;
use vectors::keyed_final_manifest::*;

/// Pin the freeze inventory, textual entry and each public binding refusal.
pub fn manifest() {
    let key = SECOND_KEY;
    let control = FreezeManifest::offer(key);
    assert_eq!(validate_manifest(&control, key), Ok(()));
    assert_eq!(control.frozen.len(), 12);
    assert_eq!(control.frozen.values().map(Vec::len).sum::<usize>(), 35);
    assert_eq!(control.computed.len(), 16);
    assert_eq!(
        control.recipe_schema,
        tapscript::keyed_constructor::KeyedLeafRecipe::new(key).schema()
    );
    assert_eq!(
        control.target_version,
        target_elements::reviewed_elements_tapscript()
            .unwrap()
            .definition()
            .version()
            .get()
    );
    assert_eq!(
        control.envelope.handles,
        transaction::keyed_construct::KEYED_POLICY_HANDLE_MAXIMUM as u64
    );
    assert_eq!(
        control.envelope.weight_units,
        vectors::keyed_scaling::STANDARD_WEIGHT
    );
    assert_eq!(
        control.envelope.signature_bytes,
        vectors::keyed_scaling::SPONSOR_SIGNATURE_BYTES as u64
    );
    assert_eq!(
        control.envelope.handles,
        vectors::keyed_scaling::PROPOSED_MAXIMUM as u64
    );
    assert_eq!(
        target_elements::reviewed_elements_tapscript()
            .unwrap()
            .definition()
            .resources()
            .policy()
            .bounds()
            .get(&target_elements::ResourceDimension::TransactionWeight)
            .and_then(|b| b.maximum()),
        Some(control.envelope.weight_units)
    );
    super::envelope::check();
    let mut bad = control.clone();
    bad.key[0] ^= 1;
    assert_eq!(validate_manifest(&bad, key), Err(FreezeRefusal::KeyBinding));
    bad = control.clone();
    bad.leaf_digest[0] ^= 1;
    assert_eq!(
        validate_manifest(&bad, key),
        Err(FreezeRefusal::LeafBinding)
    );
    bad = control.clone();
    bad.recipe_schema += 1;
    assert_eq!(validate_manifest(&bad, key), Err(FreezeRefusal::Recipe));
    bad = control.clone();
    bad.target_version += 1;
    assert_eq!(validate_manifest(&bad, key), Err(FreezeRefusal::Target));
    bad = control.clone();
    bad.envelope.handles += 1;
    assert_eq!(validate_manifest(&bad, key), Err(FreezeRefusal::Policy));
    bad = control.clone();
    bad.computed.pop();
    assert_eq!(
        validate_manifest(&bad, key),
        Err(FreezeRefusal::ComputedInventory)
    );
    bad = control.clone();
    bad.frozen.get_mut(&FrozenItem::Recipe).unwrap()[0].digest[0] ^= 1;
    assert_eq!(
        validate_manifest(&bad, key),
        Err(FreezeRefusal::FrozenInventory)
    );
    assert_eq!(validate_manifest(&control, key), Ok(()));
    text_and_boundaries(&control);
    for key in [[42; 32], SECOND_KEY] {
        scopes(key);
    }
}

fn text_and_boundaries(control: &FreezeManifest) {
    let key = SECOND_KEY;
    assert_eq!(parse_key(&"00".repeat(32)), Ok([0; 32]));
    assert_eq!(parse_key(&"00".repeat(31)), Err(FreezeRefusal::KeyWidth));
    assert_eq!(parse_key(&"00".repeat(33)), Err(FreezeRefusal::KeyWidth));
    let lower = "ab".repeat(32);
    assert_eq!(parse_key(&lower), Ok([0xab; 32]));
    assert_eq!(parse_key(&lower.to_uppercase()), Ok([0xab; 32]));
    let mut bad_text = lower;
    bad_text.pop();
    bad_text.push('g');
    assert_eq!(parse_key(&bad_text), Err(FreezeRefusal::KeyEncoding));
    for coordinate in 0..4 {
        let mut bad = control.clone();
        match coordinate {
            0 => bad.envelope.signature_bytes += 1,
            1 => bad.envelope.weight_units += 1,
            2 => bad.envelope.sponsors += 1,
            3 => bad.envelope.outputs += 1,
            _ => unreachable!(),
        }
        assert_eq!(validate_manifest(&bad, key), Err(FreezeRefusal::Policy));
    }
    for (refusal, boundary) in [
        (FreezeRefusal::KeyWidth, FreezeBoundary::TextKey),
        (FreezeRefusal::KeyEncoding, FreezeBoundary::TextKey),
        (
            FreezeRefusal::SourceDrift(String::new()),
            FreezeBoundary::SourceFreeze,
        ),
        (FreezeRefusal::FrozenInventory, FreezeBoundary::SourceFreeze),
        (
            FreezeRefusal::ComputedInventory,
            FreezeBoundary::SourceFreeze,
        ),
        (FreezeRefusal::KeyBinding, FreezeBoundary::ArtifactBinding),
        (FreezeRefusal::LeafBinding, FreezeBoundary::ArtifactBinding),
        (FreezeRefusal::Recipe, FreezeBoundary::ArtifactBinding),
        (FreezeRefusal::Target, FreezeBoundary::ArtifactBinding),
        (FreezeRefusal::Policy, FreezeBoundary::ArtifactBinding),
    ] {
        assert_eq!(refusal.boundary(), boundary);
    }
}

fn scopes(key: [u8; 32]) {
    use architecture::{
        keyed::KeyedManifest,
        semantic::{KeyedOperationId as K, SemanticOperationId as O},
    };
    use compiler::{
        AnalysisPolicy, ProofSearchLimits, bind_input,
        keyed_plan::{PlacementSearchLimits, plan},
    };
    use linker::{keyed_bundle::link_keyed_bundle, keyed_reference::keyed_parameter_offers};
    use tapscript::{
        keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle},
        keyed_constructor::KeyedLeafRecipe,
    };
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    for (scope, relations, evidence) in [
        (vec![K::Source], 4, 4),
        (vec![K::Merge], 5, 3),
        (vec![K::Source, K::Merge], 9, 7),
    ] {
        let manifest = KeyedManifest::contract().unwrap();
        let declaration = realization::keyed::KeyedRealizationDraft::derive(&manifest)
            .unwrap()
            .validate(&manifest)
            .unwrap();
        let limit = std::num::NonZeroU64::new(100_000).unwrap();
        let input = bind_input(
            &manifest,
            declaration,
            scope.into_iter().map(O::Keyed).collect(),
            AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
        )
        .unwrap();
        let plan = plan(&input, PlacementSearchLimits::new(limit, limit)).unwrap();
        let p = plan.project();
        let selection = KeyedBundleSelection {
            binding: p.binding.clone(),
            scope: p.scope.clone(),
            proof: 0,
            placements: p.plans[0].operations.keys().map(|k| (*k, 0)).collect(),
        };
        let original_selection = selection;
        let selection = vectors::keyed_final_selection::first_selection(p).unwrap();
        assert_eq!(selection, original_selection);
        let recipe = KeyedLeafRecipe::new(key);
        let emission = emit_keyed_bundle(&target, &plan, selection, recipe).unwrap();
        let mut offers = keyed_parameter_offers(&target, recipe);
        let linked = link_keyed_bundle(&target, &emission, &offers).unwrap();
        offers.reverse();
        assert_eq!(
            link_keyed_bundle(&target, &emission, &offers),
            Ok(linked.clone())
        );
        if p.scope.len() == 1 {
            assert_eq!(
                linked,
                prior::linked(key, p.scope[0] == O::Keyed(K::Source))
            );
        }
        assert_eq!(linked.bytes(), predicted_leaf(key));
        assert_eq!(linked.project().evidence.len(), evidence);
        assert_eq!(
            linked
                .project()
                .selected
                .operations
                .values()
                .map(|o| o.relations.len())
                .sum::<usize>(),
            relations
        );
    }
}

/// Accept data extremes and distinguish synthetic arithmetic refusals from keys.
pub fn edge_keys() {
    use tapscript::keyed_constructor::{KeyedCurveCapability, KeyedSemanticKey, KeyedTweakOutcome};
    struct Synthetic(KeyedTweakOutcome);
    impl KeyedCurveCapability for Synthetic {
        fn internal_key_is_a_point(&self, _: &[u8; 32]) -> bool {
            true
        }
        fn output_key(&self, _: &[u8; 32], _: &[u8; 32]) -> KeyedTweakOutcome {
            self.0
        }
    }
    use vectors::keyed_constructor::{
        KeyedConstructorRefusal as R, PublicKeyedCurve, construct_keyed,
    };
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    for key in [[0; 32], [255; 32], [128; 32]] {
        let commitment =
            transaction::keyed_taproot::commit_keyed_bundle(&target, &prior::linked(key, true))
                .unwrap();
        prediction::check(key, &commitment);
        assert_eq!(
            tapscript::keyed_constructor::KeyedSemanticKey::new(key).bytes(),
            &key
        );
    }
    assert!(
        construct_keyed(
            &target,
            KeyedSemanticKey::new(SECOND_KEY),
            &PublicKeyedCurve
        )
        .is_ok()
    );
    // Synthetic capability outputs, NOT keys alleged to reach hash preimages.
    for (outcome, refusal) in [
        (
            KeyedTweakOutcome::TweakAboveGroupOrder,
            R::TweakAboveGroupOrder,
        ),
        (
            KeyedTweakOutcome::TweakedPointIsIdentity,
            R::TweakedPointIsIdentity,
        ),
    ] {
        assert_eq!(
            construct_keyed(
                &target,
                KeyedSemanticKey::new(SECOND_KEY),
                &Synthetic(outcome)
            ),
            Err(refusal)
        );
    }
    // Production's exact scalar/point core controls live in keyed_curve.rs.
    assert_eq!(
        transaction::keyed_taproot::KeyedCommitmentRefusal::TweakAboveGroupOrder.refusal_kind(),
        "tweak_above_group_order"
    );
    assert_eq!(
        transaction::keyed_taproot::KeyedCommitmentRefusal::TweakedPointIsIdentity.refusal_kind(),
        "tweaked_point_is_identity"
    );
}

/// Refuse foreign members, successors and a noncanonical script at their owners.
pub fn bindings() {
    use transaction::{
        Outpoint, TargetOutput, TargetTransaction, Txid,
        keyed_inputs::{KeyedInputOffer, KeyedInputRefusal, KeyedInputRole},
        keyed_request::{KeyedRequestRefusal, keyed_merge_request},
        keyed_taproot::{
            KeyedAuthenticationRefusal as A, authenticate_keyed_path, commit_keyed_bundle,
        },
    };
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let (packets, snapshots) = fixture::graph(SECOND_KEY);
    let own = commit_keyed_bundle(&target, &prior::linked(SECOND_KEY, false)).unwrap();
    let other = commit_keyed_bundle(&target, &prior::linked([42; 32], false)).unwrap();
    let mut inputs: Vec<_> = packets[2]
        .inputs
        .iter()
        .enumerate()
        .map(|(i, row)| KeyedInputOffer {
            input_index: u32::try_from(i).unwrap(),
            parent: &row.parent,
            outpoint: Outpoint::new(Txid::from_internal(row.txid), row.vout).unwrap(),
            sequence: row.sequence,
            flags: 0,
            role: if row.member {
                KeyedInputRole::Member
            } else {
                KeyedInputRole::Collateral
            },
            capability: None,
        })
        .collect();
    assert!(keyed_merge_request(&target, SECOND_KEY, &own, &inputs).is_ok());
    let foreign = replace_output_zero(&packets[2].inputs[1].parent, other.output_program());
    inputs[1].parent = &foreign;
    inputs[1].outpoint = Outpoint::new(Txid::from_internal(prior::txid(&foreign)), 0).unwrap();
    assert!(keyed_merge_request(&target, SECOND_KEY, &own, &inputs[..1]).is_ok());
    assert_eq!(
        keyed_merge_request(&target, SECOND_KEY, &own, &inputs).unwrap_err(),
        KeyedRequestRefusal::Input {
            index: 1,
            refusal: KeyedInputRefusal::Authentication(A::WrongProgram)
        }
    );
    assert_eq!(
        authenticate_keyed_path(&own, own.leaf(), own.control()),
        Ok(())
    );
    let mut nonminimal = vec![0x4c, 32];
    nonminimal.extend_from_slice(&own.leaf()[1..]);
    assert_eq!(
        authenticate_keyed_path(&own, &nonminimal, own.control()),
        Err(A::WrongLeaf)
    );
    let mut evidence = fixture::reader_input(SECOND_KEY, &packets, &snapshots);
    evidence
        .keys
        .insert(vectors::keyed_history_reader::Key([42; 32]));
    assert_eq!(
        vectors::keyed_history_reader::read(&evidence)
            .unwrap()
            .relations()
            .len(),
        4
    );
    let tx = TargetTransaction::decode(&snapshots[2].complete).unwrap();
    let mut outputs = tx.outputs().to_vec();
    let old = &outputs[0];
    outputs[0] = TargetOutput::new(
        old.asset(),
        old.value(),
        old.nonce(),
        other.output_program().to_vec(),
    );
    let bad = TargetTransaction::new(
        tx.version(),
        tx.inputs().to_vec(),
        outputs,
        tx.lock_time(),
        tx.witnesses().to_vec(),
    )
    .unwrap()
    .encode();
    evidence.association[2].1 = vectors::keyed_history_reader::Address(prior::txid(&bad));
    evidence.bytes.insert(30, bad);

    assert_eq!(
        vectors::keyed_history_reader::read(&evidence),
        Err(vectors::keyed_history_reader::Refusal::WrongSuccessor {
            event: 30,
            input: 0
        })
    );
}

/// Capture exactly one completion or refusal event for each public boundary.
pub fn tracing() {
    use std::sync::{Arc, Mutex};
    #[derive(Clone)]
    struct Writer(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let captured = bytes.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || Writer(captured.clone()))
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        let offer = FreezeManifest::offer(SECOND_KEY);
        assert_eq!(validate_manifest(&offer, SECOND_KEY), Ok(()));
        assert_eq!(
            validate_manifest(&offer, [42; 32]),
            Err(FreezeRefusal::KeyBinding)
        );
        assert_eq!(parse_key(&"00".repeat(32)), Ok([0; 32]));
        assert_eq!(parse_key("00"), Err(FreezeRefusal::KeyWidth));
    });
    let text = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
    for message in [
        "keyed freeze validated",
        "keyed freeze refused",
        "keyed freeze key parsed",
        "keyed freeze key refused",
    ] {
        assert_eq!(
            text.lines().filter(|l| l.contains(message)).count(),
            1,
            "{text}"
        );
    }
    assert!(text.contains("key_binding") && text.contains("key_width"));
    assert!(text.contains("validate_keyed_freeze") && text.contains("parse_keyed_freeze_key"));
    let rows: Vec<_> = text.lines().collect();
    assert_eq!(rows.len(), 4, "{text}");
    for row in rows {
        assert!(row.contains("vectors::keyed_final_manifest"), "{row}");
        assert!(!row.contains("WARN") && !row.contains("ERROR"), "{row}");
        assert!(
            row.contains(if row.contains("keyed freeze validated") {
                "INFO"
            } else {
                "DEBUG"
            }),
            "{row}"
        );
    }
}

fn replace_output_zero(bytes: &[u8], program: &[u8]) -> Vec<u8> {
    let tx = transaction::TargetTransaction::decode(bytes).unwrap();
    let mut outputs = tx.outputs().to_vec();
    let old = &outputs[0];
    outputs[0] =
        transaction::TargetOutput::new(old.asset(), old.value(), old.nonce(), program.to_vec());
    transaction::TargetTransaction::new(
        tx.version(),
        tx.inputs().to_vec(),
        outputs,
        tx.lock_time(),
        tx.witnesses().to_vec(),
    )
    .unwrap()
    .encode()
}
