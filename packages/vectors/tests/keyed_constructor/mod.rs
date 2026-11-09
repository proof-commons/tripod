//! Offline constructor controls; synthetic funding is not a mined observation.

use std::collections::BTreeSet;
use std::process::Command;

use tapscript::keyed_constructor::{
    KeyedCurveCapability, KeyedSemanticKey, KeyedTweakOutcome, keyed_leaf_program,
};
use tapscript::{
    AbstractLimits, AbstractStackState, TapscriptProgram, program_stack_profile, validate_program,
};
use target_elements::{FailureCause, StackValueType, reviewed_elements_tapscript};
use target_elements_conformance::constructor::curve::generator;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::constructor::{
    FixtureTapTree, UNSPENDABLE_INTERNAL_KEY, construct,
};
use transaction::bytes::TargetTransaction;
use vectors::keyed_birth_native::{BirthLeaf, build_birth_parent};
use vectors::keyed_constructor::{
    KeyedConstructorDescriptor, KeyedConstructorManifest, KeyedConstructorRefusal,
    KeyedInternalKeyDerivation, KeyedNativeSubject, KeyedSpendPath, PublicKeyedCurve,
    authenticate_keyed_predecessor, construct_keyed, reconstruct_keyed,
};

// Test handles only, never deployment defaults or plan-level identities.
const K: [u8; 32] = [0x31; 32];
const L: [u8; 32] = [0x32; 32];

fn descriptor(key: [u8; 32]) -> KeyedConstructorDescriptor {
    construct_keyed(
        &reviewed_elements_tapscript().expect("target"),
        KeyedSemanticKey::new(key),
        &PublicKeyedCurve,
    )
    .expect("construction")
}

fn predecessor(d: &KeyedConstructorDescriptor) -> TargetTransaction {
    let leaf = BirthLeaf {
        script: d.leaf_bytes.clone(),
        program: d.output_program.clone(),
        control_block: d.control.bytes.clone(),
    };
    let parent =
        build_birth_parent(&leaf, &super::keyed_birth::coin(0, 0)).expect("synthetic funding");
    TargetTransaction::decode(&parent.encode()).expect("actual output bytes")
}

#[test]
fn canonical_key_literal_is_data_before_both_clauses() {
    let target = reviewed_elements_tapscript().expect("target");
    let program = keyed_leaf_program(&target, KeyedSemanticKey::new(K)).expect("leaf");
    let bytes = program.encode(&target);
    assert_eq!(bytes[0], 32);
    assert_eq!(&bytes[1..33], K);
    assert_eq!(
        &bytes[33..],
        &[
            0x75, 0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0xcd, 0xca, 0, 0xd1, 0x7b, 0x88,
            0x88, 0x51
        ]
    );
    assert_eq!(
        TapscriptProgram::decode(&target, &bytes),
        Ok(program.clone())
    );
    let empty = AbstractStackState::from_main(vec![]);
    let result = validate_program(
        &target,
        &program,
        &empty,
        AbstractLimits::for_target(&target),
    )
    .expect("typed schedule");
    assert_eq!(
        result.success(),
        &BTreeSet::from([AbstractStackState::from_main(vec![StackValueType::Bytes {
            minimum: 1,
            maximum: 1
        }])])
    );
    assert!(result.nonaborting_failure().is_empty());
    assert_eq!(
        result.aborts(),
        &BTreeSet::from([
            FailureCause::UnsupportedExecutionDomain,
            FailureCause::IntrospectionContextUnavailable,
            FailureCause::MalformedScriptNumber,
            FailureCause::IntrospectionIndexOutOfRange,
            FailureCause::UnequalOperands
        ])
    );
    assert!(result.signature_forms().is_empty());
    let profile = program_stack_profile(
        &target,
        &program,
        &empty,
        AbstractLimits::for_target(&target),
    );
    assert_eq!((profile.peak_main(), profile.peak_alternate()), (4, 0));
    assert!(
        keyed_leaf_program(&target, KeyedSemanticKey::new([0; 32])).is_ok(),
        "no nonzero or signing-key law"
    );
}

#[test]
fn repeated_construction_reproduces_every_public_byte() {
    let target = reviewed_elements_tapscript().expect("target");
    let first = descriptor(K);
    assert_eq!(first, descriptor(K));
    assert_eq!(
        reconstruct_keyed(&target, &first, &PublicKeyedCurve),
        Ok(first.clone())
    );
    let public = serde_json::to_vec(&first).expect("descriptor JSON");
    let offered = serde_json::from_slice(&public).expect("descriptor only");
    assert_eq!(
        reconstruct_keyed(&target, &offered, &PublicKeyedCurve),
        Ok(first)
    );
}

#[test]
fn distinct_semantic_key_changes_only_literal_and_dependent_commitments() {
    let k = descriptor(K);
    let l = descriptor(L);
    assert_eq!(k.schema, l.schema);
    assert_eq!(k.internal_key, l.internal_key);
    assert_eq!(k.internal_key_derivation, l.internal_key_derivation);
    assert_eq!(k.leaf_version, l.leaf_version);
    assert_eq!(k.leaf_bytes[0], l.leaf_bytes[0]);
    assert_eq!(k.leaf_bytes[33..], l.leaf_bytes[33..]);
    assert_eq!(k.control.siblings, l.control.siblings);
    assert_ne!(k.leaf_hash, l.leaf_hash);
    assert_ne!(k.merkle_root, l.merkle_root);
    assert_ne!(k.output_program, l.output_program);
    // Finite evidence, not universal injectivity or a new cryptographic premise.
}

#[test]
fn nums_derivation_and_public_hash_control_machinery_agree() {
    let target = reviewed_elements_tapscript().expect("target");
    for key in [K, L] {
        let d = descriptor(key);
        assert_eq!(d.internal_key, sha256(&generator().uncompressed_bytes()));
        assert_eq!(d.internal_key, UNSPENDABLE_INTERNAL_KEY);
        assert_eq!(
            d.internal_key_derivation,
            KeyedInternalKeyDerivation::Sha256UncompressedGenerator
        );
        assert_eq!(
            d.leaf_hash,
            transaction::taproot::leaf_hash(target.definition().leaf_version(), &d.leaf_bytes)
        );
        let reference = construct(&d.internal_key, &d.tree, &d.tree).expect("public reference");
        assert_eq!(&d.merkle_root, reference.merkle_root());
        assert_eq!(&d.output_key, reference.output_key());
        assert_eq!(d.output_program, reference.output_program());
        assert_eq!(d.control.bytes, reference.control_block());
    }
}

#[test]
fn foreign_internal_key_is_a_constructor_policy_refusal() {
    let target = reviewed_elements_tapscript().expect("target");
    let mut offered = descriptor(K);
    assert!(reconstruct_keyed(&target, &offered, &PublicKeyedCurve).is_ok());
    offered.internal_key = generator().x_only_bytes();
    assert_eq!(
        reconstruct_keyed(&target, &offered, &PublicKeyedCurve),
        Err(KeyedConstructorRefusal::InternalKeyPolicy)
    );
}

#[test]
fn extra_leaf_is_a_typed_census_refusal() {
    let target = reviewed_elements_tapscript().expect("target");
    let mut offered = descriptor(K);
    assert!(reconstruct_keyed(&target, &offered, &PublicKeyedCurve).is_ok());
    offered.tree = FixtureTapTree::branch(offered.tree.clone(), FixtureTapTree::leaf(vec![0x51]));
    assert_eq!(
        reconstruct_keyed(&target, &offered, &PublicKeyedCurve),
        Err(KeyedConstructorRefusal::LeafCensus)
    );
}

#[test]
fn complete_path_census_authenticates_each_synthetic_predecessor() {
    let target = reviewed_elements_tapscript().expect("target");
    for key in [K, L] {
        let d = descriptor(key);
        let parent = predecessor(&d);
        let manifest = KeyedConstructorManifest::new(
            &target,
            &d,
            parent.outputs()[0].program(),
            &PublicKeyedCurve,
        )
        .expect("bound census");
        assert_eq!(
            manifest.paths(),
            &[
                KeyedSpendPath::CanonicalScriptLeaf,
                KeyedSpendPath::ConditionalNumsKeyPath
            ]
        );
        assert_eq!(manifest.descriptor().tree.leaf_count(), 1);
        assert_eq!(d.leaf_hash, d.merkle_root);
        assert_eq!(d.control.siblings, [[0_u8; 32]; 0]);
        assert_eq!(d.control.bytes.len(), 33);
        assert_eq!(
            d.control.bytes[0],
            d.leaf_version | u8::from(d.control.parity)
        );
        assert_eq!(&d.control.bytes[1..], d.internal_key);
    }
    let k = descriptor(K);
    let foreign = predecessor(&descriptor(L));
    assert_eq!(
        authenticate_keyed_predecessor(
            &target,
            &k,
            foreign.outputs()[0].program(),
            &PublicKeyedCurve
        ),
        Err(KeyedConstructorRefusal::PredecessorCommitment)
    );
}

#[test]
fn altered_descriptor_fields_do_not_pass_public_reconstruction() {
    let target = reviewed_elements_tapscript().expect("target");
    let d = descriptor(K);
    let mut mutants = Vec::new();
    let mut x = d.clone();
    x.schema += 1;
    mutants.push(x);
    let mut x = d.clone();
    x.semantic_key = L;
    mutants.push(x);
    let mut x = d.clone();
    x.leaf_bytes[1] ^= 1;
    mutants.push(x);
    let mut x = d.clone();
    x.leaf_version ^= 2;
    mutants.push(x);
    let mut x = d.clone();
    x.leaf_hash[0] ^= 1;
    mutants.push(x);
    let mut x = d.clone();
    x.merkle_root[0] ^= 1;
    mutants.push(x);
    let mut x = d.clone();
    x.output_key[0] ^= 1;
    mutants.push(x);
    let mut x = d.clone();
    x.output_program[2] ^= 1;
    mutants.push(x);
    let mut x = d.clone();
    x.control.parity = !x.control.parity;
    mutants.push(x);
    let mut x = d.clone();
    x.control.bytes[0] ^= 1;
    mutants.push(x);
    let mut x = d.clone();
    x.control.siblings.push(d.leaf_hash);
    mutants.push(x);
    let mut x = d;
    x.tree = FixtureTapTree::leaf(vec![0x51]);
    mutants.push(x);
    for mutant in mutants {
        assert_eq!(
            reconstruct_keyed(&target, &mutant, &PublicKeyedCurve),
            Err(KeyedConstructorRefusal::DescriptorMismatch)
        );
    }
}

struct RefusingCurve(KeyedTweakOutcome);
impl KeyedCurveCapability for RefusingCurve {
    fn internal_key_is_a_point(&self, _: &[u8; 32]) -> bool {
        true
    }
    fn output_key(&self, _: &[u8; 32], _: &[u8; 32]) -> KeyedTweakOutcome {
        self.0
    }
}

#[test]
fn curve_refusals_are_terminal_without_rekeying_or_nonce_search() {
    let target = reviewed_elements_tapscript().expect("target");
    for (outcome, refusal) in [
        (
            KeyedTweakOutcome::InternalKeyNotAPoint,
            KeyedConstructorRefusal::InternalKeyNotAPoint,
        ),
        (
            KeyedTweakOutcome::TweakAboveGroupOrder,
            KeyedConstructorRefusal::TweakAboveGroupOrder,
        ),
        (
            KeyedTweakOutcome::TweakedPointIsIdentity,
            KeyedConstructorRefusal::TweakedPointIsIdentity,
        ),
    ] {
        assert_eq!(
            construct_keyed(&target, KeyedSemanticKey::new(K), &RefusingCurve(outcome)),
            Err(refusal)
        );
    }
}

#[test]
fn native_subjects_are_described_but_remain_outstanding() {
    let target = reviewed_elements_tapscript().expect("target");
    let d = descriptor(K);
    let parent = predecessor(&d);
    let manifest = KeyedConstructorManifest::new(
        &target,
        &d,
        parent.outputs()[0].program(),
        &PublicKeyedCurve,
    )
    .expect("manifest");
    assert_eq!(
        manifest.native_subjects(),
        &[
            KeyedNativeSubject::WrongControlParity,
            KeyedNativeSubject::UncommittedLeaf,
            KeyedNativeSubject::KeyPathAttempt
        ]
    );
    for row in manifest.native_subjects() {
        assert_ne!(row.changed_coordinate(), "");
        assert_ne!(row.expected_boundary(), "");
        assert!(
            row.collateral()
                .contains("no non-witness transaction change")
        );
        assert_eq!(
            row.standing(),
            "capture outstanding; no native outcome admitted"
        );
    }
}

#[test]
fn descriptor_reconstructs_in_a_fresh_process_without_creator_state() {
    for key in [K, L] {
        let directory = tempfile::tempdir().expect("public descriptor directory");
        let input = directory.path().join("descriptor.json");
        let output = directory.path().join("reconstructed.json");
        let original = descriptor(key);
        std::fs::write(&input, serde_json::to_vec(&original).expect("public bytes"))
            .expect("input");
        let result = Command::new(std::env::current_exe().expect("test executable"))
            .env_clear()
            .env("TRIPOD_KEYED_DESCRIPTOR_INPUT", &input)
            .env("TRIPOD_KEYED_DESCRIPTOR_OUTPUT", &output)
            .args([
                "--exact",
                "keyed_constructor::fresh_process_reconstruction_child",
                "--nocapture",
            ])
            .output()
            .expect("fresh process");
        assert!(
            result.status.success(),
            "child stdout={} stderr={}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        let reconstructed: KeyedConstructorDescriptor =
            serde_json::from_slice(&std::fs::read(&output).expect("child output"))
                .expect("public descriptor");
        assert_eq!(reconstructed, original);
        assert_eq!(
            std::fs::read(&input).expect("input"),
            std::fs::read(&output).expect("output")
        );
    }
}

#[test]
fn fresh_process_reconstruction_child() {
    let Some(input) = std::env::var_os("TRIPOD_KEYED_DESCRIPTOR_INPUT") else {
        return;
    };
    let offered: KeyedConstructorDescriptor =
        serde_json::from_slice(&std::fs::read(input).expect("public input"))
            .expect("descriptor alone");
    let target = reviewed_elements_tapscript().expect("public target");
    let rebuilt =
        reconstruct_keyed(&target, &offered, &PublicKeyedCurve).expect("no creator state");
    let output = std::env::var_os("TRIPOD_KEYED_DESCRIPTOR_OUTPUT").expect("output destination");
    std::fs::write(output, serde_json::to_vec(&rebuilt).expect("public output")).expect("output");
}
