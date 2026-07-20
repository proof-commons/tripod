use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    ARCHITECTURE, AssetId, BoundId, DeltaKind, ObjectId, OperationId, ProjectionId, RootId, RootUse,
};

use crate::{
    Count, ObservedAsset, ObservedCanonicalDelta, ObservedObject, ObservedObjectKind,
    ObservedObjectRef, ObservedOpenFlow, ObservedRootEffect, ObservedSide, OperationObservation,
    OwnerId, ProtocolAmount, RealizationScope, RelationId, RelationKind, RelationStatus,
    RelationSubject, RepresentationMode, TransactionSide, derive, evaluate_operation,
};

type ObservationMutation = Box<dyn Fn(&mut OperationObservation)>;
type RelationCase = (&'static str, ObservationMutation, RelationId);

const ALICE: OwnerId = OwnerId([1_u8; 32]);
const BOB: OwnerId = OwnerId([2_u8; 32]);
const CAROL: OwnerId = OwnerId([3_u8; 32]);
const DAVE: OwnerId = OwnerId([4_u8; 32]);

fn pilot_realization() -> crate::ScopedRealizationSpec {
    derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap()
}

fn receipt(side: ObservedSide, ordinal: u32, value: u64, owner: OwnerId) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::ReceiptLive),
        asset: ObservedAsset::Declared(AssetId::U),
        value: ProtocolAmount::new(value).unwrap(),
        owner: Some(owner),
        representation: RepresentationMode::Explicit,
    }
}

fn lbtc_owned(side: ObservedSide, ordinal: u32, value: u64, owner: OwnerId) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
        asset: ObservedAsset::Declared(AssetId::Lbtc),
        value: ProtocolAmount::new(value).unwrap(),
        owner: Some(owner),
        representation: RepresentationMode::Explicit,
    }
}

fn lbtc(side: ObservedSide, ordinal: u32, value: u64) -> ObservedObject {
    lbtc_owned(side, ordinal, value, CAROL)
}

fn valid_split_observation() -> OperationObservation {
    let input0 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 0,
    };
    let output0 = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 0,
    };
    let output1 = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 1,
    };

    OperationObservation {
        operation: OperationId::TransferLive,
        objects: vec![
            receipt(ObservedSide::Input, 0, 100, ALICE),
            receipt(ObservedSide::Output, 0, 40, BOB),
            receipt(ObservedSide::Output, 1, 60, CAROL),
        ],
        protocol_signers: BTreeSet::from([ALICE]),
        sponsor_signers: BTreeSet::new(),
        canonical_deltas: vec![ObservedCanonicalDelta {
            asset: AssetId::U,
            kind: DeltaKind::Lateral,
            amount: ProtocolAmount::new(100).unwrap(),
            sources: vec![input0],
            destinations: vec![output0, output1],
            destruction_tag: None,
        }],
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::TransferInputMax, Count::new(64)),
            (BoundId::TransferOutputMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    }
}

fn valid_sponsored_observation() -> OperationObservation {
    let mut observation = valid_split_observation();
    let sponsor_input = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 1,
    };
    let sponsor_change = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 2,
    };

    observation
        .objects
        .push(lbtc_owned(ObservedSide::Input, 1, 10, CAROL));
    observation
        .objects
        .push(lbtc_owned(ObservedSide::Output, 2, 7, CAROL));
    observation.open_flows.push(ObservedOpenFlow {
        kind: architecture::OpenFlowKind::FeeSponsor,
        sources: vec![sponsor_input],
        destinations: vec![sponsor_change],
        fee: ProtocolAmount::new(3).unwrap(),
    });
    observation.sponsor_signers.insert(CAROL);

    observation
}

fn multi_owner_sponsor_observation() -> OperationObservation {
    let mut observation = valid_split_observation();
    let carol_input = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 1,
    };
    let dave_input = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 2,
    };
    let sponsor_change = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 2,
    };

    observation.objects.extend([
        lbtc_owned(ObservedSide::Input, 1, 10, CAROL),
        lbtc_owned(ObservedSide::Input, 2, 7, DAVE),
        lbtc_owned(ObservedSide::Output, 2, 12, CAROL),
    ]);
    observation.open_flows.push(ObservedOpenFlow {
        kind: architecture::OpenFlowKind::FeeSponsor,
        sources: vec![carol_input, dave_input],
        destinations: vec![sponsor_change],
        fee: ProtocolAmount::new(5).unwrap(),
    });

    observation
}

fn evaluate(observation: &OperationObservation) -> crate::ConformanceReport {
    let realization = pilot_realization();

    evaluate_operation(
        &realization.relation_graph,
        &realization.relation_node_by_id,
        &realization.relation_evaluation_order,
        observation,
    )
    .unwrap()
}

fn relation_id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::TransferLive, kind, subject)
}

fn input_cardinality() -> RelationId {
    relation_id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    )
}

fn output_cardinality() -> RelationId {
    relation_id(
        RelationKind::Cardinality,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::ReceiptLive,
        },
    )
}

fn input_recognition() -> RelationId {
    relation_id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    )
}

fn output_recognition() -> RelationId {
    relation_id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::ReceiptLive,
        },
    )
}

fn authorization() -> RelationId {
    relation_id(
        RelationKind::Authorization,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    )
}

fn input_closure() -> RelationId {
    relation_id(
        RelationKind::OutputClosure,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    )
}

fn output_closure() -> RelationId {
    relation_id(
        RelationKind::OutputClosure,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Output,
            object: ObjectId::ReceiptLive,
        },
    )
}

fn conservation() -> RelationId {
    relation_id(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    )
}

fn canonical_delta_policy() -> RelationId {
    relation_id(
        RelationKind::Conservation,
        RelationSubject::Projection {
            projection: ProjectionId::TransitionCertificate,
        },
    )
}

fn sponsor() -> RelationId {
    relation_id(RelationKind::SponsorIsolation, RelationSubject::Sponsor)
}

fn roots() -> RelationId {
    relation_id(RelationKind::RootPolicy, RelationSubject::Operation)
}

fn projections() -> RelationId {
    relation_id(RelationKind::ProjectionPolicy, RelationSubject::Operation)
}

fn representation() -> RelationId {
    relation_id(
        RelationKind::Representation,
        RelationSubject::Representation {
            object: ObjectId::ReceiptLive,
        },
    )
}

fn failed(report: &crate::ConformanceReport, relation: &RelationId) -> bool {
    report.verdicts.iter().any(|verdict| {
        verdict.relation == *relation
            && matches!(
                verdict.status,
                RelationStatus::Failed { .. } | RelationStatus::Blocked { .. }
            )
    })
}

#[test]
fn relation_declaration_order_does_not_change_relation_graph_projection() {
    let realization = pilot_realization();
    let operation = realization
        .operations
        .get(&OperationId::TransferLive)
        .unwrap();
    let declarations = operation.relations.clone();
    let dependencies = operation.relation_dependencies.clone();
    let mut reversed = declarations.clone();
    reversed.reverse();
    let first = crate::relation::build_relation_graph(declarations, dependencies.clone()).unwrap();
    let second = crate::relation::build_relation_graph(reversed, dependencies).unwrap();

    assert_eq!(
        crate::relation::project_relation_graph(&first.0),
        crate::relation::project_relation_graph(&second.0),
    );
}

#[test]
fn valid_live_transfer_satisfies_every_runtime_relation() {
    let report = evaluate(&valid_split_observation());

    assert!(
        report.is_conformant(),
        "failed relations: {:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

#[test]
fn canonical_delta_amount_mutation_fails() {
    let mut observation = valid_split_observation();

    observation.canonical_deltas[0].amount = ProtocolAmount::new(99).unwrap();

    let report = evaluate(&observation);

    assert!(
        failed(&report, &canonical_delta_policy()),
        "a canonical delta whose amount disagrees with its referenced objects must fail"
    );
}

#[test]
fn canonical_delta_empty_duplicate_fails() {
    let mut missing = valid_split_observation();
    missing.canonical_deltas.clear();

    let missing_report = evaluate(&missing);

    assert!(
        failed(&missing_report, &canonical_delta_policy()),
        "live transfer without its required lateral U delta must fail"
    );

    let realization = pilot_realization();
    let mut duplicated = valid_split_observation();
    duplicated
        .canonical_deltas
        .push(duplicated.canonical_deltas[0].clone());

    let error = evaluate_operation(
        &realization.relation_graph,
        &realization.relation_node_by_id,
        &realization.relation_evaluation_order,
        &duplicated,
    )
    .unwrap_err();

    assert_eq!(
        error,
        crate::RealizationError::ObservedCanonicalPartitionOverlap
    );
}

#[test]
fn live_receipt_input_without_owner_fails() {
    let mut observation = valid_split_observation();

    observation
        .objects
        .iter_mut()
        .find(|object| {
            object.reference.side == ObservedSide::Input
                && object.kind == ObservedObjectKind::Declared(ObjectId::ReceiptLive)
        })
        .expect("fixture has a live-receipt input")
        .owner = None;
    observation.protocol_signers.clear();

    let report = evaluate(&observation);

    assert!(
        failed(&report, &input_recognition()),
        "owner presence is part of live-receipt input recognition"
    );
}

#[test]
fn live_receipt_output_without_owner_fails() {
    let mut observation = valid_split_observation();

    observation
        .objects
        .iter_mut()
        .find(|object| {
            object.reference.side == ObservedSide::Output
                && object.kind == ObservedObjectKind::Declared(ObjectId::ReceiptLive)
        })
        .expect("fixture has a live-receipt output")
        .owner = None;

    let report = evaluate(&observation);

    assert!(
        failed(&report, &output_recognition()),
        "owner presence is part of live-receipt output recognition"
    );
}

#[test]
fn valid_sponsored_live_transfer_conforms() {
    let report = evaluate(&valid_sponsored_observation());

    assert!(
        report.is_conformant(),
        "failed relations: {:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

#[test]
fn missing_sponsor_signer_fails() {
    let mut observation = valid_sponsored_observation();

    observation.sponsor_signers.clear();

    let report = evaluate(&observation);

    assert!(
        failed(&report, &sponsor()),
        "balanced sponsor value without sponsor authorization must fail"
    );
}

#[test]
fn one_missing_sponsor_owner_among_many_fails() {
    let mut observation = multi_owner_sponsor_observation();

    observation.sponsor_signers.insert(CAROL);

    let report = evaluate(&observation);

    assert!(
        failed(&report, &sponsor()),
        "every consumed sponsor owner must authorize the sponsor flow"
    );
}

#[test]
fn every_sponsor_owner_authorizes_multi_owner_flow() {
    let mut observation = multi_owner_sponsor_observation();

    observation.sponsor_signers.extend([CAROL, DAVE]);

    let report = evaluate(&observation);

    assert!(
        report.is_conformant(),
        "failed relations: {:?}",
        report.failed_relations().collect::<Vec<_>>()
    );
}

#[test]
fn live_transfer_negative_observations_name_the_load_bearing_relation() {
    for (name, mutate, relation) in relation_cases() {
        let mut observation = valid_split_observation();
        mutate(&mut observation);
        let report = evaluate(&observation);

        assert!(
            failed(&report, &relation),
            "case {name} did not fail {relation:?}"
        );
    }
}

fn relation_cases() -> Vec<RelationCase> {
    vec![
        (
            "missing signer",
            Box::new(|observation| observation.protocol_signers.clear()),
            authorization(),
        ),
        (
            "time locked input",
            Box::new(|observation| {
                observation.objects[0].kind =
                    ObservedObjectKind::Declared(ObjectId::ReceiptTimeLocked);
            }),
            input_closure(),
        ),
        (
            "ash output",
            Box::new(|observation| {
                observation.objects[1].kind = ObservedObjectKind::Declared(ObjectId::Ash);
            }),
            output_closure(),
        ),
        (
            "amount mismatch",
            Box::new(|observation| observation.objects[1].value = ProtocolAmount::new(41).unwrap()),
            conservation(),
        ),
        (
            "wrong canonical delta kind",
            Box::new(|observation| {
                observation.canonical_deltas[0].kind = DeltaKind::OwnerlessLateral;
            }),
            canonical_delta_policy(),
        ),
        (
            "extra output beyond bound",
            Box::new(|observation| {
                observation
                    .bounds
                    .insert(BoundId::TransferOutputMax, Count::ONE);
            }),
            output_cardinality(),
        ),
        (
            "input bound exceeded",
            Box::new(|observation| {
                observation
                    .bounds
                    .insert(BoundId::TransferInputMax, Count::ZERO);
            }),
            input_cardinality(),
        ),
        (
            "burn projection",
            Box::new(|observation| {
                observation.projections.insert(ProjectionId::BurnEvent);
            }),
            projections(),
        ),
        (
            "root effect",
            Box::new(|observation| {
                observation.root_effects.push(ObservedRootEffect {
                    root: RootId::State,
                    use_kind: RootUse::Succession,
                });
            }),
            roots(),
        ),
        (
            "public committed representation",
            Box::new(|observation| {
                observation.objects[1].representation = RepresentationMode::PublicCommitted;
            }),
            representation(),
        ),
        (
            "foreign output object",
            Box::new(|observation| observation.objects[1].kind = ObservedObjectKind::Unrecognized),
            output_closure(),
        ),
    ]
}

#[test]
fn live_transfer_sponsor_isolation_is_load_bearing() {
    let source = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 1,
    };
    let destination = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 2,
    };
    let cases: Vec<(&str, ObservationMutation)> = vec![
        (
            "unbalanced sponsor flow",
            Box::new(move |observation| {
                observation.objects.push(lbtc(ObservedSide::Input, 1, 10));
                observation.objects.push(lbtc(ObservedSide::Output, 2, 6));
                observation.open_flows.push(ObservedOpenFlow {
                    kind: architecture::OpenFlowKind::FeeSponsor,
                    sources: vec![source],
                    destinations: vec![destination],
                    fee: ProtocolAmount::new(3).unwrap(),
                });
            }),
        ),
        (
            "sponsor U mixed into open flow",
            Box::new(|observation| {
                observation.open_flows.push(ObservedOpenFlow {
                    kind: architecture::OpenFlowKind::FeeSponsor,
                    sources: vec![ObservedObjectRef {
                        side: ObservedSide::Input,
                        ordinal: 0,
                    }],
                    destinations: Vec::new(),
                    fee: ProtocolAmount::ZERO,
                });
            }),
        ),
    ];

    for (name, mutate) in cases {
        let mut observation = valid_split_observation();
        mutate(&mut observation);
        let report = evaluate(&observation);

        assert!(
            failed(&report, &sponsor()),
            "case {name} did not fail sponsor isolation"
        );
    }
}
