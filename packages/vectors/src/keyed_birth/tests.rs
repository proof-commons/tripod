//! Synthetic preparation subjects; no native transaction is executed or admitted.

use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    TargetContractVersion, reviewed_elements_tapscript, validate_reviewed_development_binding,
};

use super::*;

fn packet() -> Wave1TargetPacket {
    let target = reviewed_elements_tapscript().expect("reviewed target");
    let deployment = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            TargetContractVersion::V2,
            DeploymentEnvironment::Development,
            [1; 32],
            [2; 32],
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("development declaration");
    Wave1TargetPacket::bind(
        target,
        deployment,
        Wave1ExecutableProvenance {
            node_name: "synthetic node".into(),
            node_version: "synthetic version".into(),
            binary_revision: "a".repeat(12),
            intended_tip: "a".repeat(40),
            framework_revision: "a".repeat(40),
            executable_path: "/fixture/elementsd".into(),
            executable_sha256: [3; 32],
            provenance_record: "/fixture/PROVENANCE.txt".into(),
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
            startup_arguments: vec!["-chain=elementsregtest".into()],
            relay: floor(PolicyFactOrigin::Declared),
            mempool: floor(PolicyFactOrigin::Declared),
            block: floor(PolicyFactOrigin::Declared),
        },
    )
    .expect("complete synthetic preparation")
}

fn floor(origin: PolicyFactOrigin) -> BirthFeeFloor {
    BirthFeeFloor {
        satoshis_per_kvb: 100,
        origin,
        source: "synthetic policy reference".into(),
    }
}

fn rebind(packet: Wave1TargetPacket) -> Result<Wave1TargetPacket, BirthPacketRefusal> {
    Wave1TargetPacket::bind(
        packet.target,
        packet.deployment,
        packet.provenance,
        packet.executor,
        packet.policy,
    )
}

fn refuses(subjects: Vec<BirthSubject>, expected: BirthPacketRefusal) {
    assert_eq!(
        BirthCampaignManifest::new(packet(), subjects),
        Err(expected)
    );
}

fn context(second: bool) -> BirthPairContext {
    let output = BirthOutput {
        program: vec![0x51, 0x20, 7],
        asset: [4; 32],
        value: 10_000,
    };
    BirthPairContext {
        parent_bytes: vec![if second { 2 } else { 1 }],
        parent_id: [if second { 6 } else { 5 }; 32],
        parent_outputs: [output.clone(), output.clone()],
        funding_outpoint: BirthOutpoint {
            transaction_id: [if second { 8 } else { 7 }; 32],
            output_index: 1,
        },
        funding_output: BirthOutput {
            program: vec![0, 0x14, 9],
            asset: [4; 32],
            value: 1_000,
        },
        spending_outputs: vec![output],
        fee: 1_000,
    }
}

fn subjects() -> Vec<BirthSubject> {
    BirthSubjectId::ALL
        .into_iter()
        .map(|id| {
            let (birth, current) = id.positions();
            let context = context(current == 1);
            let negative = id.control().is_some();
            BirthSubject {
                id,
                control: id.control(),
                current_input_index: current,
                spent_outpoint: BirthOutpoint {
                    transaction_id: context.parent_id,
                    output_index: birth,
                },
                context,
                changed_coordinate: negative.then_some(BirthCoordinate::SpentOutputIndex),
                collateral: if negative {
                    BirthCollateral::ALL.into_iter().collect()
                } else {
                    BTreeSet::new()
                },
                boundary: if negative {
                    BirthBoundary::BornFirstEqualVerify
                } else {
                    BirthBoundary::ScriptAcceptance
                },
            }
        })
        .collect()
}

#[test]
fn packet_binds_reviewed_rpc_transports_but_not_an_unreviewed_adapter() {
    for version in ["2.1.0", "2.1.1"] {
        let mut offered = packet();
        offered.executor.adapter_version = version.into();
        assert_eq!(rebind(offered.clone()), Ok(offered));
    }
    let mut offered = packet();
    offered.executor.adapter_version = "2.1.2".into();
    assert_eq!(rebind(offered), Err(BirthPacketRefusal::ExecutorBinding));
}

#[test]
fn complete_packet_retains_target_deployment_and_separate_policy() {
    let packet = packet();
    assert!(packet.deployment().welded_to(packet.target()));
    assert_eq!(packet.deployment().binding().network_id(), [1; 32]);
    assert_eq!(packet.deployment().binding().genesis_id(), [2; 32]);
    assert_eq!(packet.executor().protocol_schema, 8);
    assert_eq!(packet.provenance().executable_sha256, [3; 32]);
    assert_eq!(packet.policy().block.origin, PolicyFactOrigin::Declared);
    assert_eq!(rebind(packet.clone()), Ok(packet));
}

#[test]
fn inactive_development_declaration_does_not_bind_a_birth_experiment() {
    let mut offered = packet();
    offered.deployment = validate_reviewed_development_binding(
        offered.target(),
        DevelopmentDeploymentBinding::new(
            TargetContractVersion::V2,
            DeploymentEnvironment::Development,
            [1; 32],
            [2; 32],
            ActivationDeclaration::new(false, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("internally consistent inactive declaration");
    assert_eq!(rebind(offered), Err(BirthPacketRefusal::TargetBinding));
}

#[test]
fn incomplete_provenance_fields_refuse() {
    let mutations: [fn(&mut Wave1ExecutableProvenance); 8] = [
        |p| p.node_name.clear(),
        |p| p.node_version.clear(),
        |p| p.binary_revision.clear(),
        |p| p.intended_tip.clear(),
        |p| p.framework_revision.clear(),
        |p| p.executable_path.clear(),
        |p| p.executable_sha256 = [0; 32],
        |p| p.provenance_record.clear(),
    ];
    for mutate in mutations {
        let mut offered = packet();
        mutate(&mut offered.provenance);
        assert!(rebind(offered).is_err());
    }
}

#[test]
fn mismatched_binary_and_framework_revisions_refuse() {
    let mutations: [fn(&mut Wave1ExecutableProvenance); 3] = [
        |p| p.binary_revision = "b".repeat(12),
        |p| p.framework_revision = "b".repeat(40),
        |p| p.intended_tip = "A".repeat(40),
    ];
    for mutate in mutations {
        let mut offered = packet();
        mutate(&mut offered.provenance);
        assert_eq!(rebind(offered), Err(BirthPacketRefusal::ProvenanceBinding));
    }
}

#[test]
fn changed_chain_protocol_or_adapter_refuses() {
    let mutations: [fn(&mut Wave1ExecutorBinding); 4] = [
        |e| e.chain = "another-chain".into(),
        |e| e.protocol_schema += 1,
        |e| e.adapter_name = "another-adapter".into(),
        |e| e.adapter_version = "another-version".into(),
    ];
    for mutate in mutations {
        let mut offered = packet();
        mutate(&mut offered.executor);
        assert_eq!(rebind(offered), Err(BirthPacketRefusal::ExecutorBinding));
    }
}

#[test]
fn incomplete_policy_fields_refuse() {
    let mutations: [fn(&mut BirthPolicy); 7] = [
        |p| p.representation.clear(),
        |p| p.fee_policy.clear(),
        |p| p.startup_arguments.clear(),
        |p| p.startup_arguments.push(String::new()),
        |p| p.relay.source.clear(),
        |p| p.mempool.source.clear(),
        |p| p.block.source.clear(),
    ];
    for mutate in mutations {
        let mut offered = packet();
        mutate(&mut offered.policy);
        assert!(matches!(
            rebind(offered),
            Err(BirthPacketRefusal::Incomplete(_))
        ));
    }
}

#[test]
fn policy_origins_are_preserved_without_promoting_declarations() {
    let mut offered = packet();
    offered.policy.relay = floor(PolicyFactOrigin::Observed);
    offered.policy.relay.source = "synthetic retained node response".into();
    let bound = rebind(offered).expect("separate origins");
    assert_eq!(bound.policy().relay.origin, PolicyFactOrigin::Observed);
    assert_eq!(bound.policy().block.origin, PolicyFactOrigin::Declared);
    assert_ne!(bound.policy().relay.source, bound.policy().block.source);
}

#[test]
fn four_subject_census_has_two_one_coordinate_pairs() {
    let packet = packet();
    let manifest = BirthCampaignManifest::new(packet.clone(), subjects()).expect("manifest");
    assert_eq!(manifest.packet(), &packet);
    assert_eq!(manifest.subjects().len(), 4);
    assert_eq!(
        manifest
            .subjects()
            .iter()
            .map(|s| (s.spent_outpoint.output_index, s.current_input_index))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 0), (0, 1), (1, 1)]
    );
    for pair in manifest.subjects().as_chunks::<2>().0 {
        assert_eq!(pair[0].context, pair[1].context);
        assert_eq!(pair[1].control, Some(pair[0].id));
        assert_eq!(
            pair[1].changed_coordinate,
            Some(BirthCoordinate::SpentOutputIndex)
        );
        assert_eq!(
            pair[1].collateral,
            BirthCollateral::ALL.into_iter().collect()
        );
    }
    assert_eq!(
        BirthSubjectId::SUBMISSION_ORDER,
        [
            BirthSubjectId::A1,
            BirthSubjectId::A0,
            BirthSubjectId::A3,
            BirthSubjectId::A2,
        ]
    );
}

#[test]
fn missing_duplicate_and_reordered_subjects_refuse() {
    let mut missing = subjects();
    missing.pop();
    let mut duplicate = subjects();
    duplicate[1] = duplicate[0].clone();
    let mut reordered = subjects();
    reordered.swap(0, 1);
    for offered in [missing, duplicate, reordered] {
        refuses(offered, BirthPacketRefusal::SubjectCensus);
    }
}

#[test]
fn every_fixed_pair_coordinate_is_checked() {
    let mutations: [fn(&mut BirthPairContext); 9] = [
        |c| c.parent_bytes.push(9),
        |c| c.funding_outpoint.output_index += 1,
        |c| c.funding_outpoint.transaction_id = [9; 32],
        |c| c.funding_output.value += 1,
        |c| c.funding_output.asset = [9; 32],
        |c| c.funding_output.program.push(9),
        |c| c.fee += 1,
        |c| c.spending_outputs.push(c.funding_output.clone()),
        |c| {
            for output in &mut c.parent_outputs {
                output.program.push(9);
            }
            c.spending_outputs[0].program.push(9);
        },
    ];
    for mutate in mutations {
        let mut offered = subjects();
        mutate(&mut offered[1].context);
        refuses(offered, BirthPacketRefusal::PairInvariant);
    }
}

#[test]
fn unequal_parent_outputs_and_changed_successor_refuse() {
    let mutations: [fn(&mut BirthPairContext); 5] = [
        |c| c.parent_outputs[1].program.push(9),
        |c| c.parent_outputs[1].asset = [9; 32],
        |c| c.parent_outputs[1].value += 1,
        |c| c.spending_outputs[0].program.push(9),
        |c| c.spending_outputs.clear(),
    ];
    for mutate in mutations {
        let mut offered = subjects();
        mutate(&mut offered[0].context);
        refuses(offered, BirthPacketRefusal::PairInvariant);
    }
}

#[test]
fn wrong_control_placement_outpoint_or_boundary_refuses() {
    let mutations: [fn(&mut BirthSubject); 6] = [
        |s| s.control = Some(BirthSubjectId::A2),
        |s| s.current_input_index = 1,
        |s| s.spent_outpoint.output_index = 0,
        |s| s.spent_outpoint.transaction_id = [9; 32],
        |s| s.changed_coordinate = None,
        |s| s.boundary = BirthBoundary::ScriptAcceptance,
    ];
    for mutate in mutations {
        let mut offered = subjects();
        mutate(&mut offered[1]);
        refuses(offered, BirthPacketRefusal::SubjectCoordinate);
    }
}

#[test]
fn second_placement_requires_nonzero_funding_birth_position() {
    let mut offered = subjects();
    offered[2].context.funding_outpoint.output_index = 0;
    offered[3].context.funding_outpoint.output_index = 0;
    refuses(offered, BirthPacketRefusal::SubjectCoordinate);
}

#[test]
fn missing_negative_collateral_and_spurious_control_collateral_refuse() {
    for collateral in BirthCollateral::ALL {
        let mut offered = subjects();
        offered[1].collateral.remove(&collateral);
        refuses(offered, BirthPacketRefusal::Collateral);
        let mut offered = subjects();
        offered[0].collateral.insert(collateral);
        refuses(offered, BirthPacketRefusal::Collateral);
    }
}

#[test]
fn reused_parent_or_sponsor_between_pairs_refuses() {
    let mut parent_reuse = subjects();
    for subject in &mut parent_reuse[2..] {
        subject.context.parent_id = [5; 32];
        subject.spent_outpoint.transaction_id = [5; 32];
    }
    let mut sponsor_reuse = subjects();
    for subject in &mut sponsor_reuse[2..] {
        subject.context.funding_outpoint.transaction_id = [7; 32];
    }
    for offered in [parent_reuse, sponsor_reuse] {
        refuses(offered, BirthPacketRefusal::ReusedPairFunding);
    }
}

#[test]
fn missing_parent_and_output_data_are_not_complete_subjects() {
    let mutations: [fn(&mut BirthPairContext); 3] = [
        |c| c.parent_bytes.clear(),
        |c| c.funding_outpoint.transaction_id = [0; 32],
        |c| c.funding_output.program.clear(),
    ];
    for mutate in mutations {
        let mut offered = subjects();
        mutate(&mut offered[0].context);
        assert!(matches!(
            BirthCampaignManifest::new(packet(), offered),
            Err(BirthPacketRefusal::Incomplete(_))
        ));
    }
}
