//! Offline projection checks in the existing maturity-native integration target.
//! Synthetic bytes are not native transactions or constructor evidence.

mod native;

use std::collections::BTreeSet;

use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use vectors::keyed_birth::{
    BirthFeeFloor, BirthOutpoint, BirthOutput, BirthPolicy, PolicyFactOrigin,
    Wave1ExecutableProvenance, Wave1ExecutorBinding, Wave1TargetPacket,
};
use vectors::keyed_continuation::*;

fn packet() -> Wave1TargetPacket {
    let target = reviewed_elements_tapscript().expect("target");
    let deployment = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            [1; 32],
            [2; 32],
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("deployment");
    let floor = BirthFeeFloor {
        satoshis_per_kvb: 100,
        origin: PolicyFactOrigin::Declared,
        source: "synthetic".into(),
    };
    Wave1TargetPacket::bind(
        target,
        deployment,
        Wave1ExecutableProvenance {
            node_name: "synthetic".into(),
            node_version: "synthetic".into(),
            binary_revision: "a".repeat(12),
            intended_tip: "a".repeat(40),
            framework_revision: "a".repeat(40),
            executable_path: "/synthetic/elementsd".into(),
            executable_sha256: [3; 32],
            provenance_record: "synthetic".into(),
        },
        Wave1ExecutorBinding {
            chain: "elementsregtest".into(),
            protocol_schema: 8,
            adapter_name: "elements-native-executor".into(),
            adapter_version: "2.1.0".into(),
        },
        BirthPolicy {
            representation: "explicit policy asset".into(),
            fee_policy: "node-defaults".into(),
            startup_arguments: vec!["-daemon=0".into()],
            relay: floor.clone(),
            mempool: floor.clone(),
            block: floor,
        },
    )
    .expect("packet")
}

fn program(payload: u8) -> Vec<u8> {
    let mut bytes = vec![0x51, 0x20];
    bytes.extend([payload; 32]);
    bytes
}

fn subjects() -> Vec<ContinuationSubject> {
    let successor = BirthOutput {
        program: program(1),
        asset: [4; 32],
        value: 10_000,
    };
    let change = BirthOutput {
        program: program(2),
        asset: [4; 32],
        value: 900,
    };
    let context = ContinuationPairContext {
        parent_bytes: vec![1],
        spent_outpoint: BirthOutpoint {
            transaction_id: [5; 32],
            output_index: 0,
        },
        current_input_index: 0,
        predecessor: successor.clone(),
        public_key: [7; 32],
        leaf: vec![0x51],
        alternative_leaf: vec![0x51, 0x51, 0x75],
        alternative_program: program(3),
        funding_outpoint: BirthOutpoint {
            transaction_id: [6; 32],
            output_index: 1,
        },
        funding_output: BirthOutput {
            program: program(2),
            asset: [4; 32],
            value: 1_000,
        },
        control_outputs: [successor, change],
        fee: 100,
    };
    ContinuationSubjectId::ALL
        .into_iter()
        .map(|id| {
            use ContinuationSubjectId as I;
            let mut outputs = context.control_outputs.clone();
            let boundary = match id {
                I::Control => ContinuationBoundary::ScriptAcceptance,
                I::WrongProgram => {
                    outputs[0].program = program(3);
                    ContinuationBoundary::ProgramEqualVerify
                }
                I::WrongVersion => {
                    outputs[0].program[0] = 0;
                    ContinuationBoundary::VersionEqualVerify
                }
                I::DisplacedSuccessor => {
                    outputs.swap(0, 1);
                    ContinuationBoundary::ProgramEqualVerify
                }
            };
            let negative = id != I::Control;
            ContinuationSubject {
                id,
                control: negative.then_some(I::Control),
                context: context.clone(),
                outputs,
                changed_coordinate: id.coordinate(),
                collateral: if negative {
                    ContinuationCollateral::ALL.into_iter().collect()
                } else {
                    BTreeSet::new()
                },
                boundary,
                attribution: ContinuationAttribution::SharedEqualityRequiresIndependentEvidence,
            }
        })
        .collect()
}

fn refuses(subjects: Vec<ContinuationSubject>, refusal: ContinuationManifestRefusal) {
    assert_eq!(
        ContinuationCampaignManifest::new(packet(), subjects),
        Err(refusal)
    );
}

#[test]
fn census_retains_packet_one_coordinate_and_shared_site_limits() {
    let packet = packet();
    let manifest = ContinuationCampaignManifest::new(packet.clone(), subjects()).expect("manifest");
    assert_eq!(manifest.packet(), &packet);
    assert_eq!(
        manifest.subjects().iter().map(|s| s.id).collect::<Vec<_>>(),
        ContinuationSubjectId::ALL
    );
    for subject in manifest.subjects() {
        assert_eq!(subject.context.spent_outpoint.output_index, 0);
        assert_eq!(subject.context, manifest.subjects()[0].context);
        assert_eq!(subject.changed_coordinate, subject.id.coordinate());
        assert_eq!(
            subject.attribution,
            ContinuationAttribution::SharedEqualityRequiresIndependentEvidence
        );
    }
    assert_eq!(
        manifest.response_retention(),
        ContinuationResponseRetention::CompleteWireJson
    );
    assert_eq!(
        ContinuationSubjectId::SUBMISSION_ORDER,
        [
            ContinuationSubjectId::WrongProgram,
            ContinuationSubjectId::WrongVersion,
            ContinuationSubjectId::DisplacedSuccessor,
            ContinuationSubjectId::Control
        ]
    );
}

#[test]
fn missing_duplicate_and_reordered_continuation_subjects_refuse() {
    let mut missing = subjects();
    missing.pop();
    let mut duplicate = subjects();
    duplicate[1] = duplicate[0].clone();
    let mut reordered = subjects();
    reordered.swap(1, 2);
    for offered in [missing, duplicate, reordered] {
        refuses(offered, ContinuationManifestRefusal::SubjectCensus);
    }
}

#[test]
fn every_continuation_pair_context_coordinate_is_fixed() {
    let mutations: [fn(&mut ContinuationPairContext); 16] = [
        |c| c.parent_bytes.push(9),
        |c| c.spent_outpoint.transaction_id = [9; 32],
        |c| c.current_input_index = 1,
        |c| c.predecessor.value += 1,
        |c| c.predecessor.asset = [9; 32],
        |c| c.public_key = [9; 32],
        |c| c.leaf.push(9),
        |c| c.alternative_leaf.push(9),
        |c| c.alternative_program = program(9),
        |c| c.funding_outpoint.transaction_id = [9; 32],
        |c| c.funding_outpoint.output_index += 1,
        |c| c.funding_output.program = program(9),
        |c| c.funding_output.value += 1,
        |c| c.funding_output.asset = [9; 32],
        |c| c.control_outputs[1].value += 1,
        |c| c.fee += 1,
    ];
    for mutate in mutations {
        let mut offered = subjects();
        mutate(&mut offered[1].context);
        refuses(offered, ContinuationManifestRefusal::PairInvariant);
    }
}

#[test]
fn every_row_keeps_birth_position_valid() {
    for index in 0..4 {
        let mut offered = subjects();
        offered[index].context.spent_outpoint.output_index = 1;
        refuses(offered, ContinuationManifestRefusal::SubjectCoordinate);
    }
}

#[test]
fn wrong_control_coordinate_and_boundary_refuse() {
    let mutations: [fn(&mut ContinuationSubject); 3] = [
        |s| s.control = None,
        |s| s.changed_coordinate = None,
        |s| s.boundary = ContinuationBoundary::VersionEqualVerify,
    ];
    for mutate in mutations {
        let mut offered = subjects();
        mutate(&mut offered[1]);
        refuses(offered, ContinuationManifestRefusal::SubjectCoordinate);
    }
}

#[test]
fn missing_and_surplus_continuation_collateral_refuse() {
    for collateral in ContinuationCollateral::ALL {
        for index in 1..4 {
            let mut offered = subjects();
            offered[index].collateral.remove(&collateral);
            refuses(offered, ContinuationManifestRefusal::Collateral);
        }
        let mut offered = subjects();
        offered[0].collateral.insert(collateral);
        refuses(offered, ContinuationManifestRefusal::Collateral);
    }
}

#[test]
fn undeclared_output_changes_refuse_beside_each_valid_projection() {
    let mutations: [fn(&mut BirthOutput); 3] = [
        |o| o.value += 1,
        |o| o.asset = [9; 32],
        |o| o.program.push(9),
    ];
    for index in 0..4 {
        for output_index in 0..2 {
            for mutate in mutations {
                let mut offered = subjects();
                mutate(&mut offered[index].outputs[output_index]);
                refuses(offered, ContinuationManifestRefusal::PairInvariant);
            }
        }
    }
}

#[test]
fn version_negative_keeps_payload_and_displacement_is_exact_permutation() {
    let manifest = ContinuationCampaignManifest::new(packet(), subjects()).expect("manifest");
    let rows = manifest.subjects();
    assert_eq!(rows[2].outputs[0].program[0], 0);
    assert_eq!(
        &rows[2].outputs[0].program[1..],
        &rows[0].outputs[0].program[1..]
    );
    assert_eq!(
        rows[3].outputs,
        [rows[0].outputs[1].clone(), rows[0].outputs[0].clone()]
    );
    let mut offered = subjects();
    offered[3].outputs = offered[0].outputs.clone();
    refuses(offered, ContinuationManifestRefusal::PairInvariant);
}

#[test]
fn incomplete_recipe_and_parent_projections_refuse() {
    let mutations: [fn(&mut ContinuationPairContext); 8] = [
        |c| c.parent_bytes.clear(),
        |c| c.spent_outpoint.transaction_id = [0; 32],
        |c| c.funding_outpoint.transaction_id = [0; 32],
        |c| c.leaf.clear(),
        |c| c.alternative_leaf = c.leaf.clone(),
        |c| c.alternative_program = c.predecessor.program.clone(),
        |c| c.predecessor.program[0] = 0,
        |c| c.funding_output.value = 0,
    ];
    for mutate in mutations {
        let mut offered = subjects();
        mutate(&mut offered[0].context);
        refuses(offered, ContinuationManifestRefusal::IncompleteContext);
    }
}

#[test]
fn displaced_output_cannot_silently_change_the_version_boundary() {
    let mut offered = subjects();
    for subject in &mut offered {
        subject.context.control_outputs[1].program[0] = 0;
        let index = usize::from(subject.id != ContinuationSubjectId::DisplacedSuccessor);
        subject.outputs[index].program[0] = 0;
    }
    refuses(offered, ContinuationManifestRefusal::PairInvariant);
}
