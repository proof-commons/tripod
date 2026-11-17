//! Fresh-process reconstruction from public fixed-form inputs.

use std::io::Write;

use architecture::keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_fixed_plan::{PlacementSearchLimits, plan},
};
use elements::hashes::sha256d;
use linker::{
    keyed_fixed_bundle::link_keyed_fixed_bundle,
    keyed_fixed_reference::keyed_fixed_parameter_offers,
};
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use tapscript::keyed_fixed_bundle::{KeyedFixedBundleSelection, emit_keyed_fixed_bundle};
use tapscript::keyed_fixed_constructor::KeyedFixedLeafRecipe;
use target_elements::reviewed_elements_tapscript;
use transaction::keyed_fixed_construct::{
    KeyedFixedConstructionRequest, KeyedFixedOutputOffer, construct_keyed_fixed,
};
use transaction::keyed_fixed_inputs::KeyedFixedInputRefusal;
use transaction::keyed_fixed_inputs::{KeyedFixedInputOffer, KeyedFixedInputRole};
use transaction::keyed_fixed_request::{
    KeyedFixedRequestRefusal, keyed_fixed_merge_request, keyed_fixed_source_request,
};
use transaction::keyed_fixed_taproot::{
    KeyedFixedAuthenticationRefusal, commit_keyed_fixed_bundle,
};
use transaction::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};
use vectors::keyed_fixed_constructor::{PublicKeyedCurve, construct as independent_commitment};

fn parameters(seed: u8) -> KeyedFixedParameters {
    KeyedFixedParameters {
        key: [seed; 32],
        asset: [3; 32],
        value: KeyedFixedValue::new(1_000).unwrap(),
    }
}

fn parent(program: &[u8], value: u64, salt: u32) -> Vec<u8> {
    TargetTransaction::new(
        2,
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([1; 32]), salt).unwrap(),
            123,
        )],
        vec![TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal([3; 32])),
            ValueField::Explicit(value),
            NonceField::Null,
            program.to_vec(),
        )],
        0,
        vec![InputWitness::new(vec![])],
    )
    .unwrap()
    .encode()
}

fn offer<'a>(parent: &[u8], index: u32, role: KeyedFixedInputRole) -> KeyedFixedInputOffer<'_, 'a> {
    KeyedFixedInputOffer {
        parent,
        input_index: index,
        outpoint: Outpoint::new(
            Txid::from_internal(sha256d::Hash::hash(parent).to_byte_array()),
            0,
        )
        .unwrap(),
        sequence: 123,
        flags: 0,
        role,
        capability: None,
    }
}

#[allow(clippy::too_many_lines)] // Keep the public reconstruction, migration refusal and byte comparisons in one child subject.
fn reproduce(seed: u8) {
    let target = reviewed_elements_tapscript().unwrap();
    let q = parameters(seed);
    let independent = independent_commitment(&target, q, &PublicKeyedCurve).unwrap();
    let manifest = KeyedFixedManifest::contract().unwrap();
    let declaration = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
        .unwrap()
        .validate(&manifest)
        .unwrap();
    let limit = std::num::NonZeroU64::new(100_000).unwrap();
    for operation in [KeyedFixedOperationId::Source, KeyedFixedOperationId::Merge] {
        let input = bind_input(
            &manifest,
            declaration.clone(),
            vec![operation],
            AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
        )
        .unwrap();
        let plan = plan(&input, PlacementSearchLimits::new(limit, limit)).unwrap();
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
        let linked = link_keyed_fixed_bundle(
            &target,
            &emission,
            &keyed_fixed_parameter_offers(&target, recipe),
        )
        .unwrap();
        let commitment = commit_keyed_fixed_bundle(&target, &linked).unwrap();
        assert_eq!(commitment.root(), &independent.root);
        assert_eq!(commitment.tweak(), &independent.tweak);
        assert_eq!(commitment.output_program(), independent.output_program);
        assert_eq!(
            commitment.control().as_slice(),
            independent.control.as_slice()
        );
        let sponsor = parent(
            &[
                0, 20, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            ],
            1_000_000,
            1,
        );
        let member = parent(commitment.output_program(), 1_000, 2);
        if seed == 12 && operation == KeyedFixedOperationId::Merge {
            let prior = independent_commitment(&target, parameters(11), &PublicKeyedCurve).unwrap();
            let prior_member = parent(&prior.output_program, 1_000, 3);
            assert!(matches!(
                keyed_fixed_merge_request(
                    &target,
                    q,
                    &commitment,
                    &[
                        offer(&sponsor, 0, KeyedFixedInputRole::Collateral),
                        offer(&prior_member, 1, KeyedFixedInputRole::Member),
                    ],
                ),
                Err(KeyedFixedRequestRefusal::Input {
                    index: 1,
                    refusal: KeyedFixedInputRefusal::Authentication(
                        KeyedFixedAuthenticationRefusal::WrongProgram
                    ),
                })
            ));
        }
        let mut offers = vec![offer(&sponsor, 0, KeyedFixedInputRole::Collateral)];
        if operation == KeyedFixedOperationId::Merge {
            offers.push(offer(&member, 1, KeyedFixedInputRole::Member));
        }
        let source;
        let merge;
        let request = if operation == KeyedFixedOperationId::Source {
            source = keyed_fixed_source_request(&target, q, &commitment, &offers).unwrap();
            KeyedFixedConstructionRequest::Source(&source)
        } else {
            merge = keyed_fixed_merge_request(&target, q, &commitment, &offers).unwrap();
            KeyedFixedConstructionRequest::Merge(&merge)
        };
        let count = usize::from(operation == KeyedFixedOperationId::Merge);
        let tx = construct_keyed_fixed(
            &target,
            request,
            KeyedFixedOutputOffer {
                policy_asset: AssetId::from_internal(q.asset),
                successor_value: q.value.get(),
                additional_outputs: vec![TargetOutput::new(
                    AssetField::Explicit(AssetId::from_internal(q.asset)),
                    ValueField::Explicit(998_000 + count as u64 * 1_000),
                    NonceField::Null,
                    vec![
                        0, 20, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
                    ],
                )],
                fee: Some(1_000),
            },
        )
        .unwrap();
        assert_eq!(
            tx.transaction().outputs()[0].program(),
            independent.output_program
        );
        assert_eq!(
            tx.transaction().outputs()[0].value(),
            ValueField::Explicit(1_000)
        );
        if count == 1 {
            assert_eq!(
                tx.transaction().witnesses()[1].stack(),
                [independent.leaf.clone(), independent.control.clone()]
            );
        }
    }
}

#[test]
fn fixed_public_source_merge_reproduce_in_fresh_process() {
    if let Ok(seed) = std::env::var("TRIPOD_KEYED_FIXED_CHILD") {
        reproduce(seed.parse().unwrap());
        return;
    }
    for seed in [11, 12] {
        let start = std::time::Instant::now();
        let child = std::process::Command::new("/usr/bin/timeout")
            .args(["--kill-after=5", "120"])
            .arg(std::env::current_exe().unwrap())
            .env_clear()
            .arg("--exact")
            .arg("fixed_public_source_merge_reproduce_in_fresh_process")
            .env("TRIPOD_KEYED_FIXED_CHILD", seed.to_string())
            .output()
            .unwrap();
        writeln!(
            std::io::stdout().lock(),
            "fixed reproduction seed={seed} child wall={:.3}s timeout=120s exit={}",
            start.elapsed().as_secs_f64(),
            child.status
        )
        .unwrap();
        let stdout = String::from_utf8_lossy(&child.stdout);
        assert!(
            child.status.success(),
            "{stdout}\n{}",
            String::from_utf8_lossy(&child.stderr)
        );
        assert_eq!(
            stdout
                .lines()
                .filter(|line| *line == "running 1 test")
                .count(),
            1
        );
        assert_eq!(
            stdout
                .lines()
                .filter(|line| line.starts_with("test result: ok. 1 passed; 0 failed;"))
                .count(),
            1
        );
    }
}
